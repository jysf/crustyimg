---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.

stage:
  id: STAGE-012                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null

project:
  id: PROJ-002                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-05
shipped_at: null

value_contribution:
  advances: >
    Delivers the differentiating capability of PROJ-002: `optimize` stops preserving
    the input's format and starts CHOOSING the best one automatically — "the local
    f_auto" — and prints why. This is the user-visible payoff of the Analysis layer
    and the 0.3.0 headline.
  delivers:
    - "`optimize <file>` (no `--format`) auto-decides output format + lossy/lossless from
      the Analysis layer, drives the existing SSIMULACRA2 search across a ≤3-format shortlist,
      and ships the smallest artifact that meets the target (or leaves an optimal file unchanged)"
    - "`--profile <web|docs|preserve>` — `web` default; `preserve` reproduces today's
      format-preserving behaviour exactly (strictly additive)"
    - "`--explain` — a concise, auditable trace: features, class, candidates tried
      (format/quality/score/bytes), the winner + one-line reason, and savings %"
  explicitly_does_not:
    - Add the declarative goal-solver / planner (`max_size`/`min_quality` objective) — PROJ-003
    - Add indexed/lossy-PNG output (needs a permissive quantizer, PROJ-007) — interim, row-B
      few-colour graphics use lossless WebP
    - Change behaviour when the user pins `--format`/`-o` (the pin bypasses the engine)
---

# STAGE-012: auto-decide & explain

## What This Stage Is

The stage that ships the PROJ-002 headline: **`optimize` picks the format for you and explains
it.** It inserts one new step between decode and the existing quality search — **candidate-format
selection** — that reads the STAGE-011 `Analysis`, shortlists ≤3 candidate formats via a
deterministic decision tree, drives the *already-shipped* SSIMULACRA2 perceptual search (and
`--max-size` byte-budget path) across them, and ships the smallest artifact that meets the target
while never emitting one larger than the source. A `--profile <web|docs|preserve>` selects the
bias (`preserve` = today's behaviour, so the change is strictly additive), and `--explain`
prints a concise, auditable trace of the decision. When this ships (0.3.0), "make this web-good"
stops meaning "guess a format and a quality" and starts meaning "give it the file; it decides and
tells you why."

## Why Now

- **It's the differentiating payoff of the Analysis layer** and the validated core: Cloudinary
  `f_auto` adoption proves developers want the tool to choose the format. crustyimg becomes the
  **local, deterministic, explained** version — for teams not on a media CDN.
- **It's cheap and additive.** It's an orchestrator around `src/quality/` — no new search math,
  no new default dependency — and it only fires when the user hasn't pinned a format, so pinned
  workflows are unchanged and `--profile preserve` is a clean regression anchor.
- **It sets the seam the planner (PROJ-003) will wrap.** Designing the decision engine now, with
  the shortlist/winner logic factored cleanly, means the goal-solver generalizes it rather than
  duplicating a second format loop (the "one decision engine, two entry points" reconciliation,
  `docs/research/proj-002-findings.md §9`).

## Success Criteria

- `optimize photo.png` with no `--format` produces the smaller of the shortlisted candidates at
  equal perceptual quality, beating the source — or leaves an already-optimal file unchanged,
  never larger.
- **Clear-win guard:** the output format switches away from the source format ONLY when the byte
  win clears a `FORMAT_SWITCH_THRESHOLD` (else the source format is kept — no surprising switch for
  a marginal gain); the chosen format + savings are ALWAYS reported (a one-line summary by default,
  full detail under `--explain`).
- `optimize --explain photo.png` prints: features detected, class, each candidate
  (format/quality/score/bytes/met), the winner + one-line reason, and source→output savings %.
- `optimize --profile preserve` == today's `optimize` (byte-identical output) — the regression
  anchor.
- The decision is deterministic: same `(pixels, profile, feature-flags, mode)` ⇒ same output.
- Pinning `--format`/`-o <ext>` bypasses the engine (deterministic escape hatch); AVIF appears
  as a candidate only in byte-budget mode and only when the `avif` feature is built.
- `just deny` green; no new default dependency; 3-OS CI green; STAGE-006 hardening upheld.

## Scope

### In scope
- The candidate-format decision engine in `optimize`: feature vector → decision tree → ordered
  ≤3-format shortlist → per-candidate solve via the existing search → winner rule (smallest
  meeting target, beats source, deterministic tie-breaks). `--profile web|docs|preserve`.
  **(SPEC-048)**
- `--explain` (+ `--explain=json`): the decision trace, human-readable to stderr and a
  machine-readable variant (reused later as a manifest field). **(SPEC-049)**

### Explicitly out of scope
- The declarative goal-solver / planner (`max_size`/`min_quality` as a first-class goal object) —
  PROJ-003 generalizes this stage's engine.
- Indexed/lossy-PNG output (permissive quantizer, PROJ-007) — row-B graphics use lossless WebP.
- Applying auto-decide to `shrink`/`convert`/`responsive` — v1 targets `optimize`; generalize
  later. (Pinned-format commands are unaffected by definition.)

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-048 (design) — format auto-decision in `optimize`: `Analysis`-driven decision tree +
  ≤3 candidate shortlist + per-candidate solve over the existing SSIMULACRA2 search/`LossyFormat`
  seam + winner rule + `--profile web|docs|preserve`; engine + profiles + AVIF-byte-budget-only
  recorded in DEC-048
- [ ] SPEC-049 (design) — `--explain` trace (features/class/candidates/winner/savings), human +
  `--explain=json`; the `ExplainTrace` type (schema in DEC-049) reused later as a manifest field

**Count:** 0 shipped / 0 active / 2 pending (both specs designed ahead; await STAGE-011)

## Design Notes

- **Compose, don't reinvent** (`docs/research/proj-002-design-format-engine.md`): per candidate,
  call the existing `auto_quality`/`fit_under_size`; lossless candidates are a single encode
  (`score=100`, always meet a perceptual target). Winner = smallest bytes meeting the target that
  also beats source; tie-breaks smaller-bytes → shortlist-order → source-format (fully
  deterministic, integer bytes).
- **AVIF asymmetry falls out for free:** AVIF satisfies `supports_lossy_quality` but not
  `supports_perceptual_quality` (no decoder, DEC-020) → the shortlist builder only adds it in
  byte-budget mode. Gate each candidate on the mode's predicate; no special case.
- **Search-cost control:** shortlist ≤3; reuse the capped (≤8-iter) search; lossless = single
  encode; fast paths (tiny images skip the shortlist; row-B tries lossless-WebP first and stops
  if it beats source). All thresholds in one `DecisionPolicy` consts block.
- Weighty decisions → DEC-*: the decision-tree thresholds/profiles, and the `ExplainTrace`
  schema (kept serde-shaped but hand-rolled to JSON in v1 — no `serde_json` runtime dep yet).
- **Auto-decide is the DEFAULT for `optimize`** (the marquee) — decided 2026-07-05. **Migration is a
  non-issue** (no released users yet), so design for the best forward behaviour, not back-compat.
  The `FORMAT_SWITCH_THRESHOLD` clear-win guard (above) prevents surprising format switches; the
  explicit overrides are `--format`/`-o <ext>` (pin, bypasses the engine) and `--profile preserve`
  (force format-preserving). Put the threshold in the `DecisionPolicy` consts block.

## Dependencies

### Depends on
- **STAGE-011** — the `Analysis` layer (feature vector + `OptBucket`) the decision reads.
- STAGE-008 (PROJ-001) — `src/quality/` (SSIMULACRA2 search, `fit_under_size`, `LossyFormat`
  seam) and the DEC-016 encode-byte-sync contract, composed unchanged.
- DEC-004 (codec gating / exit 4), DEC-015 (format precedence — the pin bypass), DEC-020/021/022.

### Enables
- PROJ-003 (planner) — generalizes this stage's decision engine into the goal-driven solver.
- PROJ-004 (lint) — the `format/legacy-format` and `indexed-png-opportunity` rules reuse the
  decision engine; `explain` output patterns reused.
- PROJ-005 (manifest) — the `ExplainTrace` becomes the per-image `optimization` field.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
