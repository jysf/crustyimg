# SPEC-109 — BUILD prompt

Cycle: **build**. You are NOT the architect — the spec is settled and is your primary
context. This is **test-and-fixture work on a shared engine module**: judgment and
calibration, not mechanical edits. Nothing here changes classifier behaviour.

**The one-line summary of your job:** today the classifier's headline guard cannot fail.
Make it able to fail, and prove it.

## Read in order

1. `/AGENTS.md` — conventions (fixtures, DCO, `just` recipes, cycle discipline).
2. `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-109-evidence-integrity-for-the-classifier-calibration.md`
   — the whole spec, especially `## Acceptance Criteria` and `## Notes for the Implementer`.
3. `/projects/PROJ-010-post-launch-correctness-and-consolidation/stages/STAGE-034-classifier-regression-fix.md`
   — the measured fixture table and why rule 3.5 turns out to be load-bearing.
4. `/docs/research/pr113-classifier-review-findings.md` — findings 10–15 are your sites;
   section "Re-derivation (2026-07-25)" is the ground truth for numbers.
5. `src/analysis/mod.rs` (the `classify()` cascade, the threshold constants, the `FX_`
   fixture constants and the test module), `tests/cli.rs`, `tests/audit_bench.rs`.
6. `decisions/DEC-047-classification-thresholds-and-fallback-bias.md` — you amend it.

## Before you change anything: establish the baseline

Run the mutation **first**, and record the result:

```bash
# src/analysis/mod.rs — PHOTO_ENTROPY_STRONG: 4.0 -> 5.5
cargo test --release --lib analysis
```

It should pass **52/52**. If it does not, stop and reconcile before building — the design
measured 52/52 and a disagreement means something moved under us. Revert the mutation and
confirm the tree is clean before starting real work.

This is not ceremony. Without the before-run you cannot claim at verify that you changed
anything ([[a-control-you-never-verified-applied-is-not-a-control]]).

Also reproduce the design's fixture table so you know your build agrees:

```bash
cargo build --release --locked
for f in grayscale_photo_leica grayscale_photo_canon color_photo_fuji dithered_graphic; do
  ./target/release/crustyimg web tests/fixtures/classify/$f.png --max 8192 --json -o /dev/null
done
```

Expect entropy **6.07 / 6.83 / 6.37 / 3.03**.

## What to build

Branch `spec-109-classifier-evidence-integrity` off `main`.

1. **Commit the two boundary specimens** DEC-047 cites and the repo lacks:
   `tests/fixtures/classify/photo_entropy_floor.png` (≈4.58) and
   `tests/fixtures/classify/dither_16color.png` (≈3.43). Register them as `FX_` constants
   beside the existing four. **Seed them independently** — construct from a documented
   recipe, measure what you get, assert *that* value. Do **not** hunt for an input that makes
   the current code print 4.58; that produces a fixture which cannot fail, which is the exact
   defect this spec exists to remove ([[fixtures-from-the-code-under-test-cannot-fail]]).
   Record the recipe next to the fixtures.
2. **Tighten the calibration guard** (`src/analysis/mod.rs:945`) so it asserts the documented
   gap. It currently holds for any threshold in **(3.03, 6.07]** — 3.04 wide against DEC-047's
   documented (3.43, 4.58], width 1.15. State the achieved bounds in the failure message.
3. **Repair four test functions plus two fixtures.** The review called these five sites; two
   of its five are the same test (`tests/cli.rs:4381` is a doc comment, `:4392` the
   signature). The real list is in the spec's AC-4 … AC-9.
4. **Amend DEC-047** — its two false claims and its evidence roster (AC-10).

## Make the Failing Tests pass

Seven, listed in the spec's `## Failing Tests`. Note the deliberate odd one:
`optimize_detailed_icc_source_ships_lossy_disposition` **may pass on day one**. That is
expected and is exactly why it must be paired with the mutation — a test that has only ever
passed is not yet evidence ([[a-plausible-test-result-is-not-a-checked-one]]).

## The gate that decides whether this spec delivered

**AC-3.** With `PHOTO_ENTROPY_STRONG = 5.5`, `cargo test --release --lib analysis` must go
**RED**. Record which test fails and its message. Then check the other side: **3.2 must also
fail something** — a guard that catches only one edge of its window is half a guard.

If the specimens land and 5.5 still leaves the suite green, this spec has **not** delivered,
however many tests were added. Say so rather than shipping.

## Non-negotiables

- **Do not change any threshold value, move classification, or touch the cascade.** That is
  SPEC-108, a separate PR (`one-spec-per-pr`, blocking). If a fix here seems to need a
  behaviour change, stop and report it — do not expand scope.
- **Do not "fix" the `iso_luma` fixture by nudging it under the threshold.** It sits at
  3.3964 against a 4.0 assertion because `(l + 2*j).clamp(0,255)` saturates red, giving 25
  occupied luma bins where the four flat panels intend ~5. Either make the generator produce
  what the comment claims, or make the comment true and assert the measured value.
- **`checker_graphic.jpg` is already committed at entropy 2.78** — AC-6 needs no new fixture,
  only an ICC profile attached to it. The comment claiming that branch is unreachable is
  false; correct it.
- **AC-8 un-gates a test on the lean build**, so the `--no-default-features` leg is not
  optional in your own checking.
- Count the guard sites yourself before claiming completeness. The review's own list
  conflates a doc comment with a signature; treat any handed list as a claim
  ([[mechanical-sweeps-need-a-mechanical-check]]).

## Verify before you hand back

**Clean full matrix** — this is shared engine-module code and an incremental build
false-greens here (it cost this repo about a day on SPEC-105):

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Build into a fresh `CARGO_TARGET_DIR` and **confirm the log actually says
`Compiling crustyimg`** ([[a-stale-incremental-build-is-a-false-green]]).

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times
  for a missing `-s`.
- **Never `git reset --hard`.** A previous session did and silently destroyed uncommitted work.
- **`rtk` silently corrupts output** — it has returned "0 matches" for greps against files
  that plainly match, mangled `ls` and `cargo` output, and reported a 0.36 s build that
  compiled nothing. Use `python3` for scans, cross-check every count with a **positive control
  that must return nonzero**, and `rtk proxy <cmd>` when you need real stdout.
- **`just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md`** (known
  `find_spec` glob bug) — move files by hand with `git mv`.
- **If another session is open on this repo at the same time, work in a git worktree.**
  Check `git status` before assuming the tree is yours.

## When you finish

Fill in `## Build Completion` in the spec (branch, PR, ACs met, deviations, follow-ups) and
the three build-phase reflection questions.

**Measure your own cost and report it back.** Build is a metered cycle and a null is not
acceptable (`just cost-audit` enforces it at ship) — but do not estimate it either. Your
session transcript carries per-message `usage`, so sum it yourself:
`~/.claude/projects/<cwd-slug>/<session-id>.jsonl`, adding `input_tokens`,
`output_tokens`, `cache_creation_input_tokens` and `cache_read_input_tokens` over every
line with `.message.usage`; take duration from the first/last `timestamp`. The session id
is the last path component of your scratchpad directory. Write the measured numbers into
`cost.sessions`, price the components separately rather than with the flat 80/20 rule
(cache reads dominate a long cycle and the shortcut overstates by an order of magnitude),
and **end your return message with the `## Cost readout` block** from
`projects/_templates/prompts/cost-snippet.md` — the orchestrator reads cost from your
return, not by hunting for it.

Then update the `- [ ] **build**` line in
`specs/SPEC-109-evidence-integrity-for-the-classifier-calibration-timeline.md`, including the
before/after mutation results.
