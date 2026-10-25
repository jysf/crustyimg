# SPEC-131 — BUILD prompt: recipe step params are validated at load

Cycle: **build**. Fresh CLI session, own worktree, **Opus** (the AC tables are a 14-key sweep, and
sweep thoroughness is where a cheaper builder has lost before):

```
just wt feat/spec-131-strict-recipe-step-params -- --model opus
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own.

**What you are building:** an unknown, wrong-typed, out-of-range (including `nan`/`inf`) or
mode-unused key in a recipe step is refused when the recipe is loaded, with one error line naming
the step index, op and key, on `apply --recipe`, `build` and wasm `transform`. Every recipe that is
valid today still loads, and its `to_toml()` stays byte-identical.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The spec must be at `cycle: build`, and its design
commit `63fbfc0` must be in your history. If either is not the case, stop and report.

## Read in order, before any code
1. **The spec** — `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-131-typed-per-operation-parameter-structs.md`.
   Read `## Frame`, `## Design`, the 15 ACs, `## Failing Tests` (including the negative-control
   table), `## Implementation Context` and `## Notes for the Implementer` in full. **Its design
   calls are settled.** If one turns out wrong, say so in Build Completion and in your report; do
   not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Binding.
3. DEC-061 (accepted risk #1 and its matching "Alternatives" entry: what you retire), DEC-014,
   DEC-005, DEC-031, DEC-064, DEC-100, DEC-058, DEC-101, DEC-099. `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. Unknown step keys are a load-time error; DEC-102 supersedes DEC-061's **accepted risk #1**
   (not the whole DEC). No "new in 0.8.0" hint; one **Breaking** CHANGELOG line.
2. (2026-10-25) **Keys the step's mode never reads are refused too** (AC-5), as designed.
   Documented combinations (`tile` with `gravity`/`margin`, `center` with `margin`) stay accepted.
3. (2026-10-25) **No upper bound on watermark `size` in this spec.** It is its own backlog item;
   do not invent a limit.

## Reserved
📌 **DEC-102 is yours** (highest on `main` is DEC-101). Write it at build, as the spec says, with
the **block-list** `affected_scope` form. An inline array is silently dropped by
`scripts/decisions-audit.sh`, so a DEC written that way governs nothing and nothing warns you:
```yaml
affected_scope:
  - src/operation/registry.rs
  - src/operation/mod.rs
  - src/recipe/mod.rs
```
Add the two one-line "superseded by DEC-102" markers to DEC-061 (accepted risk #1 and its
alternative); follow how DEC-101 marked DEC-058.

## Order
1. **Goldens first, on an untouched tree, in their own commit** (Call 2). Capture
   `tests/fixtures/recipe_corpus/` and the `recipes/*.toml` goldens from unmodified `main` code.
   Verify will re-run the bless test on `main` and diff, so a golden captured after a `src/` edit
   is a fixture from the code under test, and it cannot fail.
2. **Write every Failing Test, run against unmodified `src/`, and record that each RED test is
   RED for the reason the spec gives** (and each **guard** is GREEN). A test never seen red is not
   a guard. Record the run in Build Completion.
3. Implement registry schema → constructors → recipe index/Display/marker, smallest seam first.
4. **Push a WIP commit as soon as it compiles, before the matrix.** (`git commit -s`, push the
   branch.) A long build with zero commits has cost this repo $40 once.
5. Negative controls (AC-14): revert **one** condition at a time, rebuild, confirm exactly the row's
   tests flip RED and the rest stay GREEN, restore. Evidence is the behavioural flip, never a hash.
   ⚠ Reverting source does not rebuild the binary: rebuild before each run.
6. Clean matrix (AC-15), then docs (AC-13), then PR.

## The things most likely to go wrong
- **The goldens.** If any existing recipe's `to_toml()` changes, Call 2 is broken: stop and report
  rather than re-blessing. Re-blessing to make the test green is the one move that hides the bug.
- **TOML integers are `i64`.** `margin = -5` became `0` through `try_from(..).ok()`; `UInt` must
  refuse negatives and anything above `u32::MAX`. `Number` takes integers and finite floats;
  `opacity = 1` must still serialize as `1`.
- **Count the nouns.** AC-3 is 14 keys, AC-4 is 17 rows, AC-5 is 6 refused + its positive rows,
  AC-7 is single **and** batch, AC-12 is five parts. A table test that silently skips a row is
  the SPEC-128 "overlay/font" trap again. Make each table test assert its row count.
- **`tests/registry_seam.rs`'s existing test passes unedited** (AC-12d). If you have to touch it,
  the seam changed and the design call is broken: report it.
- **Stay off other specs' files.** `src/cli/**`, `src/wasm.rs`, `src/build/**`,
  `tests/apply_batch.rs` and `tests/build.rs` are **not yours**. SPEC-132 (`src/cli/build.rs`,
  `src/build/watch.rs`) and SPEC-134 may be building in parallel. `CHANGELOG.md`,
  `docs/api-contract.md` and `tests/wasm_roundtrip.rs` are shared by appending: add your own lines
  or test, do not reflow anyone else's. SPEC-133 builds **after** you on `Watermark`, so leave
  "adding a key is one line in `WATERMARK_PARAMS`" true.
- **`just demo-build` is the wasm bundle-size gate**, and `just check` never runs it. This change
  reaches wasm through `src/operation/**`. If the gate moves, that is a purchase: stop and report
  the byte delta, do not edit the gate (DEC-066's ledger records every moved gate).

## Guardrails
- ⛔ No version bump, no release, no `CACHE_SCHEMA_VERSION` change. Everything in PROJ-011 ships as
  ONE release (0.8.0); merging is fine, tagging is the maintainer's.
- **Every commit `git commit -s`.** The "DCO sign-off check" fails any PR with an unsigned commit;
  it has bitten this repo four times.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background; when it exits, read a direct
  snapshot (`gh pr checks <PR>`) at the PR's true head SHA and check **every** leg, including
  windows-latest and the wasm/demo jobs.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file, do not "fix" it. A piped command reports the **pipe's** exit code: redirect and read `$?`.
- zsh does not word-split unquoted parameters: build argument lists explicitly, loop with
  `while IFS= read -r`. macOS has no `timeout(1)`.
- Run the matrix legs **sequentially**, each with its own fresh `CARGO_TARGET_DIR`. Never run a
  background build while editing source for a revert.
- Comments and user-facing text are behaviour-first, with no spec/DEC ids.
- **Budget ~150 exchanges.** If the matrix is green and you are past that, stop and report what is
  left rather than polishing.

## When you finish
1. Fill `## Build Completion`: every file from `git diff --name-only origin/main...HEAD` (not from
   memory; past builds were short by 1–3 files), the RED baseline, the six negative-control flips,
   the matrix legs, the `demo-build` result, and an honest reflection naming any design call that
   turned out wrong.
2. **Measure your own cost.** Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where
   `<slug>` is your working directory's absolute path with every `/` and `.` replaced by `-`.
   Identify it **by content, not recency**: `grep -l "SPEC-131 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   ⚠ **Dedup by `.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing
   an id repeat identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens`
   while `output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`.
   Summing every line over-counts ~2×; taking the first line's output under-counts ~11×; both
   mistakes are on record. Price each component at the model that actually ran (Opus $5/$25 per
   MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate, cache_read ×0.10), never a flat rate.
   Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
   `estimated_usd` and the rate anchors.
3. `just advance-cycle SPEC-131 verify`. Open the PR: title
   `feat(recipe): validate recipe step params at load (SPEC-131)`, body per AGENTS §13 (Summary,
   Spec metadata, Decisions referenced, Constraints checked, New decisions: DEC-102), ending with
   the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-131 build: PR #NNN at <head SHA> — CI <green | red: legs> `. Then, short: AC
completion (15 rows, done / not done); RED baseline confirmed; the six negative-control flips;
matrix + `demo-build` result (with the bundle byte delta); files changed vs the spec's list
(anything extra or missing); any design call that turned out wrong; your measured cost.
