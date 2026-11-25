# SPEC-160 — BUILD prompt: `edit --pad` and `--trim`, and the `parse_color` sign fix

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms (six clap fields, `EditOp::{Pad, Trim}`, `PixelPad`, `FlagWorded`,
`format_fixed_by_flags`, three gated alpha checks in `run_pixel_op`, a `flag` field on
`IntroducedAlpha`, one rule in `parse_color`), every message verbatim, 16 ACs, 18 failing tests
(34 replay and order rows, 21 parse rows, 24 signed-colour rows) and a 22-row mutation table, **all
driven at design on a prototype**. The code is about 480 lines; most of the work is the tables and
the doc sentences. Be thorough across every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-160-edit-pad-trim -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal. This spec closes STAGE-058.

## What you are building
- **`edit --pad N|W:H`**, **`--pad-color`** (required with `--pad`; hex with any alpha, or
  `transparent`), **`--pad-gravity`** (aspect form only), **`--trim`** (a plain switch),
  **`--trim-color`**, **`--trim-tolerance`** (`u8`), in typed order (DEC-114).
- **`PixelPad`** (copy `PixelCrop`'s shape): `--pad N` becomes `border = N/long` of the image the step
  **receives**, checked in pixels first (wider than the long edge, the edge cap, the area cap: exit 1).
  ⚠ It must override `introduced_alpha`; without it every alpha check passes silently and JPEG gets a
  black border, exit 0 (M1).
- **`FlagWorded`**, one wrapper for the aspect pad and for `trim`: it forwards all five `Operation`
  methods and rewrites each `'key' = value` in an apply message to `flag value`.
- **The alpha checks in `run_pixel_op`**, gated: `alpha_check = forced_format.is_none() &&
  pipeline.introduces_alpha()`. One input: checked before the pipeline, exit 4, nothing written. A
  batch whose format `--format` or the name template's literal extension fixes: checked once before
  any image is read, exit 4, no out-dir. A kept-format batch: per input, before its pipeline, exit 6.
  **The gate is the point**: without it, existing exit codes move (M4; FT-8's four rows).
- **`IntroducedAlpha.flag: Option<&'static str>`**, and `UnstorableAlpha`'s `Display` names the flag
  (`--pad-color "transparent" has alpha 00, …`) when it is set. `pad.rs` sets `flag: None`.
- **`parse_color`** (`src/text/mod.rs`): every byte after an optional `#` must be an ASCII hex digit
  (`is_ascii_hexdigit`), replacing the `is_ascii()` check. Keep SPEC-152's non-ASCII test rows.
- **`run_edit`**'s batch `--save-recipe` message: picked by the first pixel-length step; the `--crop`
  line stays byte for byte.
- **The `watermark` nine-gravities test** (folded in, test-only): FT-17's hand-worked offsets.
- **Tests and docs** as the spec lists them.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.
The design added **no wasm test**, so `scripts/wasm-ci-check.sh` and `tests/wasm_roundtrip.rs` should
not change. If CI's wasm leg makes you touch either (SPEC-159's build hit exactly that), say so.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `d7f46b2` is in your history;
- DEC-115 part 4 still says the per-input check for an unforced format "arrives with `edit --pad`"
  (you build what it promises; ship rewrites the sentence after the merge).

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists: #240 (`src/cli/ops.rs`, an import block next to yours), #226 (`src/cli/mod.rs`,
the module doc and dispatch's doc), #237 (`CHANGELOG.md`, expect a textual conflict; keep both).
After every rebase, run `gh pr list --state open --json number,title,files` and compare; name any new
overlap in Build Completion. If a rebase conflicts in a file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-160-edit-pad-and-trim-and-parse-color-refuses-a-sign.md`.
   Read it in full: `## Frame` with the maintainer rulings, `## Design` (especially "Design
   corrections to the frame", "The code shape" and "Every message, verbatim"), every AC, the Failing
   Tests, `## Implementation Context` (overlap, docs, CHANGELOG verbatim, the mutation table, the
   build checkpoints) and `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it. Leave "DEC-115's ship-time replacement text" alone: ship
   applies it.
2. **DECs:** DEC-114 (typed order, the batch rule, pixel checks), DEC-115 (the alpha rule), DEC-095,
   DEC-106 (messages name the flag), DEC-112. Read them yourself.
3. The code the spec cites at `file:line`, before you change it. Line numbers were taken on `main` at
   `8ebc81a`; find each place by content if it has moved.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 5, 10, 12, 13 and 14). These bind PROJ-017. Also `/guidance/constraints.yaml`.
5. SPEC-153 and SPEC-159 in `specs/done/`: their Build Completions, verify findings and ship
   reflections. SPEC-159's only product defect was a doc sentence its build drove on the exit-4 path
   and not on the exit-6 path.

## Maintainer rulings (settled — do not re-open)
1. One `--pad N|W:H`, matching `--crop`; `--pad-gravity` with the aspect form only.
2. `--trim` is a plain switch; `--trim-color` and `--trim-tolerance` are written to the recipe only
   when given. `--pad-color` is required, with no default. No named colours: hex or `transparent`.
3. `--pad N` with `--save-recipe` on a batch is refused, exit 2, like a pixel `--crop` (DEC-114 part 6).
4. A colour with a sign is refused everywhere `parse_color` reads one.
5. [M], uncut; this spec closes STAGE-058, with `watermark`'s nine gravities folded in.
6. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 1–6 are the spec's six checkpoints.
1. **The failing tests, RED recorded.** Write FT-1 to FT-18 and run them on `main`'s code before any
   source edit. Quote the baseline: the `edit` rows fail at clap (`unexpected argument '--pad'`), the
   signed-colour rows load (exit 0), FT-8's four gating rows and FT-17 pass on `main` (say if any
   doesn't). Open the PR **as a draft** with this commit.
2. **`parse_color`**, FT-16 green, and FT-13's recipe and `watermark --color` rows.
3. **The flags:** `EditOp`, the clap fields, `edit_op_order`'s table, `build_edit_ops`, `PixelPad`,
   `FlagWorded`, the batch message. FT-1 to FT-4 and FT-9 to FT-14 green.
4. **The alpha checks:** `format_fixed_by_flags`, the three gated checks, `IntroducedAlpha.flag` and
   its `Display`. FT-5 to FT-8 and FT-15 green.
   - Expected messages and pixels come from the spec's literals, **never** from `Display` or the run,
     which are the code under test.
   - Every refusal test asserts the message text **and** what's on disk (nothing written, no
     out-dir, or the other inputs written), not only the exit code.
5. **Docs and CHANGELOG**, FT-17 and FT-18. Each doc row at the `file:line` the spec gives, found by
   content. The CHANGELOG lines are verbatim in the spec. **Drive every behaviour sentence on every
   exit path it names**, using the spec's list ("Behaviour sentences, and every exit path each
   names"): for example, a translucent fill on one input `-o .jpg` (4), a kept JPEG source into
   `--out-dir` (4, no dir), a `--format jpeg` batch (4), a `{stem}.gif` batch at alpha 80 (4), a
   kept-format batch (6), and `transparent` into a `{stem}.gif` batch (0). Quote each drive in Build
   Completion.
6. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it**. If the quoted line doesn't
   exist in your code, make the closest edit, quote what you applied, and say why. For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected, `$?` read, and read the red set
     **off that run**. Compare it with the table and call out every test the table doesn't list,
     including FT-18's `no_op_flag_message_lists_every_flag` (the design left it out of every red set
     because it was red in every prototype row; on your real code it shouldn't be);
   - **M14** is "all green" by design (neither `pad` nor `trim` refuses through the hook): drive it
     and say whether that still holds;
   - **Trap 14**: say whether the lib unit tests compile with `main`'s two files restored;
   - restore, rebuild, and show it green.

   Paste each mutation's red output, trimmed to the lines that name tests, into Build Completion.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
7. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`, one after
   another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each.
   Then `just wasm-check` and `just wasm-test` once (`alpha.rs` and `parse_color` compile for wasm;
   `just check` runs neither).

## The things most likely to go wrong
- **The wrapper eating `introduced_alpha`** (M1, M1b): both wrappers must forward it, with the flag
  set by struct update (`IntroducedAlpha { flag: …, ..a }`).
- **An ungated check.** It passes every alpha row and moves the `--format bogus` exit codes (M4).
- **`-o` treated as batch-wide.** The batch branch never reads `-o`; only `--format` and the
  template's literal extension fix the format for a batch.
- **The long edge.** `N/long` of the image the step **receives**, after any `--resize-max`,
  `--rotate` or `--trim` typed before it; never the input's size, never the short edge (M9).
- **A border pad and a quarter turn commute**, so they can't be an order test. Use the spec's three
  pairs.
- **16-bit fixtures:** write the frame through a helper that knows each colour type (`c × 257`);
  `DynamicImage::put_pixel` round-trips through RGBA8.
- **`-f-f-f` after a space is a clap error**, not a colour error: pass colours as `--flag=VALUE`.
- **Retyped messages.** Copy every message from the spec byte for byte; `×` is U+00D7.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean, Windows and wasm.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths.
- Don't edit DEC-115 or the spec's timeline: both live on `main`, and ship and the orchestrator keep
  them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each
  commit on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`, and it reads `$R:r…` as a
  modifier, so brace it (`${R}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done. Write scratch files in your session's scratchpad, never `/tmp`
  or the system temp dir, and don't leave PR-body files behind.
- **Budget ~110 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed; give the count and say whether it includes the spec file;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why;
   - each behaviour sentence's drive on each exit path, quoted;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out;
   - the matrix legs, the wasm check and test, and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push (SPEC-159's build measured
   early and under-counted by $4).
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-160 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any
     `subagents/` transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-160 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Title: `feat(edit): --pad and --trim flags, and a colour with a sign is refused (SPEC-160)`.
   - Body per AGENTS §13:
     - Summary, including that `edit --pad` refuses a translucent fill the output can't store, and
       that a hex colour with a `+` now fails to load where it used to load as a different colour;
     - Spec metadata;
     - Decisions referenced: DEC-095, DEC-106, DEC-112, DEC-114, DEC-115;
     - Constraints checked;
     - New decisions: none (DEC-115's wording follows at ship);
     - Breaking: a signed hex colour is refused (CLI and recipes); library: `IntroducedAlpha` gains a
       `flag` field;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-160 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- the RED baseline on `main`;
- AC completion (as a count, plus any row not met);
- each behaviour sentence's drive, one line per exit path;
- each mutation's flips against the table, the edit you applied where it differs from the quoted
  one, and every cell that turned out wrong;
- the matrix result, `just wasm-check`/`just wasm-test`, and the CI legs, including lean, Windows and
  wasm;
- the overlap check after your last rebase;
- files changed against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
