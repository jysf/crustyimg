# SPEC-108 — VERIFY prompt

Cycle: **verify**. You are NOT the builder and NOT the architect. Your job is to try to
**falsify** the claim that SPEC-108 delivered. A verify cycle that agrees with everything has
usually re-read the build's reasoning back to itself.

**You are deliberately not given the build's readout.** Derive your own answers from the branch
and the spec. Being handed the conclusions turns this into grading an answer sheet you have seen.

Branch: `spec-108-classify-the-source-image`, commit `68eeac0`, on top of `main` @ `14eab28`.
Spec: `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-108-classification-placement-and-scale-aware-entropy.md`

**Work in a git worktree.** Another session is open on this repo, and a shared-directory run has
already corrupted one cycle on this project.

## What the change is

`Analysis::compute` now runs on the decoded **source** image in `src/cli/optimize.rs`, before
`pipeline.run` resizes it, so `--max` can no longer change an image's content class. Nine
acceptance criteria in the spec are the contract.

## The two things most likely to be wrong

Spend your effort here first. Everything else is secondary.

### 1. EXIF fallout — predicted at design, absent from the build's account

The pipeline bakes orientation and then **drops the metadata bundle**. So the old post-pipeline
analysis *always* saw `has_exif: false` — `web` was silently never using the EXIF camera prior.
Classifying the source means **rule 2 (`if has_exif { return Photograph, 0.9 }`) now fires before
entropy is consulted, for every EXIF-bearing input.**

That is defensible — DEC-047 calls EXIF the decisive prior. But it is a **behaviour change on a
large input class**, and it creates a plausible new harm in the same family as the bug being fixed:

- A **scanned document** saved as JPEG with camera/scanner EXIF now classifies `Photograph` →
  lossy, where it previously reached the Document or graphic rules by entropy.
- A **graphic or screenshot** that carries EXIF does the same.

**Establish empirically what happens.** Build fixtures both ways — the same image with and without
EXIF — and drive them through `web` on this branch and on `main`. Report the delta as a table. If
the class flips for a non-photograph, that is a finding, whether or not it violates a stated AC.

Also check the reverse: `tests/cli.rs:5023`'s `web` coverage historically used
`jpeg_with_exif(...)`, whose EXIF made rule 2 return before the entropy rules ran. Confirm the
no-EXIF path still has real coverage after this change.

### 2. Three pre-existing tests were modified to accommodate the `has_alpha` fix

The build found `has_alpha` was read from the post-pipeline buffer, which is always internally
RGBA — so a plain JPEG reported `has_alpha: true`. Fixing it to read the source required changing
three tests whose fixtures had depended on the old behaviour.

**That is the exact pattern SPEC-109 existed to police.** For each of the three, decide
independently: does the new assertion express *corrected truth*, or was it adjusted until it
passed? Reconstruct what each test was originally guarding and confirm the guard survives.
A test whose expected value moved to match the implementation has stopped being evidence.

## Then verify the acceptance criteria

All nine, in the spec. Do not take a green test as proof a criterion is met — check that the test
exercises the thing the criterion names.

Specific traps on this spec:

- **`--max 128` looks like a pass and is not.** It returns `icon` → lossless because the Icon rule
  fires on *size* before entropy. A test asserting "lossless at 128" goes green on the broken
  build too. **Assert the class.**
- **AC-6 deleted rule 6.** Confirm `PHOTO_ENTROPY` and `PHOTO_FLAT_MAX` are gone or genuinely
  live — no inert constants. `-D warnings` cannot see this class of dead code, which is why it
  survived before ([[a-guards-advertised-reach-is-a-claim]]).
- **AC-5's band test constructs a `ClassifyInput` directly** rather than synthesizing an image,
  justified as impractical pixel-by-pixel. Judge whether that is legitimate (it tests `classify()`,
  the actual unit) or whether it dodges the integration path the criterion cares about.
- **AC-7 is a lean-leg claim.** Verify it on `--no-default-features`, driven, not read.
- **Both `pipeline.run` sites.** `:989` was the named one; there was another at `:160`. Confirm
  mechanically whether it feeds a classify path, and cite the grep
  ([[mechanical-sweeps-need-a-mechanical-check]]).

## The instrument must still work

SPEC-109's mutation control is the guard on this whole stage. With
`PHOTO_ENTROPY_STRONG = 5.5`, `cargo test --release --lib analysis` must go **RED**. Run it
yourself, plus a **must-fail control** (e.g. 7.0) so you know your edit is being compiled —
without it, a "correct" result is indistinguishable from a build that never recompiled.

⚠ **Reverting a mutation does not rebuild the binary.** After reverting, `./target/**/crustyimg`
stays mutated until something triggers a rebuild — this bit SPEC-109's verify, which briefly saw
*every* photo fixture report `graphic-logo`. Rebuild and confirm `Compiling crustyimg` before any
measurement. A result that dramatic is a build-state symptom, not a classifier symptom.

## Full matrix — clean, isolated

Shared engine code. Use a **fresh, per-leg `CARGO_TARGET_DIR`** — a concurrent shared-dir run
corrupted the lean leg during this spec's build:

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
```

Reference totals on `main` before this change: **776 / 796 / 803**. Report yours.

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times.
- **Never `git reset --hard`.**
- **`rtk` silently corrupts output** — it has dropped the newest commit from `git log`, returned
  "0 matches" against matching files, and mangled `ls`/`cargo`. Intermittent, so a clean
  comparison proves nothing about the next call. Cross-check anything load-bearing with `python3`
  or plain `git`, plus a positive control that must return nonzero.
- **Do not open or merge the PR.** Maintainer's call.

## When you finish

Write a readout to `specs/prompts/SPEC-108-readouts.md`: per-AC verdict (verified / not verified /
could-not-test — "could not test" is a legitimate and useful answer), every number you re-derived,
the EXIF delta table, your judgement on each of the three modified tests, and an explicit list of
what you did **not** check. Update the timeline's `verify` line.

Report cost per the block at the end of the build prompt — measured from your own transcript,
priced by component. Note that `cache_read` summed per-message counts the same cached prefix once
per message, so `tokens_total` overstates distinct work; report it anyway, but do not read it as
effort.

**A finding is worth more than a green tick.** If everything genuinely checks out, say so briefly
and spend the remaining effort on what you could not verify.
