#!/usr/bin/env node
// The ddphotos edit page, loaded in a real headless Chrome.
//
// The parity harness (tests/ddphotos_parity.mjs) runs the page's recipes in Node. What
// it cannot see is the page itself: that it loads, how many renders the worker is asked
// for, that a burst of form changes is one render, and what the CLI panel says. This
// builds a small fake ddphotos site in a temp folder (the page, the slim wasm package
// as its vendor/ folder, two photos), serves it, and drives Chrome over the DevTools
// Protocol (scripts/lib/chrome.mjs, no browser driver).
//
// The render counter is a script injected before the page's own, wrapping
// Worker.prototype.postMessage, so the page carries no test hook.
//
// Run:  node tests/ddphotos_page.mjs --pkg <slim pkg dir> [--keep]
// `just ddphotos-page` builds the package and runs it; CI runs it after the parity step.
//
// There is no skip path: no Chrome, a CHROME that does not exist, a missing package, a
// failed wait or any failed row exits 1.

import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { findChrome, launchChrome } from "../scripts/lib/chrome.mjs";
import { startServer } from "../scripts/serve.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..");
const EDIT_DIR = join(ROOT, "integrations/ddphotos/edit");
const FIXTURES = join(HERE, "fixtures/ddphotos");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ── the rows' literals ───────────────────────────────────────────────────────

const LOADED = '{"load":1,"preview":1,"make":0}';
const AFTER_BURST = '{"load":1,"preview":2,"make":0}';

// The sentence the CLI panel shows, with the web copy's upright size in the gap.
const cliNote = (size) =>
  `Run it on the original, or on a file at least as large as the web copy this page edits (${size}). ` +
  "On a smaller file the CLI makes a smaller image than this page did, and may refuse the watermark, without warning.";

// The CLI recipe for photo W at 800 px, JPEG, no watermark (what H3 leaves the form at).
const RECIPE_W_800 =
  'version = "2"\nformat = "jpeg"\n\n[[step]]\nop = "auto-orient"\n\n' +
  '[[step]]\nop = "resize"\nmode = "max"\nwidth = 800\n\n[[step]]\nop = "optimize"\n';

// ── arguments ────────────────────────────────────────────────────────────────

let rows = 0;
let failed = 0;

function finish() {
  console.log(`ddphotos-page: ${rows} rows, ${failed} failed`);
  return failed === 0 ? 0 : 1;
}

/// A failure that stops the run: the page never got to where the next row can read it.
class Stop extends Error {}

function parseArgs(argv) {
  const args = { keep: false };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--pkg") args.pkg = argv[++i];
    else if (argv[i] === "--keep") args.keep = true;
    else throw new Stop(`unknown argument ${argv[i]}; usage: node tests/ddphotos_page.mjs --pkg <slim pkg dir> [--keep]`);
  }
  if (!args.pkg) throw new Stop("--pkg <slim pkg dir> is required (there is no default that could point at a stale build)");
  for (const f of ["crustyimg.js", "crustyimg_bg.wasm"]) {
    if (!existsSync(join(resolve(args.pkg), f))) {
      throw new Stop(`wasm package missing or incomplete: expected ${join(resolve(args.pkg), f)}`);
    }
  }
  return args;
}

// ── the fake site ────────────────────────────────────────────────────────────

// What the page reads of a ddphotos site, and no more: config.json, albums.json, one
// album's index.json (fileName, src.full, src.grid) and the photos. The photos are the
// parity fixtures: W (a 1600 × 1071 WebP) and R (a JPEG stored 1600 × 1071 with an
// EXIF rotation, upright 1071 × 1600).
function buildSite(pkg) {
  const site = mkdtempSync(join(tmpdir(), "ddphotos-page-site-"));
  const json = (path, value) => {
    mkdirSync(dirname(join(site, path)), { recursive: true });
    writeFileSync(join(site, path), JSON.stringify(value));
  };
  json("albums/config.json", { siteName: "Test site", copyrightOwner: "Jane Doe", encrypted: false });
  json("albums/albums.json", [{ slug: "test", title: "Test album", count: 2, dateSpan: "2026" }]);
  const photos = ["photo.webp", "photo_rot.jpg"];
  json("albums/test/index.json", {
    slug: "test",
    title: "Test album",
    photos: photos.map((fileName) => ({ fileName, src: { full: `full/${fileName}`, grid: `full/${fileName}` } })),
  });
  mkdirSync(join(site, "albums/test/full"), { recursive: true });
  for (const f of photos) cpSync(join(FIXTURES, f), join(site, "albums/test/full", f));
  // The page as ddphotos-edit-install lays it out: its files, and vendor/ beside them.
  const edit = join(site, "edit");
  mkdirSync(join(edit, "vendor"), { recursive: true });
  for (const f of readdirSync(EDIT_DIR)) cpSync(join(EDIT_DIR, f), join(edit, f));
  for (const f of ["crustyimg.js", "crustyimg_bg.wasm"]) cpSync(join(pkg, f), join(edit, "vendor", f));
  return site;
}

// ── driving the page ─────────────────────────────────────────────────────────

// Runs before any page script, in every new document.
const COUNTER = `
  window.__renders = { load: 0, preview: 0, make: 0 };
  const post = Worker.prototype.postMessage;
  Worker.prototype.postMessage = function (msg, ...rest) {
    const type = msg && msg.type;
    if (Object.hasOwn(window.__renders, type)) window.__renders[type]++;
    return post.call(this, msg, ...rest);
  };
`;

const exceptions = [];
let cdp;
let baseUrl;

const q = (s) => JSON.stringify(s);
const byId = (id) => `document.getElementById(${q(id)})`;

/// The page's `#error` text while it is showing a failure, else null.
const ERROR_NOW = `(() => { const e = ${byId("error")}; return e && !e.hidden && e.className === "error" ? e.textContent : null; })()`;

/// Poll `expression` every 100 ms until it is truthy. Fails at once, quoting the page's
/// own error box and every page exception so far, when `#error` shows a failure, and
/// after `timeoutMs` otherwise.
async function waitFor(expression, what, timeoutMs = 60_000) {
  const deadline = Date.now() + timeoutMs;
  const stop = (lead, error) =>
    new Stop(
      `${lead} waiting for ${what}; the page's error box: ${error ?? "none"}; ` +
        `page exceptions: ${exceptions.length ? exceptions.join(" | ") : "none"}`,
    );
  while (Date.now() < deadline) {
    const state = await cdp.eval(`({ done: Boolean(${expression}), error: ${ERROR_NOW} })`);
    if (state.done) return;
    if (state.error !== null) throw stop("the page failed", state.error);
    await sleep(100);
  }
  throw stop("timed out", await cdp.eval(ERROR_NOW));
}

async function open(path) {
  await cdp.send("Page.navigate", { url: `${baseUrl}${path}` });
}

/// Wait until the first preview is on screen: the engine's JPEG, as a blob URL.
async function waitForFirstPreview(which) {
  await waitFor(
    `${byId("preview")}.src.startsWith("blob:") && ${byId("busy")}.hidden && !${byId("make")}.disabled`,
    `the first preview on ${which}`,
  );
}

async function makeCopy(which) {
  await cdp.eval(`${byId("make")}.click()`);
  await waitFor(`!${byId("result")}.hidden`, `the copy on ${which}`, 120_000);
}

const read = (expression) => cdp.eval(expression);
const renders = () => read("JSON.stringify(window.__renders)");
const collapse = `.replace(/\\s+/g, " ").trim()`;
const readNote = () => read(`(${byId("cli-note")}?.textContent ?? "<no #cli-note>")${collapse}`);

/// One row: its name, and an error text, or null when it passed.
function row(name, problem) {
  rows++;
  if (problem === null) {
    console.log(`PASS ${name}`);
    return;
  }
  failed++;
  console.log(`FAIL ${name}: ${problem}`);
}

const expectEqual = (got, want, label = "") => (got === want ? null : `${label}${got}, expected ${want}`);

// ── the run ──────────────────────────────────────────────────────────────────

async function run(args, chromePath) {
  const pkg = resolve(args.pkg);
  const site = buildSite(pkg);
  const profile = mkdtempSync(join(tmpdir(), "ddphotos-page-chrome-"));
  let server;
  let chrome;
  let exited;
  try {
    ({ server, url: baseUrl } = await startServer({ root: site }));
    try {
      ({ chrome, cdp, exited } = await launchChrome(chromePath, profile));
    } catch (e) {
      throw new Stop(e.message);
    }

    cdp.on("Runtime.exceptionThrown", (p) => {
      exceptions.push(p.exceptionDetails.exception?.description ?? p.exceptionDetails.text);
    });
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: 1200,
      height: 900,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: COUNTER });

    // W: a landscape WebP.
    await open("/edit/index.html?album=test&photo=1");
    await waitForFirstPreview("W");

    // H1 the page loads.
    const state = await read(
      `({ error: ${byId("error")}.hidden ? "hidden" : ${byId("error")}.textContent, ` +
        `editor: ${byId("editor")}.hidden ? "hidden" : "shown", engine: ${byId("engine")}.textContent })`,
    );
    const loaded = state.error === "hidden" && state.editor === "shown" && state.engine === "Engine ready";
    row("H1 page loads", loaded ? null : `error ${q(state.error)}, editor ${state.editor}, engine ${q(state.engine)}`);

    // H2 the worker is asked for one load and one preview.
    row("H2 renders at load", expectEqual(await renders(), LOADED, "renders "));

    // H3 three size changes in one tick are one more preview, once the debounce passes.
    const before = await read(`${byId("preview")}.src`);
    await read(
      `(() => { for (const v of ["1200", "1080", "800"]) { ${byId("size")}.value = v; ` +
        `${byId("size")}.dispatchEvent(new Event("input", { bubbles: true })); } })()`,
    );
    await waitFor(
      `${byId("preview")}.src !== ${q(before)} && ${byId("preview")}.src.startsWith("blob:") && ${byId("busy")}.hidden`,
      "the preview after three size changes",
    );
    await sleep(600);
    row("H3 three quick size changes add one preview", expectEqual(await renders(), AFTER_BURST, "renders "));

    // H4 the CLI panel's sentence, with W's size. (Make runs the form as H3 left it.)
    await makeCopy("W");
    row("H4 the CLI sentence on W", expectEqual(`"${await readNote()}"`, `"${cliNote("1600 × 1071")}"`));

    // H5 the panel's recipe on W: 800 px, JPEG, no crop, no watermark.
    const recipe = await read(`${byId("recipe")}.textContent`);
    row("H5 the panel's recipe on W", expectEqual(JSON.stringify(recipe), JSON.stringify(RECIPE_W_800)));

    // R: a JPEG stored sideways, upright 1071 × 1600. A new document, so a new counter.
    await open("/edit/index.html?album=test&photo=2");
    await waitForFirstPreview("R");
    row("H6 renders at load on R", expectEqual(await renders(), LOADED, "renders "));

    // H7 the sentence names the upright size, not the stored one.
    await makeCopy("R");
    row("H7 the CLI sentence on R", expectEqual(`"${await readNote()}"`, `"${cliNote("1071 × 1600")}"`));

    // H8 nothing threw on the page.
    row("H8 no page errors", exceptions.length === 0 ? null : exceptions.join(" | "));
  } finally {
    // Close the socket, stop Chrome and wait for it to be gone before its profile goes:
    // deleting a profile Chrome is still writing into fails with ENOTEMPTY.
    try {
      cdp?.close();
    } catch {
      /* already closed */
    }
    if (chrome) {
      chrome.kill();
      await exited;
    }
    if (server) {
      server.closeAllConnections?.();
      await new Promise((r) => server.close(r));
    }
    // Chrome's helper processes can still be flushing into the profile just after the
    // browser itself has exited, so a delete that fails is retried for about 15 s.
    let deleteError = null;
    for (let attempt = 0; attempt < 60; attempt++) {
      try {
        rmSync(profile, { recursive: true, force: true });
        deleteError = null;
        break;
      } catch (e) {
        deleteError = e;
        await sleep(250);
      }
    }
    if (deleteError) console.log(`warning: could not delete ${profile}: ${deleteError.message}`);
    if (args.keep) console.log(`fake site kept in ${site}`);
    else rmSync(site, { recursive: true, force: true });
  }
}

let code;
try {
  const args = parseArgs(process.argv.slice(2));
  const chromePath = findChrome();
  if (!chromePath) {
    const at = process.env.CHROME ? ` at CHROME=${process.env.CHROME}` : "";
    throw new Stop(`no Chrome or Chromium found${at}; install one or set CHROME=/path/to/chrome`);
  }
  await run(args, chromePath);
} catch (e) {
  failed++;
  console.log(`FAIL ${e instanceof Stop ? e.message : (e.stack ?? e)}`);
} finally {
  code = finish();
}
process.exit(code);
