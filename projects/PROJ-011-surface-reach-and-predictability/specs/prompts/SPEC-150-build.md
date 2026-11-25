# SPEC-150 — BUILD prompt: two messages tell the truth, and `--help` carries no spec ids

Cycle: **build**. Fresh CLI session, own worktree, **Opus** (the design asks for it: a string-literal lexer, the path
derivation and 25 mutation rows with wrong-cause traps are judgement work, not a mechanical sweep). The design spells
out every message, replacement sentence, doc line and mutation row. The care goes into the tests: every check must be
shown red where it should be, with its failure line.

```
just wt feat/spec-150-two-messages-tell-the-truth -- --model opus
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this prompt into a fresh
session and pastes your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017** (SPEC-152 is framed; a build may start while you work). Do **not**
edit: `src/operation/`, `src/recipe/`, `src/text/mod.rs`, `docs/recipes.md`, `recipes/README.md`,
`skills/crustyimg/SKILL.md`, `CHANGELOG.md`, `integrations/ddphotos/README.md`, `tests/common/mod.rs`,
`tests/wasm_roundtrip.rs`, `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
`tests/fixtures/recipe_corpus/`, `scripts/wasm-ci-check.sh`, `projects/PROJ-017-*`. Use your own `CARGO_TARGET_DIR`.
If a rebase conflicts in a PROJ-017 file, stop and say so; do not resolve it. Run `gh pr list` and
`git log origin/main --oneline -10` after your last rebase: if a PROJ-017 build has landed or is open and touches
`src/cli/mod.rs`, `src/wasm.rs`, `docs/cli-reference.md` or `AGENTS.md`, keep your hunks to the lines the spec names.
- **Shared but free (smallest hunk):** `src/cli/mod.rs`, `src/wasm.rs` (the refusal sentence only), `docs/cli-reference.md`,
  `AGENTS.md`. The proposed text for `docs/recipes.md:148` and `CHANGELOG.md` stays in the spec; the orchestrator
  places it.
- **Wasm counts:** this spec adds no wasm test. `tests/wasm_roundtrip.rs:270-283` asserts four words
  (watermark, image, filesystem, out of scope) that the new browser sentence keeps; confirm that test (or the existing
  assertion) still passes and do not edit the file. If you find you must add a wasm test, stop and say so.

## Rulings (settled; do not re-open)
1. A folder with no images stays an error, **exit 3**; only the wording changes, produced in **one** place.
2. Public library enums gain variants (`SourceError::{NoImagesInFolder, NoImagesMatched}`, `CliError::InputIsAFolder`).
   Accepted for 0.8.3 (the crate is 0.x and CLI-first); no `#[non_exhaustive]`.
3. `lint` with several arguments that all fail names the first cause. Accepted.
4. The subfolder hint shows for empty and text-only folders too. Accepted.
5. Ids in code comments are out of scope.

## Step 0
`git fetch origin && git rebase origin/main`. Check the spec is at `cycle: build` and the design commit `613bf23` is in
your history; record `cargo`/`rustup` versions. If any check fails, stop and report.

## Read in order, before any code
1. The spec in full: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-150-two-messages-tell-the-truth-and-help-carries-no-spec-ids.md`
   (`## Frame`, the 17 ACs, `## Failing Tests` incl. the baseline and the 25-row mutation table, Implementation
   Context incl. the settled design calls, the Docs table, the file list and Notes for the Implementer). **The design
   calls are settled.** If one turns out wrong, say so in Build Completion and the report; do not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: read-first, rules 1 to 7, traps 1 to 17 (binding;
   2, 10 to 17 apply directly). `/guidance/constraints.yaml`.
3. DEC-003 (amended by design), `src/source/mod.rs`, the 8 callers named in the spec, `src/cli/common.rs`,
   `src/metadata/mod.rs`, `src/wasm.rs:225-245`, `tests/cli_flags_honest.rs` (the model for a command-tree walk).

## Order (verify checks each step)
1. **The tests first, against unmodified `main`.** Write `tests/messages_tell_the_truth.rs` and
   `tests/help_has_no_internal_ids.rs` with their fixtures. Run them on `main`'s `src/` and quote each RED **with its
   failure line** (trap 13), as the spec's baseline section lists: FT-G1 red with 207 hit lines over 44 pages; FT-G3
   red on 4 shells (bash green); FT-G4 red on `src/wasm.rs:237` only; FT-G6 red on the 2 demo lines; FT-G2 and FT-G5
   green by design; FT-M1's `view` rows must fail on the message, not on "not a terminal". Commit and push a WIP
   checkpoint (long cycles must checkpoint).
2. **The source change:** `source::resolve_all(args, reader, OnEmpty::{Fail,Skip})`, the two error variants and exit
   arms, all 8 callers (`lint` uses `Skip`), `refuse_folder` and `CliError::InputIsAFolder` for `responsive`, `diff`,
   `meta copy`, `quoted()`. Checkpoint.
3. **The messages:** `reject_quality_with_auto` (verb argument), the `meta` template and producers (`copy` names
   whether `--from` or `--to`), the browser refusal sentence, `demo/demo.js:434`, `demo/index.html:163`. Checkpoint.
4. **The id sweep in help:** the 8 doc comments in `src/cli/mod.rs` with the spec's exact sentences, then
   `docs/cli-reference.md` (including `:312`: the spec says SPEC-111 is the wrong target; follow the spec's line).
5. **Docs** per the spec's table: `docs/api-contract.md`, `docs/architecture.md:148-151`, `AGENTS.md:501-503` (DEC-003
   itself was amended by design; confirm, do not re-amend). Do not touch the PROJ-017-owned lines.
6. **Gates (read `$?`, no pipes):** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (default **and**
   `--no-default-features`), `just check`, `cargo build --no-default-features` and
   `cargo test --no-default-features --test help_has_no_internal_ids --test messages_tell_the_truth`, `just wasm-check`.
   The Windows leg cannot run here: say so and quote it from the draft PR (CRLF, `\` paths).
7. **Local mutations: all 25 rows**, one at a time in a scratch worktree: apply, check `git diff` is non-empty, run the
   checks the row names, record every red **with its quoted line**, and the tests that must stay green; revert exactly
   (`git diff` empty, trap 14), confirm green. If time runs short, the spec's named cut is FT-G6's `include_str!` half
   and FT-U5: report before cutting.
8. **Before the PR:** `git fetch origin`; rebase; rerun both new test files and `tests/cli.rs`. Open the **draft PR**.

## The things most likely to go wrong
- ⛔ Never push a mutation. `git diff origin/main...HEAD` must show none.
- ⛔ Nothing publishes or tags. No `git tag`, no `gh workflow run`, no branch-protection change.
- **A guard that sees little** (traps 11, 17): `render_help()` on a subcommand sees 18 of 207 id lines. Render through
  `try_get_matches_from([..path, "--help"])`, derive the page set from each page's `Commands:` list, and show the
  count it saw (22 paths, 44 pages) and the mutation that limits the walk to the root red.
- **A wrong-cause pass** (trap 13): a missing path must still print `input not found or unreadable: <path>`; a
  `chmod 000` folder (including through `lint`) must too. An `is_dir()` re-check after the fact lies on those rows.
  Restore the old code **verbatim** as each control (trap 14).
- **The lexer:** `//` inside a string, raw strings, `'"'` char literals, lifetimes, continuation lines, `#[cfg(test)]`
  skipped by brace matching with code after the test module still scanned, CRLF normalised, and the pattern split so
  the test cannot match itself. All 12 lexer fixtures run.
- **Windows:** nothing skips silently (paths with `\`, CRLF). `quoted()` escapes `"` only; `{path:?}` is rejected
  because it doubles backslashes.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`. zsh does not word-split unquoted
  parameters; macOS has no `timeout(1)`.
- **Stay in the spec's file list** (above and in the spec). Anything else is a deviation; say why.

## Guardrails
- Every commit `git commit -s`, ending `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. DCO fails any
  unsigned commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge. (The maintainer clicks "ready for review" after green CI.)
- **Never poll CI.** Background `gh pr checks <PR> --watch`; confirm the checks exist first (a wait that exits while
  legs are pending has waited for nothing); when it exits read one snapshot at the PR's true head SHA, every leg
  including windows-latest.
- Scratch worktrees and temp files in a scratch directory, deleted when done.
- **Budget ~90 exchanges.** Checkpoint-push after each numbered step.

## When you finish
1. **Fill `## Build Completion`:** every file from `git diff --name-only origin/main...HEAD`; the RED-on-`main` lines
   then GREEN; each of the 25 mutations' red checks with lines; the Windows leg result; a build-phase reflection
   naming any design call that turned out wrong.
2. **Measure your own cost.** Transcript under `~/.claude/projects/<slug>/` (absolute working directory with every
   `/`, `.`, `_` replaced by `-`); identify **by content**: search `BUILD prompt: two messages tell the truth`
   (Python or `/usr/bin/grep`). ⚠ Dedup by `.message.id`: input, cache_creation, cache_read once per id, **MAX**
   `output_tokens`. Price per component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25,
   cache_read ×0.10). Append a `build` entry to `cost.sessions`.
3. `just advance-cycle SPEC-150 verify`. Do **not** edit the spec's timeline file (the orchestrator keeps it).
   PR title: `fix(cli): folders, help and a few messages tell the truth (SPEC-150)`. Body per AGENTS §13: Summary; Spec
   metadata; Decisions referenced (DEC-003); Constraints checked; the user-visible change for 0.8.3's notes (the
   spec's proposed text); and the Claude Code footer.

## Report back (your final message)
First line: `SPEC-150 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- commits, and that no mutation is in the diff; AC completion (17 rows);
- the RED-on-`main` lines, then GREEN; each of the 25 mutations' red checks (a table);
- the guard's page count, hit counts, and gates run with results (default, lean, wasm-check);
- the Windows leg's result; files changed vs the spec's list; any design call that turned out wrong; any out-of-spec
  finding;
- your measured cost.
