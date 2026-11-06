---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-113
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

created_at: 2026-11-05
# Reads DEC-003 and DEC-017; amends nothing. Answers the question SPEC-145 deferred.
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/rotate.rs
  - src/operation/flip.rs
  - src/operation/crop.rs
  - src/operation/mod.rs
  - src/sink/mod.rs
  - tests/rotate_flip.rs
  - recipes/README.md
  - docs/recipes.md
  - docs/data-model.md
  - docs/api-contract.md

tags:
  - recipe
  - geometry
  - rotate
  - flip
  - crop
  - exif
  - orientation
  - metadata

references:
  - dec-003
  - dec-017
  - dec-095
  - dec-110
  - dec-112
---

# DEC-113: A geometry step acts on the pixel buffer it receives, and writes no metadata

## Decision

**1. The rule.** A geometry step (`crop`, `rotate`, `flip`, and later `pad`, `trim` and
straighten) acts on the pixel buffer it receives and writes no metadata. That buffer is in the
**stored** orientation unless an earlier `auto-orient` step baked the EXIF Orientation tag into it.
`apply` and `build` add no orientation step of their own. A recipe meant as the photo is displayed
starts with `auto-orient`, and every geometry op's doc entry says so.

**2. What reaches the file.** The pixel lane writes no container metadata (DEC-003), so no output of
a geometry step carries an Orientation tag or `PixelXDimension`/`PixelYDimension`. No op resets,
strips or composes the tag on the way out, because no tag goes out.

**3. What stays in memory.** A geometry op keeps the `Image`'s metadata bundle
(`Image::with_pixels`), as `crop` already does. A later `auto-orient` in the same recipe therefore
still reads the source's tag and applies it to the buffer it receives. For the pure rotations
(tags 3, 6, 8) that makes `rotate` then `auto-orient` give the same pixels as `auto-orient` then
`rotate`, because rotations commute. For the mirrored tags (2, 4, 5, 7) the two orders differ, and
rule 1's "`auto-orient` first" is the order the docs give. Only `auto-orient` drops the bundle
(DEC-017).

**4. The one change that reopens the trap.** If the pixel lane ever writes the bundle's EXIF (a
keep-metadata option, a new encoder path), it **must** set Orientation to 1 and rewrite or drop
`PixelXDimension`/`PixelYDimension` on the way out. Otherwise a turned or flipped buffer goes out
with the source's tag and a viewer turns it a second time. The obligation is held in three places,
so whoever makes that change meets it:
- **the code site:** `src/sink/mod.rs`'s module doc, under `## Encoding`, says it beside
  `encode_to_bytes_with`, the one place a pixel-lane `Image` becomes bytes;
- **a test:** `tests/rotate_flip.rs`'s orientation row reads a written file with kamadak-exif, not
  crustyimg's writer, and fails on any Orientation other than absent or 1, with a message naming
  the obligation. It guards the **Orientation half only**: its input carries no pixel-dimension
  tags, so the dimensions half is held by the comment and this DEC alone;
- **this DEC**, whose `affected_scope` lists `src/sink/mod.rs`, so `scripts/decisions-audit.sh`
  surfaces it to a change there.

**5. What `crop` shipped is unchanged.** Crop's outputs carry no tag (driven below), its bundle is
kept in memory by `with_pixels`, and its docs already say to put `auto-orient` first. This DEC writes
down the rule crop already follows.

## Context

SPEC-145 (crop, DEC-112) deferred one question to STAGE-058's rotate/flip item: what an EXIF
Orientation tag and `PixelXDimension`/`PixelYDimension` mean after a geometry step on an input that
was not auto-oriented. The feared trap was a stale tag: the stored pixels turned by a step, the tag
left saying "rotate 90", and a viewer rotating twice.

SPEC-147's frame drove it on 0.8.2 (release build, scratch target dir). Inputs made with
ImageMagick, tagged with `exiftool -Orientation#=6 -ExifIFD:ExifImageWidth=60
-ExifIFD:ExifImageHeight=40 -Artist=probe` (60×40 JPEG and PNG), and a 60×40 PNG with a 10×10 red
block at the stored top-left, tagged 6. Every output read with
`exiftool -s -Orientation# -ExifImageWidth -ExifImageHeight -ImageSize -Artist`:

| run | output | Orientation | ExifImageWidth/Height | Artist |
|---|---|---|---|---|
| `apply` crop `x=0 y=0 width=0.5 height=1`, JPEG/PNG/WebP | 30×40 | absent | absent | absent |
| `apply` `auto-orient`, JPEG/PNG | 40×60 | absent | absent | absent |
| `apply` `invert`, JPEG/PNG | 60×40 | absent | absent | absent |
| `edit --invert`, JPEG/PNG | 40×60 (the verbs bake first) | absent | absent | absent |

SPEC-147's design added (same method, `85fb481`): a full-frame crop on the tag-6 PNG writes 60×40
with no tag, and a full-frame crop **then** `auto-orient` writes **40×60**, as does `invert` then
`auto-orient`. So the bundle a step keeps reaches a later `auto-orient`, and still never reaches the
file.

Three things follow:

1. **The stale-tag trap cannot happen today.** No output carries an Orientation tag, so no viewer
   rotates twice; no output carries pixel dimensions, so none is stale.
2. **The live trap is the opposite one, and it is not new.** A recipe without `auto-orient` drops
   the tag without applying it: the `invert` recipe on the tagged PNG wrote the stored 60×40 pixels
   with the red block still top-left, so a photo that displayed as a 40×60 portrait now displays
   sideways, exit 0. Every recipe without an `auto-orient` step does this, `crop` included. The
   verbs bake first; `apply` was scoped out of that fix and stays out. A warning for this case is
   filed as its own STAGE-058 item.
3. **For `rotate` it can read as "nothing happened".** ImageMagick `-rotate 90` of the stored
   pixels against crustyimg's `auto-orient` output differs in 0 pixels. On a tag-6 input that was
   not auto-oriented, `rotate angle = 90` on the stored buffer produces exactly the image the source
   already displayed as. Rule 1's docs line is the answer, not a code change.

## Alternatives Considered

- **Compose the tag** (turn the stored pixels and write a corrected tag back). Rejected: there is no
  tag-writing path on the pixel lane to compose into (DEC-003), and building one is the
  keep-metadata feature, which rule 4 governs.
- **Write Orientation = 1 on every geometry output.** Rejected for now: it has nothing to write
  into, for the same reason. It *is* the rule 4 obligation the day the lane writes EXIF.
- **Auto-orient implicitly in `apply`, or inside each geometry op.** Rejected: it gives `rotate` a
  different buffer than `crop` sees in the same recipe, changes what every existing recipe without
  `auto-orient` produces on a tagged input (silently, for recipes users already rely on), and a
  recipe that already starts with `auto-orient` gains nothing.
- **Drop the bundle in every geometry op, as `auto-orient` does.** Rejected: a later `auto-orient`
  would then silently do nothing (the image written sideways), unlike after `crop` and `invert`
  today; and `Image::from_parts` also resets the truncated-JPEG and animated-input flags that
  `with_pixels` carries.

## Consequences

- `rotate` and `flip` (SPEC-147) keep the bundle with `with_pixels` and write no tag. STAGE-059's
  straighten and STAGE-058's `pad`/`trim` follow the same rule without a new decision.
- STAGE-060's page and STAGE-058's `edit` flags emit `auto-orient` as the first step of any recipe
  meant in display coordinates, as `edit --save-recipe` already does.
- A future keep-metadata feature inherits rule 4 as a requirement, not a nice-to-have.
- `apply` on a tagged input without `auto-orient` still writes the stored orientation. Unchanged,
  documented, and the warning is its own item.

## Validation

- SPEC-147's AC-37: a tag-6 PNG through `rotate angle = 90` alone writes the stored pixels turned
  once and no Orientation tag (kamadak-exif), with controls proving the reader sees the input's tag
  and the tag reaches the pipeline.
- SPEC-147's AC-38/AC-39: `auto-orient` then `rotate 90`, and `rotate 90` then `auto-orient`, both
  give the stored pixels turned twice (rule 3).
- SPEC-147's M7 (the sink made to write the bundle's EXIF) turns AC-37 red; M7b (the bundle dropped
  in `rotate`) turns AC-39 red.
- The corpus guard (SPEC-141): no existing recipe output changes.

## References

- DEC-003 (the pixel lane carries no container metadata), DEC-017 (`auto-orient` bakes and drops),
  DEC-095 (colour type kept), DEC-110 (native and wasm agree), DEC-112 (crop's units).
- SPEC-145 (crop; deferred this question), SPEC-147 (rotate and flip; this DEC's frame and design),
  SPEC-110 (the verbs bake orientation first).
