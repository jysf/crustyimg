---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-110
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-039
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the decision is made, the
                                   # sweep is already driven and inlined, and the
                                   # change is threading one existing operation.
                                   # Verify stays on Opus.
  created_at: 2026-08-03

references:
  decisions:
    - DEC-003
    - DEC-017
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
  related_specs:
    - SPEC-107
    - SPEC-108
    - SPEC-111

value_link: >
  STAGE-039's D-1: stop the shipped verbs handing back a sideways image, and
  make one orientation rule true across the whole pixel lane instead of six
  verbs being wrong by accident.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Drove a purpose-built
        Orientation=6 fixture through every pixel-lane verb on a release build
        and read output dimensions + EXIF presence; the full table is in the
        Context. Also audited the five existing callers of the orientation
        fixture builders to establish test fallout.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 61879452
      duration_minutes: 1118
      recorded_at: 2026-08-04
      tokens_breakdown:
        input: 530
        output: 235030
        cache_creation: 2192141
        cache_read: 59451751
      estimated_usd: 29.58
      note: >
        MEASURED — summed .message.usage across 265 assistant messages in this
        session's own transcript (not dispatched as a subagent). duration_minutes
        is first-to-last transcript timestamp (~18h39m calendar span), which
        includes idle time between turns, not continuous active compute.
        estimated_usd priced per component at Sonnet anchors ($3/$15 per MTok
        in/out; cache_creation x1.25 input, cache_read x0.10 input) since
        claude-sonnet-5 is the model that actually ran.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 48955092
      duration_minutes: 1369
      recorded_at: 2026-08-06
      tokens_breakdown:
        input: 480
        output: 111068
        cache_creation: 1274948
        cache_read: 47568596
      estimated_usd: 20.72
      note: >
        Punch-list pass (second build session) — MEASURED, summed .message.usage
        across 240 unique assistant message ids in this session's own transcript
        (deduped: each API response appears as multiple JSONL lines, one per
        content block, all carrying the same usage snapshot; summing raw lines
        would overcount). Computed after the CI matrix went green, before this
        session's own closing messages, so the true total is a few messages
        higher than recorded here (this session cannot fully measure itself
        while still running). duration_minutes is first-to-last transcript
        timestamp, includes idle
        time between turns. estimated_usd priced per component at Sonnet anchors
        ($3/$15 per MTok in/out; cache_creation x1.25 input, cache_read x0.10
        input), same formula as the first build session.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-110: bake EXIF Orientation on every pixel-lane verb

## Context

STAGE-039 framed this as *"`convert` orientation: decide, fix, sweep"* — `run_convert`
(`src/cli/optimize.rs:507`) builds `Pipeline::new()` at `:538`, an empty pipeline, so the
pixel-lane re-encode drops the metadata bundle while the rotation it described is never
applied. **The sweep was driven at design, and it is where most of the defect lives.**

### Measured, `target/release/crustyimg` at `d854038`

A purpose-built JPEG, stored **1200×800** with a real one-entry `Orientation=6` IFD (built
the way `tests/common::wrap_with_orientation_app1` builds them). Orientation 6 means the
correct **display** size is **800×1200**.

| verb | output | expected if baked | | EXIF kept? |
|---|---|---|---|---|
| `convert --format png` | 1200×800 | 800×1200 | **not baked** | no |
| `convert --format jpeg` | 1200×800 | 800×1200 | **not baked** | no |
| `resize --max 600` | 600×400 | 400×600 | **not baked** | no |
| `thumbnail --size 300` | 300×200 | 200×300 | **not baked** | no |
| `edit --invert` | 1200×800 | 800×1200 | **not baked** | no |
| `edit --resize-max 600` | 600×400 | 400×600 | **not baked** | no |
| `responsive --widths 600` | 600×400 | 600×900 | **not baked** | no |
| `web` | 800×1200 | 800×1200 | baked ✓ | no |
| `optimize` | 800×1200 | 800×1200 | baked ✓ | no |
| `auto-orient` | 800×1200 | 800×1200 | baked ✓ | no |
| `edit --auto-orient` | 800×1200 | 800×1200 | baked ✓ | no |

**Seven invocations return a sideways image, and every one of them also drops the EXIF.**
That is the sharp edge: the information needed to correct the output is destroyed by the
same operation that made it wrong. A user cannot recover — the tag is gone and the pixels
were never rotated.

### The current split is not a design

Nothing distinguishes the two groups except which pipeline builder each verb happened to
call. `web` and `optimize` go through `optimize_pipeline()` (`src/cli/optimize.rs:781`),
which does `Pipeline::new().push(orient)` — auto-orient pinned first. Every other verb
builds its own pipeline without it. There is no rule that explains why `web` bakes and
`resize` does not, and **a rule nobody can state is a rule nobody maintains** — which is
how seven invocations drifted wrong without anyone noticing.

### DEC-003's own falsifiability condition is currently false

This is not only a bug; it is a decision record that has stopped describing the code.
DEC-003 chose the two-lane design and wrote its success test as:

> *"Right if: a resize preserves orientation + ICC + copyright and drops GPS by default"*

with the body asserting *"Orientation/ICC survive transforms."* `AGENTS.md:448` repeats it
in the glossary: default-preserve keeps orientation. **A `resize` today neither preserves
the tag nor bakes it.** Left alone, this is the same class of decay that made
`docs/launch-readiness.md` read red for two weeks while five of its blockers had shipped.

### Why no test caught it

Because none exists. All five callers of `jpeg_with_orientation` / `wrap_with_orientation_app1`
outside `tests/common` are on verbs that **already bake** — `tests/cli.rs:439/531/603`
(`auto-orient`), `:2902` (`optimize`), `:4110` (SPEC-108's classification fixture) — plus
`tests/lint.rs:351` and `src/lint/rules.rs`, which lint rather than transform. **No test
asserts orientation behaviour on `convert`, `resize`, `thumbnail`, `responsive`, or `edit`
at all.** [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]

### The decision

**Bake everywhere.** Maintainer decision, 2026-08-03. Pin the existing `auto-orient`
operation first on every pixel-lane verb, matching what `web`/`optimize` already do.

Rejected alternatives, recorded so they are not re-proposed:

- **Preserve the tag instead** (what DEC-003 literally says). More faithful on paper, and it
  keeps `convert` byte-faithful — but it needs a container-lane write on every pixel encode,
  per output format, and it still renders sideways in any viewer that ignores EXIF. Baking
  delivers the outcome the user actually wants: the picture looks right.
- **Split by verb intent** — bake for `thumbnail`/`responsive`/`resize`, preserve for
  `convert`. Defensible per-verb, but it is two rules to document and remember, and the seam
  is exactly where the next bug hides.

**The strongest objection is weaker than it looks.** "`convert` should be a byte-faithful
re-encode" is the real argument against — but `convert` **already discards all metadata**
on the re-encode, so it is not faithful in any archival sense today. Baking the rotation
into pixels arguably improves the fidelity of what survives, since the alternative in force
right now is that the rotation information is *destroyed* rather than applied or kept.

**Sub-decision: `edit --auto-orient` becomes an accepted no-op.** It cannot be removed — the
CLI surface was frozen in STAGE-030 — so it stays, is documented as "now the default", and
must keep exiting 0. **No opt-out flag is added.** There is no evidence of demand for
"give me the stored pixels", and adding an escape hatch deserves its own spec with a real
user behind it — the same reasoning DEC-063 used to file `--max-pixels` rather than build it.

## Goal

Make one orientation rule true across the entire pixel lane: every verb that re-encodes
pixels bakes EXIF Orientation first, so no shipped verb can hand back a sideways image.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs:781-798` — `optimize_pipeline()`, the builder that already pins
    `auto-orient` first. This is the shape to reuse, not to re-invent.
  - `src/cli/optimize.rs:507-542` — `run_convert` and its empty `Pipeline::new()` at `:538`.
  - `src/cli/ops.rs` — `run_pixel_op` and the `resize` / `thumbnail` / `edit` handlers.
  - the `responsive` handler (`src/cli/optimize.rs:~1600`, the third `pipeline.run` site
    SPEC-108's build identified).
  - `src/operation/mod.rs:629` `orientation_from_exif_segment`, and the `auto_orient_*` unit
    tests at `:1342-1380` — the operation's existing contract and coverage.
  - `decisions/DEC-003-*.md` — the default-preserve policy this spec resolves against the
    code; `decisions/DEC-017-*.md` — what `auto-orient` does.
  - `tests/common/mod.rs:102-172` — `jpeg_with_orientation` and
    `wrap_with_orientation_app1`, the fixture builders to use. **Do not hand-roll new ones.**
- **Related code paths:** `src/cli/`, `src/operation/`, `src/pipeline/`.

## Outputs

- **Files modified:**
  - `src/cli/optimize.rs` — `run_convert` and `responsive` pin `auto-orient` first.
  - `src/cli/ops.rs` — `resize`, `thumbnail`, `edit` likewise; `--auto-orient` becomes a
    no-op that still exits 0.
  - `docs/api-contract.md` — `convert`'s "pure re-encode (decode once, no pixel transform)"
    line, and the orientation behaviour of each affected verb.
  - `docs/cli-reference.md` — `edit --auto-orient`'s description.
  - `decisions/DEC-003-*.md` — an amendment section reconciling the record with the code.
  - `decisions/DEC-NNN-*.md` — **a new decision** for "bake on every pixel-lane verb",
    carrying the measured table and both rejected alternatives.
- **New exports:** none. Prefer factoring the existing `optimize_pipeline()` prefix into a
  shared helper over copying `.push(orient)` into six places.

## Acceptance Criteria

- [ ] **AC-1.** A JPEG stored 1200×800 with `Orientation=6` produces **800×1200** through
      `convert --format png`, `convert --format jpeg`, and `edit --invert`; **400×600**
      through `resize --max 600` and `edit --resize-max 600`; **200×300** through
      `thumbnail --size 300`; and **600×900** through `responsive --widths 600`. These are
      the exact cells the design measured as wrong.
- [ ] **AC-2.** The four already-correct paths are unchanged: `web`, `optimize`,
      `auto-orient` and `edit --auto-orient` still produce 800×1200. This is the
      regression guard against over-correcting — a double rotation is the obvious failure
      mode of this change and it would be invisible on a square fixture.
- [ ] **AC-3.** **Orientation 1 and no-EXIF inputs are byte-identical to before** on every
      affected verb. Baking must be a genuine no-op for the overwhelming majority of inputs;
      assert on output **bytes**, not just dimensions. This is what makes the change safe.
- [ ] **AC-4.** A **non-square** fixture is used throughout. A square input cannot
      distinguish "baked" from "not baked" from "baked twice", so a square fixture would
      make this entire spec vacuous. [[fixtures-from-the-code-under-test-cannot-fail]]
- [ ] **AC-5.** All eight orientation values 1–8 are driven through **one** representative
      verb, asserting dimensions where the value implies a 90° rotation (5,6,7,8 swap; 1,2,3,4
      do not). The current code applies none of them; a fix that handles only 6 is not a fix.
- [ ] **AC-6.** `edit --auto-orient` still parses and exits **0**, and is documented as now
      being the default. No flag is removed — the CLI surface is frozen (STAGE-030).
- [ ] **AC-7.** The `auto-orient` prefix is applied via a **shared helper**, not copied into
      each handler. A mechanical check on the sweep: cite the grep showing every pixel-lane
      pipeline construction site and state its scope as a claim.
      [[mechanical-sweeps-need-a-mechanical-check]]
- [ ] **AC-8.** `docs/api-contract.md` no longer calls `convert` a "pure re-encode … no pixel
      transform", and states the orientation behaviour for each affected verb.
      [[documentation-has-no-green]]
- [ ] **AC-9.** **DEC-003 is reconciled with the code.** Its "Right if: a resize preserves
      orientation" condition and its "Orientation/ICC survive transforms" claim are amended
      to describe what the code now does (bake, not preserve), with the amendment dated and
      the reasoning recorded. `AGENTS.md:448`'s glossary line is corrected to match.
- [ ] **AC-10.** A **negative control** proves the suite is load-bearing: reverting the
      `auto-orient` prefix on any one verb must turn at least one test **RED**. Record it.
      Remember that reverting source does not rebuild the binary.
      [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-11.** Clean **full-matrix** green from fresh per-leg `CARGO_TARGET_DIR`s, run
      sequentially: default, `--no-default-features`, `--features webp-lossy`; `clippy -D
      warnings` each; `fmt --check`; plus `just wasm-test`. Confirm each log says
      `Compiling crustyimg`. **Run every leg through `rtk proxy` from the first one** — rtk
      has collapsed `cargo test` output and deleted that very line.
      [[a-green-gate-on-one-os-is-not-the-required-matrix]] **Read the CI legs before
      claiming the matrix is clean** — a local macOS pass is not the required matrix.

## Failing Tests

Written during **design**, BEFORE build. Expected to FAIL against current `main` except
where noted.

- **`tests/cli.rs`** (or a new `tests/orientation.rs` if that file is unwieldy)
  - `"convert_bakes_orientation_into_pixels"` — AC-1, both target formats. **Fails today**
    (1200×800).
  - `"resize_bakes_orientation_before_bounding"` — AC-1. **Fails today** (600×400 not
    400×600). This one matters most: the bound is applied to the *wrong axis* today, so the
    output is not merely mis-tagged, it is the wrong size.
  - `"thumbnail_bakes_orientation"` — AC-1. **Fails today.**
  - `"responsive_bakes_orientation"` — AC-1. **Fails today** (600×400 not 600×900).
  - `"edit_bakes_orientation_without_the_flag"` — AC-1. **Fails today.**
  - `"already_correct_verbs_do_not_double_rotate"` — AC-2, covering `web`, `optimize`,
    `auto-orient`, `edit --auto-orient`. **Passes today**; it is the over-correction guard
    and must be written anyway.
  - `"orientation_1_and_no_exif_are_byte_identical"` — AC-3. **Passes today** vacuously;
    it becomes the safety proof once the change lands.
  - `"all_eight_orientation_values_are_applied"` — AC-5. **Fails today** for every value.
  - `"edit_auto_orient_flag_still_exits_zero"` — AC-6. **Passes today**; guards the frozen
    surface against an over-eager cleanup.
- **Negative control** (AC-10, run and recorded, not committed)
  - Remove the `auto-orient` prefix from `convert` → `convert_bakes_orientation_into_pixels`
    must go RED.

## Implementation Context

### Decisions that apply

- `DEC-003` — the dual-lane design and default-preserve policy. **This spec changes what
  that record claims about orientation**, so the amendment in AC-9 is part of the work, not
  a nicety.
- `DEC-017` — operations may READ the captured `MetadataBundle` to parameterize a pixel
  transform; `auto-orient` bakes then drops the bundle. This spec adds callers, not
  behaviour.
- **A new DEC is required** for "bake on every pixel-lane verb", carrying the measured
  table and both rejected alternatives.

### Constraints that apply

- `test-before-implementation` (**blocking**) — the Failing Tests go in first.
- `clippy-fmt-clean` (**blocking**) — on every leg of AC-11, including wasm.
- `one-spec-per-pr` (**blocking**) — SPEC-111 (`build` cannot run bundled recipes) is a
  separate PR, as is the `docs/data-model.md` chore.
- `every-public-fn-tested` — applies to the shared helper AC-7 introduces.

### Prior related work

- `SPEC-107` (shipped) — the immediately preceding spec; its lesson applies directly here.
  Its build reported a clean local matrix while Windows CI was red, and its follow-up verb
  list was wrong in **both** directions until verify drove 16 invocations. **Drive the verbs;
  do not reason about them from the call graph.**
- `SPEC-108` (shipped) — moved classification before the resize pipeline, and its verify
  found a real EXIF-driven classification flip on document-shaped content. Classification
  only runs on the `optimize`/`web` decide path, which this spec does not touch — but if you
  see a classification change, that is a finding worth reporting, not absorbing.

### Out of scope (for this spec specifically)

- **An opt-out flag** (`--no-auto-orient` or similar). Filed, not built — see the
  sub-decision above.
- **Removing `edit --auto-orient`.** The CLI surface is frozen.
- **Preserving any other metadata.** ICC, copyright/artist and the rest of DEC-003's
  preserve set are untouched here; AC-9 amends only the orientation claim. If the sweep
  suggests ICC is also being dropped against the record, **report it as a finding and file
  it** — do not fix it here.
- SPEC-111's `build` wiring, and the `docs/data-model.md` worked-example chore.

## Notes for the Implementer

- **Reuse, do not copy.** `optimize_pipeline()` already builds exactly the prefix you need.
  Factor it so there is one place that knows "pixel-lane pipelines start with auto-orient" —
  AC-7 is about there being a single site, because six copies is how the next verb gets
  added without it.
- **The obvious bug in this change is a double rotation.** `web`/`optimize` already push
  `orient`; if the shared helper is applied on top of the existing push, orientation gets
  baked twice and a 90° case comes back 180° off. AC-2 is the guard, and it only bites on a
  **non-square** fixture — hence AC-4.
- **`resize` is the worst case for users, not `convert`.** Today `resize --max 600` applies
  the bound to the wrong axis, so the output is the wrong *size*, not just mis-rotated. Lead
  the PR description with that.
- **Use the committed fixture builders.** `tests/common::jpeg_with_orientation(w, h, o)` and
  `wrap_with_orientation_app1` already exist and are already the house style. A hand-rolled
  TIFF in a new test is how SPEC-093's byte-order corruption got in.
- **Drive every verb; do not infer from the call graph.** SPEC-107's follow-up list was wrong
  in both directions because it was reasoned from `run_pixel_op` membership rather than
  driven. The design's table above was produced by running the binary — extend it the same way.
- **Check `responsive` carefully.** Its output is width-pinned, so a baked result is
  600×**900**, not a dimension swap. A test copied from the `resize` case will assert the
  wrong thing.
- **A square fixture makes this spec vacuous.** Stated twice on purpose.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-110-orientation`
- **PR (if applicable):** [#133](https://github.com/jysf/crustyimg/pull/133) — NOT merged, handed off to verify.
- **All acceptance criteria met?** yes — AC-1 through AC-11, including the AC-10 negative
  control (see below) and AC-11's full matrix (see below).
- **Punch-list pass (second build session, Sonnet, own worktree, 2026-08-05):** verify
  returned ⚠ PUNCH LIST on PR #133 — the code was "correct, safe, and better-tested than
  the spec asked for" on everything it measured, but it failed on the spec's own **Goal**
  because `watermark` shipped unbaked. This pass:
  - **Fixed the one blocking item.** `run_watermark` (`src/cli/ops.rs:1085`) built
    `Pipeline::new().push(...)` instead of `auto_orient_prefix()?.push(...)` — the identical
    shape as the already-fixed `resize`/`thumbnail`. One-line fix. Added a tenth test,
    `watermark_bakes_orientation` (`tests/orientation.rs`), and drove the negative control
    myself: reverted the fix, rebuilt (confirmed via a real recompile, not a stale binary),
    watched it fail at exactly the wrong dimensions the punch list described (1200×800, not
    800×1200); restored, confirmed green.
  - **Made the records true.** DEC-086's Decision/Context/Consequences now name `watermark`
    as a seventh baked call site (`auto_orient_prefix()` has 7 callers, not 6);
    `src/cli/optimize.rs:782`'s doc comment dropped its watermark exception;
    `docs/api-contract.md`'s `watermark` section now states the bake.
  - **Corrected a mischaracterization**, not just a wording nit: this build's own Follow-up
    work and DEC-086's Consequences called the `edit --save-recipe` recipe-replay
    divergence "pre-existing, unchanged by this decision." Verify drove it and measured the
    opposite — `main`: direct `edit --invert` (1200×800) and the same recipe replayed via
    `apply` (1200×800) agree; this branch: direct `edit --invert` (800×1200) and the replay
    (1200×800) diverge. **This decision introduces the divergence.** Corrected here, in
    DEC-086, and (same false claim, found additively while in the files)
    `docs/api-contract.md`'s `edit` section and `docs/moat.md`'s STAGE-005 summary; filed to
    land in SPEC-111 in all four places. Also dropped a self-contradicting parenthetical in
    `docs/cli-reference.md`'s `edit` section (claimed byte-pinned, then described the same
    divergence in the next sentence).
  - **Strengthened AC-5's test** (non-blocking, verify flagged it as worth closing cheaply).
    `all_eight_orientation_values_are_applied` asserted dimensions only, so orientations
    1–4 (axis-preserving) asserted the same (40,30) an UNBAKED build also produces — 4 of 8
    assertions could never fail. Added a quadrant-marker fixture (source top-left quarter
    black, rest white; local to this test, independent of `common::gradient_jpeg`, which
    varies only along X and so cannot distinguish orientation 4's vertical-only flip from a
    no-op) plus a corner-brightness check derived directly from the EXIF orientation spec's
    transform definitions, for a content-level assertion on all eight values. Drove a
    negative control myself: mutated `AutoOrient::apply` to always rotate 180° regardless of
    the actual tag, rebuilt, and watched orientation 2 fail with the WRONG corner dark while
    dimensions would have stayed correct (40×30, matching what o=2 expects) — proving the
    old dimension-only assertion would have stayed green on exactly this class of bug.
    Reverted the mutation, confirmed the diff against the committed file was empty, restored
    green.
  - **Re-ran the full matrix** clean from fresh per-leg `CARGO_TARGET_DIR`s, sequentially,
    every leg through `rtk proxy`, every log confirmed showing `Compiling crustyimg`:
    **lean 805 / default 824 / webp-lossy 831 passed, 0 failed** — reconciles exactly
    against the prior reference (804/823/830) plus the one new test (the AC-5 strengthening
    added assertions to an existing test, not a new `#[test]` fn, so it contributes 0 to the
    count). `clippy --all-targets -D warnings` clean on all three legs. `cargo fmt --check`
    clean. `just wasm-test` **30/30**, unchanged (no wasm-side change this pass).
- **New decisions emitted:**
  - `DEC-086` — bake EXIF orientation on every pixel-lane verb (measured table + both
    rejected alternatives; `decisions/DEC-086-bake-orientation-on-every-pixel-lane-verb.md`).
    `DEC-003` is amended in place (dated 2026-08-04 section), not superseded — only its
    orientation claim changes; ICC/copyright/GPS are untouched.
    **Amended on the punch-list pass** — see above: `watermark` added as a seventh baked
    call site, and the `edit --save-recipe` Consequences bullet corrected to say this
    decision introduces the recipe-replay divergence rather than inheriting it.
- **Deviations from spec:**
  - Went slightly beyond the `Outputs` list's named `docs/cli-reference.md` scope (which
    named only `edit --auto-orient`'s description): also corrected the `resize`/
    `thumbnail`/`convert`/`responsive` lines there, since they made the same now-false
    "no pixel changes" / "preserving aspect" claims AC-8 required fixing in
    `docs/api-contract.md`. Leaving one doc file internally inconsistent with the other
    seemed worse than the small scope add.
  - `edit`'s `build_edit_ops` was left unchanged (still adds an explicit `auto-orient` op
    to its ops list when `--auto-orient` is passed) rather than making the flag contribute
    zero ops. This means `edit --auto-orient` runs `AutoOrient::apply` twice in a row when
    combined with the new unconditional prefix — confirmed safe/idempotent (the first bake
    drops the metadata bundle the op reads, so the second call is a true no-op on every
    input, not a double rotation) and it keeps `--save-recipe`'s captured recipe accurate
    for that one flag combination. The alternative (stripping auto-orient from
    `build_edit_ops`) would have made `edit --auto-orient` alone produce an empty `ops`
    list, requiring a second special case in the "at least one op flag" check to avoid a
    false usage-error — more moving parts for the same observable behavior.
  - The build prompt's reference test totals (lean 797 / default 816 / webp-lossy 823)
    were stale by exactly 2 in every leg: measuring a clean `origin/main` worktree directly
    gives lean 795 (confirmed zero failures). My branch's lean total is 804 = 795 + the 9
    tests added — reconciles exactly. Default (823) and webp-lossy (830) show the same
    +9 delta, consistent with the same stale-by-2 reference. Not investigated further
    (not a SPEC-110 regression — the discrepancy exists identically on `origin/main`
    before this branch's changes).
- **Follow-up work identified (the `edit --save-recipe` bullet corrected on the punch-list
  pass; the watermark bullet closed out on the punch-list pass):**
  - `edit --save-recipe`'s captured recipe does not record the CLI-level auto-orient
    prefix as a step. A recipe saved from an `edit` invocation now diverges when replayed
    via `apply --recipe FILE`: verify drove it and measured direct `edit --invert` on the
    design's fixture at 800×1200 (baked) against the same recipe replayed via `apply` at
    1200×800 (unbaked). **This PR introduces the divergence, not inherits it** — before
    this decision `edit` never baked either, so direct invocation and a recipe replay
    agreed (both unbaked). Flagged, not fixed (`one-spec-per-pr`; `apply`/recipe pixel-lane
    wiring is SPEC-111's territory — filed there). Documented in `docs/api-contract.md`'s
    `edit` section and `src/cli/ops.rs`.
  - `watermark` (`src/cli/ops.rs::run_watermark`) was missed by the spec's measured table,
    `Outputs`, and acceptance criteria, and shipped in PR #133 without the
    `auto_orient_prefix()` every other pixel-lane verb got — the identical
    `Pipeline::new().push(...)` shape as the fixed `resize`/`thumbnail`. Verify caught it
    (17-subcommand classification) and it is fixed in this punch-list pass: `watermark`
    now bakes via the shared prefix, with a tenth test (`watermark_bakes_orientation`,
    `tests/orientation.rs`) alongside the original nine.
  - DEC-003's ICC/copyright preserve claims were NOT re-investigated (out of scope,
    explicitly declined per the build prompt's "report it, don't fix it" instruction).
    `AGENTS.md`/every affected verb's own doc comments already state the pixel lane drops
    ALL metadata on re-encode, which is in tension with DEC-003's ICC preserve claim
    predating this spec — not a new finding from this sweep, just noted for whichever
    future spec finally reconciles it.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing structural was unclear — the design table, traps, and file/line pointers were
   precise enough to implement directly. The one judgment call the spec left open was
   *how* `edit --auto-orient` should become a no-op: whether the flag should still
   contribute an explicit op to `build_edit_ops`'s list (redundant-but-safe alongside the
   new prefix) or contribute nothing (requiring a second special case in the "at least one
   flag" check). The spec's wording ("accepted, documented no-op") is consistent with
   either reading; I picked the smaller, lower-risk diff (see Deviations) but a sentence
   pinning this in the spec would have saved the analysis.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `edit --save-recipe` recipe-capture gap (see Follow-up work) isn't a constraint
   that was missing so much as a consequence the spec's Outputs/AC list didn't anticipate,
   because `edit`'s pre-change behavior never baked, so the gap didn't exist yet. Not a
   spec defect — a genuinely new interaction surfaced by the fix itself.

3. **If you did this task again, what would you do differently?**
   — Verify the "reference test totals" claim against a fresh `origin/main` build BEFORE
   starting implementation, not after finishing the matrix. It cost one extra worktree +
   test run to discover the stale-by-2 baseline late; doing it first would have let me
   state "expect lean 795+9=804" up front instead of investigating a surprise mismatch at
   the end.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
