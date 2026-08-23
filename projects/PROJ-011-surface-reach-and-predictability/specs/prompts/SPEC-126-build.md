# SPEC-126 — BUILD prompt

Cycle: **build**. You are **Sonnet 5** — the spec's `implementer` says so and the dispatch matches;
price at **Sonnet anchors ($3/$15 per MTok)**, read from `.message.model` in your own transcript,
not from this sentence.

**One-line summary:** `apply`'s multi-input path does no output-format resolution at all — it
ignores `--format` entirely and preserves the source format, while `apply` at one input defaults to
PNG. The two arities disagree in both directions, and neither matches `build`.

**All four design calls are settled. None is left to you.** Your job is the fix, the controls and
the tests.

## Read in order

1. **The spec** — `.../specs/SPEC-126-apply-and-build-agree-on-output-format.md`. **7 ACs, 4
   settled calls, 4 failing tests.**
2. **The stage** — `STAGE-049-apply-and-build-agree.md`, and **PROJ-011's `brief.md`** for why this
   is the project's entry point.
3. **DEC-005** (recipes round-trip through the registry), **DEC-058** (the lockfile cache key —
   why `build` and `apply` disagreeing matters beyond tidiness).
4. `/AGENTS.md` §4, §11, §12, §13, §15.

## The measured defect, to reproduce before you change anything

JPEG source, plain pixel recipe (auto-orient + resize, no terminal `optimize`):

```
apply 1 JPEG,  no --format    -> src.png    <- the source format is CHANGED
apply 2 JPEGs, no --format    -> src.jpg    <- preserved
apply 1 JPEG,  --format png   -> src.png    <- honoured
apply 2 JPEGs, --format png   -> src.jpg    <- the flag is SILENTLY IGNORED
```

⚠ **`--name-template` is not the discriminator** — an explicit `{stem}.{ext}` behaves identically.
It is purely single-input vs many. **Reproduce all four rows before touching code**; a fix aimed at
the wrong discriminator will pass a bad test.

## ⚡ Call 1 is settled BY MEASUREMENT — do not re-litigate it, but do record it

**Preserve the source format.** Not because it is obviously right — the opposite is arguable, since
a plain pixel recipe has no format opinion and PNG avoids JPEG→JPEG generation loss — but because
**`apply` at one input is the sole outlier on the whole surface**:

```
resize / thumbnail / watermark  -> preserve
build                           -> preserve
apply, 2 inputs                 -> preserve
apply, 1 input                  -> PNG        <- the only one that differs
```

**So `apply`-single moves and nothing else does.** ⚠ **Put that reasoning in the DEC**, including
why the opposite case loses: consistency across six paths beats a local optimum on one, and
changing `build` instead would invalidate every existing `*.build.lock`.

## ⚡ Call 4 is the one most likely to be got wrong

**Assert that `apply` and `build` AGREE — do not assert that `apply` writes `.jpg`.**

A test pinning the format string pins *the answer*; a test pinning agreement pins *the property*.
The first goes green again the day someone changes the default for a good reason, while the paths
silently diverge. Compare **bytes**, not the extension and not the summary line — that is
`tests/input_svg.rs`'s (SPEC-115) established style in this repo.

## The controls

- **AC-5 — one revert per independent condition.** Reverting the multi-input resolution must **not**
  also disable AC-2's single-input assertion. If one revert turns both red, they are co-dependent
  and the controls prove less than they appear (AGENTS §15, and SPEC-113 shipped a vacuous test
  exactly this way). **Drive each condition separately and record both.**
- **AC-2 needs two source formats.** Drive a JPEG source *and* a PNG source, or "preserved" is
  indistinguishable from "always PNG" — the exact bug you are fixing.
- **AC-1 needs two target formats**, so it cannot pass by coincidence of the source.
- **AC-6 is the blast-radius control:** `resize`, `thumbnail`, `watermark` and `build` must be
  byte-identical to `main` on the corpus. **This spec moves `apply` only.**

## Guardrails

- **Own git worktree:**

  ```
  git -C ~/PSeven/experiments/crustimg_redo_plus/crustyimg worktree add \
    ../crustyimg-spec126 -b fix/spec-126-apply-and-build-agree main
  ```

  `git branch --show-current` before any commit.

- **Your DEC is DEC-098. The ID is reserved — do not run `next_id`**, which scans only the working
  tree and has produced collisions here before.
- ⛔ **This is byte-changing and MUST NOT SHIP ALONE.** It batches into PROJ-011's single lockfile
  migration with STAGE-050. **Do not bump the version. Do not cut a release. Do not merge the PR.**
- **⚡ NEVER POLL CI.** `gh pr checks <PR> --watch --interval 30`, backgrounded, then **left alone**
  — do not re-read a running watcher's output. Measured: ~$60 of one build went on polling. Take
  the cost reading **once, after CI settles**; a reading taken when you think you are finished
  under-reports by ~30 %.
- ⚠ **`gh pr checks --watch`'s summary line is not reliable** — it has reported hundreds of
  "pending" on a fully green PR here. Read the **direct** `gh pr checks <PR>` snapshot, at your
  **true head SHA**.
- ⚠ **`cargo test` in an interactive terminal fails `display_sink_refuses_non_tty`** — a known,
  filed, environment-dependent test (it assumes stdout is not a tty). **Redirect stdout**
  (`cargo test > /tmp/t.log 2>&1; echo $?`) and it passes. This is not your bug; do not fix it.
- A piped command reports the **pipe's** exit code — redirect and read `$?`. zsh does **not**
  word-split unquoted parameters; use `while IFS= read -r`. Use `/usr/bin/grep` for any counted
  sweep.
- **Budget ~150 exchanges.** Checkpoint and report past that — four consecutive cycles have blown
  this without the checkpoint firing.
- **Push a WIP as soon as it compiles**, before the matrix.

## When you finish

1. Fill in `## Build Completion`, including the reflection questions and **every file the diff
   touches, built from `git diff --name-only` rather than recall** — the template now asks for this
   because two builds in a row listed one file short.
2. Append a build cost session (`cost-snippet.md`); price at the anchors `.message.model` reports.
3. Write **DEC-098** with `affected_scope` covering every file it governs, and **record Call 1's
   measured table** — it is the decision's evidence.
4. `just advance-cycle SPEC-126 verify`, and **confirm it moved** with `git diff`.
5. Open the PR. **Do not merge it.**
6. **File any finding on the stage's `## Spec Backlog` as `- [ ]`, then run `just backlog` and read
   it back.** `docs/backlog.md` is read by **no command**.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
