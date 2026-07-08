# SPEC-060 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — SVG rasterize as a default, pure-Rust input; Failing Tests (default-build
  intrinsic-dims decode, viewBox-only sizing, oversize→LimitsExceeded, malformed→typed error,
  external-file-ref ignored, dir-source discovery, optimize→png / convert→webp, bundled-font text)
  + full Implementation Context. Load-bearing item done in design: a firsthand probe of the resvg
  stack → **finding: the whole tree is PERMISSIVE (no license exception; corrects the framing's
  MPL-2.0 assumption); the only cost is a RUSTSEC-2026-0192 advisory ignore for ttf-parser via the
  usvg text stack.** Render + security + font API compiled and verified against resvg 0.47.0.
  Rasterizer choice → **DEC-054** (emit at build). Framing, 2026-07-08.
- [ ] **build** — wire `src/image/svg.rs` (hardened usvg parse → tiny-skia Pixmap → straight RGBA8 →
  canonical `Image`) into `decode_with_limits` (content-sniff dispatch), add `svg` to
  `IMAGE_EXTENSIONS`, deny.toml RUSTSEC-2026-0192 advisory ignore (no license exception), fixture +
  tests + fuzz target, DEC-054. Verify default + lean + `just deny` + clippy + fmt; check MSRV.
- [ ] **verify** — fresh session; re-run all gates independently, confirm hostile-input safety
  (external-ref refused, cap-before-raster), lean build + `just deny` green, DEC-054 consistent.
- [ ] **ship** — merge PR, cost sessions + totals, ship reflection, archive to done/, update
  STAGE-017 backlog; carry `fuzz/svg_decode` as a pre-1.0 hardening gate (like `fuzz/avif_decode`).
