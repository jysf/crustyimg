---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-109
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-30
# Refines DEC-104 (decided from the mark) and DEC-106 (flag checks before file reads);
# supersedes neither.
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/mod.rs
  - src/cli/ops.rs
  - src/cli/mod.rs
  - docs/api-contract.md
  - docs/data-model.md
  - docs/cli-reference.md
  - recipes/README.md
  - skills/crustyimg/SKILL.md

tags:
  - watermark
  - cli
  - recipe
  - validation
  - untrusted-input-hardening

references:
  - dec-104
  - dec-106
  - dec-102
  - dec-031
  - dec-015
---

# DEC-109: A watermark that would draw nothing is refused, decided from the mark at 16-bit precision

## Decision

**The rule.** A watermark whose mark has no pixel with alpha > 0 once its opacity is applied is
refused before any input image is read: exit 2 on the CLI, exit 1 for a recipe (`apply`, `build`,
`--watch`), and a thrown error from wasm `transform`. Nothing is written. It is decided from the
mark alone, like DEC-104's coverage count, never from the image it would be drawn on.

**Precision: 16 bits.** "Alpha after opacity" is `round(a16 × opacity)` on the overlay's 16-bit
view, the expression the 16-bit composite branch uses. One private function, `faded_alpha16`,
serves both the check and that branch, so they cannot disagree. The largest alpha decides it, since
rounding is monotone. For an opaque mark the threshold is `opacity < 0.5/65535` (≈ 7.63e-6); for
a mark whose largest alpha8 is 1 it is `opacity < 0.5/257` (≈ 1.95e-3).

The 8-bit composite branch rounds at 8 bits, so it draws zero alpha for
`0.5/(257·a8) ≤ opacity < 0.5/a8` (an opaque mark at `0.001` draws all 1800 px of a 16-bit image
and nothing on an 8-bit one). The reverse never happens: a mark refused at 16 bits always draws
nothing at 8 bits. The 8-bit gap is accepted. The branch is chosen by the image, not the mark, and
an 8-bit image already ignores alpha 1 on a dark base. That makes it the "faint mark that changes
nothing on one particular image", which is drawn as asked.

**Six causes, checked in this order** (`Watermark::invisible_mark() -> Option<InvisibleMark>`):

1. `OpacityZero`: opacity is 0 (including `-0` and a recipe value below f32's smallest positive);
2. `TransparentColor`: a text mark whose colour has alpha 0;
3. a mark with no visible pixel at all (`visible == 0`, DEC-104's count): `BlankText` when the
   text is all whitespace, else `TextTooSmall`; `TransparentImage` for an image mark;
4. `TooFaint`: the mark has visible pixels, but the opacity rounds every alpha to 0.

**Two call sites.** The end of `Watermark::from_params` (every recipe surface), and
`run_watermark` (the CLI builds the op directly, DEC-031). The public constructors `new_image` and
`new_text` stay infallible, and `Refusal` gains no variant: the API change is additive (a new
`#[non_exhaustive]` enum and a new method).

**CLI order (refines DEC-106).** Opacity 0 and colour alpha 00 are true of the mark whatever the
font or overlay, so they join DEC-106's flag checks, before any file is read. Blank text, a
too-small size and a transparent image need the rendered or loaded overlay. They are checked after
the `--font`/`--image` read and before any input is read. Blank text is **not** a flag check: in
the bundled Go font, tab, U+2000–U+200A and U+3000 are whitespace yet draw the `.notdef` box.

## Context

Before this, all five ways to ask for an invisible mark (`opacity 0`, blank text, a tiny `size`,
colour alpha 00, a transparent image) exited 0 and wrote the image unmarked, on every surface. The
maintainer ruled on 2026-10-30 that opacity 0 is refused like the rest: to turn a watermark off,
remove the step. They also ruled that the rule is decided from the mark, and that a positive opacity
too small to keep any alpha is refused too. The frame measured that "would change no pixel of this
image" depends on the image: white on white never changes; opacity 0.005 changes nothing on an
8-bit image and every mark pixel on a 16-bit one.

## Alternatives Considered

- **Count changed pixels on the image.** Rejected for the same reason DEC-104 rejected it: it
  refuses a mark on one image and passes it on another, and it can only be known per image, after
  the input is read.
- **Count `visible` after opacity.** Rejected: `visible` is DEC-104's coverage figure, documented as
  "before opacity", and moving it would change which marks the 5 % cap refuses.
- **8-bit precision (the stricter branch).** Rejected: it refuses `--opacity 0.001`, which draws
  every mark pixel of a 16-bit image, and it still would not make 8-bit output change.
- **`round(65535 × opacity) == 0`.** Rejected: it ignores the mark's largest alpha. It passes
  `--color ffffff01 --opacity 0.0019`, which draws nothing.
- **Amend DEC-104.** Rejected: DEC-104's refusals are per image (a batch skips the image). This one
  is per invocation (exit 2, nothing written).
- **Make `opacity = 0` a table bound (`min_inclusive: false`).** Rejected: it rewords today's
  `--opacity 1.5` message, and the table's phrase cannot say "remove the step".
- **Fallible constructors or a new `Refusal` variant.** Rejected for a patch release (semver).
  The library-caller gap is filed for the next minor.

## Consequences

- Scripts that used `opacity 0` as an off switch now fail (exit 2 or 1), so the CHANGELOG lists it
  under **Breaking**.
- A library caller that builds a `Watermark` directly and never calls `invisible_mark()` can still
  composite nothing with `Ok`. This is filed as a STAGE-054 entry for the next minor.
- A faint mark that keeps alpha is drawn even where it changes nothing (white on white, a low
  opacity on 8-bit output, text at size 0.18). The docs say so.
- `visible`, the 5 % cap, `WATERMARK_PARAMS`, `params()`, the lockfile and cache keys are
  unchanged.

## Validation

Right if each cause is refused with its own line on the CLI, `apply`, `build` and wasm. The
boundary pairs must hold (opaque `0.0000076` refused / `0.0000077` drawn; alpha8-1 `0.0019` /
`0.002`; text size `0.17` / `0.18`). `invisible_mark()` must agree with the 16-bit composite on a
black 16-bit image. Every accepted mark must hash identically to goldens captured on the pre-change
`main`.

## Revisit if

- The 8-bit composite branch is changed to share `faded_alpha16` (the gap closes; re-prove bytes).
- `new_image`/`new_text` become fallible at a minor release (move the check into them).
- A `scale` resize is observed to leave an image mark with no alpha (that is per image, and not
  covered here).

## References

- Related specs: SPEC-143 (this rule), SPEC-135 (DEC-106's flag checks), SPEC-133 (DEC-104).
- Related decisions: DEC-104, DEC-106, DEC-102, DEC-031, DEC-015, DEC-045.
- Constraints: `untrusted-input-hardening`, `test-before-implementation`.
