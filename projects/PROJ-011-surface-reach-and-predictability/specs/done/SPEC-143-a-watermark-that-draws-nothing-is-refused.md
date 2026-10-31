---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-143
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - DEC-109   # a watermark that would draw nothing is refused, decided from the mark at 16-bit precision (this design)
    - DEC-015   # exit codes: 2 usage, 1 recipe, 3 input
    - DEC-045   # the bundled font; its .notdef box is why tab and U+3000 draw
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4115263
      tokens_breakdown:
        input: 60
        output: 67627
        cache_creation: 158602
        cache_read: 3888974
      estimated_usd: 4.63
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Interactive design session in its own worktree (docs/spec-143-design). Measured from the
        session's transcript (99391a8f-….jsonl, identified by content), deduped by `.message.id`
        (30 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25,
        cache_read x0.10: $0.0003 + $0.99 + $1.94 + $1.69. Measured before the commit/push turns,
        so it slightly under-counts.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 10917598
      tokens_breakdown:
        input: 140
        output: 55453
        cache_creation: 176437
        cache_read: 10685568
      estimated_usd: 4.7
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Build session in its own worktree (feat-spec-143-refuse-invisible-watermark). Re-measured at ship from the full transcript (8e0c9f1c-….jsonl, identified by content: 'BUILD prompt: a watermark'), deduped by `.message.id` (68 unique ids, all claude-sonnet-5-5): input/cache_creation/cache_read once per id, MAX output_tokens per id. Priced by component at Sonnet $3/$15 per MTok, cache_creation x1.25, cache_read x0.10: $0.0004 + $0.66 + $3.21 + $0.83. Replaces the earlier $3.88 / 8,667,901 tokens, which was measured before the final turns.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 14588346
      tokens_breakdown:
        input: 166
        output: 72981
        cache_creation: 202142
        cache_read: 14313057
      estimated_usd: 10.25
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Verify session in its own worktree (verify-spec-143). Measured at ship from the full transcript (644d2ca8-….jsonl, identified by content: 'VERIFY prompt: a watermark'), deduped by `.message.id` (83 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once per id, MAX output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10: $0.0008 + $1.26 + $7.16 + $1.82. Outcome: punch list (docs and tests only).
    - cycle: punchlist
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 5491020
      tokens_breakdown:
        input: 114
        output: 27154
        cache_creation: 99820
        cache_read: 5363932
      estimated_usd: 2.39
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Punch-list session in its own worktree (fix-spec-143-punch). Re-measured at ship from the full transcript (6219b4c5-….jsonl, identified by content: 'PUNCH LIST prompt: docs that match'), deduped by `.message.id` (55 unique ids, all claude-sonnet-5-5): input/cache_creation/cache_read once per id, MAX output_tokens per id. Priced by component at Sonnet $3/$15 per MTok, cache_creation x1.25, cache_read x0.10: $0.0003 + $0.37 + $1.61 + $0.41. Replaces the earlier $1.86 / 4,081,425 tokens, which was measured before the final turns.
    - cycle: ship
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1195190
      tokens_breakdown:
        input: 28
        output: 5581
        cache_creation: 63717
        cache_read: 1125864
      estimated_usd: 0.66
      duration_minutes: null
      recorded_at: 2026-10-31
      note: >
        Ship session in its own worktree (docs-spec-143-ship). Measured from the session's transcript (7963ea1d-….jsonl, identified by content: 'SHIP prompt: a watermark'), deduped by `.message.id` (14 unique ids, all claude-sonnet-5-5), priced at Sonnet $3/$15 per MTok with cache_creation x1.25 and cache_read x0.10. Measured before the final cost-edit, commit and push turns, so it slightly under-counts.
  totals:
    tokens_total: 43467305
    estimated_usd: 28.18
    session_count: 6
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

### Maintainer rulings (2026-10-30, settled)

1. **Decided from the mark.** A mark with no pixel of alpha > 0 after opacity is refused. A faint
   mark that changes nothing on a particular image (white on white, a low opacity on 8-bit) is
   drawn as asked.
2. **A positive opacity too small to keep any alpha is refused too**, with the "too faint" cause.
3. **`new_image` / `new_text` stay public and infallible in 0.8.2.** The library-caller gap is
   the STAGE-054 entry the frame filed, for the next minor release.

## Design

Design session 2026-10-30 (Opus 5.5), own worktree `docs/spec-143-design`, on `main` @ `6cfdbf9`.
Every row below was driven on a release build of that commit (`crustyimg 0.8.1`) in a fresh
`CARGO_TARGET_DIR`, deleted afterwards. Changed pixels were counted exactly with ImageMagick
(difference, max over channels, threshold 0, count); negative control: a base against itself
counts 0. Fixtures: 400×300 `dark` (#202020, 8-bit), `dark16` (#202020, 16-bit), **`black16`**
(black, 16-bit), `white`; `logo.png` (60×30 opaque white), `logo1.png` (60×30 white, alpha8 1),
`clear.png` (60×30, alpha 0).

**`black16` is the detector for "the mark kept some alpha".** On a black 16-bit base, a white
pixel with alpha16 = 1 changes a sample (measured: `--opacity 0.0000077` changes 1800 px there and
0 px on `dark16`, whose truncating blend swallows alpha16 = 1). So a 0-px count on `black16` means
"no alpha survived", and a non-zero count means "some did". Every boundary row uses it.

### What `main` does today (the RED baseline)

Every refusal row below exits **0** on `main` with **0 px changed** and no stderr; every order row
exits **3**. Driven:

| Row | Base | `main` |
|---|---|---|
| CLI `--text " "`, `--text $' '` | dark16 | 0, 0 px |
| CLI `--image logo.png --opacity 0.0000076` | black16 | 0, **0 px** |
| CLI `--image logo.png --opacity 0.0000077` | black16 | 0, **1800 px** |
| CLI `--text "© crustyimg" --color ffffff01 --opacity 0.0019` | black16 | 0, **0 px** |
| CLI `--text "© crustyimg" --color ffffff01 --opacity 0.002` | black16 | 0, **1133 px** |
| CLI `--image logo1.png --opacity 0.0019` / `0.002` | black16 | 0, 0 px / 0, 1800 px |
| CLI `--text "© crustyimg" --size 0.17` / `0.18` | black16 | 0, **0 px** / 0, **1 px** |
| CLI `--image logo.png --opacity 0.001` | dark (8-bit) / dark16 | 0, **0 px** / 0, **1800 px** |
| recipe `opacity = 0` (integer), `opacity = 1e-50`, `opacity = 0.000001`, `opacity = 0.0000076`, `text = "  "`, `image = "clear.png"`, `color = "#ffffff00"` | black16 | 0, 0 px each |
| recipe `opacity = 0.0000077` (image) | black16 | 0, 1800 px |
| CLI `watermark /no/such.png --text hi --opacity 0` | — | **3** `input not found or unreadable` |
| CLI `--image /no/l.png --opacity 0` | — | **3** `could not read image input` |
| CLI `--text " " --font /no/f.ttf` | — | **3** (stays 3 after the fix, see Call 3) |
| `apply --recipe` (`opacity = 0`) `/no/such.png` | — | **3** |

**Odd characters (bundled Go font), `--text <c>` on dark16, opacity 1:**

| Text | px | Why |
|---|---|---|
| `" "` (U+0020), U+00A0 NBSP | **0** | mapped to an empty glyph: no outline, the 1×1 fallback |
| U+00A0 + `a` | 186 | draws the `a` |
| U+0009 tab, U+000B, U+0085, U+1680, U+2000, U+2003, U+200A, U+2028, U+202F, U+205F, U+3000 | **68** each | `char::is_whitespace` is true, but the font has no glyph: the `.notdef` box is drawn |
| U+200B ZWSP, U+200D ZWJ, U+2060, U+034F, U+180E, U+FE0F, U+FEFF, U+0001, U+4E2D (CJK), U+1F600 (emoji) | **68** each | no glyph: `.notdef` box |
| U+00AD soft hyphen | 27 | the font draws it |

So with the bundled font, the only texts that draw nothing at a normal size are runs of U+0020 and
U+00A0. **Blank text is not decidable from the flag:** `--text $'\t'` and `--text $'　'` are
whitespace by every definition and still draw a 68-px mark. A pre-render `is_whitespace` check
would refuse marks that draw, against ruling 2. This corrects the frame (§4 listed blank text as
flag-decidable); see Call 3.

### Call 1 — the predicate, and why it cannot disagree with the composite

**The 8-bit vs 16-bit numbers.** For an overlay whose largest alpha is `a8` (8-bit overlay, so its
16-bit view is `a16 = 257·a8`), the two composite branches leave every pixel at alpha 0 when:

| Branch (chosen by the **base** image's bit depth, `is_sixteen_bit(original_color)`, `src/operation/mod.rs:1758`) | zero alpha when | opaque mark (`a8 = 255`) | `a8 = 1` mark |
|---|---|---|---|
| 16-bit (`:1777-1780`, `(a16 as f32 * op).round() as u16`) | `op < 0.5 / (257·a8)` | `op < 7.6295e-6` | `op < 1.9455e-3` |
| 8-bit (`:1833-1836`, `(a8 as f32 * op).round() as u8`) | `op < 0.5 / a8` | `op < 1.9608e-3` | `op < 0.5` |

- **The gap exists one way.** For `0.5/(257·a8) ≤ op < 0.5/a8` the mark keeps alpha at 16 bits but
  rounds to 0 at 8 bits. Driven: `--image logo.png --opacity 0.001` changes 1800 px of `dark16`
  and 0 px of `dark`.
- **The reverse cannot happen.** If `round(a16·op) = 0` then `a16·op < 0.5`. For an 8-bit overlay
  `a8·op = a16·op/257 < 0.002`, so `round(a8·op) = 0`. For a 16-bit overlay `to_rgba8` gives
  `a8 ≤ (a16 + 128)/257`, so `a8·op < 0.5·(1 + 128/a16)/257 ≤ 0.251`, which rounds to 0. A mark
  the 16-bit rule refuses is always drawn as nothing on the 8-bit branch too.
- **The predicate follows the 16-bit branch, not the stricter 8-bit one**, for three reasons.
  (1) The branch is chosen by the image, and ruling 2 says the rule is decided from the mark.
  (2) The 8-bit rule would refuse `--opacity 0.001`, which draws all 1800 px of a 16-bit image.
  (3) It would not make 8-bit output change anyway: on `dark` an alpha8 of 1 changes 0 px (the
  frame measured the first change at alpha 2, `op ≥ 0.00589`). An 8-bit output that a faint mark
  leaves unchanged is exactly the "faint mark that changes nothing on one particular image" that
  ruling 2 draws as asked. **On the 16-bit branch the predicate and the composite cannot disagree,
  because they call the same function** (below). On the 8-bit branch an accepted mark may draw
  zero alpha, by ruling; the docs say so.

**The code.** All in `src/operation/mod.rs`:

```rust
/// An opacity applied to one 16-bit alpha, exactly as the composite applies it.
fn faded_alpha16(alpha: u16, opacity: f32) -> u16 {
    (alpha as f32 * opacity).round() as u16
}

/// The overlay's largest alpha, on its 16-bit view (exact for 8- and 16-bit overlays).
fn max_alpha16(overlay: &DynamicImage) -> u16 {
    overlay.to_rgba16().pixels().map(|p| p.0[3]).max().unwrap_or(0)
}

/// Why a watermark would draw no visible pixel. Checked in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvisibleMark {
    /// `opacity` is 0.
    OpacityZero,
    /// A text mark's colour has alpha 0.
    TransparentColor,
    /// A text mark whose text is all whitespace, and draws nothing.
    BlankText,
    /// A text mark that draws nothing at its size.
    TextTooSmall,
    /// An image mark with no pixel of alpha > 0.
    TransparentImage,
    /// The mark has visible pixels, but `opacity` rounds every alpha to 0.
    TooFaint,
}

impl Watermark {
    /// Why this mark would draw no visible pixel, or `None` when it draws one.
    /// Decided from the mark alone, at 16-bit precision. A faint mark can still
    /// change nothing on a particular image (white on white, or an 8-bit image).
    pub fn invisible_mark(&self) -> Option<InvisibleMark> { … }
}
```

- The 16-bit composite branch's opacity loop (`:1777-1780`) becomes
  `px.0[3] = faded_alpha16(px.0[3], self.opacity);`. It is the same expression, so no byte changes
  (AC-11 proves it). The 8-bit branch (`:1833-1836`) is **not** touched.
- `Watermark` gains one private field `max_alpha: u16`, set by `max_alpha16(&overlay)` in all three
  constructors (`new_image` `:1451`, `new_text` `:1482`, `from_params` `:1634`), next to `visible`.
  `visible_pixels` and `visible` are untouched (a separate pass over the overlay; the mark is small).
  This keeps DEC-104's coverage figure out of reach.
- `invisible_mark()`, in this order:
  1. `self.opacity == 0.0` → `OpacityZero` (also true for `-0.0`, and for an f64 below f32's
     smallest positive value, which reads as `0.0`);
  2. text mode and `color[3] == 0` → `TransparentColor`;
  3. `self.visible == 0` → text mode: `text.chars().all(char::is_whitespace)` → `BlankText`,
     otherwise `TextTooSmall`; image mode → `TransparentImage`;
  4. `faded_alpha16(self.max_alpha, self.opacity) == 0` → `TooFaint`;
  5. otherwise `None`.
- Step 4 is monotone: `round(a·op)` never decreases as `a` grows, so the max alpha decides it for
  every pixel. `TooFaint` implies `0 < opacity < 1`: at opacity 1, `faded_alpha16(max, 1.0) = max ≥ 1`
  whenever `visible > 0`.
- **Additive only:** a new public enum (`#[non_exhaustive]` from day one), a new public method, a
  new private field, two private functions. `new_image`/`new_text` keep their signatures and stay
  infallible; `Refusal` is untouched. Re-export `InvisibleMark` wherever `Watermark` is exported.

### Call 2 — where it runs

- **Recipe (`apply`, `build`, `--watch`, wasm `transform`):** at the end of
  `Watermark::from_params` (`src/operation/mod.rs`, after `let visible = …` at `:1634`, before
  `Ok(Watermark { … })`): build the struct, call `invisible_mark()`, and on `Some(cause)` return
  `RegistryError::InvalidParams { op: "watermark", reason }` with the recipe line from Call 4. The
  message is built **in `from_params`**, from the params as written (`color` and `image` strings,
  `size` and `opacity` as the f32 the op holds). One call site covers every recipe surface, since
  all four reach `from_params` through the registry (`src/operation/registry.rs:306`).
- **CLI `watermark`:** in `run_watermark` (`src/cli/ops.rs:1430`), two places (Call 3).
- The CLI message is built in `src/cli/ops.rs` from the flags as typed; the recipe message in
  `src/operation/mod.rs` from the keys. Two private `match`es over `InvisibleMark`, one per surface.
  No shared template: each line says what to do on its own surface ("do not run watermark" vs
  "remove this step").

### Call 3 — the CLI order (DEC-106)

`run_watermark` becomes:

1. the existing flag checks, unchanged: `--size`, `--scale`, `--opacity` (table range), `--gravity`,
   `--color` (parse);
2. **new flag checks**, right after the colour parses: `opacity == Some(0.0)` → the `OpacityZero`
   line; text mode (`src.image.is_none()`) with `color[3] == 0` → the `TransparentColor` line;
3. `watermark_overlay` (reads `--font` or `--image`: a missing file is exit 3, as today);
4. build the op (unchanged);
5. **new:** `if let Some(cause) = op.invisible_mark()` → `CliError::Usage(<CLI line>)`, exit 2;
6. `run_pixel_op` (the first read of any input).

- **Only opacity 0 and colour alpha 00 are flag checks.** Both are true of the mark whatever the font
  or overlay. Blank text is not (tab and U+3000 draw a box), nor is a too-small size or a transparent
  image. These three are mark checks: after the font/overlay read, before any input.
- Step 5 also re-checks causes 1–2 (the predicate is complete on its own). Under mutation M3 below,
  that is what keeps the opacity-0 and colour rows passing with the same words, which is why M3 is
  only caught by the order rows.
- **Rows that prove each order** (FT-4): `watermark /no/such.png --text hi --opacity 0` → 2
  (flag check before input); `--image /no/l.png --opacity 0` → 2 (flag check before the overlay
  read); `--text hi --font /no/f.ttf --color ffffff00` → 2 (before the font read);
  `watermark /no/such.png --text " "` → 2 and `watermark /no/such.png --image clear.png` → 2 (mark
  check before input); `--text " " --font /no/f.ttf` → **3** (the font is read before blankness is
  known; pinned so a pre-render whitespace check shows up).

### Call 4 — the twelve messages

CLI: exit 2, stderr is exactly the one line. Recipe: exit 1, the line follows DEC-102's prefix,
`error: recipe step N ('watermark'): ` (in `build`, after `error: target #N (recipe …) `).

| Cause | CLI (exit 2) | Recipe (exit 1, after the prefix) |
|---|---|---|
| `OpacityZero` | `error: --opacity 0 draws no mark; to leave an image unmarked, do not run watermark` | `'opacity' = 0 draws no mark; to leave images unmarked, remove this step` |
| `TooFaint` | `error: --opacity 0.000001 is too faint: the mark would draw no visible pixel; use a larger --opacity` | `'opacity' = 0.000001 is too faint: the mark would draw no visible pixel; use a larger opacity` |
| `BlankText` | `error: --text has no visible characters; use text with at least one visible character` | `'text' has no visible characters; use text with at least one visible character` |
| `TextTooSmall` | `error: --text draws no visible pixel at --size 0.01; use a larger --size` | `'text' draws no visible pixel at 'size' = 0.01; use a larger size` |
| `TransparentColor` | `error: --color ffffff00 is fully transparent; use an alpha above 00, or RRGGBB` | `'color' "ffffff00" is fully transparent; use an alpha above 00, or RRGGBB` |
| `TransparentImage` | `error: --image clear.png is fully transparent; use an image with visible pixels` | `'image' "clear.png" is fully transparent; use an image with visible pixels` |

- **Values.** `OpacityZero` prints a literal `0`, never the value (so `-0` and `opacity = 1e-50`
  both read `0`; the f32 the op holds is 0). `TooFaint`'s opacity and `TextTooSmall`'s size print
  the f32 with `{}` (Rust's shortest round-trip form: `0.000001`, `0.1`, `0.0000076`), via the
  existing `flag_value_text` on the CLI. **Never widen to f64**: `f64::from(0.000001_f32)` prints
  `0.0000009999999974752427`, and `0.1_f32` prints `0.10000000149011612` (trap 10; FT-1 has both).
  `--color` and `--image` print the string as typed (`#ffffff00` stays `#ffffff00`); the recipe
  wraps it in `"…"` (`{:?}`), as DEC-102's other string messages do. `TextTooSmall` with no
  `size` key prints the default, `'size' = 32`.
- **`TextTooSmall` names the size, not a cause it cannot prove.** With the bundled font, a
  non-whitespace text that draws nothing at opacity 1 only happens at a small size. With a custom
  `--font` that maps a character to an empty glyph, "draws no visible pixel at --size 32" is still
  true.
- **`--text` is not echoed** in `BlankText`: it is whitespace, and printing it shows nothing.

### Call 5 — the boundary rows (trap 10)

| Mark | Largest refused | Smallest accepted | Arithmetic (`round(a16·op)`) | `main` on black16 |
|---|---|---|---|---|
| opaque (`a16 = 65535`) | `0.0000076` | `0.0000077` | 0.498 → 0 / 0.505 → 1 | 0 px / 1800 px |
| alpha8 1 (`a16 = 257`): `--color ffffff01`, `logo1.png` | `0.0019` | `0.002` | 0.488 → 0 / 0.514 → 1 | 0 px / 1133 px (text), 1800 px (image) |
| 16-bit overlay, `a16 = 4` (lib + CLI `faint16.png`) | `0.1` | `0.13` | 0.4 → 0 / 0.52 → 1 | 0 px / 1800 px (and 0 px on 8-bit `dark` even at opacity 1) |
| text `"© crustyimg"`, opacity 1 | size `0.17` | size `0.18` | `visible` 0 / > 0 | 0 px / 1 px |

The alpha8-1 row is where the naive shortcut differs: `op × 65535` gives 124.5 at `0.0019` and
would accept a mark that draws nothing. The 16-bit-overlay row is where 8-bit and 16-bit
precision differ at opacity 1: its 8-bit view has alpha 0. Each pair is a test (FT-2, FT-3, FT-14)
and each has a mutation (M6–M8). A row refusing only `0.000001` could not tell this threshold from
"refuse below 0.01".

### Call 6 — bytes

- **In repo (CI-guarded):** extend `accepted_cases()` in `tests/watermark_cli_params.rs` (the
  existing golden test, `tests/fixtures/watermark/cli_params.golden`) from 3 cases to 13. Change the
  tuple's base `(u32, u32)` into a small `Base` enum: `Gradient(w, h)` (today's `write_gradient`),
  `Gradient16(w, h)` (the same gradient written as `Rgb16`, each sample ×257), `White(w, h)`. New
  cases, each run on the CLI and as a one-step recipe:

  | name | base | CLI args | recipe body |
  |---|---|---|---|
  | `image_opacity_0_6` | Gradient(400,300) | `--image logo.png --opacity 0.6` | `image = "logo.png"`, `opacity = 0.6` |
  | `image_tile` | Gradient(400,300) | `--image logo.png --tile` | `image = "logo.png"`, `tile = true` |
  | `image_scale_tenth` | Gradient(400,300) | `--image logo.png --scale 0.1` | `scale = 0.1` |
  | `text_16bit` | Gradient16(400,300) | `--text "© crustyimg"` | `text = "© crustyimg"` |
  | `faint_opacity` | Gradient16(400,300) | `--image logo.png --opacity 0.00001` | `opacity = 0.00001` |
  | `faint_opacity_boundary` | Gradient16(400,300) | `--image logo.png --opacity 0.0000077` | `opacity = 0.0000077` |
  | `faint_size` | Gradient16(400,300) | `--text "© crustyimg" --size 0.2` | `size = 0.2` |
  | `faint_8bit` | Gradient(400,300) | `--image logo.png --opacity 0.001` | `opacity = 0.001` |
  | `white_on_white` | White(400,300) | `--text "© crustyimg"` | `text = "© crustyimg"` |

  **The goldens are blessed on unmodified `main` source**, in the build's **first commit** (test
  changes only, no `src/` change), with `CRUSTYIMG_BLESS=1 cargo test --test watermark_cli_params
  bless -- --ignored`. The fix comes in later commits. The test asserts the golden has 12 lines.
  It is green on `main` by construction; it is the guard that the fix changes no accepted pixel.
  **Mutations that turn it red:** (a) in the 16-bit branch, `faded_alpha16` using `.floor()`
  instead of `.round()` → `faint_opacity` (0.655 → 0) and `faint_opacity_boundary` (0.505 → 0)
  red: each now draws nothing, which hashes differently from the golden that drew (`text_16bit`
  stays green: at opacity 1 the loop does not run); (b) M6 (8-bit predicate) → `faint_opacity`, `faint_opacity_boundary`, `faint_8bit` red
  (`run_case` asserts exit 0). The `0.00001` and `0.001` faint rows guard against over-refusal.
- **At verify (out of repo):** build `main` and the branch in separate fresh target dirs; run each
  recipe-corpus fixture (`tests/fixtures/recipe_corpus/v1_watermark_image_placement.toml`,
  `v1_watermark_text_font.toml`, `v2_watermark_integer_numbers.toml`, assets from
  `tests/fixtures/recipe_corpus/` as `tests/recipe_round_trip.rs` resolves them) through
  `apply --recipe` with both binaries on the same input and `cmp` the outputs. Negative control:
  the same `cmp` against the output of the fixture with its `opacity`/`size` nudged must differ.
  The 0.8.2 version bump is not part of this spec; it changes cache keys (crate version), not
  watermark pixels.

### Call 7 — CHANGELOG: **Breaking**

Scripts that use `opacity 0` as an off switch go from exit 0 to exit 2 (CLI) or 1 (recipe). That is
the same shape as 0.8.1's `build` flags entry, which went under **Breaking**. Text for
`## [Unreleased]`:

```markdown
### Breaking

- **`watermark` refuses a mark that would draw nothing** (exit `2`; in a recipe, `apply` and
  `build` exit `1`; nothing is written). This covers `--opacity 0` / `opacity = 0`, an opacity
  too small to leave any visible pixel, text with no visible characters (`--text " "`), text too
  small to draw at its `--size`, a fully transparent `--color` (alpha `00`) and a fully
  transparent `--image`. These used to exit `0` and write the image without a mark. To leave
  images unmarked, remove the watermark step, or do not run `watermark`. A faint mark that still
  has a visible pixel is drawn as before, even where it changes nothing on a particular image
  (white text on a white photo, or a very low opacity on an 8-bit image).
```

### Call 8 — the DEC: new DEC-109

DEC-109 (written in this design commit) records the rule, the 16-bit precision with the gap
numbers, the six causes and their order, the two call sites, and the CLI order. It is a new DEC
rather than an amendment to DEC-104. DEC-104's refusals are per image (a batch skips the image and
carries on); this one is per invocation (exit 2, nothing written). It also refines DEC-106's
order: opacity 0 and colour alpha 00 join the flag checks, and the three mark checks sit between
the font/overlay read and the inputs. The build adds one line to DEC-106's Consequences pointing
to DEC-109 (doc edit, listed in Outputs).

### Size: [S], upper end

About 90 lines of code (the enum, the predicate, two message `match`es, three call points). The
test matrix is 16 failing tests, most of them table rows in existing files. The frame's cut is
already applied: `build` and wasm get one test each (two wasm rows). All six causes are kept.

## Inputs

- **Files to read:** `src/operation/mod.rs` (`Watermark`, `visible_pixels` `:1398`, constructors
  `:1440-1500`, `from_params` `:1518-1644`, `refusal`/`apply` `:1700-1860`);
  `src/cli/ops.rs` (`check_watermark_flag` `:1351`, `flag_value_text` `:1371`,
  `watermark_overlay` `:1390`, `run_watermark` `:1430`); `src/text/mod.rs` (`render_text` `:133`,
  the 1×1 fallback `:197`; read only, untouched); `src/wasm.rs:243`;
  `tests/watermark_cli_params.rs` (helpers, golden scheme); `tests/recipe_watermark.rs`;
  `tests/build_watermark_cache.rs` (build harness); `tests/wasm_roundtrip.rs:307`
  (`transform_refuses_a_mark_that_does_not_fit`, the shape to copy).
- **Related code paths:** `src/operation/registry.rs:306` (`from_params` registration).

## Outputs

- **Files created:** none (the golden file exists; it gains 9 lines).
- **Files modified:**
  - `src/operation/mod.rs`: `faded_alpha16`, `max_alpha16`, `InvisibleMark`, the `max_alpha`
    field, `invisible_mark()`, the `from_params` call and its recipe messages, the 16-bit
    branch's opacity loop calling `faded_alpha16`; unit tests FT-13 to FT-16.
  - `src/cli/ops.rs`: the two flag checks and the post-build call in `run_watermark`, the CLI
    message `match`.
  - `src/cli/mod.rs:426-427`: `--opacity` help `/// Mark opacity: above 0, at most 1 (default 1).`
    `:412` `--text` help: `Text to render; must draw at least one visible character (text mode;
    XOR --image).`
  - `src/lib.rs` (or wherever `Watermark` is re-exported): re-export `InvisibleMark`, if
    `Watermark` is re-exported there.
  - `tests/watermark_cli_params.rs`, `tests/fixtures/watermark/cli_params.golden`,
    `tests/recipe_watermark.rs`, `tests/build_watermark_cache.rs`, `tests/wasm_roundtrip.rs`.
  - Docs: `recipes/README.md:185` and `:236-237`; `docs/cli-reference.md:266`, `:268`, `:281`;
    `skills/crustyimg/SKILL.md:186`; `docs/data-model.md` (one sentence under the key table near
    `:232`; the `0.0..=1.0` row stays); `docs/api-contract.md:371`, `:384-385`, and the refusal
    list `:415-426`; `CHANGELOG.md` (Call 7); `decisions/DEC-106-…md` (one Consequences line).
- **New exports:** `pub enum InvisibleMark` (`#[non_exhaustive]`, six unit variants) and
  `pub fn Watermark::invisible_mark(&self) -> Option<InvisibleMark>`.
- **Database changes:** none. No recipe key, no `params()` change, no `Cargo.lock`, version or
  `CACHE_SCHEMA_VERSION` change.

## Acceptance Criteria

Surfaces: **CLI** = `crustyimg watermark`; **apply** = `crustyimg apply --recipe`; **build** =
`crustyimg build`; **wasm** = `transform`; **lib** = the library API. "Refused" on the CLI = exit 2,
stderr exactly the one Call 4 line, no output file, every input file byte-identical to before. On
apply = exit 1, one `error:` line equal to the prefix plus the Call 4 recipe line, no output, inputs
unchanged. Test inputs are generated in-test: `G(w,h)` an 8-bit RGB gradient, `black16` a 400×300
16-bit black RGB PNG, `logo` the existing 60×30 opaque gradient (`write_logo`), `logo1` 60×30
white with alpha8 1, `clear` 60×30 all alpha 0, `faint16` 60×30 16-bit RGBA white with alpha16 4.
"RED on `main`" means exit 0 with nothing refused, as driven in `## Design`.

**Refusals, every cause, every surface**
- [ ] **AC-1. CLI refuses each of the six causes.** On `G(400,300)` (boundary rows on `black16`):
      `--text hi --opacity 0`; `--image logo --opacity 0`; `--image logo --opacity 0.000001`;
      `--image faint16 --opacity 0.1`; `--text " "`; `--text $'  '`;
      `--text "© crustyimg" --size 0.01`; `--text hi --color ffffff00`;
      `--text hi --color '#ffffff00'`; `--image clear`. Each refused with its line (Call 4, values
      as in AC-8). RED: all exit 0.
- [ ] **AC-2. apply refuses each of the six causes.** One-step recipes on `black16`:
      `text = "hi"` + `opacity = 0` (integer); `image = "logo.png"` + `opacity = 1e-50` (below
      f32's smallest positive, trap 9: reads as `0`, reported as `'opacity' = 0`);
      `image` + `opacity = 0.000001`; `text = " "`; `text = "  "`; `text` + `size = 0.01`;
      `text` + `color = "ffffff00"`; `image = "clear.png"`. Each refused with its recipe line.
      RED: all exit 0.
- [ ] **AC-3. build refuses before writing anything.** A manifest with one target over two inputs
      whose recipe has `text = "hi"`, `opacity = 0` → exit 1; stderr has two `error:` lines, the
      target (`target #0`) and then the step (`recipe step 0 ('watermark'): 'opacity' = 0 draws no
      mark; to leave images unmarked, remove this step`); the output directory holds no file.
      RED: exit 0.
- [ ] **AC-4. wasm throws, and the module survives.** `transform` on a 24×24 PNG with
      `text = "hi"` + `opacity = 0`, and with `text = " "`: each returns `Err` whose message
      contains the recipe line (`'opacity' = 0 draws no mark`, `'text' has no visible characters`);
      no panic or abort; a following identity `transform` succeeds. Runner: `just wasm-test`
      (wasm-bindgen-test-runner). RED: both return `Ok`.

**The threshold (Call 1, Call 5)**
- [ ] **AC-5. The opacity threshold is where the mark's largest alpha rounds to 0 at 16 bits.**
      CLI on `black16`: `--image logo --opacity 0.0000076` refused (`TooFaint`, prints
      `0.0000076`); `--opacity 0.0000077` exit 0 and output ≠ input; `--text "© crustyimg"
      --color ffffff01 --opacity 0.0019` refused (`TooFaint`); `--opacity 0.002` exit 0 and
      output ≠ input. apply: `opacity = 0.0000076` (image) refused; `0.0000077` exit 0 and output
      ≠ input; `color = "ffffff01"` + `opacity = 0.002` exit 0 and output ≠ input. lib: the Call 5
      table rows (opaque, `a16 = 257`, `a16 = 4`) on both sides. RED: the refused rows exit 0.
- [ ] **AC-6. The size threshold is where the text first draws.** CLI on `black16`:
      `--text "© crustyimg" --size 0.17` refused (`TextTooSmall`, prints `0.17`); `--size 0.18`
      exit 0 and output ≠ input. RED: 0.17 exits 0.
- [ ] **AC-7. The predicate agrees with the 16-bit composite.** lib: for three white overlays
      (4×4, 8-bit opaque, 8-bit alpha 1, 16-bit alpha16 4; small enough that DEC-104's 5 % cap never fires) and the opacities
      `0, 0.000001, 0.0000076, 0.0000077, 0.0001, 0.0019, 0.002, 0.1, 0.13, 1.0`,
      `invisible_mark().is_none()` equals "`apply` on a 64×48 16-bit black RGB image changes at
      least one sample". 30 pairs, both outcomes present.

**Order and messages**
- [ ] **AC-8. Values print as typed, never widened.** The AC-1 lines read `--opacity 0.000001`,
      `--opacity 0.1` (the `faint16` row), `--color ffffff00`, `--color #ffffff00`,
      `--image clear.png`, `--size 0.01`; the AC-2 lines read `'opacity' = 0.000001`,
      `'opacity' = 0` (for both `0` and `1e-50`), `'color' "ffffff00"`, `'image' "clear.png"`,
      `'size' = 0.01`. None contains `0.0000009999999974752427` or `0.10000000149011612`.
- [ ] **AC-9. CLI: flag causes precede the font/overlay read; mark causes precede any input read.**
      `watermark /no/such.png --text hi --opacity 0` → 2 (`OpacityZero` line);
      `G --image /no/l.png --opacity 0` → 2 (`OpacityZero`);
      `G --text hi --font /no/f.ttf --color ffffff00` → 2 (`TransparentColor`);
      `/no/such.png --text " "` → 2 (`BlankText`); `/no/such.png --image clear` → 2
      (`TransparentImage`); a directory of two PNGs with `--opacity 0 --out-dir out` → 2, `out`
      absent or empty, both inputs unchanged; `G --text " " --font /no/f.ttf` → **3** (pinned).
      RED: the first six exit 3, 3, 3, 3, 3, 0.
- [ ] **AC-10. apply: the refusal precedes the input read.** `apply --recipe <opacity = 0>
      /no/such.png` → exit 1 with the `OpacityZero` recipe line. RED: exit 3.

**Nothing that draws is refused, and nothing accepted changes**
- [ ] **AC-11. Faint and odd marks that draw are accepted.** CLI: `--image logo --opacity 0.001`
      on a 16-bit gradient (exit 0, output ≠ input); `--image faint16` at opacity 1 on `black16`
      (exit 0, output ≠ input); `--text $'\t'`, `--text $'　'`, `--text $'‍'` on
      `G(400,300)` (exit 0, output ≠ input: the font's fallback box); `--text "© crustyimg"` on a
      white base (exit 0). apply: `text = "　"` (exit 0, output ≠ input).
- [ ] **AC-12. Accepted marks are byte-identical to `main`, on both paths.** The 12-case golden
      test (Call 6) passes, with goldens blessed on unmodified `main` in the build's first commit;
      the verify-time corpus `cmp` (Call 6) shows no difference.
- [ ] **AC-13. Nothing else moves.** `opacity_does_not_change_coverage` passes with no edit;
      every existing test in `tests/watermark_cli_params.rs` (including the `--opacity 1.5` /
      `1.1` rows), `tests/recipe_strict_params.rs`, `tests/watermark_damage.rs` and
      `tests/recipe_watermark.rs` passes unedited (except the golden count); `Watermark::new_image`
      and `new_text` keep their signatures (existing callers compile unchanged); no
      `Cargo.lock`, version, `CACHE_SCHEMA_VERSION` or `WATERMARK_PARAMS` change in the diff.

**Help, docs and record** (verify reads these; documentation has no green)
- [ ] **AC-14.** `watermark --help` has an `--opacity` line containing `above 0` (FT-7; RED: no
      line). The doc rows in Outputs are written, `recipes/README.md:236-237` no longer says
      `opacity = 0` is drawn, the CHANGELOG has the Call 7 entry under **Breaking**, DEC-106 has
      its pointer line, and no help text, message or CHANGELOG line carries a spec or DEC id.

## Failing Tests

Written during **design**, BEFORE build. Each is RED on `main` for the reason given.

- **`tests/watermark_cli_params.rs`** (extend)
  - **FT-1** `"a mark that draws nothing is refused on the cli"`: AC-1 + AC-8, the 10-row table;
    asserts exit 2, `error_lines(stderr) == [expected]`, no `-o` file, input bytes unchanged.
    Asserts the row count is 10. RED: every row exits 0.
  - **FT-2** `"the opacity threshold is where the last alpha rounds away on the cli"`: AC-5 CLI,
    4 rows on `black16`. RED: the two refused rows exit 0.
  - **FT-3** `"the size threshold is where the text first draws"`: AC-6. RED: 0.17 exits 0.
  - **FT-4** `"a mark that draws nothing is refused before any input is read"`: AC-9, 7 rows.
    RED: 3, 3, 3, 3, 3, 0 (the pinned `--font` row is 3 on `main` too).
  - **FT-5** `"a faint or odd mark that draws is accepted"`: AC-11 CLI, 6 rows. Green on `main`;
    it is the guard against over-refusal (M6 and M9b turn it red).
  - **FT-6** `accepted_marks_are_byte_identical_to_main` (extend): AC-12, 13 cases (the 13th, `image_opacity_0_6_16bit`, is a 16-bit mid-opacity
    composite), golden count 13. Green on `main` by construction.
  - **FT-7** `"opacity help states the range"`: AC-14's help half. RED: no `--opacity` help line.
- **`tests/recipe_watermark.rs`** (extend)
  - **FT-8** `"a recipe mark that draws nothing is refused before any output"`: AC-2 + AC-8 (8
    rows), AC-5's refused apply row (`opacity = 0.0000076`) and two low-alpha rows
    (`color = "ffffff01"` at `opacity = 0.0019`; the 16-bit alpha-4 `faint16` image at
    `opacity = 0.1`): 11 rows, exit 1, exact line, no
    output, input unchanged; asserts 11 rows. RED: all exit 0.
  - **FT-9** `"a recipe refusal wins over a missing input"`: AC-10. RED: exit 3.
  - **FT-10** `"a faint recipe mark is drawn"`: AC-5's accepted apply rows (2) and AC-11's
    `text = "　"` row. Green on `main`; guard.
- **`tests/build_watermark_cache.rs`** (extend)
  - **FT-11** `"build refuses a mark that draws nothing before writing"`: AC-3. RED: exit 0.
- **`tests/wasm_roundtrip.rs`** (extend; `just wasm-test`)
  - **FT-12** `"transform refuses a mark that draws nothing"`: AC-4, 2 rows plus the survival
    call. RED: both `Ok`.
- **`src/operation/mod.rs`** (unit, beside `opacity_does_not_change_coverage`)
  - **FT-13** `"invisible mark names each cause"`: every cause from `new_text`/`new_image`, plus
    `None` for a normal text and a normal image mark (the negative controls), plus the order:
    opacity 0 with colour alpha 0 → `OpacityZero`; colour alpha 0 with `" "` → `TransparentColor`;
    `" "` at size 0.01 → `BlankText`; `"© crustyimg"` at size 0.01 → `TextTooSmall`;
    `"\u{3000}"` → `None`. RED: does not compile (new API).
  - **FT-14** `"invisible mark threshold follows the largest alpha"`: the Call 5 lib rows (opaque
    `0.0000076`/`0.0000077`; `a16 = 257` `0.0019`/`0.002`; opaque `0.001` → `None`; `a16 = 4` at
    `1.0` → `None` and `0.1` → `TooFaint`). RED: does not compile.
  - **FT-15** `"invisible mark agrees with the composite"`: AC-7. RED: does not compile.
  - **FT-16** `"from params refuses a mark that draws nothing"`: `from_params` with `text = "hi"`,
    `opacity = 0` → `Err(InvalidParams)` whose reason equals the `OpacityZero` recipe line; with
    `image` bytes resolved and `opacity = 0.000001` → reason equals the `TooFaint` line. RED: `Ok`.

Normal-mark negative controls on every path: FT-6's text and image cases (CLI + apply), FT-13's
`None` rows (lib), and the existing `transform_runs_a_text_only_watermark_with_no_font_key`
(wasm) and `build_hits_when_overlay_bytes_unchanged` (build), all of which stay green.

### Mutation table (for verify; trap 12)

Apply each on the build branch, rebuild **clean** (fresh `CARGO_TARGET_DIR`), confirm the binary
changed, run the full list, and record which tests went red. "Recipe rows" = FT-8, FT-9, FT-11,
FT-12, FT-16. "CLI rows" = FT-1, FT-2, FT-3, FT-4.

| # | Mutation | Exact location | Expected red | Expected green |
|---|---|---|---|---|
| M1 | Drop the predicate from `from_params` | the `invisible_mark()` call at the end of `Watermark::from_params`, `src/operation/mod.rs` | FT-8 (11 rows), FT-9, FT-11, FT-12 (both rows; wasm), FT-16 | all CLI rows, FT-13 to FT-15 |
| M2 | Drop the post-build call from the CLI | step 5 of `run_watermark`, `src/cli/ops.rs` | FT-1 rows 3–7 and 10 (too faint ×2, blank ×2, too small, clear image), FT-2 refused rows, FT-3 0.17, FT-4 `/no/such.png --text " "` and `--image clear` (exit 3) | FT-1 rows 1–2, 8–9 (early checks), FT-4's flag rows, all recipe rows |
| M3 | Drop the CLI flag checks | step 2 of `run_watermark` | FT-4 `--image /no/l.png --opacity 0` and `--font /no/f.ttf --color ffffff00` (exit 3) | every other row (step 5 refuses with the same words), recipe rows |
| M4a | `visible == 0` alone, no post-opacity check | delete step 4 of `invisible_mark()` | FT-1 `0.000001` and `faint16 0.1` rows, FT-2 refused rows, FT-8 `0.000001` and `0.0000076` rows, FT-13, FT-14 `TooFaint` rows, FT-15, FT-16's `TooFaint` half | `OpacityZero` rows (step 1), blank/small/colour/image rows |
| M5 | Check per image, not per invocation | remove the three call points (CLI steps 2 and 5, `from_params`) and return an `OperationError` at the top of `Watermark::apply` when `invisible_mark()` is `Some` | every CLI and recipe row (message words and exit codes change; FT-4 and FT-9 exit 3), FT-12 (message), FT-11, FT-15 (it calls `apply`) | FT-13, FT-14 |
| M6 | 8-bit precision | `max_alpha16` reads `to_rgba8()` and step 4 compares `round(a8·op)` | FT-5 (`0.001`, `faint16`), FT-6 (`faint_opacity`, `faint_opacity_boundary`, `faint_8bit`), FT-2 and FT-10 accepted rows (`0.0000077`, `ffffff01 0.002`), FT-14 accepted rows, FT-15 | refused rows |
| M7 | The naive shortcut | step 4 compares `round(65535·op) == 0` (ignores the max alpha) | FT-2 `ffffff01 0.0019` row, FT-1 `faint16 0.1` row, FT-14 `a16 = 257` and `a16 = 4` refused rows, FT-15 | the opaque-mark rows |
| M8 | Floor instead of round | step 4 uses `(a·op) < 1.0` | FT-2 and FT-10 `0.0000077` / `0.002` rows, FT-6 `faint_opacity` (0.655) and `faint_opacity_boundary`, FT-14 accepted rows, FT-15 | refused rows |
| M9a | Classifier order swapped | step 3 tests "too small" before whitespace (always `TextTooSmall` for text) | FT-1 blank rows (2), FT-4 `/no/such.png --text " "`, FT-8 blank rows (2), FT-12 blank row, FT-13 | all other causes |
| M9b | Blank text as a flag check | step 2 also refuses `text.chars().all(char::is_whitespace)` before rendering | FT-4 pinned `--font` row (2, not 3), FT-5 tab / U+3000 rows, FT-10 U+3000 row | the blank rows (same words) |
| M10 | Widen to f64 in a message | `TooFaint` prints `f64::from(opacity)` (CLI and recipe) | FT-1 `0.000001` and `0.1` rows, FT-2 `0.0000076` row, FT-8 `0.000001` and `0.0000076` rows, FT-16 `TooFaint` half | all other rows |

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-109` (new, this design) — the rule, the 16-bit precision, the six causes and their order,
  the two call sites, the CLI order.
- `DEC-104` — `visible` and the 5 % cap are decided from the mark, before opacity; untouched.
- `DEC-106` — every watermark flag check before every file read; opacity 0 and colour alpha 00
  join that set; the build adds a pointer line.
- `DEC-102` — the recipe key table and its `recipe step N ('watermark'):` prefix; `opacity` stays
  `0.0..=1.0` so the `--opacity 1.5` message is unchanged.
- `DEC-031` — the CLI builds the op directly, hence two call sites.
- `DEC-015` — exit codes: 2 usage, 1 recipe/pipeline, 3 input.
- `DEC-045` — the bundled Go font and the skrifa/zeno rasterizer; its `.notdef` box is why
  tab and U+3000 draw.

### Constraints that apply

- `test-before-implementation` — the failing tests above, and the goldens blessed on `main`
  before any `src/` change.
- `untrusted-input-hardening` — a recipe from anywhere must not produce an unmarked image at
  exit 0.

### Prior related work

- `SPEC-135` (shipped) — the flag checks at the top of `run_watermark`, `flag_value_text`, the
  golden scheme in `tests/watermark_cli_params.rs`, trap 10's widened-print lesson.
- `SPEC-133` (shipped) — DEC-104's fit and coverage refusals; `mark_size` as the
  one-helper-for-check-and-composite precedent `faded_alpha16` follows.
- `SPEC-131`, `SPEC-128` (shipped) — the recipe schema and `from_params`.

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
- The 8-bit composite branch's opacity loop (`:1833-1836`) and `src/text/mod.rs`: read, not changed.
- The font-read error wording (`could not read image input` for a missing `--font`): existing.

## Notes for the Implementer

- **Commit order matters.** Commit 1: the extended `accepted_cases()`, the `Base` enum and the
  blessed golden, with no `src/` change, blessed on that tree. Then the failing tests, then the
  fix. Verify checks the golden commit has no `src/` diff against `main`.
- **Write `faded_alpha16` first and switch the 16-bit loop to it** before writing the predicate;
  FT-6 must stay green at that point (it is a pure refactor).
- Messages: copy them from Call 4 character for character; FT-1 and FT-8 compare whole lines.
- `faint16`: write it with `image::ImageBuffer::<Rgba<u16>, _>` and save as PNG; check in the test
  that it decodes back as 16-bit with alpha 4 (a control that the fixture is what the row needs).
- `black16`: an `Rgb<u16>` all-zero 400×300 PNG (a 60×30 mark on a smaller base trips the 5 % cap). Every boundary row asserts output ≠ input on it,
  which is how "the mark kept alpha" is observed.
- wasm: copy `transform_refuses_a_mark_that_does_not_fit` (`tests/wasm_roundtrip.rs:307`); format
  the error with `format!("{:?}", JsValue::from(err))`.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the pipe's exit code.
- Push a WIP commit once it compiles, before the full matrix.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-143-refuse-invisible-watermark`
- **PR (if applicable):** draft, opened after this record (number in the build report)
- **All acceptance criteria met?** yes (AC-1 to AC-14; two wording notes under Deviations)
- **New decisions emitted:** none beyond DEC-109 (from design)
- **Commits (in order, all `-s`):**
  - `2ccbdaf` golden: 12 accepted cases and the `Base` enum, blessed on unmodified `src/`
    (`git show 2ccbdaf -- src/` is empty)
  - `a028d9d` integration tests FT-1 to FT-12, against unmodified `src/`
  - `8a91c83` refactor: `faded_alpha16`, the 16-bit opacity loop calls it (FT-6 and
    `opacity_does_not_change_coverage` green at this commit)
  - `fe5d0de` the fix, with unit tests FT-13 to FT-16 (pushed as the WIP once it compiled)
  - `9bb4d6e` docs, CHANGELOG and the DEC-106 pointer line
- **Files this diff touches** (`git diff --name-only origin/main...HEAD`):
  - `src/operation/mod.rs` — `faded_alpha16`, `max_alpha16`, `InvisibleMark`, the `max_alpha`
    field, `invisible_mark()`, the `from_params` call and recipe messages, unit tests FT-13 to FT-16
  - `src/cli/ops.rs` — the opacity-0 and colour-alpha-00 flag checks, the post-build call, the CLI
    message `match`
  - `src/cli/mod.rs` — `--opacity` help line, `--text` help
  - `tests/watermark_cli_params.rs` — `Base` enum, 12-case golden, FT-1 to FT-5, FT-7
  - `tests/fixtures/watermark/cli_params.golden` — 9 new lines
  - `tests/recipe_watermark.rs` — FT-8 to FT-10
  - `tests/build_watermark_cache.rs` — FT-11
  - `tests/wasm_roundtrip.rs` — FT-12
  - `CHANGELOG.md` — Unreleased, Breaking entry (Call 7)
  - `docs/api-contract.md`, `docs/cli-reference.md`, `docs/data-model.md`,
    `recipes/README.md`, `skills/crustyimg/SKILL.md` — the refusal, the opacity range, the
    faint-mark sentence; `recipes/README.md` no longer says `opacity = 0` is drawn
  - `decisions/DEC-106-….md` — one Consequences line pointing at DEC-109
  - this spec file
- **RED baseline on `main` (commits 1–2, unmodified `src/`):**
  FT-1, FT-2, FT-3 RED (first refused row exits 0, nothing refused); FT-4 rows exit 3, 3, 3, 3, 3, 0
  and the pinned `--font` row 3 (probed on the `main` binary); FT-7 RED (no `--opacity` help
  line); FT-8 RED (exit 0); FT-9 RED (exit 3). FT-11 and FT-12 as first written were red on
  `main` for the wrong reason: the 5 % coverage cap (FT-12's 64×48 base threw "the mark would
  cover 9.05 %"; FT-11's 64×64 inputs skipped the mark at "6.79 %"), not because a mark drew
  nothing. After the punch list enlarged the bases (FT-11 256×256, FT-12 256×192, mark under
  5 %), on `main`'s `src/`: FT-11 RED at exit 0 (`built 1 target, 2 outputs (0 cached, 2
  rebuilt)`, no coverage message) and FT-12 RED because `transform` returned `Ok` (the wasm
  runner's `expect_err` received the PNG bytes), no coverage message in either. FT-5, FT-6, FT-10 GREEN (the guards). FT-13 to FT-16 call the new API,
  so they landed with the fix.
- **Mutations** (each: patch applied, `git diff` non-empty, fresh `CARGO_TARGET_DIR`, restored,
  tree clean after). Names are the tests that went red:
  - M1 (no call in `from_params`): FT-8, FT-9, FT-11, FT-16; wasm FT-12 red. CLI rows green.
  - M2 (no post-build call): FT-1, FT-2, FT-3, FT-4. Early-check rows and recipe rows green.
  - M3 (no CLI flag checks): FT-4 only.
  - M4a (no post-opacity step): FT-1, FT-2, FT-8, FT-13, FT-14, FT-15, FT-16. FT-3, FT-4 green.
  - M5 (per-image check in `apply`): FT-1, FT-2, FT-3, FT-4, FT-8, FT-9, FT-11, FT-16, **and
    FT-15** (see Deviations); wasm FT-12 red.
  - M6 (8-bit precision): FT-2, FT-5, FT-6 (red at `faint_opacity`, which now exits 2),
    FT-10, FT-14, FT-15.
  - M7 (`65535 × op` shortcut): FT-1, FT-2, FT-14, FT-15.
  - M8 (floor, `< 1.0`): FT-2, FT-6, FT-10, FT-14, FT-15.
  - M9a (classifier order): FT-1, FT-4, FT-8, FT-13; wasm FT-12 red.
  - M9b (blank text as a flag check, CLI and recipe): FT-4, FT-5, FT-10.
  - M10 (`f64` in the message): FT-1, FT-2, FT-8, FT-16.
  - Golden mutation (a), `.floor()` in `faded_alpha16`: FT-6 (at `faint_opacity`: the predicate now
    refuses it first), FT-2, FT-10, FT-14. Golden mutation (b) is M6: FT-6 as above.
- **Matrix** (each leg a fresh target dir, deleted after; fmt `--check` clean, clippy `-D warnings`
  clean on every native leg): default 1129 passed / 0 failed; `--no-default-features` 1067 / 0;
  `--features webp-lossy` 1134 / 0; `just wasm-test` 47 passed / 0 failed. `decisions-audit.sh`
  exits 0 (existing advisory scope-overlap warnings only).
- **Deviations from spec:**
  - AC-3 says `build` prints "one `error:` line" with `target #0` and the step text. It prints
    two lines (`error: target #0 (recipe r.toml, out dist)` then `error: recipe step 0
    ('watermark'): …`). FT-11 asserts those two lines. The behaviour is as designed; only the
    AC's count was loose (AC-3 now says two lines).
  - M5's expected-green list said FT-13 to FT-15. FT-15 calls `apply` on every row, and under M5
    `apply` errors for a refused mark, so it goes red. FT-13 and FT-14 stay green.
  - M4a also turns FT-13 red (it asserts a `TooFaint` row); the table did not list it.
  - (The mutation table now carries both corrections.)
  - Golden mutations: under both, `run_case` fails on its exit-0 assertion at `faint_opacity`
    before any hash is compared, so FT-6 is red for "refused", not for a different hash.
  - `src/lib.rs` needed no change: `Watermark` is not re-exported there, and `InvisibleMark` is
    public at `operation::`.
- **Follow-up work identified:**
  - The library-caller gap (a caller of `new_image`/`new_text` that never calls
    `invisible_mark()`) stays the STAGE-054 entry the frame filed, for the next minor.

### Punch list (verify)

Docs and tests only; `git diff --stat` touches nothing in `src/`.

**The four fixes**
1. Docs: `docs/cli-reference.md` and `skills/crustyimg/SKILL.md` no longer say a mark "is drawn as
   asked, even one too small or faint to see", and now say that only `--opacity 0` and a colour
   alpha `00` exit 2 before any file is read; blank text, a too-small size and a transparent image
   are refused after the `--font`/`--image` read and before any input (so `--text " " --font
   /no/f.ttf` exits 3). DEC-109 and `docs/api-contract.md` say `--watch` reports the error lines
   and keeps watching (it does not exit 1). The long line in `cli-reference.md` is wrapped.
2. FT-8 gained `color = "ffffff01"` at `opacity = 0.0019` and the 16-bit alpha-4 `faint16` image
   at `opacity = 0.1` (11 rows).
3. FT-6 gained `image_opacity_0_6_16bit` (13 cases, 13 golden lines).
4. FT-11 now uses 256×256 inputs and FT-12 a 256×192 base, so the mark is under the 5 % cap.

**Prove it can fail** (each: `git diff` non-empty, fresh `CARGO_TARGET_DIR`, restored, green after)
- Item 2, `max_alpha` set to `65535` in `from_params`: `recipe_watermark` goes red at
  `"text = \"hi\"\ncolor = \"ffffff01\"\nopacity = 0.0019": left: Some(0) right: Some(1)`. The
  loop stops at the first failure, so the `faint16 0.1` row was run alone (first new row removed
  temporarily) and went red the same way (`left: Some(0) right: Some(1)`). Verify found this mutation
  green before the change (not re-run here).
- Item 3, `self.opacity * 0.999` in the 16-bit opacity loop (not in `faded_alpha16`): FT-6 red at
  `tests/watermark_cli_params.rs:280`, `assertion \`left == right\` failed` comparing the hash
  lists; the only line that differs is `image_opacity_0_6_16bit` (a hash mismatch, not an exit
  code).
- Item 4, enlarged FT-11 and FT-12 against `main`'s `src/`: FT-11 fails at `assertion \`left ==
  right\` failed: stderr: target #0: 2 inputs × r.toml → dist / built 1 target, 2 outputs (0
  cached, 2 rebuilt)`, `left: Some(0) right: Some(1)`. FT-12 fails at `tests/wasm_roundtrip.rs:353`,
  `a mark that draws nothing is refused: [137, 80, 78, 71, …` (the `Ok` PNG bytes). No coverage
  message in either. On the head with M1 (the `invisible_mark()` call in `from_params` dropped):
  FT-11 red at `tests/build_watermark_cache.rs:394` and wasm FT-12 red at line 353.
- The 16-bit 0.6 golden line (`39edaae5…c799`) blessed on the head passes unchanged on `origin/main`
  (`accepted_marks_are_byte_identical_to_main`, 13 goldens, run not blessed).

**Matrix** (fresh target dir per leg, run sequentially, deleted after; `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` clean): default 1129 passed / 0 failed;
`--no-default-features` 1067 / 0; `just wasm-test` 47 / 0.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Only the AC-3 "one error line" count (`build` prints two) and what "FT-13 to FT-15 green"
   means under M5 when FT-15 applies the op. Everything else, messages and boundaries included,
   could be copied.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The golden-first order and the `black16` detector were both in the design and both held.

3. **If you did this task again, what would you do differently?**
   — Nothing design-level; no design call turned out wrong. I would write a per-case golden diff
   helper, since the golden test reports one string and a mutation hides which case flipped.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The code was right on every path, and the problems were all in how the evidence was
   recorded. (a) The build recorded its RED baseline by exit code, so FT-11 and FT-12 passed
   review while failing on `main` for the 5 % coverage cap, not because the mark drew nothing;
   it even wrote FT-12 up as "returned Ok", which was false. A RED row is only RED for its reason
   if its failure message says so, so the baseline should quote the message and a coverage-cap
   message should count as a wrong-reason red. (b) The golden's mutations went red only through
   a refusal (`run_case` failed its exit-0 assertion at `faint_opacity`) and never by hash, so
   `opacity × 0.999` in the 16-bit loop stayed green until verify added a 16-bit 0.6 case. A
   golden should have one mutation that turns it red by hash, and the build should say which.
   (c) Two of verify's four items were things the design could have listed: a recipe-side
   mutation (`max_alpha = 65535`) that no FT-8 row could see because every too-faint row used an
   opaque logo, and two doc sentences that the changed behaviour made false.
   All three of the build's mutation-table deviations were spec errors, so the table needed a
   second pass at design.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied. HANDOFF traps 1–12 do not cover either failure. Trap 10 (a derived
   value needs an input where it differs from the naive one) is the nearest to the 16-bit
   mid-opacity gap, but it is about derived values, not about what a golden proves. Trap 11 is
   about skips, and trap 12 is about where a mutation is applied and which tests go red. Propose
   one new trap, 13: **"A RED baseline row is RED for its reason only if its failure message says
   so, and a golden guards the bytes only if one mutation turns it red by hash."** Its rule: the
   build's baseline quotes each RED row's failure message and the prompt names the message that
   would be a wrong reason (here, the coverage cap); and the prompt for a golden asks for one
   mutation whose red is a hash mismatch rather than a refusal or an exit code. No constraint or
   DEC change: DEC-109 and DEC-106 stand as designed. The two stale doc sentences were fixed by
   the punch list.

3. **Is there a follow-up spec I should write now before I forget?**
   — One small backlog entry, filed in STAGE-054 and grouping verify's leftovers: `faint_opacity`
   and `faint_opacity_boundary` hash identically in the golden (both composite to the same bytes,
   `6f10ba3e…`), so the boundary row does not guard the boundary by hash; and `docs/api-contract.md`
   has one unwrapped line about `--watch` (from verify's notes; I did not locate it). The two
   existing entries stay as they were: the library-constructor gap (waits for 0.9) and "no CI job
   runs the wasm round-trip tests" (filed by the orchestrator, 2764a24; not filed again).
