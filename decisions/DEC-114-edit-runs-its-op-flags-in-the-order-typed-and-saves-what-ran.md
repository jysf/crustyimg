---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-114
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-017
repo:
  id: crustyimg

created_at: 2026-11-10
# Stands beside DEC-005, which never stated an order. Supersedes SPEC-032's canonical-order
# statement (a spec, not a DEC). Reads DEC-112 and DEC-113; amends neither.
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/mod.rs
  - src/cli/ops.rs
  - tests/edit.rs
  - tests/edit_geometry.rs
  - docs/cli-reference.md
  - docs/api-contract.md
  - docs/recipes.md
  - recipes/README.md

tags:
  - cli
  - edit
  - recipe
  - geometry
  - ordering
---

# DEC-114: `edit` runs its op flags in the order typed, and `--save-recipe` writes what ran

## Decision

`crustyimg edit` runs its op flags in the order the user types them, the way a recipe runs its
steps, after an `auto-orient` that always comes first; `--save-recipe` writes the steps that ran,
in that order, with every pixel length resolved to a fraction of the image its step received.

The rule has seven parts:

1. **Op flags run in typed order.** An op flag is one that adds a step: `--resize-max`, `--invert`,
   `--crop`, `--rotate`, `--flip`, and any op flag added later (`--pad`, `--trim`). The position
   is where the flag appears on the command line, whatever its spelling (`--crop X` or
   `--crop=X`) and whether it comes before or after the input path. A flag that clap fills from
   a default, such as a switch left off, has no position and adds no step.
2. **`auto-orient` is not ordered: it always runs first.** `edit` bakes EXIF orientation before
   any op flag's step, and the saved recipe always starts with `auto-orient`. `--auto-orient` is
   accepted anywhere and changes nothing, and it alone still satisfies "at least one op flag".
   Placing it later would make a saved `[rotate, auto-orient]` replay a mirrored tag (2, 4, 5, 7)
   differently from what `edit` ran (DEC-113 rule 3; driven on Orientation 5 and 7, 62 of 72
   samples differ).
3. **Each op flag appears at most once.** A repeat is refused (exit 2) before any file is read.
   A step that repeats belongs in a recipe: save one and edit the TOML.
4. **Modifier flags have no position.** A flag that qualifies an op (`--crop-gravity` now;
   `--pad-color`, `--pad-gravity`, `--trim-color`, `--trim-tolerance` later) attaches to its op
   wherever it is typed, requires that op (exit 2 without it), and is written into that op's step
   only when given. Part 3 makes the attachment unambiguous.
5. **A pixel length in a flag is in pixels of the image as that step receives it,** after every
   step typed before it. It becomes a fraction (DEC-112) when the step runs, by `f64` division of
   the pixel count by that image's side, and the recipe writes it at full `f64` precision (the
   shortest text that reads back to the same `f64`, never a fixed number of decimals). A pixel
   length past the edge of the image the step receives is an apply failure (exit 1, or that
   input fails in a batch) whose message names the flag and that image's size.
6. **`--save-recipe` writes what ran, after it ran.** The recipe is built once the run succeeds:
   `auto-orient`, then one step per op flag in run order, with part 5's fractions. A failed run
   writes no recipe. A pixel length is resolved against one image, so `--save-recipe` with a
   flag that takes pixels needs a single input image (exit 2 otherwise, before any image is read).
7. **No pixel unit enters a recipe.** DEC-112 is unchanged: a recipe carries fractions, and the
   flag is the only place pixels appear.

## Context

STAGE-058 put five geometry ops (`crop`, `rotate`, `flip`, `pad`, `trim`) into recipes and owes
"`edit` flags for all five, with a written ordering rule". Geometry ops don't commute: on a
60×40 image, crop-the-left-half then rotate 90 gives 40×30, and rotate 90 then the same crop
gives 20×60. Even today's flags don't: on a 61×41 noise image, `[invert, resize max 30]` and
`[resize max 30, invert]` differ in 1789 of 1800 samples.

Until now `edit` ran `auto-orient → resize → invert` whatever the typed order (SPEC-032). That
order lived in code comments and `docs/cli-reference.md`, which cited DEC-005, but DEC-005 only
says a recipe is "an ordered list" built through one registry; it never stated an order. This
DEC stands beside DEC-005 and replaces SPEC-032's canonical-order statement.

## Alternatives Considered

- **Option A: canonical positions** (`auto-orient → trim → rotate → flip → crop → resize-max →
  invert → pad`)
  - What it is: a fixed order, whatever is typed. Today's bytes never change.
  - Why rejected: a command typed in a different order silently gives a different result, with
    exit 0 (`--rotate 90 --crop …` would crop before turning). Users must learn a seven-step
    order. Weighed by the maintainer on 2026-11-10 and declined in favour of typed order.
- **Option B: typed order for geometry flags only, `invert` last**
  - What it is: keep `invert` at the end.
  - Why rejected: `--resize-max` is geometry, so `--invert --resize-max` still changes unless
    resize is exempt too, and the rule becomes hard to state.
- **Option C: canonical positions, refusing out-of-order typing**
  - What it is: exit 2 when the typed order differs from the canonical one.
  - Why rejected: it refuses commuting pairs (`--invert --crop`) or needs a commutation table.
- **Option D (chosen): typed order**
  - What it is: parts 1–7 above.
  - Why selected: it's the order a user reads off their own command, it's what ImageMagick's
    operators do, the saved recipe reads like the command (the brief's "the recipe step is the
    deliverable, the flag is the convenience"), and a pixel length can be defined in one sentence.

## Consequences

- **Positive:** `--rotate 90 --crop …` crops the turned image. The saved recipe lists the steps in
  the order typed and replays to the same pixels. New op flags (`--pad`, `--trim`) join without
  changing this DEC.
- **Negative:** `edit --invert --resize-max N` now inverts before resizing, so its output changes
  (a CHANGELOG "Changed" line). `--save-recipe` with a pixel `--crop` refuses a batch input.
- **Neutral:** `--resize-max N --invert`, either flag alone, and any `--auto-orient` placement keep
  their bytes and their saved recipe text. Repeated op flags were already refused by clap.

## Validation

Right if `tests/edit_geometry.rs`' replay matrix stays green (every typed order replays through
`apply` to the same decoded pixels, and the saved steps match the typed order) and the byte pin
in `tests/edit.rs` holds for the unchanged invocations. Revisit if users ask for a repeated op
flag often enough that a recipe is the wrong escape hatch, or if `auto-orient` gains a reason to
run anywhere but first.

## References

- Related specs: SPEC-153 (this rule's first flags: `--crop`, `--rotate`, `--flip`), the
  `--pad`/`--trim` entry after it in STAGE-058, SPEC-032 (the canonical order this replaces),
  SPEC-110 and SPEC-111 (`auto-orient` first, and recorded)
- Related decisions: DEC-005 (recipes are ordered lists; states no order), DEC-106 (flag values
  checked before any file is read, messages name the flag), DEC-112 (fractions, rounded edges),
  DEC-113 (geometry acts on the buffer it receives; mirrored tags and `auto-orient`)
