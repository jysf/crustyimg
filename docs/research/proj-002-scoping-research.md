# Deep-research brief — scoping crustyimg's next wave (PROJ-002)

**For a separate research session.** Self-contained: read this, do the research, produce the
deliverable at the bottom. You have `WebSearch`/`WebFetch`. This is *research + a ranked
proposal*, **not** implementation — do not write code or frame specs.

## Context (what crustyimg is)

crustyimg is a shipped, public (v0.2.0 on crates.io / Homebrew / GitHub Releases),
**permissive (MIT OR Apache-2.0), pure-Rust, zero-config, intent-based** image CLI. Its
architecture is an `Operation` trait + registry + TOML recipes + a Source/Sink abstraction,
so **most new features are "just another `Operation`"** (or a new encoder/Sink) that drops
into the pipeline + recipe system with no architectural change. Differentiators today:
outcome-driven compression (SSIMULACRA2 perceptual auto-quality), a real command surface
(`optimize`/`shrink`/`responsive`/`diff`), verifiable metadata privacy (container-lane, no
pixel re-encode), reproducible recipes, and STAGE-006 input hardening.

**Read first (internal inputs — build on these, don't re-derive):**
- `docs/backlog.md` — the candidate feature waves (PROJ-002 geometry/effects/format;
  PROJ-003 input formats/quality; stretch). Already ranked into tentative waves.
- `docs/feature-exploration.md` — the full feature catalog + the workflow model.
- `docs/moat.md` — the defensible core + "where the moat is thin (honest)".
- `guidance/license-watchlist.yaml` — capabilities declined for license reasons + revisit
  triggers (RAW, HEIC, gifski, etc.).
- `decisions/DEC-004` (pure-Rust codec/dep policy), `DEC-018` (permissive license / cargo
  deny), the `no-agpl-default-deps` constraint.

## The strategic hypothesis to test

The maintainer is separately building a **web-content tool** (blog / photo-gallery / simple
site builder) — explicitly a *separate* tool, NOT to be merged into crustyimg. The working
thesis: **position crustyimg as the image-asset engine that feeds web workflows** — it
produces optimized assets *plus a machine-readable manifest*; the web tool (and any SSG /
build step) consumes them. Test whether this thesis is sound and what features it implies.

## Research questions

1. **What do users of adjacent image CLIs actually ask for and love?** Survey real demand:
   `sharp` (Node), ImageMagick / `magick`, `squoosh`/`@squoosh/cli`, `sips`, `vips`,
   `oxipng`/`jpegoptim`/`cwebp`, `imgproxy`/`thumbor` (server-side). Mine GitHub issues
   (most-reacted feature requests), HN/Reddit/Lobsters threads, and "awesome image
   processing" lists. What's frequently requested, frequently praised, or a common
   pain-point? Note especially: smart/attention crop, placeholders (blurhash/thumbhash),
   responsive-set + manifest generation, favicon/icon sets, palette/dominant-color,
   background removal, format coverage (AVIF/JXL/WebP), watch mode, social/OG cards.
2. **Pure-Rust crate landscape + license for each candidate.** For every feature that looks
   promising, find the pure-Rust crate(s) that could implement it, and record: crate,
   version, **license** (must be permissive for a default dep; AGPL/copyleft → opt-in
   feature + `cargo-deny` exception at best, per DEC-018/DEC-004), maturity/maintenance,
   and whether it pulls C/system deps (a red flag — crustyimg is zero-system-deps by
   default). Candidates to check at minimum: `resvg`/`usvg` (SVG), `blurhash`/`thumbhash`,
   image-crate AVIF/`ravif`, `jxl-oxide`/`zune-jpegxl` (JPEG XL), color-quantization crates
   (`color_quant`, `exoquant`, `kmeans_colors`), saliency/entropy-crop approaches,
   `notify` (watch mode), ONNX/`ort`-based background removal (assess weight/licensing).
3. **What's underserved?** Where is there a real gap — a thing web/design/dev people want
   that no permissive pure-Rust CLI does well today? (This is where crustyimg's moat can
   widen.) Be honest about where the demand is thin.
4. **Does the "web-asset engine + manifest" thesis hold?** Look at how static-site
   generators / image pipelines (Astro assets, Next/Image, `@astrojs/image`, eleventy-img,
   `sharp`-based build steps, `imagetools`) consume image assets — what a manifest would
   need to contain (variant paths, dimensions, byte sizes, blurhash, dominant color,
   `srcset`/`<picture>`) to be genuinely useful to a downstream tool. Validate or refute.

## Constraints the proposal must respect

- **Permissive-only for default features** (MIT/Apache/BSD/Zlib); pure-Rust; zero system
  deps by default. Copyleft/native → opt-in cargo feature + documented exception only.
- Fits the **`Operation`/`Sink`/recipe** architecture (say so per feature; flag anything
  that needs a new subsystem — e.g. a manifest Sink, a saliency pass, a network Source).
- New **untrusted-input** surfaces inherit STAGE-006 hardening (decode limits, no-panic).
- Don't re-litigate settled declines (HEIC HEVC = no permissive path; gifski = AGPL) unless
  the landscape has genuinely changed since 2026-06.

## Deliverable (write to `docs/research/proj-002-findings.md`)

1. **Evidence summary** — the top ~10–15 candidate features, each with: one-line demand
   evidence (with source links), the pure-Rust crate + license + maturity, architecture fit
   (drop-in Operation / new Sink / new subsystem), and a complexity estimate (S/M/L).
2. **A value × effort × fit ranking** — a table the maintainer can scan; call the top tier.
3. **A verdict on the web-asset-engine thesis** — hold or fold, with the manifest schema it
   implies if it holds.
4. **A recommended PROJ-002 shape** — a proposed thesis (for the `brief.md` `value.thesis`),
   2–4 stages with their candidate specs, and what to ship as **0.3.0** first (the backlog's
   current bet is `crop` as the lead — confirm or challenge with evidence).
5. **Open questions / risks** for the maintainer to decide.

Keep it grounded and cite sources. The output feeds a follow-on **planning session** that
frames PROJ-002 (`brief.md` + stages + the first specs).
