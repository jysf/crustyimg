# SPEC-147 — VERIFY prompt: `rotate` by quarter turns and `flip`, as recipe steps

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-147. Your job is to find what the build's own evidence can't show. CI green
and 58 ticked ACs are claims until you drive them.

```
just wt verify/spec-147 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-147-rotate-flip:refs/remotes/origin/feat/spec-147-rotate-flip
git reset --hard origin/feat/spec-147-rotate-flip
```

Draft PR #252's head was `ceda510` when this was written. CI was green on every leg at
`f2f5b59`, the last code commit; `ceda510` adds only the spec's Build Completion, cost and cycle.
Record the SHA you verify. The maintainer will paste your final message back to the orchestrator,
so it must stand on its own.

## What the build claims
- **ACs:** 58 of 58 met.
- **Red baseline:** against `main`, `tests/rotate_flip.rs` was 0 passed, 14 failed, with
  `unknown operation 'rotate'` / `'flip'`. The orientation controls A and B passed, so the PNG
  `eXIf` chunk reaches the metadata bundle.
- **Mutations:** all 12 rows driven (M1–M10, M6b, M7b), and every red set matched the table, with
  two exceptions:
  - M8 row 3: `count full 50` goes red as a wrong verdict, not as "message lacks expected 51";
  - M10 also turns `to_toml_is_byte_identical_to_main_over_the_corpus` red, which the table didn't
    list.

  The build drove them in a scratch worktree of the pushed head.
- **Corpus:** the second CI run was green on every leg. `cmp` is clean across the three targets
  for both recipes, and no existing file changed (`--diff-filter=MD` empty, 10 files added).
- **wasm:** `just wasm-test` 51 passed and `just wasm-test-slim` 49 passed, locally and in CI.
  `--self-test` 18/18, `workflow` 16/16, front-matter clean.
- **Matrix:** default 1180, lean 1117 (the new tests ran there), `webp-lossy` 1184. Bundle +15,469 B
  raw, +2,423 B brotli.
- **Deviations:** `tests/common/mod.rs` changed but isn't in `## Outputs` (the Failing Tests section
  asks for it); the unit tests landed with the ops, not in the red commit; one extra test,
  `every_step_matches_its_index_map_in_every_colour_type`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-147-rotate-by-quarter-turns-and-flip-as-recipe-steps.md`.
   Read `## Frame`, `## Design`, the ACs, the Failing Tests, the mutation table and Build
   Completion.
2. DEC-113 (the metadata rule, new), DEC-003, DEC-095, DEC-102, DEC-110, DEC-112.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" (rule 7 is new)
   and "Traps this wave paid for" (traps 10, 12, 14 and 17 especially).

## Maintainer rulings the build had to honour
1. A positive `angle` is clockwise; `-90` turns anticlockwise.
2. `flip` keeps `axis = "horizontal" | "vertical"`; `horizontal` mirrors left and right.
3. No refusal message or doc line names a stage, spec or DEC id.
4. `angle = 0`, `360` and `-360` load as a no-op.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` matches the spec's `## Outputs` list plus the spec
     and `tests/common/mod.rs`. Name every other extra or missing file, and read the
     `tests/common/mod.rs` hunk: is it only what the Failing Tests section asks for?
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty, and `CACHE_SCHEMA_VERSION`
     and the lockfile format are unchanged (AC-57).
   - The `src/sink/mod.rs` hunk is a comment only (AC-40).
   - Every commit is signed off.
   - `main` has moved since the PR's base. Say whether the PR still merges cleanly, and whether any
     commit on `main` since the base touches a file this PR changes.
2. **The messages, mechanically.** Extract every refusal message from the spec's "Exact
   messages" table, and the same messages from the code and the tests. Diff them by script, not by
   eye. Report every byte that differs. Confirm each R- and F-row has its own case, and that the
   tests assert the row count. Then `/usr/bin/grep` every new user-facing string (messages, doc
   comments, doc lines) for `SPEC-`, `STAGE-`, `DEC-` and `PROJ-` (ruling 3).
3. **Direction and axis, driven through the real binary.** Use a release build from this branch
   or `CARGO_BIN_EXE`'s binary, never one on `PATH`.
   - Make a small non-square, asymmetric PNG with a tool other than crustyimg (ImageMagick or
     Python stdlib), in 8-bit RGB and in 16-bit RGBA.
   - Run `rotate` at 90, -90, 180 and 270, and `flip` on both axes. Compare each output
     pixel-for-pixel with an independent tool's result: ImageMagick `-rotate 90` (clockwise) and
     `-flop` (left↔right; ImageMagick's `-flip` is top↔bottom). Quote each comparison.
   - Check that colour type and bit depth are kept (AC-8): read the output's PNG header
     independently.
   - Drive `angle = 0`, `360` and `-360`: the output pixels equal the input's (ruling 4).
4. **The metadata rule, driven (DEC-113).**
   - Tag a PNG with `exiftool -Orientation#=6`. Run `rotate angle = 90` with no `auto-orient`.
     Read the output with `exiftool`: there's no Orientation tag, and the pixels are the stored
     pixels turned clockwise.
   - Run `auto-orient` then `rotate`, and `rotate` then `auto-orient`. Say whether the second gives
     the same pixels as the first, as AC-39 claims, and explain why from the code (the bundle kept
     through `with_pixels`).
   - Read the orientation test (AC-37) and its two controls. Say whether control B would catch a
     decoder that drops `eXIf`, as the build's baseline implies.
5. **Mutations, driven by you.** Don't re-run all twelve. Drive these, each rebuilt clean, and
   compare what goes red with the spec's table and the build's account:
   - **M1** (direction: `rotate270` for `rotate90`): say which round-trip tests stay green, which
     is why the position test exists;
   - **M6b** (`f32` read): `1e-300` must load as a no-op under the mutation, and R16 go red;
   - **M7b** (bundle dropped): AC-39's test must go red;
   - **M10** (restore `main`'s `with_builtins` verbatim, trap 14), including the extra row the
     build found.
   - **M8** (wasm count): set `FULL_COUNT=50` and run `--self-test` and a `count full` on a
     51-passed log. Check the build's corrected cell for row 3.
6. **The normalisation table.** Read the test that covers it. Each accepted value in
   {−360, −270, −180, −90, 0, 90, 180, 270, 360}, plus `90.0`, `-90.0` and `-0.0`, has its own
   row with its expected quarter-turn count.
7. **The corpus.**
   - `git diff --diff-filter=MD --name-only origin/main...HEAD -- tests/fixtures/recipe_corpus/`
     is empty.
   - Run `cmp` on the three per-target files for each new recipe.
   - Read both `.golden` files: keys in `to_toml`'s order, and `recipe_round_trip` green on them.
   - Check the literals (`RECIPES` 18, `ROWS` 60, `GOLDEN_LINES` 63, the `54`, `pairs.len()` 18)
     against a count you make yourself.
8. **Docs (AC-48 to AC-56).** Read each surface at its `file:line`, and say whether it says what
   the AC says. The direction sentence and the `flip` sentence (rulings 1 and 2) must be there,
   and the `rotate` entry must say to put `auto-orient` first for a recipe meant as displayed.
   `tests/docs_ops.rs` must resolve every new example.
9. **wasm.** Run `just wasm-test` and `just wasm-test-slim` once each, and quote the result lines.
   Read the two wasm tests: do they assert exact pixels and the load-refusal text (AC-45, AC-46)?
   Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `… workflow`.
10. **`written` moved into `registry.rs`.** `crop.rs` imports it and its copy is gone. `crop`'s
    tests are unchanged on this branch (AC-58): `git diff origin/main...HEAD -- tests/crop.rs` is
    empty.
11. **Full suite.** Run the default leg once and the lean leg once, each in a fresh target dir,
    with `--no-fail-fast`. Redirect stdout and read `$?`.
12. **CI.** Run `gh pr checks 252` once at the head, and say whether every leg is green.

## Classify every finding (HANDOFF rule 7)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or table cell that misstates something the code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list. If a bound, maximum or "can't happen"
appears anywhere in the spec or DEC-113, say whether it was driven (trap 17).

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session; the PROJ-011 orchestrator is running its
  own builds. Never poll CI. Never drive the maintainer's browser.
- Delete scratch target dirs and scratch images, and leave the tree clean.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-147 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #252 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #252 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–12, one or two lines each, with evidence;
- the punch items, numbered, each with its kind (wording / assert / claim), its evidence and the
  fix you'd expect;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with the stage it belongs to;
- your cost, with the token breakdown.
