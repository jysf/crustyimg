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
- **Security:** passes a hardening/security assessment before ship — decode
  limits set, output path-traversal safe, `cargo audit` clean, recipes
  validated (constraint `untrusted-input-hardening`; STAGE-006).
- **Ergonomics:** common single-image tasks are one short command with
  sensible defaults (e.g. `crustyimg resize photo.jpg --max 800`); added
  power comes from recipes + batch, never from burdening the simple case
  (constraint `ergonomic-defaults`).

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
- STAGE-006 — Hardening & security assessment: decode limits,
  path/symlink-traversal tests, `cargo audit`/`deny` in CI, recipe validation,
  a threat-model verification pass against SECURITY.md.
- STAGE-007 — Release & distribution (MVP exit gate): Cargo.toml publish
  metadata + license, CHANGELOG/semver/tags, a cargo-dist release pipeline
  (cross-platform binaries → GitHub Releases), a Homebrew tap, optional
  crates.io publish, and README/install/usage polish.
- Core formats now (JPEG/PNG/GIF/BMP/TIFF/ICO/etc.).

### Explicitly out of scope
- WebP output (fast-follow), AVIF (feature-gated, later).
- Effects/filters catalog (grayscale, sepia, solarize, pixelize, edges) —
  the `Operation` trait makes these cheap to add later; lead fast-follow.
- Full color/tone suite (levels/curves), montage/contact-sheet, `compare`/SSIM.
- `open` in external app (Preview/browser).
- Interactive TUI editor.
- ICC color conversion, placeholder fetch from public APIs.
- **Geometry extras — `crop` (rect / gravity / center / aspect), `rotate`,
  `flip`/`flop`, `trim`, `pad`** — explicitly on the roadmap but deferred to
  a near-term follow-up wave (see "Enables"). `crop` is the lead item.
  Already cataloged in `docs/feature-exploration.md` (Geometry); each is just
  another `Operation`, so they drop in without architectural change.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-001 (shipped on 2026-06-14) — Foundation: CI, image model, Operation trait, pipeline, recipe + source/sink (7 specs, PRs #1-#7)
- [x] STAGE-002 (shipped on 2026-06-15) — View & info (read-only): `view`, `info` (+ `--exif`, `--json`) (2 specs, PRs #8-#9; DEC-013)
- [x] STAGE-003 (shipped on 2026-06-15) — Transform & output: `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient` (6 specs SPEC-010–015, PRs #11–#16; DEC-014/015/016/017)
- [~] STAGE-008 (active) — **Modern formats & quality** (the differentiator wave): perceptual auto-quality (`shrink --target`/`--ssim` via SSIMULACRA2), `--max-size` byte budget, AVIF (feature-gated), WebP. *Re-prioritized 2026-06-16 to run here, right after STAGE-003 (Option B). Numeric id ≠ execution order — see the stage file's sequencing note.*
- [ ] STAGE-004 (pending) — Compose & metadata: `watermark`; `strip`/`clean --gps`/`set`/`copy-metadata`
- [ ] STAGE-005 (pending) — Batch & recipes: `edit`, `--save-recipe`, `apply` (parallel + progress)
- [ ] STAGE-006 (pending) — Hardening & security assessment: decode limits, traversal tests, cargo-audit in CI, recipe validation, threat-model pass
- [ ] STAGE-007 (pending) — Release & distribution (MVP exit gate): Cargo metadata + CHANGELOG/tags, cargo-dist release pipeline, Homebrew tap, optional crates.io, README/install polish

**Count:** 3 shipped / 1 active / 4 pending

> **Roadmap re-prioritization (2026-06-16).** STAGE-008 (Modern formats &
> quality) was inserted ahead of the originally-planned STAGE-004–007 as the
> differentiator core ("set the look, not the number" + modern formats), per the
> 2026-06-16 decision handoff (Option B, user-chosen). It carries the next free
> numeric id but executes in the position shown above; STAGE-004–007 keep their
> ids and slide later. **Drive order by stage status/priority, not by number.**

## Dependencies

### Depends on
- External: Rust toolchain + crates (image, imageproc, viuer, clap, kamadak-exif,
  serde/toml, rayon, indicatif, fast_image_resize, img-parts/little_exif — to be
  pinned during design). No third-party services.
- Previous projects: none (first project).
- Reference only: the original prototype in sibling `crustimg/` (not built upon).

### Enables
- Near-term follow-up wave — **geometry extras** (`crop` first, then
  `rotate`, `flip`/`flop`, `trim`, `pad`): high-value, low-complexity
  `Operation`s that slot straight into the pipeline and recipes.
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
