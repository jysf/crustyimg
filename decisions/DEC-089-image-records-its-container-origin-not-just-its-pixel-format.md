---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-089                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.85
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

created_at: 2026-08-13
supersedes: null
superseded_by: null

affected_scope:
  - "src/image/**"
  - "src/cli/optimize.rs"
  - "src/analysis/decide.rs"

tags:
  - image-model
  - source-format
  - svg
  - heic
  - raw
  - avif
  - passthrough
---

# DEC-089: `Image` records the container's origin, not just the decoded pixel format

## Decision

`Image` gains a second fact alongside `source_format()`: `source_container()` (`SourceContainer`
— `Native | Svg | Heic | RawPreview`), set once, at decode time, by the three decoders that adopt a
raster stand-in label (SVG/HEIC → `Png`, RAW's extracted preview → `Jpeg`). Any code that needs to
know "are the raw bytes on disk actually a valid file of `source_format`?" asks this recorded fact,
never re-derives it from the bytes at guard time. `source_format()` itself is unchanged — it still
names the decoded pixels' provenance, and `info` still depends on that (DEC-055).

## Context

`optimize`'s auto-decide passthrough (`pick_winner → None`) ships the raw source bytes unchanged
whenever no re-encoded candidate beats them on size (SPEC-084). That reading is correct only when
the source file **is** a valid output of the format it claims. For SVG (`source_format() == Png`,
bytes are XML), HEIC (`Png`, bytes are ISOBMFF/HEIF), and a RAW preview (`Jpeg`, bytes are the whole
TIFF-family container), it is not — and the branch is easy to reach, not exotic: a tiny vector/icon
beats every raster re-encode of itself on bytes alone. SPEC-115 is the build that closes this; this
decision records the model-level fix it depends on.

The natural first instinct is to add a **sniff at the guard**: `::image::guess_format(&raw).ok() ==
Some(fmt)`, mirroring what SPEC-113's pinned-path guard (`ops::write_pixel_output`) already does.
That is deliberately NOT what this decision does, for reasons Alternatives details below.

## Alternatives Considered

- **Sniff at the guard (`guess_format`), matching SPEC-113's pinned path — rejected for the decide
  path.** `image` 0.25.10's AVIF magic match requires the ISOBMFF `ftyp` box's **major brand**
  literally `avif`; this crate's own `sniff::is_avif` also accepts a `mif1` major brand carrying
  `avif` in its *compatible* brands (and unit-tests exactly that shape,
  `is_avif_detects_compatible_brand`). On the **pinned** path a `guess_format` false negative there
  means "force a re-encode" — today's behaviour, conservative. On the **decide** path the identical
  false negative would turn a *correct* passthrough into a *worse*, lossy-over-lossy re-encode. Same
  expression, opposite blast radius on the two paths — which is why they cannot share one guard
  built on `guess_format`.

  **Verified during build, and it changes the practical stakes but not the choice:** `avif-parse`
  2.1.0 (this crate's real AVIF container parser) independently enforces the identical
  `major_brand == b"avif"` rule (`avif-parse-2.1.0/src/lib.rs:751-756`) — so a `mif1`-major,
  `avif`-compatible file never actually reaches a decoded `Image` in this crate at all; it fails at
  **load**, with a typed `Decode` error, before any guard runs. The specific AVIF divergence the
  design cycle predicted is therefore not reachable with this dependency version (SPEC-115's Build
  Completion records this as a deviation from the design's AC-6). That does not make the recorded-
  origin choice moot: `guess_format`-at-the-guard is still the wrong general shape — it is a second,
  independent opinion about the same bytes the decoder already committed to, asked at a different
  time with different code, and the SVG/HEIC/RAW-preview case proves the two opinions *can* diverge
  in a way that matters (a plain XML/ISOBMFF/TIFF-family container correctly sniffs as *not* the
  adopted label, so `guess_format` happens to work there — but only because those containers are
  unambiguous, not because sniffing-at-the-guard is sound in general).

- **A bare `source_format_is_adopted: bool` instead of an enum — rejected.** Call 4 (SPEC-115)
  requires the report to name the REAL container (`"svg"`/`"heic"`/`"raw"`), not just flag that
  `source_format` is untrustworthy. A bool would have to be paired with a second lookup keyed on
  something else to recover the name; the enum carries both facts in the one place they are set.

- **Change `Image::from_parts`'s signature to require the container — rejected.** `from_parts` is
  `pub` on a published crate with roughly a dozen call sites, mostly tests. An additive builder
  (`with_source_container`, defaulting to `Native`) keeps every existing call site correct by
  construction and costs nothing at the two internal sites (`decode_with_limits`'s SVG/HEIC arms,
  `raw_preview`) that need to set it explicitly.

## Consequences

- **Positive:** the passthrough guard (`optimize_decide_one`, both exits) becomes a fact lookup, not
  a heuristic — `source_container.is_native()`. It cannot be fooled by a container shape a future
  sniff doesn't recognize, because it never re-sniffs; it trusts the same decision the decoder
  already made.
- **Positive:** `info_raw_reports_jpeg_dims` (SPEC-061/DEC-055) stays green, unmodified — the proof
  this change did not bleed into `source_format()`'s existing meaning. `source_format()` still names
  the decoded pixels' provenance; `source_container()` is the new, separate fact.
- **Negative / accepted:** `optimize <svg-or-heic-or-raw-preview>` can now produce an output LARGER
  than the source (e.g. a 336 B SVG re-encoding to a 444 B WebP) — this is the fix working, not a
  regression: there was never a valid same-format passthrough available for these three families,
  only an invalid one that happened to be smaller.
- **Negative / accepted:** a degenerate (zero-area) SVG/HEIC/RAW-preview source now fails with a
  typed `CliError::Analysis` (exit 1) instead of silently passing through unrasterizable bytes under
  a raster label — there is no correct output to fall back to for that shape.
- **Neutral:** the AVIF `mif1`-major trap this decision was partly motivated by is currently
  unreachable through this crate's own decode path (see Alternatives) — the model change is
  general-purpose and correct regardless, but its most vivid justifying scenario is unproven in
  practice, only in principle. Revisit if `avif-parse` (or a future AVIF decoder swap) relaxes its
  major-brand check.

## Validation

- **Right if:** `optimize`/`web`/`apply --recipe web`/`build` never write source bytes that are not
  a valid file of the format they report, for SVG/HEIC/RAW-preview inputs, on every feature leg; a
  genuinely native-container passthrough (any raster format, including AVIF) still ships
  byte-identical to `main`; `info`'s RAW/HEIC/SVG reporting is provably untouched.
- **Revisit when:** a fourth adopted-label family is added (another decoder with no matching
  `::image::ImageFormat` variant) — it must set `SourceContainer` too, not lean on `Native`'s
  default; or if `avif-parse`/the AVIF decoder is swapped for one that accepts a non-`avif` major
  brand, making the AC-6 scenario this decision anticipated finally reachable and worth a live test.

## References

- SPEC-115 (this build); SPEC-113 (`pipeline_altered_source`, the pinned-path sibling guard this
  decision explicitly does NOT share a `guess_format` implementation with); SPEC-061/DEC-055 (the
  RAW preview's adopted `Jpeg` label `info` depends on); SPEC-060/DEC-054 (SVG's adopted `Png`
  label); DEC-052/DEC-056 (HEIC's adopted `Png` label, decode-only, off-by-default).
- `src/image/sniff.rs` — `is_avif` and its `is_avif_detects_compatible_brand` unit test, the
  evidence for the `mif1`/compatible-brand spelling being legal and reachable at the SNIFF layer.
- `avif-parse-2.1.0/src/lib.rs:751-756` — the independent major-brand enforcement found during
  build, verified directly against both `::image::guess_format` and `avif_parse::read_avif` on the
  same mutated bytes.
