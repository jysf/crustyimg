---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-048
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: L                    # L: the decision engine + CLI wiring + winner rule
                                   # (the ExplainTrace surface is split out to SPEC-049)

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-05

references:
  decisions: [DEC-004, DEC-015, DEC-016, DEC-019, DEC-020, DEC-021, DEC-022, DEC-024, DEC-047, DEC-048]
  constraints: [untrusted-input-hardening, no-agpl-default-deps, no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-046, SPEC-047, SPEC-016, SPEC-017, SPEC-022]

value_link: >
  Ships STAGE-012's headline: `optimize` stops preserving the input's format and starts
  CHOOSING the best one automatically — "the local f_auto" — the 0.3.0 differentiator.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-048: format auto-decision in `optimize` — "the local f_auto"

## Context

Today `optimize` (DEC-024, `src/cli/mod.rs:238`/`:2763`) is **format-preserving**: auto-orient,
optional `--max`, auto-tune *quality only* against a visually-lossless target, write the input's
own format. The validated differentiator (Cloudinary `f_auto` adoption) is the tool *choosing* the
format. This spec inserts **one new step** between decode and the quality search —
**candidate-format selection** — that reads the STAGE-011 `Analysis` (`opt_bucket` + `has_alpha` +
`unique_colors` + `flat_ratio`), shortlists ≤3 candidate formats via a deterministic decision
tree, drives the *already-shipped* SSIMULACRA2 perceptual search (and the `--max-size` byte-budget
path) across them, and ships the smallest artifact that meets the target while **never emitting one
larger than the source**.

Crucially, this is a **thin orchestrator over `src/quality/`** — no new search math, no new
dependency. It composes `auto_quality` / `fit_under_size` / the `LossyFormat` two-predicate seam
exactly as shipped. It also sets the seam the PROJ-003 planner will **wrap, not rebuild**: one
"decision engine," two entry points (`optimize` = auto-decide default; the future `plan` =
explicit goal). See `docs/research/proj-002-design-format-engine.md` (this spec) and
`-design-planner.md` §Composition (the wrap seam), reconciled in `proj-002-findings.md §9`.

The `--explain` surface is **split out to SPEC-049** — this spec produces the internal decision
record (candidate array + winner + reason); SPEC-049 renders it. The decision-tree thresholds, the
`--profile` set, the winner rule, and the AVIF-byte-budget-only rule are recorded in **DEC-048**.

## Goal

When the user has **not** pinned a format, make `optimize <file>` auto-decide the output format +
lossy/lossless disposition from the `Analysis` layer, solve each shortlisted candidate through the
existing search, and ship the smallest artifact that meets the perceptual target (or byte budget)
and beats the source — deterministically, with a `--profile <web|docs|preserve>` bias
(`preserve` == today's behaviour, byte-identical) and adding no dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-format-engine.md` — the authoritative decision tree (rows
    A–F), the composition-with-search contract, the winner rule + tie-breaks, the search-cost
    fast paths, the edge-cases→exit table.
  - `docs/research/proj-002-design-planner.md` §"Composition" + §"Reuse verbatim" — the seam the
    planner will wrap; design Phase A/B/D as pure, testable functions with a `Candidate` struct so
    PROJ-003 generalizes rather than duplicates.
  - `src/quality/mod.rs` — reuse **verbatim**: `auto_quality` (`:431`), `fit_under_size` (`:529`),
    `SizeFit` (`:470`), `QualityChoice` (`:148`), `SearchConfig` (`:110`), the `LossyFormat`
    predicates (`:304` — `supports_lossy_quality` vs `supports_perceptual_quality`).
  - `src/analysis/mod.rs` (SPEC-046/047) — read `opt_bucket`, `has_alpha`, `unique_colors`,
    `flat_ratio`, `dims`; the engine switches on these, it does not re-scan pixels.
  - `src/cli/mod.rs` — `Optimize` args (`:238`), `run_optimize` (`:2763`), `optimize_auto_config`
    (`:2722`), `output_format_for` (`:1477` — the pin-bypass precedence), the `run_pixel_op`
    fan-out.
  - `src/sink/mod.rs:250` — `CodecNotBuilt → exit 4` (DEC-004), the pinned-unbuilt-codec path.
- **Related code paths:** `src/quality/` is the composition target; do **not** change any
  `search_*`/`auto_*`/`fit_under_size`/`score`/`LossyFormat` signature.

## Outputs

- **Files created:** a decision-engine module — `src/quality/decide.rs` (or `src/analysis/decide.rs`)
  holding the pure Phase A/B/D logic: `FeatureVector` derivation from `Analysis`, `format_shortlist`,
  `Candidate`, and `pick_winner`. It depends only on `::image` + `crate::quality` + `crate::analysis`
  (no `clap`/`sink`/`files`) so the planner can reuse it. **The built+capability-valid format set is
  passed in from the CLI** (keeps the engine free of `sink`), mirroring the planner brief's seam.
- **Files modified:** `src/cli/mod.rs` — add `--profile <web|docs|preserve>` to `Optimize`; in
  `run_optimize`, when no format is pinned and `profile != preserve`, run the engine to select the
  format, then route the winner through the existing sink/fan-out. `preserve` and pinned-format
  paths call the *existing* code unchanged.
- **New exports:**
  - `pub enum Profile { Web, Docs, Preserve }` (default `Web`).
  - `pub struct Candidate { fmt, disposition, bytes, score, met_target, quality }`.
  - `pub fn format_shortlist(analysis, profile, mode, built_valid: &[ImageFormat]) -> Vec<ImageFormat>`
    (≤3, ordered, unbuilt/invalid dropped **before** searching).
  - `pub fn pick_winner(candidates, source_bytes, mode) -> Option<&Candidate>` (the winner rule).
  - A single `DecisionPolicy` consts block (all thresholds: `photo_score` cutoffs, `flat` gates,
    palette gate, fast-path sizes, and the **`FORMAT_SWITCH_THRESHOLD`** clear-win guard) — the
    DEC-048 tuning surface.
  - A default **one-line result summary** (chosen format + savings %) written to **stderr** after
    the decision — `optimize` is not fully silent; the format choice is always reported. `--quiet`
    suppresses it; `--explain` (SPEC-049) replaces it with the full trace.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `optimize photo.png` with **no `--format`** produces the smaller of the shortlisted
  candidates at equal perceptual quality, beating the source — or, if no candidate beats the
  source, **leaves the file unchanged / passes through** (never emits a larger file), exit 0.
- [ ] `optimize --profile preserve <file>` is **byte-identical** to today's `optimize <file>` (the
  regression anchor — the engine is bypassed entirely).
- [ ] Pinning `--format`/`-o <ext>` **bypasses the engine** (deterministic escape hatch); the
  existing `output_format_for` precedence (DEC-015) is unchanged.
- [ ] The shortlist is ≤3, ordered, and contains **only built + capability-valid** candidates
  (unbuilt codecs dropped before any encode); if the tree proposes only unbuildable candidates it
  falls back to lossless PNG (always built).
- [ ] **AVIF appears as a candidate only in byte-budget mode** and only when the `avif` feature is
  built — this falls out of gating each candidate on the mode's `LossyFormat` predicate
  (`supports_perceptual_quality` excludes AVIF, DEC-020; `supports_lossy_quality` includes it). No
  AVIF special-case.
- [ ] Per-candidate solve **reuses `src/quality/` unchanged**: lossy+perceptual → `auto_quality`;
  lossy+size → `fit_under_size`; lossless → single encode with `score=100, met_target=true`
  (perceptual) or `met = bytes ≤ budget` (size). Measured bytes use the sink's exact encode path
  (DEC-016) so the winner's bytes == the shipped file.
- [ ] **Winner rule (perceptual/default):** among `met_target` candidates, smallest bytes that also
  beats source bytes; tie-breaks (deterministic): smaller bytes → earlier shortlist order → source
  format. **Size mode:** smallest fitting bytes; if none fit, globally smallest best-effort with
  `met=false` surfaced (exit 0).
- [ ] **Clear-win guard (`FORMAT_SWITCH_THRESHOLD`):** the output switches to a format *different
  from the source* only when its byte win over the best same-format candidate clears
  `FORMAT_SWITCH_THRESHOLD`; otherwise the source format is kept (no surprising switch for a
  marginal gain). A winning same-format re-encode, or passthrough when nothing beats source, is
  unaffected. The threshold lives in `DecisionPolicy` and is recorded in DEC-048.
- [ ] **Always-report:** after deciding, `optimize` prints a **one-line summary** (chosen format +
  savings %) to stderr by default — the format choice is never silent. `--quiet` suppresses it;
  `--explain` (SPEC-049) supersedes it with the full trace. (No back-compat constraint — auto-decide
  is the new default; `--profile preserve` is the only format-preserving escape besides a pin.)
- [ ] Determinism: identical `(pixels, profile, feature-flags, mode)` ⇒ identical output (integer
  byte comparisons; all thresholds in the `DecisionPolicy` block; no RNG/wall-clock).
- [ ] Bounded work: shortlist ≤3 × capped search (≤`MAX_SEARCH_ITERS=8`); lossless = single
  encode; documented fast paths (tiny images skip the shortlist and byte-compare only; row-B
  few-colour tries lossless-WebP first and stops if it beats source). A top-level assertion bounds
  the in-memory encode count.
- [ ] `just deny` green; **no new default dependency**; 3-OS CI green (including the lean
  `--no-default-features` build — the engine must compile and degrade when lossy-WebP/AVIF features
  are absent); STAGE-006 hardening upheld (no panic on any input; unsatisfiable target →
  best-effort + exit 0).

## Failing Tests

Written during **design**, BEFORE build. The **Phase A/B/D logic is pure** and unit-tested with
synthetic `Candidate`s (exactly like the existing `src/quality` search tests) — no encoding needed
to test the winner rule. Integration tests exercise the real `optimize` CLI on generated fixtures.

- **`src/quality/decide.rs` (unit tests — pure logic, synthetic candidates)**
  - `"winner: smallest met candidate that beats source wins"` — three synthetic candidates, one
    smaller & met → it wins.
  - `"winner: none beats source → None (passthrough)"` — all candidates ≥ source bytes → `None`
    (the unchanged-file path).
  - `"winner tie-break: equal bytes → earlier shortlist order"` — two equal-byte met candidates →
    the earlier one wins; a third test adds source-format as the final tie-break.
  - `"winner: unmet perceptual target excluded"` — a candidate with `met_target=false` is never
    chosen even if smallest.
  - `"clear-win guard: cross-format win below threshold keeps source format"` — a different-format
    candidate that beats source but by < `FORMAT_SWITCH_THRESHOLD` over the best same-format
    candidate → source format kept; a second case with the win ≥ threshold → the switch is taken.
  - `"size mode: smallest fitting; none fit → smallest best-effort met=false"`.
  - `"shortlist: photo (Lossy bucket, no alpha) → [WebP(lossy), JPEG]"` (row E) and
    `"shortlist: few-colour graphic (palette gate) → [WebP(lossless), PNG]"` (row B).
  - `"shortlist: alpha + photographic → lossy RGB keeps alpha, JPEG excluded"` (row D).
  - `"shortlist: AVIF only in size mode + only when built"` — perceptual mode → no AVIF; size mode
    with `avif` built → AVIF appended last; unbuilt → absent (cfg-gated test).
  - `"shortlist: unbuildable proposals fall back to lossless PNG"`.
  - `"shortlist ≤ 3 always"` — property over each row.
- **`tests/optimize_auto.rs` (integration — real CLI on generated fixtures)**
  - `"optimize --profile preserve == legacy optimize (byte-identical)"` — the regression anchor.
  - `"optimize (web) on a synthetic photo re-encodes smaller, correctly-oriented, exit 0"`.
  - `"optimize (web) on a flat few-colour PNG chooses a lossless candidate, never smears"`.
  - `"optimize on an already-optimal tiny file leaves it unchanged, exit 0"`.
  - `"optimize prints a one-line format+savings summary to stderr by default; --quiet silences it"`.
  - `"pinned --format bypasses the engine (output is the pinned format)"`.
  - `"pinned unbuilt codec → exit 4 (DEC-004)"`.
  - `"determinism: two runs of optimize (web) on the same input → identical bytes"`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-048` (emitted with this spec) — the decision-tree thresholds, the `web|docs|preserve`
  profile set, the winner rule + tie-breaks, and the **AVIF-byte-budget-only** rule. Also records
  the "one decision engine, two entry points" seam that PROJ-003's planner wraps.
- `DEC-024` — `optimize`'s current shape (the `preserve` regression anchor is exactly this).
- `DEC-016` — encode byte-for-byte sync: measured candidate bytes == shipped file bytes; reuse the
  sink's encode path, don't re-encode differently.
- `DEC-019` — the SSIMULACRA2 perceptual search this composes; **do not** re-implement it.
- `DEC-020/021/022` — AVIF output-only (no decoder → no perceptual scoring), WebP lossless default
  + lossy-WebP feature. These drive the `LossyFormat` predicates the shortlist gates on.
- `DEC-015` — output-format precedence (the pin bypass) + partial-batch exit 6.
- `DEC-004` — codec gating / exit 4 for a pinned unbuilt codec.
- `DEC-047` — `opt_bucket` is the classifier verdict the tree switches on.

### Constraints that apply
- `untrusted-input-hardening` — the engine runs on untrusted input: no panic; unsatisfiable
  target → best-effort + exit 0; every loop bounded (shortlist ≤3, search ≤8).
- `no-agpl-default-deps` / `no-new-top-level-deps-without-decision` — pure orchestration over
  shipped code; no new dependency.
- `ergonomic-defaults` — the no-flag `optimize` must be *right by default* (web profile); the
  engine only fires when the user hasn't pinned a format.
- `test-before-implementation` — the pure winner-rule tests are the contract; write them first.

### Prior related work
- `SPEC-016`/`SPEC-017`/`SPEC-022` (shipped) — the perceptual search, `--max-size` dimension
  fallback, and `optimize` command this composes. `SPEC-046`/`SPEC-047` (this project) — the
  `Analysis`/`opt_bucket` the engine reads.

### Out of scope (for this spec specifically)
- The `--explain` surface — **SPEC-049** (renders the `Candidate` array + winner this spec
  produces). Design the internal decision record so SPEC-049 only formats it.
- The declarative goal-solver / planner (`max_size`/`min_quality` as a goal object) — PROJ-003
  *wraps* this engine; do not build a second format loop here.
- Indexed/lossy-PNG output (needs a permissive quantizer, deferred PROJ-007) — row-B few-colour
  graphics use **lossless WebP**; PNG stays full-colour.
- Applying auto-decide to `shrink`/`convert`/`responsive` — v1 targets `optimize` only.

## Notes for the Implementer

- **Compose, don't reinvent.** If you find yourself writing a quality loop, stop — call
  `auto_quality`/`fit_under_size`. The only new logic is Phase A (feature vector from `Analysis`),
  Phase B (shortlist), and Phase D (winner) — all pure and unit-testable without encoding.
- The AVIF asymmetry needs **no special case**: gate every candidate on the active mode's
  `LossyFormat` predicate and AVIF-in-perceptual-mode simply reports `false` and is dropped.
- Keep the `Candidate` struct and `pick_winner`/`format_shortlist` signatures clean and
  `sink`-free — the planner (PROJ-003) will call them with a goal object instead of a profile. The
  "built + capability-valid format set" is passed **in** from the CLI; the engine never touches
  `sink`.
- All thresholds (photo_score cutoffs, flat/palette gates, fast-path sizes) live in one
  `DecisionPolicy` consts block referenced by DEC-048 — one tuning surface, like
  `MAX_SEARCH_ITERS`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
