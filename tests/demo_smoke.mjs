#!/usr/bin/env node
// SPEC-077: the demo's earned verdict — the page, in a real browser, over HTTP.
//
// Everything in this wave up to now ran the wasm in NODE (`initSync` with bytes we
// handed it). The browser path is different in the one way that can break it:
// `init()` fetches `crustyimg_bg.wasm` and calls `WebAssembly.instantiateStreaming`,
// which needs the server to send `application/wasm`. A page that renders perfectly
// and cannot instantiate the engine is THE failure mode here, and no amount of Node
// testing would see it. So this serves the assembled demo and drives it in headless
// Chrome, through the real user path:
//
//   serve → load → init() → put a PNG in the file picker → convert → read the DOM
//   → fetch the download's blob → decode the bytes with a parser we wrote ourselves.
//
// It also asserts the pitch: the .wasm arrives as `application/wasm`, and NOTHING
// is fetched during the conversion — no backend, no upload, no round trip.
//
// Chrome is driven over the DevTools Protocol directly (a WebSocket + JSON-RPC,
// ~80 lines below). No Puppeteer/Playwright: a demo whose whole point is "no
// toolchain, no install" should not need a 300 MB browser driver in devDependencies
// to prove it works. Chrome itself is the only requirement — set CHROME to point at
// it if it is not where we look.
//
// Run: `just demo-smoke`.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { makePng, readIhdr } from "../scripts/lib/png.mjs";
import { startServer } from "../scripts/serve.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const demoDir = join(repoRoot, "demo");

const FIXTURE_W = 64;
const FIXTURE_H = 48;
const MAX_EDGE = 32; // the resize the second conversion asks for

// The AVIF fixture is big on purpose: rav1e is serial and slow, and the claim under
// test is about what the page can do WHILE it encodes. A thumbnail would be done
// before there was anything to observe.
const AVIF_W = 800;
const AVIF_H = 600;
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
  /// fetches) is invisible from the page's session — which is exactly what happened
  /// the first time the conversions moved off the main thread.
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
/// `document.body.dataset` THROWS rather than politely returning undefined. (It
/// does not reliably throw on a fast machine, which is precisely why it has to be
/// written this way — CI found it; my laptop never would have.)
///
/// THE PARENTHESES ARE LOAD-BEARING. This string gets interpolated into page-side
/// expressions as `${PAGE_STATE} === 'done'`, and `===` binds tighter than `??`.
/// Unparenthesized, that parses as `state ?? (null === 'done')` — i.e. `state ??
/// false` — which is TRUTHY for every state the page can be in. Every waitFor()
/// below would return on its first poll having waited for nothing, and the reads
/// after it would race the conversion they are supposed to be waiting for.
const PAGE_STATE = "(document.body?.dataset.state ?? null)";

/// Poll the page until `expression` is truthy (or give up). The demo mirrors its
/// state onto <body data-state>, so this is how we wait for `init()` and for a
/// conversion — no arbitrary sleeps.
async function waitFor(cdp, expression, what, timeoutMs = 60_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await cdp.eval(expression)) return;
    await sleep(100);
  }
  const state = await cdp.eval(PAGE_STATE);
  const err = await cdp.eval("document.getElementById('status')?.textContent ?? ''");
  await die(`timed out waiting for ${what} (page state: ${state} — "${err?.trim()}")`);
}

// ── an independent WebP header parse ──────────────────────────────────────────
// The engine's own `info()` already reports the output's dimensions, but that is
// the crate agreeing with itself. This reads the container the way a decoder that
// never met crustyimg would: RIFF envelope, then the VP8L bitstream's 14-bit
// width-1/height-1 fields. (This build's WebP is lossless — VP8L — because lossy
// WebP is a C library and this is the pure-Rust engine.)
function readWebp(buf) {
  const riff = buf.subarray(0, 4).toString("ascii") === "RIFF";
  const webp = buf.subarray(8, 12).toString("ascii") === "WEBP";
  const chunk = buf.subarray(12, 16).toString("ascii");
  const out = { riff, webp, chunk, width: null, height: null };

  if (chunk === "VP8L" && buf[20] === 0x2f) {
    // 14 bits width-1, then 14 bits height-1, little-endian bit order.
    const bits = buf.readUInt32LE(21);
    out.width = (bits & 0x3fff) + 1;
    out.height = ((bits >> 14) & 0x3fff) + 1;
  }
  return out;
}

// ── an independent AVIF container parse ───────────────────────────────────────
// The engine CANNOT check its own AVIF output: this build encodes AVIF and cannot
// decode it (DEC-065), so `info()` — the crate agreeing with itself, which would be
// weak proof anyway — is not even available. An outside reader is the only proof
// there is. This is one, written from the ISOBMFF/AVIF spec: the `ftyp` box's major
// brand, and the `ispe` (image spatial extents) box, which is where an AVIF states
// its true dimensions. (The browser's own decoder is the second opinion; `sips` is a
// third on macOS.)
function readAvif(buf) {
  const out = { ftyp: false, brand: null, width: null, height: null };
  if (buf.length < 16 || buf.subarray(4, 8).toString("ascii") !== "ftyp") return out;
  out.ftyp = true;
  out.brand = buf.subarray(8, 12).toString("ascii");

  // `ispe` sits nested several boxes deep (meta → iprp → ipco → ispe). Walking the
  // whole tree to reach it would be a decoder; finding the box header is enough to
  // read what the file says about itself.
  const at = buf.indexOf("ispe", 0, "ascii");
  if (at > 0) {
    out.width = buf.readUInt32BE(at + 8); // 4 B box type, 4 B version+flags, then w, h
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

// Every path the page asks the SERVER for. If the conversion ever went to a
// backend, it would show up here (and in the CDP network log below, which also
// sees requests that never reach us).
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
    // Chrome's sandbox needs kernel namespaces that many containerized CI runners
    // do not grant; without this it exits instantly there. Only in CI — a developer's
    // machine keeps the sandbox on, because there is no reason not to.
    ...(process.env.CI ? ["--no-sandbox"] : []),
    "about:blank",
  ],
  { stdio: ["ignore", "ignore", "pipe"] },
);
cleanups.push(() => chrome.kill());

// Chrome writes the port it actually took into DevToolsActivePort.
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

// ── attach to the engine's thread ─────────────────────────────────────────────
// The engine no longer runs in the page (SPEC-078): demo.js starts a module Worker
// and the Worker is what init()s the wasm. A Worker is a SEPARATE CDP TARGET, so
// from the page's session it is invisible — its console errors, and the .wasm fetch
// itself, simply do not appear. (That is not a theory: moving the conversions off
// the main thread turned the "the .wasm was served as application/wasm" check below
// into "no response at all", because the page never requested it again.)
//
// So auto-attach. `waitForDebuggerOnStart` pauses the worker at its first line, which
// is the only way to have Network enabled BEFORE it fetches the .wasm; nothing runs in
// there until `runIfWaitingForDebugger`. Everything the worker does — requests,
// responses, console errors, uncaught exceptions — now flows into the same lists as
// the page's, keyed by method.
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

// The whole point: this resolves only if instantiateStreaming succeeded — IN THE
// WORKER, which is where init() now happens.
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

// WAIT for the value; do not snapshot it. On a slow runner the navigation can
// commit twice — the poll above sees `ready` on the first document, and a bare
// read a moment later lands on the second one while its module is still booting,
// which reads back as an empty string. (CI caught exactly that; every check after
// it passed, because by then the second document had finished.) Waiting on the
// end state is correct regardless of how many documents got there: the page sets
// the version BEFORE it declares itself ready, so this can only converge.
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
  `the .wasm was served as application/wasm (got ${wasmResponse?.mimeType ?? "no response at all"}) ` +
    `— instantiateStreaming rejects anything else, which is why file:// cannot work`,
);

check(consoleErrors.length === 0, "no console errors while loading");
if (consoleErrors.length) console.error(`    ${consoleErrors.join("\n    ")}`);

// ── 4. convert: PNG in → WebP out ─────────────────────────────────────────────

console.log("\n── PNG in → WebP out, in the browser ──");

const fixtureDir = mkdtempSync(join(tmpdir(), "crustyimg-fixture-"));
cleanups.push(() => rmSync(fixtureDir, { recursive: true, force: true }));
const fixture = join(fixtureDir, "fixture.png");
writeFileSync(fixture, makePng(FIXTURE_W, FIXTURE_H));

// The real user path: hand the file to the page's <input type=file>, exactly as a
// picker or a drop would. Not a test hook, not a synthetic call into the module.
const { root } = await cdp.send("DOM.getDocument");
const { nodeId } = await cdp.send("DOM.querySelector", {
  nodeId: root.nodeId,
  selector: "#file",
});
const requestsBeforeConvert = requests.length;
await cdp.send("DOM.setFileInputFiles", { nodeId, files: [fixture] });

// Chrome fires `change` when the files are set; if a future Chrome stops doing so,
// nudge it rather than hanging for 30s on a protocol detail.
await sleep(300);
if ((await cdp.eval(PAGE_STATE)) === "ready") {
  await cdp.eval(
    "document.getElementById('file').dispatchEvent(new Event('change', { bubbles: true }))",
  );
}

await waitFor(cdp, `${PAGE_STATE} === 'done'`, "the conversion to finish");

const result = await cdp.eval(
  "JSON.stringify({ ...document.getElementById('result').dataset, " +
    "inDims: document.getElementById('in-dims').textContent, " +
    "inFormat: document.getElementById('in-format').textContent, " +
    "download: document.getElementById('download').getAttribute('download'), " +
    "href: document.getElementById('download').href })",
).then(JSON.parse);

check(
  result.inDims === `${FIXTURE_W}×${FIXTURE_H}` && result.inFormat === "png",
  `info(input) → ${result.inDims} ${result.inFormat} (expected ${FIXTURE_W}×${FIXTURE_H} png)`,
);
check(Number(result.outBytes) > 0, `the conversion produced ${result.outBytes} bytes`);
check(
  result.outFormat === "webp" &&
    Number(result.outWidth) === FIXTURE_W &&
    Number(result.outHeight) === FIXTURE_H,
  `the engine reads its own output back as ${result.outWidth}×${result.outHeight} ${result.outFormat}`,
);
check(
  result.href?.startsWith("blob:") && result.download === "fixture.webp",
  `a download is produced: <a download="${result.download}"> pointing at a blob: URL (no server round trip)`,
);

// The load-bearing decode: pull the download's actual BYTES out of the browser and
// parse them here, with a parser the crate had no hand in.
const outHex = await cdp.eval(`
  (async () => {
    const buf = await (await fetch(document.getElementById('download').href)).arrayBuffer();
    return [...new Uint8Array(buf.slice(0, 32))].map(b => b.toString(16).padStart(2, '0')).join('');
  })()
`);
const webp = readWebp(Buffer.from(outHex, "hex"));
check(
  webp.riff && webp.webp,
  `the downloaded bytes are a real RIFF/WEBP container (${webp.chunk} chunk — lossless)`,
);
check(
  webp.width === FIXTURE_W && webp.height === FIXTURE_H,
  `the downloaded bytes' own VP8L header says ${webp.width}×${webp.height} (independent parse; ` +
    `expected ${FIXTURE_W}×${FIXTURE_H})`,
);

// ── 5. it is client-side, and it is not lying about it ────────────────────────

console.log("\n── 100% client-side ──");

const duringConvert = requests
  .slice(requestsBeforeConvert)
  .filter((u) => !u.startsWith("blob:") && !u.startsWith("data:"));
check(
  duringConvert.length === 0,
  `the conversion made ZERO network requests (only the page's own blob: URL was read back)`,
);
if (duringConvert.length) console.error(`    ${duringConvert.join("\n    ")}`);

const offOrigin = requests.filter(
  (u) => !u.startsWith(baseUrl) && !u.startsWith("blob:") && !u.startsWith("data:"),
);
check(
  offOrigin.length === 0,
  `the page loaded nothing off-origin — no CDN, no font, no analytics (${requests.length} requests, all ${baseUrl})`,
);
if (offOrigin.length) console.error(`    ${offOrigin.join("\n    ")}`);
console.log(`    served: ${served.join(", ")}`);

// ── 6. the other output format, and the resize ────────────────────────────────

console.log("\n── PNG out, resized to a max long edge ──");

// Both controls, then one conversion — the same path a user takes, and the one
// that exercises `transform()` with a real recipe instead of `optimize()`.
await cdp.eval(`
  const fmt = document.getElementById('format');
  fmt.value = 'png';
  document.getElementById('maxedge').value = '${MAX_EDGE}';
  fmt.dispatchEvent(new Event('change', { bubbles: true }));
`);
await waitFor(cdp, `${PAGE_STATE} === 'done'`, "the resize conversion");

const resized = await cdp.eval(
  "JSON.stringify({ ...document.getElementById('result').dataset, " +
    "download: document.getElementById('download').getAttribute('download') })",
).then(JSON.parse);

// mode = "max" caps the LONG edge, so a 64×48 becomes 32×24.
const expectW = MAX_EDGE;
const expectH = Math.round((FIXTURE_H * MAX_EDGE) / FIXTURE_W);
check(
  resized.outFormat === "png" &&
    Number(resized.outWidth) === expectW &&
    Number(resized.outHeight) === expectH,
  `transform(recipe: resize max ${MAX_EDGE}) → ${resized.outWidth}×${resized.outHeight} ${resized.outFormat} ` +
    `(expected ${expectW}×${expectH} png)`,
);
check(resized.download === "fixture.png", `the download follows the format: ${resized.download}`);

const pngHex = await cdp.eval(`
  (async () => {
    const buf = await (await fetch(document.getElementById('download').href)).arrayBuffer();
    return [...new Uint8Array(buf.slice(0, 32))].map(b => b.toString(16).padStart(2, '0')).join('');
  })()
`);
const ihdr = readIhdr(Buffer.from(pngHex, "hex"));
check(ihdr.signature, "the downloaded bytes are a real PNG");
check(
  ihdr.width === expectW && ihdr.height === expectH,
  `the downloaded PNG's own IHDR says ${ihdr.width}×${ihdr.height} (independent parse; expected ` +
    `${expectW}×${expectH})`,
);

check(consoleErrors.length === 0, "still no console errors after two conversions");
if (consoleErrors.length) console.error(`    ${consoleErrors.join("\n    ")}`);

// ── 7. input reach: SVG + JPEG + GIF + WebP ───────────────────────────────────

console.log("\n── every input format the page claims ──");

// The SVG is the repo's hand-written fixture — a file no part of this engine
// produced. The raster fixtures below ARE minted by the engine (there is no second
// JPEG/GIF encoder here), so they prove the BROWSER path — File → arrayBuffer →
// decode — accepts each format, not that the codecs are correct; the codecs are the
// crate's own test suite's job.
const { initSync, transform: nodeTransform } = await import(
  join(demoDir, "vendor", "crustyimg.js")
);
initSync({ module: readFileSync(join(demoDir, "vendor", "crustyimg_bg.wasm")) });

const keep = 'version = "1"\n\n[[step]]\nop = "resize"\nmode = "exact"\nwidth = 64\nheight = 48\n';
const srcPng = makePng(FIXTURE_W, FIXTURE_H);
for (const fmt of ["jpeg", "gif", "webp"]) {
  writeFileSync(join(fixtureDir, `fixture.${fmt}`), nodeTransform(srcPng, keep, fmt));
}

// Reset the controls — no resize, WebP out — so each input is judged on its decode.
await cdp.eval(`
  document.getElementById('format').value = 'webp';
  document.getElementById('maxedge').value = '';
`);

/// Put a file in the picker and wait for the page to finish converting THAT file.
///
/// Waiting for `state === 'done'` is not enough here, and the difference is a real
/// bug that bit: by the time we drop the SECOND file, the page is ALREADY 'done'
/// from the first. The wait is then satisfied by the previous conversion's terminal
/// state, and we read the previous file's numbers — the check goes green or red on
/// the wrong image, one behind. (Forcing `state = 'ready'` first only narrows the
/// window; it does not close it, and it lost the race ~3 runs in 8.)
///
/// So don't wait on a level that is already true — wait on something ONLY the new
/// conversion can produce. `render()` overwrites `#result.dataset` wholesale, so a
/// unique token stamped there before the drop is proof of freshness when it is gone:
/// nothing but a real render of the new file can remove it.
let dropSeq = 0;
async function drop(path) {
  const token = `awaiting-conversion-${++dropSeq}`;
  await cdp.eval(`
    document.getElementById('result').dataset.outFormat = ${JSON.stringify(token)};
    document.body.dataset.state = 'ready';
  `);

  const doc = await cdp.send("DOM.getDocument");
  const { nodeId: input } = await cdp.send("DOM.querySelector", {
    nodeId: doc.root.nodeId,
    selector: "#file",
  });
  await cdp.send("DOM.setFileInputFiles", { nodeId: input, files: [path] });

  await waitFor(
    cdp,
    `${PAGE_STATE} === 'done' && ` +
      `document.getElementById('result')?.dataset.outFormat !== ${JSON.stringify(token)}`,
    `the conversion of ${path}`,
  );
  // Safe to snapshot now: render() writes the dataset and the readouts in one
  // synchronous pass, so the token's absence means all of them are this file's.
  return cdp
    .eval(
      "JSON.stringify({ ...document.getElementById('result').dataset, " +
        "inDims: document.getElementById('in-dims').textContent, " +
        "inFormat: document.getElementById('in-format').textContent })",
    )
    .then(JSON.parse);
}

// A rasterized SVG reports `png` — it HAS become a lossless raster (SPEC-060's
// materialized-raster convention). The fixture is 40×30 in its own viewBox.
const svg = await drop(join(repoRoot, "tests", "fixtures", "svg", "rect_text_40x30.svg"));
check(
  svg.inDims === "40×30" && svg.inFormat === "png",
  `SVG in → rasterized to ${svg.inDims} (reported as ${svg.inFormat}: it is a raster now) → ` +
    `${svg.outWidth}×${svg.outHeight} ${svg.outFormat} out`,
);
check(Number(svg.outBytes) > 0, `SVG → WebP produced ${svg.outBytes} bytes in the browser`);

for (const fmt of ["jpeg", "gif", "webp"]) {
  const r = await drop(join(fixtureDir, `fixture.${fmt}`));
  check(
    r.inFormat === fmt && r.inDims === `${FIXTURE_W}×${FIXTURE_H}`,
    `${fmt.toUpperCase()} in → info() reads ${r.inDims} ${r.inFormat} → ${r.outWidth}×${r.outHeight} ` +
      `${r.outFormat} out (${r.outBytes} B)`,
  );
}

// ── 8. AVIF out — the headline, and the reason the worker exists ──────────────

console.log("\n── PNG → AVIF, while the page keeps running ──");

// A BIG fixture on purpose. rav1e is serial and slow, and this test's whole claim is
// about what happens DURING a slow encode — a 64×48 thumbnail would be over before
// there was anything to observe. (The encode is also the demo's headline: it is the
// only thing here that a browser cannot already do for you.)
const bigPng = join(fixtureDir, "big.png");
writeFileSync(bigPng, makePng(AVIF_W, AVIF_H));

// THE PROBE. Two independent main-thread heartbeats — a timer and a rAF frame
// callback — each counting how many times it ran while the page was in the
// `converting` state. If the conversion were still a synchronous wasm call on the
// page's thread, NEITHER could run during it: the thread would be inside rav1e, and
// by the time it came back the state would already be `done`. So a non-zero count
// here is not a smell test, it is the difference between the two architectures.
await cdp.eval(`
  window.__probe = { ticks: 0, duringConvert: 0, frames: 0, framesDuringConvert: 0 };
  const converting = () => document.body.dataset.state === 'converting';
  setInterval(() => {
    window.__probe.ticks++;
    if (converting()) window.__probe.duringConvert++;
  }, 20);
  (function frame() {
    window.__probe.frames++;
    if (converting()) window.__probe.framesDuringConvert++;
    requestAnimationFrame(frame);
  })();
  document.getElementById('format').value = 'avif';
  document.getElementById('maxedge').value = '';
`);

const requestsBeforeAvif = requests.length;
const avifOut = await drop(bigPng);
const probe = await cdp.eval("JSON.stringify(window.__probe)").then(JSON.parse);
const encodeMs = Number(avifOut.elapsedMs);

check(
  avifOut.outFormat === "avif" &&
    Number(avifOut.outWidth) === AVIF_W &&
    Number(avifOut.outHeight) === AVIF_H,
  `PNG → AVIF: ${avifOut.outWidth}×${avifOut.outHeight} ${avifOut.outFormat}, ${avifOut.outBytes} B ` +
    `(expected ${AVIF_W}×${AVIF_H} avif) — the option SPEC-077 had to disable`,
);

// A window worth measuring: if rav1e finished this in 50 ms, the responsiveness
// claim below would be true but untested. (It does not; it takes seconds.)
check(
  encodeMs >= MIN_ENCODE_MS,
  `the AVIF encode took ${encodeMs} ms — a real blocking window (>= ${MIN_ENCODE_MS} ms), long ` +
    `enough that a main-thread encode would have frozen the tab for it`,
);
check(
  probe.duringConvert >= 5 && probe.framesDuringConvert >= 5,
  `THE MAIN THREAD STAYED ALIVE THROUGH IT — ${probe.duringConvert} timer callbacks and ` +
    `${probe.framesDuringConvert} animation frames ran DURING the ${encodeMs} ms encode ` +
    `(a main-thread encode blocks both: the page cannot paint the spinner it is showing)`,
);

// THE NEGATIVE CONTROL — and it is not optional. A probe that cannot detect a frozen
// main thread would report "responsive" for a page that is not, and the check above
// would pass for the wrong reason (this wave has already shipped one green wait that
// was waiting for nothing). So freeze the main thread ON PURPOSE, in the same state
// the page shows while converting, and confirm the probe goes to ZERO — which is what
// it would have read all along if the engine had stayed on this thread.
const control = await cdp
  .eval(
    `(async () => {
      const before = { ...window.__probe };
      document.body.dataset.state = 'converting';
      const until = performance.now() + ${BLOCK_MS};
      while (performance.now() < until) {} // a synchronous wasm call, imitated exactly
      const ticks = window.__probe.duringConvert - before.duringConvert;
      const frames = window.__probe.framesDuringConvert - before.framesDuringConvert;
      document.body.dataset.state = 'done'; // still synchronous: nothing queued has run yet
      return JSON.stringify({ ticks, frames });
    })()`,
  )
  .then(JSON.parse);
check(
  control.ticks === 0 && control.frames === 0,
  `and the probe CAN see a freeze: a deliberate ${BLOCK_MS} ms main-thread block ran ` +
    `${control.ticks} timers / ${control.frames} frames — so the counts above are evidence, not noise`,
);

// The independent decode, twice over. `readAvif` below is a container parse we wrote
// from the ISOBMFF spec — the crate had no hand in it — and `createImageBitmap` is
// Chrome's own libavif, a decoder from a different project in a different language.
// (The engine CANNOT check this one itself: it encodes AVIF and cannot decode it,
// DEC-065. That asymmetry is exactly why an outside opinion is the only proof here.)
const avifBytes = await downloadBytes(cdp);
const box = readAvif(avifBytes);
check(
  box.ftyp && box.brand === "avif",
  `the downloaded bytes are a real AVIF container (ftyp brand "${box.brand}", ${avifBytes.length} B ` +
    `— parsed here, by a reader the crate never touched)`,
);
check(
  box.width === AVIF_W && box.height === AVIF_H,
  `the AVIF's own ispe box says ${box.width}×${box.height} (independent parse; expected ` +
    `${AVIF_W}×${AVIF_H})`,
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
  chromeDecode.w === AVIF_W && chromeDecode.h === AVIF_H,
  `Chrome's own AVIF decoder reads the bytes back as ${chromeDecode.w}×${chromeDecode.h} — an ` +
    `independent decoder, in another language, agreeing this is the image`,
);

// And a third, when the platform has one: macOS's `sips` is a decoder from neither
// this crate nor this browser. Skipped elsewhere (CI is Linux) rather than faked.
if (process.platform === "darwin") {
  const avifFile = join(fixtureDir, "out.avif");
  writeFileSync(avifFile, avifBytes);
  const sips = spawnSync("sips", ["-g", "format", "-g", "pixelWidth", "-g", "pixelHeight", avifFile], {
    encoding: "utf8",
  });
  const said = `${sips.stdout ?? ""}`.replace(/\s+/g, " ").trim();
  check(
    sips.status === 0 &&
      /format:\s*avif/i.test(said) &&
      new RegExp(`pixelWidth: ${AVIF_W}\\b`).test(said) &&
      new RegExp(`pixelHeight: ${AVIF_H}\\b`).test(said),
    `macOS \`sips\` — a third decoder, from neither the crate nor the browser — reads it as ` +
      `${AVIF_W}×${AVIF_H} AVIF ("${said}")`,
  );
} else {
  console.log(`  · (sips is macOS-only — skipped on ${process.platform}; two decoders agreed above)`);
}

const duringAvif = requests
  .slice(requestsBeforeAvif)
  .filter((u) => !u.startsWith("blob:") && !u.startsWith("data:"));
check(
  duringAvif.length === 0,
  `the AVIF encode made ZERO network requests — the worker's traffic is in this log too, so a ` +
    `conversion that phoned home from the worker could not hide here`,
);
if (duringAvif.length) console.error(`    ${duringAvif.join("\n    ")}`);

// ── 9. AVIF in — the browser decodes what the engine cannot ───────────────────

console.log("\n── .avif in (the browser decodes what the engine cannot) ──");

// The engine has no AVIF decoder (DEC-065) — an `.avif` input used to be a typed
// error on this page. The worker now hands it to `createImageBitmap`, draws it on an
// OffscreenCanvas, and gives the engine lossless PNG pixels. The fixture is the
// repo's own 16×16 AVIF, which nothing in this build could read a week ago.
await cdp.eval(`
  document.getElementById('format').value = 'webp';
  document.getElementById('maxedge').value = '';
`);
const fromAvif = await drop(join(repoRoot, "tests", "fixtures", "avif", "solid_16x16.avif"));

check(
  fromAvif.inFormat === "avif" && fromAvif.inDims === "16×16",
  `an .avif INPUT is read: ${fromAvif.inDims} ${fromAvif.inFormat} (the page says avif, not "png" — ` +
    `it does not pretend the browser's PNG hand-off is the file you dropped)`,
);
check(
  fromAvif.outFormat === "webp" &&
    Number(fromAvif.outWidth) === 16 &&
    Number(fromAvif.outHeight) === 16 &&
    Number(fromAvif.outBytes) > 0,
  `.avif → WebP: ${fromAvif.outWidth}×${fromAvif.outHeight} ${fromAvif.outFormat}, ` +
    `${fromAvif.outBytes} B — the dimensions survived the createImageBitmap → canvas → engine path`,
);

const whereText = await cdp.eval("document.getElementById('ex-where').textContent");
check(
  /browser decoded the AVIF input/i.test(whereText),
  `and the page SAYS whose decoder that was, rather than quietly implying the engine did it`,
);

// ── 10. the decision, on the page ─────────────────────────────────────────────

console.log("\n── the readout shows the decision ──");

const readout = await cdp
  .eval(
    "JSON.stringify({ ...document.getElementById('result').dataset, " +
      "inBytes: document.getElementById('in-bytes').textContent, " +
      "outBytes: document.getElementById('out-bytes').textContent, " +
      "delta: document.getElementById('delta').textContent, " +
      "exFormat: document.getElementById('ex-format').textContent, " +
      "exSize: document.getElementById('ex-size').textContent, " +
      "exDims: document.getElementById('ex-dims').textContent, " +
      "exQuality: document.getElementById('ex-quality').textContent })",
  )
  .then(JSON.parse);

check(
  /\d+ (B|KB|MB)/.test(readout.inBytes) && /\d+(\.\d+)? (B|KB|MB)/.test(readout.outBytes),
  `the DOM shows the bytes in and out — "${readout.inBytes}" → "${readout.outBytes}"`,
);
check(
  /\d+% (smaller|bigger)/.test(readout.delta) &&
    Number.isInteger(Number(readout.savedPct)) &&
    /\d+% (saved|larger)/.test(readout.exSize),
  `and the SAVING, as a percentage — "${readout.delta.trim()}" (data-saved-pct="${readout.savedPct}")`,
);
check(
  readout.exFormat.includes(`${readout.inFormat} → ${readout.outFormat}`) &&
    readout.exFormat.includes("you chose the format"),
  `and the format it chose, and WHO chose it — "${readout.exFormat}"`,
);
check(
  readout.exDims.includes(`${readout.outWidth}×${readout.outHeight}`),
  `and the dimensions — "${readout.exDims}"`,
);
check(
  /lossless/i.test(readout.exQuality),
  `and how the quality was decided, honestly (WebP here is LOSSLESS — this build has no lossy ` +
    `WebP) — "${readout.exQuality.slice(0, 60)}…"`,
);

// ── 11. Auto: let the engine choose ───────────────────────────────────────────

console.log("\n── Auto — the engine picks the format ──");

// The other half of "intent": the user says what they want, not how. `optimize(bytes,
// "auto")` runs the engine's own analysis + format shortlist (the same code the CLI's
// `optimize` runs) inside the wasm, and whatever it picks is what the page reports.
await cdp.eval("document.getElementById('format').value = 'auto';");
const auto = await drop(fixture);

check(
  ["avif", "webp", "png", "jpeg"].includes(auto.outFormat) && Number(auto.outBytes) > 0,
  `Auto → the engine chose ${auto.outFormat} for a ${FIXTURE_W}×${FIXTURE_H} gradient PNG ` +
    `(${auto.outBytes} B) — its own analysis + format shortlist, running in the browser`,
);
const autoSays = await cdp.eval("document.getElementById('ex-format').textContent");
check(
  autoSays.includes("the engine chose the format"),
  `and the readout attributes the choice to the engine — "${autoSays}"`,
);

check(consoleErrors.length === 0, "no console errors in the page OR the worker, after all of it");
if (consoleErrors.length) console.error(`    ${consoleErrors.join("\n    ")}`);

// ── 12. and the reason the server exists ──────────────────────────────────────

console.log("\n── the file:// failure mode is real (which is WHY we serve) ──");

// The whole recipe/deploy story rests on "this page cannot be opened from the
// filesystem". That claim is load-bearing enough to test rather than repeat: open
// the very same index.html as a file:// URL and confirm it (a) cannot run, and
// (b) says so instead of hanging on a spinner.
//
// The failure is EARLIER than "instantiateStreaming rejects the MIME type": an ES
// module script is fetched under CORS, and file:// is an opaque origin, so demo.js
// is blocked before it executes at all — which is why index.html carries a classic
// script whose only job is to explain this.
//
// It needs its OWN tab: Chrome refuses to navigate an http:// page to a file:// URL,
// so reusing the tab above would silently test nothing. Target.createTarget opens the
// file:// URL as a top-level navigation, the way double-clicking the file does.
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
// `#drop` is unhidden only by demo.js, and only after init() resolves.
const moduleRan = await fileCdp.eval("!!document.querySelector('#drop:not([hidden])')");

check(
  !moduleRan,
  "over file://, the page cannot convert anything — the module script is CORS-blocked before it runs",
);
// Assert WHO is speaking, not just that someone is. demo.js's own catch block also
// mentions serving over HTTP, so a message merely matching /served over http/i would
// be satisfied by the module having run and failed at instantiateStreaming — exactly
// the mechanism this test claims is impossible. The classic script's wording is
// distinct ("will not load its code"); demo.js's catch always opens with "Could not
// load the WebAssembly engine". Requiring the former and forbidding the latter pins
// the real failure: the module never executed, and the non-module script is what
// survived to explain it.
check(
  fileState === "error" &&
    /will not load its code/i.test(fileStatus) &&
    !/Could not load the WebAssembly engine/i.test(fileStatus),
  `over file://, it is the CLASSIC script that speaks — demo.js never ran, so its catch never ` +
    `fired — and the page SAYS why instead of hanging on "Loading…" — "${fileStatus.trim().slice(0, 52)}…"`,
);

await cleanup();

console.log("");
if (failed) {
  console.error(`demo-smoke: ${failed} check(s) FAILED\n`);
  process.exit(1);
}
console.log(
  `demo-smoke: the demo loads crustyimg ${crateVersion} in a real browser over HTTP and converts ` +
    `images client-side. ✓\n`,
);
