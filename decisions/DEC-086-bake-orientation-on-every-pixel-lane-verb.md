---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-086
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-04
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/optimize.rs"
  - "src/cli/ops.rs"
  - "docs/api-contract.md"
  - "docs/cli-reference.md"
  - "decisions/DEC-003-metadata-dual-lane.md"

tags:
  - orientation
  - exif
  - metadata
  - pixel-lane
  - correctness
---

# DEC-086: bake EXIF orientation on every pixel-lane verb (not just `web`/`optimize`/`auto-orient`)

## Decision

Every verb that re-encodes pixels — `convert`, `resize`, `thumbnail`, `edit`, `responsive`,
in addition to the already-correct `web`/`optimize`/`auto-orient` — pins the existing
`auto-orient` operation first, via one shared prefix (`auto_orient_prefix()`,
`src/cli/optimize.rs`). **Bake, do not preserve.** `edit --auto-orient` becomes an accepted,
documented no-op (the flag cannot be removed — STAGE-030 froze the CLI surface); **no
opt-out flag is added.**

## Context

STAGE-039 framed this as *"`convert` orientation: decide, fix, sweep"*. Driving a
purpose-built JPEG — stored **1200×800** with a real one-entry `Orientation=6` IFD (correct
display size **800×1200**) — through every pixel-lane verb on the `d854038` release build
measured:

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

Seven invocations returned a sideways image, and **every one of them also dropped the
EXIF** — the tag needed to correct the output by hand was destroyed by the same operation
that made the output wrong. Nothing distinguished the two groups except which pipeline
builder each verb happened to call: `web`/`optimize` route through `optimize_pipeline()`
(`Pipeline::new().push(orient)`), every other verb built its own pipeline without it. No
rule explained the split — a rule nobody can state is a rule nobody maintains, which is
how seven invocations drifted wrong without anyone noticing.

`resize` is the worst case for users, not `convert`: the `--max` bound was applied to the
**wrong axis**, so the output was the wrong *size*, not merely mis-rotated.

This also left `DEC-003`'s own falsifiability condition false — its Validation section
claims *"a resize preserves orientation … Orientation/ICC survive transforms,"* which was
no longer true of the code (nor, per the measured table, ever true of `convert`,
`thumbnail`, `edit`, or `responsive`). `DEC-003` is amended separately (dated section, this
spec) rather than superseded — its ICC/copyright/GPS claims are untouched; only the
orientation claim changes.

## Alternatives Considered

- **Preserve the tag instead (what DEC-003 literally says).**
  - What it is: write the EXIF Orientation tag back into the container on every affected
    verb's output, rather than rotating pixels.
  - Why rejected: more faithful on paper, and it would keep `convert` byte-faithful — but
    it requires a container-lane write on every pixel encode, per output format (a new
    write path for PNG/WebP/AVIF/etc., not just JPEG), and the output still renders
    sideways in any viewer that ignores EXIF (a large, non-zero population — most web
    `<img>` rendering, many image-processing pipelines). Baking delivers the outcome the
    user actually wants: the picture looks right, everywhere, unconditionally.

- **Split by verb intent — bake for `thumbnail`/`responsive`/`resize`, preserve for
  `convert`.**
  - What it is: treat `convert` as archival (byte-faithful re-encode) and the
    presentation-oriented verbs as display-faithful (bake).
  - Why rejected: defensible per-verb, but it is two rules to document and remember, and
    the seam is exactly where the next bug hides (which verb is "archival" is a judgment
    call that will not stay obvious as verbs are added). It also does not actually buy
    `convert` byte-faithfulness: `convert` **already discards all metadata** on its
    re-encode today, so it is not archivally faithful in any sense already — the
    strongest argument for this split is weaker than it looks.

- **Bake everywhere (chosen).**
  - What it is: one rule — every pixel-lane verb bakes orientation first, via a single
    shared prefix.
  - Why selected: one rule to state and maintain; matches what `web`/`optimize` already
    did; the pixel lane already discards all other metadata on re-encode (DEC-003), so
    baking orientation into pixels arguably *improves* the fidelity of what survives
    compared to the status quo, where the rotation information was neither applied nor
    kept — it was destroyed.

**Sub-decision: `edit --auto-orient` becomes an accepted no-op.** The CLI surface was
frozen in STAGE-030, so the flag cannot be removed; it stays, documented as "now the
default," and must keep exiting 0. **No opt-out flag** (e.g. `--no-auto-orient`) is added:
there is no evidence of demand for "give me the stored, un-rotated pixels," and an escape
hatch deserves its own spec with a real user behind it — the same reasoning DEC-063 used to
file `--max-pixels` rather than build it.

## Consequences

- **Positive:** one orientation rule is true across the entire pixel lane; no shipped verb
  can hand back a sideways image. `resize`'s worst-case bug (wrong-axis bound) is fixed as
  a side effect of the same change. The prefix is factored once
  (`auto_orient_prefix()`) rather than copied into six call sites, so the next pixel-lane
  verb inherits correct behavior by construction rather than by remembering to add it.
- **Negative:** `convert` is no longer a byte-faithful re-encode for the (small) minority
  of inputs carrying a non-1 orientation tag — its output pixels now differ from the input
  pixels for those cases. `edit --auto-orient` is a vestigial no-op flag on the frozen CLI
  surface. The `edit --save-recipe` recipe capture does not record the CLI-level bake as a
  step, so a recipe saved from an `edit` invocation that omitted `--auto-orient` will not
  reproduce baking when replayed via `apply --recipe` (pre-existing gap, unchanged by this
  decision — flagged, not fixed, `one-spec-per-pr`).
- **Neutral:** Orientation 1 and no-EXIF inputs (the overwhelming majority of real-world
  images) are unaffected — `AutoOrient::apply` no-ops on both, returning the input `Image`
  completely unchanged, so output bytes are identical to before this decision for those
  inputs.

## Validation

Right if: driving the same measured fixture (1200×800, `Orientation=6`) through
`convert`/`resize`/`thumbnail`/`edit`/`responsive` now produces the display-correct
dimensions (800×1200, or 600×900 for `responsive --widths 600`, width-pinned); the four
already-correct verbs (`web`/`optimize`/`auto-orient`/`edit --auto-orient`) are unchanged
at 800×1200 (not double-rotated); orientation 1 / no-EXIF inputs are byte-identical to
before on every affected verb; and reverting the shared prefix on any one verb turns at
least one test RED (SPEC-110's AC-10 negative control). Revisit if: a real user requests
the un-rotated stored pixels (then file the opt-out flag spec DEC-086 explicitly declined
to build), or `edit --save-recipe`'s recipe-capture gap causes an actual reported
surprise (then give the CLI-level bake its own recorded recipe step).

## References

- Related specs: SPEC-110 (this decision's origin), SPEC-107 (prior related work — its
  build reported a clean local matrix while Windows CI was red; its follow-up verb list
  was wrong in both directions from reasoning about `run_pixel_op` call-graph membership
  instead of driving the binary — the same discipline this spec's design table applied),
  SPEC-108 (moved classification before the resize pipeline; classification is untouched
  by this spec).
- Related decisions: DEC-003 (metadata dual-lane + default-preserve policy — amended by
  this decision's orientation claim, dated section in the DEC-003 file itself), DEC-017
  (operations may read the captured `MetadataBundle`; `auto-orient` is the op this
  decision adds callers of, not new behavior for), DEC-063 (the `--max-pixels` filed-not-
  built precedent this decision's "no opt-out flag" reasoning follows).
- External docs: none.
