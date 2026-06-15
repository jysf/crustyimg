# crustyimg: a recipe-driven image CLI, rebuilt clean

*2026-06-14 · project overview & roadmap*

## The idea

`crustyimg` is a fast, scriptable command-line tool for viewing and
transforming images — the everyday stuff: resize, shrink-for-web,
thumbnail, convert, strip metadata, watermark, inspect. Its one defining
idea is a **load-once pipeline**:

```
Source (file | glob | dir | stdin)
  → load the image once
  → apply an ordered list of Operations in memory
  → write to a Sink (file | dir+template | stdout | terminal)
```

Because that operation list is **serializable — a recipe (TOML)** — the
same edit you tune on one image replays unchanged across a thousand. Tune
once, replay across many. Simple things stay one short command
(`crustyimg resize photo.jpg --max 800`); complex, repeatable workflows
become a recipe you run over a whole directory.

The name is a nod to Rust's mascot, [Ferris the crab / "rustacean"](https://rustacean.net/) —
a crusty crab for a tool written in Rust.

## Why a rebuild

There was an earlier prototype. It proved the features were useful but had
accreted into ~1,000 lines of overlapping boolean flags, **two competing
image libraries**, hardcoded output paths, dead modules, and zero tests.
Rather than refactor, we restarted clean — single image model, a pluggable
`Operation` trait, tests and multi-OS CI from the very first commit.

## What's built (STAGE-001 — the foundation)

The foundation stage shipped as **7 specs (PRs #1–#7)**:

- a compiling Rust lib+binary with **green CI on Linux/macOS/Windows**;
- a canonical `Image` model (one image library, metadata captured at load);
- a **decode-once `Operation` pipeline** (no per-op disk round-trips);
- `Source` (file/glob/dir/stdin) and `Sink` (file/dir-template/stdout/display),
  both with **path-traversal hardening built in**;
- **TOML recipes + an operation registry** that round-trip;
- a **clap subcommand CLI** that drives the whole chain — `apply --recipe`
  already runs end-to-end.

~97 tests, clippy/fmt enforced, 11 architecture decisions recorded.

## How it's being built

Spec-driven, with a multi-agent loop: an orchestrator plays architect,
and each cycle — design → build → verify → ship — runs as a separate fresh
agent. Design and adversarial verification run on a stronger model; the
mechanical build runs on a cheaper one. Every shipped piece is reviewed
cold by an independent agent before it merges. It's deliberate, traceable,
and surprisingly cheap.

## Roadmap

| Stage | Delivers |
|---|---|
| ✅ **001 Foundation** | pipeline core + CLI (done) |
| **002 View & info** | `view`, `info` (+EXIF) — first user-facing commands |
| **003 Transform & output** | `resize`, `shrink`, `thumbnail`, `convert`, `auto-orient` |
| **004 Compose & metadata** | `watermark`; EXIF `strip` / `clean --gps` / `set` |
| **005 Batch & recipes** | `edit` + `--save-recipe`, parallel `apply` over many files |
| **006 Hardening & security** | decode limits, traversal tests, `cargo audit`, threat-model pass |

Beyond the MVP: geometry extras (**crop** first), an effects catalog,
WebP/AVIF output, `compare`/SSIM quality tuning, and — eventually — a
terminal UI for live-preview editing that exports a recipe.

The throughline: a clean, trait-based core where "a wide and interesting
feature set" is just more `Operation`s plugged into a pipeline that already
knows how to run one image or a whole batch.
