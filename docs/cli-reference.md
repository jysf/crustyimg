# crustyimg — CLI reference

The complete command reference. For a quick tour of the handy examples, see the
[README Usage section](../README.md#usage); for the design contract (stdin/stdout rules,
per-command guarantees), see [`docs/api-contract.md`](api-contract.md).

```
crustyimg [GLOBAL OPTIONS] <COMMAND> [INPUT...] [OPTIONS]
```

## Conventions

- **Inputs are positional.** Most commands accept a single file, a **glob**
  (`'*.jpg'` — quote it to let crustyimg expand it, incl. recursive `**`), a
  **directory**, or `-` (stdin).
- **Multiple inputs require `--out-dir`.** Writing many results to one `-o` file is a
  usage error (exit 2). Output names follow `--name-template` (default keeps the stem +
  the target extension).
- **Per-command fan-out is sequential;** `apply --recipe` is the parallel path (`-j N`).
- **Pipes stay clean:** with `-o -` (stdout) all diagnostics go to stderr.
- **Default drop-GPS:** pixel-lane encodes strip GPS unless you pass `--keep-gps`.

## Global options

Available on every command (before or after the subcommand):

| Option | Short | Description |
|---|---|---|
| `--output <PATH>` | `-o` | Output file for single-input commands. `-` = stdout. |
| `--out-dir <DIR>` | | Output directory for multi-input / batch commands (created if missing). |
| `--name-template <T>` | | Output name template, e.g. `{stem}_web.{ext}` (`{stem}`, `{ext}`). |
| `--format <FMT>` | | Force the output format (else inferred from `-o`'s extension, or kept). |
| `--quality <0-100>` | `-q` | Encoder quality where the format supports it (e.g. JPEG). |
| `--jobs <N>` | `-j` | Parallel workers for `apply` batch (default = CPU count). |
| `--yes` | `-y` | Assume "yes" to overwrite prompts (non-interactive). |
| `--quiet` | `-Q` | Suppress non-error output. |
| `--verbose` | `-v` | Increase verbosity (repeatable, `-vv`); logs to stderr. |
| `--keep-gps` | | Keep GPS/location tags on pixel-lane encodes (default drops them). |
| `--version` / `--help` | `-V` / `-h` | Standard. |

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success. |
| `1` | Runtime error (decode/encode/op failed; input exceeds a resource limit). |
| `2` | Usage error (bad args) — clap's standard code. |
| `3` | Input not found / unreadable (or an empty glob). |
| `4` | Unsupported format / codec not built (AVIF output without `--features avif`; a `.heic` input without `--features heic`). |
| `5` | Output write refused (exists without `--yes`; path traversal; symlinked destination). |
| `6` | Partial batch failure (some inputs failed; summary on stderr; others still wrote). |
| `7` | A check/gate was not satisfied (e.g. `diff --fail-under` scored below the threshold). |

---

## View & inspect

### `view <INPUT> [--width N] [--height N]`
Display an image directly in the terminal (via `viuer`). `--width`/`--height` bound the
render size. Requires a TTY; on the lean (`--no-default-features`) build, `view` is
omitted.
```sh
crustyimg view photo.jpg
crustyimg view photo.jpg --width 80
```

### `info <INPUT> [--exif] [--json]`
Print dimensions, format, byte size on disk, color type, bit depth, alpha, and EXIF/ICC
presence. `--exif` dumps EXIF tags; `--json` emits machine-readable JSON to stdout.
```sh
crustyimg info photo.jpg
crustyimg info photo.jpg --exif
crustyimg info photo.jpg --json
```

### `diff <A> <B> [--fail-under N] [--json]`
Compute an SSIMULACRA2 perceptual similarity score of `B` vs `A` (higher = more similar).
`--fail-under N` turns it into a CI visual-regression gate: a score below `N` exits `7`
(distinct from a runtime error). `--json` emits a machine-readable result.
```sh
crustyimg diff original.jpg compressed.jpg
crustyimg diff original.jpg compressed.jpg --fail-under 70
```

---

## Resize & thumbnail

### `resize <INPUT...> <MODE>`
Resize with a SIMD backend. Exactly one **mode** is required:

| Mode | Meaning |
|---|---|
| `--max <N>` | Bound the long edge to `N` px. **Never upscales.** |
| `--fit <WxH>` | Fit inside `W×H`, keeping aspect. **Never upscales.** |
| `--exact <WxH>` | Resize to exactly `W×H` (ignores aspect). |
| `--percent <P>` | Scale to `P` percent. |
| `--fill <WxH>` | Scale to fill `W×H` (may exceed one dimension). |
| `--cover <WxH>` | Scale to fill then **crop** to exactly `W×H`. |

```sh
crustyimg resize photo.jpg --max 1200 -o out.jpg
crustyimg resize photo.jpg --cover 800x800 -o square.jpg
crustyimg resize *.jpg --max 800 --out-dir web/
```

### `thumbnail <INPUT...> [--size N] [--square]`
Convenience resize to a small bounded size. `--square` crops to a centered square.
```sh
crustyimg thumbnail photo.jpg --size 200 --square -o thumb.jpg
crustyimg thumbnail *.png --size 150 --out-dir thumbs/
```

---

## Web optimize & convert

### `shrink <INPUT...> [--max N] [--target T | --ssim S | --max-size SIZE]`
Optimize for web: resize (long edge, default ≤ 1600 px) + quality encode + strip
metadata. Choose **at most one** quality mode:

| Option | Meaning |
|---|---|
| `--target <visually-lossless\|high\|medium>` | Auto-tune quality to a perceptual preset (SSIMULACRA2). |
| `--ssim <0-100>` | Auto-tune quality to a specific SSIMULACRA2 score. |
| `--max-size <SIZE>` | Fit under a byte budget (e.g. `200KB`); lowers quality, then dimensions. |

```sh
crustyimg shrink photo.jpg --max 1200 -o out.webp
crustyimg shrink photo.jpg --max 1600 --target high -o out.jpg
crustyimg shrink photo.jpg --max-size 200KB -o out.jpg
```

### `optimize <INPUT...> [--max N] [--target T | --ssim S | --max-size SIZE]`
One-button "make it web-good": auto-orient + strip metadata + perceptual re-encode,
**visually-lossless by default**, format/size-preserving. `--max` optionally bounds the
long edge; `-o`/`--format` pick the output format; the quality flags override the default
target.
```sh
crustyimg optimize photo.jpg -o out.webp
crustyimg optimize *.jpg --out-dir web/
```

### `convert <INPUT...> --format FMT [--max-size SIZE]`
Pure re-encode to another format (no pixel changes). `--format` is required
(`png`/`jpeg`/`gif`/`bmp`/`tiff`/`ico`/`webp`; `avif` needs a build with `--features
avif`). `--max-size` fits a byte budget (JPEG target).
```sh
crustyimg convert photo.png --format webp -o out.webp
crustyimg convert *.png --format webp --out-dir out/
```

### `responsive <INPUT> --widths W1,W2,… [--formats F1,F2,…] [--no-snippet]`
Generate a width × format responsive set into `--out-dir` (never upscaling) and print a
paste-ready `<picture>`/srcset snippet to stdout. `--no-snippet` suppresses the snippet.
Single input.
```sh
crustyimg responsive hero.jpg --widths 320,640,1280 --formats webp,jpeg --out-dir web/
```

---

## Orientation & metadata

The metadata commands operate on the image **container** — pixels are not re-decoded, so
these carry no quality cost.

### `auto-orient <INPUT...>`
Bake the EXIF orientation into pixels, then clear the tag (fixes silent-rotation). A no-op
when no orientation tag is present.
```sh
crustyimg auto-orient photo.jpg -o fixed.jpg
```

### `strip <INPUT...>`
Remove **all** container metadata (EXIF/IPTC/XMP/ICC). JPEG and PNG.
```sh
crustyimg strip photo.jpg -o clean.jpg
crustyimg strip *.jpg --out-dir clean/
```

### `clean <INPUT...> --gps`
Selectively remove **only** GPS/location tags, preserving everything else (orientation,
copyright, ICC). JPEG and PNG.
```sh
crustyimg clean photo.jpg --gps -o nogeo.jpg
```

### `set <INPUT...> [--artist S] [--copyright S] [--description S]`
Write named EXIF tags (creating a fresh EXIF block if the input has none).
```sh
crustyimg set photo.jpg --artist "Jane Doe" --copyright "© 2026" -o tagged.jpg
```

### `copy-metadata --from SRC --to DST`
Copy EXIF + ICC from one image's container onto another (pixels untouched). JPEG in v1.
```sh
crustyimg copy-metadata --from original.jpg --to edited.jpg
```

---

## Watermark

### `watermark <INPUT...> (--image PATH | --text STRING) [options]`
Overlay an **image** or **text** watermark at a gravity anchor. Exactly one of `--image`
or `--text` is required (mutually exclusive).

| Option | Applies to | Meaning |
|---|---|---|
| `--image <PATH>` | image | Overlay image. |
| `--text <STRING>` | text | Text to render. |
| `--gravity <G>` | both | Anchor: `north`/`south`/`east`/`west`/`center`/`northeast`/… (default `southeast`). |
| `--opacity <O>` | both | Opacity `0.0`–`1.0`. |
| `--margin <M>` | both | Margin from the edge, in px. |
| `--scale <S>` | image | Scale the overlay proportionally (e.g. `0.1`). |
| `--tile` | image | Tile the overlay across the image. |
| `--font <PATH>` | text | TTF/OTF font (default: bundled Go font). |
| `--size <N>` | text | Font size in px (default `32`). |
| `--color <HEX>` | text | `RRGGBB` / `#RRGGBB` / `RRGGBBAA` (default `ffffff`). |

```sh
crustyimg watermark photo.jpg --image logo.png --gravity southeast --opacity 0.6 -o out.jpg
crustyimg watermark photo.jpg --text "© crustyimg" --size 32 --color FFFFFF -o out.jpg
```

---

## Recipes & batch

### `edit <INPUT> [--auto-orient] [--resize-max N] [--invert] [--save-recipe FILE]`
Chain an ordered op list on a **single** image in one decode → ops → encode pass. Ops
apply in a fixed canonical order regardless of flag order (`auto-orient` → `resize` →
`invert`), so the result is deterministic. At least one op flag is required.
`--save-recipe FILE` serializes the chain to a TOML recipe (byte-pinned to what `apply`
of that recipe produces).
```sh
crustyimg edit photo.jpg --auto-orient --resize-max 1600 -o out.jpg
crustyimg edit hero.jpg --auto-orient --resize-max 1600 --save-recipe web.toml
```

### `apply --recipe FILE <INPUT...>`
Replay a saved recipe across a file, glob, or directory **in parallel** (rayon; `-j N`
bounds workers, default = CPU count) with a progress bar on stderr. Honors the global
`--out-dir` and `--name-template`. Per-input failures are summarized and exit `6`; other
inputs still write.
```sh
crustyimg apply --recipe web.toml *.jpg --out-dir out/ --name-template "{stem}_web.{ext}" -j 8
```

### `build [FILE]`
Run a **declared build**: every `[[target]]` in a build manifest (default
`./crustyimg.build.toml`). Each target binds sources — a glob, directory, or path, or a
list of them — to a recipe file, an output directory, and an optional name template.
Paths in the manifest resolve relative to the working directory.

Every target is validated up front (recipe parsed, pipeline probed, sources resolved), so
a typo in one target aborts the build before any output is written. Targets then run
through the same parallel per-input path as `apply`: per-output failures are summarized
and exit `6`; a malformed manifest exits `2`; a missing manifest or recipe exits `3`.

Unlike `apply`, `build` **overwrites its own declared outputs without `--yes`** — a build
owns its output tree and must be re-runnable (DEC-057). It only ever writes inside each
target's `out` directory.

```toml
# crustyimg.build.toml
version = 1

[[target]]
source = "assets/**/*.png"      # or ["a/*.png", "b/"]
recipe = "recipes/web.toml"
out    = "dist/img"
name   = "{stem}_web.{ext}"     # optional; default "{stem}.{ext}"
```
```sh
crustyimg build                 # discovers ./crustyimg.build.toml
crustyimg build ci.build.toml -j 8
crustyimg build --no-cache      # rebuild everything, ignore the cache
crustyimg build --check         # verify against the lockfile; exit 7 on drift
```

#### Incremental rebuilds (the cache)

`build` is **incremental**. Before decoding an input it computes a key from everything
that can change the output — the source's bytes and extension, the resolved recipe, the
encode quality, and this binary's version and codec features — and looks it up in a
local content-addressed store at `.crustyimg/cache/` (relative to the working
directory). On a hit it writes the cached output and skips the decode/pipeline/encode
entirely; on a miss it does the work and stores the result.

The summary reports both:

```
built 1 target, 8 outputs (0 cached, 8 rebuilt)   # a cold build
built 1 target, 8 outputs (8 cached, 0 rebuilt)   # a re-run with no changes
built 1 target, 8 outputs (7 cached, 1 rebuilt)   # one source edited
```

The cache is **local only** — a directory, no network, no cache server. It is a pure
optimization: a hit restores a deleted output byte-for-byte, and a corrupt, truncated,
or deleted entry falls back to a clean rebuild rather than serving bad bytes. Clear it
with `rm -rf .crustyimg` (there is no automatic eviction yet), and add `.crustyimg/` to
your `.gitignore`. `--no-cache` bypasses it in both directions: no entry is read, none
is written, and every input is rebuilt.

#### The lockfile (`--check` / `--frozen`)

Every build writes **`crustyimg.build.lock`** next to the manifest — commit it. It records
one entry per output: the file it wrote, the source and recipe that produced it, the
**cache key** (an identity of the *inputs*), the **hash** of the bytes written, and their
size — plus one `[env]` block naming the crustyimg version, `arch-os`, and features the
build ran under. Outputs are sorted by path, so two clean builds on one machine produce a
byte-identical lockfile and a review diff shows only what actually moved.

`build --check` re-runs the build and compares it against the committed lockfile instead of
refreshing it. It exits **0** when they agree and **7** on drift, naming each output that
moved, and it **never modifies the lockfile**. `--frozen` and `--locked` are aliases (there
is no network to lock out). A missing lockfile under these flags is drift, not an error.

```sh
crustyimg build --check           # the CI gate: exit 7 if the build drifted
crustyimg build --check --strict  # also fail on cross-environment byte differences
```

What counts as drift is the honest part. The **key** is a function of the inputs alone, so
it reproduces on any machine: if a source, recipe, quality, or the tool version changed, the
key changes and `--check` fails — always. An **output-hash** difference is judged against
`[env]`: on the *same* `arch-os` it is a real regression (exit 7), on a *different* one it is
expected encoder variance, reported as a note but not a failure. `--strict` promotes that
note to a failure, for shops pinned to one toolchain and arch that want byte-identity
enforced. crustyimg does not promise cross-arch byte-identity, because its encoders can't.

For the review-grade question — *did the image actually change?* — compare pixels, not
encoder bytes: `crustyimg diff a.png b.png --fail-under 90` (SSIMULACRA2, also exit 7).

A malformed, oversized, or unknown-version lockfile exits **2** before anything is built.
A plain `crustyimg build` owns the file and simply regenerates it.

---

## Shell

### `completions <bash|zsh|fish|powershell|elvish>`
Print a shell-completion script to stdout. Installing it into your shell's completion
directory is your (or your package manager's) step.
```sh
crustyimg completions zsh > "${fpath[1]}/_crustyimg"
crustyimg completions bash >> ~/.bash_completion
crustyimg completions fish > ~/.config/fish/completions/crustyimg.fish
```
