# PROJ-002 scoping — findings & ranked proposal

> **What this is.** The evidence-based scoping deliverable for crustyimg's next wave
> (PROJ-002), answering `docs/research/proj-002-scoping-research.md`. Research + a ranked
> proposal — **not** implementation, not framed specs. Feeds a follow-on planning session
> that writes `brief.md` + stages + first specs.
>
> **Method.** Internal inputs (`docs/backlog.md`, `feature-exploration.md`, `moat.md`,
> `guidance/license-watchlist.yaml`, DEC-004/018) + a survey of adjacent-tool demand
> (sharp, ImageMagick, libvips, thumbor/imgproxy, squoosh, SSG pipelines, rimage/truss)
> and the pure-Rust/permissive crate landscape (crates.io/docs.rs/lib.rs, verified July
> 2026). Sources cited inline. Research date: **2026-07-05**.
>
> **Bottom line up front.** The sharpest, most defensible thesis is **crustyimg = an
> opinionated image *optimization engine*, not an editor**: analyze → plan → transform →
> encode → validate → **explain**, producing the best artifact under measurable
> constraints — and emitting a manifest a web build consumes. The **web-asset-engine**
> thesis still holds, as the *delivery layer* on top of that engine (thesis §5). The
> highest-value 0.3.0 is **the analysis/decision layer** (format recommendation + explain
> + classification), not geometry — this **challenges the backlog's crop-lead bet** (§6),
> and the challenge is grounded: the engine layer extends the *existing* moat ("set the
> look, not the number"), is deterministic and **needs zero new dependencies**, and
> differentiates; crop is cheap table-stakes parity that can ride along. AI features
> (super-res, face-crop, bg-removal, OCR) are correctly **opt-in/feature-gated or folded**.

---

## 0. Grounding — what's already shipped (don't re-propose it)

Shipped surface (v0.2.1): `view info resize thumbnail shrink convert optimize responsive
auto-orient watermark strip clean set edit apply copy-metadata`. **WebP output is done**
(default `webp-lossy`; `responsive` format variants) — drop the backlog's "WebP output"
item. **AVIF** ships feature-gated (pure-Rust rav1e). `auto-orient` shipped. The engine
already does **outcome-driven compression**: perceptual auto-quality (`--target`/`--ssim`
via SSIMULACRA2) + byte budgets (`--max-size` with dimension-fallback). **This is the seed
of the optimization-planner thesis** — PROJ-002 generalizes it.

Architecture hooks already in the tree that make the recommendation cheap:

- **`Gravity` anchor** (Center/North/…, `src/operation/mod.rs`), shared with `watermark`;
  **`resize --fit fill` already center-crops**. → rect/gravity/aspect `crop` = assembly of
  existing parts (**S**).
- **`responsive` already computes per-variant data** (widths, formats, actual dims, paths,
  dedupe, fallback) and emits it *only* as an HTML `<picture>`/srcset snippet. → a
  **manifest is a near-free `--manifest` flag on an existing command**, not a subsystem.
- **`info --json` / `diff --json` exist** (hand-rolled; `serde` derive is a runtime dep,
  `serde_json` a dev-dep). → JSON output pattern established.
- **`fast_image_resize` enlarges** (Lanczos3 is symmetric): basic upscaling = "remove the
  no-upscale guard + a flag" at `src/operation/mod.rs:~531`. No new dep.
- Deliberately **no `imageproc`** (drags sdl2/nalgebra) — effects/convolution must hand-roll
  or use a light crate.

Constraint recap (DEC-018 / `deny.toml`): allowlist = MIT, Apache-2.0(+LLVM-exc), BSD-2/3,
Zlib, Unlicense, Unicode-3.0 (**MPL-2.0 and CC0 not on it**). LGPL only via per-crate
exception; GPL/AGPL unusable; **C/system deps violate zero-system-deps** (this is why `ort`
and any HEVC/mozjpeg/libavif path is off the default). New untrusted-input surfaces inherit
STAGE-006 hardening.

---

## 1. The thesis, sharpened: an optimization *engine*, not an editor

The most defensible framing, and the one most faithful to the moat crustyimg already has:

> **Give crustyimg a declarative goal; it analyzes the image, picks the format, produces
> the smallest artifact that meets a measurable quality/size target, explains every
> decision, and emits a manifest a web build can consume. Not a Photoshop-lite.**

The evaluation lens becomes **"does X help produce a better artifact automatically?"** —
which cleanly separates the moat (optimization, automation, reproducibility, analysis,
observability) from the rabbit holes (manual editing, artistic effects, photo
manipulation). Priorities in order: **optimization > automation > reproducibility >
analysis > observability.**

Why this is the right thesis, not a pivot: the shipped wedge is *already* "set the look,
not the number" (outcome-driven perceptual compression). The **planner**, **format
recommendation**, and **explain** are the apex of that existing engine — they *widen the
moat*. Geometry/effects (the backlog wave) widen the *surface*. Widen the moat first.

**Demand-validated framing (2026-07-05) — how to position the engine features so they hit
real demand, not first-principles:**
- **Format recommendation = a *silent auto-decide default* ("the local `f_auto`"), not an
  advisory report.** The strongest demand signal is Cloudinary `f_auto`/`q_auto` adoption
  (≈86% bandwidth cuts; baked into Gatsby/build plugins; Cloudinary's own "use auto for
  everything" guidance) — developers want the tool to *choose the format+quality for them*.
  So `optimize` should just pick; the "recommendation" is the transparency layer on top, not
  a separate advisory command. crustyimg is the **local/CI/no-CDN `f_auto`** for teams not on
  a media CDN. ([Cloudinary](https://cloudinary.com/documentation/image_optimization))
- **`lint` = source-file, no-URL, deterministic exit code.** Lighthouse ships four image
  audits ("serve next-gen formats", "properly size", "efficiently encode", "offscreen") and
  Lighthouse CI enforces image budgets — but **all of it runs in-browser against a deployed
  page**. It structurally *cannot* lint a source asset tree without a rendered URL, and
  generic `--maxkb` git gates are **format-blind**. The white space is a **source-file-level,
  no-URL, format-aware, pass/fail** linter. Don't compete on Lighthouse's "next-gen formats"
  framing — own the pre-deploy / design-asset / no-Lighthouse-able-URL slice.
  ([Lighthouse audits](https://developer.chrome.com/docs/lighthouse/performance/uses-webp-images),
  [lighthouse-ci](https://github.com/GoogleChrome/lighthouse-ci/blob/main/docs/configuration.md))
- **`explain` = concise default** (format chosen / bytes saved % / one-line why, optional
  perceptual score) — Squoosh-grade polish, proven table-stakes, *not* a release-winning
  headline on its own. ([Squoosh UX](https://squoosh.app/))
- **Classification = internal enabler only.** Zero user-facing demand; it powers format-rec
  and appears at most as a one-word `explain` label ("detected: photograph → AVIF"). Don't
  build or market it standalone. (Prior art is all internal: cwebp `-mixed` lossy/lossless
  heuristic.)

**Relationship to the web-asset-engine thesis (prior direction):** complementary, not
competing. The optimization engine *produces* the artifact; the manifest *hands it off* to
the web tool / SSG. Engine = identity; manifest = interface. Full verdict in §5.

**Boundary (unchanged, critical):** crustyimg emits assets + `manifest.json`; it must
**not** grow HTML page generation, templating, routing, a dev server, or OG-card
*rendering* (that's `satori`/`@vercel/og` — a layout/typography problem, confirmed
out of scope). Those belong to the maintainer's separate web-content tool. The manifest is
the seam that keeps them separate.

---

## 2. Competitive landscape — what to exploit, borrow, and avoid

### What to exploit (ImageMagick's structural weaknesses)
The three most-cited IM developer pains, and crustyimg's answer:
1. **Security-by-configuration** — IM delegates PS/EPS/PDF to GhostScript; the ImageTragick
   (CVE-2016-3714) → recurring GhostScript RCE lineage (CVE-2024-29510, in-the-wild) means
   *every* deployment on untrusted input must hand-harden `policy.xml`
   ([imagetragick.com](https://imagetragick.com/), [Red Hat](https://access.redhat.com/security/vulnerabilities/ImageTragick),
   [Hadrian](https://hadrian.io/blog/imagemagick-zero-days-bypass-multiple-security-policies-what-defenders-need-to-know)).
   → **crustyimg already wins this**: STAGE-006 hardening, no external delegates,
   pure-Rust, safe on untrusted input by default. This is a headline, now CVE-backed.
2. **Memory/perf blowup on large images** — IM loads whole images; libvips author's bench
   ~3 GB vs ~200 MB, real migrations 2.3–5.5× faster; drove Rails 7 → libvips, Mastodon to
   evaluate ([Criteo](https://medium.com/criteo-engineering/boosting-image-processing-performance-from-imagemagick-to-libvips-268cc3451d55),
   [Mastodon #20269](https://github.com/mastodon/mastodon/issues/20269)). → §7 (fused
   pipeline) is the long-term answer; not PROJ-002.
3. **Cryptic order-sensitive syntax** ([IM discussion #1984](https://github.com/ImageMagick/ImageMagick/discussions/1984)).
   → crustyimg's subcommand/recipe model + sane per-format defaults already answers this.

### What to borrow (one idea each)
- **libvips: a lazy, *fused* operation pipeline** — compile a multi-op recipe into one
  streamed pass over tiles/strips so intermediates stay cache-resident and no full
  intermediate is allocated ([libvips "How it works"](https://www.libvips.org/API/current/how-it-works.html)).
  crustyimg today "loads once, applies all ops in memory, encodes once" (already better
  than per-op re-reads) but still materializes the full image. Fusing is a real large-image
  differentiator — **§7, future, not PROJ-002.**
- **sharp: a fluent chain compiled to one pipeline + great per-format defaults + narrow
  scope** ([sharp](https://sharp.pixelplumbing.com/)). crustyimg's structural edge over
  sharp: a **single self-contained binary, no runtime** (sharp is Node + native addon).
- **truss-image ([nao1215/truss](https://github.com/nao1215/truss)): one I/O-agnostic core
  → CLI + HTTP + WASM frontends.** Immature (~7★) — borrow the *pattern*, not the project.
  It fits crustyimg's Operation/recipe core **if the core stays free of fs/CLI
  assumptions** and non-pure-Rust codecs are feature-gated for the WASM target. Given
  crustyimg's pure-Rust posture, **WASM + HTTP frontends over the recipe core are realistic
  and cheap later** — this is the strongest argument for the "engine core" framing (§7).

### Closest competitor
**`rimage`** ([SalOne22/rimage](https://github.com/SalOne22/rimage), ~407★, MIT/Apache) —
squoosh-inspired optimizer CLI. **But**: it wraps **C** codecs (mozjpeg, libavif) and is
**narrow** — optimize/convert/quantize only, **no crop/rotate/transforms**. crustyimg
differs on exactly the axes that matter: **pure-Rust + safe-on-untrusted + a general
optimization *engine* + outcome-driven perceptual quality**. rimage owns the narrow "shrink
with modern codecs" niche; crustyimg should own "the optimization engine that decides *and
explains*." Other Rust tools (`oxipng`, `zune-image`, `caesium-clt`, `ril`) are
components/libraries or narrower, not head-on competitors.

---

## 3. Evidence summary — candidate features

Grouped by role in the engine. Demand (with source), crate + license + maturity, arch fit,
complexity S/M/L. **★ = deterministic, zero-new-dependency** (the low-risk on-thesis core).

### The engine core (analysis + decision) — the differentiators

| Feature | Demand / rationale | Crate / license | Arch fit | Cx |
|---|---|---|---|---|
| ★ **Format recommendation** | Highest value-per-effort; the sibling of perceptual auto-quality. Mirrors cwebp `-lossless` auto + sharp/squoosh format pickers (alpha + is-it-a-photo gate). | none — histogram / alpha / color-count / gradient-entropy analysis | new **Analyzer** + `--auto-format` | **M** |
| ★ **`explain` mode** | Observability/trust: "detected photograph, high noise, recommended AVIF, saved 41%, SSIMULACRA2 93.2" vs "compressed 37%." On the verification moat. | none | reads `Analysis`; new report | **S–M** |
| ★ **Image classification** (photo/logo/illustration/UI/doc/icon) | *Upstream enabler* — classify → format/crop/compress all improve. Deterministic heuristics (color count, edge density, alpha, entropy, histogram). | none | new **Analyzer** (foundation) | **M** |
| ★ **Optimization planner** (declare goal → engine decides format/resize/normalize/params) | **The apex / potential defining feature.** Generalizes the shipped `--target`/`--max-size` engine to also choose format + geometry + normalize. | none | decision layer over `Analysis` + existing search | **L** |
| ★ **`lint`** (`crustyimg lint images/`) | **The other defining feature — "clippy for images."** PNG-should-be-AVIF, exceeds budget, ICC missing, orientation wrong, **metadata/GPS leak** (folds in the privacy moat), wrong colorspace. CI-shaped (exit codes exist). | none — composes classification + format-rec + budgets + existing metadata lane + `diff` | new command over `Analysis` | **M** |
| ★ **Batch intelligence** | Classify-per-file → optimize each differently (400 icons vs 300 photos). Natural over `apply` + classification. | none | over existing parallel `apply` | **M** |
| ★ **Optimization profiles** (`web`/`docs`/`social`/`ecommerce`/`retina`) | One-command presets = **curated recipes** (crustyimg already has recipes). | none | recipe presets | **S** |

### Delivery layer (the web-asset thesis, §5)

| Feature | Demand (source) | Crate / license | Arch fit | Cx |
|---|---|---|---|---|
| **manifest** (`responsive --manifest`, `apply --manifest`) | SSGs consume exactly this shape (eleventy-img metadata object; Next `blurDataURL`; Astro `getImage`). The gap is a **non-Node** producer. | `serde` (dep); promote `serde_json` to runtime | flag on existing cmds / new Sink | **S–M** |
| **placeholder** (thumbhash / blurhash) | Real gap for **dynamic/remote/bulk** images frameworks don't see ([Next #26168](https://github.com/vercel/next.js/discussions/26168)). | `fast-thumbhash` 0.2 (MIT) / `blurhash` 0.2.3 (Apache/MIT, never enable `gdk-pixbuf`) | read-side Op / manifest field | **S** |
| ★ **dominant color / palette** | Manifest field + LQIP bg + format-rec input. | `kmeans_colors` 0.7.1 (MIT/Apache) | read-side Op / field | **S** |
| **favicon / multi-size ICO** | Strong differentiator: `favicons` npm ~258k dl/wk ([npm](https://www.npmjs.com/package/favicons)); fiddly elsewhere. | `ico` 0.5.0 (MIT) — true multi-size (`image` `IcoEncoder` writes one image only) | new multi-output Sink | **M** |

### Observability

| Feature | Demand / rationale | Crate / license | Arch fit | Cx |
|---|---|---|---|---|
| ★ **compression heatmap** (`explain`/`diff` visual) | Per-region SSIMULACRA2/error map PNG — "where did compression hurt?" Novel, educational, on-moat; SSIMULACRA2 already computed internally. | none | extends `diff` | **M** |
| **perceptual hash / dedup** | DAM-flavored duplicate/near-dup detection. Analysis-fit, not core. | `image_hasher` 3.1.1 (MIT/Apache; **not** stale `img_hash`) | read-side Op | **S–M** |

### Geometry (backlog wave — now companion, not lead)

| Feature | Demand (source) | Crate / license | Arch fit | Cx |
|---|---|---|---|---|
| **`crop` (rect/gravity/aspect)** | Strong table stakes (every tool has it). | none — reuses `Gravity` + center-crop | drop-in Op | **S** |
| smart crop (saliency → face) | Strongest raw demand ([smartcrop.js ~13k★](https://github.com/jwagner/smartcrop.js/), [sharp #295](https://github.com/lovell/sharp/issues/295)); **no ML needed** (heuristic is the industry standard). | `smartcrop2` 0.4 (MIT) or clean-room (~few hundred lines); face tier = `rustface` (BSD-2, **stale 2020**, opt-in, probe dep graph) | Op (+ `--print-crop`) | **M** |
| rotate / flip | Table stakes. | `image` built-ins | drop-in Op | **S** |
| trim / pad | Moderate utility ([IM `-trim`](https://www.baeldung.com/linux/image-crop-borders-white-spaces)). | hand-roll | drop-in Op | **S–M** |

### Auto-color, upscale, compose (automation, not editing)

| Feature | Demand / rationale | Crate / license | Arch fit | Cx |
|---|---|---|---|---|
| ★ **auto color** (normalize / auto-contrast / auto-WB) | **Draw the line at *automatic*** — normalize/auto-contrast/gray-world-WB = optimization; manual brightness/saturation sliders = the editor rabbit hole, skip. | `palette` 0.7.6 (Apache/MIT) for HSL/hue; hand-roll LUTs + gray-world/temperature | drop-in Op / auto-pass | **S–M** |
| ★ **basic upscaling** (Lanczos3/Mitchell/CatmullRom) | Real: low-res CMS assets, logos, avatars. Just "remove the no-upscale guard + flag." | `fast_image_resize` (already a dep) | flag on resize | **S** |
| **AI super-resolution** (Real-ESRGAN/waifu2x) | Opt-in, later. **The one AI feature that's *not* a hard trap**: pure-Rust runtime + clean model license. | `rten` (pure Rust, no C/BLAS) or `tract` CPU (pure Rust); models **Real-ESRGAN BSD-3 / waifu2x MIT**, ~4.6 MB (general-x4v3) external | opt-in feature, BYO/bundle-small | **L** |
| composition / overlay (blend modes) | Bounded: overlay/opacity/gravity/pad/scale — no layers. Extends `watermark`. | hand-roll (multiply/screen = 3–5 lines each) or `image-blend` | drop-in Op | **S–M** |

### Folded / out of scope

| Feature | Verdict | Why |
|---|---|---|
| **OCR** | **Fold (opt-in at most)** | Pure-Rust-viable (`ocrs`/`rten`, MIT/Apache) but improves *metadata*, not the *artifact* — off-thesis (analysis product, not optimizer). |
| **background removal** | **Fold** | Confirmed trap: good models (u2net/IS-Net 170 MB; DUTS training-data license undercuts the Apache label, [U-2-Net #208](https://github.com/xuebinqin/U-2-Net/issues/208)); the only clean combo (`tract`+u2netp ~5 MB) is quality users call broken. BYO-model opt-in at most. |
| **JPEG XL output** | **Fold (decode later)** | Safari-only ~15% ([caniuse](https://caniuse.com/jpegxl)); no mature permissive *lossy* encoder. `jxl-oxide` decode (MIT/Apache) is a cheap PROJ-003 *input* win. |
| **network placeholder fetch** (Picsum/Unsplash) | **Defer** | First network dep → fresh threat model + DEC. |
| **HTML/CSS site scan** | **Web-tool side** | Scanning a website is the web-content tool's job; crustyimg exposes *generation* via manifest/responsive. |
| **registry / plugins / hosted service / state files** | **Out of scope** (§6) | Platform superstructure that fights the single-binary/zero-config identity; Terraform's mutable state is its *worst* part — a pure-function pipeline needs none. |

---

## 4. Value × effort × fit ranking

Value = demand × strategic (moat/thesis). Effort lower=better. Fit = arch + license
cleanliness. ★ = deterministic zero-new-dep.

| Feature | Value | Effort | Fit | Tier |
|---|:--:|:--:|:--:|:--:|
| ★ Format recommendation | 5 | 4 (M) | 5 | **T0 — engine core** |
| ★ `explain` mode | 5 | 5 (S–M) | 5 | **T0 — engine core** |
| ★ Image classification | 5 | 4 (M) | 5 | **T0 — foundation** |
| ★ `lint` | 5 | 4 (M) | 5 | **T1 — defining** |
| ★ Optimization planner | 5 | 2 (L) | 5 | **T1 — defining/apex** |
| manifest output | 5 | 4 (S–M) | 5 | **T1 — delivery** |
| ★ auto-color (normalize/auto-contrast) | 4 | 4 (S–M) | 5 | **T1** |
| placeholder (thumbhash) | 4 | 5 (S) | 5 | **T1 — delivery** |
| ★ dominant color / palette | 4 | 5 (S) | 5 | **T1** |
| ★ compression heatmap | 4 | 4 (M) | 5 | **T2 — observability** |
| ★ batch intelligence + profiles | 4 | 4 (S–M) | 5 | **T2** |
| favicon / ICO | 4 | 3 (M) | 4 | **T2 — delivery** |
| `crop` (rect/gravity/aspect) | 4 | 5 (S) | 5 | **T2 — cheap companion** |
| ★ basic upscaling | 3 | 5 (S) | 5 | **T2** |
| smart crop (saliency) | 4 | 3 (M) | 4 | **T3 — later** |
| composition / blend | 3 | 4 (S–M) | 5 | **T3** |
| perceptual hash / dedup | 3 | 4 (S–M) | 5 | **T3** |
| AI super-res (opt-in) | 3 | 2 (L) | 3 | **T3 — feature-gated** |
| OCR / bg-removal / JXL-out | 2 | 1 | 1–2 | **Fold** |

**Top tier called:** **T0 = the engine core** (format-rec + explain + classification, on a
new shared `Analysis` layer). **T1 = the defining features + delivery** (lint, planner,
manifest, placeholder, auto-color). Everything T0–T2 is permissive, pure-Rust,
zero-system-deps; T0/T1 is largely **zero-new-dependency**.

---

## 5. Web-asset-engine thesis: **HOLD** (as the delivery layer)

Sound, and crustyimg executes it cheaply — with one correction and a sharper wedge.

- **Consumption shape is proven/stable.** eleventy-img returns per-format
  `{ format, width, height, url, sourceType, srcset, size }`; Next static import surfaces
  `width/height/blurDataURL`; Astro `getImage` returns `src`/`attributes`/`srcSet`. A
  manifest carrying these is directly consumable.
- **The gap is real and non-Node-shaped.** Frameworks generate this only for build-time
  images; **dynamic/remote/bulk** images (CMS, user content, CDN) are the dev's problem
  ([Next #26168](https://github.com/vercel/next.js/discussions/26168)). Incumbent producers
  (sharp, plaiceholder, squoosh) are Node/native.
- **Correction (important):** the "SSGs can't do modern formats" wedge is **mostly closed**
  — Hugo (v0.162, late 2025) and Zola (Feb 2025) both shipped native AVIF encode. **Don't
  pitch on AVIF.** The wedge is the **manifest + placeholders + non-Node single binary**.
- **Strongest demand signal:** `@squoosh/cli` / libSquoosh **abandoned by Google**; the
  community responded with maintained forks and **Rust re-implementations (`rimage`)** —
  a vacated category leader ([npm deprecation](https://www.npmjs.com/package/@squoosh/cli),
  [rimage](https://github.com/SalOne22/rimage)). Real audience: Jekyll/Eleventy/Makefile/CI
  shops displacing ImageMagick + dead squoosh-cli. (Counter-evidence, stated honestly: Zola
  users culturally want features *in Zola*, not a companion binary — the most Rust-aligned
  audience is the least likely to adopt a second tool.)
- **crustyimg already computes the hard parts** — `responsive` builds the variant data;
  manifest is a serialization change. Placeholder/dominant-color/byte-size are the only new
  fields, each S-sized.

### Proposed manifest schema (v1) — LOCKED against verbatim tool shapes

Field names verified against eleventy-img's return object, Astro `getImage` `GetImageResult`,
Next `StaticImageData`, and vite-imagetools `Picture` (sources linked at foot of §5).

```json
{
  "manifest_version": 1,
  "generator": "crustyimg 0.x",
  "images": [
    {
      "source": "photos/hero.jpg",      // original input path (distinct from outputs)
      "width": 4032, "height": 3024, "aspectRatio": 1.3333,
      "class": "photo",                 // value-add — from classification (§3)
      "dominantColor": "#3b5a72",       // value-add — no standard SSG consumer reads it
      "placeholder": {                  // INTEROPERABLE form = base64 data-URI…
        "blurDataURL": "data:image/webp;base64,UklGR…",   // Next/responsive-loader read this
        "blurWidth": 8, "blurHeight": 6,
        "thumbhash": "1QcSHQ…", "blurhash": "LEHV6n…"      // …extras (need client-side JS)
      },
      "variants": [                     // per-format × per-width — eleventy-img's exact shape
        { "url": "/img/hero-320w.webp",  "outputPath": "out/hero-320w.webp",
          "filename": "hero-320w.webp", "width": 320,  "height": 240,
          "format": "webp", "size": 11834, "sourceType": "image/webp" },
        { "url": "/img/hero-1280w.webp", "outputPath": "out/hero-1280w.webp",
          "filename": "hero-1280w.webp", "width": 1280, "height": 960,
          "format": "webp", "size": 98211, "sourceType": "image/webp" }
      ],
      "sources": { "webp": "/img/hero-320w.webp 320w, /img/hero-1280w.webp 1280w" },
      "fallback": { "url": "/img/hero-1280w.jpg", "width": 1280, "height": 960 }
    }
  ]
}
```

Locked conventions (each is what a real consumer uses, not a guess):
- **`url`** (web path) **+ `outputPath`** (disk) **+ `filename`** — eleventy separates all
  three; don't collapse to one `path` (no tool exposes a top-level `path`).
- **`size`** (bytes, eleventy) — not `bytes`. **`sourceType`** (MIME, eleventy) alongside
  bare **`format`** — not `mime`. **`aspectRatio`** camelCase (imagetools/unpic).
- **Placeholder: emit `blurDataURL` (a tiny base64 webp/png data-URI) as the primary,
  interoperable field** — Next (`blurDataURL`) and responsive-loader (`placeholder`) consume
  it directly, no client JS. Carry `thumbhash`/`blurhash` as *extras* (better quality/size
  but plugin-specific, need a decoder). This revises the earlier "thumbhash-primary" call:
  thumbhash wins on merit, `blurDataURL` wins on drop-in interop — ship both, default the
  data-URI.
- **`sources`** (format-keyed srcset) **+ `fallback`** give consumers a ready `<picture>`.
- `class`/`dominantColor` are crustyimg value-adds (no standard SSG consumer) — keep opt-in.
- Keep it **flat and stable** (downstream never re-probes); version from day one; gate
  `placeholder`/`dominantColor`/`class` behind flags so the fast path stays fast.

Sources: [11ty image-js](https://www.11ty.dev/docs/plugins/image-js/),
[Astro astro-assets](https://docs.astro.build/en/reference/modules/astro-assets/),
[Next Image](https://nextjs.org/docs/pages/api-reference/components/image),
[imagetools types](https://github.com/JonasKruckenberg/imagetools/blob/main/docs/directives.md).

---

## 6. Recommended PROJ-002 shape

### Proposed thesis (for `brief.md` `value.thesis`)
> **crustyimg is an opinionated image *optimization engine*: give it a declarative goal and
> it analyzes the image, picks the format, and produces the smallest artifact that meets a
> measurable quality/size target — then *explains* every decision and emits a manifest a web
> build consumes. Pure-Rust, one binary, zero system deps, safe on untrusted input. It
> extends the wedge from *one optimized file* to *a described, verifiable set of assets*.
> Not an editor.**

### The lead decision — engine-led (maintainer-confirmed 2026-07-05, leaning)
The brief asked to confirm-or-challenge crop as the lead. **Called: challenge it — lead with
the engine layer, not geometry.** Rationale: the engine layer extends the *actual moat*,
differentiates (nobody does deterministic format-rec + explain in a CLI), is
**zero-new-dependency**, and builds the `Analysis` foundation everything else needs.
**Crop/geometry is reframed as an "add-anytime companion" track** — it fits cleanly, is
cheap, low-risk, and worth doing, but is *not* direction-defining and can land in any release
when wanted (maintainer's read, 2026-07-05). It does not gate or shape the engine wave.

### Stages (direction, not commitment)
- **STAGE-011 — Engine foundation & decision layer (0.3.0, recommended lead).** The shared
  immutable **`Analysis`** context (histogram, entropy, edge density, alpha coverage) +
  **image classification** + **format recommendation** + **`explain` mode**. Ship **rect/
  gravity crop + rotate/flip** alongside as the cheap table-stakes companion. → *headline:
  "crustyimg now picks the format and explains its optimization."*
- **STAGE-012 — Defining features & delivery (0.4.0).** **`lint`** ("clippy for images",
  folding in the privacy moat) + the **manifest** Sink + **placeholder** (thumbhash) +
  **dominant color** + **auto-color** (normalize/auto-contrast) + **profiles**. Build toward
  the **optimization planner** here.
- **STAGE-013 — Observability & batch (0.5.0).** **compression heatmap**, **batch
  intelligence**, favicon set, perceptual hash/dedup.
- **STAGE-014 — Opt-in intelligence (feature-gated, 0.6.0+).** saliency smart-crop; **AI
  super-res** via `rten`/`tract` (pure-Rust, BYO/small model); composition/blend; optional
  OCR. All behind Cargo features; pure-Rust preferred; **never `ort`/C++ on the default or
  released path.**

### 0.3.0 headline (demand-validated framing)
The validated headline is **"`optimize` now auto-decides format + quality per image (the
local `f_auto`) and tells you what it did"** — i.e. format-rec as a *silent default* inside
the existing `optimize`, + a concise `explain`, on the new `Analysis` layer, with
classification internal. Build toward the **planner** (the general "declare a goal, engine
decides" solver — the fully-validated apex) across the wave. **`lint`** (source-file,
no-URL, deterministic exit) is the killer CI differentiator and the natural 0.4.0 headline.
Crop/geometry rides along whenever convenient (companion track, not direction).

---

## 7. Architecture notes

- **The one genuine architectural addition: a shared immutable `Analysis` context computed
  once.** crustyimg has `Operation` (≈ `Transform`) and `Sink` (≈ `Encoder`); it lacks the
  **`Analyzer`/`Analysis`** layer, which is *prerequisite* for classification/format-rec/
  planner/explain/lint (all read it). It's an **evolution, not a rewrite** — the pixel
  pipeline stays; add an analysis pass upstream + a decision layer above. Compute expensive
  maps (histogram, entropy, edge, saliency, alpha) once, share across transforms. Do this
  **first** to avoid tech debt (aligns with the "don't just add CLI flags" instinct).
- **Keep the core I/O-agnostic** (no `std::fs`/CLI assumptions in decode/transform/encode)
  so a WASM and an HTTP frontend can grow over the same recipe core later (truss-image
  pattern, §2). Cheap insurance; costs little now, unlocks a lot.
- **Fused/streaming pipeline (libvips idea)** — compile a recipe to one streamed pass over
  tiles/strips. Real large-image perf/memory win, but a big change — **future, not PROJ-002.**
- **AI feature-flag correction:** a `onnx = ["ort"]` feature reintroduces a **C++ system
  dep** — breaks zero-system-deps *and* can't sit in the cargo-dist release binary. If AI
  lands, prefer **`candle`/`rten`/`tract` (pure-Rust)**, opt-in, **BYO/small model**, never
  default/released. `ort`, ncnn, tesseract/leptonica are the traps.

---

## 8. Open questions / risks

1. **Lead call (the decision):** engine-led 0.3.0 (format-rec + explain + `Analysis` +
   cheap crop) — recommended — **or** hold the backlog's crop-lead? Determines STAGE-011.
2. **0.3.0 headline (demand-validated):** lead with **auto-deciding `optimize` (local
   `f_auto`) + concise `explain`** on the `Analysis` layer; build toward the **planner**
   (validated apex — "declare a goal, engine decides") across the wave; hold **`lint`**
   (source-file, no-URL — the validated CI white space) for the 0.4.0 headline. Reframe
   away from advisory "recommend"/audit language (Lighthouse owns that framing).
3. **`serde_json` runtime dep** — the design briefs REVISE this: keep JSON **hand-rolled** for
   `explain`/`lint` v1 (matching the existing `write_json`/`write_diff_json`, no runtime dep),
   and only promote `serde_json` if/when the **manifest** schema (§5) grows too big to hand-roll.
   The `ExplainTrace` (planner) is serde-serializable in shape but need not force the dep in v1.
   Don't block the engine wave on it.
4. **`smartcrop2` dep vs clean-room** — probe on real images either way (load-bearing crate
   practice); leaning clean-room keeps the dep tree minimal.
5. **Manifest schema** — §5 is now locked against verbatim eleventy-img/Astro/Next/
   imagetools shapes; the remaining task is to **coordinate field names with the separate
   web-content tool** so it consumes the manifest without a shim (they share this contract).
6. **AI scope discipline** — if super-res/face-crop land, they must be Cargo-feature-gated,
   pure-Rust runtime, BYO/small model, off the released binary. `rustface` is stale (2020) —
   probe the dep-graph/MSRV collision before committing.
7. **Landscape shifts to record** in `guidance/license-watchlist.yaml` (independent of
   feature choice): `quantette` (MIT/Apache) now gives a permissive quantizer (unblocks the
   two GIF/indexed-PNG watchlist items); `resvg`/`usvg`/`tiny-skia` **relicensed off MPL** to
   Apache/MIT+BSD (SVG input viable); `rav1d` (BSD-2) enables pure-Rust AVIF decode;
   `imagequant` confirmed GPL/commercial (keep declined, now with a real way back).
8. **Scope discipline on identity:** keep crustyimg a **pure-function CLI**. Its determinism
   story is *stronger* than Terraform's precisely because it's **stateless** (input+recipe =
   state; no state files, the #1 Terraform pain). Borrow `plan`/dry-run and a content-
   addressed cache (both fit a CLI); avoid state, registry, dynamic plugins, hosted service,
   and any commercial direction — a separate strategic track, not a PROJ-002 feature wave.

---

---

## 9. Design deep-dives (build-ready briefs feeding the planning session)

Five codebase-grounded design briefs (2026-07-05) elaborate the engine-led wave. Each is
design-only (no code); the planning session turns them into specs + DECs.

- **`proj-002-design-analysis-layer.md`** — the foundational `src/analysis/` layer (peer of
  `operation/`, computed-once, immutable, NOT a recipe step / NOT on the `Operation` trait).
  Migration sequence keeps every existing test green (land standalone → wire a read-only
  consumer → let the engine consume it). **Build this first.**
- **`proj-002-design-format-engine.md`** — the silent auto-decide inside `optimize` ("local
  f_auto"): the deterministic decision tree + thresholds, composing the existing SSIMULACRA2
  search + `LossyFormat` seam.
- **`proj-002-design-planner.md`** — the general goal-solver (the validated apex): goal schema,
  objective precedence (size-hard/quality-soft), bounded format×quality×dimension search,
  `ExplainTrace`.
- **`proj-002-design-lint.md`** — `crustyimg lint`: source-file/no-URL/deterministic rule
  catalog, Lighthouse parity+differentiation, exit-7 reuse. No new crates v1.
- **`proj-002-design-classification.md`** — internal deterministic classifier (four features
  carry the decision; six labels → three optimization buckets).

**Convergence (strong signal):** all five independently land on the same `src/analysis/`
foundation, reuse the existing quality search + exit-7 plumbing verbatim, add **no new default
deps**, and stay pure-Rust/no-panic. Low architectural risk.

**One reconciliation the planning session MUST resolve:** the **format engine and the planner
overlap** — the format engine's candidate-shortlist *is* the planner's Phase B. Treat them as
**one "decision engine" subsystem with two entry points**: `optimize` (auto-decide default, no
explicit goal) and a goal-driven `plan`/flags path (explicit `max_size`/`min_quality`). Don't
spec them as two independent features that both grow a format loop. Sequence:
`Analysis` layer → format auto-decide in `optimize` (0.3.0) → generalize to the goal-driven
planner (across the wave) → `lint` on top (0.4.0).

---

*Sources linked inline; primary evidence (GitHub issues with reaction/dependent counts,
npm/crates counts, tool source, caniuse, CVE records) favored over SEO content. Crate
versions/licenses verified against the crates.io API, July 2026. Decision-ready input for the
PROJ-002 planning session; frames no specs itself.*
