# PROJ-002 design brief — `crustyimg lint` ("clippy for image assets")

> Source-file-level, no-URL, deterministic, pass/fail linter for an image asset tree.
> Read-only + advisory (never writes pixels; every finding names a crustyimg fix command).
> Design-only, 2026-07-05. Target: STAGE-012 (0.4.0). **No new crates for v1.**

## Positioning (load-bearing — don't drift)
Lighthouse's four image audits (`uses-webp-images`, `uses-optimized-images`,
`uses-responsive-images`, `offscreen-images`) and Lighthouse-CI budgets **all run in-browser
against a deployed page** — they need a URL and a layout (responsive compares *rendered* size,
DPR-aware; offscreen is a viewport/lazy-load check meaningless for a source file). Generic
`--maxkb` git-gates (`check-added-large-files`) are the only pre-deploy option and are
**format-blind**. crustyimg's white space = **pre-deploy, no-URL, format-aware, per-file,
pass/fail** — the linter you run on `assets/`/`content/` in CI *before* anything is
Lighthouse-able. Don't compete on Lighthouse's "next-gen formats" framing; own the source tree.

## Rule catalog (each maps 1:1 to a shipped capability — lint is composition over `Analysis`)
Capability sources: **info** (`run_info`/`InfoReport`: dims, file_size, format, color_type,
bit_depth, has_alpha, has_icc, has_exif) · **exif-read** (kamadak-exif, Orientation+GPS) ·
**format-rec** (the Analyzer/format engine) · **quality-search** (SSIMULACRA2 `score`) ·
**metadata-lane** (strip/clean-gps — the *fix* engine).

| Rule | Detects | Powered by | Severity | Fix | Default? |
|---|---|---|---|---|---|
| `format/legacy-format` | photo would encode smaller at **equal SSIMULACRA2** in a modern format (proves ≥ savings-threshold via a real probe) | format-rec + quality | warn | `optimize --format <fmt>` | ✓ |
| `size/oversized-bytes` | exceeds per-format/per-glob byte budget (format-*aware* `--maxkb`) | info | **error** | `optimize`/`shrink --max-size` | ✓ |
| `dims/oversized-dimensions` | natural dims exceed a **declared** intended width + slack (source-file analogue of "properly size", no page) | info | warn | `resize --max <W>` | opt-in |
| `quality/excessive-jpeg-quality` | re-encode at VL target scores ≥ anchor while saving ≥ threshold | quality | warn | `optimize` | ✓ |
| `format/indexed-png-opportunity` | RGB(A) PNG with few colors → palette PNG, no perceptual loss (Lighthouse never checks) | format-rec | info | `optimize` | ✓ |
| `format/non-progressive-jpeg` | baseline JPEG above a size floor | info+scan | info | `optimize` | opt-in |
| `color/missing-icc`\|`unexpected-icc` | wide image missing ICC (renders wrong) / bulky ICC to strip for web | info(has_icc) | info | tag sRGB / `strip` | opt-in |
| `orient/orientation-not-baked` | EXIF Orientation≠1 (pixels not upright; some pipelines ignore the tag) | exif-read | warn | `auto-orient` | ✓ |
| **`privacy/gps-metadata-leak`** | EXIF GPS present — location leak in a public asset (the privacy moat) | exif-read | **error** | `clean --gps` | ✓ |
| `privacy/camera-metadata` | non-GPS identifying EXIF (Make/Model/Serial/DateTime) | exif-read | info | `strip` | opt-in |
| `color/wrong-colorspace` | CMYK/non-sRGB JPEG or needless 16-bit PNG for web | info | warn | `convert --format` | ✓ |
| `format/animated-gif` | animated GIF that should be WebP/video | info+frame-sniff | warn | `convert --format webp` | ✓ |
| `size/truncated-or-corrupt` | decode fails/truncated (free — decode already runs) | info(decode) | **error** | (re-export) | ✓ |
| `format/alpha-in-jpeg-source` | needs alpha but is JPEG (fringing) | info | info | `convert --format png\|webp` | opt-in |
| `dupe/near-duplicate` | perceptual near-dup committed twice | image_hasher (**v2**, new dep) | info | (dedupe) | opt-in v2 |

**Savings-threshold gate (signal-to-noise, critical):** "could be smaller" rules only fire past
a configurable floor — default `{min_bytes=4096, min_percent=10}`, borrowing Lighthouse's own
4 KiB floor. This is what makes `lint` quiet enough to leave on in CI.

**Severity model (clippy/eslint/ruff-aligned):** `error` = wrong/leaking (GPS, hard budget
breach, corrupt) → fails CI · `warn` = should-fix measured savings/correctness risk · `info` =
opt-in polish. Only `error` (and `warn` under `--max-warnings`) affects exit code.

## Lighthouse parity + differentiation
Parity: `uses-webp-images`→`format/legacy-format` (per file, no page); `uses-optimized-images`
(q85, ≥4KiB)→`quality/excessive-jpeg-quality` (perceptual target, adopt the 4KiB floor);
`uses-responsive-images`→`dims/oversized-dimensions` (**partial** — substitute a *declared*
intended width, honest opt-in); `offscreen-images`→**no equivalent** (runtime/viewport, no
source meaning — explicitly not implemented). **Differentiation (Lighthouse structurally can't
on source files):** GPS/camera-metadata leak (stripped by CDN before browser), orientation-not-
baked (browser applies at paint), ICC/colorspace (no Lighthouse audit), indexed-PNG strategy,
per-file legacy-format with no URL, corrupt-source.

## Config + CLI + exit
`crustyimg lint [PATHS]... --config --format human|json --select --ignore --max-warnings
--max-intended-width --savings-threshold --no-config --explain <rule>`. Reuses `source::resolve`
globs/dirs; non-images skipped; read-only (no `-o`). Config `.crustyimg-lint.toml` auto-discovered
walking up to repo root (ruff `select`/`ignore` prefixes + `per-file-ignores`, eslint per-rule
severity, per-glob `[[lint.budget]]` with `max_bytes`+`max_intended_width`). **Zero-config works**
(default rules at default severities). **Exit reuses the existing exit-7 `CheckFailed`** (DEC-025,
src/cli/mod.rs:428 — its comment already anticipates "reusable by the future EXIF audit-linter"):
0 clean · **7** ≥1 error or warnings>`--max-warnings` (info never fails) · 2 usage/bad config ·
3 no inputs. **Decode failure is a *finding* (`size/truncated-or-corrupt`→7), not exit 1** — the
one deliberate divergence; a linter reports a broken asset, doesn't abort.

CI fit: one binary + an exit code (the deliberate contrast with Lighthouse CI's server/URL). Ship
a GitHub Action snippet + a `pre-commit` local hook (the format-aware upgrade from
`check-added-large-files`) + a `just lint-images` recipe.

## Output
Human: grouped-by-file (eslint/ruff style), findings sorted by severity, **fix line is always a
runnable crustyimg command**, summary with total potential savings. `--json`: **hand-rolled**
(matches `write_json`/`write_diff_json`, no serde_json runtime dep) — `{schema, findings[{file,
rule, severity, message, fix, bytes_saved}], summary{files_scanned, errors, warnings, infos,
potential_bytes_saved, passed}}`. Switch to `serde::Serialize` only if serde is promoted for the
manifest (don't block on it).

## Build notes / DEC
No new crates v1 (dupe/near-dup = image_hasher, v2) · `legacy-format`/`indexed-png` depend on the
**format engine/Analyzer landing first**; privacy/orientation/size/colorspace/corrupt depend only
on shipped capabilities → can ship in a first `lint` cycle · determinism guard (no network/mtime/
wall-clock) · **license guard: detect compiled-in encoders at runtime — don't suggest `--format
avif` in a build without AVIF** (degrade to WebP or drop) · new `DEC-lint-command-and-rule-catalog`
(pin rule ids as a stability surface, 3-severity model, exit-7 reuse, 4096B/10% default,
config-discovery order).

Prior art: Lighthouse audits + LHCI budgets (the URL-bound thing we don't replicate),
`check-added-large-files` (the format-blind gate we replace), rimage/squoosh/ImageOptim-CLI
(optimizers, not linters — the gap), clippy/eslint/ruff (config+severity+exit conventions).
