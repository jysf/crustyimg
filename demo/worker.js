// The engine's thread (SPEC-078, reframed by SPEC-080). Every conversion the demo
// does happens HERE, in a module Web Worker, and never on the page's thread.
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
// THE `web` FLOW, MIRRORED (SPEC-080). The demo's default is the flagship `web` verb
// (SPEC-085 / DEC-070): auto-orient, downscale the long edge to 2048, then modernize
// (AVIF for photos, lossless WebP for graphics) and score the winner. `optimizeDetailed`
// does NOT resize (DEC-068), so THIS FILE does the geometry itself — via `transform`
// with the SAME recipe machinery the CLI reads from disk (DEC-005), so the resize is
// the engine's own `fast_image_resize`, not a browser resampler. Then it calls
// `optimizeDetailed` (speed 10 in the browser; the CLI stays 6 — DEC-020) and reports
// what it did. The never-bigger guard is pure PAGE logic (demo.js); the worker just
// converts and hands both byte counts back.
//
// The engine is loaded exactly as the page loaded it (DEC-067): the package is built
// `--target web`, so `init()` is explicit and resolves `crustyimg_bg.wasm` relative to
// THIS module's URL — `demo/vendor/crustyimg.js` → `demo/vendor/crustyimg_bg.wasm`.
// A module worker (`new Worker(url, { type: 'module' })`) can `import`, and its
// `import.meta.url` is its own script URL, so the same vendored package works
// unchanged in here. (Proven by the browser smoke, which fails if the worker cannot
// instantiate the engine.)
//
// AVIF, BOTH WAYS ROUND. This build ENCODES AVIF and cannot DECODE it (DEC-065 —
// the decoder is a different codec that does not build for bare wasm32). So an
// `.avif` INPUT is decoded by the browser instead — `createImageBitmap` → an
// OffscreenCanvas → PNG bytes — and those PNG bytes are what the engine sees. Same
// asymmetry on the way out: the engine cannot read its own AVIF back, so the
// output's dimensions are read by the browser's decoder too. Neither path adds a
// codec to the wasm (the `pure-rust-codecs-default` constraint holds): they borrow
// the one already in the browser.

import init, { info, optimizeDetailed, transform, version } from "./vendor/crustyimg.js";

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

// ── the conversion ───────────────────────────────────────────────────────────

/// The `web` geometry as a recipe: auto-orient, then (unless full resolution was
/// asked for) downscale the long edge to `cap`. It is the SAME recipe TOML the CLI
/// reads from disk (DEC-005), built through the SAME registry, encoded to a LOSSLESS
/// PNG — so `optimizeDetailed` gets exactly the pixels the CLI's `web` would, and the
/// resize is `fast_image_resize`, not the browser's resampler. `mode = "max"` never
/// upscales, so a source already within `cap` passes through untouched.
function geometryRecipe(cap) {
  let recipe = 'version = "1"\n\n[[step]]\nop = "auto-orient"\n';
  if (Number.isInteger(cap) && cap > 0) {
    recipe += `\n[[step]]\nop = "resize"\nmode = "max"\nwidth = ${cap}\n`;
  }
  return recipe;
}

/// One conversion, start to finish:
///
///   [browser-decode an .avif] → transform(auto-orient + downscale) → PNG
///   → optimizeDetailed(→ the chosen format, speed 10)
///
/// The downscale runs BEFORE the encode and the analysis, exactly as `web` orders it,
/// so Auto picks the format for the DOWNSCALED image (a 2 MP photo, not a 12 MP one).
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
  // handed back, whose dimensions are the AVIF's — so width/height are right, and the
  // format is the one we sniffed above, not "png".
  const i = info(pixels);
  input.width = i.width;
  input.height = i.height;
  input.format ??= i.format;

  // The web geometry. `cap` is the long-edge limit: the default 2048, an Advanced
  // override, or null for "keep full resolution".
  const cap = Number.isInteger(job.maxEdge) && job.maxEdge > 0 ? job.maxEdge : null;
  pixels = transform(pixels, geometryRecipe(cap), "png");
  const scaled = info(pixels);

  // Modernize + score. `optimizeDetailed` does not resize (we just did). speed 10 in
  // the browser (DEC-020). A byte budget only comes from the Advanced control.
  const budget = Number.isInteger(job.maxBytes) && job.maxBytes > 0 ? job.maxBytes : undefined;
  const result = optimizeDetailed(
    pixels,
    job.format && job.format !== "auto" ? job.format : "auto",
    Number.isInteger(job.speed) ? job.speed : 10,
    budget,
    undefined,
  );
  const outBytes = result.bytes; // a getter that CLONES — read it once.
  const elapsedMs = Math.round(performance.now() - started);

  const back = await readBack(outBytes);

  // An unsatisfiable byte budget returns over-budget bytes SILENTLY (SPEC-079 note),
  // so a budget that was asked for but missed is flagged rather than implied met.
  const budgetMissed = budget !== undefined && outBytes.length > budget;

  return {
    input,
    scaled: { width: scaled.width, height: scaled.height },
    output: {
      width: back.width,
      height: back.height,
      format: back.format,
      readBy: back.readBy,
      bytes: outBytes.length,
      quality: result.quality ?? null,
      speed: result.speed ?? null,
    },
    // Raw SSIMULACRA2 (can be negative; ~100 is visually identical) or null when the
    // engine could not score it — `scoredBy` says which.
    score: result.score ?? null,
    scoredBy: result.scoredBy,
    budget: budget ?? null,
    budgetMissed,
    elapsedMs,
    // Transferred, not copied: the bytes leave this thread's heap entirely.
    out: outBytes.buffer,
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
