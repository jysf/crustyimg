---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-048
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-013
repo:
  id: crustyimg

created_at: 2026-08-21
shipped_at: null

# ⚠ PLACEMENT IS AN OPEN QUESTION — see "Why Now → The thesis problem".
# This stage is filed under PROJ-010 so it is visible to `just backlog` today.
# Half of it does not fit PROJ-010's thesis and probably wants its own project.
value_contribution:
  advances: >
    The correctness half advances PROJ-010's thesis directly — "no 'it made my
    file bigger' surprises" extends to "no 'it silently threw away my data'
    surprises," on shipped verbs, which is what STAGE-046 exists for. The
    capability half (animated AVIF output) does NOT advance PROJ-010's thesis
    and is flagged rather than rationalised.
  delivers:
    - "Multi-image input (multi-page TIFF, multi-size ICO) is never silently reduced to one image"
    - "Animated input has somewhere to go: an animated AVIF output path that preserves every frame and its timing"
    - "`lint`'s advice becomes true — it can recommend a command that does not destroy data"
  explicitly_does_not:
    - "Read video containers, or decode any codec crustyimg does not already ship (DEC-088)"
    - "Emit animated WebP — no pure-Rust encoder exists (`image-webp` 0.2.4 writes VP8X but no ANIM/ANMF)"
    - "Handle alpha in animated output — deferred to a follow-on, refused explicitly until then"
    - "Re-open SPEC-119, which fixed the animation axis and is shipped"
---

> ⚠ **RE-SCOPED AND RE-HOMED 2026-08-23. Read this before the body below.**
> **Animated AVIF output is forked out** to its own design track (it becomes **PROJ-012** once a
> `mp4-atom` DEC exists and `docs/research/draft-spec-animated-avif-output.md` is split into
> buildable specs). **This stage keeps only the INPUT side** — multi-size ICO, multi-page TIFF,
> `lint` detection, `info`-on-animated — which is tractable today.
> ⚠ **It also STAYS IN PROJ-010.** An earlier plan moved it to PROJ-011; PROJ-011 was then re-cut
> around a single capability outcome (a declared `build` that can watermark a site), and silent
> multi-image data loss — real as it is — is not in the way of that. It belongs in the correctness
> lane. **The body below still describes both halves; treat its animated-AVIF items, parts (a) and
> (b), as moved rather than as scope.**

# STAGE-048: Multi-Image Input Completeness

## What This Stage Is

crustyimg today has a hole with two sides. On the way **in**, a file carrying more than one
image is silently reduced to one — SPEC-119 closed that for animation (GIF / APNG / animated
WebP), but multi-page TIFF and multi-size ICO still lose data with exit 0 and no diagnostic.
On the way **out**, there is no multi-frame format at all, so even a correctly-detected
animation has nowhere to go but a refusal.

When this stage ships, a multi-image input is either **preserved or refused with a reason** —
never silently narrowed — and an animation can be re-encoded as an animated AVIF that keeps
every frame and its timing, sized by the existing perceptual quality search.

The two sides are separable and should ship separately, but they belong in one stage because
**neither is complete alone**: refusing to flatten without an animated output leaves users
stuck, and an animated output path while TIFF pages still vanish silently is a capability
built on an unfixed floor.

## Why Now

**The correctness half is a live, measured defect on shipped verbs.** Driven on `main` at
`dcd43c8` (2026-08-21), with fixtures built independently of the code under test and their
structure verified before conversion:

| input | contains | `convert` output | lost | exit | stderr | `lint` | `optimize` reports |
|---|---|---|---|---|---|---|---|
| 3-page TIFF | greys 70/140/210 | 8×8 png, pixel **70** | **2 pages** | 0 | none | 0 warn | **"86% smaller"** |
| 3-size ICO | 16/32/64 = R/G/B | **64×64**, blue | **16px + 32px** | 0 | none | 0 warn | **"74% smaller"** |

That is, word for word, the defect SPEC-119's own `value_link` described: *"accepts a valid
file, discards every frame but the first, reports the loss as a win … and — through `lint` —
actively recommends the command that does it."* It was fixed on one axis and left on two.

**The capability half has measurements, not just an idea.** From
`docs/research/draft-spec-animated-avif-output.md`, driven end to end out-of-crate: a
308,156 B / 36-frame GIF → **27,564 B at SSIMULACRA2 86.7** (**11.2×**); animated WebP's best
measured point was 172,492 B at 84.1, so **AVIF is 6.3× smaller at higher quality**. The whole
path is pure Rust and patent-clear — `rav1e` encodes, `re_rav1d` decodes, both already in-tree.

### ⚠ The thesis problem — a real one, stated rather than smoothed over

**PROJ-010's thesis is launch-gating correctness and consolidation.** The correctness half sits
inside it comfortably. **The animated-AVIF half does not** — it is new capability, and AGENTS §3
exists precisely to stop a stage that "seems valuable but doesn't contribute to the project
thesis" from being rationalised into one.

This stage is filed under PROJ-010 anyway, for one reason: **`just backlog` only surfaces the
active project**, so a capability parked outside it is invisible to every command the next
session runs — the failure mode that hid the TIFF/ICO measurement in `docs/backlog.md` for
four days. Visibility now beats taxonomy now.

**Maintainer call, and it should be made before any spec here is framed:** either (a) accept a
capability stage inside PROJ-010 and say so in the brief, or (b) open **PROJ-011** for
multi-frame reach and move the animated-AVIF half there, leaving the correctness half on
STAGE-046 where it already is. **(b) is the honest structure**; (a) is the fast one.

## Success Criteria

- A multi-page TIFF and a multi-size ICO through `convert`, `optimize` and `web` either
  preserve every image or **fail with a message naming what would be lost** — never exit 0
  silently, and never report the loss as a size win.
- `lint` flags a multi-image input rather than reporting `0 error · 0 warn · 0 info`.
- A GIF with N frames re-encodes to an animated AVIF whose decoded frame count is **N**,
  asserted with an **independent decoder** (`re_rav1d`), not the encoder's own packet count.
- Frame timing round-trips: per-frame durations and loop count survive.
- Output is sized by the **existing** perceptual quality search, not a fixed quantizer.
- A near-lossless control scores **high** before any quality claim is believed
  [[a-flat-quality-curve-means-a-colour-bug-not-a-codec]].

## Scope

### In scope
- Detection and honest handling of multi-image TIFF and ICO input.
- An animated-AVIF writer: muxer, frame timing, loop count.
- Reuse of `quality::search` and the `AVIF_SPEED` discipline.
- Bounding total frames and total decoded pixels **before** decoding, mirroring `check_caps`.
- A `DEC-*` for `mp4-atom` (`no-new-top-level-deps-without-decision`).

### Explicitly out of scope
- Video containers; any codec not already shipped (DEC-088).
- Animated WebP output — no pure-Rust encoder exists.
- Alpha in animated output — refused explicitly, deferred.
- Re-opening SPEC-119.

## Spec Backlog

- [ ] (not yet written) — [S] **ICO: never silently discard entries.** ⚡ **Do this first — it
  is the tractable one.** The icon directory is in the file's header, so the entry count is
  readable *before* any decode; `image`'s `IcoDecoder::new` calls `best_entry()` and drops the
  rest. Detect, then decide: refuse, or preserve, or warn. A decision either way is progress;
  silence is not.

- [ ] (not yet written) — [M] **TIFF: multi-page input is not silently reduced to page 1.**
  ⚠ **Materially harder than ICO and must be scoped separately.** `image` exposes **no
  multi-page API at all** (`TiffDecoder` has `fn new` only), so pages 2..N are **unreachable
  *and undetectable*** through the current dependency. Detecting the condition at all needs
  the `tiff` crate directly — which makes even *"warn and refuse"* a dependency question, not
  a one-line guard. **A spec promising ICO and TIFF together will discover this mid-build.**

- [ ] (not yet written) — [S] **`lint` sees multi-image input.** Today: `0 error · 0 warn ·
  0 info` on both fixtures. Depends on the two above for detection.

- [x] **MOVED to PROJ-012, 2026-08-23 — animated AVIF output, parts (a) and (b).** The muxer and
  frame/timing round-trip, and the quality search and speed pinning. **PROJ-012's thesis is
  *every defect lint can name, crustyimg can act on***, and animated output is what closes
  `format/animated-gif` and `format/animated-input` — rules that today flag an animation the tool
  cannot re-encode. That is a better home than "multi-frame", which grouped input and output work
  by subject matter rather than by purpose.
  ⚠ **The measured traps moved with it** and are recorded in PROJ-012's stage plan, not lost here:
  budget the muxer at **~1,000 lines** (`mp4-atom` supplies boxes, not a muxer), **speed 10 is a
  trap for animation**, assert frame count with an **independent decoder** rather than the
  encoder's packet count, and **run a near-lossless colour-range control first** — a near-lossless
  encode scoring 57.2 was traced to `Range: Limited` and scored 96.5 after the fix.

- [ ] (not yet written) — [S] **`optimize` can name TIFF and ICO.** Seen while driving the
  defect above: `optimize` reports `source_format` as **`other`** for both. SPEC-115 was
  *"never passes through bytes it cannot name"* and SPEC-117 taught `source_format` the real
  container for svg/heic/raw. **Check whether `other` is correct here or the same gap on two
  more formats** — one look, before specing anything else in this stage.

**Count:** 4 pending / 1 closed-or-moved — re-derive with a grep you just ran.
a tally you did not just run.

## Design Notes

- **Two problems, one description.** "Multi-image input is silently narrowed" covers ICO and
  TIFF, but the mechanisms are unrelated: ICO's is a *choice* (`best_entry()` scores and
  discards), TIFF's is an *absence* (no API exists). Anything written as one spec will split
  itself during build.
- **The independent-decoder rule is load-bearing here.** An encoder reporting N packets is not
  evidence of N frames. `re_rav1d` is in-tree and is the oracle
  [[verify-wasm-output-with-an-independent-decoder]].
- **Structural assertions before pixel assertions.** A frame-count check that runs after a
  pixel comparison can be skipped by an early failure.
- **The colour-range trap, measured in the draft:** an AVIF whose score is *insensitive to the
  quality knob* has a range/matrix bug, not a codec problem — a near-lossless encode scoring
  57.2 was traced to `Range: Limited` against full-range input, and scored 96.5 after the fix.
  Run that control first.
- **Browser support is a moving fact.** caniuse's 94.65% measures **still** AVIF and does not
  break out sequences — an upper bound, not the number. Re-check before any published claim.

## Dependencies

### Depends on
- **STAGE-046** — SPEC-119 fixed the animation axis; this stage is the rest of the surface.
  The TIFF/ICO item is currently filed there and should move here if this stage is accepted.
- **A `DEC-*` for `mp4-atom`** — `no-new-top-level-deps-without-decision`, and the licence gate
  DEC-018 (`just deny`).
- **A maintainer ruling on placement** (see "Why Now").

### Enables
- `lint` advice that is true rather than destructive.
- The animated-image findings banked by PR #177 becoming shipped capability instead of
  research.
- A future alpha-in-animation follow-on, explicitly deferred here.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
