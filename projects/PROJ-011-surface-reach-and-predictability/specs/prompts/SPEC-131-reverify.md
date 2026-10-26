# SPEC-131 — RE-VERIFY prompt: the punch-list fix (draft PR #203)

Cycle: **verify** (re-verify of a follow-up fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build this fix, and you are not the session that wrote
the punch list.

```
just wt verify/spec-131-fix -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/fix/spec-131-punch-list
```
PR #203 is a **draft**; its head was `ee004f6` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## What it claims to fix
SPEC-131 is on `main` (`ba8b740`). Its retroactive verify returned three punch items (read
`## Verify Readout` in `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-131-typed-per-operation-parameter-structs.md`):
1. A positive number below f32's smallest value (`1e-50`) passed the range check, then read as
   `0.0` (`percent` → 1×1 image, `size` → no watermark, `scale`), all at exit 0. Fix: the range
   now applies to both the f64 and `f64::from(n as f32)` in `ParamKind::Number`'s check
   (`src/operation/registry.rs`).
2. No test for a one-step recipe whose only step is a keyed `optimize` marker. Fix: a sibling test.
3. One CHANGELOG Breaking line overstated. Fix: reworded.
The PR touches only `src/operation/registry.rs`, `tests/recipe_strict_params.rs`, `CHANGELOG.md`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Do not fix what you find; report it. Do not edit
the spec or timeline.

## Check, in order
1. **Reproduce the bug on `main` and its absence on the PR**, release binaries, each in a fresh
   `CARGO_TARGET_DIR`: `apply --recipe` with `percent = 1e-50`, `size = 1e-50`, `scale = 1e-50` →
   `main` exits 0 and writes (1×1 / unmarked); the PR exits 1, writes nothing, and names the key.
2. **Edges of the new comparison**, on the PR binary: a value that is tiny but still non-zero in
   f32 (`1e-40`, subnormal) is **accepted** for an exclusive-0 key; `opacity = 1.00000001` (reads
   as `1.0` in f32 but is > 1 as f64) is **refused**; `opacity = 1.0`, `-0.0` where 0 is allowed,
   and an integer value behave as before. Judge whether refusing on either side (f64 or f32) is
   the right rule, and say so.
3. **Mutations:** revert only the f32 comparison → exactly the new 1e-50 test goes RED; apply M10
   (skip the marker check for a one-step recipe) → exactly the new one-step test goes RED. Rebuild
   after each revert (reverting source does not rebuild the binary); run the **full** test suite
   under the first mutation, which the builder did not.
4. **Nothing else moved:** `git diff --stat origin/main...HEAD` is the 3 files; the CHANGELOG diff
   changes only SPEC-131's Breaking line (SPEC-132/134 entries untouched); `to_toml` goldens test
   still passes; DEC-102 already states the f32 rule (the builder says it does; check the line).
5. **CI:** `gh pr checks 203` at the head you verified, every leg (windows-latest, lean, MSRV,
   feature legs, browser smoke). The builder's local default and lean legs ran before a `cargo fmt`
   change, so CI is the evidence for the final SHA. Do not poll.

## Guardrails
`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to a
file. A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters.
macOS has no `timeout(1)`. **Budget ~40 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/` (`<slug>` = your working directory's
absolute path with `/` and `.` replaced by `-`); find it by content:
`grep -l "SPEC-131 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Opus
$5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat. Report tokens_total, the
breakdown and estimated_usd.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #203 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #203 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then items 1–5, one or two lines each with evidence, then your cost.
