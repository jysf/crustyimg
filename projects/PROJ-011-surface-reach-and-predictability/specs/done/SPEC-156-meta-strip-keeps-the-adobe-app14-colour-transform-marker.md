---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-156
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: design
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 13776717
      tokens_breakdown:
        input: 180
        cache_creation: 203345
        cache_read: 13493072
        output: 80120
      estimated_usd: 10.02
      note: >
        Measured from the session JSONL (found by its first message, `prompts/SPEC-156-design.md`), deduped by
        message.id (87 messages; max output_tokens per id), 0 subagent transcripts. Measured after the design
        commit, before this cost commit's own turns. The push to `main` was left to the maintainer (the session's
        auto-mode classifier refused a direct push).
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: build
      date: 2026-11-22
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 6608157
      tokens_breakdown:
        input: 98
        cache_creation: 135697
        cache_read: 6436853
        output: 35509
      estimated_usd: 2.97
      note: >
        Re-measured at ship (2026-11-23) from the session JSONL (found by its first message, `prompts/SPEC-156-build.md`), deduped by
        message.id (49 messages; max output_tokens per id), 0 subagent transcripts. Was 5459639 tokens / $2.53, measured before its last turns.
        Rates (Sonnet: $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read (x0.10), $15 output).
    - cycle: verify
      date: 2026-11-23
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 3430158
      tokens_breakdown:
        input: 64
        cache_creation: 106694
        cache_read: 3297746
        output: 25654
      estimated_usd: 2.96
      note: >
        Re-measured at ship (2026-11-23) from the session JSONL (found by its first message, `prompts/SPEC-156-verify.md`), deduped by
        message.id (31 messages; max output_tokens per id), 0 subagent transcripts. Replaces the numbers the punch-list prompt handed over (3287233 tokens / $2.76), which were taken before verify's last turns.
        Rates (Opus: $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read (x0.10), $25 output).
    - cycle: punchlist
      date: 2026-11-23
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 2303187
      tokens_breakdown:
        input: 56
        cache_creation: 68759
        cache_read: 2220841
        output: 13531
      estimated_usd: 1.13
      note: >
        Re-measured at ship (2026-11-23) from the session JSONL (found by its first message, `prompts/SPEC-156-punchlist.md`), deduped by
        message.id (27 messages; max output_tokens per id), 0 subagent transcripts. Was 1430168 tokens / $0.76, measured before the doc commit's own turns.
        Rates (Sonnet: $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read (x0.10), $15 output).
    - cycle: ship
      date: 2026-11-23
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 3412892
      tokens_breakdown:
        input: 72
        cache_creation: 86859
        cache_read: 3310149
        output: 15812
      estimated_usd: 1.56
      note: >
        Re-measured by the orchestrator from the ship session's own final figure (36 messages,
        measured just after the push); the first entry, $1.16, was taken before its last commands.
        Measured from the session JSONL (found by its first message, `prompts/SPEC-156-ship.md`), deduped by
        message.id (27 messages; max output_tokens per id), 0 subagent transcripts. Measured before the
        final bookkeeping commands and the commit's own turns. Rates (Sonnet): $3/MTok input,
        $3.75 cache_creation (x1.25), $0.3 cache_read (x0.10), $15 output.
  totals:
    tokens_total: 34717977
    estimated_usd: 22.80
    session_count: 6
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

## Design

Designed 2026-11-22 (Opus) on `main` at `9560c2c`. Everything below was driven on a release build of `main`
(`crustyimg 0.8.2`) and on a prototype of this design in a throwaway worktree with its own `CARGO_TARGET_DIR`
(never pushed, deleted at the end). Tools: libjpeg-turbo 3.2.0 (`cjpeg`/`djpeg`), ImageMagick 7.1.2-34,
`image` 0.25.10 with `zune-jpeg` 0.5.15 (both from `Cargo.lock`).

### Design corrections to the frame

1. **The signature is `Adobe` plus a length floor, not `Adobe\0`.** The byte after `Adobe` is the high byte of the
   DCTEncodeVersion field (`00 64` = 100), not a NUL terminator. libjpeg and `zune-jpeg`
   (`zune-jpeg-0.5.15/src/headers.rs:485-490`) both check only the five bytes `Adobe` and a minimum length, then
   read the transform at payload offset 11. Driven: the CMYK fixture with that byte set to `0x01`
   (`Adobe\x01\x64…\x02`) still decodes to `srgb(191,64,32)` in djpeg and in crustyimg. A literal `Adobe\0` check
   would strip a segment decoders read, which is the bug this spec fixes. **The rule: keep an APP14 whose payload
   starts with `Adobe` and is at least 12 bytes long** (`Adobe` 5, version 2, flags0 2, flags1 2, transform 1).
   This keeps the ruling's intent (only Adobe's segment is kept, any other APP14 goes) and matches what a
   decoder reads. FT-4 pins it, and mutation M3b (`starts_with(b"Adobe\0")`) turns FT-4 red.
2. **A too-short `Adobe` APP14 is stripped.** Driven, the CMYK fixture's APP14 cut to 11 or 5 payload bytes:
   djpeg ignores it and renders `srgb(93,134,30)`, the same as no APP14 at all; crustyimg refuses the file
   (`Format error decoding Jpeg: "Too short of a length for App14 segment"`). Stripping it can't change what
   libjpeg shows, and it carries no transform to keep. FT-6 pins it, and mutation M5a turns FT-6 red.
3. **The three-component fixture's numbers.** The frame's −21.30 came from its own gradient. The design's
   recipe below gives djpeg (8,8) `srgb(128,128,191)` → `srgb(216,83,128)` after a `main` strip, AE 107.885,
   SSIMULACRA2 **−35.7927**. Same conclusion: it breaks.
4. **`docs/api-contract.md:500-507` is a surface the frame missed.** Its `meta strip` entry already promises
   "decoded pixels byte-identical", which is false on `main` for these files. It gets the same sentence as
   `docs/cli-reference.md` (DEC-015 governs that file).
5. **`src/metadata/mod.rs:11-12` (the module doc) is a second comment to change.** It says `strip_all` removes
   "JPEG APP1..APP15 + COM".

### Driven on `main` and on the prototype

- **Every caller of `strip_all`.** `/usr/bin/grep -rn "strip_all" src/ tests/` outside `src/metadata/`: one
  production caller, `run_strip` (`src/cli/ops.rs:970`, `meta strip`), plus the golden in
  `tests/metadata.rs:780` (`meta_subcommands_match_old_verbs`, which calls `strip_all` itself on an input with
  no APP14, so it moves with the code). `src/sink/mod.rs:617` names it in a comment only. No recipe step,
  `optimize`, `web`, `build`, `responsive` or wasm path calls it: their "strip metadata" is the pixel lane's
  re-encode (DEC-003's SPEC-150 amendment), which writes no APP14 (frame, `resize --max 16` of the CMYK
  fixture writes `APP0 … SOS`). `src/wasm.rs` has no `metadata::` call (its two "strip" hits are about recipe
  markers, `:184`, `:188`). So no caller strips **after** a re-encode, and the opposite bug (an RGB scan
  labelled YCCK) has no path.
- **Who reads "metadata is gone".** Outputs of `meta strip` from `main` (`_s`) and the prototype (`_p`):
  - `info cmyk_s.jpg` / `info cmyk_p.jpg`: `icc: no`, `exif: no` in both; only `file size` changes,
    `280 bytes` → `296 bytes` (+16, the APP14 segment). `info --exif --json`: the only differing field is
    `"file_size_bytes":280` → `296`. `signed_gps.jpg`: `81672 bytes` → `81707 bytes` (+35).
  - `lint` on a folder of the three stripped files: the same single finding both times,
    `warn color/wrong-colorspace: CMYK JPEG (not an sRGB web colorspace)`, `3 scanned · 0 error · 1 warn ·
    0 info`, exit 0. No rule reads APP14 or "not stripped" (`/usr/bin/grep -n "0xEE\|APP14\|Adobe"
    src/lint/*.rs` hits only an ICC comment at `src/lint/rules.rs:13`).
  - `diff --json cmyk.jpg <stripped>`: `"score":41.9374` → `"score":100.0000`.
  - There is no `meta show` verb (`crustyimg meta --help` lists strip, clean, copy, set).
  - Goldens: none pins a stripped output that carries APP14. The prototype's full run kept every existing
    test green (1356 passed, 0 failed).
  - Visible changes, so in AC-6 and the CHANGELOG: the stripped file keeps its colours, and it is larger by
    the APP14 segment (16 bytes in the common 14-byte form).
- **The segment bytes.** CMYK fixture at offset 2:
  `ff ee 00 0e 41 64 6f 62 65 00 64 00 00 00 00 02` (`Adobe`, version `00 64`, flags0 `00 00`, flags1
  `00 00`, transform 2). `signed_gps.jpg` at offset 122906, length 33:
  `ff ee 00 21 41 64 6f 62 65 00 64 00 00 00 00 01 03 00 10 03 02 03 06 00 00 00 00 00 00 00 00 00 00 00 00`
  (transform 1, then 19 bytes of small integers and zeros).
- **Truncated and foreign APP14, driven** (each built from the CMYK fixture's body; djpeg pixel (8,8), then
  crustyimg's decode, then what `main` and the prototype's strip leave):
  - full Adobe, 12 bytes: `srgb(191,64,32)`, `srgb(191,64,32)`; `main` drops it, prototype keeps it.
  - `Adobe\x01…` (version byte 1): `srgb(191,64,32)`, `srgb(191,64,32)`; `main` drops, prototype keeps.
  - `Adobe` + 6 bytes (11, no transform): `srgb(93,134,30)`, decode error; both drop.
  - `Adobe` only (5 bytes): `srgb(93,134,30)`, decode error; both drop.
  - `Ducky\x00\x64…\x02` (12 bytes, not Adobe): `srgb(93,134,30)`, `srgb(93,134,30)`; both drop.
  - no APP14: `srgb(93,134,30)`, `srgb(93,134,30)`.
- **`image` honours APP14** on the locked version: the prototype's unit tests decode the CMYK fixture to
  `[191, 64, 32, 255]` at (8,8) and the three-component fixture to `[128, 128, 191, 255]`, the same values as
  djpeg. djpeg (and ImageMagick's AE) stays the independent check in this spec's evidence.
- **The two existing sweeps.** `strip_all_jpeg_removes_all_metadata` (`src/metadata/mod.rs:619`, loop at
  `:630`) and `meta_verbs_are_byte_identical_on_unsigned_input` (`:1704`, reconstruction loop at `:1712`) both
  run on `jpeg_with_exif()` (`:536`), an `image`-encoded JPEG with no APP14, so neither can fail on this bug.
  Each gets an Adobe APP14 in its input (FT-7, FT-8).
- **wasm and the lean build.** `pub mod metadata` is compiled on every target (`src/lib.rs:59`), so
  `strip_all` and the new helpers compile into wasm32, but nothing in `src/wasm.rs` calls them (no bundle
  change expected). `meta strip` is in the native CLI on every feature set: the change runs in `check`
  (3 OS), `lean` (`--no-default-features`), `avif`, `webp-lossy`, `heic`, and `msrv` (build only).
  `wasm-roundtrip` and the bundle-size gate compile it without running it.
- **Baseline before the fix.** On the prototype's tests with `main`'s `strip_all` body (row M1 below), the
  pixel tests go red with the messages quoted there. That is the positive control: the three-component
  fixture and the CMYK fixture fail on `main`'s code.

## Inputs

- **Files to read:** `src/metadata/mod.rs` (`strip_all` `:157-195`, the test module from `:365`, the sweeps
  at `:619` and `:1704`); `src/cli/ops.rs:960-971` (`run_strip`); `tests/metadata.rs:612-620`
  (`decode_rgba`); `tests/c2pa_manifest.rs:327-357` (the C2PA control);
  `zune-jpeg-0.5.15/src/headers.rs:474-520` (what the decoder reads).
- **External APIs:** none.
- **Related code paths:** `src/metadata/`.

## Outputs

- **Files created:** `tests/fixtures/metadata/rgb_adobe_t0_ids123.jpg` (452 bytes, recipe below) and
  `tests/fixtures/metadata/README.md` (the recipe).
- **Files modified:** `src/metadata/mod.rs` (two constants, two helpers, `strip_all`'s loop and doc comment,
  the module doc, two updated tests, six new tests); `tests/metadata.rs` (one new test);
  `docs/cli-reference.md`; `docs/api-contract.md`; `CHANGELOG.md`.
- **New exports:** none (the helpers are private).
- **Database changes:** none.

## Acceptance Criteria

Each AC names its checks (FT ids below). Every check names what turns it red.

- [ ] **AC-1 `meta strip` keeps Adobe's colour-transform APP14 byte for byte, and the stripped JPEG decodes to
  the same pixels.** For each of: a 4-component YCCK JPEG (transform 2), a 3-component RGB JPEG (transform 0)
  with component ids 1,2,3, a real camera JPEG (transform 1, 33-byte APP14), and an Adobe APP14 whose
  version byte is not 0. Decoded RGBA of the output equals decoded RGBA of the input; the output's APP14
  equals a literal of the input's. — FT-1, FT-2, FT-3, FT-4 (library); FT-9 (the `meta strip` binary, YCCK).
- [ ] **AC-2 Everything else in APP1..APP15, and COM, is still removed.** On `signed_gps.jpg` the output's
  header is exactly `APP14 DQT SOF2 DHT SOS` (APP1 ×2, APP2, APP11 ×2, APP13 gone) and carries no C2PA
  manifest; on a seeded JPEG, APP1 and COM are gone and every marker `0xE1..=0xEF` except `0xEE` is absent.
  — FT-3, FT-7.
- [ ] **AC-3 An APP14 that is not Adobe's, too short to hold the transform, or an `Adobe`-prefixed segment under another marker, is removed** (widened at punch list). A 12-byte
  `Ducky…` APP14, an 11-byte `Adobe…` APP14, and a 16-byte `Adobe_CM…` APP13 are all gone after strip. — FT-5, FT-6, FT-10 (added at punch list).
- [ ] **AC-4 A file with nothing to strip comes back unchanged.** `strip_all` and `meta strip` of the CMYK
  fixture, and `strip_all` of the three-component fixture, return the input bytes exactly. — FT-1, FT-2,
  FT-9.
- [ ] **AC-5 Nothing else changes.** `meta strip` on a C2PA-signed file still exits 0 with empty stderr and
  removes APP11 (`tests/c2pa_manifest.rs:327`, unchanged). `meta clean --gps`, `meta set` and `meta copy`
  bytes are unchanged (FT-8 and the existing pins listed under "What does not change"). The PNG lane and
  every pixel-lane output are unchanged (no code touched; existing pins). — FT-8, existing tests.
- [ ] **AC-6 The user-visible change is documented.** `docs/cli-reference.md`, `docs/api-contract.md`, the
  module doc and `strip_all`'s doc comment say strip keeps JFIF and Adobe's colour-transform segment so the
  image decodes the same; the CHANGELOG "Fixed" line says what changed and that the output keeps that
  segment. The size change (+16 bytes on the common form) is a consequence, shown in `info` only as
  `file size`. — docs review at verify (the exact sentences are under Implementation Context).

## Failing Tests

Written during **design**, BEFORE build. All but FT-7 and FT-8 are new; FT-7 and FT-8 replace existing bodies.
Every one was written and run on the prototype: green with the fix, red set per row in the mutation table.
Expected values are literals (from djpeg or the fixture's bytes) or the input decoded by the same decoder,
never output of the code under test.

Shared test helpers to add in `src/metadata/mod.rs`'s test module (next to `jpeg_has_marker`, `:558`):
`CMYK_YCCK_JPEG`, `RGB_T0_IDS123_JPEG`, `SIGNED_GPS_JPEG` (`include_bytes!` of the three fixtures,
`"../../tests/fixtures/…"`); `ADOBE_APP14_YCBCR_CONTENTS = b"Adobe\x00\x64\x00\x00\x00\x00\x01"`;
`segment_bytes(marker, contents)` (`FF marker len contents`); `with_segments(jpeg, &[(marker, contents)])`
(inserts after SOI, or after APP0 when the first segment is APP0, via `img_parts::jpeg::JpegSegment::new_with_contents`);
`walk_segments(bytes)` (an independent header walk, **not** `img-parts`: from offset 2, read `FF marker len`,
push `(marker, the segment's bytes)`, stop after SOS); `header_markers`, `app14_segments`; `rgba(bytes)`;
`assert_same_render(original, stripped)` (compares RGBA and, on a mismatch, reports
`stripped file decodes differently: N of M pixels differ, first at (x, y): [..] -> [..]` instead of dumping
the buffer, since `signed_gps.jpg` is 2100×1500).

- **`src/metadata/mod.rs` (unit tests)**
  - **FT-1** `strip_all_keeps_the_adobe_app14_of_a_ycck_jpeg` — asserts: fixture APP14 equals the 16-byte literal
    `ff ee 00 0e 41 64 6f 62 65 00 64 00 00 00 00 02` and the fixture decodes to `[191, 64, 32, 255]` at (8,8)
    (preconditions); the output decodes the same as the input (`assert_same_render`) and to
    `[191, 64, 32, 255]` at (8,8); the output's APP14 list equals `[that literal]`; the output equals the
    input bytes ("nothing to strip").
  - **FT-2** `strip_all_keeps_the_adobe_app14_of_an_rgb_jpeg_with_numeric_component_ids` — asserts the
    fixture's bytes first: `[2..18]` is `ff ee 00 0e 41 64 6f 62 65 00 64 00 00 00 00 00` (transform 0);
    `[87..89]` is `ff c0` and `[97]`, `[100]`, `[103]` are `1, 2, 3`; `[322..324]` is `ff da` and `[327]`,
    `[329]`, `[331]` are `1, 2, 3`; it decodes to `[128, 128, 191, 255]` at (8,8). Then: same render, the
    same (8,8) value after strip, APP14 kept as the literal, output equals input bytes.
  - **FT-3** `strip_all_keeps_only_the_adobe_app14_of_a_signed_camera_jpeg` — asserts: fixture header markers
    are `[EB, EB, E1, ED, E1, E2, EE, DB, C2, C4, DA]` (precondition); output header markers are exactly
    `[EE, DB, C2, C4, DA]`; output APP14 equals `segment_bytes(0xEE, <the 31-byte literal payload>)`;
    `!has_manifest(&out)`; same render.
  - **FT-4** `strip_all_keeps_an_adobe_app14_whatever_its_version_byte` — input: the CMYK fixture with file
    offset `[11]` (the version's high byte; SOI 0–1, marker 2–3, length 4–5, `Adobe` 6–10) set from `0x00` to
    `0x01`, built in the test; asserts the input's `[6..12]` is `b"Adobe\x01"` (precondition); output APP14 equals the input's `[2..18]`; same render and
    `[191, 64, 32, 255]` at (8,8).
  - **FT-5** `strip_all_removes_an_app14_that_is_not_adobes` — input: `base_image(ImageFormat::Jpeg)` with
    APP14 `b"Ducky\x00\x64\x00\x00\x00\x00\x01"` (12 bytes) inserted; asserts one APP14 in the input and none
    in the output.
  - **FT-6** `strip_all_removes_an_adobe_app14_too_short_to_hold_the_transform` — input: `base_image(Jpeg)`
    with APP14 `b"Adobe\x00\x64\x00\x00\x00\x00"` (11 bytes); asserts one APP14 in the input and none in the
    output.
  - **FT-7** `strip_all_jpeg_removes_all_metadata` (existing, `:619`, body replaced) — input:
    `with_segments(jpeg_with_exif(), [(0xEE, ADOBE_APP14_YCBCR_CONTENTS), (0xFE, b"a comment")])`;
    preconditions APP1, APP14 and COM present; after strip, for each marker in `0xE1..=0xEF`: `0xEE`'s
    segments equal `[segment_bytes(0xEE, ADOBE_APP14_YCBCR_CONTENTS)]`, every other marker absent; COM
    absent (`COM should be gone`); APP0 present (`assert!(jpeg_has_marker(&out, 0xE0), "APP0/JFIF should be
    kept")`, new); no EXIF readable (the existing tail, unchanged).
  - **FT-8** `meta_verbs_are_byte_identical_on_unsigned_input` (existing, `:1704`, strip part changed) —
    `strip_input` = `with_segments(jpeg_with_exif(), [(0xEE, ADOBE_APP14_YCBCR_CONTENTS)])`, plus a
    precondition that it has APP14; the expected bytes rebuild strip with
    `jpeg.segments_mut().retain(|s| strip_keeps_jpeg_segment(s.marker(), s.contents()))` in place of the
    `:1712` loop. The rest of the test (set/clean/copy parts) is unchanged. Its comment at `:1705-1706`
    stays true. This test checks the lane is byte-identical; it is not the APP14 oracle (FT-1 to FT-7 are).
- **`tests/metadata.rs` (binary)**
  - **FT-9** `meta_strip_keeps_the_colours_of_an_adobe_cmyk_jpeg` — runs `crustyimg meta strip
    tests/fixtures/lint/cmyk_16x16.jpg -o -`; asserts exit 0, stdout decodes (`decode_rgba`) to
    `[191, 64, 32, 255]` at pixel (8,8) (index `(8*16+8)*4`), stdout decodes the same as the file, and stdout
    equals the file's bytes. This is the Trap-14 control (it lives outside `src/metadata/mod.rs`).
  - **FT-10** `strip_all_removes_an_adobe_cm_app13` *(added at punch list)* — input:
    `with_segments(jpeg_with_exif(), [(0xED, b"Adobe_CM\x00\x01\x00\x00\x0c\x00\x00\x00")])`; asserts one APP13
    in the input (precondition) and none in the output, and that the payload is at least `ADOBE_APP14_MIN_LEN`
    bytes and starts with `Adobe`, so a pass can only come from the marker check, not the length or the
    signature. Photoshop writes an APP13 like this.
- **C2PA control (existing, unchanged):** `tests/c2pa_manifest.rs::meta_strip_removes_the_jumbf_segment_by_name`
  (`:327`): exit 0, APP11 gone, stderr empty. Green on `main` and on the prototype; FT-3 adds
  `!has_manifest` on the library side.

### Mutation table (traps 12–14): apply each alone, record every test red

Driven at design on the prototype. "Full" = one `cargo test --no-fail-fast` of every target (62 test binaries
+ doc-tests), red set read off that run; prototype baseline 1356 passed / 0 failed (sum of the `test result`
lines). "Lib" = `cargo test --lib --no-fail-fast` after FT-4 and FT-7's APP0 assertion were added (baseline 603
passed / 0 failed); FT-4 and that assertion live only in the lib target, so the lib run completes the red set.
Test ids: FT-1…FT-9 above; **C** = `tests/c2pa_manifest.rs::meta_strip_removes_the_jumbf_segment_by_name`;
**L** = `tests/lint.rs::every_printed_fix_run_verbatim_fixes_its_finding`. A test not listed as red stayed green.

- **M1 — the old sweep restored (the negative control, `0xEE` back in the sweep).** `strip_all`'s JPEG arm,
  the `retain(…)` line replaced by `main`'s verbatim
  `for marker in 0xE1u8..=0xEF { jpeg.remove_segments_by_marker(marker); } jpeg.remove_segments_by_marker(0xFE);`.
  **Red:** FT-1, FT-2, FT-3, FT-4, FT-7, FT-8, FT-9 (full: 1350 passed / 6 failed; lib adds FT-4). **Green:**
  FT-5, FT-6, C, L. *Driven.* Messages:
  - FT-1: `stripped file decodes differently: 256 of 256 pixels differ, first at (0, 0): [191, 64, 32, 255] -> [93, 134, 30, 255]`
  - FT-2: `stripped file decodes differently: 256 of 256 pixels differ, first at (0, 0): [0, 0, 255, 255] -> [178, 0, 0, 255]`
  - FT-3: `assertion left == right failed  left: [219, 194, 196, 218]  right: [238, 219, 194, 196, 218]`
  - FT-4: `left: []  right: [[255, 238, 0, 14, 65, 100, 111, 98, 101, 1, 100, 0, 0, 0, 0, 2]]` (APP14 list)
  - FT-7: `the Adobe APP14 should be kept unchanged  left: []  right: [[255, 238, 0, 14, 65, 100, 111, 98, 101, 0, 100, 0, 0, 0, 0, 1]]`
  - FT-8: `strip_all must be byte-identical on unsigned input` (the right side carries `255, 238, 0, 14, …`)
  - FT-9: `pixel (8,8)  left: [93, 134, 30, 255]  right: [191, 64, 32, 255]`
- **M2a — exception widened to a neighbour, APP13.** `strip_keeps_jpeg_segment`:
  `!metadata || is_adobe_colour_transform(marker, contents) || marker == 0xED`. **Red:** FT-3
  (`left: [237, 238, 219, 194, 196, 218]  right: [238, 219, 194, 196, 218]`). Full 1355/1; lib 602/1. *Driven.*
- **M2b — exception widened to APP11 (C2PA).** Same line, `|| marker == 0xEB`. **Red:** FT-3, C
  (`strip must remove the JUMBF/APP11 manifest segment, driven against a REAL signed fixture …`). Full 1354/2;
  lib 602/1. *Driven.*
- **M3 — the signature check dropped** (`is_adobe_colour_transform` without `&& contents.starts_with(b"Adobe")`).
  **Red:** FT-5 (`a non-Adobe APP14 is metadata`). Full 1355/1; lib 602/1. *Driven.*
- **M3b — the ruling's literal bytes** (`contents.starts_with(b"Adobe\0")`). **Red:** FT-4
  (`left: []  right: [[255, 238, 0, 14, 65, 100, 111, 98, 101, 1, …]]`). Lib 602/1. *Driven on the lib
  target*; other targets *reasoned* green (no other fixture's version byte is non-zero).
- **M4 — FT-8's reconstruction left as the old loop** (FT-8's `retain(…)` replaced by the `0xE1..=0xEF` loop plus
  `remove_segments_by_marker(0xFE)`; `strip_all` fixed). **Red:** FT-8
  (`strip_all must be byte-identical on unsigned input`). Full 1355/1; lib 602/1. *Driven.*
- **M5a — length floor one too low** (`const ADOBE_APP14_MIN_LEN: usize = 11;`). **Red:** FT-6
  (`a truncated APP14 is not read by decoders`). Full 1355/1; lib 602/1. *Driven.*
- **M5b — length floor one too high** (`= 13`). **Red:** FT-1, FT-2, FT-4, FT-7, FT-9 (the 12-byte form is the
  common one; FT-3's 31-byte payload stays green). Full 1352/4; lib 599/4. *Driven.*
- **M6 — the exception widened to every APP14 and to APP2 (ICC)** (`!metadata || marker == 0xEE || marker == 0xE2`).
  **Red:** FT-3, FT-5, FT-6, **L** (`printed fixes do not fix their findings: still_reported=1 unchanged=1 / row
  unexpected-icc … fix: "crustyimg meta strip icc.jpg -o icc.jpg -y"`: lint's printed fix for an unexpected ICC is
  `meta strip`, so keeping APP2 breaks it). Full 1352/4 over all 62 test binaries; the run was then killed
  (exit 143) before the doc-test stage, which has no strip test. Lib 600/3. *Driven.*
- **M7 — COM no longer removed** (`let metadata = (0xE1..=0xEF).contains(&marker);`). **Red:** FT-7
  (`COM should be gone`). Full 1355/1; lib 602/1. *Driven.*
- **M8 — APP0 swept too** (`let metadata = (0xE0..=0xEF).contains(&marker) || marker == 0xFE;`). **Red:** FT-7
  (`APP0/JFIF should be kept`). Lib 602/1. *Driven on the lib target*; other targets *reasoned* (FT-9's fixture
  has no APP0).
- **T14 — `main`'s `src/metadata/mod.rs` restored verbatim** (`git show origin/main:src/metadata/mod.rs`). FT-1 to
  FT-8 leave with the file (they live in its test module). **Red:** FT-9
  (`pixel (8,8)  left: [93, 134, 30, 255]  right: [191, 64, 32, 255]`). Full 1350 passed / 1 failed. *Driven.*
  FT-9 is the control that survives Trap 14.
- **MK — the marker clause always true** *(added at punch list)* (`marker == JPEG_ADOBE_MARKER` →
  `(marker == JPEG_ADOBE_MARKER || true)` in `is_adobe_colour_transform`). **Red:** FT-10 only
  (`an Adobe-prefixed segment under another marker is metadata`). Full 1357 passed / 1 failed; lib 603/1.
  Before FT-10, all 1,357 tests stayed green on this mutant. *Driven.*

Also driven: `cargo clippy --all-targets -- -D warnings` clean on the prototype; `cargo build --target
wasm32-unknown-unknown --lib` with the rustup toolchain on `PATH` (`justfile:119`: a bare `cargo` finds
Homebrew's `rustc`, which has no wasm32 `core`) builds with one warning, the existing `MANIFEST_DROPPED_WARNING`
unused on wasm32 (SPEC-150's item, PR #240), and none from the new helpers.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### The code (each piece with where it goes)

In `src/metadata/mod.rs`, right after `MANIFEST_DROPPED_WARNING` (`:150-155`) and before `strip_all`'s doc
comment (`:157`). If PR #240 has merged first, that constant has moved out of this file; put these after
`has_manifest` (`:126-148`) instead.

```rust
/// JPEG APP14 (`0xEE`) — the marker Adobe's colour-transform segment is
/// written under.
const JPEG_ADOBE_MARKER: u8 = 0xEE;

/// The shortest Adobe APP14 payload a decoder reads the colour transform
/// from: `Adobe` (5 bytes), version (2), flags0 (2), flags1 (2), transform
/// (1). libjpeg ignores a shorter one; zune-jpeg refuses it.
const ADOBE_APP14_MIN_LEN: usize = 12;

/// Whether a JPEG segment is the Adobe colour-transform segment. Its last
/// field tells a decoder how the scan's components are coded (RGB or YCbCr
/// for three, CMYK or YCCK for four). Without it a decoder guesses from the
/// component ids, and for some files guesses wrong, so the image decodes to
/// different colours. An APP14 that is not Adobe's, or too short to hold the
/// transform, is not read by decoders and is treated as metadata.
fn is_adobe_colour_transform(marker: u8, contents: &[u8]) -> bool {
    marker == JPEG_ADOBE_MARKER
        && contents.len() >= ADOBE_APP14_MIN_LEN
        && contents.starts_with(b"Adobe")
}

/// Whether [`strip_all`] keeps a JPEG segment: everything outside APP1..APP15
/// and COM, plus the Adobe colour-transform segment.
fn strip_keeps_jpeg_segment(marker: u8, contents: &[u8]) -> bool {
    let metadata = (0xE1..=0xEF).contains(&marker) || marker == 0xFE;
    !metadata || is_adobe_colour_transform(marker, contents)
}
```

`strip_all`'s JPEG arm (`:171-175`): replace the `for marker in 0xE1u8..=0xEF { … }` loop and the
`remove_segments_by_marker(0xFE)` line with

```rust
            // APP1 (EXIF/XMP) .. APP15, plus COM (0xFE). APP0/JFIF and the
            // Adobe colour-transform APP14 are kept.
            jpeg.segments_mut()
                .retain(|segment| strip_keeps_jpeg_segment(segment.marker(), segment.contents()));
```

(`img_parts::jpeg::Jpeg::segments_mut` returns `&mut Vec<JpegSegment>`; `JpegSegment::marker()` and
`contents()` are public in img-parts 0.4.0.) `strip_keeps_jpeg_segment` is the one helper; FT-8's
reconstruction (`:1712`) calls it too, so M4 (the old loop left in FT-8) is a real mutant. Run `cargo fmt`
after: the `retain` line is formatted by rustfmt, don't hand-wrap it.

`has_manifest`, `JPEG_MANIFEST_MARKER` (its doc at `:100-104` stays true), `MANIFEST_DROPPED_WARNING`,
`PNG_METADATA_CHUNKS`, `write_exif_block`, `copy_metadata`, `clean_gps`, `set_tags` and `run_strip` are not
touched.

### The new fixture (the build makes it; commit the result)

`tests/fixtures/metadata/rgb_adobe_t0_ids123.jpg`, a 16×16 3-component Adobe JPEG with transform 0 and
component ids 1,2,3. Recipe (libjpeg-turbo 3.2.0 `cjpeg`; another version may give other bytes, so check the
hash, and if it differs, check the offsets below by the asserts in FT-2 and record the new hash):

1. A 16×16 binary PPM, pixel `(x, y)` = `(16x, 16y, 255 − 8x)`, rows top to bottom:
   ```
   python3 -I -c "import sys; px=b''.join(bytes([x*16, y*16, 255-x*8]) for y in range(16) for x in range(16)); sys.stdout.buffer.write(b'P6\n16 16\n255\n'+px)" > grad16.ppm
   ```
   sha256 `9ab37bf50d6b2a140914253443f05e3551c9bfebe7cb48a296b2ec60eba45caa`.
2. `cjpeg -rgb -quality 90 grad16.ppm > rgbRGB.jpg` → 452 bytes, sha256
   `7d106aa3cb5aeb08534dd8193d550bb9174bd034d4be99d1878ccf40136173c9`. Segments: `APP14(t=0) DQT SOF0 DHT DHT SOS`,
   APP14 at offset 2, SOF0 at 87, SOS at 322. Component ids are `R G B` (`0x52 0x47 0x42`).
3. Byte edit: set offsets **97, 100, 103** (SOF0 component ids) and **327, 329, 331** (SOS component
   selectors) from `0x52, 0x47, 0x42` to `0x01, 0x02, 0x03`. Result: 452 bytes, sha256
   `dd3ec8ba9942afbeaab6e5e024afd0e32ae7418c3173cd72c91fb2490fdbf414`.
4. Check: `djpeg -pnm` of the result, pixel (8,8) is `srgb(128,128,191)`; after `main`'s strip it is
   `srgb(216,83,128)`.

`tests/fixtures/metadata/README.md` states that recipe (steps 1–4, the three hashes) and says the file is made
by tools independent of crustyimg, like `tests/fixtures/lint/README.md`. FT-2's byte asserts prove the ids and
transform; the render alone doesn't. Binary fixtures need no `.gitattributes` entry (there is none today and
the other `.jpg` fixtures pass on Windows).

### Docs (exact text)

1. `docs/cli-reference.md:231`, now `Remove **all** container metadata (EXIF/IPTC/XMP/ICC). JPEG and PNG.`
   Becomes:
   `Remove **all** container metadata (EXIF/IPTC/XMP/ICC). JPEG and PNG. On a JPEG it keeps the two segments a
   decoder needs to read the image, JFIF and Adobe's colour-transform marker, so the stripped file decodes to the
   same pixels.`
2. `docs/api-contract.md:501-502`, now `Remove **all** container metadata (EXIF/IPTC/XMP/ICC) via \`img-parts\`
   segment/chunk removal — no pixel re-decode (decoded pixels byte-identical).` Insert after that sentence:
   `On JPEG, APP0 (JFIF) and Adobe's colour-transform APP14 are kept: a decoder reads them, and without the
   APP14 some CMYK and RGB JPEGs decode to different colours.` The rest of the entry is unchanged.
3. `src/metadata/mod.rs:157-160` (`strip_all`'s doc), the JPEG bullet becomes:
   ```
   /// - **JPEG:** drop APP1..APP15 (`0xE1..=0xEF` — EXIF/XMP/ICC/**JUMBF/C2PA
   ///   at `0xEB`**/…) and COM (`0xFE`). Two segments a decoder needs are kept:
   ///   APP0/JFIF, and the Adobe APP14 colour-transform segment, without which
   ///   some CMYK and RGB JPEGs decode to different colours.
   ```
4. `src/metadata/mod.rs:11-12` (module doc), now `(JPEG APP1..APP15 + COM; PNG …)`. Becomes
   `(JPEG APP1..APP15 except Adobe's colour-transform APP14, + COM; PNG \`eXIf\`/\`iCCP\`/\`tEXt\`/…)`.
5. **No change, checked:** `README.md:193` (`remove ALL metadata (EXIF/IPTC/XMP/ICC)`),
   `skills/crustyimg/SKILL.md:148` (`remove all EXIF/IPTC/XMP/ICC`), `--help` (`src/cli/mod.rs:589`,
   `Remove all metadata at the container level.`), `SECURITY.md:88`, `run_strip`'s doc (`src/cli/ops.rs:960`).
   APP14 is not metadata in any of their lists, so they stay true. By the maintainer's ruling, the
   orientation/ICC warning is the STAGE-063 orientation entry's.

**Behaviour sentences the build drives, and verify drives again** (each with the check):
- B1 "keeps … JFIF": FT-7's APP0 assertion (its input is an `image`-encoded JPEG, which starts with APP0;
  `with_segments` inserts after it). Mutation M8 turns it red.
- B2 "keeps … Adobe's colour-transform marker": FT-1 to FT-4, FT-9.
- B3 "decodes to the same pixels": FT-1 to FT-4 (`assert_same_render`), FT-9.
- B4 "Remove all container metadata (EXIF/IPTC/XMP/ICC)": FT-3 (APP1, APP2, APP13 gone), FT-7.
- B5 (api-contract) "without the APP14 some CMYK and RGB JPEGs decode to different colours": the M1 row
  (FT-1, FT-2 red with a pixel mismatch).
- Exit paths `docs/api-contract.md:503-507` names (exit 4 unsupported format, multiple inputs need
  `--out-dir`, exit 6 batch failure, overwrite refused without `-y`, no-metadata input a no-op exit 0) are
  unchanged and pinned by `tests/metadata.rs` `strip_unsupported_format_exits_4` (`:317`),
  `strip_multi_input_requires_out_dir` (`:334`), `strip_batch_partial_failure_exits_6` (`:381`),
  `strip_refuses_overwrite_without_yes` (`:412`); the no-op is FT-1/FT-2/FT-9's "output equals input".

### CHANGELOG (verbatim, under `## [Unreleased]` → `### Fixed`, after the existing colour entry)

```
- **`meta strip` keeps a JPEG's colours.** It removed Adobe's APP14 segment along with the
  metadata, and that segment tells a decoder how the scan's colours are coded: an Adobe CMYK
  JPEG came back with orange turned green, and some RGB JPEGs changed colour too. Strip now keeps
  that segment (and JFIF, as before), so the stripped file decodes to the same pixels; it is
  16 bytes larger than before for a typical file. Any other APP14 is still removed.
```

### What does not change, and what pins it

- `meta clean --gps`, `meta set`, `meta copy` output bytes: FT-8's set/clean/copy halves (unchanged), plus
  `tests/metadata.rs` `meta_subcommands_match_old_verbs` (`:774`) and `meta_set_matches_old_set` (`:850`).
  No new pin: their code is not touched.
- The PNG lane: `strip_all_png_removes_metadata_chunks` (`src/metadata/mod.rs:650`) and
  `tests/c2pa_manifest.rs` `meta_strip_and_set_remove_the_png_cabx_manifest_chunk` (`:362`). The PNG arm is not
  touched.
- Pixel-lane outputs: no pixel-lane file is touched; their goldens and the full suite (1356 green on the
  prototype) pin them.
- C2PA: `has_manifest` reads only `0xEB`; strip passes `warn_on_manifest_drop = false` (`src/cli/ops.rs:970`).
  Pinned by `meta_strip_removes_the_jumbf_segment_by_name` (`tests/c2pa_manifest.rs:327`) and FT-3's
  `!has_manifest`.

### DEC text for ship (do **not** edit either DEC in build; ship applies these after the merge)

**DEC-003, append before `## References`:**

```
## Amendment (<ship date>, SPEC-156)

**`strip` keeps the two segments a JPEG decoder needs to read the scan.** The Validation line above
says *"`strip` removes everything"*, and DEC-029's probe recorded *"decoded pixels byte-identical"*. Both
held only because the probe JPEG had no Adobe APP14. Strip's JPEG sweep removed APP14 (`0xEE`), whose
transform byte tells a decoder how the scan's components are coded; without it a decoder guesses from the
component ids. Driven at SPEC-156's frame and design: an Adobe YCCK JPEG
(`tests/fixtures/lint/cmyk_16x16.jpg`) went from `srgb(191,64,32)` to `srgb(93,134,30)` at (8,8) in
libjpeg-turbo, ImageMagick and crustyimg's own decoder (SSIMULACRA2 41.94), and a 3-component Adobe RGB JPEG
with component ids 1,2,3 broke the same way (SSIMULACRA2 −35.79).

**Corrected claim:** `strip` removes all metadata segments and keeps APP0 (JFIF) and Adobe's
colour-transform APP14: an APP14 whose payload starts with `Adobe` and is at least 12 bytes long, the test
libjpeg and zune-jpeg apply before reading the transform. Any other APP14 is removed. Its promise is that the
image decodes to the same pixels, not only that the scan bytes are unchanged. It still drops ICC and EXIF
orientation, which can change how a viewer shows the image (STAGE-063's orientation entry; the ICC question
under the SPEC-150 amendment above).
```

Also add `tests/fixtures/metadata/**` to DEC-003's `affected_scope` (block-list form).

**DEC-029, after the first probe bullet (the `remove_segments_by_marker(0xE1..=0xEF, 0xFE)` one, `:66-69`):**

```
  > Note (<ship date>, SPEC-156): "decoded pixels byte-identical" held because the probe JPEG had no
  > Adobe APP14. `strip` now keeps that segment; see DEC-003's SPEC-156 amendment.
```

### Overlap with other open work (from their file lists, `gh pr list --state open --json number,title,files`
and `git branch -r`, 2026-11-22)

- **PR #240** (`chore/wasm-clippy`, open, last updated 2026-11-02): touches `.github/workflows/ci.yml`,
  `justfile`, `src/cli/ops.rs`, `src/metadata/mod.rs`. In `src/metadata/mod.rs` it deletes
  `MANIFEST_DROPPED_WARNING` and its doc (`:150-155`), the lines right above `strip_all`'s doc comment where
  this spec inserts its constants. Not a logical conflict, but a textual one is likely: on a rebase, keep both
  (#240's deletion and this spec's insertion). It does not touch `strip_all` or the tests.
- **PR #237** (heifer): `src/image/*`, `src/source/mod.rs`, `CHANGELOG.md`, others. Shares only
  `CHANGELOG.md` (a different section is likely; on a conflict keep both entries).
- **PR #229** (`docs/roadmap.md`), **PR #226** (`AGENTS.md`, `src/cli/mod.rs`): no shared file.
- **Branches `feat/spec-15[4-8]*`:** none pushed (`git branch -r` lists none).
- SPEC-155 (designed alongside) plans `src/sink/mod.rs`, `src/cli/ops.rs`, `src/cli/build.rs`; this spec
  touches none of them. SPEC-157's file list is its own.
- STAGE-063's file: on a rebase conflict, keep both sides and recount the `**Count:**` line with
  `/usr/bin/grep -c`. On a conflict in any other file, stop and report. Never edit
  `projects/PROJ-017-geometry-as-recipe-steps/`.

### Verification the build runs

- Baseline first (trap 13): on `main`'s code, with the new tests added, run the full suite and quote each
  red row's failure message; FT-1, FT-2, FT-4, FT-9 must fail on a pixel or APP14 mismatch (messages under
  M1), never on a fixture precondition.
- Every mutation row below, each alone, with a full `cargo test --no-fail-fast` of every target, stdout
  redirected to a file (`display_sink_refuses_non_tty` fails on a TTY); read the red set off that run.
- `cargo clippy --all-targets -- -D warnings`, `cargo test --no-default-features`,
  `cargo build --target wasm32-unknown-unknown --lib` (each in a fresh `CARGO_TARGET_DIR` per feature set).
- Re-drive djpeg on the committed fixture and on `meta strip` of it (steps 4 above), and `crustyimg diff` of
  the CMYK fixture vs its strip (`100.0000`).

### Decisions that apply

- `DEC-003` — the container lane; this spec corrects its "strip removes everything" (ship-time amendment).
- `DEC-029` — `img-parts` does the segment work; its probe note gets a pointer.
- `DEC-015` — governs `docs/api-contract.md`, edited here.

### Constraints that apply

- `metadata-not-via-pixel-encode` — strip stays in the container lane; the fix edits the segment list only.

### Prior related work

- `SPEC-026` (shipped) — `strip` and `clean --gps`, the `0xE1..=0xEF` sweep.
- `SPEC-114` (shipped) — the C2PA manifest drop, `meta_verbs_are_byte_identical_on_unsigned_input`.
- PR #240 — see the overlap map.

### Out of scope (for this spec specifically)

- ICC (JPEG APP2, PNG `iCCP`) dropped by strip or the pixel lane: the DEC-003 entry folded into
  SPEC-150, pending a ruling.
- EXIF orientation dropped by strip, and the docs warning about it: the STAGE-063 orientation entry.
- MPF, gain maps and JPEG XT secondary data.
- `has_manifest`, `MANIFEST_DROPPED_WARNING`, `PNG_METADATA_CHUNKS`.
- SPEC-148 O-5 (the lint fixture README omits the CMYK fixture's size) and SPEC-150's wasm item stay in their
  entries. The new fixture goes in a new `tests/fixtures/metadata/` folder, so this spec doesn't edit the lint
  README.

### Size

**[S] holds.** Production code: two constants, two helpers (~12 lines), a 3-line loop swap, three doc edits.
Tests: 6 new unit tests + 2 rewritten + 1 binary test. Prototype `/usr/bin/git diff --stat` against
`origin/main`: `src/metadata/mod.rs | 285 +++…---`, `tests/metadata.rs | 25 +++`, 296 insertions, 14
deletions (before FT-4's 14 lines and the APP0 assert were added); production code is ~40 of those lines, doc
comments included. 9 tests (6 + 2 + 1). One new 452-byte fixture. The cost is in the evidence (the mutation table), not the code.

## Notes for the Implementer

- Write the tests first and run them on `main`'s code (M1 is that baseline). Each red row must fail for its
  reason, with the messages quoted under M1: FT-1/FT-2 `stripped file decodes differently: …`, FT-4 and FT-7
  the APP14 list (`left: []`), FT-9 `pixel (8,8)`. A red on a `"fixture"` precondition is the wrong reason.
- Push a checkpoint commit after the fixture and the tests, and another after the fix.
- `git commit -s` on every commit (DCO).
- No DEC edits, no version bump.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-156-strip-keeps-app14`
- **PR (if applicable):** #269 (draft)
- **All acceptance criteria met?** yes (AC-1 to AC-6; AC-6's exact sentences are for verify's docs review)
- **New decisions emitted:** none (DEC-003's amendment and DEC-029's note follow at ship)
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`; 7 files, not counting this spec file or its timeline):
  - `src/metadata/mod.rs` — two constants, `is_adobe_colour_transform`, `strip_keeps_jpeg_segment`; `strip_all`'s JPEG arm is a `retain`; `strip_all` doc and module doc; test helpers; FT-1 to FT-6 new, FT-7 and FT-8 rewritten
  - `tests/metadata.rs` — FT-9 (`meta_strip_keeps_the_colours_of_an_adobe_cmyk_jpeg`)
  - `tests/fixtures/metadata/rgb_adobe_t0_ids123.jpg` — new 452-byte fixture
  - `tests/fixtures/metadata/README.md` — new, the recipe and three hashes
  - `docs/cli-reference.md` — the `meta strip` sentence
  - `docs/api-contract.md` — the `meta strip` sentence
  - `CHANGELOG.md` — one `### Fixed` line
  - `Cargo.toml`, `Cargo.lock`, `decisions/`: no change (empty `git diff --stat origin/main...HEAD` for them).
- **Fixture:** `cjpeg` = libjpeg-turbo 3.2.0 (build 20260630). All three hashes equal the spec's: `grad16.ppm` 9ab37bf5…, `rgbRGB.jpg` 7d106aa3… (452 B), result dd3ec8ba… (452 B). Offsets 97/100/103 and 327/329/331 held `52 47 42`. djpeg (8,8) of the result: `[128,128,191]`; of `meta strip` of it: `[128,128,191]`, bytes identical to the input.
- **RED baseline** (helpers added, `strip_all` still `main`'s loop; full `cargo test --no-fail-fast`, 2 targets failed, 7 tests red):
  - FT-1: `stripped file decodes differently: 256 of 256 pixels differ, first at (0, 0): [191, 64, 32, 255] -> [93, 134, 30, 255]`
  - FT-2: `stripped file decodes differently: 256 of 256 pixels differ, first at (0, 0): [0, 0, 255, 255] -> [178, 0, 0, 255]`
  - FT-3: `left: [219, 194, 196, 218]  right: [238, 219, 194, 196, 218]` (header markers)
  - FT-4: `left: []  right: [[255, 238, 0, 14, 65, 100, 111, 98, 101, 1, 100, 0, 0, 0, 0, 2]]`
  - FT-7: `the Adobe APP14 should be kept unchanged  left: []  right: [[255, 238, 0, 14, 65, 100, 111, 98, 101, 0, 100, 0, 0, 0, 0, 1]]`
  - FT-8: `strip_all must be byte-identical on unsigned input`
  - FT-9: `pixel (8,8)  left: [93, 134, 30, 255]  right: [191, 64, 32, 255]`
  - FT-5 and FT-6 passed on `main` (it strips every APP14), as expected.
- **Behaviour sentences driven:** B1 FT-7's APP0 assert, M8 red. B2/B3 FT-1 to FT-4 and FT-9. B4 FT-3, FT-7. B5 the M1 row (FT-1, FT-2 red with a pixel mismatch). `crustyimg diff` of the CMYK fixture against its strip: `ssimulacra2: 100.0000`, bytes identical. `signed_gps.jpg` strip: exit 0, 81672 -> 81707 bytes (+35).
- **Mutations** (scratch worktree of the pushed head, own `CARGO_TARGET_DIR`, `cargo clean -p crustyimg` before each, full `--no-fail-fast` of every target including doc-tests, red set read off the run; edits are the spec's quoted ones). **Every row matches the table; no cell was wrong.**
  - M1: red FT-1, FT-2, FT-3, FT-4, FT-7, FT-8, FT-9 (1350 passed / 7 failed)
  - M2a: red FT-3 (1356/1)
  - M2b: red FT-3, `meta_strip_removes_the_jumbf_segment_by_name` (1355/2)
  - M3: red FT-5 (1356/1)
  - M3b: red FT-4 only, on every target (1356/1)
  - M4: red FT-8 (1356/1)
  - M5a: red FT-6 (1356/1)
  - M5b: red FT-1, FT-2, FT-4, FT-7, FT-9 (1352/5)
  - M6: red FT-3, FT-5, FT-6, `every_printed_fix_run_verbatim_fixes_its_finding`; the run finished, doc-tests included (1353/4)
  - M7: red FT-7 (1356/1)
  - M8: red FT-7 only, on every target (1356/1)
  - T14 (`main`'s `src/metadata/mod.rs` restored): red FT-9 only (1350/1)
  - The counts are one higher than the design's because FT-4 is in the baseline.
- **Matrix** (each leg in a fresh `CARGO_TARGET_DIR`; clippy `--all-targets -D warnings`, `fmt --check`, full test): default 1357 passed / 0 failed; `--no-default-features` 1293 / 0; `--features webp-lossy` 1361 / 0. `cargo build --target wasm32-unknown-unknown --lib` (rustup cargo): exit 0, one warning, the existing `MANIFEST_DROPPED_WARNING` unused.
- **Overlap after the last rebase** (`gh pr list --state open`): #240 touches `src/metadata/mod.rs` (as the spec says; not rebased over, still open). #268 (SPEC-155) touches `CHANGELOG.md` **and also `docs/api-contract.md` and `docs/cli-reference.md`**, which the prompt did not list: a textual conflict is possible in all three. #237 touches `CHANGELOG.md`. #229, #226: no shared file.
- **Deviations from spec:**
  - The RED commit adds the constants and helpers while `strip_all` keeps `main`'s loop, because FT-8 calls `strip_keeps_jpeg_segment`. That is the M1 shape the design drove.
  - `with_segments` uses `enumerate` over the insert offset, not a manual counter (clippy `explicit_counter_loop`).
- **Follow-up work identified:** none new.

### Punch list (verify)

- **FT-10** `strip_all_removes_an_adobe_cm_app13` (test module only; code head `a691427`). Precondition: one APP13 in the input. Asserts none after strip, and that the payload is 16 bytes (at least `ADOBE_APP14_MIN_LEN`) and starts with `Adobe`.
- **MK** (`marker == JPEG_ADOBE_MARKER` → `(marker == JPEG_ADOBE_MARKER || true)`; scratch worktree, own `CARGO_TARGET_DIR`, `cargo clean -p crustyimg`, full `--no-fail-fast`): 1357 passed / 1 failed. Red: FT-10 only, `an Adobe-prefixed segment under another marker is metadata`. Lib 603/1.
- **Unmutated head:** 1358 passed / 0 failed.
- **Clippy `--all-targets -D warnings` and `fmt --check`:** clean on the default and `--no-default-features` builds, each in a fresh `CARGO_TARGET_DIR`.
- **CI snapshot:** at `a691427`, every leg green: 3-OS build/test/clippy/fmt, lean, msrv, avif, webp-lossy, heic ×2, wasm round-trip, browser smoke, ddphotos parity, cargo-deny, DCO, front-matter, cost-capture (the release legs skip).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocked. FT-8 needs the helper, so the RED baseline needs the helpers present; the spec doesn't say so.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The overlap list names #268 only for `CHANGELOG.md`; it also touches both docs files.

3. **If you did this task again, what would you do differently?**
   — Run the mutation loop from a script from the start, and keep scratch output in the scratchpad (one run wrote to the system temp dir). No design call turned out wrong.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Design cost $10.02, 2.4x the frame ($4.16) and about 4x the build ($2.53), for an [S] spec, and it bought
   three things the build and verify then relied on: the version-byte correction (the byte after `Adobe` is the
   version's high byte, so "starts with `Adobe\0`" would still have dropped `Adobe\x01...` and left it orange),
   the reproducible fixture recipe (verify rebuilt it byte for byte, and the shipped binary reproduces its pixels
   below), and a 12-row mutation table that the build matched with no wrong cell. Most of that was not avoidable:
   the correction changed the rule. What was avoidable is the table's gap: every guard in
   `is_adobe_colour_transform` but one had a row, and the one without it, the `marker == 0xEE` clause, was found
   by verify (mutant MK, 1,357 green) and cost a $1.13 punch list. The table was built from the clauses design
   judged risky, not from the predicate's clause list. The build's overlap list named #268 only for
   `CHANGELOG.md`, though #268 also touches both docs files; it was written from memory of the PRs, not from
   `gh pr list --json files`. Shipped, on `main`'s release build: the fixture and `cmyk_16x16.jpg` strip to
   byte-identical files, and djpeg's (8,8) is unchanged for both.

2. **Does any template, constraint, or decision need updating?**
   — Three proposals, none applied. (a) "A predicate with N clauses needs a mutation row per clause, each with an
   input that passes every other clause" (FT-10's shape): AGENTS section 15-16 and HANDOFF item 12 cover where a
   row is applied, the target set it is read off, and the all-green row, but none says one row per clause; not
   covered. (b) "A red baseline that depends on new test helpers must say so": not stated in AGENTS, HANDOFF or
   `guidance/`; not covered. (c) "The overlap list comes from `gh pr list --state open --json number,title,files`,
   not from memory": SPEC-154's design prompt tells the design to run that command, but no standing rule or
   template line does, and SPEC-156's list missed #268's docs files; partly covered, as a per-prompt instruction
   only. SPEC-154's ship reflection proposed three other items; none of them is these.

3. **Is there a follow-up spec I should write now before I forget?**
   — None new. The orientation finding is already STAGE-063's "`meta strip` drops EXIF orientation, and no doc
   says so" entry, and the transform-5 decode refusal is in `guidance/ideas.yaml`
   (`decode-unknown-adobe-transform`); the evidence names no gap those two miss.
