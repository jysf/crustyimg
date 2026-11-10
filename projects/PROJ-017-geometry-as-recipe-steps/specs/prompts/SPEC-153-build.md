# SPEC-153 — BUILD prompt: `edit` flags for crop, rotate and flip, in typed order

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms (`EditOp`, `edit_op_order`, `EditFlags`, `PixelCrop`, `crop_fractions`,
`build_flag_op`), every message verbatim, a byte pin recorded from `main`, 27 ACs, 31 test
functions (86 rows) and a 15-row mutation table, all but a few cells driven at design on a
prototype. The code is two files; most of the work is the replay and refusal tables and the doc
surfaces. Be thorough across every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-153-edit-geometry-flags -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal. This spec is line 8 of PROJ-017's build order; `edit --pad`/`--trim`
follows it and closes STAGE-058.

## What you are building
- **Typed order** (`src/cli/mod.rs`): `run()` keeps the `ArgMatches`; `edit_op_order` reads each op
  flag's index, filtered to `ValueSource::CommandLine`, and sorts by it. `--auto-orient` is not
  ordered: orientation is always baked first, and the flag stays an accepted no-op that counts as an
  op flag.
- **Three flags:** `--crop WxH+X+Y|W:H` with `--crop-gravity` (aspect form only, `requires = "crop"`),
  `--rotate DEGREES` (`Option<i64>`, `allow_negative_numbers`), `--flip horizontal|vertical`.
- **`build_edit_ops`** builds steps in typed order through the registry, with `build_flag_op`
  swapping the op's quoted key for the flag name in its message.
- **`PixelCrop`**: converts pixels to fractions from the size of the image the step receives,
  refuses past the edge (A1/A2, exit 1), records the fractions it used.
- **`run_edit`**: the batch refusal (P16), and the recipe built **after** the run from what ran.
- **DEC-114 is already on `main`** (written at design). Don't edit it; if the code can't follow it,
  stop and report.
- **Tests:** `tests/edit_geometry.rs` (new), `tests/edit.rs` (FT-1 byte pin, FT-2 replacing
  `edit_flag_order_independent`), and the `src/cli/` unit tests (FT-27, FT-31 to FT-33).
- **Docs and CHANGELOG** as the spec's "Docs" section lists them.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `3be1af0` is in your history;
- `decisions/DEC-114-edit-runs-its-op-flags-in-the-order-typed-and-saves-what-ran.md` exists.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists: PR #240 (`src/cli/ops.rs` import block), PR #226 (`src/cli/mod.rs` near
`run()`), PR #237 (`CHANGELOG.md` `### Changed`). PROJ-011's SPEC-155 to SPEC-158 may open PRs on
`src/cli/` while you build. After every rebase, run
`gh pr list --state open --json number,title,files` and compare against that list; if a new PR
touches `src/cli/mod.rs`, `src/cli/ops.rs` or `tests/edit.rs`, say so in Build Completion. If a
rebase conflicts in a file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-153-edit-flags-for-the-five-geometry-ops-and-the-ordering-rule.md`.
   Read it in full: `## Frame` (for the why, and the maintainer rulings), `## Design` (especially
   "Design corrections to the frame", "Drives the prompt asked for", "The design" and "Every message,
   verbatim"), AC-1 to AC-27, the Failing Tests, `## Implementation Context` (overlap, docs and the
   mutation table) and `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-114 (this spec's rule), DEC-005, DEC-102, DEC-106, DEC-112, DEC-113. Read them
   yourself.
3. The code the spec cites at `file:line`: `Commands::Edit`, `run()` and the dispatch in
   `src/cli/mod.rs`; `run_pixel_op`, `build_edit_ops`, `run_edit` and the four unit tests in
   `src/cli/ops.rs`; `auto_orient_prefix` in `src/cli/optimize.rs`; `crop.rs`, `rotate.rs`,
   `flip.rs`; `Recipe::to_toml`.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10, 12, 13 and 14). These bind PROJ-017. Also `/guidance/constraints.yaml`.
5. SPEC-152 in `specs/done/`: its Build Completion and ship reflection. Its build drove a different
   mutant from the spec's for one row, and that row went unconfirmed until verify.

## Maintainer rulings (settled — do not re-open)
1. **Typed order** for every op flag; `auto-orient` always first and positionless; each op flag at
   most once (exit 2).
2. **`edit --invert --resize-max N` changes its output** (`0x62d0525855cff6a7` →
   `0x5d248c136b8b3ad7` on the pin fixture), with a CHANGELOG "Changed" line.
3. **`--crop` takes `WxH+X+Y` (pixels) and `W:H` (aspect) only.**
4. **A pixel crop past the edge of the image the step receives is exit 1**, naming the flag and that
   image's size.
5. **`--pad`/`--trim` are not in this spec.**
6. **From design (DEC-114 part 6):** with `--save-recipe`, a pixel `--crop` needs one input image;
   a batch is refused (P16, exit 2) before any image is read. A failed run writes no recipe.
7. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 2–5 are the spec's four checkpoints.
1. **Red tests first.** Write FT-1 and run it on `main`'s code **before any source edit**: it must
   pass there. Quote that run. Then write the rest of the integration tests (FT-2, and
   `tests/edit_geometry.rs`), run them on `main`, and quote the baseline: FT-2 fails on the hash
   (`0x62d0525855cff6a7`), and the `edit_geometry.rs` rows fail on clap's `unexpected argument
   '--crop' found` (quote it once; for FT-24 quote the old three-flag message). The unit tests can't
   compile before the new functions exist; they land with stage 2. Open the PR **as a draft** with
   this commit.
2. **Typed order with `--resize-max` and `--invert` only.** `run()` keeps the matches, `EditOp`,
   `edit_op_order` with the `value_source` filter, `EditFlags`, the new `build_edit_ops` and the
   post-run recipe. FT-1 and FT-2 green; the four old unit tests rewritten (FT-27); FT-31.
3. **`--rotate` and `--flip`**, through `build_flag_op`, with the refusal rows for both.
4. **`--crop` and `PixelCrop`.**
   - `parse_crop_flag` exactly as the spec words it (`x` U+0078 only; `×` U+00D7 is refused), P1, P2,
     P7; the aspect form through the registry, with `--crop-gravity` saved only when given.
   - `crop_fractions` divides in `f64` (`f64::from(x) / f64::from(iw)`), and the values go into the
     recipe untouched: no formatting, no rounding.
   - The past-the-edge check runs in `u64` pixels before any conversion.
   - Find the crop step in tests by `op = "crop"`, never by index (M1 showed an index lookup panics).
   - Expected rectangles, fractions and hashes come from the spec's literals, **never** from
     `crop_fractions` or the run, which are the code under test.
5. **Batch and docs.** P16, FT-28 to FT-30. Then each docs AC at the `file:line` the spec gives;
   line numbers may have shifted, so find each place **by content**. `crustyimg edit --help` keeps
   the spec's sentences. CHANGELOG: three "Added" lines and one "Changed" line, placed as the
   overlap section says for PR #237.
6. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it**. If the quoted line doesn't
   exist in your code (you named something differently), make the closest edit, quote what you
   applied, and say why. For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected, `$?` read; record which tests go
     red against the spec's red/green lists, including every cell marked *reasoned* (FT-8, FT-17,
     FT-23, FT-26, FT-27, FT-29, FT-31 to FT-33, and `tests/orientation.rs`'
     `edit_auto_orient_flag_still_exits_zero` under M15): those tests exist now, so they are driven;
   - restore, rebuild, and show it green.

   **M3** is an emulation; its FT-9 red is a known artifact of the emulation. **Trap 14** restores
   `main`'s `src/cli/mod.rs` and `src/cli/ops.rs` verbatim (12 tests red, FT-1, FT-25 and the 9
   `tests/edit.rs` tests green). Paste each mutation's red output, trimmed to the lines that name
   tests, into Build Completion. Where a cell is wrong, correct it in Build Completion and say so;
   don't edit the design's table.
7. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`, one after
   another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each.
   `edit` is in the lean binary, so the lean leg must run `tests/edit_geometry.rs`, not skip it.
   Confirm AC-25's wasm grep (no `edit` code in `src/wasm.rs`) and that CI's wasm counts didn't move.

## The things most likely to go wrong
- **Dropping the `value_source` filter**, or reading `index_of` for a flag that wasn't typed. A
  defaulted `--invert` then joins the order and `edit in.png` stops exiting 2 (M2; FT-1 reds by
  hash).
- **Converting against the decoded size.** A pixel crop after `--resize-max` or `--rotate 90` is in
  the pixels of the image at that step (M3).
- **Formatting the fractions.** `{:.6}` refuses (640, 9, 631) at `apply`'s load and nothing else in
  half a billion rectangles, so FT-12's literal row is the only guard (M4).
- **Building the recipe before the run** (M5). It compiles; the empty-cell guard is what catches it.
- **`--auto-orient` taking a position** (M6). Only the Orientation-5 row catches it; Orientation 6
  alone stays green.
- **The flag name not substituted** into the op's message (M13): `'aspect' must be …` instead of
  `--crop must be …`.
- **An exit-code-only past-the-edge test.** With the check removed the run still exits 1, through
  the op's fraction message; only the message assertion reds (M10).
- **Retyped messages.** Copy every P- and A-message from the spec byte for byte, including `×`
  (U+00D7) in A1/A2.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including the lean one and wasm.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths (AC-25).
- Don't widen `rect_edges` or `parse_aspect`; the design needs neither.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, or stop and report.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each
  commit on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done. Write scratch files in your session's scratchpad, never
  `/tmp`, and don't leave PR-body files behind.
- **Budget ~110 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed; say whether the count includes the spec file;
   - FT-1 on `main` and after, both runs quoted;
   - AC-1 to AC-27, one line each: met, or not and why;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out and every reasoned cell
     marked confirmed or corrected;
   - the matrix legs and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-153 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any
     `subagents/` transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Measure at your last turn, and append a `build` entry to `cost.sessions` with `tokens_total`, a
     `tokens_breakdown`, `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-153 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(cli): edit --crop, --rotate and --flip, run in the order typed (SPEC-153)`.
   - Body per AGENTS §13:
     - Summary, including the behaviour change: `edit --invert --resize-max N` now inverts first;
     - Spec metadata;
     - Decisions referenced: DEC-005, DEC-102, DEC-106, DEC-112, DEC-113;
     - Constraints checked;
     - New decisions: DEC-114 (on `main` since design);
     - Breaking: none in the library; the CLI change above is a CHANGELOG "Changed" line;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-153 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly, as plain lists:
- FT-1 on `main` and after;
- AC completion (27 rows, as a count, plus any row not met);
- each mutation's flips against the table, the edit you applied where it differs from the quoted
  one, and every cell that turned out wrong;
- the matrix result and the CI legs, including lean and wasm;
- the overlap check after your last rebase;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
