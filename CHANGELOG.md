# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

---

## [0.6.0] - 2026-07-24

A small release with one behavior change: AVIF now works out of the box. Also
adds published benchmarks and the WebAssembly library on npm.

### Added

- **`crustyimg-wasm` on npm** — the engine compiled to WebAssembly, installable
  with `npm install crustyimg-wasm`. It runs client-side in the browser or in
  Node: no native addon, no postinstall build step, and no dependencies. Same
  engine as the CLI, so a recipe behaves the same in both.
- **`BENCHMARKS.md`** — an equal-quality comparison against sharp, ImageMagick,
  `@squoosh/cli`, and cwebp over real photographs, with the machine, the pinned
  tool versions, and the exact commands stated so you can check it. Every tool's
  output is scored with the same perceptual metric rather than compared at
  whatever quality it happened to produce. crustyimg is neither the smallest nor
  the fastest, and the document says where it loses and why. Re-run it on your
  own images with `just bench-compare --corpus /path/to/photos`.

### Changed

- **AVIF encode is now in the default build.** Every distributed binary
  (Homebrew, the Releases-page downloads, the shell/PowerShell installers,
  and a plain `cargo install crustyimg`) now includes the AVIF encoder with
  no extra flag — previously it needed a `--features avif` build from
  source. This is a **behavior change**: `web` and `optimize` can now pick
  AVIF for lossy-family photos where they couldn't before, so upgrading may
  change the output file for the same command and input. A
  `--no-default-features` (lean) build still leaves AVIF out.

### Fixed

- The README quoted a median saving of 98%; re-measuring the same corpus with
  the same command gives 97%. Corrected.

---

## [0.5.0] - 2026-07-20

A large release: a faster, smarter default engine and the `web` flagship command,
broad new input-format support, a reproducible incremental build system, a
WebAssembly build with a client-side browser demo, and a frozen CLI surface.

**This release has breaking CLI changes.** While the version is `0.x`, a minor bump
may rename or remove commands — see Removed and Changed before upgrading.

### Added

- **`web`** — make an image web-ready in one command: downscale, re-encode to the
  smallest modern format that beats the downscaled image (AVIF for photos, lossless
  WebP or PNG for graphics), strip metadata, auto-orient, and report an SSIMULACRA2
  quality score. Size-insensitive — a 24 MP photo is about as fast as a small one.
- **Declarative, reusable recipes** — `apply --recipe` with bundled `web`, `gallery`,
  and `product` recipes; tune settings on one image, save the recipe, and replay it
  across a batch. The same recipe file runs in the browser via the WebAssembly build.
- **New input formats, pure Rust and on by default** — AVIF decode, SVG rasterize,
  and RAW embedded-preview extraction (reads `.DNG`/`.CR2`/etc. by pulling the
  embedded JPEG preview, not a full RAW develop). HEIC decode is available behind an
  off-by-default `heic` feature (system libheif, local builds only).
- **`crustyimg build`** — a declared, incremental build from a `crustyimg.build.toml`
  manifest, with a content-addressed cache and `--watch`. A no-change re-run is a full
  cache hit that skips every decode/encode; `--no-cache` bypasses it.
- **`optimize --verify`** — opt in to computing the SSIMULACRA2 score for a run
  (added to the JSON report as `ssim`).
- **Machine-readable output** — `--json` and `--timing` on `optimize`/`web`/`apply`,
  `lint --format json`, and a committed offline benchmark (`just bench`, `--corpus`).
- **WebAssembly build** — the pure-Rust engine compiles to wasm. A client-side demo
  (drop an image, convert to AVIF, read the score — entirely in your browser, nothing
  uploaded) runs at https://jysf.github.io/crustyimg/.
- **CI integration** — two GitHub Actions wrap the binary: `jysf/setup-crustyimg`
  (install in CI) and `jysf/crustyimg-action` (lint or optimize with inline PR
  annotations).

### Changed

- **The default optimization is now fast and AVIF-aware.** The default decision does a
  single fixed-quality encode to the smallest modern format that beats the source,
  with a first-class "kept, already optimal" passthrough — instead of the slower
  quality search that ran by default before.
- **`optimize` is now a keep-dimensions byte primitive** — best format at good quality,
  never larger than the source. The perceptual and byte-budget searches are opt-in via
  `--target` / `--ssim` / `--max-size`.
- **Metadata commands moved into a `meta` group** — `strip` → `meta strip`, `clean` →
  `meta clean`, `copy-metadata` → `meta copy`, `set` → `meta set`. (`auto-orient` stays
  top-level.)
- **Dependency requirements use caret ranges instead of exact pins**, so crustyimg is
  friendlier to depend on as a library; reproducible builds come from the committed
  `Cargo.lock` (use `cargo install --locked`).

### Removed

- **`shrink`** — its behavior is folded into `web` (and upgraded with AVIF).

### Fixed

- The metadata write path corrupted numeric EXIF tags (orientation, GPS) on big-endian
  input; the writer now preserves the input's byte order.
- AVIF decode: a data race under load and an empty-stream abort, both fixed.
- `web`'s "never larger than the source" guarantee is now reconciled with the
  downscaled baseline.

### Security

- Peak decode memory is bounded before allocation by a declared-pixel budget, and a
  hardening pass tightened decode limits and error handling.

---

## [0.4.0] - 2026-07-06

Image **linting** (PROJ-004). `crustyimg lint` is a format-aware, no-URL,
per-file CI linter for an image asset tree ("clippy for image assets"): it flags
privacy / format / size / colorspace problems, names a runnable `crustyimg` fix
for each, and exits `7` on an error finding. Plus a GitHub Actions on-ramp. Zero
new default dependencies; the default build stays pure-Rust / zero-system-deps.

### Added

- **`crustyimg lint [PATHS]…`** — a read-only, advisory image-asset linter:
  source-resolution fan-out (globs / dirs / files, non-images skipped), a
  `Rule` / `Finding` / `Severity` framework, grouped-by-file output, and
  CI-native exit codes (`0` clean · `7` error finding · `2` usage · `3` no
  inputs) reusing the exit-7 `CheckFailed` gate.
- **Rule catalog** (each names a runnable fix): `privacy/gps-metadata-leak`,
  `privacy/camera-metadata`, `orient/orientation-not-baked`,
  `size/oversized-bytes`, `size/truncated-or-corrupt`,
  `dims/oversized-dimensions`, `color/wrong-colorspace`, `color/missing-icc`,
  `color/unexpected-icc`, `format/animated-gif`.
- **`.crustyimg-lint.toml` config** (auto-discovered): ruff-style
  `select` / `ignore` + `per-file-ignores`, eslint-style per-rule severity,
  per-glob `[[budget]]`, and a savings threshold; plus the CLI flags
  `--config` / `--no-config` / `--select` / `--ignore` / `--max-warnings` /
  `--max-intended-width` / `--savings-threshold`.
- **`lint --format json|sarif`** — hand-rolled reports, no new dependency: a
  stable `crustyimg.lint/v1` JSON report and SARIF 2.1.0 for GitHub
  code-scanning (`github/codeql-action/upload-sarif`).
- **CI on-ramp:** the [`setup-crustyimg`](https://github.com/jysf/setup-crustyimg)
  and [`crustyimg-action`](https://github.com/jysf/crustyimg-action) GitHub
  Actions (their own repos), a `.pre-commit-hooks.yaml` hook, and a
  `just lint-images` recipe — drop image linting into any CI in three lines.

---

## [0.3.1] - 2026-07-06

Dependency-hygiene patch: no user-facing behavior change.

### Security

- Bumped `crossbeam-epoch` 0.9.18 → 0.9.20 to clear **RUSTSEC-2026-0204** — an
  invalid pointer dereference in its `fmt::Display` impl for `Atomic`/`Shared`
  on a null pointer. It reaches crustyimg only as a deep transitive of
  `rayon`/`ravif` and is never `Display`-formatted, so real exposure was nil;
  this restores a green `cargo deny` supply-chain gate.

---

## [0.3.0] - 2026-07-06

The optimization engine (PROJ-002). `optimize` now looks at the image and picks
the best output format for you — the "local `f_auto`" — and explains why. Built
on a new shared image-analysis layer. Zero new default dependencies; the default
build stays pure-Rust / zero-system-deps.

### Added

- **`optimize` auto-decides the output format** (the "local `f_auto`"). With no
  `--format`, it analyzes the image, shortlists up to three candidate formats,
  drives the existing SSIMULACRA2 perceptual search (or the `--max-size` byte
  budget) across them, and ships the smallest artifact that beats the source —
  never a larger file (SPEC-048).
- **`optimize --profile <web|docs|preserve>`** selects the bias: `web` (default)
  auto-picks the format; `docs` widens the lossless/crisp-text bias; `preserve`
  keeps the input's format (the previous behavior) (SPEC-048).
- **`optimize --explain` / `--explain=json`** print an auditable trace of the
  decision — detected features, class, every candidate tried
  (format/quality/bytes/met-target), the winner, and the savings — human-readable
  to stderr or JSON to stdout (schema `crustyimg.optimize.explain/v1`) (SPEC-049).
- A new internal **image-analysis layer** (`src/analysis/`): a computed-once
  `Analysis` context (histogram, entropy, edge density, alpha coverage, capped
  unique-color count, dominant color) plus deterministic, no-ML classification
  (photograph / graphic-logo / icon / document / ui-screenshot) that biases the
  format decision (SPEC-046, SPEC-047).

### Changed

- **`optimize`'s default now auto-decides the output format** instead of
  preserving the input format — e.g. a photographic PNG may be shipped as a
  smaller JPEG or WebP. Pass `--profile preserve` (or pin `--format` / `-o
  <ext>`) for the previous format-preserving behavior. The chosen format and
  savings are always reported on stderr (silence with `--quiet`). **Breaking**
  for scripts that relied on `optimize` keeping the input format.

### Notes

- AVIF appears as an auto-decision candidate only in `--max-size` (byte-budget)
  mode and only when built with `--features avif` — it has no decoder, so it
  cannot be perceptually scored (DEC-020).
- Indexed/lossy-PNG output is still deferred (it needs a permissive quantizer);
  few-color graphics use lossless WebP in the interim.

---

## [0.2.1] - 2026-07-05

Maintenance release: dependency currency + a scheduled advisory audit. No
user-facing behavior change.

### Changed

- Updated the `fast_image_resize` SIMD resize backend 5.5.0 → 6.0.0 and
  `indicatif` 0.18.4 → 0.18.6. Resize output and behavior are unchanged
  (PATCH-003).

### Security

- Added a weekly scheduled `cargo-deny` advisory audit
  (`.github/workflows/scheduled-audit.yml`) so newly-published RustSec
  advisories against existing dependencies are caught between commits, not just
  on push (PATCH-003).

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

[Unreleased]: https://github.com/jysf/crustyimg/compare/v0.5.0...HEAD
[0.6.0]: https://github.com/jysf/crustyimg/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/jysf/crustyimg/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/jysf/crustyimg/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/jysf/crustyimg/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/jysf/crustyimg/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/jysf/crustyimg/releases/tag/v0.2.1
[0.2.0]: https://github.com/jysf/crustyimg/releases/tag/v0.2.0
[0.1.1]: https://github.com/jysf/crustyimg/releases/tag/v0.1.1
[0.1.0]: https://github.com/jysf/crustyimg/releases/tag/v0.1.0
