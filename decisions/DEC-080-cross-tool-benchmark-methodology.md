---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-080
  type: decision
  confidence: 0.85
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

created_at: 2026-07-20
supersedes: null
superseded_by: null

affected_scope:
  - "scripts/bench-compare.py"
  - "BENCHMARKS.md"

tags:
  - benchmark
  - cross-tool
  - iso-quality
  - ssimulacra2
  - avif
  - methodology
  - reproducibility
  - corpus
  - sharp
  - squoosh
  - imagemagick
---

# DEC-080: cross-tool benchmark methodology (BENCHMARKS.md)

## Decision

`BENCHMARKS.md` compares crustyimg against sharp, ImageMagick, `@squoosh/cli`
(and cwebp, labelled WebP-only) on an **iso-quality** basis: every tool runs the
**same web-ready pipeline** — downscale the long edge to ≤ 2048 px, then encode
AVIF — and the harness (`scripts/bench-compare.py`) drives each tool via a fixed
per-tool quality grid to the grid point whose **SSIMULACRA2 score lands nearest
82** ("high"), then reports **bytes, wall-time, and the matched score per size
bucket**. The single scorer for every tool is `crustyimg diff` (SSIMULACRA2),
run against **that tool's own lossless 2048 px downscale** so the quality column
measures encode fidelity, resampler-neutral. The methodology, grids, tool
versions, machine, and corpus provenance are fixed **before** the numbers are
read — no post-hoc tuning to win.

## Context

The launch's headline ("`crustyimg web` makes photos a median 98 % smaller in
2–5 s") is exactly the claim r/rust and HN will re-run and try to break. A
size comparison without matched quality is meaningless (a q40 file is "smaller"
because it's worse), and an unfair competitor invocation discredits the whole
launch. SPEC-083 needs a comparison where a skeptic can (a) see the quality is
equal, measured by one metric, and (b) re-run it and get the same table.

This extends the committed-bench discipline (DEC-074, `just bench`,
`scripts/bench.py`, offline/reproducible/`--corpus`) from crustyimg-only to a
cross-tool comparison. The crustyimg side is the shipped **0.5.0** engine.

**One load-bearing fact drives the whole design: AVIF is a compile-time feature
(`--features avif`), OFF in every distributed binary (crates.io / Homebrew /
GitHub Releases build default features only — DEC-052 / dist-workspace.toml).**
The flagship `web`→AVIF path therefore requires `cargo install crustyimg
--features avif` (still pure Rust — ravif/rav1e, no system libs) — the same
build the wasm demo already ships. BENCHMARKS.md benchmarks that build and
states the requirement plainly; hiding it would be the dishonesty the doc exists
to avoid.

### Design-time calibration (probe, before fixing the method)

A throwaway probe on two corpus photos confirmed the method is viable:

- The quality→SSIMULACRA2 mapping is smooth and monotonic for every tool, so the
  82 band is reliably hittable by a fixed grid (no fragile search / interpolation):
  crustyimg `-q` ≈ 90–91, sharp `-q` ≈ 77–80, ImageMagick `-quality` ≈ 68–70,
  squoosh `cqLevel` ≈ 12–14, cwebp `-q` ≈ 92 (WebP).
- `crustyimg web`'s fixed fast-AVIF is byte-identical to `convert --format avif
  -q 80` — not `-q 85`, which the design-time probe got wrong. Measured over the
  full corpus it lands **SSIMULACRA2 73.5–79.0, median 75.2**. (The first pass
  recorded "== `-q 85`, ≈ 79–82" and used it to justify 82 as "crustyimg's real
  operating point". Both halves were wrong, and the error flattered the setup: it
  made the band look like crustyimg's home turf when it is ~7 points above it.)
- **Why the band is 82 anyway.** Not because it is crustyimg's default — it
  isn't. 82 is the band every tool's quality grid can bracket, and it is a quality
  a reader would actually ship. Anchoring on crustyimg's true default (~75) would
  have dragged four competitors down to a lower-quality band picked to suit
  crustyimg's fast preset; 82 instead tunes crustyimg *up* (`-q 85`–`-q 92`),
  away from the setting its preset is built for. If the choice biases anything it
  biases against crustyimg, which is the right direction for a doc whose purpose
  is credibility. crustyimg's actual default is reported separately as its own
  `web` row rather than smuggled in as the band.
- `crustyimg diff` **requires identical dimensions** (`cannot compare images of
  different dimensions`) — so each tool is scored against a downscale it produced
  at the same target size, which also makes the score resampler-neutral (each
  tool judged on its own encode). Re-measured with every tool's downscale
  aspect-correct: the tools' lossless downscales score **91–94** against each
  other on three of four sampled photos, but only **~82** for sharp against the
  others on one 24 MP photo. So the resampler is *not* uniformly a second-order
  effect, and own-reference scoring is doing real work — against a shared
  reference that photo's resampler gap would have been charged to sharp's encoder.
- `crustyimg`'s native perceptual search (`optimize --ssim`) does **not** apply to
  AVIF (it needs to decode each candidate; warns and falls back to fixed quality),
  so the crustyimg grid uses `convert --format avif -q`, not `--ssim`.
- crustyimg refuses to overwrite an existing output (exit 5) without `-y`; the
  harness writes each run to a unique path.

## Alternatives Considered

- **Option A: "smallest file wins" (no quality control).**
  - What it is: run each tool at a fixed `-q`, rank by output bytes.
  - Why rejected: it's the cherry-pick the doc exists to refute — a smaller file
    at lower quality isn't a win. Quality must be equalized and shown.

- **Option B: trust each tool's self-reported `-q` as "matched quality".**
  - What it is: encode everything at, say, `-q 80` and call it iso-quality.
  - Why rejected: a q80 JPEG, q80 AVIF, and cqLevel-20 AVIF are not the same
    perceptual quality. The whole point is one *measured* metric across tools.

- **Option C: honest scatter (fixed sensible setting per tool, plot size-vs-quality).**
  - What it is: the spec's fallback framing — no single winner, show the trade.
  - Why not chosen (but partially kept): weaker claim, and the probe showed the
    band *is* hittable, so iso-quality (the stronger claim) is available. The
    scatter's honesty is preserved by still printing every tool's matched score
    next to size+time (the reader sees where each tool actually landed, and the
    per-effort size/time trade is visible in the table).

- **Option D: full-resolution encode shootout (no downscale).**
  - What it is: encode the full-res original to AVIF, compare at native pixels.
  - Why not the primary: it's a clean encoder micro-comparison but it is *not*
    the product — `web` downscales, and most of the 98 % comes from the
    downscale. Benchmarking only full-res would undersell the real workflow and
    inflate everyone's time. Kept as an optional secondary cross-check.

- **Option E (chosen): iso-quality at a fixed 82 band over the real web-ready
  pipeline (downscale ≤2048 + AVIF), one scorer, per-tool own-reference scoring,
  fixed grids picked-nearest-82.**
  - Why selected: it's the strongest claim the data supports *and* honest — same
    inputs, same measured quality, same scorer, exact commands and versions
    stated, reproducible (only wall-times vary), and it mirrors what the tool
    actually does.

## The fixed method (frozen here)

- **Corpus.** `_incoming0` — 8 real photographs, 0.7–47 MP, six camera models
  across four brands (verified from EXIF: Fujifilm X100F; Nikon COOLPIX P1100,
  D3300, D750; Leica Q2 Monochrom; Apple iPhone 15 — note: the design brief's
  "5 cameras / Sony" was wrong, corrected here from the files). `.JPG`/`.jpeg`/
  `.png`. Not committed (privacy + repo bloat, DEC-074); provenance and the size
  distribution are stated in the doc, and the harness takes `--corpus <dir>` so
  anyone points it at their own.
- **Pipeline (identical per tool):** downscale long edge to ≤ 2048 px (never
  upscale) → encode AVIF. cwebp is the same pipeline to WebP, labelled. Every
  tool says "long edge" differently, and two of the four dialects were wrong on
  the first pass, so the harness **measures every output** (reference and every
  grid point) against the source long edge and aspect ratio and **exits 3** if
  any tool's output departs. This check is load-bearing, not defensive: the
  quality column cannot catch a distorted downscale, because each tool is scored
  against its OWN reference — a squashed encode judged against a squashed
  reference still lands in the band. `--self-test` exercises the guard against
  known-good and known-bad shapes with no corpus and no tools installed.
- **Per-core (single-thread) comparison:** re-time the *same encodes* — the
  harness's `--q-from` reuses each tool's matched quality from the main run — with
  sharp pinned to `VIPS_CONCURRENCY=1`. Only the thread count changes, so the
  table answers "how much of the gap is threading" rather than comparing two
  different encoder settings. Re-picking the band under one thread would move
  sharp's quality (libaom's output shifts with thread count), which is the thing
  being controlled for.
- **Scorer:** `crustyimg diff A B` (SSIMULACRA2), one metric for all, B scored
  against A = **that tool's own lossless 2048 px downscale** (encode fidelity;
  same number `crustyimg web` reports).
- **Matched quality:** fixed per-tool grid; select the point with score nearest
  82 (tie → fewer bytes). Grids (calibrated to bracket the band):
  - crustyimg 0.5.0 `--features avif`: `convert --format avif -q {80,85,88,90,92,94}`
  - sharp-cli 5.2.0 (sharp 0.34.4): `-f avif -q {50,60,70,78,85}`
  - ImageMagick 7.1.2-27: AVIF `-quality {45,55,65,72,80}`
  - `@squoosh/cli` 0.7.2: `--avif cqLevel {23,18,14,10,6}` (lower = better).
    Its `--resize` takes **one** axis: given both `width` and `height` it stretches
    the image to that box instead of fitting inside it, so the harness constrains
    the long axis only and lets squoosh derive the other from the source aspect.
  - cwebp 1.6.0 (WebP-only, labelled): `-q {78,85,90,93,96}`
- **Tools + versions** pinned above. AVIF backends: crustyimg = ravif/rav1e
  (pure Rust); sharp = libvips/libaom; ImageMagick = libheif 1.23.1/libaom;
  squoosh = its own aom wasm. **@squoosh/cli 0.7.2 is archived and does not run
  on Node ≥ 18** — benchmarked under Node 16 (a finding, stated as context).
- **Machine:** Apple M4 Pro, 14 cores, macOS 26.5.2, 48 GB.
- **Threading (the speed caveat):** crustyimg is single-threaded; sharp/libvips,
  ImageMagick, and squoosh use multiple threads/workers by default. Wall-times
  are reported at each tool's default threading (what a user gets) with this
  stated prominently, so the speed column is read fairly.
- **Effort:** each tool runs at its own default speed/effort preset (crustyimg's
  `web` is a deliberately fast AVIF preset); effort is not normalized across
  encoders (the presets aren't comparable), it is disclosed, and the time column
  shows what each tool spent for its size.
- **Buckets:** small (< 2 MP), medium (2–12 MP), large (> 12 MP).
- **Reproducibility:** grids + nearest-82 selection + SSIMULACRA2 are
  deterministic; only wall-times vary run-to-run. Two full runs are captured and
  compared; the harness emits the table (no hand-edited numbers).

## Consequences

- **Positive:** a defensible, reproducible, equal-quality comparison a skeptic
  can re-run with one command; the quality column is one consistent metric, not
  each tool's marketing `-q`; the honest losses (single-thread wall-clock vs
  libvips; the fast-AVIF default's ~75 "high" trade; WebP's format disadvantage)
  are visible in the same table, not buried.
- **Negative:** the harness needs the competitors installed (node/npm + a Node-16
  shim for squoosh, brew ImageMagick, cwebp) and `--features avif` for the
  crustyimg side — heavier than the offline `just bench`. A single band means the
  table answers "at one shippable quality, how do these tools compare", not "at
  every quality" — printing each tool's matched score and its whole grid is the
  mitigation.
- **Neutral:** the numbers come from a private corpus, so they are reproducible
  in *shape* by anyone but exact bytes depend on the photos; the doc states this
  and ships the command, not a promise.

## Validation

Right if: verify re-runs `scripts/bench-compare.py --corpus <dir>` and the table
regenerates within wall-time noise (scores/bytes identical); every competitor
command in the doc runs at its pinned version; every crustyimg command runs on
the 0.5.0 `--features avif` binary; no README benchmark claim contradicts the
doc. Revisit if: a new AVIF encoder default shifts a tool off the band (recalibrate
the grid), a competitor version bumps its quality scale, or the crustyimg engine
changes its fast-AVIF operating point (re-anchor the band).

## References

- Related specs: SPEC-083 (this — BENCHMARKS.md), SPEC-088 (the committed
  crustyimg-only bench this extends), SPEC-082 (the README headline substantiated).
- Related decisions: DEC-074 (committed-bench policy, `--corpus`, offline/no-telemetry),
  DEC-069 (fast-AVIF default + two-regime quality — the ~80 "high" operating point),
  DEC-052 (why AVIF/HEIC stay off the distributed binary — the `--features avif` requirement),
  DEC-020 (AVIF behind an off-by-default feature).
- Scorer: `docs/cli-reference.md` §diff (SSIMULACRA2).
