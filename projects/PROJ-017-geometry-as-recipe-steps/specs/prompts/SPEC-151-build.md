# SPEC-151 — BUILD prompt: `pad` as a recipe step

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design
gives the op in code terms, every refusal's exact message, the colour-type table, the gravity
offsets as literals, 83 ACs and a 14-row mutation table, 12 of whose rows were driven at design on
a prototype. The code is one file of about 200 lines; most of the work is sweeps across colour
types, gravities, refusals and doc surfaces. Be thorough across every row: Sonnet's known weak spot
is sweep thoroughness.

```
just wt feat/spec-151-pad -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. Keep it plain text with short lists; a wide box-drawn table can
garble when copied out of the terminal. This spec is line 5 of PROJ-017's build order; `trim`
(line 6) and the `edit` flags spec (line 8) follow it.

## What you are building
- **`src/operation/pad.rs`:** the `pad` op. Two forms told apart by their keys: `aspect` (with
  optional `gravity`, default `center`) or `border` (a fraction of the long edge, on every side).
  `color` is required and must be opaque. It acts on the buffer it receives and writes no metadata
  (DEC-113), keeping the metadata bundle through `with_pixels`. It never uses the refusal hook; its
  two per-image conditions (a border that rounds to 0 px, a canvas past the caps) fail at apply,
  checked before anything is allocated.
- **Shared code, no behaviour change:** resize's caps hoisted in `src/operation/mod.rs`, and
  `parse_aspect` made `pub(super)` in `src/operation/crop.rs`. Registration in
  `src/operation/registry.rs`.
- **Tests:** unit tests in `pad.rs`, a new `tests/pad.rs`, registry test extensions, two wasm rows
  in `tests/wasm_roundtrip.rs`, and two corpus recipes (`v2_pad_aspect`, `v2_pad_border`) with their
  literal bumps.
- **`scripts/wasm-ci-check.sh`:** both constants and the self-test fixtures.
- **Two folded backlog entries:**
  - `orientation_of` moves from `tests/rotate_flip.rs` to `tests/common/mod.rs` and becomes strict:
    only `Err(exif::Error::NotFound(_))` means "no tag", and any other error fails the test. The
    corpus README's "Adding a recipe" table loses its numbers, word for word as the spec's
    "README text (AC-69)" gives it.
  - The `Operation::refusal` doc comment, word for word as "Doc comment text (AC-70)" gives it.
- **The docs surfaces** the spec lists, and an `### Added` entry under `CHANGELOG.md`'s
  `## [Unreleased]`.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `be08bb0` is in your history;
- `ls decisions/` shows no DEC written for this spec (none is expected).

If any check fails, stop and report.

⚠ The PROJ-011 orchestrator is running STAGE-063 (SPEC-149 and SPEC-150) at the same time, and has
been sent the spec's keep-clear list ("Notes for the Implementer"). If your rebase conflicts in a
file outside the spec's list, stop and report. **Re-read `FULL_COUNT` and `SLIM_COUNT` on `main`
after your last rebase:** the spec expects 51/49 → 53/51, but if STAGE-063 added a wasm test, add 2
to whatever main says and say so in Build Completion.

## Read in order, before any code
1. **The spec:** `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-151-pad-as-a-recipe-step.md`.
   Read it in full: `## Frame` (for the why), `## Design` (especially "Design corrections to the
   frame", "The op, in code terms", "The colour-type rule", "Gravity, by absolute position" and
   "Exact messages"), AC-1 to AC-83, the Failing Tests, the mutation table,
   `## Implementation Context` (the README and doc-comment text) and `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-095, DEC-102, DEC-110, DEC-112 and DEC-113. Read them yourself.
3. `tests/fixtures/recipe_corpus/README.md`: the bless procedure. Follow it exactly. Note that you
   also rewrite its "Adding a recipe" table in this spec (stage 5): bless by the procedure first,
   using the table as it stands, then rewrite it.
4. `scripts/wasm-ci-check.sh`, the constants **and** `self_test`.
5. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10, 11, 12, 14, 15 and 17). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.
6. SPEC-147 in `specs/done/`, the last op added this way: its Build Completion (two mutation-table
   cells it corrected) and its ship reflection.

## Maintainer rulings (settled — do not re-open)
1. **`transparent` is out of this spec.** `"transparent"` and any colour with alpha below `ff` are
   refused at load with the spec's message, which says it isn't supported yet and names no later
   spec.
2. **`color` is required, with no default.**
3. **A `border` is a fraction of the long edge,** the same pixels on every side.
4. **A grey image with a chromatic fill widens to RGB** (RGBA if it had alpha), at the same depth.
   A grey fill on a grey image keeps the luma type, and a no-op returns the input unchanged.
5. No message or doc line names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. These are the spec's six checkpoints, plus the mutations and the matrix.
1. **Red tests first.** Write the integration, registry and `tests/rotate_flip.rs` tests the spec
   lists. (The unit tests in `pad.rs` can't compile before the module exists; they land with stage
   2.) Run them against `main`'s code and quote the baseline: the `tests/pad.rs` rows fail with
   `unknown operation 'pad'`. Open the PR **as a draft** with this commit.
2. **The op, green locally.**
   - Implement `pad` and the registration as the spec words them. The canvas is the **per-axis
     integer formula** from "Design corrections to the frame", not the frame's "long side kept";
     AC-2's rows and AC-3's sweep hold it.
   - Every refusal message is **byte-for-byte** the spec's text: copy it, never retype it.
   - Expected offsets and pixels come from the spec's literals (the gravity table, the
     colour-type table, `#123456` → `0x1212/0x3434/0x5656`), **never** from `Gravity::placement`,
     `layout` or `image::imageops`, which are the code under test.
   - The caps and the 0 px check run before the canvas is allocated (AC-59).
   - Check the metadata test's control B (5×7 after `auto-orient`) before you trust its pad rows.
3. **The corpus, for your own target.**
   - Add the two `.toml` files and write their `.golden` files by the README.
   - Raise every literal in one commit, as AC-62 gives them: `RECIPES` 20, `ROWS` 66,
     `GOLDEN_LINES` 69, the `54` → 60, `pairs.len()` 20, and the doc comments that state the counts.
   - Bless your own target only.
   - ⛔ **`git status` must show only added files plus the literal lines.** If any existing
     `.output` or `.golden` is modified, stop: that's AC-64 failing, and it means something else
     moved. Report it; don't re-bless around it.
   - Push. Start `gh pr checks <PR> --watch` **in the background**. Never poll. When it exits, copy
     the other two targets' `.output` files from the failing log's `----- BEGIN … -----` blocks
     (`gh run view <run> --log-failed`). They count only once a **second** CI run is green on every
     leg. Then `cmp` the three per-target files for each recipe, and quote the result (AC-63).
4. **wasm and the script, in one commit.**
   - Add the two wasm tests (AC-65, AC-66).
   - Set the two constants (53/51, or main's numbers + 2), **and** move the `--self-test` fixtures
     to match in the same commit, as AC-67 spells them. Each row keeps its meaning.
   - Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and
     `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh workflow`, and `./scripts/validate-frontmatter.sh`.
     Quote each result line.
   - Run `just wasm-check`, `just wasm-test` and `just wasm-test-slim` locally if you can, and quote
     each result line. Then read CI's two wasm count steps.
5. **The folded entries.** `orientation_of` moved and strict (AC-68), the README table (AC-69) and
   the doc comment (AC-70), each word for word from the spec. `tests/rotate_flip.rs` still passes
   with the shared helper.
6. **The docs.** Make each docs AC true at the `file:line` the spec gives. Line numbers may have
   shifted, so find each place **by content**. `tests/docs_ops.rs` must resolve every new
   `op = "pad"` example.
7. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. Twelve rows were
   driven at design on a prototype, so a mismatch on the real code is a finding. M13 and M14 are
   reasoned; driving them is the evidence. For each row:
   - apply it at the location the spec names, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected, `$?` read; record exactly which
     rows go red against the spec's red/green lists;
   - restore, rebuild, and show it green.

   **M10** edits a corpus recipe in the scratch copy only. **M12** restores `main`'s
   `with_builtins` verbatim, and every `tests/pad.rs` integration row must go red. Paste each
   mutation's red output, trimmed to the lines that name rows, into Build Completion. Where a cell
   is wrong, correct the table in Build Completion and say so; don't edit the design's table.
8. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`. Run them one
   after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check`
   on each. `pad` has no feature gate, so the lean leg must run its tests, not skip them. Also
   record CI's wasm bundle-size delta.

## The things most likely to go wrong
- **The frame's canvas formula.** It's wrong for a square or wrong-way image; the design corrected
  it. Build from the design.
- **An expected offset computed with the code under test.** Literals only. M1 and M2 prove the
  position rows bite.
- **A no-op that isn't one.** "Already at the aspect" means the computed canvas equals the image,
  and then the step returns the input unchanged, type included (a grey image with a colour fill
  stays grey on a no-op).
- **Allocating before the cap check.** An `aspect = "65535:1"` canvas would be gigabytes.
- **`f32` on the border path.** Read `border` as `f64` (M14).
- **Dropping the metadata bundle.** Use `with_pixels` (M13).
- **The wasm counts without the self-test.** Both move in one commit (stage 4).
- **The README table getting numbers back.** Stage 3 blesses by the old table; stage 5 replaces it
  with the spec's text.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including the lean one, the wasm
  job and the bundle-size gate.
- **Stay in the spec's files.** The PROJ-011 orchestrator is keeping clear of them for you.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI.** Run `gh pr checks --watch` in the background only.
- **Don't chase `main`.** Rebase only at Step 0 and if a conflict forces it; the orchestrator
  updates the branch before merge.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each
  commit on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done.
- Comments and user-facing text read behaviour-first, with no spec, stage or DEC ids.
- **Budget ~110 exchanges.** The second CI round for the corpus files is expected. If you're past
  budget with the matrix green, stop and report what's left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed; say whether the count includes the spec file;
   - AC-1 to AC-83, one line each: met, or not and why;
   - every mutation row, with its applied line, the rows that went red and the rows that stayed
     green, against the spec's table, with every mismatch called out and every reasoned cell marked
     confirmed or corrected;
   - the corpus: each CI leg's result, and the `cmp` of the three per-target files;
   - the wasm script's three local checks, the local and CI wasm counts, and the bundle-size delta;
   - the matrix legs;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-151 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Measure at your last turn, and append a `build` entry to `cost.sessions` with `tokens_total`, a
     `tokens_breakdown`, `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-151 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(recipes): pad to an aspect or by a border, as a recipe step (SPEC-151)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-095, DEC-102, DEC-110, DEC-112, DEC-113;
     - Constraints checked;
     - New decisions: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-151 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly, as plain lists:
- AC completion (83 rows, as a count, plus any row not met);
- each mutation's flips against the table, and every cell that turned out wrong;
- the corpus: CI legs and the three-file `cmp`; and confirmation that no existing `.output`
  changed;
- the wasm counts (local and CI), whether main's base had moved, and the self-test result;
- the matrix result and the bundle-size delta;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
