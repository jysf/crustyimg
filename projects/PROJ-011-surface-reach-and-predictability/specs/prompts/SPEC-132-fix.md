# SPEC-132 — FIX prompt: the verify punch list, on the same draft PR #201

Cycle: **build** (punch-list follow-up). Fresh CLI session, **Sonnet**, on the PR's own branch:

```
just wt feat/spec-132-watch-recipe-assets -- --model sonnet
```

(That reuses the build's existing worktree if it is still there; close any old session in it
first.) You did not build or verify SPEC-132. Its verify returned a punch list, reached
independently by two verify sessions at the same head (`1a18a01`). Fix exactly the items below on
branch `feat/spec-132-watch-recipe-assets`, pushing to the **same draft PR #201**. The maintainer
pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git status` (must be clean) and `git log -1` (expect `1a18a01` or a later
commit on this branch). Then `git merge origin/main` (not rebase: the PR is shared and already has
merge commits) and resolve any `CHANGELOG.md` conflict by keeping every other spec's lines.
Read `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-132-build-watch-watches-recipe-asset-paths.md`
(`## Build Completion`, the `make_watcher` deviation) and `decisions/DEC-060-*.md`.

## Context: the out-of-spec fix this PR carries
`build --watch` on Linux rebuilt itself in a loop while idle (**207 builds in 45 s** on the pre-PR
code, measured by verify in Docker): inotify reports the build's own *reads* of the manifest,
recipes and assets as events. The PR's `make_watcher` filter (`is_read_only_access`) drops open /
read-close events and keeps write-close, modify, create, rename and remove. Verify confirmed: 0
idle builds after the fix; all five edit kinds (in-place write, temp+rename save, `cp` over,
delete+recreate, `touch`) still rebuild exactly once on Linux and macOS; only the inotify backend
ever emits those access kinds, so macOS and Windows are unaffected. The code is sound; the records
are not.

## Required
1. **`CHANGELOG.md` `[Unreleased]` → Fixed: a separate entry for the Linux loop.** It fixes a
   user-visible bug in released 0.7.x. Suggested: "**`build --watch` no longer rebuilds itself in
   a loop on Linux.** Its own reads of the manifest, recipes and watermark files were reported as
   changes, so the loop re-ran continuously while idle." Keep the existing asset-watch entry.
2. **DEC-060: record the event-kind rule.** Add an amendment line (follow how DEC-060 was amended
   by SPEC-132's design, dated) saying: open, read and read-close notifications are **not**
   changes; write-close, modify, create, rename and remove **are**; only the inotify backend sends
   the dropped kinds, so FSEvents and Windows are unaffected. Keep `affected_scope` in block-list
   form; `src/cli/build.rs` is already listed.

## Also do (cheap, and verify found each by a surviving mutation)
3. **AC-8(d):** give the test a string asset beside `image = 5` and assert **exactly** that one
   path comes back (today an empty result also passes it).
4. **AC-7(d):** add a root-level source case (`source = "*.png"` with `brand/logo.png`), asserting
   `brand` is registered once (kills mutation M4: removing the `.` special case in `covers`).
5. **AC-5:** assert exactly **one** `cannot watch` line for the still-missing root across rebuilds
   (kills M5: re-registering after every batch, which the design rejected).
For each of 3–5: show it **fails** under its mutation (apply, rebuild in a fresh target dir, run,
restore) and passes without it.

## Boundaries
Only `CHANGELOG.md`, `decisions/DEC-060-*.md`, `src/build/watch.rs` (tests), `src/cli/build.rs`
(the `watch_tests` module only) and `tests/build_watch.rs`. No behaviour change in `src/`. Do not
edit the spec's timeline (it lives on `main`). You may append a `build` entry for this fix to the
spec's `cost.sessions` on this branch and a line to `## Build Completion` naming the fix.

## Finish
- Clean matrix: default, `--no-default-features`, `--features webp-lossy`, each in a **fresh**
  `CARGO_TARGET_DIR`, **sequential** (pass feature flags as separate literal arguments; zsh does
  not word-split); tests + `clippy --all-targets -D warnings`; `fmt --check`.
- Every commit `git commit -s` (DCO). Push the branch. ⛔ The PR stays a **draft**; never mark it
  ready.
- **Never poll CI.** Background `gh pr checks 201 --watch`; when it exits, read a direct snapshot
  at the true head SHA and check every leg (ubuntu is the leg that matters for the Linux loop).
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the pipe's exit code. macOS has no `timeout(1)`.
- **Budget ~60 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-132 — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Sonnet
$3/$15 per MTok, Opus $5/$25; cache_creation ×1.25, cache_read ×0.10), never flat.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-132 fix: draft PR #201 at <head SHA> — CI <green | red: legs>`. Then: items 1–5
(done / not done); the three test mutations, each failing before and passing after; files changed
in this fix; your cost (`tokens_total`, `tokens_breakdown`, `estimated_usd`).
