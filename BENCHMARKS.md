# Benchmarks

How does crustyimg actually compare to the tools people already use? This is an
honest, **equal-quality**, reproducible comparison of `crustyimg` against
[**sharp**](https://sharp.pixelplumbing.com/) (Node/libvips, the incumbent),
[**ImageMagick**](https://imagemagick.org/), and
[**`@squoosh/cli`**](https://github.com/GoogleChromeLabs/squoosh) — plus
**cwebp** (WebP-only, for format context) — on **size, speed, and quality**, over
a corpus of real photographs.

The point is credibility, not a favourable scoreboard. Where crustyimg loses, this
doc says so — and it does lose on two axes. Read to the end.

## TL;DR

At **matched perceptual quality** (every tool's output scored by the same
SSIMULACRA2 metric and tuned to the same "high" band), on the web-ready job of
_downscale a photo to ≤ 2048 px and encode AVIF_:

- **crustyimg is never the smallest, and it's the slowest.** sharp (libvips)
  produces the smallest AVIF more often than anyone else (4 of the 8 photos);
  sharp and ImageMagick, which use every CPU core, finish roughly 3–9× and 4–14×
  faster on the clock. crustyimg's files run from about parity to ~50% larger
  than sharp's, and it takes ~1–5 s where sharp takes ~0.3–1 s.
- **But the gap is threading and a deliberate speed preset, not a weak encoder.**
  crustyimg is single-threaded by design (it's the same engine that runs in the
  browser); pin sharp to a single thread and the two trade wins photo-for-photo.
  crustyimg's compression is in the same league as the others, not behind them.
- **What crustyimg actually offers** is a single static Rust binary with no system
  libraries and no native addon, a perceptual quality score built into the tool
  (the same metric used to judge everyone here), RAW-file input, and the exact
  same engine in the browser. sharp needs a native libvips addon; ImageMagick is a
  large C install; `@squoosh/cli` is archived and no longer runs on current Node.
- **AVIF is the honest comparison, and cwebp can't play** — at matched quality a
  dedicated WebP encoder is larger than every AVIF tool here (1.2× to 3.0× the
  smallest AVIF, worst on detailed photos). It's here as context, labelled, not
  as a contestant it can't win.

So: if you want the absolute smallest file or the fastest batch on a many-core
box, sharp is excellent and this doc will tell you so. If you want one dependency-
free binary that gives you a competitive AVIF, a real quality number, RAW support,
and the same code in the browser — that's crustyimg.

## How quality is matched (the whole point)

"Smallest file" is meaningless without equal quality — a smaller file is trivial
if you just lower the quality. So every number below is at **matched quality**:

- **One scorer for every tool.** Each tool's output is scored with `crustyimg
  diff` (SSIMULACRA2, a perceptual metric where ~70 is "high" and ~90 is "visually
  lossless"), against **that tool's own lossless downscale** of the source. Scoring
  each encode against its own downscale isolates *encode* fidelity and doesn't
  reward or punish a tool for its resampler. That's not a formality: the tools'
  lossless downscales of the same photo score 92–94 against each other on most of
  this corpus, but on one 24 MP photo sharp's lands at ~82 against everyone
  else's. Scored against a single shared reference, that difference would have
  been charged to sharp's *encoder*.
- **Same pipeline for every tool — measured, not assumed.** Downscale the long
  edge to ≤ 2048 px (never upscaling), then encode AVIF. This is the real "make it
  web-ready" job, and it's where the size savings come from — most of a 47 MP
  photo's shrink is the downscale, which every tool can do. Every tool spells
  "long edge" differently, so the harness measures every output it produces and
  fails the run if one came back a different shape than the source. That check
  matters more than it sounds: because each tool is scored against its own
  reference, a tool that stretched the image would score its distorted encode
  against its distorted reference and land in the band like everyone else. The
  quality column cannot police the downscale, so something else has to.
- **Each tool is tuned to the same band.** For each photo, every tool is swept
  over a fixed quality grid and the setting whose score lands nearest **82** is
  chosen. Nobody is run at "smallest", nobody is tuned to win. The matched score is
  shown in every row so you can see where each tool actually landed (79.0–83.6).
  82 is *not* crustyimg's default operating point — `crustyimg web` lands around
  75 (see below). It's a band every tool's quality grid can bracket, high enough
  to be a quality anyone would ship. Matching everyone at crustyimg's own default
  would have pulled the competitors down to a lower-quality band chosen to suit
  crustyimg; 82 tunes crustyimg *up*, away from the setting its fast preset is
  built for, which is the conservative direction to be wrong in.
- **No hand-edited numbers.** The tables are emitted by the harness
  (`scripts/bench-compare.py`); scores and byte counts are deterministic (two runs
  produce identical numbers), and only the wall-times vary because they're
  measurements. The methodology was fixed before the numbers were read.

## The corpus

8 real photographs, **0.7 to 47 megapixels**, from six cameras across four brands —
Fujifilm (X100F), Nikon (COOLPIX P1100, D3300, D750), Leica (Q2 Monochrom), and
Apple (iPhone 15) — as shot, JPEG and PNG, with real metadata and colour profiles.
These are private photos, so they are **not committed to the repo** (real EXIF/GPS,
and detailed photographs don't compress, so they'd bloat the tree). The harness
takes `--corpus <dir>` so you point it at your own; see
[Reproduce it yourself](#reproduce-it-yourself). Size distribution: one 0.7 MP,
two 8 MP, one 16 MP, three 24 MP, one 47 MP.

## The machine and the tools

- **Machine:** Apple M4 Pro, 14 cores, macOS 26.5.2, 48 GB. Nothing else running.
- **crustyimg 0.5.0**, built `--features avif`. AVIF encode is a compile-time
  feature (pure Rust — ravif/rav1e, no system libraries), off in the default
  distributed binary; install it with `cargo install crustyimg --features avif`.
  It's the same pure-Rust AVIF encoder the browser demo ships.
- **sharp-cli 5.2.0** (sharp 0.34.4, libvips) on Node 22 — AVIF via libvips/libaom.
- **ImageMagick 7.1.2-27** — AVIF via libheif 1.23.1 / libaom.
- **`@squoosh/cli` 0.7.2** — AVIF via its own libaom-in-wasm. It is **archived**,
  and it no longer starts on current Node (it crashed on Node 22 here); it was run
  on Node 16. That's context, not a dig — it's part of why a maintained
  alternative is worth having.
- **cwebp 1.6.0** (libwebp) — **WebP only, no AVIF.** Included as a labelled
  format-context row, not an AVIF competitor.

### Exact commands

Each tool downscales the long edge to ≤ 2048 px and encodes at the quality `Q`
that matched the band (`Q` varies per photo; see the tables). crustyimg's one-
command default is `web`; the matched-quality row uses `convert -q` to tune it.

`E` below is the target long edge — `min(2048, the source's long edge)`, so a
photo already under 2048 px is never enlarged. These are the exact commands the
harness runs.

```sh
# crustyimg — one command, its own default fast-AVIF quality
crustyimg web photo.jpg --max E -o out.avif

# crustyimg — tuned to the matched-quality band
crustyimg resize photo.jpg --max E -o ds.png
crustyimg convert ds.png --format avif -q Q -o out.avif

# sharp
sharp -i photo.jpg -o out.avif resize E E --fit inside --withoutEnlargement -f avif -q Q

# ImageMagick
magick photo.jpg -resize 'ExE>' -quality Q out.avif

# @squoosh/cli  (archived — run it on an older Node, e.g. Node 16)
#   one axis only: given both, squoosh stretches the image to that box
squoosh-cli --avif '{"cqLevel":Q}' \
  --resize '{"enabled":true,"width":E,"method":"lanczos3"}' -d out/ photo.jpg
#   ...and for a portrait source, "height":E instead

# cwebp  (WebP, not AVIF) — the 0 axis is derived from the other
cwebp -q Q -resize E 0 photo.jpg -o out.webp      # portrait source: -resize 0 E
```

## Results — by photo size

Median across the photos in each bucket. **Score** is the matched SSIMULACRA2
(higher = better; all clustered near the 82 target). **Size** is the output; **vs
source** is how much smaller than the original; **time** is the median wall-clock
of the downscale-and-encode.

### Large photos (> 12 MP — the headline case: 5 photos, 16–47 MP)

| Tool | Format | Score | Median size | vs source | Median time |
|---|---|---:|---:|---:|---:|
| sharp | AVIF | 82.7 | **85 KB** | 99.4% | 785 ms |
| squoosh | AVIF | 81.8 | 88 KB | 99.3% | 2543 ms |
| crustyimg | AVIF | 81.7 | 123 KB | 99.1% | 2809 ms |
| ImageMagick | AVIF | 81.1 | 162 KB | 97.4% | 629 ms |
| cwebp | WebP | 82.4 | 166 KB | 98.8% | **314 ms** |
| _crustyimg `web` (default)_ | AVIF | 75.1 | 47 KB | 99.5% | 2343 ms |

sharp wins size and beats crustyimg on the clock by ~4×. crustyimg lands in the
middle of the AVIF pack on size and last on speed. (ImageMagick covers 4 of the 5
photos here — it errored on the 47 MP Leica; see the caveats.)

### Medium photos (2–12 MP: 2 photos, 8 MP)

| Tool | Format | Score | Median size | vs source | Median time |
|---|---|---:|---:|---:|---:|
| sharp | AVIF | 82.1 | **176 KB** | 89.1% | 543 ms |
| squoosh | AVIF | 82.6 | 193 KB | 88.2% | 1985 ms |
| crustyimg | AVIF | 81.6 | 201 KB | 87.8% | 4418 ms |
| ImageMagick | AVIF | 81.6 | 215 KB | 87.0% | 353 ms |
| cwebp | WebP | 82.2 | 268 KB | 83.9% | **273 ms** |
| _crustyimg `web` (default)_ | AVIF | 75.4 | 132 KB | 91.9% | 3954 ms |

### Small photos (< 2 MP: 1 photo, 0.7 MP)

| Tool | Format | Score | Size | vs source | Time |
|---|---|---:|---:|---:|---:|
| sharp | AVIF | 81.8 | **198 KB** | 86.4% | 284 ms |
| crustyimg | AVIF | 81.6 | 203 KB | 86.1% | 992 ms |
| ImageMagick | AVIF | 82.1 | 206 KB | 85.9% | 96 ms |
| squoosh | AVIF | 83.0 | 218 KB | 85.1% | 835 ms |
| cwebp | WebP | 83.5 | 240 KB | 83.5% | **73 ms** |
| _crustyimg `web` (default)_ | AVIF | 75.2 | 154 KB | 89.5% | 967 ms |

## Results — every photo

Nothing hidden — all 8 photos, matched to the band. Size in KB, time in ms.

| Photo | MP | crustyimg | sharp | ImageMagick | squoosh | cwebp (WebP) |
|---|---:|---|---|---|---|---|
| DSC_0163 | 0.7 | 203 KB · 81.6 · 992 ms | **198 KB** · 81.8 · 284 ms | 206 KB · 82.1 · 96 ms | 218 KB · 83.0 · 835 ms | 240 KB · 83.5 · 73 ms |
| IMG_3855 | 7.8 | 282 KB · 81.4 · 5282 ms | 274 KB · 82.7 · 671 ms | **268 KB** · 80.9 · 379 ms | 290 KB · 82.6 · 2115 ms | 329 KB · 80.9 · 284 ms |
| DSCF1154 | 8.0 | 120 KB · 81.8 · 3554 ms | **79 KB** · 81.6 · 416 ms | 163 KB · 82.2 · 327 ms | 95 KB · 82.5 · 1854 ms | 207 KB · 83.5 · 261 ms |
| DSCN3478 | 15.9 | 371 KB · 81.7 · 3945 ms | 380 KB · 83.3 · 855 ms | **348 KB** · 81.5 · 467 ms | 422 KB · 83.2 · 2192 ms | 424 KB · 83.3 · 298 ms |
| DSC_0974 | 24.0 | 155 KB · 81.0 · 2809 ms | **109 KB** · 79.0 · 1004 ms | 157 KB · 81.2 · 616 ms | 181 KB · 81.5 · 2114 ms | 185 KB · 80.8 · 288 ms |
| DSC_2011 | 24.2 | 123 KB · 82.7 · 3226 ms | **85 KB** · 80.7 · 785 ms | 167 KB · 80.8 · 695 ms | 88 KB · 81.2 · 2797 ms | 166 KB · 82.4 · 336 ms |
| DSC_9952 | 24.2 | 37 KB · 81.6 · 2575 ms | 27 KB · 82.7 · 483 ms | 105 KB · 81.1 · 641 ms | **21 KB** · 81.8 · 2543 ms | 65 KB · 82.3 · 314 ms |
| L1024678 | 46.7 | 64 KB · 83.1 · 2382 ms | 48 KB · 83.6 · 606 ms | — (see caveats) | **48 KB** · 82.9 · 3056 ms | 98 KB · 82.8 · 439 ms |

_Each cell: output size · matched SSIMULACRA2 · median time. Bold = smallest AVIF
for that photo. sharp is smallest on 4 of 8, ImageMagick on 2, squoosh on 2 (the
47 MP row is effectively a tie — squoosh 47,730 bytes to sharp's 47,907).
crustyimg is never the smallest, but on every photo it's within ~1.7× of the
smallest — and on one (DSCN3478) it edges out sharp._

## Reading the results honestly

**crustyimg is slower on the clock, because it's single-threaded.** It runs 3–9×
slower than sharp and 4–14× slower than ImageMagick, which use every core. This is
a real cost of a design choice: crustyimg is a synchronous, single-threaded engine with no async
runtime — the same code path that has to run single-threaded in the browser. To
show it's threading and not a slow encoder, here is crustyimg against sharp **pinned
to one thread** (`VIPS_CONCURRENCY=1`). These are the *same encodes* as the tables
above — each tool keeps the exact quality setting it matched there, so the only
thing that changes is the thread count:

| Photo | MP | crustyimg (1 core) | sharp (1 thread) |
|---|---:|---:|---:|
| DSC_0163 | 0.7 | **999 ms** · 81.6 | 1322 ms · 81.9 |
| IMG_3855 | 7.8 | 5316 ms · 81.4 | **4324 ms** · 82.7 |
| DSCF1154 | 8.0 | 3625 ms · 81.8 | **2497 ms** · 81.6 |
| DSCN3478 | 15.9 | **3943 ms** · 81.7 | 4811 ms · 83.4 |
| DSC_0974 | 24.0 | **2805 ms** · 81.0 | 3139 ms · 78.7 |
| DSC_2011 | 24.2 | **3232 ms** · 82.7 | 3868 ms · 80.6 |
| DSC_9952 | 24.2 | 2560 ms · 81.6 | **2001 ms** · 82.8 |
| L1024678 | 46.7 | 2404 ms · 83.1 | **1666 ms** · 83.5 |

_Time · matched SSIMULACRA2, bold = faster. The score is shown because "at matched
quality" has to stay true here too: holding the quality setting fixed, sharp's
one-thread output is not byte-identical to its 14-thread output (libaom's
threading changes the bitstream — 0.3–1.7% fewer bytes), so its score moves by up
to 0.27 points. crustyimg's outputs are byte-for-byte identical across the two
runs, which is what you'd expect from a single-threaded encoder and a useful check
that nothing else changed._

Per core the two trade wins — crustyimg is faster on four of the eight, slower on
four, with margins from 12% to 45% either way. The wall-clock gap in the main
tables is almost entirely that sharp uses 14 cores and crustyimg uses one.

**crustyimg's AVIF files are competitive but not the smallest.** At matched
quality they run roughly parity-to-50% larger than sharp's. crustyimg encodes AVIF
with a fast rav1e preset chosen for interactive and in-browser use; that trades
some compression for encode simplicity. It's in the same order of magnitude as
libvips and clearly ahead of ImageMagick's AVIF on several photos — it's just not
the size champion.

**crustyimg's one-command default is tuned smaller, not bigger.** `crustyimg web`
(one command, no dials) uses that fast-AVIF preset and lands at **75 SSIMULACRA2**
median across the corpus (73.5–79.0) — still "high", but a real notch below the 82
band the table matches everyone to, and well below "visually lossless" (~90).
That's a deliberate size/speed trade for a sensible default, not a defect: at that
setting `web` produces the smallest files of any crustyimg row (a median 98%
smaller than the source) and is a touch faster. Concretely, `web`'s preset is
byte-for-byte `convert --format avif -q 80`, and the matched-quality rows tune
crustyimg *up* to `-q 85`–`-q 92` to reach the band, so the quality comparison is
fair. If you want the smaller default, that's what `web` gives you out of the box.

**WebP is a different weight class.** cwebp at matched quality is bigger than every
AVIF encoder here — from ~1.2× the smallest AVIF on a couple of photos to ~3.0× on
detailed ones — because that's the format, not the tool. It's fast and universal;
if you need AVIF's size, you need AVIF.

**ImageMagick is fast but the least size-efficient, and less tolerant of odd
inputs.** Its AVIF is quick (libaom threads internally) but often the largest at
matched quality — on one 24 MP photo it was 105 KB where the others were 21–37 KB.
It also refused the 47 MP Leica outright ("Incorrect data in iCCP"): that file
carries a malformed embedded colour profile, which crustyimg, sharp, and squoosh
all read without complaint. It's excluded from ImageMagick's row for that photo
rather than papered over.

## What crustyimg is for

The benchmark says crustyimg isn't the smallest or the fastest. Here's the case it
*does* make, and none of it shows up in a size column:

- **One static binary, zero system dependencies.** Pure Rust, including the AVIF
  encoder (`cargo install crustyimg --features avif`). No libvips to link, no
  ImageMagick to install, no native Node addon to compile, no Python. sharp is
  excellent but pulls a native libvips addon; ImageMagick is a large C toolchain;
  `@squoosh/cli` is archived and won't start on a current Node. crustyimg is one
  download or one `cargo install`.
- **Quality is a number the tool gives you.** The SSIMULACRA2 score this whole
  comparison uses to keep everyone honest is built into crustyimg — `web` and
  `optimize` report it as part of the encode, and `diff --fail-under` exits
  non-zero so you can gate a build on it. Other tools can measure quality (`magick
  compare -metric DSSIM`, `cwebp -print_ssim`), but as a separate step against a
  reference you supply, and not with SSIMULACRA2.
- **It reads RAW.** `.dng`, `.cr2`, `.nef`, `.arw` and more, via the camera's
  embedded preview. sharp and squoosh can't open those at all.
- **The same engine runs in the browser.** The [demo](https://jysf.github.io/crustyimg/)
  is this exact code compiled to WebAssembly — which is *why* it's single-threaded.

Competitive compression, a real quality metric, RAW input, and the same code
everywhere, in one dependency-free binary. That's the trade against a few hundred
milliseconds and a few dozen kilobytes.

## Reproduce it yourself

The harness drives the real binaries and scores every output with the same metric
— no numbers are typed by hand. Install the competitors (they are not crustyimg
dependencies), then run it on your own photos:

```sh
# competitors (macOS shown; any OS works)
npm i -g sharp-cli @squoosh/cli      # @squoosh/cli is archived — run it on Node 16
brew install imagemagick webp

# crustyimg with the AVIF encoder
cargo install crustyimg --features avif        # or: cargo build --release --features avif

# run the comparison on your corpus
just bench-compare --corpus /path/to/your/photos
# or directly:
python3 scripts/bench-compare.py --corpus /path/to/your/photos \
  --squoosh-node ~/.nvm/versions/node/v16.20.2/bin/node   # a Node 16 for squoosh
```

The harness prints the same size / speed / matched-score numbers, per photo and per
size bucket (`--json` for machine-readable output). A tool that isn't installed is
**labelled "NOT RUN"**, never silently dropped. It also measures every output and
refuses the run (exit 3) if a tool didn't get the same downscale as the others —
so pointing it at portrait photos, or at a tool version whose resize flags have
changed, gets you an error rather than a quietly wrong table.
`--self-test` checks that guard on its own, with no corpus and no tools installed.

Scores and sizes are deterministic: two identically-configured runs here
reproduced all 141 of them exactly. Wall-times move, because they're measurements
— a median of 1.6% between those two runs, and up to ~20% on the sub-second ones,
where a few tens of milliseconds is a large share. Point `--corpus` at your own
photos and you'll get your own numbers — which is the whole idea.

---

_These numbers are from one machine and one corpus; the exact bytes depend on the
photos. What's reproducible is the shape of the result and the method — run the
command above and check. For the crustyimg-only offline benchmark over the
committed sample corpus, see `just bench`._
