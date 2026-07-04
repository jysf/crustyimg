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

```sh
# View an image in the terminal
crustyimg view photo.jpg

# Inspect dimensions, format, byte size, EXIF presence
crustyimg info photo.jpg
crustyimg info photo.jpg --json      # machine-readable JSON to stdout

# Optimize for web: auto-orient + strip metadata + visually-lossless encode
crustyimg optimize photo.jpg -o out.webp

# Shrink: resize long edge to ≤1200 px, output as WebP
crustyimg shrink photo.jpg --max 1200 -o out.webp

# Resize a batch to ≤800 px into an output directory
crustyimg resize *.jpg --max 800 --out-dir web/

# Pipe: read from stdin, write to stdout (all diagnostics go to stderr)
crustyimg resize - --max 800 -o - < in.jpg > out.jpg

# Perceptual diff: SSIMULACRA2 score of b vs a; exit 7 when below threshold
crustyimg diff original.jpg compressed.jpg --fail-under 70

# Generate responsive image variants + paste-ready <picture>/srcset snippet
crustyimg responsive hero.jpg --widths 320,640,1280 --formats webp,jpeg --out-dir web/
```

Run `crustyimg --help` for the full command surface, or `crustyimg <cmd> --help`
for per-command options.

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
