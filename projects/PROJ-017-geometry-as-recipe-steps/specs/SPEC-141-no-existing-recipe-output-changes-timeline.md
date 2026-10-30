# SPEC-141 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-141-<cycle>.md`.

## Instructions

- [x] **frame**: `prompts/SPEC-141-frame.md` (2026-10-30, Opus). GO, [S]. Premise confirmed on a
  release build of `8ccc2d1`: no test runs a corpus recipe, and a Lanczos3→CatmullRom mutation
  leaves `recipe_round_trip` green (cli-golden goes red on 4 resize cases, but bundled `web` stays
  green on 64×48). Pin pre-encode pixels everywhere, bytes for PNG/lossless WebP only (DEC-059).
  Resize-pixel stability across the three CI legs is unmeasured, so design measures it first.
- [x] **design**: `prompts/SPEC-141-design.md` (2026-10-30, Opus). [S], no `src/` change. In-process seam
  pins 42 pixel rows + 6 PNG/WebP byte rows; the binary checks the seam (2 rows) and the bundled `optimize`
  format (3 rows, `rgb8` only, cfg default codecs). RGBA fixtures enlarged to 1730×2600 / 2600×1730: the frame's
  1100×700 left 6 bundled resizes as no-ops. Both outcomes (shared vs per-target goldens) specified; M1–M7 driven/listed.
- [ ] **build**: `prompts/SPEC-141-build.md`
- [ ] **verify**: `prompts/SPEC-141-verify.md`
- [ ] **ship**: `prompts/SPEC-141-ship.md`
