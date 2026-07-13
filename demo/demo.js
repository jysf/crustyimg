// The demo page (SPEC-077). The whole app: load the wasm engine, convert a
// dropped image with it, show before/after, offer the download.
//
// HOW THE ENGINE LOADS (DEC-067). `crustyimg-wasm` is built `--target web`, so
// `init()` is EXPLICIT — importing the module does not fetch the 1.3 MB `.wasm`;
// awaiting `init()` does. With no argument it resolves the binary as
// `new URL('crustyimg_bg.wasm', import.meta.url)` and hands it to
// `WebAssembly.instantiateStreaming`, which requires the server to send it as
// `application/wasm`.
//
// ⚠ THAT IS WHY THIS PAGE MUST BE SERVED OVER HTTP. `just demo-serve` serves it
// locally with the right MIME; GitHub Pages does too. See demo/README.md.
//
// What actually happens on `file://` is worse than a failed instantiation, and
// worth stating precisely because it is silent (measured in headless Chrome, not
// assumed): THIS FILE NEVER RUNS. An ES module script is fetched under CORS, and a
// file:// origin is opaque, so the browser blocks the module before a single line
// of it executes — no import, no `init()`, no `catch` below, no error anywhere. The
// page just sits on "Loading the engine…" forever. The `instantiateStreaming` MIME
// problem is real and would bite next, but it never gets that far. The classic
// (non-module) script in index.html is what survives to explain this, and the
// browser smoke asserts the whole failure mode rather than trusting this comment.
//
// SINGLE-THREADED, ON THE MAIN THREAD. Every conversion below is a synchronous
// wasm call, so it must be a FAST one: lossless WebP and PNG. AVIF encode is
// rav1e — seconds of serial work that would freeze the tab — so it is disabled
// here and lands in SPEC-078 behind a Web Worker.

import init, { info, optimize, transform, version } from "./vendor/crustyimg.js";

const el = (id) => document.getElementById(id);

const ui = {
  status: el("status"),
  drop: el("drop"),
  file: el("file"),
  controls: el("controls"),
  format: el("format"),
  maxEdge: el("maxedge"),
  error: el("error"),
  result: el("result"),
  delta: el("delta"),
  download: el("download"),
  version: el("version"),
};

// The page's state, mirrored onto <body data-state> — the browser smoke waits on
// it, and it is the honest thing to show a user anyway.
function setState(state) {
  document.body.dataset.state = state;
}

setState("loading");

// The current input, kept so a control change re-converts without a re-drop.
let source = null; // { bytes, name, meta, url }
let outUrl = null; // the previous blob: URL, revoked on the next conversion

function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

function showError(message) {
  ui.error.textContent = message;
  ui.error.hidden = false;
  ui.result.hidden = true;
  setState("error");
}

function clearError() {
  ui.error.hidden = true;
  ui.error.textContent = "";
}

// ── boot ─────────────────────────────────────────────────────────────────────

try {
  await init();
  ui.version.textContent = `crustyimg ${version()}`;
  ui.status.textContent = "Engine loaded — everything below runs on your machine.";
  ui.status.className = "status status-ready";
  ui.drop.hidden = false;
  ui.controls.hidden = false;
  setState("ready");
} catch (e) {
  // The failure this page exists to prove doesn't happen. Name the likely cause
  // rather than showing a bare stack: `file://` is what bites people.
  ui.status.className = "status status-error";
  ui.status.textContent =
    `Could not load the WebAssembly engine: ${e.message ?? e}. ` +
    (location.protocol === "file:"
      ? "This page is open as a file:// URL — it must be served over HTTP (try `just demo-serve`)."
      : "The server must send crustyimg_bg.wasm as application/wasm.");
  setState("error");
  throw e;
}

// ── input ────────────────────────────────────────────────────────────────────

async function load(file) {
  clearError();
  const bytes = new Uint8Array(await file.arrayBuffer());

  let meta;
  try {
    meta = info(bytes);
  } catch (e) {
    // A typed error from the engine (an unreadable file, or an AVIF input — the
    // wasm build encodes AVIF but cannot decode it, DEC-065).
    showError(`Can't read ${file.name}: ${e.message ?? e}`);
    return;
  }

  source = { bytes, name: file.name, meta, url: URL.createObjectURL(file) };
  convert();
}

// ── convert ──────────────────────────────────────────────────────────────────

function convert() {
  if (!source) return;
  clearError();
  setState("converting");

  const format = ui.format.value;
  const maxEdge = Number.parseInt(ui.maxEdge.value, 10);

  // Two engine entry points, and the choice is the user's intent:
  //   * a max-edge cap → `transform` with a real recipe — the SAME recipe TOML the
  //     CLI reads from disk, replayed byte-for-byte in the browser;
  //   * no cap → `optimize`, which re-encodes as well as this build can.
  // `optimize` is given an EXPLICIT format on purpose: with "auto" the engine may
  // shortlist AVIF, and AVIF encode on the main thread would freeze the tab.
  const recipe =
    Number.isInteger(maxEdge) && maxEdge > 0
      ? `version = "1"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = ${maxEdge}\n`
      : null;

  let out;
  const started = performance.now();
  try {
    out = recipe
      ? transform(source.bytes, recipe, format)
      : optimize(source.bytes, format);
  } catch (e) {
    showError(`Conversion failed: ${e.message ?? e}`);
    return;
  }
  const elapsed = Math.round(performance.now() - started);

  // Ask the engine what it actually produced rather than trusting what we asked
  // for — the same decode a consumer of the file would do.
  let outMeta;
  try {
    outMeta = info(out);
  } catch (e) {
    showError(`The engine produced bytes it can't read back: ${e.message ?? e}`);
    return;
  }

  render(out, outMeta, elapsed);
}

function render(out, outMeta, elapsed) {
  const { bytes, meta, name, url } = source;

  el("in-img").src = url;
  el("in-dims").textContent = `${meta.width}×${meta.height}`;
  el("in-format").textContent = meta.format;
  el("in-bytes").textContent = fmtBytes(bytes.length);

  if (outUrl) URL.revokeObjectURL(outUrl);
  const format = ui.format.value;
  const blob = new Blob([out], { type: `image/${format}` });
  outUrl = URL.createObjectURL(blob);

  el("out-img").src = outUrl;
  el("out-dims").textContent = `${outMeta.width}×${outMeta.height}`;
  el("out-format").textContent = outMeta.format;
  el("out-bytes").textContent = fmtBytes(out.length);

  const stem = name.replace(/\.[^.]+$/, "");
  ui.download.href = outUrl;
  ui.download.download = `${stem}.${format}`;
  ui.download.textContent = `Download ${stem}.${format} (${fmtBytes(out.length)})`;

  const ratio = out.length / bytes.length;
  const pct = Math.abs(Math.round((1 - ratio) * 100));
  if (out.length < bytes.length) {
    ui.delta.className = "delta smaller";
    ui.delta.textContent = `${pct}% smaller — ${fmtBytes(bytes.length)} → ${fmtBytes(out.length)}, converted in ${elapsed} ms.`;
  } else {
    // Honesty over flattery: this build's WebP is LOSSLESS (lossy WebP is a C
    // library, and this is the pure-Rust engine), so re-encoding an already-lossy
    // JPEG can legitimately grow it. Say that instead of hiding it.
    ui.delta.className = "delta bigger";
    ui.delta.textContent =
      `${pct}% bigger — ${fmtBytes(bytes.length)} → ${fmtBytes(out.length)}, converted in ${elapsed} ms. ` +
      `Lossless output can't beat an already-lossy source; try a max long edge, or wait for AVIF.`;
  }

  // The smoke reads these: what the engine reported, straight off the DOM.
  ui.result.dataset.outBytes = String(out.length);
  ui.result.dataset.outWidth = String(outMeta.width);
  ui.result.dataset.outHeight = String(outMeta.height);
  ui.result.dataset.outFormat = outMeta.format;
  ui.result.hidden = false;
  setState("done");
}

// ── wiring ───────────────────────────────────────────────────────────────────

ui.file.addEventListener("change", () => {
  const file = ui.file.files?.[0];
  if (file) load(file);
});

for (const event of ["dragenter", "dragover"]) {
  ui.drop.addEventListener(event, (e) => {
    e.preventDefault();
    ui.drop.classList.add("hot");
  });
}

for (const event of ["dragleave", "drop"]) {
  ui.drop.addEventListener(event, (e) => {
    e.preventDefault();
    ui.drop.classList.remove("hot");
  });
}

ui.drop.addEventListener("drop", (e) => {
  const file = e.dataTransfer?.files?.[0];
  if (file) load(file);
});

ui.format.addEventListener("change", convert);
ui.maxEdge.addEventListener("change", convert);
