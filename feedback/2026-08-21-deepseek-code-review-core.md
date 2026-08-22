---
source: DeepSeek (external LLM code review)
received: 2026-08-21
scope: crustyimg core — architecture, errors, performance, testing, security, docs, CLI, RAW
triaged_by: orchestrator session f20dabb9, 2026-08-21
---

# DeepSeek code review, batch 1 — raw capture + triage

The raw batch is reproduced verbatim below. **Nothing in it changes output bytes**, so none of
it gates the 0.7.1 tag.

## Triage summary

Every claim was checked against the code rather than taken at face value
([[verify-handed-crate-lists-adversarially]]). Of ~22 items:

| bucket | n | meaning |
|---|---:|---|
| **already done** | 5 | the repo already has it; the review's premise is stale or wrong |
| **conflicts with a shipped DEC** | 2 | needs a maintainer ruling to adopt, not silent adoption |
| **actionable** | ~11 | real, new, worth filing |
| **needs measurement first** | ~4 | plausible, but the premise is unverified |

### Already done — premise wrong

- **"Add fuzz targets for AVIF / SVG / HEIC decode."** All four already exist:
  `fuzz/fuzz_targets/{avif_decode,heic_decode,raw_preview,svg_decode}.rs`. The review saw only
  `raw_preview`.
- **"`--verbose` exists via `tracing`."** `tracing` is not a dependency (0 matches in
  `Cargo.toml`). Diagnostics go to stderr gated by `-v`/`--quiet` (AGENTS §11).
- **"Shell completions are *likely* supported via `clap_complete`."** They are:
  `clap_complete = "4.6.5"`, `src/cli/mod.rs:21`, `completions <SHELL>` subcommand. The hedge
  ("likely") is the tell that this was inferred rather than checked.
- **"Integrate `indicatif` for a progress bar."** `indicatif = "0.18.6"` is a dependency and
  `ProgressBar` is already wired in `src/cli/build.rs:621-623`. Whether `apply` also has one is a
  narrower, real question — but the CLI is not uniformly silent.
- **"Add an `examples/` directory."** It exists with 9 files. They are all fixture generators and
  measurement probes, not user-facing workflows — so the *idea* (workflow examples) survives, the
  *premise* does not.
- **"Create `docs/recipe-schema.md`; the TOML recipe system is under-documented."**
  `docs/recipes.md` exists. Whether it covers every key/type/range is a fair question; "no docs" is
  not.

### Conflicts with a shipped decision — maintainer ruling required

- ⛔ **"Create a top-level `CrustyError` wrapping `ImageError`, IO errors and CLI errors."**
  This contradicts **DEC-007**, which is explicit: library code returns typed `thiserror` enums and
  **only** `main.rs`/`cli` use `anyhow` at the binary boundary. A unified crate-wide error collapses
  exactly the boundary DEC-007 draws, and would put CLI concerns in the library's type. Adopting it
  means superseding DEC-007, not extending it.
- ⛔ **"Split `Image` into `ImageCore` / `ImageMetadata` / `ImageOperations`."**
  `ImageOperations` contradicts **DEC-002** — the `Operation` trait *is* the extension point, and
  transforms are deliberately not methods on `Image`. `ImageMetadata` cuts across **DEC-003**'s
  pixel-lane / container-lane split, which is a lane separation rather than a struct one.
  ⚠ It also mis-prioritizes: `Image`'s 17 `pub fn` sit in a file of **797 production lines**, while
  the repo's own filed decomposition target, `src/cli/optimize.rs`, is at **1,876**. Ranking by
  God-Object smell rather than by measurement picks the wrong file.

### Actionable — real and new

- `SourceContainer` has neither `Display` nor `FromStr`. Small, correct.
- `LimitsExceeded` does not say *which* limit was hit (dimension / pixel count / allocation).
- RAW preview error `"no decodable embedded JPEG preview"` conflates "none present" with
  "present but corrupt" — a real diagnosis gap for camera-specific reports.
- Per-operation memory caps on top of the global `Limits` pixel budget.
- `proptest` for operation invariants; golden-file tests for resize/crop/watermark.
- Coverage on `operation/` and `pipeline/`.
- A `crustyimg bench` subcommand — no such verb exists today; `just bench` is the current entry.
- `docs.rs` build with `--all-features` so wasm/HEIC items are documented.
- **A wasm32 CI leg.** ⚠ Already filed on STAGE-042 as a chore — the review **independently
  corroborates** it, which is the most valuable thing in the batch: two sources, no contact.
- Using `kamadak-exif` (already a dependency) to read the embedded-preview offset/length instead of
  scanning, per §8. This intersects the RAW-develop-as-its-own-library direction.

### Needs measurement before it is worth filing

- **"RAW byte-scanning iterates byte-by-byte; use `memchr`."** The scan uses `.windows(2).position(..)`
  (`src/image/raw.rs:319-320`), not a naive index loop. `memchr` may still win on 50 MB files, but the
  stated premise is wrong and the win is unquantified. **Measure before specing.**
- **"`apply --recipe` runs single-threaded to maintain WASM parity."** A `--jobs` flag exists
  (`src/cli/mod.rs:86`) and batch parallelism via rayon is **DEC-006**. Whether `apply` specifically
  honours it is a real question; the stated rationale is not the repo's.
- **"`with_pixels` clones the pixel buffer unnecessarily during metadata-only ops."** Plausible and
  worth a profile; no measurement offered.
- **"Aim for >80% coverage."** The repo has deliberately never set a percentage gate (AGENTS §12:
  "No hard percentage gate"). A number without a rationale is not an improvement over that.

---

## Raw batch, verbatim

> DeepSeek Code Review & Quality Improvement Prompt – crustyimg Core
>
> Context: You are a Staff-level Rust Software Engineer performing a deep review of the crustyimg
> codebase. Below is a comprehensive list of actionable improvements across architecture, error
> handling, performance, testing, security, and documentation. Address these items systematically.
>
> **1. Architecture & Modularity** — Image Struct Refactor (split into ImageCore / ImageMetadata /
> ImageOperations); SourceContainer Ergonomics (Display + FromStr).
>
> **2. Error Handling** — Unified crate-wide `CrustyError`; improve `LimitsExceeded` context; RAW
> preview error specificity ("no preview found" vs "preview corrupted").
>
> **3. Performance** — optional `--parallel` for `apply --recipe` behind a `parallel` feature;
> zero-copy metadata stripping in `with_pixels`; built-in `crustyimg bench`; `memchr`/SIMD for the
> RAW `FF D8 FF` scan.
>
> **4. Security & Safety** — fuzz targets for AVIF / SVG / HEIC decode; per-operation memory caps.
>
> **5. Testing** — coverage >80% on `operation/` and `pipeline/`; `proptest` for invariants; golden
> file tests; WASM CI verification.
>
> **6. Documentation** — `examples/` with `optimize_website.rs`, `ci_gate.rs`, `batch_convert.rs`;
> `docs/recipe-schema.md`; docs.rs with `--document-private-items` and `--all-features`.
>
> **7. CLI & UX** — `indicatif` progress bar for batch jobs; `completions <shell>` subcommand and
> README docs; `-v`/`-vv` verbosity levels via `tracing`.
>
> **8. RAW module** — byte-scanning for JPEG previews is vendor-fragile; use the `exif` crate to
> read the exact embedded-preview offset/length instead.
