# SPEC-044 — BUILD cycle prompt (Sonnet, prescriptive)

You are the **build** implementer for **SPEC-044**: swap the `watermark --text` glyph
rasterizer from `ab_glyph` to **`skrifa` + `zeno`**, dropping the unmaintained
`ttf-parser` and removing the `RUSTSEC-2026-0192` `deny.toml` ignore. This is a
**behavior-preserving dependency swap** — the public API and user-visible output do not
change.

## Where to work

- **Worktree (do ALL work here):**
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg-wt-spec044`
- **Branch (already checked out there):** `feat/spec-044-skrifa-zeno` (based on `main`).
- Do **not** touch the main checkout. Verify with `git -C <worktree> rev-parse --abbrev-ref HEAD`
  before you start — it must print `feat/spec-044-skrifa-zeno`.

## Read first (in the worktree)

1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-044-swap-ab-glyph-for-skrifa-zeno-to-drop-ttf-parser.md`
   — the spec. Its **## Failing Tests** and **## Implementation Context** are your
   contract; the latter has the **probe-verified** skrifa+zeno calls. Build on them; do
   not re-derive the API.
2. `decisions/DEC-045-text-watermark-rasterizer-skrifa-zeno.md` — the decision + kerning
   rationale.
3. `src/text/mod.rs` — the module you are rewriting.

## What to change

1. **`Cargo.toml`** — remove the `ab_glyph = "=0.2.32"` dependency and its comment block
   (the `ab_glyph`/`ttf-parser` `std`-feature note). Add:
   - `skrifa = "=0.44.0"`
   - `zeno = "=0.3.3"`
   Write a short comment justifying them (pure-Rust, MIT/Apache, `ttf-parser`-free,
   `fontations` project; the rasterizer for `watermark --text`, DEC-045). Keep the deps
   alphabetically/logically grouped as the file already does.

2. **`src/text/mod.rs`** — re-implement `render_text`'s internals on skrifa+zeno. Keep the
   **exact** public signatures of `render_text`, `parse_color`, `DEFAULT_FONT`, and the
   `TextError` enum. Preserve the two-pass layout→composite structure and the
   source-over keep-the-larger-alpha compositing rule. Use the pen/negate-y/placement
   approach spelled out in the spec's Implementation Context. Notes:
   - Parse: `FontRef::new(font_bytes).map_err(|e| TextError::Font(e.to_string()))?`.
   - Unmapped char → notdef: `charmap.map(ch).unwrap_or(GlyphId::new(0))`.
   - Whitespace/empty outline → advance only, no bounds (guard on `width==0 || height==0`).
   - All-whitespace / no drawable glyphs → return `RgbaImage::from_pixel(1,1,Rgba([0,0,0,0]))`.
   - Alpha: `((cov as f32 / 255.0) * color[3] as f32).round()`, clamped to 255.
   - **No kerning** (DEC-045).
   - Update the module doc-comment (lines ~9–15) — replace `ab_glyph` references with
     `skrifa`/`zeno`.

3. **`src/lib.rs`** — update the `src/text` reference (~line 22) from `ab_glyph` to
   `skrifa`/`zeno`.

4. **`deny.toml`** — delete the `RUSTSEC-2026-0192` ignore entry **and** its explanatory
   comment block. Leave the `-0194`/`-0195`/`-2024-0436` entries untouched (those are the
   next spec's job).

5. **Tests** — add the four new tests from the spec's **## Failing Tests** to the
   `#[cfg(test)] mod tests` in `src/text/mod.rs`, and keep the existing six passing:
   - `render_text_accumulates_advance`
   - `render_text_whitespace_contributes_advance`
   - `render_text_all_whitespace_is_1x1`
   - `render_text_height_tracks_font_size`

## Gates (all must pass, in the worktree)

Run and make green — paste the results into the Build Completion section:

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --no-default-features        # the lean/headless build
cargo deny check advisories bans sources licenses
cargo tree | grep -c ttf-parser          # MUST be 0
cargo tree | grep -c ab_glyph            # MUST be 0
```

The `deny` check must pass **with the `-0192` entry removed** — that is the whole point.
If it flags something unexpected, stop and report; do **not** re-add an ignore.

After `cargo fmt`, re-stage **all** files fmt touched (`git add -u`) before committing —
a later fmt reformatting an already-committed file breaks CI even when local
`cargo fmt --check` passes.

## Finish

1. Fill in the spec's **## Build Completion** section (branch, acceptance criteria met,
   any deviations, the 3 build-reflection answers) and flip the timeline `build` line to
   `[x]` with a one-line summary.
2. Commit on `feat/spec-044-skrifa-zeno` (include `Cargo.lock`). Message:
   `fix(SPEC-044): rasterize watermark text on skrifa+zeno; drop ttf-parser (-0192)`
   ending with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.
3. Push the branch and open a PR against `main` with `gh pr create`. PR body: what changed,
   the gate results (esp. `cargo tree` ttf-parser=0 and `deny` green), and "behavior-
   preserving; see SPEC-044 / DEC-045". End the body with the Claude Code attribution line.
4. **Do NOT merge.** Report the PR number/URL and the gate output back to me.

## Guardrails

- Behavior parity, **not** byte-identity — do not chase matching ab_glyph's anti-aliasing.
- Do not change `parse_color`, `TextError` variants, the bundled font, or `run_watermark`.
- Do not expand scope to the EXIF writer, `--help` cleanup, or dep bumps — those are
  separate STAGE-010/011/012 work.
- If a probe-verified call doesn't compile as written, prefer the nearest skrifa 0.44 /
  zeno 0.3.3 equivalent and note the deviation — do not swap crates.
