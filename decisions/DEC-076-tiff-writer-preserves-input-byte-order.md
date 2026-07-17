---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-076
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-16
supersedes: null
superseded_by: null

# Amends DEC-046's serialization rule; governs the container lane's TIFF writer.
affected_scope:
  - src/metadata/tiff.rs
  - src/metadata/tiff/**
  - src/metadata/mod.rs

tags:
  - metadata
  - exif
  - byte-order
  - correctness
  - testing-gap
---

# DEC-076: the in-house TIFF writer preserves the input's byte order

## Decision

The container lane's TIFF-IFD writer re-emits a block **in the byte order it was
parsed in** (`Tiff::byte_order`; a block created from scratch is little-endian),
instead of normalizing every block to little-endian as DEC-046 specified.

## Context

`meta clean --gps` — the **privacy** verb, documented as preserving "everything
else (orientation, copyright, ICC)" — silently corrupted the numeric EXIF tags of
every **big-endian** photo. `meta set` did the same. Reproduced on a real JPEG with
exiftool (SPEC-093):

```
baseline (MM):             Orientation 6      GPSLatitude 50.4957
after `meta clean --gps`:  Orientation 1536   (GPS correctly removed)
after `meta set --artist`: Orientation 1536   GPSLatitude 50.4843223958333
```

### The mechanism

`parse` stores each entry's value bytes **verbatim, in the input's byte order**
(`Entry::value` is deliberately opaque so unknown tags round-trip). `serialize`
then emitted a hardcoded `II` (little-endian) header — DEC-046's "normalize to
little-endian, matching what `little_exif` emitted" — while copying those
big-endian value bytes straight through. Nothing byte-swapped them. The result is
a block whose header lies about its own contents: every reader dutifully
re-interprets big-endian bytes as little-endian.

- **Orientation** (SHORT, inline): `0x00 0x06` read little-endian = `0x0600` =
  **1536** = `6 << 8`.
- **Thumbnail length** (LONG): 6,430 → **504,954,880**, leaving the IFD1
  thumbnail pointer dangling. A third symptom nobody had reported, found while
  fixing this and fixed by the same change.
- **GPS** (RATIONAL): the worst case, because it stays *plausible*.
  `50° 29' 44.52"` is stored `50/1, 29/1, 1113/25`. Byte-reversing a `u32` whose
  value fits in the low byte multiplies it by `2^24` — and a RATIONAL is a
  **ratio**, so that factor cancels: `50/1` → `838860800/16777216` = exactly 50.
  Degrees and minutes survive by pure arithmetic luck. `1113/25` does not
  (`1113` needs two bytes), so seconds drift `44.52"` → `3.56"`. The output is a
  well-formed coordinate ~1.3 km from the truth — far worse than an obvious error.

### Why it survived a green suite for a month

Two independent blind spots, both worth naming:

1. **ASCII tags are byte-order-immune, and ASCII was all we tested.** `meta set`
   writes Artist/Copyright/ImageDescription — all ASCII. Every fixture was seeded
   by calling `tiff::serialize` itself, which *could only emit little-endian*. So
   the suite had **no big-endian fixture at all** and **no assertion on a numeric
   tag's value** (`clean_gps_removes_only_gps` asserted Orientation `is_some()`,
   never `== 6`). The tests ran; they could not fail. A suite whose fixtures come
   from the code under test cannot observe that code misreading a byte order.
2. **Byte-identity proofs certified the defect.** SPEC-087/089 proved `meta`
   byte-identical to the *pre-move binary* — which was equally broken.
   **Identical to the old bytes ≠ correct bytes.** An oracle that shares the
   defect ratifies it.

### A refutation that was itself wrong — worth recording

SPEC-093's framing formed this exact hypothesis, tested it, and **refuted** it:
driving `-ExifByteOrder=MM` **and** `-ExifByteOrder=II` both produced `1536`, so
the bug was declared unconditional and the mechanism undiagnosed. The framing was
right about the code and wrong about its own experiment: **exiftool's
`-ExifByteOrder` only applies when EXIF is created from scratch; on a file that
already carries EXIF it is silently ignored.** The "II" arm was still an MM file.
Verified directly — `-ExifByteOrder=II` on an MM-EXIF JPEG leaves it MM.

The lesson generalizes past this bug: **a control that was never verified to have
applied is not a control.** The refutation had the shape of evidence (two arms,
same result) while testing one condition twice. When an experiment refutes a clean
hypothesis, confirm the independent variable actually moved before discarding it.

## Alternatives Considered

- **Option A: keep normalizing to little-endian, byte-swapping values on parse.**
  - What it is: decode every value into a canonical form per its TIFF type, then
    re-encode little-endian on write.
  - Why rejected: it forces the writer to *understand* every value it touches,
    which is exactly what the opaque `(tag, ty, count, value)` model avoids. Any
    type we model wrong — or don't model, since `type_size` deliberately treats
    unknown codes as byte-sized — gets silently mangled. It trades a bug we can
    fix for a permanent class of them, and it is strictly more code.

- **Option B: refuse big-endian input.**
  - What it is: error on `MM` blocks.
  - Why rejected: `MM` is spec-legal and common (Canon, Nikon, many phones). This
    turns silent corruption into a loud failure on ordinary photos — a different
    bug.

- **Option C (chosen): preserve the input's byte order.**
  - What it is: record the order at parse; write the header, tags, types, counts,
    and offsets in that order; keep passing values through verbatim.
  - Why selected: it makes the *existing* opaque-value model correct instead of
    working around it. Value bytes never need interpreting, so round-trip
    fidelity holds for **every** TIFF type — including types this module doesn't
    model and tags it has never heard of. It is also the smallest change: the
    header, four write sites, and a `bool` threaded through `put_ifd`.

## Consequences

- **Positive:** untargeted tags round-trip byte-identically in either order, for
  every type. Correctness no longer depends on the writer knowing what a value
  means. Fixes the reported Orientation/GPS corruption and the unreported
  thumbnail one.
- **Negative:** output is no longer uniformly little-endian, so `serialize`'s
  bytes now depend on the input — a byte-for-byte comparison against a
  little-endian expectation will differ for MM inputs. This is the point: the old
  uniformity *was* the bug. (Amends DEC-046, which chose normalization to match
  `little_exif`; that crate is gone, so the compatibility argument is moot.)
- **Neutral:** `minimal()` — the no-existing-EXIF path — stays little-endian, so
  freshly created blocks are unchanged.

## Validation

Graded with **exiftool** (a decoder we didn't write; our own reader shares the
writer's assumptions and cannot grade this):

| verb | input | Orientation | GPSLatitude |
|---|---|---|---|
| `meta clean --gps` | MM | **6** ✅ | removed ✅ |
| `meta set --artist` | MM | **6** ✅ | **50.4957** ✅ |
| `meta copy` | MM donor | **6** ✅ | **50.4957** ✅ |
| all three | II | **6** ✅ | **50.4957** ✅ |

Eight tests cover it (SHORT/LONG/RATIONAL/ASCII/UNDEFINED, sub-IFDs, IFD1
thumbnail, both orders), seeded by a builder (`src/metadata/tiff/fixture.rs`)
**deliberately independent of `serialize`** — it takes typed values and encodes
them itself, so a fixture cannot inherit the writer's idea of byte order. All
eight were **mutation-tested**: reverting the fix in place fails all eight, which
is the check that the coverage bites rather than merely exists.

`meta strip` (segment-level removal) and `meta copy` (segment-level graft) never
parse the TIFF block, so neither could exhibit this — **confirmed with fixtures
and exiftool, not assumed**; `copy_metadata_preserves_big_endian_numeric_tags`
correctly does *not* fail under the mutation.

**Revisit if:** a real-world reader rejects a preserved-`MM` block (none should —
TIFF 6.0 §2 requires readers to honor the header), or the container lane ever
needs to *rewrite* a numeric value, which would need per-type encoding in the
block's order.

## References

- Related specs: SPEC-093 (this fix), SPEC-045 (the in-house writer),
  SPEC-026/027 (where the bug entered), SPEC-087/089 (byte-identity proofs that
  were structurally blind to it)
- Related decisions: **DEC-046** (amended — its "normalize to little-endian"
  serialization rule is replaced by order preservation), DEC-003 (container lane),
  DEC-017 (orientation is load-bearing — a corrupted tag is a real rotation bug),
  DEC-030 (`meta copy` is JPEG-only)
- External docs: TIFF 6.0 §2 (byte order, value layout); EXIF 2.3 §4.6.5 (IFD1
  thumbnail); exiftool `-ExifByteOrder` ("has effect only when the EXIF is
  created from scratch")
