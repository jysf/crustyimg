# SPEC-141 — RE-VERIFY 2 prompt: the real golden-file lookup is pinned

Cycle: **verify (re-verify 2)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did
not design, build or fix SPEC-141. Scope is narrow: did punch list 2 close its one item without
breaking anything? Short session.

```
just wt verify/spec-141-re2 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-141-recipe-output-guard:refs/remotes/origin/test/spec-141-recipe-output-guard
git reset --hard origin/test/spec-141-recipe-output-guard
```

Draft PR #233's head was `dec2c6a` when this was written (a spec-only commit). `6376d65` is the
last code commit, and CI run `41229750086` there is green. Record the SHA you verify. The
maintainer will paste your final message back to the orchestrator, so it must stand on its own.

## What the last re-verify found, and what punch list 2 claims
- **Found:** FT-9 never called `golden_path_for`, so putting the old `|| !path.exists()` panic
  back verbatim (mutation A) left all 9 tests green.
- **Claimed:** `a_target_without_goldens_fails` is extended: a known target with an absent file
  must return `Ok`. Mutation (A) turns it red with exit 101.
- **Cleanups claimed:** the stray-file check prints `missing: [...]` and `extra: [...]`. FT-4 now
  says "differ from the golden's png row". The README lists the `GOLDEN_LINES` doc comment.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix.

## Check, in order
1. **Scope.**
   - `git diff --stat 6b90d71 HEAD` lists only the test, the README and the spec.
   - `git diff 6b90d71 HEAD -- 'tests/fixtures/recipe_corpus/*.output' src/ Cargo.toml Cargo.lock`
     is empty.
   - `git diff --name-only 6376d65 HEAD` lists only the spec.
   - Every commit is signed off.
2. **Mutation (A), driven by you.** Put the old panic back in `golden_path_for` exactly, rebuild
   clean, and run `cargo test --test recipe_corpus_output`. The extended test must go red. Then
   restore, rebuild clean, and show it green.
3. **Read the extended test whole.**
   - Does it use a stem that can never exist, so no fixture is touched?
   - Does it still assert FT-8's `Err` for an unknown target?
   - Could it pass if `golden_path_for` returned `Ok` with the **wrong** path? If so, that's a
     punch item.
4. **The stray-file message.** Move `bundled_web.linux-x86_64.output` aside: the failure names it
   under `missing`. Restore it, and show `git status --short` empty.
5. **CI.** Run `gh pr checks 233` once at the head. If the spec-only head has no run of its own,
   say so; the code is `6376d65`'s.
6. **Full suite.** Run the default leg once, in a fresh target dir, with `--no-fail-fast`.
   Redirect stdout and read `$?`.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep`
  for anything you count.
- Never poll CI. Delete scratch target dirs, and leave the tree clean.
- **Budget ~15 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "RE-VERIFY 2 prompt: the real golden-file lookup" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #233 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #233 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then items 1–6, one or two lines each with evidence; anything punch list 2 broke; optional
cleanups, listed separately; and your cost.
