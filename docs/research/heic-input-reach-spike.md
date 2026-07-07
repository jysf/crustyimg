# Spike: HEIC (and modern-format) input reach — findings

> Go/no-go feasibility spike for the input-reach wave, run 2026-07-07 and verified against a
> second-opinion review the same day. **Decision: Option A** — keep input-reach as a wave;
> permissive default path = **AVIF-decode + SVG + RAW embedded-preview**; **HEIC = feature-gated
> real decode only** (DEC-052). Do NOT ship HEVC decode in the default binary. Pairs with
> `decisions/DEC-052-*.md` and the `heic-heif-decode` entry in `guidance/license-watchlist.yaml`.

## The question

Can the input-reach wave lead with "point crustyimg at your iPhone photos and they just work" —
i.e. HEIC decode on the **default**, pure-Rust, zero-system-dep path — while honoring
`no-agpl-default-deps` and `pure-rust-codecs-default`? Short answer: **no** — but input-reach is
still a strong wave, just with an honest headline.

## Two independent walls

HEIC = HEIF/ISOBMFF container (easy, pure-Rust) + **HEVC/H.265 intra decode** (the hard part).
Two separate blockers each independently force HEIC out of the default binary:

### Wall 1 — Copyright/license
Corrected from the first-pass conclusion. It is **not** true that "no permissive pure-Rust HEVC
decoder exists." Accurate statement: **no *mature, HEIC-proven* permissive pure-Rust HEVC decoder
exists yet.** The landscape:

| Crate | License | What it is | HEIC-readiness |
|---|---|---|---|
| `imazen/heic` | **AGPL-3.0 / commercial** | Real decoder, 49/49 HEVC conformance, SIMD, `forbid(unsafe)` | Mature — but **AGPL-blocked** (`no-agpl-default-deps`) |
| `roticv/rust_h265` 0.1.0 | **MIT OR Apache-2.0** ✓(verified) | Real pixel decoder, Main/Main10 8/10-bit **4:2:0**, byte-exact vs FFmpeg, no deps | Immature: 4:2:0-only, no HDR-SEI, Annex-B in published 0.1.0 (repo adds hvcC), single-author/unaudited |
| `media-codec-h265` 0.1.1 (Zhou Wei) | **MIT OR Apache-2.0** | Parser + decoder, **explicit HVCC** support, used in 6 crates | Unproven on HEIC/chroma/conformance |
| `OxideAV/oxideav-h265` 0.0.8 | **MIT** | Byte-exact incl **4:2:2/4:4:4 + tiles**, Main-Still-Picture test, `oxideav-mp4` for ISOBMFF | Very active but v0.0.x; Fuzz CI failing |
| `scuffle-h265` | MIT/Apache | **SPS/NAL header parser only — NOT a codec** | n/a (front-end only) |
| `libheif-rs` → libheif | **LGPL-3.0-or-later** + C | Mature standard; decode = libde265 (LGPL), exclude x265 (GPL, encode) | The realistic opt-in `heic` backend; C system dep |

So the copyright wall is *solvable* pure-Rust (adopt/fork `rust_h265`/`oxideav`, ~4–8 person-weeks
+ a HEIF container parser; from-scratch would be ~24–40) — but none of these is Wave-1-ready, and
solving copyright does not solve Wall 2.

### Wall 2 — HEVC patents (independent of the software license)
HEVC/H.265 is covered by the **Access Advance** patent pool. A copyright license — AGPL,
commercial, MIT, *or* Apache — grants **zero** patent rights. This exposure attaches to **every**
decode path equally: libheif, `imazen/heic`, and any future permissive pure-Rust decoder alike.
**Therefore "once a mature permissive decoder appears we can ship HEIC by default" is false** — the
patent question persists and is a legal decision (counsel), not a code one. Every peer converges on
the same posture: Firefox ships no software HEVC decoder (OS/hardware only), ffmpeg pushes risk
downstream (FFmpegKit retired Jan 2025 over this), Fedora strips H.265, and
ImageMagick/libvips/`image-rs` treat HEIC as an optional off-by-default component. Our channels
(Homebrew + GitHub Releases) are US-hosted, without Fedora's France-hosted-repo mitigation.

## HEIC ≠ RAW on the preview fallback

Embedded-preview extraction rescues **RAW** (nearly every RAW embeds a *full-resolution* JPEG
preview — extractable with permissive TIFF/box parsing, no codec; see the `raw-camera-decode`
watchlist Tier-1). It does **NOT** rescue HEIC: an iPhone HEIC's embedded thumbnail is itself
**HEVC-coded at ~320×240** (and the full image is ~48 HEVC tiles + a `grid` item) — there is no
JPEG preview and nothing HEVC-free to display. So HEIC's only honest options are **feature-gated
real decode or nothing** — no "preview" middle path. RAW and HEIC therefore get *visibly different*
treatment in the same wave.

## Decision — Option A (confirmed)

- **Default build (no system deps):** modern-format input reach = **AVIF decode** (rav1d,
  BSD-2-Clause — AVIF/AV1 is patent-*free* via the AOMedia royalty-free grant) + **SVG** (resvg,
  MIT) + **RAW embedded-preview** extraction (permissive, no codec). AVIF-decode also feeds the
  shipped `optimize`/auto-format engine as a new candidate input — value beyond input reach alone.
- **HEIC = opt-in only:** feature-gated behind a `heic` cargo feature (libheif decode-only, system
  lib where present), **never** in a distributed default binary — for the AGPL wall AND the patent
  wall (DEC-052). Permissive pure-Rust decoders tracked on the watchlist; even when mature they
  stay patent-gated.
- **Do NOT block or reorder the wave** around the lost HEIC headline. AVIF-decode alone justifies
  keeping input-reach early.
- **The "watch it just work" marketing moment relocates** to the WASM demo-page wave (in-browser
  AVIF/SVG conversion, client-side, patent posture handled the same way).
- **Apps sidestep both walls:** the native macOS/iOS apps decode HEIC via the OS (ImageIO /
  PhotoKit, `objc2-image-io` — permissive), which is free, full-quality, and clears both the AGPL
  and patent problems (Apple holds the platform license). CLI-on-macOS could optionally use the
  same OS path; Windows WIC needs the paid HEVC extension (probe, never assume); Linux has nothing
  stock. This layering is recorded privately in `business/` (apps) and here for the CLI.

## Positioning language (use verbatim-ish for HEIC)

> "HEIC decoding is available in the `heic` build where a system HEVC codec is present. We don't
> ship HEVC in the default binary: the mature pure-Rust decoders are AGPL, and HEVC carries
> third-party patent exposure regardless of code license. AVIF, SVG, and camera RAW work in the
> default build with no system dependencies."

## Security note (whichever decode path)

HEIC decode is untrusted-input binary parsing (BLASTPASS CVE-2023-41064, CVE-2025-43300 —
metadata-vs-payload dimension mismatch — and the libde265/libhevc CVE lineage). Any in-tool decoder
(pure-Rust or libheif) needs: `#![forbid(unsafe_code)]` where possible, no-panic parsing, checked
arithmetic, strict dimension/allocation caps (model `image::Limits`), **container-vs-SPS dimension
cross-checks**, bounded box-nesting/item-count/`iref`-graph (cycle detection for grids), and
**mandatory `cargo-fuzz`/OSS-Fuzz** on both the container and the bitstream.

## Watchlist / DEC updates from this spike
- `guidance/license-watchlist.yaml` → `heic-heif-decode`: reframed "impossible" → "immature,
  revisit," naming `rust_h265` + `media-codec-h265` (+ `oxideav-h265`) with the three trigger
  conditions and the separate patent gate; libheif corrected to LGPL-3.0.
- `decisions/DEC-052`: HEIC decode feature-gated, never default — rationale cites both walls.
