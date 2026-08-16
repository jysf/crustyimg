---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-120
  type: task                       # a measurement spike; ships no behaviour
  cycle: design
  blocked: false
  priority: high                   # it GATES the linear-light fix
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-046
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5       # NOT Sonnet. This spec's whole output is a
                                   # judgment about whether an instrument can
                                   # see an effect — the one thing a measured
                                   # spec cannot delegate to sweep-thoroughness.
  created_at: 2026-08-15

references:
  decisions:
    - DEC-019
    - DEC-074
  constraints:
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs:
    - SPEC-088

value_link: >
  STAGE-046's falsification gate. The linear-light entry set its own premise
  test and this spec runs it, so the repo either fixes a measured defect or
  closes an unmeasured one — instead of shipping a plausible improvement.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Found that SSIMULACRA2
        cannot score a downscale against its source (equal dimensions,
        `report.rs:329`), which reshapes the whole experiment; settled the
        reference-generation and instrument-validation calls that follow from it.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-120: measure the linear-light premise before fixing it

## Context

`docs/backlog.md`'s linear-light entry ends with its own falsification gate:

> **Open question that decides whether the premise holds:** does SSIMULACRA2
> (DEC-019) score the linear-light output *better* than the current output on a
> representative downscale? If it does not, the premise is wrong and this should
> be closed rather than specced. **Measure that first.**

This spec is that measurement. It is the gate on STAGE-046's last item.

**It ships no behaviour.** Its deliverable is a number, a verdict, and a
recorded decision. Any resampling code it writes is a throwaway prototype and
must not be merged as production behaviour.

### The measured facts it starts from

`Resize::apply` converts to RGBA8 and hands `PixelType::U8x4` to
`fast_image_resize` with Lanczos3 (`src/operation/mod.rs:395-527`). No gamma
handling, and — confirmed by repo-wide grep at design — no premultiplied alpha
anywhere on the resize path (the only premultiply code is
`src/image/avif.rs:517`, the MIAF `prem` flag on AVIF decode).

The claimed visible effect is specific: **high-contrast edges darken on
downscale, worst on thin bright features against dark backgrounds.**

## The design calls — settled here, not deferred to build

### Call 1 — SSIMULACRA2 CANNOT score a downscale against its source. The experiment must supply a reference.

`src/cli/report.rs:329`: *"The two images MUST have equal dimensions
(SSIMULACRA2 requires it)."* So the obvious experiment — score the 512px output
against the 2048px source — **is not runnable**, and anyone who tries it will get
an error, not a result.

**The shape that works:** produce a **reference downscale** at the target
dimensions from an independent high-precision implementation, then score *both*
candidates against that reference at equal dimensions:

```
source ──┬─► crustyimg today (sRGB U8x4 Lanczos3)  ─┐
         ├─► prototype (linear-light f32)          ─┼─► SSIMULACRA2 vs reference
         └─► REFERENCE (independent, f32 linear)   ─┘
```

### Call 2 — The reference must NOT come from this codebase.

Generate it with an independent implementation — numpy/Pillow with explicit
linearization, or ImageMagick with an explicit colorspace. **Not**
`fast_image_resize` with different flags, and **not** crustyimg.

A reference produced by the code under test cannot fail the code under test
[[fixtures-from-the-code-under-test-cannot-fail]]. State which tool and version
generated it, and commit the generator script — not necessarily the images.

> The "no ImageMagick" rule in AGENTS §12 governs **test fixtures**, which must
> be generated natively so the suite is hermetic. This is a one-off measurement
> harness, not a fixture, and its whole validity depends on independence. Using
> an outside tool here is the *right* call, not an exception being smuggled.

### Call 3 — Prove the instrument can see the effect BEFORE trusting a null result. This is the load-bearing call.

**SSIMULACRA2 may simply not be sensitive to this.** It is a perceptual metric
tuned for compression artifacts, and it consumes 8-bit sRGB
(`src/quality/mod.rs:68`). Whether it registers a systematic luminance shift from
gamma-incorrect resampling is **itself an open question** — and if it does not, a
null result means *"wrong instrument,"* not *"premise false."*

Those two conclusions lead to opposite decisions, so the experiment must
distinguish them:

- **Positive control (required).** Construct an extreme case where the darkening
  is large and directly measurable — thin bright lines on black, downscaled hard.
  Confirm the physical error is big, then confirm **SSIMULACRA2 registers it**.
- **If the scorer cannot see even the extreme case**, the verdict is
  **"SSIMULACRA2 is the wrong gate for this question"** — not "the premise is
  wrong." Say so, and propose the instrument that would settle it.

A gate you never proved could fire is not a gate
[[a-control-you-never-verified-applied-is-not-a-control]].

### Call 4 — Measure the physical quantity too, not only the perceptual score.

The premise names a concrete effect, so measure it concretely: **mean luminance
error against the reference**, per case, alongside the SSIMULACRA2 delta. This is
independent of the scorer's sensitivity and cannot return an uninterpretable
null.

**If the two disagree — physical error large, perceptual score flat — that
disagreement IS the finding**, and it is a more useful result than either number
alone. Report both regardless of which way they point.

### Call 5 — The alpha half gets its own, simpler measurement.

Premultiplied alpha is a different premise (halos around transparent edges) with
a different correct oracle. Do not route it through SSIMULACRA2. Downscale an
image with hard transparent edges, compare against a premultiplied reference, and
report the **maximum colour error in pixels adjacent to the alpha edge**. One
number, stated, so the fix spec is not carrying an unmeasured half.

## Inputs

- **Corpus:** `bench/corpus/` (SPEC-088, DEC-074) — license-clean and committed.
  `graphic_large.png` is the closest existing case to the premise's worst case;
  `photo_forest_cc0.jpg` is the representative photo. **Add one synthetic
  worst-case** (thin bright features on dark) for Call 3's positive control.
- **Files to read:** `src/operation/mod.rs:395-527` (`Resize::apply`);
  `src/quality/mod.rs:25-100` (the scorer and its 8-bit sRGB input);
  `src/cli/report.rs:329` (the equal-dimensions rule); `bench/corpus/README.md`.
- **DEC-019** (why SSIMULACRA2 is the repo's perceptual oracle) and **DEC-074**
  (the bench harness contract).

## Outputs

- **A written result** appended to `docs/backlog.md`'s linear-light entry: the
  numbers, the verdict, and the instrument-validity finding from Call 3.
- **A DEC** recording the outcome **either way** — proceed or close. A closed
  premise is a decision worth the same record as an accepted one.
- **The harness**, committed under `scripts/` or `bench/`, so the number can be
  re-derived rather than trusted [[a-number-from-an-unproven-path-is-not-a-measurement]].
- **No change to `src/`** beyond a clearly-marked throwaway prototype that is
  **not merged**.

## Acceptance Criteria

- [ ] **AC-1.** The reference downscale is generated by an **independent**
      implementation, with the tool and version stated, and its generator
      committed.
- [ ] **AC-2.** **The positive control fires.** The extreme case shows a large,
      directly-measured luminance error, and whether SSIMULACRA2 registers it is
      **explicitly reported** — this determines how a null on the realistic cases
      is read.
- [ ] **AC-3.** Both metrics reported per case: SSIMULACRA2 delta **and** mean
      luminance error vs the reference. At minimum: the synthetic worst case,
      `graphic_large.png`, and `photo_forest_cc0.jpg`.
- [ ] **AC-4.** **A verdict, stated plainly**, as exactly one of: *premise holds,
      spec the fix* / *premise does not hold, close it* / **_SSIMULACRA2 cannot
      settle this; here is the instrument that can_**. The third is a legitimate
      outcome, not a failure to finish.
- [ ] **AC-5.** The alpha measurement (Call 5) reported as one number with its
      method.
- [ ] **AC-6.** **The result is reproducible from the committed harness** — re-run
      it and confirm the numbers land in the same place. A measurement nobody can
      re-derive is an anecdote.
- [ ] **AC-7.** **No production behaviour change.** `git diff` against `main`
      shows no functional change to `src/`; the prototype is excluded or reverted.
      Confirm the shipped test suite is untouched and still green.
- [ ] **AC-8.** A DEC recording the outcome, with `affected_scope` covering
      `src/operation/**` when the verdict is *proceed*, or `[]` when it is
      *close*.

## Failing Tests

**This spec has none, and that is correct.** It is a measurement, not a
behaviour change: there is nothing to assert about the shipped binary, and
`test-before-implementation` does not apply.

**AC-2's positive control is what makes the result trustworthy**, and it is the
load-bearing criterion here — the same role a failing test plays elsewhere. A
reviewer should treat it that way. **Do not manufacture tests to satisfy a
convention this spec is outside of.**

The committed harness (AC-6) is the reproducibility guarantee, not the test suite.

## Implementation Context

### Decisions that apply
- **DEC-019** — SSIMULACRA2 as the perceptual oracle. **This spec is entitled to
  question whether it is the right oracle *for this question*** — that is Call 3,
  and finding against it is a legitimate outcome, not decision drift.
- **DEC-074** — the bench harness contract; follow its shape if you extend it.

### Constraints that apply
- `clippy-fmt-clean`, `one-spec-per-pr`.
- `test-before-implementation` — **does not apply**, see Failing Tests.

### Prior related work
- **SPEC-088** — built `bench/corpus` and `scripts/bench.py`. Read it before
  writing a new harness; extending is better than duplicating.

### Out of scope
- **Fixing anything.** If the premise holds, the fix is the next spec.
- The colour-type / bit-depth defect (a separate STAGE-046 item, though it shares
  `Resize::apply`).
- Choosing the production resampling implementation.

## Notes for the Implementer

- **A null result is only meaningful if AC-2 passed.** That is the whole design
  of this spec — do not report "no difference" without first establishing the
  instrument can see a difference that certainly exists.
- **You are allowed to conclude the gate was the wrong gate.** Say so clearly if
  so; it is more useful than a forced verdict.
- **Report the disagreement if the two metrics disagree** — do not reconcile them
  into a tidier story than the data supports.
- **Budget: this is an S.** If it is running past ~2 hours, stop and report what
  you have with what remains unmeasured.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). **Own git worktree.** **Do
  not merge the PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md`, including
  `just advance-cycle SPEC-120 verify` — and confirm the `cycle:` line moved.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
