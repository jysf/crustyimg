# SPEC-132 — RE-VERIFY prompt: the punch-list fix on draft PR #201

Cycle: **verify** (re-verify of a punch-list fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build SPEC-132, write its punch list, or fix it.

```
just wt verify/spec-132-fix -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-132-watch-recipe-assets
```
PR #201 is a **draft**; its head was `1e88c8d` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## What was verified before, and what the fix claims
SPEC-132 (`build --watch` watches recipe asset paths) was verified at `1a18a01` by two independent
sessions; both returned the same punch list. The code was judged sound, including an out-of-spec
fix: on Linux, inotify reported the build's own *reads* as events, so `--watch` rebuilt itself in a
loop while idle (207 builds in 45 s on 0.7.x). `make_watcher` now drops open / read-close events.
Read `## Verify Readout` and `## Build Completion` in
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-132-build-watch-watches-recipe-asset-paths.md`.

The fix (commits `775a4c9`, `1e88c8d`, after a merge of `main`) claims, with **no `src/` behaviour
change**:
1. `CHANGELOG.md` `[Unreleased]` → Fixed: a separate entry for the Linux loop.
2. `decisions/DEC-060-*.md`: a dated amendment — open, read, read-close are not changes;
   write-close, modify, create, rename, remove are; only inotify sends the dropped kinds.
3. **AC-8(d)** (`src/cli/build.rs`, `watch_tests`): a string asset beside `image = 5`; asserts
   exactly `["f.ttf"]` comes back. Kills **M3**: the asset collector returns nothing.
4. **AC-7(d)** (`src/build/watch.rs`): `watch_roots_root_level_source_covers_an_asset_dir_once`,
   `source = "*.png"` with `brand/logo.png`; asserts `.` is a recursive root and `brand` is
   registered **0** times as a root of its own (it is already covered by `.`). Kills **M4**:
   removing the `root == "."` special case in `covers`. The fix prompt said "registered once"; the
   orchestrator meant *watched once*, which is what this asserts — judge whether the test pins it.
5. **AC-5** (`tests/build_watch.rs`): `watch_warns_once_about_a_root_that_stays_missing` asserts
   exactly one `cannot watch … brand` line across three builds. Kills **M5**: re-registering
   missing roots after every batch.

The fixing session's report says it did not re-run the mutations itself; an earlier session on
the same branch did, and `## Build Completion` records it. Treat that as unproven.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope:** the fix is `f357f1c..HEAD` (the two merges before it bring in only `main`; confirm
   `git diff 8dddbfa f357f1c` is empty). `git diff --stat f357f1c HEAD` shows only the five files
   the fix names plus the SPEC-132 spec file, and every `src/` hunk is inside a `#[cfg(test)]`
   module.
2. **Records:** the CHANGELOG Fixed entry is accurate and other specs' entries are untouched; the
   DEC-060 amendment states the rule above, is dated, and `affected_scope` is still block-list
   form.
3. **Mutations, each yourself:** apply M3, M4, M5 one at a time, rebuild in a **fresh**
   `CARGO_TARGET_DIR` (reverting source does not rebuild the binary), and show the named test goes
   RED — and that it is the *only* test that goes RED for M4 and M3 (run the lib's watch tests in
   full). Restore, rebuild, show green. For M5 a single `tests/build_watch.rs` run under the
   mutation is enough; then confirm the test is not vacuous: the count it reads is 1, not 0, on the
   unmutated code (a positive control — e.g. temporarily print it, or check the log line exists).
4. **Flakiness:** run `tests/build_watch.rs` three times on the unmutated code; any failure is a
   finding, with the test name.
5. **CI:** `gh pr checks 201` at the head you verified — every leg, including ubuntu (the Linux
   loop), windows-latest, lean and the feature legs. Do not poll.

## Guardrails
`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to a
file. A piped command reports the pipe's exit code; redirect and read `$?`. zsh does not
word-split unquoted parameters. macOS has no `timeout(1)`. **Budget ~40 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-132 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Opus
$5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat. Report tokens_total, the
breakdown and estimated_usd.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #201 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #201 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then items 1–5, one or two lines each with evidence, then your cost.
