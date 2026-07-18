# SPEC-095 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] design — framed build-ready 2026-07-18 (while SPEC-080 was in verify). Closes DEC-069: the wasm
  demo AVIF path encodes at q80 (`AVIF_DEFAULT_QUALITY`, the encoder default in the no-search arm) while
  native `web` uses q85 (`FAST_LOSSY_QUALITY`), so the demo OVERSTATES savings vs the real CLI. Fix = the
  `(_, true) => (None, None)` arm in `optimize_detailed` (`src/wasm.rs:459`) passes `FAST_LOSSY_QUALITY`;
  rebuild the wasm; tighten the demo "approximates" → "same engine + quality" copy. Native `convert`
  (q80 byte-identity) untouched. **Build AFTER SPEC-080 merges** (edits the merged demo). Complexity S.
- [ ] build — single session, primary checkout (touches src/wasm.rs + wasm rebuild + demo copy).
- [ ] verify — single session, primary checkout.
- [ ] ship — orchestrator.
