# SPEC-145 — RE-VERIFY prompt: the four cleanups, and nothing else moved

Cycle: **verify (re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did
not design, build or fix SPEC-145. Scope is narrow: did the punch list close its four cleanups
without breaking anything? Short session.

```
just wt verify/spec-145-re -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-145-crop:refs/remotes/origin/feat/spec-145-crop
git reset --hard origin/feat/spec-145-crop
```

The first verify approved draft PR #244 at `6611e79`. The punch list pushed on top of it. Record
the SHA you verify. The maintainer will paste your final message back to the orchestrator, so it
must stand on its own.

## What the punch list was asked to do
1. Tighten `crop_fraction_agrees_at_1200_and_6000`'s bound from 0.6 to 0.4, keeping the seed and
   the case count.
2. Add an explicit `assert_eq!(rows.len(), 4, …)` to the A refusal table.
3. Fix `recipes/README.md`'s "fails (exit 1)": true for one input only. A batch where every input
   fails exits 6, and a partial failure exits 6 with the valid outputs still written. Fix the same
   claim on any other crop doc surface.
4. Bring the spec's mutation table up to date (M4b's single-input exit 2, the extra red rows for
   M2, M4, M6 and M12), and change any stated 0.6 bound in the spec or DEC-112 to 0.4.

No `src/` change was allowed.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix.

## Check, in order
1. **Scope.**
   - `git diff --stat 6611e79 HEAD` lists only the crop test file(s), the doc surfaces from fix 3,
     the spec, and DEC-112 if fix 4 needed it.
   - `git diff 6611e79 HEAD -- src/ Cargo.toml Cargo.lock tests/fixtures/` is empty.
   - Every commit is signed off.
2. **The bound, driven by you.**
   - Read the test: the bound is 0.4, and the seed (`0x5EED_0145`) and the 200,000-case assert are
     unchanged.
   - Apply M1 (floor the right edge), rebuild clean, and confirm the test is red. Restore, rebuild,
     and confirm it's green.
   - Read the punch list's own "0.6 but not 0.8" mutation in its Build Completion. Say whether it
     really lands between the bounds: green at 0.6, red at 0.4.
3. **The A-table assert.** Read it. Comment out one A row, and say what fails: the new assert,
   or only the array type. Restore.
4. **The docs claim, driven.** On this branch's binary, drive one input with a 0 px crop, a
   two-input batch where both fail, and a two-input batch with one bad and one good input. Quote
   each exit code, and say whether any output was written. Then read every changed doc line, and
   say whether it matches what you drove. Grep the crop doc surfaces for any `exit 1` claim the
   punch list missed.
5. **The spec text.** Read the mutation table's M2, M4, M4b, M6 and M12 rows against the first
   verify's account: the M4b single input exits 2; a batch exits 0, writing both images
   uncropped. If DEC-112 changed, read the diff. It must change only the stated bound.
6. **Nothing regressed.** Run the crop test file, `tests/docs_ops.rs`, and the default suite once,
   in a fresh target dir with `--no-fail-fast`. Redirect stdout and read `$?`.
7. **CI.** Run `gh pr checks 244` once at the head, and say whether every leg is green.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep`, so use `/usr/bin/grep` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Never poll CI. Delete scratch target dirs
  and files, and leave the tree clean.
- **Budget ~20 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-145 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #244 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #244 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then items 1–7, one or two lines each with evidence; anything the punch list broke; optional
cleanups, listed separately; and your cost, with the token breakdown.
