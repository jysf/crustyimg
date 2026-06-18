---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-026
  type: decision
  confidence: 0.80
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-18
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/mod.rs

tags:
  - cli
  - responsive
  - srcset
  - html-emission
  - web-prep
---

# DEC-026: responsive command shape + HTML <picture>/srcset emission

## Decision

`crustyimg responsive <input> --widths <list> --out-dir <dir> [--formats <list>]`
generates one width-scaled variant per (width × format) — resizing **by target
width, preserving aspect, never upscaling** (the resize `fit W×BIG` primitive),
deduped by actual width — and **prints a paste-ready `<picture>`/srcset HTML
snippet to stdout** (opt-out via `--no-snippet`). Emitting HTML **is in scope** for
crustyimg (it is opt-in output, the whole point of a responsive generator). v1 uses
fixed `-q`/format-default quality (lossy default 80); blurhash/thumbhash placeholders,
perceptual/`--max-size` per-variant quality, glob/batch input, and a `sizes`
attribute are **deferred**. It adds **no new dependency**.

## Context

STAGE-009's web-delivery item (roadmap Month 2) is the responsive set — the artifact
a web developer ships. The open questions this DEC settles:

1. Is "crustyimg emits HTML" in scope? (Roadmap open question #3.)
2. How are the widths interpreted (long-edge vs width), and what about upscaling?
3. What quality do variants use, and does the modern-format engine drive them in v1?
4. How much (blurhash, art-direction, batch) is in v1 vs deferred?

Constraints: `ergonomic-defaults`, `decode-once` (DEC-002), `untrusted-input-hardening`
(write into out-dir safely), feature-gated codecs (DEC-004), no new dep without a DEC.

## Alternatives Considered

- **Option A: only write the variant files; no HTML.**
  - Why rejected: the `<picture>`/srcset snippet is the differentiator — assembling
    it by hand is exactly the tedium users avoid. Writing files without the snippet
    leaves the hardest part to the user and abandons the "format auto-negotiation in
    HTML" gap.

- **Option B: full responsive suite in v1 (multi-format auto-pick + perceptual
  per-variant + blurhash placeholder + art-direction `<source media>`).**
  - Why rejected: too large for one spec, and each rider carries its own design
    question (blurhash needs a new dependency + DEC; perceptual-per-variant multiplies
    the metric cost and inherits the AVIF-no-decode wrinkle). Ship the core, defer the
    riders.

- **Option C (chosen): width×format variants + `<picture>`/srcset to stdout, fixed
  quality, no-upscale/dedup, riders deferred.**
  - Why selected: delivers the whole differentiating workflow (multi-width, optional
    multi-format, paste-ready HTML) as pure composition with no new dependency;
    interpreting `--widths` as true widths (via `fit W×BIG`) makes the srcset `w`
    descriptors accurate; no-upscale + dedup-by-actual-width keeps output honest; and
    the deferrals are clean, independent follow-ups.

## Consequences

- **Positive:** one command turns a source image into a deployable responsive set +
  correct HTML; reuses the resize op + per-format sink + decode-once; no dependency →
  `cargo deny` unaffected; feature-gated formats reuse `convert`'s up-front exit-4.
- **Negative:** `--widths` are true widths, so a portrait image scaled to width W has
  height > W — fine for srcset, but users thinking "longest edge" may be mildly
  surprised (documented). No `sizes` attribute means the user must add one for
  non-100vw layouts.
- **Negative:** v1 variants are NOT perceptually optimized per-width (fixed quality) —
  a follow-up can route each variant through `resolve_effective_quality`.
- **Neutral:** single-input only; the fan-out is over widths×formats, not inputs.

## Validation

- Right if: `responsive hero.jpg --widths 320,640,1280 --formats webp,jpeg --out-dir
  dist/` produces correct variants + a `<picture>` block that pastes into a page and
  works, and users reach for it instead of hand-writing srcset.
- Revisit when: the blurhash placeholder spec is written (adds a dep + DEC); or if
  users want perceptual-per-variant (route through the STAGE-008 search); or if
  art-direction (`<source media>`) / `sizes` control is requested.

## References

- Related specs: SPEC-024 (this command), SPEC-011 (resize fan-out), SPEC-014
  (`convert` feature-gate pattern), SPEC-022 (`optimize`)
- Related decisions: DEC-008 (resize backend), DEC-015 (format precedence), DEC-016
  (quality), DEC-004 (feature-gated codecs), DEC-002 (decode-once)
- Discussions: 2026-06-16 roadmap handoff (responsive set; open question #3 on HTML
  emission — recommended yes)
