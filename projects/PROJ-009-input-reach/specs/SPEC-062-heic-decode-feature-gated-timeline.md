# SPEC-062 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — HEIC decode behind an off-by-default `heic` feature (implements DEC-052); Failing
  Tests (default-build `.heic`→exit-4 CodecNotBuilt, `#[cfg(feature="heic")]` decode to dims,
  dimension-cap→LimitsExceeded, corrupt→typed error, is_heic-vs-avif brands, dir-source discovery,
  optimize→webp / convert→png under the feature) + full Implementation Context. Load-bearing probe done
  in design: brew libheif 1.23.1 + a real `.heic` (via `sips`) + libheif-rs 2.7.0 → **decoded to 64×48
  with correct pixels; libheif-rs/-sys are MIT so `just deny` stays green with NO exception (the LGPL is
  the SYSTEM C lib, invisible to deny — unlike ansi_colours); system-linked via pkg-config**. Approach →
  **DEC-056** (emit at build). Key wiring: a decode-side `ImageError::CodecNotBuilt`→exit-4 (mirror
  SinkError), ftyp-brand detection (AVIF dispatched first), a system-libheif CI job, and exclusion from
  every distributed artifact (DEC-052). Framing, 2026-07-08.
- [ ] **build** — add `src/image/heic.rs` (`is_heic` ftyp scan + `#[cfg(feature="heic")]` libheif-rs
  decode → canonical `Image`, DEC-034-capped, stride-honoring), `ImageError::CodecNotBuilt`→exit-4,
  dispatch in `decode_with_limits` (after AVIF), `.heic`/`.heif` in `IMAGE_EXTENSIONS`,
  `heic = ["dep:libheif-rs"]`, system-libheif CI job, distribution excludes heic, LGPL attribution,
  fixture via sips + tests + fuzz target, DEC-056. Verify default(exit-4) + `--features heic`(decode) +
  lean + `just deny`(green, no exception) + clippy×3 + fmt; check MSRV (bindgen/libheif-sys floor).
- [ ] **verify** — fresh session; re-run all gates independently, confirm default-build exit-4 + clear
  message, feature decode + caps + stride, deny green with no new exception, distribution excludes heic,
  DEC-056 consistent with DEC-052.
- [ ] **ship** — merge PR, cost sessions + totals, ship reflection, archive to done/, update STAGE-019
  backlog; carry `fuzz/heic_decode` as a pre-1.0 hardening gate; **PROJ-009 project-ship** (final stage —
  fill the project-level reflection in brief.md).
