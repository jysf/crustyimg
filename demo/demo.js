// The demo page (SPEC-077, SPEC-078; reframed by SPEC-080). The whole app: hand a
// dropped image to the engine, make it web-ready, show what it decided, and turn the
// result into an invitation to run the real CLI. Nothing is uploaded anywhere.
//
// THE HERO IS ONE PATH. Drop a photo and the demo runs the `web` flow automatically
// (SPEC-085 / DEC-070): downscale the long edge to 2048, modernize to AVIF (photos)
// or lossless WebP (graphics), never hand back a bigger file, and score the result.
// No controls to touch. Everything that used to be a front-and-centre knob lives
// behind the Advanced <details>, collapsed — and only the Advanced "keep full
// resolution" path is slow enough to earn the megapixel warning + elapsed timer.
//
// THIS FILE NO LONGER TOUCHES THE ENGINE. Every conversion runs in demo/worker.js —
// a module Web Worker — and this file only posts bytes to it and renders what comes
// back (SPEC-078). The AVIF encoder (rav1e) is serial and takes seconds, and a wasm
// call is synchronous, so converting HERE would freeze the tab for the whole encode.
// In the worker it freezes nothing: the page keeps painting, the timer keeps
// counting, the controls stay alive. It is a separate THREAD, not shared memory — no
// SharedArrayBuffer, no wasm threads, and therefore no COOP/COEP headers.
//
// ⚠ THE PAGE MUST BE SERVED OVER HTTP. On `file://` this ES module is CORS-blocked
// and never runs at all — no import, no init(), no error — so index.html carries a
// classic script that survives to explain it. See demo/README.md; the browser smoke
// asserts that whole failure mode rather than trusting this comment.

// The `web` recipe, VERBATIM from recipes/web.toml (DEC-070). Inlined rather than
// fetched so the funnel needs zero network requests (the SPEC-077 guarantee), and
// the demo smoke asserts this string is byte-identical to the file on disk — so it
// cannot silently drift from the recipe the CLI actually runs.
const WEB_RECIPE = `# The \`web\` flagship flow (SPEC-085), shipped in-binary so \`web <inputs>\` and
# \`apply --recipe web <inputs>\` reach the identical engine.
#
# Downscale the long edge to a web-friendly size (never upscaling), bake EXIF
# orientation + strip metadata, then let the fast AVIF-aware decision (SPEC-084,
# \`Mode::Fast\`) pick the smallest modern format that beats the DOWNSCALED image,
# and score the winner (SSIMULACRA2) on that output. The downscale to a dimension
# bound is the contract, so an already-small source ABOVE that bound can re-encode
# LARGER than the original — reported honestly as "N% larger" and flagged
# (\`larger_than_source\`), never hidden (SPEC-090, DEC-075). For an unconditional
# never-bigger guarantee that keeps dimensions, use \`optimize\`. The terminal
# \`optimize\` step is what invokes that decision instead of a plain
# format-preserving write.
version = "1"
name = "web"
description = "Downscale to 2048px, modernize (AVIF/lossless-WebP), and score (size reported honestly)."

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 2048

[[step]]
op = "optimize"
`;

const DEFAULT_MAX_EDGE = 2048; // the `web` long-edge target
const SLOW_MP = 4; // encoded megapixels at/above which we warn + show the elapsed timer
const DEBOUNCE_MS = 300; // supersede rapid Advanced-control changes

const el = (id) => document.getElementById(id);

const ui = {
  status: el("status"),
  drop: el("drop"),
  file: el("file"),
  advanced: el("advanced"),
  format: el("format"),
  maxEdge: el("maxedge"),
  keepFull: el("keepfull"),
  maxBytes: el("maxbytes"),
  error: el("error"),
  busy: el("busy"),
  mpWarning: el("mp-warning"),
  elapsed: el("elapsed"),
  result: el("result"),
  inImg: el("in-img"),
  outImg: el("out-img"),
  delta: el("delta"),
  resizeNote: el("resize-note"),
  score: el("score"),
  scoreValue: el("score-value"),
  scoreBand: el("score-band"),
  scoreMeter: el("score-meter"),
  scoreFill: el("score-fill"),
  scoreSource: el("score-source"),
  download: el("download"),
  funnel: el("funnel"),
  funnelCmd: el("funnel-cmd"),
  copyCmd: el("copy-cmd"),
  recipeText: el("recipe-text"),
  version: el("version"),
};

// The page's state, mirrored onto <body data-state> — the CSS dresses it, the
// browser smoke waits on it, and it is the honest thing to show a user anyway.
// `data-slow` is set when the encode is the slow full-resolution path, which is the
// only one that shows the megapixel warning + counting timer.
function setState(state, slow = false) {
  document.body.dataset.state = state;
  document.body.dataset.slow = String(slow);
  const busy = state === "converting";
  for (const control of [ui.file, ui.format, ui.maxEdge, ui.keepFull, ui.maxBytes]) {
    control.disabled = busy;
  }
  ui.busy.hidden = !busy;
  ui.mpWarning.hidden = !(busy && slow);
  ui.elapsed.hidden = !(busy && slow);
}

setState("loading");

let source = null; // { file, url, width, height } — kept so an Advanced change re-converts
let outUrl = null; // the previous blob: URL, revoked on the next conversion
let jobSeq = 0;
let debounceTimer = null;
let timerId = null;

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

// ── the elapsed timer (honest: seconds counted, never a fake %) ────────────────

function startTimer(slow) {
  stopTimer();
  if (!slow) return;
  const started = performance.now();
  ui.elapsed.textContent = "0.0 s";
  timerId = setInterval(() => {
    ui.elapsed.textContent = `${((performance.now() - started) / 1000).toFixed(1)} s`;
  }, 100);
}

function stopTimer() {
  if (timerId !== null) {
    clearInterval(timerId);
    timerId = null;
  }
}

// ── the engine's thread ──────────────────────────────────────────────────────

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
    ui.advanced.hidden = false;
    setState("ready");
    return;
  }

  if (m.type === "failed") {
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
function ask(bytes, params) {
  const id = ++jobSeq;
  // Anything still in flight is now stale: forget it (newest wins). The worker will
  // still finish it, but nobody is listening and the page is not blocked either way.
  pending.clear();
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    worker.postMessage(
      { id, bytes: bytes.buffer, format: params.format, maxEdge: params.maxEdge, maxBytes: params.maxBytes, speed: params.speed },
      [bytes.buffer],
    );
  });
}

// ── the controls, read into a job ──────────────────────────────────────────────

function jobParams() {
  const keepFull = ui.keepFull.checked;
  const edge = Number.parseInt(ui.maxEdge.value, 10);
  const maxEdge = keepFull ? null : Number.isInteger(edge) && edge > 0 ? edge : DEFAULT_MAX_EDGE;
  const kb = Number.parseInt(ui.maxBytes.value, 10);
  const maxBytes = Number.isInteger(kb) && kb > 0 ? kb * 1024 : null;
  return { format: ui.format.value, maxEdge, maxBytes, speed: 10 };
}

/// Will this conversion be the slow full-resolution path? Only then do we warn and
/// show the timer. Based on the megapixels that will actually be ENCODED — a 2048px
/// downscale of a 12 MP photo encodes ~3 MP and is fast; keeping full resolution on
/// the same photo encodes all 12 and is not.
function encodedMegapixels(params) {
  if (!source?.width || !source?.height) return null;
  const longEdge = Math.max(source.width, source.height);
  const cap = params.maxEdge;
  const scale = cap && longEdge > cap ? cap / longEdge : 1;
  return (source.width * scale * (source.height * scale)) / 1e6;
}

// ── input ────────────────────────────────────────────────────────────────────

async function load(file) {
  clearError();
  if (source?.url) URL.revokeObjectURL(source.url);
  const url = URL.createObjectURL(file);
  // Probe the natural dimensions page-side so the slow-path decision (below) can be
  // made before the worker even starts. `createImageBitmap` decodes every raster
  // format the browser supports (AVIF included); an SVG or an unreadable file just
  // leaves dims null → calm spinner, and the engine reports any real error.
  let dims = null;
  try {
    const bmp = await createImageBitmap(file);
    dims = { width: bmp.width, height: bmp.height };
    bmp.close();
  } catch {
    /* dimensions are a nice-to-have for the timer heuristic, not required */
  }
  source = { file, url, width: dims?.width ?? null, height: dims?.height ?? null };
  await convert();
}

// ── convert ──────────────────────────────────────────────────────────────────

async function convert() {
  if (!source) return;
  clearError();

  const params = jobParams();
  const mp = encodedMegapixels(params);
  const slow = mp !== null && mp >= SLOW_MP;
  if (slow) {
    ui.mpWarning.textContent = `${mp.toFixed(1)} MP at full resolution — this can take a few seconds. The page stays responsive.`;
  }
  setState("converting", slow);
  startTimer(slow);

  const bytes = new Uint8Array(await source.file.arrayBuffer());

  let result;
  try {
    result = await ask(bytes, params);
  } catch (e) {
    stopTimer();
    showError(`Can't convert ${source.file.name}: ${e.message ?? e}`);
    return;
  }

  stopTimer();
  render(result);
}

// ── the decision, shown ──────────────────────────────────────────────────────

function render(m) {
  const { input, scaled, output, score, scoredBy, budget, budgetMissed, elapsedMs } = m;

  ui.inImg.src = source.url;
  el("in-dims").textContent = `${input.width}×${input.height}`;
  el("in-format").textContent = input.format;
  el("in-bytes").textContent = fmtBytes(input.bytes);

  // NEVER-BIGGER — pure page logic. `web` reports size honestly and can go larger
  // (a small source above the 2048 bound re-encodes bigger); the demo layers a
  // never-bigger guard on top and hands the ORIGINAL back when nothing beat it.
  const keptOriginal = output.bytes >= input.bytes;

  if (outUrl) URL.revokeObjectURL(outUrl);
  let downloadBlob, downloadName, shownBytes, shownFormat, shownW, shownH, shownUrl;
  if (keptOriginal) {
    downloadBlob = source.file; // the original bytes, untouched
    downloadName = source.file.name;
    shownBytes = input.bytes;
    shownFormat = input.format;
    shownW = input.width;
    shownH = input.height;
  } else {
    downloadBlob = new Blob([m.out], { type: `image/${output.format}` });
    const stem = source.file.name.replace(/\.[^.]+$/, "");
    downloadName = `${stem}.${output.format}`;
    shownBytes = output.bytes;
    shownFormat = output.format;
    shownW = output.width;
    shownH = output.height;
  }
  outUrl = URL.createObjectURL(downloadBlob);
  shownUrl = keptOriginal ? source.url : outUrl;

  ui.outImg.src = shownUrl;
  el("out-dims").textContent = `${shownW}×${shownH}`;
  el("out-format").textContent = shownFormat;
  el("out-bytes").textContent = fmtBytes(shownBytes);

  ui.download.href = outUrl;
  ui.download.download = downloadName;
  ui.download.textContent = `Download ${downloadName} (${fmtBytes(shownBytes)})`;

  const saved = Math.round((1 - shownBytes / input.bytes) * 100);
  if (keptOriginal) {
    ui.delta.className = "delta kept";
    ui.delta.textContent =
      `Already optimized — kept your file (${fmtBytes(input.bytes)}). Nothing beat it, so nothing ` +
      `changed. (The CLI's \`web\` would report this honestly as larger; \`optimize\` never grows a file.)`;
  } else {
    ui.delta.className = "delta smaller";
    ui.delta.textContent = `${saved}% smaller — ${fmtBytes(input.bytes)} → ${fmtBytes(shownBytes)}`;
  }

  // Honest about the downscale — stated, not hidden.
  const resized = scaled.width !== input.width || scaled.height !== input.height;
  ui.resizeNote.textContent = keptOriginal
    ? ""
    : resized
      ? `Resized to ${scaled.width}×${scaled.height} for web (from ${input.width}×${input.height}).`
      : `Kept at ${input.width}×${input.height} — already within the ${DEFAULT_MAX_EDGE}px web size.`;

  renderScore(score, scoredBy, shownFormat, keptOriginal, budgetMissed, budget, shownBytes);
  renderFunnel();

  // The smoke reads these: what the engine reported, straight off the DOM.
  const ds = ui.result.dataset;
  ds.inBytes = String(input.bytes);
  ds.inFormat = input.format;
  ds.decodedBy = input.decodedBy; // "engine", or "browser" for an .avif input (DEC-065)
  ds.outBytes = String(shownBytes);
  ds.outWidth = String(shownW);
  ds.outHeight = String(shownH);
  ds.outFormat = shownFormat;
  ds.scaledWidth = String(scaled.width);
  ds.scaledHeight = String(scaled.height);
  ds.resized = String(resized);
  ds.keptOriginal = String(keptOriginal);
  ds.savedPct = String(saved);
  ds.scoredBy = scoredBy;
  ds.score = score == null ? "" : String(score);
  ds.budgetMissed = String(!!budgetMissed);
  ds.elapsedMs = String(elapsedMs);
  ui.result.hidden = false;
  setState("done");
}

/// A raw SSIMULACRA2 value → a band a non-expert can read. The value is RAW
/// (SPEC-079), NOT a 0–100 percentage: ~100 is visually identical, it can exceed 100,
/// and it goes NEGATIVE on a bad encode (a q20 JPEG measured −4.70). These bands cover
/// that whole range — the meter fill is clamped for the bar's sake, the number is not.
function scoreBand(s) {
  if (s >= 95) return { label: "indistinguishable", cls: "band-top" };
  if (s >= 85) return { label: "visually lossless", cls: "band-top" };
  if (s >= 70) return { label: "high", cls: "band-high" };
  if (s >= 50) return { label: "medium", cls: "band-mid" };
  if (s >= 30) return { label: "low", cls: "band-low" };
  return { label: "very low", cls: "band-bad" };
}

/// The score panel, honestly (SPEC-081). When a number was genuinely measured it is
/// shown RAW on an interpretable band + meter; when there is no honest number to show
/// (lossless output, a kept original, or a browser that couldn't decode the AVIF to
/// measure) the meter is hidden and the panel says in words what happened — never a
/// fabricated score. `scoredBy` (from the worker) decides which case this is:
///   engine  — the JPEG search's own SSIMULACRA2
///   browser — the AVIF, decoded back in the browser and scored (the SPEC-080 gap, closed)
///   lossless — pixels preserved, nothing to score
///   unavailable — scoring failed; be candid
function renderScore(score, scoredBy, format, keptOriginal, budgetMissed, budget, shownBytes) {
  let value = null; // the raw SSIMULACRA2 to display, or null for a worded-only panel
  let source; // one honest line on where the number came from — or why there isn't one

  if (keptOriginal) {
    source = "Your original was already the best — no re-encode, so there is no new score to measure.";
  } else if (scoredBy === "engine" && score != null) {
    value = score;
    source = "Measured by the engine — the perceptual search that chose this JPEG's quality.";
  } else if (scoredBy === "browser" && score != null) {
    value = score;
    source =
      "Measured by decoding the AVIF back in your browser and scoring it against the resized input — " +
      "the one output the engine can't score itself (it has no AVIF decoder, DEC-065).";
  } else if (scoredBy === "lossless") {
    source = `Lossless ${format.toUpperCase()} — every pixel is preserved, so there is no quality loss to score.`;
  } else {
    // "unavailable" (or any unexpected state): candid, not a fabricated number.
    source =
      "Couldn't score this output — your browser wouldn't decode the AVIF back to measure it. " +
      "(Chrome, Firefox and Safari have all decoded AVIF since 2020–23; a very old browser will not.)";
  }
  if (budgetMissed) {
    source += ` (Couldn't hit the ${fmtBytes(budget)} budget — the smallest this encoder produced was ${fmtBytes(shownBytes)}.)`;
  }

  if (value == null) {
    ui.score.dataset.mode = "worded";
    ui.scoreValue.textContent = "";
    ui.scoreBand.textContent = "";
    ui.scoreBand.className = "score-band";
    ui.scoreMeter.hidden = true;
  } else {
    const band = scoreBand(value);
    ui.score.dataset.mode = "measured";
    ui.scoreValue.textContent = `SSIMULACRA2 ${value.toFixed(1)}`;
    ui.scoreBand.textContent = band.label;
    ui.scoreBand.className = `score-band ${band.cls}`;
    // The bar can't render a negative width or overflow past full, so the FILL is
    // clamped to [0, 100] — the raw number above it is not.
    ui.scoreFill.style.width = `${Math.max(0, Math.min(100, value))}%`;
    ui.scoreFill.className = `score-fill ${band.cls}`;
    ui.scoreMeter.hidden = false;
  }
  ui.scoreSource.textContent = source;
}

/// THE FUNNEL. Every conversion becomes an invitation to run the real thing: the
/// exact command for the file just dropped, a copy button, and the recipe verbatim.
function renderFunnel() {
  const cmd = `crustyimg web ${source.file.name}`;
  ui.funnelCmd.textContent = cmd;
  ui.funnelCmd.dataset.cmd = cmd; // the exact string the copy button writes
  ui.recipeText.textContent = WEB_RECIPE;
  ui.funnel.hidden = false;
}

/// Copy `text` to the clipboard. Prefers the async Clipboard API, and falls back to a
/// hidden-textarea `execCommand('copy')` — which works in the contexts the async API
/// won't (an insecure origin, an older browser). Returns whether either succeeded.
async function copyText(text) {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* fall through to the legacy path */
  }
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.setAttribute("readonly", "");
    ta.style.position = "fixed";
    ta.style.top = "-1000px";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    const done = document.execCommand("copy");
    ta.remove();
    return done;
  } catch {
    return false;
  }
}

ui.copyCmd.addEventListener("click", async () => {
  const cmd = ui.funnelCmd.dataset.cmd ?? ui.funnelCmd.textContent;
  if (await copyText(cmd)) {
    ui.copyCmd.dataset.copied = cmd; // proof for the smoke; also drives the label
    ui.copyCmd.textContent = "Copied!";
    setTimeout(() => {
      ui.copyCmd.textContent = "Copy";
    }, 1500);
  } else {
    ui.copyCmd.textContent = "Press ⌘/Ctrl-C";
  }
});

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

// Advanced-control changes re-convert the current image — debounced so dragging a
// number or flipping options fast does not stack a job per keystroke (newest wins,
// and the in-flight worker result is superseded by `ask()` anyway).
function scheduleConvert() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (source) convert();
  }, DEBOUNCE_MS);
}

ui.keepFull.addEventListener("change", () => {
  ui.maxEdge.disabled = ui.keepFull.checked;
  scheduleConvert();
});
ui.format.addEventListener("change", scheduleConvert);
ui.maxEdge.addEventListener("input", scheduleConvert);
ui.maxBytes.addEventListener("input", scheduleConvert);
