# Benchmark corpus — provenance & license (SPEC-088)

These images are **100% synthetic** — generated deterministically from pure math
by [`examples/gen_bench_corpus.rs`](../../examples/gen_bench_corpus.rs). There is
**no camera capture, no EXIF/GPS, and no private data** of any kind. They exist so
`just bench` can measure `web`/`optimize` savings, time, and quality over a
committed, reproducible corpus **offline** — with nothing to license and nothing
to leak.

## License

Dedicated to the public domain under **CC0-1.0** (no rights reserved). Being
machine-generated from a committed formula, they are also trivially reproducible:

```sh
cargo run --example gen_bench_corpus        # rewrites this directory
```

## Contents (photo × graphic, small × large)

| file                | kind    | size    | format | why |
|---------------------|---------|---------|--------|-----|
| `photo_small.jpg`   | photo   | 256×256 | JPEG   | lossy-family source (gradient + sinusoids) |
| `photo_large.jpg`   | photo   | 512×512 | JPEG   | 4× pixels — shows `web`'s size-insensitivity |
| `graphic_small.png` | graphic | 256×256 | PNG    | flat-colour blocks → lossless/graphic branch |
| `graphic_large.png` | graphic | 512×512 | PNG    | larger graphic (lossless WebP/PNG wins) |

Kept deliberately small so the repo stays lean; the encode work, not the source
bytes, is what the harness measures.

## What the smoke numbers show (and honestly don't)

The graphics compress hard (lossless WebP, ~86–94% smaller). The **synthetic
photos are smooth enough to already be near-optimal as JPEG, so `web`/`optimize`
correctly _pass them through_ (0%, never-bigger)** — a real, useful path to
exercise, not a bug. Two honest caveats of a *smoke* corpus:

* Real-detail photos (where a modern codec beats the JPEG source) are what the
  maintainer runs via `--corpus`; baking that detail into a committed file means
  incompressible noise and a bloated repo.
* `web`'s downscale advantage over `optimize` only shows above the 2048px long-edge
  bound. These images are ≤512px, so here `web` == `optimize`; the size-insensitive
  win appears on the real (large) corpus.

## Real-corpus numbers

This synthetic set is a **smoke corpus** — enough to prove the harness and produce
representative ratios. For the honest launch numbers (SPEC-083 / BENCHMARKS.md),
point the harness at a real corpus **without committing those photos**:

```sh
python3 scripts/bench.py --corpus /path/to/real/photos --bin ./target/release/crustyimg
```

The real photos never enter git (privacy / `no-secrets-in-code`); only this
synthetic corpus is committed.
