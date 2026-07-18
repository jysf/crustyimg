---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-080
  type: story
  cycle: build
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-13

references:
  decisions: [DEC-020, DEC-064, DEC-065, DEC-068, DEC-069, DEC-070]
  constraints: [untrusted-input-hardening, ergonomic-defaults]
  related_specs: [SPEC-077, SPEC-078, SPEC-079, SPEC-085]

value_link: >
  Make the demo BE the flagship. The live demo is a squoosh-style "pick a format" tool that defaults
  to lossless-WebP (which makes a photo BIGGER). Reframe it to the shipped `web` flow as one opinionated
  action — drop a photo, it downscales + modernizes to AVIF + never ships bigger + scores the result,
  in seconds — and turn every conversion into an adoption moment by showing the exact `crustyimg web`
  command that does it. The demo's job isn't to convert one image; it's to convert a visitor into a user.

cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 450000
      estimated_usd: 4.05
      recorded_at: 2026-07-18
      note: >
        Main-loop build (not a separately-metered subagent), so tokens_total is an
        order-of-magnitude ESTIMATE ([[autonomous-run-cost-estimates]]); estimated_usd
        at Opus 4.8 list ($5/$25, ~80/20 in/out, no cache discount). Demo-only reframe:
        read the whole demo + the SPEC-079 wasm surface + the analysis classifier;
        design-time probes against the vendored wasm to pin the photo→AVIF and
        never-bigger fixtures ([[probe-load-bearing-crates-at-design]]); a full demo
        rewrite (index/js/worker/css) + a rewritten headless-Chrome smoke; one wasm
        build and several end-to-end smoke runs in a real browser.
  totals:
    tokens_total: 450000
    estimated_usd: 4.05
    session_count: 1
---

# SPEC-080: demo = the `web` flow, one-click, with a CLI adoption funnel

## Context

STAGE-029's measured investigation found the live demo mis-serves its most common visitor, and the
STAGE-030 taxonomy reconciliation settled what the demo hero should BE: **the `web` flow** (SPEC-085) —
the flagship verb, made tangible in the browser. This reframes the demo around that, on top of the
surface **SPEC-079** shipped.

**Product decisions (maintainer, 2026-07-18):**
1. **One-click `web` hero; advanced controls hidden.** Drop → the `web` flow runs automatically, no
   choices required. The old format picker / max-edge / quality controls move behind an **"Advanced"**
   disclosure, collapsed by default. Opinionated "it just works" — beats squoosh by *not* making you
   choose, and matches the CLI's flagship.
2. **First-class CLI adoption funnel.** After each conversion, prominently show the exact command
   (`crustyimg web <file>`) + the `web` recipe + a copy button. The demo deliberately converts curiosity
   into installs — its strategic reason to exist.

**Why the live demo is wrong today (measured, STAGE-029):**
- Default output is **lossless WebP** → makes an already-lossy photo *bigger* (the opposite of intent).
- The busy state is a bare spinner; AVIF at full resolution + speed 6 was ~33 s on a 12 MP photo → reads
  as a hang.
- No "keep the original" path when nothing beats the source.

**The reframe dissolves most of that:** the `web` flow **downscales the long edge to 2048 by default**,
so the encode operates on ~2 MP, not 12 — a 2048px AVIF at **speed 10** is ~1–2 s, not 33 s. The
perf-UX machinery (timer, megapixel warning, debounce) survives but moves to a **fallback for the
Advanced "keep full resolution" path**; the default hero is simply fast.

It is **demo files only** (`demo/index.html`, `demo/demo.js`, `demo/worker.js`, `demo/demo.css`, and the
vendored recipe text) — **no engine/wasm change** (SPEC-079 owns the surface; SPEC-081 owns the rich
score/diff UI).

## Goal

Reframe the demo so its **default, zero-choice action is the `web` flow** — downscale long-edge to 2048,
Auto-modernize (AVIF for photos / lossless-WebP for graphics via SPEC-079), at speed 10, never shipping a
file bigger than the original, and reporting the SSIMULACRA2 score — and **turn each result into a CLI
adoption moment** (the `crustyimg web` command + recipe + copy). Move all format/resolution/quality
controls behind a collapsed **Advanced** disclosure; keep the honest perf UX as the Advanced-path fallback.

## Inputs

- **Files to read:** `demo/demo.js`, `demo/worker.js`, `demo/index.html`, `demo/demo.css` (the whole
  demo — small); `demo/README.md` (the candor to preserve).
- **The SPEC-079 surface (its dependency, reconcile against SHIPPED names):** `optimizeDetailed(input,
  out_format, speed?, maxBytes?, target?) → OptimizeResult { bytes, format, quality, speed, score,
  scoredBy }` and the `score(a, b)` binding. Note: `optimizeDetailed` does **not** resize — the demo
  performs the **downscale itself** (the worker already has the decoded bitmap; resize via
  canvas/`createImageBitmap` before encoding), then calls `optimizeDetailed` on the 2048px image.
- **`recipes/web.toml`** — the actual `web` recipe, shown verbatim in the funnel (vendor its text into
  the demo, or read it; it must match the shipped recipe).
- **SPEC-085 / `web`** — what the flagship flow *is* (downscale 2048 → content-modernize → never-bigger
  → strip → orient → score), so the demo's approximation is honest about what it mirrors.

## Outputs

- **`demo/worker.js`** — downscale the decoded image to a **2048px long edge by default** (skip if already
  ≤ 2048) before encoding; call `optimizeDetailed` (not `optimize`) with **speed 10** for AVIF; return the
  richer result (format / quality / speed / score / scoredBy) + the pre/post dimensions.
- **`demo/demo.js`** — the hero flow + the funnel:
  - **Default action = `web`:** on drop, run downscale-2048 → Auto-modernize → never-bigger, no controls
    touched. State the downscale honestly in the result ("resized to 2048px for web").
  - **Never-bigger:** if the result ≥ the input, hand back the **original** for download, labeled
    "already optimized — kept your file" (pure page logic; the page has both byte counts).
  - **CLI adoption funnel:** after each conversion, render a prominent block — the exact command
    `crustyimg web <original-filename>` with a **copy button**, the `web` recipe (collapsible, verbatim
    from `web.toml`), and a short honest line that the demo *approximates* `web` in-browser while the CLI
    runs the real thing on whole folders (`crustyimg web *.jpg`). Include an install pointer.
  - **Score readout (minimal):** show the returned SSIMULACRA2 score as a value in the result (SPEC-081
    owns the rich diff UI). Handle the shipped gotchas: `score` is **raw SSIMULACRA2, not 0–100** and
    **can be negative** — render it honestly, never assume a 0–100 range; if `maxBytes` is ever exercised
    (Advanced), self-check `bytes.length` because an unsatisfiable budget returns over-budget bytes
    silently (SPEC-079 note).
  - **Advanced disclosure (collapsed):** format override (incl. the old picker), a **max-edge** control
    (incl. "keep full resolution" = no downscale), quality/maxBytes. Only when a user opts into a slow
    path (full resolution / a big encode) do the **megapixel warning + live elapsed timer + debounce**
    apply — carry them from the current spec as the Advanced-path fallback.
- **`demo/index.html`** — recast around the one-click hero: a drop zone + a single implicit "make it
  web-ready" result, with Advanced as a collapsed `<details>`; the funnel block in the result area.
- **`demo/demo.css`** — styling for the funnel (command + copy + collapsible recipe), the "kept original"
  state, the Advanced disclosure, and the timer/warning (Advanced-path).

## Acceptance Criteria

- [ ] **Default is the `web` flow:** dropping a **photo** with **no controls touched** downscales the long
      edge to ~2048px, produces a **smaller AVIF** (via SPEC-079 Auto), and downloads with the right
      extension — no format choice required, and never a bigger lossless file.
- [ ] The default hero on a typical (e.g. 12 MP) photo completes in **a few seconds** (downscale-2048 +
      speed 10), not tens of seconds — the perf problem is gone by construction, not by a spinner.
- [ ] **Never-bigger:** when the best result ≥ the input, the UI shows a **"kept your original"** state and
      the download hands back the **original bytes**, with an honest one-line reason.
- [ ] **CLI adoption funnel:** every successful conversion shows the exact `crustyimg web <file>` command
      with a working **copy-to-clipboard** button, the `web` recipe (verbatim from `web.toml`), and the
      honest "the CLI runs this on whole folders" framing. The command reflects the dropped file's name.
- [ ] **Advanced is collapsed by default** and contains the format / max-edge (incl. keep-full-resolution)
      / quality controls; the hero works without ever opening it.
- [ ] **Advanced-path perf UX:** choosing "keep full resolution" (or an otherwise slow encode) shows the
      **megapixel warning + a counting-up elapsed timer** (honest, no fake %); the page stays responsive
      (the SPEC-078 Web-Worker guarantee holds); rapid control changes **debounce/supersede** (newest
      wins, no stacked jobs).
- [ ] The score is shown as an **honest raw value** (handles negatives; not assumed 0–100).
- [ ] The browser smoke (`just demo-smoke` / the SPEC-077/078 headless-Chrome driver) passes end-to-end
      (drop → convert → download, **zero network requests**), updated for the new hero + funnel; hostile/
      edge inputs still surface a clean error, no hang.

## Failing Tests

Written at design; the demo's earned verdict is **browser-driven** (SPEC-077/078 precedent), not units.

- **Headless-Chrome demo smoke (extend the SPEC-077/078 driver)**
  - `"default_is_web_flow_smaller_avif"` — drop a large photographic PNG on defaults (no controls touched);
    assert the result is `format == "avif"`, `outBytes < inBytes`, **and the output long edge ≤ ~2048**
    (the downscale ran).
  - `"never_bigger_keeps_original"` — drive an input the engine can't beat; assert the "kept original"
    state and that the download `blob` byte length == the input's.
  - `"funnel_shows_web_command_and_copies"` — after a conversion, assert the funnel renders
    `crustyimg web <filename>` and the copy button writes that exact string to the clipboard; assert the
    `web.toml` recipe text is present and matches the vendored recipe.
  - `"advanced_full_resolution_shows_timer"` — open Advanced, pick keep-full-resolution on a > ~6 MP input;
    assert the megapixel warning + an **increasing** elapsed-time element appear and the page stays
    interactive (a control remains clickable — the 078 negative control).
- **Manual/verify (documented, driven at verify):** the default hero is visibly fast on a real 12 MP
  photo (downscale-2048 + speed 10); the funnel command copies correctly; the never-bigger path is honest
  on an already-optimized JPEG; the score renders sanely on a low-quality input (negative value handled).

## Implementation Context

### Decisions that apply
- `DEC-070` (SPEC-085) — the `web` flow this mirrors (downscale 2048 → content-modernize → never-bigger →
  score). The demo **approximates** it via the wasm surface + a page-side downscale; be honest that the
  CLI is the real thing.
- `DEC-068` (SPEC-079) — the `optimizeDetailed`/`score` surface the demo consumes (it does **not** resize;
  the demo downscales itself).
- `DEC-069` — native(`web` q85) vs wasm(q80) AVIF-quality divergence exists; don't claim the in-browser
  result is byte-identical to the CLI — "approximates."
- `DEC-064`/`DEC-065` — the wasm cfg boundary + AVIF-encode-not-decode asymmetry; `.avif` inputs decode
  via `createImageBitmap` (SPEC-078). Reuse that seam; add nothing to the wasm.
- `DEC-020` — rav1e speed; the demo passes **10** through SPEC-079's knob (the CLI stays 6).

### Constraints that apply
- `untrusted-input-hardening` — hostile/huge inputs surface a clean typed error already (SPEC-078); keep
  that with the new hero + Advanced path — no hangs, no cryptic failures.
- `ergonomic-defaults` — the default must be what a photo-dropper wants (web-ready + smaller), choices
  honest (never silently bigger; the downscale is stated, not hidden).

### Prior related work
- `SPEC-077`/`SPEC-078` — the page/worker/`.avif`-input seam/browser smoke this reshapes.
- `SPEC-079` (hard dependency, shipped) — the `optimizeDetailed`/`score` surface. Reconcile names against
  the shipped surface.
- `SPEC-085` (shipped) — the `web` verb + `web.toml` the demo mirrors and teaches.

### Out of scope (for this spec specifically)
- Any `src/`/wasm change (SPEC-079 owns the surface; the demo downscales page-side).
- The **rich** SSIMULACRA2 score/diff UI (SPEC-081) — this spec shows the score as a value; 081 adds the
  visual input↔output diff/gauge.
- A **multi-recipe showcase** (offering gallery/product recipes as demo modes) — a compelling *future*
  enhancement (teaches "crustyimg is a recipe engine"), but it would push this past M; frame its own spec
  if wanted. This spec's funnel is `web` only.
- Mobile-specific layout beyond "must not break" (the real-device test is a STAGE-028 human task).

## Notes for the Implementer
- **Keep the demo thin.** A marketing artifact, not a web app — recast controls, no framework, no pile of
  options. The hero is *one path*; Advanced is a `<details>`.
- **Downscale is page-side and honest.** `optimizeDetailed` doesn't resize; the worker resizes the decoded
  bitmap to a 2048px long edge (skip if already smaller) before encoding, and the UI *says* it did.
- **Never-bigger is pure page logic** (`output.bytes >= input.bytes` → switch download to the original).
- **The funnel is the point, not a footer.** Command + copy + recipe, prominent in the result. Honest:
  "approximates `web`; the CLI runs the real thing on whole folders."
- **Timer honesty:** count elapsed seconds; never fake a percentage (one blocking rav1e call reports
  nothing — the SPEC-078 rationale stands).
- **Carry the candor** from `demo/README.md` (WebP lossless-only, AVIF encode-not-decode). HN rewards it.
- **Score gotchas (SPEC-079, shipped):** raw SSIMULACRA2, can be **negative**; unsatisfiable `maxBytes`
  returns over-budget bytes **silently** — render honestly / self-check bytes if Advanced exposes a budget.

---

## Build Completion
- **Branch:** `spec-080-demo-web-hero`
- **PR (if applicable):** (opened against `main`)
- **All acceptance criteria met?** Yes — all seven, driven end-to-end in headless Chrome:
  - **Default = `web` flow:** dropping a 2200×1650 photo with no controls touched → AVIF, 10.4 MB → 2.0 MB (smaller), downscaled to 2048×1536 (long edge ≤ 2048), download `photo.avif` from a `blob:` URL. Auto chose AVIF (photo bucket); the page states the downscale.
  - **Fast by construction:** the hero downscales to 2048 then encodes at speed 10 — a few seconds, and it is the calm-spinner path (no timer), because the perf problem is gone, not hidden.
  - **Never-bigger:** a moderate-quality 42 KB JPEG the engine's best re-encode can't beat → "kept your file" state; the download is the **original bytes, byte-for-byte** (41952 B == 41952 B), original filename.
  - **Funnel:** every result shows `crustyimg web <file>` reflecting the dropped name, a working copy button (clipboard read back byte-for-byte), the `web.toml` recipe **byte-identical** to disk (verbatim, 1108 B), the "runs on whole folders" line + an install pointer.
  - **Advanced collapsed by default** with format / max-edge / keep-full-resolution / byte-budget; the hero works without opening it.
  - **Advanced-path perf UX:** keep-full-resolution on a 6.3 MP photo → megapixel warning + a **counting-up** elapsed timer (0.0s → 0.7s during the ~5.1 s encode); the main thread ran 268 timer + 642 rAF callbacks during it, and the negative control (a deliberate 400 ms freeze) reads 0/0.
  - **Score honest:** raw SSIMULACRA2, negatives handled, never assumed 0–100; AVIF/lossless say *why* they're unscored rather than showing a blank.
  - **Zero network + clean errors:** the smoke asserts 0 network requests during every conversion (worker traffic included), nothing off-origin, and the file:// failure mode still speaks.
- **New decisions emitted:** None. This spec consumes DEC-068/069/070/020/064/065; it adds no wasm surface and no new dependency, so no DEC is warranted.
- **Deviations from spec:**
  1. The downscale runs via the engine's own `transform` (auto-orient + `resize max`, the same recipe TOML + `fast_image_resize` the CLI's `web` uses) rather than the spec's suggested `canvas`/`createImageBitmap` resample. This is a **more faithful** mirror of `web` (same resampler, same order) and reuses proven machinery; `createImageBitmap` is still used for `.avif` **input** decode and for reading AVIF output dimensions back. Net-honest, and it's what "the demo *approximates* `web`" should mean at the pixel level.
  2. Advanced exposes a byte-budget (`maxBytes`) as the "quality/maxBytes" knob rather than a raw quality slider — the shipped wasm surface takes `target`/`maxBytes`, not a quality argument (DEC-064); the budget exercises the size search and the unsatisfiable-budget self-check (SPEC-079 note).
  3. The funnel command is `crustyimg web <name>` unquoted (matches the exact-string smoke assertion); filenames with spaces would need quoting — noted as a possible polish, out of scope here.
- **Follow-up work identified:** SPEC-081 (the rich SSIMULACRA2 diff/gauge UI — this spec shows the score as a value and honestly says when it's unscored, leaving the browser-decode-and-`score()` for AVIF to 081); a multi-recipe showcase (gallery/product modes) remains a future spec; mobile real-device test stays a STAGE-028 human task.
### Build-phase reflection (3 questions, short answers)
1. **What was unclear in the spec that slowed you down?** — Whether the 2048 downscale should be a page-side `canvas` resample (Inputs/Notes wording) or the engine's own `resize` recipe. I chose the latter as the more faithful `web` mirror; a one-line "use the engine's resize for pixel-faithfulness" would have removed the ambiguity. Also, which formats actually yield an engine `score` (only JPEG) wasn't obvious until I re-read the surface — the hero's AVIF is always `scoredBy: "none"`, so "show the score" mostly means "honestly say why there isn't one".
2. **Was there a constraint or decision that should have been listed but wasn't?** — No missing constraint. Worth flagging for the next demo spec: the "synthetic math is not a photograph" lesson bit the fixture design — a gradient routes lossless, so the AVIF hero test needs a genuinely photographic fixture (I added `makePhotoPng`); a design-time probe against the vendored wasm confirmed photo→AVIF and the never-bigger relationship before I wrote the assertions.
3. **If you did this task again, what would you do differently?** — Write the four smoke assertions first and run them red against the old demo *before* touching the page, rather than building the demo and smoke together. The tests do genuinely gate the new behavior (old demo has no funnel/keepfull/timer, defaulted to WebP, had no never-bigger), but a literal red→green transition would be cleaner evidence. I'd also probe fixtures earlier — it's the cheapest way to avoid a plausible-but-wrong test.

---

## Reflection (Ship)
1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
