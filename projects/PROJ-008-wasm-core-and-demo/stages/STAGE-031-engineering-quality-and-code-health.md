---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-031                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-008                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-19
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Protects the launch push by lowering the risk and friction of post-launch change — the codebase
    a Show-HN spike will draw contributors and scrutiny to should be legible and safe to modify.
  delivers:
    - A legible, decomposed CLI module tree (no more one 6.5k-line file) so post-1.0 changes are
      reviewable and low-risk.
    - Single sources of truth for duplicated internals (e.g. one JSON escaper), removing silent-drift
      hazards.
  explicitly_does_not:
    - Change any user-visible behavior, CLI surface, or output (byte-identical is the gate).
    - Add features, dependencies, or performance work — this is maintainability only.
---

# STAGE-031: engineering quality and code health

## What This Stage Is

The home for the maintainability/code-health work surfaced by the pre-launch Rust audit (the "6
directives" probe). When its specs ship, the codebase is measurably easier and safer to change —
no more single mega-modules, no duplicated internal helpers that can silently diverge — **with
zero change to behavior, surface, or output.** This stage is deliberately separate from the launch
stages (028 README/BENCHMARKS, 030 CLI freeze): those define what ships; this keeps what ships
maintainable. Not every audit directive lands here — several were N/A or design changes; only the
confirmed, behavior-preserving hygiene items do.

## Why Now

Framed now because the Rust audit surfaced concrete, well-evidenced items and framing is cheap;
**executed deliberately, gated on maintainer review, not on the launch clock.** The one unambiguous
structural item (the 6,483-line `src/cli/mod.rs`) only grows harder to split as more lands on it, so
capturing the plan now — before README/BENCHMARKS work touches the CLI docs — is timely. Nothing here
blocks Show HN; a legible codebase is what a launch spike's contributors and reviewers land in.

## Success Criteria

- Each shipped spec proves **byte-identical behavior** (output/exit-code diff against a pre-change
  oracle) — a maintainability change that alters behavior has failed.
- The full test matrix (native, lean `--no-default-features`, wasm) stays green throughout.
- Duplicated internal helpers identified by the audit have a single source of truth.

## Scope

### In scope
- Behavior-preserving structural refactors and internal de-duplication surfaced by the Rust audit.

### Explicitly out of scope
- Any behavior/surface/output change; new features; new dependencies; performance work.
- Signature/API changes (e.g. argument-struct bundling) — those are separate cosmetic follow-ups,
  never bundled into a behavior-preserving move (it would destroy the byte-identity gate).
- The audit directives shelved by the maintainer (D1/D2/D3/D5/D6 — see Design Notes) — recorded in the
  audit doc, not acted on. (D4 IS in scope, as SPEC-098's decision record.)

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-097 (shipped 2026-07-19, PR #103, ~$17.48) — decomposed `src/cli/mod.rs` **6,483 → 1,426**
  lines into a `build/report/optimize/ops/common` submodule tree + deduped `escape_json` to one source.
  **Byte-identical behavior** proven by an independent oracle (27/27 golden + a function-body diff across
  ~170 fns); 0 tests dropped; no signature/visibility change. The r/rust-facing code-legibility win.
- [x] SPEC-098 (shipped 2026-07-19, PR #102, **DEC-078**, ~$2.1) — **dependency-pinning strategy DECISION
  RECORD**, closing the audit's D4 thread: exact `=` pins stay policy for the binary today; caret
  relaxation of the library-public deps is a mandatory, deferred prerequisite of the crates.io publish
  (backlog #5); no migration now; refines AGENTS.md §5 / DEC-011/013. Docs-only, zero code change.

**Count:** 2 shipped (SPEC-097 cli split, SPEC-098 DEC-078) / 0 active / 0 framed.

**Queued for this stage (not yet framed):**
- **crates.io / pinning correction** — DEC-078's premise ("not on crates.io") is FALSE; crustyimg is
  published (0.4.0, `has_lib:true`, auto-published every tag). So the caret migration of the
  library-public deps is a real, current cleanup (not deferred), and DEC-078 + STAGE-007/DEC-041/
  RELEASING.md/the audit D4 are stale. One small spec: caret-migrate library-public rows (next release)
  + supersede/correct DEC-078 + de-stale the release docs. Verified 2026-07-19.
- **strict-JSON `escape_json`** (SPEC-097 follow-up, low priority) — `0x7F`/≥0x20 controls pass through
  unescaped (byte-identical to pre-split main; a *behavior* change to fix, own spec).

## Design Notes

**Source: the pre-launch Rust audit** (`docs/research/proj-008-rust-directives-audit.md`, landed on main
2026-07-19). Of the 6 directives + 1 structural finding it evaluated, the maintainer (2026-07-19) adopted
two and **shelved the rest**:

- **Adopted:** the `src/cli/mod.rs` structural split → **SPEC-097**; the D4 pinning decision → **SPEC-098
  / DEC-078**.
- **Shelved (no action — recorded in the audit doc, do not re-raise):**
  - **D1** checked-math-on-dimensions — **SATISFIED**; every buffer-sizing multiply is already u64/capped
    (the two plain-`usize` sites are proven-bounded, not defects). Only belt-and-suspenders remained; not
    worth it. (Note: the real typed cap error is `ImageError::LimitsExceeded`, not `DimensionsTooLarge`.)
  - **D2** zero-allocation-pipeline — **N/A**; per-op allocation is real but not the bottleneck (encode
    dominates); a scratch-buffer rewrite chases a non-cost.
  - **D3** miette CLI diagnostics — **design change** vs DEC-007 (thiserror + exit-code mapping); a real
    UX idea but not pursued now.
  - **D5** static-dispatch-in-hot-loops — **N/A** (false premise); `Box<dyn Operation>` dispatches ~3×
    per image, not per pixel.
  - **D6** tile-based-parallelism — **N/A** (false premise); rayon parallelizes across images, there is no
    intra-image striping to convert.

Standing gate for every spec here: **byte-identical behavior**, proven against a pre-change oracle.

## Dependencies

### Depends on
- STAGE-YYY (in this project or previous) — what it provides
- External: <third-party, vendor, approval>

### Enables
- STAGE-ZZZ — what becomes possible once this ships

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
