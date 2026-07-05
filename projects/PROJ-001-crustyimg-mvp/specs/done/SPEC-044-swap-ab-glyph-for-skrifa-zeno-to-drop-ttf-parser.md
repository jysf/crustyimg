---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-044
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-010
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5     # build cycle runs on Sonnet (prescriptive prompt)
  created_at: 2026-07-04

references:
  decisions: [DEC-045, DEC-032, DEC-042, DEC-031, DEC-004, DEC-018]
  constraints: [no-new-top-level-deps-without-decision, pure-rust-codecs-default, no-agpl-default-deps]
  related_specs: [SPEC-030, SPEC-029]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-010's <capability>". Optional; null is acceptable.
value_link: "Removes the unmaintained `ttf-parser` from the tree so STAGE-010 can delete the RUSTSEC-2026-0192 ignore toward a clean 0.2.0."

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
      recorded_at: 2026-07-04
      notes: >
        Main-loop orchestrator work, not separately metered. Two design-time probes:
        (1) disproved the backlog's `fontdue` plan — fontdue 0.9.3 still pulls
        `ttf-parser 0.21.1` and RUSTSEC-2026-0192 is crate-wide (`patched=[]`), so it
        would not clear the ignore; (2) verified `skrifa 0.44` + `zeno 0.3.3` on the
        real Go-Regular (ascent/advance/glyph-bounds match; `(coverage, Placement)`
        analog of ab_glyph `px_bounds()`+`draw()`). Authored DEC-045, the spec (failing
        tests + probe-verified implementation context), STAGE-010, and the build prompt.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 113188
      estimated_usd: 0.62
      duration_minutes: 27
      recorded_at: 2026-07-04
      notes: >
        Real metered subagent on Sonnet. subagent_tokens=113188, duration_ms=1622668.
        estimated_usd at Sonnet list (~$3/$15 per MTok, ~80/20). Rewrote `src/text/mod.rs`
        on skrifa+zeno (y-negating `ZenoPen`, coverage-buffer composite, no kerning);
        swapped Cargo.toml deps; deleted the `-0192` deny.toml entry; 6 existing + 4 new
        text tests green; all gates green (`cargo tree` ttf-parser/ab_glyph = 0). PR #49.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 59862
      estimated_usd: 0.53
      duration_minutes: 4
      recorded_at: 2026-07-04
      notes: >
        Real metered independent Explore subagent on Opus. subagent_tokens=59862,
        duration_ms=213175. Adversarial review of the rasterization port (y-negation,
        alpha normalization cov/255*base, buffer indexing, whitespace/bounds union,
        source-over) + API preservation + hardening + no scope creep; re-ran all gates.
        VERDICT PASS, no defects. Orchestrator additionally ran a visual old-vs-new
        pixel A/B (mean channel diff 3.07; legible, same placement) before merge.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-04
      notes: >
        Main-loop orchestrator: squash-merged PR #49 (6d79f1b), ran the ship
        bookkeeping (cost, timeline, STAGE-010 backlog, archive), confirmed CI green on
        main. First `deny.toml` ignore eliminated toward the clean 0.2.0.
  totals:
    tokens_total: 173050
    estimated_usd: 1.15
    session_count: 4
---

# SPEC-044: swap ab_glyph for skrifa+zeno to drop ttf-parser

## Context

This is the first spec of **STAGE-010** (advisory elimination). `watermark --text`
currently rasterizes glyphs with `ab_glyph` (DEC-032), which pulls the **unmaintained
`ttf-parser`** crate transitively (`ab_glyph` → `owned_ttf_parser` → `ttf-parser`).
**RUSTSEC-2026-0192** flags `ttf-parser` as unmaintained; crustyimg ships v0.1.x with a
documented `deny.toml` ignore for it (DEC-042). This spec removes `ttf-parser` from the
tree at the source so that ignore can be deleted — one step toward a clean-`deny` 0.2.0
(PROJ-001, STAGE-010).

The original backlog named `fontdue` for this job. **A design-time probe disproved the
premise:** fontdue `0.9.3` *still* depends on `ttf-parser 0.21.1`, and RUSTSEC-2026-0192 is
crate-wide (`patched = []`, `informational = "unmaintained"`), so fontdue would not remove
the ignore. The advisory's own recommended alternative — **`skrifa`** (Google
`fontations`) — is `ttf-parser`-free. **DEC-045** records the retarget to `skrifa` + `zeno`
and supersedes DEC-032's rasterizer choice. The bundled Go font, `--font` override, and the
public `src/text` API are all retained.

## Goal

Re-implement `src/text/mod.rs`'s glyph rasterization on **`skrifa` `=0.44.0`** (outlines +
metrics) + **`zeno` `=0.3.3`** (mask rasterization), removing the `ab_glyph` dependency and
the `RUSTSEC-2026-0192` ignore from `deny.toml`, with **no user-observable change** to
`watermark --text` output.

## Inputs

- **Files to read:**
  - `src/text/mod.rs` — the module being rewritten (public API + the two-pass layout/raster
    algorithm to preserve).
  - `decisions/DEC-045-text-watermark-rasterizer-skrifa-zeno.md` — the decision, the
    probe-verified API calls, and the kerning-drop rationale.
  - `decisions/DEC-032-text-watermark-ab-glyph-and-bundled-go-font.md` — the superseded
    rasterizer choice; the retained font/boundary decisions.
  - `Cargo.toml` (lines ~52–60) — the `ab_glyph` dep block to replace.
  - `deny.toml` (the `[advisories]` block, ~lines 85–90) — the `-0192` entry to delete.
  - `src/cli/mod.rs` `run_watermark` (~line 2600) — the sole caller; confirms the public
    API must not change.
- **External crates:** `skrifa` `=0.44.0` (MIT OR Apache-2.0), `zeno` `=0.3.3` (Apache-2.0
  OR MIT). Docs: https://docs.rs/skrifa/0.44.0, https://docs.rs/zeno/0.3.3.
- **Related code paths:** `src/text/`, `src/cli/mod.rs`.

## Outputs

- **Files modified:**
  - `src/text/mod.rs` — swap the rasterizer internals of `render_text`; the public
    signature, `parse_color`, `DEFAULT_FONT`, and `TextError` are **unchanged**. Add the
    new tests listed under *Failing Tests*.
  - `Cargo.toml` — remove `ab_glyph = "=0.2.32"`; add `skrifa = "=0.44.0"` and
    `zeno = "=0.3.3"`; update the surrounding comment block (the `ab_glyph` `std`-feature
    note no longer applies).
  - `deny.toml` — delete the `RUSTSEC-2026-0192` ignore entry and its comment.
- **New exports:** none. (`render_text`, `parse_color`, `DEFAULT_FONT`, `TextError`
  keep their exact signatures.)
- **Database changes:** none.

## Acceptance Criteria

- [x] `cargo tree` shows **no `ttf-parser`** (and no `ab_glyph`/`owned_ttf_parser`).
- [x] `deny.toml` has no `RUSTSEC-2026-0192` entry; `just deny`
      (`cargo deny check advisories bans sources licenses`) **passes**.
- [x] `render_text`, `parse_color`, `DEFAULT_FONT`, and `TextError` keep their exact
      signatures; `run_watermark` in `src/cli/mod.rs` compiles unchanged.
- [x] All **existing** `src/text` tests still pass (they are the behavioral regression
      contract), and the **new** tests below pass.
- [x] `watermark --text "© me"` renders legible text at the gravity anchor with
      `--size`/`--color`/`--opacity`/`--font` behaving as before (manual/verify check).
- [x] Both the full build and the **lean** build (`cargo build --no-default-features`)
      compile; `cargo fmt --check` and `cargo clippy` are clean.

## Failing Tests

Written during **design**, BEFORE build. Add these to the `#[cfg(test)] mod tests` in
`src/text/mod.rs`. They pin the invariants the `skrifa`+`zeno` path must satisfy; the
existing six tests must continue to pass unchanged.

- **`src/text/mod.rs` (tests module)**
  - `"render_text_accumulates_advance"` — `render_text(DEFAULT_FONT, "WWW", 32.0, WHITE)`
    is **wider** than `render_text(DEFAULT_FONT, "W", 32.0, WHITE)`. Asserts horizontal
    advance accumulates across glyphs via the new layout pass.
  - `"render_text_whitespace_contributes_advance"` — `render_text(DEFAULT_FONT, "A B", …)`
    is **wider** than `render_text(DEFAULT_FONT, "AB", …)` at the same size. Asserts a
    whitespace glyph advances the pen even though it produces no coverage.
  - `"render_text_all_whitespace_is_1x1"` — `render_text(DEFAULT_FONT, "   ", 32.0, WHITE)`
    returns a **1×1 fully-transparent** image (the "no drawable glyphs" branch), matching
    today's behavior.
  - `"render_text_height_tracks_font_size"` — for `"Hg"` (ascender+descender), the canvas
    height at `64.0` is within a sane window of the pixel size (e.g. `> 32 && <= 96`) and
    strictly greater than at `16.0`. Asserts skrifa metrics scale the raster.

> The headline outcome — `ttf-parser` gone + `-0192` deleted + `just deny` green — is a
> **gate**, not a unit test (verified in build/verify via `cargo tree` + `just deny`, per
> Acceptance Criteria), because `deny` state isn't assertable from a `#[test]`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### The verified swap (probe results — build on these, don't re-derive)

A design-time probe ran `skrifa 0.44.0` + `zeno 0.3.3` against the real
`assets/fonts/Go-Regular.ttf`. Confirmed calls:

```rust
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};
use zeno::{Command, Mask, Point};

// Parse (replaces FontRef::try_from_slice); Err -> TextError::Font(e.to_string()).
let font = FontRef::new(font_bytes).map_err(|e| TextError::Font(e.to_string()))?;
let size = Size::new(size_px);
let loc = LocationRef::default();

let ascent = font.metrics(size, loc).ascent;          // 30.234375 @32px — matches ab_glyph
let charmap = font.charmap();                          // charmap.map(ch) -> Option<GlyphId>
let gmetrics = font.glyph_metrics(size, loc);          // gmetrics.advance_width(gid) -> Option<f32>
let outlines = font.outline_glyphs();                  // outlines.get(gid) -> Option<OutlineGlyph>
```

**Outline → coverage.** Implement a small `OutlinePen` that collects `zeno::Command`s,
**negating y** (skrifa emits y-up font space; the raster is y-down):

```rust
#[derive(Default)]
struct ZenoPen(Vec<Command>);
impl OutlinePen for ZenoPen {
    fn move_to(&mut self, x: f32, y: f32)  { self.0.push(Command::MoveTo(Point::new(x, -y))); }
    fn line_to(&mut self, x: f32, y: f32)  { self.0.push(Command::LineTo(Point::new(x, -y))); }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.0.push(Command::QuadTo(Point::new(cx, -cy), Point::new(x, -y)));
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.0.push(Command::CurveTo(Point::new(a, -b), Point::new(c, -d), Point::new(x, -y)));
    }
    fn close(&mut self) { self.0.push(Command::Close); }
}
```

Then, per glyph:
```rust
let mut pen = ZenoPen::default();
outlines.get(gid)?.draw(DrawSettings::unhinted(size, loc), &mut pen).ok()?; // whitespace: 0 cmds
let (coverage, placement) = Mask::new(pen.0.as_slice()).render();
// placement: { left, top, width, height } — the y-down analog of ab_glyph px_bounds().
// coverage: Vec<u8>, row-major, len == width*height, values 0..=255.
```

**Keep the existing two-pass structure of `render_text`:**
1. **Pass 1 (layout):** walk `text.chars()`, `gid = charmap.map(ch).unwrap_or(GlyphId::new(0))`
   (notdef for unmapped, mirroring `ab_glyph`'s `glyph_id`). Render each glyph once, storing
   `(coverage, placement, pen_x)`. A glyph's absolute pixel origin is
   `x = pen_x + placement.left`, `y = ascent + placement.top` (`top` is negative for the part
   above the baseline). Accumulate the min/max of `[x, x+width) × [y, y+height)` over glyphs
   that produced coverage. Advance `pen_x += gmetrics.advance_width(gid).unwrap_or(0.0)`.
   **No kerning** (DEC-045: nil effect on the bundled font).
2. **No drawable glyphs** (all whitespace / empty) → return the existing
   `RgbaImage::from_pixel(1, 1, Rgba([0,0,0,0]))`.
3. **Pass 2 (composite):** size the canvas to `ceil(max-min)`; for each stored glyph, blit
   `coverage` at offset `(glyph_x - min_x, glyph_y - min_y)`, with
   `alpha = round((cov as f32 / 255.0) * color[3] as f32)`, and the **same** keep-the-larger-
   alpha source-over rule the current code uses (`if a >= cur.0[3] { put }`).

This preserves the exact compositing semantics; only the glyph source (ab_glyph → skrifa/
zeno) and the coverage encoding (0..1 float closure → 0..=255 byte buffer) change.

### Decisions that apply

- `DEC-045` — this swap: `skrifa`+`zeno`, drop `ttf-parser`, delete the `-0192` ignore;
  drop pairwise kerning. **Supersedes DEC-032's rasterizer choice.**
- `DEC-032` — retained: bundled Go-Regular font + the `include_bytes!` default; the
  `--font PATH` override. Only its `ab_glyph` rasterizer choice is superseded.
- `DEC-042` — the security assessment that accepted the `-0192` ignore this spec removes.
- `DEC-031` — the IO boundary: font *file* reads happen in `run_watermark`, never in
  `src/text`; the module stays pure (`font_bytes: &[u8]` in, `RgbaImage` out). Unchanged.
- `DEC-004` / `DEC-018` — pure-Rust, permissive-license policy; `skrifa`+`zeno` comply.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — satisfied: the new deps are recorded in
  DEC-045 (and a net -1/+2 top-level swap).
- `pure-rust-codecs-default` / `no-agpl-default-deps` — `skrifa`+`zeno` are pure-Rust and
  MIT/Apache; `just deny` (licenses) stays green.

### Prior related work

- `SPEC-030` (shipped) — introduced `src/text` and `watermark --text` on `ab_glyph`.
- `SPEC-029` (shipped) — the `Watermark` op / gravity compositing path this overlay feeds;
  **not touched** by this spec.

### Out of scope (for this spec specifically)

- Real text shaping, GPOS kerning, multi-line layout, alignment, or stroke — a future spec
  if wanted (would add `harfrust`/`rustybuzz`).
- Feature-gating the bundled font or the rasterizer for binary size — not now (DEC-045).
- The other STAGE-010 items (EXIF writer; `--help` jargon cleanup) — separate work.
- Any change to `parse_color`, `TextError` variants, or the `run_watermark` CLI surface.

## Notes for the Implementer

- **Behavior parity, not byte-identity.** A different rasterizer will not produce pixel-
  identical output; the bar is that the existing tests pass and text is legible at the same
  anchor. Do **not** try to match `ab_glyph`'s anti-aliasing exactly.
- `zeno::Mask::new(&commands).render()` auto-computes the tight `Placement` — you do **not**
  need to pre-size the mask; that's why per-glyph bounds come for free (the `px_bounds()`
  analog).
- Whitespace glyphs: `outlines.get(gid)` may return `Some` with a zero-command outline →
  `render()` yields a `0×0` placement / empty coverage. Guard on `width == 0 || height == 0`
  (or empty coverage) and treat as "advance only, no bounds".
- Keep the module's doc-comment accurate: update the `ab_glyph` references in the header
  (lines ~9–15) and `src/lib.rs` (~line 22) to `skrifa`/`zeno`.
- Run **both** builds: `cargo test`, `cargo clippy`, `cargo fmt --check`, `cargo build
  --no-default-features`, and `cargo deny check advisories bans sources licenses` — the
  last must pass **after** you delete the `-0192` entry (that's the whole point).
- `Cargo.lock` will change; commit it.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-044-skrifa-zeno`
- **PR (if applicable):** opened against `main` via `gh pr create` (see PR URL reported
  to the orchestrator).
- **All acceptance criteria met?** yes
- **New decisions emitted:** none (DEC-045 was authored at design time; build followed it
  as written, no deviation requiring a new/amended DEC).
- **Deviations from spec:**
  - None from the probe-verified API. One naming difference from the spec's illustrative
    snippet: `skrifa::instance::{LocationRef, Size}` were imported directly (not via a
    `skrifa::prelude` re-export) alongside `skrifa::{FontRef, GlyphId, MetadataProvider}` —
    same effective calls, just spelled out per `cargo doc`'s actual module layout.
  - The per-glyph compositing loop iterates the `Vec<u8>` coverage buffer in row-major
    order explicitly (`coverage[(gy*width+gx)]`) rather than a closure callback (zeno's
    `Mask::render()` returns a flat buffer, not a `draw(|x,y,cov| ..)` callback like
    `ab_glyph`'s `OutlineGlyph::draw`) — a mechanical consequence of the new API's shape,
    not a behavioral deviation.
- **Follow-up work identified:**
  - None beyond the stage's existing backlog (EXIF writer, `--help` cleanup — already
    called out as out-of-scope in the spec).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing significant. The spec's Implementation Context gave probe-verified calls
   that compiled essentially as written; the only extra step was confirming exact import
   paths (`skrifa::instance::{LocationRef, Size}` vs. a flatter re-export) by reading the
   crate source directly, since the spec's snippet used shorthand `use` paths.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-045's kerning-drop rationale and the two-pass layout/composite contract were
   sufficient; no ambiguity required an undocumented judgment call.

3. **If you did this task again, what would you do differently?**
   — Nothing procedurally different — the probe-first design made this a low-friction,
   mostly-mechanical swap. If anything, I'd note for future specs of this shape that
   pre-reading the exact crate source (not just docs.rs prose) for the pen/mask trait
   signatures paid off immediately and avoided any compile-fix iteration.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Nothing major. The high-leverage move was probing the *actual dep tree* before
   trusting the backlog's "fontdue drops ttf-parser" plan — that premise was wrong and
   would have shipped a rasterizer change for zero advisory benefit. Lesson generalized:
   for any "swap X to drop dep Y" item, run `cargo tree` on the candidate first. Also
   worth keeping: the visual old-vs-new pixel A/B closed the one gap that gates/tests
   couldn't (behavior parity is a claim about pixels, not just bounds/advance math).

2. **Does any template, constraint, or decision need updating?**
   — DEC-032's rasterizer choice is now superseded by DEC-045 (recorded in both). No
   template/constraint change. The backlog was corrected in-place with the fontdue
   dead-end lesson. `--help` still leaks `(STAGE-004)` etc. — already tracked as the
   STAGE-012 jargon-cleanup item (observed again during the visual A/B render).

3. **Is there a follow-up spec I should write now before I forget?**
   — The next STAGE-010 spec is the **in-house TIFF-IFD EXIF writer** (drop `little_exif`
   → kill RUSTSEC-2026-0194/-0195 + the `paste` -2024-0436 chain), already in the stage
   backlog. No new spec surfaced from this one.
