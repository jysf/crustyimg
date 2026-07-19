# SPEC-095 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] design — framed build-ready 2026-07-18 (while SPEC-080 was in verify). Closes DEC-069: the wasm
  demo AVIF path encodes at q80 (`AVIF_DEFAULT_QUALITY`, the encoder default in the no-search arm) while
  native `web` uses q85 (`FAST_LOSSY_QUALITY`), so the demo OVERSTATES savings vs the real CLI. Fix = the
  `(_, true) => (None, None)` arm in `optimize_detailed` (`src/wasm.rs:459`) passes `FAST_LOSSY_QUALITY`;
  rebuild the wasm; tighten the demo "approximates" → "same engine + quality" copy. Native `convert`
  (q80 byte-identity) untouched. **Build AFTER SPEC-080 merges** (edits the merged demo). Complexity S.
- [x] build — Sonnet, primary checkout (src/wasm.rs + wasm rebuild + demo copy). PR #99. All 7 acceptance
  criteria met; flagged a transient full-suite avif flake for verify to weigh.
- [x] verify — ✅ APPROVED (Opus, primary checkout). Demo's real hero AVIF is q85 from a fresh-from-HEAD
  wasm rebuild (demo-smoke, sips-graded valid; wasm-test byte-equals an independent q85 encode, mutation
  to q80 bites); both wasm sites anchor to `sink::FAST_LOSSY_QUALITY`, no bare 85; native `convert`
  byte-identical to the pre-spec parent binary (286 B, q80). Flake CLEARED — 3× isolated `--features avif`
  full runs all 777/0, PR #99 CI avif job green; environmental target/ contention, not a regression, not
  the SPEC-091 DisjointMut abort. All gates green.
- [ ] ship — orchestrator.
