---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-056                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-016                      # parent project
repo:
  id: crustyimg

created_at: 2026-10-28
shipped_at: null

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    The whole of it. This is PROJ-016's only stage: the feature and adapter that
    make RAW mean the sensor, and the metadata truthfulness that makes `info`
    stop reporting the preview's properties as the file's.
  delivers:
    - "An off-by-default `raw-develop` feature that develops a DNG's sensor plane into real pixels"
    - "A preview fallback, so a RAW file never stops working because development failed"
    - "Truthful RAW `info`: real dimensions, bit depth and orientation, with EXIF from the container's IFD0"
    - "Two tiers that never share a cache entry"
  explicitly_does_not:
    - "Change the default build's RAW behaviour or output bytes"
    - "Add colour RAW — irradiance 0.1.0 is monochrome-only, and that is its PROJ-002"
    - "Add a runtime develop/preview switch (DEC-107 says why the compile-time gate comes first)"
    - "Touch the operation pipeline — the adapter emits 16-bit and DEC-095 carries it"
---

# STAGE-056: RAW develop behind a feature, and truthful RAW metadata

## What This Stage Is

When this stage ships, a build compiled with `--features raw-develop` turns a
Leica Q2 Monochrom DNG into an image developed from its sensor plane, reports
that file's real properties through `info`, and records which of the two decode
tiers produced the pixels. A build without the feature behaves exactly as
0.8.x does.

Two specs: the capability, then the truthfulness it makes possible.

## Why Now

**Because the design is reviewed and the open questions are answered.**
irradiance's PROJ-001/STAGE-004 holds the cross-repo plan; crustyimg reviewed
it on 2026-10-28 against this repo's HEAD and settled the six questions it
raised — `Luma16` rather than an RGB broadcast, `develop_version` in
`compute_key` rather than the lockfile, EXIF via irradiance's existing public
API rather than a new typed helper, `ImageFormat::Tiff` plus a container label
rather than the deferred `SourceFormat` enum, a new project rather than
PROJ-014, and colour plus the float buffer staying out. Nothing here is
waiting on a decision.

**Because `info` is wrong today and this is the only thing that fixes it.**
`docs/backlog.md` records that RAW loses 100% of EXIF and that RAW orientation
is never read — and that the measured DNG's embedded previews carry **no APP1
at all**, so threading the preview's EXIF forward cannot work even in
principle. The container's IFD0 is the only source. A DNG container parser is
what reads it, and this stage is where one arrives.

## Success Criteria

- `crustyimg web q2m.dng -o photo.avif` exits 0 with a publishable
  sensor-developed image
- `crustyimg info q2m.dng` reports the sensor's dimensions, bit depth and
  orientation, and non-empty EXIF
- The default build's RAW output is **byte-identical** to 0.8.x — asserted,
  with the assertion driven on a real RAW fixture
- A RAW that `irradiance` refuses decodes via preview at exit 0, and `info`
  names the tier that ran
- `--no-default-features` builds; `just wasm-check` passes (develop is
  native-only and must not reach the engine's wasm surface)
- A `DEVELOP_VERSION` bump invalidates the cache entry; an unrelated change
  does not
- `cargo publish --dry-run` succeeds with the feature's dependency committed

## Scope

### In scope
- The items in the Spec Backlog below

### Explicitly out of scope
- Colour RAW, vendor containers, a runtime tier switch, the `SourceFormat`
  enum, ICC interpretation, any operation-pipeline change — see the project
  brief, which carries the reason for each
- Publishing `irradiance`. That happens in irradiance; this stage consumes it.

## Spec Backlog

Ordered. The second depends on the first.

- [ ] (not yet written) — **[M] The `raw-develop` feature, the adapter, and the
  two tiers.** The optional `irradiance` dependency behind an off-by-default,
  native-only feature; `Image::raw_develop(bytes)` beside `raw_preview`,
  running irradiance's chain and emitting `DynamicImage::ImageLuma16`; the
  `decode_path` fork with fall-back-to-preview on any `irradiance::Error`;
  `SourceContainer::RawDeveloped` and its label; `raw-develop` registered in
  `cache::feature_signature()`; `irradiance::DEVELOP_VERSION` folded into
  `compute_key` as its own field. Governed by DEC-107.
- [ ] (not yet written) — **[M] Truthful RAW metadata.** Real `width`/`height`
  and bit depth from `develop::output_dimensions`; an orientation field on
  `InfoReport` (there is none today) surfaced in `info`; `source_container`
  surfaced in `info` as `optimize` already does; `MetadataBundle` EXIF
  populated from the container's IFD0 via irradiance's existing public API.

**Count:** 0 shipped / 0 active / 2 pending

Spec ids are assigned at `just new-spec`, not reserved here. At framing the next
free id was **SPEC-139**, but another session is working SPEC-137/138 on the
0.8.1 line — re-derive rather than trusting that number.

## Design Notes

**Read DEC-107 first.** It settles which tier is the default, what happens when
development fails, why absence of the feature is silent rather than an error
(unlike `heic`), and why the cache work cannot be deferred.

**The one line that cannot be forgotten.** `cache::feature_signature()` is a
**hand-maintained allowlist** — it enumerates `avif`, `heic`, `webp-lossy` with
`cfg!` checks, and its own doc comment says *"Omitting one that does affect
bytes serves a stale artifact."* The two tiers produce different pixels from
byte-identical input, so without the `raw-develop` entry a cache populated by a
default build serves that preview to a develop-enabled build. That is a silent
wrong-image bug. It is one line, and it belongs in the first spec, not a
follow-up.

**Verify `L16` survives the sinks before building on it.** The adapter emits
single-channel 16-bit because irradiance is monochrome-only and an RGB
broadcast would triple the footprint of a 47 MP plane while adding nothing. PNG
and grayscale JPEG handle `L16`; **AVIF and WebP are unverified**. DEC-095 added
an `eight_bit_downgrade_warning` to `sink::encode_to_bytes_with` for exactly
this class of surprise, so check whether either silently widens or downgrades,
and record the finding in the spec rather than discovering it mid-build.

**The crates.io gate is a shipping gate, not a building gate.** Build the
adapter against a bare `path` dependency; the committed manifest needs
`irradiance = { version = "0.1", path = "../irradiance", optional = true }`,
and the `version` requirement means irradiance 0.1.0 must actually be published
or `cargo publish` fails for the whole crate — optional dependencies are not
exempt. Prefer finishing after the publish over committing a manifest that
cannot ship.

**Every citation in the handed plan is a pointer, not a fact.** The integration
plan was written against HEAD `139b381` and its own header says to re-verify.
Spot-checked on 2026-10-28: `decode_path:291`, `raw_preview:670`,
`from_parts:389`, `with_source_container:411`, `feature_signature:217`,
`compute_key:260`, `run_info:217`, `source_container_label` in
`cli/optimize.rs:1082`, `lock.rs`'s `deny_unknown_fields:131` — all correct
then. One was wrong: there is **no `ImageInfo` struct**; it is `InfoReport` at
`src/cli/report.rs:22`. Re-read all of them at build; `main` has moved several
times since.

**A fixture is needed and the repo has none.** The develop path cannot be
tested without a real Q2M DNG, and an uncompressed one is ~95 MB — too large to
commit. Settle at design how the test gets its input: a generated DNG via
`dnglab makedng` (analytically known answers, no camera needed), a cropped real
file, or a test gated on a corpus path. This is the most likely thing to stall
the build.

## Dependencies

### Depends on
- **External: `irradiance` 0.1.0 on crates.io** — hard gate for shipping
- **PROJ-009 / SPEC-061 / DEC-055** (shipped) — Tier 1, now also the fallback
- **PROJ-010 / SPEC-115 / DEC-089** (shipped) — `SourceContainer`
- **PROJ-007 / DEC-058** (shipped) — the cache key
- **DEC-095** (shipped) — bit-depth preservation, which carries the adapter's
  16-bit output through the pipeline unchanged

### Enables
- Colour RAW as a dependency version bump once irradiance's PROJ-002 ships
- Closing `docs/backlog.md`'s RAW EXIF and RAW orientation defects

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
