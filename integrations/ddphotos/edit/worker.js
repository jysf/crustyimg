// The engine's thread. Every call into crustyimg-wasm happens here, never on the
// page's thread: a wasm call is synchronous, and an AVIF encode can take seconds,
// which would freeze the page.
//
// Protocol: the page posts { id, type, ... } and gets back { id, ok, ... } or
// { id, ok: false, message }. The photo's bytes are sent once ("load") and kept
// here, so each preview only sends the recipe.

import init, { info, optimizeDetailed, score, transform, version } from "./vendor/crustyimg.js";

const msg = (e) => e?.message ?? String(e);

let source = null; // the published photo's bytes, as fetched by the page

const booted = init().then(
  () => self.postMessage({ type: "ready", version: version() }),
  (e) => self.postMessage({ type: "failed", message: msg(e) }),
);

// Preview quality. A preview only needs to look right on screen, so it uses a fast
// fixed-quality JPEG rather than the perceptual search the final copy gets.
const PREVIEW_QUALITY = 88;

// AVIF speed for the final copy: 10 is the fastest rav1e setting. In a browser tab
// it costs a few percent more bytes and saves most of the wait.
const AVIF_SPEED = 10;

/// This build encodes AVIF but cannot decode it, so to score an AVIF result the
/// browser decodes it and the engine scores the decoded pixels.
async function avifToPng(bytes) {
  const bitmap = await createImageBitmap(new Blob([bytes], { type: "image/avif" }));
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  canvas.getContext("2d").drawImage(bitmap, 0, 0);
  bitmap.close();
  const png = await canvas.convertToBlob({ type: "image/png" });
  return new Uint8Array(await png.arrayBuffer());
}

/// A recipe with a top-level quality, for the preview encode.
function withQuality(recipe, quality) {
  return recipe.replace('version = "2"\n', `version = "2"\nquality = ${quality}\n`);
}

async function handle(req) {
  switch (req.type) {
    case "load": {
      source = new Uint8Array(req.bytes);
      const i = info(source);
      const meta = { width: i.width, height: i.height, format: i.format };
      i.free(); // wasm-side objects are not garbage-collected
      return meta;
    }

    case "preview": {
      if (!source) throw new Error("no photo loaded");
      const out = transform(source, withQuality(req.recipe, PREVIEW_QUALITY), "jpeg");
      return { bytes: out };
    }

    case "make": {
      if (!source) throw new Error("no photo loaded");
      const started = performance.now();

      // 1. Apply the edit (resize, watermark) and keep it lossless, so the encoder
      //    and the score both start from exactly the edited pixels.
      const edited = transform(source, req.recipe, "png");

      // 2. Encode. For a lossy format this searches for the smallest file that still
      //    scores as visually the same as `edited`. An empty format lets it choose.
      const result = optimizeDetailed(edited, req.format, AVIF_SPEED, undefined, undefined);
      const bytes = result.bytes;
      const format = result.format;
      const encoderQuality = result.quality;
      const engineScore = result.score;
      result.free();

      // 3. Score against the edit. The engine reports its own score where it ran a
      //    search; otherwise score here (decoding AVIF in the browser first).
      let quality = engineScore;
      if (quality === undefined) {
        try {
          const candidate = format === "avif" ? await avifToPng(bytes) : bytes;
          quality = score(edited, candidate);
        } catch {
          quality = undefined; // a score is a nice-to-have; the copy is still good
        }
      }

      const out = info(edited); // dimensions of the edit itself
      const width = out.width;
      const height = out.height;
      out.free();
      return {
        bytes,
        format,
        encoderQuality,
        score: quality,
        width,
        height,
        ms: Math.round(performance.now() - started),
      };
    }

    default:
      throw new Error(`unknown request ${req.type}`);
  }
}

self.onmessage = async (event) => {
  const req = event.data;
  try {
    await booted;
    const reply = await handle(req);
    const transfer = reply.bytes ? [reply.bytes.buffer] : [];
    self.postMessage({ id: req.id, ok: true, ...reply }, transfer);
  } catch (e) {
    self.postMessage({ id: req.id, ok: false, message: msg(e) });
  }
};
