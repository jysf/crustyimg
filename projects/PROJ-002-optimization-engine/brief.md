---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-002                      # stable, zero-padded, never reused
  status: active                    # proposed | active | shipped | cancelled
  priority: high                    # critical | high | medium | low
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg                     # must match .repo-context.yaml

created_at: 2026-07-05
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    An analysis-driven decision engine: crustyimg looks at the image, decides the
    best format and encoding automatically — the "local f_auto" — and explains why.
    This turns "optimize this" from a format-and-quality guessing game into a
    declared outcome produced deterministically by one safe, pure-Rust binary, and
    it lays the shared analysis layer that later linting, planning, and manifest
    work all read. It advances the moat from "smallest file at a quality you name"
    to "smallest file in the format the tool picks for you, with the reasons shown."
  beneficiaries:
    - Web/content developers optimizing an asset tree in a build/CI/pre-deploy step
    - CI pipelines and Makefile/GitHub-Actions shops that want no service and no Node
    - The maintainer — the shared `Analysis` layer is the base every later wave reads
      (lint, planner, manifest)
  success_signals:
    - "`optimize photo.png` with no `--format` auto-picks the smaller of {AVIF/WebP/PNG/JPEG}
      at equal perceptual quality and beats the source — or leaves it unchanged if already optimal"
    - "`optimize --explain` prints the chosen format, the candidates tried, bytes saved %,
      and the perceptual score — the decision is auditable, not a black box"
    - "classification routes photo→lossy and logo/flat-graphic→lossless correctly on a
      labeled fixture corpus (starting thresholds tuned in-build)"
    - "the `Analysis` layer ships standalone with every existing test green — no regression
      to edit/apply or the byte-stable recipe round-trip"
    - "the default build stays pure-Rust, zero-system-deps; `just deny` green; no new default
      dependency (the decision engine is pure analysis + the existing search)"
  risks_to_thesis:
    - Format heuristics can misfire on gray-zone images (gradient UI, photo-of-a-document) →
      wrong format; mitigated by a safe fallback (photograph/lossy) bounded by the existing
      SSIMULACRA2 perceptual guarantee
    - The extra analysis pass + multi-format trial encodes may be too slow on large batches →
      mitigated by a ≤3-format shortlist, the capped (≤8-iter) search, and fast-path short-circuits
    - Auto-decide may read as a black box vs. Cloudinary `f_auto` if the `explain` trace isn't clear
    - Indexed-PNG wins need a permissive quantizer (deferred, `quantette`) → some PNG savings
      left on the table until a later wave
---

# PROJ-002: the optimization engine core

## What This Project Is

The wave that turns crustyimg from "auto-tunes the quality of a format you name" into an
**analysis-driven optimization engine that picks the format for you and explains the choice**.
Two coherent pieces: (1) a new shared, computed-once **`Analysis`** layer that looks at the
decoded image (histogram, entropy, edge density, alpha coverage, colour count, dominant
colour, and an internal photo-vs-graphic **classification**); and (2) a **format
auto-decision** step inside the existing `optimize` command — the "local `f_auto`" — that uses
that analysis to shortlist candidate formats, drives the *already-shipped* SSIMULACRA2 search
across them, ships the smallest artifact that meets the target, and prints an **`explain`**
trace of every decision. It is deliberately scoped as the *foundation + first differentiating
ship*; the general goal-solver (the planner), source-file `lint`, and the web-asset manifest
are the next projects that all read the `Analysis` layer this one builds.

## Why Now

- **It extends the actual moat, cheaply.** The shipped wedge is already "set the look, not the
  number" (perceptual auto-quality). Format auto-decision + explain are the apex of that engine
  and are **deterministic and zero-new-dependency** — pure analysis + logic on the decoded
  buffer, composing the existing `src/quality/` search + `LossyFormat` seam. Nothing else on
  the roadmap is both this differentiating and this cheap.
- **Demand is validated.** Cloudinary `f_auto`/`q_auto` adoption (≈86% bandwidth cuts, baked
  into build plugins) proves developers want the tool to *choose* the format, not be *told*;
  and Hugo/Zola closing the AVIF gap means the wedge is the *local, deterministic, explained*
  decision, not "we have AVIF." (`docs/research/proj-002-findings.md §5.`)
- **It is the dependency root for the wave.** The `Analysis` layer is prerequisite for
  `lint` (PROJ-004), the planner (PROJ-003), and per-image manifest fields (PROJ-005). Build
  it first, once, shared.

## Success Criteria

- `optimize <file>` with no `--format` chooses format + encoding automatically and produces the
  smallest artifact that meets the perceptual target (or the byte budget), beating the source —
  or leaves an already-optimal file unchanged, never emitting a larger one.
- `optimize --explain <file>` reports the decision: features detected, class, candidates tried
  (format/quality/score/bytes), the winner and the one-line reason, and source→output savings.
- Classification + the format decision route photo→lossy-family and logo/flat→lossless-family
  correctly on a labeled fixture corpus; the decision is deterministic (same input+profile+flags
  ⇒ same output).
- The `Analysis` layer lands as a standalone module with unit tests and **zero regression** to
  existing commands, the recipe byte-stable round-trip, or the load-once/decode-once model.
- Default build stays pure-Rust / zero-system-deps; `cargo deny check licenses` green; no new
  default dependency; STAGE-006 hardening upheld (bounded, no-panic on untrusted input).

## Scope

### In scope
- **STAGE-011 — Analysis foundation:** a new `src/analysis/` module (`Analysis` immutable
  computed-once context + feature extractors + internal `classification`), typed `AnalysisError`
  (no-panic, bounded), unit tests, registered in `lib.rs`. **No CLI behaviour change** — lands
  standalone so every existing test stays green.
- **STAGE-012 — Auto-decide & explain:** format auto-decision inside `optimize` (the decision
  tree + candidate shortlist, composing the existing SSIMULACRA2 search + `LossyFormat` seam);
  a `--profile <web|docs|preserve>` default; and an `--explain` trace. Ships as **0.3.0**.

### Explicitly out of scope
- **The general optimization *planner*** (declarative `max_size`/`min_quality` goal solver) —
  PROJ-003. This project ships the auto-decide *default* inside `optimize`; the planner
  *generalizes* it. Design the format-decision code as the seam the planner will wrap (one
  "decision engine," two entry points — `docs/research/proj-002-findings.md §9`).
- **`lint`** (source-file linter) — PROJ-004; reads this project's `Analysis` layer.
- **The web-asset manifest / placeholders / favicon** — PROJ-005.
- **Geometry, effects, auto-colour, upscaling** — PROJ-006 (the "add-anytime companion" track).
- **Indexed/lossy-PNG output** — needs a permissive quantizer (`quantette`), deferred to
  PROJ-007; interim, lossless WebP covers the "few-colour graphic" case.
- Surfacing classification as a user-facing feature — it stays internal (at most a one-word
  `explain` label). No standalone `classify` command.

## Stage Plan

> **Scope confirmed 2026-07-05 (maintainer decision):** the *focused engine core* — Analysis
> foundation + auto-decide `optimize` + explain. The goal-driven **planner is PROJ-003** (it
> generalizes STAGE-012's decision engine); `lint`=PROJ-004, manifest=PROJ-005, crop=PROJ-006.

Format: `- [status] STAGE-ID — one-line summary`

- [~] STAGE-011 (active, IN PROGRESS) — Analysis foundation: `src/analysis/` layer (features +
  internal classification), lands standalone, all existing tests green — both specs
  (SPEC-046, SPEC-047) designed; SPEC-046 is the first build target
- [ ] STAGE-012 (proposed) — Auto-decide & explain: format auto-decision in `optimize`
  ("local f_auto") + `--profile` + `--explain`; ships 0.3.0 — both specs (SPEC-048, SPEC-049)
  designed ahead, awaiting STAGE-011

**Count:** 0 shipped / 1 active / 1 pending

## Dependencies

### Depends on
- **PROJ-001 (shipped MVP)** — the load-once `Image`/decode pipeline, the encoding `Sink`, and
  critically `src/quality/` (the SSIMULACRA2 search, `--max-size` fit, the `LossyFormat`
  two-predicate seam) which the format decision composes without modification.
- DEC-004 (pure-Rust codec policy), DEC-018 (permissive license gate), DEC-019 (perceptual
  search), DEC-020/021/022 (AVIF/WebP capability profiles), DEC-016 (encode byte-for-byte sync).
- External: no new default dependency. No third-party services.

### Enables
- **PROJ-003 (planner)** — the goal-driven solver generalizes the format-decision engine.
- **PROJ-004 (lint)** — every rule reads the `Analysis` layer; `explain` output patterns reused.
- **PROJ-005 (web-asset manifest)** — per-image `class`/`dominant_color`/`optimization` fields
  come from `Analysis` + the `ExplainTrace`.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
