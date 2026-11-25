# SPEC-146 — FIX prompt: verify punch list for PR #245

Cycle: **build (fix round)**. Fresh CLI session, **Sonnet**, on the existing PR branch. Verify (Opus,
`5098e4f`) returned a PUNCH LIST: the job works, every live row went red for its own reason, the merged
tree runs 47 and 45. Two small defects in `scripts/wasm-ci-check.sh`, plus optional shape-check hardening.

```
just wt fix/spec-146 -- --model sonnet
```

Then, in the worktree: `git fetch origin && git checkout -B fix/spec-146 origin/feat/spec-146-wasm-roundtrip-ci`
and work on that branch tree. You will **push to `feat/spec-146-wasm-roundtrip-ci`** (the PR's branch):
`git push origin HEAD:feat/spec-146-wasm-roundtrip-ci`. The PR stays a **draft**; no `gh pr ready`, no merge,
no tags. Use your own `CARGO_TARGET_DIR`.

⚠ Another orchestrator thread owns PROJ-017 (`src/operation/`, `src/recipe/`, `docs/recipes.md`,
`tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`). Stay out. Files you may touch:
`scripts/wasm-ci-check.sh`, the SPEC-146 spec (Build Completion addendum only), and, only if the optional
items below need it, `.github/workflows/ci.yml` comments. Nothing else.

## P1 — `err()` must write to stderr (required)
`err()` writes its `::error::` line to stdout. The lock step runs `version=$(scripts/wasm-ci-check.sh lock-version)`
and `runner_version` runs `want=$(lock_version …)`, so each captures the error into a variable and CI prints
nothing (driven: an empty lock exits 1 silently). The spec promises `found N wasm-bindgen entries in Cargo.lock,
expected 1`. Fix: `err() { echo "::error::…" >&2; }` (keep your existing message format).
- RED first (trap 13): with the old code, show an empty-lock run through `set -euo pipefail; version=$(…)` exits 1
  with **no output**, and the same for `runner-version <empty lock>`. Quote both.
- GREEN: same runs now show the `::error::found 0 … expected 1` line on stderr. Self-test captures `2>&1`; confirm
  `scripts/wasm-ci-check.sh --self-test` still passes 17/17.
- Add a self-test row (or a shape assertion) that fails if `err()` goes back to stdout, and show the old code makes
  it red (trap 14: restore exactly after).

## P2 — anchor the floor call (required)
Shape-check assertions 10 and 11 accept the count call as a substring. These each leave C1 at 13/0-failed while
the live job would stay green on a compiled-out module:
- `# scripts/wasm-ci-check.sh count full "$log"` (commented out)
- `scripts/wasm-ci-check.sh count full "$log" || true`
- `echo scripts/wasm-ci-check.sh count slim "$log"`

Fix: match the call as a whole line, `^\s*scripts/wasm-ci-check.sh count (full|slim) "\$log"\s*$` (adapt to the
tool the assertions already use, keep it POSIX-grep/awk compatible with macOS and Linux runners).
- RED: apply each of the three decoys (full and slim, so 6 mutations) in a scratch copy of `ci.yml`, run
  `scripts/wasm-ci-check.sh workflow`, and quote the new FAIL line for each. Before the fix, show at least the
  three above are green (13 assertions, 0 failed), so the control is proven live.
- GREEN: real `ci.yml` passes; every mutation reverted, `git diff` empty after each (trap 14).

## Optional hardening (do if cheap; skip and say so if not)
Add shape assertions for: no `continue-on-error` in the job; the slim step's `if:` is exactly
`!cancelled() && steps.wasm-check.outcome == 'success'`; no job-level `env:`. Each needs its own mutation red
with the failing line quoted. Count the assertions and update any hard-coded "13" (spec, script, docs) in one
sweep with a grep. If this gets large, drop it and file it as a STAGE-053 line in the report instead.

## Not in scope
Header wording ("CI runs every subcommand"), the spec's C10-on-main procedure, the frame's npm_smoke claim:
the orchestrator edits spec text on `main`. Do not touch them.

## Before pushing
1. `git fetch origin`; if `tests/wasm_roundtrip.rs` or `src/wasm.rs` changed on `main`, rerun `just wasm-test`
   and `just wasm-test-slim`, report the counts, update `FULL_COUNT`/`SLIM_COUNT` only if they moved.
2. Run `scripts/wasm-ci-check.sh --self-test`, `scripts/wasm-ci-check.sh workflow`,
   `scripts/ci-code-paths.sh --self-test`, `actionlint .github/workflows/ci.yml` (only the baseline SC2001 allowed).
   Redirect and read `$?`; no pipes.
3. `git commit -s` each commit, ending with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
4. Push to the PR branch, background `gh pr checks 245 --watch` (never poll), then read one snapshot at the head SHA.
   Confirm `wasm round-trip tests (Node)` still prints 47 and 45 passed.
5. Append a short "Fix round" paragraph to the spec's Build Completion, and a `build-fix` entry in `cost.sessions`
   (dedup JSONL by `.message.id`, MAX output, per-component pricing; find the transcript by searching
   `FIX prompt: verify punch list`).

## Report (final message, pasted to the orchestrator)
First line: `SPEC-146 fix: PR #245 at <head SHA> — CI <green | red: legs>`.
Then: P1 red/green lines; P2 red lines per decoy and green; optional items done or dropped; counts seen; files
changed (`git diff --name-only 5098e4f..HEAD`); anything out of spec; measured cost.
