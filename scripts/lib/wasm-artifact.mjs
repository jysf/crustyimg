// The build-profile fingerprint (SPEC-075, DEC-066) — shared by every gate that
// has to answer "did this .wasm come through `just wasm-build`?".
//
// What actually needs proving is "this .wasm came through the size-profiled
// `just wasm-build`". Size alone is only a PROXY for that, and a weak one: the
// profiled build (~1.39 MB brotli) and a stock-profile build (1,503,485 B,
// measured) are just 7.8% apart, so a single hand-picked ceiling between them has
// ~4% of daylight on either side. It would false-trip on any legitimate 4% growth
// — and *misreport the cause* when it did ("a bare cargo build would blow this") —
// while its discriminating power erodes as the artifact grows, because a stock
// build grows with it.
//
// So the profile is asserted STRUCTURALLY, and size is asserted separately:
//
//  1. `strip = true` — one of DEC-066's three levers — is directly observable in
//     the binary: a stripped .wasm has no debug-name table. Measured: the profiled
//     artifact carries a 43 B `name` custom section, a stock-profile one carries
//     980,293 B. Four orders of magnitude — categorical rather than a threshold,
//     and immune to however much the code legitimately grows.
//  2. The wire size is then a plain REGRESSION baseline keyed to the measured
//     profiled artifact, so a real size regression is reported as a size
//     regression instead of being misdiagnosed as a bypassed recipe.
//  3. The floor still asserts the AVIF encoder is actually in (DEC-065) — a lean
//     build is hundreds of KB smaller and quietly ships an artifact that cannot do
//     the thing the demo is for.

export const WASM_NAME_SECTION_MAX = 4_096;
export const WASM_BROTLI_BASELINE = 1_394_631;
export const WASM_BROTLI_TOLERANCE = 0.05;
export const WASM_BROTLI_MAX = Math.round(WASM_BROTLI_BASELINE * (1 + WASM_BROTLI_TOLERANCE));
export const WASM_BROTLI_MIN = Math.round(WASM_BROTLI_BASELINE * (1 - WASM_BROTLI_TOLERANCE));

/// Walk the module's sections and return the custom ones as name → payload length.
/// Id 0 is a custom section whose payload opens with a LEB128-length-prefixed
/// name; `name` is the debug symbol table that `strip` removes. This is how we see
/// the build profile in the artifact itself.
export function customSections(buf) {
  const out = new Map();
  let o = 8; // skip the magic + version words
  const leb = () => {
    let r = 0;
    let s = 0;
    let b;
    do {
      b = buf[o++];
      r |= (b & 0x7f) << s;
      s += 7;
    } while (b & 0x80);
    return r >>> 0;
  };
  while (o < buf.length) {
    const id = buf[o++];
    const len = leb();
    const end = o + len;
    if (end > buf.length) break; // malformed; nothing further is trustworthy
    if (id === 0) {
      const nameLen = leb();
      out.set(buf.toString("utf8", o, o + nameLen), len);
    }
    o = end;
  }
  return out;
}

/// The size of the `name` debug section — 0 on a stripped (profiled) artifact,
/// ~980 KB on a stock-profile one.
export function nameSectionSize(wasmBytes) {
  return customSections(wasmBytes).get("name") ?? 0;
}
