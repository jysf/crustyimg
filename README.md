# crustyimg

**Tell the tool the outcome you want — a visual quality or a file-size budget,
in a modern format — and get the smallest file that meets it, from one pure-Rust
binary with zero system dependencies.** "Set the look, not the number."

A fast CLI for viewing and transforming images: resize, shrink/optimize-for-web,
inspect, strip metadata, watermark, and generate responsive image sets — all from
a single static binary with no system dependencies.

## Install

**Works today:**

```sh
# From source (always current)
cargo install --git https://github.com/jysf/crustyimg

# Or clone and build manually
git clone https://github.com/jysf/crustyimg
cd crustyimg
cargo build --release
# binary: ./target/release/crustyimg
```

**Once v0.1.0 is published** (see [RELEASING.md](RELEASING.md) for the release checklist):

```sh
# cargo (crates.io)
cargo install crustyimg

# Homebrew
brew install jysf/tap/crustyimg

# Prebuilt binary — download from the GitHub Releases page
# https://github.com/jysf/crustyimg/releases
```

### Feature notes

The `view` command (terminal image preview) is **on by default** — a plain
`cargo install` or release binary includes it. For a headless, smaller binary
(CI / server / container), build without it:

```sh
cargo install --git https://github.com/jysf/crustyimg --no-default-features
```

Two additional codecs are opt-in (compile-time features):

| Feature | What it adds |
|---|---|
| `webp-lossy` | Lossy WebP encode (libwebp, C dep) — by default WebP is lossless only |
| `avif` | AVIF output via ravif (pure Rust, no nasm/system libs) |

Enable with `--features webp-lossy,avif` at build/install time.

## Usage

Run `crustyimg --help` for the full surface, or `crustyimg <cmd> --help` for a
command's options. Every transform accepts a single file, a glob, a directory, or `-`
(stdin); see [**Batch & recipes**](#batch--recipes-multiple-files) below to run over many.

### View & inspect

```sh
crustyimg view photo.jpg                     # display in the terminal (viuer)
crustyimg info photo.jpg                      # dimensions, format, size, color, EXIF/ICC presence
crustyimg info photo.jpg --exif               # dump EXIF tags
crustyimg info photo.jpg --json               # machine-readable JSON to stdout
```

### Resize & thumbnail

```sh
crustyimg resize photo.jpg --max 1200 -o out.jpg   # bound the long edge (never upscales)
crustyimg resize photo.jpg --exact 800x600 -o out.jpg
crustyimg resize photo.jpg --percent 50 -o out.jpg
crustyimg resize photo.jpg --fit 800x800 -o out.jpg    # fit inside, keep aspect
crustyimg resize photo.jpg --cover 800x800 -o out.jpg  # fill + crop to exactly 800x800
crustyimg thumbnail photo.jpg --size 200 --square -o thumb.jpg
```

### Optimize / shrink for web

```sh
# One-button "make it web-good": auto-orient + strip metadata + visually-lossless re-encode
crustyimg optimize photo.jpg -o out.webp

# Shrink: resize long edge to ≤1200 px, encode as WebP
crustyimg shrink photo.jpg --max 1200 -o out.webp

# Perceptual auto-quality: smallest file that still clears a visual target (JPEG/WebP)
crustyimg shrink photo.jpg --max 1600 --target high -o out.jpg
crustyimg shrink photo.jpg --ssim 85 -o out.jpg

# Byte budget: fit under a size, lowering quality then dimensions as needed
crustyimg shrink photo.jpg --max-size 200KB -o out.jpg
```

### Convert formats

```sh
crustyimg convert photo.png --format webp -o out.webp   # PNG/JPEG/GIF/BMP/TIFF/ICO/WebP
crustyimg convert photo.jpg --format webp --max-size 150KB -o out.webp
crustyimg convert photo.jpg --format avif -o out.avif   # AVIF: needs a build with `--features avif`
```

### Auto-orient & metadata

```sh
crustyimg auto-orient photo.jpg -o fixed.jpg     # bake EXIF orientation into pixels, clear the tag
crustyimg strip photo.jpg -o clean.jpg            # remove ALL metadata (EXIF/IPTC/XMP/ICC)
crustyimg clean photo.jpg --gps -o nogeo.jpg      # remove only GPS/location, keep the rest
crustyimg set photo.jpg --artist "Jane Doe" --copyright "© 2026" -o tagged.jpg
crustyimg copy-metadata --from original.jpg --to edited.jpg   # copy EXIF+ICC between images
```

> Pixel-lane encodes drop GPS by default; pass `--keep-gps` to retain it.

### Watermark

```sh
crustyimg watermark photo.jpg --image logo.png --gravity southeast --opacity 0.6 -o out.jpg
crustyimg watermark photo.jpg --image logo.png --tile --scale 0.1 -o out.jpg
crustyimg watermark photo.jpg --text "© crustyimg" --size 32 --color FFFFFF -o out.jpg
```

### Compare (perceptual diff)

```sh
# SSIMULACRA2 score of b vs a; --fail-under makes it a CI visual-regression gate (exit 7 if below)
crustyimg diff original.jpg compressed.jpg --fail-under 70
crustyimg diff a.png b.png --json
```

### Responsive image sets

```sh
# Width × format variants + a paste-ready <picture>/srcset snippet on stdout
crustyimg responsive hero.jpg --widths 320,640,1280 --formats webp,jpeg --out-dir web/
```

### Batch & recipes (multiple files)

Pass many inputs (a list, a glob, or a directory) to any transform — multi-input runs
require `--out-dir`, and `--name-template` controls output names (`{stem}`, `{ext}`):

```sh
crustyimg shrink *.jpg --max 1200 --out-dir web/                 # a glob
crustyimg convert photos/ --format webp --out-dir out/           # a whole directory
crustyimg thumbnail *.png --size 200 --square --out-dir thumbs/
crustyimg strip *.jpg --out-dir clean/ --name-template "{stem}_clean.{ext}"
```

For a repeatable multi-step pipeline over a large set, tune it once, save a recipe, then
replay it **in parallel** (`-j` workers, progress bar):

```sh
# Tune on one image and save the recipe
crustyimg edit hero.jpg --auto-orient --resize-max 1600 --save-recipe web.toml

# Replay across a batch, in parallel
crustyimg apply --recipe web.toml *.jpg \
  --out-dir out/ --name-template "{stem}_web.{ext}" -j 8
```

> Per-command fan-outs run sequentially; `apply --recipe` is the parallel path (`-j N`,
> default = CPU count). A failed input in a batch doesn't abort the rest — the run exits
> `6` (partial batch) with a stderr summary.

### Piping (stdin / stdout)

```sh
# `-` reads stdin / writes stdout; all diagnostics stay on stderr so pipes are clean
crustyimg resize - --max 800 -o - < in.jpg > out.jpg
cat in.jpg | crustyimg convert - --format webp -o - > out.webp
```

### Handy global options

`-o/--output` (`-` = stdout) · `--out-dir` · `--name-template` · `-q/--quality` ·
`--format` · `-j/--jobs` · `-y/--yes` (assume yes to overwrite) · `-Q/--quiet` ·
`-v/--verbose` · `--keep-gps`. Exit codes: `0` ok, `1` runtime error, `2` usage, `3`
input not found, `4` unsupported format, `5` output refused, `6` partial batch, `7`
check failed (e.g. `diff --fail-under`).

## Shell completions

`crustyimg` can generate completion scripts for bash, zsh, fish, powershell, and
elvish. Pipe the output into your shell's completion directory:

```sh
# zsh — add to your $fpath
crustyimg completions zsh > "${fpath[1]}/_crustyimg"

# bash — append to your completions file
crustyimg completions bash >> ~/.bash_completion

# fish
crustyimg completions fish > ~/.config/fish/completions/crustyimg.fish

# powershell
crustyimg completions powershell >> $PROFILE

# elvish
crustyimg completions elvish >> ~/.config/elvish/lib/completions.elv
```

Where to install the script is your (or your package manager's) step — the
command writes the script to stdout only.

## Changelog & releases

- **[CHANGELOG.md](CHANGELOG.md)** — what changed in each version, in
  [Keep a Changelog](https://keepachangelog.com) format. The `[Unreleased]`
  section tracks work merged since the last release.
- **[RELEASING.md](RELEASING.md)** — the versioning policy (SemVer; `0.x` minor
  bumps may carry breaking CLI changes), the `vX.Y.Z` annotated-tag convention,
  and the release-cut checklist a maintainer follows to publish a new version.

## License

`crustyimg` is dual-licensed under **MIT OR Apache-2.0** — use whichever suits
you. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

## Developing crustyimg

crustyimg is built with a **spec-driven workflow** (Claude plays every role —
architect, implementer, reviewer — across separate sessions). The full contributor
guide lives in **[docs/development.md](docs/development.md)**; the agent conventions
are in **[AGENTS.md](AGENTS.md)**.
