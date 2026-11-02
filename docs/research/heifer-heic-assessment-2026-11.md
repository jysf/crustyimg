# Heifer for crustyimg — HEIC decoder assessment (2026-11-02)

**Verdict: the best HEIC backend candidate so far, recorded on the watchlist; adopt nothing today.**
`heifer` is the first permissive pure-Rust HEIC decoder proven here on real iPhone photos. Every
HEVC image in every test file decodes **byte-identical to ffmpeg**. That includes the tiled photos
the July candidates could not decode. It is the obvious replacement for system libheif **inside the
opt-in `heic` feature** (DEC-056), and nothing else.

**What it does not change.** HEVC patents (DEC-052) attach to every decoder whatever its licence.
So HEIC stays out of the default build and out of **every distributed artifact**. That includes the
hosted browser demo and the ddphotos editor: a wasm build of heifer would remove the libheif
problem there, but publishing it would still distribute an HEVC decoder.

**Status.** This is an assessment, not a backlog. There is no stage work and no new DEC, because
nothing was adopted. The revisit triggers are in §6, recorded where tooling reads them:
`guidance/license-watchlist.yaml` (`heic-heif-decode`) and `guidance/questions.yaml`
(`heifer-heic-backend`).

**Method.** I checked crate facts against the crates.io API, the GitHub API and the downloaded
crate sources, not the README. The correctness evidence is our own differential, checked in at
[`docs/probes/heifer-heic-differential/`](../probes/heifer-heic-differential/) and re-run from that
location before this was written (Appendix).

**Shelf-life.** Numbers are stamped 2026-11-02, on an Apple-silicon Mac, heifer **0.3.20**.
heifer shipped 0.3.21 and 0.3.22 the same day, and has published 28 releases in 26 days. **Re-run the probe at the candidate version before citing any of it.**

---

## 1. What heifer is

| | |
|---|---|
| Repo | `HarpeLm/Heifer`, created 2026-10-07 |
| Licence | MIT OR Apache-2.0 (all five crates) |
| Crates | `heifer` (decode/encode facade, optional `image` integration), `heifer-isobmff` (container), `heifer-hevc-dec`, `heifer-hevc-enc`, `heifer-cli` |
| Dependencies | `thiserror` only, plus `image` behind a feature. No C, no build script |
| `unsafe` | `unsafe_code = "forbid"` in every crate's lints |
| MSRV | 1.85, below crustyimg's 1.90 floor |
| Activity | 93 commits, 91 by one author (account created 2026-08-23); 28 crates.io releases since 0.1.0 on 2026-10-08; CI, release and demo workflows green |
| Adoption | ~1,300 downloads, 1 reverse dependency, 3 stars |

It also encodes HEIC. crustyimg's posture is decode-only, and the `heifer` facade depends on
`heifer-hevc-enc` **unconditionally**: there is no feature to leave the encoder out (§4).

## 2. Evidence

**Corpus.** 8 files, none checked in:
- two real iPhone photos from the ddphotos testbed sample (iPhone 15 Pro with an HDR gain map, and
  iPhone 11 Pro), each a 4032×3024 grid of 48 tiles;
- the macOS `Mac Blue` and `Sonoma` wallpapers (6016×6016, 36 tiles; Sonoma holds two images);
- a macOS Portrait background (1920×1080, 12 tiles);
- a macOS Weather tile with alpha;
- crustyimg's two `tests/fixtures/heic` files.

### 2.1 Decoder correctness: raw YUV vs ffmpeg

Every `hvc1` item in each file (grid tiles, thumbnails, gain-map tiles, alpha planes) was extracted
with heifer's own container parser. Each was decoded to raw planar YUV by `heifer-hevc-dec` and by
ffmpeg, then compared with `cmp`.

| File | HEVC items | Byte-identical |
|---|---|---|
| iPhone 15 Pro (gain map) | 50 | 50 |
| iPhone 11 Pro | 49 | 49 |
| Mac Blue | 36 | 36 |
| Sonoma | 72 | 72 |
| Portrait background | 12 | 12 |
| Weather tile (alpha) | 2 | 2 |
| fixtures (2 files) | 2 | 2 |
| **Total** | **223** | **223** |

**Negative control.** The same first tile, decoded with heifer's loop filters off
(`--no-filters`), differs from ffmpeg. So the comparison can fail.

**Formats real phones rarely use.** x265 streams in 4:2:0, 4:2:2 and 4:4:4 at 8, 10 and 12 bits
(8 streams) are all byte-identical to ffmpeg. This matters because in July `rust_h265` silently
mis-decoded 4:4:4 to the wrong dimensions.

### 2.2 RGB output vs libheif 1.23.7

| File | heifer | libheif | Identical | Within ±1 | Max diff |
|---|---|---|---|---|---|
| iPhone 15 Pro | 0.10–0.36 s | 2.0 s | 82.5% | 97.3% | 21 |
| iPhone 11 Pro | 0.10 s | 2.0 s | 90.9% | 98.5% | 24 |
| Mac Blue | 0.31 s | 10.6 s | 75.3% | 97.2% | 20 |
| Sonoma | 0.36 s | 20.4 s | 80.3% | 99.0% | 68 |
| Portrait background | 0.03 s | 0.28 s | 84.4% | 94.5% | 71 |
| Weather tile (alpha) | 0.01 s | 0.06 s | 100% | 100% | 0 |

The comparison uses libheif's bilinear chroma upsampling. Its nearest-neighbour mode is further
from heifer (max diff up to 122).

The differences come from **chroma upsampling, not decoding**:
- the YUV is byte-identical (§2.1);
- a 4:4:4 file, which needs no upsampling, is pixel-identical to libheif (PSNR 120);
- the gap depends on which upsampling filter libheif uses: on the three files tested both ways,
  the maximum difference roughly halved when libheif switched from nearest-neighbour to bilinear.

This matters for the `heic` feature's existing tests. Swapping the backend would move pixel values
slightly, so golden hashes would need regenerating, and the change should be called out.

Speed is wall-clock time for decoding and writing a PNG, CLI against CLI, on the same machine
(10 performance cores). heifer decodes grid tiles on scoped threads. I did not check how many
threads `heif-dec` used, so treat the ratio as indicative.

### 2.3 Robustness

The iPhone 15 Pro file was corrupted 13 ways: truncated at 100 B, 2 KB, ⅛, ½ and end − 1 KB, plus 8
seeds of 300 random byte flips. Every case ended with a clean error (exit 1), for example
`HEVC: invalid bitstream: end_of_subset_one_bit must be 1`. There were no panics (exit 101) and no
hangs (30 s limit).

This is a smoke test, not fuzzing. heifer runs cargo-fuzz in its own CI. Before adoption, run
crustyimg's `fuzz/fuzz_targets/heic_decode.rs` against it, and use a debug profile so overflow
checks are on.

### 2.4 Not tested

- Android and non-Apple files: Samsung, Xiaomi and Sony are claimed upstream, but I had no samples.
- Real 10-bit photos: only the synthetic 10/12-bit streams above.
- Gain-map **output**: the gain-map tiles decode correctly, but HDR reconstruction was not checked.
- EXIF orientation and `irot`/`imir` on a rotated photo: both test photos were upright.
- wasm32. The facade sizes its tile threads with `available_parallelism()`, which falls back to 1
  when that fails. Even so, it calls `std::thread::scope`, so a wasm build must be **run**, not
  just compiled (see the "wasm32 compiles is not wasm32 runs" lesson).

## 3. Against the earlier candidates

| | rust_h265 (July) | oxideav-heif + -h265 (Oct) | **heifer (Nov)** |
|---|---|---|---|
| Licence | MIT/Apache | MIT | MIT/Apache |
| HEIF container | none (we would write it) | yes | yes, with gain maps, alpha, overlays |
| Real iPhone grid | 0 frames | decodes, 42–48 dB vs `sips` | **byte-exact vs ffmpeg** |
| 4:4:4 | silently wrong | claimed | byte-exact |
| Extra patent pools | none | H.264 and H.266 in default features | none |
| Maturity | 1 release | v0.0.x, rebuilt 2026-05 | 4 weeks, one author, 28 releases |

On the evidence available, heifer is the strongest candidate on every axis except age.

## 4. What adoption would look like

The shape is the one oxideav already pointed to: replace `libheif-rs` **inside the opt-in `heic`
feature**, and change nothing else.

**What it would buy:**
- no system libheif (`brew install libheif` / `apt-get install libheif-dev`), so no `pkg-config`
  and no `libheif-plugin-libde265` split;
- no LGPL notice;
- Windows support for the `heic` feature, which DEC-056 currently leaves unsupported;
- removal of a C decoder with a long CVE history from the feature's attack surface.

**What it would not buy.** Any distributed HEIC: no default binary, Homebrew bottle, hosted demo or
editor build.

**The encoder coupling.** The `heifer` facade owns the parts we would need: grid composition,
alpha, transforms and `decode_srgb`. It always compiles in `heifer-hevc-enc`. There are three ways
to handle that:
1. **Ask upstream for an `encode` cargo feature.** This is the cleanest option. It needs the
   maintainer's OK before anyone opens an issue.
2. Depend on the facade and never call the encoder. This is defensible only because `heic` builds
   are never distributed, and it still weakens the "decode-only" line in `docs/licensing.md`.
3. Use `heifer-isobmff` + `heifer-hevc-dec` directly and rewrite the facade's grid and colour glue
   ourselves. That is real work, and it duplicates the code that is already tested.

**Cost.** One spec, roughly the size of SPEC-062:
- swap the backend in `src/image/heic.rs`, keeping the decode caps and the typed error mapping;
- regenerate the golden hashes;
- point the existing fuzz target at the new backend;
- update `docs/licensing.md` and DEC-056 (a new DEC supersedes its libheif choice).

## 5. Risks

- **Bus factor.** One author, four weeks old. The usual mitigation is to pin an exact version and
  re-run this probe on every bump. Because the `heic` feature is opt-in and local, the damage if
  heifer is abandoned is limited: fall back to libheif.
- **Release churn.** Several releases a week. Pin `=0.3.x` and upgrade deliberately.
- **Provenance.** heifer describes itself as an independent implementation validated against ITU-T
  conformance streams, libheif's fuzz corpus and ffmpeg. I did not audit it for code copied from
  LGPL libde265. A spot comparison would be due diligence before adopting it.

## 6. Recommendation and pre-registered triggers

**Watch.** Revisit when **any** of these fire:

| # | Trigger | Check |
|---|---|---|
| H1 | **Demand**: someone asks for `heic` without installing libheif, or for `heic` on Windows | A feedback item or issue |
| H2 | **Forcing**: libheif-rs or system libheif breaks the `heic` CI job, or a libheif/libde265 CVE affects the decode path | CI / RUSTSEC / distro advisories |
| H3 | **Stability**: heifer ≥ 0.4 or 1.0, **and** an optional encoder feature, **and** 6 months without a breaking API change (earliest **2027-05-02**) | crates.io + CHANGELOG |

**Before adopting, whichever trigger fires:**
- re-run `docs/probes/heifer-heic-differential/run.sh` at the candidate version;
- add Android and real 10-bit photos, plus a rotated iPhone photo;
- run crustyimg's `heic_decode` fuzz target in a debug profile;
- write the DEC that supersedes DEC-056's libheif choice.

**What NOT to do:**
- put heifer in the default build or any distributed artifact (DEC-052);
- ship it in the hosted demo or editor;
- use its encoder;
- read this assessment as a reason to re-open the patent question. That stays "ask counsel".

## 7. Sources

- crates.io API (2026-11-02): `heifer` 0.3.20 (max 0.3.22), downloads and reverse dependencies.
- Crate sources for `heifer`, `heifer-isobmff`, `heifer-hevc-dec`, `heifer-hevc-enc` and
  `heifer-cli` 0.3.20, read from the cargo registry: `Cargo.toml` (dependencies, features,
  `rust-version`, lints), `src/lib.rs` (threading).
- GitHub API: `repos/HarpeLm/Heifer` metadata, contributors, commits and Actions runs; the README.
- crustyimg: DEC-052, DEC-056, `docs/licensing.md`, `guidance/license-watchlist.yaml`,
  `docs/research/heic-input-reach-spike.md`, `docs/research/oxideav-adoption-assessment-2026-10.md`.
- Black-box oracles: ffmpeg/ffprobe with libx265, libheif `heif-dec` 1.23.7, ImageMagick 7.1.2-34.

---

## Appendix — reproducing the probe

```sh
docs/probes/heifer-heic-differential/run.sh /tmp/heifer-probe \
  IMG_xxxx.heic other.heic "/System/Library/Desktop Pictures/Mac Blue.heic" ...
HEIFER_VERSION=0.3.22 docs/probes/heifer-heic-differential/run.sh /tmp/heifer-probe-22 ...
```

The first file should be a real multi-tile phone photo: it is also used for the negative control
and the corruption sweep. The script exits non-zero on any YUV mismatch, a failed control, or a
crash or hang on corrupt input. I re-ran it from the repo on 2026-11-02 against the corpus in §2,
starting from an empty work directory, and it printed `PASS`.
