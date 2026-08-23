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
    What you ask crustyimg for should be what you get, however you spell the
    request — and it should never silently do less. Nine measured defects say
    otherwise today, and every one was found by USING the tool rather than
    auditing it: the same recipe writes a different format under `apply` than
    under `build`; `--format` is silently ignored on a multi-input batch; a
    recognized extension on `-o` changes the encode quality, the scoring and the
    report; `--explain` goes silent exactly where `--json` errors; a multi-page
    TIFF or multi-size ICO is reduced to one image and the loss is reported as a
    size win. Underneath them sits one structural gap: a recipe cannot express a
    watermark, a format or a quality, so the batch path is strictly weaker than
    the single-file path it exists to automate — which is WHY `apply --format`
    exists, and why its being broken matters. Closing these makes the tool do
    what a user who has read `--help` would predict, which is testable in a way
    that "improve quality" is not.
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

PROJ-010 asked whether crustyimg's shipped verbs were **correct**. This asks whether they are
**consistent and complete** — whether a request means the same thing however it is spelled, and
whether the batch path can express what the single-file path can do.

Two clusters, and they are the same wound seen from two sides:

1. **Invocation inconsistency.** `apply` and `build` disagree on the default output format for the
   same recipe. `apply --format` is honoured on one input and silently ignored on two. A
   recognized extension on `-o` switches `web` from auto-decide to pinned-convert — changing
   quality 85 → 80, the scoring, and the report — because a filename was read as a format request.
   `--explain` returns exit 0 and zero bytes where `--json` raises a clear usage error.
2. **Silent incompleteness.** A multi-page TIFF returns page 1 and `optimize` calls it *"86%
   smaller"*; a multi-size ICO returns one entry and calls it *"74% smaller"*; `lint` reports
   `0 error · 0 warn · 0 info` on both. And a recipe cannot carry a watermark, a format or a
   quality at all — the registry holds four operations, and `Recipe` has no field for either.

⚡ **The link that makes this one project rather than two:** `apply --format` exists *because* a
recipe cannot carry a format. It is the workaround for the structural gap — and the workaround is
the thing that is broken. Fixing them separately would mean fixing the same wound twice.

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
- ⚡ **Animated AVIF output** — forked to its own design track, see the Stage Plan. This project
  neither builds nor blocks it.
- **Video containers, and any codec not already shipped** (DEC-088).
- **Animated WebP output** — no pure-Rust encoder exists (`image-webp` 0.2.4 writes `VP8X` but
  emits no `ANIM`/`ANMF`).
- **New verbs.** The surface is 18 verbs; this project widens what existing ones reach and makes
  them agree with each other. It does not add to the roster.
- **The engineering-quality backlog** — the three external review batches, STAGE-042's remainder,
  the `F32x4` resource-cap gap. Real work, no user-facing thesis. ⚠ Smuggling it in here is the
  drift this project was created to stop, and it is the single most likely way this project
  loses its shape.

## Stage Plan

**Ordered by dependency, not by appetite.** Every stage here is byte- or behaviour-changing on a
shipped verb, so the whole project carries **one** lockfile migration and ships as **one** release
— the same batching STAGE-046 used. That is what makes the ordering load-bearing rather than a
preference.

- [ ] **STAGE-049 — the conformance matrix. FIRST, and it is not optional.**
  A `(command × output-flag)` sweep asserting **encode-identical bytes and report parity** across
  every combination, plus every verb's default quality asserted explicitly so a silent change goes
  red. ⚡ **This is the regression net for the three stages after it**, all of which change encode
  behaviour on shipped verbs — building it last would mean building it after the thing it catches.
  It is also the test that would have caught the `-o` divergence before a user did.
  ⛔ **Blocked on one decision: SPEC-118 already exists, framed since 2026-08-15, for
  substantially this instrument.** Framing is the expensive part. **Resolve which absorbs which
  before writing a line** — do not build a second matrix beside a framed one.

- [ ] **STAGE-050 — invocation consistency.** The three defects where the spelling changes the
  meaning: `apply --format` ignored on multi-input and the `apply`/`build` default disagreement
  (one defect, not two — `apply`'s multi-input path does no format resolution at all);
  the `-o`-extension pin; `--explain`'s silence under a pin.
  ⛔ **Needs a maintainer ruling on the `-o` pin before it can be specced** — warn when an
  extension triggers a pin, require `--format` to pin and treat `-o` as destination only, or
  document it loudly and keep the behaviour. All three are defensible; the spec cannot be written
  until one is chosen.

- [ ] **STAGE-048 — input completeness.** ⚠ **Re-scoped: animated AVIF output has moved out**
  (see below), leaving the input side, which is tractable today. Multi-size ICO (the entry count
  is readable from the header — do this one first); multi-page TIFF (⚠ **materially harder** —
  `image` exposes no multi-page API, so pages 2..N are unreachable *and undetectable*, making even
  "warn and refuse" a dependency question); `lint` detection for both; `info` describing an
  animation as a still; the `IMAGE_EXTENSIONS` gaps.
  ⛔ **Needs the ICO round-trip ruling** — warn / fix / accept — because a real fix changes bytes.

- [ ] **STAGE-051 — recipe reach.** A `watermark` operation in the registry with its ten
  parameters, plus **output format and quality as `Recipe` fields**, plus typed per-operation
  parameter structs so schema errors surface at parse time rather than partway through a batch.
  ⚠ **Sequenced after STAGE-050 deliberately**: `apply --format` is the workaround for the missing
  `Recipe` format field, so fixing the workaround first establishes what the field has to mean.
  ⚠ **One design call inside it:** `watermark --size` is **absolute pixels**, so a recipe-level
  watermark only behaves consistently if the recipe normalises dimensions first. That is a
  constraint on the design, not a detail to discover during build.

**Count:** 0 shipped / 0 active / **4 pending** — re-derive with a grep you just ran; never restate
a tally you carried forward.

### Forked out: animated AVIF output

**Moved to its own design track by maintainer decision, 2026-08-23.** It is the highest-ceiling
work here — measured at **11.2× vs GIF** and **6.3× smaller than animated WebP at higher quality**,
pure Rust and patent-clear — and it is also the least ready: complexity **L** with its own draft
saying *"L means split it"*, a **~1,000-line** muxer measured rather than guessed, and a new
dependency needing a DEC.

⚡ **Forking it does not delay it, because what blocks it is not build capacity — it is that it is
unspecified.** The next work it needs is **design**: a `mp4-atom` DEC, and splitting
`docs/research/draft-spec-animated-avif-output.md` into buildable specs. That runs in parallel with
this project without competing for build sessions or touching this project's migration. It becomes
**PROJ-012** when it is specced, and carries its own migration then.

## Dependencies

### Depends on
- **v0.7.1 shipped** ✅ (2026-08-22, live on all three channels). Every stage here is byte- or
  behaviour-changing, so this project's changes batch into **one** migration in a single later
  release — the constraint that makes the stage ordering load-bearing.
- ⛔ **Three maintainer rulings, each blocking a specific stage and none of them expensive:**
  the **`-o` pin** (blocks STAGE-050), the **ICO round-trip** — warn / fix / accept (blocks
  STAGE-048), and **SPEC-118 vs the conformance matrix** — which absorbs which (blocks STAGE-049,
  which everything else depends on, so this is the one to answer first).
- **PROJ-010 stays open.** Its triage found ~24 actionable items; it is the correctness/quality
  lane, not a project that ships. Six of its items move here.

### Enables
- `lint` advice that is true rather than destructive.
- The animated-image findings banked by PR #177 becoming shipped capability rather than research.
- A recipe surface strong enough that `--save-recipe` could reasonably be offered on more
  than one verb.

## Project-Level Reflection

*Filled in when status moves to shipped.*
