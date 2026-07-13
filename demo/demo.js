// The demo page (SPEC-077, SPEC-078). The whole app: hand a dropped image to the
// engine, show what it decided, offer the download. Nothing is uploaded anywhere.
//
// THIS FILE NO LONGER TOUCHES THE ENGINE. Every conversion runs in demo/worker.js —
// a module Web Worker — and this file only posts bytes to it and renders what comes
// back (SPEC-078). That is not tidiness: the AVIF encoder (rav1e) is serial and takes
// seconds, and a wasm call is synchronous, so converting HERE would freeze the tab for
// the whole encode — which is why SPEC-077 shipped with AVIF disabled. In the worker
// it freezes nothing: the page keeps painting, the spinner spins, the controls stay
// alive. It is a separate THREAD, not shared memory — no SharedArrayBuffer, no wasm
// threads, and therefore no COOP/COEP headers (which GitHub Pages could not set).
//
// HOW THE ENGINE LOADS (DEC-067). `crustyimg-wasm` is built `--target web`, so
// `init()` is EXPLICIT — importing the module does not fetch the 1.3 MB `.wasm`;
// awaiting `init()` does. With no argument it resolves the binary as
// `new URL('crustyimg_bg.wasm', import.meta.url)` and hands it to
// `WebAssembly.instantiateStreaming`, which requires the server to send it as
// `application/wasm`. The worker does that now, but the rule is unchanged.
//
// ⚠ THAT IS WHY THIS PAGE MUST BE SERVED OVER HTTP. `just demo-serve` serves it
// locally with the right MIME; GitHub Pages does too. See demo/README.md.
//
// What actually happens on `file://` is worse than a failed instantiation, and
// worth stating precisely because it is silent (measured in headless Chrome, not
// assumed): THIS FILE NEVER RUNS. An ES module script is fetched under CORS, and a
// file:// origin is opaque, so the browser blocks the module before a single line of
// it executes — no import, no worker, no error anywhere. The page just sits on
// "Loading the engine…" forever. The `instantiateStreaming` MIME problem is real and
// would bite next, but it never gets that far. The classic (non-module) script in
// index.html is what survives to explain this, and the browser smoke asserts the
// whole failure mode rather than trusting this comment.

const el = (id) => document.getElementById(id);

const ui = {
  status: el("status"),
  drop: el("drop"),
  file: el("file"),
  controls: el("controls"),
  format: el("format"),
  maxEdge: el("maxedge"),
  error: el("error"),
  busy: el("busy"),
  result: el("result"),
  delta: el("delta"),
  download: el("download"),
  version: el("version"),
};

// The page's state, mirrored onto <body data-state> — the CSS dresses it (that is
// where the busy state comes from), the browser smoke waits on it, and it is the
// honest thing to show a user anyway.
//
// The busy state is a SPINNER, not a percentage. A conversion is one blocking call
// inside the worker (rav1e reports nothing as it goes), so a progress bar here would
// be a lie about information we do not have. What we can honestly show — and what
// actually matters — is that the page is alive while it happens.
function setState(state) {
  document.body.dataset.state = state;
  const busy = state === "converting";
  for (const control of [ui.file, ui.format, ui.maxEdge]) control.disabled = busy;
  ui.busy.hidden = !busy;
}

setState("loading");

let source = null; // the current input: { file, url } — kept so a control change re-converts
let outUrl = null; // the previous blob: URL, revoked on the next conversion
let jobSeq = 0;

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

// ── the engine's thread ──────────────────────────────────────────────────────

// A MODULE worker: it `import`s the same vendored package, and `import.meta.url`
// inside it resolves the .wasm sitting next to it. `new URL(…, import.meta.url)`
// rather than a bare "./worker.js" so the path is right whatever the page's own URL
// is (a subdirectory on Pages, say).
const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });

const pending = new Map(); // job id → { resolve, reject }

worker.addEventListener("message", (ev) => {
  const m = ev.data;

  if (m.type === "ready") {
    ui.version.textContent = `crustyimg ${m.version}`;
    ui.status.textContent =
      "Engine loaded — everything below runs on your machine, in a background thread.";
    ui.status.className = "status status-ready";
    ui.drop.hidden = false;
    ui.controls.hidden = false;
    setState("ready");
    return;
  }

  if (m.type === "failed") {
    // The failure this page exists to prove doesn't happen: the engine could not be
    // instantiated. Name the likely cause rather than showing a bare stack.
    ui.status.className = "status status-error";
    ui.status.textContent =
      `Could not load the WebAssembly engine: ${m.message}. ` +
      "The server must send crustyimg_bg.wasm as application/wasm.";
    setState("error");
    return;
  }

  const job = pending.get(m.id);
  pending.delete(m.id);
  if (!job) return; // superseded by a newer conversion — this result is stale
  if (m.type === "error") job.reject(new Error(m.message));
  else job.resolve(m);
});

worker.addEventListener("error", (e) => {
  ui.status.className = "status status-error";
  ui.status.textContent = `The engine's worker could not start: ${e.message ?? "unknown error"}.`;
  setState("error");
});

/// Send one conversion to the worker. The bytes are TRANSFERRED, not copied — they
/// leave this thread's heap — so they are re-read from the `File` for each job.
function ask(bytes, format, maxEdge) {
  const id = ++jobSeq;
  // Anything still in flight is now stale: its result would overwrite this one's.
  // Forget it. (The worker will still finish it — a wasm call cannot be interrupted
  // — but nobody is listening, and the page is not blocked either way.)
  pending.clear();
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, bytes: bytes.buffer, format, maxEdge }, [bytes.buffer]);
  });
}

// ── input ────────────────────────────────────────────────────────────────────

async function load(file) {
  clearError();
  if (source?.url) URL.revokeObjectURL(source.url);
  source = { file, url: URL.createObjectURL(file) };
  await convert();
}

// ── convert ──────────────────────────────────────────────────────────────────

async function convert() {
  if (!source) return;
  clearError();
  setState("converting");

  const maxEdge = Number.parseInt(ui.maxEdge.value, 10);
  const bytes = new Uint8Array(await source.file.arrayBuffer());

  let result;
  try {
    result = await ask(bytes, ui.format.value, Number.isInteger(maxEdge) ? maxEdge : null);
  } catch (e) {
    showError(`Can't convert ${source.file.name}: ${e.message ?? e}`);
    return;
  }

  render(result);
}

// ── the decision, shown ──────────────────────────────────────────────────────

function render(m) {
  const { input, output, elapsedMs, quality } = m;

  el("in-img").src = source.url;
  el("in-dims").textContent = `${input.width}×${input.height}`;
  el("in-format").textContent = input.format;
  el("in-bytes").textContent = fmtBytes(input.bytes);

  if (outUrl) URL.revokeObjectURL(outUrl);
  const blob = new Blob([m.out], { type: `image/${output.format}` });
  outUrl = URL.createObjectURL(blob);

  el("out-img").src = outUrl;
  el("out-dims").textContent = `${output.width}×${output.height}`;
  el("out-format").textContent = output.format;
  el("out-bytes").textContent = fmtBytes(output.bytes);

  const stem = source.file.name.replace(/\.[^.]+$/, "");
  ui.download.href = outUrl;
  ui.download.download = `${stem}.${output.format}`;
  ui.download.textContent = `Download ${stem}.${output.format} (${fmtBytes(output.bytes)})`;

  // The headline number — and its SIGN is the honest part. A lossless output (this
  // build's WebP and PNG) can legitimately come out bigger than an already-lossy
  // source. Say so instead of hiding it, and now that AVIF works, point at it.
  const saved = Math.round((1 - output.bytes / input.bytes) * 100);
  const shrank = output.bytes < input.bytes;
  ui.delta.className = shrank ? "delta smaller" : "delta bigger";
  ui.delta.textContent = shrank
    ? `${saved}% smaller — ${fmtBytes(input.bytes)} → ${fmtBytes(output.bytes)}`
    : `${Math.abs(saved)}% bigger — ${fmtBytes(input.bytes)} → ${fmtBytes(output.bytes)}` +
      (quality.mode === "lossless"
        ? " · lossless output can't beat an already-lossy source — try AVIF, or cap the long edge"
        : "");

  // WHY it came out that way: four lines, each of them something the engine actually
  // did — not a summary written to flatter it.
  el("ex-format").textContent =
    `${input.format} → ${output.format} ` +
    (ui.format.value === "auto" ? "(the engine chose the format)" : "(you chose the format)");
  el("ex-size").textContent =
    `${fmtBytes(input.bytes)} → ${fmtBytes(output.bytes)} · ` +
    (shrank ? `${saved}% saved` : `${Math.abs(saved)}% larger`);
  el("ex-dims").textContent =
    input.width === output.width && input.height === output.height
      ? `${output.width}×${output.height} (unchanged)`
      : `${input.width}×${input.height} → ${output.width}×${output.height}`;
  el("ex-quality").textContent = quality.note;
  el("ex-where").textContent =
    `${elapsedMs} ms in a Web Worker — off the page's thread, on your machine, no upload. ` +
    (input.decodedBy === "browser"
      ? "Your browser decoded the AVIF input (this build encodes AVIF but cannot decode it) and the engine took it from there. "
      : "") +
    (output.readBy === "browser"
      ? "Your browser's decoder read the output's dimensions back — an independent check that these bytes really are the image."
      : "The engine decoded its own output to check it.");

  // The smoke reads these: what the engine reported, straight off the DOM.
  ui.result.dataset.inBytes = String(input.bytes);
  ui.result.dataset.inFormat = input.format;
  ui.result.dataset.outBytes = String(output.bytes);
  ui.result.dataset.outWidth = String(output.width);
  ui.result.dataset.outHeight = String(output.height);
  ui.result.dataset.outFormat = output.format;
  ui.result.dataset.savedPct = String(saved);
  ui.result.dataset.elapsedMs = String(elapsedMs);
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
