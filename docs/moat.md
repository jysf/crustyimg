# crustyimg — the moat

*A living strategy note: what makes crustyimg defensibly different, what's built,
and what still has to land. Snapshot: 2026-06-18, after STAGE-008 + STAGE-009.*

## The one-line claim

**Tell the tool the outcome you want — a visual quality or a file-size budget, in a
modern format — and get the smallest file that meets it, from one pure-Rust binary
with zero system dependencies.** "Set the look, not the number."

This reframes image prep from *guess-a-quality-knob* to *declare-an-intent*. Almost
nothing in the CLI layer does this, and that's the wedge.

## Why the space is open

- **The product/CLI layer is stale.** ImageMagick is cryptic and heavy (Windows DLL
  pain); libvips/`vips` is fast but obscure and a system dependency; `sharp` is
  Node + native; squoosh-cli is unmaintained; ImageOptim is macOS-only; the
  single-format optimizers (jpegoptim/oxipng/pngquant/cwebp/avifenc) each do exactly
  one format; exiftool is hard and strips incompletely; imgproxy/thumbor are servers,
  the wrong shape for a build step.
- **The component (crate) layer is hot** — a pure-Rust imaging frontier exists
  (zune, the awxkee SIMD cluster, ravif, jxl-oxide, ssimulacra2) — but **nobody
  assembles it into one ergonomic, single-binary tool.**
- **Two industry shifts crustyimg rides:** (1) "encode to a perceptual *target*, not
  a quality number"; (2) pure-Rust safe-SIMD replacing C/FFI across the stack.

## What's built (the defensible core)

### 1. The engine — outcome-driven compression (STAGE-008)
- **Perceptual auto-quality:** binary-search the encoder quality against the
  **SSIMULACRA2** metric → the smallest file that still clears a visual target
  (`--target visually-lossless` / `--ssim <N>`).
- **Byte budgets:** `--max-size <KB>` with a dimension-reduction fallback when
  quality alone can't fit — works for every format.
- **Modern formats:** WebP (pure-Rust, default) and AVIF (feature-gated), behind a
  clean `LossyFormat` seam. The search is encoder-agnostic.
- All pure-Rust by default, zero system deps.

### 2. The surface — commands a person actually runs (STAGE-009)
- **`optimize`** — one button: auto-orient + strip metadata + perceptual
  visually-lossless re-encode, format/size-preserving. The "just make this web-good"
  default.
- **`diff`** — a perceptual SSIMULACRA2 score *and* a `--fail-under` CI
  visual-regression gate (its own exit code 7).
- **`responsive`** — width × format variants + a paste-ready `<picture>`/srcset
  snippet. The deliverable a web developer actually ships.

### 3. Verification — claims you can check (STAGE-009)
- `diff` makes quality measurable; the `criterion` benchmark net measures the hot
  paths. Standing rule (DEC-028): **any size/speed claim is gated on equal quality.**
  Honesty is itself a differentiator in a space full of quality-blind "smaller!"
  claims.

### Structural bets underneath all of it
- **One static binary, all formats, zero system deps by default.**
- **Permissive license (MIT/Apache)** — load-bearing: it's why the codecs are
  pure-Rust and why the tool is freely embeddable/redistributable.
- **Reproducible** — a load-once `Operation` pipeline + TOML recipes as the base.

## Where the moat is still thin (honest)

| Axis | Status | Lands in |
|---|---|---|
| Verifiable privacy (selective `clean --gps`, EXIF audit-as-linter) | **not built** — `optimize` strips everything as a side effect, but the *verifiable* story is missing | STAGE-004 |
| Surfaced reproducibility (`edit`/`--save-recipe`/batch `apply`/`watch`) | core exists; not surfaced as a headline workflow | STAGE-005 |
| Proof at scale (cross-tool + quality-per-byte comparisons, `BENCHMARKS.md`) | only the local micro-net exists | STAGE-006 / later |
| Distribution (release binaries, brew, crates.io) | not released — *a moat nobody can `brew install` isn't fully real* | STAGE-007 |

## Net read

The moat is **strong and genuinely differentiated on the quality + modern-formats +
web-delivery axis, with a credibility leg started.** Two more high-value axes
(verifiable privacy, surfaced reproducibility) plus distribution remain to make it
complete. The next stage (STAGE-004, compose & metadata) is the start of the privacy
axis — and it also unlocks `optimize`'s selective-preserve upgrade (DEC-024 revisit).

## Pointers

- Engine: `src/quality/` (the SSIMULACRA2 search + `LossyFormat` seam), `src/sink/`
  (per-format encode). Surface: `src/cli/` (`optimize`/`diff`/`responsive`).
- Decisions: DEC-019 (perceptual), DEC-020/021/022 (AVIF/WebP), DEC-023 (size
  fallback), DEC-024 (optimize), DEC-025 (diff + exit 7), DEC-026 (responsive),
  DEC-027 (display default), DEC-028 (benchmarking + equal-quality principle).
- Roadmap + competitive synthesis: `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md`.
