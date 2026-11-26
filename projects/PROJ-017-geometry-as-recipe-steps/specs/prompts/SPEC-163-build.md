# SPEC-163 — BUILD prompt: an aspect that follows each image's orientation

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms:
- one private helper, `follow_orientation`, in `src/operation/mod.rs`;
- one `orientation` row of kind `OneOf(["match"])` on `crop` and on `pad`;
- two refusals, one for crop's rectangle form and one for pad's border form;
- two `edit` flags built like the gravity flags.

It also gives every message verbatim, 18 ACs, 20 failing tests (14 of them in a new suite), 26 parity
rows, 3 fixtures and an 18-row mutation table. All of it was driven at design on a prototype **except
the parity rows**, which were only reasoned because the design did no wasm build. The code is about
140 lines; most of the work is the tests, the parity rows and the doc sentences. Be thorough across
every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-163-aspect-follows-orientation -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. **Write it as plain lists, not box-drawn tables.**

## What you are building
- **`orientation = "match"`**, an optional key on `crop`'s and `pad`'s aspect form. With it, a
  landscape image (strictly wider than tall) gets the aspect's larger side as its width, a portrait
  image gets it as its height, and a square image takes the aspect **as written**. Without the key,
  sizes are exactly today's.
- **`follow_orientation(a, b, w, h)`** runs on the size of the image the step **receives**, before
  `aspect_size` / `aspect_canvas`. Those two functions stay byte for byte as they are.
- **Load refusals** (exit 1, nothing written): the key on a rectangle crop or a border pad, an
  unknown value, the wrong case, the wrong type. Each form refusal goes **after** the gravity
  refusal, so gravity's message wins when both keys are present.
- **`edit --crop-orientation match`** and **`--pad-orientation match`**: each requires its `--crop` /
  `--pad`; refused with a pixel `--crop` or a border `--pad N` (exit 2), after the gravity flag's
  refusal. `--save-recipe` writes the key only when the flag was typed.
- **Tests, parity rows, fixtures, docs and CHANGELOG**, as the spec lists them.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.
The design changes **no wasm code and no wasm test**, so `src/wasm.rs`, `scripts/wasm-ci-check.sh`
and `tests/wasm_roundtrip.rs` should not change. If CI's wasm leg makes you touch one, say so.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `d2e9fae` is in your history;
- `tests/orientation.rs` exists (SPEC-110's suite). **Never write over it**: the new suite is
  `tests/aspect_orientation.rs`. The design prototype overwrote it once by accident.
- `decisions/DEC-112-…md` has **no** rule 8 yet. Ship applies it after the merge, from the text in
  the spec.

If any check fails, stop and report.

⚠ **Overlap.** The spec's map lists only parked PR #237, which overlaps on `CHANGELOG.md` (keep both
sides). After every rebase, run `gh pr list --state open --json number,title,files` and compare; name
any new overlap in Build Completion. If a rebase conflicts in a file outside the spec's list, stop and
report. Don't touch #237 in any way.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-163-an-aspect-that-follows-each-image-s-orientation.md`.
   Read it in full:
   - `## Frame`, with the maintainer rulings;
   - `## Design`, especially "Design corrections to the frame", "The code shape", "Every message,
     with its exit code", "The replay rows", "What doesn't change", "Fixtures", "The parity rows"
     and "Docs";
   - every AC and every Failing Test;
   - `## Implementation Context`: the mutation table, and `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it. Leave "DEC-112's rule 8" and "The page port" alone: ship and
   SPEC-161 use them.
2. **DECs:** DEC-112 (fractions and rounding), DEC-113 (a step acts on the buffer it receives),
   DEC-114 (typed order, the batch rule), DEC-106 (messages name the flag). Read them yourself.
3. The code the spec cites at `file:line`, before you change it. Line numbers were taken on `main` at
   `1331fb7`; find each place by content if it has moved.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 5, 10, 12, 13 and 14). These bind PROJ-017. Also `/guidance/constraints.yaml`.
5. SPEC-160 in `specs/done/`: its Build Completion, verify findings and ship reflection. It is the
   same shape of change (`edit` flags on `crop`/`pad`).

## Maintainer rulings (settled — do not re-open)
1. The key is `orientation = "match"`; absent means off.
2. A square takes the aspect as written; `1:1` with the key changes nothing. Landscape means strictly
   wider than tall, so 1071×1072 is portrait.
3. `--crop-orientation` and `--pad-orientation` are in this spec, at [M].
4. DEC-112 gains a rule 8, applied at ship, not in this PR.
5. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 1–4 are the spec's four checkpoints.
1. **The ops, then FT-1 to FT-11 green.**
   - First write FT-1 to FT-11 and the five moved expectations (FT-16 to FT-18), and run them on
     `main`'s code before any source edit. Quote the RED baseline: the recipe rows refuse with
     `unknown key 'orientation'` (exit 1), and FT-4 passes (it checks the test's own table). Say if
     any row disagrees.
   - Then `src/operation/mod.rs`, `crop.rs`, `pad.rs` and the registry tests.
   - Open the PR **as a draft** with this stage's commit.
2. **The flags, then FT-12 to FT-15 green.** `src/cli/mod.rs` and `src/cli/ops.rs`. The recipe writer
   needs no change: it saves `op.params()`. Build `with_text` from a `BTreeMap` via `from_map`, as
   the gravity code does, so the saved keys stay sorted (FT-13's literal texts depend on it).
3. **The fixtures and the parity rows.**
   - Make the three fixtures with **ImageMagick**, using the spec's exact commands, never with
     crustyimg (fixtures from the code under test cannot fail). Check each size with
     `magick identify`, and record each file's bytes and SHA-256 in
     `tests/fixtures/ddphotos/README.md`, as the spec says.
   - Add the 26 rows and the self-test N3 to `tests/ddphotos_parity.mjs` without moving `TABLE`,
     `ROW_COUNT`, the `GOLDEN_*` values, N1/N2 or the alpha rows.
   - Run `just ddphotos-parity` locally in both modes. It needs `node`, `wasm-pack` and a Python with
     Pillow. Paste the summary line (`N rows, 0 row failures, …`) and every `PASS N3 …` line into
     Build Completion.
   - **This is the first time the parity rows run.** The design reasoned their sizes and their N3 red
     set from the native sizes. If a row disagrees with its literal, find out which side is wrong
     before you change anything, and say so.
4. **Docs and CHANGELOG.**
   - Make each doc change at the `file:line` the spec gives, found by content. That includes
     `docs/cli-reference.md`, which the frame missed.
   - The CHANGELOG lines are verbatim in the spec.
   - **Drive each of the eleven behaviour sentences on every exit path it names**, using the spec's
     list ("Behaviour sentences and every exit path each names"): one input and a folder batch, with
     exit 0, 1 and 2 where each applies. Quote each drive in Build Completion.
5. **Mutations, one at a time** (HANDOFF traps 12 and 14). Run them in a scratch `git worktree` of
   your pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes
   each mutant's exact edit: **apply that edit, not your own version of it**. If the quoted line
   doesn't exist in your code, make the closest edit, quote what you applied, and say why. For each
   row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because reverting the source doesn't rebuild the
     binary;
   - run the full suite (not only the design's listed suites) with `--no-fail-fast`, stdout
     redirected, `$?` read, and read the red set **off that run**. Compare it with the table and call
     out every test the table doesn't list;
   - **M5 is the negative control**: the key loads but is never read. It must turn the rule's rows
     red and leave the refusal rows green;
   - **M16 (trap 14):** say whether the lib unit tests compile with `main`'s `src/operation/` and
     `src/cli/` restored, and that only FT-4 stays green in the new suite;
   - for the parity rows, run M5 against `just ddphotos-parity` too: exactly N3's eight rows should
     go red. Say whether any other mutant's parity cell can be driven cheaply, and drive the ones you
     can;
   - restore, rebuild, and show it green.

   Paste each mutation's red output into Build Completion, trimmed to the lines that name tests.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
6. **Clean matrix:** default, `--no-default-features` and `--features webp-lossy`, one after another,
   each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each. Then run
   `just wasm-check` and `just wasm-test` once (`just check` runs neither).

## The things most likely to go wrong
- **The wrong size fed to the helper.** It takes the `iw, ih` the step receives (`u32`), not the
  input's size and not the `u64` copies. AC-5 pins this: an Orientation-6 image is landscape without
  `auto-orient` and portrait after it.
- **A square swapped either way** (M1, M15). Use `>` and `<`, never `>=` or `<=`, and no tolerance
  (M14).
- **Refusal order.** The form refusal goes after gravity's, in both ops and in `build_edit_ops`.
- **The table row's place.** Crop's row goes last; pad's goes after `gravity`. The unknown-key
  messages print that order.
- **Parsing `"match"` twice.** Once the table has checked it, `params.contains("orientation")` is
  enough.
- **Retyped messages.** Copy every message from the spec byte for byte.
- **A blessed golden.** No `CRUSTYIMG_BLESS` run.
  `git diff --name-status origin/main -- tests/fixtures/` must show only `A` lines plus
  `M tests/fixtures/ddphotos/README.md`.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean, Windows, wasm and the
  advisory ddphotos-parity job.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths.
- Don't edit DEC-112 or the spec's timeline: both live on `main`, and ship and the orchestrator keep
  them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push**, and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- In an interactive terminal `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout to
  a file.
- A piped command reports the **pipe's** exit code, so redirect and read `$?`.
- Gate each commit on its test run; don't chain `git commit` after `cargo test` with `;`.
- zsh does not word-split unquoted parameters, so loop with `while IFS= read -r`. It also reads
  `$R:r…` as a modifier, so brace it (`${R}:`).
- macOS has no `timeout(1)`.
- `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done, and say how much space that freed.
- Write scratch files in your session's scratchpad, never in `/tmp` or the system temp dir. Don't
  leave PR-body files behind.
- **If a push is refused by a permission check, stop and report.** Name the commits and the command
  for the maintainer to run. Don't work around the check.
- **Budget ~110 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed. Give the count and say whether it includes the spec file;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not met and why;
   - each behaviour sentence's drive on each exit path, quoted;
   - the parity summary lines, both modes;
   - every mutation row: the edit you applied, the tests that went red and those that stayed green,
     against the spec's table, with every mismatch called out;
   - the matrix legs, the wasm check and test, and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push. A build that measured early
   has under-counted by $4 before.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. The `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-163 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any
     `subagents/` transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per MTok,
     Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-163 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Title: `feat(recipe): an aspect crop or pad can follow each image's orientation (SPEC-163)`.
   - Body per AGENTS §13:
     - a summary, including that `orientation = "match"` turns the aspect per image, that a square
       takes it as written, and that the two `edit` flags save the key;
     - the spec metadata;
     - decisions referenced: DEC-106, DEC-112, DEC-113, DEC-114;
     - constraints checked;
     - new decisions: none (DEC-112's rule 8 follows at ship);
     - breaking: none. The key is new and optional, and a recipe without it is unchanged;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-163 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- the RED baseline on `main`;
- AC completion, as a count, plus any row not met;
- each behaviour sentence's drive, one line per exit path;
- the parity summary in both modes, and any row whose reasoned literal was wrong;
- each mutation's red set against the table, the edit you applied where it differs from the quoted
  one, and every cell that turned out wrong;
- the matrix result, `just wasm-check` / `just wasm-test`, and the CI legs, including lean, Windows,
  wasm and ddphotos-parity;
- the overlap check after your last rebase;
- the files changed, against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- the space freed by cleanup;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
