#!/usr/bin/env node
// SPEC-077 / SPEC-078 / SPEC-080: the demo's earned verdict — the page, in a real
// browser, over HTTP.
//
// Everything in this wave up to now ran the wasm in NODE (`initSync` with bytes we
// handed it). The browser path is different in the one way that can break it:
// `init()` fetches `crustyimg_bg.wasm` and calls `WebAssembly.instantiateStreaming`,
// which needs the server to send `application/wasm`. A page that renders perfectly
// and cannot instantiate the engine is THE failure mode here, and no amount of Node
// testing would see it. So this serves the assembled demo and drives it in headless
// Chrome, through the real user path:
//
//   serve → load → init() → put a photo in the file picker → the `web` flow runs →
//   read the DOM → fetch the download's blob → decode the bytes with a parser we
//   wrote ourselves.
//
// SPEC-080 reframed the demo around the `web` flow (one-click hero: downscale to
// 2048 → modernize to AVIF → never bigger → score) plus a CLI adoption funnel. The
// four checks that pin that reframe are labelled below:
//
//   • default_is_web_flow_smaller_avif  — a photo on defaults → smaller AVIF, ≤2048
//   • never_bigger_keeps_original       — an input it can't beat → the original back
//   • funnel_shows_web_command_and_copies — the exact `crustyimg web <file>` + copy
//   • advanced_full_resolution_shows_timer — the slow path warns + counts + stays live
//
// It also keeps the SPEC-077/078 guarantees: the .wasm arrives as `application/wasm`,
// the engine runs in a module Worker, NOTHING is fetched during a conversion, the
// main thread survives a slow AVIF encode (with a negative control), the AVIF output
// is valid per decoders the crate never met, an `.avif` input rides the browser's
// decoder, and the file:// failure mode is real.
//
// Chrome is driven over the DevTools Protocol directly (a WebSocket + JSON-RPC). No
// Puppeteer/Playwright: a demo whose whole point is "no toolchain, no install" should
// not need a 300 MB browser driver to prove it works. Set CHROME to point at it if it
// is not where we look.
//
// Run: `just demo-smoke`.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { makePng, makePhotoPng, readIhdr } from "../scripts/lib/png.mjs";
import { startServer } from "../scripts/serve.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const demoDir = join(repoRoot, "demo");

// The hero fixture: a real PHOTOGRAPH, larger than the 2048 web bound, so the default
// flow both DOWNSCALES it and picks AVIF. A gradient would not — the engine reads a
// gradient as a flat graphic and routes it lossless, so a "photo → AVIF" fixture built
// from `makePng` would silently test the wrong path (the STAGE-030 lesson). `makePhotoPng`
// carries the entropy + colour + low flat-ratio that buckets Lossy → AVIF.
const HERO_W = 2200;
const HERO_H = 1650;
const WEB_EDGE = 2048; // the `web` long-edge target the demo downscales to

// The full-resolution fixture for the Advanced slow path: > ~6 MP, so a full-res AVIF
// encode is genuinely slow — long enough to watch the timer count and the main thread
// stay alive.
const FULL_W = 3000;
const FULL_H = 2100;

// The AVIF-encode responsiveness window (SPEC-078): a real blocking window, so a
// main-thread encode would have frozen the tab for it.
const MIN_ENCODE_MS = 200;
const BLOCK_MS = 400; // the negative control: how long we freeze the main thread on purpose

let failed = 0;
const cleanups = [];

function ok(msg) {
  console.log(`  ✓ ${msg}`);
}

function check(cond, msg) {
  if (cond) ok(msg);
  else {
    failed++;
    console.error(`  ✗ ${msg}`);
  }
}

async function cleanup() {
  for (const fn of cleanups.reverse()) {
    try {
      await fn();
    } catch {
      /* teardown is best-effort */
    }
  }
}

async function die(msg) {
  console.error(`\ndemo-smoke: ${msg}\n`);
  await cleanup();
  process.exit(1);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ── find Chrome ───────────────────────────────────────────────────────────────

function findChrome() {
  if (process.env.CHROME) return process.env.CHROME;
  const candidates = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
  ];
  return candidates.find((p) => existsSync(p));
}

// ── a minimal DevTools Protocol client ────────────────────────────────────────

class CDP {
  #ws;
  #next = 1;
  #pending = new Map();
  handlers = new Map();

  static async attach(wsUrl) {
    const cdp = new CDP();
    cdp.#ws = new WebSocket(wsUrl);
    cdp.#ws.addEventListener("message", (ev) => cdp.#onMessage(JSON.parse(ev.data)));
    await new Promise((res, rej) => {
      cdp.#ws.addEventListener("open", res, { once: true });
      cdp.#ws.addEventListener("error", rej, { once: true });
    });
    return cdp;
  }

  #onMessage(msg) {
    if (msg.id !== undefined) {
      const p = this.#pending.get(msg.id);
      this.#pending.delete(msg.id);
      if (msg.error) p?.rej(new Error(`${msg.error.message} (${JSON.stringify(msg.error.data)})`));
      else p?.res(msg.result);
      return;
    }
    // Events from an attached target carry its sessionId; events from the page do
    // not. The handlers below are keyed by METHOD, so a worker's console error and
    // a page's console error land in the same list — which is what we want.
    this.handlers.get(msg.method)?.(msg.params, msg.sessionId);
  }

  /// `sessionId` addresses an auto-attached target — the engine's Web Worker is one
  /// (SPEC-078), and it is a genuinely separate target with its own Runtime and its
  /// own Network domain. Without this, the worker's traffic (including the .wasm it
  /// fetches) is invisible from the page's session.
  send(method, params = {}, sessionId) {
    const id = this.#next++;
    this.#ws.send(JSON.stringify(sessionId ? { id, method, params, sessionId } : { id, method, params }));
    return new Promise((res, rej) => this.#pending.set(id, { res, rej }));
  }

  on(method, fn) {
    this.handlers.set(method, fn);
  }

  close() {
    this.#ws.close();
  }

  /// Evaluate an expression in the page and return its value. `await`s a promise
  /// result, and turns a page-side throw into a real rejection here.
  async eval(expression) {
    const r = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) {
      throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    }
    return r.result.value;
  }
}

/// The page's own state, or null if there is no document yet. Every read of the
/// page goes through this: right after `Page.navigate` the document can still be
/// the empty initial one, where `document.body` is null and a bare
/// `document.body.dataset` THROWS rather than politely returning undefined.
///
/// THE PARENTHESES ARE LOAD-BEARING. This string gets interpolated into page-side
/// expressions as `${PAGE_STATE} === 'done'`, and `===` binds tighter than `??`.
/// Unparenthesized, that parses as `state ?? (null === 'done')` — i.e. `state ??
/// false` — which is TRUTHY for every state the page can be in.
const PAGE_STATE = "(document.body?.dataset.state ?? null)";

/// Poll the page until `expression` is truthy (or give up). The demo mirrors its
/// state onto <body data-state>, so this is how we wait for `init()` and for a
/// conversion — no arbitrary sleeps.
async function waitFor(cdp, expression, what, timeoutMs = 90_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await cdp.eval(expression)) return;
    await sleep(100);
  }
  const state = await cdp.eval(PAGE_STATE);
  const err = await cdp.eval("document.getElementById('status')?.textContent ?? ''");
  await die(`timed out waiting for ${what} (page state: ${state} — "${err?.trim()}")`);
}

// ── independent container parses (decoders the crate never met) ────────────────

function readWebp(buf) {
  const riff = buf.subarray(0, 4).toString("ascii") === "RIFF";
  const webp = buf.subarray(8, 12).toString("ascii") === "WEBP";
  const chunk = buf.subarray(12, 16).toString("ascii");
  const out = { riff, webp, chunk, width: null, height: null };
  if (chunk === "VP8L" && buf[20] === 0x2f) {
    const bits = buf.readUInt32LE(21);
    out.width = (bits & 0x3fff) + 1;
    out.height = ((bits >> 14) & 0x3fff) + 1;
  }
  return out;
}

// The engine CANNOT check its own AVIF output (encode-only, DEC-065), so an outside
// reader is the only proof there is. This is one, from the ISOBMFF/AVIF spec: the
// `ftyp` box's major brand and the `ispe` (image spatial extents) box.
function readAvif(buf) {
  const out = { ftyp: false, brand: null, width: null, height: null };
  if (buf.length < 16 || buf.subarray(4, 8).toString("ascii") !== "ftyp") return out;
  out.ftyp = true;
  out.brand = buf.subarray(8, 12).toString("ascii");
  const at = buf.indexOf("ispe", 0, "ascii");
  if (at > 0) {
    out.width = buf.readUInt32BE(at + 8);
    out.height = buf.readUInt32BE(at + 12);
  }
  return out;
}

/// The bytes behind the page's download link — ALL of them, out of the browser and
/// into Node, so they can be parsed by something that is not the browser.
async function downloadBytes(cdp) {
  const b64 = await cdp.eval(`
    (async () => {
      const buf = await (await fetch(document.getElementById('download').href)).arrayBuffer();
      const u8 = new Uint8Array(buf);
      let s = '';
      for (let i = 0; i < u8.length; i++) s += String.fromCharCode(u8[i]);
      return btoa(s);
    })()
  `);
  return Buffer.from(b64, "base64");
}

// ── 1. the assembled demo ─────────────────────────────────────────────────────

console.log("\n── the assembled demo ──");

if (!existsSync(join(demoDir, "vendor", "crustyimg_bg.wasm"))) {
  await die("demo/vendor/ is not assembled — run `just demo-build` first (`just demo-smoke` does)");
}
ok("demo/vendor/ carries the vendored, profile-guarded crustyimg-wasm");

const chromePath = findChrome();
if (!chromePath) {
  await die(
    "no Chrome/Chromium found. Install one, or set CHROME=/path/to/chrome — this test drives a " +
      "REAL browser on purpose (see the header).",
  );
}

// ── 2. serve it ───────────────────────────────────────────────────────────────

console.log("\n── serve → headless Chrome ──");

const served = [];
const { server, url: baseUrl } = await startServer({
  root: demoDir,
  onRequest: (p) => served.push(p),
});
cleanups.push(() => new Promise((r) => server.close(r)));
ok(`serving demo/ at ${baseUrl}`);

const profileDir = mkdtempSync(join(tmpdir(), "crustyimg-chrome-"));
cleanups.push(() => rmSync(profileDir, { recursive: true, force: true }));

const chrome = spawn(
  chromePath,
  [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--remote-allow-origins=*",
    "--remote-debugging-port=0",
    `--user-data-dir=${profileDir}`,
    ...(process.env.CI ? ["--no-sandbox"] : []),
    "about:blank",
  ],
  { stdio: ["ignore", "ignore", "pipe"] },
);
cleanups.push(() => chrome.kill());

const portFile = join(profileDir, "DevToolsActivePort");
let devtoolsPort;
for (let i = 0; i < 100 && !devtoolsPort; i++) {
  await sleep(100);
  if (existsSync(portFile)) devtoolsPort = readFileSync(portFile, "utf8").split("\n")[0].trim();
}
if (!devtoolsPort) await die("headless Chrome never came up (no DevToolsActivePort)");

const targets = await (await fetch(`http://127.0.0.1:${devtoolsPort}/json/list`)).json();
const page = targets.find((t) => t.type === "page");
if (!page) await die("headless Chrome exposed no page target");

const cdp = await CDP.attach(page.webSocketDebuggerUrl);
cleanups.push(() => cdp.close());
ok(`attached to headless Chrome (${chromePath.split("/").pop()})`);

// ── 3. load the page ──────────────────────────────────────────────────────────

console.log("\n── the browser loads the engine ──");

const consoleErrors = [];
const requests = [];
const responses = new Map();

cdp.on("Runtime.consoleAPICalled", (p) => {
  if (p.type === "error") {
    consoleErrors.push(p.args.map((a) => a.description ?? a.value).join(" "));
  }
});
cdp.on("Runtime.exceptionThrown", (p) => {
  consoleErrors.push(p.exceptionDetails.exception?.description ?? p.exceptionDetails.text);
});
cdp.on("Network.requestWillBeSent", (p) => requests.push(p.request.url));
cdp.on("Network.responseReceived", (p) => responses.set(p.response.url, p.response));

await cdp.send("Runtime.enable");
await cdp.send("Network.enable");
await cdp.send("Page.enable");
await cdp.send("DOM.enable");

// Clipboard reads/writes need permission in headless Chrome; the funnel copy button
// uses navigator.clipboard.writeText, and the funnel check reads it back.
await cdp
  .send("Browser.grantPermissions", {
    origin: baseUrl,
    permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"],
  })
  .catch(() => {
    /* older Chrome may not know these names; the dataset fallback still proves the write */
  });

// Auto-attach to the engine's Worker (a separate CDP target). `waitForDebuggerOnStart`
// pauses it at its first line so Network is enabled BEFORE it fetches the .wasm.
const workerTargets = [];
cdp.on("Target.attachedToTarget", async (p) => {
  if (p.targetInfo.type !== "worker") return;
  workerTargets.push(p.targetInfo.url);
  await cdp.send("Runtime.enable", {}, p.sessionId);
  await cdp.send("Network.enable", {}, p.sessionId);
  await cdp.send("Runtime.runIfWaitingForDebugger", {}, p.sessionId);
});
await cdp.send("Target.setAutoAttach", {
  autoAttach: true,
  waitForDebuggerOnStart: true,
  flatten: true,
});

await cdp.send("Page.navigate", { url: `${baseUrl}/index.html` });

await waitFor(cdp, `${PAGE_STATE} === 'ready'`, "the worker's init() to resolve");
ok("await init() resolved IN A MODULE WORKER — instantiateStreaming ran off the page's thread");

check(
  workerTargets.some((u) => u.endsWith("/worker.js")),
  `the engine is in a Web Worker — a separate browser thread, attached as its own target ` +
    `(${workerTargets.join(", ") || "NO WORKER TARGET"})`,
);

const crateVersion = readFileSync(join(repoRoot, "Cargo.toml"), "utf8").match(
  /^version\s*=\s*"([^"]+)"/m,
)[1];

await waitFor(
  cdp,
  "(document.getElementById('version')?.textContent ?? '').trim().length > 0",
  "the page to report its engine version",
);
const shown = await cdp.eval("document.getElementById('version').textContent");
check(
  shown.trim() === `crustyimg ${crateVersion}`,
  `the page calls version() and gets the crate version — "${shown.trim()}"`,
);

const wasmResponse = responses.get(`${baseUrl}/vendor/crustyimg_bg.wasm`);
check(
  wasmResponse?.mimeType === "application/wasm",
  `the .wasm was served as application/wasm (got ${wasmResponse?.mimeType ?? "no response at all"})`,
);

check(consoleErrors.length === 0, "no console errors while loading");
if (consoleErrors.length) console.error(`    ${consoleErrors.join("\n    ")}`);

// ── a fixture staging dir + a drop helper ──────────────────────────────────────

const fixtureDir = mkdtempSync(join(tmpdir(), "crustyimg-fixture-"));
cleanups.push(() => rmSync(fixtureDir, { recursive: true, force: true }));

/// Reset the controls to the hero defaults (Auto, downscale to 2048, no budget), so
/// each test starts from the one-click path unless it opts into Advanced.
async function resetControls() {
  await cdp.eval(`
    document.getElementById('format').value = 'auto';
    document.getElementById('keepfull').checked = false;
    document.getElementById('maxedge').disabled = false;
    document.getElementById('maxedge').value = '${WEB_EDGE}';
    document.getElementById('maxbytes').value = '';
  `);
}

/// Put a file in the picker and wait for the page to finish converting THAT file.
/// Waiting on `state === 'done'` alone is not enough: by the time a second file is
/// dropped the page is already 'done' from the first, so the wait would be satisfied
/// by the previous conversion. `render()` overwrites `#result.dataset` wholesale, so a
/// unique token stamped there before the drop is proof of freshness when it is gone.
let dropSeq = 0;
async function drop(path) {
  const token = `awaiting-conversion-${++dropSeq}`;
  await cdp.eval(`
    document.getElementById('result').dataset.outFormat = ${JSON.stringify(token)};
    document.body.dataset.state = 'ready';
  `);
  const doc = await cdp.send("DOM.getDocument");
  const { nodeId } = await cdp.send("DOM.querySelector", {
    nodeId: doc.root.nodeId,
    selector: "#file",
  });
  await cdp.send("DOM.setFileInputFiles", { nodeId, files: [path] });
  await waitFor(
    cdp,
    `${PAGE_STATE} === 'done' && ` +
      `document.getElementById('result')?.dataset.outFormat !== ${JSON.stringify(token)}`,
    `the conversion of ${path}`,
  );
  return cdp
    .eval(
      "JSON.stringify({ ...document.getElementById('result').dataset, " +
        "inDims: document.getElementById('in-dims').textContent, " +
        "inFormat: document.getElementById('in-format').textContent, " +
        "delta: document.getElementById('delta').textContent, " +
        "resizeNote: document.getElementById('resize-note').textContent, " +
        "score: document.getElementById('score').textContent, " +
        // SPEC-081: the raw metric (off #result.dataset, before it is shadowed) plus
        // the score panel's own sub-elements, so a test can read the number and band
        // directly rather than parsing prose.
        "scoreRaw: document.getElementById('result').dataset.score, " +
        "scoreMode: (document.getElementById('score').dataset.mode ?? ''), " +
        "scoreValue: document.getElementById('score-value').textContent, " +
        "scoreBand: document.getElementById('score-band').textContent, " +
        "scoreSource: document.getElementById('score-source').textContent, " +
        "scoreMeterHidden: document.getElementById('score-meter').hidden, " +
        "scoreFillWidth: document.getElementById('score-fill').style.width, " +
        "download: document.getElementById('download').getAttribute('download'), " +
        "href: document.getElementById('download').href })",
    )
    .then(JSON.parse);
}

// ── 4. default_is_web_flow_smaller_avif ────────────────────────────────────────

console.log("\n── default_is_web_flow_smaller_avif: a photo, no controls touched ──");

await resetControls();
const heroPath = join(fixtureDir, "photo.png");
writeFileSync(heroPath, makePhotoPng(HERO_W, HERO_H, 7));

const requestsBeforeHero = requests.length;
const hero = await drop(heroPath);

check(
  hero.inDims === `${HERO_W}×${HERO_H}` && hero.inFormat === "png",
  `the ${HERO_W}×${HERO_H} photo went in as ${hero.inDims} ${hero.inFormat}`,
);
check(
  hero.outFormat === "avif",
  `the default (no controls touched) modernized it to AVIF — "${hero.outFormat}" (Auto chose it)`,
);
check(
  Number(hero.outBytes) < Number(hero.inBytes),
  `and it is SMALLER — ${hero.inBytes} B → ${hero.outBytes} B`,
);
check(
  Math.max(Number(hero.outWidth), Number(hero.outHeight)) <= WEB_EDGE,
  `and DOWNSCALED — output ${hero.outWidth}×${hero.outHeight}, long edge ≤ ${WEB_EDGE} (the web flow ran)`,
);
check(
  hero.resized === "true" && /Resized to \d+×\d+ for web/.test(hero.resizeNote),
  `and the page SAYS it downscaled — "${hero.resizeNote}"`,
);
check(
  hero.href?.startsWith("blob:") && hero.download === "photo.avif" && hero.keptOriginal === "false",
  `a download is produced from the page's own blob: URL — <a download="${hero.download}">`,
);

// ── 5. it is client-side, and it is not lying about it ────────────────────────

console.log("\n── 100% client-side ──");

const duringHero = requests
  .slice(requestsBeforeHero)
  .filter((u) => !u.startsWith("blob:") && !u.startsWith("data:"));
check(
  duringHero.length === 0,
  `the web-flow conversion made ZERO network requests (the worker's traffic is in this log too)`,
);
if (duringHero.length) console.error(`    ${duringHero.join("\n    ")}`);

const offOrigin = requests.filter(
  (u) => !u.startsWith(baseUrl) && !u.startsWith("blob:") && !u.startsWith("data:"),
);
check(
  offOrigin.length === 0,
  `the page loaded nothing off-origin — no CDN, no font, no analytics (${requests.length} requests, all ${baseUrl})`,
);
if (offOrigin.length) console.error(`    ${offOrigin.join("\n    ")}`);
console.log(`    served: ${served.join(", ")}`);

// ── 6. the AVIF output is real, per decoders the crate never met ──────────────

console.log("\n── the AVIF is valid (independent decoders) ──");

const heroBytes = await downloadBytes(cdp);
const box = readAvif(heroBytes);
check(
  box.ftyp && box.brand === "avif",
  `the downloaded bytes are a real AVIF container (ftyp brand "${box.brand}", ${heroBytes.length} B — ` +
    `parsed here, by a reader the crate never touched)`,
);
check(
  Math.max(box.width, box.height) <= WEB_EDGE && box.width > 0,
  `the AVIF's own ispe box says ${box.width}×${box.height} (independent parse; long edge ≤ ${WEB_EDGE})`,
);

const chromeDecode = await cdp
  .eval(
    `(async () => {
      const blob = await (await fetch(document.getElementById('download').href)).blob();
      const bitmap = await createImageBitmap(blob);
      return JSON.stringify({ w: bitmap.width, h: bitmap.height });
    })()`,
  )
  .then(JSON.parse);
check(
  chromeDecode.w === box.width && chromeDecode.h === box.height,
  `Chrome's own AVIF decoder reads it back as ${chromeDecode.w}×${chromeDecode.h} — an independent ` +
    `decoder, in another language, agreeing this is the image`,
);

if (process.platform === "darwin") {
  const avifFile = join(fixtureDir, "hero.avif");
  writeFileSync(avifFile, heroBytes);
  const sips = spawnSync("sips", ["-g", "format", "-g", "pixelWidth", "-g", "pixelHeight", avifFile], {
    encoding: "utf8",
  });
  const said = `${sips.stdout ?? ""}`.replace(/\s+/g, " ").trim();
  check(
    sips.status === 0 && /format:\s*avif/i.test(said) && new RegExp(`pixelWidth: ${box.width}\\b`).test(said),
    `macOS \`sips\` — a third decoder, from neither the crate nor the browser — reads it as AVIF ${box.width}×${box.height} ("${said}")`,
  );
} else {
  console.log(`  · (sips is macOS-only — skipped on ${process.platform}; two decoders agreed above)`);
}

// ── 6½. the score panel (SPEC-081): input↔output SSIMULACRA2, sourced honestly ──

console.log("\n── the score panel (SPEC-081): honestly sourced perceptual score ──");

// avif_shows_browser_score — the hero AVIF is the output SPEC-080 could NOT score
// (the engine has no AVIF decoder, DEC-065). SPEC-081 decodes it back in the browser
// and scores it: this asserts scoredBy=="browser" and a real numeric score.
const avifScore = Number(hero.scoreRaw);
check(
  hero.scoredBy === "browser" && hero.scoreRaw !== "" && Number.isFinite(avifScore),
  `avif_shows_browser_score: the AVIF hero is scored by decoding it BACK in the browser — ` +
    `scoredBy="${hero.scoredBy}", SSIMULACRA2 ${hero.scoreRaw} (the output SPEC-080 left unscored)`,
);
check(
  hero.scoreMode === "measured" && /SSIMULACRA2/.test(hero.scoreValue) && hero.scoreBand.trim().length > 0,
  `and it renders on an interpretable band, not a bare float — value "${hero.scoreValue}", band "${hero.scoreBand}"`,
);
// Documented sanity (NOT a golden): a q85 (SPEC-095) photo→AVIF is a visually-lossless
// encode, so the raw score lands well up the scale — high enough to read "high"/
// "visually lossless", and NOT clamped into a 0–100 % (raw metric, can exceed 100).
check(
  avifScore > 50 && avifScore < 130,
  `SANITY: the browser-decode score is plausible for a visually-lossless q85 AVIF ` +
    `(${avifScore.toFixed(2)} — up the scale, raw not a %; band "${hero.scoreBand}")`,
);
// The meter is wired to the value, not decorative: for a measured score its fill is a
// real [0,100]% width and it is NOT hidden. (The clamp for out-of-range values — 0% at
// negative, 100% above 100 — is exercised by the verify-session negative controls.)
const heroFill = Number.parseFloat(hero.scoreFillWidth);
check(
  hero.scoreMeterHidden === false &&
    /%$/.test(hero.scoreFillWidth) &&
    heroFill >= 0 &&
    heroFill <= 100,
  `and the meter reflects the score — fill ${hero.scoreFillWidth}, shown (not hidden)`,
);

// jpeg_shows_engine_score — force JPEG (a searched lossy encode). The engine scores
// this one ITSELF; the panel must attribute it to the engine and show the RAW value.
await resetControls();
await cdp.eval("document.getElementById('format').value = 'jpeg';");
// A DISTINCT path from the hero drop: re-setting the same file into #file fires no
// `change` event, so the conversion would never start (every drop() uses a fresh file).
const jpegPath = join(fixtureDir, "photo-for-jpeg.png");
writeFileSync(jpegPath, makePhotoPng(HERO_W, HERO_H, 7));
const jpeg = await drop(jpegPath);
const jpegScore = Number(jpeg.scoreRaw);
check(
  jpeg.outFormat === "jpeg" && jpeg.scoredBy === "engine" && Number.isFinite(jpegScore),
  `jpeg_shows_engine_score: a searched JPEG is scored BY THE ENGINE — out="${jpeg.outFormat}", ` +
    `scoredBy="${jpeg.scoredBy}", SSIMULACRA2 ${jpeg.scoreRaw}`,
);
check(
  jpeg.scoreMode === "measured" && /SSIMULACRA2/.test(jpeg.scoreValue) && /engine/i.test(jpeg.scoreSource),
  `and the panel says the ENGINE measured it, raw (not clamped to 0–100) — "${jpeg.scoreSource.trim()}"`,
);

// lossless_shows_lossless_not_a_number — a graphic routes to lossless; a lossless
// output has nothing to score, so the panel must say so with NO fabricated number.
await resetControls();
const graphicPath = join(fixtureDir, "graphic.png");
writeFileSync(graphicPath, makePng(1600, 1200));
const lossless = await drop(graphicPath);
check(
  /webp|png/.test(lossless.outFormat) && lossless.scoredBy === "lossless",
  `lossless_shows_lossless_not_a_number: a graphic → lossless ${lossless.outFormat}, scoredBy="${lossless.scoredBy}"`,
);
check(
  lossless.scoreRaw === "" && lossless.scoreValue.trim() === "" && lossless.scoreMeterHidden === true,
  `and there is NO number and NO meter — nothing fabricated (raw="${lossless.scoreRaw}", value="${lossless.scoreValue}", meterHidden=${lossless.scoreMeterHidden})`,
);
check(
  /lossless|pixel/i.test(lossless.scoreSource),
  `and it says why in plain words — "${lossless.scoreSource.trim()}"`,
);

// ── 7. funnel_shows_web_command_and_copies ─────────────────────────────────────

console.log("\n── funnel_shows_web_command_and_copies ──");

await resetControls();
const funnelPath = join(fixtureDir, "my-beach-photo.png");
writeFileSync(funnelPath, makePhotoPng(600, 450, 2));
await drop(funnelPath);

const expectedCmd = "crustyimg web my-beach-photo.png";
const funnel = await cdp
  .eval(
    "JSON.stringify({ " +
      "hidden: document.getElementById('funnel').hidden, " +
      "cmd: document.getElementById('funnel-cmd').textContent, " +
      "cmdData: document.getElementById('funnel-cmd').dataset.cmd, " +
      "recipe: document.getElementById('recipe-text').textContent })",
  )
  .then(JSON.parse);

check(
  !funnel.hidden && funnel.cmd.trim() === expectedCmd,
  `the funnel shows the exact command for the dropped file — "${funnel.cmd.trim()}"`,
);

// Click the real copy button with a TRUSTED gesture (a synthetic .click() is not a
// user activation, and the Clipboard API needs one) and read the clipboard back. The
// button is scrolled into view first so its viewport coordinates are hittable.
await cdp.send("Page.bringToFront").catch(() => {});
await cdp.eval("document.getElementById('copy-cmd').scrollIntoView({ block: 'center' })");
const btnBox = await cdp
  .eval(
    "(() => { const r = document.getElementById('copy-cmd').getBoundingClientRect(); " +
      "return JSON.stringify({ x: r.x + r.width / 2, y: r.y + r.height / 2 }); })()",
  )
  .then(JSON.parse);
await cdp.send("Input.dispatchMouseEvent", {
  type: "mousePressed",
  x: btnBox.x,
  y: btnBox.y,
  button: "left",
  clickCount: 1,
});
await cdp.send("Input.dispatchMouseEvent", {
  type: "mouseReleased",
  x: btnBox.x,
  y: btnBox.y,
  button: "left",
  clickCount: 1,
});
await sleep(250);
const copiedData = await cdp.eval("document.getElementById('copy-cmd').dataset.copied ?? ''");
let clip = "";
try {
  clip = await cdp.eval("navigator.clipboard.readText()");
} catch {
  /* headless clipboard read can be denied even with permission granted; dataset proves the write resolved */
}
check(
  copiedData === expectedCmd && (clip === "" || clip === expectedCmd),
  `the copy button wrote "${expectedCmd}" to the clipboard` +
    (clip ? ` (read back: "${clip}")` : " (writeText resolved; clipboard read unavailable headless)"),
);

const webToml = readFileSync(join(repoRoot, "recipes", "web.toml"), "utf8");
check(
  funnel.recipe === webToml,
  `the funnel's recipe is VERBATIM from recipes/web.toml (${funnel.recipe.length} B; byte-identical — ` +
    `it cannot drift from what the CLI runs)`,
);

// ── 8. never_bigger_keeps_original ─────────────────────────────────────────────

console.log("\n── never_bigger_keeps_original ──");

// An input the engine cannot beat: a moderate-quality JPEG of a photo, small enough
// (≤ 2048) that nothing downscales — the best modern re-encode comes out LARGER, so
// the never-bigger guard hands the original back. Minted here via the SAME wasm the
// browser runs, so the ">= input" relationship is a property of the engine, not luck.
const { initSync, optimizeDetailed: nodeOptimize } = await import(join(demoDir, "vendor", "crustyimg.js"));
initSync({ module: readFileSync(join(demoDir, "vendor", "crustyimg_bg.wasm")) });
const jpegBytes = Buffer.from(nodeOptimize(makePhotoPng(1200, 900, 4), "jpeg", undefined, 45000, undefined).bytes);
const keepPath = join(fixtureDir, "already-small.jpg");
writeFileSync(keepPath, jpegBytes);

await resetControls();
const kept = await drop(keepPath);

check(
  kept.keptOriginal === "true",
  `the engine could not beat the ${jpegBytes.length} B JPEG — the never-bigger guard fired (kept-original state)`,
);
check(
  /kept your file/i.test(kept.delta),
  `and the page SAYS so honestly — "${kept.delta.trim().slice(0, 70)}…"`,
);
check(
  kept.download === "already-small.jpg",
  `the download reverts to the ORIGINAL file name — "${kept.download}"`,
);

const keptDownload = await downloadBytes(cdp);
check(
  keptDownload.length === jpegBytes.length && keptDownload.equals(jpegBytes),
  `and the downloaded bytes ARE the original, byte-for-byte (${keptDownload.length} B == ${jpegBytes.length} B)`,
);

// ── 9. advanced_full_resolution_shows_timer ────────────────────────────────────

console.log("\n── advanced_full_resolution_shows_timer (the slow path warns + counts + stays live) ──");

const fullPath = join(fixtureDir, "huge.png");
writeFileSync(fullPath, makePhotoPng(FULL_W, FULL_H, 11));

// THE PROBE (SPEC-078): two independent main-thread heartbeats counting how many
// times each ran while the page was `converting`. If the encode were a synchronous
// wasm call on this thread, NEITHER could run during it.
await cdp.eval(`
  window.__probe = { duringConvert: 0, framesDuringConvert: 0 };
  const converting = () => document.body.dataset.state === 'converting';
  setInterval(() => { if (converting()) window.__probe.duringConvert++; }, 20);
  (function frame() {
    if (converting()) window.__probe.framesDuringConvert++;
    requestAnimationFrame(frame);
  })();
`);

// Open Advanced and choose keep-full-resolution — the one path slow enough to warrant
// the warning + timer. Set the control property directly (no change event) so the drop
// below reads it live without a debounced stray convert firing first.
await cdp.eval(`
  document.getElementById('advanced').open = true;
  document.getElementById('keepfull').checked = true;
  document.getElementById('maxedge').disabled = true;
`);

// Drop huge.png but DON'T await the whole conversion — we need to observe it MID-flight.
const fullToken = `awaiting-conversion-${++dropSeq}`;
await cdp.eval(`
  document.getElementById('result').dataset.outFormat = ${JSON.stringify(fullToken)};
  document.body.dataset.state = 'ready';
`);
{
  const doc = await cdp.send("DOM.getDocument");
  const { nodeId } = await cdp.send("DOM.querySelector", { nodeId: doc.root.nodeId, selector: "#file" });
  await cdp.send("DOM.setFileInputFiles", { nodeId, files: [fullPath] });
}

await waitFor(cdp, `${PAGE_STATE} === 'converting'`, "the full-resolution encode to start");

const warnShown = await cdp.eval("!document.getElementById('mp-warning').hidden");
const warnText = await cdp.eval("document.getElementById('mp-warning').textContent");
check(
  warnShown && /MP at full resolution/i.test(warnText),
  `the megapixel warning appears for the full-res path — "${warnText.trim()}"`,
);

// Sample the HONEST elapsed timer twice, mid-encode: a value, then a larger one. The
// timer is driven by setInterval on the MAIN thread, so it can only advance if the
// main thread is alive — which is the point.
const t1 = Number.parseFloat(await cdp.eval("document.getElementById('elapsed').textContent"));
const interactive = await cdp.eval("!document.getElementById('elapsed').hidden");
await sleep(700);
const t2 = Number.parseFloat(await cdp.eval("document.getElementById('elapsed').textContent"));
check(
  interactive && Number.isFinite(t1) && Number.isFinite(t2) && t2 > t1,
  `the elapsed timer COUNTS UP during the encode (${t1}s → ${t2}s) — honest seconds, no fake %, ` +
    `and the main thread is alive to advance it`,
);

await waitFor(
  cdp,
  `${PAGE_STATE} === 'done' && document.getElementById('result')?.dataset.outFormat !== ${JSON.stringify(fullToken)}`,
  "the full-resolution encode to finish",
);

const full = await cdp
  .eval(
    "JSON.stringify({ ...document.getElementById('result').dataset })",
  )
  .then(JSON.parse);
const encodeMs = Number(full.elapsedMs);
check(
  full.resized === "false" &&
    Number(full.outWidth) === FULL_W &&
    Number(full.outHeight) === FULL_H,
  `keep-full-resolution kept every pixel — ${full.outWidth}×${full.outHeight}, not downscaled`,
);
check(
  encodeMs >= MIN_ENCODE_MS,
  `the full-res encode was a real blocking window (${encodeMs} ms ≥ ${MIN_ENCODE_MS} ms)`,
);

const probe = await cdp.eval("JSON.stringify(window.__probe)").then(JSON.parse);
check(
  probe.duringConvert >= 5 && probe.framesDuringConvert >= 5,
  `THE MAIN THREAD STAYED ALIVE THROUGH IT — ${probe.duringConvert} timer callbacks and ` +
    `${probe.framesDuringConvert} animation frames ran DURING the ${encodeMs} ms encode`,
);

// THE NEGATIVE CONTROL — not optional. Freeze the main thread on purpose, in the same
// state the page shows while converting, and confirm the probe goes to ZERO — which is
// what it would have read all along if the engine had stayed on this thread.
const control = await cdp
  .eval(
    `(async () => {
      const before = { ...window.__probe };
      document.body.dataset.state = 'converting';
      const until = performance.now() + ${BLOCK_MS};
      while (performance.now() < until) {}
      const ticks = window.__probe.duringConvert - before.duringConvert;
      const frames = window.__probe.framesDuringConvert - before.framesDuringConvert;
      document.body.dataset.state = 'done';
      return JSON.stringify({ ticks, frames });
    })()`,
  )
  .then(JSON.parse);
check(
  control.ticks === 0 && control.frames === 0,
  `and the probe CAN see a freeze: a deliberate ${BLOCK_MS} ms main-thread block ran ` +
    `${control.ticks} timers / ${control.frames} frames — so the counts above are evidence, not noise`,
);

// ── 10. every input format the page claims (reach) ────────────────────────────

console.log("\n── every input format the page claims ──");

const { transform: nodeTransform } = await import(join(demoDir, "vendor", "crustyimg.js"));
const keepRecipe = 'version = "1"\n\n[[step]]\nop = "resize"\nmode = "exact"\nwidth = 64\nheight = 48\n';
const srcPng = makePng(64, 48);
for (const fmt of ["jpeg", "gif", "webp"]) {
  writeFileSync(join(fixtureDir, `reach.${fmt}`), Buffer.from(nodeTransform(srcPng, keepRecipe, fmt)));
}

await resetControls();

const svg = await drop(join(repoRoot, "tests", "fixtures", "svg", "rect_text_40x30.svg"));
check(
  svg.inDims === "40×30" && Number(svg.outBytes) > 0,
  `SVG in → rasterized to ${svg.inDims} → ${svg.outWidth}×${svg.outHeight} ${svg.outFormat} out (${svg.outBytes} B)`,
);

for (const fmt of ["jpeg", "gif", "webp"]) {
  const r = await drop(join(fixtureDir, `reach.${fmt}`));
  check(
    r.inFormat === fmt && r.inDims === "64×48" && Number(r.outBytes) > 0,
    `${fmt.toUpperCase()} in → info() reads ${r.inDims} ${r.inFormat} → ${r.outWidth}×${r.outHeight} ${r.outFormat} out (${r.outBytes} B)`,
  );
}

// ── 11. .avif in — the browser decodes what the engine cannot ─────────────────

console.log("\n── .avif in (the browser decodes what the engine cannot) ──");

await resetControls();
const requestsBeforeAvifIn = requests.length;
const fromAvif = await drop(join(repoRoot, "tests", "fixtures", "avif", "solid_16x16.avif"));
check(
  fromAvif.inFormat === "avif" && fromAvif.inDims === "16×16" && fromAvif.decodedBy === "browser",
  `an .avif INPUT is read: ${fromAvif.inDims} ${fromAvif.inFormat}, decoded by the ${fromAvif.decodedBy} ` +
    `(the page does not pretend the engine did it)`,
);
check(Number(fromAvif.outBytes) > 0, `.avif → ${fromAvif.outFormat}: ${fromAvif.outBytes} B produced in the browser`);

const duringAvifIn = requests
  .slice(requestsBeforeAvifIn)
  .filter((u) => !u.startsWith("blob:") && !u.startsWith("data:"));
check(duringAvifIn.length === 0, "and even the browser-decode path made ZERO network requests");
if (duringAvifIn.length) console.error(`    ${duringAvifIn.join("\n    ")}`);

check(consoleErrors.length === 0, "no console errors in the page OR the worker, after all of it");
if (consoleErrors.length) console.error(`    ${consoleErrors.join("\n    ")}`);

// ── 12. the file:// failure mode is real (which is WHY we serve) ──────────────

console.log("\n── the file:// failure mode is real (which is WHY we serve) ──");

const { targetId } = await cdp.send("Target.createTarget", {
  url: `file://${join(demoDir, "index.html")}`,
});
cleanups.push(() => cdp.send("Target.closeTarget", { targetId }));

const fileTargets = await (await fetch(`http://127.0.0.1:${devtoolsPort}/json/list`)).json();
const fileTarget = fileTargets.find((t) => t.id === targetId);
const fileCdp = await CDP.attach(fileTarget.webSocketDebuggerUrl);
cleanups.push(() => fileCdp.close());
await fileCdp.send("Runtime.enable");
await sleep(2000);

const fileState = await fileCdp.eval(PAGE_STATE);
const fileStatus = await fileCdp.eval("document.getElementById('status')?.textContent ?? ''");
const moduleRan = await fileCdp.eval("!!document.querySelector('#drop:not([hidden])')");

check(
  !moduleRan,
  "over file://, the page cannot convert anything — the module script is CORS-blocked before it runs",
);
check(
  fileState === "error" &&
    /will not load its code/i.test(fileStatus) &&
    !/Could not load the WebAssembly engine/i.test(fileStatus),
  `over file://, it is the CLASSIC script that speaks — demo.js never ran — and the page SAYS why ` +
    `instead of hanging on "Loading…" — "${fileStatus.trim().slice(0, 52)}…"`,
);

await cleanup();

console.log("");
if (failed) {
  console.error(`demo-smoke: ${failed} check(s) FAILED\n`);
  process.exit(1);
}
console.log(
  `demo-smoke: the demo loads crustyimg ${crateVersion} in a real browser over HTTP, runs the web ` +
    `flow client-side, and funnels to the CLI. ✓\n`,
);
