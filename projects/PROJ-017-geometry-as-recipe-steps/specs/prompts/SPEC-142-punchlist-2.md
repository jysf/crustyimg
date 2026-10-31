# SPEC-142 — PUNCH LIST 2 prompt: one spec sentence, and a failure count that counts every failure

Cycle: **build (punch list 2)**. Fresh CLI session, own worktree, **Sonnet**. Tiny: one spec
sentence and one harness summary line. You did not design, build or verify SPEC-142.

```
just wt fix/spec-142-punch2 -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-142-ddphotos-parity:refs/remotes/origin/test/spec-142-ddphotos-parity
git reset --hard origin/test/spec-142-ddphotos-parity
```
Push to the PR branch with `git push origin HEAD:test/spec-142-ddphotos-parity`. Draft PR
#234's head was `474c9fc` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
The re-verify confirmed both punch items closed.
- A crashing classifier now fails the `changes` step, with no `code=` written. The old body under
  the same crash exits 0 with `code=false`, as the control.
- P5 pins row 12, and the derived literal equals the spec's pasted text byte for byte.
- Native hashes match across the Mac and the Intel Xeon on all 16 rows.

Two things remain.
1. **One spec sentence names the wrong run.** The "Build Completion correction" (about spec lines
   1224–1225) says head run `41147305687` (`f204d43`) "ran on an AMD EPYC 7763, not the EPYC
   9V45". That run's log says **EPYC 9V45**, and so does `41144102452`. The 7763 was **verify's**
   run `41147581056`, at `1b1c010`. The original line (about `:1173`) was right.
2. **`N rows, M failed` hides table-level failures.** Under the preview mutation, P5 goes red and
   the run exits 1, but the line still reads `16 rows, 0 failed`. Two sessions have now tripped on
   it.

## The fix (settled, do not re-decide)
1. **Rewrite the correction** to say that verify's run `41147581056` (`1b1c010`) landed on a
   second CPU, an AMD EPYC 7763, and that the build's head run was on the EPYC 9V45 as recorded.
   Check each run's CPU line once in its log before you write it (`gh run view <run> --log --job
   <ddphotos-parity job>`).
2. **The summary line counts every failure.** Keep `<N> rows` as the row count. Make the failure
   count include the table-level checks (P1–P5 and anything else that fails outside a row line).
   For example: `16 rows, 0 row failures, 1 table failure (P5)`. Name each failing table check.
   - Exit status is unchanged: any failure exits non-zero.
   - Update the spec's one-line-format note to match.

**Not in scope:** replacing the derived row-12 literal with a verbatim one. The re-verify found it
pins placement and equals the pasted text, so leave it.

Stay in these files: `tests/ddphotos_parity.mjs` and the SPEC-142 spec. `git diff --stat 474c9fc`
must show nothing else, and `git diff 474c9fc -- src/ Cargo.toml Cargo.lock .github/` must be
empty.

## Prove it can fail
Run each of these one at a time and record the result. Redirect output and read `$?`.
- **The preview mutation.** Make `previewRecipe` return its input. The summary line must name
  P5 as a table failure, and the run must exit non-zero. Restore.
- **A row failure.** Re-drive N1. The line must count the red rows as row failures, with the same
  rows as before (`{3,4,7,8,11,12,13,14,16}`). Restore.
- Run `--self-test`: B1–B10, N1 and N2 give their listed verdicts.
- Then `just ddphotos-parity` from clean (`rm -rf target/ddphotos-parity`) is green, and its
  summary line reads 0 and 0.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No `src/`
  change. Don't edit the test bed.
- **Never poll CI.** Run `gh pr checks 234 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA. Read the `ddphotos-parity` job's log once, and quote
  its summary line and its CPU line.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Delete scratch files and `target/ddphotos-parity` when you're done.
- **Budget ~20 exchanges.**

## When you finish
- Add a `### Punch list 2 (re-verify)` subsection to the spec's `## Build Completion`, listing
  the two fixes, each control with its result, and the CI run id with its quoted lines.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-142 — PUNCH LIST 2 prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist-2` entry to the spec's `cost.sessions`.
  - Also append the **re-verify** session's entry, which it measured but couldn't write (it was
    read-only): input 70, output 15,114, cache_creation 77,893, cache_read 2,940,676, Opus,
    **$2.34**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 punch list 2: PR #234 at <head SHA> — CI <green | red: jobs>`.

Then, short:
- `git diff --stat 474c9fc`;
- the corrected sentence, quoted, and the CPU line from each of the three runs;
- the summary line under the preview mutation, under N1, and clean, quoted;
- the CI run id, its summary line and its CPU line;
- anything that did not go as written;
- your measured cost.
