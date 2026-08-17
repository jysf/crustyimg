# A video tool on top of crustyimg — assessment (2026-08-15)

**Verdict: do not build it.** No project; three small items, all attaching to work already
scheduled — and after correction, **two**: see §11.

**Verdict, long form:** Neither of the two questions the brief called genuinely open
survives measurement. The crustyimg dependency is load-bearing for exactly one scope, that scope
is blocked by an input-codec problem crustyimg cannot help with, and no wedge against ffmpeg
survives contact with that same problem. §11 records what *is* worth taking from the session —
three measured results that belong to work already scheduled.

**Source.** An ideation session run read-only against this repo on 2026-08-15, modelled on
`docs/lab-plan-2026-08.md`. Nothing in the repo was modified.

**What was done to it.** Every load-bearing claim in the input was re-checked against the repo,
the crates.io API, a downloaded tarball, or a compiled-and-run out-of-crate probe. **Nine findings
changed the answer** and are recorded in §0 with evidence. Two of them falsified claims I had
myself already written down mid-session — those are marked, because the method that caught them is
the transferable part.

**Status.** An assessment, not a backlog. **Nothing here is committed work.** The only items
proposed for tracking are in §11, and they attach to existing stages rather than creating a project.

**Shelf-life.** Every crate probe is stamped 2026-08-15. F1 in particular is a *moving* finding —
it was true in a different direction eight weeks ago and may be true in another direction in six
months. Re-run the appendix before citing any of it.

---

## 0. Findings — what did not survive checking

### F1 — There IS a pure-Rust H.264 decoder now. It is seven weeks old. **This finding reversed mid-session.**

The brief's framing ("Rust video is decode-rich and write-poor") and my own first conclusion were
both wrong, and they were wrong for the same reason: **I probed a guessed list of crate names.**
A hand-written list of twelve plausible names returned `openh264` (C bindings), `h264-reader`
(syntax only), `less-avc` (a limited *encoder*, 2023) and `h264` (a reserved placeholder), and I
wrote down "no permissive pure-Rust H.264 decoder exists."

A **mechanical sweep** of the crates.io search API over `h264 / h.264 / avc / hevc / h265 / vp9 /
video decoder` falsified that immediately:

| crate | v | licence | dl | first publish | what it claims |
|---|---|---|---|---|---|
| `rusty_h264-decoder` | 0.10.0 | BSD-2-Clause | 14,256 | **2026-06-27** | Baseline + B-slices + most of High, CAVLC **and CABAC** |
| `rust_h264` | 0.4.0 | MIT OR Apache-2.0 | 21,457 | 2026-04-05 | pure-Rust H.264/AVC decoder |
| `rust_h265` | 0.1.0 | MIT OR Apache-2.0 | 73,568 | 2026-04-22 | pure-Rust HEVC Main / Main 10 |
| `oxideav-h265` | 0.0.9 | MIT | 13,285 | 2026-07-09 | pure-Rust HEVC parser + decoder |
| `heif-oxide` | 0.1.0 | MIT OR Apache-2.0 | 50,922 | 2026-07-27 | HEIC decode via `rust_h265` |

The lesson is the repo's own: **a guessed name list is not a sweep**
([[mechanical-sweeps-need-a-mechanical-check]]). The sweep's own scope is also a claim — it is
seven queries against the search endpoint, top 12 by downloads each.

**Probed adversarially, `rusty_h264-decoder` is the only serious candidate, and it is genuinely
good work** — 11,359 src lines, `#![forbid(unsafe_code)]`, typed errors that never panic, a
mutation fuzzer with committed CABAC seeds and three fixed DoS-class bugs, and honest benchmark
methodology (ABBA-alternated, 9/9 at z=3.00, and it *retracts* an earlier flawed harness). It
is ~2× slower than ffmpeg's native decoder and says so.

Two things I expected to find and did **not**, both corrections to my own suspicion:

- **`asm` is not a default feature.** `default = ["global-alloc"]`. The vendored openh264 SIMD
  kernels live in a separate `rusty_h264-accel` crate (593 downloads, effectively unused) that is
  opt-in. A plain `cargo add rusty_h264-decoder` is pure Rust with no nasm. Unlike rav1e (F2), the
  naive dependency is clean.
- The BSD-2-Clause licence passes `no-agpl-default-deps` without an exception.

**What remains disqualifying is not the code.** Its own README's limitations section reads: *"Not
yet: CABAC `I_PCM`, High-profile 8×8 CABAC residual, and full JVT-suite conformance."* The headline
number is **35 of 35 *clean* streams from openh264's conformance corpus** — not the JVT suite. It
is **seven weeks old**, 12 versions in that window, single-vendor. Adopting it as the input path
for a shipped tool would make an unproven crate the load-bearing component of the whole product.

**And the patent question is untouched by any of it.** OpenH264's royalty-free position comes from
Cisco paying the pool cap and distributing *prebuilt binaries*; that shelter does not transfer to a
clean-room reimplementation. This is exactly the reasoning that already gated HEIC in this repo
([[heic-no-permissive-rust-decoder]] — *"immature HEVC + patents → opt-in only"*). AVC has the same
pool structure. **I did not establish the patent position and cannot — that is a lawyer question,
not a probe.** It is listed as a pre-registered question in §12, and it is the one that would have
to be answered *before* any code, not during.

### F2 — A naive `rav1e` dependency is F2-of-the-lab-plan again, and worse. Measured.

`rav1e` 0.8.1 is already in the tree via `ravif` → the default `avif` feature. `ravif` takes it
with `default-features = false`. Measured with `cargo tree -f '{p} FEATS[{f}]'` in a scratch crate
depending on crustyimg by path:

```
baseline (crustyimg only):        rav1e v0.8.1 FEATS[]
+ naive `rav1e = "0.8.1"`:        rav1e v0.8.1 FEATS[asm, av-metrics, binaries, cc, clap,
                                    clap_complete, console, default, fern, git_version, ivf,
                                    nasm-rs, nom, scan_fmt, signal-hook, signal_support,
                                    threading, y4m]
```

**Zero features → eighteen.** Package count 214 → 234. The twenty added crates include
**`nasm-rs` 0.3.2** (a build-time assembler requirement on x86-64), **`libgit2-sys` 0.18.7**,
`libz-sys`, `pkg-config` and `vcpkg`.

Because Cargo features are additive across the graph, this does not merely bloat the consumer —
**it rebuilds crustyimg's own AVIF encoder inside that binary with `asm` and `git_version` on.**
A single naive line in a downstream `Cargo.toml` converts crustyimg from "single static binary,
zero system dependencies" into something that needs nasm and links libgit2. That directly
contradicts `pure-rust-codecs-default` (blocking) and the firm tier of `docs/territory.md`.

The fix is one attribute (`default-features = false`), and it is the same shape as the lab plan's
F2 — but the blast radius is larger: F2's naive `image` line added *decoders*; this one adds a
*toolchain requirement*.

### F3 — The per-frame seam works, executed rather than argued. And rav1e drives multi-frame.

The brief's load-bearing architectural claim — *"Operations are pure, so a downstream tool can
decode frames itself and run each through a crustyimg Pipeline"* — is **confirmed by a compiled and
executed probe**, not just by reading:

```
PIPELINE: 8 frames, 64x64 -> 32x32
QUALITY: self=100.00 vs_frame4=-19.04
RAV1E:  packets=8 keyframes=1 bytes=374
ORACLE: re_rav1d decoded 8 frames from 8 packets
PROBE OK                                          # exit 0, read directly, not through a pipe
```

The mechanism: `Image::from_parts(DynamicImage, ImageFormat, Option<MetadataBundle>)` is `pub`
(`src/image/mod.rs:293`), so a decoded frame enters a `Pipeline` **with no encode/decode byte
round-trip**. `Operation::apply(&self, img: Image) -> Result<Image>` is pure and takes by value
(`src/operation/mod.rs:137-151`); `Image` wraps exactly one `DynamicImage`
(`src/image/mod.rs:164-178`), so single-frame-by-construction is true of the *type* and irrelevant
to the *consumer*.

Two further results from the same probe, both stronger than the brief assumed:

- **rav1e is a real video encoder from out of crate.** 8 frames in → 8 packets, **1 keyframe**
  (i.e. it did inter-frame prediction), 374 bytes — driven through `Config::new_context` /
  `send_frame` / `receive_packet` with `default-features = false`. No new system dependency.
- **The round trip closes in pure Rust.** `re_rav1d` decoded all 8 frames back. Encode *and*
  decode for AV1 are already in crustyimg's tree.

**Negative control:** changing the assertion to `assert_eq!(packets, N + 1)` fails at exit **101**
with the expected message; restoring it returns exit 0. An earlier revision of the same probe
failed to compile at exit 101 on `E0603` (§F4). The harness can go red.

### F4 — The public API leaks a *third* crate item, extending lab-plan F2

The lab plan recorded `image` and `toml` leaking through the public API un-re-exported. The probe
found a third, specific and more awkward: **`Image::from_parts` takes `image::ImageFormat`, and
`crustyimg::image::ImageFormat` is private** — it is a `use`, not a `pub use`
(`src/image/mod.rs:25`). First probe run:

```
error[E0603]: ... `ImageFormat` ... private enum          exit 101
error[E0433]: cannot find module or crate `toml`
```

So an out-of-crate consumer cannot call the one constructor that makes the whole per-frame seam
work without declaring `image` itself — which is precisely the dependency lab-plan F2 shows is
unsafe to declare naively. The two findings compose badly: **the API forces the consumer into the
exact line that widens the decoder surface.** The lab plan's proposed fix (`pub use ::image;` /
`pub use ::toml;` in `src/lib.rs`) covers this too, and this finding raises its priority from
ergonomics to correctness-of-guidance. It is a re-export, not a visibility widening — the measured
zero-widening result stands.

### F5 — SSIMULACRA2's temporal blindness, reproduced in a video context and measured

The brief warned that a perceptual metric cannot see temporal defects. Rather than restate it, the
probe reproduced the live defect's exact signature on AV1:

```
TEMPORAL DEMO: sent 8 frames, output has 5.
  frame-count oracle:                  FAIL (caught it)
  SSIMULACRA2(frame1 vs frame1): 100.0 PASS (blind!)
```

Three of eight frames discarded; the perceptual oracle scores a perfect 100.0. This is
`quality::score(&DynamicImage, &DynamicImage)` (`src/quality/mod.rs:99`), which calls
`compute_frame_ssimulacra2` — it is a **frame** metric by construction, not by accident.

The same probe shows the metric is not useless, merely out of domain: frame 0 vs frame 4 scores
**-19.04**. It discriminates fine on pixels; it has no opinion about time.

**This is a runnable demonstration of `docs/backlog.md:979-983`'s claim**, in a codec the repo
already ships, and it is the single most transferable artifact of the session (§11).

### F6 — `mp4-atom` falsifies "webm-iterable is the only write-capable one"

Handed as fact; false. Two of the five containers probed can write, and the better one was not in
the table at all.

**`mp4-atom` 0.15.0** — MIT OR Apache-2.0 (both LICENSE files present and real), 23,631 src lines,
243,922 downloads, updated 2026-07-31, from the Media-over-QUIC project (`kixelated`). Pure Rust:
`bytes`, `derive_more`, `num`, `pastey`, `serde`, `thiserror`, `tracing` — **no `-sys` crate**. It
is symmetric: `pub trait Encode { fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()>; }`
(`src/coding.rs:45`).

Critically for anything in this repo's orbit, it already models what would be needed:

- **`av01` sample entry and `av1C` config box** — `src/moov/trak/mdia/minf/stbl/stsd/av01.rs:21,87`
- the complete sample table — `stts`, `stsc`, `stsz`, `stco`, `co64`, `stss`, `ctts`
- a committed test decoding a **real libavif animated AVIF** — `avis` major brand with
  `msf1`/`iso8`/`mif1`/`miaf` compatible brands (`src/test/libavif_anim.rs:3-27`) — and an
  `assert_encode_decode()` round-trip on the `av01` atom (`src/test/av1.rs:236`)

`mp4` 0.14.0 also writes (`Mp4Writer::write_start/write_sample/write_end`, `src/writer.rs:63,101,133`)
but is stale (2023-08-01) and has **no AV1 support** (`grep -l 'av01\|Av1' src/` → nothing).

Its README is honest about what it is *not*: *"low level, performing encoding/decoding of the
binary format without validation or interpretation... You have to know what boxes to expect!"* It
gives you the boxes, not a muxer.

### F7 — The muxer price, measured rather than estimated

Downloaded and counted, 2026-08-15:

| implementation | src lines | what it is |
|---|---:|---|
| `ivf` 0.1.4 | **152** | the minimal AV1 container — **no browser plays it** |
| `mp4` 0.14 `writer.rs` + `track.rs` | **1,007** | the muxing *driver* on top of a box library — the honest analogue |
| `webm-iterable` 0.6.4 | **1,330** | generic read+write Matroska (input said 1,330 — exact) |
| `matroska` 0.30.1 | **2,221** | read-only (input said 2,283 — 3% off) |
| `re_mp4` 0.5.1 | 7,912 | read-side |
| `mp4` 0.14 total | 10,402 | box library + writer |
| `mp4-atom` 0.15.0 | 23,631 | typed box tree, read+write |

**The number that matters is 1,007** — not the size of a box library you depend on, but the size of
the driver you write on top of one. The floor (152, IVF) is unusable for delivery.

### F8 — "Pure-Rust audio encoding does not exist" is too strong; the conclusion survives anyway

`flacenc` 0.5.1 (Apache-2.0, 600,932 downloads) is a pure-Rust FLAC **encoder**. So the claim as
stated is false.

The practical conclusion is unaffected, and it is worth separating the two: every *lossy* codec
anyone would ship for delivery is a C binding — `opus` 0.3.1 and `libopus_sys` (libopus),
`fdk-aac` 0.8.0 (fdk-aac), `vorbis_rs` 0.5.6 (libvorbis), `audiopus` (last touched 2021). FLAC is
lossless, so it is the wrong tool for web delivery regardless of language.

Same shape as the lab plan's F6 lut-cube correction: **the stated reason was wrong and the call was
right.** Recording it so the false half is not repeated as a fact.

### F9 — crustyimg supplies ~14% of itself to a video tool, and 4 ops

Counted (`find src -name '*.rs' | xargs wc -l`): **28,920 src lines total.**

| module | lines | reusable by a video tool? |
|---|---:|---|
| `operation` | 2,042 | yes — but see the op count below |
| `quality` | 1,158 | yes, per frame |
| `recipe` | 722 | yes |
| `pipeline` | 223 | yes |
| **subtotal — the shared core** | **4,145** | **14.3% of crustyimg** |
| `sink` | 1,258 | only for *still* output |
| `image` | 3,638 | still decode/encode; nothing about frames |
| `cli` / `build` / `analysis` / `metadata` / `lint` / `source` / `text` | 19,879 | no |

And the registry holds **four** ops — `identity`, `invert`, `resize`, `auto-orient`
(`src/operation/registry.rs:80-83`), lab-plan F3, re-verified. For video, `identity` is a no-op,
`invert` is a novelty, and `auto-orient` is EXIF-driven (video carries orientation in the `tkhd`
matrix instead). **The usable inheritance is one op: `resize`.**

---

## 1. THE CRUX — is the crustyimg dependency load-bearing? (Q1)

**It is load-bearing for exactly one scope, and the answer flips completely between two scopes that
the phrase "video tool" hides.** That split is the real finding.

**Video → video (transcode, trim, clip).** crustyimg supplies `resize` and a per-frame perceptual
score. Everything hard — demux, frame decode, encoder driving, rate control, keyframe placement,
timescales, muxing (F7: ~1,000 lines), A/V sync — is net new and comes from nowhere in this repo.
Worse, per-frame SSIMULACRA2 is *the wrong instrument*: nobody tunes a video encoder frame-by-frame
on a still-image metric. **Here "on top of crustyimg" is a story, not an architecture.** The honest
form of this project is a standalone crate that happens to share a maintainer.

**Video → images (poster, thumbnail, contact sheet, frame extraction).** crustyimg supplies
essentially the entire output half: the format decision, the quality search, the encoders, naming,
the manifest, `lint`, batch. The video part is a **source adapter** — one new `Source` that yields
frames. Here the dependency is not merely load-bearing, it is the whole product, and the tool sits
squarely inside `docs/territory.md`'s existing claim rather than extending it.

So Q1's answer is **"yes, for video → images"** — and that would be the end of a happy session,
except that §2 shows the scope where the dependency is load-bearing is also the scope with no
wedge, and both scopes collide with the same input problem (F1).

**One more thing kills the video → video framing independently of any codec question.** DEC-091's
Fence B: *"The workhorse emits artifacts a build consumes."* A poster frame, a thumbnail sheet and a
sprite atlas are build artifacts. A transcoded MP4 is a build artifact too — but it is one that the
**manifest cannot describe and `lint` cannot check**, because every claim crustyimg makes about an
artifact ("72% smaller · ssim 100.0") is a *frame* claim. Extending the manifest to video means
extending the quality vocabulary to video, which is a new research problem, not an integration.

---

## 2. Why would anyone use this instead of ffmpeg? (Q2)

**I cannot name a wedge.** Reporting that, as the brief permits.

Five candidates were tested rather than assumed. Four fail, and the fifth is not about video.

**1. "Single static binary, zero system deps."** Dies on F1's timing. Today the only pure-Rust
decoders for the formats users actually have are seven weeks to four months old, single-vendor, and
self-describe as short of full conformance. You may have zero system dependencies *or* a
production-grade decode path for real files, not both — until these crates mature. This is not a
permanent no; it is a **"not for at least a year, and only if `rusty_h264` is still maintained
then."**

**2. "Permissive licence where ffmpeg's GPL/LGPL matrix blocks commercial redistribution."** This is
a real pain for redistributors and the most attractive-sounding angle. **It inverts under F1's
patent paragraph.** The licence problem people have with ffmpeg is *copyright*; the problem with
shipping your own H.264 decoder is *patents*, and a permissive licence provides no shelter from a
patent pool at all. Trading a known, well-understood LGPL obligation for an unquantified patent
exposure is not a wedge — it is a worse deal wearing a better label. Note this is the one place
where a *pure-Rust* implementation is strictly worse off than a binding: `openh264` users at least
inherit Cisco's arrangement.

**3. "Reproducible / deterministic output."** rav1e is deterministic at fixed settings, so this is
achievable. But ffmpeg with pinned settings is reproducible enough in practice, and no user is
blocked on this today. A nice property, not a reason to switch.

**4. "Build cache + manifest."** Genuinely differentiating — and it is differentiating **for image
assets**, which crustyimg already owns and already ships. Adding video *input* does not strengthen
it. See §3 for the two-command version of this that already works.

**5. "Safe on untrusted input."** The strongest-sounding, and it inverts hardest. Video decoders are
the most CVE-dense category in media software, so a memory-safe one is a real win — and
`rusty_h264-decoder`'s posture (`forbid(unsafe_code)`, fuzzed to never panic, typed errors, three
DoS bugs found and gated) is *exactly* crustyimg's. But crustyimg's current safety claim is strong
precisely because it declines the unsafe formats. Shipping a seven-week-old decoder for the most
hostile input class in the domain would make the claim **weaker than it is today**, on the repo's
flagship differentiator. And DEC-088 decision 2 forecloses the escape hatch: no tier-3 delegation,
so "shell out to ffmpeg for decode" is not available either.

**The falsifiable sentence, and its falsification.** The best I could construct:

> *"A single static binary that turns video into delivery-grade image assets with a manifest, with
> no ffmpeg install and no system dependencies."*

**Falsified by F1 + §3.** The "no ffmpeg install" clause requires a decoder that is not yet
trustworthy, and everything after "turns video into" already works today with zero new code.

The surviving true version — *"…for AV1 and WebM sources"* — is falsifiable, accurate, and serves
approximately nobody, because AV1 is a *delivery* format. Nothing a user owns is AV1 on the way in.

---

## 3. What is the actual scope? (Q3) — and the answer that costs nothing

The brief's candidates, triaged:

| candidate | needs audio? | crustyimg load-bearing? | blocked by F1? |
|---|---|---|---|
| thumbnail / poster extraction | no | **yes, heavily** | yes |
| contact sheets from video | no | **yes, heavily** | yes |
| frame-accurate trimming | yes (or it is useless) | no | yes |
| short-clip transcode | yes | no | yes |
| GIF/animation → modern video | no | n/a — **not ours** (§4 of the brief; STAGE-046) | **no** |

Everything that does not touch audio is the same product — *get frames out, hand them to
crustyimg* — and it is exactly the scope §1 calls load-bearing. Which makes the following result
the most important practical finding of the session.

**It already works. Driven end to end on the shipped 0.7.0 binary:**

```
$ crustyimg web ./frames --out-dir ./out
frames/frame_00000.png: png → avif · 1195 → 767 B (36% smaller) · ssim 91.6
frames/frame_00001.png: png → avif · 1270 → 832 B (34% smaller) · ssim 90.7
...
frames/frame_00007.png: png → avif · 1268 → 1079 B (15% smaller) · ssim 88.5
                                                          exit 0, 8/8 frames converted
```

Given `ffmpeg -i in.mp4 frames/%05d.png`, the entire non-audio value of the proposed tool is two
commands, no new software, and it is **DEC-088 tier 1 (file interchange) — the preferred tier**,
which that DEC specifically notes is *"hashable — so it participates in the build cache key."* A
spawned or embedded decoder would *lose* that property (DEC-088, Alternatives).

`crustyimg` reads one image from stdin, not a stream, so `ffmpeg ... | crustyimg` (tier 2) does not
work for a frame sequence — you must write frames to a directory first. **I first recorded that as a
gap worth closing and then measured that it is not.** `compute_key` takes `input_hash: &Hash` —
the cache is content-addressed on the input *bytes* (`src/build/cache.rs:245-252`), so a directory of
extracted frames caches **per frame**: re-running re-encodes only what changed. A piped stream cannot
participate in that cache at all. DEC-088's preference for tier 1 (*"hashable — so it participates in
the build cache key"*) is not a stylistic ranking; it is the reason the file route is **better** here,
not merely acceptable. **No item.**

---

## 4. Where is the audio line? (Q4)

**Moot, and worth recording anyway** because the answer disqualifies the scopes that need it.

The brief hoped remuxing a passthrough track would dodge the encoder gap. It does — F6 shows
`mp4-atom` can write the container, and the track bytes pass through untouched, so no audio encoder
is required. **The reason it still fails is A/V sync, not encoding.** The moment you trim or
re-time video, an untouched audio track must be cut on a *frame* boundary that does not align with
a *sample* boundary; correcting that requires decode, resample (`rubato`), and re-encode — and
re-encode is where F8's C-bindings wall stands. Passthrough is only free when you change nothing
about timing, which is to say when you are not editing.

**Recommended line, if this is ever revisited: no audio at all.** Not "audio behind a feature" —
the webp-lossy/DEC-022 precedent does not transfer. There, the default build still does its whole
job and one *output* option is missing. Here, a default build that cannot carry audio cannot
produce a usable clip at all, so the feature flag would be gating the product rather than an
option. That asymmetry is the reason §3's audio-free scopes are the only defensible ones — and they
are the ones that need no new tool.

---

## 5. How much is the muxer, really? (Q5)

**~1,000 lines of driver on top of a box library** (F7), not the 150–250 the animated-WebP estimate
suggests and not the 1,330 of a generic Matroska library.

MP4 vs WebM, checked rather than assumed: **MP4 is the easier target here**, contrary to the usual
intuition that Matroska is simpler. The reason is specific and not about the formats in the
abstract — `mp4-atom` already ships `av01` + `av1C` and a committed animated-AVIF test (F6), so the
AV1-specific half is done, whereas the WebM route would need CodecPrivate/CodecID work on top of
`webm-iterable`'s generic element model. The MP4 driver's remaining cost is **bookkeeping**
— timescales, sample-to-chunk tables, chunk offsets, sync-sample flags — which is tedious,
well-specified, and exactly the kind of thing the measured 1,007-line comparable prices honestly.

This is priced for §11's benefit, not this project's.

---

## 6. What does "correct" mean, and how is it tested? (Q6)

**The oracle question has a clean answer, and it is the one part of this assessment that is
unambiguously good news** — which is why §11 hands it onward.

The ladder from `docs/lab-plan-2026-08.md` §4 transfers, with **one mandatory new bottom rung**:

> **Tier −1 — the structural oracle. Frame count, frame order, and presentation timing, asserted
> independently of pixels, using a decoder you did not write.**

Not a refinement of the existing tiers — a **precondition** for them. F5 measured why: a pipeline
that discarded 3 of 8 frames scores SSIMULACRA2 **100.0**. Every pixel-based tier in the ladder
runs *after* this one, or it certifies a lie.

The tier is cheap and already available: `re_rav1d` is in the tree, and the probe drove
encode → decode → count in a few dozen lines, with a negative control. This satisfies the repo's own
standard ([[verify-wasm-output-with-an-independent-decoder]]) — the decoder is a different codebase
from the encoder, so a count it produces is *"an independent value the code under test cannot
fabricate"* (`src/image/avif.rs:767-793`).

Above it, Tier 0 (replay equivalence) is the weakest rung for video and should not be leaned on:
`tests/edit.rs:216` proves byte-identity for PNG, but neither rav1e nor any video encoder guarantees
bit-reproducibility across platforms and thread counts. For video, Tier 0 must be replaced by
Tier −1 plus decoded-buffer digests, not extended.

---

## 7. Tool, library, or neither? (Q7)

**Neither — for a video tool.** And the brief's own hypothesis, that the highest-value output might
be a muxer crate rather than a CLI, is **half right in a way that relocates it entirely.**

The ecosystem gap is real: pure-Rust *muxing* is thinner than decode or encode. But F6 shows it is
thinner than the brief thought only for WebM — `mp4-atom` already covers ISOBMFF, permissively and
with AV1 in place. So the crate-shaped opportunity is smaller than hoped and partly already taken.

More decisively: **the consumer for such a muxer is not a video tool. It is crustyimg itself.**
`docs/backlog.md:944` is a live defect whose repair (STAGE-046) needs a container *writer* —
RIFF/ANMF for animated WebP, and ISOBMFF for animated AVIF, which is literally AV1 OBUs in an
`avis`-brand container. crustyimg already has the AV1 encoder (rav1e) and decoder (re_rav1d) in its
tree, and F6 found a permissive library with the exact boxes.

**That is the whole finding of Q7:** the muxing capability is worth having, its first-party consumer
already exists in this repo with a scheduled stage and a driven reproduction, and routing it through
a speculative second binary would be strictly worse — a new project to justify a component that a
shipped product already needs.

**I want to be explicit about a scope risk here rather than quietly act on it.** The brief instructs
that animated GIF → animated WebP/AVIF is STAGE-046's and that concluding otherwise is a scope error
to argue, not assume. **I am not claiming it.** I am reporting three measured facts that arrived in
this session and belong to it (F5, F6, F7) and recommending they be handed over — §11. The animated
*AVIF* option in particular is a *new output candidate* for that stage to accept or decline on its
own terms, not a change to its scope, and it should be weighed against the in-house RIFF route the
backlog already prices.

---

## 8. Licence and dependency review (Q8)

Every licence below was read from the crate's own `LICENSE` file where one ships, not only the
crates.io metadata field — the lab plan's F6 found that field lying in both directions.

| crate | v | licence (verified) | dl | updated | verdict |
|---|---|---|---|---|---|
| `rav1e` | 0.8.1 | BSD-2-Clause | 39.6M | 2025-06-16 | **already in tree** — must be `default-features = false` (F2) |
| `re_rav1d` | 0.1.3 | BSD-2-Clause | — | in `Cargo.lock` | **already in tree**, native-only (`src/wasm.rs:20`) |
| `mp4-atom` | 0.15.0 | MIT OR Apache-2.0 (both files present) | 243,922 | 2026-07-31 | **the find.** Pure Rust, no `-sys`, `av01`/`av1C` |
| `rusty_h264-decoder` | 0.10.0 | BSD-2-Clause | 14,256 | 2026-08-13 | permissive and good; **7 weeks old + patents** → no |
| `webm-iterable` | 0.6.4 | MIT | 343,886 | 2024-12-12 | write-capable; 2 years stale |
| `matroska` | 0.30.1 | MIT/Apache-2.0 | 277,739 | 2026-04-15 | read-only |
| `re_mp4` | 0.5.1 | MIT | 1.85M | 2026-07-08 | read-only |
| `mp4` | 0.14.0 | MIT | 12.1M | **2023-08-01** | writes, but stale and **no AV1** |
| `symphonia` | 0.6.1 | MPL-2.0 (LICENSE = real MPL 2.0 text) | 10.4M | 2026-08-13 | read-side only; see below |
| `flacenc` | 0.5.1 | Apache-2.0 | 600,932 | 2025-12-18 | pure-Rust FLAC encode (F8) |
| `opus` / `fdk-aac` / `vorbis_rs` / `audiopus` | — | — | — | — | **all C bindings** — the real audio wall |
| `ffmpeg-next` | 9.0.0 | WTFPL *wrapper* | 6.35M | 2026-08-05 | links ffmpeg (C, LGPL/GPL). Tier 3 — not built |

**On `symphonia` / MPL-2.0** — the brief's read is correct and worth preserving even though the
crate is not needed: MPL-2.0 is *file-level* weak copyleft, compatible with static linking into a
permissive binary, and `deny.toml` already carries the `avif-parse` precedent for a documented
per-crate exception. It is **not** a blanket no under `no-agpl-default-deps`. Recording this so the
next session that meets an MPL crate does not re-derive it.

**On `ffmpeg-next`'s "WTFPL"** — this is the *wrapper's* licence and is close to meaningless here.
The obligations come from the ffmpeg libraries it links (LGPL-2.1+, or GPL if built
`--enable-gpl`). Reading the metadata field alone would produce exactly the wrong conclusion —
another instance of the F6 lesson.

**Nothing is recommended for adoption.** `mp4-atom` is recorded as *available and cleared on
licence* for STAGE-046 to evaluate; adopting it would require a DEC under
`no-new-top-level-deps-without-decision` and a `just deny` run.

---

## 9. Anti-goals — naming these is most of the value

Even given a "do not build" verdict, these make the *next* proposal cheap to adjudicate:

1. **crustyimg does not decode video.** Not behind a feature, not via a delegate. Frames arrive as
   files (tier 1) or they do not arrive. This is the tier-3 prohibition (DEC-088 decision 2)
   applied to its most tempting case.
2. **crustyimg does not emit video.** The manifest, `lint` and every quality claim in the tool are
   *frame* claims. A video output would be an artifact the tool cannot describe or check — Fence B
   (DEC-091) says that is not a workhorse artifact.
3. **No audio, ever, in this repo.** Not gated, not passthrough. §4.
4. **A perceptual score is never evidence about time.** Any future multi-frame work asserts frame
   count, order and timing *first*, with an independent decoder (§6, F5).
5. **"It's in the dependency tree" is not a reason to build something.** rav1e and re_rav1d being
   present made this project *look* nearly free. F9 measures the actual inheritance at one usable op.

---

## 10. Pre-mortem — how this fails if built anyway

The brief asks that the top risk not be technical if an adoption risk is bigger. It is not
technical, and it is not adoption either.

**Top risk: it consumes the maintainer's attention at the exact moment the shipped product has four
measured output-fidelity defects on its flagship verbs.** STAGE-046 exists because `convert`,
`optimize` and `web` silently destroy animated input and report `ssim 100.0` while doing it —
on a path `lint` actively recommends. A video project would draw the scarce resource (one
maintainer, sequencing by dependency and value) toward a speculative second binary while the
shipped one is publishing a claim it contradicts. **The cost is not the video tool's failure; it is
STAGE-046's delay.**

**Second: the "nearly free" illusion is unusually strong here** and would survive a casual review.
Everything looks present — an AV1 encoder, an AV1 decoder, a pure operation trait, a quality metric,
a manifest. F3 even *proves* the pipeline works end to end. The measured reality is one usable op
(F9), a ~1,000-line muxer (F7), and an input path that cannot open the user's files (F1). A project
that demos convincingly in week one and stalls in month three is worse than one that never starts.

**Third: shipping a seven-week-old decoder for the most hostile input class in the domain would put
the repo's strongest differentiator at risk** on a codebase nobody in this project can audit. If
that decoder has one CVE-class bug, the damage lands on crustyimg's "safe on untrusted input"
claim, not on the video tool's.

**Fourth, and the one that would hurt most: an ffmpeg-shaped tool invites ffmpeg-shaped requests.**
Filters, subtitles, streaming, hardware acceleration, `-vf` compatibility. crustyimg's scope
discipline works because the territory has an edge (`docs/territory.md`). Video has no natural edge
short of reimplementing ffmpeg — DEC-088 rejected "decide per feature" for exactly this reason:
*"deciding case by case is how an anti-goal erodes silently."*

---

## 11. What to take from this session

Three results arrived here and belong to work that is already scheduled. **None creates a project.**

**(a) → SPEC-119: confirmatory, not new. Say so.** While this session ran, `SPEC-119
animated-input-is-never-silently-flattened` landed on `main` (7c49340) and **already carries this
requirement**, derived independently: **AC-6** — *"The assertion is structural, never the quality
score... A test that asserts 'ssim stayed high' is vacuous by construction here and will be
rejected"* — with a named failing test,
`"animated_output_frame_count_is_asserted_structurally"`.

F5 therefore **adds no requirement**. What it adds is an *executed* demonstration of the premise
(3 of 8 frames lost, SSIMULACRA2 **100.0**, frame count catches it) in a codec this repo already
ships, plus a working template for the independent-decoder half — SPEC-119 says *"decode the
output's frame count"* without pinning that the decoder must be one you did not write
([[verify-wasm-output-with-an-independent-decoder]]). Two agreeing derivations from different
directions is evidence the call is right, and that is worth recording as exactly that — **not**
re-filed as a finding SPEC-119 is missing.

**(b) → STAGE-046's *animated-output* spec, not SPEC-119.** F6: `mp4-atom` is available,
permissive, pure Rust, and ships `av01`/`av1C`. SPEC-119 is scoped to piece **(a)** of the backlog
entry — stop the data loss, warn — and explicitly not to new output capability, so this belongs to
the later spec that takes piece **(b)**. It is a *second output candidate* (animated AVIF alongside
animated WebP) with F7's measured ~1,000-line muxer-driver price attached, so the comparison against
the in-house RIFF route is priced on both sides. **Not a scope change — an input to a decision that
stage owns.**

**(c) → the `pub use` fix in `src/lib.rs`: raise its priority.** F4 shows the leak is worse than
recorded — `Image::from_parts` requires `image::ImageFormat`, which is private, so any out-of-crate
consumer is *forced* into the naive `image` dependency that lab-plan F2 measures as adding six
decoders. Two lines (`pub use ::image;`, `pub use ::toml;`), no visibility widening, and it closes a
correctness-of-guidance hole for the lab work that *is* scheduled.

---

## 12. Pre-registered questions — the revisit trigger

This verdict is **timing-dependent, not permanent**. It should be revisited if and only if all four
are answered affirmatively, with measurements:

1. **Is `rusty_h264-decoder` (or a successor) still maintained, and does it pass the full JVT
   conformance suite?** Its own README lists that as outstanding today. Re-probe: publish cadence,
   maintainer count, and the conformance claim as stated *by the crate*, not by a summary.
2. **What is the AVC patent position for an independently-implemented decoder distributed in a
   permissively-licensed binary?** The blocking question, and **not answerable by probing** — it is
   a lawyer question. Until answered, F1's crate maturity is irrelevant, because a mature decoder
   with an unresolved patent exposure is not adoptable under the licensing goal in
   `no-agpl-default-deps`'s rationale ("freely embeddable... incl. in closed-source/commercial
   products").
3. **Has anyone actually asked for this?** The lab plan's top pre-mortem risk was "no user"; this
   project's §3 shows the non-audio value is already two commands. A real request — with the reason
   the two-command version is insufficient — would be new information and should be recorded before
   any design.
4. **Would the frame-source work be needed by the workhorse anyway?** If STAGE-046 ends up building
   multi-frame decode + a container writer for animated images, the marginal cost of video input
   drops sharply, and this assessment should be re-run against the *then*-current tree rather than
   this one. That is the most likely path by which the answer changes.

---

## Appendix — reproducing the probes

All probes ran **outside the repo**, in scratch crates each with their own `CARGO_TARGET_DIR`
([[concurrent-differently-featured-builds-corrupt-a-shared-target-dir]]). Nothing in `crustyimg`
was modified. Every exit code was read **directly, never through a pipe**
([[a-piped-command-reports-the-pipes-exit-code]]).

**1. rav1e feature unification (F2).** Scratch crate depending on `crustyimg` by path;
`cargo tree -f '{p} FEATS[{f}]' | grep rav1e`, then again with `rav1e = "0.8.1"` added.
Baseline `FEATS[]` → naive `FEATS[18 features]`; package count 214 → 234;
`cargo tree | grep -E '(nasm-rs|libgit2-sys|libz-sys|pkg-config|vcpkg)'` non-empty only in the
naive arm. **Control:** the two arms differ, so the measurement discriminates.

**2. The seam + AV1 round trip + temporal demo (F3, F4, F5).** One scratch crate,
`crustyimg` (path) + `rav1e` (`default-features = false`) + `re_rav1d` + `image`. Eight synthetic
frames → `Image::from_parts` → `Recipe::from_toml` → `build_pipeline` → `run` →
`Config::new_context`/`send_frame`/`receive_packet` → `re_rav1d` `send_data`/`get_picture`.
Output reproduced in F3/F5. **Negative controls, both fired:** `assert_eq!(packets, N + 1)` → exit
**101** with the expected message; an earlier revision referencing the private
`crustyimg::image::ImageFormat` → `E0603`, exit **101**. Restoring each → exit 0.

**3. Tier-1 already works (§3).** Eight PNG frames written by a standalone Python script (**not**
by crustyimg — [[fixtures-from-the-code-under-test-cannot-fail]]), then
`./target/release/crustyimg web <dir> --out-dir <out>` on the **shipped 0.7.0 release binary**.
Exit 0, 8/8 → AVIF. First attempt exited **2** (`multiple inputs require --out-dir`), which is a
control on the invocation: the binary rejects the wrong form rather than silently doing nothing.

**4. Crate probes (F1, F6, F7, F8, Q8).** crates.io API, 2026-08-15, `User-Agent` set. The H.264
sweep used the **search endpoint** over seven queries (`h264`, `h.264`, `avc`, `hevc`, `h265`,
`vp9`, `video decoder`), top 12 by downloads each — a guessed name list had already produced the
wrong answer (F1). Line counts by `static.crates.io` tarball + `find src -name '*.rs' | xargs wc -l`.
Licences read from the extracted `LICENSE*` files, not the metadata field.

**5. crustyimg's own numbers (F9).** `find src -name '*.rs' | xargs wc -l`, per module.
Op count from `src/operation/registry.rs:80-83`, cross-checked against lab-plan F3.
