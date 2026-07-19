// A PNG fixture, and a PNG header parse — both in plain JS, on purpose.
//
// The fixture is deliberately NOT produced by the code under test: an input the
// crate's own encoder wrote would let a broken decode-encode round-trip agree with
// itself. This is a hand-rolled 8-bit RGB PNG (filter 0 scanlines, zlib IDAT) —
// the format's own spec, written by neither Rust nor the `image` crate. `readIhdr`
// is the other half of that: a decoder we did not write, so an output's dimensions
// are checked independently of anything the crate claims about them.
//
// Shared by the npm smoke (SPEC-075) and the browser demo smoke (SPEC-077).

import { deflateSync } from "node:zlib";

export const PNG_SIGNATURE = "89504e470d0a1a0a";

function crc32(buf) {
  let c = ~0;
  for (const b of buf) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}

function pngChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

/// An 8-bit RGB PNG carrying a gradient — not a solid fill, which survives almost
/// any bug.
export function makePng(width, height) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // colour type: truecolour RGB (no alpha)
  // 10..12 = compression, filter, interlace — all 0.

  const raw = Buffer.alloc(height * (1 + width * 3));
  for (let y = 0; y < height; y++) {
    const row = y * (1 + width * 3);
    raw[row] = 0; // filter type 0 (None)
    for (let x = 0; x < width; x++) {
      const p = row + 1 + x * 3;
      raw[p] = (x * 255) / (width - 1);
      raw[p + 1] = (y * 255) / (height - 1);
      raw[p + 2] = 128;
    }
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

/// A deterministic pseudo-random generator — seeded so a fixture is byte-stable
/// across runs (mulberry32). `Math.random()` would make the AVIF fixtures
/// non-reproducible, which matters when a size assertion is on the line.
function mulberry32(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/// An 8-bit RGB PNG carrying PHOTOGRAPHIC content — structured detail (a few sine
/// bands) blended with heavy per-pixel noise. A smooth gradient is NOT enough:
/// the engine's analysis reads a gradient as a flat GRAPHIC and routes it to a
/// lossless format, so a "convert a photo to AVIF" fixture built from `makePng`
/// would silently test the wrong path (the lesson STAGE-030 kept relearning —
/// synthetic math is not a photograph). This one has >256 colours, high luma
/// entropy, and a low flat-region fraction, so `Analysis` buckets it `Lossy` and
/// Auto picks AVIF — while the structure keeps rav1e from choking the way pure
/// noise would. Deterministic given `seed`.
export function makePhotoPng(width, height, seed = 1) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // colour type: truecolour RGB (no alpha)

  const rng = mulberry32(seed);
  const clamp = (v) => (v < 0 ? 0 : v > 255 ? 255 : v | 0);
  const raw = Buffer.alloc(height * (1 + width * 3));
  for (let y = 0; y < height; y++) {
    const row = y * (1 + width * 3);
    raw[row] = 0; // filter type 0 (None)
    for (let x = 0; x < width; x++) {
      const p = row + 1 + x * 3;
      // Structure (varies the local mean so it reads as content, not a flat fill)
      // plus a wide noise term (drives entropy up and the flat-ratio down).
      const base = 128 + 70 * Math.sin(x * 0.06) + 55 * Math.sin(y * 0.045 + 1.3);
      raw[p] = clamp(base + 90 * (rng() - 0.5));
      raw[p + 1] = clamp(base * 0.8 + 40 * Math.sin((x + y) * 0.03) + 90 * (rng() - 0.5));
      raw[p + 2] = clamp(150 - 0.4 * base + 90 * (rng() - 0.5));
    }
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

/// Read a PNG's signature and IHDR dimensions out of its first 24 bytes.
export function readIhdr(head) {
  return {
    signature: head.subarray(0, 8).toString("hex") === PNG_SIGNATURE,
    width: head.readUInt32BE(16),
    height: head.readUInt32BE(20),
  };
}
