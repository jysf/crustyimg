---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-110
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-10-30
# Narrows what DEC-064's "transform runs the same core the CLI runs" promises about pixels.
# Supersedes nothing; DEC-008 and DEC-092 stand unchanged.
supersedes: null
superseded_by: null

# Block-list form on purpose: the audit drops an inline array (PROJ-011 HANDOFF trap 7).
affected_scope:
  - tests/ddphotos_parity.mjs
  - tests/ddphotos_decode.py
  - tests/fixtures/ddphotos/**
  - integrations/ddphotos/edit/**
  - src/wasm.rs
  - src/operation/mod.rs

tags:
  - wasm
  - parity
  - resize
  - testing
  - ddphotos

references:
  - dec-064
  - dec-008
  - dec-092
  - dec-059
  - dec-066
---

# DEC-110: Native and wasm agree on dimensions exactly and on resampled pixels within 1/255

## Decision

**Rule B.** When the same recipe runs through native `crustyimg apply` and wasm `transform` on
the same input bytes, on the same runner:

1. **Dimensions are always exact.** A size difference fails before any pixel is compared.
2. **A recipe with no resampling step is exact:** zero differing pixels. In SPEC-142's row table
   these are rows **1, 3, 9 and 11**, the only rows whose recipe has no `resize` step.
   `cliRecipe` always resizes to the page's long edge, so rows 5 and 7 resample even though their
   edits (`none`, `wm`) ask for no new size.
3. **A recipe that resamples may differ by at most 1/255 per channel, on at most K pixels.**
   - **K** is one literal in `tests/ddphotos_parity.mjs`. The SPEC-142 build sets it to the largest
     differing-pixel count observed on any resampling row, over both targets it has run (aarch64
     locally and the first x86_64 CI run). No headroom is added. The build records K in its Build
     Completion and in this file's Validation section.
   - **The cap** is 0.01 % of a row's pixels, `floor(W × H / 10 000)`: 171 at 1600×1071, 96 at
     1200×803, and **78** at 1080×723 (and 723×1080). One K covers every resampling row, so the cap
     in force is the smallest, **78**. (The ruling's prompt said 77; 1080 × 723 = 780,840, so 0.01 %
     is 78.08 and the floor is 78.)
   - **Above the cap, the build stops** and reports BLOCKED, with the row and its numbers. It does
     not raise the cap. Raising it is a new decision.
4. **The driven negative control still fails.** Native `opacity = 0.6` → `0.61` differs by 3/255
   on the lossless rows and 13–22/255 on the lossy ones. Both are above the 1/255 limb, so N1 stays
   red under this rule.

## Context

PROJ-017's thesis is that a recipe from the ddphotos edit page means the same thing to the native
CLI as to the page's wasm build. SPEC-142's frame measured exact agreement (48/48 rows
pixel-identical) at a 1024 px long edge. Its design then drove the page's real sizes (1200, 1080,
800, 400) on aarch64 at `9fbfbe1`. The recipe was `auto-orient` + `resize mode = "max"`, PNG out,
decoded by Pillow 12.3.0. Each case ran twice:

| input | 1200 | 1080 | 800 | 400 |
|---|---|---|---|---|
| landscape WebP 1600×1071 | **1 px, max 1** (both runs) | 0 | 0 | 0 |
| portrait WebP 1071×1600 (no EXIF tag) | **1 px, max 1** | 0 | **2 px, max 1** | 0 |
| JPEG 1600×1071, EXIF orientation 6 | **1 px, max 1** | **1 px, max 1** | 0 | 0 |
| the same JPEG, tag stripped | 0 | 0 | 0 | 0 |

- **It is deterministic.** Both runs agreed, and the full wasm build gives the same table as the
  slim one.
- **One example:** landscape at 1200, pixel (507, 417): native `(66, 56, 37, 255)`, wasm
  `(66, 55, 37, 255)`. That is a rounding boundary.
- **The JPEG decoder is not the cause.** With the EXIF tag stripped, the JPEG matches at every
  size. With the tag, `auto-orient` changes the shape the resize sees.

**Probable cause. This is a prior, not proven.** `resize` works in linear light in `f32`
(DEC-092). It uses `powf` for the sRGB curve, then Lanczos3 on `fast_image_resize`'s
per-architecture SIMD backend (DEC-008). Either step can round differently on native and wasm32.
Nobody has isolated which one. x86_64 (AVX2, glibc `powf`) is unmeasured, and it may differ more.

The frame ruled the property "engine parity, exactly", and that can't be met without a change to
`src/`. The maintainer ruled (B) on 2026-10-30.

## Alternatives Considered

- **Option A: exact, by fixing the engine first.**
  - What it is: make resampling bit-identical across targets. That means one `powf` on every
    target, and an `f32` convolution with the same evaluation order on every backend (or the
    portable backend everywhere). SPEC-142 would then assert zero differing pixels everywhere.
  - Why not now: PROJ-015's working-type change (`F32x4` → `U16x4`) rewrites this same resize path
    anyway, so an exactness fix made today would be rewritten by it. It would also move native
    bytes for every resize (SPEC-141's guard would flag it), may cost the SIMD speed that is
    DEC-008's reason, and needs its own spec outside PROJ-017's build order. **Exactness stays a
    target.** It is filed as `native-wasm-resize-exactness` in `guidance/ideas.yaml`, gated on that
    PROJ-015 stage.

- **Option B2: a looser bound** (for example 2/255, or a percentage of pixels with no 1/255 limb,
  or a PSNR floor).
  - Why rejected: everything measured is 1/255 on 1–2 pixels. A looser bound would hide the first
    real divergence, which is the job the harness exists to do. A PSNR floor alone would pass a
    3/255 shift on a few pixels, which is N1's size, so it would leave the negative control unable
    to fail. The bound should be the smallest one that holds today, plus a cap that turns growth
    into a stop.

- **Option B (chosen): bounded, with an exact limb and a capped K.**
  - Why selected: it is green on today's measured behaviour and keeps every no-resample row exact.
    It stays able to fail on a one-parameter change (N1) and on one pixel of geometry (N2, through
    the dimensions limb). K comes from measurement, not choice, and the cap means K can't grow
    quietly. A 1/255 step on a few pixels per million is invisible on the page. It is also inside
    the cross-target noise that DEC-059 already declines to pin.

## Consequences

- **Positive:** SPEC-142 can build now. The harness stays exact wherever exactness holds (no
  resampling) and turns any growth in the resize difference into a red build, not a drift.
- **Negative:** "the same recipe gives the same pixels" is now true only within the bound wherever
  the recipe resamples. Straighten (STAGE-059) and any later interpolating step have no exact
  claim either. Each states its own bound, and none inherits this one by default.
- **Neutral:**
  - **SPEC-141's native goldens are unaffected.** They are native-vs-native on one target, and
    this rule says nothing about them.
  - **Bytes on resampling rows are reported, not asserted** (a 1/255 pixel changes PNG bytes).
    The exception is JPEG under SPEC-142's x86 outcome (i): JPEG bytes are asserted only if both
    targets showed them equal.
  - **Lossy rows carry a risk this rule does not hide.** A 1/255 difference before a lossy encode
    (JPEG, AVIF) can decode as more than 1/255 afterwards. aarch64 shows none (row 10 is
    byte-equal). If x86 shows it, that row is over the bound and the build reports BLOCKED.
  - **PROJ-015's working-type stage must re-measure this bound.** That change rewrites the resize
    path. It must rerun the parity harness, record the new maximum, and either confirm K or open a
    new decision. If native pixels move, it must also re-bless SPEC-141's goldens. The
    `src/operation/mod.rs` entry in `affected_scope` exists so that `just decisions-audit
    --changed` names this DEC on that change.

### Why this `affected_scope`

- `tests/ddphotos_parity.mjs`: the comparator, K and the exact-row set live here.
- `tests/ddphotos_decode.py`: the decoder the pixel counts come from. A change to it changes
  what "differing" means.
- `tests/fixtures/ddphotos/**`: K was measured on these bytes. Regenerating them changes pixels
  (their README says so), so K must be re-measured.
- `integrations/ddphotos/edit/**`: the page's recipes, and so which rows resample.
- `src/wasm.rs`: `transform`, the wasm side of the comparison.
- `src/operation/mod.rs`: the resize path and the probable seat of the difference. Not
  `src/operation/**`: the other operation files don't resample.
- Not `ci.yml` or `justfile`: they run the harness and don't define the rule.

## Validation

This decision is wrong, and must be revisited, if any of these happens:

- **Any resampling row differs by more than 1/255** in any channel, on any target. That is outside
  the rule, not a reason to widen it.
- **K exceeds the cap (78)** on any target.
- **A no-resample row (1, 3, 9, 11) differs by any amount.** Rule B never covered those rows.
- **N1 stops failing.** Then the bound has grown past the smallest real change it must detect.
- **PROJ-015's working-type change lands.** Re-measure. If the difference goes to zero, Option A
  arrived for free: set K = 0 and supersede this DEC with an exact one.

**K as set by the SPEC-142 build: K = 1.**

- **Where it was observed:** 1 differing pixel, 1/255 in one channel, on each of rows 2, 6, 15 and
  16, on both targets. Every other row matched exactly.
  - aarch64-apple-darwin (the maintainer's Mac).
  - x86_64-unknown-linux-gnu, GitHub-hosted `ubuntu-latest`: AMD EPYC 9V45 96-Core Processor,
    with avx2, avx512f and sse4_1. CI run 41144102452 (job `ddphotos page parity`, draft PR #234).
- **Wasm agreed with itself across targets:** all 16 wasm output SHA-256 hashes were identical on
  the Mac and on the runner, and the two targets' per-row `px`/`max` were identical.
- The resizer picks its SIMD path from the CPU at run time, so a runner with a different CPU model
  could in principle give another count. The cap (78) is unchanged.

## References

- Related specs: SPEC-142 (the harness that asserts this), SPEC-141 (native goldens, unaffected),
  SPEC-120/SPEC-122 (linear-light resize)
- Related decisions: DEC-064 (wasm runs the same core), DEC-008 (resize backend), DEC-092 (linear
  light in `f32`), DEC-059 (no cross-arch byte pins), DEC-066 (the slim build's profile)
- Ruling: maintainer, 2026-10-30, recorded in SPEC-142's "Maintainer rulings (continued)"
- Filed: `guidance/ideas.yaml` `native-wasm-resize-exactness`; PROJ-015 brief, working-type risk
