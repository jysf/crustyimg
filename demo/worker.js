// The engine's thread (SPEC-078). Every conversion the demo does happens HERE, in
// a module Web Worker, and never on the page's thread.
//
// WHY. rav1e — the AVIF encoder compiled into this .wasm (DEC-065) — is serial and
// slow: seconds of straight-line work for a photo. Called from the page's thread it
// would freeze the tab (no paint, no scroll, no click) for the whole encode, which
// is why SPEC-077 shipped with AVIF disabled. A Worker is a separate thread, so the
// encode blocks nothing the user can see. That is the ONLY reason this file exists,
// and it is the whole of the fix: no SharedArrayBuffer, no wasm threads, no
// COOP/COEP headers — the engine is still single-threaded, it is simply not on the
// thread that draws.
//
// The engine is loaded exactly as the page loaded it (DEC-067): the package is built
// `--target web`, so `init()` is explicit and resolves `crustyimg_bg.wasm` relative to
// THIS module's URL — `demo/vendor/crustyimg.js` → `demo/vendor/crustyimg_bg.wasm`.
// A module worker (`new Worker(url, { type: 'module' })`) can `import`, and its
// `import.meta.url` is its own script URL, so the same vendored package works
// unchanged in here. (That was this spec's load-bearing assumption; it is proven by
// the browser smoke, which fails if the worker cannot instantiate the engine.)
//
// AVIF, BOTH WAYS ROUND. This build ENCODES AVIF and cannot DECODE it (DEC-065 —
// the decoder is a different codec that does not build for bare wasm32). So an
// `.avif` INPUT is decoded by the browser instead — `createImageBitmap` → an
// OffscreenCanvas → PNG bytes — and those PNG bytes are what the engine sees. Same
// asymmetry on the way out: the engine cannot read its own AVIF back, so the
// output's dimensions are read by the browser's decoder too. Neither path adds a
// codec to the wasm (the `pure-rust-codecs-default` constraint holds): they borrow
// the one already in the browser.

import init, { info, optimize, transform, version } from "./vendor/crustyimg.js";

const msg = (e) => e?.message ?? String(e);

// Boot immediately — the 1.3 MB .wasm is worth fetching while the user is still
// choosing a file. The page waits for `ready` before it accepts a drop, and shows
// `failed` (with the reason) if the engine could not be instantiated.
let bootError = null;
const booted = init().then(
  () => self.postMessage({ type: "ready", version: version() }),
  (e) => {
    bootError = e;
    self.postMessage({ type: "failed", message: msg(e) });
  },
);

// ── the AVIF ↔ browser seam ──────────────────────────────────────────────────

/// Is this an AVIF file? An ISOBMFF `ftyp` box whose brand list mentions `avif`.
/// Cheap and byte-level on purpose: the engine's `info()` would only tell us by
/// throwing, and `File.type` is the OS's guess, not the file's own claim.
function isAvif(bytes) {
  if (bytes.length < 16) return false;
  const head = new TextDecoder("latin1").decode(bytes.subarray(4, 32));
  return head.startsWith("ftyp") && head.includes("avif");
}

/// Decode with the BROWSER and hand the engine PNG bytes instead.
///
/// The engine cannot decode AVIF, but every browser that can show one already has a
/// decoder; `createImageBitmap` is it. The bitmap is drawn to an OffscreenCanvas
/// (available in a worker — a `<canvas>` element is not) and re-encoded as lossless
/// PNG, so nothing is lost between the browser's pixels and the engine's.
async function decodeInBrowser(bytes, label) {
  let bitmap;
  try {
    bitmap = await createImageBitmap(new Blob([bytes], { type: "image/avif" }));
  } catch (e) {
    // Be specific: this is the browser's decoder failing, not the engine's. A
    // browser too old for AVIF lands here, and so does a corrupt file.
    throw new Error(
      `this build can't decode ${label} itself, so it asked the browser to — and the browser ` +
        `refused: ${msg(e)}. (Chrome/Firefox/Safari have decoded AVIF since 2020–23; a very old ` +
        `browser will not.)`,
    );
  }
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  canvas.getContext("2d").drawImage(bitmap, 0, 0);
  bitmap.close();
  const png = await canvas.convertToBlob({ type: "image/png" });
  return new Uint8Array(await png.arrayBuffer());
}

/// What the engine produced, read back. Normally that is the engine's own `info()`
/// — the same decode a consumer of the file would do. For AVIF it CANNOT be (no
/// decoder), so the browser reads it instead, which is a stronger check anyway: an
/// independent decoder agreeing that these bytes are an image of the right size.
async function readBack(out) {
  try {
    const i = info(out);
    return { width: i.width, height: i.height, format: i.format, readBy: "engine" };
  } catch (e) {
    if (!isAvif(out)) throw e;
    const bitmap = await createImageBitmap(new Blob([out], { type: "image/avif" }));
    const back = { width: bitmap.width, height: bitmap.height, format: "avif", readBy: "browser" };
    bitmap.close();
    return back;
  }
}

// ── how the quality was chosen ───────────────────────────────────────────────
// Said out loud rather than implied, because the answer differs by format and two
// of the three are not what a user would guess. There is no quality SLIDER: the
// shipped wasm surface (DEC-064) takes no quality argument — `optimize` decides.

function qualityNote(format) {
  switch (format) {
    case "jpeg":
      return {
        mode: "searched",
        note: "quality searched with SSIMULACRA2 until the result is visually lossless (the engine's own auto-quality — it decodes every candidate to score it)",
      };
    case "avif":
      return {
        mode: "default",
        note: "encoded at the encoder's default quality — not searched: a perceptual search has to DECODE each candidate to score it, and this build encodes AVIF without being able to decode it (DEC-065)",
      };
    default:
      return { mode: "lossless", note: "lossless — every pixel is preserved, so there is no quality to choose" };
  }
}

// ── the conversion ───────────────────────────────────────────────────────────

/// One conversion, start to finish. The chain is deliberately uniform:
///
///   [browser-decode an .avif] → [transform(recipe) → PNG, if a size cap was asked]
///   → optimize(→ the chosen format)
///
/// The final encode always goes through `optimize`, so the quality story is the same
/// however the user got here (and "Auto" — let the engine pick the format — works
/// with a resize too, which it could not if the resize did the encoding). When a
/// size cap is set, the resize is a real `transform` with a real recipe — the SAME
/// recipe TOML the CLI reads from disk (DEC-005) — encoded to a LOSSLESS PNG so the
/// intermediate throws nothing away before `optimize` gets its turn.
async function convert(job) {
  await booted;
  if (bootError) throw bootError;

  const original = new Uint8Array(job.bytes);
  let pixels = original;
  const input = { bytes: original.length, decodedBy: "engine" };

  if (isAvif(original)) {
    pixels = await decodeInBrowser(original, "AVIF");
    input.decodedBy = "browser";
    input.format = "avif";
  }

  const started = performance.now();

  // `info()` on the engine-readable bytes: for an AVIF that is the PNG the browser
  // handed back, whose dimensions are the AVIF's — so the width/height are right,
  // and the format is the one we sniffed above, not "png".
  const i = info(pixels);
  input.width = i.width;
  input.height = i.height;
  input.format ??= i.format;

  if (Number.isInteger(job.maxEdge) && job.maxEdge > 0) {
    const recipe = `version = "1"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = ${job.maxEdge}\n`;
    pixels = transform(pixels, recipe, "png");
  }

  const out = optimize(pixels, job.format === "auto" ? "auto" : job.format);
  const elapsedMs = Math.round(performance.now() - started);

  const output = await readBack(out);
  output.bytes = out.length;

  return {
    input,
    output,
    elapsedMs,
    quality: qualityNote(output.format),
    // Transferred, not copied: the bytes leave this thread's heap entirely.
    out: out.buffer,
  };
}

self.addEventListener("message", async (ev) => {
  const job = ev.data;
  try {
    const result = await convert(job);
    self.postMessage({ type: "done", id: job.id, ...result }, [result.out]);
  } catch (e) {
    self.postMessage({ type: "error", id: job.id, message: msg(e) });
  }
});
