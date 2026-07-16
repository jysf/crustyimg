# Benchmark corpus — provenance & license (SPEC-088)

The corpus `just bench` measures. Everything here is **license-clean** — four
machine-generated images plus one **CC0 (public-domain) photograph** — so the set
can be committed, re-run offline, and audited by anyone, with nothing to license
and nothing to leak. **No private photo, no camera EXIF, no GPS** is present in
any of it (`bench_corpus_is_license_clean` asserts this through the real decoder).

## Contents

Every `class` below is **the engine's own verdict**, obtained by driving
`crustyimg optimize <file> --json` and reading the `class` field — not a label we
assigned. An earlier version of this corpus called the gradients "photo"; the
classifier disagreed, so the files were renamed to match the tool.

| file | class (per the tool) | dims | format | what it exercises |
|------|----------------------|------|--------|-------------------|
| `photo_forest_cc0.jpg` | `photograph` | 800×532 | JPEG | the **lossy/AVIF branch** — AVIF wins (~30% smaller, SSIMULACRA2 ≈ 81) |
| `gradient_small.jpg` | `graphic-logo` | 256×256 | JPEG | never-bigger **passthrough** (0%): already JPEG-optimal |
| `gradient_large.jpg` | `graphic-logo` | 512×512 | JPEG | same, at 4× pixels |
| `graphic_small.png` | `graphic-logo` | 256×256 | PNG | the **lossless-WebP** branch (~86% smaller) |
| `graphic_large.png` | `graphic-logo` | 512×512 | PNG | same, at 4× pixels (~94% smaller) |

The smooth synthetic gradients measure a flat-region ratio of 1.00, which routes
them to `graphic-logo` — synthetic math does not make a photograph, which is
exactly why a real photo is committed alongside them.

## Provenance & license

### `photo_forest_cc0.jpg` — real CC0 photograph

* **Source:** ["Petite Hesse spruce forest undergrowth, Waimes, 2023"](https://commons.wikimedia.org/wiki/File:Petite_Hesse_spruce_forest_undergrowth,_Waimes,_2023.jpg)
  on Wikimedia Commons.
* **Author:** Wikimedia Commons user
  [DimiTalen](https://commons.wikimedia.org/wiki/User:DimiTalen) — own work,
  photographed 2023-10-21.
* **License:** **CC0-1.0** (Creative Commons Zero, Public Domain Dedication) — no
  rights reserved; Commons reports `AttributionRequired = false`. Credited here
  anyway as good practice, not obligation.
* **Retrieved:** 2026-07-16, via the Commons API (license verified from the API's
  own `extmetadata`, not assumed from the category).
* **Modifications:** downscaled from 6016×4000 to 800×532 and re-encoded as JPEG
  (q≈65) to keep the repo lean, then **all metadata stripped** with
  `crustyimg meta strip`. The committed file carries a bare JFIF header and
  nothing else: no EXIF, no GPS, no ICC, no XMP, no comment (verified by
  `crustyimg info --json` and a raw JPEG-segment scan).
* **Why it exists:** it is the only image whose pixels the classifier calls
  `photograph`, so it is the only one that reaches the lossy/AVIF path. Without
  it `just bench` builds `--features avif` and never encodes a single AVIF.

### The four synthetic images

Generated deterministically from pure math (`sin`/`cos`, no input files, no
network) by [`examples/gen_bench_corpus.rs`](../../examples/gen_bench_corpus.rs).
There is no capture and no third-party content, so there is nothing to license;
they are likewise placed in the public domain under **CC0-1.0**. Reproduce them
byte-for-byte with:

```sh
cargo run --example gen_bench_corpus   # rewrites the four synthetic files only
```

That command does **not** write `photo_forest_cc0.jpg` (a real photo cannot be
generated from a formula); it is committed as-is and left untouched.

## What the smoke numbers show — and honestly don't

`just bench` prints these same caveats as a footer, so nobody has to read this
file to learn the limits:

* **`web` == `optimize` on every committed row.** `web`'s whole point is the
  downscale to a 2048px long edge; every committed image is well under that, so
  the downscale never fires and the two verbs converge. STAGE-030's headline
  (`web` ~98% and size-insensitive vs `optimize` ~24%) is **invisible here by
  construction** — a >2048px photo of real foliage cannot be both committed and
  lean (it lands in the MBs, or in the KBs while looking terrible).
* **Four of five rows never reach AVIF.** They classify `graphic-logo` and take
  lossless-WebP or passthrough. That is correct behaviour and worth
  regression-testing — but it is not the pitch.
* **Timings are wall-clock measurements**, so they vary run to run; savings and
  scores are deterministic.

What this corpus *does* buy: proof that the harness works, that it consumes the
CLI's own `--json` (so report and bench cannot drift), and live coverage of all
three decision branches — AVIF-lossy, lossless-WebP, and never-bigger passthrough.
It is scaffolding, not evidence.

## Real-corpus numbers

**Every headline number for BENCHMARKS.md (SPEC-083) must come from a real
corpus**, which the harness reads without those photos ever entering git:

```sh
just bench --corpus /path/to/real/photos
# or: python3 scripts/bench.py --corpus /path/to/real/photos --bin ./target/release/crustyimg
```

The footer caveat automatically disappears when `--corpus` points somewhere other
than this directory — those numbers are real, and are not hedged.
