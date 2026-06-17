---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-019
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-16
supersedes: null
superseded_by: null

affected_scope:
  - src/quality/**
  - src/cli/**
  - Cargo.toml

tags:
  - quality
  - perceptual
  - ssimulacra2
  - auto-quality
  - jpeg
  - metric
  - permissive
---

# DEC-019: Perceptual auto-quality — adopt `ssimulacra2`; SSIMULACRA2 metric, target presets, and the quality-search policy

## Decision

crustyimg adopts **SSIMULACRA2** as its perceptual quality metric, via the
**`ssimulacra2` crate (v0.5.1, BSD-2-Clause, pure-Rust)** as a **default**
dependency (not feature-gated; permissive, so DEC-018 / `no-agpl-default-deps` is
satisfied — confirmed by `just deny`). On top of it, `shrink` gains an **opt-in
perceptual auto-quality** mode (`--target <preset>` / `--ssim <score>`) governed by
the following durable policy:

1. **Metric.** SSIMULACRA2 score (`f64`), where **higher = more similar** and
   **~100 = visually identical**. A score is computed between a *reference* image
   and a *distorted* candidate of the **same dimensions** via
   `ssimulacra2::compute_frame_ssimulacra2(reference, distorted)`, feeding two
   `ssimulacra2::Rgb` values built from 8-bit sRGB pixels normalized to `0.0..=1.0`
   with `TransferCharacteristic::SRGB` + `ColorPrimaries::BT709`.

2. **Target presets (tunable constants, anchored to the metric author's published
   interpretation).** `visually-lossless = 90.0`, `high = 70.0`, `medium = 50.0`.
   `--ssim <N>` takes an explicit score in `0.0..=100.0`. These are *constants in
   code*, not gospel — revisit as we gather real outputs (hence confidence 0.8).

3. **Search policy.** For a JPEG output, **binary-search the integer encoder
   quality** over `1..=100` for the **lowest quality whose score ≥ the target**
   (the smallest file that still clears the bar). Each candidate is encoded to JPEG
   **in memory** (the DEC-016 `JpegEncoder::new_with_quality` path), decoded back,
   and scored against the reference (the already-resized original). The search is
   **capped at 8 distinct candidate evaluations**, **memoizes** scores, and runs on
   the **already-decoded** image (DEC-002 decode-once holds; candidates never touch
   disk). If **no** quality in range meets the target (e.g. an unreachable
   `--ssim 100`, or a pathologically noisy tiny image), the search returns the
   **highest quality (best-effort)** with `met_target = false` — it never errors for
   "unreachable target."

4. **Scope & precedence.** v1 is **JPEG-only** and lives on `shrink`. Auto-quality
   is **opt-in**: without `--target`/`--ssim`, `shrink` keeps its fixed default
   quality (80, DEC-016). `--target`, `--ssim`, and `-q` are **mutually exclusive**
   (you either pin a quality or search for one). For a **non-JPEG** output format
   the target is **ignored** (encoder default is used) — exactly mirroring DEC-016's
   "`-q` is ignored for lossless formats." The search itself is **encoder-agnostic**
   (generic over a `FnMut(u8) -> Result<f64, _>` scorer) so SPEC-017's `--max-size`
   budget and SPEC-018/019's AVIF/WebP quality reuse it unchanged.

The metric + search live in a self-contained `src/quality/` module that depends only
on `::image`, `ssimulacra2`, `thiserror`, and `std` (no clap/cli/sink/fs) — the same
layering discipline as `src/operation`.

## Context

STAGE-008's flagship is "**set the look, not the quality number**" — the frontier
paradigm (oavif, jpegli, squoosh auto) of encoding to a *perceptual target* rather
than a magic quality integer. crustyimg already had the JPEG quality-encode path
(DEC-016: `-q` → `JpegEncoder::new_with_quality`, threaded `run_pixel_op →
Sink::write → encode_to_bytes`) and the `shrink` command (SPEC-013). What was
missing was (a) a way to *measure* perceptual quality and (b) a loop to *search* for
the encoder setting that hits a target. Both had to be chosen under the now-live
license gate (DEC-018), which rules out the best AGPL options as defaults.

A 2026-06-15/16 research pass plus a docs.rs verification (2026-06-16) established:
- **SSIMULACRA2** is the current best open perceptual metric for still-image
  compression (correlates with human judgement far better than PSNR/SSIM/MS-SSIM),
  and the **`ssimulacra2` crate is pure-Rust and BSD-2-Clause** — permissive,
  default-buildable, deps (yuvxyb/num-traits/thiserror/etc.) all permissive. It is a
  *metric* crate (no decode/encode/resize), so it does not violate
  `single-image-library`.
- A single SSIMULACRA2 comparison is well under ~100ms on typical web images, so a
  capped 6–8-step binary search is sub-second — fast enough for an interactive CLI.
- `visually-lossless ≈ score 90` is the metric author's stated rule of thumb (with
  ~70 "high", ~50 "medium"); good enough as a shipped default, explicitly tunable.

This is the metric foundation the rest of the roadmap leans on: the byte-budget
(SPEC-017), AVIF/WebP quality (SPEC-018/019), the equal-quality basis for every
future benchmark claim, and a later `diff`/visual-regression command all reuse it.

## Alternatives Considered

- **Metric: PSNR / SSIM / MS-SSIM (e.g. `dssim`).**
  - Why rejected: these correlate poorly with human perception for modern
    compression (the whole reason SSIMULACRA2 exists). `dssim` is also a *different*
    metric (SSIM-family) and not the frontier choice. SSIMULACRA2 is the point.

- **Metric: an AGPL perceptual encoder/metric (jpegli-rs / zenjpeg / butteraugli
  via an AGPL crate).**
  - Why rejected: **DEC-018 / `no-agpl-default-deps`** — static linking would
    relicense the whole binary. `ssimulacra2` (BSD-2) gives the perceptual signal
    without the copyleft.

- **Search: encode-the-whole-range / linear scan of all 100 qualities.**
  - Why rejected: ~100 encode+decode+score passes per image is needlessly slow when
    the score is (near-)monotonic in quality. Binary search hits the answer in
    ≤ 8 evaluations.

- **Search: a fixed quality "good enough" table per format (no scoring).**
  - Why rejected: that's just DEC-016's default-80 with extra steps — it doesn't
    adapt to image content, which is the entire value of perceptual targeting (a
    flat UI screenshot and a noisy photo need very different qualities to look
    equally good).

- **Make auto-quality the default for `shrink` (no opt-in).**
  - Why rejected: it changes `shrink`'s long-standing behavior and adds per-image
    search cost to every invocation. Opt-in (`--target`/`--ssim`) keeps the simple
    case fast and unsurprising; the fixed default (80) still serves "just shrink it."

- **Error when the target is unreachable.**
  - Why rejected: surprising and unergonomic — "give me visually-lossless" should
    still produce the *best available* output, not fail. Best-effort + a
    `met_target=false` signal (loggable under `-v`) is friendlier; a strict mode can
    come later if wanted.

- **Build the search around a concrete encoder (not generic over a scorer).**
  - Why rejected: SPEC-017's size budget and SPEC-018/019's AVIF/WebP need the same
    search. A `FnMut(u8) -> Result<f64, _>` scorer keeps the loop reusable AND makes
    it deterministically unit-testable with a synthetic monotonic function.

## Consequences

- **Positive:** crustyimg can target a *look* ("visually-lossless", "high") or a
  precise SSIMULACRA2 score, choosing the smallest JPEG that clears it —
  content-adaptive, sub-second, pure-Rust, permissive. The metric + generic search
  are the reusable foundation for `--max-size`, AVIF/WebP quality, benchmarks, and a
  future `diff`. `shrink`'s default behavior is unchanged (opt-in).
- **Negative:** a new default dependency (`ssimulacra2` + its transitive tree) — but
  permissive and gate-checked. The search costs up to ~8 in-memory encode/decode
  passes per image (acceptable, capped; one extra final re-encode at the chosen
  quality is not yet cached). The preset→score anchors (90/70/50) are judgement
  calls that may need tuning (confidence 0.8). SSIMULACRA2 needs images large enough
  for its internal 6× downscale — pathologically tiny inputs may score
  unreliably/error (surfaced as a typed `QualityError`, not a panic).
- **Neutral:** JPEG-only and `shrink`-only in v1 by design; generalization is
  deferred to later specs, not blocked. Metadata is still dropped on the pixel-lane
  re-encode (DEC-003) regardless of quality — unchanged.

## Validation

Right if: `shrink photo.jpg --target visually-lossless -o out.jpg` produces a JPEG
that scores ≥ 90 and is smaller than a max-quality encode, chosen in a capped
search; a lower `--ssim`/`--target` yields a smaller file than a higher one on the
same image; `--target`+`--ssim` (or either + `-q`) is a usage error (exit 2); a PNG
output ignores the target without error; `cargo deny check licenses` stays green
with `ssimulacra2` added. Revisit if: the preset→score anchors prove wrong on real
content (retune the constants); SSIMULACRA2's cost becomes a bottleneck on large
batches (cache/parallelize the search — STAGE-005); a `--strict`/`--json` need
emerges; or AVIF/WebP land and the per-encoder quality ranges differ enough to need
per-format search bounds.

## References
- Related specs: SPEC-016 (this — the flagship), SPEC-013 (`shrink` + DEC-016
  quality encode this drives), SPEC-017 (`--max-size`, reuses the search), SPEC-014
  (the `run_pixel_op` `forced_format` param the `auto` param mirrors).
- Related decisions: DEC-016 (encode quality / the JPEG `new_with_quality` path),
  DEC-018 (`no-agpl-default-deps` — why a permissive metric; the `just deny` gate),
  DEC-004 (`pure-rust-codecs-default` — `ssimulacra2` is pure-Rust, default-able),
  DEC-002 (decode-once — candidates stay in memory), DEC-015 (format precedence /
  the search runs only for JPEG outputs), DEC-007 (typed errors → exit codes).
- Related constraints: `no-agpl-default-deps`, `no-new-top-level-deps-without-decision`,
  `single-image-library` (a metric crate is not a second pixel library),
  `decode-once-no-per-op-disk`, `ergonomic-defaults`.
- External docs: https://docs.rs/ssimulacra2/0.5.1/ssimulacra2/ ,
  SSIMULACRA2 metric (Cloudinary / Jon Sneyers):
  https://github.com/cloudinary/ssimulacra2
