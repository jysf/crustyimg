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

Always link `/edit/index.html`, never `/edit/`. The folder URL fails on both of
ddphotos' own servers:

- **Dev server** (`./ddphotos run`): it serves only the files in `config/static/`, so
  `/edit/` falls through to the site's 404 page.
- **ddphotos' Apache image**: its `.htaccess` strips trailing slashes, so `/edit/`
  redirects to `/edit`, which shows an Apache directory listing of the page's files.
  An `edit.html` redirect next to the folder does not help.

A plain static server maps `/edit/` to `index.html`, but the full name works everywhere.

## Install

You need, in a crustyimg checkout:

- [`just`](https://github.com/casey/just) and Node.js (`brew install just node`)
- rustup's **stable** toolchain with the wasm target. Homebrew's own `cargo` does not
  have it; the build finds the rustup one itself, even if Homebrew's is first on
  `PATH`:
  ```
  rustup toolchain install stable
  rustup target add --toolchain stable wasm32-unknown-unknown
  ```
- `wasm-pack` (`brew install wasm-pack`). binaryen is not needed: the build turns
  `wasm-opt` off on purpose.

Then:

```
just ddphotos-edit-install ~/my-ddphotos/config
```

That builds the slim wasm package (below) and copies the page into
`config/static/edit/`. Rebuild the site (`./ddphotos build`, or `make web-npm-build`);
photogen does not need to run again. A running dev server picks the page up without a
restart. Then open `/edit/index.html`.

The page and its JavaScript are about 35 KB. The engine is a separate `.wasm`: the
**slim build**, the full browser build without SVG input, which a ddphotos site never
feeds it (`just wasm-build-slim`, into `pkg-slim/`). It downloads only when someone
opens a photo in the editor, never on the gallery pages or the album and photo
pickers.

How much a visitor downloads depends on the server:

| Server | `.wasm` transfer |
|---|---|
| Compresses `application/wasm` (Cloudflare Pages does, with Brotli) | 0.78 MB |
| Does not compress it (ddphotos' Apache image and its `nginx.conf`, as shipped) | 3.9 MB |

On Apache, one line in the site config or `.htaccess` compresses it (needs `mod_deflate`):

```
AddOutputFilterByType DEFLATE application/wasm
```

On nginx, add `application/wasm` to `gzip_types` with `gzip on;`.

Serve `.wasm` as `application/wasm`. Apache, nginx, Cloudflare Pages and S3 do this by
default. ddphotos' Vite dev server does not, so in `./ddphotos run` the browser logs a
warning and loads it more slowly, but it still works.

## What it can do

| | |
|---|---|
| **Size** | Long edge: as published, 1200, 1080, 800 or 400 px. Never upscales. |
| **Watermark** | Text, position, size (scaled to the output, and made smaller when the text would not fit a portrait or narrow photo), colour, opacity. Uses crustyimg's bundled font. |
| **Format** | JPEG (searched for the smallest file that scores 90 for similarity), AVIF (fixed high quality), PNG, WebP (lossless only in this build), or let the engine choose (the smallest file at high, not identical, quality: usually AVIF). |
| **Preview** | Live, re-rendered by the engine about 200 ms after each change. |
| **CLI** | Every result shows the same edit as a crustyimg recipe, plus the `crustyimg apply` command to run on the original file. Run it in the folder the album's `base:` points to in `albums.yaml`, with the recipe saved as `edit.toml`: the command uses the photo's path relative to that folder. |

Work runs in a Web Worker, so the page stays responsive during an encode. The JPEG
quality search is the slow part in wasm: on 1200 px copies in headless Chromium on an
Apple-silicon Mac, JPEG took about 3 s and AVIF about 1.3 s. A phone or an older laptop
has not been measured.

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
- **The CLI does not convert colour profiles yet.** The page edits the published copy,
  which photogen has already converted to sRGB, so its download is right. Run on an
  original in Display P3 (most iPhone photos) or Adobe RGB, the CLI drops the profile
  without converting the pixels, and the copy looks duller. Use the page's download for
  those photos until this is fixed.
- **JPEG copies are large.** The published copy is already a lossy WebP, and the
  search for a similarity of 90 to it usually lands at q95 to q100: about 650 KB for a
  1200 px copy, where the CLI's fast fixed quality (q75) gives about 290 KB from the
  original. On some photos even q100 scores just under 90; the page says so when that
  happens.

## Licence

ddphotos is AGPL-3.0. This page is crustyimg's (MIT or Apache-2.0). It contains no
ddphotos code. It only reads the JSON a ddphotos site publishes
(`/albums/config.json`, `/albums/albums.json`, `/albums/<slug>/index.json`).

## How it was tested

On a stock ddphotos site (its sample albums plus our own, one of them
password-protected), built with ddphotos' own tools and served three ways: its Vite dev
server, the exported site on a plain static server, and ddphotos' own Apache Docker
image. The album and photo pickers, preview, every format choice, and the page's CLI
command on the originals were all exercised in headless Chromium, and every download
was decoded with independent decoders. Not yet tested on nginx or Cloudflare Pages.
