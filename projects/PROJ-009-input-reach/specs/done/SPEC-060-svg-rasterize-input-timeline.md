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
- [x] **build** — wire `src/image/svg.rs` (hardened usvg parse → tiny-skia Pixmap → straight RGBA8 →
  canonical `Image`) into `decode_with_limits` (content-sniff dispatch), add `svg` to
  `IMAGE_EXTENSIONS`, deny.toml RUSTSEC-2026-0192 advisory ignore (no license exception), fixture +
  tests + fuzz target, DEC-054. Verify default + lean + `just deny` + clippy + fmt; check MSRV.
  **PR #66, 2026-07-08.** Result: all gates green (555 default tests pass incl. 9 new SVG tests, lean
  build + `just deny` + clippy + fmt clean); licenses needed NO exception (probe finding held), one
  RUSTSEC-2026-0192 advisory ignore added; MSRV floor unchanged (1.90, resvg/usvg are 1.87). DEC-054
  emitted. Ready for a fresh verify session.
- [x] **verify** — ✅ APPROVED (fresh Opus session, run independently). All gates re-run locally
  (default+lean cargo test 555, clippy default+lean, fmt, `just deny`, decisions-audit) + drove the
  CLI on hostile (file:// + http refs → exit 0 clean raster) and bomb (100000² → LimitsExceeded in
  ~4 ms) inputs. Every acceptance criterion mapped to a real test; no license exception (probe held),
  one RUSTSEC-2026-0192 advisory ignore w/ revisit trigger; source_format=Png; MSRV 1.90; PR #66 20/20
  CI green. No punch list. 2026-07-08.
- [x] **ship** — squash-merged PR #66 → main (7414af3); appended verify+ship cost sessions + totals
  (325k, labelled estimates §4), ship reflection, marked cycle ship; archived to done/; STAGE-017
  shipped (single-spec stage). `fuzz/svg_decode` carried as a pre-1.0 hardening gate in docs/roadmap.md.
  2026-07-08.
