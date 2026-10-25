# SPEC-133 — BUILD prompt: `watermark` refuses to damage an image

Cycle: **build**. Fresh CLI session, own worktree, **Opus** (27 ACs across five surfaces, two
optimize branches and two build plans; thoroughness across that roster is the job):

```
just wt feat/spec-133-watermark-damage-rule -- --model opus
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own.

**What you are building:** a watermark that would not fit inside the image (text or image, after
`scale`; margin on the anchored edges only), or whose own pixels would cover more than 5 % of it,
is refused before anything is drawn. One explicit input → exit 2, nothing written. A batch,
recipe batch or `build` → that image is written **without the mark** (its other steps still
apply), one warning line per image plus a count, exit 0, lockfile written. A mark that passes is
byte-identical to `main`.

## Step 0 — precondition, then get onto current `main`
⛔ **SPEC-131 must be merged to `main` first.** Run `git fetch origin` and check
`git log origin/main --oneline | grep SPEC-131` shows its merged PR (a `feat(recipe): … (SPEC-131)
(#NNN)` commit). If it does not, **stop and report** — do not build against SPEC-131's branch.
Then `git rebase origin/main`. The spec must be at `cycle: build` with design commit `d0d10bd` in
your history.

## Read in order, before any code
1. **The spec** — `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-133-watermark-refuses-to-damage-an-image.md`.
   Read `## Frame`, `## Design`, all 27 ACs, `## Failing Tests` (including the changed-tests list
   and the N1–N9 negative-control table), `## Implementation Context` and `## Notes for the
   Implementer` in full. **Its design calls are settled.** If one turns out wrong, say so in Build
   Completion and your report; do not quietly re-decide it.
2. **SPEC-131's merged code**: `Watermark::from_params`, `WATERMARK_PARAMS` and the unused-by-mode
   check, in `src/operation/mod.rs` / `src/operation/registry.rs`. Your text-mode `scale` /
   `tile = true` refusal goes inside that mechanism (Call 5).
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Binding.
4. DEC-003, DEC-031, DEC-064, DEC-100, **DEC-085** (warn and exit 0), DEC-015 (exit codes),
   DEC-058/059/101 (cache and lockfile), DEC-102 (SPEC-131's). `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
- Option D (2026-10-24): fit test (primary) + **5 % coverage cap, fixed**. Measured
  geometrically, never SSIMULACRA2. Single explicit input → error, **exit 2**, naming the numbers.
- (2026-10-25) **A skipped image is written without the mark**, other steps applied; a skip is
  **not a failure** (no exit 6, `build` still writes its lockfile).
- (2026-10-25) **Skips alone exit 0**, one warning line per image and a skip count.
- (2026-10-25) **The cap applies to unscaled image overlays too** (an opaque 64×64 logo on
  256×256 = 6.25 % is refused). `scale` and `tile` are exempt from the cap; one tile must fit.
- (2026-10-25) **A cached `build` does not repeat the skip line** (accepted and documented, AC-23);
  the fix is its own STAGE-050 item. Do not change the cache or lockfile format.

## Reserved
📌 **DEC-104 is yours**, written at build, **block-list** `affected_scope` (an inline array is
silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/operation/mod.rs
  - src/pipeline/mod.rs
```
List every file it governs.

## Order
1. **Goldens first** (AC-10): the `#[ignore]`d bless test, run on an untouched tree, in its own
   commit. Verify re-runs it on `main` and diffs. A golden captured after a `src/` edit cannot fail.
2. **Write every new test, then run the whole suite on unmodified `src/`.** Record that each new
   test is RED for the reason the spec gives. Do **not** change the 12 existing fixtures yet: they
   are green on unmodified code, and they flip only once the rule exists.
3. Implement: the shared post-scale-size helper (both bit-depth branches + `refusal`) → `refusal()`
   → `run_with(Fail|Skip)` → each surface → exit code → reporting.
4. **Push a WIP commit as soon as it compiles**, before the matrix.
5. Now update the existing tests that the spec lists: **fixture sizes only**, plus SPEC-131's two
   rows (Call 5). After the change, each one must still assert what it asserted before. Verify will
   diff every changed test and refuse any change beyond a fixture size or an assertion on that
   size.
6. Negative controls N1–N9, one at a time, binary rebuilt each time, flips recorded.
   ⚠ Reverting source does not rebuild the binary.
7. Clean matrix (AC-27), docs (AC-25), PR.

## The things most likely to go wrong
- **Weakening a test to make it pass.** The 12 fixture changes are the one place a builder could
  hide a regression. Change a size, never an expectation. If a test needs more than a size change,
  stop and report it.
- **Counting coverage wrong.** Count the mark's own non-transparent pixels on `to_rgba16()`, not
  `to_rgba8()` (16-bit alpha 1..=256 becomes 0 in 8 bits). Integer maths only:
  `covered * 100 > w * h * 5`, all `u64`. Opacity does not change the count (AC-5).
- **A skip reaching the failure path.** In `build`, `failed > 0` returns `PartialBatch` and skips
  the lockfile (`src/cli/build.rs` ~823-829). A skip must never be counted there. AC-22 pins it
  with a skip + a real failure + a good image in one run.
- **Missing a loop.** There are five watermark-applying loops plus wasm: `watermark` (1 and N),
  `apply --recipe` plain (1 and N), `apply --recipe` ending in `optimize` (pinned-format branch
  and auto-decide branch, 1 and N), `build` (Preserve plan and Decide plan). Each AC names its
  surface; a test passing on one loop says nothing about the others.
- **`build --check` after a skip must stay clean**: the lock records the unmarked bytes. Add
  nothing to the lock record.
- **Shared files.** SPEC-134 (`--name-template`) edits the same CLI functions you do
  (`run_pixel_op`, `run_apply`, `encode_one`/`apply_one`, `build_one`, the build summary). It will
  not build at the same time as you, but if it merged first, rebase onto it and keep both
  behaviours; if you merge first, it rebases onto you. Touch only the functions the spec lists.
- **`just demo-build` is the wasm bundle-size gate** and `just check` never runs it. If it moves,
  stop and report the byte delta; do not edit the gate (DEC-066's ledger).

## Guardrails
- ⛔ No version bump, no release, no cache-schema or lockfile change. PROJ-011 ships as ONE release
  (0.8.0); merging is fine, tagging is the maintainer's.
- **Every commit `git commit -s`.** The "DCO sign-off check" fails any PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background; when it exits, read a direct
  snapshot at the PR's true head SHA and check every leg, including windows-latest and wasm/demo.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the **pipe's** exit code: redirect and read `$?`.
- zsh does not word-split unquoted parameters: loop with `while IFS= read -r`. macOS has no
  `timeout(1)`.
- Matrix legs **sequentially**, each with a fresh `CARGO_TARGET_DIR`. Never run a background build
  while editing source for a revert.
- Comments and user-facing text are behaviour-first, with no spec/DEC ids.
- **Budget ~180 exchanges.** This is the largest spec in the release. If the matrix is green and
  you are past that, stop and report what is left.

## When you finish
1. Fill `## Build Completion`: every file from `git diff --name-only origin/main...HEAD` (not from
   memory), the RED baseline, the N1–N9 flips, each changed existing test with its before/after
   fixture size, the matrix legs, the `demo-build` result, and an honest reflection naming any
   design call that turned out wrong.
2. **Measure your own cost.** Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where
   `<slug>` is your working directory's absolute path with every `/` and `.` replaced by `-`.
   Identify it **by content, not recency**: `grep -l "SPEC-133 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   ⚠ **Dedup by `.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing
   an id repeat identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens`
   while `output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`.
   Summing every line over-counts ~2×; taking the first line's output under-counts ~11×; both
   mistakes are on record. Price each component at the model that actually ran (Opus $5/$25 per
   MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate, cache_read ×0.10), never a flat rate.
   Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
   `estimated_usd` and the rate anchors.
3. `just advance-cycle SPEC-133 verify`. Open the PR: title
   `feat(watermark): refuse a mark that does not fit or covers over 5% (SPEC-133)`, body per
   AGENTS §13 (Summary, Spec metadata, Decisions referenced, Constraints checked, New decisions:
   DEC-104), ending with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-133 build: PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short: AC
completion (27 rows); RED baseline confirmed; N1–N9 flips; the changed existing tests (sizes
only — say so, or name any that needed more); matrix + `demo-build` result; files changed vs the
spec's list; any design call that turned out wrong; your measured cost.
