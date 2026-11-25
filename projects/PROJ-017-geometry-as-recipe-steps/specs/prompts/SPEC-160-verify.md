# SPEC-160 — VERIFY prompt: edit --pad and --trim, and parse_color refuses a sign

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-160. Your job is to find what the build's own evidence can't show. CI green
and 16 ticked ACs are claims until you drive them.

```
just wt verify/spec-160 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-160-edit-pad-trim:refs/remotes/origin/feat/spec-160-edit-pad-trim
git reset --hard origin/feat/spec-160-edit-pad-trim
```

Draft PR #264's head was `23a5c0e` when this was written. CI was green on every leg there (18
passed, 8 release jobs skipped), as checked by the orchestrator. Record the SHA you verify. The
maintainer will paste your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

## What the build claims
- **ACs:** 16 of 16 met.
- **Red baseline on `main`:**
  - `tests/edit_pad_trim.rs` failed 13 of 13, on clap's unknown `--pad`/`--trim`.
  - FT-8's `--pad` row exited 2 on `main`; its crop, resize and watermark rows gave 1, 1 and 2.
  - FT-17 passed on `main`, as expected for a test-only pin.
  - FT-18 failed (help text and the no-op line).
  - The lib tests didn't compile until `IntroducedAlpha.flag` and `EditOp::Trim` existed. With
    those removed, FT-16 failed on `+f+f+f`.
  - The first FT-13 lacked `version = "1"` and was fixed in the next commit.
- **CI:** the first run's Windows leg failed. The build's own tests asserted `/in/a.jpg` with
  forward slashes, and the build "normalised the separators in those assertions".
- **Mutations:** 22 rows, each driven in a scratch worktree with a clean rebuild and the full
  suite. 16 rows match the table, and M14 stays green. Four cells were incomplete:
  - **M6a** also reds `tests/trim.rs` `a_non_ascii_colour_is_refused_not_a_panic` and
    `every_trim_refusal_names_its_reason`.
  - **M9** also reds FT-4 and FT-9.
  - **M10a** also reds `tests/pad.rs` `every_colour_type_pads_exactly` and
    `gravity_corners_land_by_absolute_position`.
  - **M10b** also reds `every_recipe_output_matches_its_golden`.
  - **Trap 14:** with `main`'s two files restored, the lib tests fail to compile (E0063), and
    cargo stops there. So the table's integration rows for trap 14 did not run.
  - `no_op_flag_message_lists_every_flag` was red in no row.
- **Docs:** every behaviour sentence was driven on every exit path (list in Build Completion).
- **Matrix:** default 1350 passed, lean 1286 passed, `webp-lossy` exit 0. Clippy and fmt pass on
  all three in fresh target dirs. `just wasm-check` and `just wasm-test` pass (56 passed).
- **Files:** 13, exactly the spec's `## Outputs`, plus the spec.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. Any mutation you drive stays
in your worktree and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-160-edit-pad-and-trim-and-parse-color-refuses-a-sign.md`.
   Read these sections:
   - `## Frame` with the maintainer rulings;
   - `## Design`: corrections 1–8, the code shape, "Every message, verbatim", the replay matrix,
     "What doesn't change", wasm and cache;
   - the ACs, the Failing Tests and the docs list;
   - the CHANGELOG text and DEC-115's ship-time replacement text;
   - the mutation table and Build Completion.
2. DEC-115 (`decisions/DEC-115-*.md`) on `main`, then DEC-015, DEC-095, DEC-102, DEC-106, DEC-112
   and DEC-114.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 5, 10, 12, 13 and 14.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. STAGE-058's entries 1–4 moved out; the nine-gravities pin is folded in, so this spec closes
   the stage.
2. [M], uncut.
3. One `--pad N|W:H` flag, matching `--crop`. No second flag.
4. Design corrections as rulings:
   - Named colours are not accepted; the colour is hex only.
   - The batch-wide format is `--format` or the template's literal extension, never `-o`.
   - No wasm row.
5. No message, help line or comment names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`. Give the real count and
     name every file outside the list.
   - Confirm these are unchanged: `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION`,
     `scripts/wasm-ci-check.sh`, `tests/wasm_roundtrip.rs` and DEC-115.
   - Check that every commit is signed off.
   - Check that the PR merges cleanly onto current `main`, and that no commit on `main` since the
     merge base touches a file this PR changes.
   - Run `gh pr list --state open --json number,files` and name every open PR that touches
     `src/cli/ops.rs`, `src/cli/mod.rs`, `src/operation/alpha.rs`, `src/text/mod.rs` or
     `CHANGELOG.md`. The build names #240, #237 and #226.
2. **The alpha rule against the code.**
   - Point to the code for each of the three gated checks:
     - one input: exit 4;
     - batch-wide `--format` or template extension: exit 4, no out-dir;
     - kept-format batch: exit 6 per input.
   - Point to the gate (`alpha_check = forced_format.is_none() && pipeline.introduces_alpha()` or
     its final form). Say what `forced_format` is on every route into `run_pixel_op`.
   - Say whether a forced format can ever skip a check it should run. For example, does
     `--format jpeg` with a translucent `--pad` still exit 4, and which line refuses it?
   - Then read DEC-115 on `main` and the spec's ship-time replacement text. For each part of the
     replacement, point to the code that does it, and say whether ship can apply the text as
     written. Any part the code doesn't follow is a finding against the spec, not against DEC-115.
   - Run `just decisions-audit` and quote its result.
3. **Every route through `run_pixel_op`.**
   - Enumerate the callers from the code yourself.
   - For each caller, say whether a pipeline there can report `introduces_alpha()`. If it can,
     name the check that covers it. If it can't, say why.
   - Drive FT-8's four rows on the binary and compare their exit codes with `main`'s binary built
     the same way.
   - Then drive one more row the tests don't have: `edit --pad 3 --pad-color transparent
     --format bogus`. Say which error wins, and whether that is what the spec's "Where each
     failure happens" implies.
4. **The messages, mechanically.**
   - Extract every line from the spec's "Every message, verbatim", and the same lines from the
     code and the tests. Diff them with a script, not by eye.
   - Drive each line on a release build of this branch (fresh `CARGO_TARGET_DIR`, never a binary
     on `PATH`). Compare stderr with `==` and check the exit code.
   - Add rows the tests don't have:
     - `--pad-color "#FF000080"` (upper case) into GIF: the colour is quoted as typed;
     - `--pad-color ff000080` (no `#`) into JPEG;
     - an aspect `--pad` into a template `{stem}.gif` batch with `transparent`: exit 0.
   - Then run `/usr/bin/grep` over every new user-facing string and comment for `SPEC-`, `STAGE-`,
     `DEC-` and `PROJ-`.
5. **`--pad N` as a fraction, driven independently.**
   - The frame drove `round((N/long)·long) == N` for every pair up to 50,000. Find the line that
     turns `border` back into pixels at apply and confirm it rounds the same way the frame's drive
     did. If it uses a different rounding (floor, ceil, `as u32`), drive the worst pairs again
     against the real code.
   - On the binary, save `--pad 7` on 60×40 and replay it with `apply` on the same image and on a
     120×80 copy. Say what the second gives (14 px?) and whether `docs/recipes.md` says so.
   - Check the long-edge refusal at exactly `N = long` (exit 0, border 1) and at `long + 1`
     (exit 1), on a portrait and a landscape image.
   - Decode the padded outputs with a decoder you didn't write and that isn't `image`
     (ImageMagick, or a stdlib Python PNG decoder). Check the fill pixel and that every source
     pixel is copied unchanged.
   - For each colour type, give the fill's 16-bit value as `c × 257`.
6. **The replay matrix's expected values.**
   - Confirm that FT-1 to FT-4's sizes and colour types are literals, and that no expected value
     comes from the code under test.
   - Confirm that `framed(kind)` and `noise` are built in the test and not through crustyimg code.
   - Then drive three FT-1 rows on the binary: `l16` aspect, `la8` transparent and `rgba16`
     `--trim --trim-color 111111 --trim-tolerance 1`. Read the colour type and size with
     ImageMagick.
7. **`parse_color`, every route.**
   - Read the new check and say what it accepts byte by byte.
   - Drive these on all six routes FT-13 names, plus `build` with a recipe:
     - the four signed strings;
     - `""`, `"#"`, `"#fff"`, `"ffff"`, `"fffffff"`, `"#ff00ff "` (trailing space) and
       `" #ff00ff"`;
     - `"０f0f0f"` (full-width digit), `"#ff00ffé"` and `"0x0f0f"`.
   - Each must be refused with its exact line, or accepted only if every byte after an optional
     `#` is ASCII hex and the length is 6 or 8.
   - Then confirm that `parse_color` has no target-dependent code (`cfg`, float, time), which is
     why the design gave it no wasm row. If the build left `just wasm-test` green, say so; you
     need not drive wasm yourself.
8. **`--trim`.** Check the whole-match line with and without `--trim-color` and
   `--trim-tolerance`, and the batch row (exit 6, the other input written). Then:
   - say whether the tolerance's 16-bit scaling (×257) is used on an L16 input, with one driven
     row;
   - drive `--trim --trim-color` with an alpha colour (`11111180`) on an RGBA input, and say what
     the docs say it does.
9. **Batch `--save-recipe` (AC-11).**
   - Drive `--pad N` alone and `--pad N` before a pixel `--crop` on a two-image directory: exit 2,
     the exact line, no out-dir, no recipe.
   - Drive the aspect form and `--trim` from the same directory: they save.
   - Drive one file and stdin `-`: they save.
   - Confirm SPEC-153's crop line is byte-identical to `main`'s.
10. **Watermark's nine gravities (FT-17).** Work the nine offsets out yourself, in Python from the
    rule (free space 37×28, centre floors, margin 3 from each edge, centre ignores the margin).
    Compare them with FT-17's literals and with the binary's output, read with ImageMagick.
11. **The Windows fix.** Read the commit that made CI green. For each assertion it changed, say
    whether it is as strong as before. An `ends_with` on a normalised path is fine; a `contains`
    on a shorter fragment, or a dropped line, is a finding (assert).
12. **Mutations, driven by you.** Don't re-run all 22. Drive the rows below. Rebuild clean after
    each one (reverting source doesn't rebuild the binary). Use the **exact edit the spec quotes**,
    run the full suite with `--no-fail-fast`, and compare the red set with the table and with the
    build's account.
    - **M4** (gate removed): which rows go red, and does FT-8 catch it on its own?
    - **M2** (batch-wide pre-check removed): FT-6 reds on the fallback's exit 6, and not on
      something else.
    - **M9** (fraction on the short edge): confirm the build's extra FT-4 and FT-9 cells.
    - **M10b** (bottom margin dropped): confirm `every_recipe_output_matches_its_golden` reds, and
      name the recipe that catches it.
    - **M6a** (`from_str_radix` on the raw pair): confirm the two `tests/trim.rs` cells.
    - **Trap 14:** restore `main`'s `src/cli/ops.rs` and `src/cli/mod.rs`, then run the
      integration targets on their own (`cargo test --test edit_pad_trim --test edit_geometry
      --test watermark`), which don't need the lib's unit tests to compile. Say whether the
      table's integration rows hold: FT-1 to FT-13 red, FT-17 and all 23 `edit_geometry.rs` tests
      green. If the integration targets don't build either, say what fails.
    - Then confirm the tree is restored (`git status` clean, the suite green).
13. **Docs (AC-15) and CHANGELOG, each sentence driven.**
    - Read every row of the spec's docs list on the PR: `docs/cli-reference.md`,
      `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md` and `CHANGELOG.md`.
    - For **each sentence that states behaviour**, quote it and drive it on the binary, or say
      which earlier item drove it. Pay attention to:
      - every exit-code row (2, 4, 6 and 1) in `api-contract.md` and `cli-reference.md`;
      - any sentence on what a saved `border` does on a different-sized image (item 5);
      - the colour sentences: hex only, and a sign refused.
    - SPEC-153's only defects were doc sentences nobody drove. Treat these as the likeliest place
      for one.
    - CHANGELOG:
      - a user should understand each line without the spec;
      - check the library-only Breaking line against the code: is `IntroducedAlpha` public and
        constructible outside the crate, so that adding `flag` breaks a caller?
    - `tests/docs_ops.rs` passes.
14. **Full suite.** Run the default leg once and the lean leg (`--no-default-features`) once. Use a
    fresh target dir for each, and `--no-fail-fast`. Redirect stdout and read `$?`. Show the
    `tests/edit_pad_trim.rs` counts in both legs.
15. **CI.** Run `gh pr checks 264` once at the head, and say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment, DEC front-matter field or table cell that misstates something the
  code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom, and over
what range. Examples:
- the rounding bound up to 50,000;
- "no splitting input exists" for M14;
- "a forced format makes the check moot";
- "no stale cache entry can be served";
- the 30 + 3 + 1 replay rows.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters, and macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Never poll CI and never write your own
  wait loop (AGENTS §16 habit 6). Never drive the maintainer's browser.
- Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  Delete scratch target dirs and scratch images, and leave the tree clean.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-160 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #264 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #264 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–15, one or two lines each, with evidence;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the mutation-table cells ship should correct, one line each;
- the DEC-115 replacement text: apply as written, or the change it needs;
- out-of-spec findings, each with the stage it belongs to (STAGE-058 closes with this spec, so
  name another stage or `guidance/ideas.yaml`);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
