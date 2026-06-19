---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-030
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-004
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-06-18

references:
  decisions: [DEC-032, DEC-031, DEC-002, DEC-015, DEC-007]
  constraints:
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - pure-rust-codecs-default
  related_specs: [SPEC-029]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-004's <capability>". Optional; null is acceptable.
value_link: >
  Adds text mode to `watermark` (render a string at a gravity anchor) — the
  last STAGE-004 command; completes the compose-and-metadata stage.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec
        (Failing Tests + Implementation Context); emitted DEC-032 (ab_glyph +
        bundled BSD-3 Go font, no imageproc); two design-time probes (ab_glyph
        layout/raster of the Go font; imageproc dep-tree pulls sdl2 → rejected).
        Added ab_glyph + the font asset; just deny + lean build green.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: >
        text watermark: src/text render_text/parse_color (ab_glyph, bundled Go
        font) + watermark --text mode reusing SPEC-029 Watermark compositing;
        DEC-032; no imageproc.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-030: text watermark render text at gravity anchor

## Context

The **last STAGE-004 command** — text mode for `watermark`. Where SPEC-029's
`watermark --image LOGO` overlays an image, `watermark --text "© me"` rasterizes a
string and composites it at a gravity anchor. When this ships, **STAGE-004 is
complete** (compositing + the full metadata lane) and every single-image MVP command
exists.

The insight (DEC-032): **render the text to a transparent RGBA overlay, then reuse the
entire SPEC-029 compositing path** (`Gravity::placement` + the `Watermark` op). So the
only genuinely new code is "string + font → `RgbaImage`" via **`ab_glyph`** (the
rasterizer; layout is hand-rolled). The default font is **bundled**
(`assets/fonts/Go-Regular.ttf`, BSD-3, embedded with `include_bytes!`), overridable
with `--font PATH`. We deliberately avoid `imageproc` (it pulls `sdl2`/`nalgebra`) —
see DEC-032.

`watermark` exists as the SPEC-029 image command; this spec **adds text mode** to the
same subcommand (`--image` XOR `--text`). Governing: **DEC-032** (ab_glyph + font),
**DEC-031** (overlay/IO boundary — reused), **DEC-002** (`Operation`).

## Goal

Add `watermark <inputs…> --text STRING [--font PATH] [--size N] [--color HEX]` (plus
the shared `--gravity`/`--opacity`/`--margin`): rasterize the text with `ab_glyph`
into an RGBA overlay and composite it through the existing `Watermark` op. Default
font is the bundled Go font; `--image` and `--text` are mutually exclusive (exactly
one required). No `imageproc`.

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — `Gravity` (+ `placement`), the `Watermark` op +
    `Watermark::new`, its RGBA compositing `apply` (SPEC-029) — **reused verbatim**.
  - `src/cli/mod.rs` — the `Commands::Watermark` clap variant + `run_watermark`
    (SPEC-029); extend both. `CliError::Usage`/load→exit-3 mapping.
  - `decisions/DEC-032-*.md` (the ab_glyph probe + font), `DEC-031` (IO boundary).
  - `assets/fonts/Go-Regular.ttf` — the bundled default (already committed).
- **External crate (added by DEC-032):** `ab_glyph` `=0.2.32`
  (https://docs.rs/ab_glyph/0.2.32) — `FontRef`/`FontVec`, `Font`, `ScaleFont`,
  `PxScale`, `point`, `outline_glyph`, `OutlinedGlyph::{px_bounds, draw}`. KEEP `std`
  (do not set `default-features=false`).
- **Related code paths:** `src/text/` (new), `src/operation/mod.rs`, `src/cli/mod.rs`,
  `tests/watermark.rs`.

## Outputs

- **Files created:**
  - `src/text/mod.rs` — pure text rendering (no file IO; data + pixels only):
    - `pub const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");`
    - `pub fn render_text(font_bytes: &[u8], text: &str, size_px: f32, color: [u8; 4]) -> Result<RgbaImage, TextError>`
      — lay out the string (advance/kern/ascent via `ab_glyph`), rasterize each glyph's
      coverage into a tightly-sized transparent `RgbaImage`, glyph pixels = `color`
      with `alpha = round(coverage * color.a)`.
    - `pub fn parse_color(s: &str) -> Result<[u8; 4], TextError>` — `RRGGBB` /
      `#RRGGBB` (alpha 255); also accept `RRGGBBAA`. Bad hex → error.
    - `pub enum TextError { Font(String), Color(String), Empty }` (`thiserror`).
  - `src/lib.rs` — `pub mod text;`.
- **Files modified:**
  - `src/cli/mod.rs` — extend `Commands::Watermark`: make `image` `Option<String>`;
    add `text: Option<String>`, `font: Option<String>`, `size: Option<f32>`,
    `color: Option<String>`, with clap `conflicts_with`/`required_unless_present` so
    `--image` XOR `--text`. Extend `run_watermark`: in **text mode**, load the font
    (`--font PATH` at the IO boundary → exit 3 on failure; else `DEFAULT_FONT`), parse
    `--color` (default white `ffffff`) + `--size` (default `32.0`, `≤0` → exit 2),
    `text::render_text(..)` → `DynamicImage::ImageRgba8` → build a `Watermark` op with
    that overlay (gravity/opacity/margin; scale=None, tile=false) → `run_pixel_op`.
  - `Cargo.toml` — `ab_glyph = "=0.2.32"` (done at design).
  - `docs/api-contract.md` — extend the `watermark` entry with text mode (done at design).
- **New exports:** `crate::text::{render_text, parse_color, TextError, DEFAULT_FONT}`.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg watermark <INPUTS...> --image LOGO  [gravity/opacity/scale/margin/tile]   # SPEC-029 (unchanged)
crustyimg watermark <INPUTS...> --text STRING [--font PATH] [--size N] [--color HEX]
                                              [--gravity G] [--opacity O] [--margin M]
```

- **`--image` XOR `--text`** — exactly one is required. Neither, or both → exit **2**
  (enforce with clap `required_unless_present`/`conflicts_with`).
- **Text-only flags** (`--text`/`--font`/`--size`/`--color`) used in image mode, or
  **image-only flags** (`--scale`/`--tile`) in text mode → exit **2** (clap
  `conflicts_with`, or a runtime check).
- **`--font PATH`** (optional) — a TTF/OTF; loaded at the CLI IO boundary. Missing/
  unreadable/unparseable → exit **3**. Omitted → the bundled Go font (DEFAULT_FONT).
- **`--size N`** (default **32.0**) — font size in px; `N ≤ 0` → exit **2**.
- **`--color HEX`** (default **`ffffff`** white) — `RRGGBB`/`#RRGGBB`/`RRGGBBAA`;
  malformed → exit **2**.
- **Shared (text mode):** `--gravity` (default southeast), `--opacity` (0–1, default 1
  — multiplies the rendered text's alpha), `--margin`. `--scale`/`--tile` are
  image-only.
- **Compositing:** the rendered text RGBA overlay flows through the **same**
  `Watermark` op as SPEC-029 — gravity placement, opacity, margin, clipping, and the
  standard `run_pixel_op` fan-out (single → stdout/`-o`/`--out-dir`, multi →
  `--out-dir`, per-input failure → exit 6) are all inherited unchanged.

## Rendering mechanics (PINNED — probe-verified, DEC-032)

`text::render_text(font_bytes, text, size_px, color)`:
1. `let font = FontRef::try_from_slice(font_bytes).map_err(|e| TextError::Font(..))?;`
   (CLI passes the bundled const or the `--font` bytes). Empty `text` → `TextError::Empty`.
2. `let scale = PxScale::from(size_px); let sf = font.as_scaled(scale);`
3. First pass — layout: walk `text.chars()`, accumulate `x += sf.h_advance(gid)` (+
   `sf.kern(prev, gid)`), collect each `font.outline_glyph(gid.with_scale_and_position(
   scale, point(x, sf.ascent())))`; track the union of `px_bounds()` to size the canvas
   (width = ceil(max x extent), height = ceil(ascent − descent) or the bounds union).
4. Allocate a transparent `RgbaImage::from_pixel(w, h, [0,0,0,0])`; second pass —
   `outlined.draw(|gx, gy, c| { let a = (c * (color[3] as f32)).round() as u8; write
   [color[0],color[1],color[2],a] at (bounds.min + (gx,gy)) with source-over if
   overlapping })`. (Probe: "© me" @48px → ~98×40 px, 1827 coverage pixels.)
5. `Ok(canvas)`. Pure — **no file IO** (font bytes are an input); lives in `src/text/`.

## Acceptance Criteria

- [ ] `watermark base --text "© me" -o out` writes a composited image (exit 0) that
  decodes and differs from the base; rendered text pixels are present.
- [ ] Default font works with **no `--font`** (bundled Go font).
- [ ] `--font assets/fonts/Go-Regular.ttf` (explicit) also works; a missing `--font` →
  exit **3**.
- [ ] `--color ff0000` renders red text (sampled glyph pixels are ~red); bad `--color`
  → exit **2**.
- [ ] `--size` scales the text (larger size → taller rendered overlay).
- [ ] `--gravity`/`--opacity`/`--margin` behave as in SPEC-029 (text block anchored,
  alpha-scaled).
- [ ] `--image` XOR `--text`: neither → exit 2; both → exit 2.
- [ ] `--size 0` → exit 2; image-only `--scale`/`--tile` with `--text` → exit 2.
- [ ] No `imageproc`; `cargo deny` green; the **lean build** (`--no-default-features`)
  still compiles (ab_glyph keeps `std`).

## Failing Tests

Written during **design**, BEFORE build. Native fixtures; bundled font for unit tests.

- **`src/text/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"render_text_produces_nonempty_coverage"` — `render_text(DEFAULT_FONT, "Hi", 32.0,
    [255,255,255,255])` → non-empty image with ≥1 fully/partly opaque pixel.
  - `"render_text_applies_color"` — render in `[255,0,0,255]` → some pixel is ~red
    (r high, g/b low, a>0).
  - `"render_text_size_scales"` — height at size 64 > height at size 16.
  - `"render_text_empty_is_error"` — `""` → `TextError::Empty`.
  - `"render_text_bad_font_is_error"` — junk bytes → `TextError::Font`.
  - `"parse_color_hex_variants"` — `ffffff`/`#000000`/`ff0000`/`ff000080` parse;
    `zzz`/`fff`(if 3-digit unsupported) → `TextError::Color`.
- **`tests/watermark.rs` (integration, extend)**
  - `"text_watermark_writes_output"` — `watermark base.png --text "©" -o out.png` →
    exit 0; out differs from base.
  - `"text_watermark_default_font"` — no `--font` → exit 0.
  - `"text_watermark_custom_font"` — `--font assets/fonts/Go-Regular.ttf` → exit 0.
  - `"text_watermark_missing_font_exits_3"` — `--font nope.ttf` → exit 3.
  - `"text_watermark_bad_color_exits_2"` — `--color zzz` → exit 2.
  - `"watermark_requires_image_or_text_exits_2"` — neither flag → exit 2.
  - `"watermark_image_and_text_conflict_exits_2"` — both → exit 2.
  - `"text_watermark_size_zero_exits_2"` — `--size 0` → exit 2.
- **`tests/cli.rs`** — `watermark` already in the subcommand lists; the SPEC-029
  NotImplemented-stub sample was repointed to `edit` — leave it.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-032` — **the key one.** `ab_glyph` (KEEP `std`), no `imageproc`; bundled BSD-3
  Go font via `include_bytes!`; render-to-overlay then reuse SPEC-029 compositing.
- `DEC-031` — `--font PATH` loads at the CLI IO boundary; `src/text/` stays file-free
  (font bytes are an input; the bundled const is compile-time data).
- `DEC-002` — text reuses the `Watermark` `Operation`; no new op type needed.
- `DEC-015`/`DEC-007` — fan-out + exit codes via `run_pixel_op`; typed errors, no
  `unwrap`/`expect`/`panic!` off test paths; CLI maps `TextError`→exit 2/3.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — satisfied by DEC-032 (ab_glyph).
- `pure-rust-codecs-default` — ab_glyph is pure-Rust; lean build still compiles.
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-029` (shipped, PR #33) — the `watermark` image overlay: `Gravity`,
  `Watermark`, `run_watermark`, `tests/watermark.rs`. **Read it first — text mode
  extends it and reuses the compositing wholesale.**

### Out of scope (for this spec specifically)

- Multi-line text, alignment, text stroke/outline, background box, rotation — later.
- Recipe round-trip of watermark (STAGE-005, per DEC-031).
- Additional bundled fonts / feature-gating the font or ab_glyph (DEC-032 follow-up if
  binary size matters).
- Auto-fitting text size to the base (user sets `--size`).

## Notes for the Implementer

- **Reuse, don't reinvent:** render the text to an `RgbaImage`, wrap as
  `DynamicImage::ImageRgba8`, and feed it to `Watermark::new(overlay, "<text>", gravity,
  opacity, /*scale*/None, margin, /*tile*/false)`. The existing `apply` composites it —
  you do NOT write new gravity/opacity/clipping code.
- **`ab_glyph` `std` trap (DEC-032):** the dep is `ab_glyph = "=0.2.32"` (default
  features). Do NOT add `default-features = false`. Run `cargo build
  --no-default-features` locally — it must still compile.
- **IO boundary:** load `--font PATH` with `std::fs::read` in `run_watermark` (→ exit
  3); never open a file in `src/text/` or `src/operation/`.
- **clap XOR:** `image: Option<String>` `#[arg(long, conflicts_with = "text")]`;
  `text: Option<String>` `#[arg(long, required_unless_present = "image")]`. Verify
  neither/both → exit 2 (clap usage). Keep the SPEC-029 image path byte-for-byte.
- Keep diagnostics on stderr; default `--color` = white, `--size` = 32.0.
- Add the text integration tests to `tests/watermark.rs`; unit tests for `render_text`/
  `parse_color` in `src/text/mod.rs`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-030-text-watermark`
- **PR (if applicable):** `feat(operation): text watermark via ab_glyph (SPEC-030)`
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — DEC-032 (at design) governs.
- **Deviations from spec:**
  - The new `src/text/mod.rs` lives in its own top-level module (`pub mod text;`)
    as the spec's Outputs prescribe, NOT under `src/operation/` (DEC-032's prose
    mentions `operation/` once; the spec's authoritative Outputs/build prompt pin
    `src/text/`). No file IO either way.
  - All-whitespace text (no drawable glyphs) renders a 1×1 transparent overlay
    instead of erroring — a defensive total path; the spec only pins empty → Empty.
- **Follow-up work identified:**
  - Multi-line / alignment / stroke / background box / rotation (out of scope here;
    a later typography spec, per DEC-032 follow-up).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Minor: DEC-032's prose says the render fn "lives in `operation/`", while the
   spec Outputs + build prompt pin `src/text/`. I followed the build prompt
   (`src/text/`), which is the authoritative surface. Nothing else was unclear —
   the PINNED rendering mechanics were precise enough to implement directly.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The `#[allow(clippy::too_many_arguments)]` on `run_watermark` (already
   present at SPEC-029) had to be re-applied after I rewrote the signature; worth a
   one-line note that extending an `allow`-ed fn keeps the allow, but not a missing
   constraint.

3. **If you did this task again, what would you do differently?**
   — Reconcile DEC-032's "operation/" wording with the spec up front to avoid the
   momentary ambiguity, and lift the `WatermarkSource` bundling struct out earlier
   (it kept the arg count from growing past the existing allow).

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
