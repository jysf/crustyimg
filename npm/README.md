# crustyimg-wasm

Resize, convert, and optimize images **in the browser** — the [crustyimg](https://github.com/jysf/crustyimg)
engine compiled to WebAssembly.

No native addon. No postinstall compile step. No server. The package is one `.wasm` file plus
generated JavaScript glue, so it installs identically on every OS and CPU, and the pixels never
leave the user's machine.

```bash
npm install crustyimg-wasm
```

## Caveats

Read this before integrating:

- **You must call `await init()` (or `initSync()`) once before anything else.** There is no
  auto-instantiation — see Usage below.
- **Every call is synchronous and single-threaded.** There is no worker or thread pool inside the
  `.wasm` — a call runs on whatever JS thread invokes it and blocks that thread until it returns. A
  multi-megapixel AVIF encode can take several seconds; run it in a Web Worker if you don't want
  your page's main thread (or UI) to freeze for that long.
- **AVIF encodes here but does not decode here.** An AVIF *input* throws. `score()` (below) also
  can't grade an AVIF *output* directly, because scoring decodes both images — decode the AVIF in
  the browser first (`createImageBitmap`) and score the decoded bytes instead.
- Not a drop-in server-side `sharp`/`libvips` replacement — this runs client-side (or in Node via
  `initSync`), with the format reach and performance profile of a wasm build, not a native one.

## Usage

The package is an ES module built with `wasm-pack --target web`, so **you must initialize it
once** before calling anything: the `.wasm` is a separate ~5.7 MB file (~1.3 MB over the wire with
Brotli), and the package will not fetch it behind your back.

### Browser

```js
import init, { info, transform, optimize, score, version } from "crustyimg-wasm";
import wasmUrl from "crustyimg-wasm/crustyimg_bg.wasm?url"; // Vite; see below for other setups

await init({ module_or_path: wasmUrl });

const bytes = new Uint8Array(await file.arrayBuffer()); // e.g. from an <input type="file">

const meta = info(bytes);
console.log(meta.width, meta.height, meta.format, meta.hasAlpha);

const recipe = `
version = "1"

[[step]]
op = "resize"
mode = "fit"
width = 1200
`;

const png = transform(bytes, recipe, "png");
const jpeg = optimize(bytes, "jpeg"); // let the engine pick a quality that hits the perceptual target

const quality = score(bytes, jpeg); // SSIMULACRA2, 0-100+; ~100 is visually identical to the input
console.log(quality);
```

`optimize` runs the same perceptual quality search the CLI runs (an SSIMULACRA2-targeted binary
search), so it is doing real work — expect it to take meaningfully longer than a plain `transform`.
`optimizeDetailed` (see the API table) runs the same search but also reports the quality/speed it
picked and its own score, so you don't have to call `score()` yourself for a non-AVIF result.

Serve the `.wasm` with `Content-Type: application/wasm` and, ideally, Brotli — it is a static
asset, and it is the download the user waits for.

### Node

`fetch` cannot read `file://` URLs, so in Node hand the bytes to `initSync` directly:

```js
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { initSync, info, transform } from "crustyimg-wasm";

const require = createRequire(import.meta.url);
const wasm = readFileSync(require.resolve("crustyimg-wasm/crustyimg_bg.wasm"));
initSync({ module: wasm });

console.log(info(readFileSync("photo.png")).width);
```

### Bundlers

The `--target web` build works with Vite, webpack 5, esbuild, and Rollup — all of them can hand you
a URL for the `.wasm` asset (the exact spelling differs: `?url` in Vite, `new URL("...", import.meta.url)`
in webpack/Rollup). What matters is that **you** resolve the URL and pass it to `init()`.

## API

| Export | Signature | What it does |
|---|---|---|
| `init` (default) | `(opts?: { module_or_path }) => Promise<…>` | Fetch/instantiate the `.wasm`. Call once, await it. |
| `initSync` | `({ module: BufferSource }) => …` | Instantiate from bytes you already have (Node, or a preloaded `ArrayBuffer`). |
| `info` | `(bytes) => ImageInfo` | Decode and report `width`, `height`, `format`, `hasAlpha`. |
| `transform` | `(bytes, recipeToml, outFormat) => Uint8Array` | Run a recipe (resize, crop, watermark, auto-orient, …) and encode. |
| `optimize` | `(bytes, outFormat \| "auto") => Uint8Array` | Re-encode well: pick a format when asked to, and search for the smallest quality that still hits the perceptual target. |
| `optimizeDetailed` | `(bytes, outFormat, speed?, maxBytes?, target?) => OptimizeResult` | Same engine as `optimize`, but reports what it did: `.bytes`, `.format`, `.quality`, `.speed` (AVIF only), `.score` (perceptual search only — `undefined` for AVIF, which this build can't decode back to score), `.scoredBy` (`"engine"` or `"none"`). |
| `score` | `(reference, candidate) => number` | The engine's SSIMULACRA2 score between two encoded images. Both must be formats this build can *decode* — so an AVIF candidate must be decoded elsewhere (e.g. the browser's `createImageBitmap`) before scoring. |
| `version` | `() => string` | The crate version this `.wasm` was built from. |

Every fallible call **throws a typed `Error`** on bad input — it never panics the module out from
under your page.

Recipes are the same TOML the CLI reads, built through the same operation registry, so a recipe you
tuned in the terminal replays byte-for-byte in the browser. See the
[recipe docs](https://github.com/jysf/crustyimg#recipes).

## Formats

|  | Formats |
|---|---|
| **Decode (in)** | PNG, JPEG, GIF, WebP, SVG (rasterized) |
| **Encode (out)** | PNG, JPEG, GIF, WebP, **AVIF** |

AVIF is **encode-only** here: the browser already decodes AVIF natively (`createImageBitmap`), so
shipping a second AVIF decoder into the bundle would have bought nothing. Feed an AVIF input
through the browser's own decoder first. TIFF/BMP/ICO decode and HEIC/RAW inputs are native-CLI
only — they are not worth their weight in a browser bundle.

An unsupported input, or an output format this build cannot encode, returns a typed error rather
than a surprise.

## Versioning

The npm version tracks the `crustyimg` crate version exactly — `crustyimg-wasm@x.y.z` is the
crate's `x.y.z` compiled to wasm. Pre-1.0, a minor bump may break the API.

## Related

- **[`crustyimg`](https://github.com/jysf/crustyimg)** — the native CLI (`view`, `apply`, `optimize`,
  `lint`, `build`, …). This package is its engine, in your page.

## License

MIT OR Apache-2.0.
