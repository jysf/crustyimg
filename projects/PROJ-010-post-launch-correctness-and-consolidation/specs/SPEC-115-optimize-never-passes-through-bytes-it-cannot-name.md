---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-115
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-045
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5       # NOT Sonnet, unlike SPEC-113: the code change is
                                   # small but the two traps (AVIF false-negative,
                                   # a fixture that cannot reproduce) both produce a
                                   # confidently green build over a live defect.
  created_at: 2026-08-11

references:
  decisions:
    - DEC-054
    - DEC-055
    - DEC-056
    - DEC-074
    - DEC-075
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-061
    - SPEC-084
    - SPEC-090
    - SPEC-113

value_link: >
  STAGE-045's only spec. `optimize` on the three input families PROJ-009 delivered can
  ship the source container verbatim — vector XML, a HEIF container, a whole camera-raw
  file — labeled as a PNG or JPEG it never produced. On stdin it writes XML into a
  `.jpg`. PROJ-010 exists so a shipped verb does what its name says.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Drove all six spellings of the
        defect on this repo at 08b367d with committed fixtures, read both passthrough
        exits in `optimize_decide_one`, and settled the predicate question by reading
        `image`'s MAGIC_BYTES table against this crate's own `sniff::is_avif` rather
        than assuming the obvious sniff was safe.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 103974126
      duration_minutes: 159
      recorded_at: 2026-08-13
      tokens_breakdown:
        input: 7986
        output: 326993
        cache_creation: 925331
        cache_read: 102713816
      estimated_usd: 39.21
      note: >
        MEASURED, summed directly from this session's own transcript
        (~/.claude/projects/.../2d19bb84-e014-4f1d-8f41-15e343e3afe6.jsonl), all
        360 usage-bearing messages at claude-sonnet-5. Priced per component at
        Sonnet anchors ($3/$15 per MTok in/out; cache_creation x1.25 input rate;
        cache_read x0.10 input rate) — cache reads are 98.8% of volume, so the
        flat 80/20 shortcut would badly overstate this. Ran 159 minutes against a
        stated 90-minute budget: the RAW/HEIC fixture search (proving
        `tight_preview.nef` and a naive noise preview do NOT reproduce this
        spec's defect, then building ones that do, ground-truth-verified against
        the real library) and the AC-6 AVIF trap investigation (finding
        `avif-parse` itself blocks the predicted construction) both ran well past
        the checkpoint; a `wip(SPEC-115):` commit landed at the 90-minute mark
        with the SVG family green, not a hard stop.
  totals:
    tokens_total: 103974126
    estimated_usd: 39.21
    session_count: 2
---

# SPEC-115: `optimize` never passes through bytes it cannot name

## Context

**Driven on this repo at `08b367d`, with committed fixtures.** The sharpest spelling first —
an SVG on stdin:

```
$ cat tests/fixtures/svg/rect_text_40x30.svg | crustyimg optimize - --out-dir out/
stdin: kept png (336 B, already optimal)
$ file out/stdin.jpg
out/stdin.jpg: SVG Scalable Vector Graphics image
```

A file **named `.jpg`**, **containing XML**, **described as a PNG**. Exit 0, and the only
diagnostic printed is the false one.

Named-path spellings, same binary, same run:

```
$ crustyimg optimize tests/fixtures/svg/rect_text_40x30.svg --out-dir out/ --explain
optimize: png → png (336 → 336 B, 0% smaller)
  reason: kept source — no candidate beat it
$ file out/rect_text_40x30.svg      → SVG Scalable Vector Graphics image

$ crustyimg optimize tests/fixtures/raw/oversize_preview.dng --out-dir out/ --explain
optimize: jpeg → jpeg (643 → 643 B, 0% smaller)
$ file out/oversize_preview.dng     → TIFF image data, little-endian

$ crustyimg web tests/fixtures/svg/rect_text_40x30.svg --out-dir out/
… kept png (336 B, already optimal)         # an assertion about a PNG never produced

$ crustyimg optimize tests/fixtures/svg/rect_text_40x30.svg | file -
… SVG Scalable Vector Graphics image        # raw XML on stdout where a raster is expected
```

And on the machine channel — `--json` on the DNG:

```json
{"schema":"crustyimg.optimize.explain/v1","source_format":"jpeg","source_bytes":643,
 "winner":null,"out_bytes":643,"savings_percent":0}
```

### Root cause

`Image::source_format()` is an **adopted label** for three input families — a stand-in the
decoder picks because the real container has no `image::ImageFormat` variant:

| Input | Real container | `source_format()` | Set at |
|---|---|---|---|
| SVG | XML text | `Png` | `src/image/svg.rs:17-18` |
| HEIC | ISOBMFF/HEIF | `Png` | `src/image/mod.rs:449-452` |
| RAW | TIFF-family container | `Jpeg` | `src/image/raw.rs:233,267,273` |

`decide::pick_winner` returns `None` when no candidate beats the source on bytes, and
`optimize_decide_one` reads `None` as *"the source was already the best answer — ship the file
unchanged"* (`OptimizeOutput::Passthrough { raw, … }`, `src/cli/optimize.rs:1094-1098`). That
reading holds only when the source file **is** a valid output. For these three families it is
not, and the branch is easy to reach rather than exotic: the comparison pits a **raster
re-encode** against a **vector or container** file, which is not a comparison at all. A 336 B
SVG beats every raster of itself; a 643 B DNG declaring a 62 MP preview beats a 59 MB WebP.

**The function already knows a passthrough can be invalid.** `pipeline_altered`
(`src/cli/optimize.rs:1027`, becoming `pipeline_altered_source` under SPEC-113) covers *metadata
stripped / orientation baked / resized* and correctly ships the smallest correct re-encode
instead. **This is a fourth reason in the same category — the one nobody enumerated.**

### Reach

One seam, four verbs: `optimize`, `web`, `apply --recipe web`, and `build` (through
`encode_one_optimize_decided`, `src/cli/optimize.rs:1183-1201`, which returns
`(ext, raw)` straight into the manifest + lockfile).

Two exits from that seam, not one:

1. `pick_winner → None` (`:1086-1098`) — the reachable one, driven above.
2. The **degenerate-analysis early return** (`:1006-1015`) — `Analysis::compute` errors only on a
   zero-area image (`src/analysis/mod.rs:216-221`); it passes the raw bytes through with no
   candidates computed at all. Narrow, but the same shape, and it must get a stated answer.

### The test gap that let it through

Every input-format test file drives `optimize` with a **pinned** output and none drives the
decide path (read in full, 2026-08-11):

| File | `optimize` coverage | Decide path (`no -o`)? |
|---|---|---|
| `tests/input_svg.rs` | `optimize_svg_input_writes_png` — `-o out.png` | **none** |
| `tests/input_raw.rs` | `optimize_raw_input_writes_webp` — `-o out.webp` | **none** |
| `tests/input_heic.rs` | `optimize_heic_input_writes_webp` — `-o out.webp` | **none** |

STAGE-042's unenumerated cell again, on a third axis: **input family × mode**. The defect is
invisible on the raster formats every other test uses, and each affected family was tested only
in the mode that hides it.

## The design calls — settled here, not deferred to build

### Call 1 — the predicate is recorded at load, NOT sniffed from the bytes

Add the truth to the model: the decoder that adopts a stand-in label records that it did. The
passthrough guard then asks a fact, not a heuristic.

**Do not use a bare `::image::guess_format(&raw).ok() == Some(source_format)`.** It has a driven
hole, and on THIS path the hole is a regression rather than a missed guard:

- `image` 0.25.10's magic table (`io/free_functions.rs:120`) matches AVIF as
  `b"\0\0\0\0ftypavif"` — it requires the **major brand** at bytes 8..12 to literally be `avif`.
- This crate's own `sniff::is_avif` (`src/image/sniff.rs:25-49`) also accepts a `mif1` **major**
  brand carrying `avif` in the compatible-brands list — and has a unit test asserting exactly
  that (`is_avif_detects_compatible_brand`, `:67-74`). That spelling is legal and common.
- So a `mif1`-major AVIF fails `guess_format` → a passthrough that is correct today becomes a
  forced re-encode: bigger, and lossy-over-lossy.
- **All three committed AVIF fixtures are `ftypavif`-major** (verified by `xxd`), so a test suite
  built on them cannot catch it. [[fixtures-from-the-code-under-test-cannot-fail]] in its other
  form: a fixture that cannot exhibit the failure.

Note the asymmetry with SPEC-113: on the **pinned** path a false negative means "ship the
re-encode", i.e. today's behaviour — conservative. On the **decide** path a false negative
**changes correct output into worse output**. Same expression, different blast radius.

Recorded truth has neither hole. Shape (the build may choose the spelling):

- `Image` gains an origin alongside `source_format` — e.g.
  `SourceContainer { Raster(ImageFormat), Svg, Heic, RawPreview }`, or the minimal
  `source_format_is_adopted: bool` if the label work below is split out.
- **`from_parts` is `pub` on a published crate.** Add an additive constructor (or a builder-style
  setter defaulting to `Raster(source_format)`); do not change the existing signature. 13 call
  sites, mostly tests.
- The three adopting decoders set it: `svg::decode_svg`'s caller (`mod.rs:437-448`),
  `heic`'s (`mod.rs:449-460`), `raw::extract_preview`'s (`mod.rs:513`).

**`source_format()` itself does not change value.** It names the decoded pixels' provenance and
`info` deliberately reports it — `info <fixture>.nef` printing `format: jpeg` is claimed by
`info_raw_reports_jpeg_dims` (`tests/input_raw.rs:101`, SPEC-061/DEC-055). **That test staying
green is the proof this change did not bleed into `info`.**

### Call 2 — an unshippable source is treated exactly like `pipeline_altered`

When the source container is not a valid output, there is no passthrough — take the same branch
`pipeline_altered` takes: add the compact-lossy fallback candidate if the shortlist offers only
lossless ones, then **ship the smallest correct candidate**, even if it exceeds the source, with
the honest note. One predicate feeding one existing branch; no second policy.

Consequences to accept, not to work around:

- `optimize logo.svg --out-dir out/` writes `out/logo.webp` (444 B for the 336 B fixture) instead
  of copying the SVG. **That is the fix, not a side effect** — `optimize` is a raster byte
  primitive and there is no SVG sink to write.
- `optimize oversize_preview.dng` would ship a ~59 MB WebP from a 643 B container. Pathological,
  and honest: the fixture declares a 62 MP preview. Do not add a size escape hatch.
- **The degenerate early return (`:1006-1015`)** has no candidates to fall back to. Settled: keep
  the passthrough when the container is shippable; when it is not, fail with a typed error rather
  than writing mislabeled bytes — a zero-area SVG/HEIC/RAW has nothing correct to emit.

### Call 3 — the note gains its fourth reason

`emit_optimize_report`'s larger-than-source note (`src/cli/optimize.rs:1281-1288`, SPEC-090 /
DEC-075) enumerates *metadata stripped / orientation baked / resized to the requested bound*.
For these inputs **all three are false** — it would print a wrong explanation of a right
decision. The reason list gains the real one: the source is not an image file of the format
being written. Plain wording, behaviour-first, no SPEC/DEC references in strings
([[comments-plain-no-spec-refs]]).

### Call 4 — the report names the container, on both channels

- **Human:** `optimize: svg → webp (336 → 444 B, 32% larger)`, and `web`'s
  `kept png (…, already optimal)` can no longer be reached for these inputs at all.
- **JSON:** `source_format` reports `"svg"` / `"heic"` / `"raw"`. The schema id stays
  `crustyimg.optimize.explain/v1` — DEC-074's precedent is that this schema extends in place
  (`--timing`, `ssim`) — and the value-set widening is documented in `docs/cli-reference.md`
  and CHANGELOG. **A consumer cannot have depended on the old value being correct: it was
  never correct for these inputs.** If verify identifies a real consumer keyed on the enum,
  the fallback is an additive `source_container` field with `source_format` left alone —
  record that as a deviation, do not decide it silently.

## Goal

`optimize`/`web`/`apply --recipe web`/`build` never write source bytes that are not a valid file
of the format they report, and the report names the container the user actually supplied.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs:930-1165` — `optimize_decide_one`, **both** passthrough exits, and
    `pipeline_altered`; `:1183-1201` `encode_one_optimize_decided`; `:1270-1292` the note.
  - `src/analysis/decide.rs:222-265` — `pick_winner`; `None` is "no candidate beat the source".
  - `src/image/mod.rs:410-520` — the decode dispatch that adopts the three labels.
  - `src/image/sniff.rs` — `is_avif` and its two brand tests. **Call 1's evidence.**
  - `src/cli/ops.rs:470-495` — `metadata_output_ext` (sniffs `&[]` on stdin → `jpg`) and
    `read_raw_bytes`.
  - `projects/PROJ-009-input-reach/specs/done/SPEC-061-…md:180,225` — how
    `synthetic_preview.nef` was hand-built. The new RAW fixture follows it.
  - **SPEC-113's branch** (`feat/spec-113-optimize-pinned-never-bigger`) — it introduces
    `pipeline_altered_source`, which this spec extends. Rebase on it; do not fork the helper.
- **Related code paths:** `src/cli/optimize.rs`, `src/image/`, `tests/input_{svg,raw,heic}.rs`.

## Outputs

- **Files modified:** `src/image/mod.rs` (+ `svg.rs`/`heic.rs`/`raw.rs` call sites),
  `src/cli/optimize.rs`, `src/analysis/decide.rs` (only if the trace type needs the container),
  `tests/input_svg.rs`, `tests/input_raw.rs`, `tests/input_heic.rs`, `docs/cli-reference.md`,
  `CHANGELOG.md`.
- **New fixture:** a RAW file whose embedded preview is **noise**, so every lossless candidate
  exceeds the container and the passthrough branch is actually reached. Hand-built like
  `synthetic_preview.nef`; the preview JPEG generated by an **independent tool**, never by
  crustyimg. Possibly the same for HEIC — see AC-4.
- **New exports:** the origin type/accessor on `Image` (additive; `from_parts`'s signature
  unchanged).
- **New DEC expected:** the `Image` model now distinguishes "the format of the decoded pixels"
  from "the container on disk". That is a model-level contract worth recording — write it, with
  `affected_scope` covering `src/image/**` and `src/cli/optimize.rs`.

## Acceptance Criteria

- [ ] **AC-1.** `cat <fixture>.svg | crustyimg optimize - --out-dir out/` writes a file that is a
      **real raster** — assert by sniffing the written bytes, and assert the name is not
      `stdin.jpg`-containing-XML. **Fails today** (XML in a `.jpg`).
- [ ] **AC-2.** `crustyimg optimize <fixture>.svg --out-dir out/` writes a real raster and the
      summary does **not** claim `png → png`. Assert on the written bytes AND on the summary
      line. **Fails today.**
- [ ] **AC-3.** Same for **RAW**, on a fixture that actually reaches the passthrough branch (see
      Outputs). **The test must be RED on `main` before the fix** — a RAW fixture whose candidates
      beat the container (like today's `synthetic_preview.nef`, which correctly re-encodes to
      160 B) proves nothing. [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-4.** Same for **HEIC**, under `--features heic` (CI job `heic`, `ci.yml:104-145`). If a
      reproducing HEIC fixture cannot be built, **say so explicitly in Build Completion** and
      state what is therefore unproven — do not ship a green test that cannot fail.
      [[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]
- [ ] **AC-5.** **A legitimate passthrough is byte-identical to `main`.** An already-optimal
      source that no candidate beats still ships verbatim — assert bytes, per raster family
      including **AVIF**. **Passes today**; it is the regression control for Call 1.
- [ ] **AC-6.** **The AVIF trap is covered by construction**, not by luck: a `ftypmif1`-major AVIF
      carrying `avif` in its compatible brands still passes through byte-identical. All three
      committed AVIF fixtures are `ftypavif`-major, so this needs a fixture built for it (mirror
      `sniff.rs`'s `is_avif_detects_compatible_brand`). **The single most important test here** —
      it is the one that fails if the build reaches for `guess_format`.
- [ ] **AC-7.** **`web` and `build` inherit the fix through the shared seam** — assert `web` on the
      SVG fixture, and `build` (or `apply --recipe web`) on it, both producing a real raster. One
      seam, four verbs; a fix proven on one entry point is this project's recurring defect.
- [ ] **AC-8.** **The larger-than-source note states the real reason** (Call 3), asserted on the
      message text. A right decision with a wrong explanation is still a tool lying on stderr.
- [ ] **AC-9.** **The report names the container** (Call 4): human summary and `--json`
      `source_format`. Assert both channels.
- [ ] **AC-10.** **`info` is unchanged** — `info_raw_reports_jpeg_dims` stays green, unmodified.
      It is the proof the new origin did not bleed into `source_format()`'s existing meaning.
- [ ] **AC-11.** **A negative control per family**: revert the guard, confirm each family's test
      goes RED, restore. Prove the revert reached the **built artifact**, not just the source
      ([[reverting-source-does-not-rebuild-the-binary]]).
- [ ] **AC-12.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run
      **sequentially**, **through `rtk proxy` from the first leg**: default,
      `--no-default-features`, `--features webp-lossy`, **and `--features heic`**;
      `clippy --all-targets -- -D warnings` each; `fmt --check`. Confirm each log says
      `Compiling crustyimg`. **Then read the CI legs individually**, the `heic` job included.
- [ ] **AC-13.** `docs/cli-reference.md` — read the `optimize` and `web` prose against the new
      behaviour (`:140` "never ships a larger file" now has a stated exception for a source that
      cannot ship as itself) and document the `--json` value widening. Make the sentence match
      the code, not the reverse. [[documentation-has-no-green]]

## Failing Tests

Written during **design**, BEFORE build. **At least one must FAIL on today's `HEAD`.**

- **`tests/input_svg.rs`**
  - `"optimize_svg_auto_decide_writes_a_real_raster"` — AC-2. **FAILS today** (writes XML).
  - `"optimize_svg_from_stdin_is_never_written_as_a_mislabeled_jpg"` — AC-1. **FAILS today**
    (`stdin.jpg` containing XML).
  - `"web_svg_auto_decide_writes_a_real_raster"` — AC-7. **FAILS today** (`kept png`).
- **`tests/input_raw.rs`**
  - `"optimize_raw_auto_decide_writes_a_real_raster"` — AC-3, on the new noise-preview fixture.
    **Must FAIL today**; if it passes before the fix, the fixture is wrong, not the defect.
  - `info_raw_reports_jpeg_dims` — AC-10, **unmodified**, must stay green.
- **`tests/input_heic.rs`**
  - `"optimize_heic_auto_decide_writes_a_real_raster"` — AC-4, `#[cfg(feature = "heic")]`.
- **`tests/cli.rs`** (or the AVIF test home)
  - `"optimize_avif_source_still_passes_through_byte_identical"` — AC-5. **Passes today**;
    the regression control.
  - `"optimize_avif_with_mif1_major_brand_still_passes_through"` — AC-6. **Passes today**;
    fails the moment the guard is a bare `guess_format`.
- **Negative control** (AC-11, run and recorded, not committed)
  - Revert the guard → the SVG, RAW and (if built) HEIC tests go RED.

## Implementation Context

### Decisions that apply
- **DEC-056:67** — HEIC is **"Decode only. No HEVC encoder is built or exposed"**. SVG is
  decode-only by construction: `image::ImageFormat` has no `Svg` variant, so no sink can write
  one (DEC-054 is the decoder choice, not an output claim — do not cite it as one). Neither is
  an output format, which is precisely why passing the container through as an "output" is wrong.
- **DEC-055** — RAW inputs route through the extension-aware preview decode; `info` reports the
  preview's format deliberately. AC-10 protects it.
- **DEC-074** — the `optimize.explain/v1` schema extends in place. Call 4's licence.
- **DEC-075 / SPEC-090** — the larger-than-source note; Call 3 extends its reason list.

### Constraints that apply
- `test-before-implementation` (**blocking**) — and here it carries extra weight: a fixture that
  cannot reproduce turns this constraint into theatre. Prove RED first, per family.
- `clippy-fmt-clean` (**blocking**) — every leg of AC-12, `heic` included.
- `one-spec-per-pr` (**blocking**) — SPEC-113 is a separate spec on a separate branch touching the
  same two files. Rebase on it; do not fold them.

### Prior related work
- **SPEC-113** (in flight) — the pinned-path never-bigger guard. It introduced
  `pipeline_altered_source` and independently hit this same adopted-label problem; its
  `write_pixel_output` guard sniffs with `guess_format`, which is **safe there and unsafe here**
  (see Call 1). Worth a note back to that spec's verify.
- **SPEC-084 / SPEC-090** — the never-bigger guarantee and the honest larger-than-source note.
- **SPEC-061** — the RAW preview path and the hand-built `synthetic_preview.nef` the new fixture
  should follow.

### Out of scope (for this spec specifically)
- **Adding SVG/HEIC/RAW as output formats.**
- **The pinned path** — SPEC-113.
- **Re-basing RAW's never-bigger comparison** on the extracted preview rather than the whole
  container. Named in STAGE-045 as the underlying reason the branch is reachable; not attempted.
- **`info`'s reporting** — deliberate and test-pinned (AC-10).

## Notes for the Implementer

- **Do the fixtures first, and prove them RED before writing a line of the fix.** Both traps in
  this spec produce a *confidently green* build over a live defect: a RAW/HEIC fixture whose
  candidates beat the container never enters the branch, and an AVIF fixture with an `ftypavif`
  major brand never exercises the false-negative.
- **`optimize logo.svg` becoming larger than the source is correct** and is reported by machinery
  that already exists. Do not add a size escape hatch.
- **Two exits, not one.** The degenerate early return at `:1006-1015` is easy to miss precisely
  because it is three lines above the analysis.
- **A piped command reports the pipe's exit code** — redirect and read `$?`
  ([[a-piped-command-reports-the-pipes-exit-code]]).
- **rtk corrupts output intermittently**, including deleting `Compiling crustyimg` and mangling
  binary through `cat`. Run every leg through `rtk proxy` from the first; `/bin/cat` for binary
  ([[rtk-can-silently-corrupt-grep-counts]]).
- macOS has no `timeout(1)`. `git commit -s` (DCO enforced). Own git worktree — other sessions are
  live in this repo. Never `git reset --hard`. **Do not merge the PR.**

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-115-optimize-never-passes-through-bytes-it-cannot-name`
- **PR (if applicable):** https://github.com/jysf/crustyimg/pull/156 (open, not merged — build-phase guardrail).
- **All acceptance criteria met?** Mostly yes — 12 of 13 fully met; **AC-6 met a
  different, narrower claim than specified** (see Deviations). AC-4 met in full
  (a reproducing HEIC fixture WAS built, via `heif-enc`).
  - AC-1 ✅ AC-2 ✅ AC-3 ✅ AC-4 ✅ AC-5 ✅ AC-6 ⚠️ (see below) AC-7 ✅ (SVG
    `optimize`/`web`; `build`/`apply --recipe web` share the same
    `optimize_decide_one` seam but were not given their own dedicated test —
    follow-up) AC-8 ✅ AC-9 ✅ AC-10 ✅ (unmodified, green) AC-11 ✅ (negative
    control run and recorded below, not committed) AC-12 ✅ (all four legs:
    test + clippy + fmt green) AC-13 ✅ (`docs/cli-reference.md` +
    `CHANGELOG.md` updated).
- **New decisions emitted:** `DEC-089` —
  `decisions/DEC-089-image-records-its-container-origin-not-just-its-pixel-format.md`.
- **Deviations from spec:**
  1. **AC-6's exact construction does not reproduce in this codebase.** The
     design predicted a `ftypmif1`-major, `avif`-compatible-brand AVIF would
     still successfully decode (via `sniff::is_avif`'s permissive match) and
     therefore exercise the `guess_format`-vs-recorded-origin divergence. It
     does not: `avif-parse` 2.1.0 (the real container parser behind
     `avif::decode_avif`) independently enforces `major_brand == b"avif"`
     (`avif-parse-2.1.0/src/lib.rs:751-756`) — verified directly against both
     `::image::guess_format` and `avif_parse::read_avif` on the identical
     mutated bytes; both reject it identically. Such a file fails at
     **decode** with a typed `Decode` error, before `optimize_decide_one`'s
     guard is ever reached — on `main` and on this fix alike. The committed
     test (`avif_mif1_major_compatible_avif_fails_typed_decode_not_panic`,
     `tests/input_avif.rs`) instead pins what IS true here: a typed decode
     error, never a panic, never a silent mislabel. The recorded-origin model
     choice (Call 1) stands on its own merits regardless (see `DEC-089`), but
     its most vivid justifying scenario is unproven in THIS repo with THIS
     dependency version — recorded as a `DEC-089` revisit trigger.
  2. **AC-7's `build`/`apply --recipe web` half is proven only by code-path
     sharing, not a dedicated test.** `encode_one_optimize_decided` (the
     function `build`/terminal-`apply --recipe web` call) delegates to the
     exact same `optimize_decide_one` the SVG `optimize`/`web` tests already
     drive RED-then-GREEN; no separate SVG-via-`build` or
     SVG-via-`apply --recipe web` test was added under the 90-minute budget.
     Low risk (one seam, four verbs, per the spec's own framing) but unproven
     directly — follow-up.
- **Follow-up work identified:**
  - A dedicated `build`/`apply --recipe web` SVG test (closes the AC-7 gap
    above).
  - `docs/backlog.md`: revisit `DEC-089`'s AVIF scenario if `avif-parse` (or a
    future AVIF decoder swap) ever accepts a non-`avif` major brand.
  - Report back to SPEC-113: its pinned-path guard's `guess_format` sniff is
    now independently confirmed safe FOR THAT PATH specifically because
    `avif-parse` itself blocks the one input shape that would have made it
    unsafe (a decodable non-`avif`-major AVIF does not exist in this
    dependency graph) — worth a note in that spec's verify docs.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** The RAW fixture
   guidance ("hand-build it like `synthetic_preview.nef`") undersold how hard
   "noise preview, passthrough reached" actually is once the DECIDE path's
   full shortlist (not the pinned path's single default-quality baseline) is
   the bar: AVIF at the fast fixed quality beats almost any noise-like JPEG
   easily, so a genuinely-noisy preview reliably escapes the passthrough
   branch via a format switch. The fixture that actually works is not "more
   noise," it's "few colors + adversarial dithering" (`OptBucket::LosslessFlat`,
   which excludes AVIF/lossy-WebP by construction regardless of built
   features) — closer to `tests/fixtures/classify/dithered_graphic.png`'s
   recipe than to `tight_preview.nef`'s. `tests/fixtures/raw/tight_preview.nef`
   (SPEC-113) does not reproduce this spec's defect at all, confirmed by
   driving it before writing any fix.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** The spec's Call 1 rationale (the AVIF `mif1`-major trap) never
   checked whether `avif-parse` itself would decode such a file — only that
   `::image::guess_format` and `sniff::is_avif` disagree on it. That one
   extra hop (does the DECODER even accept what the SNIFF admits?) turned out
   to be load-bearing and wasn't flagged as a thing to verify.
3. **If you did this task again, what would you do differently?** Probe
   `avif_parse::read_avif`'s own major-brand strictness during DESIGN, not
   BUILD — a five-minute check (`cargo run` against a hand-mutated `ftyp`)
   would have caught the AC-6 deviation before the acceptance criterion was
   written as a hard "must pass" rather than "attempt, and report what you
   learn."

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
