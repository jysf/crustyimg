# SPEC-030 build prompt — text watermark (`watermark --text`)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-030 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-032. This is the LAST
STAGE-004 command — text mode for the existing `watermark`. Make the spec's
`## Failing Tests` pass with the smallest correct change, then open a PR and STOP.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-030-text-watermark-render-text-at-gravity-anchor.md`
   — `## Command surface (PINNED)`, `## Rendering mechanics (PINNED)`, `## Failing Tests`,
   `## Notes for the Implementer`.
2. `decisions/DEC-032-text-watermark-ab-glyph-and-bundled-go-font.md` — the ab_glyph
   probe (layout/raster), the bundled Go font, the `std`-feature trap, and "no imageproc".
3. `decisions/DEC-031-*.md` — the overlay / IO-boundary rule (reused).
4. `src/operation/mod.rs` — `Gravity` (+ `placement`), `Watermark` + `Watermark::new`,
   its RGBA `apply` (SPEC-029) — **reused verbatim**.
5. `src/cli/mod.rs` — the `Commands::Watermark` clap variant + `run_watermark`
   (SPEC-029); extend both.

## What to build (the only new code is "text → RgbaImage")
- `src/text/mod.rs` (new; `pub mod text;` in `src/lib.rs`) — pure, NO file IO:
  - `pub const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");`
  - `pub fn render_text(font_bytes: &[u8], text: &str, size_px: f32, color: [u8;4]) -> Result<RgbaImage, TextError>`
    — `ab_glyph` layout (advance/kern/ascent) + rasterize glyph coverage into a tight
    transparent `RgbaImage`; glyph alpha = `round(coverage * color[3])`. Empty text →
    `TextError::Empty`; bad font → `TextError::Font`. (Mirror DEC-032's probe.)
  - `pub fn parse_color(&str) -> Result<[u8;4], TextError>` — `RRGGBB`/`#RRGGBB`/`RRGGBBAA`.
  - `pub enum TextError { Font(String), Color(String), Empty }` (`thiserror`).
- `src/cli/mod.rs`:
  - Extend `Commands::Watermark`: make `image` `Option<String>`; add `text: Option<String>`,
    `font: Option<String>`, `size: Option<f32>`, `color: Option<String>`. clap: `image`
    `#[arg(long, conflicts_with = "text")]`, `text` `#[arg(long, required_unless_present = "image")]`
    so `--image` XOR `--text` (neither/both → exit 2).
  - Extend `run_watermark`: in TEXT mode — load font (`--font PATH` via `std::fs::read`
    at the IO boundary → exit 3; else `DEFAULT_FONT`), `parse_color`(default `ffffff`)
    → exit 2 on bad, `size` (default 32.0; `≤0` → exit 2), `text::render_text(..)`
    → `DynamicImage::ImageRgba8` → `Watermark::new(overlay, text, gravity, opacity,
    None, margin, false)` → `Pipeline` → `run_pixel_op(.., global.quality, None, None)`.
    Keep the SPEC-029 image path unchanged. Map `TextError` → exit 2.

## Hard rules
- **Reuse SPEC-029 compositing** — do NOT write new gravity/opacity/clip code; build a
  `Watermark` op with the rendered overlay.
- **NO file IO in `src/text/` or `src/operation/`** (DEC-031) — `--font` loads in
  `run_watermark`; the bundled font is `include_bytes!` (compile-time data). **NO `imageproc`.**
- **ab_glyph `std` trap (DEC-032):** keep `ab_glyph = "=0.2.32"` (already in Cargo.toml);
  do NOT set `default-features = false`. `cargo build --no-default-features` MUST compile.
- Typed errors; NO `unwrap`/`expect`/`panic!` off test paths. Diagnostics to stderr.
  Reuse GLOBAL `-o`/`--out-dir`/`-q`/`-y` (no local shadowing).
- Native fixtures; unit tests use `DEFAULT_FONT`. Every named test in `## Failing Tests`
  (6 unit in src/text/mod.rs, 8 integration in tests/watermark.rs) must EXIST and PASS.

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features    # CI 'lean build' — ab_glyph std must survive
cargo deny check licenses            # ab_glyph already pinned (DEC-032); stay green
```

## Git / PR
- Branch `feat/spec-030-text-watermark` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(operation): text watermark via ab_glyph (SPEC-030)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-032, DEC-031, DEC-002,
  DEC-015, DEC-007 / Constraints checked / New decisions — "No new DEC" — DEC-032 at design).
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; orchestrator fills at ship).

## Cost
```
- cycle: build
  agent: claude-opus-4-8
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "text watermark: src/text render_text/parse_color (ab_glyph, bundled Go font) + watermark --text mode reusing SPEC-029 Watermark compositing; DEC-032; no imageproc"
```
(Use the agent id of the session that actually runs the build.)

## When done
`just advance-cycle SPEC-030 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
