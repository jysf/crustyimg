---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-063
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-11
supersedes: null
superseded_by: null

affected_scope:
  - src/image/mod.rs
  - src/image/raw.rs
  - src/image/avif.rs
  - src/image/svg.rs
  - src/image/heic.rs
  - tests/fuzz_regressions.rs
  - tests/fixtures/fuzz/raw_preview/
  - docs/research/proj-009-fuzz-run.md

tags:
  - untrusted-input-hardening
  - decoders
  - memory
  - caps
  - pre-1.0-gate
---

# DEC-063: bound PEAK decode memory with a declared-pixel budget (64 Mpix)

## Decision

crustyimg enforces a **total-pixel cap on an image's DECLARED dimensions, checked
before the decode allocates**:

```
MAX_IMAGE_PIXELS = 64 Mpix (67_108_864 px, ≈ 8192×8192)
```

derived from a **peak-memory budget of 1 GiB** and a **4× amplification factor**
over the RGBA output:

```
budget 1 GiB ÷ (4 bytes/px RGBA × 4× amplification) = 67_108_864 px = 64 Mpix
```

It is enforced by one shared helper — `src/image/mod.rs::check_pixel_budget(w, h)`
(saturating `u32 × u32 → u64`, returning `ImageError::LimitsExceeded`, never a
panic) — called from **every** decode seam before the big allocation:

| path | where the declared dims come from | seam |
|------|-----------------------------------|------|
| generic (JPEG/PNG/…) | `ImageReader` header peek | `decode_with_limits` |
| RAW preview | the candidate JPEG's SOF | `raw::decode_jpeg_with_limits` |
| AVIF | container (`ispe`) metadata | `avif::check_caps` (+ dav1d's `frame_size_limit`) |
| SVG | the resolved render size | `svg::check_caps` |
| HEIC (opt-in) | the image handle | `heic::check_caps` |

`MAX_IMAGE_DIMENSION` (65 535, DEC-034) remains the per-side backstop;
`MAX_ALLOC_BYTES` (512 MiB) remains the single-allocation bound. This is the
**product** bound neither of them could be.

## Context

`image::Limits.max_alloc` bounds a **single** allocation — the crate decrements it
per `reserve()` and restores it on free — **not** the decoder's cumulative/peak
working set. And `image` 0.25's `Limits` has **no** total-pixel or peak field, so
per-side caps are all it offers: a 65535×65535 image is 4.3 **billion** pixels and
passes both of them.

SPEC-069's fuzz gate turned that structural gap into a confirmed, product-facing
memory DoS (**F-RAW-1**): a **782-byte** `.nef` whose embedded JPEG's SOF declares
**16384×9776** (160 Mpix) drove `crustyimg info` to a **1.93 GB** peak working set
and **exited 0** — a ~2470× amplification that passed every DEC-034 cap (each side
< 65 535; the ~480 MB RGB output < the 512 MB `max_alloc`). It is not RAW-specific:
the same declared dimensions balloon a plain `.jpg` decode the same way. In a CI or
build context — the "run it on untrusted files" thesis of PROJ-007 — a sub-kilobyte
file that costs 2 GB is a real denial of service.

Since the pixel library cannot express the bound, the bound has to be **ours**, and
it has to run on the **declared** dimensions, **before** the decoder allocates: file
size tells you nothing (`image`'s JPEG decoder pads truncated entropy data out to
the full declared frame — a sub-kilobyte file legitimately produces a full 64 Mpix
image).

## The numbers, and how they were chosen

- **Amplification factor (4×).** SPEC-069 measured ~1.9 GB peak against a ~480 MB
  RGB output for the JPEG path (≈4×; the decoder holds coefficient/component
  buffers alongside the output). Measured again in this build on a real 24 MP
  (6000×4000) photo: a full `convert … --format webp` (decode + encode) peaked at
  **280 MB** against a 96 MB RGBA output — ≈2.9×. So **4× is the conservative
  envelope**, not a guess.
- **Budget (1 GiB).** A bound a CI runner or a laptop can absorb per image without
  the process becoming the reason the build fails.
- **⇒ cap = 64 Mpix.**

## The tradeoff (stated, not hidden)

**What is rejected:** any image over 64 Mpix. Concretely, a **>64 MP medium-format
frame** (e.g. a 100 MP Fujifilm GFX / Phase One back, or a very large stitched
panorama) is refused with `LimitsExceeded` — a real capability loss for a real, if
uncommon, user. The 160 Mpix bomb and the 100 Mpix "bomb-shaped but legal" image
are indistinguishable from the header; the cap treats them alike.

**What is kept:** essentially all consumer and prosumer photography. 24 MP
(6000×4000) is 36% of the budget; 50 MP (8688×5792) is 75%; 8K video frames, phone
photos, screenshots, and web imagery are far under. Both sides of the boundary are
pinned by tests (`legitimate_large_image_within_budget_decodes`,
`declared_at_cap_pixels_pass_the_budget_check`).

**If that tradeoff is wrong for someone**, the dial is a single constant, and the
honest fix is a documented opt-in (a `--max-pixels` flag / config key) rather than
a silently higher default — filed as a follow-up, deliberately not built here (no
user has asked; adding an escape hatch to a security bound is a decision that
deserves its own spec).

## What it supersedes

The AVIF and SVG paths already checked `w * h * 4 > max_alloc`, i.e. an **implicit
128 Mpix** single-RGBA-buffer bound (512 MiB / 4). The 64 Mpix budget is tighter and
**uniform**, so it supersedes that implicit bound on those paths and extends the
same bound to the generic and RAW paths, which had **no** pre-decode dimension check
at all. AVIF additionally passes the budget to dav1d as its `frame_size_limit`, so
the decoder rejects an oversize AV1 frame at header-parse time rather than
allocating for it.

## What this does NOT close (no overclaim)

**F-AVIF-3 remains OPEN.** It is an over-allocation *inside `avif-parse`* during
**container parsing** (`read_avif_meta`), which happens **before** any frame
dimensions exist to check — a dimension peek cannot reach it, and closing it means
vendoring or patching `avif-parse` upstream. SPEC-070's `box_sizes_fit` guard
(SPEC-069) already blunts the specific inflated-`ftyp` shape, but the general
parse-stage class stands. It stays the separately-filed upstream item; this decision
closes the **decode-stage** peak only (F-RAW-1 + the general JPEG/PNG path).

## Alternatives Considered

- **Option A: raise/lower the per-side `MAX_IMAGE_DIMENSION`.**
  - Why rejected: a per-side cap cannot bound a product. Tightening it enough to
    bound memory (≈8192/side) would reject a legitimate 16000×2000 panorama
    (32 Mpix, cheap) while still admitting expensive squarer images. The product is
    the thing that costs memory, so the product is what to cap.

- **Option B: a custom global allocator / real peak-RSS accounting.**
  - Why rejected: a genuinely accurate peak bound means tracking every allocation
    process-wide (or an out-of-process watchdog). That is a large, invasive change
    with its own failure modes, for a bound that a pre-decode dimension estimate
    already delivers at ~zero cost. Explicitly out of scope (SPEC-070).

- **Option C: patch/vendor `image` to add a total-pixel limit upstream.**
  - Why rejected for now: right long-term home (and worth proposing upstream), but
    it does not close the gap on the paths we do NOT route through `image`'s limits
    logic (AVIF/SVG/HEIC), and it makes the fix hostage to an upstream release. The
    crustyimg-side check covers all five paths today and remains correct if `image`
    later grows the field.

- **Option D (chosen): a crustyimg-side declared-pixel budget at every decode seam.**
  - One constant, one helper, five call sites, no new dependency, and the check runs
    for a few hundred bytes of header parse.

## Consequences

- **Positive:** the F-RAW-1 bomb's peak RSS drops from **1.93 GB → 8.7 MB** (a
  222× reduction) and it now exits **1** (`LimitsExceeded`) with a message naming
  the budget, instead of exiting 0 after a 2 GB decode. Every decode entry
  (`info`/`convert`/`optimize`/`build`/`lint`) is bounded, including the ones that
  had no pre-decode dimension check. Because the reproducer is now rejected at the
  header, it graduates into the always-on `fuzz_corpus_never_panics` smoke, which
  SPEC-069 could not do (a ~2 GB-alloc input risked OOM-killing CI).
- **Negative:** >64 Mpix images are rejected (see the tradeoff above). The generic
  path pays one extra header parse per decode (microseconds; the bytes are already
  in memory) because `ImageReader::into_dimensions()` consumes its reader.
- **Neutral:** no new dependency; `Cargo.toml`/`Cargo.lock`/`deny.toml` unchanged.
  The cap lives in a single `const`, so revisiting the budget is one edit.

## Verification

- `check_pixel_budget` unit tests: at the cap → Ok, one pixel over → Err, the
  F-RAW-1 dims → Err, `u32::MAX × u32::MAX` → Err with no overflow panic.
- Rejected-before-decode tests on the generic and RAW paths, using sub-kilobyte
  files that declare 160 Mpix (they *cannot* hold the pixels, so the rejection can
  only be pre-decode).
- The real binary: `/usr/bin/time -l crustyimg info <reproducer>` → 8.7 MB peak,
  exit 1 (was 1.93 GB, exit 0). A real 24 MP photo still decodes to 6000×4000 and
  converts (280 MB peak).
- `tests/fuzz_regressions.rs::raw_pixel_bomb_is_limits_exceeded_not_multi_gb_decode`
  + the reproducer committed at `tests/fixtures/fuzz/raw_preview/pixel_bomb.nef`
  (sha256 `d4276ee78007720b294e1865d651f49bf4f9bd07f73e017fb1121a789276fbf9`).
