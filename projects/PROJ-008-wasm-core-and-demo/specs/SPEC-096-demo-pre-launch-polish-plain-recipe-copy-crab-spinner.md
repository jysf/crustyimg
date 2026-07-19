---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-096
  type: chore
  cycle: design
  blocked: false
  priority: medium
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-18

references:
  decisions: [DEC-070]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-080, SPEC-081, SPEC-085]

value_link: >
  Pre-launch polish so the demo and the recipes a visitor copies read like a human wrote them,
  and the busy state doesn't imply "stuck". Cheap credibility before Show HN.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-096: demo pre-launch polish — plain recipe copy + crab spinner

## Context

Two small, user-facing warts surfaced while prepping for launch:

1. **The bundled recipe headers read like AI wrote them and leak internal traceability.** The header
   comment in `recipes/web.toml` (and `gallery.toml` / `product.toml`) carries `SPEC-085`, `SPEC-084`,
   `SPEC-090`, `DEC-075`, and internal symbol names (`Mode::Fast`, `larger_than_source`), plus a
   kitchen-sink "everything about it" style. These headers are **user-facing**: they ship in-binary via
   `include_str!` (`src/recipe/bundled.rs:42`) and the `web` one is shown **verbatim in the demo's CLI
   adoption funnel** (`WEB_RECIPE` in `demo/demo.js`, pinned byte-identical by `tests/demo_smoke.mjs`),
   so a visitor literally copies this TOML into their own repo. Internal `src/` traceability comments are
   fine and out of scope — this is only the shipped recipe headers.

2. **The demo's busy indicator is a spinning glyph that reads as "stuck".** `<span class="spinner">`
   (`demo/index.html:82`, `@keyframes spin` in `demo/demo.css`) spins next to "Making it web-ready in a
   background thread — the page stays responsive." The maintainer wants it replaced with a **static crab
   emoji (🦀)** as a placeholder until a real logo/image exists.

Behavior is unchanged in both cases — recipe **steps** are untouched (TOML comments are ignored by the
parser, so recipe output is byte-identical), and the busy state is purely presentational.

## Goal

Make the shipped recipe headers plain, behavior-first prose (no spec/DEC refs, no internal symbol
names), keeping `web.toml` and the demo's `WEB_RECIPE` byte-identical; and replace the spinning busy
glyph with a static 🦀 placeholder. No engine or recipe-behavior change.

## Inputs

- **Files to read:** `recipes/web.toml`, `recipes/gallery.toml`, `recipes/product.toml`;
  `demo/demo.js` (the `WEB_RECIPE` constant, ~line 29); `demo/index.html` (busy block ~line 76–85);
  `demo/demo.css` (`.spinner` / `@keyframes spin`, ~line 195–231); `tests/demo_smoke.mjs` (the verbatim
  assertion ~line 718); `src/recipe/bundled.rs` (`include_str!` sites).
- **Related code paths:** the recipe/CLI tests in `tests/cli.rs` and `src/recipe/`.

## Outputs

- **Files modified:**
  - `recipes/web.toml` — replace the header comment with the plain copy below. `version`/`name`/
    `description` and all `[[step]]` blocks UNCHANGED.
  - `recipes/gallery.toml`, `recipes/product.toml` — same treatment, plain copy below.
  - `demo/demo.js` — update `WEB_RECIPE` to match the new `recipes/web.toml` **verbatim** (byte-identical,
    the `tests/demo_smoke.mjs` pin).
  - `demo/index.html` — replace the spinning `<span class="spinner">` with a static 🦀 (keep the
    surrounding text + `aria-hidden`/accessibility intent).
  - `demo/demo.css` — remove/neutralize the `spin` animation for the busy glyph (the crab does not spin);
    keep the reduced-motion handling coherent.
  - `tests/demo_smoke.mjs` and/or `tests/cli.rs` — the guard tests below.

### The plain recipe copy (use verbatim)

`recipes/web.toml`:
```
# Prepare an image for the web: resize the long edge down to 2048px
# (never upscaling) and re-encode to the smallest modern format that
# beats the resized image — AVIF for photos, lossless WebP for graphics.
#
# Because it resizes, a source already under that size can come out
# larger than the original; when it does, the new size is reported
# plainly, not hidden. To keep the original dimensions and never grow
# the file, use `optimize` instead.
```

`recipes/gallery.toml`:
```
# Prepare gallery and lightbox images: like `web`, but resizes the long
# edge to 2560px for full-bleed display, then re-encodes to the smallest
# modern format. A source already under that size can come out larger
# than the original; the new size is reported plainly.
```

`recipes/product.toml`:
```
# Prepare product-card and catalogue images: like `web`, but resizes the
# long edge to 1600px, then re-encodes to the smallest modern format. A
# source already under that size can come out larger than the original;
# the new size is reported plainly.
```

## Acceptance Criteria

- [ ] The header comment of every bundled recipe (`recipes/web.toml`, `gallery.toml`, `product.toml`)
      contains **no `SPEC-` or `DEC-` reference and no internal symbol name** (`Mode::Fast`,
      `larger_than_source`), and reads as plain behavior-first prose.
- [ ] Recipe **behavior is byte-identical**: `version`/`name`/`description`/`[[step]]` blocks unchanged;
      running `web`/`gallery`/`product` on a fixture produces the same output as before this spec.
- [ ] `demo/demo.js` `WEB_RECIPE` is **byte-identical** to `recipes/web.toml` — `tests/demo_smoke.mjs`'s
      verbatim assertion stays green.
- [ ] The demo busy indicator shows a **static 🦀** (no spin animation); the "Making it web-ready…" text
      is unchanged and the layout is not broken.
- [ ] All existing recipe/CLI tests and the browser smoke pass; **zero `src/` engine-logic change**
      (only `recipes/*.toml` comment bytes + demo files + tests).

## Failing Tests

- **`tests/cli.rs`** (or a small unit test near `src/recipe/bundled.rs`)
  - `"bundled_recipe_headers_are_plain"` — for each bundled recipe TOML, assert its comment lines contain
    no `"SPEC-"`, no `"DEC-"`, and none of the internal symbol strings. A mechanical guard so the copy
    can't regress (matches the "cite a mechanical check" discipline).
- **`tests/cli.rs`**
  - `"bundled_recipe_behavior_unchanged"` — assert the parsed recipe (`version`/`name`/steps) for `web`
    is exactly as expected — proves the header rewrite didn't touch behavior. (If an equivalent assertion
    already exists, extend it rather than duplicate.)
- **`tests/demo_smoke.mjs`**
  - the existing verbatim `WEB_RECIPE == recipes/web.toml` assertion must stay green after the rewrite.
  - `"busy_indicator_is_a_static_crab"` — drive a conversion; assert the busy element renders 🦀 and has
    **no running CSS animation** (e.g. the `.spinner` spin class is gone / `animation: none`).

## Implementation Context

### Decisions that apply
- `DEC-070` — the bundled-recipe registry + `web == apply --recipe web`; the recipe TOMLs are the shipped
  source of truth (`include_str!`). Change the comment bytes only; keep the registry/behavior intact.

### Constraints that apply
- `ergonomic-defaults` — user-facing text must be readable by a non-expert; plain over comprehensive.

### Prior related work
- `SPEC-080` (shipped) — the demo funnel that shows `WEB_RECIPE` verbatim + the "copy to CLI" story.
- `SPEC-081` (shipped) — the most recent demo change; `demo/*` is current.
- `SPEC-085` (shipped) — introduced the bundled recipes whose headers this cleans up.

### Out of scope
- Any `src/` engine or recipe-parsing logic change (comments only in the TOMLs).
- Internal `src/` traceability comments (the `// SPEC-090 … (DEC-075)` style is an accepted convention).
- A real logo/graphic — the 🦀 is an explicit placeholder the maintainer will replace later.
- README / BENCHMARKS copy (STAGE-028, separate).

## Notes for the Implementer
- **Sync is load-bearing.** `tests/demo_smoke.mjs` reads `recipes/web.toml` and asserts `WEB_RECIPE`
  equals it byte-for-byte. Update both to the identical text (mind trailing newline/whitespace).
- **Keep the crab a dumb placeholder** — a static glyph, accessible label preserved; don't build an
  animated mascot. It gets replaced.
- The reduced-motion `@media (prefers-reduced-motion)` block around the spinner (demo.css ~230) should
  stay coherent once the animation is gone — a static crab already satisfies reduced-motion.

---

## Build Completion
- **Branch:** `spec-096-demo-polish`
- **PR (if applicable):** none (build session only, no PR/merge per instructions)
- **All acceptance criteria met?** yes
  - Header comments in `recipes/{web,gallery,product}.toml` are the verbatim plain copy from the spec — no `SPEC-`/`DEC-` refs, no `Mode::Fast`/`larger_than_source`. Mechanically guarded by `bundled_recipe_headers_are_plain`.
  - Recipe behavior byte-identical: `version`/`name`/`description`/`[[step]]` untouched in all three files; `bundled_recipe_behavior_unchanged` pins the parsed `web` recipe exactly.
  - `demo/demo.js`'s `WEB_RECIPE` matches `recipes/web.toml` byte-for-byte (verified via `eval`-based comparison, not just string diff, to account for the backtick-escaping in the template literal); `tests/demo_smoke.mjs`'s existing verbatim assertion is green (690 B).
  - Busy indicator is a static 🦀 (`demo/index.html:82`, `#busy-icon`), `.busy-icon { animation: none; }` in `demo/demo.css` — no spin keyframes remain. `busy_indicator_is_a_static_crab` confirms both the glyph and `getComputedStyle(...).animationName === "none"`.
  - Full Rust suite (`cargo test --features avif`): 434+ tests, 0 failed. `cargo clippy --all-targets --features avif -- -D warnings`: clean. `cargo fmt --check`: clean. `just demo-smoke` (real headless Chrome, over HTTP): all checks green, including the new crab test, mid a 5.4 s full-resolution encode. Zero `src/` engine-logic changes — only `recipes/*.toml` comment bytes, `demo/*`, `src/recipe/bundled.rs` (tests only), and `tests/demo_smoke.mjs`.
- **New decisions emitted:** none.
- **Deviations from spec:**
  - Both new Rust guard tests landed in `src/recipe/bundled.rs`'s existing `#[cfg(test)] mod tests` (next to `bundled_recipe_names_resolve`) rather than `tests/cli.rs` — the spec offered this as the first option ("`tests/cli.rs` (or a small unit test near `src/recipe/bundled.rs`)") and it's the more local, already-established home for bundled-recipe assertions.
  - Also fixed two comments outside the spec's explicit file list that referenced the removed spinner and would otherwise have gone stale: the busy-block comment in `demo/index.html` ("alongside the spinner" → "alongside the crab") and one line in `demo/demo.js` ("calm spinner" → "no MP warning"). Both are demo-file prose, not `src/` engine logic, and leaving them would have shipped a comment describing behavior that no longer exists.
  - `busy_indicator_is_a_static_crab` piggybacks on the existing `advanced_full_resolution_shows_timer` mid-encode window (the one place `state === 'converting'` is reliably observable) rather than opening a new drop — cheaper and the spec's Inputs section already pointed at that same busy block.
- **Follow-up work identified:** none — the 🦀 is an explicit placeholder per the spec, to be swapped for a real logo/image later (not this spec's job).

### Build-phase reflection (3 questions, short answers)
1. **What was unclear in the spec that slowed you down?** — Nothing major. The one thing I had to work out myself: comparing `WEB_RECIPE` to `recipes/web.toml` by regexing the raw JS source text gives a false mismatch (the backtick-escape `\`` in the source isn't the same character sequence as the evaluated string's literal backtick) — had to eval the template literal properly to get a trustworthy byte-identity check.
2. **Was there a constraint or decision that should have been listed but wasn't?** — No — DEC-070 was the right and sufficient reference; nothing else needed pulling in.
3. **If you did this task again, what would you do differently?** — Same approach. Running the mutation/negative-control checks (inject a `SPEC-` string, widen the resize, re-add a spin animation) before calling the tests done was worth the extra `just demo-smoke` cycle — it's the cheapest way to know a guard test actually guards something.

---

## Reflection (Ship)
1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
