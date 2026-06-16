# Handoff — decide STAGE-004 + the medium-term roadmap (fresh Opus session)

Paste the block below into a new Opus session in the `crustyimg` repo. Unlike the
prior handoff (which continued a build), this one's job is a **decision + design**:
pick what STAGE-004 is, then scaffold/design its first spec. It carries the
current state plus a large strategic-research synthesis (frontier scan,
competitive gaps, a GIF feature design, a benchmarking plan, new feature ideas,
and a proposed 6-month roadmap) gathered 2026-06-15/16.

The **orchestration mechanics** (design→build→verify→ship, model routing, the
hard-won gotchas, git/PR flow) are unchanged — read
`docs/sessions/2026-06-15-stage-003-continue-handoff.md` for the full process and
do NOT relearn its gotchas (push design before build; check branch before commit;
verify every named test exists; recover dropped subagents; branch-protection
merge dance; `just advance-cycle`/`archive-spec` mis-glob → ship by hand).

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust image CLI
being rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git
repo, remote `git@github.com:jysf/crustyimg.git`, `gh` authed).

**Orient first:** `AGENTS.md`; `projects/PROJ-001-crustyimg-mvp/brief.md`;
`docs/api-contract.md`; `docs/feature-exploration.md` + `docs/backlog.md` (the
project's own prior feature research — build on it, don't repeat it);
`guidance/constraints.yaml`; the decisions in `decisions/` (esp. DEC-002 op
trait, DEC-003 metadata dual-lane, DEC-004 codec policy, DEC-014 op-params,
DEC-015 format/exit-6, DEC-016 quality, DEC-017 ops-read-metadata); the shipped
`src/cli/mod.rs`, `src/operation/mod.rs`, `src/sink/mod.rs`. Read the prior
handoff for process. Run `just status` / `just roadmap`.

## Where we are (2026-06-16)

STAGE-001 (foundation, 7 specs), STAGE-002 (view/info, 2 specs), and **STAGE-003
(transform & output, 6 specs SPEC-010–015) are SHIPPED.** `main` clean, no open
PRs, 207 tests, 3-OS CI green, 16 DECs (DEC-002…017). License **MIT OR
Apache-2.0** (permissive — this is load-bearing, see Constraints). Live commands:
`view`, `info`, `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient`,
`apply` (single-input). ~5,300 LOC src / ~3,700 LOC tests, ~10 pure-Rust crates,
zero system deps by default (`viuer` behind a feature). `rayon`/`indicatif` and
the metadata-write crates (`img-parts`/`little_exif`) are NOT deps yet.

## THE DECISION: what is STAGE-004?

The original PROJ-001 plan has STAGE-004 = *compose & metadata* (watermark + the
EXIF lane), STAGE-005 = batch & recipes, STAGE-006 = hardening, STAGE-007 =
release. The user is **open to re-prioritizing** ("whatever moves cleanest and
fastest; the 004-vs-005 order doesn't matter; competitors are stale so there's no
race"). They want to spend ~1–5 weeks bulldozing through interesting
features/filters and want a ~6-month roadmap.

Three framings for STAGE-004 (pick one; recommendation follows):

- **Option A — Compose & metadata (the original plan).** `watermark` (image +
  text) + the container metadata lane (`strip`, `clean --gps`, `set`,
  `copy-metadata`) with **privacy-by-default + an EXIF audit-as-CI-linter**
  (`crustyimg audit *.jpg` → reports GPS/serial leaks, exits non-zero). Pulls in
  `img-parts`/`little_exif` (needs a DEC). Finishes the MVP feature set as scoped;
  delivers the #2 unserved competitive gap (verifiable privacy).

- **Option B — Modern formats + perceptual quality (RECOMMENDED).** WebP output
  (pure-Rust) + a **`--max-size <KB>` byte-budget** + the flagship **perceptual
  auto-quality** (`shrink --target visually-lossless` / `--ssim 90`,
  binary-searching encoder quality against SSIMULACRA2 via `ssimulacra2_rs`) +
  AVIF (`ravif`). This is the differentiator core ("set the look, not the
  number"), rides the frontier paradigm shift, builds directly on shipped
  `shrink`+DEC-016, is small and pure-Rust, **unblocks the modern-format parts of
  animation/responsive**, and backs every future benchmark claim. Needs a DEC
  (metric + thresholds + the search loop).

- **Option C — Animation (GIF + responsive sets).** `gif`/`animate` (frames→GIF
  with quality defaults + `--max-size` budget + GIF→optimized-GIF, pure-Rust;
  video→GIF and animated-WebP feature-gated) + the responsive-set generator
  (`responsive` → multi-width × multi-format + a paste-ready `<picture>`/`srcset`
  snippet). User explicitly wants easy automated GIFs. The modern-format pieces
  soft-depend on B (WebP/AVIF).

**Recommendation: B as STAGE-004**, because it's the moat *and* the dependency
root for C, it's the smallest pure-Rust build on already-shipped code, and it
makes crustyimg *different*, not just *good*. Then **C** (animation + responsive
+ blurhash rider), then **A** (metadata/watermark + recipe exposure), then
geometry/effects breadth, then benchmark-publication + hardening + release. A's
EXIF-audit-linter and recipe exposure are independent and can interleave anytime.

**This thread's first task: confirm the STAGE-004 choice with the user (or
proceed on B if they've pre-approved), then `just new-spec` + design its first
spec** following the orchestration model. If B: the first spec is likely **WebP
output** (smallest, unblocks the rest) or the **perceptual auto-quality** flagship.

## Strategic context (research synthesis, 2026-06-15/16)

### Competitive reality
The product/CLI layer is **stale** — ImageMagick (cryptic, heavy, Windows DLL
hell), libvips/`vips` (fast but obscure CLI, system dep), sharp (Node + native),
squoosh-cli (unmaintained since 2023), ImageOptim (macOS-only), the single-format
optimizers (jpegoptim/oxipng/pngquant/cwebp/avifenc — each does ONE format),
exiftool (hard + incompletely strips), imgproxy/thumbor (servers, wrong shape).
Closest peer **rimage** (Rust) has no recipes, no dry-run, no metadata, Windows
path bugs. **Top unserved gaps, ranked: (1) reproducible declarative pipelines,
(2) privacy-by-default + verification, (3) single static binary all-OSes, (4) one
tool all-formats, (5) dry-run/diff, (6) quality-verified compression, (7) format
auto-negotiation.** crustyimg's structural bets already claim #1/#3/#4 and are
on track for #2.

### The frontier (the COMPONENT layer is hot, even though the product layer is stale)
Watchlist worth riding: **zune-jpeg/zune-image** (image-rs is migrating its
default JPEG decoder to it — your decode backbone); the **awxkee cluster**
(`pic-scale` color-correct SIMD resize, `moxcms` pure-Rust ICC — kills lcms2
FFI, `yuvutils-rs`); **jxl-oxide** (mature pure-Rust JXL *decode*) + Google's
`jxl-rs` (now in Chrome 145, flagged); **Imazen's `zen*`** (`zenjpeg`/`jpegli-rs`
perceptual JPEG, `zenwebp` pure-Rust *lossy* WebP, `mozjpeg-rs`, `codec-eval`) —
technically exciting **but AGPL** (see Constraints); **oavif** (Zig) — the
paradigm: encode to a perceptual *target*, not a Q number; **tract** (pure-Rust
ONNX) — the only single-binary-safe path to optional AI ops. **Two shifts:** (1)
"perceptual target, not a quality number"; (2) pure-Rust safe-SIMD replacing C
FFI across the whole stack.

### The "stack bet" (net-new positioning)
No incumbent assembles the pure-Rust frontier into one binary. crustyimg can be
**the only single static binary doing perceptual-target, color-correct,
modern-format image prep with zero system deps**: decode zune-jpeg · resize
fast_image_resize (or pic-scale) · encode stock/zune JPEG + WebP + ravif AVIF ·
ICC moxcms · JXL decode jxl-oxide · quality ssimulacra2_rs. That's a category
claim, not a feature.

### Codec bets (verdicts, with confidence)
- **JPEG encoder:** keep stock `image`/`zune-jpeg` baseline. Do NOT take an AGPL
  encoder (jpegli-rs/zenjpeg/mozjpeg-rs) as a default — see Constraints. (high)
- **WebP:** ship it. Pure-Rust lossy WebP now exists (`zenwebp` — but AGPL; check
  license) ; `image`/`image-webp` is lossless-only for *static*. Resolve the
  encoder-license question in a DEC. Animated WebP encode is NOT pure-Rust
  (needs libwebp) → feature-gate. (high on "ship WebP", medium on which crate)
- **AVIF:** `ravif`/`rav1e`, pure-Rust, mature for still images; slower at top
  quality — expose the speed knob. (high)
- **JXL:** skip as an *encode* target now (no pure-Rust encoder; Chrome not
  default-on until ~H2 2026). Optionally accept JXL *input* via jxl-oxide for
  free. Revisit when Chrome flips. (high)
- **Perceptual metric:** SSIMULACRA2 (`ssimulacra2_rs`, pure-Rust, <~100ms/compare
  → a 6–8-step binary search is sub-second). `visually-lossless ≈ score ≥ ~90`
  (ship as a tunable constant). (high on metric; medium on the exact cutoff)
- **AI ops (denoise/upscale/bg-removal):** keep core model-free. If ever:
  opt-in `tract` + download-on-demand tiny models for denoise/light-upscale only;
  **background removal is off-limits** for the single binary (176MB models +
  C++ runtime). (high)

## Constraints that now bind feature choices

- **License: crustyimg is MIT/Apache (permissive) → it cannot take an AGPL
  dependency** (would force the whole binary to AGPL). This RULES OUT, as
  defaults: **gifski** (best-quality GIF), **jpegli-rs/zenjpeg** (perceptual
  JPEG), likely **zenwebp** and **imagequant/libimagequant** (lossy PNG/GIF
  quantization — GPL/commercial). Use permissive pure-Rust alternatives (`image`
  + `color_quant` NeuQuant, `ravif`, etc.), or feature-gate an AGPL crate behind
  an explicit opt-in build the user accepts. **Record this in a DEC** the first
  time it bites (it will bite WebP, GIF, and lossy-PNG work). Add a constraint
  `no-agpl-default-deps` to `guidance/constraints.yaml`.
- All existing constraints still apply (single-image-library, pure-rust-codecs-
  default, decode-once-no-per-op-disk, no-unwrap, clippy-fmt-clean `--all-targets`,
  every-public-fn-tested, untrusted-input-hardening, ergonomic-defaults).
- Adding any new top-level dep needs a DEC (`no-new-top-level-deps-without-decision`).

## The GIF feature (user-requested: "automated GIFs, made easy")

**Is it on competitor lists? Effectively NO as a clean one-command experience.**
ffmpeg owns video→GIF but via the hostile two-pass `palettegen`/`paletteuse`
filtergraph; ImageMagick makes low-quality global-palette GIFs; gifsicle only
*optimizes* existing GIFs; gifski is best-quality but **AGPL** and needs ffmpeg
for video; ezgif is web-only. **"Easy automated GIF with quality defaults from a
single clean CLI" is a genuine gap.**

**Design (`gif`/`animate` command):**
- **Pure-Rust, ship now:** frames/image-sequence → animated GIF (`image`
  `GifEncoder` + `color_quant` NeuQuant per-frame + your own dithering); GIF →
  optimized GIF (re-quantize + frame de-dup); the killer ergonomic **`--max-size
  <KB>`** budget (iteratively drop fps → colors → dimensions to hit a cap — nobody
  ships this cleanly). Good defaults: fps 15, width ≤ 640, 256 colors, dithering
  on (`--no-dither` escape hatch for flat UI), loop infinite.
- **Feature-gated (native dep, honest):** **video → GIF** (the most-demanded
  workflow, but needs ffmpeg — pure-Rust video decode is too immature; gate as
  `--features video` with a clear runtime error if ffmpeg absent); **animated
  WebP/AVIF** encode (libwebp / immature AVIF).
- **High-value convert:** **GIF → WebP/AVIF** shrinks GIFs 30–50%+ — on-brand
  with `convert`/`shrink`, but the WebP-animated encode is the feature-gated part.
- Honestly market the default as "great GIFs, zero deps," NOT "beats gifski."
  Fold into the recipe system as a `gif` recipe.

## Benchmarking plan (user: "figure out how we can benchmark")

A staged plan; (c)+(d) double as a **marketing differentiator** (honest,
reproducible, quality-aware benchmarks — almost nobody combines all three):
1. **Micro-benches — `criterion`** (`just bench`): resize / decode / encode /
   full pipeline, on committed tiny in-memory fixtures. *Cheap regression net —
   do this early, even as a `chore` spec.* (low effort)
2. **CLI wall-clock — `hyperfine`** (`just bench-cli`): startup time, throughput
   (Mpx/s, computed in a wrapper), peak RSS via `/usr/bin/time -l` (macOS) /`-v`
   (Linux). Warm cache, best-of-N, single + batch. (low–med)
3. **Cross-tool comparison — `just bench-compare`**: a committed, license-clean
   corpus (a few photos + PNG screenshots + an EXIF-oriented image, few MB) ×
   {ImageMagick, vips, sharp, rimage, oxipng/cwebp}. Copy libvips's methodology:
   same op, same output, warm cache, **always report time + peak RSS + output
   bytes**, pin/name the machine + tool versions. (medium)
4. **Quality-per-byte — `ssimulacra2_rs`** (`just bench-quality`): encode at equal
   bytes (or equal quality) and score SSIMULACRA2 → "N% smaller at equal quality"
   becomes provable. Start with quality-at-fixed-size tables; BD-rate curves
   (`bdr-ssimu2`-style) later. **You MUST gate any size/speed claim on equal
   quality from here — else the comparison is meaningless.** (medium)
5. **CI regression:** `github-action-benchmark` for free non-blocking trend
   tracking; add **CodSpeed** as the only low-noise *blocking* gate when wanted.
   Skip `iai-callgrind` (Valgrind has no Apple-Silicon support; it's blind to the
   SIMD hot path anyway). (low–med)
6. **Publish `BENCHMARKS.md`** + a one-command reproducer = credibility for a
   young tool. (low, once 3+4 exist)

Note (dev machine is Apple Silicon — `darwin`): `/usr/bin/time -l` for RSS;
instruction-count benches are Linux-CI-only.

## New / outside-the-box near-term feature ideas

Marked **[NEW]** (not in `backlog.md`) vs **[backlog]** (already cataloged,
possibly worth pulling forward). All pure-Rust unless noted.

- **[NEW] `--max-size <KB>` byte-budget** on `shrink`/`convert`/`gif` — hit a file
  size, not a quality number. Top-ranked user pain; pairs with perceptual
  auto-quality (size-target vs quality-target). High value, near-term.
- **[NEW] `optimize` one-button command** — `crustyimg optimize photo.jpg` =
  auto-orient + drop-GPS + perceptual-target re-encode (+ optional modern format).
  The "just make this web-good" default that bundles the differentiators.
- **[NEW] `diff` (perceptual + visual)** — `crustyimg diff a.png b.png` → an
  SSIMULACRA2 score + a highlighted pixel-diff image; `--fail-under 90` exit-code
  gate for CI visual-regression (sibling of the EXIF audit-linter). Reuses the
  metric you build for auto-quality.
- **[NEW] `watch` mode** — `crustyimg watch images/ --recipe web.toml --out-dir
  dist/` reprocesses on change (`notify` crate). The dev-loop version of the
  incremental "image bundler"; delightful for content authors.
- **[NEW] `--json` everywhere** — every command emits machine-readable results
  (what it did, in/out bytes, score) → first-class CI/scripting citizen.
- **[NEW] ASCII / half-block / braille render** (`view --ascii`) — image in a
  non-graphical terminal; cheap delight, complements viuer.
- **[NEW] sprite-sheet / texture-atlas packing** — pack many images into one
  sheet + emit JSON/CSS coords (rect-packing). Game-dev + web-icon niche.
- **[backlog→pull-forward] `crop` (lead geometry op), `rotate`, `flip`, `trim`,
  `pad`** — the "filters bulldoze"; each is one `Operation`.
- **[backlog→pull-forward] effects catalog** — grayscale/sepia/invert/blur/
  unsharp/pixelize/edges; recipes make presets trivial. The fun breadth.
- **[backlog→pull-forward] text/caption annotation** (`ab_glyph` +
  `imageproc::drawing`) — also enables GIF captions / meme-style output.
- **[backlog] smart crop (saliency-first; face-aware opt-in)** — port smartcrop.js
  algorithm (pure-Rust); strong wow for the thumbnail/avatar crowd.
- **[backlog] palette / dominant-color → CSS**; **blurhash/thumbhash placeholder**
  (rider on the responsive set); **contact-sheet/montage** (folder visual index).
- **Skeptic flags:** AI alt-text (breaks pure-Rust; needs a model/API — opt-in
  external only); near-duplicate detection (off-persona, photo-library not
  web-prep); ICC color management (real but photographer-skewed — do a quiet
  convert-to-sRGB default, not a headline).

## Proposed 6-month roadmap (a menu, not a contract)

Front-loads the differentiators (the moat + the demo), then the fun breadth, then
the proof + polish. Benchmark micro-net lands early; benchmark *publication* mid.

- **Month 1 — Modern formats + quality core (STAGE-004 = Option B):** WebP output;
  `--max-size` byte budget; **perceptual auto-quality `shrink --target`** (flagship,
  +DEC); AVIF. Wire `criterion` + `hyperfine` micro/CLI benches as a regression net.
- **Month 2 — Web-prep power + ergonomics:** responsive set + `<picture>`/srcset
  emission (+ blurhash rider); `optimize` one-button; `diff` perceptual/visual + CI gate.
- **Month 3 — Animation:** `gif`/`animate` (frames→GIF, `--max-size`,
  GIF→optimized; feature-gated video→GIF + animated WebP). GIF captions if text
  annotation lands.
- **Month 4 — Geometry + effects bulldoze:** `crop` (lead) + rotate/flip/trim/pad;
  effects catalog; text/caption annotation. The "filters" fun weeks.
- **Month 5 — Metadata, privacy & recipes surfaced (Option A work):** metadata lane
  (`strip`/`clean --gps`/`set`/`copy-metadata`) + **EXIF audit-as-linter**; surface
  recipes (`edit`/`--save-recipe`/batch `apply`) + `watch` mode. The reproducibility
  moat goes user-facing.
- **Month 6 — Proof + polish:** full benchmark comparison + quality-per-byte +
  `BENCHMARKS.md`; smart crop (saliency); contact-sheet; hardening baseline
  (decode limits, traversal tests, cargo-audit in CI) + release polish.

Independents that can interleave anytime: EXIF audit-linter, `--json`, the
benchmark micro-net, ASCII render.

## Open questions to resolve (raise with the user / settle in a DEC)
1. STAGE-004 = A, B, or C? (recommend B.)
2. WebP encoder crate + the AGPL question (zenwebp AGPL vs `image`/libwebp-FFI vs
   feature-gate). Settle in a DEC alongside `no-agpl-default-deps`.
3. Is "crustyimg emits HTML (`<picture>`/srcset)" in-scope? (recommend yes —
   it's the responsive-set differentiator, and it's opt-in output.)
4. Re-scope PROJ-001's remaining stages around this roadmap, or frame the
   differentiator work as PROJ-002? (The brief currently scopes STAGE-004–007 as
   metadata→recipes→hardening→release.)

Start by reading the orientation files + the prior handoff, then bring the
STAGE-004 decision to the user (with the recommendation), and on their go,
scaffold + design its first spec.
</content>
