---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-016
  status: proposed
  priority: medium
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-10-28
shipped_at: null

value:
  thesis: >
    crustyimg's RAW support currently means "the camera's embedded JPEG
    preview." Behind an off-by-default `raw-develop` feature it comes to mean
    the sensor — real developed pixels, and metadata read from the container
    rather than from the preview — without changing one byte of the default
    build's output. The claim is narrow on purpose: one camera's files, decoded
    correctly and verifiably, beats a broad claim nothing checks.
  beneficiaries:
    - "Anyone handing crustyimg a RAW file and expecting the image rather than the camera's JPEG — today `info` reports the preview's dimensions as if they were the sensor's"
    - "The maintainer's own Leica Q2 Monochrom workflow, which is the dogfood and the reason the corpus exists"
    - "`irradiance`, which gets its first real consumer and therefore its first evidence that the library's shape is right"
    - "crustyimg's own honesty — DEC-055's RAW support is real but its reach is routinely overstated in docs written after it shipped"
  success_signals:
    - "`crustyimg web q2m.dng -o photo.avif` produces a publishable image developed from the sensor, not the preview"
    - "`crustyimg info q2m.dng` reports the sensor's dimensions, bit depth and orientation — not the preview's — and EXIF survives"
    - "The default build's RAW output is byte-identical to 0.8.x, proven rather than assumed"
    - "A camera `irradiance` cannot model still decodes via preview at exit 0, with the tier it used visible in `info`"
    - "A `DEVELOP_VERSION` bump produces a cache miss; an unrelated change does not"
  risks_to_thesis:
    - "⚠ `irradiance` supports ONE camera at 0.1.0 (Leica Q2 Monochrom, monochrome DNG, uncompressed). Every success signal above is about that one file, and nothing here proves the second camera will be cheap"
    - "⚠ The final `Cargo.toml` line cannot be committed until `irradiance` 0.1.0 is on crates.io — an optional `path` dependency with no `version` makes `cargo publish` fail for the WHOLE crate, not just the feature. This gates shipping, not building"
    - "⚠ Two decode paths for one input family means every RAW test matters twice, and the develop path's tests need a fixture the repo does not have today (a real Q2M DNG is ~95 MB uncompressed)"
    - "The silent preview fallback can mask a develop failure from someone who explicitly asked for development. `SourceContainer` makes it discoverable, not loud — if that is too quiet it needs a follow-on, and we will only find out from use"
    - "A compile-time-only gate may mean nobody ever enables it. That is the quiet failure mode: the feature works, ships, and is never exercised outside CI"
---

# PROJ-016: RAW Develop

## What This Project Is

The wave that makes crustyimg's RAW support mean the sensor.

Today a `.dng` or `.nef` is decoded by scanning the container for its largest
embedded JPEG preview (SPEC-061/DEC-055) — permissive, fast, and genuinely
useful for a quick web derivative, but it is the camera's interpretation at the
camera's resolution, and `info` reports the preview's properties as though they
were the file's. This project adds a second tier behind an off-by-default
`raw-develop` feature: hand the container to `irradiance`, get real developed
pixels and container-true metadata, and fall back to the preview whenever that
cannot be done.

The default build does not change. That is a success criterion, not a caveat.

## Why Now

**Because the blocker DEC-055 recorded has actually been removed.** DEC-055
deferred Tier 2 on the grounds that it "needs LGPL `rawler` … or a multi-month
from-scratch effort." The from-scratch effort was made — in a separate repo, as
a permissive general-purpose library rather than inside crustyimg — so the
dichotomy that argument rested on no longer holds. `irradiance` is pure Rust,
MIT/Apache, zero runtime dependencies, and verified bit-exact against an
independent implementation. DEC-055's cost note is amended in this change
rather than left to mislead the next person who reads it.

**Because the integration design is already reviewed.** irradiance's
PROJ-001/STAGE-004 carries the cross-repo plan, and crustyimg reviewed it on
2026-10-28 against this repo's actual HEAD — confirming the call chain, the
`Image` adapter seam, the decision to emit `Luma16` rather than an RGB
broadcast, `develop_version` in `compute_key` rather than the lockfile, and
that ICC and the `SourceFormat` enum stay out of scope. The open questions were
answered before any spec was written.

**Because `info` is currently wrong in a way nobody filed.** `docs/backlog.md`
records that RAW loses 100% of EXIF and that RAW orientation is never read, and
notes the measured DNG's embedded previews carry no APP1 at all — so threading
the preview's EXIF forward *cannot* work. The container's IFD0 is the only
source, and a DNG container parser is exactly what reads it. The capability and
the defect have the same fix.

**Because it was mis-filed as frontier work.** `docs/roadmap.md` listed "RAW
Tier-2" inside a 2.0+ row alongside AI super-res and face-aware crop. It is not
that kind of item: it is framed, it has an implementation nearing 0.1.0, and it
needs one feature flag. This change gives it its own wave.

## Success Criteria

- `crustyimg web q2m.dng -o photo.avif` exits 0 and produces a publishable
  image developed from the sensor plane
- `crustyimg info q2m.dng` reports the sensor's real dimensions, bit depth and
  orientation, with EXIF populated from the container's IFD0
- `--no-default-features` builds; the default build's RAW behaviour and output
  bytes are **unchanged**, asserted rather than assumed
- A RAW file `irradiance` cannot develop decodes via the preview at exit 0, and
  `info` says which tier produced the image
- The two tiers never share a cache entry, and a `DEVELOP_VERSION` bump
  invalidates
- No change to the released binary's dependency graph

## Scope

### In scope
- The `raw-develop` cargo feature and the optional `irradiance` dependency
- `Image::raw_develop(bytes)` — the single adapter where `irradiance`'s buffers
  become a `DynamicImage`
- The `decode_path` routing fork, and the fall-back-to-preview rule
- `SourceContainer::RawDeveloped` and its label
- `develop_version` in the build cache key, and `raw-develop` in
  `feature_signature()`
- Truthful RAW `info`: real dimensions and bit depth, an orientation field on
  `InfoReport`, the source container surfaced, EXIF from IFD0
- The DECs: DEC-107 (the two-tier rule), and the amendments to DEC-055 and
  DEC-089

### Explicitly out of scope
- **Colour RAW.** `irradiance` 0.1.0 is monochrome-only — no CFA, so no
  demosaic, white balance or colour matrix exists to call. That is irradiance's
  PROJ-002, and crustyimg needs no change when it lands beyond a version bump.
- **Any vendor container** — no NEF, RAF, CR2, ARW, NRW. DNG only, because that
  is what `irradiance` reads.
- **A runtime `--raw develop|preview` flag.** DEC-107 explains why the
  compile-time gate comes first and why a runtime flag can be added on top
  later without undoing it.
- **The `SourceFormat` enum.** Thrice-deferred; `ImageFormat::Tiff` plus the
  container label is enough here, and blocking RAW on it would be the fourth
  deferral.
- **ICC interpretation.** PROJ-012 owns ICC transforms (`qcms` only, per its
  own risk note on DEC-088).
- **Any change to the operation pipeline.** The adapter emits 16-bit and
  DEC-095 carries it through unchanged. A float working path is PROJ-015.
- **Publishing `irradiance`.** That happens in irradiance. This project
  *consumes* 0.1.0 and is gated on it.

## Stage Plan

- [ ] STAGE-056 (proposed) — RAW develop behind a feature, and truthful RAW metadata

**Count:** 0 shipped / 0 active / 1 pending

One stage, because the two specs in it are a capability and the truthfulness
that capability makes possible, and shipping the first without the second
leaves `info` lying about a file it can now read properly.

## Dependencies

### Depends on
- **External: `irradiance` 0.1.0 on crates.io.** ⚠ Hard gate for shipping. The
  adapter can be *built* against a local path dependency; the committed
  `Cargo.toml` cannot carry a `path`-only optional dependency without breaking
  `cargo publish` for the entire crate.
- **Hardware/corpus:** a Leica Q2 Monochrom DNG. Held by the maintainer; the
  repo has no fixture of this kind today and one is needed for the develop
  path's tests.
- **PROJ-009 / SPEC-061 + DEC-055** (shipped) — Tier 1, which stays the default
  and becomes the fallback
- **PROJ-010 / SPEC-115 + DEC-089** (shipped) — `SourceContainer`, which gains
  a variant
- **PROJ-007 / DEC-058** (shipped) — the build cache key this extends
- **DEC-095** (shipped) — ops preserve bit depth, so a 16-bit adapter output
  survives the pipeline

### Enables
- irradiance's own PROJ-001 success signal — *"crustyimg — its RAW claim
  becomes true rather than 'the camera's embedded JPEG'"*
- Colour RAW as a version bump rather than a project, once irradiance's
  PROJ-002 lands
- Closing `docs/backlog.md`'s RAW EXIF and RAW orientation defects, which have
  no other fix

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
