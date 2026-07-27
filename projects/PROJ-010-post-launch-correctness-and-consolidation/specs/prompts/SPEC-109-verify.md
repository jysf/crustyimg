# SPEC-109 — VERIFY prompt

Cycle: **verify**. You are NOT the builder and NOT the architect. Your job is to try to
**falsify** the claim that SPEC-109 delivered. A verify cycle that agrees with everything has
usually just re-read the build's own reasoning back to itself.

**You are deliberately not being given the build session's readout.** Do not go looking for
it. Derive your own answers from the branch and the spec, then compare at the end if you
want. Being handed the conclusions turns this into grading an answer sheet you have already
seen.

Branch: `spec-109-classifier-evidence-integrity` (two commits on top of `main`).
Spec: `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-109-evidence-integrity-for-the-classifier-calibration.md`

## The one thing that decides this cycle

SPEC-109's entire purpose is that **the classifier's calibration guard can now fail.**
Everything else is supporting work. So:

```bash
# src/analysis/mod.rs — PHOTO_ENTROPY_STRONG
# run each, record which tests fail and their messages, then revert
4.0  -> expect GREEN   (the shipped value)
5.5  -> expect RED     (reinstates the SPEC-105 bug)
3.2  -> expect RED     (the other edge of the window)
```

Run all three yourself. **Also run a control that proves your edit is being picked up** — a
value that must fail loudly, e.g. `7.0`. Without it, "green at 4.0" is indistinguishable from
a build that never recompiled ([[a-control-you-never-verified-applied-is-not-a-control]]).

If 5.5 does not go red, the spec has not delivered regardless of how many tests were added.

## Attack these specifically

Each of these is a place where the work could *look* done and not be.

1. **Are the specimens genuinely independent?** `scripts/seed-classify-specimens.py` claims it
   never calls crustyimg and measures entropy with its own implementation. **Verify that by
   reading it**, not by trusting the comment. A fixture whose asserted value came from the code
   under test cannot fail ([[fixtures-from-the-code-under-test-cannot-fail]]). Check its
   positive control actually reproduces the four pre-existing fixtures.
2. **Does the width cap do what it claims?** The guard reportedly fails if a specimen is
   dropped. **Drive it**: remove or rename a specimen and confirm RED. A guard's advertised
   reach is a claim ([[a-guards-advertised-reach-is-a-claim]]).
3. **AC-6 — is the SPEC-084 branch actually reached?** A test that exercises nothing reports
   green ([[a-harness-that-exercises-nothing-reports-green]]). Assert the branch at
   `src/cli/optimize.rs:1059` is **hit**, not merely that a test with a promising name passes.
   Instrument it if you have to.
4. **AC-7 — does the no-EXIF test take the no-EXIF path?** Rule 2 returns early on EXIF. If
   the fixture carries EXIF, the test proves nothing about the path it names.
5. **AC-8 — un-gated into a tautology?** The test now runs on every leg. Does it still
   *assert* anything that could fail on the lean leg, or was the gate swapped for a weaker
   assertion? Make it fail on purpose.
6. **AC-9 — which did they fix, the generator or the comment?** Either is allowed. What is not
   allowed is nudging the fixture under the threshold to make the number look right. Confirm
   the asserted bin count is the measured one.
7. **Zero behaviour change.** This spec must not alter classification. Diff the **non-test**
   portions of `src/analysis/mod.rs` and `src/analysis/decide.rs`. If any production path
   changed, that is a scope violation and belongs to SPEC-108.
8. **The 32-colour deviation.** The spec asked for a 16-colour dither; a 32-colour one was
   committed. Check the arithmetic yourself: quantising to L levels costs ≈ `log2(256/L)` bits,
   so 16 levels of a 6.07–6.83-bit source should land ≈2.46–2.88 — below the 3.03 dither
   already committed, which would leave the lower bound unmoved. Confirm or refute. Then check
   `DEC-047` records the substitution rather than silently swapping the specimen it cites.

## Also check

- **DEC-047's corrections are accurate**, not merely present. It now makes measured claims
  (an `Icon`-rule entropy figure, the 7.08-at-`--max`-256 refutation). Re-measure at least one.
- **Every claim in `tests/fixtures/classify/RECIPES.md` that is checkable, check.**
  `python3 scripts/seed-classify-specimens.py --check` should agree with the committed bytes.
- **Scope:** the branch also modifies `projects/_templates/prompts/cost-snippet.md`, a shared
  template. Judge whether that belongs in this PR under `one-spec-per-pr` (blocking), and say
  so plainly either way.

## Full matrix — run it clean, do not trust a cached green

Shared engine-module code. Fresh `CARGO_TARGET_DIR`, and **confirm the log says
`Compiling crustyimg`** on each leg ([[a-stale-incremental-build-is-a-false-green]]):

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
```

### One loose end handed to you deliberately

The orchestrator ran this matrix from an empty target dir and got **776 / 796 passed, 0
failed** on the lean and default legs, with three clippy legs and `fmt --check` all exiting 0.
The **`--features webp-lossy` leg is unresolved**: one run reported 352 passed / 0 failed, a
second reported 0 — almost certainly a log-capture artifact rather than a real failure, but it
was not run down. **Re-run that leg cleanly and report the real number.** Do not assume it is
fine because the other two were.

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times.
- **Never `git reset --hard`.**
- **`rtk` silently corrupts output** — cross-check counts with `python3` plus a positive
  control that must return nonzero; use `rtk proxy <cmd>` for real stdout.
- **Do not open or merge the PR.** That is the maintainer's call.
- If another session is open on this repo, work in a worktree; check `git status` first.

## When you finish

Write a readout to
`/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/prompts/SPEC-109-readouts.md`:
per-AC verdict (verified / not verified / could-not-test — "could not test" is a legitimate
and useful answer), every number you re-derived, anything you had to fix, and an explicit
list of what you did **not** check. Update the timeline's `verify` line.

Report your cost per `cost-snippet.md` — measured from your own transcript, not estimated.

**A finding is worth more than a green tick here.** If everything genuinely checks out, say so
briefly and spend the remaining effort on what you could not verify.
