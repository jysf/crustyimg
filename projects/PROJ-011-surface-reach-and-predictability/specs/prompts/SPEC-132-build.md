# SPEC-132 — BUILD prompt: `build --watch` watches recipe asset paths

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet** (an S spec with a fully specified
design; verify runs on Opus):

```
just wt feat/spec-132-watch-recipe-assets -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own.

**What you are building:** under `build --watch`, editing a watermark step's `image` or `font`
file rebuilds, wherever that file lives, including after the recipe is re-pointed to a new file
mid-session and when the file's directory is created after the watch started. Nothing outside
`build --watch` changes.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The spec must be at `cycle: build` and its design
commit `1145c31` must be in your history. If not, stop and report.

## Read in order, before any code
1. **The spec** — `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-132-build-watch-watches-recipe-asset-paths.md`.
   Read `## Frame`, `## Design`, the 12 ACs, `## Failing Tests`, `## Implementation Context` and
   `## Notes for the Implementer` in full. **Its six design calls are settled.** If one turns out
   wrong, say so in Build Completion and your report; do not quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Binding.
3. **DEC-060** (watch semantics; already amended on `main` by the design commit — do not write
   DEC-103, the reservation is released), DEC-061 (accepted risk #2: watch roots unclamped),
   DEC-100, `/guidance/constraints.yaml`.

## Order
1. **Write every failing test first and run it against unmodified `src/`.** The 4 `watch.rs` unit
   tests and 4 `recipe_asset_paths` tests are RED by not compiling; confirm that is the only reason.
   Then run the five end-to-end tests on unmodified code and confirm **each is RED for the reason
   it asserts** (a deadline on the bytes / `error:` observable), not a setup panic. Run AC-6
   (control) and confirm it is GREEN. Record all of it in Build Completion. A test never seen red
   for the right reason is not a guard.
2. Implement in the spec's order: `watch_roots` third argument → `recipe_asset_paths` /
   `watch_set_for` → `register_roots` collecting failed roots → `watch_impl` recompute + retry.
3. **Push a WIP commit once it compiles with the unit tests green**, before the end-to-end runs
   and the matrix (`git commit -s`, push the branch).
4. Negative controls (AC-9): revert C1, C2, C3 **one at a time**, rebuild the binary from scratch
   each time, record exactly which tests flip, restore. Evidence is the behavioural flip.
   ⚠ Reverting source does not rebuild the binary. Rebuild before each run.
5. Clean matrix (AC-11), docs (AC-12), PR.

## The things most likely to go wrong
- **The fake pass.** An asset beside the recipe, under a source root, or at the project root is
  already watched on `main`, so its test passes without your fix. Every end-to-end test's asset
  sits outside all of them, and each test's doc comment says where and why.
- **macOS fires a build on its own at startup.** Never assert on a raw attempt count from start.
  Baseline with `settled_attempts`, then assert the content observable: changed `dist/a.png` bytes
  for `image`, a new `error:` line for a corrupted `font`.
- **`notify`'s "cannot watch" text differs per OS.** For AC-5 assert only `cannot watch` and
  `brand` on the same line.
- **The lean build** (`--no-default-features`) has no `watch` feature. Keep `recipe_asset_paths`
  and `watch_set_for` behind `#[cfg(feature = "watch")]` or clippy fails on dead code; keep
  `watch::watch_roots` unconditional.
- **Stay in your functions.** In `src/cli/build.rs` you touch only `register_roots`, `watch_impl`,
  the new `recipe_asset_paths` / `watch_set_for`, and a new `mod watch_tests` placed right after
  `watch_impl` — **not** the shared `mod tests` at the end of the file. SPEC-134 will edit
  `prepare_target` / `target_format_plan` in the same file, and SPEC-131 is building in parallel
  (`src/operation/**`, `src/recipe/**`). `src/cli/common.rs`, `src/build/cache.rs`,
  `src/build/lock.rs` and `src/operation/**` must not appear in your diff (AC-10). Add your
  `CHANGELOG.md` entry without reflowing anyone else's.
- **Linux inotify and Windows are where watch timing differs**, and your machine is macOS. The
  3-OS CI is the real gate for the end-to-end rows: read every leg, not the summary.

## Guardrails
- ⛔ No version bump, no release, no cache-schema or lockfile change. Everything in PROJ-011 ships
  as ONE release (0.8.0); merging is fine, tagging is the maintainer's.
- **Every commit `git commit -s`.** The "DCO sign-off check" fails any PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background; when it exits, read a direct
  snapshot (`gh pr checks <PR>`) at the PR's true head SHA and check every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file, do not "fix" it. A piped command reports the **pipe's** exit code: redirect and read `$?`.
- zsh does not word-split unquoted parameters: loop with `while IFS= read -r`. **macOS has no
  `timeout(1)`.**
- Run matrix legs **sequentially**, each with a fresh `CARGO_TARGET_DIR`. Never run a background
  build while editing source for a revert.
- Comments and user-facing text are behaviour-first, with no spec/DEC ids.
- **Budget ~100 exchanges.** If the matrix is green and you are past that, stop and report what is
  left.

## When you finish
1. Fill `## Build Completion`: every file from `git diff --name-only origin/main...HEAD` (not from
   memory), the RED baseline, the three negative-control flips, the matrix legs, and an honest
   reflection naming any design call that turned out wrong.
2. **Measure your own cost.** Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where
   `<slug>` is your working directory's absolute path with every `/` and `.` replaced by `-`.
   Identify it **by content, not recency**: `grep -l "SPEC-132 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   ⚠ **Dedup by `.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing
   an id repeat identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens`
   while `output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`.
   Summing every line over-counts ~2×; taking the first line's output under-counts ~11×; both
   mistakes are on record. Price each component at the model that actually ran (Sonnet $3/$15 per
   MTok, Opus $5/$25; cache_creation ×1.25 the input rate, cache_read ×0.10), never a flat rate.
   Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
   `estimated_usd` and the rate anchors.
3. `just advance-cycle SPEC-132 verify`. Open the PR: title
   `fix(build): --watch watches recipe asset paths (SPEC-132)`, body per AGENTS §13 (Summary, Spec
   metadata, Decisions referenced: DEC-060 as amended, Constraints checked, New decisions: none),
   ending with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-132 build: PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short: AC
completion (12 rows); RED baseline confirmed (and the AC-6 control GREEN); the three
negative-control flips; matrix result; files changed vs the spec's list; any design call that
turned out wrong; your measured cost.
