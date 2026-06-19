# SPEC-030 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-030-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the LAST STAGE-004 command (text mode for
  `watermark`). Emitted **DEC-032** (`ab_glyph` rasterizer + bundled BSD-3 Go font via
  `include_bytes!`, NOT `imageproc` — it pulls sdl2/nalgebra). Two design-time probes:
  ab_glyph laid out + rasterized the Go font (with the `std`-feature trap found and
  documented), and imageproc's dep-tree (sdl2) ruled it out. Added `ab_glyph =0.2.32`
  + `assets/fonts/Go-Regular.ttf` (+ LICENSE) — `just deny` + lean build green. Design
  = render text → RGBA overlay → reuse the SPEC-029 `Watermark` compositing. Design +
  DEC-032 + font asset pushed to `main` before build.
- [x] build (2026-06-18, PR #34) — foreground metered subagent (Opus, 122k tok,
  ~12 min). Added `src/text/mod.rs` (`render_text`/`parse_color`/`TextError`/
  `DEFAULT_FONT`, ab_glyph, no file IO) + `watermark --text` mode (clap `--image` XOR
  `--text`; renders to an RGBA overlay reused through the SPEC-029 `Watermark` op).
  14 new tests (6 unit + 8 integration). No imageproc, no new DEC, lean build green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED,
  no concerns. Confirmed no imageproc/sdl2/nalgebra, the ab_glyph `std`-trap avoided
  (lean build compiles), no file IO in `src/text`/`src/operation`, full SPEC-029
  compositing reuse, render/color/size/exit-code correctness, image mode still
  passing. Orchestrator re-ran gates: `cargo test` 357 ok (0 failed), clippy/fmt/
  deny clean, `cargo build --no-default-features` (lean) clean.
- [x] ship (2026-06-18, PR #34 squash-merged → `db642e4`) — reflections + cost
  totals filled (build 121999 real / verify ~50k est; totals 171999 / $1.55 / 4),
  STAGE-004 backlog flipped + **stage shipped** (5/5), archived to `specs/done/`,
  `just cost-audit` green + cost-capture + lean build confirmed on main CI.
