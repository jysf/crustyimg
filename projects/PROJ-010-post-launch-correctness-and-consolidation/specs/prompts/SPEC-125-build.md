# SPEC-125 — BUILD prompt

Cycle: **build**. You are **Sonnet 5** — the spec's `implementer` says so and the dispatch matches;
price at **Sonnet anchors ($3/$15 per MTok)**, from `.message.model` in your own transcript, not
from this sentence.

**One-line summary:** `convert --format webp` on a 16-bit source silently halves it to 8-bit and
prints `ssim 100.0` — a perfect score reported by a metric computed on 8-bit renderings, which
structurally cannot see the loss. SPEC-121 shipped a downgrade warning for JPEG and *lossy* WebP
only; *lossless* WebP (no feature flag, always built) takes the same silent path.

**This is a reporting fix. It changes no output bytes** — AC-5 and AC-6 pin that. That is what makes
it the flexible spec in the release sequence.

## ⛔ Gate: do not cut your branch until PR #184 has merged

SPEC-124 is at verify on `fix/spec-124-pin-the-avif-encoder-tile-count`, and **it edits the same
regions of `src/sink/mod.rs` that you will**: it inserts `AVIF_TILE_THREADS` at `:103`, immediately
below SPEC-121's `eight_bit_downgrade_warning` block (`:82-109`), and rewrites `:687` and `:745`
— the two existing warn call sites. Branching from `main` before #184 lands buys you a conflict in
exactly the code you are changing.

**Check first:** `gh pr view 184 --json state,mergedAt`. If it has not merged, **stop and report**;
do not start and do not work around it.

## Read in order

1. **The spec** — `.../specs/SPEC-125-lossless-webp-never-silently-halves-bit-depth.md`.
   **7 ACs, 3 design calls, 4 failing tests.**
2. **STAGE-042's backlog item** — the full measured evidence, filed by SPEC-121's punch-list cycle.
   **Read it; do not re-derive it.**
3. **DEC-095** (colour type / bit depth preserved — SPEC-121/122's rule, which you must NOT reopen),
   **DEC-090** (the honest-size line), **DEC-019** (the byte-budget search's scorer — your hard
   boundary, see Call 2).
4. `/AGENTS.md` §4, §11 (stderr, never stdout), §12, §13, §15.

## ⚠ The spec's line references have drifted. Grep, do not trust them.

The spec cites *"candidates visible at `src/sink/mod.rs:216-226`"*. On today's `main` that range is
the `SinkInput` struct — the real format list is `format_from_ext`'s match at **`:247-268`**
(`png`, `jpg/jpeg`, `gif`, `bmp`, `tif/tiff`, `ico`, `webp`, `avif`, …). The existing diagnostic is
`eight_bit_downgrade_warning` at **`:103`**, called from **`:687`** (JPEG) and **`:747`** (lossy
WebP). **All of these shift once #184 merges** — locate them by name, not by number.

## ⚡ Call 1 — the 8-bit-only set is a PRIOR to measure, not a list to copy

The spec names `Gif`, `Bmp`, `Ico` and lossless `WebP` as candidates and says **`Tiff` and `Png` are
*believed* 16-bit-capable, so they must NOT warn.** That word is deliberate. It is a prior.

**You cannot settle this by reading `image`'s docs or grepping `src/`**
[[a-grep-of-src-cannot-see-a-dependencys-default]] — a format's real depth ceiling is a property of
the encoder actually linked into *this* build's feature set, and this repo has already been burned
twice by a dependency behaving differently from its documented default (DEC-094: `ravif`'s
`threading` feature swaps in a whole different module, and `cargo tree -e features` cannot see it).

**Drive it behaviourally instead:** encode a >8-bit source to each candidate target, decode the
result back, and read the depth that survived. That measurement *is* AC-2's derivation, and it is
also the mechanical check that keeps the set from going stale
[[mechanical-sweeps-need-a-mechanical-check]] — a table-driven test that encodes-and-reads-back goes
red on its own when a dependency's capability changes, which a hand-maintained match arm never will.

**If Tiff or Png turns out NOT to hold 16 bits — or if a format you expected to warn actually
does hold the depth — that is a finding, not a failure.** Report it prominently; it changes the
warning's reach and it contradicts a written prior.

⚠ Some targets are feature-gated. State which feature set your measured table was taken under, and
whether the answer differs across `default`, `--no-default-features`, and `--features webp-lossy`.

## ⚠ Call 2 — the `ssim 100.0` line, and the boundary you must not cross

This is the half that makes the bug dangerous: the tool's own quality instrument reports the
opposite of the truth. **A bare `100.0` across a depth change is not acceptable.** Three options,
and you pick one **with the reasoning in the DEC**: suppress the figure, qualify it (*"ssim 100.0
(8-bit comparison; source was 16-bit)"*), or compute at source depth if the scorer can.

⛔ **DEC-019 anchors the scorer used by `optimize`'s byte-budget search. If your fix would touch
that path, STOP AND REPORT — do not proceed.** This spec is a reporting fix, not a search change,
and AC-6 pins `optimize`'s candidate selection byte-identical to `main` on the corpus. Perturbing
candidate selection here would be a byte change on a shipped verb, smuggled in under a spec that
promises none.

## Call 3 — do not reopen SPEC-121's narrowing rule

DEC-095 is settled and shipped. You are adding a diagnostic where the *target format* cannot hold
what the pipeline correctly preserved. **No `Operation` body changes.**

## The controls

- **AC-5 is the negative control and it must be driven both ways**: an 8-bit source through the same
  verbs warns **nowhere**, and its output is **byte-identical to `main`**. One revert per
  independent condition (AGENTS §15) — and the evidence for AC-1/AC-2/AC-4 is the **behavioural
  flip**, a test observed going red, not a changed binary hash.
- **AC-3 must be DRIVEN, not reasoned.** `web` / `optimize` reach this through the
  smallest-candidate search, which is how most users hit it. Run the verbs.
- **AC-4 asserts the rendered line**, not an internal value — the defect is what the user reads.

## Guardrails

- **Own git worktree, cut from `main` after #184 merges:**

  ```
  git -C ~/PSeven/experiments/crustimg_redo_plus/crustyimg worktree add \
    ../crustyimg-spec125 -b fix/spec-125-lossless-webp-never-silently-halves-bit-depth main
  ```

  `git branch --show-current` before any commit.

- **Your DEC is DEC-097. The ID is reserved — do not run `next_id`**, which scans only the working
  tree: DEC-096 currently exists *only* on SPEC-124's unmerged branch and is invisible to it.
  SPEC-119 and SPEC-120 both minted DEC-092 this way.
- **⚡ NEVER POLL CI, and do not re-read a backgrounded watcher's output while it runs.**
  `gh pr checks <PR> --watch --interval 30`, then leave it alone. Measured: ~$60 of SPEC-122's
  $103.60 build went on polling; SPEC-123's spent $5.80. Take the cost reading **once, after CI
  settles** — a cycle that reports cost when it thinks it is finished under-reports it by ~30%.
- ⚠ **A green local matrix does not predict CI.** `main` went red **without a commit** when stable
  floated to 1.98 and added the `chunks_exact` lint. Your local matrix runs the toolchain installed;
  CI resolves `stable`. If CI fails for a toolchain reason unrelated to your diff, **split the fix
  to its own PR** and `update-branch` this one.
- ⚠ **`gh pr checks --watch`'s own summary line is not reliable** — on SPEC-124 it reported
  `451 pass / 0 fail / 223 pending` and exited 0 while a direct `gh pr checks <PR>` showed the real
  state. **Read the direct snapshot for the verdict**, and read it at your **true head SHA**:
  SPEC-124's build read CI green, then pushed one more commit and never re-read, so its AC-9 claim
  covers a SHA that is no longer the head.
- **Budget ~150 exchanges.** Checkpoint and report past that — SPEC-124's build ran 503 against 150,
  SPEC-121's 555 against ~250, and neither checkpoint fired.
- **Push a WIP as soon as it compiles**, before the matrix.
- macOS has no `timeout(1)`. `git commit -s`. A piped command reports the **pipe's** exit code —
  redirect and read `$?`. zsh does **not** word-split unquoted parameters (`for f in $files` runs
  once on the whole blob and every per-file check silently passes) — use `while IFS= read -r`, and
  write `"${B}:path"`, never `$B:path`. Both were caught only by a **positive control** whose answer
  was already known.
- **Do not merge the PR. Do not bump the version.**

## When you finish

1. Fill in `## Build Completion` including the three reflection questions, and **list every file the
   diff touches** — SPEC-122's Deviations claimed "`src/operation` and `tests/` only" and was wrong
   by two `scripts/` files, which left `affected_scope` blind.
2. Append a build cost session (`cost-snippet.md`); price at the anchors `.message.model` reports.
3. Write **DEC-097** with `affected_scope` covering every file it governs, and **record Call 1's
   measured capability table in it** — that table is the decision's evidence, and the next spec that
   touches this set should read it rather than re-measure.
4. `just advance-cycle SPEC-125 verify`, and **confirm it moved** with `git diff`.
5. Open the PR. **Do not merge it.**
6. **File any finding on the stage's `## Spec Backlog` as `- [ ]`, then run `just backlog` and read
   it back.** `docs/backlog.md` is read by **no command**. This failed three times last session.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
