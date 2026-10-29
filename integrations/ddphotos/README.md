# An edit page for ddphotos sites (prototype)

A static page that adds **"make a copy of this photo"** to a
[ddphotos](https://github.com/dougdonohoe/ddphotos) site: resize it, add a text
watermark, re-encode it as JPEG, AVIF, WebP or PNG, and download the result. It runs
crustyimg compiled to WebAssembly, in the visitor's browser. Nothing is uploaded, and
the site still needs no server.

It does not modify ddphotos. It reads only the files a ddphotos site already
publishes, and it is installed through ddphotos' own `config/static/` folder, which
the site build copies to the web root.

```
/edit/                          pick an album
/edit/?album=<slug>             pick a photo
/edit/?album=<slug>&photo=<n>   edit photo n (1-based, like ddphotos permalinks)
```

## Install

```
just ddphotos-edit-install ~/my-ddphotos/config
```

That builds the wasm package and copies the page into `config/static/edit/`. Rebuild
the site (`./ddphotos photogen`, then `./ddphotos build`) and open `/edit/`.

The page and its JavaScript are about 70 KB. The engine is a separate `.wasm` of about
5.8 MB, or 1.3 MB with Brotli. It only downloads when someone opens `/edit/`, never on the gallery pages.
Serve `.wasm` as `application/wasm`. Apache, nginx, Cloudflare Pages and S3 do this by
default. ddphotos' Vite dev server does not, so in `./ddphotos run` the browser logs a
warning and loads it more slowly, but it still works.

## What it can do

| | |
|---|---|
| **Size** | Long edge: as published, 1200, 1080, 800 or 400 px. Never upscales. |
| **Watermark** | Text, position, size (scaled to the output), colour, opacity. Uses crustyimg's bundled font. |
| **Format** | JPEG (searched for the smallest file that still looks like the edit), AVIF (fixed high quality), PNG, WebP (lossless only in this build), or let the engine choose. |
| **Preview** | Live, re-rendered by the engine about 200 ms after each change. |
| **CLI** | Every result shows the same edit as a crustyimg recipe, plus the `crustyimg apply` command to run on the original file. |

Work runs in a Web Worker, so the page stays responsive during an encode.

## Limits

- **It edits the published copy, not the original.** ddphotos publishes a 1600 px lossy
  WebP. The originals stay with the site owner. The CLI recipe on each result is the way
  to make the same edit from the original.
- **No crop, rotate or colour.** crustyimg recipes have only `resize`, `auto-orient`,
  `watermark` and `invert` today. Crop and rotate need a spec. Colour grading is planned
  in PROJ-015.
- **Edits are not saved.** The site is static, so the page can only download files.
- **No way in from the lightbox.** ddphotos has no hook for adding a button to its
  photo viewer, so visitors reach `/edit/` by URL. A small upstream change to ddphotos
  would be needed for an "Edit" button.
- **Password-protected albums are skipped.** Their index files are encrypted.
- **The CLI and the page choose JPEG quality differently.** The CLI's `optimize` step
  uses a fast fixed quality (q75). The browser build searches for a similarity score of
  90, which often lands near q95. The edit is the same, but the files differ in size.

## Licence

ddphotos is AGPL-3.0. This page is crustyimg's (MIT or Apache-2.0). It contains no
ddphotos code. It only reads the JSON a ddphotos site publishes
(`/albums/config.json`, `/albums/albums.json`, `/albums/<slug>/index.json`).

## How it was tested

Against a mock site built in the JSON shape of ddphotos' `pkg/photogen/json.go`, using
ddphotos' sample photos as 1600 px lossy WebPs. It has **not** been run against real
`photogen` output, because `photogen` needs libvips.
