---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-156
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-11-22

references:
  decisions:
    - DEC-003
    - DEC-029
  constraints: []
  related_specs:
    - SPEC-026
    - SPEC-114
    - SPEC-154

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: "STAGE-063's success criterion that `meta strip` leaves a JPEG rendering the same"

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
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 5186866
      tokens_breakdown:
        input: 88
        cache_creation: 122340
        cache_read: 5029296
        output: 35142
      estimated_usd: 4.16
      note: >
        Measured from the session JSONL (found by its first message, `prompts/SPEC-156-frame.md`), deduped by
        message.id (44 messages; max output_tokens per id), 0 subagent transcripts. Measured after the push,
        before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-156: meta strip keeps the Adobe APP14 colour-transform marker

## Context

`meta strip` is the container-lane verb that removes all metadata without re-encoding pixels
(DEC-003, DEC-029). `strip_all` (`src/metadata/mod.rs:166`) removes every JPEG segment from `0xE1`
to `0xEF`, plus COM. That range includes APP14 (`0xEE`, "Adobe"), whose transform byte tells a
decoder how to read the scan: YCCK or plain CMYK for four components, YCbCr or RGB for three.
Without it the decoder guesses, and for some files it guesses wrong. A user who strips an Adobe
YCCK JPEG gets a file whose scan bytes are identical but whose colours are not: the orange
`#BF4020` fixture becomes green `#5D861E`. Found by the 2026-11-10 external code review (P1),
STAGE-063 backlog entry SPEC-156, PROJ-011.

## Goal

`meta strip` keeps the Adobe APP14 segment, so a stripped JPEG decodes to the same pixels as the
original in every decoder that honours APP14, while every other APP1..APP15 segment and COM are
still removed. A regression test compares decoded pixels, not only the absence of metadata.

## Frame

**Verdict: GO.** Size **S**, confirmed. The build starts after SPEC-154 has merged.

### Premise check (driven 2026-11-22, release build of `main` at ff3c3fc, `crustyimg 0.8.2`, fresh `CARGO_TARGET_DIR`)

Fixture `tests/fixtures/lint/cmyk_16x16.jpg`: segments `APP14(transform=2) DQT DQT SOF0 DHT×4 SOS`.

```
$ crustyimg meta strip cmyk.jpg -o cmyk_s.jpg      → exit 0
cmyk_s.jpg segments: DQT DQT SOF0 DHT×4 SOS        (APP14 gone)
$ magick compare -metric AE   cmyk.jpg cmyk_s.jpg null:   → 56.4706 (0.220588), exit 1
$ magick compare -metric RMSE cmyk.jpg cmyk_s.jpg null:   → 20541.1 (0.313437), exit 1
$ crustyimg diff cmyk.jpg cmyk_s.jpg                       → ssimulacra2: 41.9374, exit 0
djpeg pixel (8,8):     original srgb(191,64,32)  stripped srgb(93,134,30)
crustyimg pixel (8,8): original srgb(191,64,32)  stripped srgb(93,134,30)
```

The review's 41.94 reproduces exactly. Two independent decoders (libjpeg-turbo's `djpeg`,
ImageMagick) and crustyimg's own decoder all agree that the render changes.

### Answers

1. **Premise and the three-component case.** Confirmed above for YCCK. Three components, driven
   on a 16×16 gradient made with `cjpeg -rgb` (Adobe transform 0):
   - Component IDs `R`,`G`,`B` (cjpeg's default): strip leaves AE 0, SSIMULACRA2 100.0000.
     Decoders fall back on the component IDs and still read RGB.
   - The same file with IDs patched to 1,2,3 (a byte edit of SOF0 and SOS, `cjpeg -rgb` output):
     strip → AE 85.5216 (0.334069), RMSE 26492.6 (0.404251), SSIMULACRA2 **−21.3010**;
     djpeg (8,8) `srgb(107,64,117)` → `srgb(92,137,0)`, crustyimg identical. Yes, it breaks too.
   - Not affected (driven): three components with transform 1 (`tests/fixtures/c2pa/signed_gps.jpg`,
     strip → AE 0, SSIMULACRA2 100), and four components with transform 0 (the CMYK fixture with
     the byte patched 2→0, strip → AE 0). YCbCr and plain CMYK are what a decoder assumes anyway.
   - So the fix should keep APP14 unconditionally rather than reason about which transforms matter.
2. **Every path that drops or copies APP14.** Sweep:
   `grep -rn 'img_parts\|Jpeg::from_bytes\|strip_all\|copy_metadata\|metadata::' src | grep -v '^src/metadata/'`
   and `grep -rn '0xEF\|0xEE\|E1u8\|0xE1\b' src tests`.
   - Drops: `strip_all` only (`src/metadata/mod.rs:172`). `clean_gps` and `set_tags` go through
     `write_exif_block`, which removes only `JPEG_MANIFEST_MARKER`; driven on the CMYK fixture,
     both keep `APP14(t=2)` and render `srgb(191,64,32)`.
   - Test sweeps over the full range: `src/metadata/mod.rs:630`
     (`strip_all_jpeg_removes_all_metadata`) and `:1712`
     (`meta_verbs_are_byte_identical_on_unsigned_input`). Both run on `jpeg_with_exif()`, an
     `image`-encoded JPEG with **no APP14**, so neither can go red on this axis today; they need the
     exception and an input that carries APP14. `tests/c2pa_manifest.rs:4,349` mention the sweep
     only in prose. `src/image/raw.rs:246` (`0xE0..=0xEF`) is a marker-plausibility check for RAW
     previews, not a strip.
   - Copies: `copy_metadata` grafts only EXIF (`set_exif`) and ICC (`set_icc_profile`); it never
     moves APP14, and DST keeps its own. Driven: `meta copy --from cmyk --to rgb123` keeps
     `APP14(t=0)` and the render; `--from rgb123 --to cmyk` keeps `APP14(t=2)` and the render.
     No pixel-lane encode carries markers (`grep set_icc\|icc_profile\|set_exif src` hits only
     `src/metadata/`); `resize --max 16` of the CMYK fixture writes `APP0 … SOS`, no APP14, and
     renders `srgb(191,64,32)`. No path copies APP14 the wrong way.
3. **Which other APPn are structural.** What decoders read to render the primary image: libjpeg
   reads APP0 (JFIF) and APP14 (Adobe); crustyimg's decode path also reads APP1 (EXIF orientation)
   and APP2 (ICC detection, `src/image/mod.rs:739`). The rest of `0xE1..=0xEF`: APP1 EXIF/XMP (metadata; orientation is render-affecting,
   see the finding below), APP2 ICC (out of scope, see below) and MPF (secondary images and gain
   maps, not the primary render), APP3–APP7 vendor data, APP8 SPIFF (a JFIF alternative that
   libjpeg ignores), APP9–APP10 vendor, APP11 JPEG XT/JUMBF/C2PA (the base image renders without
   it), APP12 Ducky/Picture Info, APP13 Photoshop IRB/IPTC (metadata), APP15 vendor.
   **Recommendation: keep APP14 only**, and only when its payload starts with `Adobe\0`, so an
   unrelated APP14 still goes. What a kept APP14 can leak, from the bytes:
   `ff ee 00 0e 41 64 6f 62 65 00 64 00 00 00 00 02` (CMYK fixture) is `Adobe`, DCTEncodeVersion
   100, flags0, flags1, transform. The signed fixture carries a 33-byte variant
   (`… 00 01 03 00 10 03 02 03 06 00 …`): the same fields plus 19 bytes of small integers and
   zeros. No text, names, dates, locations or device ids. Keep the segment verbatim; don't rewrite
   it.
4. **C2PA.** The fix doesn't touch `JPEG_MANIFEST_MARKER` (`0xEB`), `has_manifest` or
   `MANIFEST_DROPPED_WARNING`: the exception is `0xEE` only. Driven baseline on
   `signed_gps.jpg` (`APP11 APP11 APP1 APP13 APP1 APP2 APP14`): `meta strip` → exit 0, empty
   stderr, output `DQT SOF2 DHT SOS`; `meta clean --gps` and `meta set --artist X` → exit 0 with
   the manifest-dropped warning. Strip passes `warn_on_manifest_drop = false`
   (`src/cli/ops.rs:970`) and `has_manifest` reads only `0xEB`, so a surviving APP14 cannot change
   what the warning reports. After the fix, strip on that file should leave `APP14 DQT SOF2 DHT SOS`
   and still print nothing. `tests/c2pa_manifest.rs::meta_strip_removes_the_jumbf_segment_by_name`
   stays as the control.
5. **PNG.** `PNG_METADATA_CHUNKS` (`src/metadata/mod.rs:117`) strips `eXIf`, `iCCP`, `tEXt`, `zTXt`,
   `iTXt`, `tIME`, `caBX`. `gAMA`, `cHRM`, `sRGB`, `cICP`, `pHYs`, `bKGD`, `tRNS` and the critical
   chunks are kept (anything not listed survives). Render-affecting chunks removed: `iCCP` (the ICC
   question, already owned by the DEC-003 entry folded into SPEC-150) and `eXIf` (orientation, same
   finding as JPEG APP1 below). Nothing new to file for PNG beyond that finding; PNG has no
   colour-transform marker.
6. **The regression.** `image`'s JPEG decoder honours APP14: driven above, crustyimg decodes the
   original to `srgb(191,64,32)` and the stripped file to `srgb(93,134,30)`, the same as `djpeg`.
   So the test can decode with `image` (no new dev-dependency) and still go red. The oracle is
   "the same decoder gives the same pixels before and after", which doesn't depend on crustyimg
   having written the fixture.
   - Fixtures: `tests/fixtures/lint/cmyk_16x16.jpg` (YCCK, transform 2, ImageMagick); a new
     3-component Adobe transform-0 JPEG with component IDs 1,2,3 (`cjpeg -rgb`, then a recorded
     byte edit of the IDs; the README states the recipe); `tests/fixtures/c2pa/signed_gps.jpg`
     (APP1, APP2, APP11, APP13 and APP14 in one real file).
   - Assertions: for each fixture, decoded RGBA of `strip_all(input)` equals decoded RGBA of the
     input exactly; the output's APP14 segment is byte-identical to the input's; and on
     `signed_gps.jpg` no other APP1..APP15 segment or COM survives (an independent segment walk,
     like `jpeg_has_app11`). The two existing sweeps (`:630`, `:1712`) get the exception and an
     APP14-carrying input.
   - Negative control: put `0xEE` back into the sweep and show the pixel tests go red (each
     fixture's failure message quoted). Also show a positive control: the 3-component fixture
     fails pre-fix on `main` (premise above says it will).
   - Would it catch an ICC-driven change? **No.** The fixtures carry no ICC that the code drops and
     `image` doesn't apply ICC when decoding (it returns the raw RGB), so a dropped profile leaves
     decoded pixels identical. Catching it would need a colour-managed comparison. That belongs to
     the ICC question, not here.
7. **Mutation rows.** Red set read off one full `cargo test --no-fail-fast` of every target
   (AGENTS §15), not a filtered run:
   - M1: drop the exception (sweep `0xE1..=0xEF` again) → the pixel tests for both fixtures and the
     APP14-kept assertion go red.
   - M2: widen the exception to a neighbour (keep `0xED` or `0xEB` too, e.g. `0xED..=0xEE`) → the
     "nothing else survives" assertion on `signed_gps.jpg` goes red (and, for `0xEB`,
     `meta_strip_removes_the_jumbf_segment_by_name`).
   - M3: drop the `Adobe\0` signature check (keep every APP14) → a test with a non-Adobe APP14
     goes red. This row exists only if design adopts the signature check.
   - M4: apply the exception in `strip_all` but leave the `:1712` reconstruction unchanged → that
     test goes red once its input carries APP14 (a check that the test update is real).
8. **DECs.** DEC-003's Validation line says "`strip` removes everything" and "no pixel re-encode for
   metadata-only commands". DEC-029's probe recorded `remove_segments_by_marker(0xE1..=0xEF, 0xFE)`
   with "decoded pixels byte-identical", which was true only because the probe JPEG had no APP14.
   Recommend an amendment to DEC-003 (design allocates it; no new DEC is needed) with this content:
   "`strip` removes all metadata segments and keeps the segments a decoder needs to read the scan:
   APP0 (JFIF) and APP14 (Adobe colour transform, only when it carries the `Adobe` signature). Its
   promise is that the image decodes to the same pixels, not only that the scan bytes are
   unchanged. It still drops ICC and EXIF orientation, which can change how a viewer shows the
   image." Add a one-line note to DEC-029's probe result pointing at the amendment.
9. **Docs.** What each says now, and after:
   - `docs/cli-reference.md:231`: "Remove **all** container metadata (EXIF/IPTC/XMP/ICC). JPEG and
     PNG." After: add "Keeps the structural JFIF and Adobe colour-transform segments, so the
     image decodes the same."
   - `README.md:193`: "remove ALL metadata (EXIF/IPTC/XMP/ICC)". Comment is fine; no change unless
     design wants the same clause.
   - `skills/crustyimg/SKILL.md:148`: "remove all EXIF/IPTC/XMP/ICC". Same as README.
   - `--help` (`src/cli/mod.rs:589`): "Remove all metadata at the container level." Accurate;
     no change.
   - `SECURITY.md:88`: "`clean --gps` / `strip` (container lane, no re-encode)". Accurate; no change.
   - `strip_all`'s doc comment (`src/metadata/mod.rs:157-165`): "drop APP1..APP15 … APP0/JFIF is
     structural and kept" and "preserving pixels exactly". Must change to name APP14.
   - CHANGELOG entry at build (a user-visible fix).
10. **Fold-ins.** Read every open STAGE-063 entry. Leave all of them:
    - SPEC-154/155/157/158, `build_watch`, watermark strict, `optimize` to stdout, finding A,
      `docs/recipes.md:148`, and the carried STAGE-054 items: different code and different risks.
    - SPEC-148 verify O-5 (the lint fixture README omits the CMYK fixture's size): one line in a
      file this spec may touch if the new fixture goes beside it. Leave it in its entry; the
      build may fix it in passing if it edits that README, and say so.
    - SPEC-150 verify (3) (`MANIFEST_DROPPED_WARNING` unused on wasm32): same file, but a wasm
      clippy question tied to PR #240; leave it.
11. **Size: S.** The code change is one exception in one loop plus its doc comment. The work is in
    verification: two decoders' worth of pixel evidence, three fixtures (one new), two existing
    tests made non-vacuous, three or four mutation rows, and a C2PA control. Verification axes:
    component count (3, 4) × transform (0, 1, 2) × component IDs (RGB letters, 1-2-3); the C2PA
    warning (unchanged); PNG (unchanged). OS legs: pure byte logic with binary fixtures and no
    paths, so Windows and Linux carry no extra risk beyond the full matrix running; check the new
    fixture is not touched by line-ending conversion (`.gitattributes` has no entry today; the
    other binary fixtures work, so follow them). The lean `--no-default-features` build uses the
    same decoder; run it at verify (memory: verify includes the lean build).

### Scope

In:
- `strip_all` keeps APP14 segments that carry the `Adobe\0` signature; everything else in
  `0xE1..=0xEF` and COM still goes.
- The pixel regression on three fixtures, the APP14-kept assertion, the nothing-else-survives
  assertion, the two existing sweeps updated and made non-vacuous.
- The doc comment, `docs/cli-reference.md`, a DEC-003 amendment (content above), a DEC-029 note,
  CHANGELOG.

Out (and where each lives):
- ICC (APP2 / PNG `iCCP`) dropped by strip and the pixel lane: the DEC-003 entry folded into
  SPEC-150, pending a ruling.
- EXIF orientation dropped by strip (JPEG APP1, PNG `eXIf`): a new STAGE-063 entry filed by this
  frame.
- MPF / gain-map / JPEG XT secondary data: not the primary render; no entry.
- `meta copy` grafting a CMYK-tagged ICC onto an RGB file: an ICC question, same owner as above
  (not driven: the CMYK fixture has no ICC).
- Any change to `has_manifest`, the C2PA warning or `PNG_METADATA_CHUNKS`.

### Rough build file list

- `src/metadata/mod.rs`: the exception in `strip_all`, its doc comment, the two test sweeps,
  new unit tests.
- `tests/metadata.rs` (or a new `tests/strip_render.rs`): CLI-level pixel regression.
- `tests/fixtures/metadata/rgb_adobe_t0_ids123.jpg` (name for design) and a README line with its
  recipe.
- `docs/cli-reference.md`, `CHANGELOG.md`.
- `decisions/DEC-003-metadata-dual-lane.md` (amendment), `decisions/DEC-029-…md` (note).

### Questions for the maintainer

- Keep APP14 only when it carries the `Adobe\0` signature, or keep every APP14? **Recommend the
  signature check**: it costs one comparison and drops a non-Adobe APP14 we know nothing about.
- Should `meta strip`'s docs warn that it drops orientation and ICC, which can change how a
  viewer shows the image? **Recommend yes, but in the new orientation entry, not here**, so this
  spec stays one fix.

### Out-of-spec findings

- **`meta strip` drops EXIF orientation, so a phone photo can display rotated; no doc says so.**
  Filed as a new `(not yet written)` STAGE-063 entry.

### Maintainer rulings (2026-11-22)

Both recommendations accepted:
1. **Keep APP14 only when it starts with `Adobe\0`.** Any other APP14 is still stripped. M3 (drop the
   signature check) is in the mutation set.
2. **The orientation and ICC warning in strip's docs belongs to the new STAGE-063 entry** ("`meta
   strip` drops EXIF orientation, and no doc says so"), not this spec. This spec's docs say only what
   strip now keeps.

The orchestrator recommends framing that orientation entry right after the four P1 specs, inside
STAGE-063.

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

- ICC (JPEG APP2, PNG `iCCP`) dropped by strip or the pixel lane: the DEC-003 entry folded into
  SPEC-150, pending a ruling.
- EXIF orientation dropped by strip: the new STAGE-063 entry this frame filed.
- MPF, gain maps and JPEG XT secondary data.
- `has_manifest`, `MANIFEST_DROPPED_WARNING`, `PNG_METADATA_CHUNKS`.

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
