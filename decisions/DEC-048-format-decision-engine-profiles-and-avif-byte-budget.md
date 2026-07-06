---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-048                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.75                   # 0.0 - 1.0, honest assessment
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-07-05
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/quality/decide.rs
  - src/cli/mod.rs

tags:
  - optimize
  - format-decision
  - profiles
  - avif
  - winner-rule
  - planner-seam
---

# DEC-048: format auto-decision engine — decision tree, web/docs/preserve profiles, winner rule, AVIF-byte-budget-only, and the planner seam

## Decision

`optimize` gains a **candidate-format decision engine** that fires only when the user has **not**
pinned a format (`--format`/`-o` absent) and `--profile != preserve`. It (A) derives a
`FeatureVector` from the `Analysis` layer, (B) shortlists **≤3** built + capability-valid candidate
formats via a **deterministic decision tree** (rows A–F of
`docs/research/proj-002-design-format-engine.md`), (C) solves each candidate by **reusing
`src/quality/` unchanged** (`auto_quality` / `fit_under_size`; lossless = single encode,
`score=100`), and (D) picks the **winner = smallest bytes that meets the target AND beats the
source** (tie-breaks: smaller bytes → shortlist order → source format), **switching to a format
different from the source only when the byte win clears a `FORMAT_SWITCH_THRESHOLD` clear-win
guard** (else the source format is kept — no surprising switch for a marginal gain). Auto-decide is
the **new default** for `optimize` (no back-compat obligation — no released users), and the format
choice is **never silent**: a one-line summary (chosen format + savings %) prints to stderr by
default (`--quiet` suppresses; `--explain` gives the full trace). `--profile <web|docs|preserve>`
selects the bias: **`web`** (default) appends **AVIF last, only in byte-budget mode and only when
the `avif` feature is built**; **`docs`** widens the graphic/lossless bias; **`preserve`** == today's
format-preserving `optimize`, byte-identical (engine off). Never emits a file larger than the
source (else passthrough, exit 0). The engine is factored as **pure, `sink`-free Phase A/B/D
functions** so the PROJ-003 planner **wraps** it — *one decision engine, two entry points* — rather
than building a second format loop.

## Context

DEC-024 shipped `optimize` as format-*preserving*. The validated differentiator (Cloudinary
`f_auto` adoption, `docs/research/proj-002-findings.md §5`) is the tool *choosing* the format.
STAGE-012 (SPEC-048) delivers the local, deterministic, explained version. The design questions
this DEC settles:

1. How does the engine choose a shortlist, and how big can it get (search-cost)?
2. What are the profiles, and how does `preserve` stay a clean regression anchor?
3. When does AVIF appear, given it has no decoder (DEC-020)?
4. What is the winner rule + tie-break order (determinism)?
5. How do we design this so the future planner (PROJ-003) doesn't duplicate it?
6. How do we avoid a *surprising* format switch when the win is marginal, and do we tell the user?
   (Maintainer decision, 2026-07-05: a `FORMAT_SWITCH_THRESHOLD` clear-win guard, and always report
   the chosen format + savings — auto-decide is the default, so it must be legible, not silent.)

Constraints in play: `no-new-top-level-deps-without-decision` / `no-agpl-default-deps` (pure
orchestration, no new dep); `untrusted-input-hardening` (bounded, no panic, unsatisfiable →
best-effort exit 0); `ergonomic-defaults` (the no-flag `web` default must be right); DEC-016 (the
measured candidate bytes == the shipped file); DEC-004/015/020/021/022 (codec gating, pin
precedence, the AVIF/WebP capability profiles).

## Alternatives Considered

- **Option A: a special-case AVIF branch (try AVIF whenever built).**
  - What it is: explicitly add AVIF to perceptual shortlists and score it somehow.
  - Why rejected: AVIF has no built-in decoder (DEC-020), so the perceptual search *cannot score*
    an AVIF round-trip. Forcing it would either lie about quality or need a decoder we don't ship.
    The clean rule falls out of the existing `LossyFormat` seam for free — see Option C.

- **Option B: build the general goal-solver (planner) now and make `optimize` a thin caller.**
  - What it is: ship the declarative `max_size`/`min_quality` objective engine in this stage.
  - Why rejected: that is PROJ-003's scope (confirmed 2026-07-05). But we *do* take its lesson —
    factor the engine so the planner wraps it (Option C's seam) instead of forcing a later
    rewrite (`proj-002-findings.md §9`, `-design-planner.md §Composition`).

- **Option C (chosen): deterministic tree + ≤3 shortlist + reuse-the-search + winner rule +
  profiles + predicate-gated AVIF + pure wrap-seam.**
  - What it is: the design-brief engine — a thin orchestrator over `src/quality/`, gating each
    candidate on the active mode's `LossyFormat` predicate (`supports_perceptual_quality` excludes
    AVIF; `supports_lossy_quality` includes it → AVIF only in byte-budget mode, no special case),
    with `web|docs|preserve` profiles and a fully deterministic winner rule, factored as pure
    Phase A/B/D functions the planner reuses.
  - Why selected: differentiating *and* cheap — no new search math, no new dependency; `preserve`
    is a byte-identical regression anchor; the AVIF asymmetry needs zero special-casing; and the
    seam means PROJ-003 generalizes instead of duplicating.

## Consequences

- **Positive:** the 0.3.0 headline ("give it the file; it decides and tells you why") ships as pure
  composition of shipped, tested code; `cargo deny` unchanged; `preserve` guarantees no regression;
  the planner (PROJ-003) inherits a clean seam; AVIF-in-budget-only is enforced by an existing
  predicate, not a fragile branch.
- **Negative:** worst-case work is ~≤3 candidates × the capped search (bounded, but more encodes
  than today's single-format path) — mitigated by the fast paths (tiny images skip the shortlist;
  row-B tries lossless-WebP first and stops if it beats source). Row-B indexed-PNG savings are left
  on the table until a permissive quantizer lands (PROJ-007) — interim uses lossless WebP.
- **Neutral:** v1 targets `optimize` only; `shrink`/`convert`/`responsive` keep their current
  (pinned-format) behaviour. AVIF perceptual scoring remains blocked on AVIF decode (DEC-020).
- **Neutral:** the `FORMAT_SWITCH_THRESHOLD` guard trades a few marginal cross-format byte wins for
  predictable output extensions; it is a `DecisionPolicy` const, tunable if the default proves too
  conservative. Because there are no released users, this is a pure forward-design choice, not a
  migration.

## Validation

- **Right if:** `optimize photo.png` (web) reliably ships the smaller of the shortlisted candidates
  at equal perceptual quality, beats the source or leaves it unchanged, `--profile preserve` is
  byte-identical to legacy, and the winner is deterministic across runs/platforms — while
  `cargo deny` and the lean `--no-default-features` build stay green.
- **Revisit when:** AVIF decode lands (AVIF becomes a perceptual candidate, not budget-only); a
  permissive quantizer lands (row-B gains indexed-PNG); the planner (PROJ-003) is built (confirm it
  wraps `format_shortlist`/`pick_winner` rather than duplicating); or profiling shows the shortlist
  cost hurts large batches (tighten the fast paths).

## References

- Related specs: SPEC-048 (this engine), SPEC-049 (`--explain` renders its decision record),
  SPEC-046/047 (the `Analysis`/`opt_bucket` it reads), SPEC-016/017/022 (the search + `optimize` it
  composes)
- Related decisions: DEC-024 (optimize shape / preserve anchor), DEC-016 (encode byte-sync),
  DEC-019 (SSIMULACRA2 search), DEC-020 (AVIF no-decoder → budget-only), DEC-021/022 (WebP
  profiles), DEC-015 (pin precedence + partial batch), DEC-004 (codec gating / exit 4), DEC-047
  (the classifier verdict it switches on)
- External docs: `docs/research/proj-002-design-format-engine.md`,
  `docs/research/proj-002-design-planner.md` §Composition (the wrap seam),
  `docs/research/proj-002-findings.md §9`
- Discussions: PROJ-002 framing session 2026-07-05
