# SPEC-155 — RE-VERIFY prompt: the two claim fixes, and nothing else

Cycle: **verify (narrow re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. The
first verify found two **claims** in `publish`; the punch list fixed both. You check those two
fixes and the sentences that now describe them. Everything else verify already approved: don't
re-open it. You did not design, build, verify or punch-list SPEC-155.

```
just wt verify/spec-155-re -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-155-atomic-writes:refs/remotes/origin/feat/spec-155-atomic-writes
git checkout --detach origin/feat/spec-155-atomic-writes
```

When this was written, draft PR #268's head was `4bec99b`. The orchestrator read its check-runs
there: 18 passed, 8 skipped (release-only), none failed, Windows and lean included. The punch-list
session may push one more commit that only replaces `CI_SNAPSHOT_PLACEHOLDER` in the spec's
`### Punch list (verify)`. If the head has moved, check that `git diff 4bec99b HEAD` touches only that
line; if it touches anything else, stop and report. Record the SHA you verify. The maintainer pastes
your final message back to the orchestrator, so it must stand on its own. **Write it as plain lists,
not box-drawn tables.**

## What the punch list claims
- **Fix 1:** `publish`'s step 2 matches `Ok(meta) if !meta.file_type().is_symlink()`; a symlink (and
  an error) gives no probe and no mode. Rows FT-E24 (lockfile linked to a `0600` file: exit 0,
  regular file, `0644` under umask 022, target untouched) and FT-E25 (dangling link: exit 0, regular
  file, nothing written through the link). Both red before the fix (`0755`; exit 5).
- **Fix 2:** `set_permissions` moved to straight after `create_new`, before the write loop. Row
  FT-U14 (a `StageFs` spy reads the staged mode at the first `write`): red before (`0644`), green
  after. Hand drive on the fixed code only: `-y` over a `0600` file under `ulimit -f 1` exited 153 and
  left the staged file at `0600`.
- **Mutants:** M12 → FT-E24, FT-E25. M13 → FT-U14. M4 re-run → FT-U5, FT-U11, FT-U14, FT-E13. M8
  re-run → FT-U6.
- **DEC-116:** "The symlink and overwrite guards (DEC-035) run before anything is created, on every
  output except the build lockfile, which has no symlink guard: a symlinked lockfile is replaced by
  a regular file, and nothing is read from or written through the link."
- **Matrix:** fmt, clippy and test green on default (1,390), `--no-default-features` (1,326) and
  `--features webp-lossy` (1,394); `just wasm-check` exit 0.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. Mutants and builds go in a
scratch worktree with its own `CARGO_TARGET_DIR` under your scratchpad; both are gone when you finish.

## Read first
- The spec on the PR branch: the mutation table, Failing Tests, AC-8, AC-14, the DEC's verbatim copy,
  and `### Punch list (verify)` under Build Completion.
- `git diff 471a0a7 HEAD -- src/sink/publish.rs tests/atomic_write.rs decisions/` in full.
- AGENTS §15: a bound or "can't happen" is a claim until its worst input is driven.

## Check, in order
1. **Scope.** `git diff --stat 471a0a7 HEAD` shows only `src/sink/publish.rs`, `tests/atomic_write.rs`,
   `docs/cli-reference.md`, `SECURITY.md`, the DEC and the spec. Every commit is signed off. The PR
   still merges cleanly onto current `main`.
2. **Fix 1, driven end to end.** Build `main` and the PR head `--release`, each in its own fresh
   `CARGO_TARGET_DIR`. On both, with umask 022, drive `build` with `crustyimg.build.lock` as:
   - a symlink to a `0600` file;
   - a dangling symlink;
   - a symlink to a directory;
   - a regular `0600` file (the mode must still carry over on the PR: this is the case fix 1 must
     not break).

   For each, give the exit code, what the lockfile is afterwards (`stat -f '%HT %Lp'`), and whether
   the link's target changed. The PR must match `main` on the three symlink cases and keep `0600` on
   the last.
3. **Fix 1 doesn't reach the guarded writers.** The four sink arms and `--save-recipe` refuse a
   symlinked destination before `publish`. Drive one sink arm (`-y` over a symlinked output) and
   `--save-recipe` over a symlink on the PR head, and confirm both are still refused with the same
   exit and message as at `471a0a7`.
4. **Fix 2, driven on both sides.** On `471a0a7` and on the PR head, `-y` over a `0600` file under
   `ulimit -f 1` with XFSZ at its default. Quote the staged file's mode on each: `0644` before, `0600`
   after. Then the same over a `0640` file and a `0400` file (the second as your own user: is it
   refused by the probe, as on `main`?). Also confirm the mode is applied to the staged path, not
   to the destination, by reading the destination's mode while the staged file is left behind.
5. **Mutants.** Each alone, `cargo clean -p crustyimg`, full `cargo test --no-fail-fast`, stdout to a
   file, red set read off the run:
   - M12 and M13: compare with the table;
   - one more, **M16**: keep fix 2 but make the guard `Ok(meta) if true` (a symlink is probed and its
     target's mode copied again, the pre-punch behaviour). It should match M12. If it doesn't, say
     why.

   M4 and M8 were re-run by the punch list; don't repeat them unless M12 or M13 disagrees with the
   table.
6. **Every claim in the new sentences.**
   - The DEC sentence says the guards run "on every output except the build lockfile". Re-run the
     writer sweep (`/usr/bin/grep -rn "publish(" src/`) and list every caller with whether it has a
     symlink guard before `publish`. A second unguarded writer makes the sentence false: that's a
     **wording** item, unless the writer's behaviour is wrong, which is a **claim**.
   - The sentence also says "the symlink and overwrite guards". Say whether the lockfile has an
     overwrite guard. If it doesn't, does the sentence imply it does?
   - "Nothing is read from or written through the link": confirm with step 2's rows.
   - The spec's verbatim copy of the DEC equals the DEC file byte for byte.
   - `docs/cli-reference.md`'s new sentence: the staged name in step 4 matches
     `.<name>.crustyimg-<…>.tmp`, and deleting it leaves the destination intact.
7. **CI.** `gh pr checks 268` once at the head you verify. Every leg, Windows and lean included.
8. **Bookkeeping, report only.** Whether `CI_SNAPSHOT_PLACEHOLDER` is still in the spec, and whether
   `cost.sessions` has a `verify` entry and a `punchlist` entry. Ship fixes these, so neither is a
   punch item.

## Classify every finding (AGENTS §15)
- **wording:** a doc, comment or spec line that misstates what the code already does;
- **assert:** a missing or loose test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or what the code does.

## Guardrails
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters, so loop with `while IFS= read -r`, and it reads `$B:r…` as a modifier, so
  brace it (`${B}:`). macOS has no `timeout(1)`. Use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails `display_sink_refuses_non_tty`.
- One fresh `CARGO_TARGET_DIR` per binary or mutant, never shared with another session.
- Never poll CI and never write your own wait loop.
- Scratch files only in your session's scratchpad, never `/tmp`. Restore any `chmod` before you delete
  a scratch folder. Leave the tree clean.
- **Budget ~25 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
  absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-155 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price at Opus's $5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10. You're read-only, so
  report the numbers; ship writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is exactly one of:
- `✅ APPROVED — merge PR #268 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #268 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–8, one or two lines each, with evidence;
- the punch items, numbered, each with its kind, evidence and expected fix;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`) and whether
  it is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
