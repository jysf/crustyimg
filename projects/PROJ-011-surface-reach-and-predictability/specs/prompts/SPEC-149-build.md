# SPEC-149 — BUILD prompt: no flag is accepted and ignored

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design spells out the guard table (8 rows),
the logger, the font/overlay messages, every doc sentence and a 12-row mutation table. The care goes into the
tests: every check must be shown red where it should be, with its failure line.

```
just wt feat/spec-149-no-flag-accepted-and-ignored -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this prompt into a
fresh session and pastes your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and its SPEC-151 (`pad`) is building at the same time** on
`feat/spec-151-pad`. Until its PR merges, do **not** edit: `src/operation/`, `src/recipe/`, `src/text/mod.rs`,
`docs/recipes.md`, `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `docs/backlog.md`,
`skills/crustyimg/SKILL.md`, `integrations/ddphotos/README.md`, `CHANGELOG.md`, `tests/fixtures/recipe_corpus/`,
`tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`, `tests/wasm_roundtrip.rs`, `tests/common/mod.rs`,
`tests/rotate_flip.rs`, `scripts/wasm-ci-check.sh`. Use your own `CARGO_TARGET_DIR`. If a rebase conflicts in a
PROJ-017 file, stop and say so; do not resolve it.
- **Deferred hunks:** the spec's Docs sweep rows for `docs/api-contract.md` (`:36`, `:37`, `:39`, `:591-593`) and
  `docs/data-model.md:122` are **not yours to apply now**. Check `gh pr list` / `git log origin/main` for SPEC-151's
  merge after your last rebase: if it has merged, apply them (smallest hunk); if not, leave them and list them in
  your report as "deferred to ship".
- **Shared but free:** `src/cli/mod.rs`, `src/cli/ops.rs`, `src/cli/optimize.rs`, `README.md`,
  `docs/cli-reference.md`, `AGENTS.md`. Keep each edit to the smallest hunk the spec names (SPEC-151 may add a verb
  arm in `src/cli/mod.rs`). `CHANGELOG.md` and `docs/recipes.md`/`SKILL.md` text stays proposed text in the spec;
  the orchestrator places it.
- **Counts:** SPEC-149 adds no wasm test. If you find you must, stop and say so (PROJ-017's verify re-checks
  `FULL_COUNT`/`SLIM_COUNT`).
- If SPEC-151 merges before your PR does and adds a verb (`pad`), add a `pad` entry to `minimal_args` in
  `tests/cli_flags_honest.rs`; otherwise FT-1 fails naming `pad`. The guard itself needs no change.

## Rulings added after design (settled)
1. The R6 message says "pixel-lane encodes", as designed. Keep it.
2. A corrupt `--image` (readable but not an image) keeps its existing message and exit 4; do not widen this spec.

## Step 0
`git fetch origin && git rebase origin/main`. Check the spec is at `cycle: build` and the design commit `d14c786`
is in your history; record `cargo`/`rustup` versions. If any check fails, stop and report.

## Read in order, before any code
1. The spec in full: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-149-no-flag-is-accepted-and-ignored.md`
   (`## Frame`, `## Design` calls 1 to 5 and the table, `## Acceptance Criteria`, `## Failing Tests` incl. the
   mutation table, Implementation Context incl. the Docs sweep, Notes for the Implementer). **The design calls are
   settled.** If one turns out wrong, say so in Build Completion and the report; do not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: read-first, rules 1 to 7, traps 1 to 17
   (binding; 2, 10 to 17 apply directly). `/guidance/constraints.yaml`.
3. DEC-003 and DEC-024 (amended by design), DEC-012, `src/cli/mod.rs` (`GlobalArgs`, the `--watch` guard in
   dispatch), `src/cli/ops.rs` (`watermark_overlay`), `src/main.rs`.

## Order (verify checks each step)
1. **The tests first, against unmodified `main`.** Write `tests/cli_flags_honest.rs` (FT-1 to FT-12) and the SVG
   fixture. Run them on `main`'s `src/` and quote each RED **with its failure line** (trap 13): the reds must be
   "exit 0, expected 2", "stderr lacks the `-v` line" and the old font message, not a compile error or harness
   bug. Commit and push a WIP checkpoint (long cycles must checkpoint).
2. **The guard table** (`src/cli/global_flags.rs`), the dispatch swap (the `--watch` message moves into the table
   unchanged; the existing `--watch` test in `tests/cli.rs` stays green), the `hide` and help strings.
3. **The logger** (`src/cli/logger.rs`, `log = "0.4.32"` in the native-only table, installed once in `run()`).
   Quote a real `-v` line from the SVG fixture. Show `cargo tree -i log` adds no crate, `cargo deny check` passes,
   and `just wasm-check` exits 0 (the wasm build must not link the logger).
4. **The font/overlay error variant** and its exit-3 arm; the two lines in `watermark_overlay`.
5. **Docs** per the Docs sweep (rules above), `guidance/ideas.yaml` is already on `main`: confirm, do not re-add.
6. **Gates (read `$?`, no pipes):** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `just check`, plus the lean build (`--no-default-features`, which has no SVG, so FT-8 uses the AVIF fixture) and
   `just wasm-check`. The Windows leg cannot run here: say so and quote it from the draft PR.
7. **Local mutations M1 to M12 (and M6b)** one at a time in a scratch worktree: apply, check `git diff` is
   non-empty, run the checks the table names, record every red **with its quoted line**, revert exactly (`git diff`
   empty, trap 14), confirm green.
8. **Before the PR:** `git fetch origin`; rebase; rerun `tests/cli_flags_honest.rs` and `tests/cli.rs`. Open the
   **draft PR**.

## The things most likely to go wrong
- ⛔ Never push a mutation. `git diff origin/main...HEAD` must show none.
- ⛔ Nothing publishes or tags. No `git tag`, no `gh workflow run`, no branch-protection change.
- **A refusal test that can't fail** (traps 11, 13, 17): derive the verb list from `Cli::command()`, assert the
  count it saw (20 today), assert nothing was written (list the directory before and after), and show M12 (empty
  command walk) red.
- **A flag the guard must NOT refuse:** `build --check/--frozen/--locked/--no-cache/--watch` and `build --check
  --strict` still run; show each exit 0 (or its existing code).
- **The logger must not change any default output:** with no `-v`, stderr and the output file are byte-identical to
  before (compare against `main`).
- **A piped command reports the pipe's exit code.** Redirect and read `$?`. zsh does not word-split unquoted
  parameters; macOS has no `timeout(1)`.
- **Stay in your files:** `src/cli/{mod,ops,global_flags,logger}.rs`, `src/main.rs` if the spec says, `Cargo.toml`,
  `Cargo.lock`, `tests/cli_flags_honest.rs`, `tests/fixtures/svg/log_levels_40x30.svg`, `README.md`,
  `docs/cli-reference.md`, `docs/USAGE.md`, `docs/moat.md`, `SECURITY.md`, `AGENTS.md`, the spec's Build
  Completion, and (only after SPEC-151 merges) the two deferred docs. Anything else is a deviation; say why.

## Guardrails
- Every commit `git commit -s`, ending `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO fails any
  unsigned commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge.
- **Never poll CI.** Background `gh pr checks <PR> --watch`; confirm the checks exist first; when it exits read one
  snapshot at the PR's true head SHA, every leg including windows-latest.
- Scratch worktrees and temp files in a scratch directory, deleted when done.
- **Budget ~75 exchanges.** Checkpoint-push after each numbered step.

## When you finish
1. **Fill `## Build Completion`:** every file from `git diff --name-only origin/main...HEAD`; the RED-on-`main`
   lines then GREEN; each local mutation's red checks with lines; the Windows leg result; the deferred hunks (or
   that they were applied); a build-phase reflection naming any design call that turned out wrong.
2. **Measure your own cost.** Transcript under `~/.claude/projects/<slug>/` (absolute working directory with every
   `/`, `.`, `_` replaced by `-`); identify **by content**: search `BUILD prompt: no flag is accepted and ignored`
   (Python or `/usr/bin/grep`). ⚠ Dedup by `.message.id`: input, cache_creation, cache_read once per id, **MAX**
   `output_tokens`. Price per component at the model that ran (Sonnet $3/$15, Opus $5/$25; cache_creation ×1.25,
   cache_read ×0.10). Append a `build` entry to `cost.sessions`.
3. `just advance-cycle SPEC-149 verify`. Do **not** edit the spec's timeline file (the orchestrator keeps it).
   PR title: `fix(cli): no flag is accepted and ignored; -v shows warnings (SPEC-149)`. Body per AGENTS §13: Summary;
   Spec metadata; Decisions referenced (DEC-003, DEC-024); Constraints checked; the user-visible change for 0.8.3's
   notes (the spec's proposed text); and the Claude Code footer.

## Report back (your final message)
First line: `SPEC-149 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- commits, and that no mutation is in the diff; AC completion (11 rows);
- the RED-on-`main` lines, then GREEN; each local mutation's red checks;
- the quoted real `-v` line; `cargo tree -i log`, `cargo deny`, `just wasm-check` results;
- the Windows leg's result; gates run and their results;
- the deferred docs hunks (applied or deferred); files changed vs the spec's list; any design call that turned out
  wrong; any out-of-spec finding;
- your measured cost.
