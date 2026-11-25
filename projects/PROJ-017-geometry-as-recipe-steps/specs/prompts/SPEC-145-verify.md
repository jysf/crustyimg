# SPEC-145 — VERIFY prompt: `crop` as a recipe step, in fractions of the image

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-145. Your job is to find what the build's own evidence can't show. CI green
and 65 ticked ACs are claims until you drive them.

```
just wt verify/spec-145 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-145-crop:refs/remotes/origin/feat/spec-145-crop
git reset --hard origin/feat/spec-145-crop
```

Draft PR #244's head was `6611e79` when this was written, with every CI leg green. Record the SHA
you verify. The maintainer will paste your final message back to the orchestrator, so it must
stand on its own.

## What the build claims
- **ACs:** 65 of 65 met.
- **Mutations:** M1–M12 each applied, rebuilt clean, run and restored.
  - M1, M3, M3b, M5, M7, M8, M9, M10 and M11 matched the table exactly.
  - M2, M4, M6 and M12 turned extra rows red as well.
  - **M4b behaved differently from the spec:** a single-input `apply` exits 2 with `crop refused:`,
    while a two-input batch exits 0 and writes both images uncropped.
- **Corpus:** the second CI run (`da31660`) was green on all 13 jobs. The three per-target
  `.output` files are byte-identical for both crop recipes, and no existing `.output` or `.golden`
  changed.
- **Local matrix:** default 1156/0, lean 1093/0, `webp-lossy` 1160/0. `just wasm-test`:
  `49 passed; 0 failed`. Bundle +17,270 B raw, +3,271 B brotli.
- **Deviations:**
  - An early rectangle golden used the spec's key order, not the alphabetical order `to_toml`
    writes. It was fixed in a later commit.
  - The wasm tests are two functions holding five rows, not "three rows".
  - The gravity unit test was changed to report apply errors per row.
- **Out-of-spec finding:** the batch refusal warning says "watermark skipped" for any op that
  refuses.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-145-crop-as-a-recipe-step-in-units-that-survive-a-resize.md`.
   Read `## Frame`, `## Design`, the ACs, the Failing Tests, the mutation table and Build
   Completion.
2. DEC-112 (the unit rule), DEC-091, DEC-095, DEC-102, DEC-109, DEC-110.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 11, 12, 14 and 15 especially).

## Maintainer rulings the build had to honour
1. Fractions only in a recipe.
2. `aspect` takes whole numbers only.
3. A crop that rounds to 0 px is refused at apply, never clamped. The message names `crop` and
   both sizes, with no step index (accepted 2026-11-05).

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` matches the spec's `## Outputs` list plus the
     spec itself. Name every extra or missing file.
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty, and `CACHE_SCHEMA_VERSION`
     and the lockfile format are unchanged (AC-64).
   - Every commit is signed off.
   - If `main` has moved since the PR's base, say whether the PR still merges cleanly.
2. **The messages, mechanically.** Extract the 36 refusal messages from the spec's "Exact
   messages" table, and the same 36 from the code and the tests. Diff them by script, not by eye.
   Report every byte that differs. Then confirm that each R- and A-row has its own case, and that
   the tests assert the row count.
3. **The thesis, driven through the real binary.** Use `CARGO_BIN_EXE`'s binary or a release build
   from this branch, never one on `PATH`.
   - Make a 6000×4000 image and a 1200×800 downscale with a tool other than crustyimg.
   - Run the same rectangle recipe on both. Scale the small result's edges by 5, and check they're
     within 1 px of the large result's. Check the large result's edges against DEC-112's formula
     by hand.
   - Repeat with one aspect recipe, on a landscape and a portrait image.
   - Check the output pixels are the input pixels at the offset. Compare with an independent
     tool (`vips`, ImageMagick or Python stdlib), not with crustyimg.
4. **Mutations, driven by you.** Don't re-run all twelve. Drive these, each rebuilt clean, and
   compare what goes red with the spec's table and the build's account:
   - **M1** (floor the right edge);
   - **M3b** (remove both the load check and the apply guard): the binary test must see exit 0
     and a written file, and still go red;
   - **M8** (f32 reading);
   - **M12** (restore `main`'s registry verbatim, trap 14).

   Then **M4b** (move the 0 px check into the refusal hook). The build found that a single-input
   `apply` exits 2 but a **two-input batch exits 0 and writes both images uncropped**. Reproduce
   both. Then, with the code restored, drive the same two-input batch on a 0 px crop. It must exit
   non-zero and write no uncropped image. If the shipped code lets any batch path write an
   uncropped image at exit 0, that's a punch item.
5. **The rounding tests can fail.** Read the two property tests: the seed, the case count and the
   bound. Say whether the bound (0.6 px) is tight enough that M1's 0.80 px would show up in any
   run, not just the seeded one.
6. **The corpus.**
   - `git diff --diff-filter=MD --name-only origin/main...HEAD -- tests/fixtures/recipe_corpus/`
     is empty.
   - Run `cmp` on the three per-target files for each crop recipe.
   - Read both crop `.golden` files. Keys are in `to_toml`'s alphabetical order, and
     `recipe_round_trip` is green on them.
   - Check the literals (`GOLDEN_LINES` 57, `ROWS` 54, `pairs.len()` 16) against a count you make
     yourself.
7. **Docs (AC-55 to AC-63).** Read each surface at its `file:line`, and say whether it says what
   the AC says. Check `docs/recipes.md` has no `crustyimg crop` verb left
   (`/usr/bin/grep -rn "crustyimg crop"` outside `target/`). The docs must also warn that
   `width = 1` means the full width.
8. **Wasm.** Run `just wasm-test` once yourself, and quote the result line. Read the five wasm rows,
   and say whether they assert exact pixels and the load-refusal text, as AC-53 and AC-54 require.
9. **The out-of-spec finding.** Find the "watermark skipped" warning in the code. Say whether any
   shipped op other than watermark can reach it today. If crop can't (it doesn't use the hook),
   it's a backlog item, not a punch item. Name the stage it belongs to.
10. **Full suite.** Run the default leg once, and the lean leg once, each in a fresh target dir,
    with `--no-fail-fast`. Redirect stdout and read `$?`.
11. **CI.** Run `gh pr checks 244` once at the head, and say whether every leg is green.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session; another orchestrator may be running
  PROJ-011 work. Never poll CI. Never drive the maintainer's browser.
- Delete scratch target dirs and scratch images, and leave the tree clean.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-145 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. You're read-only, so report the numbers; a later session writes them into
  `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #244 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #244 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–11, one or two lines each, with evidence;
- the punch items, numbered, each with its evidence and the fix you'd expect;
- optional cleanups, listed separately;
- out-of-spec findings, each with the stage it belongs to;
- your cost, with the token breakdown.
