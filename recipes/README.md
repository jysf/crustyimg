# Bundled recipes

A **recipe** is an ordered pipeline of image operations saved as TOML, optionally with the
output format and quality it should write. Tune it once,
replay it across a whole tree in parallel, and get the same bytes every time.

The three files here are compiled into the binary, so you can call them **by name** with
no file on disk:

| Name | Long edge | Pipeline | For |
|---|---|---|---|
| [`web`](web.toml) | 2048px | auto-orient → resize → optimize | The general web-prep default |
| [`gallery`](gallery.toml) | 2560px | auto-orient → resize → optimize | Full-bleed gallery / lightbox images |
| [`product`](product.toml) | 1600px | auto-orient → resize → optimize | Product cards, catalogue thumbnails |

```sh
crustyimg apply --recipe web *.jpg --out-dir out/ -j 8
```

The final `optimize` step picks the smallest modern format that beats the resized image —
AVIF for photos, lossless WebP/PNG for graphics — and reports the SSIMULACRA2 score.

> Because these recipes **resize**, a source already below the long-edge bound can come out
> larger than the original. That is reported plainly, never hidden. To keep the original
> dimensions with a never-bigger guarantee, use `crustyimg optimize` instead.

## Name or path

`--recipe <arg>` is tried as a **file path first**; only if no such file exists does it fall
back to the bundled names. So a local `web.toml` in your working directory unambiguously
shadows the bundled `web`, and existing path-based invocations keep working unchanged.

```sh
crustyimg apply --recipe web      *.jpg --out-dir out/   # bundled
crustyimg apply --recipe ./web.toml *.jpg --out-dir out/ # your file, explicitly
```

An argument that is neither a readable file nor a known bundled name exits `3`.

## Writing your own

The quickest route is to tune on one image and let `edit` write the recipe:

```sh
crustyimg edit hero.jpg --auto-orient --resize-max 1600 --save-recipe mine.toml
crustyimg apply --recipe mine.toml *.jpg --out-dir out/ -j 8
```

The round trip is **byte-stable** — replaying the saved recipe reproduces the `edit` output
byte for byte, so a recipe reviewed in a PR is exactly what runs in CI.

`edit` only records the ops it exposes (`--auto-orient`, `--resize-max`, `--invert`). For
anything else — a crop to an exact box, a watermark, an output format, or the terminal
`optimize` step the bundled recipes end in — write the TOML directly:

```toml
version = "1"
name = "avatar"
description = "Square 256x256 avatar, then modernize."

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "fill"
width = 256
height = 256

[[step]]
op = "optimize"
```

### File format

| Key | Notes |
|---|---|
| `version` | Required. `"1"`, or `"2"` to use `format` or `quality`. Any other value is refused (exit `1`). |
| `name`, `description` | Optional, for humans. |
| `format` | Optional, needs `version = "2"`. The output format: `png`, `jpeg` (or `jpg`), `gif`, `bmp`, `tiff`, `ico`, `webp`, `avif`. |
| `quality` | Optional, needs `version = "2"`. Encoder quality, an integer `0`–`100`, for formats that take one. |
| `[[step]]` | One per operation, applied in file order. |

Operations available in a recipe:

| `op` | Params |
|---|---|
| `auto-orient` | none — bakes EXIF orientation into pixels, clears the tag |
| `resize` | `mode` + its params (below) |
| `invert` | none |
| `identity` | none — a no-op |
| `watermark` | an `image` or `text` mark and its placement (below) |
| `optimize` | none — **terminal only**; picks the smallest modern format and scores it |

`resize` modes:

| `mode` | Required params | Behavior |
|---|---|---|
| `max` | `width` | Bound the long edge. Never upscales. |
| `fit` | `width`, `height` | Fit inside the box, keeping aspect. |
| `fill` | `width`, `height` | Scale to cover the box, then crop to exactly `width`×`height`. |
| `cover` | `width`, `height` | Scale to cover the box, **no crop** — one dimension usually exceeds it. May upscale. |
| `exact` | `width`, `height` | Resize to exactly `width`×`height`, ignoring aspect. |
| `percent` | `percent` | Scale by a percentage. |

`optimize` is only valid as the **last** step — it produces encoded bytes and a format
choice, not a transformed image. Anywhere else it is rejected as `unknown operation
'optimize'` (exit `1`) rather than silently reordered. A trailing `optimize` is also what
makes `apply --recipe web` identical to the `web` verb, and what `--json` reports on.

### Output format and quality

A `version = "2"` recipe can say what it writes:

```toml
version = "2"
name = "card"
format = "webp"
quality = 80

[[step]]
op = "resize"
mode = "max"
width = 1200
```

`apply --recipe` (at one input or many) and `build` both honour it and write the same bytes.
On `apply --recipe` the flags still win, in this order:

1. `--format`;
2. the extension of `-o`, or of a `--name-template` (`'{stem}.png'` writes PNG);
3. the recipe's `format`;
4. otherwise the source's own format.

On `build`, the order is: the extension of the target's `name` (`name = "{stem}.png"` writes
PNG), then the recipe's `format`, then the source's own format. `build` ignores `--format`.

`-q` wins over the recipe's `quality`. On a recipe that ends in `optimize`, a `format` pins the
encode and the automatic choice is skipped. A `version = "1"` recipe that sets either key is
refused when it loads, exit `1`:

```text
error: recipe field 'format' requires `version = "2"` (found version "1")
```

crustyimg 0.7.1 and earlier cannot read a recipe that sets `format` or `quality`. Leave both out,
and keep `version = "1"`, for a recipe older binaries must run.

### The `watermark` step

A `watermark` step sets exactly one of `image` (a logo file) or `text`:

```toml
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 1600

[[step]]
op = "watermark"
image = "logo.png"
gravity = "southeast"
opacity = 0.8
margin = 16

[[step]]
op = "watermark"
text = "© Studio"
size = 24
color = "ffffffcc"
gravity = "southwest"
margin = 16
```

| Key | Mode | Value |
|---|---|---|
| `image` | image | Path to the overlay file. |
| `text` | text | The text to draw. |
| `font` | text | Path to a TTF/OTF font file. Optional; the bundled font is used without it. |
| `size` | text | Font size in pixels, a finite number `> 0` and `<= 1024`. Default `32`. |
| `color` | text | Hex `RRGGBB`, `#RRGGBB` or `RRGGBBAA`. Default `ffffff`. |
| `gravity` | both | `center`, `north`, `south`, `east`, `west`, `northeast`, `northwest`, `southeast` or `southwest`. Default `southeast`. |
| `opacity` | both | `0.0`–`1.0`. Default `1.0`. |
| `margin` | both | Pixels from the anchored edges, a whole number `>= 0`. Default `0`. |
| `scale` | image only | Overlay width as a fraction of the image's width (`0.1` = a tenth), aspect kept. A finite number `> 0`. Without it the overlay keeps its own size. |
| `tile` | image only | `true` repeats the overlay across the image. Default `false`. |

- `image` and `font` paths are read from the **directory you run crustyimg in**, not the
  recipe's folder.
- A step with both `image` and `text`, or neither, is refused when the recipe loads (exit `1`).
- An `image` or `font` file that cannot be read stops the run before anything is written
  (exit `1`), naming the step and the file.
- `build` caches by the file's content as well as its path, so editing `logo.png` rebuilds
  every output that uses it.

### Step keys are checked

A step key its op does not take, a value of the wrong type or out of range (`nan`/`inf`
included), or a key the step's mode never reads (`height` with `mode = "max"`) refuses the
recipe when it is loaded — exit `1`, nothing written — rather than being ignored. The error
names the step, **numbered from 0**, the op and the key:

```text
error: recipe step 1 ('resize'): unknown key 'widht'; expected one of: mode, width, height, percent
```

An integer is fine where a number is expected (`percent = 50`). Every key's type and range
is listed in [`docs/data-model.md`](../docs/data-model.md).

A `watermark` step's `scale` and `tile = true` apply only to an image mark (`image = …`);
a `text` step that sets either is refused the same way. So is an `image` step that sets
`font`, `size` or `color`.

### A watermark that does not fit is left off

A `watermark` step is checked against each image before it is drawn: the mark (after
`scale`) plus its `margin` must fit inside the image, and a mark of fixed size (no `scale`,
no `tile`) may cover at most 5 % of the image's pixels. On one input a mark that fails is an
error (exit 2); the input count is taken after globs and directories resolve, so one
resulting image is refused and two or more are skipped. Over several inputs, or in `build`, that image is written **without** the
mark — every other step still applies — and stderr says so:

```text
warning: in/small.png: watermark skipped, image written without it: the 169×31 px mark does not fit inside the 24×24 px image
```

The other refusals read:

```text
the mark would cover 100.00 % of the 800×532 px image (425600 of 425600 px); the limit is 5 %
the scaled mark does not fit inside the 800×532 px image
```

The second is for a `scale` so large the mark could never fit. A mark that fits is drawn as
asked, even one too small or faint to see (`opacity = 0`, a tiny `size`).

## Beyond one recipe

- **Many source → output pairs in one file:** `crustyimg build` reads a
  `crustyimg.build.toml` manifest where each `[[target]]` binds sources to a recipe and an
  output directory — plus a content-addressed cache, a lockfile, and `--watch`.
- **In the browser:** the same TOML runs through the wasm `transform()` binding, which is
  what the demo page executes.
- **A catalog of workflows:** [`docs/recipes.md`](../docs/recipes.md) — the copy-paste
  cookbook (web prep, responsive sets, privacy, CI gates, bulk photography).
- **Every flag and exit code:** [`docs/cli-reference.md`](../docs/cli-reference.md).
