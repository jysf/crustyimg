---
stage:
  id: STAGE-050
  status: active
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

- [x] **SPEC-133** (merged 2026-10-27, PR #206 — history of the overrun item; shipped entry below, under "`watermark` stops silently doing nothing") — [S] ⚡ **`watermark --text` overruns the right edge, and it is NOT
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

- [x] **SPEC-134** (merged 2026-10-26, PR #202) — [S] **A literal extension in `--name-template` does not pin the
  output format — and single-input `apply` now makes that visible.** Surfaced by SPEC-126's
  verify, 2026-08-23, driven on both binaries with a JPEG source and a plain pixel recipe:

  | `--name-template`, 1 input | before SPEC-126 | after |
  |---|---|---|
  | `{stem}_w.jpg` | **PNG bytes inside `in_w.jpg`** — mislabelled | JPEG in `in_w.jpg` ✅ |
  | `{stem}_w.png` | PNG in `in_w.png` | **JPEG in `in_w.png`** |

  ✅ **RULED 2026-10-24 (maintainer): honour the template.** A recognised extension in
  `--name-template` pins the output format, exactly as a recognised `-o` extension already does —
  a file named `.png` contains a PNG. Rationale: effectively no users yet, so make it right going
  forward rather than carry the trap. ⚠ Byte-changing for a template whose extension differs from
  the source format; it rides 0.8.0's single lockfile migration. ⚠ This also settles the
  *direction* of the PROJ-013 / STAGE-037 `-o` item (extensions pin, deliberately) — see the
  cross-reference recorded there; that item's undocumented quality change remains open.
  **For the spec:** this reverses the rule documented in `docs/api-contract.md` ("a name
  template's literal extension names the FILE, it does not pin the FORMAT"). ⚠ **Corrected
  2026-10-25 (SPEC-134 design): no DEC held that sentence — DEC-087 says the opposite** (a `build`
  terminal-`optimize` target's literal template IS its format pin). DEC-105 **extends** DEC-087 to
  every verb; it does not reverse or supersede it. Also decide what a template extension does on `web`/`optimize` batches, where
  a `-o` extension pins *and skips the auto-decision*: consistency says the same, but that turns
  `web --out-dir d/ --name-template '{stem}.webp'` from auto-decide into a pinned convert.

  The old single-input path ignored the template entirely and always wrote PNG, so it was
  already mislabelling in one direction. SPEC-126 fixes the `.jpg` row outright and makes the
  `.png` row agree with multi-input `apply`, `resize` and `build` — the documented rule for a
  plain pixel recipe (`docs/api-contract.md` only — mis-attributed here to DEC-087 until
  2026-10-25): a name template's literal extension
  names the FILE, it does not pin the FORMAT. **Nothing regressed.** But the rule is now
  reachable at an arity where it was previously masked, and a user who writes `{stem}_w.png`
  expecting PNG will get their source format instead.

  ⚠ **This is the same family as the open `-o`-extension pin ruling**, which moved to
  **PROJ-013 / STAGE-037** when PROJ-010 shipped — where a *recognized `-o` extension*
  does pin and skip the decision. So the surface currently answers "does a literal extension
  pin?" **differently for `-o` than for `--name-template`**, and only one of the two is
  documented as deliberate. Worth ruling on together rather than separately; the maintainer
  decision is warn / honour-the-template / document-and-keep.

- [x] **SPEC-128** (shipped on 2026-09-06) — **recipes can express watermark.**
  ⚡ **Retitled from "`watermark` becomes a registry operation" — that description was wrong.**
  `Watermark` is already an `Operation`; it is *deliberately* unregistered (DEC-031), because the
  registry constructor cannot load the overlay file and `src/operation/**` must stay
  filesystem-free for wasm32. The real work is widening the seam for ops whose params name a file
  — with image watermark, text watermark and the `.cube` LUT as its three customers.
  📌 *Original wording, kept for the record:* "**`watermark` becomes a registry operation.** Its ten parameters
  (image/text, font, size, colour, gravity, opacity, scale, margin, tile) are richer than any op
  the registry has taken, which is the real work — the seam itself is documented as "the single
  seam new operations register at", but no parameter-rich op has ever used it.

- [x] **SPEC-129** (merged 2026-09-07, PR #190) — ⚡ **`build`'s cache key did not hash a watermark
  asset's own file CONTENT, only the recipe's `to_toml()` (the path).** Found during SPEC-128's
  build (2026-09-05), reproduced at SPEC-128's verify with a verified control: editing `logo.png`
  in place (same path, different bytes) made `build` report *"1 cached, 0 rebuilt"* and emit
  byte-identical pre-edit output. `apply` is unaffected (no cache). No SPEC-128 AC named this (it
  is about registering the op, not the cache), so it was filed. The mechanism was one function:
  `target_recipe_hash` hashed `to_toml()` + `plan`, and SPEC-128's `set_resolved_bytes` side
  channel is deliberately invisible to `Serialize` (DEC-100's highest-consequence guard).
  **Shipped:** SHA-256 of the resolved bytes (matches source-input precedent, DEC-058); folded in
  once per target (`target_recipe_hash`, dominated by decode+encode — AC-7 pins it); watermark-free
  recipes hash identically to `main` on every format plan (AC-4 pin, so no existing cache entry or
  lockfile line is invalidated). Amends DEC-058 clause 4 via DEC-101
  (`CACHE_SCHEMA_VERSION` deliberately unchanged; the departure from DEC-058's suggested remedy is
  written down).
  ⚠ **Merged before its verify cycle ran** (the maintainer merged on green CI, believing it was
  ready); verify ran retroactively against the merged commit and returned ⚠ PUNCH LIST, not a
  rejection — the over-invalidation guard (AC-4) was independently re-driven and held, including
  on the `Pinned`/`Decide` plans the named unit test doesn't cover. Two real findings from that
  punch list shipped as a same-day follow-up fix (`fix/spec-129-verify-punchlist`): AC-7's own test
  was a source-text check that verify proved wouldn't catch the exact regression it named (the
  hash call moved into a per-input loop) — replaced with a real dynamic counter driven by an
  actual multi-input build; and a doc comment on `target_recipe_hash` overstated a length-prefix
  guarantee the `Preserve` branch doesn't provide (harmless today — closed by TOML's own escaping
  rules, not by the hasher — but the comment was wrong and is now corrected). The third finding is
  filed below as its own backlog item.

- [x] **SPEC-132** (merged 2026-10-26, PR #201) — [S] ⚡ **`build --watch` never watches a recipe's asset paths.**
  **Shipped:** every recipe `image`/`font` asset's containing directory is a shallow watch root,
  the set is recomputed after every non-excluded batch, and a root that failed to register is
  retried. The build also fixed a Linux bug in released 0.7.x (inotify reported the build's own
  reads, so `--watch` rebuilt in a loop while idle: 207 builds / 45 s); verify required its
  CHANGELOG entry and a DEC-060 amendment, and tightened three weak tests. $25.29 over 9
  sessions. Original framing:
  Found by SPEC-129's (retroactive) verify cycle, 2026-09-07, driven with a real positive
  control: `WatchSet` (`src/build/watch.rs:106-114`) is built from the manifest's own directory
  (shallow), each source root (recursive), and each recipe's directory (shallow) — never the
  path a recipe step's `image`/`font` key names. With the overlay at `brand/logo.png` and the
  recipe at `recipes/r.toml`, editing `src/a.png` triggers a rebuild (control fired); editing
  `brand/logo.png` triggers **none**. SPEC-129 makes the CACHE correct once a build runs; the
  watch loop still never asks the question at all — a designer iterating on a logo under
  `--watch` sees no rebuild, silently, until they touch a source file for an unrelated reason.
  ⚠ **Configuration-dependent, so easy to miss**: if the overlay happens to sit inside a source
  root or beside its recipe, the existing shallow/recursive watch catches the edit by accident.
  📌 The fix shape is probably small — extend `WatchSet`'s construction to also resolve each
  target's recipe (the same `resolve_recipe_assets`/`registry.asset_keys` walk `build` already
  does) and add each resolved asset's path — but sizing and design are for whoever picks this up.

- [x] **SPEC-133** (merged 2026-10-27, PR #206) — [S] ⚡ **`watermark` refuses to damage an image.**
  **Shipped:** a watermark that does not fit the image (margin included) or that would change
  more than 5 % of the mark's own non-transparent pixels is refused: on one explicit input an
  error naming the sizes, exit 2; on a batch, multi-input `apply --recipe` or `build` the image
  is skipped with one stderr line and the run continues. Verify's punch list (four surviving
  mutations, three doc fixes) was fixed in the same PR; re-verify approved. $59.26 over 8
  sessions. Re-verify's leftovers are filed below. Original framing: ⚡ **`watermark` stops silently doing nothing.** A **measured
  defect**, driven on `main` (PNG filters undone before comparing):

  | input | `--size` | pixels changed |
  |---|---|---:|
  | 800×600 | default | 0.32 % — a sane watermark |
  | 64×64 | default | **16 %** |
  | 24×24 | default | **47 %** |
  | 24×24 | **200** | **0.00 % — exit 0, file written, no warning** |

  ✅ **RULED 2026-10-24 (maintainer): option D — fit test PLUS a 5 % coverage cap.**
  - **Fit (primary, no threshold):** refuse when the rendered text extent (plus margin) does not
    fit inside the image. Catches the `--size 200` 6-px no-op and clipped words, which no coverage
    cap can.
  - **Coverage (secondary):** refuse when the text would change more than **5 %** of the pixels.
    For the default string this adds only ~172–180 px canvases (text fits but dominates).
  - Refusal behaviour is the 2026-08-23 ruling below: single input → error naming the numbers;
    batch / recipe / `build` → skip, report, continue, never abort.
  - Rationale (maintainer): small images don't make sense to watermark; revisit if a real
    request appears. **Open for the spec, not the ruling:** whether 5 % is fixed or a knob
    (default fixed), and how `--tile` is treated (it covers the image by design).
  - Spec this together with the `--text` clipping item above: the fit test *is* that item's fix.

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

  ⚡ **RE-DRIVEN 2026-10-24 on a release build of `main` at `0ee7921`** (geometric diff via BMP,
  negative control = self-diff 0 px). Every coverage row above reproduced **to the hundredth**, and
  `main` is identical to 0.7.1 on every row, so the table is safe to rule on. Two things it adds:
  - **The silent no-op DOES reproduce, geometrically:** 24 px, `--size 200` changes **6 pixels
    (1.04 %)**. The SSIM2 74.96 above was the metric misreading again, not a disproof. **No
    coverage cap can catch this case** — only a fit test on the rendered text width (~1,050 px).
  - **Clipping loses words, not just edges:** `"© 2026 crustyimg all rights reserved"` on 256 px
    renders as `© 2026 crustyimg` — the whole rights clause is gone — at **3.73 %** coverage,
    which passes a 5 % cap. At 128 px the default string reads `© crustyi`.
  So coverage alone misses both kinds of failure that matter most; the fit test is necessary, and
  coverage can only be the secondary rule.

  📌 Visual evidence for all of the above, one image per cell at native size and 4×:
  see the contact sheet generated 2026-09-03 (artifact, linked from the session; regenerate with
  the commands in this entry rather than trusting the link to persist).

- [ ] (not yet written) — [S] **A cached `build` replays no per-input warnings.** Filed by
  SPEC-133's design (2026-10-25, Call 4). A cache hit writes the stored bytes and returns before the
  pipeline (`src/cli/build.rs:531-540`), so three warnings print on the build that produced an
  output and never on a cached re-run: SPEC-133's "watermark skipped, image written without it",
  DEC-085's truncated JPEG, and SPEC-119's animated input (the last two only on the Decide plan's
  rebuild path today — the Preserve/Pinned plans never print them at all, `encode_one` drops the
  flags). The output stays correct and locked; what is lost is the *report*: a second `build`
  says `(N cached, 0 rebuilt)` over images that are unmarked or truncated. Fix shape: the cache
  entry carries the output's warnings (frame field after `ext`, `src/build/cache.rs:455-503`),
  `CACHE_SCHEMA_VERSION` 1 → 2 + new magic, and `build_one` replays them on a hit. A schema bump
  is only a one-time rebuild, so this is not tied to 0.8.0's lockfile migration. SPEC-133 ships
  the limitation documented (`docs/api-contract.md`, `build`) with a test pinning it, which this
  item flips on purpose.

- [ ] (not yet written) — [S] **`watermark --min-size`: the opt-in value rule, settable in a
  recipe.** Split out of SPEC-133 by its frame (2026-10-24); the orchestrator confirms the split.
  It is the second half of the 2026-08-23 ruling above ("Value rule — opt-in `--min-size`,
  settable in a recipe … only the user knows what is worth protecting"), which is settled; it is
  not a new idea. Why separate: it adds a CLI flag on `watermark` and a recipe key on the step, so
  it couples to SPEC-131's strict param schema (land it after SPEC-131 and the key goes straight
  into the schema), and it needs its own single-input rule, which the ruling does not state (below
  the floor on one explicit input: error like the damage rule, or write unmarked?). It reuses
  SPEC-133's batch *skipped* outcome, so build it after SPEC-133. Open for its spec: what "size"
  means (shorter edge? longer edge? area?) and whether `apply --recipe` gets a CLI override.

- [x] **CLOSED by SPEC-128** (2026-09-06) — ⚡ **Design the registry seam for TWO parameter-rich ops, not one —
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

- [x] **SPEC-127** (shipped on 2026-09-05) — **`format` and `quality` on `Recipe`.** ⛔ Depended on
  STAGE-049; **unblocked, and specced.** 9 ACs, 4 settled design calls, 7 failing tests.
  ⚡ **Call 1 was settled by measurement:** the new fields require `version = "2"` rather than
  riding as optional v1 fields, because `Recipe` is `deny_unknown_fields` — an old binary handed a
  v1-plus-`format` recipe fails with a **TOML parse error at line 2**, while the same binary
  handed `version = "2"` says *"unsupported recipe version '2' (supported: 1)"*. Driven on `main`
  at `7181eed`. v1 stays valid and must keep serialising as `"1"`.
  ⚠ **Byte-changing on the surface**, so it batches into PROJ-011's single lockfile migration.

- [x] **SPEC-131** (merged 2026-10-25, PR #200; fix #203 2026-10-26) — [M] **Recipe step params are
  validated at load** (DEC-102): each op publishes a key/type/range schema through the registry,
  checked before its constructor; unknown, wrong-typed, non-finite, out-of-range and mode-unused
  keys are refused with `recipe step N ('op'): …`. The retroactive verify caught a regression (a
  tiny positive f64 passed the range check and read as 0 in f32); #203 fixed it. $33.70 over 7
  sessions. Original framing: ⚡ **Split out of the
  item above by SPEC-127's design (Call 4), not deferred by accident.** `OperationParams` is a
  `BTreeMap<String, toml::Value>` with hand-rolled `get_str`/`get_u32`/`get_f32`
  (`src/operation/mod.rs:39-71`), so **every op re-implements its own validation**. The payoff is
  not stylistic: schema errors would surface at parse time, which is exactly the failure a batch
  recipe user hits — one bad param discovered on input 400 of 500.

  **Why it is its own spec:** typing the map touches every operation in the registry, carries its
  own round-trip risk under DEC-005, and has **no dependency in either direction** on SPEC-127's
  schema change. Bundling the two turns an M into an L, and AGENTS §2 says split an L.
  (Originally arrived via external review batch 3 — one of the two items that survived checking.)

- [ ] (not yet written) — [S] **wasm `validate(recipe_toml)` export returning structured,
  per-step recipe errors.** Filed by SPEC-131's design (2026-10-24). Roadmap row #6's (recipe
  playground) prerequisite: no wasm export checks a recipe without an image today, and
  `transform` decodes its input before it builds the pipeline. Consumes SPEC-131's
  `OperationRegistry::param_specs` and its `recipe step N ('op'): …` errors; the natural home for
  "did you mean …?" key suggestions, which SPEC-131 declined.

- [ ] (not yet written) — [S] **No upper bound on a text watermark's `size`.** Filed by SPEC-131's
  design (2026-10-24): `size = 100000.0` in a recipe is accepted and took 52 s of wall time over 3
  small inputs at exit 0 (the text raster is enormous); `watermark --size` shares it. A
  resource-limit question in DEC-036's family, not a malformed-recipe one, so SPEC-131 refuses
  only `size <= 0` and non-finite values. Open: the cap, and whether it lives in the schema.

- [ ] (not yet written) — [S] ⚡ **The `watermark` CLI verb silently drops the mark for
  `--size nan` / `--size inf`, and accepts `--scale nan`.** Filed by SPEC-131's verify
  (2026-10-26), driven on release builds before and after SPEC-131 (identical): `watermark
  --size nan` exits 0 and writes output byte-equal to `--opacity 0`. The CLI twin of what SPEC-131
  fixed for recipes; the CLI builds the op with `new_image`/`new_text`, bypassing the recipe
  schema. ⚠ **Check against SPEC-133 first:** its fit test may already refuse a non-finite mark
  size; if so, this closes with SPEC-133 and needs only a test row.
  **Checked 2026-10-26 (SPEC-133 build + verify): NOT closed.** `--size nan`, `--size inf` and
  `--scale nan` still exit 0 (the mark renders tiny and fits); only `--scale inf` is now refused,
  and its message reads `the 4294967295×4294967295 px mark` — fix that wording here too.

- [ ] (not yet written) — [S] **`responsive` accepts `--name-template` and silently ignores it.**
  Filed by SPEC-134's design (2026-10-25), driven on `main`: `responsive in.jpg --widths 32
  --out-dir rp --name-template '{stem}.png'` writes `rp/in-32w.jpg`, exit 0 — the flag is a global
  arg (`src/cli/mod.rs:80-82`) that `responsive` never reads. Open: refuse it (exit 2, the flag
  means nothing here) or honour it (it would need a `{w}` token). SPEC-134 makes the template pin
  the format on every other encoding verb, so `responsive` is now the one verb where it does
  nothing.

- [x] **Absorbed into SPEC-134 as AC-22** (maintainer ruling 2026-10-25; shipped in SPEC-134) — [S] **`meta … -o x.png` writes the input's own container into `x.png`.**
  Filed by SPEC-134's design (2026-10-25), driven on `main`: `meta strip in.jpg -o E.png` → JPEG
  bytes in `E.png`, exit 0. SPEC-134 refuses the same mismatch when it comes from
  `--name-template` (exit 4, before writing; maintainer ruling 2026-10-25), but `-o` was outside
  that ruling. Recommend the same refusal for `-o`, reusing SPEC-134's `meta` pre-flight; needs a
  maintainer yes.

- [ ] (not yet written) — [S] **Encoding verbs resolve their inputs twice.** Filed by SPEC-134's
  re-verify (2026-10-26). To decide the format pin before branching, `web`, `optimize` and
  `apply --recipe` call `resolved_input_count` (`src/cli/common.rs`), which runs `source::resolve`
  on every argument; the chosen branch (`run_pixel_op` or `run_optimize_autodecide`) then resolves
  the same `&[String]` again. Same function both times and stdin is never drained by the count, so
  it is correct today; the cost is a second directory walk and two passes that could drift.
  Refactor: resolve once and pass `&[Input]` into both branches. Code quality, not a bug; a
  candidate for the maintainer's queued Rust cleanup.

- [ ] (not yet written) — [S] **SPEC-133's three test gaps that re-verify left optional.** Filed
  by SPEC-133's re-verify (2026-10-26, PR #206 at `f1a46d3`); the behaviour is correct, these are
  tests that would not catch a regression:
  (1) counting a **16-bit** overlay's visible pixels after `opacity` survives:
  `opacity_does_not_change_coverage` uses 0.001, and 65535 × 0.001 ≈ 66 ≠ 0 — use an opacity
  below ~7.6e-6 (e.g. 1e-6) so both bit depths are caught;
  (2) "K counts images, not steps" is untested on the `apply … optimize` auto-decide batch path —
  making `run_optimize_autodecide`'s `skipped_images` per-step (`src/cli/optimize.rs` ~1742)
  passes every `watermark_damage` test; add a two-step row there;
  (3) a dead assertion in `tests/recipe_strict_params.rs`: `!reason.contains("is not used by")`
  can never match a watermark message (its mode message reads "only applies to an image
  watermark"); assert against that text instead, or drop it. Each fix: apply the mutation in a
  fresh target dir, show RED, restore, GREEN.

- [ ] (not yet written) — [S] **`watermark` prints absolute paths in batch lines; `apply` prints
  relative ones.** Filed by SPEC-133's re-verify (2026-10-26): `watermark two/` labels its skip
  warnings with absolute paths, `apply --recipe … two/` with the path as given. Pre-existing
  (identical at `3dd23ff`; `src/cli/ops.rs` unchanged by SPEC-133). Pick one labelling rule for
  every per-input line and use it in `run_pixel_op` too; check the error lines as well as the
  skip lines.

**Count:** **7 shipped / 2 closed** / 0 in flight / 9 pending — re-derived 2026-10-27 with `grep '^- \[x\] \*\*SPEC-'` (8 lines, minus SPEC-133's history line at the top), `grep '^- \[x\] \*\*\(CLOSED\|Absorbed\)'` (2) and `grep -c '^- \[ \]'` (9).

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
