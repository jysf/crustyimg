# SPEC-132 — VERIFY prompt: `build --watch` watches recipe asset paths

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build this.

```
just wt verify/spec-132 -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-132-watch-recipe-assets
```
PR #201 is a **draft**. Its head was `1a18a01` when this prompt was written (an "Update branch"
merge of docs-only commits on top of the build's `fbe329a`); record the SHA you verify.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

**What the build claims:** under `build --watch`, editing a watermark step's `image` or `font`
rebuilds wherever the file lives, including after the recipe is re-pointed mid-session and when
the asset's directory is created after the watch started; all 12 ACs done; C1–C3 flipped cleanly;
the matrix is clean. **Plus one change outside the spec:** a filter in `make_watcher` that drops
inotify open / read-close events, because on Linux `build --watch` re-triggered itself on its own
reads (~216 attempts in 45 s), which the builder reproduced on unmodified `origin/main` in Docker.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no version bump. Do not fix what you find:
report it with evidence and a suggested fix. Do not edit the spec or its timeline; the
orchestrator records your verdict and cost on `main`.

## Read first
The spec (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-132-build-watch-watches-recipe-asset-paths.md`
on the PR branch, all of it including `## Build Completion`), **DEC-060** (watch semantics, as
amended by the design), DEC-061 (accepted risk #2), DEC-100, and HANDOFF.md's "Traps this wave
paid for".

## Seven specific things — in this order

### 1 — ⚡ The out-of-scope `make_watcher` filter (judge it; it ships in 0.8.0 either way or not)
- **Reproduce the Linux self-trigger on pre-change code yourself** (Docker or any Linux), using the
  AC-6 control on `origin/main` before this PR: count attempts over an idle window. Then the same
  on the PR head. Record both numbers. If you cannot get Linux, say so plainly; do not assume.
- **No legitimate edit is lost.** The filter drops open and read-close; it must keep write-close,
  modify, create, rename and remove. Drive, on the PR binary (Linux if you can, macOS too): an
  in-place write, an editor-style atomic save (write temp + rename over the asset), `cp` over the
  file, delete then recreate, and a `touch`. Each must rebuild exactly once (settled baseline, then
  the content observable, as the tests do).
- **macOS and Windows unchanged:** read the filter's code and say which event kinds it can drop on
  FSEvents / ReadDirectoryChangesW (ideally none that matter). Check the Windows CI leg ran the
  watch tests (not skipped).
- **Is it recorded honestly?** DEC-060 governs loop semantics: does the filter need a line in
  DEC-060 (a recorded event-kind rule) — say yes/no with the reason. It fixes a **user-visible bug
  in the released 0.7.x**, so `CHANGELOG.md` needs its own **Fixed** entry; check it has one. Has
  a test that would catch the filter being removed (run that removal as a mutation, item 4)?

### 2 — The fake-pass trap
Every end-to-end test's asset must sit outside the manifest dir, every recipe dir and every source
root. Open each of the six tests and confirm the layout, and that each asserts a content
observable (changed `dist/a.png` bytes for `image`, a new `error:` line for `font`), never a raw
attempt count from start.

### 3 — The RED baseline is real
Check out `9d38d0f^` (the PR's base on `main` before any build commit) plus only the PR's test
files, and run them: the five end-to-end tests must fail on their asserted observable and the AC-6
control must pass. The builder moved AC-5's `cannot watch` assertion to the end of the test so the
final step is reached; confirm that move did not weaken it.

### 4 — Your own mutations, beyond C1–C3
Run C1–C3 again yourself (revert one, rebuild from scratch, run, restore). Then at least three of
your own, e.g.: remove the `make_watcher` filter (does a test go red on Linux CI, or only in
Docker?); make `watch_roots` watch asset dirs **recursively**; skip the "covered by a recursive
source root" dedup; make the recompute run only on recipe edits, not every batch. Name any mutation
no test catches. The builder reports C1 leaves the AC-8 non-string test green (an empty list passes
it): judge whether that test proves anything.

### 5 — The `CHANGELOG.md` conflict resolution
The build's merge of `main` (`fbe329a`) conflicted only in `CHANGELOG.md`, where SPEC-131's
entries had landed. Confirm SPEC-131's Breaking and Fixed lines survive intact and SPEC-132's entry
(and any filter entry) sit in `[Unreleased]` without reflowing others.

### 6 — Boundaries
- `git diff --name-only origin/main...HEAD`: `src/cli/common.rs`, `src/build/cache.rs`,
  `src/build/lock.rs` and `src/operation/**` must be absent (AC-10). In `src/cli/build.rs`, list
  every function touched: the spec allowed `register_roots`, `watch_impl`, the new
  `recipe_asset_paths` / `watch_set_for` and `mod watch_tests`; `make_watcher` is the declared
  extra. Anything else is undeclared.
- Existing `tests/build_watch.rs`, `tests/build.rs`, `tests/build_cache.rs`,
  `tests/build_watermark_cache.rs`, `tests/apply_batch.rs` tests pass unedited (diff them).
- `./scripts/decisions-audit.sh --changed origin/main` (pass the base ref) flags DEC-060 for this
  diff.
- **Cost:** the build's `cost.sessions` entry was measured before the Linux fix and merge turns;
  flag it as low (do not block).

### 7 — CI, read once
`gh pr checks 201` at the head you verified: every leg, especially **ubuntu-latest** (the inotify
leg the first run failed on), windows-latest, macOS, lean (`--no-default-features` compiles the e2e
tests out) and the feature legs. Do not poll; if legs are still pending, run
`gh pr checks 201 --watch` in the background and read a direct snapshot when it exits.

## Also
Matrix once, yourself: default, `--no-default-features`, `--features webp-lossy`, each in a fresh
`CARGO_TARGET_DIR`, **sequential**; tests + `clippy --all-targets -D warnings`; `fmt --check`.
Drive the AC-12 docs sentence on the binary (documentation has no green).

## Guardrails
- **Read-only. No commits. Do not fix what you find.** Scratch directories for every driven
  project; remove any scratch worktree you add (`git worktree remove`).
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the **pipe's** exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Never run a background build while editing source for a mutation; reverting source does not
  rebuild the binary.
- **Budget ~120 exchanges.** If past it, stop and report what you covered.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. Identify it **by content, not
recency**: `grep -l "SPEC-132 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by
`.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing an id repeat
identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while
`output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`. Summing every
line over-counts ~2×; taking the first line's output under-counts ~11×. Price each component at
the model that actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input
rate, cache_read ×0.10), never a flat rate. Report `tokens_total`, a `tokens_breakdown`,
`estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #201 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #201 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then lead with **item 1** (Linux self-trigger: attempts before vs after; every legitimate edit
kind still rebuilds: yes/no; DEC-060 line needed: yes/no; CHANGELOG Fixed entry present: yes/no),
then items 2–7, short. A punch list is numbered, each item with evidence and a suggested fix. End
with your cost readout.
