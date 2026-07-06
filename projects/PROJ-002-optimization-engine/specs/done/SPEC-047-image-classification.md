---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-047
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-002
  stage: STAGE-011
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-05

references:
  decisions: [DEC-002, DEC-034, DEC-047]
  constraints: [untrusted-input-hardening, no-agpl-default-deps, no-new-top-level-deps-without-decision, test-before-implementation]
  related_specs: [SPEC-046]

value_link: >
  Delivers STAGE-011's second half — the deterministic photo-vs-graphic verdict
  that biases the format engine toward the right codec family; the decisive signal
  every STAGE-012 auto-decision reads (`opt_bucket` + `has_alpha` → codec family).

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: >
        Main-loop orchestrator (PROJ-002 framing session), not separately metered.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 85000
      estimated_usd: 0.77
      duration_minutes: 18
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — autonomous overnight run in the orchestrator main loop, NOT a metered
        subagent, so no subagent_tokens. Order-of-magnitude (~85k at Opus 4.8 ~80/20 ≈ $0.77).
        Added ImageClass/OptBucket/confidence + the no-ML cascade + 9 corpus tests to
        src/analysis/mod.rs; one iteration to fix the noise fixture (coord-XOR mixing for genuine
        >256-colour high-frequency noise). Suite 449; fmt/clippy/lean/deny green. PR #54.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 12000
      estimated_usd: 0.11
      duration_minutes: 3
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — same autonomous run; CI-driven verify (all jobs green on #54), decision-drift
        clean, post-merge suite 449. Order-of-magnitude (~12k).
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop ship bookkeeping (also shipped STAGE-011), not separately metered.
  totals:
    tokens_total: 97000
    estimated_usd: 0.88
    session_count: 4
---

# SPEC-047: deterministic image classification — `ImageClass` → three `OptBucket`s

## Context

SPEC-046 builds the `src/analysis/` layer: an immutable `Analysis` context of raw features
(histogram, entropy, edge/flat ratios, capped `unique_colors`, alpha coverage, dominant colour).
Features alone don't decide anything — the format engine needs a **verdict**: is this a photograph
(compress lossy) or a graphic/logo/icon/document (compress lossless)? This spec adds that verdict.

It is deliberately internal and no-ML. Cloudinary's `f_auto`/`q_auto` picks JPEG-vs-PNG by
detecting "photographic vs non-photographic"; we reconstruct the same "camera vs computer" bit
**deterministically from pixels + container facts**. Classification is *not* a product surface —
no `classify` command, no user-facing output beyond an optional one-word `explain` label
(SPEC-049). Its bar is: *cheap enough to always run, right often enough to beat a format-blind
default, never blocking.*

The full design (feature→rule cascade, the six labels, the three-bucket collapse, the
safe-fallback bias, the honest gray zones) is specified in
`docs/research/proj-002-design-classification.md`. The threshold constants this spec introduces —
and the fallback-to-photograph bias — are recorded in **DEC-047**.

## Goal

Extend `Analysis` (SPEC-046) with a deterministic, no-ML classifier that maps the computed
features + the image's container priors (`source_format`, `has_exif`) to an `ImageClass` (with a
`confidence`) and collapses it to an `OptBucket` (`Lossy` | `LosslessFlat` | `MixedSafe`) that the
format engine switches on — computed in the same bounded, no-panic pass, adding no dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-classification.md` — the authoritative cascade, the four
    decisive features, the six→three collapse, the fallback bias, the cited prior art.
  - `docs/research/proj-002-design-format-engine.md` — how the *consumer* uses the verdict
    (`opt_bucket` + `has_alpha` → codec family), so the enum shape serves the engine.
  - `src/analysis/mod.rs` (from SPEC-046) — the `Analysis` struct + features to read; add the
    classification fields here.
  - `src/image/mod.rs` — the container priors the classifier reads off `Image`/`ImageInfo`:
    `source_format` and `has_exif` (and `has_icc` as a weak lean). Confirm the exact accessors
    before build; do not re-parse metadata (it is already captured).
- **Related code paths:** `src/quality/mod.rs` — the `LossyFormat` seam the downstream engine
  (SPEC-048) pairs with `OptBucket`; read only to shape the enum, do not modify.

## Outputs

- **Files created:** none.
- **Files modified:** `src/analysis/mod.rs` — add `ImageClass`, `OptBucket`, the classifier, and
  three fields on `Analysis` (`class: ImageClass`, `opt_bucket: OptBucket`, `confidence: f32`),
  exposed via accessors; extend `Analysis::compute` to run the cascade after the feature pass.
- **New exports:**
  - `pub enum ImageClass { Photograph, GraphicLogo, Icon, Document, UiScreenshot }` — the fine
    label (kept for `explain` cosmetics only).
  - `pub enum OptBucket { Lossy, LosslessFlat, MixedSafe }` — the coarse verdict the engine reads.
  - `pub fn ImageClass::opt_bucket(self) -> OptBucket` — the fixed six→three collapse.
  - Accessors on `Analysis`: `pub fn class(&self) -> ImageClass`, `pub fn opt_bucket(&self)
    -> OptBucket`, `pub fn confidence(&self) -> f32`.
  - A single named consts block (the DEC-047 thresholds) — e.g. `ICON_MAX_EDGE`, `PALETTE_COLORS`
    (reuse SPEC-046's exposed cap anchor), `FLAT_GRAPHIC_RATIO`, `DOC_BIMODALITY`, `DOC_GRAY_RATIO`,
    `PHOTO_ENTROPY`, `PHOTO_FLAT_MAX`, `FALLBACK_CONFIDENCE`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `Analysis::compute` returns a `class`, `opt_bucket`, and `confidence` with **no additional
  pass** over the buffer — the cascade runs on the already-computed features + container priors
  (O(1) after SPEC-046's passes). No re-decode, no disk, no new dependency.
- [ ] Classification **never errors and never panics**: it always yields a class + confidence for
  any input SPEC-046 accepts (including 1-px / degenerate — those resolve to the fallback, not an
  error). It is *advisory*, so — unlike `compute` itself — it does not add new `AnalysisError`
  variants.
- [ ] The cascade matches the design brief, short-circuiting cheapest/strongest-first: Icon →
  Graphic/logo → Document → UI-screenshot → Photograph → **fallback = Photograph** (`confidence
  = FALLBACK_CONFIDENCE`). Contradictions resolve by **cascade precedence, not averaging**.
- [ ] The six→three collapse is fixed and total: `Photograph→Lossy`;
  `GraphicLogo|Icon|Document→LosslessFlat`; `UiScreenshot→MixedSafe`. `ImageClass::opt_bucket` is
  exhaustive (no `_ =>` arm) so a new label forces a conscious bucket choice.
- [ ] **Safe-fallback bias:** under ambiguity (`confidence < ~0.5`) the class is `Photograph`
  (→`Lossy`) — the safe downside is "file a bit larger", never "artifacts on text/edges". The
  bias, its rationale (crustyimg's lossy path is SSIMULACRA2-target-bounded), and the numeric
  thresholds are recorded in DEC-047.
- [ ] **`has_exif` is decisive** for the Photograph rule (camera prior); `source_format` biases
  (JPEG→photo lean, PNG/GIF/BMP→graphic lean, ICO→icon). These read off `Image`/`ImageInfo` with
  no re-parse.
- [ ] Determinism: same `(pixels, source_format, has_exif)` ⇒ byte-identical `class`/`opt_bucket`/
  `confidence` across runs and platforms (integer / fixed-order f32; no RNG, no wall-clock).
- [ ] Routes the labeled fixture corpus correctly (see Failing Tests): photo→`Lossy`,
  logo/flat-graphic/icon/document→`LosslessFlat`, ui-screenshot→`MixedSafe`.
- [ ] `just deny` stays green (no new dependency); **all existing tests pass unchanged**; no CLI
  command output changes (classification is still wired into nothing — STAGE-012 consumes it).

## Failing Tests

Written during **design**, BEFORE build. Fixtures are generated in-test with crustyimg's native
solid/gradient/noise generators (reuse SPEC-046's test helpers). The corpus is *synthetic but
representative* — each fixture is constructed to sit clearly inside one class so the assertion is
about the routing rule, not about a borderline pixel.

- **`src/analysis/mod.rs` (unit tests, extending SPEC-046's module tests)**
  - `"noise photo-like → Photograph → Lossy"` — a high-entropy noise RGB image (many colours,
    low flat_ratio): asserts `class == Photograph`, `opt_bucket == Lossy`.
  - `"exif prior forces Photograph even when flat-ish"` — a low-entropy image whose `Image`
    carries `has_exif = true`: asserts `class == Photograph` (the camera prior wins), documenting
    the honest gray zone (photo-of-a-document is still the right *format* call).
  - `"few-colour flat graphic → GraphicLogo → LosslessFlat"` — a 6-colour flat image with large
    fills: asserts `class == GraphicLogo`, `opt_bucket == LosslessFlat`.
  - `"tiny square → Icon → LosslessFlat"` — a 48×48 image (`max(w,h) ≤ ICON_MAX_EDGE`, aspect in
    range): asserts `class == Icon`, `opt_bucket == LosslessFlat`.
  - `"bimodal grayscale text → Document → LosslessFlat"` — a high-bimodality, high-gray-ratio,
    low-entropy image: asserts `class == Document`, `opt_bucket == LosslessFlat`.
  - `"flat-ish mid-colour screen-aspect, no exif → UiScreenshot → MixedSafe"` — a 16:9 image with
    moderate flat_ratio, colour count in (palette, ~50k], `has_exif = false`: asserts `class ==
    UiScreenshot`, `opt_bucket == MixedSafe`.
  - `"ambiguous gradient → fallback Photograph, low confidence"` — a smooth full-colour gradient
    that trips no strong rule: asserts `class == Photograph`, `opt_bucket == Lossy`, and
    `confidence <= FALLBACK_CONFIDENCE + ε` (the safe bias fired).
  - `"opt_bucket collapse is total"` — table test over every `ImageClass` variant asserting the
    fixed mapping to `OptBucket` (guards the exhaustive match).
  - `"degenerate input classifies without panic"` — a 1-px image: asserts it returns some class +
    confidence and does not panic (advisory path, no new error).
  - `"determinism: two classifications identical"` — `compute` twice on the same `Image` → equal
    `class`/`opt_bucket`/`confidence`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-002` — decode-once: the cascade reads SPEC-046's already-computed features; it never
  re-decodes and never re-parses metadata (`has_exif`/`source_format` are already on `Image`).
- `DEC-034` — decode limits: the buffer is already bounded; classification adds no allocation
  beyond the fixed consts.
- `DEC-047` (emitted with this spec) — the classification thresholds + the safe-fallback bias
  (default to `Photograph`/`Lossy` under uncertainty). The tuned constants live in one named
  block; DEC-047 explains why each anchor was chosen and its revisit trigger.

### Constraints that apply
- `untrusted-input-hardening` — classification is on the untrusted-input path: **no
  `unwrap`/`expect`/`panic!`**; it degrades to the fallback rather than erroring. All accumulators
  are already capped by SPEC-046.
- `no-agpl-default-deps` / `no-new-top-level-deps-without-decision` — hand-computed on the existing
  features; no `imageproc`, no ML crate, no new dependency at all.
- `test-before-implementation` — the corpus tests above are the contract; write them first.

### Prior related work
- `SPEC-046` (this stage) — the feature layer this spec classifies over; reuse its exposed
  `unique_colors` cap constant rather than defining a second copy.
- `DEC-024` (shipped) — `optimize` today is format-preserving; SPEC-048 uses `opt_bucket` to make
  it format-*choosing*. The enum shape here must serve that consumer.

### Out of scope (for this spec specifically)
- Wiring `class`/`opt_bucket` into any command — STAGE-012 (SPEC-048) reads it.
- A `classify` subcommand or any user-facing classification output.
- Per-field lazy memoization — classification is O(1) after the feature pass; keep it eager.
- Tuning against a *real* photo corpus — the synthetic corpus proves the routing rules; DEC-047
  records that real-corpus tuning is a revisit trigger, not a blocker for landing.

## Notes for the Implementer

- Keep the six→three collapse a single `match` with no wildcard arm, so adding a future label is a
  compile error until its bucket is chosen (the design brief warns against "six independent
  detectors" — one cascade, one collapse).
- Report a *lowered* `confidence` when a rule wins over a near-miss contradiction, so SPEC-049's
  `explain` can hedge honestly. Contradictions are resolved by precedence, never by averaging.
- `UiScreenshot` is the hardest class and the design brief permits merging it into `GraphicLogo`
  if the engine treats them identically — but the format engine's `MixedSafe` row (F, "try both
  and let measured bytes decide") *does* differ from `LosslessFlat`, so keep it distinct.
- All thresholds/consts in one named block (mirror `MAX_SEARCH_ITERS`/`AVIF_SPEED`); DEC-047
  references that block so the tuning surface is one place.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-047-classification`
- **PR (if applicable):** see STAGE-011 ship log (opened + merged in the autonomous run).
- **All acceptance criteria met?** yes — `ImageClass` (5) + `OptBucket` (3, exhaustive collapse) +
  `confidence` added to `Analysis`; 9 corpus/collapse/degenerate tests green; full suite 449
  (440 + 9); fmt/clippy/lean(+lean tests)/deny green; no new dependency; classification stays
  wired into no command.
- **New decisions emitted:**
  - None new. DEC-047 already captures the thresholds + safe-fallback-to-`Photograph` bias; the
    tuned constants live in the `src/analysis/mod.rs` classification-threshold block DEC-047 points
    to. The cascade-order refinements below are within DEC-047's stated intent.
- **Deviations from spec:**
  - **Cascade order refined vs. the design brief.** `has_exif → Photograph` is checked **early**
    (rule 2, before the graphic/document rules), to honour "`has_exif` is decisive" — a flat-ish
    camera photo must route lossy, and the corpus test `exif_prior_forces_photograph_even_when_flat`
    requires it. **Document is checked before Graphic** (both bucket `LosslessFlat`, but this makes
    the `Document` label reachable/assertable rather than being swallowed by the ≤256-colour gate).
  - **`source_format` lean is `Ico → Icon` only in v1.** The softer JPEG/PNG/GIF family leans from
    the brief are deferred — without a real labelled corpus they over-bias, and `has_exif` already
    carries the decisive camera prior. Recorded here (not a new DEC).
- **Follow-up work identified:**
  - None new. SPEC-048 consumes `opt_bucket` + `has_alpha`; `confidence` feeds SPEC-049's `explain`
    hedge. Both already in the STAGE-012 backlog.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — The brief's cascade lists Photograph last, but "has_exif decisive" implies it must be early;
   reconciling that (and Document-before-Graphic) was the main design judgement. Now explicit above.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-047 + `untrusted-input-hardening` covered it. Tuning the synthetic corpus fixtures to
   land cleanly in each class (esp. the UI-vs-graphic boundary, which needs real edges) was the
   real work — exactly the in-build tuning DEC-047 anticipated.
3. **If you did this task again, what would you do differently?**
   — State the cascade order (has_exif early, Document before Graphic) in the spec up front, and
   note that "smooth gradient" reads as *flat* under the shipped `FLAT_THRESHOLD` — so fixtures for
   the non-flat classes need genuine high-frequency content, not gradients.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — Encode the cascade *order* in the spec, not just the rule set. "has_exif is decisive" and
   "both Document and Graphic bucket LosslessFlat" have real ordering consequences that only
   surfaced when writing the corpus tests. Also: fixtures for non-flat classes need genuine
   high-frequency content — a smooth gradient reads as flat under `FLAT_THRESHOLD=4`.
2. **Does any template, constraint, or decision need updating?**
   — No. DEC-047's "tune the anchors against a labelled corpus in-build" played out exactly as
   written; the shipped constants are the tuned values it points to. Worth flagging for SPEC-048:
   the engine consumes `opt_bucket` + `has_alpha`; `confidence` is `explain`-only (SPEC-049).
3. **Is there a follow-up spec I should write now before I forget?**
   — No. STAGE-011 is complete; SPEC-048 (already written) is next and consumes this verdict.
