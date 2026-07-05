# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

- Updated the `fast_image_resize` SIMD resize backend 5.5.0 → 6.0.0 and
  `indicatif` 0.18.4 → 0.18.6. Resize output and behavior are unchanged
  (PATCH-003).

### Deprecated

### Removed

### Fixed

### Security

---

## [0.2.0] - 2026-07-05

Dependency-hygiene release. The advisory-bearing dependencies behind the three
accepted `deny.toml` ignores were eliminated **at the source** (behavior-preserving
swaps), and the `--help` text was cleaned up for end users. `cargo deny` now carries
a single documented residual ignore, down from three.

### Changed

- `--help` text now reads for end users: internal stage/spec/decision references
  and stale "stub"/"placeholder" wording were removed from command and option
  descriptions (PATCH-002).
- Text-watermark glyph rasterization now uses `skrifa` + `zeno` (the Google
  `fontations` stack) instead of `ab_glyph`. Behavior-preserving — same rendered
  output, minus legacy `kern`-table kerning (a nil change for the bundled font)
  (SPEC-044).
- EXIF writing (`set`, `clean --gps`) now uses an in-house binary TIFF-IFD writer
  instead of `little_exif`. Behavior-preserving, and the parser is hardened against
  malformed/untrusted EXIF (bounds-checked, no panics) (SPEC-045).

### Removed

- Dropped the `ab_glyph`, `ttf-parser`, `little_exif`, `quick-xml`, and `brotli`
  dependencies from the tree.

### Security

- Eliminated three `deny.toml` advisory ignores at the source:
  **RUSTSEC-2026-0192** (`ttf-parser`, unmaintained) via the `skrifa`+`zeno` swap,
  and **RUSTSEC-2026-0194** / **-0195** (`quick-xml` memory-DoS) via the in-house
  EXIF writer. One documented ignore remains — **RUSTSEC-2024-0436** (`paste`, an
  unmaintained build-time proc-macro reached only via `rav1e`/`avif`; no upstream
  fix; revisit when `rav1e` drops `paste`).

---

## [0.1.1] - 2026-07-04

### Fixed

- `--out-dir` now creates the target directory (and parents) if missing,
  consistently across all batch commands; genuine creation failures return a
  clear error. Output-name path/symlink guards unchanged (DEC-035).

---

## [0.1.0] - 2026-07-03

This is the initial MVP release: a single static Rust binary that turns image
prep from *guess-a-quality-knob* to *declare-an-intent*. Zero system dependencies
by default; all formats handled in pure Rust.

### Added

**Inspect and view**
- `view <INPUT>` — display an image directly in the terminal via `viuer` (the
  `display` feature is on by default; headless builds omit it with
  `--no-default-features`).
- `info <INPUT> [--exif] [--json]` — print dimensions, format, file size on disk,
  color type, bit depth, alpha presence, and ICC/EXIF presence. `--exif` dumps EXIF
  tags; `--json` emits machine-readable output to stdout.

**Geometry / transform**
- `resize <INPUT...>` — resize with a SIMD backend in six modes: `--max` (long-edge
  bound, never upscales), `--exact WxH`, `--percent P`, `--fit WxH` (letterbox,
  never upscales), `--cover WxH` (fill box, may upscale), `--fill WxH`
  (cover + center-crop to exact dimensions). Batch-ready with `--out-dir`.
- `thumbnail <INPUT...> [--size N] [--square]` — convenience resize: bounds the
  longest edge to N (default 256), or produces an exact N×N square via cover +
  center-crop.
- `shrink <INPUT...>` — optimize for web: resize to a long-edge bound (default 1600),
  re-encode at quality 80, drop metadata. Accepts `--target
  visually-lossless|high|medium` or `--ssim <0-100>` for perceptual auto-quality
  (binary-searches SSIMULACRA2; see below), or `--max-size <KB/MB>` for a byte
  budget with automatic dimension-reduction fallback.
- `convert <INPUT...> --format FMT` — pure re-encode to another format with no pixel
  transform. Supports all core formats plus WebP (default build) and AVIF (opt-in
  feature). `--max-size <SIZE>` fits the output under a byte budget for every format.
- `auto-orient <INPUT...>` — bake the EXIF orientation into pixels and clear the tag,
  fixing the common silent-rotation bug. A no-op when no orientation tag is present.

**Optimize, diff, and responsive web delivery (STAGE-009)**
- `optimize <INPUT...>` — one-command web prep: auto-orient + strip metadata +
  perceptual visually-lossless re-encode in a single pass. The "just make this
  web-good" default.
- `diff <A> <B> [--fail-under N] [--json]` — compute an SSIMULACRA2 perceptual
  similarity score between two images. `--fail-under N` turns it into a CI
  visual-regression gate: a score below N exits with code 7 (distinct from a
  runtime error), so CI can tell "regression detected" from "couldn't run".
- `responsive <INPUT> --widths W1,W2,… --out-dir DIR [--formats …]` — generate a
  width × format responsive image set and print a paste-ready `<picture>`/srcset
  HTML snippet to stdout.

**Perceptual auto-quality and byte budgets (STAGE-008)**
- SSIMULACRA2-driven quality search: binary-searches the encoder quality against a
  perceptual target — the smallest file that still clears a visual quality level.
  Available via `--target visually-lossless|high|medium` or `--ssim <score>` on
  `shrink` and `optimize`.
- Byte-budget mode (`--max-size <SIZE>`) on `shrink` and `convert`: lowers quality
  first, then progressively downscales dimensions as a fallback when quality alone
  cannot meet the budget. Works for every output format, including lossless ones.
- Modern formats: **WebP** is the pure-Rust default (lossless; lossy via opt-in
  `webp-lossy` feature); **AVIF** is a pure-Rust opt-in feature (`--features avif`,
  via `ravif`). Both are available as output targets for all quality/budget modes.

**Compositing and text overlays (STAGE-004 / SPEC-029-030)**
- `watermark <INPUT...> --image LOGO [--gravity G] [--opacity O] [--scale S]
  [--margin M] [--tile]` — overlay an image watermark at a compass gravity anchor
  (default `southeast`); supports tiling, opacity, and proportional scaling.
- `watermark <INPUT...> --text STRING [--font PATH] [--size N] [--color HEX]` —
  rasterize text (via `ab_glyph`) and composite it as an overlay. Default font is
  the bundled BSD-3 Go font.

**Metadata lane — container-level ops, no pixel re-encode (STAGE-004)**

All four commands operate on the image container directly — pixels are never
re-decoded, so privacy ops carry no quality cost and no recompression.

- `strip <INPUT...>` — remove all container metadata (EXIF/IPTC/XMP/ICC). Supports
  JPEG and PNG.
- `clean <INPUT...> --gps` — selectively remove GPS/location tags while preserving
  all other metadata (orientation, copyright, ICC). Supports JPEG and PNG.
- `set <INPUT...> [--artist S] [--copyright S] [--description S]` — write named
  EXIF tags, creating a fresh EXIF block when the input has none.
- `copy-metadata --from SRC --to DST` — copy EXIF + ICC from one image onto another
  without touching pixels or XMP. JPEG only in v1.
- Default drop-GPS policy on all pixel-lane encodes (`--keep-gps` to opt out).

**Recipes and parallel batch (STAGE-005)**
- `edit <INPUT> [--auto-orient] [--resize-max N] [--invert] [--save-recipe FILE]` —
  chain an ordered op list on a single image in one decode→ops→encode pass. Ops
  apply in a fixed canonical order regardless of flag order, so the result is
  deterministic. `--save-recipe FILE` serializes the chain to a TOML recipe.
- `apply --recipe FILE <INPUT...> [--out-dir DIR] [-j N]` — replay a saved recipe
  across a file, glob, or directory in parallel (rayon, `-j N` bounds workers) with
  an `indicatif` progress bar on stderr. The recipe that tuned one image runs
  unchanged across thousands. Per-input failures are summarized and exit with
  code 6; other inputs still write.
- Recipe round-trip is byte-pinned: `edit` output equals `apply`-of-the-saved-recipe
  output on the same input.

**Global options and exit-code contract**
- Global flags across all subcommands: `--output / -o`, `--out-dir`, `--format`,
  `--quality / -q`, `--verbose / -v`, `--quiet / -Q`, `--yes / -y`, `--keep-gps`,
  `--jobs / -j`.
- Stdin/stdout piping: `-` as input or output keeps diagnostic output on stderr so
  pipes stay clean.
- Typed exit codes: 0 success, 1 runtime error, 2 usage error, 3 input not found,
  4 unsupported format/codec not built, 5 output write refused, 6 partial batch
  failure, 7 check/gate not satisfied.
- `completions <bash|zsh|fish|powershell|elvish>` — print a clap-generated shell
  completion script to stdout (e.g. `crustyimg completions zsh > _crustyimg`).

### Security

- **Decode resource limits** — every image load is bounded via `image::Limits`:
  per-dimension ≤ 65 535 px, decoded allocation ≤ 512 MiB. Decompression bombs and
  forged-dimension inputs are rejected with a typed error (exit 1) before pixels are
  produced; never a panic or OOM.
- **Recipe resource limits** — untrusted recipe files over 64 KiB or with more than
  1024 steps are rejected before being read into memory.
- **Resize output cap** — a resize whose output buffer would exceed 512 MiB (upscale
  bomb via exact/percent/cover/fill, from CLI or recipe) is rejected before
  allocation.
- **Path and symlink guards** — `..`, separator characters, and absolute paths in
  output names are rejected; symlinked destinations are refused even with `--yes`.
- **Supply-chain CI** — `cargo deny check` (advisories, bans, sources, licenses)
  runs in CI on every push.
- A recorded threat model (`SECURITY.md`) maps each untrusted-input surface to its
  mitigation and the spec/decision that built it; an adversarial review over the
  cumulative diff found no unresolved finding.

---

[Unreleased]: https://github.com/jysf/crustyimg/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jysf/crustyimg/releases/tag/v0.2.0
[0.1.1]: https://github.com/jysf/crustyimg/releases/tag/v0.1.1
[0.1.0]: https://github.com/jysf/crustyimg/releases/tag/v0.1.0
