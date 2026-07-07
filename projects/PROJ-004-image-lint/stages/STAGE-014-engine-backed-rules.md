---
# Maps to ContextCore epic-level conventions.

stage:
  id: STAGE-014
  status: on_hold                    # proposed | active | shipped | cancelled | on_hold
  priority: low                      # deferred below the 1.0 line (2026-07-07 roadmap reconciliation)
  target_complete: null

project:
  id: PROJ-004
repo:
  id: crustyimg

created_at: 2026-07-06
shipped_at: null

value_contribution:
  advances: >
    Adds the "this could be smaller / wrong format" rules — the ones that make the linter more
    than a metadata checker. Each reuses PROJ-002's format-decision engine + the SSIMULACRA2
    probe behind the savings-threshold gate, so a finding is backed by a real measurement, not a
    heuristic.
  delivers:
    - "`format/legacy-format` (warn): proves via an equal-SSIMULACRA2 probe that a modern format
      saves ≥ threshold → fix `optimize --format <fmt>`; never suggests an unbuilt codec"
    - "`quality/excessive-jpeg-quality` (warn): a VL-target re-encode scores ≥ anchor while saving
      ≥ threshold → fix `optimize`"
    - "`format/indexed-png-opportunity` (info, advisory): RGB(A) PNG with few colours → palette
      PNG; stays advisory until a permissive quantizer ships (PROJ-007), interim suggests lossless
      WebP"
  explicitly_does_not:
    - Re-implement any search/decision math — it composes the shipped `src/analysis/decide.rs`
      engine + `src/quality/` search
    - Add a new default dependency
    - Ship indexed-PNG as a fix (needs PROJ-007's quantizer)
---

# STAGE-014: engine-backed rules

> **Deferred below the 1.0 line — demand-gated (2026-07-07).** `lint` already shipped as a
> 10-rule catalog in v0.4.0. This stage adds *more* lint breadth (three engine-backed rules),
> which the adoption-first roadmap reconciliation put past 1.0: further investment in the
> least-validated surface waits for a real adoption signal (Action/Eleventy users asking for it),
> and the next technical work is **HEIC/RAW/SVG input reach** instead (see `docs/roadmap.md`).
> The stage is cheap to finish (reuses the shipped engine, no new deps) and is kept **build-ready
> in spirit but unwritten** — SPEC-054/055 are *not* yet specced. Pick it up **if pulled**, not by
> default. Nothing here is cancelled; it is sequenced, not dropped.

## What This Stage Is

The stage that gives `lint` its teeth: the rules that answer "could this asset be smaller, or is
it in the wrong format?" — each proven by actually running PROJ-002's format-decision engine + the
SSIMULACRA2 perceptual probe on the file and checking the win against the savings-threshold gate.
No new compression math; these rules are a thin read over the shipped engine. It requires PROJ-002
(shipped) and the STAGE-013 framework (the `Rule`/`Finding` model + the savings-threshold config).

## Why Now

- **It's the Lighthouse-parity half** (`uses-webp-images`, `uses-optimized-images`) — but per-file
  and URL-free. The measurement is real (an actual probe), so it earns a `warn`.
- **It's cheap** — the engine + probe already exist; the rule just interprets the result.

## Success Criteria

- `format/legacy-format` fires only when a real equal-SSIMULACRA2 probe shows a built modern format
  saves ≥ the threshold; the fix names only a codec the running binary can produce (a
  license/capability guard — DEC-004).
- `quality/excessive-jpeg-quality` and `format/indexed-png-opportunity` respect the same
  savings-threshold gate; `indexed-png` stays `info`/advisory.
- No new default dependency; `just deny` green; determinism upheld (same file ⇒ same finding).

## Scope

### In scope
- `format/legacy-format` + the license/capability guard + the savings-threshold wiring. **(SPEC-054)**
- `quality/excessive-jpeg-quality` + `format/indexed-png-opportunity`. **(SPEC-055)**

### Explicitly out of scope
- SARIF / Actions / pre-commit (STAGE-015). Indexed-PNG as a fix (PROJ-007). Near-dup (v2).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-054 (not yet written) — `format/legacy-format`: equal-SSIMULACRA2 probe over the
  `src/analysis/decide.rs` engine → "could be <fmt>, saves N%"; savings-threshold gate; codec
  license/capability guard (no `--format avif` suggestion in a non-AVIF build).
- [ ] SPEC-055 (not yet written) — `quality/excessive-jpeg-quality` + `format/indexed-png-opportunity`
  (advisory) over the shipped quality search + format-rec.

**Count:** 0 shipped / 0 active / 2 pending

## Design Notes

- **The seam to watch:** `optimize`'s per-candidate solve (`solve_candidate` in `src/cli/mod.rs`)
  is CLI-private today. `legacy-format` needs the same "encode each candidate, measure bytes at an
  equal perceptual score" logic. Build cycle should factor a small shared helper (in
  `src/analysis/decide.rs` or `src/quality/`) rather than duplicate it — this also keeps the
  future planner (PROJ-003) reuse clean.
- Weighty decision → its own DEC (the legacy-format probe method + the codec-suggestion guard) if
  it goes beyond what DEC-050 already covers.

## Dependencies

### Depends on
- **PROJ-002** — `src/analysis/decide.rs` (format decision) + `src/quality/` (SSIMULACRA2 search).
- **STAGE-013** — the `Rule`/`Finding` framework + the savings-threshold config.
- DEC-004 (codec gating → the suggestion guard), DEC-019/020/021/022.

### Enables
- STAGE-015 — these rules appear in SARIF + the Action's PR annotations.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
