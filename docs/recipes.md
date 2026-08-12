# crustyimg recipe cookbook

> A catalog of standard, obvious workflows — the copy-paste "how do I…" reference. Two kinds:
> **one-liners** (single commands) and **saved recipes** (an op pipeline in TOML, replayed with
> `apply`). Each item is marked **[today]** (works in the shipped v0.7.x, or with the noted
> feature flag) or **[planned — PROJ-00N]** (unlocked by a roadmap project; see
> `docs/roadmap.md`). Living doc — add recipes as workflows appear. Snapshot: 2026-08-11.

Legend: **[today]** shipped · **[feat:X]** behind a cargo feature · **[planned — PROJ-00N]** roadmap.

For the recipe **file format** and the bundled recipes, see [`recipes/README.md`](../recipes/README.md).
For every flag and exit code, see [`docs/cli-reference.md`](cli-reference.md).

---

## 1. Web optimization (the core job)

- **Make one image web-ready (the flagship)** **[today]**
  `crustyimg web hero.jpg -o hero.avif`
  Downscale (long edge ≤ 2048) + auto-orient + strip metadata + smallest modern format
  that beats the **downscaled** image + reports the SSIMULACRA2 score. The downscale is the
  contract, so an already-small source above 2048px can come back larger than the original
  (reported honestly); for an unconditional never-bigger guarantee that keeps dimensions,
  use `optimize`.
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
  `crustyimg apply --recipe web "assets/**/*.{jpg,png}" --out-dir dist/img -j 8`
  `web` is a **bundled** recipe — no file to write (§7); rayon-parallel, progress bar.
- **Machine-readable decision report** **[today]**
  `crustyimg web hero.jpg --json --out-dir dist/` → `crustyimg.optimize.explain/v1` on stdout
  (candidates, winner, `savings_percent`, `ssim`). Add `--timing` for decode/encode/total ms.

## 2. Responsive images & delivery

- **Generate a responsive width×format set + `<picture>` snippet** **[today]**
  `crustyimg responsive hero.jpg --widths 320,640,1280,1920 --formats webp,jpeg --out-dir dist/`
  Prints a paste-ready `<picture>`/srcset block on stdout (`--no-snippet` to suppress).
- **Responsive set + a machine-readable manifest** **[planned — PROJ-005]**
  `crustyimg responsive hero.jpg --widths 320,640,1280 --formats avif,webp --manifest images.json`
  The manifest is what an SSG/build consumes (see §9).
- **Just the srcset for one format** **[today]** — use `responsive` with a single `--formats webp`.

## 3. Resize / thumbnail / crop

- **Cap the long edge** **[today]** — `crustyimg resize photo.jpg --max 1200` (never upscales)
- **Exact box, cover-crop** **[today]** — `crustyimg resize photo.jpg --cover 800x800`
  Scales to fill, then center-crops to exactly 800×800. The resize modes are **mutually
  exclusive** — pass exactly one of `--max`, `--fit`, `--fill`, `--cover`, `--exact`, `--percent`.
- **Fit inside a box, keeping aspect** **[today]** — `crustyimg resize photo.jpg --fit 800x600`
- **Thumbnail** **[today]** — `crustyimg thumbnail photo.jpg --size 256` (`--square` to crop square)
- **Crop to an arbitrary rect / gravity / aspect** **[planned — PROJ-006]**
  `crustyimg crop photo.jpg --gravity center --aspect 1:1`
  Today `resize --cover WxH` covers the common center-crop-to-a-box case.
- **Rotate / flip / trim / pad** **[planned — PROJ-006]**
- **Smart (content-aware) crop** **[planned — PROJ-006]**
  `crustyimg crop photo.jpg --smart attention --aspect 16:9`

## 4. Format conversion

- **To WebP (lossless default; lossy with a quality)** **[today]**
  `crustyimg convert logo.png --format webp` · `crustyimg convert photo.jpg --format webp -q 80` **[feat:webp-lossy]**
- **To AVIF** **[today]** — `crustyimg convert photo.jpg --format avif`
  AVIF output is a **default** feature — on in every released binary. Only a lean
  `--no-default-features` build drops it (there `--format avif` exits `4` with a rebuild hint).
- **Batch PNG → WebP** **[today]**
  `crustyimg convert "img/**/*.png" --format webp --out-dir web/`
  Conversion is a *command*, not a recipe op — recipes carry pixel ops plus a terminal
  `optimize`, so batch format migration goes through `convert` with a glob or a directory.
- **Let the engine choose the format instead** **[today]** — `optimize` / `web` decide per image
  (§1); use `convert` when you need a *specific* format.

## 5. Privacy & metadata (the verifiable-privacy lane)

- **Strip ALL metadata before publishing** **[today]** — `crustyimg meta strip photo.jpg`
- **Remove location only, keep copyright** **[today]** — `crustyimg meta clean --gps photo.jpg`
- **Stamp copyright/artist across a batch** **[today]**
  `crustyimg meta set --artist "Jane Doe" --copyright "© 2026 Jane Doe" photo.jpg`
- **Copy metadata from an original to a derivative** **[today]** — `crustyimg meta copy --from orig.jpg --to edited.jpg`
- **Note:** `optimize` drops GPS/metadata by default (privacy-safe web prep); `--keep-gps` opts out.
- **Audit a tree for metadata leaks (fail CI)** **[today]**
  `crustyimg lint assets/ --select privacy` → `privacy/gps-metadata-leak` and
  `privacy/camera-metadata`, each with a runnable fix; exit `7` on an error-severity finding.

## 6. Compositing, watermark, effects

- **Logo watermark, corner, semi-transparent** **[today]**
  `crustyimg watermark photo.jpg --image logo.png --gravity southeast --opacity 0.3 --scale 0.15`
- **Tiled watermark across the frame** **[today]** — add `--tile` to the image form.
- **Text watermark** **[today]** — `crustyimg watermark photo.jpg --text "© 2026" --gravity south`
  (`--font`, `--size`, `--color`; a font ships bundled so no `--font` is required).
- **Redact a region (pixelate / solid mask)** **[planned — PROJ-006]** — privacy-flavored, on the moat.
- **Auto color (normalize / auto-contrast)** **[planned — PROJ-006]** — automatic only, not manual sliders.
- **Upscale a small asset (Lanczos)** **[planned — PROJ-006]** — `crustyimg resize logo.png --max 512 --allow-upscale`.

## 7. Saved recipes (tune once → replay everywhere)

A recipe is an ordered op pipeline in TOML, replayed across a glob/dir in parallel. **[today]**

**Three ship inside the binary** — call them by name, no file needed:

| Name | Long edge | For |
|---|---|---|
| `web` | 2048px | the general web-prep default |
| `gallery` | 2560px | full-bleed gallery / lightbox images |
| `product` | 1600px | product cards, catalogue thumbnails |

```sh
crustyimg apply --recipe web "assets/**/*.{jpg,png}" --out-dir dist/img -j 8
```

`--recipe` takes a path **or** a bundled name, and a real file on disk always wins — a local
`web.toml` shadows the bundled `web`. An argument that is neither exits `3`.

- **Record your own from a tuned edit** **[today]**
  `crustyimg edit in.jpg --auto-orient --resize-max 1600 --save-recipe mine.toml`
  then `crustyimg apply --recipe mine.toml "assets/**/*.jpg" --out-dir dist/img -j 8`.
  `edit` records only the ops it exposes — `--auto-orient`, `--resize-max`, `--invert`.
- **Write one by hand for anything else** **[today]** — a cover-crop, or the terminal
  `optimize` step the bundled recipes end in, is written directly as TOML. Format and op
  vocabulary: [`recipes/README.md`](../recipes/README.md). An `avatar` recipe (square 256×256
  then modernize) is `auto-orient` → `resize` `mode="cover"` → `optimize`.
- **The round-trip is byte-stable** — `edit` output == `apply`-of-the-saved-recipe output, so a
  recipe reviewed in a PR is exactly what runs in CI.
- **Many targets in one declarative file** **[today]** — `crustyimg build` reads
  `crustyimg.build.toml`, where each `[[target]]` binds sources to a recipe and an out dir.
  Adds a content-addressed cache (incremental rebuild), a lockfile (`--check`/`--frozen` drift
  gate), and `--watch`. This is the "Makefile for images" path when `apply` stops scaling.

## 8. CI / verification

- **Visual-regression gate (fail the build if optimization hurt quality)** **[today]**
  `crustyimg diff original.jpg optimized.jpg --fail-under 90` → exit 7 if SSIMULACRA2 < 90.
- **Lint an asset tree (source-file, no URL, exit code)** **[today]**
  `crustyimg lint assets/ --format json` → exit `7` on any error-severity finding, `0` clean.
  `--format sarif` feeds GitHub code-scanning; `--select`/`--ignore` take ruff-style rule
  prefixes (`privacy`, `size`, `dims`, `color`, `orient`, `format`) and an unknown prefix is a
  usage error, not a silent no-op. Some rules are opt-in by design — `dims/oversized-dimensions`
  needs a declared `--max-intended-width`, `size/oversized-bytes` needs a configured budget.
- **As a GitHub Action** **[today]**
  `uses: jysf/crustyimg-action@v1` — lint mode gives inline PR annotations + a job-summary
  table; `mode: optimize` re-encodes the tree instead. Or `uses: jysf/setup-crustyimg@v1` to
  install the binary and call any command directly.
- **As a pre-commit hook** **[today]** — `repo: https://github.com/jysf/crustyimg`, hook id
  `crustyimg-lint`. The format-aware upgrade from `check-added-large-files`.
- **Reproducibility gate** **[today]** — `crustyimg build --check` (or `--frozen`) fails when the
  lockfile and the produced outputs drift.

## 9. Rolling it into a static-site generator / build tool

**The one universal pattern (works in every tool):** crustyimg runs as a **(pre-)build step**,
optimizes the asset tree, and emits a **path-keyed JSON manifest**; the SSG's data/template layer
reads it. Manifest **[planned — PROJ-005]**; the optimize/responsive/build steps work **[today]**.

```make
# the universal build target — run before the SSG; then the SSG reads data/images.json
images:
	crustyimg optimize assets/ --out-dir public/img --manifest data/images.json
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
  unlocks all six at once and never breaks). `crustyimg build` + a `crustyimg.build.toml` is the
  sturdier form of this today (§7) — declared targets, cached, lockfile-gated.

**Recommended sequencing:** ship the generic **`--manifest`** flag + docs → then the two native
plugins that showcase crustyimg best: **Eleventy** (`eleventy-crustyimg`) and **Astro** (image
service). (Ranked targets + per-tool sources live in the roadmap's Track B / the research notes.)

## 10. Photography / bulk

- **Downscale a shoot for sharing** **[today]** — `crustyimg apply --recipe gallery "shoot/*.jpg" --out-dir out/ -j 8`
- **RAW → web** **[today]** — `.dng`, `.cr2`, `.nef`, `.arw` and more open directly, by extracting
  the camera's embedded full-res JPEG preview. That is not a RAW *develop* (no demosaic, no white
  balance), but it is enough to feed a RAW straight into `web`/`optimize`.
- **HEIC → web** **[feat:heic]** — opt-in at build time and never in a released binary (immature
  permissive HEVC decoders + patent exposure); it also needs libheif's codec backend installed.
  Without the feature, pre-convert externally (`sips` / `heif-convert`).
- **Contact sheet / montage** — **[planned — later]**.

---

### Which recipes need which project
- **[today]:** the whole §1–§8 core — web/optimize/resize/thumbnail/convert/responsive(HTML)/
  meta(strip/clean/copy/set)/watermark/edit/apply/build/lint/diff, minus the §2 manifest item.
- **PROJ-002:** `optimize` auto-decides format + `--explain` — **shipped**.
- **PROJ-004:** `lint` (+ the GitHub Actions) — **shipped**.
- **PROJ-005:** `--manifest`, `favicon`, `placeholder`, dominant color — **planned**; unlocks §9.
- **PROJ-006:** `crop`/smart-crop, rotate/flip/trim/pad, redaction, auto-color, upscaling — **planned**.
- **PROJ-007:** `build` + cache + lockfile + `--watch` — **shipped**.
- **PROJ-008:** the wasm core + browser demo (recipes run in the browser via `transform()`) — **shipped**.
- **PROJ-009:** RAW preview / AVIF / SVG input reach, HEIC behind `feat:heic` — **shipped**.
