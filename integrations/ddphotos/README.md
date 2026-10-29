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
/edit/index.html                          pick an album
/edit/index.html?album=<slug>             pick a photo
/edit/index.html?album=<slug>&photo=<n>   edit photo n (1-based, like ddphotos permalinks)
```

Link to `/edit/index.html`, not `/edit/`. ddphotos' dev server (`./ddphotos run`) only
serves files from `config/static/`, so a folder URL falls through to the site's 404 page.
Production servers usually map `/edit/` to `index.html`, but the full name works everywhere.

## Install

```
just ddphotos-edit-install ~/my-ddphotos/config
```

That builds the slim wasm package (below) and copies the page into
`config/static/edit/`. Rebuild the site (`./ddphotos photogen`, then `./ddphotos build`)
and open `/edit/index.html`.

The page and its JavaScript are about 70 KB. The engine is a separate `.wasm`: the
**slim build**, 3.9 MB raw or 0.78 MB with Brotli, 37% smaller than the full browser
build (1.23 MB). It is the full build without SVG input, which a ddphotos site never
feeds it (`just wasm-build-slim`, into `pkg-slim/`). It only downloads when someone opens `/edit/`, never on the gallery pages.
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

Work runs in a Web Worker, so the page stays responsive during an encode. On a 1200px
copy of a real sample photo, AVIF took 7.5 s (142 KB) and JPEG took 19.8 s (402 KB):
the JPEG quality search is the slow part in wasm.

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

Against ddphotos' own sample site, built with its real tools: `make sample-photogen`
(needs libvips) and `make sample-build`, then served by its Vite dev server with this
page in `sample/config/static/edit/`. Album, photo and editor views, JPEG and AVIF
downloads (checked with the browser's own decoder), recursive-album photos, and videos
being skipped were all exercised there. Not yet tested on Apache, nginx or Cloudflare
Pages.
