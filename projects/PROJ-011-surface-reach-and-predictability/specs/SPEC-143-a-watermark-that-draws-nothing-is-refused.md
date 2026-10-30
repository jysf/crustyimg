---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-143
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-054
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null
  created_at: 2026-10-30

references:
  decisions:
    - DEC-104   # watermark refuses by geometry; decided from the mark, not the image; `visible` is its count
    - DEC-106   # every CLI watermark flag check precedes every file read; opacity 0 / blank text / alpha-00 colour are flag checks
    - DEC-102   # recipe step params validated at load; the `recipe step N ('watermark'):` prefix and exit 1
    - DEC-031   # the CLI builds the op directly, so the check needs a call site there as well as in from_params
  constraints:
    - test-before-implementation
    - untrusted-input-hardening
  related_specs:
    - SPEC-135  # the non-finite size/scale half of STAGE-054's first criterion; same exit codes (shipped)
    - SPEC-133  # DEC-104's fit and coverage refusals (shipped)
    - SPEC-131  # recipe watermark schema (shipped)

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-054's <capability>". Optional; null is acceptable.
value_link: >
  Closes STAGE-054's first success criterion, "no CLI or recipe path draws an invisible mark at
  exit 0": a mark with no visible pixel after opacity is refused on the CLI, in a recipe and in
  wasm, instead of writing an unmarked image at exit 0.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 7159888
      tokens_breakdown:
        input: 112
        output: 46630
        cache_creation: 143214
        cache_read: 6969932
      estimated_usd: 5.55
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive frame session in its own worktree (docs/spec-143-frame). Measured from the
        session's transcript (3f2e7963-….jsonl, identified by content), deduped by `.message.id`
        (56 unique ids, all claude-opus-5-5): input/cache_creation/cache_read taken once per id,
        MAX output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation
        x1.25 ($6.25), cache_read x0.10 ($0.50): $0.0006 + $0.90 + $3.48 + $1.17. Measured
        before the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-143: a watermark that draws nothing is refused

## Context

STAGE-054's first success criterion promises that "no CLI or recipe path draws an invisible mark
at exit 0". SPEC-135 closed the non-finite `size`/`scale` half of it. This is the other half,
filed by SPEC-135's frame (2026-10-27) as "A watermark with no visible pixels is drawn as nothing
at exit 0". The maintainer ruled on 2026-10-30 that `opacity = 0` is refused like the other
invisible marks, with the same exit codes: to turn a watermark off, remove the step.

What a user sees today (release build of `main` @ `9174087`, 0.8.1; changed pixels counted with
ImageMagick, never crustyimg):

```text
$ crustyimg watermark photo.png --text " " -o out.png                  # exit 0, 0 px changed
$ crustyimg watermark photo.png --text "© crustyimg" --size 0.01 -o …   # exit 0, 0 px changed
$ crustyimg watermark photo.png --text "© crustyimg" --opacity 0 -o …   # exit 0, 0 px changed
$ crustyimg watermark photo.png --text "© crustyimg" --color ffffff00 … # exit 0, 0 px changed
$ crustyimg watermark photo.png --image clear.png -o out.png            # exit 0, 0 px changed
```

The same five marks in a recipe (`apply --recipe`, `build`) also exit 0 and write unmarked
images. The user asked for a mark and got none, with no message.

## Goal

A watermark whose mark has no visible pixel, once its opacity is applied, is refused before any
input image is read: exit 2 on the CLI, exit 1 for a recipe (`apply`, `build`), and a thrown error
from wasm `transform`. The one-line message names the cause (opacity 0, blank text, a size too
small to draw, a fully transparent colour or image) and says what to do instead. Every mark that
draws today still gives byte-identical output.

## Frame

Frame session 2026-10-30 (Opus 5.5), own worktree `docs/spec-143-frame`, on `main` @ `9174087`.
Every row was driven on a release build in a fresh `CARGO_TARGET_DIR` (deleted afterwards).
Fixtures were made with ImageMagick: 400×300 8-bit bases `dark` (#202020), `light` (#c8c8c8) and
`white`; a 16-bit `dark16` (#202020); an opaque white 60×30 `logo.png`; and a fully transparent
60×30 `clear.png` (`xc:none PNG32:`, max alpha 0). Changed pixels were counted **exactly** with
ImageMagick: difference, then the max over channels, then threshold 0, then count. The AE
metric gave fractional counts on grayscale bases, so it was not used. Negative control: a base
against itself counts 0.

### Verdict: **GO**, [S]

The premise holds on every row and on every path. One check on the constructed mark covers every
path, because every path builds the mark before it reads an input. One finding changes the
frame's rule: whether a mark changes a pixel depends on the image (its colour and bit depth),
not just the mark. So the rule must be decided from the mark alone, as DEC-104's coverage
count already is: **refuse a mark that has no pixel with alpha > 0 after opacity is applied.** It
cannot be "refuse a mark that would change no pixel of this image". Maintainer question 1 asks
to confirm this.

### 1. Premise: confirmed on both paths (and on `build`)

| Row (dark base) | CLI `watermark` | `apply --recipe` | `build` |
|---|---|---|---|
| control: `--text "© crustyimg"` / `text = "© crustyimg"` | exit 0, **1647 px** | exit 0, 1647 px | exit 0, 1647 px (dark) / 1624 (light) |
| control: `--image logo.png` / `image = "logo.png"` | exit 0, **1800 px** | exit 0, 1800 px | — |
| blank text `" "` | exit 0, **0 px** | exit 0, **0 px** | exit 0, **0 px** on both inputs |
| `size 0.01` | exit 0, **0 px** | exit 0, **0 px** | — |
| `opacity 0` (text) | exit 0, **0 px** | exit 0, **0 px** | exit 0, **0 px** on both inputs |
| `opacity 0` (image) | exit 0, **0 px** | exit 0, **0 px** | — |
| `color ffffff00` (alpha 00) | exit 0, **0 px** | exit 0, **0 px** | — |
| transparent image `clear.png` | exit 0, **0 px** | exit 0, **0 px** | — |

No row printed anything to stderr. `--color RRGGBBAA` with alpha `00` is a fifth cause that the
backlog item did not list. wasm `transform` was not driven. By reading the code it takes the same
`Watermark::from_params` as `apply` (see 4), so it is affected wherever text mode is (image mode
cannot reach wasm, which has no assets: SPEC-128).

**Precedent:** `--text ""` is already refused on both paths. CLI: exit 2, `error: text to render
must not be empty`. Recipe: exit 1, `error: recipe step 0 ('watermark'): could not render 'text':
text to render must not be empty`. Blank text should read like this.

### 2. Does `visible_pixels()` see opacity? **No, and it must not start to.**

`visible_pixels` (`src/operation/mod.rs:1398`) counts the overlay's pixels with alpha > 0 on
its 16-bit view. It is computed at construction (`:1451` `new_image`, `:1482` `new_text`, `:1634`
`from_params`), before opacity. Opacity is applied only inside `apply`, after any `scale` resize
(`:1777-1780` 16-bit branch, `:1833-1836` 8-bit branch:
`px.0[3] = (px.0[3] as f32 * self.opacity).round() as u16|u8`). So a check on `visible == 0`
catches blank text, a tiny size, colour alpha 00 and a transparent image, but **not `opacity = 0`**.

Do not move the count after opacity. `visible` is DEC-104's coverage figure, documented as
"before `--opacity`" (`docs/api-contract.md:420-421`, DEC-104 line 55). It is pinned by
`opacity_does_not_change_coverage` (`src/operation/mod.rs:2938`). Changing it would change which
marks the 5 % cap refuses. The design needs a **second, separate predicate**: whether any pixel's
alpha is still > 0 after opacity. It should use the exact expression from the 16-bit branch, so
the check and the composite cannot disagree (as `mark_size` did for SPEC-135). It reduces to
`round(max_alpha16 × opacity) == 0`, and `opacity == 0` is the case where that is always true.

### 3. Where "almost zero" lands, and why the rule is decided from the mark

Opacity sweep, opaque white `logo.png`, CLI, exact changed-pixel counts:

| base | last opacity with 0 px | first opacity with 1800 px | arithmetic |
|---|---|---|---|
| dark #202020, 8-bit | 0.00588 | **0.00589** | effective alpha must reach 2: `op ≥ 1.5/255` |
| light #c8c8c8, 8-bit | 0.0176 | **0.0177** | alpha 5: `op ≥ 4.5/255` |
| white, 8-bit | 1.0 | **never** | white on white changes nothing at any opacity |
| dark #202020, **16-bit** | 0.0000228 | **0.0000230** | alpha16 2: `op ≥ 1.5/65535` |

At `opacity 0.002–0.0058` the mark changes 0 px of **every** 8-bit image tried, but all 1800 px of
the 16-bit one. Even with effective alpha 1, the 8-bit composite rounds down to no change.

Size sweep, `--text "© crustyimg"` at opacity 1: the first changed pixel on 8-bit dark is at
**size 0.25** (0.2 changes 0 px), and on light at **0.4** (0.3 changes 0). The 16-bit dark base
works as a detector for "the mark has some alpha > 0". There, size 0.15 changes 0 px and **0.2
changes 1 px**. So at size 0.2 the mark has visible pixels, yet it changes nothing in any 8-bit
output.

**Conclusion.** "Changes no pixel" can only be judged after compositing onto a particular image,
and that answer is image-dependent. DEC-104 already rejected that kind of measure for the
coverage cap ("Count changed pixels. Rejected: … would refuse a mark on a dark image and pass it
on a light one"). The rule this spec can keep exactly, from the mark alone, before any input is
read, is:

> **Refuse a mark with no pixel whose alpha is > 0 after opacity (counted at 16-bit
> precision).**

- It **never refuses a mark that changes a pixel**, because an alpha-0 pixel composites to the
  base unchanged (every alpha-0 row above: 0 px).
- It **lets through faint marks that may change nothing on a given image**: white on white, 8-bit
  output at opacity < 1.5/255, size 0.2. This is the same conservative shape as DEC-104 (decided
  from the mark, not the image). It needs one sentence in the docs. Maintainer question 1.
- 16-bit is the right precision. An 8-bit-precision rule would refuse `--opacity 0.001`, which
  does change a 16-bit image (1800 px).
- **Boundary rows for design (traps 9 and 10):** a positive but tiny opacity is refused when
  `round(max_alpha16 × op) == 0`. For an opaque mark that means op < 0.5/65535 ≈ 7.63e-6:
  `--opacity 0.000001` would otherwise be the new way to get an invisible mark at exit 0. The
  threshold depends on the mark's **maximum** alpha, not on 65535. Design needs a row where these
  differ, for example `--color ffffff01` (max alpha8 1, so alpha16 257) at `--opacity 0.001`:
  `round(0.257) = 0`, refused, where an "op × 65535" shortcut would pass it. That row also needs a
  mutation that swaps in the shortcut.

### 4. Every construction site: one predicate, two call sites

| Path | Builds the mark at | Then reads inputs at | Error type → exit |
|---|---|---|---|
| CLI `watermark` (image) | `src/cli/ops.rs:1466` `Watermark::new_image` | `run_pixel_op`, `:1480` | `CliError::Usage` → **2** |
| CLI `watermark` (text) | `src/cli/ops.rs:1474` `Watermark::new_text` | `:1480` | `CliError::Usage` → **2** |
| recipe step (all recipe callers) | `src/operation/mod.rs:1518` `Watermark::from_params`, via the registry (`src/operation/registry.rs:306`) | after `build_pipeline` | `RegistryError` |
| ↳ `apply --recipe` | `build_pipeline` → `from_params` | after it (driven: a `from_params` error exits 1 even when the input is missing) | → **1** |
| ↳ `build` (and `--watch`) | target validation probes the pipeline up front | after every target is validated | → **1** (driven: `error: target #0 (recipe …) error: recipe step 0 ('watermark'): …`) |
| ↳ wasm `transform` | `src/wasm.rs:243` `pixel_recipe.build_pipeline(&registry).map_err(js_err)?` | the image is decoded first (`:207`) but nothing is drawn before the build | `JsError`, a thrown JS `Error`. No abort, no panic. |

`new_image` and `new_text` are **public and infallible** (`-> Self`) in released 0.8.1:
`pub mod operation` is in `src/lib.rs:48`, and `tests/watermark_damage.rs` and
`tests/recipe_watermark.rs` call them directly. Making them return `Result` breaks semver in a
patch release. `Refusal` is not `#[non_exhaustive]`, so a new variant breaks semver too. So
**one predicate on the built `Watermark`, called from two places**: `run_watermark` after it
builds the op (still before `run_pixel_op` reads any input), and the end of `from_params` (one
place for `apply`, `build`, `--watch` and wasm). An additive public method, for example
`Watermark::draws_nothing() -> Option<cause>` (the name is design's), keeps the library API
compatible.

**The order inside the CLI matters (DEC-106).** DEC-106 says every `watermark` flag check comes
before every file read. `--opacity 0`, blank `--text` and `--color …00` are decidable from the
flags alone, so they belong with SPEC-135's flag checks at the top of `run_watermark`. Then
`watermark /no/such.png --opacity 0` gives exit 2, not 3, as `--size nan` does today. A too-small
`--size` and a transparent `--image` need the rendered or loaded overlay, so they come after the
font or overlay read and before any input. Design decides whether the flag-decidable causes are
separate pre-IO checks or the same predicate called early.

**The browser editor is not affected.** `integrations/ddphotos/edit` writes watermark steps
itself, but its opacity slider has `min="0.1"` (`index.html:105`). Empty text turns the mark off
(`edit.js:244`). `<input type="color">` has no alpha. The size is `max(1, …)` (`edit.js:230`),
and size 1 draws (6-7 px measured). None of its outputs can hit the new refusal.

### 5. Messages and exit codes

SPEC-135's refusals, driven today: `watermark … --size nan` gives exit 2 and
`error: --size must be a number > 0 and <= 1024, got nan`, also with a missing input (exit 2, not
3). The same value in `apply --recipe` gives exit 1 and `error: recipe step 0 ('watermark'):
'size' must be a number > 0 and <= 1024, got nan`. In `build` it gives exit 1 with the
`target #0 …` prefix. The new refusals use the same codes and the same shapes: a CLI line that
names the flag, and a recipe line that names the key under DEC-102's `recipe step N ('watermark'):`
prefix. Proposed wording (design finalises it; trap 10: test at least one value whose `f32`
rendering is inexact):

| Cause | CLI (exit 2) | Recipe (exit 1, after the step prefix) |
|---|---|---|
| opacity 0 | `error: --opacity 0 draws no mark; to leave an image unmarked, do not run watermark` | `'opacity' = 0 draws no mark; to leave images unmarked, remove this step` |
| opacity > 0 but no alpha survives | `error: --opacity 0.000001 is too faint: the mark would draw no visible pixel` | `'opacity' = 0.000001 is too faint: the mark would draw no visible pixel` |
| blank text | `error: --text has no visible characters` | `'text' has no visible characters` |
| size too small | `error: --size 0.01 is too small: the text draws no visible pixel; use a larger --size` | `'size' = 0.01 is too small: the text draws no visible pixel; use a larger size` |
| colour alpha 00 | `error: --color ffffff00 is fully transparent: use an alpha above 00, or RRGGBB` | `'color' "ffffff00" is fully transparent: use an alpha above 00, or RRGGBB` |
| transparent image | `error: --image clear.png has no visible pixels (every pixel is fully transparent)` | `'image' "clear.png" has no visible pixels (every pixel is fully transparent)` |

Telling "blank" from "too small" from "colour alpha 00" apart (all three render with no visible
pixel) is a classification on the inputs, not a new measurement. Check the colour's alpha first,
then whether the text is all whitespace, and otherwise call it too small. **Design should keep
`--opacity 0` off DEC-102's key table.** Changing the row to `min_inclusive: false` would also
reword today's `--opacity 1.5` message (`must be a number in 0.0..=1.0` →
`must be a number > 0 and <= 1`). That message is pinned in `tests/watermark_cli_params.rs:269`,
`:276`, `:463` and in `docs/data-model.md:338` and `docs/api-contract.md:535`, and the table's
phrase cannot say "remove the step".

### 6. Bytes

The refusal is new, and nothing else may change. Design must require these checks:
- A **before/after `cmp`** against a binary built from the pre-change `main`. Rows: every
  watermark corpus fixture (`tests/fixtures/recipe_corpus/v1_watermark_image_placement.toml`,
  `v1_watermark_text_font.toml` with `color = "ff000080"`, `v2_watermark_integer_numbers.toml`);
  the CLI controls (text, image, `--opacity 0.6`, `--tile`, `--scale 0.1`, a 16-bit base); and
  **the faint-but-accepted rows**: `--opacity 0.00001`, `--size 0.2`, white on white. These still
  exit 0 and stay byte-identical, which proves the rule does not over-refuse.
- `visible` and the coverage cap are unchanged. `opacity_does_not_change_coverage` stays green
  **without edits**.
- No new recipe key and no change to `params()`, so no lockfile or cache-key change (DEC-102).

### 7. Docs: every sentence that changes

| Where | Today | Change |
|---|---|---|
| `recipes/README.md:236-237` | "A mark that fits is drawn as asked, even one too small or faint to see (`opacity = 0`, a tiny `size`)." | **Contradicts the fix.** Rewrite: a mark with no visible pixel after opacity is refused; a faint one is drawn even if it changes nothing on a given image. |
| `recipes/README.md:185` | `opacity` … "`0.0`–`1.0`. Default `1.0`." | "above 0, up to 1.0; 0 is refused (remove the step instead)" |
| `docs/cli-reference.md:266` | `--text` "Text to render." | add: must contain a visible character |
| `docs/cli-reference.md:268` | `--opacity` "Opacity `0.0`–`1.0`." | above 0, at most 1 |
| `docs/cli-reference.md:281` | "A bad `--size`, `--scale`, `--opacity`, … exits `2` before any file…" | add the no-visible-pixel refusal (exit 2) |
| `skills/crustyimg/SKILL.md:186` | the same exit-2 sentence | the same addition |
| `docs/data-model.md:214-215`, `:232` | `opacity` "(`0.0..=1.0`, default `1.0`)"; table row `0.0..=1.0` | keep the row (see 5); add one sentence under the table: opacity 0 and marks with no visible pixel are refused at load |
| `docs/api-contract.md:371`, `:384-385`, refusal list `:415-426` | "`--opacity O` (0–1, default 1)"; the exit-2 list | add the refusal, its exit codes and the faint-mark sentence |
| `src/cli/mod.rs:426-427` | `--opacity` has **no help line at all** | add `/// Mark opacity: above 0, at most 1 (default 1).` |
| `src/cli/mod.rs:412` | `--text` "Text to render (text mode; XOR `--image`)." | optional: "must contain a visible character" |
| `CHANGELOG.md` Unreleased | — | **Changed** (user-visible): `--opacity 0` / `opacity = 0`, blank text, a too-small `size`, colour alpha 00 and a transparent overlay used to exit 0 unmarked and are now refused (exit 2 CLI, exit 1 recipe). Anyone using `opacity 0` as an off switch must remove the step. |

### 8. Size: **[S]**, upper end

The code is one predicate (a max-alpha scan, which can share the `visible_pixels` pass), one
cause classifier, two call sites, and the early flag checks on the CLI: about 60-100 lines. The
verification is the larger part. Refusal matrix: 6 causes on the CLI + 6 on `apply` + `build`
(one row: it shares `from_params`) + wasm (2 text-mode rows) = **~16 refusal rows**. Then the
CLI order rows (missing input + `--opacity 0` → 2; `--image /no/x.png --opacity 0` → 2), the
accept side (3 faint rows + controls, byte-equal), the boundary rows from 3, and their
mutations (trap 12: name where each mutation is applied, on both paths). That is in line with
SPEC-135, which was [S].

**The cut if it grows:** keep all six causes. Cutting the "too faint" cause would reopen the gap
at `--opacity 0.000001`, which is trap 9. Shrink the matrix instead: `build` and wasm get one row
each (they share `from_params` with `apply`), and the six messages can share one template
(`… draws no visible pixel: <cause>`).

### Scope

**In:** the refusal of a mark with no pixel of alpha > 0 after opacity, on the CLI, `apply`,
`build` (+ `--watch`) and wasm `transform`. The six causes and their messages. The CLI order
under DEC-106. The docs, help, CHANGELOG and DEC listed in 7.

**Out:**
- "A text watermark's raster has no memory bound beyond `size`": a separate STAGE-054 entry.
- `render_text` panicking for a library caller at a huge size: a separate STAGE-054 entry.
- A library caller of `Watermark::new_image`/`new_text` that never calls the new predicate can
  still build a mark that draws nothing. Closing it changes the public API (fallible
  constructors or a new `Refusal` variant), so it waits for a minor release. Filed as a new
  STAGE-054 entry.
- Faint marks that change no pixel of a particular image (white on white, opacity
  < 1.5/255 on 8-bit, size 0.2). They are drawn as asked by design (maintainer question 1).
- An image mark whose `scale` resize (Lanczos) leaves no alpha > 0. That depends on the image
  width and is decided per image in `apply`. Not observed: `--image logo.png --scale 0.00001`
  draws a 1×1 mark that changes 1 px. Not filed.
- Changing `visible` or the 5 % coverage rule (DEC-104).

### Rough build file list

- `src/operation/mod.rs`: the max-alpha (post-opacity) predicate and its cause; called at the
  end of `Watermark::from_params`; unit tests beside `opacity_does_not_change_coverage`.
- `src/cli/ops.rs`: early checks in `run_watermark` for opacity 0, blank text and colour alpha
  00; the predicate call after the op is built.
- `src/text/mod.rs`: probably untouched (the 1×1 fallback stays a valid overlay; the predicate
  reads it). Design confirms.
- `src/cli/mod.rs`: `--opacity` and `--text` help.
- `tests/watermark_cli_params.rs` (CLI rows and order), `tests/recipe_watermark.rs` or
  `tests/recipe_strict_params.rs` (recipe rows), `tests/build_watermark_cache.rs` or a build row,
  `tests/wasm_roundtrip.rs` (wasm text rows).
- Docs: `recipes/README.md`, `docs/cli-reference.md`, `docs/api-contract.md`,
  `docs/data-model.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`.
- `decisions/DEC-109-…md` (reserved), **or** an amendment to DEC-104, at design's call. My lean
  is **DEC-109**. DEC-104's refusals are per image (a batch skips the image), while this one is
  per invocation and decided from the mark alone. It also refines DEC-106's order rule.

### Maintainer questions

1. **Is the rule "the mark has no visible pixel after opacity" (decided from the mark), rather
   than "this image would change no pixel"?** The second is image-dependent: white on white never
   changes; opacity 0.005 changes nothing on 8-bit but all of a 16-bit image. Applying it would
   also make it a per-image skip, not an invocation error. **Recommend yes, decided from the mark**,
   matching DEC-104, with one doc sentence about faint marks.
2. **Is a positive opacity too small to keep any alpha (`--opacity 0.000001`) refused too?** It
   follows from the same rule; without it, a tiny opacity becomes the new invisible mark.
   **Recommend yes.**
3. **Leave the public constructors `new_image`/`new_text` infallible in 0.8.2, and close the
   library-caller gap at the next minor?** **Recommend yes.** Filed as a STAGE-054 entry.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- The memory bound on a text mark's raster (STAGE-054 entry "A text watermark's raster has no
  memory bound beyond `size`").
- `render_text` panicking for a library caller at a huge size (its own STAGE-054 entry).
- A library caller of `Watermark::new_image`/`new_text` that skips the new predicate. It needs a
  public-API change, so it waits for the next minor (filed as a new STAGE-054 entry).
- Faint marks that change no pixel of a particular image (white on white, opacity < 1.5/255 on an
  8-bit image, size 0.2). They are drawn as asked; the docs say so.
- A `scale` resize that leaves an image mark with no alpha (per image, at `apply`; not observed).
- Any change to `visible`, the 5 % coverage cap, or DEC-102's `opacity` table row.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
