---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-030
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-14
shipped_at: null

value_contribution:
  advances: >
    The last thing between crustyimg and a 1.0-worthy launch: freeze a CLI surface people can trust
    and make the verb everyone runs deliver the measured win. A benchmark over a real photo corpus
    proved the current default optimizes the least-valuable variable (24% in 16.5 s, 2/8 passthrough,
    49 s on a 47 MP photo) while a downscale→AVIF "web" path gets 98% in 2.7 s. This stage ships that
    flow as the flagship and cuts the surface from ~20 verbs to ~14, each with one clear intent —
    before the README and Show HN document it, so we never relaunch with a changed CLI.
  delivers:
    - "A `web` flagship verb: downscale + content-aware modernize (AVIF for photos / lossless-WebP for graphics) + never-bigger + strip + orient — the 98%/2.7s path, one command"
    - "A fast, AVIF-aware DEFAULT engine decision (fixed-quality single-encode, not the 9–74 s byte-budget search) with a first-class 'kept, already optimal' passthrough"
    - "`optimize` demoted to an honest byte-primitive (best format at good quality, keep dimensions, never-bigger, --verify); the perceptual search becomes opt-in via --target/--ssim/--max-size"
    - "A cleaner ~14-verb surface: `convert --to`, a `meta` group (strip/clean/set/copy), bundled recipes (web/gallery/product/…), `shrink` removed (its behavior folded into `web`, upgraded with AVIF)"
    - "A unified audit report + --json/--timing across lint/optimize/web, and a committed benchmark corpus + harness (no telemetry)"
  explicitly_does_not:
    - "Add a backend/service, a new codec, or ML — the territory guardrails stand"
    - "Touch the wasm surface (SPEC-079 shipped the wasm twin of the engine decision) or add a CLI --speed flag (DEC-020)"
    - "Do the README/BENCHMARKS authoring (SPEC-082/083, STAGE-028) — this stage produces the surface + numbers those will document"
    - "Preserve backward compatibility: this is a HARD CUTOVER (rename/remove/merge, no aliases, no deprecation) — done now because the surface has no dependents but the maintainer"
---

# STAGE-030: command taxonomy & CLI-quality freeze

## What This Stage Is

The pre-launch surface freeze. A measured benchmark (2026-07-14 — a real 8-photo corpus, 0.7–47 MP,
5 cameras, + an AVIF quality sweep; harness in `scratchpad/bench/`) settled the "is the hero
`optimize`, `shrink`, or modernize-to-AVIF?" question with numbers: the hero is a **`web`** flow —
downscale to a web size, then modernize by content (AVIF for photos, lossless-WebP for graphics),
never bigger. It gets **98% median savings in 2.7 s and is size-insensitive** (2–5 s from 0.7→47 MP),
where today's `optimize` default gets **24% in 16.5 s** (2/8 passthrough, up to 49 s).

So this stage (a) makes the **default engine decision fast and AVIF-aware** (SPEC-084 — the native
twin of the wasm surface SPEC-079 already shipped), (b) ships **`web`** as the flagship verb built on
bundled recipes (SPEC-085), (c) **redefines `optimize`** into an honest keep-dimensions byte-primitive
and **removes `shrink`** (SPEC-086), and (d) **cleans the surface** to ~14 one-intent verbs
(`convert --to`, a `meta` group, an elevated audit pillar) with a unified report + committed bench
(SPEC-087/088, opt. 089). It is a **hard cutover** — no aliases, no deprecation — because crustyimg
has no dependents but the maintainer, and relaunching with a changed CLI later is strictly worse.

## Why Now

- **Freeze before you document.** The README (SPEC-082) and BENCHMARKS (SPEC-083) must show the
  *final* surface and these numbers. Maintainer decision (2026-07-14): **freeze the CLI first, then
  launch.** So STAGE-030 lands before STAGE-028.
- **The win is measured, not intuited** — a real corpus + a q-sweep, not a hunch. The root cause is
  understood: the "faceplant" is the byte-budget AVIF search re-encoding rav1e (9–74 s) plus a
  conservative visually-lossless target — both fixable by admitting AVIF at *fixed* quality and making
  the search opt-in.
- **The enabling fact is already true.** Native AVIF *decode* shipped (DEC-058), so the
  `Mode::SizeBudget`-only AVIF gate in `decide.rs` is vestigial on native — the only 2× win is being
  excluded from the default path for a reason that no longer holds.
- Maintainer decision (2026-07-14): **fold into PROJ-008** as a pre-launch stage (not a new PROJ-010 —
  AGENTS.md frames a project only after the prior ships, and this is the same launch push).

## Success Criteria

- The **default** decision on a photo picks **AVIF at fixed quality via a single encode** (no 9–74 s
  search), beats the source substantially, and reports a once-computed SSIMULACRA2 score; on a graphic
  it stays **lossless**; when nothing beats the source it **passes the original through** (never bigger).
- **`web <inputs>`** exists and delivers the measured flow (downscale + content-modernize + never-bigger
  + strip + orient), size-insensitive, in a few seconds — `web == apply --recipe web`.
- **`optimize`** is an honest keep-dimensions byte-primitive with `--verify`; the perceptual/byte-budget
  **searches are opt-in** (`--target`/`--ssim`/`--max-size`). **`shrink` is gone.**
- The surface is ~**14 verbs**, each with one intent: `convert --to`, a `meta` group, bundled recipes,
  the audit pillar (lint/info/view/diff) elevated, a unified report + `--json`/`--timing`.
- A **committed benchmark corpus + harness** (seeded from `scratchpad/bench/`, no telemetry).
- The default AVIF quality is **validated on eyeballs** (a small diverse corpus + the q-sweep), not
  hardcoded blind. All gates green (native + `--features avif` + lean).

## Scope

### In scope
The engine default-decision change; the `web` verb + bundled-recipe registry; the `optimize`
redefinition + `shrink` removal; the `convert --to` rename; the `meta` consolidation; the unified
audit report + `--json`/`--timing`; the committed bench corpus/harness; all in-repo doc/test updates
the cutover requires.

### Explicitly out of scope
- README/BENCHMARKS authoring (SPEC-082/083); the wasm surface (SPEC-079); a CLI `--speed` flag
  (DEC-020); any backend/codec/ML; backward-compat aliases (deliberate — hard cutover).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`. Build order: **084 → {085, 086} → {087, 088}
→ 089 (opt)**.

- [~] SPEC-084 (design — draft adopted from the strategy session, being validated) — **engine:
  fixed-quality AVIF in the DEFAULT decision** via single-encode compare (not the search), two-regime
  quality, first-class never-bigger passthrough, score-winner-once. Native twin of SPEC-079; converges
  the native/wasm Auto paths. Emits **DEC-069**. Frame/validate first.
- [ ] SPEC-085 (not yet framed) — **`web` flagship verb** + bundled-recipe registry (`include_str!` +
  registry); `web == apply --recipe web`; ship web/gallery/product; feature RAW (`web ./raws/`).
  Consumes 084.
- [ ] SPEC-086 (not yet framed) — **redefine `optimize`** (byte-primitive, keep dims, `--verify`) +
  **remove `shrink`**; update all refs. Consumes 084.
- [ ] SPEC-087 (not yet framed) — **`meta` group** consolidation (strip/clean/set/copy); auto-orient
  stays top-level. Surface move.
- [ ] SPEC-088 (not yet framed) — **unified audit report** + `--json`/`--timing` across
  lint/optimize/web/apply + **committed bench corpus/harness** (seed from `scratchpad/bench/`).
- [ ] SPEC-089 (optional / may fold) — `convert --to` rename + social/archive recipes.

**Count:** 0 shipped / 1 in design (SPEC-084) / 5 pending.

## Design Notes

- **Hard cutover discipline.** Rename/remove/merge freely; no aliases, no deprecation, no CHANGELOG
  migration. Cost is in-repo docs + tests only. Every reference to a renamed/removed verb gets updated.
- **Honesty guardrails (non-negotiable):** passthrough is a **green** result ("kept, already optimal"),
  not a failure; **never silently enlarge** (subsumes the Track-B `optimize` fix); downscale is
  **`web`'s** opinion, not `optimize`'s (optimize keeps dimensions); don't claim *visually-lossless* at
  a generous default — say the measured score.
- **The one judgment-bound number is the default AVIF quality** (two-regime: `web` generous ~q85–90;
  `optimize` can lower via `--target`). The q-sweep shows bytes are trivial across the range after a
  downscale, so be generous — but **validate on eyeballs**, and sanity-check the `-q`→perceptual
  mapping (q80 → ~77 is more aggressive than the label). Record the chosen value + rationale in DEC-069.
- **Content branch already works** (the bucket classifier): a screenshot → lossless WebP (AVIF on it is
  a 4× regression). Do **not** fire AVIF on graphics.
- **AVIF admission is a bucket predicate**, not shortlist membership (SPEC-079's `MAX_SHORTLIST`
  truncation lesson).

## Dependencies

### Depends on
- SPEC-079 (shipped) — the wasm twin of the default decision; SPEC-084 mirrors its shape and the two
  paths converge. DEC-058 (native AVIF decode) — what makes admitting/scoring AVIF sound on native.

### Enables
- STAGE-028 (README/BENCHMARKS, SPEC-082/083) — which document the frozen surface + these numbers.
- The demo (STAGE-029, SPEC-080) — its hero becomes the **`web`** flow; reframe SPEC-080 to mirror
  `web`'s definition once SPEC-085 lands.
- The Show HN launch, on a surface we won't have to change.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
