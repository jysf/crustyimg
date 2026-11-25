# SPEC-151 — VERIFY prompt: `pad` as a recipe step

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-151. Your job is to find what the build's own evidence can't show. CI green
and 83 ticked ACs are claims until you drive them.

```
just wt verify/spec-151 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-151-pad:refs/remotes/origin/feat/spec-151-pad
git reset --hard origin/feat/spec-151-pad
```

Draft PR #255's head was `ef6befa` when this was written. CI was green on every leg at `746733c`;
`ef6befa` adds the spec's Build Completion and cost, and strips ids from the hoisted cap comments.
Record the SHA you verify. The maintainer will paste your final message back to the orchestrator,
so it must stand on its own. **Write it as plain lists, not box-drawn tables**: a wide table was
cut off in transit last time.

## What the build claims
- **ACs:** 83 of 83 met.
- **Red baseline:** `tests/pad.rs` 17 of 17 red with `unknown operation 'pad'`; the 3 registry rows
  red; with a lenient `orientation_of`, only `orientation_of_fails_on_a_malformed_exif_chunk` red.
- **Mutations:** all 15 rows (M1–M14 with M5a/M5b and M8b) driven on a scratch worktree of
  `746733c`, native only. No cell of the design's table was wrong. Extra reds: M1
  `apply_pads_a_png_end_to_end`, M4 `float_images_keep_their_variant`, M8b
  `colour_type_table_row_by_row`. M12: 23 red. M14: 0 red, as the table predicted.
- **Corpus:** `cmp` identical across the three targets for both recipes; no existing `.output` or
  `.golden` changed (`--diff-filter=MD` lists only the README; 10 files added).
- **wasm:** 53 full / 51 slim, locally and in CI. Self-test 18/18, workflow 16/0, front-matter OK.
  Slim bundle raw 4,137,932 B, brotli 829,807 B; no `main` baseline, so no delta.
- **Matrix:** default, lean and `webp-lossy`, each in a fresh target dir: clippy, fmt and
  `cargo test --no-fail-fast` all 0, 56 test binaries each.
- **Deviations:** the canvas formula lives in its own `aspect_canvas`, which `layout` calls, and
  AC-3's sweep calls `aspect_canvas` (because `layout` refuses the extreme aspects at the caps);
  ids stripped from the hoisted cap comments; `common::orientation_of` has 3 callers, not 4.
- **Out-of-spec:** the area cap counts pixels, so a padded `Rgba32F` canvas at the cap is 2 GiB.
- **Files:** the build says "31 files, plus this spec = 32". `gh pr view 255 --json files` listed
  31 paths **including** the spec when this was written. Count it yourself.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-151-pad-as-a-recipe-step.md`. Read
   `## Frame`, `## Design` (its six corrections, the colour-type table, the gravity table and the
   exact messages), the ACs, Failing Tests, the mutation table and Build Completion.
2. DEC-095, DEC-102, DEC-112, DEC-113, DEC-003.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" (rule 7) and
   "Traps this wave paid for" (traps 10, 12, 13, 14 and 17 especially).

## Maintainer rulings the build had to honour
1. `transparent` is not this spec: `color` must be opaque, and `transparent` or any alpha below `ff`
   is refused at load with its own message, which names no later spec and promises nothing.
2. `color` is required; there is no default.
3. `border` is a fraction of the **long** edge, the same width on every side.
4. A grey image with a chromatic fill widens to RGB; a grey fill keeps it grey.
5. No refusal message or doc line names a stage, spec or DEC id.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` against the spec's `## Outputs` plus the spec. Name
     every extra or missing file, and settle the 31-or-32 count.
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty; `CACHE_SCHEMA_VERSION` and the
     lockfile format are unchanged (AC-82).
   - `crop.rs` changes by one line (AC-72). The `MAX_EDGE`/`MAX_AREA` hoist leaves resize's messages
     and behaviour unchanged (AC-71): read the hunk, and name the test that pins resize's cap text.
   - Every commit is signed off.
   - `main` has moved since the PR's base (PROJ-011's SPEC-149 and SPEC-150 are in flight). Say
     whether the PR still merges cleanly, and whether any commit on `main` since the base touches a
     file this PR changes. In particular, check `scripts/wasm-ci-check.sh`'s `FULL_COUNT` and
     `SLIM_COUNT` and `tests/wasm_roundtrip.rs` on current `main`: if `main` added a wasm test, 53/51
     is now wrong after merge, and that's a punch item.
2. **The messages, mechanically.** Extract the 37 load messages (P1–P37) and the 5 apply reasons
   (A1, A2, A2b, A2c, A3) from the spec's tables, and the same from the code and `tests/pad.rs`. Diff
   them by script, not by eye; the spec's table elides a prefix on P4–P13, so expand it first. Report
   every byte that differs. Confirm the test asserts 37 rows ran. Then `/usr/bin/grep` every new
   user-facing string for `SPEC-`, `STAGE-`, `DEC-` and `PROJ-` (ruling 5, AC-81), including the
   hoisted cap comments and the `Operation`/`refusal` doc comments.
3. **The canvas, against an oracle you write.** In Python with exact rationals (`fractions`), write
   the canvas rule from the spec's prose (Design correction 1: the binding axis, round half up), not
   from the code. Then:
   - Read `aspect_canvas` and `layout`. Confirm `layout` **calls** `aspect_canvas` (not a copy), and
     that AC-3's test calls the same function, asserts 53,248 cases and at least one tie.
   - Drive the real binary on these and compare the output dimensions with your oracle: 4000×4000 →
     `16:9` and `9:16`; 6000×4000 → `4:5`; 1730×2600 → `4:5`; 1919×1080 → `16:9`; 1200×801 → `3:2`
     (the tie: 1202×801); 1918×1079 → `16:9` (a no-op). Use small stand-ins only where the size
     is irrelevant to the rule; keep the tie and the no-op at their real sizes.
   - The no-op: the output pixels equal the input's, and an `L8` input with `#123456` that needs no
     padding stays `L8` (read the PNG header yourself).
4. **Pixels, against an independent tool.** Use a release build from this branch or
   `CARGO_BIN_EXE`'s binary, never one on `PATH`. Make small, odd, asymmetric PNGs with ImageMagick
   or Python stdlib, in 8-bit RGB, 16-bit RGBA and 8-bit grey.
   - Aspect form: compare with ImageMagick
     `-background '<fill>' -gravity <g> -extent <cw>x<ch>` at AE 0, for `center` (an odd
     remainder, so the floor matters), `northwest` and `southeast`. If ImageMagick's centre rounds
     differently from the spec's floor, say so and compare against the spec's offset table instead;
     don't call it a bug in either.
   - Border form: compare with `-bordercolor '<fill>' -border <px>` at AE 0, with `px` from the long
     edge (ruling 3).
   - 16-bit fill: read a fill pixel's raw samples; `#123456` must be `0x1212, 0x3434, 0x5656`.
   - Grey + chromatic fill → RGB, grey + `#808080` → grey, grey + `#808081` → RGB (ruling 4). Read
     each output's colour type and depth from the PNG header independently.
   Quote each comparison.
5. **Apply failures and batches, driven.** On the binary: A1 (`border = 0.001` on 40×30) exits 1
   with the exact line and writes nothing; a two-input `--out-dir` batch with one 0 px input and one
   that pads exits 6, prints `error: 1 of 2 inputs failed`, and writes only the good one; the same
   for an area-cap input (A3 shape). Check the boundary pair `0.0124` (fails) and `0.0125` (adds
   1 px) on 40×30.
6. **The metadata rule, driven (DEC-113).** Tag a PNG with `exiftool -Orientation#=6`, run a `pad`
   with no `auto-orient`, and read the output with `exiftool`: no Orientation tag, and the pixels are
   the stored pixels padded. Then `pad` then `auto-orient`: the tag still reaches `auto-orient`
   (the bundle survives `with_pixels`). Read AC-14's test and control B: would it catch a decoder
   that drops `eXIf`?
7. **Mutations, driven by you.** Don't re-run all fifteen. Drive these, each rebuilt clean
   (reverting source doesn't rebuild the binary), and compare with the table and the build's
   account:
   - **M1** (gravity floor → ceil, in the function crop shares): confirm crop's own test goes red,
     and that `v2_pad_aspect` stays green because `northwest` uses no centre;
   - **M3** (`×257` → `<< 8`): the corpus must go red **by pixel hash**, same dims (trap 13);
   - **M6** (0 px check removed): say what is written instead, as the table claims (the input,
     unpadded, exit 0);
   - **M12** (trap 14: restore `main`'s `with_builtins` verbatim, not a one-line delete): the build
     says it removed the registration line. Restore the whole function from `main` and say whether
     the red set is still 23;
   - **M14** (`get_f32` for `get_f64`): the table says all green, meaning no test guards DEC-112's
     `f64` rule here. Try to refute that: search for a `border` value and image size where the `f32`
     and `f64` readings round to different pixel counts within the load range. If you find one,
     it's an **assert** item (a missing test); if the search is exhaustive and finds none, say over
     what range.
8. **The folded entries (AC-68 to AC-70).**
   - `orientation_of` in `tests/common/mod.rs` returns `None` only on `Err(exif::Error::NotFound(_))`
     and panics on any other error. Read it; then say whether `orientation_of_fails_on_a_malformed_exif_chunk`
     builds its malformed chunk independently of the code under test.
   - The corpus README's "Adding a recipe" section matches the spec's "README text (AC-69, word for
     word)" byte for byte. Diff it by script.
   - The refusal-hook doc comment matches "Doc comment text (AC-70)". Diff it by script.
9. **The corpus.**
   - `git diff --diff-filter=MD --name-only origin/main...HEAD -- tests/fixtures/recipe_corpus/`
     lists only `README.md`.
   - `cmp` the three per-target files for each new recipe.
   - Read both `.golden` files: keys in `to_toml`'s order, and `recipe_round_trip` green on them.
   - Check the literals (`RECIPES` 20, `EXPECTED_DIMS` 20, `ROWS` 66, `GOLDEN_LINES` 69, the `60`,
     `pairs.len()` 20) against a count you make yourself.
   - Recompute the two recipes' expected dims on the three fixtures from your step-3 oracle and the
     border rule, and compare with `EXPECTED_DIMS`.
10. **Docs (AC-73 to AC-80).** Read each surface and say whether it says what the AC says. `color`
    must read as required, `border` as a fraction of the long edge, and no surface may promise
    `transparent`. `tests/docs_ops.rs` must resolve every new example.
11. **wasm.** Run `just wasm-test` and `just wasm-test-slim` once each, and quote the result lines.
    Read the two wasm tests: do they assert exact pixels and the native refusal text (AC-65, AC-66)?
    Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `… workflow`. If it's cheap,
    build `main`'s slim bundle in a separate target dir and give the size delta the build couldn't.
12. **The out-of-spec finding.** The build says a padded `Rgba32F` canvas at the area cap is 2 GiB.
    `resize` shares the same cap: say whether `resize` already allows the same 2 GiB on `main`, which
    would make this a pre-existing gap rather than one `pad` opened. Reason it from the code; don't
    allocate 2 GiB to prove it.
13. **Full suite.** Run the default leg once and the lean leg once, each in a fresh target dir, with
    `--no-fail-fast`. Redirect stdout and read `$?`.
14. **CI.** Run `gh pr checks 255` once at the head, and say whether every leg is green.

## Classify every finding (HANDOFF rule 7)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or table cell that misstates something the code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list. For every bound, maximum or "can't happen"
in the spec (the ½ px bound, "no value overflows a `u64`", "apply never allocates before the caps"),
say whether it was driven, by whom, and over what range (trap 17).

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session; PROJ-011's SPEC-149 build may be running.
  Never poll CI. Never drive the maintainer's browser.
- Don't allocate a canvas at the caps to prove a cap; the caps are checked before allocation.
- Delete scratch target dirs and scratch images, and leave the tree clean.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-151 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. First line, exactly one of:
- `✅ APPROVED — merge PR #255 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #255 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–14, one or two lines each, with evidence;
- the punch items, numbered, each with its kind (wording / assert / claim), its evidence and the
  fix you'd expect;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with the stage it belongs to;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
