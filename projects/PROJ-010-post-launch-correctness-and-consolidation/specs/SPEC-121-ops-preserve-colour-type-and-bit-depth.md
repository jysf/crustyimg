---
task:
  id: SPEC-121
  type: bug
  cycle: verify
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-010
  stage: STAGE-046
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-16

references:
  decisions:
    - DEC-058
    - DEC-090
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-122
    - SPEC-090

value_link: >
  STAGE-046's largest correctness item. Three op bodies widen every image to
  RGBA8 and never narrow back, so `resize`, `thumbnail`, `edit` and the flagship
  `web` add an all-opaque alpha channel (+12.4% bytes, measured) and halve
  16-bit input — on a tool whose thesis is quality-per-byte.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Established that the
        migration story the backlog entries worried about ALREADY EXISTS —
        the cache key carries `crate::version()` and the lockfile never
        promised hash stability across versions — which removes this spec's
        largest assumed risk.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 165160452
      duration_minutes: 60
      recorded_at: 2026-08-18
      tokens_breakdown:
        input: 1110
        output: 394547
        cache_creation: 911468
        cache_read: 163853327
      estimated_usd: 58.50
      note: >
        MEASURED — transcript sum over 555 assistant messages
        (~/.claude/projects/.../97c194f8-72d1-4e26-bb0d-5bd5c78e562c.jsonl,
        identified by content — 281 hits for the branch's worktree path, not
        by recency). Priced at Sonnet anchors ($3/$15 per MTok; cache_creation
        ×1.25 input, cache_read ×0.10 input) — the model `.message.model`
        actually reports throughout. Re-measured at session end (last action
        before the Cost readout block), not mid-session.
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 25418670
      duration_minutes: 38
      recorded_at: 2026-08-18
      tokens_breakdown:
        input: 316
        output: 182365
        cache_creation: 408067
        cache_read: 24827922
      estimated_usd: 19.53
      note: >
        MEASURED — punch-list return cycle (verify returned ⚠ PUNCH LIST on
        PR #181; seven items). Transcript sum over 158 assistant messages
        (~/.claude/projects/.../c05ba4c9-b0e0-4a06-87db-139d3cc20c2c.jsonl),
        priced at OPUS anchors ($5/$25 per MTok; cache_creation ×1.25 input,
        cache_read ×0.10 input) — `.message.model` is `claude-opus-5` on all
        158. Ran in the main loop, not as a dispatched subagent, so there is
        no `subagent_tokens` to cross-check against. Read at the cost-append
        step with CI still settling, so it EXCLUDES the readout messages
        after it — under-reported by roughly one exchange, and stated rather
        than guessed at. 158 messages against the prompt's ~150 budget.
        ⚠ No `verify` entry exists in this list: the verify cycle that
        produced the punch list did not append one.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-121: ops preserve colour type and bit depth

## Context

Three `Operation::apply` bodies convert to RGBA8 and never narrow back:

| site | op |
|---|---|
| `src/operation/mod.rs:197` | `Invert::apply` |
| `src/operation/mod.rs:396` | `Resize::apply` |
| `src/operation/mod.rs:816` | `Watermark::apply` |

**Measured by driving the binary and reading the output IHDR** — a structural
assertion, not a byte guess. From a 64×64 PNG with `colour_type=2` (RGB, no alpha):

| verb | out `colour_type` | |
|---|---|---|
| `convert --format png`, `optimize`, `auto-orient` | 2 (RGB) | ✅ |
| `resize --max 32`, `thumbnail --size 32`, `edit --invert` | **6 (RGBA)** | ❌ |
| **`web`** | **6 (RGBA)** | ❌ **the flagship** |
| `watermark --text` | 6 (RGBA) | arguable — compositing genuinely needs alpha |

The clean verbs are exactly the ones that run **no `Operation`**. Every verb that
folds a real op through the pipeline comes out RGBA. **`web` is
`auto-orient → resize → optimize`, so fixing `Resize` fixes the flagship.**

**Cost of the wasted channel:** on a 512×512 representative PNG, same pixels —
RGB **377,132 B** vs RGBA **423,756 B**, **+46,624 B / +12.4%** for a channel
carrying no information.

### The same call also truncates 16-bit — one fix, two defects

`to_rgba8()` is *also* why 16-bit input is silently halved. Measured on a 32×32
`bit_depth=16` RGB PNG: `convert`/`auto-orient` preserve **16-bit RGB**;
`resize`/`edit --invert` return **8-bit RGBA** — halved *and* alpha-added.

**"crustyimg is 8-bit internally" is not accurate as stated.** Decode preserves
the `DynamicImage` variant, `Identity` and `AutoOrient` preserve it, and the
default encode path (`img.pixels().write_to(..)`, `src/sink/mod.rs`) preserves it.
**Only the three op bodies collapse it.** `DynamicImage` already has `ImageRgb16`
/ `ImageRgba16`, so **no type change is needed anywhere.**

Full evidence: `docs/backlog.md`, `## ⚠ Live defect — ops widen to RGBA and never
narrow back`. **Read it; do not re-derive it.**

## The design calls — settled here

### Call 1 — "widen to work, narrow to write"

An op preserves the input's colour type and bit depth **unless it genuinely needs
more**. Widening internally is fine; returning a widened image is the defect.

The narrowing must be **lossless-only**: narrow RGBA→RGB only when every alpha
sample is opaque, and 16→8 never (that is a downgrade, not a narrowing). An op
that was handed 8-bit returns 8-bit; one handed 16-bit returns 16-bit.

### Call 2 — `Watermark` is the exception, and it is explicit

Compositing a translucent overlay genuinely produces alpha. **`Watermark` may
return RGBA from an RGB input** — but only when the overlay actually contributed
non-opaque samples. If the composite is fully opaque, Call 1 applies and it
narrows like the others. **Decide this in code, not by exempting the op.**

### Call 3 — a lossy target's 8-bit downgrade is REPORTED, not silent

JPEG and lossy WebP are 8-bit. Feeding them 16-bit pixels is a real downgrade and
the user should be told, in the spirit of SPEC-090's honest size reporting. This
is a **one-line diagnostic at the sink**, not a new policy.

### Call 4 — ⚡ THE MIGRATION ALREADY EXISTS. Do not build one.

Both backlog entries flag "this invalidates every PROJ-007 build lockfile" as the
thing that makes the fix non-trivial, and the linear-light entry asks whether the
cache key needs a new colour-pipeline-version component.

**Established at design, by reading the code — the answer is no:**

- **`cache_key_for` already includes `crate::version()`**
  (`src/cli/build.rs:294`, via `cache::compute_key(crate::version(), …)`). A
  release changes every key, so **old and new renders cannot collide in the
  cache.** No new key component is needed.
- **The lockfile never promised output-hash stability across versions.**
  `src/build/lock.rs:32-36`: `hash` is *"recorded as observed under `[env]`, never
  promised"*, and is explicitly not stable *"across arch/OS/codec versions"*.
  `key` is a function of inputs — including the version — so a bump is *"an
  unambiguous, cross-machine drift signal"*, which is the designed behaviour.

So a user upgrading sees key changes, `--frozen` fails, and they regenerate the
lock. **That is the normal upgrade path, already specified.** This spec's job is
to **drive it and confirm it**, and make sure the release notes say outputs
change — not to invent machinery. If driving it shows something the contract does
not cover, **that is a finding: report it, do not design around it.**

### Call 5 — no new flag

"Preserve what you were given when the target format can hold it" is not
something a user should have to ask for.

## Inputs

- `src/operation/mod.rs:190-210` (`Invert`), `:390-530` (`Resize`), `:810-830`
  (`Watermark`).
- `src/sink/mod.rs` — the `write_to` path that already preserves the variant.
- `src/cli/build.rs:275-302` (`cache_key_for`) and `src/build/lock.rs:20-45` —
  Call 4's evidence.
- `docs/backlog.md`'s RGBA-widening entry.

## Outputs

- **Files modified:** `src/operation/mod.rs` (the three bodies), `src/sink/mod.rs`
  (Call 3's diagnostic), `tests/`.
- **New DEC expected:** yes — one covering the byte-change and its migration
  posture, **shared with SPEC-122** so the wave is one decision, not two.
  `affected_scope`: `src/operation/**`, `src/sink/**`.
- ⚠ **Correct the "8-bit internally" claim wherever it is written** — it outlives
  this stage. Part of this spec, not a follow-up.

## Acceptance Criteria

- [ ] **AC-1.** An RGB (`colour_type=2`) PNG through `resize`, `thumbnail`,
      `edit --invert` and **`web`** comes out **`colour_type=2`**. Asserted on the
      output **IHDR**, not on file size.
- [ ] **AC-2.** A 16-bit RGB PNG through the same verbs comes out **16-bit**.
- [ ] **AC-3.** **An RGBA input with a genuinely translucent pixel stays RGBA.**
      The narrowing is lossless-only — this is the control that stops "always
      narrow" from destroying real alpha.
- [ ] **AC-4.** **`Watermark` returns RGBA only when the overlay contributed
      non-opaque samples**, and narrows otherwise (Call 2). Both directions tested.
- [ ] **AC-5.** **A lossy target reports the 8-bit downgrade** (Call 3), pinned by
      a test asserting the message.
- [ ] **AC-6.** **The byte win is measured, not assumed** — assert the RGB output
      is materially smaller than the RGBA one on the same pixels.
- [ ] **AC-7.** **The verbs that were already correct are unchanged** —
      `convert`, `optimize`, `auto-orient` byte-identical to `main`.
- [ ] **AC-8.** **Call 4 driven, not reasoned.** Build a target with a committed
      lockfile on `main`'s binary, upgrade to the branch binary, and show: the
      cache key changes, `--frozen` fails, regeneration succeeds, and no stale
      cache entry is served. **If the contract does not hold, stop and report.**
- [ ] **AC-9.** **A negative control per op body** — revert `Invert`, `Resize` and
      `Watermark` independently; each turns only its own tests red.
      **The evidence is the behavioural flip, not a binary hash** (AGENTS §15).
- [ ] **AC-10.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      through `rtk proxy`: default, `--no-default-features`, `--features
      webp-lossy`. Clippy and `fmt --check` each. Own `main` baseline. Then read
      the CI legs individually.

## Failing Tests

Written during **design**, before build. AC-1 through AC-4 **fail on today's
`HEAD`** — this is a live defect with a genuine red-to-green transition.

- **`tests/colour_type_preservation.rs`** (new)
  - `"rgb_png_stays_rgb_through_resize_thumbnail_edit_and_web"` — AC-1. **RED.**
  - `"sixteen_bit_png_stays_sixteen_bit"` — AC-2. **RED.**
  - `"translucent_rgba_input_keeps_its_alpha"` — AC-3. Passes today; the control.
  - `"watermark_narrows_when_the_composite_is_opaque"` — AC-4. **RED.**
  - `"watermark_keeps_alpha_when_the_overlay_is_translucent"` — AC-4. Passes today.
  - `"rgb_output_is_smaller_than_rgba_for_the_same_pixels"` — AC-6.
  - `"convert_optimize_auto_orient_bytes_unchanged"` — AC-7.
- **`tests/sink.rs`** — `"lossy_target_reports_the_eight_bit_downgrade"` — AC-5. **RED.**

## Implementation Context

### Decisions that apply
- **DEC-058** — the cache-key composition Call 4 rests on.
- **DEC-090** — the diagnostic channel Call 3's message uses.

### Out of scope
- **Linear-light resampling — SPEC-122.** Same function, different premise.
  **Sequence the two together** so the byte change and its migration are paid once.
- A 16-bit-throughout pipeline as a goal. Only these three bodies collapse it.
- Any new user-facing flag.

## Notes for the Implementer

- **The narrowing must be lossless-only.** AC-3 exists because "always narrow"
  passes AC-1 and destroys real transparency.
- **Three op bodies are three claims** — AC-9 wants three reverts.
- **Do not build a migration.** Call 4 says one exists; your job is to drive it.
- **Budget in exchanges, not minutes** (~250). Cost tracks rebuilds and message
  count, not wall clock.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md`, including
  `just advance-cycle SPEC-121 verify` — and confirm the `cycle:` line moved.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-121-ops-preserve-colour-type-and-bit-depth`
- **PR (if applicable):** opened, not merged (see below)
- **All acceptance criteria met?** yes, with two findings filed rather than
  fixed (both explicitly in-scope to file, not fix — see Deviations):
  - AC-1 through AC-6, AC-9, AC-10: met and driven.
  - AC-7: met — `convert`/`optimize`/`auto-orient` confirmed byte-identical
    to `main` (`cmp` on real binary output, three verbs).
  - AC-8: driven both ways. **Without** a version bump (this build's actual
    state, `0.7.0` unchanged) the contract does NOT hold — `--check`/`build`
    silently serve stale pre-fix bytes. **With** a version bump (a
    transient, uncommitted `0.7.1` experiment, reverted before any commit)
    all four checks hold. Filed as a STAGE-042 backlog item per Call 4's
    "report it, do not design around it."
- **New decisions emitted:** DEC-095 (`decisions/DEC-095-ops-preserve-colour-type-and-bit-depth-the-byte-change-and-its-migration.md`),
  shared with SPEC-122, `affected_scope: src/operation/**, src/sink/**,
  src/image/mod.rs` (the third glob added at punch-list — see below).
- **Deviations from spec:**
  - The "Failing Tests" section describes tests as "written during design,
    before build," but no test files existed on `main` at build start (only
    the spec doc and build prompt were committed at design —
    `git log --all` shows no commit adding `tests/colour_type_preservation.rs`
    or the AC-5 sink test). Wrote them during build instead, following the
    section's descriptions; confirmed each test's RED state pre-fix via the
    AC-9 revert exercise below rather than via an actual pre-implementation
    run (since the fix and the tests were written in the same session).
  - Call 3's warning lives directly in `sink::encode_to_bytes_with` via
    `eprintln!`, not through the `Image`-carries-a-flag /
    CLI-prints-the-warning pattern `TRUNCATED_JPEG_WARNING`/
    `ANIMATED_INPUT_WARNING` use. DEC-090 (that pattern's proposed
    generalization, the `log` facade) is still `type: recommendation` /
    PROPOSED, not accepted, so there is no installed logger to route
    through; a direct `eprintln!` matches "diagnostics go to stderr"
    (AGENTS §11) and "a one-line diagnostic at the sink" literally, with
    the least new machinery.
  - ⚠ **Added at punch-list (item 5), missing from this list at build:**
    `src/image/mod.rs` is touched — `color_type_bit_depth` widened from private
    to `pub(crate)` so `operation` and `sink` can share it. Full entry under
    "Punch-list deviations" below.
  - `Resize::apply` still widens RGB to RGBA before resizing (does not
    resize the narrower `Rgb8`/`Rgb16` buffer directly, though
    `fast_image_resize`'s `IntoImageView` impls would allow it) — a
    deliberate choice to keep one shared widen/narrow rule across all three
    ops ahead of SPEC-122 landing in the same function. Recorded as an
    Alternative Considered in DEC-095, not a silent omission.
- **Follow-up work identified:**
  - STAGE-042 backlog: the cache-key-vs-version-bump finding (AC-8, above).
  - STAGE-042 backlog / DEC-095 Consequences: lossless WebP has the
    identical 8-bit-only silent-downgrade gap Call 3 documents for
    JPEG/lossy WebP, discovered while driving `web` on a 16-bit source
    (`optimize`'s smallest-candidate search picked WebP over PNG for that
    fixture). Call 3's settled scope does not cover it; filed, not fixed.
  - Corrected the "pipeline is 8-bit throughout" claim (`docs/lab-plan-2026-08.md`
    F8, `docs/roadmap.md`) per the spec's sweep requirement — grep scope:
    `README.md`, `docs/*.md`, `decisions/*.md`, `demo/*.{html,js}`, and
    doc-comment hits (`8.bit`/`bit depth`/`colour type`/`color type`) across
    `src/**`; the `src/` hits (AVIF/HEIC/SVG decoder doc comments) are
    accurate as written and left alone.

### Punch-list cycle (2026-08-18) — verify returned ⚠ PUNCH LIST, seven items

All seven addressed on the same branch. `cycle:` deliberately **left at
`verify`** for re-approval; `advance-cycle` not run; PR #181 not merged.

**1 — The 8-bit-pipeline sweep (urgent half first).** Re-ran verify's grep over
all **798** tracked files and amended **four** live premises, all in
`docs/backlog.md`: `:677-690` (inside **SPEC-122's own entry** — it told a
builder to *"convert back to 8-bit on the way out"*, which would have re-broken
this spec), `:700` (the grading-op stakes paragraph), `:823` (the `.cube` LUT
gate) and `:933-934` (the effects scope guard). **Amended, not deleted, and
deliberately conditional:** SPEC-121 preserves the depth it is *given* and does
not promote, so the quantization worry is now conditional on an 8-bit source
and the transfer-function worry is untouched — both still live for SPEC-122.
The leave-alone list (spec `:115`/`:193`, the build prompt,
`docs/backlog.md:997` and the whole Live-defect section, DEC-095's own
description) was left alone. `STAGE-046:246` now marked ✅ with the grep and
its scope cited inline.

**2 — `watermark --text` (the substantive one) — FIXED.** Reproduced: 66,313 B
RGBA vs 53,970 B RGB on a 256×256 opaque base, **18.6 %** of the file. Root
cause confirmed numerically: `image`'s `Rgba::blend` computes `a_out` in `f32`
and truncates the cast, so `1.0 + a − a` lands on `0.99999994` for **32 of the
254 possible overlay alphas** and writes 254. Fixed structurally, not by
tolerance: `restore_opaque_alpha8`/`…16` restore the alpha the maths requires,
gated on the base having had **no alpha channel at all** — so real transparency
is never touched, and the fix does not depend on `image` truncating rather than
rounding (verify's latent-CI-break concern). New test
`watermark_text_narrows_on_an_opaque_base` drives the float path through
anti-aliased glyph edges.

**3 — The false citation — CORRECTED, and the finding filed where a command
reads it.** The lossless-WebP 16-bit gap is now a `- [ ]` item in **STAGE-042**
(count 8 → 9 pending; `just backlog` confirmed to surface it), and the test
comment cites that instead of a `docs/backlog.md` entry that never existed.
Re-driven while filing: `convert --format webp` reaches it directly (not only
`web` via `optimize`'s search) and prints **`ssim 100.0`** while halving the
depth — SSIM is computed on 8-bit renderings, so the honest-size line reads as
reassurance for the one thing that went wrong.

**4 — `convert_optimize_auto_orient_bytes_unchanged` — WRITTEN.** AC-7's
evidence was a byte diff against `main`'s binary, which a test cannot carry, so
the test pins the property that diff was evidence *for*: the three clean verbs
run no `Operation`, so across six colour types (`L8`, `La8`, `Rgb8`, `Rgba8`,
`L16`, `Rgb16`) each output's colour type and bit depth equal the input's, and
`convert --format png` is byte-identical to `auto-orient`'s output. Re-drove
the literal AC-7 diff this cycle as well: **6 fixtures × 3 verbs, all
byte-identical to `main`**, with `resize` as the positive control (it differs).

**5 — The unlisted deviation — RECORDED.** See Deviations below and DEC-095's
`affected_scope`, which now includes `src/image/mod.rs`. Confirmed after the
edit: `decisions-audit --changed main` now surfaces DEC-095 for that file.

**6 — DEC-095 vs the code: one fixed in code, one fixed in the description.**
  - **Grayscale — FIXED IN CODE.** It is the same narrowing mechanism, so it
    was not scoped out: one extra clause in `narrow_rgba8`/`narrow_rgba16`,
    gated on the input being a luma type. Measured on a 32×32 gradient,
    `resize --max 16`: `L8` **852 → 340 B (−60.1 %)**, `L16`
    **1,559 → 596 B (−61.8 %)**, `La8` **962 → 447 B (−53.5 %)**. The channel
    is taken verbatim from the working buffer rather than through `to_luma8()`,
    whose luminance weights round-trip an already-gray pixel only
    approximately. Two controls added: an `Rgb8` source that happens to be gray
    stays `Rgb8` (the rule preserves, it does not minimise), and a colour
    watermark over a gray base stays RGB (it really did gain chroma).
  - **All-opaque RGBA input — FIXED IN THE DESCRIPTION.** The behaviour is
    right; DEC-095 never stated the `!original_color.has_alpha()` clause. Now
    stated, on both the colour and luma surfaces, with tests
    (`rgba_opaque_input_keeps_its_alpha_channel`,
    `graya_opaque_input_keeps_its_alpha_channel`).

**7 — The two tests that measured the wrong thing — REWRITTEN.** AC-6's test
now runs the op: the same `edit --invert` over the same pixels from an RGB
source and an RGBA source, comparing what the tool writes. It passed
identically on `main` before; it is now **RED on `main`** (verified). The
downgrade warning is built by a pure
`sink::eight_bit_downgrade_warning(ColorType, target)` that reads the depth from
the image, so a 32-bit-float source says "32-bit"; unit-tested at both depths
because `eprintln!` is not capturable in-process.

**8 — Release notes — ADDED** to `CHANGELOG.md` `[Unreleased]`, in both
directions and with the wave's shared lockfile-regeneration note (so
SPEC-122/SPEC-124 slot into the same migration rather than adding their own).
Measured: 8-bit RGB `edit --invert` **1,580 → 1,323 B (−16.3 %)**, 8-bit gray
**2,705 → 906 B (−66.5 %)**, 16-bit gray **2,511 → 2,075 B (−17.4 %)**, and
16-bit colour **1,644 → 3,510 B (+113.5 %)** / `resize --max 16`
**566 → 895 B (+58.1 %)** — restored fidelity, stated as such.

#### Punch-list deviations

- **`src/image/mod.rs` (punch-list item 5).** `color_type_bit_depth` widened
  from private to `pub(crate)` during the build so `operation` and `sink` can
  share it. Correct change, unlisted at build; now in Deviations and in
  DEC-095's `affected_scope`.
- ⚠ **One punch-list instruction could not be satisfied as literally written,
  and the reading is recorded here rather than resolved silently.** Item 2 asks
  both that *"a genuinely opaque composite narrows"* and that *"a genuinely
  translucent overlay must still keep RGBA."* Those are incompatible for an
  alpha-less base: source-over onto a fully opaque base is opaque for **every**
  overlay alpha, and `--text`'s overlay *is* translucent at every anti-aliased
  glyph edge — so under the second rule item 2 has no fix at all. Taken as: the
  **composite** decides, and "keep the control" means **do not introduce a
  numeric tolerance**, which this fix does not (it is exact, and scoped to
  bases with no alpha channel). Consequence:
  `watermark_keeps_alpha_when_the_overlay_is_translucent` was **retargeted**
  onto a base with genuine transparency — the only base from which a composite
  can come out non-opaque — and **strengthened** to assert the transparent
  pixels survive, not merely the channel. The old form passed only because of
  the `f32` truncation artifact this cycle removed (128 is one of the 32
  offending alphas), so it was describing the defect, not the behaviour
  [[a-claim-that-a-test-is-vacuous-needs-driving-too]]. A new test
  (`watermark_translucent_overlay_on_an_opaque_base_narrows`) pins the ruling
  explicitly so it cannot be re-litigated by accident.
- **Not changed, and named so it is not mistaken for an oversight:** `image`'s
  `Rgba::blend` still truncates the *colour* channels (and the alpha, for an
  alpha-bearing base) the same way. That is upstream compositing precision, not
  a narrowing question, and touching it would change watermark output bytes
  beyond this spec's scope.

#### Punch-list negative controls

- **At branch head `4391e06` (pre-punch-list): 6 of 21 RED**, and exactly the
  right 6 — `watermark_text_narrows_on_an_opaque_base`,
  `watermark_translucent_overlay_on_an_opaque_base_narrows`,
  `resize_preserves_grayscale_colour_type`,
  `edit_invert_preserves_grayscale_colour_type`,
  `resize_preserves_sixteen_bit_grayscale`,
  `graya_opaque_input_keeps_its_alpha_channel`. The other 15 stayed green,
  which is what shows the new tests are independent of the ones already fixed.
- **On `main` (`df98118`): 16 of 21 RED**, including the rewritten
  `rgb_output_is_smaller_than_rgba_for_the_same_pixels` — the whole point of
  item 7a, since its previous form was green on `main`. The 5 that stay green
  are the controls and AC-7, all of which describe behaviour `main` already
  had.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Whether the
   "Failing Tests" were actually written to disk during design (they
   weren't — see Deviations). Otherwise the spec was unusually complete:
   Call 4's migration analysis and the two controls (AC-3, AC-9) meant no
   design-level ambiguity about *what* to build, only about test-authoring
   mechanics.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** DEC-090's actual status (PROPOSED, not accepted) — the spec's
   "Implementation Context" lists DEC-090 as a decision "that applies"
   without flagging that it is unaccepted, which shapes how Call 3's
   diagnostic should be wired (see Deviations).
3. **If you did this task again, what would you do differently?** Drive
   AC-8 earlier — the finding it surfaced (cache key needs a version bump
   to change) is exactly the kind of thing that could reshape Call 4's
   framing if found at design time instead of build time. It didn't here
   (Call 4's core claim holds, conditionally), but it easily could have.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
