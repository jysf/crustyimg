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
- [x] **build**: `prompts/SPEC-141-build.md` (2026-10-30, Sonnet, ~$4.51). Draft PR #233, code head `886d388`.
  Outcome (ii): checkpoint 1 went red on Linux and Windows, so goldens are per target (42 files). 12 of 14 recipes
  differ pairwise across targets (every resampler); the text watermark and the format recipe match everywhere.
  M1–M7 driven on the shared layout; M4's line-count literal was never seen red alone. Empty `src/` diff.
- [x] **verify**: `prompts/SPEC-141-verify.md` (2026-10-30, Opus, $7.54). ⚠ PUNCH LIST at `dfaa28b`: all
  mutations matched (M4b proves the 51 literal live), goldens are stable per target, CPU dispatch ruled out; the
  goldens pin each runner image's libm `powf`. Gaps: a missing golden prints no CI-log dump, FT-3 skips the control
  lines, and the README doesn't name the literals to bump.
- [x] **punch list**: `prompts/SPEC-141-punchlist.md` (2026-10-30, Sonnet, $2.31). PR #233 code head `f6d297b`, CI green.
  A missing golden now fails with its CI-log dump (FT-9); FT-3 covers the control lines; README names every literal.
- [x] **re-verify**: `prompts/SPEC-141-reverify.md` (2026-10-31, Opus, $2.74). ⚠ PUNCH LIST at `6b90d71`: the
  README's procedure alone took a simulated new recipe green, but FT-9 never calls `golden_path_for`, so the old
  `|| !path.exists()` panic put back verbatim left every test green.
- [x] **punch list 2**: `prompts/SPEC-141-punchlist-2.md` (2026-10-31, Sonnet, $0.69). Code head `6376d65`, PR head
  `dec2c6a`. `a_target_without_goldens_fails` asserts a known target with an absent file returns the exact `Ok` path;
  the stray-file check names missing and extra files; FT-4 names the golden.
- [x] **re-verify 2**: `prompts/SPEC-141-reverify-2.md` (2026-10-31, Opus, $0.82). ✅ APPROVED at `dec2c6a`: mutation
  (A) killed (exit 101, only the extended test red), full suite 1124/0/6. The orchestrator updated the branch from
  `main` (SPEC-143's #235 had landed) and marked PR #233 ready at `d96995c` once that CI run was green.
- [ ] **ship**: `prompts/SPEC-141-ship.md`
