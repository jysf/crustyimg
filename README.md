# crustyimg

**Tell the tool the outcome you want — a visual quality or a file-size budget,
in a modern format — and get the smallest file that meets it, from one pure-Rust
binary with zero system dependencies.** "Set the look, not the number."

A fast CLI for viewing and transforming images: resize, optimize for web,
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

Three additional codecs are opt-in (compile-time features):

| Feature | What it adds |
|---|---|
| `webp-lossy` | Lossy WebP encode (libwebp, C dep) — by default WebP is lossless only |
| `avif` | AVIF output via ravif (pure Rust, no nasm/system libs) |
| `heic` | HEIC/HEIF **input** via system libheif — **local builds only**, see below |

Enable with `--features webp-lossy,avif` at build/install time.

#### HEIC is opt-in and never in a released binary

crustyimg **reads** AVIF, SVG, and RAW out of the box, but **not HEIC**. HEVC — the
codec inside a `.heic` — is covered by the Access Advance patent pool, and a
copyright license of any kind grants zero patent rights; separately, the mature
pure-Rust HEIC decoders are AGPL. Either blocker alone keeps HEIC off the default
path, so no release binary, Homebrew bottle, or `cargo install` default can decode
one. Running those on a `.heic` prints `HEIC decoding isn't built into this
crustyimg; convert the file to a supported format (JPEG, PNG, or WebP) first, or
rebuild with --features heic` and exits 4.

**Easiest fix — pre-convert.** crustyimg works in web/standard formats, so turn a
`.heic` into one first, then run crustyimg on the result:

```sh
sips -s format jpeg photo.heic --out photo.jpg   # macOS, built in
magick photo.heic photo.jpg                       # ImageMagick, any OS
```

Building `--features heic` (below) is only needed if you want crustyimg to read
`.heic` **directly** — for a one-off, pre-converting is faster.

If you accept those terms, build it yourself against a system libheif (≥ 1.17):

```sh
# macOS — Homebrew's libheif bundles its codec backends
brew install libheif

# Debian/Ubuntu — the HEVC decoder is a SEPARATE plugin package. Without it,
# libheif parses a .heic but fails to decode it ("Unsupported codec").
sudo apt-get install libheif-dev libheif-plugin-libde265

cargo build --release --features heic
```

Decode only — crustyimg never encodes HEIC. See
[docs/licensing.md](docs/licensing.md) for the LGPL attribution that a
redistributed `--features heic` build carries, and `decisions/DEC-052` for the
full rationale.

## Usage

Handy examples below; the **complete reference** (every command, flag, and exit code)
is in **[docs/cli-reference.md](docs/cli-reference.md)**. Or run `crustyimg --help` /
`crustyimg <cmd> --help`. Every transform accepts a single file, a glob, a directory, or
`-` (stdin); see [**Batch & recipes**](#batch--recipes-multiple-files) below to run over
many.

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

### Make it web-ready

```sh
# The flagship: downscale (long edge ≤2048), pick the smallest modern format that
# beats the DOWNSCALED image (AVIF for photos, lossless WebP/PNG for graphics), strip
# metadata, and report the SSIMULACRA2 score. Size-insensitive — a 24 MP photo is as
# fast as a small one. The downscale is the contract, so an already-small source above
# 2048px can come back larger than the original (reported honestly as "N% larger"); for
# an unconditional never-bigger guarantee that keeps dimensions, use `optimize` below.
crustyimg web photo.jpg -o out.avif
crustyimg web photo.jpg --max 1200 -o out.avif    # override the downscale bound

# Keep the original dimensions (the byte-primitive): fast fixed-quality, never bigger.
# Off by default the score is skipped; add --verify to report it for this run.
crustyimg optimize photo.jpg -o out.avif
crustyimg optimize photo.jpg --verify -o out.avif

# Perceptual auto-quality: smallest file that still clears a visual target (JPEG/WebP)
crustyimg optimize photo.jpg --target high -o out.jpg
crustyimg optimize photo.jpg --ssim 85 -o out.jpg

# Byte budget: fit under a size, lowering quality then dimensions as needed
crustyimg optimize photo.jpg --max-size 200KB -o out.jpg
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
crustyimg meta strip photo.jpg -o clean.jpg       # remove ALL metadata (EXIF/IPTC/XMP/ICC)
crustyimg meta clean photo.jpg --gps -o nogeo.jpg # remove only GPS/location, keep the rest
crustyimg meta set photo.jpg --artist "Jane Doe" --copyright "© 2026" -o tagged.jpg
crustyimg meta copy --from original.jpg --to edited.jpg      # copy EXIF+ICC between images
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
crustyimg web *.jpg --out-dir web/                               # a glob
crustyimg convert photos/ --format webp --out-dir out/           # a whole directory
crustyimg thumbnail *.png --size 200 --square --out-dir thumbs/
crustyimg meta strip *.jpg --out-dir clean/ --name-template "{stem}_clean.{ext}"
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

## Continuous integration

`crustyimg lint` is a **format-aware, no-URL, per-file** image-asset linter — the pre-deploy
check you run on `assets/`/`content/` in CI before anything is Lighthouse-able. It flags GPS/EXIF
privacy leaks, over-budget and wrong-format assets, non-baked orientation, corrupt files and more,
naming a runnable `crustyimg` fix for each and exiting **`7`** on any error-severity finding (a
CI-native gate). Drop it into any CI in three lines:

### GitHub Actions

The lint wrapper — installs crustyimg and annotates findings inline in the PR:

```yaml
name: images
on: [pull_request]
jobs:
  lint-images:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: jysf/crustyimg-action@v1
        with:
          paths: assets content
```

Or install the binary yourself with [`setup-crustyimg`](https://github.com/jysf/setup-crustyimg)
(generic — it enables `optimize`/`convert`/`lint` alike) and call it directly:

```yaml
      - uses: jysf/setup-crustyimg@v1
      - run: crustyimg lint assets content        # exit 7 fails the job on an error finding
      - run: crustyimg optimize assets --out-dir dist   # (any crustyimg command works)
```

- [`jysf/setup-crustyimg`](https://github.com/jysf/setup-crustyimg) — installs the CLI (via the
  checksum-verifying cargo-dist installer) on Linux/macOS/Windows.
- [`jysf/crustyimg-action`](https://github.com/jysf/crustyimg-action) — the lint/optimize wrapper
  with native PR annotations + a job-summary table.

### pre-commit

The format-aware upgrade from `check-added-large-files` — lint image assets before they land, via
[`.pre-commit-hooks.yaml`](.pre-commit-hooks.yaml):

```yaml
repos:
  - repo: https://github.com/jysf/crustyimg
    rev: v0.4.0            # a crustyimg release that ships `lint`
    hooks:
      - id: crustyimg-lint
```

### GitHub code-scanning (SARIF)

`--format sarif` emits [SARIF 2.1.0](https://sarifweb.azurewebsites.net/) — upload it to put
findings in the repo's **Security tab** (and inline in PRs) via GitHub code-scanning:

```yaml
      - uses: jysf/setup-crustyimg@v1
      - run: crustyimg lint assets --format sarif > crustyimg.sarif
        continue-on-error: true          # let the upload run even when findings fail the gate
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: crustyimg.sarif
```

### Locally / any CI

The binary + its exit code is the whole contract — no Action required:

```bash
crustyimg lint assets content            # 0 clean · 7 error finding · 2 usage · 3 no inputs
crustyimg lint assets --format json      # machine-readable report for tooling
crustyimg lint assets --format sarif     # SARIF 2.1.0 for GitHub code-scanning
just lint-images assets content          # the same, via the repo's justfile recipe
```

Tune rules with a `.crustyimg-lint.toml` (`select`/`ignore`, per-rule severity, per-glob byte
budgets, `per-file-ignores`); zero-config works out of the box.

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
