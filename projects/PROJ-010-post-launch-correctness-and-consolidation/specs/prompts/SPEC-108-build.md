# SPEC-108 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; your job is to implement it.
This is an **engine change to the shared content classifier** — the highest-risk change in
PROJ-010 and the launch gate the whole project exists for.

**Preconditions — check both before starting:**

1. **PR #114 (SPEC-109) is merged** and you are on an up-to-date `main`. Your AC-3 depends on
   its two boundary specimens existing.
2. `ls tests/fixtures/classify/` shows **seven** files, including `photo_entropy_floor.png`
   and `dither_32color.png`. If it shows five, stop — #114 has not landed.

**One-line summary of the job:** classification currently runs on the *output* of the resize
pipeline, so `--max` decides an image's content class. Make it run on the source.

## Read in order

1. `/AGENTS.md` — conventions (fixtures, DCO, `just` recipes, cost per DEC-083).
2. `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-108-classification-placement-and-scale-aware-entropy.md`
   — the whole spec. **Do not skip `### What SPEC-109 established that changes this spec's
   picture`** — four things landed after the design and two of them change what you must not do.
3. `/projects/PROJ-010-post-launch-correctness-and-consolidation/stages/STAGE-034-classifier-regression-fix.md`
   — the measured tables and the seven findings brought into scope.
4. `/docs/research/pr113-classifier-review-findings.md` §"Re-derivation (2026-07-25)" — ground
   truth for numbers. **The re-derivation supersedes the review's original figures.**
5. `src/cli/optimize.rs` (the pipeline/analyse/decide sequence), `src/analysis/mod.rs`
   (the cascade + constants), `src/analysis/decide.rs`.
6. `decisions/DEC-047-*.md` — including SPEC-109's corrections and the specimen-vs-ceiling note.

## The design decision is settled — do not re-litigate it

**Classify the source image, not the pipeline output.** The narrower alternative (delete rule
3.5's early return, gate rule 4's clauses on `entropy < PHOTO_ENTROPY_STRONG`) was evaluated at
design and **refuted by measurement**: at `--max 256` the committed fixture has **217 unique
colours ≤ `PALETTE_COLORS` 256**, so `few_colors` is TRUE and `many_colors` FALSE — which makes
rules 5 and 6 unreachable — and it falls to rule 7, whose bias is `Photograph`. It changes which
line returns `Photograph`, not the answer. The full trace is in the spec's Context.

If you find yourself concluding the narrow fix is better, you have probably missed the
`few_colors` step. Re-read the trace before proposing it.

## Before you change anything: reproduce the baseline

```bash
cargo build --release --locked
CB=./target/release/crustyimg
for m in 4096 512 256 128; do
  $CB web tests/fixtures/classify/dithered_graphic.png --max $m --json -o /dev/null
done
```

Expect: **3.03 `graphic-logo`** at 4096 and 512; **7.08 `photograph`** at 256; **7.15 `icon`**
at 128. If your build disagrees, stop and reconcile — do not build on a disagreement.

## What to build

Branch `spec-108-classify-the-source-image` off `main`.

Compute `Analysis` from the **source** image before `pipeline.run` and thread it to the decision
site. Then resolve the cascade contradictions the same change stands in — AC-5 through AC-8 in
the spec. Emit a **new DEC** recording "classify the source, not the pipeline output", including
the measured refutation of the narrow alternative so it is not re-proposed.

Nine acceptance criteria and six failing tests are in the spec. They are the contract.

## Traps — every one of these has already bitten someone on this stage

- **`--max 128` looks like a pass and is not.** It returns `icon` → lossless because the Icon
  rule fires on *size*, before entropy is consulted. A test asserting "lossless at 128" goes
  green on the broken build. **Assert the class.**
- **Moving the analysis earlier means EXIF is present where it previously was not.** The
  pipeline bakes orientation and drops metadata (DEC-017); rule 2 keys on `has_exif` and returns
  `Photograph` immediately. Classifying pre-pipeline changes behaviour for every EXIF-bearing
  input. **This is the most likely place for this spec to surprise you** — measure the fallout,
  do not assume it is inert.
- **Do NOT weaken or reorder rule 3.5.** Confirmed load-bearing from two independent directions:
  the three photo fixtures measure `flat_ratio` 0.76–0.83 and `photo_entropy_floor.png` measures
  **1.00** — all above `FLAT_GRAPHIC_RATIO` 0.60 with `edge_ratio` ≈ 0.00. Rule 4b would claim
  every one of them if 3.5 did not fire first. Its unconditional early return is the only thing
  holding real photographs off the lossless path while the flat detector stays scale-broken.
- **Rule 6's dead code does not fall out of this fix.** Under placement, rule 3.5 keeps its early
  return, so rules 5 and 6 stay unreachable. AC-6 is real work: make rule 6 reachable **or**
  delete it, and leave no inert constants (`PHOTO_ENTROPY`, `PHOTO_FLAT_MAX`). Deleting is an
  acceptable and probably correct outcome — say which you chose and why in the DEC. This breaks
  the **blocking** `clippy-fmt-clean` constraint today, and `-D warnings` cannot see it.
- **Do not treat 4.0 as validated headroom.** The margin above the graphic class is **0.16 bits**
  (Canon-frame dither 3.8396), not the 0.36 the record implied, and the true ceiling is unknown
  and filed in `docs/backlog.md`. Do not move the threshold — but do not lean on it either.
- **Check both `pipeline.run` sites.** `:989` is the one the findings name; there is another at
  `:160`. If it also feeds a classify path, fixing one leaves a second door open. Cite the grep
  when you claim it does not ([[mechanical-sweeps-need-a-mechanical-check]]).
- **SPEC-084 makes no never-bigger-than-source promise on the metadata-forced branch.** Read
  `src/cli/optimize.rs:1043-1059` before assuming otherwise; this cost the SPEC-109 build two red
  tests to learn from the code.
- **Reverting a mutation does not rebuild the binary.** After any threshold experiment,
  `./target/**/crustyimg` stays mutated until something triggers a rebuild — SPEC-109's verify
  hit this and briefly saw *every* photo fixture report `graphic-logo`. Rebuild and confirm
  `Compiling crustyimg` before taking any measurement. A result that dramatic is a build-state
  symptom, not a classifier symptom.

## Verify before handing back

**Clean full matrix from a fresh `CARGO_TARGET_DIR`** — this is shared engine code and an
incremental build false-greens here (it cost this repo about a day on SPEC-105):

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
```

Confirm the log says `Compiling crustyimg` on each leg. Reference totals after #114:
**776 / 796 / 803** passed. Your numbers should exceed these; report them.

**Also re-run SPEC-109's mutation control.** With `PHOTO_ENTROPY_STRONG = 5.5` the analysis
suite must still go RED. If your change makes that guard green again, you have broken the
instrument, and that is a blocking defect regardless of how the fixtures behave.

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times.
- **Never `git reset --hard`.**
- **`rtk` silently corrupts output** — it has dropped the newest commit from `git log`, returned
  "0 matches" against files that plainly match, and mangled `ls`/`cargo`. It is *intermittent*,
  so a clean comparison proves nothing about the next call. Cross-check anything load-bearing
  with `python3` or plain `git`, plus a positive control that must return nonzero.
- **`just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md`** — `git mv` by hand.
- **Work in a git worktree if any other session is open on this repo**, and check `git status`
  before assuming the tree is yours. This was violated once already this project and cost a
  verify session a corrupted 7-minute window.
- **Do not open or merge the PR.** Maintainer's call.

## When you finish

Fill in `## Build Completion` in the spec and the three reflection questions. Report cost per
`projects/_templates/prompts/cost-snippet.md` — **measured from your own transcript, priced by
component per DEC-083** (a flat rate overstates a cache-heavy cycle by ~14×). Close your return
with the `## Cost readout` block. Update the timeline's `build` line.

**Report what you could not do as clearly as what you did.** A stated gap is worth more than a
green tick that quietly skipped something.
