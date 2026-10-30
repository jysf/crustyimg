---
name: crustyimg
description: Prepare images for the web with the crustyimg CLI — downscale and convert to AVIF/WebP/PNG, resize, thumbnail, build responsive srcset sets, strip EXIF/GPS metadata, watermark, batch-process folders with recipes, and gate image quality or size in CI. Use when the user wants to optimize, compress, resize, convert, or clean images for a website, or check image assets in a pipeline.
---

# crustyimg

`crustyimg` is a single-binary Rust CLI that makes images web-ready. Reach for it
whenever a task involves shrinking, resizing, converting, or cleaning image files —
it replaces ad-hoc chains of ImageMagick, cwebp, avifenc and exiftool.

Written against crustyimg **0.8.x**. Run `crustyimg <command> --help` when in doubt;
the binary is the source of truth.

## Check it is installed

```sh
crustyimg --version
```

If missing, install one of these ways (ask the user before installing):

```sh
brew install jysf/tap/crustyimg
cargo install crustyimg
```

Prebuilt binaries: https://github.com/jysf/crustyimg/releases

## Pick the command

| The user wants… | Use |
|---|---|
| "Make this image web-ready" (smaller, modern format, sane size) | `web` |
| Smaller file, **same dimensions**, never bigger than the source | `optimize` |
| Under a byte budget (e.g. 200KB) | `optimize --max-size 200KB` |
| A visual-quality target | `optimize --target high` (or `visually-lossless`, `medium`) |
| A specific size or shape | `resize` |
| Small previews | `thumbnail` |
| A different format, nothing else changed | `convert --format …` |
| `<picture>`/srcset variants for a page | `responsive` |
| Remove metadata / location | `meta strip` / `meta clean --gps` |
| A logo or text overlay | `watermark` |
| The same pipeline over many files | `apply --recipe …` |
| A repeatable, cached image build for a project | `build` |
| Find problems in an asset folder (CI) | `lint` |
| Compare two images / quality gate | `diff` |
| Inspect an image | `info` (`--exif`, `--json`) |

## Rules that save you a failed run

- **One input → `-o FILE`. Many inputs → `--out-dir DIR`.** Passing several inputs with
  `-o` is a usage error (exit 2). `--out-dir` is created if missing.
- **Existing outputs are not overwritten** without `-y` (exit 5). Add `-y` in
  non-interactive runs, but only when overwriting is what the user wants.
- **Quote globs** so crustyimg expands them, including recursive `**`:
  `'assets/**/*.jpg'`. A directory works as an input too, but reads only the images directly
  inside it, not its subfolders (a folder of only subfolders exits 3); use `'assets/**/*'`
  for a whole tree.
- **The output format** comes from `--format`, else the extension of `-o` or of
  `--name-template` (`'{stem}.png'` writes PNG), else a recipe's `format`, else the source's
  own format is kept. `web` and `optimize` choose the format themselves unless you pin one.
  To keep each file's format in a template, use `{ext}`. **`build` is different:** the
  target's `name` extension decides, else the recipe's `format`, else the source's format.
  `build` refuses `--format`, `-o`, `--out-dir` and `--name-template` (exit 2, nothing
  written): set them in `crustyimg.build.toml` instead (see Project builds).
- **GPS is dropped by default** on any command that re-encodes pixels. Pass `--keep-gps`
  only if the user asks to keep location data.
- **Work on copies** when the user's originals are precious; crustyimg only writes the
  outputs you name, but a copy is cheap.

## Web prep

```sh
crustyimg web hero.jpg -o hero.avif                 # one image
crustyimg web 'photos/*.jpg' --out-dir dist/img -y   # a folder
crustyimg web hero.jpg --max 1200 -o hero.avif      # tighter long-edge bound (default 2048)
```

`web` bakes EXIF orientation, strips metadata, downscales the long edge (never
upscales), picks the smallest modern format (AVIF for photos, lossless WebP/PNG for
graphics), and prints the result's SSIMULACRA2 quality score.

Because `web` resizes, a source that was already small and tightly compressed can come
back **larger** — crustyimg says so ("N% larger"). When the user needs "never bigger,
keep my dimensions", use `optimize`:

```sh
crustyimg optimize photo.jpg -o photo.avif                   # fast, never bigger
crustyimg optimize photo.jpg --verify -o photo.avif          # also report the quality score
crustyimg optimize photo.jpg --target high -o photo.jpg -y   # search for a quality target
crustyimg optimize photo.jpg --max-size 200KB -o photo.jpg -y
crustyimg optimize photo.jpg --ssim 80 -o photo.avif         # AVIF at a quality score
```

If `optimize` can't beat the source, it keeps the original bytes and says so.
`--target` and `--ssim` search for the lowest quality that meets the score on JPEG, AVIF
and lossy WebP outputs.

## Resize and thumbnails

Pass exactly one mode:

| Mode | Result for a 300×150 source with `100x100` |
|---|---|
| `--max 100` | 100×50 — long edge bounded, never upscales |
| `--fit 100x100` | 100×50 — fits inside the box, keeps aspect, never upscales |
| `--fill 100x100` | 100×100 — scales to cover, then **crops** to exactly the box |
| `--cover 100x100` | 200×100 — scales to cover, **no crop** (may upscale) |
| `--exact 100x100` | 100×100 — stretched, aspect ignored |
| `--percent 50` | 150×75 |

For a square avatar or card image, use `--fill`, not `--cover`.

```sh
crustyimg resize photo.jpg --max 1200 -o out.jpg
crustyimg resize 'img/*.jpg' --fill 800x800 --out-dir square/ -y
crustyimg thumbnail 'img/*.png' --size 150 --square --out-dir thumbs/ -y
```

## Convert

```sh
crustyimg convert photo.png --format avif -o photo.avif
crustyimg convert 'img/*.png' --format webp --out-dir out/ -y
```

Formats: `png`, `jpeg`, `gif`, `bmp`, `tiff`, `ico`, `webp`, `avif`.

**WebP output is lossless only** — `-q` does not make it lossy. For a smaller *photo*,
convert to `avif` (or let `web`/`optimize` choose). WebP is the right pick for graphics.

## Responsive images

```sh
crustyimg responsive hero.jpg --widths 480,960,1440 --formats avif,jpeg --out-dir dist/img
```

Writes one file per width × format into `--out-dir` (never upscaling) and prints a
paste-ready `<picture>` element on stdout. Give the snippet to the user and remind them
to fill in `alt`. `--no-snippet` suppresses it.

## Metadata and privacy

These edit the file container only — no re-encode, no quality loss (JPEG and PNG).

```sh
crustyimg meta strip photo.jpg -o clean.jpg           # remove all EXIF/IPTC/XMP/ICC
crustyimg meta clean photo.jpg --gps -o nogeo.jpg     # remove only location tags
crustyimg meta set photo.jpg --artist "Jane Doe" --copyright "© 2026" -o tagged.jpg
crustyimg meta copy --from original.jpg --to edited.jpg -o merged.jpg  # without -o it rewrites edited.jpg, so needs -y
crustyimg auto-orient photo.jpg -o upright.jpg        # bake rotation into pixels
```

`meta strip`, `meta clean` and `meta set` keep the input's format: an `-o` or template that
names another format is refused (exit 4, nothing written). Convert first, then edit metadata.

These commands drop any Content Credentials (C2PA) manifest, since editing the file
would break its signature; `clean`/`copy`/`set` print a warning when they do.

Check that location data is gone (this matches tag names, not the file path):

```sh
crustyimg info clean.jpg --exif | grep -E '^\s+GPS' || echo "no GPS tags"
```

## Watermark

```sh
crustyimg watermark photo.jpg --image logo.png --gravity southeast --opacity 0.6 --scale 0.1 -o out.jpg
crustyimg watermark photo.jpg --text "© Studio" --size 32 --color FFFFFF -o out.jpg
```

Use exactly one of `--image` or `--text`. Gravity: `north`, `south`, `east`, `west`,
`center`, `northeast`, … (default `southeast`). `--scale` sets the logo's width as a fraction
of the image's width (`0.1` = a tenth); `--scale` and `--tile` are image-only.

The mark must suit each image, or crustyimg refuses it rather than damage the picture:

- The mark, plus `--margin`, must fit inside the image.
- A fixed-size mark (no `--scale`, no `--tile`) may cover at most 5 % of the image. For a logo
  over images of mixed sizes, use `--scale` (e.g. `0.1`), which is not held to the 5 %.
- On one input (`watermark` or `apply --recipe`) a refused mark is an error, exit 2, nothing
  written. With several inputs, or in `build`, that image is written **without** the mark,
  with a `warning: … watermark skipped …` line, and the run still exits 0 — read stderr.
- `--size` is a number above 0 and at most 1024. A bad `--size`, `--scale`, `--opacity`,
  `--gravity` or `--color` exits 2 before any file is read.

A mark that fits is drawn as asked, even if too small or faint to see; check the output.

## Recipes: one pipeline, many files

Three recipes are built in and callable by name:

| Recipe | Long edge | For |
|---|---|---|
| `web` | 2048px | General web images |
| `gallery` | 2560px | Full-bleed / lightbox images |
| `product` | 1600px | Product cards, catalogue images |

```sh
crustyimg apply --recipe product 'catalog/**/*.jpg' --out-dir dist/products -j 8 -y
```

`apply` runs in parallel (`-j N`, default = CPU count). If some inputs fail, the rest
still write and it exits 6 with a summary.

For a custom pipeline, write a TOML recipe:

```toml
version = "1"
name = "avatar"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "fill"      # exactly 256x256, cropped
width = 256
height = 256

[[step]]
op = "optimize"    # optional; must be the last step
```

Ops: `auto-orient`, `resize` (`mode` = `max` + `width`; `fit`/`fill`/`cover`/`exact` +
`width` + `height`; `percent` + `percent`), `watermark`, `invert`, `identity`, `optimize`.

A recipe can also watermark, and pin its output format and quality. `format`/`quality` need
`version = "2"`:

```toml
version = "2"
format = "webp"
quality = 80

[[step]]
op = "resize"
mode = "max"
width = 1600

[[step]]
op = "watermark"
image = "logo.png"   # read from the directory you run crustyimg in
scale = 0.1
opacity = 0.8
margin = 16
```

A `text` mark uses `text = "…"` with optional `font`, `size` and `color` instead of `image`.
On `apply --recipe`, `--format`, `-o` and a template extension still win over the recipe's
`format`; on `build`, the target's name extension wins over it.
`-q` wins over its `quality`. Keep `version = "1"` and leave both keys out if crustyimg 0.7.x must run it.

Every step key is checked when the recipe loads. A misspelt or unknown key, a wrong type or
range, or a key the step's mode never reads exits 1 with nothing written, naming the step
(numbered from 0) and the key, e.g.
`recipe step 1 ('resize'): unknown key 'widht'; expected one of: mode, width, height, percent`.
Fix the recipe; nothing is ignored silently.

Name outputs with `--name-template '{stem}_web.{ext}'`. A literal extension in the template
(`'{stem}.png'`) sets the output format.

## Project builds (repeatable, cached)

For a site that keeps its source images in the repo, declare targets in
`crustyimg.build.toml`:

```toml
version = 1

[[target]]
source = "assets/**/*.png"
recipe = "web"
out    = "dist/img"
name   = "{stem}.{ext}"   # optional
```

```sh
crustyimg build            # runs every target; only changed inputs are rebuilt
crustyimg build --watch    # rebuild on change
crustyimg build --check    # CI: exit 7 if outputs drifted from crustyimg.build.lock
```

The manifest owns every output: the folder is `out`, the file name is `name`, and the
format is `name`'s extension (`"{stem}.webp"`), else the recipe's `format`. `build` refuses
`--format`, `-o`/`--output`, `--out-dir` and `--name-template` with exit 2 and writes nothing,
including with `--watch` and `--check`.

`build` overwrites its own outputs without `-y`. Commit `crustyimg.build.lock`; add
`.crustyimg/` (the local cache) to `.gitignore`. `recipe` can be a path to your own recipe
file; editing a watermark's logo or font rebuilds the outputs that use it.

## Verify and gate in CI

```sh
crustyimg diff original.jpg out.avif                    # SSIMULACRA2 score, higher = closer
crustyimg diff original.jpg out.avif --fail-under 70    # exit 7 if below 70
crustyimg lint 'assets/**/*'                            # report problems, each with a fix
crustyimg lint 'assets/**/*' --format sarif > results.sarif   # GitHub code scanning
```

As a rough guide to the score: ~90 is visually lossless, ~70 is high quality, ~50 is
medium.

## Machine-readable output

- `info --json`, `diff --json`, `lint --format json`.
- `web`/`optimize --json` prints a decision report (candidates, winner, bytes, savings,
  score). The image must go to `-o FILE` or `--out-dir` — with no output, stdout is
  already the image and `--json` is refused (exit 2). It is also refused when the format is
  pinned (`--format`, an `-o` or template extension), since nothing was decided. Add
  `--timing` for timings.

## Exit codes

| Code | Meaning | What to do |
|---|---|---|
| 0 | Success | — |
| 1 | Runtime error (decode/encode failed, resource limit), or a recipe that fails to load | Read stderr; fix the named recipe step |
| 2 | Usage error, or a watermark refused on a single input | Fix the flags; check `--help` |
| 3 | Input not found, or the glob matched nothing | Check the path / quoting |
| 4 | Format not available: not built into this binary (e.g. HEIC, or AVIF on a lean build), or `meta` asked to change it | Use another format, rebuild with the feature, or `convert` first |
| 5 | Output write refused (exists, symlink, path traversal) | Add `-y` if overwriting is intended |
| 6 | Some inputs in a batch failed | Others were written; read the summary |
| 7 | A check failed (`diff --fail-under`, `build --check`, `lint` error) | Report it; this is a gate, not a crash |

## Inputs

Reads JPEG, PNG, WebP, AVIF, GIF, BMP, TIFF, ICO, SVG, and camera RAW (via the
embedded preview). HEIC is only available in builds compiled with the `heic` feature
(otherwise exit 4). SVG needs the `svg` feature, which is on by default; only a source build
with `--no-default-features` lacks it (exit 4, "rebuild with --features svg").

An SVG is rasterized at its own size. Its `<text>` draws in the bundled Go font whatever
`font-family` it names, upright and regular even when bold or italic: crustyimg never reads
system fonts. If the user needs a particular typeface, convert the text to paths first. `-` reads from stdin; `-o -` writes to stdout with all diagnostics on
stderr.
