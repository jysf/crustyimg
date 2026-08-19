---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-095
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-18
supersedes: null
superseded_by: null

# Shared with SPEC-122 (linear-light resampling, amended in 2026-08-18) —
# this record covers the byte change and its migration posture for the whole
# STAGE-046 wave. Both specs rewrite `Resize::apply`; one decision, two
# amendments, rather than two records that would have to agree with each other.
affected_scope:
  - src/operation/**
  - src/sink/**
  # `color_type_bit_depth` was widened from private to `pub(crate)` here so
  # both of the above can read a source's depth (SPEC-121 punch-list item 5).
  # Without this glob the record does not surface when that fn changes, and
  # `decisions-audit --changed` attributes the file to 11 unrelated decisions.
  - src/image/mod.rs

tags:
  - colour-type
  - bit-depth
  - operation
  - sink
  - cache-key
  - lockfile
  - reproducible-build
  - quality-per-byte
---

# DEC-095: ops preserve colour type and bit depth, and `resize` resamples in linear light — the byte change, and its migration posture

## Decision

`Invert`, `Resize` and `Watermark` (`src/operation/mod.rs`) now **widen to work,
narrow to write**: each widens its input to an RGBA working buffer at the
input's own bit depth (`RgbaImage` for 8-bit, a local `Rgba16Image` alias for
16-bit) instead of unconditionally to `RGBA8`, and narrows back to the input's
colour type on the way out whenever doing so is lossless. Narrowing is
**lossless-only**, and it **preserves rather than minimises** — it never
returns a narrower type than the input declared. One shared rule
(`narrow_rgba8`/`narrow_rgba16`) answers three independent questions:

- **Alpha.** Drop the channel only when the input colour type had none *and*
  every alpha sample in the working buffer is opaque. An `Rgba8`/`La8` input
  that happens to be entirely opaque **keeps its channel** — do not strip a
  channel the user supplied. (The code did this from the build; the first
  version of this record described the rule without the `has_alpha` clause,
  and was wrong about that behaviour. Corrected here, not in the code.)
- **Chroma.** Collapse to one channel only when the input was itself a luma
  type (`L8`/`La8`/`L16`/`La16`) *and* every pixel still has `r == g == b`.
  An `Rgb8` input whose pixels happen to be gray stays `Rgb8`; a colour
  watermark over a gray base genuinely adds chroma and stays RGB.
- **Depth.** 16→8 **never** — that is a downgrade, not a narrowing.

**`Resize` additionally resamples in linear light (SPEC-122, amended in
2026-08-18).** `Resize::apply` used to hand the widened RGBA buffer straight
to `fast_image_resize` as `U8x4`/`U16x4`, so every weighted average was taken
on non-linear sRGB samples — arithmetic on values that are not proportional to
the physical quantity. It now **linearizes → resamples in `F32x4` → re-encodes**
at the input's own bit depth. Alpha is deliberately not transformed (it is
coverage, not a light value), and premultiplication is untouched — DEC-092
refuted the premultiplied-alpha half of the same premise, and `F32x4` takes the
same `ResizeOptions` the integer types took.

**The transfer function is assumed to be sRGB, and this is the assumption
being made explicit** (SPEC-122 Call 4). crustyimg carries ICC profiles through
but does not interpret them, so an image tagged with a different transfer
function is now resampled under an assumption that is wrong *for it* — better
than the previous assumption, which was that the samples were already linear
(wrong for essentially every image), but still an assumption. ICC-aware
conversion is its own project and is explicitly not this one.

A same-size resize short-circuits before the conversion. `fast_image_resize`
does not resample in that case either — `Resizer::resize` falls through to a
row copy (`resizer.rs`'s `copy_image`) — so the round-trip would be a no-op in
exact arithmetic. It is skipped for cost, not for correctness: the round-trip
was **measured exact for all 256 8-bit and all 65,536 16-bit values**, and
removing the short-circuit leaves every test green (see Validation).

`Watermark` decides the narrow from the composite, not by exemption.
Source-over compositing can only *produce* a non-opaque pixel where the base
already had one — `a_out = a_base + a_ov·(1 − a_base)` is 1 for every overlay
alpha when `a_base` is 1 — so a base with **no alpha channel** composites to a
fully opaque result and narrows however translucent the overlay was, and a
base carrying genuine transparency keeps its channel. No `Watermark`-specific
carve-out.

`sink::encode_to_bytes_with` additionally reports (stderr, one line) when a
>8-bit source is encoded to a target that can only hold 8 bits per
channel — JPEG (always) and lossy WebP (`webp-lossy` feature, quality set).
`image`'s own `write_to`/`write_with_encoder` already perform this downgrade
silently ("methods on `DynamicImage` try to automatically convert the image
to some color type supported by the encoder" — the crate's own doc comment);
this only makes the existing, silent behaviour visible, in the spirit of
SPEC-090's honest size reporting. Lossless WebP, GIF, BMP, ICO and AVIF are
**not** covered — Call 3's settled scope named JPEG and lossy WebP only (see
Consequences: lossless WebP was measured to have the identical gap and is
filed, not fixed, here).

**No new cache-key component, no new migration machinery** (Call 4). The
byte change is real and affects every existing recipe that runs `resize`,
`thumbnail`, `edit --invert`, `web` or `watermark` — and, since SPEC-122, for every recipe
that runs `resize` at any scale other than 1:1 — but `cache_key_for`
already includes `crate::version()` (`src/cli/build.rs:294`, DEC-058), so a
version bump changes every key, and the lockfile's `hash` was never promised
stable across versions (`src/build/lock.rs:32-36`). The normal upgrade path
— key changes, `--frozen` fails, the user regenerates — was driven and
confirmed, **conditionally** (see Consequences: it holds only when the
version is actually bumped, which neither build does), and **re-driven
independently for SPEC-122's byte change with the same result**.

## Context

Full defect evidence: `docs/backlog.md`, "⚠ Live defect — ops widen to RGBA
and never narrow back" (2026-08-15). Summary: three op bodies called
`to_rgba8()` unconditionally, so every verb that runs a real `Operation` —
including the flagship `web` (`auto-orient → resize → optimize`) — added an
all-opaque alpha channel (+12.4% bytes on a 512×512 representative PNG,
measured pre-fix) and silently halved 16-bit input. The clean verbs
(`convert`, `optimize`, `auto-orient`) were exactly the ones that ran no
`Operation` — decode, `Identity`, `AutoOrient` and the default `write_to`
path all already preserved the `DynamicImage` variant.

`Resize::apply` no longer manually constructs a
`fast_image_resize::images::Image` wrapper. `fast_image_resize` 6.0.0's
`image_crate.rs` provides blanket `IntoImageView`/`IntoImageViewMut` impls
directly for `RgbaImage` and (by concrete type, not by alias name) for the
16-bit RGBA buffer this module defines locally — so `Resizer::resize(&src,
&mut dst, &opts)` takes the pixel buffers directly, at whichever bit depth
the caller widened to, with no manual `PixelType` bookkeeping. This is
mechanical simplification riding the bit-depth fix, not a separate change.
**Superseded in part by SPEC-122's amendment**: `Resize` now converts to a
`TypedImage<F32x4>` for the resample itself, so it no longer rides those
blanket impls — `Invert`/`Watermark` still work on the `image` buffers
directly, and the widen/narrow rule is unchanged for all three.

## Consequences

- **Positive — closes a real, measured defect on the flagship verb.** `web`
  on an RGB(8-bit) source now returns `colour_type=2` (was 6); on a 16-bit
  RGB source, `resize`/`edit --invert` now return 16-bit (was silently
  halved to 8-bit RGBA). Verified end-to-end via the compiled binary, not
  only unit-level (`tests/colour_type_preservation.rs`).

- **Positive — the byte win is measured, not assumed.** Same RGB pixels
  encoded as RGB vs RGBA (PNG): RGB is materially smaller
  (`rgb_output_is_smaller_than_rgba_for_the_same_pixels`).

- **Neutral — `Resize` deliberately still widens to RGBA before resizing,
  even for an alpha-less source.** `fast_image_resize` can resize a
  non-RGBA `DynamicImage` variant directly (its `IntoImageView` impl covers
  `Rgb8`/`Rgb16`/`Luma*` too), which would make `Resize` narrower-than-RGBA
  internally and skip narrowing entirely for the common RGB case. Not taken:
  SPEC-122 lands linear-light resampling in this same function next, and
  keeping one shared widen/narrow rule across all three ops (rather than a
  second, Resize-only code path) is worth more than the internal-buffer
  saving. Revisit if SPEC-122 finds the direct-variant path is a better
  foundation for its own rewrite.

- **Negative, MEASURED at AC-8 — the migration story holds only when the
  version is actually bumped; this build does not bump it.** Built a
  `crustyimg.build.toml` target with `main`'s pre-fix binary, committed the
  lockfile, then ran the branch binary (unbumped `0.7.0`, per this spec's
  own "do not bump the version" guardrail): `build --check` reported
  **"lockfile is up to date," exit 0**, and a plain `build` served the
  stale, pre-fix `rgba8` bytes from cache with **zero warning**. Rebuilding
  the identical branch source at a bumped `0.7.1`: key changed, `--check`
  failed **exit 7** with an explicit drift message, and a plain `build`
  regenerated the correct `rgb8` output. All four of AC-8's checks (key
  changes / `--frozen` fails / regeneration succeeds / no stale entry
  served) hold — **conditionally on the tag landing**. Filed as a STAGE-042
  backlog item (not fixed here — Call 4 forbids inventing cache-key
  machinery in this spec) because it sharpens SPEC-124's stage note ("must
  ship before the next tag") from a process preference into a measured
  hazard: any same-version fix merged between tags — this one included —
  is invisible to `cache_key_for` until the version actually moves.

- **Negative, MEASURED — lossless WebP has the identical 8-bit-only gap
  Call 3 warns about for JPEG/lossy WebP, and this spec does not warn on
  it.** Driving `web` on a 16-bit RGB source: `optimize`'s smallest-candidate
  search picked WebP (smaller than PNG for that fixture) via the *default*
  (lossless, no feature required) `write_to` path, and the output decoded
  as 8-bit — `image`'s own lossless WebP encoder has no 16-bit mode, so the
  same "automatically convert" silent downgrade fires there too. Call 3's
  settled scope names JPEG and lossy WebP only; widening it to cover every
  8-bit-only format (WebP lossless, GIF, BMP, ICO) is a design call this
  spec does not reopen. Filed rather than fixed — as a `- [ ]` item in
  **STAGE-042**, the backlog `just backlog` actually reads. (The build filed it
  only in this record's prose, and a `tests/colour_type_preservation.rs`
  comment cited a `docs/backlog.md` entry that was never written; both
  corrected at punch-list. Re-driven then, and it is worse than first stated:
  `convert --format webp` reaches it directly, not only `web` via `optimize`'s
  candidate search, and the run prints `ssim 100.0` — the metric is computed on
  8-bit renderings, so the honest-size line reads as reassurance for exactly
  the loss it cannot see.)

- **Neutral — corrects the "pipeline is 8-bit throughout" claim** in
  `docs/lab-plan-2026-08.md` (F8) and `docs/roadmap.md`, both of which cited
  the now-changed `to_rgba8()` call sites as evidence. F8's own point (lab's
  *future* ops risk the same banding if they don't widen the way these three
  now do) is preserved, not deleted.

- **Positive, MEASURED at punch-list — grayscale is preserved too, and it was
  the largest remaining gap.** AC-1/AC-2 name RGB only, so they were literally
  met while `Gray8 → resize` still returned `Rgb8` — Call 1 was not (AGENTS
  §15: an acceptance criterion may not transfer to a surface it was not
  written against). It is the same narrowing mechanism, so it was **fixed, not
  scoped out**: one extra clause in `narrow_rgba8`/`narrow_rgba16`, gated on
  the input being a luma type so nothing is promoted. Measured on a 32×32
  gradient, `resize --max 16`: `L8` **852 → 340 B (−60.1 %)**, `L16`
  **1,559 → 596 B (−61.8 %)**, `La8` **962 → 447 B (−53.5 %)** — against a
  pre-fix `main` that returned 962 B of RGBA8 for all three. Roughly 4× the
  relative saving of the RGB→RGBA case that motivated the spec. The channel is
  taken verbatim from the working buffer's first channel rather than through
  `to_luma8()`, whose luminance weights round-trip an already-gray pixel only
  approximately.

- **Negative, MEASURED at punch-list, now fixed — `watermark --text` did not
  narrow, so the verb's commonest invocation still paid the wasted channel.**
  Source-over onto an opaque base is mathematically always opaque, but
  `image`'s `Rgba::blend` computes `a_out` in `f32` and casts with `NumCast`,
  which truncates: `1.0 + a − a` lands on `0.99999994` for **32 of the 254
  possible overlay alphas** (128 among them), and `255 × 0.99999994` truncates
  to **254**. Anti-aliased glyph edges produce exactly those samples — 36 of
  65,536 on a 256×256 base — and defeated the opacity scan. Output was
  **66,313 B as RGBA vs 53,970 B as RGB: 18.6 %** of the file. Fixed by
  restoring the alpha the maths requires (`restore_opaque_alpha8`/`…16`)
  before narrowing, gated on the base having had no alpha channel at all — so
  no numeric tolerance is introduced and genuine transparency is never
  touched. **The fix does not depend on `image` truncating rather than
  rounding**, which verify flagged as a latent CI break in the old test.
  ⚠ **Consequence worth stating: `Watermark`'s narrow decision is now
  structural rather than empirical for an alpha-less base.** A uniformly
  half-transparent overlay over an RGB base narrows too, where before it
  stayed RGBA. That was never a decision — it was the same rounding artifact,
  and 128 is one of the 32 alphas that trips it — but it flips the old
  `watermark_keeps_alpha_when_the_overlay_is_translucent` case. That control
  was retargeted onto a base with genuine transparency (the only base from
  which a source-over composite can come out non-opaque) and strengthened to
  assert the transparent pixels themselves survive, not just the channel.
  The `f32` truncation is untouched for an alpha-bearing base: such an output
  still carries a few 254s where 255 is correct. That is upstream compositing
  precision, not a narrowing question, and is out of scope here.

- **Neutral — `src/image/mod.rs` is in `affected_scope`.**
  `color_type_bit_depth` moved from private to `pub(crate)` so `operation` and
  `sink` can share it. Measured consequence of the omission, before it was
  corrected: `just decisions-audit --changed main` attributed that file to
  **11 unrelated decisions** and this record did not surface at all.

- **Neutral — Call 3's warning names the source's real depth.** It was a pair
  of constants reading "16-bit source downgraded to 8-bit", which is false for
  a 32-bit-float source hitting the same branch (`Rgb32F`/`Rgba32F` are
  constructible through the public `Image::from_parts`). Now built by a pure
  `eight_bit_downgrade_warning(ColorType, target)` that reads the depth from
  the image and returns `None` when nothing is lost — unit-tested at both
  depths, since `eprintln!` is not capturable in-process.

### Amended 2026-08-18 — SPEC-122's consequences

- **Positive, MEASURED — the defect the spec was written against is closed, on
  the same harness and against the same outside tool.** Re-running
  `scripts/spec120_linear_light.py` against the branch binary, reference
  regenerated with ImageMagick 7.1.2-29 Q16-HDRI: SSIMULACRA2 **−63.85 →
  100.00** (synthetic worst case), **70.45 → 100.00** (`graphic_large.png`),
  **84.45 → 99.41** (`photo_forest_cc0.jpg`); mean signed linear-luminance
  error −0.104350 / −0.001386 / −0.004920 → **0.000000 on all three**. The two
  oracles agree, so there is no disagreement to report. The agreement with the
  reference is *exact* on two cases (ImageMagick's own `compare -metric AE`
  returns 0) — tighter than two independent implementations usually manage,
  and explicable: the sRGB OETF's slope is below 1 above ~0.03 linear, so it
  compresses small differences in the linear intermediate and more of them
  round to the same 8-bit code. The photo, the least regular case, is the one
  that does not land exactly (AE 0.00098).

- **Negative, MEASURED — `resize` is 3.83× slower at the op, 1.5–2.5× slower
  end to end, and the transfer function is not the reason.** `benches/pipeline.rs`
  `resize` (256²→128² RGB, criterion, 100 measurements per arm, Apple M4 Pro,
  release): **169.27 µs → 648.52 µs, +283.5% (p < 0.05)**. Whole-verb, best of
  9: 8.1→12.2 ms (800×532 → max 400), 113.9→203.5 ms (4000×2660 → max 1600),
  20.0→49.0 ms (2048²→256²). Decomposed with two nested diagnostic builds,
  both discarded: the *same* `F32x4` pipeline with **no transfer function at
  all** still costs 515.88 µs, so **72% of the added time is the `F32x4`
  working type**, not the maths; and swapping the `powf` encode for a
  threshold-table search recovers **7 µs of 479** while not even being
  bit-exact (2 pixels of 160,000 move by 1). **The cheap optimisation does not
  exist here.** Recovering the time means changing the working type — `U16x4`
  has SIMD kernels `F32x4` does not, at the cost of quantizing the linear
  intermediate, where 16 bits is close to the floor for 8-bit sRGB shadows.
  That is a design call, filed for the architect, deliberately not taken during
  build (SPEC-122 AC-9 forbids optimising out of the spec).

- **Negative — AC-6 asked for the upscale path to stay byte-identical to
  `main`, and it cannot.** An upscale *is* a resample: Lanczos3 interpolates,
  and interpolating non-linear samples is wrong in the same way averaging them
  is. Rather than gate the linearization on direction — which would leave a
  discontinuity at 100%, and no answer at all for `fill`/`cover`, where one
  axis can shrink while the other grows — the fix applies to every real
  resample. Measured against the same independent reference, the upscale was
  defective in the same direction and improved by the same fix:
  `graphic_large.png` 512²→1024² **65.93 → 100.00**, `photo_forest_cc0.jpg`
  800×532→1600×1064 **89.16 → 98.44**. The *no-op* half of AC-6 holds and is
  asserted at four colour types.

- **Neutral, MEASURED — the alpha edge improved where AC-5 predicted nothing
  would move, and DEC-092's explanation of the residual was wrong.** Max
  premultiplied-RGB edge error against the independent premultiplied reference
  went **27/255 → 0**, mean **0.364 → 0.0** (`compare -metric AE` = 0
  independently; the `use_alpha(false)` control arm still reads 68/18.34 and AE
  346, so the oracle can still fire). DEC-092 read the residual 27 as "Lanczos
  ringing at hard corners"; it was **8-bit quantization inside
  `fast_image_resize`'s own premultiply/divide round-trip**, which disappears
  when that round-trip happens in `f32`. Premultiplication itself did not move:
  C5 still shows the shipped binary differing from the non-premultiplied arm in
  10,512 pixels. Nothing regressed; the AC was simply written expecting a null
  where an improvement was available.

- **Neutral — the same-version cache hazard reproduces on a second byte
  change.** AC-8 was re-driven end to end for SPEC-122: at the unbumped 0.7.0
  the key was unchanged, `build --check` reported "lockfile is up to date"
  **exit 0**, and a plain `build` served the **stale pre-fix bytes**
  (`1c4ebb57…`) with no warning; at a bumped 0.7.1 the key moved
  (`e4abb010…` → `d8abf134…`), `--check` failed **exit 7** with an explicit
  drift message and left the lockfile untouched, and a plain `build`
  regenerated the linear-light output (`87642f1a…`, byte-identical to a direct
  `resize` on the branch). This is the STAGE-042 item SPEC-121 filed,
  independently confirmed rather than assumed — two unrelated byte changes,
  same conditional.

- **Positive, MEASURED and unlooked-for — the demo's `.wasm` shrinks 16.9%.**
  Calling `fast_image_resize`'s typed entry point (`resize_typed::<F32x4>`)
  instead of the dynamic `Resizer::resize` — which matches on `PixelType` and so
  monomorphizes the convolution and alpha kernels for all thirteen — makes twelve
  instantiations unreachable, and the linker drops them. Same toolchain, AVIF on
  both sides: **5,819,379 → 5,261,547 B raw, 1,377,233 → 1,144,864 B brotli**.
  CI's size guard went red on it (the change is outside its ±5% band on the low
  side), so `WASM_BROTLI_BASELINE` is moved to the CI-measured 1,144,921 B with
  the reason recorded beside the constant. The floor was checked before it was
  moved rather than after: a lean `--no-default-features` build measures 865,980 B,
  **20.4% below the new floor**, so the guard still catches the missing AVIF
  encoder it exists for (DEC-065). The guard's "Under:" message asserted that
  single cause and was wrong here, so it now names both — the failure mode
  `wasm-artifact.mjs`'s own header warns about.

- **Neutral — `examples/spec120_linear_probe.rs` and
  `scripts/spec120_linear_light.py` are kept, not deleted.** They are the
  acceptance test (SPEC-122 Call 3), they are named in DEC-092's
  `affected_scope`, and their controls C1/C3 now serve as the negative control
  for this change: C1 (the probe's sRGB arm reproduces the shipped binary
  pixel-exactly) flips **PASS → FAIL** and C3 (the binary agrees with
  ImageMagick's sRGB-space resize) goes from a mean |luma err| of 0.0003 to
  0.1045 — both are the behavioural flip, observable without rebuilding
  anything.

## Alternatives Considered

- **A new cache-key component (e.g. a colour-pipeline-version field).**
  Rejected — Call 4 established at design that the version component already
  covers this, and the AC-8 drive confirms it (conditionally on a version
  bump). Adding a second component would be inventing migration machinery
  the spec's Call 4 explicitly forbids.
- **Widening Call 3's warning to every 8-bit-only format.** Rejected for
  this spec — out of the settled Call 3 scope (JPEG + lossy WebP only); filed
  as a Consequence instead of silently expanded.
- **Resizing the source `DynamicImage` variant directly (no RGBA widening)
  in `Resize::apply`.** Considered and rejected for now — see the Consequences
  entry above; revisit alongside SPEC-122. **Revisited at SPEC-122 and still
  rejected:** the linear-light path converts to `F32x4` regardless, so
  resampling the narrower source variant would save one widening pass and cost
  a second code path, against a conversion that the AC-9 decomposition shows is
  not where the time goes.
- **Gating the linearization on downscale only, to keep AC-6's upscale half.**
  Rejected — it would make the correction discontinuous at 100% and undefined
  for `fill`/`cover`, where one axis can shrink while the other grows, and the
  upscale direction was measured to be defective in the same way (see
  Consequences).
- **A `U16x4` linear intermediate instead of `F32x4`.** Not taken during build.
  It is the plausible answer to the 3.83× cost, but it quantizes the linear
  intermediate — where 16 bits is close to the floor for 8-bit sRGB shadows —
  so it trades measured quality for measured speed. That is the architect's
  call; the build's job was to measure the cost, which it did.
- **A cheaper transfer function (threshold table, polynomial).** Measured and
  rejected as pointless: it recovers 7 µs of 479 and is not bit-exact.

## Validation (SPEC-122's half — added 2026-08-18)

Three negative controls, **one revert per independent condition**, each built
and run rather than reasoned about (AGENTS §15: the behavioural flip is the
evidence, not a hash):

- **Revert A — the linearization removed** (`git checkout main -- src/operation/mod.rs`,
  rebuilt). The three linear-light tests go **RED** with the expected numbers
  (`downscale_scores_better…` scores −3.99, identical to its own sRGB-space
  control; `mean_luminance_error…` reports −54.8% of the reference mean), while
  `translucent_edge_error_is_unchanged`,
  `resize_does_not_apply_the_transfer_function_to_alpha` and
  `noop_resize_is_byte_identical_to_its_source` **run and pass**. The harness on
  the same reverted binary returns **exactly −63.85 / 70.45 / 84.45** and an
  alpha edge error of **27** — the pre-fix numbers, to the digit.
- **Revert B — alpha put through the transfer function too.** Only
  `resize_does_not_apply_the_transfer_function_to_alpha` goes red; the other
  five run and pass. The two conditions are independent, not co-dependent.
- **Revert C — the same-size short-circuit removed.** **All six tests still
  pass**, which is the honest result and is why the short-circuit is described
  above as a cost measure rather than a correctness one: the transfer-function
  round-trip is exact for every representable value, so the guard has no
  behaviour for a test to see. Stated rather than papered over — a control that
  cannot go red is worth knowing about.

Two unit tests carry the exactness claim rather than a fixture that samples it:
`srgb_round_trip_is_exact_for_every_8_bit_value` (all 256) and
`srgb_round_trip_is_exact_for_every_16_bit_value` (all 65,536), plus
`srgb8_linear_table_matches_the_analytic_function`, which asserts the lookup
table is bit-identical to the per-sample conversion so the table is provably an
optimisation and not a second implementation.

The in-repo tests are **hermetic and are not the independent oracle**: AGENTS
§12 forbids shelling out to ImageMagick for fixtures, so they derive their
reference from the sRGB standard's transfer function, re-stated in the test
file rather than imported from the code under test. The outside-tool
measurement is `scripts/spec120_linear_light.py`, re-run per SPEC-122 Call 3
with the reference regenerated, never substituted (AC-4).

## References

- Related specs: **SPEC-121** (the colour-type/bit-depth half), **SPEC-122**
  (the linear-light half, same function, amended into this record 2026-08-18),
  **SPEC-120/DEC-092** (the measurement that justified SPEC-122 and refuted its
  premultiplied-alpha half), SPEC-090 (honest size
  reporting, the precedent for Call 3's diagnostic), SPEC-123/DEC-094 (the
  sibling STAGE-042 finding this migration item sits beside).
- `docs/backlog.md` — "⚠ Live defect — ops widen to RGBA and never narrow
  back."
- `projects/PROJ-010-post-launch-correctness-and-consolidation/stages/STAGE-042-release-safety-instruments.md`
  — the filed cache-key-vs-version-bump item.
- DEC-058 (cache-key composition), DEC-090 (diagnostic channel — PROPOSED,
  not yet accepted; Call 3's warning uses a plain `eprintln!` to match the
  existing `TRUNCATED_JPEG_WARNING`/`ANIMATED_INPUT_WARNING` convention
  rather than wait on DEC-090).
