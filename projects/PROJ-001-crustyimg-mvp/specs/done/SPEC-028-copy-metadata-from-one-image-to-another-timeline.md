# SPEC-028 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-028-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the last metadata-lane command. A design-time probe
  verified JPEG EXIF+ICC transfer via `img-parts`' `ImageEXIF`/`ImageICC` traits with
  byte-identical DST pixels, AND surfaced that PNG copy is non-viable (little_exif
  zTXt vs img-parts eXIf chunk) → emitted **DEC-030** (copy-metadata JPEG-only v1).
  `copy_metadata(from,to)` + `run_copy_metadata` (two inputs, single fixed output, NOT
  a fan-out; default writes back to DST in place behind `-y`). No new dep. Fleshed out
  the api-contract entry. Design + DEC-030 pushed to `main` before build.
- [x] build (2026-06-18, PR #32) — foreground metered subagent (Opus, 144k tok,
  ~6 min). Added `metadata::copy_metadata` (img-parts ImageEXIF/ImageICC transfer,
  JPEG-only) + `run_copy_metadata` (two inputs, single fixed output, default in-place
  behind `-y`, not a fan-out). 10 new tests (6 unit + 4 integration). No new dep, no
  new DEC. All gates green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED,
  no concerns. Confirmed the no-pixel-encode invariant (never builds an `Image`),
  JPEG-only enforcement (PNG → exit 4, no silent no-op), EXIF+ICC replace/clear
  semantics, output/overwrite codes (5/3/4), all 10 tests present + substantive.
  Orchestrator re-ran the gates: `cargo test` 330 ok (0 failed), clippy/fmt/deny clean.
- [x] ship (2026-06-18, PR #32 squash-merged → `cd27c5e`) — reflections + cost
  totals filled (build 143912 real / verify ~45k est; totals 188912 / $1.70 / 4),
  STAGE-004 backlog flipped (metadata lane COMPLETE), archived to `specs/done/`,
  `just cost-audit` green + cost-capture confirmed on main CI.
