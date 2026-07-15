---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-029                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-008                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-13
shipped_at: null

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    The demo is PROJ-008's highest-ROI adoption artifact — the "watch it just work" moment we
    time a Show HN against. This stage makes it actually good under real traffic: fast on a real
    photo, smart about what a user wants (a SMALLER file, not a bigger one), and able to prove the
    quality it chose rather than assert it. It turns "the demo works" into "the demo is worth
    sending strangers to."
  delivers:
    - "A demo that leads with intent (make it smaller) and picks the format that actually shrinks the input — AVIF for photos, lossless WebP for graphics — instead of a lossless default that makes photos bigger"
    - "A real photo converts in seconds, not tens of seconds: rav1e at speed 10 (~3.6x faster, measured) + an offered resize, with honest megapixel-keyed warnings and a live timer so a slow encode never reads as a hang"
    - "The engine's SSIMULACRA2 quality decision, shown as a number (input vs output) — the differentiator vs squoosh: we don't guess the quality, we measure it"
    - "A wasm optimize() surface that finally takes the arguments the demo (and library users) need: encode speed/effort, a quality/byte-budget target, and a returned perceptual score"
  explicitly_does_not:
    - "Add threads/SharedArrayBuffer/COOP-COEP (impossible on the static GitHub Pages host — the single-thread constraint stands) or a new codec"
    - "Do the launch presentation itself — README front-door + BENCHMARKS + the Show HN go/no-go are STAGE-028, which depends on this"
    - "Turn the demo into a web app or the maintainer's separate site-builder tool — it stays a thin marketing artifact"
    - "Ship the npm publish (SPEC-076, gated on maintainer approval)"
---

# STAGE-029: demo launch quality

## What This Stage Is

The stage that makes the live demo good enough to drive traffic to. A design-time investigation
(2026-07-13, measured on the real wasm engine) found the demo is technically live but mis-serves its
most common visitor — someone dropping a phone photo:

- **It's slow where it matters.** Encode cost scales with **megapixels, not file size**. A 12 MP
  photo → AVIF takes **~33 s on a fast desktop** (minutes on mobile) at the hardcoded rav1e speed 6.
  In a Web Worker it doesn't freeze the page, but a silent 30 s+ spinner reads as a hang — which is
  exactly the report that kicked this off.
- **Its defaults fight the use case.** The default output is lossless WebP, which makes an
  already-lossy photo *bigger*. "Auto" avoids AVIF on wasm (the perceptual search needs a decode the
  wasm build lacks) and falls back to a slow JPEG search — ~4–11 s for only ~13 % savings. So the
  three photo paths today are: default → *bigger*, Auto → *slow + mediocre*, AVIF → *best but 33 s*.

This stage fixes all of that: it reorients the demo around **intent ("make it smaller")** with a
format choice that actually shrinks the input, makes the expensive path affordable (a **measured
3.6× via rav1e speed 10**, plus an offered resize and honest warnings), and surfaces the engine's
**SSIMULACRA2 quality score** as the headline differentiator. The enabling engine work — a real
`optimize()` surface (speed, quality/budget, returned score, an Auto-picks-AVIF path) — lands here
too, because the demo can't do any of it against today's argument-free surface (DEC-064).

## Why Now

- **The demo is the launch.** STAGE-027 proved it runs end-to-end; a Show HN points at it. But you
  only get one first impression, and today's demo underdelivers on its own headline (in-browser
  AVIF) by being too slow and picking the wrong default. Fix before traffic, not after.
- **The findings are measured, not guessed** (a full probe reshaped the plan; see Design Notes) —
  including a hypothesis that *failed* (wasm `simd128` gave ~10 %, not the 2–4× hoped, so it's
  dropped). The levers that survive are grounded.
- **It gates STAGE-028.** The README front-door and BENCHMARKS want real before/after numbers and a
  demo that lives up to the pitch. This stage produces both; STAGE-028 presents them.

## Success Criteria

- A real ~12 MP photo, dropped on the demo with default settings, converts to a **meaningfully
  smaller** file in **a few seconds** on a normal laptop — and the UI makes an honest expectation
  set + shows progress so it never reads as hung.
- The demo **never silently presents a larger file** as a result: a lossless-that-grows outcome is
  steered away from or flagged unmistakably.
- **"Auto" picks the format that shrinks the input** — AVIF for photographic input, lossless WebP
  for graphics — without the slow perceptual search on wasm.
- The result view shows the **SSIMULACRA2 score** (input vs output) for lossy conversions, honestly
  labelled (and honest about where it can't — see Design Notes on the AVIF-decode seam).
- `optimize()` on the wasm surface accepts **encode speed/effort** and a **quality/byte-budget
  target**, and returns the **achieved perceptual score** — with the CLI/native behaviour unchanged
  unless a spec explicitly says otherwise.
- Every `just wasm-*` gate stays green; no regression to the native CLI; commits signed-off.

## Scope

### In scope
- The `optimize()` wasm-surface expansion: rav1e speed/effort knob (reopening DEC-020), a
  quality/byte-budget argument, a returned SSIMULACRA2 score, and an Auto→AVIF (fixed-quality,
  no-search) path for photographic input on wasm.
- The demo intent/defaults redesign: a "make it smaller" primary flow, cheap-first defaults, the
  never-bigger guard, an **offered** (never silent) resize, megapixel-keyed expensive-op warnings, a
  live elapsed timer, and debounced re-conversion.
- The demo default encode **speed 10** (the CLI stays 6).
- The SSIMULACRA2 diff UI.

### Explicitly out of scope
- Threads / SharedArrayBuffer / COOP-COEP; any new codec or format.
- README / BENCHMARKS / the Show HN post + go/no-go (STAGE-028).
- The npm publish (SPEC-076, gated).
- Broadening the demo into a web app or a content/site-builder.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Dependency order — **SPEC-079 (engine surface) first**, because the demo specs consume it:

- [x] SPEC-079 (shipped 2026-07-14, PR #87, DEC-068) — **wasm `optimize()` surface.** A per-call
  **encode speed** arg (reopened DEC-020 for wasm; `encode_to_bytes_with`), a **quality/byte-budget**
  target arg, a **returned SSIMULACRA2 score** + a `score(a,b)` binding, and an
  **Auto-picks-AVIF-for-photos** fixed-quality/no-search path. Native CLI unchanged; verified CLEAN
  (speed knob proven 55×, native byte-identical).
- [~] SPEC-080 (design — framed build-ready 2026-07-13) — **demo intent/defaults redesign + perf UX.**
  "Make it smaller" primary flow; Auto default that shrinks; never-bigger guard; offered resize;
  megapixel-keyed warnings + live timer + debounce; **default speed 10**. Consumes SPEC-079; build
  after it ships.
- [~] SPEC-081 (design — framed build-ready 2026-07-13) — **demo SSIMULACRA2 diff UI.** Show the
  input↔output perceptual score, honest where the AVIF-decode seam needs a browser-decode + the
  `score()` binding. Consumes SPEC-079; build after SPEC-080 (both touch `demo/`).

**Count:** 1 shipped (SPEC-079) / 0 active / 2 framed. **Strategy reconciliation RESOLVED (2026-07-14):
the demo hero is the `web` flow (STAGE-030).** SPEC-080 is to be **reframed to the `web` hero** and
must wait for SPEC-085 (which defines `web` + the bundled recipes). Build order once reframed:
**080 → 081**.

**➕ Demo recipe presets (fold into the reframed SPEC-080, consumes SPEC-085).** Ship the bundled
recipes (SPEC-085: `web`/`gallery`/`product`/…) as **one-click client-side presets** in the demo —
the *same* recipe TOMLs the CLI ships (DEC-005), run in-browser: geometry/format recipes via the
existing wasm `transform(input, recipe_toml, out_format)`; the `web`/auto-format recipe via
`optimizeDetailed` (the hero). Story: "the recipe you click here is the same one you'd run in your
build." Cheap (the wasm engine already runs recipes); sequenced after SPEC-085 so the recipes exist.

## Surface properties SPEC-079 exposes (the demo specs MUST handle these)

Captured here (not only in SPEC-080/081) so they survive any reshaping of those specs by the strategy
reconciliation — they are real properties of the shipped surface, verified 2026-07-14:

- **`score()` is raw SSIMULACRA2 — NOT bounded 0–100.** A q20 JPEG scored **−4.70**. Whatever renders
  the score (a bar / gauge / band) must handle negative values; a 0–100 widget fed −4.7 renders wrong.
- **An unsatisfiable byte budget returns over-budget bytes silently.** `optimizeDetailed(img, "avif",
  10, 100, null)` returns 676 B at q=1 — over the 100 B budget, with no signal in `OptimizeResult`
  (the native CLI *warns* here). The demo must compare `bytes.length` against its own budget before
  promising it.
- **Speed is AVIF-only**; `speed()`/`quality()` are `undefined` for genuinely lossless output, and
  `quality()` is `Some(80)` for a default-encoded AVIF (it is lossy).

## Design Notes

**The measured investigation (2026-07-13).** All numbers are from the real wasm engine (committed
`pkg/` artifact, Node `initSync`, bracketing synthetic PNGs smooth-vs-detailed):

| Lever | Effect (12 MP detailed photo → AVIF) | Verdict |
|---|---|---|
| rav1e **speed 6 → 10** | **33.6 s → 9.4 s (~3.6×)**, ~+4 % bytes | **take** — the demo default |
| **resize** to 2048 px | 6× fewer pixels → ~1.5 s combined with speed 10 | **offer** (not silent cap) |
| wasm `simd128` | 33.6 s → 30.2 s (~10 %) | **drop** — rav1e SIMD is x86/ARM intrinsics, doesn't map to wasm |
| threads | large | **impossible** — needs COOP/COEP headers GitHub Pages can't set |

**The AVIF-decode seam constrains two things.** The wasm build encodes AVIF but cannot decode it
(DEC-065). That's why (a) "Auto" avoids AVIF today (the perceptual search decodes each candidate to
score it) — SPEC-079's Auto→AVIF path must therefore pick AVIF at a fixed good quality *without* a
search; and (b) a SSIMULACRA2 score for an AVIF *output* can't be computed in-engine — SPEC-081 either
scores via the browser-decode seam the worker already uses for `.avif` inputs, or is honest that the
AVIF score is unavailable. Don't overclaim.

**Cost is megapixels, not megabytes** — every warning/threshold keys off decoded dimensions, never
file size. A 10 MB file is often a 24 MP image; a 10 MB claim would mislead.

**"Never bigger" = keep the original (passthrough), not a bigger file.** Making a file bigger is
never a user *goal* — it's only ever a side effect of a format/transparency/archival/upscale intent,
none of which is the demo's "make it smaller" job. So when nothing the engine does beats the source,
the demo **offers the original back and explains** ("already well-optimized"), mirroring the CLI's
existing `pick_winner` passthrough (`decide::pick_winner` returns `None` = keep source). The demo has
both byte counts already, so this is a page-side rule in SPEC-080 — no engine change. The legitimate
"bigger" operations (explicit `convert`, pad/canvas in Wave 5, AI super-res at 2.0) live elsewhere by
design and stay out of this flow.

**Keep the CLI still.** The speed knob and Auto changes are wasm-surface / demo concerns; the native
`AVIF_SPEED = 6` default and CLI behaviour don't change unless a spec argues for it explicitly. The
byte-parity cross-sync contract (DEC-016/019/020) means a speed arg touches both `src/sink` and
`src/quality` — respect it.

## Dependencies

### Depends on
- STAGE-027 (the live demo + Web Worker + `.avif`-input seam) — shipped.
- The `crustyimg-wasm` package / `just wasm-build` size-profiled artifact (DEC-066) — the demo
  vendors it; the surface change rebuilds it.

### Enables
- STAGE-028 (launch readiness): a demo worth pointing at, plus the real before/after speed/size
  numbers the README and BENCHMARKS need, and the SSIMULACRA2 headline for the post.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
