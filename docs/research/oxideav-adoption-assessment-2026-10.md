# OxideAV for crustyimg — adoption assessment (2026-10-28)

**Verdict: watch and revisit. Adopt nothing today; no dependency or code change.** The pre-registered
triggers are in §7, recorded where tooling reads them (`guidance/license-watchlist.yaml`,
`guidance/questions.yaml`).

**Verdict, long form.** OxideAV's AV1/AVIF stack is far better than the July watchlist entry
recorded. On a 37-file corpus it decodes **byte-identical** to crustyimg's current decoder
(`re_rav1d`), and to libaom where `re_rav1d` cannot decode. It **does** run on
`wasm32-unknown-unknown`, the one thing `re_rav1d` cannot do, but only after a one-line upstream
fix: it panics at the first decode as published. So the technical question the brief asked is
answered *yes, nearly*. The reasons not to adopt today are elsewhere:

1. **The capability is not needed yet.** In the demo, the browser already decodes AVIF for free
   (DEC-065, `createImageBitmap`). The real beneficiaries are Node/edge users of `crustyimg-wasm`
   and an in-module AVIF quality search, and neither has a demand signal.
2. **The decoder is five months old.** `oxideav-av1` was deleted and rebuilt from scratch on
   2026-05-20 under its own clean-room policy. It has one committer, and every 0.0.x release is
   semver-breaking.
3. **Natively it is 2–3× slower than `re_rav1d`** and raises the MSRV floor 1.90 → 1.95, and it
   buys nothing native users lack except grid AVIF.

On HEIC, `oxideav-heif` decodes real HEVC pixels in pure Rust under MIT. That clears the
**licence** and **system-library** axes of crustyimg's HEIC problem. It does nothing about the
**patent** axis, which is the reason HEIC is never distributed (DEC-052), so nothing changes.

**Status.** This is an assessment, not a backlog. Nothing here is committed work. No stage backlog
gets a `- [ ]`, because there is nothing to build. Per AGENTS.md that makes it a recorded
proposal-with-triggers, not a decision. No new DEC: nothing was adopted, and DEC-053 / DEC-065
already name OxideAV and `createImageBitmap` as their revisit conditions.

**Method.** I re-checked every claim in the brief, and every OxideAV README claim this assessment
relies on, against the crates.io API, the downloaded crate tarballs, the GitHub API, and probes
that were compiled and run. The probes are checked in at
[`docs/probes/oxideav-avif-differential/`](../probes/oxideav-avif-differential/) and reproduced
from that location before this was written. I treated web content as data and did not take
README claims on trust: the only correctness evidence this document relies on is our own
differential.

**Shelf-life.** Every number is stamped 2026-10-28, on an Apple-silicon Mac, `rustc` 1.98/1.99.
OxideAV ships several releases a month. **Re-run the probe before citing any of it** (Appendix).

---

## 0. Findings that changed the answer

| # | Claim (brief or July watchlist) | What the repos show (2026-10-28) |
|---|---|---|
| F1 | Watchlist (2026-07-07): `oxideav-av1` decodes **intra only**, 10/13 conformance; `oxideav-avif`'s AV1 wiring is stubbed | `oxideav-av1` 0.1.21 + `oxideav-avif` 0.0.16 decode end to end. **36/36 non-grid files byte-exact vs `re_rav1d`** over raw planes, **grid byte-exact vs libaom** (§1.2). Stale facts are now wrong in both directions. |
| F2 | `oxideav-av1`'s crates.io description: "orphan-rebuild scaffold pending clean-room re-implementation" | The description is stale and the code is not a scaffold. But the word is literal: the crate **was reset to a scaffold on 2026-05-20**. Its CHANGELOG records that the prior code's provenance "could not be defended against the workspace clean-room rule", and no old branch was kept (§6). |
| F3 | Brief: compiles to wasm32 cleanly would restore AVIF decode in the WASM build | It **compiles** cleanly but **panics on first decode**: unconditional `std::time::Instant::now()` stage timing in the AV1 frame driver. With that one call site stubbed, all 4 test files decode byte-identical to native inside a wasm VM (§1.3). *Compiles* is not *works*. |
| F4 | Brief: a wasm AVIF decoder would close the WASM build's gap | The demo's gap is **already closed** by the browser (DEC-065: `createImageBitmap`, wired in `demo/worker.js`). The gap that remains is in **Node** (DEC-067: the `--target web` package runs in Node, which has no AVIF decoder) and in-module AVIF quality search (DEC-065's alternatives table). |
| F5 | `oxideav-heif` might be container-only | It decodes HEVC pixels, through `oxideav-h265`. Its **default** features also pull `oxideav-h264` and `oxideav-h266`, which are AVC, HEVC and VVC (three patent-pooled codecs). `oxideav-avif` opts out of them (`default-features = false`); verified with `cargo tree`. |
| F6 | Sweep: OpenEXR / HDR / QOI / DDS / TGA might fill gaps | `image` 0.25.10 already has `exr`, `hdr`, `qoi`, `dds`, `tga` features. crustyimg **excluded them on purpose** as unfuzzed decode surface (`docs/lab-plan-2026-08.md` F2). There is no gap to fill. |

---

## 1. Case 1 — AVIF decode on wasm32 (`oxideav-avif` + `oxideav-av1`)

### 1.1 What crustyimg has today

- Native: `re_rav1d` =0.1.3 (BSD-2, no-asm) + `avif-parse` =2.1.0 (MPL-2.0, `deny.toml` exception)
  + in-house YUV→RGB glue (DEC-053). Grid/tiled AVIF is **rejected** (avif-parse:
  "Grid-based AVIF collage is not supported").
- wasm32: no AVIF decode. `re_rav1d` does not compile (DEC-064). **Control reproduced 2026-10-28:**
  40 errors, `off_t` / `EAGAIN` / `EINVAL` / `EIO` / `ENOENT` / `ENOMEM` missing from `libc`, plus
  `E0080` const-eval. The demo hands `.avif` input to the browser (DEC-065).

### 1.2 Correctness — native differential (the evidence everything else rests on)

The probe (`src/bin/cmp.rs`) decodes each file twice. One decode is `oxideav_avif::decode`. The
other is `avif-parse` → `re_rav1d` with crustyimg's exact pins and features. The probe compares the
**raw Y/U/V/alpha planes byte for byte**, before any colour conversion, so a difference cannot hide
in YUV→RGB rounding.

The corpus is 37 files, built by `make-corpus.sh` from crustyimg's own fixtures (no personal
images):

- libaom (`avifenc` 1.4.2 / aom 3.14.1) across bit depth 8/10/12 × chroma 4:0:0/4:2:0/4:2:2/4:4:4
- q20 / q60 / q95, lossless, speed 0
- 8- and 10-bit alpha, 2×2 tiles, a 2×2 grid
- odd 333×217, limited range, BT.2020 10-bit, a denoised film-grain encode
- 12 MP 4:0:0, and 12 MP colour 4:2:0 at 8 and 10 bit
- 4 files from **crustyimg's own `ravif` encoder** (0.7.1), and the 3 AVIFs in `tests/fixtures/avif/`

| Result | Files |
|---|---|
| Byte-exact vs `re_rav1d` on every plane | **36 / 36** non-grid files |
| Grid file (`re_rav1d` path rejects it) | oxideav decodes it, and its **Y plane is byte-exact vs libaom** (`avifdec -c aom`, 3,150,000 B) |
| Spot check of the reference itself vs libaom | `gps_q70` and `fuji_d10_420_q60` Y planes byte-exact vs libaom too |
| **Negative control** (`--mutate`: flip one byte of oxideav's Y plane) | Reported as `1 bytes differ`, so the comparison can fail |

libaom is the AV1 reference decoder and shares no code with dav1d (`re_rav1d`'s ancestor), so
agreement with both is two independent oracles.

### 1.3 wasm32 — compiled **and run**

`wasm/` is a `cdylib` + `wasm-bindgen-test`, pinned to crustyimg's versions (`wasm-bindgen`
=0.2.126, `wasm-bindgen-test` =0.3.76) and run by `wasm-bindgen-test-runner` in Node. It embeds 4
AVIFs: 8-bit 4:2:0, 10-bit + alpha, the 2100×1500 grid, and a crustyimg `ravif` encode. It asserts
geometry plus an FNV-1a hash of all decoded planes, against values from the native decode, which
§1.2 shows is byte-exact.

| Build | Result |
|---|---|
| `cargo build --target wasm32-unknown-unknown` (release) | **compiles clean**: no `unsafe`, no `libc`, no `build.rs` in `oxideav-av1`/`-avif` |
| `cargo test` in the wasm VM, **as published** | **both tests FAIL**: `panicked at library/std/src/sys/time/unsupported.rs` |
| Same, with `patch-av1.sh` stubbing `Instant` at its one import (`src/decoder/frame_driver.rs:60`; 16 call sites, all stage timing) | **both pass**: 4/4 files byte-identical to native, including the grid and 10-bit + alpha; the negative-control test passes |

Threads are not a problem on wasm: decode is serial by default (`DecodeOptions::threads`: "`None`
or `Some(1)` = serial, the default"), matching crustyimg's single-thread policy (DEC-077).

**Size.** A decode-only `cdylib` (`decode_rgba`), built with crustyimg's wasm profile (opt `s`, fat
LTO, 1 codegen unit, strip, no `wasm-opt`), measures **1,077,669 B raw / 418,548 B gzip -9 /
337,211 B brotli -q 11**. That is a standalone figure and an **upper bound** on the increment: it
was not measured inside crustyimg's bundle, where std/alloc are already paid for. Against the
shipped 1,394,313 B brotli bundle (DEC-066) it is about **+24%**, roughly what `rav1e` cost
(+345,664 B, DEC-065).

### 1.4 Performance (native, both decoders serial, release, same machine)

| File | oxideav | `re_rav1d` (no-asm) | Ratio |
|---|---|---|---|
| 4032×3024 colour 4:2:0 8-bit (3 runs) | 483–485 ms | 183–187 ms | **2.6×** |
| 4032×3024 colour 4:2:0 10-bit (3 runs) | 481–484 ms | 181–183 ms | **2.6×** |
| 2100×1500 (tiles / q70 / ravif) | 43 / 39 / 86 ms | 26 / 8 / 38 ms | 1.7–4.8× |
| 448² across the bit-depth/chroma matrix | 4–32 ms | 2–13 ms | 0.9–2.9× |

On wasm, the comparison that matters is against **no decoder at all**, so ~0.5 s for a 12 MP still
is acceptable. Natively, it is a regression on every file.

### 1.5 Robustness

- **crustyimg's 5 hostile / fuzz-regression AVIFs** (`tests/fixtures/hostile/`,
  `tests/fixtures/fuzz/avif_decode/`) all return typed `invalid data` errors. That includes the
  3 GB `ftyp` box-size bomb crustyimg had to guard by hand in `box_sizes_fit`.
- **Mutation sweep** (`src/bin/fuzzlite.rs`): 5 bases × 300 mutants (one in three truncated, the
  rest with 1–8 random byte flips), giving **1,500 decodes, 0 panics**, under `catch_unwind`. An
  abort would have killed the process; it exited 0.
- Upstream runs nightly `Fuzz` workflows on all three crates, green 2026-10-23 → 10-27. Built-in
  caps are 16384 px per axis, 2²⁸ px and 1 GiB input, and they map onto DEC-034's limits.
- **Not covered:** a debug-profile (overflow-checks) sweep, real-world camera/phone AVIFs
  (Ultra HDR gain maps, `tmap`), animated `avis`, and ≥16384 px. Do these before any adoption.

### 1.6 What it would buy, and for whom

| Beneficiary | Today | With oxideav on wasm |
|---|---|---|
| Demo page, `.avif` input | Browser decodes it (`createImageBitmap`), **free** | Duplicates the browser for about +24% bundle. **No gain** (DEC-065's own reasoning still holds) |
| `crustyimg-wasm` in **Node / edge workers**, `.avif` input | `CodecUnavailableOnTarget`, with no host fallback | Works. **Real gain, but no demand signal yet** |
| AVIF output **quality search** (`--target`/`--ssim`) inside the module | Impossible: no decoder to score candidates (DEC-065 table). The demo scores in the browser instead (SPEC-080) | Possible. **Real gain, but no spec wants it** |
| Native CLI | `re_rav1d` | 2.6× slower; adds **grid AVIF**; retires the `avif-parse` MPL and `to_method` CC0 `deny.toml` exceptions; raises MSRV from 1.90 to 1.95. **Net negative today**, except grid |

---

## 2. Case 2 — HEIF / HEIC (`oxideav-heif` + `oxideav-h265`)

**What it does.** `oxideav-heif` 0.0.13 is the HEIF container. With its default `registry`
feature it decodes HEVC items through `oxideav-h265` 0.0.16 (MIT, pure Rust). Tested on
**Apple-encoded** HEIC made by macOS `sips`, the same encoder family as iPhones:

| File | Structure | oxideav | vs Apple's decoder (`sips` → PNG), PSNR R / G / B, max \|Δ\| |
|---|---|---|---|
| 4032×3024 | grid of `hvc1` tiles, `nclx` colour | **decodes**, 1.0 s | 43.3 / 47.9 / 42.1 dB, 16 / 9 / 19 |
| 448×448 | single item | decodes, 6 ms | 54.1 / 57.6 / 49.2 dB, 6 / 2 / 10 |

These PSNRs say real HEVC reconstruction: a broken decode shows blocking well under 30 dB. They
are **not** byte-exact evidence. `sips` colour-manages its output and I could not get a second
oracle: the local Homebrew `libheif` 1.23.1 fails to load (it links a `libx265.216.dylib` that
isn't installed), and I did not change the maintainer's Homebrew setup. The upstream README claims
"every lossy HEIC layout byte-exact"; that is unverified here.

**What it solves:**
- The **copyright-licence** axis: MIT, not the AGPL `heic` crate or LGPL libheif.
- The **system-library** axis: pure Rust, no `pkg-config`, no Debian
  `libheif-plugin-libde265` split.

**What it does not solve:**
- The **HEVC patent** axis (Access Advance). As `docs/licensing.md` already says, a copyright
  licence grants no patent rights, and that exposure attaches to every decode path equally. HEIC
  therefore stays off the default build and out of every distributed artifact. **DEC-052 is
  unchanged.**
- The **maturity** half of the watchlist's trigger (b): "hvcC + grid/tiled + HDR HEIC support
  PROVEN on a real iPhone corpus". Apple-encoder grid output now decodes. HDR, gain maps and a
  real iPhone corpus remain unproven.
- And it adds a new caution: **default features pull H.264 and H.266 decoders**, which are two
  more patent pools, so any use must be `default-features = false` with only `oxideav-h265`.

**The only adoption shape that could make sense** is replacing system libheif **inside the opt-in,
never-distributed `heic` feature** (DEC-056). That removes the "is libheif installed" problem and
the LGPL notice for people who build it themselves. No one has asked for it, and the feature's
users are by definition able to install libheif. So this is a watchlist note, not work.

---

## 3. Case 3 — still-image sweep

| OxideAV crate | `image` 0.25.10 equivalent | crustyimg's position | Verdict |
|---|---|---|---|
| `oxideav-openexr` | feature `exr` | deliberately excluded (unfuzzed; `docs/lab-plan-2026-08.md` F2) | adopt nothing |
| `oxideav-hdr` | feature `hdr` | same | adopt nothing |
| `oxideav-qoi` | feature `qoi` | same | adopt nothing |
| `oxideav-dds` | feature `dds` | same | adopt nothing |
| `oxideav-tga` | feature `tga` | same | adopt nothing |
| `oxideav-pcx` | none | legacy, no demand | adopt nothing |
| `oxideav-jpegxs` | none | broadcast mezzanine codec, not a web/photo format, no demand | adopt nothing |

If any of these formats is ever wanted, the answer is to **turn on the `image` feature and fuzz
it**. A second decoder for a format the single pixel library already decodes cuts against
DEC-002 / `single-image-library` for no gain.

---

## 4. Constraint check (for the one plausible adoption, `oxideav-avif` + `oxideav-av1`)

| Constraint / DEC | Result |
|---|---|
| `single-image-library` (DEC-002) | **Compliant in the DEC-053 shape**: a codec returning planes that crustyimg's glue converts to the canonical `Image`. Caveat: pixels require the `registry` feature, which brings the **framework core** `oxideav-core` (registries, `serde_json`). Calling only `oxideav_avif::decode` is fine. Using `RuntimeContext` / registries would be adopting the framework and would trip the constraint. |
| `pure-rust-codecs-default` (DEC-004) | ✓ pure Rust, no C, no `build.rs`, no nasm |
| `no-agpl-default-deps` (DEC-018) | ✓ all MIT. The LICENSE files say MIT, not only the manifest: "© 2026 Karpelès Lab Inc." |
| `cargo deny` with crustyimg's `deny.toml` | ✓ `licenses ok, advisories ok`, **zero exceptions needed**. 17-crate tree: oxideav-{avif,av1,heif,core}, `serde_json`, `bytemuck`, `thiserror`, `memchr`, … A native swap would **retire** the `avif-parse` (MPL-2.0) and `to_method` (CC0) exceptions, and the `re_rav1d` route to the `paste` advisory. The ignore itself stays, because `rav1e` (AVIF encode) still reaches `paste`. |
| MSRV | ✗ `oxideav-core` declares `rust-version = 1.95`. crustyimg is 1.90 (max across the locked tree, so even a wasm-only target dep raises it) |
| `untrusted-input-hardening` | ✓ on what was tested (§1.5); debug-profile sweep still owed |
| DEC-077 single-thread decode | ✓ serial by default |
| AV1 patents | royalty-free (AOMedia), as DEC-053. Provenance policy is irrelevant to patents |
| `no-new-top-level-deps-without-decision` | Adoption would need a DEC. None is written, because nothing is adopted |

---

## 5. Build vs adopt

- **Build our own AV1 decoder:** no. It is a codec, and DEC-064/065 already priced "a codec port
  is a project, not a spec".
- **Patch/fork `re_rav1d` for wasm32:** still no (DEC-065). oxideav is now the cheaper route to
  the same capability: one call site upstream, versus porting dav1d-derived code off POSIX.
- **Adopt oxideav:** the code is good enough on our evidence. The *project* properties (§6) and
  the missing demand (§1.6) are what hold it back. Waiting costs nothing: the probe is checked in,
  and re-running it is minutes.

---

## 6. Maturity and provenance risk

- **The decoder is five months old.** The `oxideav-av1` CHANGELOG records an "Orphan rebuild
  (2026-05-20)", saying the crate was reset to a scaffold, "no `old` branch retained". The current
  decoder was rebuilt from that point in numbered "rounds" (r1 → r480). The workspace policy is
  written for "every contributor and every automated agent", and its velocity (~500 commits on
  `oxideav-av1` in about 6 months) suggests agent-driven development. That is not a defect in
  itself, but it means README claims are abundant and **our differential is the only evidence we
  should rely on.**
- **Clean-room enforcement is by deletion.** The workspace README records an aptX implementation
  "purged in the 2026-05-06 audit" and codecs marked "post-2026-05-18 orphan", and says violations
  must be "expunged from history (force-push)". For a downstream user, published crates.io
  versions are immutable (they can only be yanked), so a pin keeps building. But an audit can
  reset the maintained line, and the git history behind a pinned version can disappear. That is
  worth knowing before relying on its provenance story, though for an AV1 decoder the stakes are
  copyright only, and the code is MIT either way.
- **Bus factor 1.** One committer has 494 of 499 contributions on `oxideav-av1` (one other
  contributor has 5), and is the only committer on `-avif` (195) and `-heif` (126). The workspace
  has 50 stars, `-av1` 3, `-avif` 1.
- **Churn.** `oxideav-avif` is 0.0.x (every bump is semver-breaking): 14 releases since
  2026-04-17. `oxideav-core` has had 50. `oxideav-heif` was created 2026-09-18.
- **In its favour:** nightly fuzzing, `CI` + `Fuzz` green, no `unsafe` in `-av1`/`-avif`, explicit
  resource caps, and an AV1 conformance story that our independent differential confirmed rather
  than contradicted.

---

## 7. Recommendation and pre-registered triggers

**Watch and revisit.** No spike: this session's probe *was* the spike for case 1. Its answer is
"works after one upstream fix, and not needed yet". Revisit when **any** of these fire:

| # | Trigger | Check |
|---|---|---|
| T1 | **Upstream wasm fix**: `oxideav-av1` decodes on `wasm32-unknown-unknown` without a shim | `docs/probes/oxideav-avif-differential/wasm`: tests pass with `[patch]` commented out |
| T2 | **Demand**: a request for `.avif` input in `crustyimg-wasm` under Node/edge, or a spec that wants AVIF `--target`/`--ssim` inside the wasm module | A feedback item or spec. **This is the trigger that would justify a wasm-only adoption.** Size it against the ~337 KB upper bound measured inside the real bundle |
| T3 | **Stability**: `oxideav-avif` ≥ 0.1, **and** no orphan/purge event on `-av1`/`-avif`/`-heif` for 12 months (earliest **2027-05-20**), **and** a second active maintainer or org governance | crates.io + CHANGELOG + GitHub contributors |
| T4 | **Forcing**: `re_rav1d` unmaintained, a RUSTSEC advisory against it, or it blocks a toolchain/MSRV move | `cargo deny` / upstream. Then a native swap becomes the plan (oxideav already passes our differential) |
| T5 | **Grid AVIF**: a user report of a grid/tiled AVIF that crustyimg rejects | oxideav is the only permissive pure-Rust decoder verified here to decode grid correctly (vs libaom) |

**Before any adoption, whichever trigger fires:** re-run `cmp` + `fuzzlite` at the candidate
versions, **in a debug profile too** (overflow checks); add real-world AVIFs (phone, camera, Ultra
HDR gain-map); re-measure speed and the in-bundle size delta; and write the DEC (new top-level dep).

**HEIC:** no trigger here can make HEIC distributable. That stays the patent question in DEC-052,
"resolve by asking counsel". `oxideav-heif` is recorded on the watchlist as the cleanest future
backend for the **opt-in** `heic` feature, with `default-features = false`.

**What NOT to do:** adopt OxideAV as a framework (`oxideav-core` registries, `RuntimeContext`), which
trips `single-image-library`; take any video/audio/container breadth; enable `oxideav-heif`'s
default features; or read the description "orphan-rebuild scaffold" as the crate's state (F2).

**Recorded in:**
- `guidance/license-watchlist.yaml`: `avif-decode` gets the measured state that replaces the
  2026-07 line, and `heic-heif-decode` gets the oxideav-heif findings.
- `guidance/questions.yaml`: `oxideav-avif-adoption`, open, with these triggers.

---

## 8. Sources

- crates.io API, `https://crates.io/api/v1/crates/<name>` (2026-10-28), for oxideav-av1 0.1.21,
  -avif 0.0.16, -heif 0.0.13, -h265 0.0.16, -core 0.1.42, -openexr 0.0.7, -hdr 0.0.6, -qoi 0.1.7,
  -dds 0.0.6, -tga 0.0.5, -pcx 0.1.2, -jpegxs 0.0.8, -mjpeg 0.1.14, -mp4 0.0.10. All `MIT`.
- Crate tarballs, `https://static.crates.io/crates/<name>/<name>-<ver>.crate`: `Cargo.toml`
  (features, deps, `rust-version`), `LICENSE`, `README.md`, `CHANGELOG.md` (orphan rebuild
  2026-05-20), and `src/` (`unsafe`/`libc`/`Instant` greps).
- GitHub API: `repos/OxideAV/{oxideav-workspace,oxideav-av1,oxideav-avif,oxideav-heif,oxideav-h265}`
  for metadata, contributors and Actions runs. Workspace README, sections "Workspace policy:
  clean-room, no external code" and the codec status tables.
- crustyimg: DEC-002, DEC-004, DEC-018, DEC-034, DEC-052, DEC-053, DEC-056, DEC-064, DEC-065,
  DEC-066, DEC-067, DEC-077; `docs/licensing.md`; `deny.toml`; `guidance/license-watchlist.yaml`;
  `docs/lab-plan-2026-08.md` F2; `src/image/avif.rs`; `demo/worker.js`.
- Tools used as black-box oracles: `avifenc`/`avifdec` 1.4.2 (dav1d 1.5.4, aom 3.14.1), macOS
  `sips`, ImageMagick 7.1.2-29, `crustyimg` 0.7.1 (Homebrew), `cargo-deny`,
  `wasm-bindgen-test-runner` 0.2.126.

---

## Appendix — reproducing the probes

Everything is in [`docs/probes/oxideav-avif-differential/`](../probes/oxideav-avif-differential/).
I re-ran it from that location on 2026-10-28 with a fresh corpus and target directory: 36/36
byte-exact plus the grid file; wasm failed as published and passed 2/2 patched.

```sh
cd docs/probes/oxideav-avif-differential
./make-corpus.sh /tmp/oxcorpus                                   # avifenc + magick + crustyimg
cargo run --release --bin cmp -- /tmp/oxcorpus/*.avif            # differential vs re_rav1d
cargo run --release --bin cmp -- --mutate /tmp/oxcorpus/fuji_d8_420_q60.avif   # negative control
cargo run --release --bin cmp -- ../../../tests/fixtures/hostile/*.avif ../../../tests/fixtures/fuzz/avif_decode/*.avif
cargo run --release --bin fuzzlite -- 300 /tmp/oxcorpus/fuji_d8_420_q60.avif
avifdec -c aom /tmp/oxcorpus/gps_grid2x2.avif /tmp/grid.y4m      # libaom oracle for the grid
# wasm32: see wasm/Cargo.toml (unpatched → FAIL; ./patch-av1.sh + [patch] → PASS)
```

The wasm test's expected hashes depend on the encoder versions above (libaom 3.14.1 and the
`ravif` in crustyimg 0.7.1). With a different `avifenc`, regenerate them with
`cargo run --release --bin planehash -- <files>`.
