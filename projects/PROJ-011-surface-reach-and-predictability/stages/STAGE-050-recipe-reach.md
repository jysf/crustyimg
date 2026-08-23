---
stage:
  id: STAGE-050
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value_contribution:
  advances: >
    The outcome itself: one `crustyimg build` watermarks and optimizes a whole
    photo site from a manifest. Today the registry holds four ops and `Recipe`
    has no format or quality field, so the declarative path cannot express the
    job at all.
  delivers:
    - "A `watermark` operation expressible in a recipe, and therefore reachable from `build`"
    - "`format` and `quality` as `Recipe` fields, so a recipe can pin its output"
    - "Recipe schema errors surfacing at parse time instead of partway through a batch"
    - "A watermark that refuses to wreck an image, and skips one it cannot sensibly mark"
  explicitly_does_not:
    - "Add an EXIF or C2PA write to `watermark` — the lanes stay separate (DEC-003); compose them in a recipe instead"
    - "Add `--save-recipe` to more verbs — a consequence worth revisiting after, not scope here"
    - "Change which candidate `optimize` picks"
---

# STAGE-050: Recipe Reach

## What This Stage Is

The registry holds **four** operations — `identity`, `invert`, `resize`, `auto-orient` — plus a
terminal `optimize` marker that is not an operation at all. So a recipe cannot express a
**watermark**, and `Recipe` has no field for an output **format** or **quality**.

`build` binds sources to a recipe, so it inherits every one of those limits. When this stage ships,
**one `build` watermarks and optimizes a whole photo directory from a manifest** — the theF11.com
case that motivated PROJ-011.

## Why Now

The maintainer's live need: every image on theF11.com should carry a watermark, and there is no
way to say that in a manifest. The working alternative is a hand-driven three-pass pipeline
(`resize --format png` → `watermark --format png` → `optimize --verify`), which costs +0.5% bytes
and cannot be declared or locked.

📌 **Why the gap stayed invisible:** only `edit` has `--save-recipe`, so the one path that *emits*
a recipe covers only the ops a recipe can already express. Nothing that can save a recipe can do
anything a recipe cannot say.

## Success Criteria

- **One `crustyimg build` watermarks and optimizes a photo directory from a manifest**, driven end
  to end.
- A recipe can **pin an output format and quality**, so `apply --recipe` reaches what `convert`
  reaches.
- A **malformed recipe fails at parse time**, naming the bad key — not partway through image 37.
- A watermark that **cannot sensibly be applied is refused or skipped, never silently dropped.**

## Scope

### In scope
- A `watermark` operation in the registry, with its parameters and the size rules below.
- `format` and `quality` as `Recipe` fields.
- Typed per-operation parameter structs, so validation happens at parse rather than per-op.

### Explicitly out of scope
- **Metadata in a recipe.** ⚠ Worth an explicit note because it was asked and declined: an EXIF
  "watermark string" is **not** a watermark — `meta strip` removes it in one command, so it is a
  claim rather than a mark — and `meta set` already writes `--artist`/`--copyright`/`--description`
  through the container lane, which DEC-003 keeps separate from the pixel lane. **If a recipe
  should be able to express a metadata step, that is its own DEC and its own scope**, not a flag
  on `watermark`. C2PA is the repo's existing provenance answer.
- Adding `--save-recipe` to more verbs.
- Any change to `optimize`'s candidate selection.

## Spec Backlog

- [ ] (not yet written) — [M] **`watermark` becomes a registry operation.** Its ten parameters
  (image/text, font, size, colour, gravity, opacity, scale, margin, tile) are richer than any op
  the registry has taken, which is the real work — the seam itself is documented as "the single
  seam new operations register at", but no parameter-rich op has ever used it.

- [ ] (not yet written) — [S] ⚡ **`watermark` stops silently doing nothing.** A **measured
  defect**, driven on `main` (PNG filters undone before comparing):

  | input | `--size` | pixels changed |
  |---|---|---:|
  | 800×600 | default | 0.32 % — a sane watermark |
  | 64×64 | default | **16 %** |
  | 24×24 | default | **47 %** |
  | 24×24 | **200** | **0.00 % — exit 0, file written, no warning** |

  **Two rules, and they are not the same rule** (maintainer ruling, 2026-08-23):
  - **Damage rule — default on, the tool enforces it** because it is pure geometry. Single explicit
    input → **error, exit 2**, naming the numbers. Batch or recipe → **skip, report, continue**;
    ⚠ **never abort a run** — a mixed directory of large originals and generated thumbnails is the
    normal case, and aborting there makes one `build` over a site impossible.
  - **Value rule — opt-in `--min-size`, settable in a recipe**, because only the user knows what is
    worth protecting. Hard-coding a pixel floor is the tool making a business judgement.
  ⚠ Unsettled, deliberately: the coverage threshold is a number nobody has measured (~25 % separates
  the rows above — **validate, do not adopt from this table**); whether it is a knob; and that
  **`--tile` covers the image by design**, so the coverage test cannot apply to it unmodified.

- [ ] (not yet written) — [M] **`format` and `quality` on `Recipe`**, plus typed per-operation
  parameter structs. ⛔ **Depends on STAGE-049** — the two paths must agree on what a format means
  before the schema names one.

**Count:** 0 shipped / 0 active / **3 pending** — re-derive with a grep you just ran.

## Design Notes

- ⚠ **`watermark --size` is absolute pixels.** A recipe-level watermark is only consistent across a
  batch if the recipe normalises dimensions first, or the size becomes relative. This is a
  constraint on the op's design, not a detail to discover during build.
- A relative size answers batch consistency but **not** the silent-no-op half — a 5 % watermark on
  a 24 px thumbnail is still unreadable. Both halves need answering.
- **Typed params are folded in here rather than filed separately** (external review batch 3, one of
  two items that survived checking): `OperationParams` exposes hand-rolled `get_str`/`get_u32`, so
  every op re-implements validation. The payoff is not stylistic — schema errors surface at parse
  time, which is exactly the failure a batch recipe user hits.

## Dependencies

### Depends on
- ⛔ **STAGE-049** — `Recipe` cannot gain a `format` field until `apply` and `build` agree on what
  one means.

### Enables
- A `build` manifest worth wiring into CI, because it can express the whole job.
- **PROJ-012** (animated AVIF) inherits a registry that has absorbed one parameter-rich op.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
