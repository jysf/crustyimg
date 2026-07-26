# PR #113 / SPEC-105 classifier review — findings

> **Provenance.** A max-effort multi-agent code review run by the maintainer on
> 2026-07-26 against the **merged** SPEC-105 commit `54ba05e`
> ("fix(analysis): high-entropy images are never graphics"). It produced **15 findings,
> none refuted on verification**, and **posted nothing to PR #113** (confirmed: 0 comments,
> 0 reviews) — so this document is the only record. Recovered verbatim below from the
> review session's handoff file.
>
> **Trust level.** The *measured numbers* in this document came from review sub-agents
> driving the release binary. Per `a-number-from-an-unproven-path-is-not-a-measurement`,
> treat each as a **reproduction recipe to re-run**, not an established measurement.
> A separate session is re-deriving them; its results supersede the numbers here.
>
> **The structural claims below were independently confirmed by reading source**
> (orchestrator, 2026-07-26) and do not need re-checking:
>
> - Classification runs **after** the resize pipeline — `let out_img = pipeline.run(img)?`
>   (`src/cli/optimize.rs:989`) → `Analysis::compute(&out_img)` (`:1013`). `--max` therefore
>   chooses the content class, and `web` downscales to 2048 by default.
> - Rule 6 (`src/analysis/mod.rs:625`) is **unreachable dead code**: it requires
>   `entropy >= PHOTO_ENTROPY` (5.0) but is reached only when `entropy < 4.0`.
>   `PHOTO_ENTROPY` and `PHOTO_FLAT_MAX` occur nowhere else in `src/` or `tests/`
>   (raw grep + positive control) — both are inert knobs. This breaks
>   `guidance/constraints.yaml` id `clippy-fmt-clean` (severity **blocking**) while
>   `cargo clippy -D warnings` stays green, because the constants remain syntactically
>   referenced. That is why it survived both build and verify.
> - `DOC_ENTROPY_MAX` (4.5) exceeds `PHOTO_ENTROPY_STRONG` (4.0), leaving a
>   self-contradictory band `[4.0, 4.5)` resolved only by rule order.
> - The Icon rule still precedes the new rule, so DEC-047's stated "**any** image with
>   entropy >= PHOTO_ENTROPY_STRONG is a Photograph" is false for images <= 128 px.
> - Only one `Profile::Docs` arm exists (`src/analysis/decide.rs:145`), keyed on
>   `MixedSafe` — so `--profile docs` is a silent no-op for any promoted image.
> - `calibration_gap_holds_for_committed_fixtures` (`src/analysis/mod.rs:945`) asserts
>   `graphic_max < T && T <= photo_min` over committed fixtures only: a tautology across
>   that window rather than a check against the documented gap.
>
> **Status.** Not yet framed as work. The classifier regression is engine-level (it hits
> the CLI and the demo equally), it is **launch-gating**, and it is carried forward to the
> project that follows PROJ-008.

---

# Handoff: max-effort review of PR #113 / SPEC-105 (commit 54ba05e)

A max-effort multi-agent code review ran against the **merged** SPEC-105 commit
(`54ba05e`, "fix(analysis): high-entropy images are never graphics"). No code has
changed on `main` since that merge, so every finding below describes the code as it
ships today. 15 findings survived verification; none were refuted.

Your job: decide the framing (new spec under PROJ-008? amend DEC-047? split into a
regression-fix spec plus a test-integrity spec?), then drive it through the normal
cycle. This is **not** a pre-approved fix list — several findings interact, and the
right fix for the top three is probably one design decision, not three patches.

## Provenance and trust level

Measured numbers below came from review sub-agents driving the shipped
`target/release/crustyimg` binary and, in one case, a threshold mutation that was
reverted (`src/`, `tests/` were confirmed clean afterward; the only dirty file is an
unrelated pre-existing edit to `STAGE-033-post-launch-polish-and-repo-housekeeping.md`).
Treat each number as a **reproduction recipe to re-run**, not as an established
measurement — this repo's own lesson `a-number-from-an-unproven-path-is-not-a-measurement`
applies to review output too. Re-derive before you build on any of them.

## The load-bearing finding (read this first)

**Classification runs AFTER the resize pipeline.** `src/cli/optimize.rs:1013` calls
`Analysis::compute(&out_img)` on the post-pipeline image, so `--max` chooses the class.
SPEC-105's entropy rule made that choice decisive, because downscaling averages hard
edges into intermediate luma bins — which is exactly what a luma-entropy gate reads as
"photographic."

DEC-047's calibration table was measured at native size on images small enough never to
be downscaled. The `web` verb and the demo both downscale to 2048 by default. So the
calibration does not describe the path most users hit.

Reported reproductions:
- 3840x2160 code-editor screenshot: entropy 0.79 -> `document` at native; 3.62 ->
  `ui-screenshot` at `--max 2560`; **4.24 -> `photograph` at `--max 2048`**, shipping a
  lossy AVIF of 358,227 B for a 111,095 B lossless source.
- 3000x2250 1-bit halftone: 0.56 native -> **4.79 at `--max 2048`** -> lossy AVIF.
- The committed `tests/fixtures/classify/dithered_graphic.png` (the PR's single
  adversarial guard, 3.03 at native): **7.08 at `--max 256`** -> `photograph` -> lossy.

This falsifies the amendment's central safety claim — "only a hard-edged graphic forced
lossy is harmful, and none of those reach 4.0". Hard-edged graphics do reach 4.0, via the
default path, and the result is both perceptually harmful and *larger than the source*.

Design question for the spec: does classification belong before the resize (on the source
image), or does the entropy threshold need to be scale-aware? Note this is the same root
cause family as the already-queued follow-up "scale-normalize the flat/edge detector" —
consider whether one deeper fix subsumes both rather than layering a second correction.

## Findings

Severity order. Files are repo-relative.

### Correctness

1. **`src/cli/optimize.rs:1013`** — classification runs post-resize; `--max` decides the
   class. See above.

2. **`src/analysis/mod.rs:604`** — the unconditional early return preempts rule 5, so a
   screenshot or scan with substantial photographic content can no longer reach
   `UiScreenshot` / `OptBucket::MixedSafe`, the bucket built for mixed text+photo content.
   Reproduction: the repo's own `color_photo_fuji.png` composited into flat UI chrome plus
   a text bar at 1600x1000 — `document` -> lossless WebP at 25%/33% photo area (entropy
   3.35/3.92), but at **50% photo area entropy 4.93 -> `photograph` -> lossy AVIF q85**,
   smearing the text and widget borders in the other half.

3. **`src/analysis/decide.rs:145`** — `--profile docs` is now a silent no-op for every
   promoted image. The downgrade arm is keyed solely on
   `(Profile::Docs, OptBucket::MixedSafe) => OptBucket::LosslessFlat`; there is no
   `(Profile::Docs, OptBucket::Lossy)` arm. There are **no `--profile docs` tests in
   `tests/cli.rs` at all**; the only coverage is `docs_profile_makes_mixed_lossless`
   (`decide.rs:923`), which passes `MixedSafe` directly and cannot see this.

4. **`src/analysis/mod.rs:625`** — rule 6 is unreachable dead code. It requires
   `entropy >= PHOTO_ENTROPY (5.0)`, but line 625 is only reached when `entropy < 4.0`.
   Consequences: `PHOTO_ENTROPY` (`:85`) and `PHOTO_FLAT_MAX` (`:99`) appear nowhere else
   in `src/` or `tests/` and are now inert knobs; the `flat_ratio < PHOTO_FLAT_MAX` guard —
   the only thing keeping a *flat* high-entropy image out of the photo path — left the
   cascade with no replacement. No `dead_code` warning fires (the constants are still
   syntactically referenced), so `cargo clippy -D warnings` stays green.
   This breaks `guidance/constraints.yaml` id `clippy-fmt-clean` (severity **blocking**,
   paths `src/**`, `tests/**`): "No dead code; delete rather than comment out" — restated
   at `AGENTS.md:338`. Worth noting the constraint's automated gate *cannot* catch this
   class, which is why it survived both build and verify.

5. **`src/analysis/mod.rs:590`** — `DOC_ENTROPY_MAX` (4.5) exceeds `PHOTO_ENTROPY_STRONG`
   (4.0), so the cascade holds a self-contradictory band `[4.0, 4.5)` resolved only by rule
   order. A grayscale or <=256-colour scan at entropy 4.2 that misses rule 3's conjunction
   (e.g. halftone photos drag `bimodality` to ~0.30, under `DOC_BIMODALITY` 0.55) used to
   hit the palette gate -> `GraphicLogo` -> `LosslessFlat`; it now returns `Photograph` ->
   `Lossy`, so glyphs get lossy-encoded. The ordering dependency is load-bearing and
   undocumented. The only document test (`mod.rs:993`) is a synthetic two-luma-level bar
   pattern far below 4.0 and cannot detect the band.

6. **`src/analysis/mod.rs:576`** — the Icon rule still precedes the new rule, so DEC-047's
   stated rule ("**any** image with luma entropy >= `PHOTO_ENTROPY_STRONG` is a
   `Photograph`") is false for `width.max(height) <= 128`, aspect <= 2.0, no EXIF. A
   128x128 EXIF-stripped B&W photo thumbnail — the exact content the Leica fixture is
   cropped from — still classifies `Icon` -> `LosslessFlat` and ships the oversized
   lossless WebP the spec claims to have eliminated. No test covers a sub-129px photo:
   every SPEC-105 fixture is 400x300 or larger, and the only sub-129px classify test
   (`tiny_square_is_icon`, `mod.rs:983`) is a 48x48 solid fill.
   Either fix the ordering or correct the claim in DEC-047 — right now the doc overstates
   the rule's reach (`a-guards-advertised-reach-is-a-claim`).

7. **`src/analysis/decide.rs:150`** — the `OptBucket::Lossy` shortlist carries no lossless
   candidate for non-alpha input and only PNG for alpha, so promoted images lose their
   measured-bytes lossless fallback. On a lean build (`--no-default-features`) a promoted
   photograph *with alpha* gets a shortlist of exactly `[lossless(Png)]`; if the pipeline
   was altered, `optimize.rs:1053` reaches `fast_fallback_lossy_entry`, which returns
   `None` for `has_alpha == true` unless the source is AVIF/WebP with that encoder built —
   not the case on lean — so a PNG re-encode ships even when it is a blow-up, the exact
   hazard the comment at `optimize.rs:1043` names. Non-alpha lean is fine (JPEG is pushed
   unconditionally). Minor: the `if out.is_empty()` guard at `decide.rs:195` is itself
   unreachable, since every arm pushes at least one entry.

8. **`src/analysis/mod.rs:248`** — luma entropy ignores alpha, so RGB under fully
   transparent pixels feeds the histogram. Pre-existing, but the new rule made it
   class-deciding: a flat logo with dirty transparent-background RGB measures 6.25 ->
   `photograph`; the same file at `--max 500` measures 1.04 -> `graphic-logo`, because the
   resize zeroes transparent RGB. Same asset, opposite buckets, depending on whether a
   resize ran. Verdict was PLAUSIBLE — the mechanism is verified, but confirming real-world
   incidence needs a logo exported with dirty alpha (common from Photoshop/Sketch).

9. **`src/analysis/mod.rs:1014`** — `iso_luma` does not hold the invariant its comment
   promises. `(l + 2*j).clamp(0,255)` saturates red for the light panels, and that artifact
   supplies most of the fixture's entropy. `wide_flat_manycolour_with_edges_is_ui_screenshot`
   calls `iso_luma(200, j)` / `iso_luma(210, j)` with j in [-45, 44]; red saturates at
   j >= 28 (resp. 23), and at j = 44 red is 33 low, dropping luma ~10 levels. Reproducing
   the fixture gives **25 occupied luma bins, not the ~5 the four flat panels intend, and
   entropy 3.3964** — only 0.60 under the threshold it asserts, and equal to DEC-047's
   highest calibrated graphic. Any change to the luma weights, panel levels, or j range
   pushes it over 4.0, and the test then fails pointing at the classifier rather than at a
   fixture artifact, actively misled by the comment at `mod.rs:1009`.
   `ambiguous_square_falls_back_to_photograph_low_confidence` (`mod.rs:1060`) has the same
   clamping plus a second wrong comment: measured `flat_ratio` is 0.611, **above**
   `FLAT_GRAPHIC_RATIO` 0.60, so "frequent steps -> not flat-graphic" is false — only
   `edge_ratio` keeps that gate shut.

### Test integrity

These are the ones worth reading against the repo's own lessons — the pattern that
recurs is a guard with the *shape* of evidence that cannot fail.

10. **`src/analysis/mod.rs:945`** — `calibration_gap_holds_for_committed_fixtures` is a
    tautology. It ranges over fixtures measuring 6.07 / 6.83 / 6.37 against a single
    graphic at 3.03, so it holds for **any threshold in (3.03, 6.07]** — a 3-bit window,
    not the documented (3.43, 4.58] gap. **Mutation-verified: setting
    `PHOTO_ENTROPY_STRONG = 5.5` leaves the suite green**, even though 5.5 drops every real
    photo between 4.58 and 5.5 back onto the lossless path, reinstating the headline bug.
    The specimens that define the measured gap (the 4.58-floor photo, the 3.43 16-colour
    dither) were never committed. The assertion is also the literal conjunction of four
    `assert!` lines already in the three preceding tests, so it adds a fourth fixture decode
    and a fourth edit site for zero new coverage.
    Cure per `a-plausible-test-result-is-not-a-checked-one`: commit the boundary specimens,
    and add a negative control proving the harness *can* go red on a threshold move.

11. **`src/analysis/mod.rs:936`** — `dithered_graphic_stays_graphic_not_photograph`, the
    PR's single adversarial guard, only runs the fixture at native size — the one size at
    which the resolution-dependence cannot bite. At `--max 256` the same file measures 7.08
    and classifies `photograph` -> lossy AVIF: the guard passes while the defect it names is
    reachable through the default path.

12. **`tests/cli.rs:4392`** — the never-bigger ICC assertion was diluted from
    `assert_eq!(guess_format, Some(Jpeg))` to `matches!(fmt, Some(Avif) | Some(Jpeg) |
    Some(WebP))`, and the diff's own comment concedes `guess_format` cannot tell lossy from
    lossless WebP — so the format check can no longer fail for the output it names. The only
    remaining guard is `bytes.len() < blowup`, where `blowup` is a lossless WebP the test
    encodes itself with the `image` crate at default effort — self-referential, and
    satisfiable by a shipped lossless WebP encoded at higher effort. The stronger form is
    already used 200 lines away: drive `--explain=json` and assert `"disposition":"lossy"`,
    as `optimize_grayscale_photo_is_photograph_lossy_avif` (`cli.rs:4637`) does.

13. **`tests/cli.rs:4381`** — the SPEC-084 metadata-forced lossy-fallback branch
    (`src/cli/optimize.rs:1059`) lost its only end-to-end coverage: the renamed test's source
    now classifies `Photograph` -> `Lossy` and never enters it. Delete or mis-condition that
    call site and the suite stays green while a real graphic JPEG with an ICC profile ships a
    several-fold lossless blow-up. The comment justifying the change — "the SPEC-084 blow-up
    scenario is only reachable via the misclassification this spec removes" — is false: the
    PR's own `checker_graphic.jpg` (entropy 2.78) plus an ICC profile still reaches it.

14. **`tests/cli.rs:5023`** — `web_normal_case_no_larger_flag` was swapped to
    `common::jpeg_with_exif(3000, 2000)`, whose EXIF makes rule 2 return before rule 3.5 runs
    (the added comment says so: "independent of SPEC-105"). The `web` verb now has **zero**
    coverage of the no-EXIF classification path — the path the demo and RAW-preview
    extraction actually take, since both strip EXIF. Two invariants went at once: the 3000px
    source also forces a downscale to 2048, so "web shrinks" is guaranteed by resampling
    rather than by the encoder decision the test was written to pin. The comment also
    misdescribes the old helper as "high-frequency", while `detailed_rgb`
    (`tests/common/mod.rs:248`) documents itself as "a smooth gradient plus a mild 8px checker
    texture" — one of the two comments is wrong.

15. **`tests/audit_bench.rs:171`** — `json_shape_consistent_across_verbs` was put behind
    `#[cfg(feature = "avif")]` rather than fixed, and its own added comment admits the
    versioned `crustyimg.optimize.explain/v1` shape genuinely forks between
    `optimize --verify` and `web` on no-AVIF builds. A consumer on `--no-default-features`
    gets a top-level `ssim` key from one verb and not the other under the same schema version
    string. The lean leg is CI's only no-AVIF leg, so further forks there ship undetected. A
    second `#[cfg(feature = "avif")]` was added to `top_level_keys` (`:43`) purely to keep
    `-D warnings` quiet — the gate was a silencer, not a fix.

## Suggested shape (yours to overrule)

The 15 findings are not 15 independent fixes. They cluster:

- **Cluster A — scale (1, 5 partly, 8, 11).** Where classification happens relative to
  resize, and whether entropy is scale-aware. Probably one design decision, and it likely
  subsumes the queued "scale-normalize the flat/edge detector" follow-up.
- **Cluster B — cascade placement (2, 3, 4, 5, 6).** The unconditional early return was the
  broadest possible mechanism. The narrower alternative the review surfaced: gate rule 4's
  two mis-firing clauses on `entropy < PHOTO_ENTROPY_STRONG` instead. That fixes the same
  bug at the same depth, keeps rules 5 and 6 reachable, keeps `PHOTO_ENTROPY` live, and
  localizes the mask so it can be deleted verbatim once the detector is fixed. Worth
  evaluating seriously before patching rules 5/6/docs-profile individually.
- **Cluster C — evidence (10, 11, 12, 13, 14, 15).** The calibration guard, the never-bigger
  assertion, and the schema test all went green by construction. Consider a single pass that
  re-establishes each with a negative control, and commit the boundary specimens (4.58-floor
  photo, 3.43 dither) that DEC-047 cites but the repo does not contain.
- **Docs (6, and the DEC-047 safety claim in finding 1).** Two falsifiable claims in
  DEC-047 are false as written. Correct them in the same change that fixes the behavior, or
  explicitly narrow them.

## Ground rules for whoever builds this

- Re-verify every measured number above before designing against it.
- A **clean full-matrix** build for verify — default / lean / webp-lossy, clippy each, plus
  fmt. SPEC-105's own verify reported green off stale incremental artifacts and CI caught
  real no-AVIF-leg breakage; that cost about a day.
- The orchestrator re-runs the matrix rather than relaying a sub-agent's "CLEAN".
- `rtk` rewrites command output and can corrupt grep counts — cross-check any sweep count
  with raw `grep` plus a positive control, and reach for `rtk proxy` when you need real
  stdout.
