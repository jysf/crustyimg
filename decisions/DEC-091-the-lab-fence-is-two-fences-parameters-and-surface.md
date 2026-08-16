---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-091
  type: decision
  confidence: 0.78
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: 89513347-e079-4757-a1b7-02ba6aa40bb4

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-15
supersedes: null
superseded_by: null

affected_scope:
  - "docs/lab-plan-2026-08.md"
  - "docs/feature-set-triage-2026-08.md"
  - "docs/territory.md"

tags:
  - positioning
  - scope
  - lab
---

# DEC-091: the lab fence is two fences — parameter admissibility and surface admissibility

## Decision

**DEC-088's generalization fence is kept, restated at the level it actually operates on
(parameters), and paired with a second fence for the surface it cannot reach.**

> **Fence A — parameter admissibility.** A parameter is workhorse-admissible if its *meaning* is
> invariant under substituting a different image. Absolute pixel coordinates are not. Ratios,
> percentages, gravities, and value-space thresholds (luminance / chroma / alpha) are.
> **Whether the best *value* differs per image is irrelevant — that is what a search is for.**

> **Fence B — surface admissibility.** The workhorse emits **artifacts a build consumes**. Lab
> emits **decisions a human acts on** — and every lab decision must be expressible as a workhorse
> artifact.

This is **not a widening of DEC-088.** Fence A is DEC-088's fence with its domain named; Fence B
covers what Fence A returns "not applicable" for. DEC-088's substance — lab may produce anything,
the workhorse accepts only what generalizes, lab recipes are a superset, exit 4 on a lab-only op —
is unchanged.

## Context

DEC-088 fixed the fence before the first lab spec and stated its own success test: *"Right if: a
proposed lab feature can be placed on the correct side of the fence without a scope argument."*
That test was run against all 18 items in `docs/feature-set-triage-2026-08.md`. Two things came
back.

### 1. The fence adjudicates parameters, but is being asked about features

DEC-088's fence is derived from one case — masks — and it works there exactly as advertised:
absolute rect → lab, percent/gravity rect → workhorse; seed-flood coordinate → lab, luminance
threshold → workhorse. That derivation is sound and is why the fence should be kept.

But what it tests is a **parameter**: *does this value still mean something applied to a different
image?* **Thirteen of the eighteen items are not ops at all** — they are lints, sinks, codecs,
interfaces, test harnesses, and a bot. For those the fence returns "not applicable", not a side.
A test that cannot answer for 13 of 18 cases is not failing; it is being asked out of domain.

### 2. Three of the seven lab-blocked items are placed on the *workhorse* side

This is the sharper finding, and it is a contradiction rather than an ambiguity:

| item | what the fence actually says | where the triage put it |
|---|---|---|
| **§8 expression filters** | `r*0.5+0.2` is size-independent and means the same on every image → **generalizes → workhorse** | lab |
| **§9 parameter sweeps** | sweeping quality 60→90 is meaningful on any image; the workhorse already runs a quality search (`src/quality/mod.rs:255`) → **generalizes → workhorse** | lab |
| **§10 watch-preview** | watching a file and re-running generalizes fine — **`build --watch` already ships in the workhorse** (`watch` feature, default-on) | lab |

All three are lab-side for real reasons — an unbounded evaluation surface, a human-facing contact
sheet, an interactive loop — but **none of those reasons is "it doesn't generalize."** §8 is the
clearest tell: its headline design, *bake to `.cube`*, exists precisely to move it across the
fence, which concedes the fence was never what excluded it.

The same slip appears in the RAW-develop split handed in alongside: **exposure compensation is a
scalar EV value** and is perfectly meaningful across a batch, so Fence A places it in the
workhorse, not lab. (The objection "the right value differs per image" proves too much — the right
*quality* differs per image too, and quality is a workhorse parameter the engine searches for.
Hence Fence A's second sentence.) Crop splits exactly as masks do, which is what `Gravity`'s own
doc already anticipates (`src/operation/mod.rs:641`).

### Why this is an amendment and not a reversal

DEC-088's *"wrong if"* clause names two failure modes. Neither has fired:

- *"a genuinely useful op that generalizes across a batch is nonetheless unimplementable in the
  workhorse"* — not observed.
- *"a lab-only op becomes so commonly wanted in `build` that exit 4 is mostly friction"* — not
  observed; nothing is built yet.

What fired is the *"right if"* clause, partially: placement without a scope argument works for ops
and does not work for non-ops. The fix is a companion rule, not a wider fence — and DEC-088's
third clause (*"also wrong if the shared core needs constant `pub` widening"*) came back
**measured at zero** (`docs/lab-plan-2026-08.md` §F1), which is the strongest evidence the
underlying architecture is right.

## Alternatives Considered

- **Option A: widen Fence A to cover surface.** Rejected. Widening the one fence that is
  *derived* rather than asserted is how it stops being derived. DEC-088 rejected "decide per
  feature" for exactly this reason: *"deciding case by case is how an anti-goal erodes silently."*

- **Option B: leave DEC-088 alone and resolve each case by argument.** Rejected for the same
  reason, and because the contradictions above are already in a committed document — the next
  session to read the triage will meet three items filed on the side the standing fence does not
  put them.

- **Option C: reclassify §8/§9/§10 as workhorse features, honouring the fence literally.**
  Rejected. It is the fence's letter against everyone's judgment, and it would put an unbounded
  expression evaluator on the default path — the opposite of what the fence exists to protect.

- **Option D (chosen): keep Fence A, name its domain, add Fence B.** Fence B is derived rather
  than asserted — it falls out of the lab thesis (*lab is where you find the recipe; the workhorse
  is where you run it*), so it has the property that made Fence A worth keeping.

## Consequences

- **Positive.** Every one of the 18 items now places without a scope argument
  (`docs/lab-plan-2026-08.md` §2.5). The lab-only set turns out to be **small** — masks' absolute
  half, sweeps, watch-preview, an undo stack, and expression *authoring*. Most of the 18 items are
  workhorse features that were parked behind "the lab decision" because lab was the open question,
  not because they belong to lab. Three of them are unblocked by this DEC and homed in
  `docs/backlog.md` in the same change.
- **Negative.** Two rules to apply instead of one, and Fence B is newer and less battle-tested
  than Fence A. It has been applied to 18 items once, by one session.
- **Neutral.** `docs/feature-set-triage-2026-08.md` keeps its original placements; this DEC is the
  correction record, following the DEC-087 pattern of showing the call and its correction rather
  than silently editing. DEC-088's confidence (0.86) is unchanged — its mechanism verified; only
  its scope test needed a companion.

## Validation

**Right if** all three hold: a proposed lab feature places under A-then-B without a scope
argument; the lab-only set stays small (an item migrating *from* lab to the workhorse is the fence
working, not failing); and no feature needs a third fence.

**Wrong if** Fence B turns out to be a restatement of "we feel like it" — i.e. if applying it
requires arguing about who the consumer is more often than it settles the question. The tell would
be a feature whose output is plainly a build artifact but which everyone still wants in lab, or
vice versa.

**Also wrong if** the exposure-compensation call above proves unworkable in practice — if RAW
users need per-image exposure so consistently that shipping it workhorse-side is friction rather
than capability. That would mean Fence A's second sentence ("the best value differing per image is
irrelevant") is too strong, and the fence needs re-deriving, not widening.

**Confidence 0.78**, below DEC-088's 0.86 on purpose: Fence A's restatement is well-evidenced
(measured contradictions in three items), but Fence B is a design judgment applied once. Per
AGENTS.md §17 this is above the 0.7 line that would require a `questions.yaml` entry, but it
should be revisited after the first two lab stages, not treated as locked.

## References

- `DEC-088` — the decision this amends; its fence, its three "wrong if" clauses, its tier model.
- `docs/lab-plan-2026-08.md` — §2 derives this DEC; §2.5 is the 18-item placement table; §F1
  records the measured zero-widening result that clears DEC-088's third clause.
- `docs/feature-set-triage-2026-08.md` — §2 (the fence as first stated), §3 (the placements this
  DEC corrects).
- `docs/territory.md` — the three-tier scope discipline; line 40 (amended by DEC-088).
- `src/quality/mod.rs:255` — the search that makes Fence A's second sentence concrete.
- `src/operation/mod.rs:641` — `Gravity`, whose doc already anticipates the crop split.
- `DEC-087` — the show-the-correction precedent this DEC follows.
