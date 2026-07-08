---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-055
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-08
supersedes: null
superseded_by: null

affected_scope:
  - src/image/raw.rs
  - src/image/mod.rs
  - src/source/mod.rs
  - fuzz/**

tags:
  - codecs
  - raw
  - pure-rust
  - untrusted-input
  - input-reach
---

# DEC-055: RAW input = Tier-1 largest-embedded-JPEG preview (format-agnostic byte scan, no new dependency)

## Decision

The default crustyimg build reads common camera **RAW** (`.nef .nrw .cr2 .cr3
.arw .srf .sr2 .dng .raf .rw2 .orf .pef .srw .rwl .raw`) by extracting the
**embedded full-res JPEG preview** — NOT by developing the raw sensor mosaic.
The mechanism is a **format-agnostic byte scan** with **no new dependency**:

- `src/image/raw.rs` scans the RAW bytes for JPEG start-of-image markers
  (`FF D8 FF`), prunes each match on a plausible following marker byte
  (`APPn`/`DQT`/`SOFn`/`COM`), decodes each surviving candidate *from that
  offset* through the **existing DEC-034-capped `image` JPEG decoder** (which
  tolerates trailing bytes after EOI), and keeps the **largest by pixel count**.
  The largest-decodable-JPEG is the full-res preview; the small thumbnail loses.
- This one path covers **TIFF-based RAW** (`.nef/.cr2/.arw/.dng/.rw2/.orf/…`),
  **Canon CR3** (ISOBMFF), and **Fujifilm RAF** — their previews are all
  baseline JPEGs found by the same scan, with **no IFD or ISOBMFF box parsing**.
- The extracted preview *is* a JPEG, so the canonical `Image` reports
  `source_format = ImageFormat::Jpeg` (the "materialized raster format"
  convention, like SVG→`Png`, DEC-054). `info x.nef` reports `jpeg` — an accepted
  wart; a faithful `SourceFormat` enum is the shared follow-up.

**Routing is by file extension** in `Image::load` (which has the `Path`), before
the generic byte decoder: `raw::is_raw_extension(path)` →
`image::raw_preview(&bytes)`. TIFF-based RAW starts with the TIFF magic
(`II*\0`/`MM\0*`), byte-indistinguishable from a plain `.tif`, so a content sniff
would risk mis-routing legitimate TIFFs — extension routing is required, not a
convenience. RAW extensions are added to `IMAGE_EXTENSIONS` so directory/glob
sources discover them. **RAW via stdin (`from_bytes`, no path) is a v1 non-goal**
(documented; a `--format raw` hint or a `Make`/`ftyp 'crx '` sniff is a later
option). The fuzz target still reaches the byte-level entry directly, so the
untrusted path is fuzzed regardless of routing.

**Hostile-input hardening** (RAW is untrusted binary): every candidate decode
routes through the same DEC-034 `Limits` as the generic path (a decompression-
bomb preview is rejected before allocation, never an uncapped
`load_from_memory`); the number of full decode attempts is bounded by
`MAX_PREVIEW_CANDIDATES = 16` (a file stuffed with fake SOIs cannot cause
unbounded work); the plausible-marker prune skips most false SOIs in compressed
data cheaply; a false SOI that slips the prune is a skipped failed decode, not a
whole-file error. An oversize-only preview → typed `LimitsExceeded`; no decodable
preview → typed `Decode`; never a panic. Rust slicing keeps the scan in bounds;
no maker-supplied offset is ever trusted (we scan, we do not seek by IFD offset).
A `cargo-fuzz` target (`fuzz/fuzz_targets/raw_preview.rs`) exercises the scan +
decode path.

**Tier-2 RAW development stays out** (demosaic / white-balance / color-matrix):
it needs LGPL `rawler` (DEC-018, `no-agpl-default-deps`, watchlist
`raw-camera-decode`) or a multi-month from-scratch effort — deliberately deferred.

This spec adds **NO new crate** (`Cargo.toml`/`deny.toml` unchanged; `just deny`
green), so `no-new-top-level-deps-without-decision` is not tripped — but DEC-055
records the capability, approach, covered extensions, and security bounds.

## Context

crustyimg could not read camera RAW at all. Full RAW *development* is Tier-2 —
LGPL or a large effort — and is out of scope. But nearly every RAW embeds a
full-res JPEG preview (the image the camera's screen shows), and extracting that
is permissive, pure-Rust, and patent-free. SPEC-061 (STAGE-018, PROJ-009 Wave 1,
the third default input after AVIF and SVG) adds it so the default binary turns a
`.nef`/`.cr2`/`.cr3`/… into a quick web derivative end to end
(`optimize`/`convert`/`info`/`resize`/batch).

The design-time probe (2026-07-08, the repo's pinned `image` =0.25.10)
**corrected two framing assumptions**: (1) the brief's "reuses ISOBMFF glue for
CR3" is unnecessary — CR3 (and RAF) previews are found by the same scan as
TIFF-RAW; (2) the watchlist's "parse the TIFF/EXIF IFDs (kamadak-exif)" is a
*harder* path than needed. The probe proved `image::load_from_memory` tolerates
trailing bytes after a JPEG's EOI, so decode-from-each-SOI + keep-largest
extracts the full preview with no container parsing. Net: **no new dependency**,
one module, one `load` branch — collapsing what looked like a multi-spec stage
into one. The build cycle re-confirmed the mechanism against the pinned version.

## Alternatives Considered

- **Option A: parse each vendor's container (TIFF IFDs / CR3 ISOBMFF boxes) to
  locate the preview by offset.**
  - Why rejected: far more code and per-vendor fragility (SubIFD layouts differ;
    CR3 needs box parsing; RAF is bespoke), and it means trusting maker-supplied
    offsets on untrusted input. The byte scan is format-agnostic, offset-free, and
    covers all three container families at once. `src/metadata/tiff.rs` (SPEC-045)
    only follows EXIF/GPS/IFD1 pointers — not the SubIFDs where full previews
    live — so it would not even reach the preview without new code.

- **Option B: Tier-2 RAW development via `rawler` (or a from-scratch demosaic).**
  - Why rejected: `rawler` is LGPL-2.1 (DEC-018, `no-agpl-default-deps` — copyleft
    off the default path), and a from-scratch demosaic + white-balance +
    color-matrix pipeline is a multi-month effort. The camera's own embedded
    preview is good enough for the "quick web derivative" use case and needs no
    codec. Tier-2 remains a possible opt-in `rawler` feature (watchlist).

- **Option C: content-sniff RAW (route by bytes, not extension).**
  - Why rejected: TIFF-based RAW is byte-identical to a plain `.tif` at the
    header, so a sniff would mis-route legitimate TIFFs into the preview scanner
    (or require a `Make`-tag heuristic that is itself IFD parsing). Extension
    routing is unambiguous and is what `Image::load` already has the `Path` for.

- **Option D (chosen): format-agnostic largest-embedded-JPEG scan, extension-
  routed, capped, no new dep.**
  - Why selected: pure-Rust, zero new deps, patent-free, covers TIFF-RAW + CR3 +
    RAF in one bounded path, reuses the DEC-034-capped `image` JPEG decoder, and
    is hostile-input-safe (bounded candidate decodes, cap-before-alloc, typed
    errors, fuzzed). Mirrors the AVIF (DEC-053) / SVG (DEC-054) default-input
    pattern.

## Consequences

- **Positive:** the default binary reads common RAW (optimize/convert/info/
  resize/batch) with **no new dependency, no C/system dep, no copyleft, no
  patents, no demosaic**; `just deny` unchanged and green; the lean
  `--no-default-features` build is unaffected (the RAW path is non-optional, not
  gated by `display`). One format-agnostic mechanism covers three container
  families.
- **Negative / costs:**
  - `info x.nef` reports `source_format = jpeg` (the preview *is* a JPEG; there is
    no `ImageFormat::Raw`). Consistent with SVG→`png`; a faithful `SourceFormat`
    enum is a shared follow-up.
  - Output quality is the **camera's embedded preview**, not a developed RAW — by
    design (Tier-1). A RAW with no baseline-JPEG preview (e.g. Sigma `.x3f`)
    yields the typed "no preview" error, not a crash.
  - RAW **via stdin** is unsupported in v1 (extension routing has no path on the
    `from_bytes` seam) — documented follow-up.
  - Metadata is **not** captured from RAW in v1: the container's EXIF is out of
    scope and threading the winning preview's own APP1 through the scan is a
    documented follow-up, so `Image::metadata()` is `None` for RAW (best-effort).
    Downstream auto-orient therefore has no RAW orientation to act on yet.
- **Neutral:** RAW is routed by extension, independent of any `image` feature;
  the metadata lane is untouched.

## Validation

Right if: default `cargo build` (and `--no-default-features`) load a `.nef`
fixture's 64×48 preview (not its 16×12 thumbnail) on all three CI OSes;
`optimize .nef -o .webp` / `convert .nef --format png` exit 0 with the preview's
dims; a directory source discovers `.nef`; an oversize-only preview is
`LimitsExceeded` and a no-preview RAW is `Decode` (never a panic); a file stuffed
with fake SOIs decodes at most `MAX_PREVIEW_CANDIDATES` times; `just deny` stays
green with **no new crate/license/advisory**; the fuzz target finds no panics.
Revisit when: (a) Tier-2 RAW **development** is wanted (opt-in `rawler` feature,
watchlist `raw-camera-decode`); (b) RAW **via stdin** / a `--format raw` hint is
wanted; (c) RAW-container **EXIF/orientation** passthrough or a faithful
`SourceFormat` enum is wanted (each a follow-up spec); (d) a real-camera corpus
surfaces a vendor whose preview the scan misses.

## References

- Related specs: SPEC-061 (this), SPEC-058/DEC-053 (AVIF decode — the default-input
  pattern mirrored here), SPEC-060/DEC-054 (SVG rasterize — the `source_format` =
  materialized-format precedent), SPEC-045/DEC-046 (`src/metadata/tiff.rs` IFD
  parser — deliberately NOT used; the scan sidesteps IFD walking).
- Related decisions: DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018
  (`no-agpl-default-deps` — Tier-2 `rawler` LGPL stays out), DEC-052 (HEIC
  feature-gated — the patent/licensing contrast).
- Constraints: `pure-rust-codecs-default`, `no-agpl-default-deps`,
  `untrusted-input-hardening`, `no-unwrap-on-recoverable-paths`,
  `every-public-fn-tested`, `clippy-fmt-clean`, `single-image-library`.
- Watchlist: `raw-camera-decode` (Tier-2 development, LGPL `rawler`).
