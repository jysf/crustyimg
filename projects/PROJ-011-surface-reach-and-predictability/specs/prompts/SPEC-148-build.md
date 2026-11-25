# SPEC-148 — BUILD prompt: `lint` fixes what it names

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the formatter's shape, every
rule's printed fix, the quoting rules, the tests and the mutation table. The care goes into the tests: every
check must be shown red where it should be, with its failure line.

```
just wt feat/spec-148-lint-fixes-what-it-names -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this prompt into a
fresh session and pastes your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and runs at the same time.** Stay out of `src/operation/`,
`src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
`skills/crustyimg/SKILL.md` and `CHANGELOG.md`. Use your own `CARGO_TARGET_DIR`; never share one. If a rebase
conflicts in a PROJ-017 file, stop and say so; do not resolve it.

⚠ **SPEC-147 (rotate/flip) builds on PROJ-017's side at the same time and has claimed these files until its PR
merges** (2026-11-05): `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`, `tests/recipe_round_trip.rs`,
`scripts/wasm-ci-check.sh`, `README.md`, `CHANGELOG.md`, and the docs and `src/` paths above. The design lists
`tests/common/mod.rs` for fixture helpers. **Do not edit it.** Put the new helpers (`jpeg_with_gps_tags`, the ICC/Make
helpers) in `tests/lint.rs`. The shared `jpeg_with_gps` in `common` has no coordinate, so the exit-7 check in
`tests/adoption_glue.rs` would go red under the new predicate: give `adoption_glue.rs` a coordinate-bearing fixture of
its own instead of changing the shared one. If that cannot work without editing a claimed file, stop and report; do
not edit it. Take DEC-114 or higher if a DEC is needed (DEC-113 is PROJ-017's); the design says none is.

## Rulings added after design (settled)
1. **PowerShell quoting on Windows** (POSIX elsewhere), as the spec designs it.
2. **`lint -` (stdin) prints no command** (`fix: null`); a real file named `stdin` still gets its fix.
3. **`lint --help`'s "(each with a runnable fix)" line is left alone** here; SPEC-150 rewords it. Do not edit
   `src/cli/mod.rs`.

## Step 0
`git fetch origin && git rebase origin/main`. Check the spec is at `cycle: build` and its design commit
`2cdc244` is in your history; read `rustup`/`cargo` versions and record `ImageMagick`/`exiftool` availability
(the CMYK fixture needs ImageMagick; if it is missing, stop and report). If any check fails, stop and report.

## Read in order, before any code
1. The spec in full: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-148-lint-fixes-what-it-names.md`
   (`## Frame`, `## Design`, all 11 ACs, the 12 failing tests, Implementation Context, the 16-row mutation table,
   Notes for the Implementer). **The design calls are settled.** If one turns out wrong, say so in Build
   Completion and the report; do not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: read-first and traps 1 to 16 (binding; 2, 10,
   11, 12, 13, 14, 15 apply directly). `/guidance/constraints.yaml`, DEC-050.
3. `src/lint/{mod,rules,report}.rs`, `tests/lint.rs`, `tests/common/mod.rs` (read only), `tests/adoption_glue.rs`,
   `docs/github-actions.md`.

## Order (verify checks each step)
1. **The tests first, against unmodified `main`.** Write FT-1 to FT-12 and the fixtures. Run them on `main`'s
   `src/` and quote each RED **with its failure line** (trap 13): the reds must be "stdout non-empty / file
   unchanged / finding still reported" and the GPS rows, not a harness bug or compile error. Commit and push a
   WIP checkpoint (long cycles must checkpoint).
2. **The formatter and quoting** (`src/lint/mod.rs`), then the rule fragments (`rules.rs`), then the report
   layer (`report.rs`: `fix: null`, SARIF without the `— fix:` suffix). Tests go green stage by stage.
3. **The GPS predicate** (4 coordinate tags + `GPSAreaInformation`); update the shared `jpeg_with_gps` fixture and
   the `GPSLatitudeRef`-only unit fixture so existing tests stay green for the right reason.
4. **Goldens:** update the three report goldens as the spec says, and show the hash-mismatch mutation (trap 13).
5. **Docs:** the `docs/github-actions.md` fix table and the doc comments the spec lists.
6. **Gates (read `$?`, no pipes):** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `just check`; plus the lean build (`--no-default-features`) per `verify-includes-lean-no-default-features-build`
   if the spec's checks touch feature-gated code. The Windows leg cannot be run here: say so, and make sure the
   harness fails (never skips) when its shell is missing.
7. **Local mutations (M-1 to M-14)** one at a time, in a scratch worktree: apply, check `git diff` is non-empty,
   run the checks the table names, record every red **with its quoted line**, revert exactly (`git diff` empty,
   trap 14), confirm green. M-12 (Windows forced to POSIX quoting) and M-13 (the test-only shell override) are
   driven as the spec says; if one needs the Windows runner, say it is left to the draft PR's Windows leg.
8. **Before the PR:** `git fetch origin`; rebase; rerun the lint tests. Open the **draft PR**.

## The things most likely to go wrong
- ⛔ Never push a mutation. `git diff origin/main...HEAD` must show none.
- ⛔ Nothing publishes or tags. No `git tag`, no `gh workflow run`, no branch-protection change.
- **A test that passes because its fixture can't fail** (trap 13): every fix row must be shown red with the old
  formatter put back verbatim.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
- **Quoting:** test the space, the `'`, the leading `-`, a Windows `\` path (unit-level on all OSes).
- **Stay in your files:** `src/lint/{mod,rules,report}.rs`, `tests/lint.rs`, `tests/adoption_glue.rs` (its own GPS fixture; NOT `tests/common/mod.rs`), `tests/fixtures/lint/cmyk_16x16.jpg`
  and its README, `docs/github-actions.md`, and the spec's Build Completion. Anything else is a deviation; say why.
- zsh: quote separators; no word-splitting of unquoted parameters. macOS has no `timeout(1)`.

## Guardrails
- Every commit `git commit -s`, ending `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO fails any
  unsigned commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge.
- **Never poll CI.** Background `gh pr checks <PR> --watch`; when it exits read one snapshot at the PR's true head
  SHA, every leg including windows-latest (that leg is where FT-1's PowerShell path first runs: quote its result).
- Scratch worktrees and temp files in a scratch directory, deleted when done.
- **Budget ~70 exchanges.**

## When you finish
1. **Fill `## Build Completion`:** every file from `git diff --name-only origin/main...HEAD`; the RED-on-`main`
   lines then GREEN; each local mutation's red checks with lines; the Windows leg result; a build-phase
   reflection naming any design call that turned out wrong.
2. **Measure your own cost.** Transcript under `~/.claude/projects/<slug>/` (absolute working directory with every
   `/`, `.`, `_` replaced by `-`); identify **by content**: search `BUILD prompt: lint fixes what it names`
   (Python or `/usr/bin/grep`). ⚠ Dedup by `.message.id`: input, cache_creation, cache_read once per id, **MAX**
   `output_tokens`. Price per component at the model that ran (Sonnet $3/$15, Opus $5/$25; cache_creation ×1.25,
   cache_read ×0.10). Append a `build` entry to `cost.sessions`.
3. `just advance-cycle SPEC-148 verify`. Do **not** edit the spec's timeline file (the orchestrator keeps it).
   PR title: `fix(lint): printed fixes run verbatim and clear their finding; GPS needs a coordinate (SPEC-148)`.
   Body per AGENTS §13: Summary; Spec metadata; Decisions referenced (DEC-050); Constraints checked; the
   user-visible change for 0.8.3's notes (proposed text from the spec's Q5); and the Claude Code footer.

## Report back (your final message)
First line: `SPEC-148 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- commits, and that no mutation is in the diff; AC completion (11 rows);
- the RED-on-`main` lines, then GREEN; each local mutation's red checks; the goldens' hash-mismatch mutation;
- the Windows leg's result (or what is left to it); gates run and their results;
- files changed vs the spec's list; any design call that turned out wrong; any out-of-spec finding;
- your measured cost.
