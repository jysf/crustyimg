# SPEC-142 — RE-VERIFY 2 prompt: the corrected CPU sentence and the failure count

Cycle: **verify (re-verify 2)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did
not design, build or fix SPEC-142. Scope is narrow: did punch list 2 close its two items without
breaking anything? Short session.

```
just wt verify/spec-142-re2 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-142-ddphotos-parity:refs/remotes/origin/test/spec-142-ddphotos-parity
git reset --hard origin/test/spec-142-ddphotos-parity
```

Draft PR #234's head was `1001831` when this was written, and CI run `41229863113` there is green.
Record the SHA you verify. The maintainer will paste your final message back to the
orchestrator, so it must stand on its own.

## What the last re-verify found, and what punch list 2 claims
- **Found:** the spec's Build Completion correction named the wrong run for the EPYC 7763. And
  `N rows, M failed` read `0 failed` while a P5 failure exited 1.
- **Claimed:**
  - The correction now reads: the build's head run `41147305687` and the first run
    `41144102452` were on the EPYC 9V45; the 7763 was verify's run `41147581056` (`1b1c010`).
  - The summary line reads `N rows, R row failures, T table failures (names), K=… (cap …)`.
    Under the preview mutation it prints `1 table failure (P5)`; under N1, `9 row failures`.
  - **Deviation:** two table checks had no label, so the harness adds `rows-threw:` and
    `resize-set:` prefixes so each can be named.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. Don't edit the test bed.

## Check, in order
1. **Scope.**
   - `git diff --stat 474c9fc HEAD` lists only `tests/ddphotos_parity.mjs` and the spec.
   - `git diff 474c9fc HEAD -- src/ Cargo.toml Cargo.lock .github/` is empty.
   - Every commit is signed off.
2. **The sentence.** Read the corrected text, and check each of the three runs' CPU line once in
   its `ddphotos-parity` job log. Does every CPU named in the spec and in DEC-110 match its run?
3. **The summary line, driven by you.**
   - The preview mutation (`previewRecipe` returns its input): `1 table failure (P5)`, exit
     non-zero.
   - **Every table check can be named.** Read the harness for every place a table-level failure
     is recorded, and list them. Make each one fail once, or show by reading that it carries a
     label the summary prints. An unlabelled table failure that still counts, but prints no
     name, is an optional cleanup; one that **isn't counted at all** is a punch item.
   - N1: `9 row failures`, the same rows `{3,4,7,8,11,12,13,14,16}`.
   - Restore after each.
4. **Nothing regressed.** `--self-test` gives B1–B10, N1 and N2 their listed verdicts. Then
   `just ddphotos-parity` from clean (`rm -rf target/ddphotos-parity`) is green, reading
   `0 row failures, 0 table failures`.
5. **CI.** Run `gh pr checks 234` once at the head: the parity job and the matrix are green.

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Never poll CI; read each log once. Never drive the maintainer's browser.
- Delete scratch files and `target/ddphotos-parity`, and leave the tree clean.
- **Budget ~15 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "RE-VERIFY 2 prompt: the corrected CPU sentence" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #234 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #234 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then items 1–5, one or two lines each with evidence; the list of table checks and their labels;
anything punch list 2 broke; optional cleanups, listed separately; and your cost.
