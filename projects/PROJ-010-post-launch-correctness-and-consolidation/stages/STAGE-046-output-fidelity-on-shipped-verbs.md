---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-046
  status: proposed
  priority: critical                # live defects on shipped verbs, incl. the flagship
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-15
shipped_at: null

value_contribution:
  advances: >
    PROJ-010's correctness thesis at its sharpest point — the tool currently
    degrades what the user handed it, on shipped verbs, and reports the
    degradation as a success.
  delivers:
    - Animated input is never silently flattened, and `lint` stops recommending
      the command that destroys it.
    - Ops preserve the colour type and bit depth they were given when the target
      format can hold them.
    - Resampling is measured against its own premise before it is changed.
  explicitly_does_not:
    - Add animated output (animated WebP/AVIF encode) — that is the capability,
      not the repair; it is filed separately.
    - Introduce a 16-bit-throughout pipeline as a design goal. `DynamicImage`
      already carries the variants; only three op bodies collapse them.
    - Add any user-facing flag. "Preserve what you were given" is not something
      a user should have to ask for.
---

# STAGE-046: Output Fidelity on Shipped Verbs

## What This Stage Is

crustyimg's thesis is quality-per-byte. On four measured counts it currently
delivers less than it was given, silently, on verbs that shipped in 0.7.0 —
including `web`, the flagship. This stage makes the shipped verbs preserve what
they receive, and makes the tool say so when it genuinely cannot.

The defects were found and driven in separate exploration sessions on
2026-08-15. Their full measurements, root causes and fix constraints live in
`docs/backlog.md`; this stage is where they become schedulable work. **The
evidence is in the backlog entries — do not re-derive it, and do not trust this
summary over them.**

| # | defect | evidence | in `docs/backlog.md` |
|---|---|---|---|
| D-A | Animated input silently flattened; `lint` recommends the command that does it | driven end to end; 3 of 4 frames discarded, reported `72% smaller · ssim 100.0` | `## ⚠ Live defect — animated input is silently flattened` |
| D-B | Ops widen to RGBA and never narrow back | output IHDR read; `resize`/`thumbnail`/`edit`/`web` all emit `colour_type=6` from `colour_type=2` input; +12.4% bytes on a 512×512 PNG | `## ⚠ Live defect — ops widen to RGBA` |
| D-C | The same `to_rgba8()` call truncates 16-bit input to 8-bit | measured on a 32×32 `bit_depth=16` PNG; `convert`/`auto-orient` preserve, `resize`/`edit` halve | same entry as D-B |
| D-D | `resize` resamples in sRGB, not linear light; no premultiplied alpha | source read; `PixelType::U8x4` handed to Lanczos3 with no gamma handling | `## Open — resize resamples in sRGB, not linear light` |

**D-D's alpha half is confirmed.** The backlog entry filed it as *"not confirmed
— only two files were grepped."* A repo-wide grep (2026-08-15) returns
premultiply handling **only** in `src/image/avif.rs` (the MIAF `prem` flag on
AVIF decode). There is no `MulDiv` and no premultiply anywhere on the resize
path.

## Why Now

**Because STAGE-041 publishes the claim these defects contradict.** The launch
content promotes quality-per-byte. Shipping it while `web` invents an all-opaque
alpha channel, `resize` halves 16-bit input, and `lint` recommends a command
that discards three-quarters of an animation would put a claim in public that
the tool measurably does not meet. The maintainer sequenced this stage ahead of
STAGE-041 on 2026-08-15 for that reason.

Three further reasons the timing is now and not later:

1. **D-A has a wrong recommendation attached.** The linter tells users to run
   the destructive command. Every day it ships is a day users can follow it.
2. **The perceptual oracle structurally cannot catch D-A.** SSIMULACRA2 compares
   decoded-source to output, and both are frame 1 — the quantity it measures is
   *preserved* by the bug. Any future test asserting "score stayed high" stays
   green through this defect forever
   [[a-self-referential-control-cannot-detect-a-broken-pipeline]].
3. **D-B/C/D share one blast radius.** All three change output bytes for every
   existing recipe, which invalidates every PROJ-007 build lockfile. That
   migration should be paid **once**. The backlog entries reached this
   conclusion independently and both say to sequence them together.

## Success Criteria

- No shipped verb silently reduces colour type, bit depth, or frame count.
  Where a reduction is genuine and unavoidable (a lossy target is 8-bit), it is
  **reported**, in the spirit of SPEC-090's honest size reporting.
- `lint`'s `format/animated-gif` `fix:` string does not recommend a command that
  destroys data.
- The lockfile-invalidating changes land together, behind one migration story
  and one DEC, not three.
- D-D is **measured before it is fixed** — see Design Notes.

## Scope

### In scope
- The three op bodies that collapse the input: `Invert::apply`
  (`src/operation/mod.rs:197`), `Resize::apply` (`:396`), `Watermark::apply`
  (`:816-817`).
- Multi-frame detection on the pixel path, and the lint rule's advice.
- The resampling colour space and alpha handling inside `Resize::apply`.
- The build-cache key: does it need a colour-pipeline-version component so old
  and new renders cannot collide? (Open sub-question from the D-D entry.)

### Explicitly out of scope
- **Animated output encode** (animated GIF → animated WebP/AVIF). That is the
  capability; this stage is the repair. DEC-091 places it on the workhorse side
  of the fence. **Filed 2026-08-15 as a `docs/roadmap.md` post-1.0 row**, paired
  with this stage — it was *not* actually filed anywhere when this line was
  written.

  ⚠ **Two corrections to the dependency line this stage originally carried.**
  `webp-animation` v0.10.0 is **not** a clean verified take: the licence is
  right (MIT OR Apache-2.0) but it **wraps `libwebp-sys2`, a C dependency**, so
  it does not clear `pure-rust-codecs-default` and would need an off-by-default
  feature (the `webp-lossy`/DEC-022 precedent). A **pure-Rust route needs no new
  dependency at all** — `image`'s `AnimationDecoder` + the existing `Pipeline`
  per frame + `image-webp` frame encode + an in-house RIFF mux. And **animated
  AVIF is a separate, unpriced question** (a HEIF image sequence, materially
  harder than RIFF) — do not treat "WebP/AVIF" as one item. See
  `docs/backlog.md` → "animated input is silently flattened", §(b).

  **Consequence for this stage's first spec:** because the capability does not
  exist yet, the repair can only *blunt* `lint`'s advice, not correct it. The
  rule's claim that "a modern format encodes far smaller" is also, today,
  **unverified by this repo** — crustyimg cannot produce the animated format it
  recommends, so it has never measured the win. Shipping the capability is the
  first chance to.
- A 16-bit-throughout pipeline as a goal in itself. Decode, `Identity`,
  `AutoOrient` and the default encode path already preserve the variant; only
  the three op bodies collapse it. **No type change is needed anywhere.**
- Any new user-facing flag.
- `crustyimg-lab`. Per DEC-091 and the D-A entry: a defect in crustyimg cannot
  be fixed in a different binary.

## Spec Backlog

Ordered. D-A first because it is small, urgent, independent of the others, and
carries no lockfile cost.

- [x] **SPEC-119** (design 2026-08-15, shipped 2026-08-16, PR #176) — [M] animated input is never silently flattened: warn on
      the pixel path instead of discarding frames, and stop `lint` recommending
      the destructive command. **The sweep ran at design and the rule name WAS
      too narrow** — GIF, APNG and animated WebP all flatten silently, so this is
      three formats, not one. Re-estimated S → M for that reason.
- [ ] (not yet written) — [M] ops preserve colour type and bit depth: widen to
      work, narrow to write. Fixes D-B and D-C in one change across the three op
      bodies. Lockfile-invalidating.
- [x] **SPEC-120** (design 2026-08-15, shipped 2026-08-16, PR #175) — [S] measured D-D's premise before fixing
      it. **Gates the next spec.** Design found SSIMULACRA2 cannot score a
      downscale against its source (equal dimensions, `report.rs:329`), so the
      experiment needs an independently-generated reference — and must prove the
      scorer can see the effect at all before a null result is readable.
- [ ] (not yet written) — [M] ⚡ **UNBLOCKED — SPEC-120 ruled the premise HOLDS (DEC-092).**
      And **narrowed it: the premultiplied-alpha half is FALSE** — `fast_image_resize` 6.0.0
      already premultiplies by default, so this is **one premise, not two**. Resample in linear
      light,
      conditional on the measurement above. Lockfile-invalidating; sequence with
      the colour-type spec so the migration is paid once.

- [ ] (not yet written) — [M] **Three writing paths still silently flatten animated input.**
      SPEC-119 fixed `convert`/`optimize`/`web`/`build`(Decide)/`apply`(terminal-optimize).
      **Driven by its verify, 2026-08-16:** `responsive anim.gif --widths 16` writes a 1-frame
      `anim-16w.gif` from a 4-frame source, exit 0, **empty stderr** — same for APNG and WebP.
      `apply --recipe <plain pixel recipe>` and `build` with a plain recipe are silent too.
      **The stage's own Goal — "no shipped verb silently discards frames" — is therefore not yet
      met.** Not a regression: `run_responsive` has its own `Image::load`
      (`src/cli/optimize.rs:1744`) and **misses the truncated-JPEG warning as well**, so this seam
      drops *both* diagnostics. That makes it evidence for STAGE-042's conformance matrix
      (SPEC-118) as much as a fix in its own right — a verb-by-diagnostic matrix would have
      surfaced it mechanically instead of at verify.

- [ ] (not yet written) — [S] **`info` describes an animated file as a still.** It is the one
      verb whose entire job is reporting, and `run_info` (`src/cli/report.rs:240-275`) checks
      `is_truncated_jpeg()` and **never calls `is_animated_input()`** — confirmed by reading, and
      surfaced by SPEC-119's punch list. Two consequences, the second sharper than the first:
      it prints no animation warning where every pixel verb now does; and its report is
      internally inconsistent — `file_size_bytes` covers **all frames** while `decoded_bytes`,
      `width`, `height` and `color_type` come from `img.info()`, i.e. **frame 1 only**. The two
      size fields describe different things without saying so. The flag already exists and
      `Image` already carries it, so this is a report field plus a warning, not new detection.

> **Moved from STAGE-042, 2026-08-16.** Same class as the rest of this stage: the tool
> silently delivers less than it was given and exits 0.

- [ ] (not yet written) — [S] ⚠ **PRIORITY: the `IMAGE_EXTENSIONS` gap silently defeats the
  strict gate that a maintainer decision rests on.** Not a new defect — the *consequence* of the
  item below, and it is why that item is no longer routine.

  SPEC-119's Call 1 (animated input **warns and proceeds** rather than refusing) was accepted on
  2026-08-16 on one argument: **`lint --max-warnings 0` is the strict path**, so a pipeline that
  must never flatten an animation has a way to say so. Driven by SPEC-119's verify:

  ```
  lint --max-warnings 0 <dir containing anim.webp>   → exit 0   "1 scanned · 0 warn"
  lint --max-warnings 0 <dir>/anim.webp              → exit 7
  optimize <dir>/*.webp                              → warns; 408 → 240 B, 4 frames → 1
  ```

  **Directory mode — the shape CI actually uses — returns a false green.** Naming the file or
  piping stdin both work. `docs/api-contract.md` states `lint --max-warnings 0` "fails on any of
  the three formats" **with no qualifier**, which is now false as written.

  Two things follow: the contract sentence needs its qualifier (SPEC-119 punch list), and the
  `IMAGE_EXTENSIONS` fix should be **specced rather than left in the backlog**, because a
  maintainer ruling now depends on it.

- [ ] (not yet written) — [S] **`webp` is missing from `IMAGE_EXTENSIONS`, so directory and glob
  discovery silently skips `.webp` files.** `src/source/mod.rs:105-113` lists 30+ extensions —
  jpg/png/gif/bmp/tif/ico/avif/svg, eleven RAW families, heic/heif — and **not `webp`**, which is
  a supported input *and* an output format the tool writes by default. So `crustyimg web ./dir/`
  processes everything except the files crustyimg itself produced. **Reproduces on `main`,
  confirmed 2026-08-16**; found by SPEC-119's build and recorded in DEC-093, which is not a
  backlog anyone reads. The repo already knows this hazard class — `src/lint/mod.rs:217` cites
  "the IMAGE_EXTENSIONS-exposes-every-decode-caller lesson" by name. Adding an extension changes
  every decode caller, so **audit each caller and its `Err(_)` arm** rather than editing the list
  alone.

**Count:** 2 shipped (SPEC-119, SPEC-120) / 0 in flight / 6 pending

## Design Notes

- **"Widen to work, narrow to write"** is the shape of the D-B/C fix. An op
  preserves the input's colour type and bit depth unless it genuinely needs more
  (compositing a translucent overlay). `Watermark` is the arguable case —
  compositing genuinely needs alpha — and should be decided explicitly rather
  than swept in.
- **Measure before fixing D-D.** The backlog entry sets its own falsification
  gate and it should be honoured: if SSIMULACRA2 does not prefer the
  linear-light output, the premise is wrong and the entry closes. That is a
  separate, cheap spec on purpose — it is the difference between a measured
  change and a plausible one
  [[a-plausible-test-result-is-not-a-checked-one]].
- **The lockfile migration needs a DEC**, covering the cache-key question. One
  DEC for the combined byte-changing wave, not one per spec.
- **The backend is already in place.** `fast_image_resize` 5.x ships
  `U16x4`/`F32x4` and `MulDiv`; `DynamicImage` already has `ImageRgb16` /
  `ImageRgba16`. Neither fix needs a new dependency.
- ⚠ **"crustyimg is 8-bit internally" is not accurate as stated** and appears in
  places that outlive this stage. Correcting that claim wherever it is written is
  part of the colour-type spec, not a follow-up.

## Dependencies

### Depends on
- Nothing. All four defects are in shipped code and reproduce on `main`.
- STAGE-042's conformance matrix (SPEC-118) would likely have caught D-B and D-C
  mechanically. It is **not** a blocker — but if SPEC-118 is built first, its
  matrix should be checked against this stage's findings as a live test of
  whether the instrument works.

### Enables
- **STAGE-041** (launch content). The maintainer's sequencing call: the public
  quality-per-byte claim should be true before it is published.
- Animated WebP/AVIF encode — the linter's advice cannot become correct until
  the destructive path is closed.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?**
- **How many specs did it actually take?**
- **What changed between starting and shipping?**
- **Lessons that should update AGENTS.md, templates, or constraints?**
- **Should any spec-level reflections be promoted to stage-level lessons?**
