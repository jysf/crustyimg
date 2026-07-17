# crustyimg recipe cookbook

> A catalog of standard, obvious workflows — the copy-paste "how do I…" reference and the
> source for the README's recipe section. Two kinds: **one-liners** (single commands) and
> **saved recipes** (a tuned `edit` chain saved to TOML and replayed with `apply`). Each item is
> marked **[today]** (works in the shipped v0.2.x, or with the noted feature flag) or
> **[planned — PROJ-00N]** (unlocked by a roadmap project; see `docs/roadmap.md`). Living doc —
> add recipes as workflows appear. Snapshot: 2026-07-05.

Legend: **[today]** shipped · **[feat:X]** behind a cargo feature · **[planned — PROJ-00N]** roadmap.

---

## 1. Web optimization (the core job)

- **Make one image web-ready (the flagship)** **[today]**
  `crustyimg web hero.jpg -o hero.avif`
  Downscale (long edge ≤ 2048) + auto-orient + strip metadata + smallest modern format
  that beats the source + reports the SSIMULACRA2 score.
- **Optimize keeping the original dimensions** **[today]**
  `crustyimg optimize hero.jpg -o hero.opt.avif` (fast, never bigger; add `--verify` for the score).
- **Auto-decide the best format** **[today]**
  `crustyimg optimize hero.png --explain` → engine picks AVIF/WebP/PNG/JPEG and explains.
- **Hit a visual-quality target** **[today]**
  `crustyimg optimize hero.jpg --target visually-lossless -o hero.webp`
  `crustyimg optimize hero.jpg --ssim 92 -o hero.jpg`
- **Hit a file-size budget** **[today]**
  `crustyimg optimize hero.jpg --max-size 150KB -o hero.jpg` (quality search, then downscale if needed).
- **Web-optimize a whole folder** **[today]**
  `crustyimg apply --recipe web.toml "assets/**/*.{jpg,png}" --out-dir dist/img -j 8`
  (see the `web.toml` recipe in §7; rayon-parallel, progress bar).

## 2. Responsive images & delivery

- **Generate a responsive width×format set + `<picture>` snippet** **[today]**
  `crustyimg responsive hero.jpg --widths 320,640,1280,1920 --formats webp,jpeg --out-dir dist/`
  Prints a paste-ready `<picture>`/srcset block on stdout.
- **Responsive set + a machine-readable manifest** **[planned — PROJ-005]**
  `crustyimg responsive hero.jpg --widths 320,640,1280 --formats avif,webp --manifest images.json`
  The manifest is what an SSG/build consumes (see §9).
- **Just the srcset for one format** **[today]** — use `responsive` with a single `--formats webp`.

## 3. Resize / thumbnail / crop

- **Cap the long edge** **[today]** — `crustyimg resize photo.jpg --max 1200`
- **Exact box, cover-crop** **[today]** — `crustyimg resize photo.jpg --fit fill --exact 800x800`
  (center-crops to fill; the `fill` path already exists).
- **Thumbnail** **[today]** — `crustyimg thumbnail photo.jpg --size 256`
- **Crop to a rect / gravity / aspect** **[planned — PROJ-006]**
  `crustyimg crop photo.jpg --gravity center --aspect 1:1`
- **Smart (content-aware) crop** **[planned — PROJ-006]**
  `crustyimg crop photo.jpg --smart attention --aspect 16:9`

## 4. Format conversion

- **To WebP (lossless default; lossy with a quality)** **[today]**
  `crustyimg convert logo.png --format webp` · `crustyimg convert photo.jpg --format webp -q 80` **[feat:webp-lossy]**
- **To AVIF** **[today · feat:avif]** — `crustyimg convert photo.jpg --format avif`
- **Batch PNG → WebP** **[today]**
  `crustyimg apply --recipe to-webp.toml "img/**/*.png" --out-dir web/`

## 5. Privacy & metadata (the verifiable-privacy lane)

- **Strip ALL metadata before publishing** **[today]** — `crustyimg meta strip photo.jpg`
- **Remove location only, keep copyright** **[today]** — `crustyimg meta clean --gps photo.jpg`
- **Stamp copyright/artist across a batch** **[today]**
  `crustyimg meta set --artist "Jane Doe" --copyright "© 2026 Jane Doe" photo.jpg`
- **Copy metadata from an original to a derivative** **[today]** — `crustyimg meta copy --from orig.jpg --to edited.jpg`
- **Note:** `optimize` drops GPS/metadata by default (privacy-safe web prep); `--keep-gps` opts out.
- **Audit a tree for metadata leaks (fail CI)** **[planned — PROJ-004]** — `crustyimg lint --select privacy assets/`

## 6. Compositing, watermark, effects

- **Logo watermark, corner, semi-transparent** **[today]**
  `crustyimg watermark photo.jpg --image logo.png --gravity southeast --opacity 0.3 --scale 0.15`
- **Text watermark** **[today]** — `crustyimg watermark photo.jpg --text "© 2026" --gravity south`
- **Redact a region (pixelate / solid mask)** **[planned — PROJ-006]** — privacy-flavored, on the moat.
- **Auto color (normalize / auto-contrast)** **[planned — PROJ-006]** — automatic only, not manual sliders.
- **Upscale a small asset (Lanczos)** **[planned — PROJ-006]** — `crustyimg resize logo.png --max 512 --allow-upscale`.

## 7. Saved recipes (tune once → replay everywhere)

Record a chain with `edit … --save-recipe`, then `apply` it across a glob/dir in parallel. **[today]**

- **`web.toml` — blog/site image prep**
  `crustyimg edit in.jpg --auto-orient --resize 1600 --optimize --strip --save-recipe web.toml`
  then `crustyimg apply --recipe web.toml "assets/**/*.{jpg,png}" --out-dir dist/img -j 8`.
- **`product.toml` — e-commerce product image** (square, consistent)
  chain: `--resize --fit fill --exact 1000x1000` → `--optimize`; replay across the catalog.
- **`avatar.toml` — avatars/thumbnails** — `--resize --fit fill --exact 256x256` → convert webp.
- **`to-webp.toml`** — a one-op `convert --format webp` chain for batch format migration.
- The round-trip is **byte-stable**: `edit` output == `apply`-of-the-saved-recipe output — so a
  recipe reviewed in a PR is exactly what runs in CI.

## 8. CI / verification

- **Visual-regression gate (fail the build if optimization hurt quality)** **[today]**
  `crustyimg diff original.jpg optimized.jpg --fail-under 90` → exit 7 if SSIMULACRA2 < 90.
- **Lint an asset tree (source-file, no URL, exit code)** **[planned — PROJ-004]**
  `crustyimg lint assets/ --format json` → annotations in a PR, exit 7 on error.
- **As a GitHub Action** **[planned — Track B]**
  `uses: crustyimg/crustyimg-action@v1` (lint mode → inline PR annotations; optimize mode → commit-back).

## 9. Rolling it into a static-site generator / build tool

**The one universal pattern (works in every tool):** crustyimg runs as a **(pre-)build step**,
optimizes the asset tree, and emits a **path-keyed JSON manifest**; the SSG's data/template layer
reads it. Manifest **[planned — PROJ-005]**; the optimize/responsive steps work **[today]**.

```make
# the universal build target — run before the SSG; then the SSG reads data/images.json
images:
	crustyimg optimize assets/ --out public/img --manifest data/images.json
```
```json
// data/images.json — KEY BY SOURCE PATH (every template layer does a lookup);
// each entry is SELF-CONTAINED (sandboxed tools can't re-invoke the binary)
{ "assets/hero.jpg": {
    "variants": [{ "url": "/img/hero-1600.avif", "format": "avif", "width": 1600, "height": 900 }],
    "srcset": { "avif": "/img/hero-1600.avif 1600w, /img/hero-800.avif 800w", "webp": "…" },
    "width": 1600, "height": 900, "dominantColor": "#3b4a5a",
    "blurDataURL": "data:image/webp;base64,…(keep ≤10px)…" } }
```

**Native plugin vs build-step-only** (from integration research):
- **Sandboxed → build-step-only** (can't run an external binary; the manifest is the whole contract):
  - **Hugo** — read via `resources.Get "data/images.json" | transform.Unmarshal` (modern) or
    `.Site.Data.images`. Huge audience, but Hugo has native image processing → pitch is "better
    codecs + SSIMULACRA2 auto-quality + LQIP," not "images at all."
  - **Zola** (Rust, single binary) — `load_data(path="images.json", format="json")`. The
    culturally-aligned "single-binary talks to single-binary" story; smaller audience.
  - **Jekyll** — `_data/images.json` → `site.data.images`. (A Ruby plugin *can* shell out, but not
    on GitHub-Pages hosted builds → the data-file route is the portable one.)
- **JS-based → a native plugin is possible** (can shell out or wrap the manifest in-process):
  - **Eleventy — best effort/reward.** A thin **`eleventy-crustyimg`** async shortcode
    (`{% crustyimg "hero.jpg", "alt" %}`) that calls the binary and returns `<picture>` — a
    drop-in analog of the beloved `eleventy-img`, minus the Sharp/native dep.
  - **Astro — the premium story.** A **custom local image service** (`transform()` is async,
    buffer-in/out) so `<Image />` "just works"; pair with a manifest import for LQIP/dominant-color
    (which the stock service contract doesn't surface cleanly).
  - **Next.js / Vite** — fills the **dynamic/remote-image gap** (`next/image` only auto-generates
    `blurDataURL` for *static* imports): a manifest with `blurDataURL` + dims a component reads;
    replaces hand-rolled Plaiceholder/Sharp scripts.
- **Generic Make / npm-script / CI** — the lowest common denominator; document this **first** (it
  unlocks all six at once and never breaks).

**Recommended sequencing:** ship the generic **`--manifest`** flag + docs → then the two native
plugins that showcase crustyimg best: **Eleventy** (`eleventy-crustyimg`) and **Astro** (image
service). (Ranked targets + per-tool sources live in the roadmap's Track B / the research notes.)

## 10. Photography / bulk

- **Downscale a shoot for sharing** **[today]** — `crustyimg apply --recipe share.toml "shoot/*.jpg" --out-dir out/ -j 8`
- **HEIC/RAW → web** — pre-convert HEIC externally (`sips`/`heif-convert`) then crustyimg **[today]**;
  native RAW Tier-1 embedded-preview extraction is **[planned — PROJ-007]**.
- **Contact sheet / montage** — **[planned — later]**.

---

### Which recipes need which project
- **[today]:** web/optimize/resize/thumbnail/convert/responsive(HTML)/meta(strip/clean/copy)/set/watermark/
  edit/apply/diff — the whole §1–§8 core (minus the lint/manifest items).
- **PROJ-002:** `optimize` auto-decides format + `--explain` (shipped).
- **PROJ-004:** `lint` (+ the GitHub Action).
- **PROJ-005:** `--manifest`, `favicon`, `placeholder`, dominant color — unlocks §9 SSG integration.
- **PROJ-006:** `crop`/smart-crop, redaction, auto-color, upscaling.
- **PROJ-007:** RAW input, indexed-PNG (permissive quantizer).
