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

# Shared with SPEC-122 (linear-light resampling lands in the same
# `Resize::apply` function next) — this record covers the byte change and its
# migration posture for the whole STAGE-046 wave. SPEC-122 amends this file
# rather than minting a second decision.
affected_scope:
  - src/operation/**
  - src/sink/**

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

# DEC-095: ops preserve colour type and bit depth — the byte change, and its migration posture

## Decision

`Invert`, `Resize` and `Watermark` (`src/operation/mod.rs`) now **widen to work,
narrow to write**: each widens its input to an RGBA working buffer at the
input's own bit depth (`RgbaImage` for 8-bit, a local `Rgba16Image` alias for
16-bit) instead of unconditionally to `RGBA8`, and narrows back to the input's
colour type on the way out whenever doing so is lossless. Narrowing is
**lossless-only** — RGBA→RGB only when every alpha sample in the working
buffer is opaque, and 16→8 **never** (that is a downgrade, not a narrowing).
`Watermark` decides the narrow from the actual composited alpha, not by
exemption: a fully-opaque overlay over an RGB base narrows like the other two
ops; a genuinely translucent overlay leaves non-opaque samples and stays RGBA
— one shared rule (`narrow_rgba8`/`narrow_rgba16`) covers both directions,
no `Watermark`-specific carve-out.

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
`thumbnail`, `edit --invert`, `web` or `watermark` — but `cache_key_for`
already includes `crate::version()` (`src/cli/build.rs:294`, DEC-058), so a
version bump changes every key, and the lockfile's `hash` was never promised
stable across versions (`src/build/lock.rs:32-36`). The normal upgrade path
— key changes, `--frozen` fails, the user regenerates — was driven and
confirmed, **conditionally** (see Consequences: it holds only when the
version is actually bumped, which this build does not do).

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
  spec does not reopen. Filed rather than fixed —
  `tests/colour_type_preservation.rs` documents the boundary inline rather
  than asserting a claim `web` cannot keep for a 16-bit source.

- **Neutral — corrects the "pipeline is 8-bit throughout" claim** in
  `docs/lab-plan-2026-08.md` (F8) and `docs/roadmap.md`, both of which cited
  the now-changed `to_rgba8()` call sites as evidence. F8's own point (lab's
  *future* ops risk the same banding if they don't widen the way these three
  now do) is preserved, not deleted.

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
  entry above; revisit alongside SPEC-122.

## References

- Related specs: **SPEC-121** (this spec), **SPEC-122** (linear-light
  resampling, same function, amends this record), SPEC-090 (honest size
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
