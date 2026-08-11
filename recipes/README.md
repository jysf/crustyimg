# Bundled recipes

A **recipe** is an ordered pipeline of image operations saved as TOML. Tune it once,
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
anything else — a cover-crop, or the terminal `optimize` step the bundled recipes end in —
write the TOML directly:

```toml
version = "1"
name = "avatar"
description = "Square 256x256 avatar, then modernize."

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "cover"
width = 256
height = 256

[[step]]
op = "optimize"
```

### File format

| Key | Notes |
|---|---|
| `version` | Required. `"1"` today. |
| `name`, `description` | Optional, for humans. |
| `[[step]]` | One per operation, applied in file order. |

Operations available in a recipe:

| `op` | Params |
|---|---|
| `auto-orient` | none — bakes EXIF orientation into pixels, clears the tag |
| `resize` | `mode` + its params (below) |
| `invert` | none |
| `identity` | none — a no-op |
| `optimize` | none — **terminal only**; picks the smallest modern format and scores it |

`resize` modes:

| `mode` | Required params | Behavior |
|---|---|---|
| `max` | `width` | Bound the long edge. Never upscales. |
| `fit` | `width`, `height` | Fit inside the box, keeping aspect. |
| `fill` | `width`, `height` | Scale to fill the box (may exceed one dimension). |
| `cover` | `width`, `height` | Scale to fill, then crop to exactly `width`×`height`. |
| `exact` | `width`, `height` | Resize to exactly `width`×`height`, ignoring aspect. |
| `percent` | `percent` | Scale by a percentage. |

`optimize` is only valid as the **last** step — it produces encoded bytes and a format
choice, not a transformed image. Anywhere else it is rejected as `unknown operation
'optimize'` (exit `1`) rather than silently reordered. A trailing `optimize` is also what
makes `apply --recipe web` identical to the `web` verb, and what `--json` reports on.

## Beyond one recipe

- **Many source → output pairs in one file:** `crustyimg build` reads a
  `crustyimg.build.toml` manifest where each `[[target]]` binds sources to a recipe and an
  output directory — plus a content-addressed cache, a lockfile, and `--watch`.
- **In the browser:** the same TOML runs through the wasm `transform()` binding, which is
  what the demo page executes.
- **A catalog of workflows:** [`docs/recipes.md`](../docs/recipes.md) — the copy-paste
  cookbook (web prep, responsive sets, privacy, CI gates, bulk photography).
- **Every flag and exit code:** [`docs/cli-reference.md`](../docs/cli-reference.md).
