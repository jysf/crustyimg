# SPEC-147 — BUILD prompt: `rotate` by quarter turns and `flip`, as recipe steps

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design
gives both ops in code terms, every refusal's exact message, the normalisation table, the index
maps, 58 ACs and a 12-row mutation table. The work is mechanical, but most of it is sweeps across
colour types, angles and refusals. Be thorough across every row: Sonnet's known weak spot is sweep
thoroughness.

```
just wt feat/spec-147-rotate-flip -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. This spec is line 4 of PROJ-017's build order. The any-angle rotate
stage extends this `rotate` op, and the `edit` flags spec needs both ops.

## What you are building
- **`src/operation/rotate.rs`** and **`src/operation/flip.rs`:** two ops. `rotate` takes `angle`, a
  TOML number read as `f64` in [-360, 360], a multiple of 90, clockwise. `flip` takes
  `axis = "horizontal" | "vertical"`. Both act on the buffer they receive and write no metadata,
  per **DEC-113**: they keep the metadata bundle in memory through `with_pixels`, as `crop` does.
- **Registration** in `src/operation/registry.rs`, with `crop`'s private `written` helper moved
  there and `crop.rs` importing it.
- **A comment-only paragraph** in `src/sink/mod.rs`'s `## Encoding` doc (AC-40). No code change.
- **Tests:** `tests/rotate_flip.rs`, unit tests in both op files, and two rows in
  `tests/wasm_roundtrip.rs`.
- **Two corpus recipes** (`v2_rotate_90`, `v2_flip_horizontal`), with per-target `.output` files,
  and the literal bumps.
- **`scripts/wasm-ci-check.sh`:** both constants and the self-test fixtures.
- **The docs surfaces** the spec lists (AC-48 to AC-56), and an `### Added` entry under
  `CHANGELOG.md`'s `## [Unreleased]`.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `8699ede` is in your history;
- `decisions/DEC-113-*.md` exists.

If any check fails, stop and report.

⚠ The PROJ-011 orchestrator is running its last stage (STAGE-063, SPEC-148 to 150) at the same
time, and has been told which files to keep clear. If your rebase conflicts in a file outside the
spec's list, stop and report. Unlike SPEC-145's build, CI **does** run the wasm round-trip tests
now (`wasm round-trip tests (Node)`), so read that job on every push.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-147-rotate-by-quarter-turns-and-flip-as-recipe-steps.md`.
   Read it in full: `## Frame`, `## Design` (especially "The ops, in code terms", "Normalisation",
   "Index maps" and "Exact messages"), AC-1 to AC-58, the Failing Tests, the mutation table,
   `## Implementation Context` and `## Notes for the Implementer`. **The design calls are
   settled.** If one turns out wrong, say so in Build Completion and in your report; don't quietly
   re-decide it.
2. **DECs:** DEC-113 (the metadata rule, new), DEC-003, DEC-095, DEC-102, DEC-110 and DEC-112.
   Read them yourself; don't trust a summary.
3. `tests/fixtures/recipe_corpus/README.md`: the bless procedure. Follow it exactly.
4. `scripts/wasm-ci-check.sh`, the constants **and** `self_test` (around :200–230).
5. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10, 11, 12, 14 and 15 especially). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.
6. SPEC-145 in `specs/done/`, for how the last op landed: its Build Completion, both punch lists
   and its ship reflection.

## Maintainer rulings (settled — do not re-open)
1. **A positive `angle` is clockwise**, as displayed after any earlier step. `-90` turns
   anticlockwise.
2. **`flip` keeps `axis = "horizontal" | "vertical"`.** `horizontal` mirrors left and right.
3. **No refusal message names a stage, spec or DEC id.** The non-multiple refusal ends
   "rotating by any other angle is not supported yet".
4. **`angle = 0`, `360` and `-360` load as a no-op.**

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push
after each numbered stage. These are the spec's five checkpoints.
1. **Red tests first.** Write every failing test the spec lists. Run them against `main`'s code
   and quote the baseline: the integration rows fail with `unknown operation 'rotate'` /
   `'flip'`. Open the PR **as a draft** with this commit.
2. **The ops, green locally.** Implement both ops and the registration as the spec words them.
   Every refusal message is **byte-for-byte** the spec's text: copy it, never retype it. Expected
   pixels come from the spec's hand-written index maps, **never** from `image::imageops`, which is
   the code under test. Check the orientation test's control B (5×7 after `auto-orient`) before
   you trust its rotate row; if the PNG decoder doesn't hand the `eXIf` chunk to the bundle, stop
   and report, and don't switch to a JPEG.
3. **The corpus, for your own target.**
   - Add the two `.toml` files and write their `.golden` files by the README.
   - Raise every literal in one commit, as the spec gives them: `RECIPES` 16 → 18, `ROWS` 54 → 60,
     `GOLDEN_LINES` 57 → 63, the `48` in `EXPECTED_DIMS.len() * FIXTURES.len()` → 54,
     `EXPECTED_DIMS` length 18, `pairs.len()` 18, and the doc comments that state the counts.
   - Bless your own target only.
   - ⛔ **`git status` must show only added files plus the literal lines.** If any existing
     `.output` or `.golden` is modified, stop: that's AC-43/44 failing, and it means something
     else moved. Report it; don't re-bless around it.
   - Push. Start `gh pr checks <PR> --watch` **in the background**. Never poll. When it exits,
     copy the other two targets' `.output` files from the failing log's `----- BEGIN … -----`
     blocks (`gh run view <run> --log-failed`). They count only once a **second** CI run is green
     on every leg. Then `cmp` the three per-target files for each recipe, and quote the result.
4. **wasm and the script, in one commit.**
   - Add the two wasm tests.
   - Set `FULL_COUNT=51` and `SLIM_COUNT=49`, **and** move the `--self-test` fixtures to match in
     the same commit: the fixture log lines and rows 1–5 and 9. Each row keeps its meaning: exact
     full, off-by-one full, a full log read as slim, exact slim. SPEC-145 changed only the
     constants and went red in `front-matter validation`.
   - Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and
     `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh workflow` (the workflow check fails on a
     US-ASCII locale), and `./scripts/validate-frontmatter.sh`. Quote each result line.
   - Run `just wasm-check`, `just wasm-test` and `just wasm-test-slim` locally if you can, and
     quote each result line (51 and 49 passed). Then read CI's two wasm count steps.
5. **The docs.** Make each docs AC (AC-48 to AC-56) true at the `file:line` the spec gives. Line
   numbers may have shifted, so find each place **by content**. `tests/docs_ops.rs` must resolve
   every new `op = "rotate"` / `op = "flip"` example.
6. **Mutations, one at a time** (HANDOFF traps 12 and 14): M1–M10 with M6b and M7b. Every pixel
   row in the table is marked **reasoned**, not driven, because no op code existed at design. So
   driving each row is the evidence, and a mismatch is a finding. For each one:
   - apply it at the location the spec names, and check `git diff` is non-empty;
   - rebuild **clean** (a fresh `CARGO_TARGET_DIR`, or `cargo clean -p crustyimg`), because a
     reverted source doesn't rebuild the binary;
   - run the tests, and record exactly which rows go red against the spec's red/green lists;
   - restore, rebuild, and show it green.

   **M9** edits a corpus recipe on a scratch copy only. Afterwards, show `git status` clean.
   **M10** restores `main`'s `with_builtins` verbatim, and every `tests/rotate_flip.rs` row must go
   red. Paste each mutation's red output, trimmed to the lines that name rows, into Build
   Completion. Where a reasoned cell is wrong, correct the table in Build Completion and say so;
   don't edit the design's table to match.
7. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`. Run them one
   after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check`
   on each. Neither op has a feature gate, so the lean leg must run their tests, not skip them.
   Also record CI's wasm bundle-size delta.

## The things most likely to go wrong
- **A round trip that passes on a wrong-way rotation.** Four anticlockwise turns are also the
  identity. The position tests (AC-3, AC-4) are what catch direction; M1 and M2 prove they bite.
- **An expected pixel computed with the code under test.** Hand-written index maps only.
- **f32 anywhere on the angle path.** M6b shows `1e-300` reads as `0.0` under f32 and silently
  becomes a no-op. Read `angle` as `f64`.
- **Dropping the metadata bundle.** It looks harmless because the sink writes no EXIF, but a later
  `auto-orient` then silently does nothing (M7b, AC-39). Use `with_pixels`.
- **The wasm counts without the self-test.** Both move in one commit (stage 4).
- **A green leg on one OS isn't the matrix.** Read every CI leg, including the lean one, the wasm
  job and the bundle-size gate.
- **Stay in the spec's files.** The PROJ-011 orchestrator is keeping clear of them for you.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. AC-57 asserts this; paste `git diff --stat origin/main...HEAD`
  for those paths.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI.** Run `gh pr checks --watch` in the background only.
- **Don't chase `main`.** Main's branch rule requires the PR to be up to date before merge, and
  each push to `main` makes it stale. Rebase only at Step 0 and if a conflict forces it; the
  orchestrator updates the branch before merge.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
  zsh does not word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir when
  you're done with it.
- Comments and user-facing text read behaviour-first, with no spec, stage or DEC ids. That
  includes every refusal message and every doc line.
- **Budget ~100 exchanges.** The second CI round for the corpus files is expected. If you're past
  budget with the matrix green, stop and report what's left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a
     phrase on what changed;
   - AC-1 to AC-58, one line each: met, or not and why;
   - every mutation row, with its applied line, the rows that went red and the rows that stayed
     green, against the spec's table, with every mismatch called out and every reasoned cell
     marked confirmed or corrected;
   - the corpus: each CI leg's result, and the `cmp` of the three per-target files;
   - the wasm script's three local checks, the local and CI wasm counts, and the bundle-size delta;
   - the matrix legs;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-147 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Measure at your last turn, and append a `build` entry to `cost.sessions` with
     `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-147 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(recipes): rotate by quarter turns and flip, as recipe steps (SPEC-147)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-003, DEC-095, DEC-102, DEC-110, DEC-112;
     - Constraints checked;
     - New decisions: DEC-113 (written at design, on `main`);
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-147 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly:
- AC completion (58 rows, as a count, plus any row not met);
- each mutation's flips against the table, and every reasoned cell that turned out wrong;
- the corpus: CI legs and the three-file `cmp`; and confirmation that no existing `.output`
  changed;
- the wasm counts (local and CI) and the self-test result;
- the matrix result and the bundle-size delta;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
