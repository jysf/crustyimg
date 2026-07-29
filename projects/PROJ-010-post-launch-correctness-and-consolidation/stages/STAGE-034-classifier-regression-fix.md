---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-034
  status: proposed
  priority: critical
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-26
shipped_at: null

value_contribution:
  advances: >
    Fixes the one known engine regression that makes the flagship `web` verb produce a
    *worse* output on real inputs — the classifier promotes dithered/halftoned graphics
    to `photograph` after the resize pipeline flips their entropy, producing a lossy AVIF
    **18.5×** the input size at the default `--max 2048` (844,492 B out for a 45,527 B
    source, SSIMULACRA2 69.2), and 35× at `--max 2560`. Without this stage, a Show HN demo
    of the default path is actively misleading on a class of inputs the audience is likely
    to try.
  delivers:
    - "The classifier runs before the resize pipeline (or its entropy thresholds are scale-aware), so `--max` cannot flip the content class on any input"
    - "The committed boundary specimens that prove the fix — and the negative controls that prove the guard would catch a regression"
    - "DEC-047's two false claims corrected; its evidence roster complete"
    - "The cascade left internally consistent: rule 6 reachable or deleted, and the DOC_ENTROPY_MAX / PHOTO_ENTROPY_STRONG contradiction band resolved"
  explicitly_does_not:
    - "Redesign the classifier around a new metric or a new architecture — this fixes the known defect, restores the cascade's internal consistency, and shores up the evidence"
    - "Touch the wasm path, the demo, the CLI surface, or any codec — pure pixel-lane engine work"
---

# STAGE-034: Classifier regression fix

## What This Stage Is

The launch-gating engine fix. The classifier runs ***after*** the resize pipeline. The real order is
`pipeline.run` (`src/cli/optimize.rs:989`) → `Analysis::compute` (`:1013`) → `decide::format_shortlist`
(`:1026`) — so the `web` verb's default `--max 2048` downscale can push a dithered/halftoned graphic's
entropy above the `PHOTO_ENTROPY_STRONG` threshold, promoting it to `photograph` and a lossy AVIF
**18.5× larger** than the input.

This stage fixes the classification placement (or makes the threshold scale-aware), restores the
cascade's internal consistency, and then shores up the evidence integrity that the review found
wanting: no committed boundary specimens for DEC-047's cited values, a headline calibration guard
that is a tautology (stays green with the bug-reinstating value 5.5), and two false claims in DEC-047.

Two specs, sequenced: (A) the fix, (B) the evidence cleanup. Spec A is the blocker; Spec B ensures the
fix stays fixed.

**The measured numbers here supersede the review's originals.** Design against
`docs/research/pr113-classifier-review-findings.md` §"Re-derivation (2026-07-25)", not against the
review's headline figures. Two of the review's claims did **not** reproduce at the stated magnitude
and one blends two defects:

- **Screenshots are not the blast radius.** Four substituted screenshots top out at entropy
  1.14 / 0.80 / 2.04 / 2.37 and stay `graphic-logo`/`document`, all lossless. The mechanism is
  confirmed, the magnitude is not. **A screenshot-only fixture corpus would go green against this
  defect** — scope the fixtures to dithered and halftoned sources.
- **Favicons are not affected either.** Sub-129 px input hits the `Icon` rule and stays lossless.
- **"13–35×" blends two defects.** Measured is **18.5×** at `--max 2048` (35× at `--max 2560`); the
  13× figure is SPEC-105's already-fixed oversized-*lossless* misclassification.

## Why Now

This is the highest-priority item between PROJ-008's close and any launch. The defect is live on the default path of the default verb. A user running `crustyimg web <graphic.png>` gets a worse file silently. Every day before the fix is a day the flagship path produces wrong results on real inputs, and a launch would amplify that gap to exactly the audience that matters most.

## Success Criteria

- A dithered/halftoned graphic that currently produces an **18.5×** larger lossy AVIF through the default `web` path instead produces a correct lossless (or smaller lossy) output — verified by driving the committed boundary specimens against a release build. The known-bad case is the 1-bit halftone: 45,527 B in → 844,492 B out, `larger_than_source: true`, SSIMULACRA2 69.2.
- The committed `tests/fixtures/classify/dithered_graphic.png` no longer flips at `--max 256` (currently 3.03 native → **7.08 → `photograph` → lossy AVIF**, SSIMULACRA2 81.8).
- The fix is proved by re-running the review's negative control: `PHOTO_ENTROPY_STRONG` set to 5.5 must FAIL a guard (proving the guard can detect the regression). Today it does not — `cargo test --release --lib analysis` passes **52/52** with that mutation applied.
- Each of the **six named guard sites** has a negative control proving the harness can go red — not "every numeric threshold guard", since three of the six are not numeric-threshold guards.
- **Rule 6 is either reachable or deleted**, and `PHOTO_ENTROPY` / `PHOTO_FLAT_MAX` are either live or gone — no inert knobs left behind.
- **The `[4.0, 4.5)` contradiction band is resolved**, with a test that would catch its return; a halftone scan at entropy 4.2 with `bimodality` ~0.30 does not reach `Photograph`.
- **Rule 5 is reachable**: the mixed UI+photo composite at 50% photo area no longer promotes to lossy AVIF q85.
- **On the lean leg**, a promoted photograph with alpha does not ship a PNG blow-up (`decide.rs:150`).
- `--profile docs` has a decided, tested behaviour for promoted images, and the first `--profile docs` tests in `tests/cli.rs`.
- DEC-047's two false claims are corrected; its evidence roster contains the boundary specimens it cites.
- All gates green on a **clean full matrix** — default / `--no-default-features` / `--features webp-lossy`, clippy `-D warnings` on each, plus `fmt --check`. This is engine code: an incremental build false-greens here, and did, for about a day on SPEC-105. Confirm the log says `Compiling crustyimg`. [[a-stale-incremental-build-is-a-false-green]]

## Scope

### In scope
- Fix the classification placement or make the entropy threshold scale-aware so `--max` cannot flip the content class.
- Commit the **two** boundary specimens DEC-047 cites but the repo does not contain: the **4.58-floor photo** and the **3.43 16-colour dither**. (The review's "5" was the five diluted *guard sites*, conflated with the specimens.)
- Re-establish each diluted guard with a negative control, at the **six named sites** listed under Spec 2 below.
- Correct DEC-047's false claims (the "any image" reach claim, the safety claim about hard-edged graphics).
- Update the classifier's test module with the new guards.
- **The six review findings brought in deliberately** — see "Findings brought into scope" below.

### Explicitly out of scope
- Rearchitecting the classifier around a **new metric** or a different decision structure. Restoring the
  existing cascade's internal consistency **is** in scope; replacing it is not.
- Changes to the CLI surface, wasm build, demo, or any codec.
- Any of the code-health or housekeeping items that belong in later stages.
- **Luma entropy ignoring alpha** (`src/analysis/mod.rs:248`). Verdict is PLAUSIBLE, not confirmed, and
  the re-derivation could **not test** it — the dirty-alpha case was never attempted. It needs a real
  specimen (a logo exported from Photoshop/Sketch with dirty transparent-background RGB) before it is
  spec-able. Recorded in `docs/backlog.md` with the specimen as the first task.

## Findings brought into scope

The review produced 15 findings. The draft of this stage covered three (the placement bug and two
test-integrity items). Seven more are in scope, and the reason each is here is that **the chosen fix
already stands in that code** — they are not additions to the stage's shape:

| Finding | Site | Why it is in scope |
|---|---|---|
| Rule 6 is unreachable dead code | `src/analysis/mod.rs:625` | Breaks `guidance/constraints.yaml` id `clippy-fmt-clean`, severity **blocking** ("No dead code; delete rather than comment out"), restated at `AGENTS.md:338` — and the automated gate **cannot** catch it, because the constants are still syntactically referenced. More load-bearing than the constraint: `PHOTO_ENTROPY` (`:85`) and `PHOTO_FLAT_MAX` (`:99`) are now inert knobs, and the `flat_ratio < PHOTO_FLAT_MAX` guard — the only thing keeping a *flat* high-entropy image off the photo path — left the cascade with no replacement. That is exactly the input class this stage exists to fix. |
| `DOC_ENTROPY_MAX` 4.5 > `PHOTO_ENTROPY_STRONG` 4.0 | `src/analysis/mod.rs:590` | A self-contradictory band `[4.0, 4.5)` resolved only by undocumented rule order. The findings doc names **halftone photos dragging `bimodality` to ~0.30** (under `DOC_BIMODALITY` 0.55) as what misses rule 3 and lands in the band → `Photograph` → lossy glyphs. Fixing placement without resolving this leaves a second route to the same wrong answer. |
| Rule 5 preempted for mixed UI+photo | `src/analysis/mod.rs:604` | Free under the narrow fix — gating rule 4's two mis-firing clauses **keeps rules 5 and 6 reachable** by construction. Carry it as an **acceptance criterion**, not extra scope. Note the re-derivation found the review's "safe" band does **not** exist: 25% / 33% / 50% photo area measured 4.56 / 4.89 / 5.30, all `photograph` → lossy AVIF. |
| `Icon` rule ordering | `src/analysis/mod.rs:576` | The draft already commits to correcting DEC-047's "any image" claim. Decide **ordering-fix vs claim-correction explicitly** rather than defaulting to the doc edit: a 128×128 EXIF-stripped photo thumbnail measures entropy 6.02 and still classifies `Icon` → lossless, and that is a gallery-pipeline output, not a corner case. |
| `OptBucket::Lossy` carries no lossless fallback | `src/analysis/decide.rs:150` | The finding is conditional on "if the pipeline was altered" — **this stage is the thing that alters the pipeline**. On a lean build a promoted photograph *with alpha* gets a shortlist of exactly `[lossless(Png)]` and `fast_fallback_lossy_entry` returns `None`, so a PNG blow-up ships. Treat as a precondition check on our own change, on the lean leg. (Minor, same site: the `if out.is_empty()` guard at `decide.rs:195` is itself unreachable — every arm pushes at least one entry.) |
| `iso_luma` fixture artifact | `src/analysis/mod.rs:1014` | Belongs in **Spec 2** with the other diluted guards — same class exactly. `wide_flat_manycolour_with_edges_is_ui_screenshot` reproduces at **25 occupied luma bins, not the ~5 its four flat panels intend, and entropy 3.3964** — 0.60 under the threshold it asserts — because `(l + 2*j).clamp(0,255)` saturates red, and the comment at `:1009` denies it. Any change to the luma weights, panel levels or `j` range pushes it over 4.0 and the test then fails pointing at the classifier. `ambiguous_square_falls_back_to_photograph_low_confidence` (`:1060`) has the same clamping plus a provably wrong comment: measured `flat_ratio` is **0.611, above** `FLAT_GRAPHIC_RATIO` 0.60, so "frequent steps → not flat-graphic" is false; only `edge_ratio` keeps that gate shut. [[fixtures-from-the-code-under-test-cannot-fail]] |
| `--profile docs` is a silent no-op | `src/analysis/decide.rs:145` | The downgrade arm is keyed solely on `(Profile::Docs, OptBucket::MixedSafe) => OptBucket::LosslessFlat`; there is **no `(Profile::Docs, OptBucket::Lossy)` arm**, so the flag does nothing for every promoted image. There are **no `--profile docs` tests in `tests/cli.rs` at all** — the only coverage (`decide.rs:923`) passes `MixedSafe` directly and cannot see it. A user-visible flag that does nothing is a bad thing to have live at launch. Note this is a **behaviour question** ("what *should* `docs` do to a promoted image?"), not just a missing arm — the spec must decide it, not assume it. |

## Spec Backlog

- [ ] SPEC-108 (**design cycle complete 2026-07-26**) — **Classification placement and scale-aware entropy.** **Decision: classify the source image, not the pipeline output.** The narrow rule-4 gating alternative was evaluated as instructed and **measurably refuted** — see below. Also resolves the `DOC_ENTROPY_MAX` band, rule 6's reachability, `decide.rs:150`'s lossless fallback on the lean leg, and `--profile docs`. 9 acceptance criteria, 6 failing tests + the mutation control written at design.
- [x] SPEC-109 (**shipped 2026-07-27, PR #114 `408b0f9`, 86,491,591 tokens / $60.97**) — **Evidence integrity.** Committed the two boundary specimens (independently seeded, measured outside the crate), tightened the calibration guard from a **(3.03, 6.07]** window to **(3.6414278, 4.5176096]**, repaired four diluted guard sites plus the two `iso_luma` fixtures, and corrected DEC-047's two false claims. **11/11 ACs verified independently.** The gate that mattered: `PHOTO_ENTROPY_STRONG = 5.5` left the analysis suite green at 52/52 before, and now goes **RED**; 3.2 goes red too. Deviations: a 32-colour dither rather than 16 (a 16-level dither of this repo's photos lands at 2.46–2.88, below the 3.03 already committed), and AC-8 un-gated the schema test rather than fixing the fork — root cause is a `has_alpha` disagreement, which is SPEC-108's AC-7.

**Sequencing decision (2026-07-26): SPEC-109 → SPEC-108.** Today `PHOTO_ENTROPY_STRONG = 5.5` leaves `cargo test --release --lib analysis` green at 52/52. Building the fix first would mean proving it with a guard that has never been shown to move. SPEC-109 is fixtures and guards only, so nothing in it can be invalidated by SPEC-108's placement change.

**Measured during SPEC-109's design — the calibration window, with our own numbers:**

| fixture | class | entropy | flat_ratio | edge_ratio | unique_colors |
|---|---|---|---|---|---|
| `grayscale_photo_leica.png` | photograph | 6.07 | 0.83 | 0.00 | 182 |
| `grayscale_photo_canon.png` | photograph | 6.83 | 0.83 | 0.00 | 233 |
| `color_photo_fuji.png` | photograph | 6.37 | 0.76 | 0.00 | 4096 (sat) |
| `dithered_graphic.png` | graphic-logo | 3.03 | 0.49 | 0.28 | 9 |
| `checker_graphic.jpg` | graphic-logo | 2.78 | 0.00 | 1.00 | 8 |

The guard at `mod.rs:945` therefore holds for **any threshold in (3.03, 6.07]** — 3.04 wide against DEC-047's documented gap of (3.43, 4.58], width 1.15. Loose by ~2.6×, which is why it cannot see 5.5.

⚠ **A finding neither the review nor the re-derivation had: all three photo fixtures measure `flat_ratio` 0.76–0.83, ABOVE `FLAT_GRAPHIC_RATIO` (0.60), with `edge_ratio` 0.00, below `GRAPHIC_EDGE_MAX` (0.08).** Rule 4b would classify every one of them as `GraphicLogo` if rule 3.5 did not fire first. **Rule 3.5 is load-bearing — weakening it is not a safe simplification**, which independently confirms SPEC-108's decision to change the classifier's input rather than its cascade.

**Correction to the review's site list:** `tests/cli.rs:4381` and `:4392` are the **same test** — `:4381` is its doc comment, `:4392` its signature. The work is **four distinct test functions plus the two `iso_luma` fixtures**, not five plus two.

**The six guard sites for Spec 2** — the draft said "every numeric threshold guard", but three of these are
not numeric-threshold guards at all, and **none of the sites appeared anywhere in the draft**. As written
it could have shipped having touched only the first one:

1. `src/analysis/mod.rs:945` — the calibration guard (the tautology; holds for any threshold in (3.03, 6.07]).
2. `tests/cli.rs:4392` — the never-bigger ICC assertion (diluted to a `matches!` that cannot fail; the remaining `blowup` check is self-referential).
3. `tests/cli.rs:5023` — the `web` no-EXIF classification path (now zero coverage; the demo and RAW-preview extraction both take it).
4. `tests/cli.rs:4381` — the SPEC-084 lossy-fallback coverage (the branch lost its only end-to-end test).
5. `tests/audit_bench.rs:171` — the `#[cfg(feature = "avif")]`-silenced schema test (the gate was a silencer, not a fix; the lean leg is CI's only no-AVIF leg).
6. `src/analysis/mod.rs:1014` / `:1060` — the two `iso_luma` fixtures and their wrong comments.

**Count:** 1 shipped (SPEC-109) / 0 active / 1 pending (SPEC-108, design complete, build prompt ready)

## Design Notes

- **✅ RESOLVED 2026-07-26 by SPEC-108's design cycle: option (a), placement. Option (b) was measured and refuted.**

  The two candidates were (a) classify *before* the resize pipeline, and (b) the narrower fix — delete rule 3.5's unconditional early return and gate rule 4's two clauses on `entropy < PHOTO_ENTROPY_STRONG`. (b) was the attractive one: same depth, keeps rules 5 and 6 reachable, keeps `PHOTO_ENTROPY` live, deletable verbatim later.

  **Measured against the committed fixture, release build, `web --json`:**

  | `--max` | class | entropy | edge | flat | unique_colors |
  |---|---|---|---|---|---|
  | 4096 / 512 | `graphic-logo` | 3.03 | 0.28 | 0.49 | **9** |
  | **256** | **`photograph`** | **7.08** | 0.05 | 0.27 | **217** |
  | 128 | `icon` | 7.15 | 0.07 | 0.36 | 207 |

  At `--max 256` the fixture has **217 unique colours ≤ `PALETTE_COLORS` 256**, so `few_colors` is TRUE and **`many_colors` is FALSE**. Under (b): rule 3 fails on entropy, rule 4a is gated off by the new `entropy < 4.0` condition, rule 4b fails on `flat_ratio 0.27 < 0.60`, and **rules 5 and 6 are unreachable because both require `many_colors`.** It falls to rule 7 — whose bias is `Photograph`.

  **(b) changes which line returns `Photograph`, not the answer.** And the reason generalises: since rule 7's fallback is `Photograph` by design (DEC-047), *any* fix that merely stops the graphic gates from firing still lands on `Photograph`. (b) is structurally incapable of fixing an input whose correct answer is "graphic" — which is this defect's entire blast radius.

- **⚠ The risk the brief recorded has materialised.** The brief noted that three of the seven brought-in findings were only cheap if (b) won. Under (a), rule 3.5 keeps its early return, so **rules 5 and 6 stay unreachable** and rule 6's dead code must be fixed explicitly rather than falling out. SPEC-108 carries it as AC-6 (reachable or deleted, no inert constants). It did not silently disappear.
- **The trigger is the downscale ratio, not the input size.** Any dithered or halftoned source whose long edge exceeds 2048 by more than ~20% is exposed at the default. Conversely `--max 128` re-routes a promoted image back to `icon` → lossless (the dithered fixture: 7.08/`photograph` at 256, 7.15/`icon` at 128) — the `Icon` ordering bug seen from the other side, masking the entropy rule at exactly the thumbnail sizes a gallery pipeline emits. Fixture coverage must span both sides of that.
- **Negative controls are the key verify gate.** The review found that the headline calibration test stays green with `PHOTO_ENTROPY_STRONG` at 5.5 — the value that reinstates the original bug. Every spec in this stage must prove its guard fails when the constant moves to a regression value. This is [[a-plausible-test-result-is-not-a-checked-one]] mechanized.
- **DEC-047 corrections are part of the fix, not separate documentation.** The DEC made two false claims (the "any image" reach claim, the safety claim about hard-edged graphics being harmless). These are corrected in the same branch as the code fix so the decision record and the implementation stay consistent.
- **The review's re-derived numbers supersede the original numbers** (`docs/research/pr113-classifier-review-findings.md` §"Re-derivation (2026-07-25)"). The spec design should use those as the ground truth for acceptance criteria.

## Dependencies

### Depends on
- The classifier and pipeline code shipping in PROJ-008 (the code this stage fixes).
- The review findings in `docs/research/pr113-classifier-review-findings.md` — the re-derived boundary specimens and negative control designs.

### Enables
- STAGE-035 (hostile input pass) — launch-gating, sequenced next; no code dependency, but logically completes the pre-launch correctness picture.
- The Show HN / r/rust launch — the classifier regression is the one open blocker on the default path.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
