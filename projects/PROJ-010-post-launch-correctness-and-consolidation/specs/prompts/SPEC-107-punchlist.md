# SPEC-107 — PUNCH LIST prompt (verify → build)

Cycle: **build** (second pass). Fresh session, own worktree. Verify returned **⚠ PUNCH LIST** on
PR #127 — the code is sound and all 11 acceptance criteria were empirically re-derived, but
**AC-6 is not met as worded** (verify drove the hole), and two record corrections would otherwise
be archived as false.

**Work on the existing build branch `feat/spec-107-hostile-input-pass`** and push to the same PR
#127. This is not a new spec and not a new PR.

There is also an **unmerged verify branch**, `verify/spec-107-cost-and-timeline` (`7b78dda`),
carrying only the verify cost session and timeline line. `src/`, `tests/`, `docs/`, `decisions/`
there are byte-identical to the build head, so it will not conflict with your code changes — but
check before you assume, and do not duplicate its bookkeeping.

## Item 1 — the only unmet acceptance criterion. Do this first.

`tests/hostile_inputs.rs:233-243`. AC-6 requires the debug-profile carve-out be **"narrow enough
that a new panic still fails the suite."** Verify drove both halves:

- A panic that **replaces** the known banner → 2 tests RED. ✅ Half holds.
- A panic-shaped leak that **coexists** with the banner on the same input → suite **8/8 GREEN**. ❌

The hole is that `stderr.contains("bad parser state bytes left")` plus a last-line check never
screens the lines *in between*, and this test alone does not screen `panicked` /
`RUST_BACKTRACE` / `unwrap`.

**Fix:** assert the extra lines are **only** the known banner block — not merely that the output
contains it. Then **drive both mutations again yourself** and record both results. A carve-out
whose additive case you have not personally seen go red is not fixed.
[[test-a-carve-out-additively-not-just-by-replacement]]

## Items 2–3 — correct the record before it is archived

These matter because the archived Build Completion is what the next reader learns from.

**Item 2 — the headline deviation was wrong, and the design matrix was right.** Correct the spec
at **:573-583** and reflection Q1 at **:622-632**. The facts, driven by verify across the full
shape × dimension cross product:

| shape | dims | error |
|---|---|---|
| bare | 100000² | `LimitsExceeded("Image size exceeds limit")` |
| extended | 100000² | same |
| bare | 20000² | `Decode("unexpected end of file")` |
| extended | 20000² | `LimitsExceeded` (`check_pixel_budget`) |

A **second cap nobody named** explains it: at 100000² the upstream per-dimension limit
(`MAX_IMAGE_DIMENSION = 65_535`, DEC-034) fires *inside the header read*, so the bare shape
reaches a real `LimitsExceeded`. `optimize_detailed_rejects_oversize_without_panic` **is not
vacuous.** The IDAT+IEND fix was genuinely necessary — but **only for the 20000² corpus fixture**,
which sits under the dimension cap and therefore depends on the pixel-count budget (DEC-063).

Say what the real residual is, because it is the *opposite* of what the report claimed: **no wasm
test reaches `check_pixel_budget` at all**, though that test's docstring advertises both DEC-034
and DEC-063. File it as a follow-up candidate; do not fix it here (`one-spec-per-pr`).

The timeline's existing wording ("on the native path") is already accurate — leave it.

**Item 3 — the follow-up verb list is wrong in both directions.** Spec **:604-613** and timeline
**:78-81**. Verify drove 16 invocations. `edit` and `watermark` (primary input) **do** warn — both
route through `run_pixel_op` — as does `apply --recipe web`. Actually unwarned: `diff` (both
inputs), `responsive`, `apply` with a plain pixel recipe (single + batch), `build`,
`watermark --image` (overlay only), plus **`lint` and `meta strip`, which the list omits.**

Correct it: drop `edit`/`watermark`, add `lint`/`meta strip`. This list is about to scope a
follow-up spec, so a wrong list sends real work at the wrong files. Verify's sweep scope, stated
as a claim: `/usr/bin/grep -rn "Image::load\|Image::from_bytes" src/` — 36 hits, 14 in `src/cli/`,
each resolved to its enclosing fn; it would miss a load reached through an untraced helper.

## Items 4–6 — quality and doc accuracy

**Item 4 — `tests/wasm_roundtrip.rs`, one-line assertion strengthening.**
`wasm_empty_obu_avif_is_an_error_not_an_abort` gets `CodecUnavailableOnTarget` from the AVIF
sniff — the identical error two pre-existing tests in the same file already pin with stronger
assertions (`msg.contains("AVIF")`, `!msg.contains("--features")`). As written it asserts less
than its own file's convention and cannot distinguish the crafted container from any AVIF.
`wasm_truncated_jpeg_does_not_kill_the_module` would stay green if its generator ever stopped
producing a JPEG. Add message assertions to both.

**Item 5 — doc overclaims.** The corpus `README.md` header and `docs/launch-readiness.md` both say
the corpus "is driven through both the native CLI and headless wasm"; of the 8 files **only
`empty_alpha_obu.avif` is.** Fix both to say what is actually true — this is a launch-gating
document and an overclaim there is the specific failure mode that made the board stale before.
[[documentation-has-no-green]] Also: `docs/cli-reference.md:36` still defines `--quiet` as
"Suppress non-error output", now literally false given DEC-085's unconditional warning; and
`api-contract.md:99` names 5 verbs in a way that reads as exhaustive when **11** warn.

**Item 6 — nits.** `hostile_inputs.rs:148-150` claims a test-harness hang bound; **there is none
configured** (no nextest or libtest timeout) — either configure one or drop the claim, but do not
leave an unbacked guarantee in a comment. The cost note says "98.4% cache reads"; actual is
**98.57%**. (`tokens_total`, `$81.04`, and `agent` vs the pinned `implementer` all reconcile
exactly — leave those.)

## Also worth knowing

**DEC-085's decision stands, but its rationale is inaccurate.** It claims "every other advisory
warning is cosmetic"; verify found two recorded decisions that contradict it — **DEC-023** gates a
downscale warning behind `--quiet` precisely because it is "surprising if unnoticed," and
**DEC-075** gates the larger-than-source note. The convention DEC-085 departs from is *stronger*
than it says, which makes the departure more notable, not less. Correct the rationale; keep the
decision.

**What verify could not check** — do not treat these as cleared: `view` (needs a tty; hung under
`script`), `build` (verify's manifest syntax was wrong; unwarned by inspection only), and
**release-profile stderr cleanliness** (only debug was driven). If any is cheap for you to drive,
do it and say so; otherwise leave them stated as gaps.

## Verify before handing back

Re-run the **clean full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, sequentially, **through
`rtk proxy` from the first leg** — the previous build hit an rtk failure mode that collapsed
`cargo test` output and deleted the `Compiling crustyimg` line, which is the control proving the
build was not incremental. Treat a missing `Compiling` line as a tooling failure first.

Reference after this build: **lean 797 / default 816 / webp-lossy 823**, `just wasm-test` 30/30.
If your Item 1 and Item 4 changes alter those counts, reconcile the delta explicitly.

## Guardrails

Own worktree; the build and verify worktrees are both still checked out. `git commit -s`. Never
`git reset --hard`. Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a
positive control. macOS has no `timeout(1)`. **Do not merge the PR.**

## When you finish

Append a **second build session** entry to `cost.sessions` (do not overwrite the first). Update
`## Build Completion` with what changed on this pass, and the timeline's build line. Close with
the `## Cost readout` block, verbatim, as the last thing you emit.
