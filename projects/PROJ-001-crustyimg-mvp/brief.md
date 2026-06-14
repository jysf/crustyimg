---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-001                      # stable, zero-padded, never reused
  status: active                    # proposed | active | shipped | cancelled
  priority: high                    # critical | high | medium | low
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg                     # must match .repo-context.yaml

created_at: 2026-06-13
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    A single image model fronted by a pluggable Operation pipeline lets a
    user tune an edit once on one image and replay the exact same recipe
    across many images — making routine image prep (especially for the web)
    faster and more repeatable than clicking through a GUI or memorizing
    ImageMagick incantations.
  beneficiaries:
    - Web/content developers preparing images for sites and blogs
    - Power users who script and live in the terminal
    - The maintainer — a clean, trait-based base that is cheap to extend
  success_signals:
    - view / info / resize / shrink / thumbnail / strip work on real images
    - a recipe tuned on one image runs unchanged across a whole directory in one command
    - watermark (image) and metadata edit/clean (incl. drop-GPS) work end to end
    - multi-OS CI (Linux/macOS/Windows) is green
    - the binary installs cleanly from a release artifact (brew / crates.io)
  risks_to_thesis:
    - The recipe abstraction may add complexity users don't want over simple one-shot flags
    - The metadata dual-lane (preserve/edit vs strip) may be harder than estimated and slip
    - Pure-Rust JPEG encode may not match mozjpeg on size/quality, weakening "better for web"
    - Terminal display limits the "experiment like an editor" feel without a TUI (out of scope here)
---

# PROJ-001: crustyimg MVP — clean rebuild

## What This Project Is

The first wave of work: rebuild `crustyimg` from scratch as a fast,
scriptable image CLI. It carries forward the useful feature set of an
earlier prototype (terminal view, resize, shrink, thumbnail, EXIF) but on
a fundamentally better architecture — one canonical image model, a
pluggable `Operation` pipeline, and a recipe mechanism that lets the same
sequence of operations run on a single image or a whole batch. Tests and
multi-OS CI are present from the first spec.

## Why Now

An earlier gemini-built prototype (in the sibling `crustimg/` folder)
proved the feature set is useful but accreted into ~1,000 lines of
overlapping boolean flags, two competing image libraries used
inconsistently, hardcoded output filenames, dead non-compiling modules,
and zero tests. Rather than refactor that, we restart clean on a
spec-driven process so the result is something shippable and extensible —
the foundation for a "wide and interesting feature set" over later waves.

## Success Criteria

- A user can run `view`, `info`, `resize`, `shrink`, `thumbnail`, and
  `strip` on real images, each backed by tests.
- The same recipe (e.g. "resize to 1200px + sharpen + watermark + strip
  GPS") runs unchanged on one image and on a directory of images, in one
  command, in parallel, with a progress indicator.
- `watermark` (image overlay) works; EXIF can be read, stripped, GPS
  selectively cleaned, and basic tags set.
- Multi-OS CI is green; `cargo test`, `clippy`, and `fmt --check` pass.
- The binary is installable from a release artifact.

## Scope

### In scope
- STAGE-001 — Foundation: project, CI, single image model, `Operation`
  trait, pipeline, recipe (de)serialization, source/sink abstractions.
- STAGE-002 — View & info (read-only): `view`, `info` (+ `--exif`).
- STAGE-003 — Transform & output: `resize`, `shrink`/optimize,
  `thumbnail`, `convert`, `auto-orient`.
- STAGE-004 — Compose & metadata: `watermark` (image; text trailing) and
  the metadata lane (`strip`, `clean --gps`, `set` tags, `copy-metadata`).
- STAGE-005 — Batch & recipes: `edit` (one-shot multi-op), `--save-recipe`,
  `apply --recipe <glob/dir>` with rayon parallelism + progress.
- Core formats now (JPEG/PNG/GIF/BMP/TIFF/ICO/etc.).

### Explicitly out of scope
- WebP output (fast-follow), AVIF (feature-gated, later).
- Effects/filters catalog (grayscale, sepia, solarize, pixelize, edges) —
  the `Operation` trait makes these cheap to add later; lead fast-follow.
- Full color/tone suite (levels/curves), montage/contact-sheet, `compare`/SSIM.
- `open` in external app (Preview/browser).
- Interactive TUI editor.
- ICC color conversion, placeholder fetch from public APIs.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-001 (active) — Foundation: CI, image model, Operation trait, pipeline, recipe + source/sink
- [ ] STAGE-002 (pending) — View & info (read-only): `view`, `info` (+ `--exif`)
- [ ] STAGE-003 (pending) — Transform & output: `resize`, `shrink`, `thumbnail`, `convert`, `auto-orient`
- [ ] STAGE-004 (pending) — Compose & metadata: `watermark`; `strip`/`clean --gps`/`set`/`copy-metadata`
- [ ] STAGE-005 (pending) — Batch & recipes: `edit`, `--save-recipe`, `apply` (parallel + progress)

**Count:** 0 shipped / 1 active / 4 pending

## Dependencies

### Depends on
- External: Rust toolchain + crates (image, imageproc, viuer, clap, kamadak-exif,
  serde/toml, rayon, indicatif, fast_image_resize, img-parts/little_exif — to be
  pinned during design). No third-party services.
- Previous projects: none (first project).
- Reference only: the original prototype in sibling `crustimg/` (not built upon).

### Enables
- Future projects: effects/filters catalog, WebP/AVIF, `open`/external,
  `compare`/SSIM, color/tone suite, and a recipe-driven TUI editor — all
  additive on the `Operation` + recipe architecture this project lays down.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
