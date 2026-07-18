# SPEC-090 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-090-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-16. Origin: SPEC-088's verify measured `web` shipping a file
  14% LARGER than a 3000px source; the pre-spec oracle reproduces it → pre-existing SPEC-085 behavior,
  honestly reported, but the documented promise is measured against a different baseline than the code
  enforces (`pick_winner` compares against the DOWNSCALED intermediate, not the original file). Spec
  decides claim-vs-behavior with evidence; recommendation (A) correct-the-claim + surface it, to be
  proven or refuted at build. DEC-075 at build.
- [x] build — 2026-07-17, branch `spec-090-web-never-bigger`. Option **(A)**: dimension contract wins,
  docs corrected, larger-than-original surfaced (stderr `note:` + additive/gated `larger_than_source`
  `--json` field). Pre-spec oracle reproduced 36% larger; `web==apply` + `optimize` byte-identity both
  verified against the parent-commit binary. **Framing mechanism was imprecise** (`source_bytes` is the
  original, not the downscaled intermediate — the `pipeline_altered` override is the real path). DEC-075
  emitted. All gates green. PR (pending).
- [x] verify — 2026-07-18, primary checkout, branch `spec-090-web-never-bigger`. **✅ APPROVED after one verify fix.**
  Mechanism confirmed independently (`pick_winner` filters `bytes < source_bytes` where `source_bytes` =
  the original raw file, `read_raw_bytes` at cli/mod.rs:4479-4480; the larger output ships via
  `None if pipeline_altered` at ~4562 — the build's correction of the spec's imprecise framing is right, not
  a third story). Signal-fires-iff-larger proven on the real `--features avif,webp-lossy` binary vs the
  parent-commit (`0eac2cf`) oracle on the SAME source (313525 B → AVIF 340040 B): current adds
  `"larger_than_source":true` + the stderr `note:`; parent has neither; common-path JSON is byte-identical
  (additive/gated holds); negative control (normal photo, 30% smaller) emits no flag/note. `web==apply --recipe
  web` byte-identical (image+JSON+note); `optimize` byte-identical to parent across 5 inputs; SPEC-084 "N%
  larger" unregressed; `--quiet` suppresses the human note but keeps the machine flag.
  **Defect found + fixed:** the two heavy e2e tests were gated `#[cfg(not(feature = "avif"))]`, but CI's
  `webp-lossy` feature job (no avif, lossy WebP present) also ran them — lossy WebP crushes the 512px downscale
  to 40060 B « the 187320 B source, so `assert!(out > src)` failed (CI: `125 passed; 2 failed`). This is the
  [[a-green-gate-on-one-os-is-not-the-required-matrix]] cousin: the premise needs NO lossy encoder, not just
  no-avif. Widened both gates to `not(any(avif, webp-lossy))`; the codec-independent `analysis::decide` unit
  tests (which passed under webp-lossy in CI) carry the signal for every feature build. Re-ran green:
  default, `--features avif` (428 passed), `--features webp-lossy` (fixed, 0 fail), clippy (default+webp-lossy),
  fmt, `--no-default-features`, `just validate`/`bench`/`bench-micro`. (build_watch + convert/responsive
  local fails are parallel-contention flakes — pass serially, green in CI.)
- [x] ship — squash-merged PR #96 (**cc9b832**) 2026-07-18, full three-OS+feature matrix green at b486815.
  Bookkeeping: cycle→ship, 3 cost sessions with `model:` (build $7.65 / verify $6.48 / ship $0.60 ≈
  **$14.73** — an S whose cost was the AVIF-oracle verification loop, not the diff), DEC-075, timeline,
  archive, memory + brag. **STAGE-030 held ACTIVE per maintainer** (not closed at this merge). Lessons:
  the framing-mechanism correction (triple-checked) + the webp-lossy gate defect
  ([[a-green-gate-on-one-os-is-not-the-required-matrix]] on a feature flag).
