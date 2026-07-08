# SPEC-058 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — AVIF decode as a default pure-Rust input; Failing Tests (default-build decode,
  decode-cap, corrupt→typed error, dir-source discovery, optimize→webp, feature-gated round-trip)
  + full Implementation Context. Load-bearing item: the decoder-dependency probe → **DEC-053**
  (candidates: `re_rav1d` via `image` vs `rav1d` BSD-2 + ISOBMFF glue; `zenavif` AGPL excluded).
  Framing, 2026-07-07.
- [x] **build** — probe confirmed `re_rav1d` (no-asm) + `avif-parse` decode a real AVIF (encode→parse→decode
  round-trip, 8/10-bit + alpha, no nasm); wired `src/image/avif.rs` (YUV→RGB honoring depth/chroma/range/
  matrix/premult-alpha) into `decode_with_limits` (ftyp-brand dispatch, DEC-034 caps), added `avif` to
  `IMAGE_EXTENSIONS`, deny.toml MPL/CC0 per-crate exceptions, fixture + tests + fuzz target, DEC-053,
  watchlist resolved. Default+lean+avif builds, clippy×3, fmt, `just deny`, 300+ tests green. No SPEC-059
  needed (avif-parse covered the container). MSRV 1.89→1.90. Branch `feat/spec-058-avif-decode`. 2026-07-07.
- [ ] **verify**
- [ ] **ship**
