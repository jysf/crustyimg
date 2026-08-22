---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-011
  status: proposed
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-08-21
shipped_at: null

value:
  thesis: >
    crustyimg's reach and predictability should match what its own surface already
    implies. Three gaps say otherwise today, and all three were found by using the
    tool rather than by reading it: a file carrying more than one image is silently
    reduced to one; a recipe cannot express watermark or an output format, so the
    batch path is strictly weaker than the single-file path it claims to automate;
    and a recognized extension on `-o` silently changes the encode quality, the
    scoring and the report. Closing them makes the tool do what a user who has read
    `--help` would predict — which is a different and more testable claim than
    "add features".
  beneficiaries:
    - "Anyone who feeds crustyimg a multi-page TIFF, a multi-size ICO or an animation and currently gets one image back with no diagnostic"
    - "Anyone automating with recipes, who today cannot express watermark, an output format, or a quality — the three things the CLI does most"
    - "Anyone reading a size or ssim number from `web`, which changes meaning depending on whether the output path had a recognized extension"
    - "The maintainer, who found all three of these by using the tool on real work and should not have to"
  success_signals:
    - "A multi-page TIFF and a multi-size ICO are either preserved or refused with a message naming what would be lost — never exit 0 silently, never reported as a size win"
    - "An animated GIF re-encodes to an animated AVIF whose decoded frame count equals the input's, asserted with an independent decoder (`re_rav1d`), not the encoder's packet count"
    - "A recipe can express watermark and can pin an output format and quality — so `apply --recipe` reaches what `watermark` and `convert` reach"
    - "A `(command x output-flag)` matrix asserts encode-identical bytes and report parity across every combination, so a divergence like the `-o` pin fails CI rather than reaching a user"
    - "Every verb's default quality is asserted in a test, so a silent change to any of them goes red"
  risks_to_thesis:
    - "⚠ Two of the three pillars are byte-changing on shipped verbs, so the wave carries a lockfile migration and must be batched into one release the way STAGE-046's was — sequencing this wrong costs users two migrations instead of one"
    - "The animated-AVIF muxer is measured at ~1,000 lines and needs a new dependency (`mp4-atom`) plus its DEC. It is the largest single piece of work here and the one most likely to be under-estimated"
    - "TIFF multi-page detection is not reachable through `image` at all, so even 'warn and refuse' is a dependency question. A stage that assumes it is a guard will stall"
    - "⚠ 'Predictability' can rationalise almost any change. If a spec here cannot name the user-visible surprise it removes, it does not belong in this project"
---

# PROJ-011: Surface Reach and Predictability

## What This Project Is

PROJ-010 asked whether crustyimg's shipped verbs were **correct**. This project asks
whether they are **complete and predictable** — whether the tool reaches what its own
surface implies, and behaves the way a user reading `--help` would expect.

Three gaps, all found by using the tool on real work rather than by reading it:

1. **Reach into multi-image input and out to multi-frame output.** A multi-page TIFF or
   a multi-size ICO is silently reduced to one image; an animation, correctly refused
   since SPEC-119, has nowhere to go because there is no multi-frame output format.
2. **Recipe reach.** The registry holds four operations. `watermark` — a whole verb with
   ten parameters — cannot be written in a recipe, and neither can an output format or
   quality: `Recipe` has no field for either. The batch path is strictly weaker than the
   single-file path it exists to automate.
3. **Surface predictability.** A recognized extension on `-o` silently switches `web`
   from auto-decide to pinned-convert, changing the encode quality (85 → 80), the
   scoring, and the report — three things a user has no reason to associate with a
   filename.

## Why Now

**All three were found the same way: by a maintainer using the tool, not auditing it.**
That is the signal. PROJ-010's wave was found by audit and measurement; these were found
by someone trying to get work done and being surprised. Surprises that survive an audit
are the ones that reach users.

**The correctness half is live and measured**, driven on `main` at `4514345` with
fixtures built independently of the code under test: a 3-page TIFF returns page 1 and
`optimize` calls it *"86% smaller"*; a 3-size ICO returns the 64px and calls it *"74%
smaller"*; `lint` reports `0 error · 0 warn · 0 info` on both.

**The capability half has measurements, not an idea.** A 308,156 B / 36-frame GIF →
**27,564 B at SSIMULACRA2 86.7** — **11.2×**, and **6.3× smaller than animated WebP at
higher quality**. The path is pure Rust and patent-clear; `rav1e` and `re_rav1d` are
already in-tree.

**And PROJ-010's thesis no longer covers any of it.** Its thesis is launch-gating
correctness; the launch shipped, and its remaining stages are a mix of genuine leftovers
and capability filed there only because `just backlog` surfaces the active project. That
drift is what AGENTS §3 exists to catch. **This project is where the capability half goes
so it stops borrowing a thesis that does not fit it.**

## Success Criteria

- A multi-page TIFF and a multi-size ICO are **preserved or refused with a reason** —
  never silently narrowed, never reported as a size win.
- An animated GIF re-encodes to an animated AVIF whose decoded frame count is **N**,
  asserted with an **independent decoder**, with per-frame timing and loop count
  round-tripping.
- `apply --recipe` reaches what `watermark` and `convert` reach: a recipe can carry a
  watermark step and can pin an output format and quality.
- A **`(command × output-flag)` matrix** asserts encode-identical bytes and report parity
  across every combination — the test that would have caught the `-o` pin divergence
  before a user did.
- **Every verb's default quality is asserted in a test**, so a silent change goes red.
- `lint`'s advice is true: it never recommends a command that destroys data.

## Scope

### In scope
- Multi-image input detection and honest handling (TIFF pages, ICO entries).
- Animated AVIF output: muxer, frame timing, loop count, reusing the existing quality search.
- Recipe reach: a `watermark` operation in the registry; output format and quality on `Recipe`.
- Surface predictability: the `-o`-extension pin rule, and the documented defaults behind it.
- The `(command × output-flag)` encode-identity and reporting-parity matrix.

### Explicitly out of scope
- **Video containers, and any codec not already shipped** (DEC-088).
- **Animated WebP output** — no pure-Rust encoder exists (`image-webp` 0.2.4 writes `VP8X`
  but emits no `ANIM`/`ANMF`).
- **Alpha in animated output** — refused explicitly, deferred to a follow-on.
- **New verbs.** The CLI surface is 18 verbs; this project widens what existing ones reach,
  it does not add to the roster.
- **The engineering-quality backlog** — the two external review batches, STAGE-042's
  remaining items, and the `F32x4` resource-cap gap. Real work, no user-facing thesis;
  smuggling it in here is the drift this project was created to stop.

## Stage Plan

- [ ] **STAGE-048** (proposed) — **Multi-frame reach.** Already drafted, currently filed
  under PROJ-010 for `just backlog` visibility. **Moves here.** Six items: ICO first (the
  tractable one — the entry count is in the header), TIFF separately (unreachable through
  `image`, so a dependency question), `lint` detection, animated AVIF in two parts, and one
  look at why `optimize` reports `source_format` as `other` for both.

- [ ] (not yet defined) — **STAGE-049, recipe reach.** A `watermark` operation in the
  registry with its ten parameters, and output format + quality as `Recipe` fields.
  ⚠ **Scope these together.** Watermark is the ask; the format/quality hole is larger and
  structural, and a spec that closes only the first will re-open the file.
  📌 The reason this stayed invisible: **only `edit` has `--save-recipe`**, so the one path
  that emits a recipe covers only ops a recipe can already express.

- [ ] (not yet defined) — **STAGE-050, surface predictability.** The `-o`-extension pin
  ruling and the `(command × output-flag)` matrix. ⚠ **The matrix should land FIRST** — it
  is the regression net for the other two stages, both of which change encode behaviour on
  shipped verbs.

**Count:** 0 shipped / 0 active / **3 pending** — re-derive with a grep you just ran; never
restate a tally you carried forward.

## Dependencies

### Depends on
- **v0.7.1 shipping first.** Two of the three pillars are byte-changing on shipped verbs,
  and the lockfile migration keys on `crate::version()` — so this wave's byte-changers must
  be batched into a single later release, exactly as STAGE-046's were.
- **A `DEC-*` for `mp4-atom`** (`no-new-top-level-deps-without-decision`, licence gate DEC-018).
- **A maintainer ruling on the `-o` pin** — warn, re-trigger, or document-and-keep.
- **PROJ-010 closing**, or an explicit decision to run both.

### Enables
- `lint` advice that is true rather than destructive.
- The animated-image findings banked by PR #177 becoming shipped capability rather than research.
- A recipe surface strong enough that `--save-recipe` could reasonably be offered on more
  than one verb.

## Project-Level Reflection

*Filled in when status moves to shipped.*
