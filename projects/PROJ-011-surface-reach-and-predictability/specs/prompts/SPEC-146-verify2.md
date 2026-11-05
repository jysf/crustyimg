# SPEC-146 — VERIFY round 2 (narrow)

Opus, read-only, fresh worktree: `just wt verify/spec-146-r2 -- --model opus`. Target: PR #245 at the head SHA
the fix report names. Context: round 1 verified `5098e4f` and returned punch items P1 (`err()` to stderr) and
P2 (anchor the floor call); the fix prompt is `prompts/SPEC-146-fix.md`.

Check only:
1. `git diff 5098e4f..<head> --name-only` is limited to `scripts/wasm-ci-check.sh`, the spec, and optionally
   `ci.yml` comments / shape assertions. No `src/`, `tests/`, PROJ-017 path. Commits signed off.
2. P1: drive an empty lock through `set -euo pipefail; v=$(scripts/wasm-ci-check.sh lock-version <empty>)` and
   `runner-version <empty>`; the `::error::` line must appear on stderr. Restore the old `err()` exactly: the new
   guard must go red for its reason. `--self-test` 17/17 both ways as appropriate.
3. P2: re-apply the three decoys (comment, `|| true`, `echo`) for both full and slim; each must now FAIL
   `scripts/wasm-ci-check.sh workflow`. Try two more of your own (e.g. `if false; then … fi`-style wrapper,
   a trailing `# comment`-less variant with different spacing) and say which pass or fail.
4. Any optional assertions added: each defeated by its own mutation; the assertion count updated everywhere.
5. Merge `main` into a scratch tree: still 47 and 45 (or the counts the fix report states); note PR #244 order.
6. CI at the head: all legs incl. windows; the job prints 47/45.
Do not push to #245. Throwaway PRs only if you need a live row, from `verify-scratch/*`, closed with
`gh pr close --delete-branch`; `git ls-remote --heads origin 'verify-scratch/*'` empty at the end.
Report first line: `✅ APPROVED — merge PR #245 at commit <SHA>.` or `⚠ PUNCH LIST …` / `❌ REJECTED …`, then
checks 1–6, cost (dedup by `.message.id`, MAX output, per-component Opus $5/$25, cache ×1.25/×0.10).
