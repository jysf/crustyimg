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
- [x] **verify** — ✅ APPROVED (fresh Opus session, run independently). All gates re-run locally:
  8/8 acceptance criteria mapped to real tests, cargo test 542, feature-gated round-trip, lean +
  `just deny` + clippy + fmt clean, decisions-audit clean, DEC-053 consistent, real-file color
  check, MSRV floor confirmed 1.90, PR #65 20/20 CI green. 1 non-blocking punch-list item →
  pre-1.0 gate: run `fuzz/avif_decode` under nightly+cargo-fuzz. 2026-07-07.
- [x] **ship** — squash-merged PR #65 → main (c0bc928); appended verify+ship cost sessions +
  totals (640k, labelled estimates §4), ship reflection, marked cycle ship; archived to done/;
  STAGE-016 shipped (single-spec stage). Fuzz-run carried as a pre-1.0 hardening gate. 2026-07-07.
