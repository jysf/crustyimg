# SPEC-145 — RE-VERIFY 2 prompt: the drift claim matches what f64 does

Cycle: **verify (re-verify 2)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did
not design, build or fix SPEC-145. Scope is narrow: did punch list 2 make its three text fixes,
and change nothing else? Short session.

```
just wt verify/spec-145-re2 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-145-crop:refs/remotes/origin/feat/spec-145-crop
git reset --hard origin/feat/spec-145-crop
```

The last re-verify ran at `0c6f722`. Punch list 2 pushed on top of it. Record the SHA you verify.
The maintainer will paste your final message back to the orchestrator, so it must stand on its
own.

## What the last re-verify found, and what punch list 2 was asked to do
- **Found:** DEC-112 and the spec called 0.4 px the maximum drift. In `f64`, values near a
  half-pixel tie reach 0.6: `width = 0.009583333333333333` gives 12 px at 1200 and 57 px at 6000.
  And the "+0.1 control" row said "green at 0.6", but it's red at 0.6 too (drift
  0.6000000000000227).
- **Asked:**
  1. Restate the claim everywhere as "0.4 measured over the seeded sweep; 0.6 is the bound in
     general, reached at `f64` near-ties", with the driven example. AC-5 and the test comment
     say the 0.4 bound holds for this seed only. Rewrap the touched lines, and restore DEC-112's
     backticks.
  2. Correct the +0.1 row: red at both bounds.
  3. Make the test's panic message print the bound the assert uses.

  Code and the test bound unchanged. Any `src/operation/crop.rs` hunk only inside `mod tests`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix.

## Check, in order
1. **Scope.**
   - `git diff --stat 0c6f722 HEAD` lists only DEC-112, the spec and `src/operation/crop.rs`.
   - Every `crop.rs` hunk is inside `mod tests`, and none changes the assert's bound
     (`0.4 + 1e-9`), the seed, or the case count.
   - `git diff 0c6f722 HEAD -- Cargo.toml Cargo.lock tests/` is empty.
   - Every commit is signed off.
2. **No false maximum is left.** Run `/usr/bin/grep -rn "0\.4\|maximum\|largest"` on DEC-112, the
   spec, and `src/operation/crop.rs`, and read every hit. Each one must be true: 0.4 as the
   sweep's measurement, 0.6 as the general bound. Quote any that still says 0.4 is the maximum.
3. **The near-tie, driven by you.** On this branch's release binary, crop `x = 0`, `y = 0`,
   `width = 0.009583333333333333`, `height = 1` on a 1200 px and a 6000 px-wide image made by a
   tool other than crustyimg. Quote both widths, and check the DEC's example matches them.
4. **The +0.1 row.** Read it. It must say red at both 0.4 and 0.6. If cheap, drive the mutation
   once against `0.6 + 1e-9` and say which way it goes. The row should be true at the bound it
   names.
5. **The message.** Set the bound to 0.3 temporarily, and confirm the panic names 0.3. Restore,
   rebuild, and confirm green.
6. **Nothing regressed.** Run the crop unit tests, `tests/crop.rs` and the default suite once, in a
   fresh target dir, with `--no-fail-fast`. Redirect stdout and read `$?`.
7. **CI.** Run `gh pr checks 244` once at the head, and say whether every leg is green.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep`.
- Never share a `CARGO_TARGET_DIR` with another session. Never poll CI. Delete scratch dirs and
  images, and leave the tree clean.
- **Budget ~15 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "RE-VERIFY 2 prompt: the drift claim" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #244 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #244 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then items 1–7, one or two lines each with evidence; anything punch list 2 broke; optional
cleanups, listed separately; and your cost, with the token breakdown.
