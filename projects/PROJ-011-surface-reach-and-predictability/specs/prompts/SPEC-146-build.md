# SPEC-146 — BUILD prompt: CI runs the wasm round-trip tests

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the YAML, the script
and the docs sentences word for word, so the build is mostly transcription. The care goes into the
checks: every check must be shown red where it should be, with its failure line.

```
just wt feat/spec-146-wasm-roundtrip-ci -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and is running at the same time.** Its `crop` build
edits `src/operation/`, `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`,
`tests/fixtures/recipe_corpus/` and may add wasm tests. Stay out of all of them. Use your own
`CARGO_TARGET_DIR`; never share one with another session. If a rebase conflicts in a PROJ-017
file, stop and say so; do not resolve it.

## What you are building
- `.github/workflows/ci.yml`: a new job `wasm-roundtrip` (gated by `changes`), two new steps in the
  `metadata` job, and the `ddphotos-parity` comment block replaced.
- `scripts/wasm-ci-check.sh` (new, with `--self-test`) and `scripts/ci-code-paths.sh` (`\.cargo/`
  added, new table rows, the summary that counts mismatches, crashes and junk).
- Comment-only edits in `justfile` and `.cargo/config.toml`, the install hint reading the version
  from `Cargo.lock`, one line in `AGENTS.md` §6, one sentence each in `docs/roadmap.md` and
  `docs/research/proj-008-wasm-build.md`.
- No `src/` change. No DEC.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`, and its design commit `db22e57` is in your history;
- `actionlint --version` works (record it), and `rustup target list --installed` shows
  `wasm32-unknown-unknown` for the stable toolchain `just wasm-check` uses.

If any of those fails, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-146-ci-runs-the-wasm-round-trip-tests.md`.
   Read in full: `## Frame` (its rulings), `## Design`, all 11 ACs, `## Failing Tests` (C1 to C12
   and the self-test rows for `wasm-ci-check.sh`), `## Implementation Context` (the job, the
   script, the `metadata` steps, the classifier, the docs, the mutation table M1 to M17), and
   `## Notes for the Implementer`. **The design calls are settled.** If one turns out wrong, say so
   in Build Completion and in your report; do not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for". Binding; traps 11, 12, 13, 15 and 16 apply directly. And
   `/guidance/constraints.yaml`.
3. DEC-064, DEC-065, DEC-066, and `.github/workflows/ci.yml`, `scripts/ci-code-paths.sh`, the
   `justfile` wasm recipes.

## Order (verify checks each step)
1. **The checks first, against unmodified `main`.** Write `scripts/wasm-ci-check.sh` and the new
   classifier rows. Run C1, C3 and C10 on `main`'s `ci.yml`/classifier and quote each RED **with its
   failure line** (trap 13): the failures must be "job `wasm-roundtrip` missing" and
   `.cargo/config.toml -> false, expected true`, not a script bug. Commit and push (a WIP
   checkpoint: long cycles must checkpoint).
2. **The classifier change,** then C10 and C11 go GREEN.
3. **The job and the two `metadata` steps,** as the spec gives them. Run
   `scripts/wasm-ci-check.sh --self-test`, `scripts/wasm-ci-check.sh workflow`,
   `scripts/ci-code-paths.sh --self-test` and `actionlint .github/workflows/ci.yml`. The only
   actionlint finding allowed is the baseline SC2001 note at the `changes` job. Do not "fix" it.
4. **The local checks you can drive:** C4 (stub runner printing `0.2.125`, then no runner on
   `PATH`), and the floor shape of C7 (the `tests/wasm_roundtrip.rs:30` `#[cfg(any())]` change,
   then restore it exactly; `git diff` empty, and say so). Use your own target dir. Quote every
   output and read `$?` directly, with no pipe.
5. **Docs and comments** per the spec's "Docs, word for word", then C12.
6. **Before you open the PR:** `git fetch origin`. Run `git diff origin/main -- tests/wasm_roundtrip.rs`
   and `git log origin/main --oneline -- tests/wasm_roundtrip.rs src/wasm.rs`. If `main` now has
   more or fewer wasm tests than 47 and 45, run `just wasm-test` and `just wasm-test-slim` again,
   set `FULL_COUNT` and `SLIM_COUNT` to the numbers you see, and say so in the report. The floor is
   an exact count, so a wasm test added on `main` by another thread changes it.
7. **Open the draft PR** and let CI run. Record C5: the `wasm-roundtrip` job's log lines showing
   `47 passed` and `45 passed` (or your updated numbers), quoted, and the job's name, conclusion
   and wall time. Push one trivial doc touch only if you need a second run for the warm time; the
   spec asks for cold and warm times.
8. **Run the mutation rows you can drive locally** (M1, M2, M3, M5 shape, M8, M12 to M17), one row
   at a time in a scratch worktree: apply it, check `git diff` is non-empty, run C1, C3, C10 and
   C11 as the table says, record every red check **with its quoted line**, then revert and confirm
   green. M4, M6, M7, M9, M10 and M11 are live rows that verify drives; do not push them.

## The things most likely to go wrong
- ⛔ **Never push a mutation.** M4, M6 and the others that change `src/wasm.rs`,
  `tests/wasm_roundtrip.rs` or the install version are for verify's throwaway commits. Your
  `git diff origin/main...HEAD` must show none of them.
- ⛔ Nothing publishes or tags. No `git tag`, no `gh workflow run`, no branch-protection change.
- **A piped `cargo test` hides its exit code.** The job's steps use `>log 2>&1 || rc=$?` with no
  pipe, as the spec shows. Do not "simplify" them.
- **No `env:` anywhere in the new job** (trap 16). If you think one is needed, stop and report.
- **Write the YAML as given.** Action tags, `fallback: none`, `package-manager-cache: false`, the
  Node `'22'` pin and the error-message wording come from the spec and are checked by C1.
- **The shape check and the self-test run in `metadata`, not in `wasm-roundtrip` or `changes`.**
  The spec says why. Do not move them.
- **The job name is stable.** The check's name is the job's `name:`; the maintainer will require
  it by that name.
- zsh: quote `"===="`-style separators, and don't rely on word-splitting. macOS has no
  `timeout(1)`.
- **Stay in your files:** `.github/workflows/ci.yml`, `scripts/ci-code-paths.sh`,
  `scripts/wasm-ci-check.sh`, `justfile` (comments and install hint), `.cargo/config.toml`
  (comment), `AGENTS.md` §6, `docs/roadmap.md`, `docs/research/proj-008-wasm-build.md`, and the
  SPEC-146 spec (Build Completion only). Anything else is a deviation; say why.

## Guardrails
- **Every commit uses `git commit -s`,** ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails any PR with an
  unsigned commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge.
- **Never poll CI.** Background `gh pr checks <PR> --watch`. When it exits, read one direct
  snapshot at the PR's true head SHA, every leg including windows-latest. A watch that exits before
  the checks register has waited for nothing: confirm the checks exist first.
- A piped command reports the pipe's exit code: redirect and read `$?`.
- Delete scratch worktrees and stub directories when done. Keep scratch files in a scratch
  directory, never in `~`.
- **Budget ~70 exchanges.** If the PR run is green and you are past that, stop and report what is
  left.

## When you finish
1. **Fill `## Build Completion`:**
   - every file from `git diff --name-only origin/main...HEAD`, not from memory;
   - C1, C3, C10 RED on `main` with their lines, then GREEN; C4, the local part of C7, C5, C12
     quoted;
   - each local mutation's red checks, with lines;
   - the cold and warm job times;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**: search for `BUILD prompt: CI runs the wasm round-trip`.
     `grep -c` can come back empty through the `rtk` wrapper; count with Python or `/usr/bin/grep`.
   - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
     that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
     **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
     output under-counts about 11×.
   - Price each component at the model that actually ran (read `.message.model`), never at a flat
     rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
     cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance and leave the PR a draft.**
   - Run `just advance-cycle SPEC-146 verify`.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `ci(wasm): run the wasm round-trip tests on every code change (SPEC-146)`.
   - Body per AGENTS §13: Summary; Spec metadata; Decisions referenced (DEC-064, DEC-065,
     DEC-066); Constraints checked; the maintainer's follow-up (make the job a required check
     after its first green run on `main`); and the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-146 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commits, and that no mutation is in the diff;
- AC completion (11 rows);
- C1, C3, C10 RED on `main` (with lines), then GREEN; C4, C7 (local), C5, C12;
- the wasm test counts you saw, and whether you changed `FULL_COUNT`/`SLIM_COUNT`;
- each local mutation's red checks;
- cold and warm job times;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
