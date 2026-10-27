# SPEC-133 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-133-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-133-frame.md`). **GO, M (top of M).** Premise
      confirmed on `main` @ `8471fb1` (24 px `--size 200` → 6 px changed, exit 0; long string on
      256 px clipped at 3.73 %; self-diff 0). Image overlays clip too; recipe text accepts
      `tile`/`scale` the CLI refuses (in scope). `--min-size` split to its own backlog item. Size
      is driven by the batch *skipped* outcome across 5 driver loops + `build`'s lockfile/cache.
      Sequential after SPEC-131 (`src/operation/mod.rs`).
- [x] **design** — 2026-10-25 (`prompts/SPEC-133-design.md`). 27 ACs, 33 new tests (1 is the bless generator) + 12 existing
      tests re-fixtured (measured by a throwaway prototype run: 12 flips in 5 files). Fit (margin on
      anchored axes, after scale) + 5 % of the mark's own non-transparent pixels (not changed
      pixels — row CNT shows those depend on the image); `tile`/`scale` coverage-exempt. Seam: a
      defaulted `Operation::refusal` + `Pipeline::run_with(Fail|Skip)`. Cached `build` does not
      replay the skip line (documented; filed). Text-mode `scale`/`tile = true` refused inside
      SPEC-131's mode check — edits two SPEC-131 test rows. Builds after SPEC-131; not parallel
      with SPEC-134. DEC-104 at build.
- [x] **build** — 2026-10-26 (`prompts/SPEC-133-build.md`). Opus. Draft PR #206 at `3dd23ff`, CI
      green on all 16 legs. 27/27 ACs; N1–N9 flipped; goldens blessed on untouched `src/`
      (`ab94505`). Deviations: corpus recipe `v2_watermark_integer_numbers` lost `scale = 1` (a
      text step; needs verify's ruling); 4 predicted flips passed hollow under Skip, re-fixtured
      and probed. `--size nan` item stays open. $22.18 (measured before the PR).
- [x] **verify** — 2026-10-26 (`prompts/SPEC-133-verify.md`). Opus. **⚠ PUNCH LIST** at `3dd23ff`.
      Code correct on every surface; passing marks byte-identical to `main` (9 cases); skip
      probe over all integration tests hits only the 8 skip tests. Mutations M1 (16-bit count),
      M2 (count after opacity — AC-5 hollow), M5 (K per step), M7 (judge the original input)
      survive. Docs: `--help` 5 % sentence false for `--tile`; glob/dir-of-one → exit 2
      undocumented; a CHANGELOG sentence describes behaviour no release had. Ruling: the recipe
      `text` + `scale` refusal breaks no released user (0.7.1 has no recipe watermark). $11.16.
- [x] **fix** — 2026-10-26 (`prompts/SPEC-133-fix.md`). Sonnet. PR #206 at `458254d`, CI green.
      M1/M2/M5/M7 each killed by a new or corrected test (RED/GREEN in fresh target dirs);
      `--help`, glob/dir-of-one and CHANGELOG fixed; cleanups 8–12 done. Out-of-spec:
      `docs/data-model.md:248` still says "one explicit input" — sent back to the fix session.
      $2.72.
- [x] **re-verify** — 2026-10-27 (`prompts/SPEC-133-reverify.md`). Opus. **✅ APPROVED** at
      `f1a46d3` (after a one-line `docs/data-model.md` fix). M1/M2/M5/M7 each RED then green;
      the refactor's stderr byte-identical to `3dd23ff` over six batches; SPEC-131's moved rows
      fail for their own reason. Three optional test gaps + a path-label inconsistency filed on
      STAGE-050. $3.22. **Merged 2026-10-27 as `75f82e7` (PR #206).**
- [~] **ship** — prompt: `prompts/SPEC-133-ship.md`. Sonnet, own worktree `docs/spec-133-ship`.
