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

- [ ] (not yet written) — [S] ⚡ **`watermark --text` overruns the right edge, and it is NOT
  simply a function of text width.** Driven 2026-09-03 on `main` (0.7.1 + SPEC-126), one source
  photo, `watermark --text "© crustyimg"` with no other flags:

  ⚠ **CORRECTED 2026-09-04 by geometric measurement — the first version of this entry was WRONG.**
  It claimed the glyph was *cut* at 448×448. It is not. The changed-pixel bounding box at 448 is
  **168×31 at (279,417)**, so the run ends at x=446 on a 0..447 canvas: **flush, with a 1 px
  margin, but not clipped.** I read a glyph sitting hard against the edge, in a 448 px image
  displayed small, as a glyph that had been cut — a visual read written up with the confidence of
  a measurement. That is the exact failure SPEC-126's ship reflection named hours earlier.

  **What is actually true, measured:**

  | fact | evidence |
  |---|---|
  | The run is anchored **flush bottom-right, ~1 px margin** | bbox `TOUCHES bottom` at **every** canvas ≥128 px |
  | The glyph run is **168×31 px**, constant | identical bbox from 192 px to 1200 px canvas |
  | It **is** clipped when the text is wider than the canvas | 168 px bbox at x=0 touching left+right for canvases ≤168 px; first clean fit at **172 px** |
  | The boundary is **string-dependent**, not a canvas size | on one 256 px canvas: `"©"` → 24 px bbox; `"© crustyimg"` → 168 px, fits; `"© 2026 crustyimg all rights reserved"` → 248 px, **already clipped** |

  So "does it fit" is a real predicate and a **separate** one from coverage — but it cannot be
  expressed as a canvas-size threshold, because it is a function of the RENDERED TEXT EXTENT.
  The rule needs that extent, which the text layer already computes.

  ⚠ **This is a SECOND question from the coverage rule, and the damage rule must answer both.**
  "Is the watermark proportionate to the image" and "does the watermark fit inside the image" are
  different predicates, and a size-only rule answers only the first. A watermark that is correctly
  sized and still clipped is just as broken to the user.

- [ ] (not yet written) — [S] **A literal extension in `--name-template` does not pin the
  output format — and single-input `apply` now makes that visible.** Surfaced by SPEC-126's
  verify, 2026-08-23, driven on both binaries with a JPEG source and a plain pixel recipe:

  | `--name-template`, 1 input | before SPEC-126 | after |
  |---|---|---|
  | `{stem}_w.jpg` | **PNG bytes inside `in_w.jpg`** — mislabelled | JPEG in `in_w.jpg` ✅ |
  | `{stem}_w.png` | PNG in `in_w.png` | **JPEG in `in_w.png`** |

  The old single-input path ignored the template entirely and always wrote PNG, so it was
  already mislabelling in one direction. SPEC-126 fixes the `.jpg` row outright and makes the
  `.png` row agree with multi-input `apply`, `resize` and `build` — the documented rule for a
  plain pixel recipe (`docs/api-contract.md`, DEC-087): a name template's literal extension
  names the FILE, it does not pin the FORMAT. **Nothing regressed.** But the rule is now
  reachable at an arity where it was previously masked, and a user who writes `{stem}_w.png`
  expecting PNG will get their source format instead.

  ⚠ **This is the same family as the open `-o`-extension pin ruling**, which moved to
  **PROJ-013 / STAGE-037** when PROJ-010 shipped — where a *recognized `-o` extension*
  does pin and skip the decision. So the surface currently answers "does a literal extension
  pin?" **differently for `-o` than for `--name-template`**, and only one of the two is
  documented as deliberate. Worth ruling on together rather than separately; the maintainer
  decision is warn / honour-the-template / document-and-keep.

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

  **Two design inputs measured 2026-09-03, before any spec is written:**

  ⚡ **THE COVERAGE CURVE IS NOW MEASURED — and ~25 % is far too permissive.** Driven 2026-09-04
  with a geometric measure (decode both to RGBA8, count differing pixels; harness + controls in
  the session record). **The threshold is the ruling this stage still needs, and here is the data
  it should be ruled on:**

  | canvas | coverage | canvas | coverage |
  |---:|---:|---:|---:|
  | 24 px | **45.31 %** | 256 px | 2.51 % |
  | 32 px | **25.20 %** | 384 px | 1.12 % |
  | 48 px | 17.23 % | 448 px | 0.82 % |
  | 64 px | 13.11 % | 640 px | 0.40 % |
  | 96 px | 9.25 % | 800 px | 0.26 % |
  | 128 px | 7.18 % | 1200 px | 0.11 % |
  | 192 px | 4.47 % | | |

  Above the clipping boundary the glyph run is a **constant 1647 px**, so coverage is exactly
  `glyph_area / canvas_area` — no second render needed once the extent is known.

  ⚠ **The backlog's ~25 % lands at a 32 px canvas.** A 25 % rule would reject **only** 24 px and
  32 px, and pass 48 px (17 %), 64 px (13 %) and 128 px (7 %) — all of which are visibly ruined in
  the contact sheet. **~25 % is not a validated threshold; it is the value that happened to
  separate the two most extreme rows of the original three-row table.** The visual read puts the
  acceptable boundary nearer **2–5 %** (192–256 px). That gap is the ruling.

  📌 **The old reference points re-derived close but all high:** 24 px 47 %→**45.3 %**,
  64 px 16 %→**13.1 %**, 800 px 0.32 %→**0.26 %**. Right ballpark, wrong for building a rule on.

  ⚡ **Consider making CLIPPING the primary damage rule instead of coverage.** It needs no
  threshold at all — the text either fits the canvas or it does not — it is objective, and it
  tracks the visual read better than any percentage: everything below ~170 px for the default
  string is clipped, which is exactly the band that looks broken. Coverage would then be the
  secondary rule for the sizes where the text fits but still dominates.

  ⚡ **SSIMULACRA2 is the WRONG instrument for this rule.** Driven across the same six sizes, the
  score is **not monotonic in the damage**: 24 px scores **+50.7** while 64 px scores **−82.0**,
  though the 24 px output is visibly the more destroyed. The metric is perceptual and is not built
  for canvases this small — at 24 px most of the glyph run falls *outside* the frame, so there is
  less **measured** difference exactly where there is more actual damage. **The coverage rule needs
  a GEOMETRIC measure — the fraction of pixels the glyphs cover — not a perceptual one.** Anything
  built on `diff`/SSIM2 will be wrong at precisely the sizes the rule exists to protect.

  ⚠ **The `--size 200` on 24 px row did NOT reproduce as a silent no-op.** Same flags on
  `tests/fixtures/classify/color_photo_fuji.png` downscaled to 24 px: the output differs
  perceptibly from the input (SSIM2 **74.96**), not 0.00 %. Either the no-op depends on the
  specific text or gravity, or it needs a different input to surface. **The table above is a set of
  reference points to re-drive, not measurements to build against** — which is what this entry
  already said, now confirmed the hard way.

  📌 Visual evidence for all of the above, one image per cell at native size and 4×:
  see the contact sheet generated 2026-09-03 (artifact, linked from the session; regenerate with
  the commands in this entry rather than trusting the link to persist).

- [ ] (not yet written) — [S] ⚡ **Design the registry seam for TWO parameter-rich ops, not one —
  the LUT op is the known second customer.** ⚠ **This item is a design constraint on the watermark
  work above, NOT a commitment to build a LUT op in this stage.**

  **A `.cube` LUT op is already a decided "take"** — `docs/feature-set-triage-2026-08.md` §2, with
  §3.1 recording the maintainer's **2026-08-10 reversal** of the "take the crate" advice once the
  crate data landed: `lut-rs` **does not exist**; `lut-cube`'s licence is **non-standard** on
  crates.io (a `cargo deny` question under DEC-018 before it is anything else); `wagahai_lut` is
  **v0.1.0, one release, 544 downloads**. **Ruling: build the `.cube` reader in-house**, sized at
  **~100 lines to parse + ~50 for trilinear interpolation** — about a fifth of the TIFF-IFD writer
  this repo already maintains, and consistent with its in-housing precedent (`little_exif` →
  `src/metadata/tiff.rs`, 718 lines; `ab_glyph` → `skrifa` + `zeno`, both confirmed absent from
  `Cargo.lock`).

  **Why it belongs in this stage even though it is not built here:** `watermark` is about to be the
  **first parameter-rich op the registry has ever taken**. A LUT is the second, and it is already
  decided. **A seam widened for one op is a seam widened twice.** Design it knowing a second
  customer exists — parameter shape, validation, and how a typed params struct generalises — rather
  than discovering it after.

  📌 ⚠ **The LUT op itself is UNSCHEDULED and belongs to no project.** It was decided 2026-08-10 and
  has been invisible since: `docs/feature-set-triage-2026-08.md` is **read by no command**, so
  `just backlog` and `just status` have never seen it. This entry exists so the decision is at
  least reachable from tooling. **Whether the LUT op gets a home is a separate maintainer call** —
  and worth asking what else is in that triage doc, since a decided feature was lost in it once.

- [ ] (not yet written) — [M] **`format` and `quality` on `Recipe`**, plus typed per-operation
  parameter structs. ⛔ **Depends on STAGE-049** — the two paths must agree on what a format means
  before the schema names one.

**Count:** 0 shipped / 0 active / **4 pending** — re-derive with a grep you just ran.

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
