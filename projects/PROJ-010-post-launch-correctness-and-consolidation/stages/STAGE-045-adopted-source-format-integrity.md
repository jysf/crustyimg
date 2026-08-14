---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-045                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-11
shipped_at: null

value_contribution:
  advances: >
    PROJ-010's thesis is that a shipped verb should do what its name says on an ordinary
    input. STAGE-043 found the pinned path carrying a defect the decide path had already
    fixed. This stage is the mirror image: the DECIDE path carries a defect of its own,
    on the three input families PROJ-009 added. `optimize` on an SVG, a HEIC, or a RAW
    file can ship the source container verbatim — vector XML, a HEIF container, a whole
    camera-raw file — while reporting it as a PNG or a JPEG it never produced.
  delivers:
    - "`optimize`/`web`/`build` never emit bytes that are not the format they report"
    - "An SVG piped on stdin is never written as a `.jpg` containing XML"
    - "The auto-decision reports the container the user actually supplied, not an adopted stand-in"
  explicitly_does_not:
    - "Add SVG, HEIC, or RAW as OUTPUT formats. They stay input-only (DEC-054/DEC-056)."
    - "Change the pinned path — that is STAGE-043/SPEC-113, in flight."
    - "Revisit the never-bigger guarantee for sources that ARE a valid output. Untouched."
---

# STAGE-045: adopted-source-format integrity on the decide path

## What This Stage Is

`Image::source_format()` is not always the format of the bytes on disk. For three input
families it is an **adopted label** — a stand-in the decoder picks because the container has
no `image::ImageFormat` variant at all:

| Input | Real container | `source_format()` reports | Set at |
|---|---|---|---|
| SVG | XML text | `Png` | `src/image/svg.rs:17-18` |
| HEIC | ISOBMFF/HEIF | `Png` | `src/image/mod.rs:449-452` |
| RAW (`.nef`/`.cr2`/`.dng`/…) | TIFF-family RAW container | `Jpeg` | `src/image/raw.rs:233,267,273` |

The auto-decide path reads that label as though it named the bytes on disk, in two places:
it decides a **passthrough** is a valid output (`optimize_decide_one`'s `OptimizeOutput::Passthrough`
ships `read_raw_bytes` verbatim), and it **reports** the label as the source format in both the
human summary and the `--json` audit record.

When this stage ships, an output that claims to be a PNG is a PNG, and the report names the
container the user actually handed over.

## Why Now

Driven on this repo at `08b367d`, with committed fixtures — no contrived input required.

**The clearest spelling first.** `optimize` reading an SVG from stdin:

```
$ cat tests/fixtures/svg/rect_text_40x30.svg | crustyimg optimize - --out-dir out/
stdin: kept png (336 B, already optimal)
$ file out/stdin.jpg
out/stdin.jpg: SVG Scalable Vector Graphics image
```

**A file named `.jpg`, containing XML, described as a PNG.** All three labels disagree with the
bytes, exit 0, and the only diagnostic printed is the false one.

Named-path spellings of the same defect:

```
$ crustyimg optimize tests/fixtures/svg/rect_text_40x30.svg --out-dir out/ --explain
optimize: png → png (336 → 336 B, 0% smaller)
  reason: kept source — no candidate beat it
$ file out/rect_text_40x30.svg
… SVG Scalable Vector Graphics image          # the extension is honest; the report is not

$ crustyimg optimize tests/fixtures/raw/oversize_preview.dng --out-dir out/ --explain
optimize: jpeg → jpeg (643 → 643 B, 0% smaller)
$ file out/oversize_preview.dng
… TIFF image data, little-endian              # the whole RAW container, reported as a JPEG

$ crustyimg web tests/fixtures/svg/rect_text_40x30.svg --out-dir out/
… kept png (336 B, already optimal)           # an assertion about a PNG that was never produced

$ crustyimg optimize …/rect_text_40x30.svg | file -
… SVG Scalable Vector Graphics image          # raw XML on stdout, where a raster is expected
```

And the machine channel says it too — `--json` on the DNG:

```json
{"schema":"crustyimg.optimize.explain/v1","source_format":"jpeg","source_bytes":643,
 "winner":null,"out_bytes":643,"savings_percent":0}
```

### Why it happens

`decide::pick_winner` returns `None` when no candidate beats the source on bytes, and `None`
means *"the source was already the best answer — ship it unchanged."* That reasoning holds only
when the source file **is** a valid output. For these three families it is not, and the
condition is easy to hit rather than exotic: the comparison pits a **raster re-encode** against
a **vector or container** file, which is not a comparison at all. A 336 B SVG beats every raster
of itself; a 643 B DNG declaring a 62 MP preview beats a 59 MB WebP of that preview.

`optimize_decide_one` already knows that a passthrough can be invalid — `pipeline_altered`
covers *metadata stripped / orientation baked / resized* and correctly ships the smallest
correct re-encode instead. **This is a fourth reason in exactly the same category, and it is
the one nobody enumerated.**

### Why it is stage-worthy rather than a footnote

- It is **live on 0.7.0** and reachable through four verbs — `optimize`, `web`,
  `apply --recipe web`, and `build` (via `encode_one_optimize_decided`) — because all four
  share the one seam.
- The three affected families are **PROJ-009's entire deliverable**. "Default AVIF+SVG+RAW
  decode" shipped as input reach; on the flagship verb's default invocation, that reach
  currently produces a mislabeled file.
- It is the **same unenumerated-cell pattern** STAGE-042 exists to instrument, on a new axis:
  every input-format test file (`tests/input_svg.rs`, `tests/input_raw.rs`, `tests/input_heic.rs`)
  drives `optimize` with a **pinned** `-o out.png`/`-o out.webp` and **none** drives it with no
  `-o`. The decide path is untested for all three families. Verified by reading all three files,
  2026-08-11.
- STAGE-043's scope note said the decide path "already has the never-bigger guarantee and
  reports honestly." The first half is true; **the second half is what this stage falsifies.**
  Corrected in that stage's Design Notes rather than silently.

## Success Criteria

- **No output claims a format its bytes are not.** Driven per family, asserting on the written
  bytes with a decoder/sniff the tool did not produce — not on the extension and not on the
  summary line.
- **The stdin spelling is covered**, since it is the one where the extension lies too
  (`metadata_output_ext(input, &[])` sniffs an empty slice and defaults to `jpg`).
- **A legitimate passthrough still passes through, byte-identical.** An already-optimal PNG/JPEG/
  WebP/AVIF source keeps its exact bytes. Whatever predicate decides "the source is shippable"
  must be proven not to false-negative on **AVIF**, whose brand handling differs between
  `image::guess_format` and this crate's own `sniff::is_avif` (see the spec — this is the trap).
- **The report names the real container** on both the human and `--json` channels.
- **A negative control per family**: revert the guard, watch that family's test go red.
- Full matrix clean **including the `heic` CI job**, and the CI legs read individually.

## Scope

### In scope
- The passthrough decision in `src/cli/optimize.rs`'s `optimize_decide_one` — both exits: the
  `pick_winner → None` arm and the earlier degenerate-analysis early return at `:1006-1015`.
- Whatever seam establishes "the source container really is `source_format`" — see the spec's
  settled design call.
- The report/`--json` source-format label for an adopted-label input.
- Decide-path (`no -o`) tests in `tests/input_svg.rs`, `tests/input_raw.rs`, `tests/input_heic.rs`,
  plus any fixture needed to reproduce (see the spec: a fixture that does not reproduce is the
  whole risk).

### Explicitly out of scope
- **Adding SVG/HEIC/RAW as output formats.** They remain input-only.
- **The pinned path** (STAGE-043 / SPEC-113, in flight in a separate worktree). The two changes
  meet at `pipeline_altered_source`; coordinate, do not merge the specs.
- **The never-bigger guarantee for sources that are valid outputs.** Untouched.
- **RAW's `source_bytes` semantics beyond this defect.** That the never-bigger comparison uses
  the whole container rather than the extracted preview is noted in the spec as the underlying
  reason the branch is reachable; re-basing the comparison is not attempted here.

## Spec Backlog

- [x] **SPEC-115** (framed 2026-08-11, shipped 2026-08-14, PR #156) — **`optimize` never passes
  through bytes that are not the format it reports.** The guard, the honest label, and decide-path
  tests for all three families. Complexity **M** — the code change is small and the design call is
  settled, but it carries a fixture-construction problem (a fixture that does not reproduce is a
  green test over a live defect) and an AVIF false-negative trap that would turn a correct
  passthrough into a regression. *Both hazards materialised: two candidate fixtures failed to reach
  the branch before a dithered `LosslessFlat` one worked, and the AVIF trap turned out to be
  unreachable because `avif-parse` rejects a non-`avif` major brand before any guard runs.*

- [ ] **Pin `build` and `apply --recipe web` against the adopted-format defect** (filed at
  SPEC-115's verify, 2026-08-14). Both delegate unconditionally to the fixed `optimize_decide_one`,
  and verify **drove both green on the real binary** — `apply --recipe web` on the SVG fixture
  produces a real WebP with the new fourth reason in its note, and `build` writes
  `rect_text_40x30.webp`. Neither is test-pinned, so a refactor could re-break them silently while
  the SVG/RAW/HEIC tests stay green. Small: two integration tests against existing fixtures, no new
  behaviour. Not a defect today — a missing regression pin on behaviour that currently works.

**Count:** 1 shipped (SPEC-115) / 0 active / 1 pending (pin `build` + `apply --recipe web`, not framed)

## Design Notes

- **Sequence it with STAGE-043, not after it.** Both stages change the same two files and both
  answer the same question — *when are the raw source bytes a valid output?* SPEC-113 introduced
  `pipeline_altered_source` as the shared answer; this stage adds the second half of it. Landing
  them far apart invites a second, divergent copy of the judgement.
- **Ship in the same release as STAGE-043**, for the same reason STAGE-043 gives: the launch post
  names `optimize`, and "it wrote my SVG into a .jpg" is a top comment rather than a bug report.
- **The cell that escaped is (input family × mode), not (entry point × mode).** STAGE-042's
  matrix design is already gaining a decide-vs-pinned axis from STAGE-043. This stage supplies
  the third: **input family** — because the defect is invisible on the raster formats every other
  test uses, and every one of the three affected families was tested only in the mode that hides it.

## Dependencies

### Depends on
- PROJ-009 (shipped) — the three adopted-label decoders whose inputs this affects.
- STAGE-043 / SPEC-113 — introduces `pipeline_altered_source`, the helper this extends.

### Enables
- STAGE-042's matrix gains its third axis (input family).

## Stage-Level Reflection

*Filled in when status moves to shipped.*
