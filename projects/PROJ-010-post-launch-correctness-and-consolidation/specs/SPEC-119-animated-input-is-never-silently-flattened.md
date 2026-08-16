---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-119
  type: bug
  cycle: verify
  blocked: false
  priority: critical               # silent data loss on shipped verbs, and the
                                   # tool's own linter recommends the command
  complexity: M                    # framed S on the stage; the design sweep
                                   # raised it — the defect is 3 formats, not 1

project:
  id: PROJ-010
  stage: STAGE-046
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # the sweep (the part Sonnet is weakest at) is
                                   # done here and its result is binding below.
                                   # Verify stays Opus.
  created_at: 2026-08-15

references:
  decisions:
    - DEC-085
    - DEC-004
    - DEC-092
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-107
    - SPEC-116

value_link: >
  STAGE-046's first and most urgent item. The tool accepts a valid animated
  file, discards every frame but the first, reports the loss as a 72% win with a
  perfect quality score, and — through `lint` — actively recommends the command
  that does it.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Ran the multi-frame sweep
        the backlog entry asked for (result in Call 3, with its grep), settled
        the refuse-vs-warn call against DEC-085 and STAGE-030, and found the
        carrier for the signal already exists as `Image::truncated_jpeg`.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 135194829
      duration_minutes: 60.1
      recorded_at: 2026-08-16
      tokens_breakdown:
        input: 906
        output: 333061
        cache_creation: 903047
        cache_read: 133957815
      estimated_usd: 48.57
      note: >
        MEASURED — summed .message.usage over 453 assistant messages in this
        session's own transcript, priced per-component at Sonnet anchors
        ($3/$15 per MTok; cache_creation x1.25, cache_read x0.10). Cache reads
        are 99.1% of tokens_total, so the flat 80/20 shortcut would badly
        overstate this — priced by component per the pricing note.
  totals:
    tokens_total: 135194829
    estimated_usd: 48.57
    session_count: 1
---

# SPEC-119: animated input is never silently flattened

## Context

`crustyimg lint anim.gif` warns and tells the user to run a command. That
command destroys their file:

```
$ crustyimg lint anim.gif
anim.gif
  warn format/animated-gif: animated GIF (a modern format encodes far smaller)
    fix: crustyimg convert --format webp anim.gif        <-- the tool's own advice

$ crustyimg convert anim.gif --format webp -o fixed.webp
$ crustyimg optimize anim.gif ; crustyimg web anim.gif
anim.gif: gif → webp · 423 → 118 B (72% smaller) · ssim 100.0
```

Output: **118 B, zero `ANMF` chunks, no `VP8X`** — a *static* WebP. **3 of 4
frames discarded.** Exit 0, no warning, on `convert`, `optimize` **and** `web`.

Driven end to end on 2026-08-15 against a valid 4-frame looping GIF built with
`image`'s own `GifEncoder`. Full record in `docs/backlog.md`,
`## ⚠ Live defect — animated input is silently flattened`. **Read it — the
evidence lives there, not here.**

### Why this outranks the rest of STAGE-046

1. **The linter recommends the destructive command.** The other three defects
   degrade output; this one instructs the user into data loss.
2. **The loss is reported as a win.** "72% smaller" is true only because
   three-quarters of the content was thrown away.
3. **`ssim 100.0` certifies it, and structurally always will.** The score
   compares decoded-source to output, and both are frame 1 — the quantity the
   oracle measures is *preserved by the bug*. This is not a weak check; it is a
   check that cannot see this class at all. **Any test asserting "the score
   stayed high" will stay green through this defect forever**
   [[a-self-referential-control-cannot-detect-a-broken-pipeline]].

### Root cause

Frame decoding exists in exactly one place, and only to count. `gif_is_animated`
(`src/lint/rules.rs:302-310`) builds a `GifDecoder` and takes 2 frames purely to
test `>= 2`. The pixel path never sees frame 2: `Image::from_bytes` →
`decode_with_format` (`src/image/mod.rs:551`) → `decode_with_limits` (`:461`) →
`ImageReader` → **one** `DynamicImage`.

**The linter knows the file is animated and the encoder path does not.**

## Goal

No shipped verb silently discards frames. When crustyimg reduces an animation to
a single frame, it says so on stderr, and `lint` stops recommending a command
that does it.

## The design calls — settled here, not deferred to build

### Call 1 — WARN and proceed. Do not refuse.

The backlog entry left this open ("refuse … or warn loudly"). It is settled as
**warn, exit 0, still write frame 1**, for three reasons:

- **DEC-085 is the precedent and it is directly on point.** SPEC-107 faced the
  same shape — a valid-enough input that decodes to a degraded image — and chose
  warn-don't-fail explicitly. This spec follows it rather than splitting the
  repo's behaviour across two rules.
- **The exit-code surface is frozen (STAGE-030).** Refusing turns commands that
  exit 0 today into failures on three shipped verbs.
- **PROJ-007 lockfiles reference those outputs.** A new hard failure breaks
  existing builds for a file the user may well have wanted flattened.

**Exit 4 would be the wrong code anyway.** Per `docs/api-contract.md`, 4 is
"unsupported or undetectable format" in three senses — unrecognisable bytes, or a
decoder/encoder not built, whose messages *name a feature to rebuild with*. An
animated GIF is recognised, decodable, and has no feature to name.

> **✅ CONFIRMED by the maintainer, 2026-08-16** — *"ok to the warn and proceed, even though I
> don't love it."* The reservation is recorded because it is legitimate: warning and proceeding
> still destroys the animation, it just narrates the destruction. A user piping `convert` over a
> directory without reading stderr loses their frames exactly as they do today.
>
> **What answers the reservation: the strict path already exists, in the right verb.** `lint`
> carries severities and a failing mode — `src/lint/mod.rs:49-53` (`Error` "fails CI", `Warn`
> "fails only under `--max-warnings`") and `src/cli/report.rs:474`, where *≥1 `Error` finding, or
> a warn count over `--max-warnings`, exits non-zero*. So
> **`crustyimg lint --max-warnings 0` already fails on an animated GIF today.**
>
> That is a cleaner separation than making `convert` refuse: **`lint` is the gate, `convert` is
> the tool.** A CI pipeline that must never flatten an animation has an existing, documented way
> to say so, and the frozen exit-code surface stays frozen.
>
> **This strengthens Call 1 and it REVISES Call 4 — see below.**

### Call 2 — The signal rides on `Image`, exactly like `truncated_jpeg`.

`Image` already carries a per-decode degradation flag for precisely this purpose
(`src/image/mod.rs:174`), set at decode and turned into a stderr warning by the
CLI layer. **Add a sibling field, do not invent a mechanism.**

This is deliberate leverage: SPEC-107 built the pattern, and SPEC-116 (just
merged to `verify`) finished threading it through the last verb. **The emit sites
already exist** — `src/cli/ops.rs:336`, `:431`, `src/cli/optimize.rs:1473`,
`:1522`, and `src/cli/build.rs`'s Decide arm. This spec adds a second condition
at those same sites, not a new plumbing route.

Constructors that cannot produce a multi-frame source (`raw_preview`, SVG
rasterization) set it `false` with the same one-line rationale
`truncated_jpeg: false` carries at `:579`.

### Call 3 — THREE formats are affected, not one. This sweep is binding.

The backlog entry asked for a mechanical sweep and flagged that
`format/animated-gif` "may itself be too narrow." **It is.** Swept at design
against the pinned dependency:

```
$ grep -rn "impl.*AnimationDecoder.*for" \
    ~/.cargo/registry/src/index.crates.io-*/image-0.25.10/src/
.../src/codecs/gif.rs:426:          impl<'a, R: BufRead + Seek + 'a> AnimationDecoder<'a> for GifDecoder<R>
.../src/codecs/png.rs:514:          impl<'a, R: BufRead + Seek + 'a> AnimationDecoder<'a> for ApngDecoder<R>
.../src/codecs/webp/decoder.rs:104: impl<'a, R: 'a + BufRead + Seek> AnimationDecoder<'a> for WebPDecoder<R>
```

| format | enabled in this repo | multi-frame | cheap detection API |
|---|---|---|---|
| GIF | `image/gif` ✓ | ✓ | `into_frames().take(2).count() >= 2` (already used) |
| **APNG** | `image/png` ✓ | ✓ | **`PngDecoder::is_apng() -> ImageResult<bool>`** (`png.rs:160`) |
| **animated WebP** | `image/webp` ✓ | ✓ | **`WebPDecoder::has_animation() -> bool`** (`decoder.rs:31`) |
| TIFF | `image/tiff` ✓ | ✗ no `AnimationDecoder` impl; multi-page not exposed | — |
| ICO | `image/ico` ✓ | ✗ multi-*size* container, not animation | — |
| BMP / JPEG | ✓ | ✗ single-frame | — |

**Two of the three have cheaper detection than GIF's** — a boolean, with no
frame decode — so "detection costs a decode" is not an argument against covering
them.

**AVIF is the honest gap.** AVIF sequences exist, but crustyimg decodes AVIF
through `re_rav1d` (`src/image/avif.rs`), not `image`'s codec, so
`AnimationDecoder` does not apply and this sweep says nothing about it.
**Determine whether the `re_rav1d` path can receive a sequence and report the
finding. If you cannot settle it, say so in Build Completion and state that AVIF
is therefore unproven** — do not quietly imply coverage this spec did not earn.

### Call 4 — `lint`'s advice becomes honest; the rule is NOT renamed.

`format/animated-gif`'s `fix:` string (`src/lint/rules.rs:234-255`) currently
recommends the destructive command. **Until animated output exists there is no
correct fix**, so the finding keeps warning (an animated GIF genuinely is large)
and its `fix:` stops naming a command that loses data — say plainly that
crustyimg cannot yet re-encode animation without flattening it.

**REVISED 2026-08-16, after Call 1 was confirmed.** The original text said "do not rename or
broaden the rule ID — the asymmetry is intentional." That was written when `lint` was incidental
to this spec. It no longer is.

Call 1's confirmation makes `lint --max-warnings 0` **the designated strict path** for users who
cannot tolerate a silent flatten. If the rule only detects GIF, then **APNG and animated-WebP
users have no strict option at all** — the pixel verbs warn and proceed for them, and the gate
that would have caught it is blind. That is no longer a tidy-up; it is a hole in the answer this
spec gives the maintainer's reservation.

So: **the rule must cover what the defect covers — all three formats from Call 3.** The rule
*ID* is still a user-visible surface, so:

- **Keep `format/animated-gif` firing for GIF** so existing config and output do not break.
- **Add coverage for APNG and animated WebP.** Whether that is a broadened rule under a new
  id (e.g. `format/animated-input`) with `format/animated-gif` kept as an alias, or two sibling
  rules, is a **build-cycle call** — make it, justify it in Build Completion, and note whether any
  config surface (`.crustyimg-lint.toml` or equivalent) needs a migration note.
- **The `fix:` string stops naming a destructive command** for all of them. Until animated output
  exists there is no correct fix; say plainly that crustyimg cannot yet re-encode animation
  without flattening it.

If broadening turns out to need a config migration, **stop and report** rather than shipping one
inside this spec.

## Inputs

- **Files to read:** `src/image/mod.rs:164-178` (the `Image` fields and the
  `truncated_jpeg` precedent), `:461` (`decode_with_limits`, the seam), `:551`;
  `src/lint/rules.rs:232-310`; `src/cli/ops.rs:328-340` (the emit shape);
  `src/image/avif.rs` for Call 3's open question.
- **`docs/backlog.md`**, the animated-input entry — the driven evidence.
- **DEC-085** (warn-don't-fail) and **DEC-004** / `docs/api-contract.md`'s exit
  table (why not 4).
- **Fixtures:** build the animated GIF natively with `image`'s `GifEncoder`, the
  way `src/lint/rules.rs:361` already does. **Do not commit a binary fixture** and
  do not shell out. APNG and animated WebP fixtures must likewise be generated,
  or the format declared untested — see AC-7.

## Outputs

- **Files modified:** `src/image/mod.rs` (the field + detection), `src/lint/rules.rs`
  (the `fix:` string), the four CLI emit sites, `tests/` (new + `tests/lint.rs`).
- **New exports:** an accessor mirroring `is_truncated_jpeg`.
- **New DEC expected:** yes — one recording Call 1 (warn, not refuse) and its
  reasoning, since it sets policy for a whole input class. `affected_scope`
  covering `src/image/**` and `src/cli/**`.

## Acceptance Criteria

- [x] **AC-1.** An animated GIF through `convert`, `optimize` and `web` **warns on
      stderr**, naming the input and saying frames were discarded. Assert on the
      message, not on non-empty stderr.
- [x] **AC-2.** **Exit stays 0 and frame 1 is still written.** Per Call 1.
- [x] **AC-3.** **Not `--quiet`-gated**, matching DEC-085 and its sibling. Pinned
      by a test, because the adjacent cache warning *is* quiet-gated.
- [x] **AC-4.** **A static GIF produces no warning.** The did-not-break-it
      control — without it, "always warn" passes AC-1 and ruins the verb.
      [[a-harness-that-exercises-nothing-reports-green]]
- [x] **AC-5.** **APNG and animated WebP warn too**, per Call 3, with their own
      tests and their own static controls.
- [x] **AC-6.** **The assertion is structural, never the quality score.** Assert
      the discarded frames directly — count `ANMF` chunks / decode the output's
      frame count. A test that asserts "ssim stayed high" is **vacuous by
      construction** here and will be rejected.
- [x] **AC-7.** **`lint`'s `fix:` string no longer names a command that discards
      frames**, pinned by a test asserting the absence, not just the new text.
- [x] **AC-7b.** **`lint` detects all three animated families**, per the revised Call 4, and
      **`lint --max-warnings 0` exits non-zero on each** — the strict path is the answer this
      spec gives to "warn and proceed still loses data", so it must be driven, not assumed.
      A static counterpart of each family stays clean.
- [x] **AC-8.** **Byte output is unchanged** for every input that is not
      multi-frame. This spec adds a diagnostic; it must not perturb encoding.
      Compare against `main`'s binary, not against a sibling verb on the same
      branch — a same-branch cross-check cannot see a change that moved both.
      [[fixtures-from-the-code-under-test-cannot-fail]]
- [x] **AC-9.** **A negative control per format**: revert the detection for GIF,
      APNG and WebP **independently**; confirm each format's test goes RED and
      the static controls stay green. **Three controls, not one coarse revert** —
      three detection sites are three independent claims. Prove each revert
      reached the built artifact.
      [[reverting-source-does-not-rebuild-the-binary]]
- [x] **AC-10.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run
      **sequentially**, through `rtk proxy` from the first leg: default,
      `--no-default-features`, `--features webp-lossy`. Clippy and `fmt --check`
      each. **Establish your own `main` baseline.** Then read the CI legs
      individually.

## Failing Tests

Written during **design**, BEFORE build. **All of the AC-1/AC-5 tests FAIL on
today's `HEAD`** — this is a real defect, so there is a genuine red-to-green
transition and `test-before-implementation` applies in its usual form.

- **`tests/hostile_inputs.rs`** (or a new `tests/animated_inputs.rs` — say which
  and why)
  - `"animated_gif_warns_on_every_pixel_verb"` — AC-1. **FAILS today.**
  - `"animated_gif_still_writes_frame_one_and_exits_zero"` — AC-2. Passes today;
    pins that the warning did not become a failure.
  - `"animated_warning_survives_quiet"` — AC-3. **FAILS today.**
  - `"static_gif_emits_no_animation_warning"` — AC-4. Passes today; the control.
  - `"apng_warns_on_every_pixel_verb"` — AC-5. **FAILS today.**
  - `"animated_webp_warns_on_every_pixel_verb"` — AC-5. **FAILS today.**
  - `"static_png_and_static_webp_emit_no_animation_warning"` — AC-5's controls.
  - `"animated_output_frame_count_is_asserted_structurally"` — AC-6.
- **`tests/lint.rs`**
  - `"animated_gif_rule_does_not_recommend_a_flattening_command"` — AC-7.
    **FAILS today.**
- **Negative controls** (AC-9, run and recorded, not committed) — per format.

## Implementation Context

### Decisions that apply
- **DEC-085** — warn-don't-fail, unconditional gating. Binding on Calls 1 and the
  AC-3 gating.
- **DEC-004** / `docs/api-contract.md` exit table — why exit 4 is not this.

### Constraints that apply
- `test-before-implementation` (**blocking**) — several tests red on `HEAD`.
- `clippy-fmt-clean`, `one-spec-per-pr`, `no-unwrap-on-recoverable-paths`
  (detection must not `unwrap` — `is_apng()` returns `ImageResult`).

### Prior related work
- **SPEC-107** — created the warn-don't-fail pattern and `Image::truncated_jpeg`.
- **SPEC-116** — threaded that flag through the last verb. **Read its diff**: the
  emit sites this spec extends are exactly the ones it touched, and its
  `apply`-vs-`build` string-equality test is the right shape for AC-1.

### Out of scope
- **Animated output** (animated WebP/AVIF encode). That is the capability; this spec closes the
  destructive path so that work can later make the linter's advice true.

  > **CORRECTED 2026-08-16.** This section previously read *"`webp-animation` v0.10.0
  > (MIT OR Apache-2.0) is verified and filed separately."* **Both halves were wrong**, and the
  > error was inherited rather than made here: a correction (`7ca85a2`) was pushed to PR #170's
  > branch after GitHub captured its head, so the squash merge dropped it silently, and this spec
  > was authored against the resulting `main`.
  >
  > - **The licence is right, the verdict is not.** `webp-animation` 0.10.0 depends on
  >   `libwebp-sys2 ^0.2`, **non-optional** — checked against the crates.io API, not the triage
  >   note. It is a C wrapper, so it does **not** clear `pure-rust-codecs-default` and would need
  >   an off-by-default feature on the `webp-lossy`/DEC-022 precedent.
  > - **It was filed nowhere.** `docs/roadmap.md` contained zero occurrences of "animated"
  >   (positive control: `crop` = 8).
  >
  > A **pure-Rust route needs no new dependency**: `image`'s `AnimationDecoder` for frames
  > (already used at `src/lint/rules.rs:303` to *count* them), the existing `Pipeline` run once
  > per frame, `image-webp` 0.2.4 already in-tree for encode, and an in-house `VP8X`/`ANIM`/`ANMF`
  > RIFF mux estimated at 150–250 lines. **Animated AVIF is a separate, unpriced item** — a HEIF
  > image sequence, where the container is the gap, not the encoder (`rav1e` 0.8.1 is already in
  > the tree and encodes multiple frames).
  >
  > **None of this changes SPEC-119's scope** — animated output stays out. It changes what the
  > follow-up is, so this spec should not hand the next reader a dependency decision that was
  > never verified.
- **Renaming/broadening the lint rule ID** — Call 4.
- The other three STAGE-046 defects. They share `Resize::apply` and a lockfile
  blast radius; this one shares neither.

## Notes for the Implementer

- **The score cannot be your oracle.** AC-6 exists because the obvious assertion
  is the one that structurally cannot fail. Count frames.
- **Three detection sites are three claims** — AC-9 wants three reverts. One
  coarse revert shipped a vacuous test on SPEC-113.
- **Do not flip Call 1.** If you believe refuse is right, say so and stop; it
  changes the exit-code surface and that is not a build-cycle decision.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
- **Checkpoint early** — push a WIP once it compiles, before the matrix.
- macOS has no `timeout(1)`. `git commit -s` (DCO). **Own git worktree.** **Do
  not merge the PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md`, including
  `just advance-cycle SPEC-119 verify` — and confirm the `cycle:` line moved.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-119-animated-input-never-silently-flattened`
- **PR:** [#176](https://github.com/jysf/crustyimg/pull/176) (open, not merged, per prompt instructions)
- **All acceptance criteria met?** yes — all 11 (AC-1 through AC-10, including AC-7b).
- **New decisions emitted:** DEC-092 (`decisions/DEC-092-animated-input-warns-and-proceeds-lint-is-the-strict-gate.md`)
  — Call 1 (warn-and-proceed, DEC-085's sibling) and Call 4's separate-rule-id choice,
  `affected_scope` covering `src/image/**`, `src/cli/**`, `src/lint/**`, and the new tests.
- **AVIF (Call 3's open question): PROVEN SAFE, not merely unproven.** Read
  `avif-parse` 2.1.0's `read_avif` source directly (not inferred): it checks the
  `ftyp` major brand before touching `meta`/`iloc`, and an animated-AVIF sequence
  (`avis`) is rejected with a typed `Error::Unsupported("Animated AVIF is not
  supported. Please use real AV1 videos instead.")` — a hard decode error, before
  any pixel allocation. `Image::from_bytes` therefore never constructs an `Image`
  from an animated AVIF at all. This is a stronger finding than the spec asked
  for ("determine whether the path can receive a sequence") — it does not merely
  fail to receive one, it is refused with an actionable message. No code change
  was needed on this path; `decode_avif_inner`/`map_parse_err` already surface it
  correctly.
- **Lint rule id (Call 4's build-cycle choice): two sibling rules, not a
  broadened rule with an alias.** `format/animated-gif` is untouched (still
  GIF-only, existing config/output unaffected) and a new `format/animated-input`
  covers APNG + animated WebP. Reasoning in the `AnimatedInput` doc comment
  (`src/lint/rules.rs`) and in DEC-092: an alias would need config-layer
  resolution machinery (`known_rule_ids`, `select`/`ignore`/`severity`) that does
  not exist today; two single-format rules need none, and no config migration.
- **Detection is centralized, not duplicated.** `src/image/mod.rs`'s
  `detect_animated_input`/`gif_is_animated`/`png_is_apng`/`webp_is_animated` are
  the ONE place all three formats are detected — `Image::from_bytes` sets the
  flag at decode time, and `lint`'s `AnimatedGif`/`AnimatedInput` rules read
  `target.decoded().ok()?.is_animated_input()` (the same flag) instead of
  re-decoding. This directly answers the spec's stated root cause ("the linter
  knows the file is animated and the encoder path does not") — after this
  change there is exactly one detector, consulted from both places, so they
  cannot disagree. The AC-9 negative controls confirm the three format
  detectors are still three independent claims (reverting one flips exactly
  its own tests red, both the pixel-verb warning tests AND the lint tests for
  that format, and leaves the other two formats' tests green).
- **Deviations from spec:**
  - The spec's Call 2 text names `raw_preview` **and "SVG rasterization"** as
    the two constructors that must set the flag `false` explicitly. In the
    implementation, only `raw_preview` needs an explicit `animated_input: false`
    — it is a separate constructor that never calls `detect_animated_input`. SVG
    (and HEIC) do NOT need a special case: `Image::from_bytes` computes the flag
    generically from `(original bytes, resolved source_format)` for every decode
    path, and since SVG/HEIC bytes are never valid GIF/PNG/WebP bytes, all three
    format-specific detectors fail-safe to `false` on them through their normal
    `Err(_) => false` decode-error arm — the same mechanism that makes a corrupt
    file read as "not animated" rather than needing a dedicated branch. Same
    effect as the spec described, fewer special cases; noted here so a future
    reader does not go looking for an SVG-specific `animated_input: false` that
    was never needed.
  - `optimize_decide_one`'s return tuple grew a 5th element and tripped
    clippy's `type_complexity` lint at the existing 4-tuple's threshold; added
    `#[allow(clippy::type_complexity)]` with a one-line rationale rather than
    introducing a named struct (which would still thread the same number of
    fields through 3 call sites for no clarity gain) — matches the existing
    `#[allow(clippy::too_many_arguments)]` precedent elsewhere in the same file.
- **Follow-up work identified (found, not fixed — out of this spec's scope):**
  - **`src/source/mod.rs`'s `IMAGE_EXTENSIONS` allow-list is missing `webp`.**
    WebP decode is a default-feature, fully supported input format, but
    directory/glob discovery (used by `lint <dir>`, `convert dir/*`, etc.)
    silently skips `.webp` files — a directory containing only a `.webp` file
    resolves to zero inputs (`lint` exits 3, "no resolvable inputs"). This is
    NOT a SPEC-119 regression (reproduces identically on `main`); discovered
    while writing AC-7b's WebP test, which had to lint the fixture by its own
    single-file path (never extension-filtered) to work around it. Recorded in
    DEC-092's Consequences; `tests/lint.rs`'s two SPEC-119 tests document the
    workaround inline. Fixing `IMAGE_EXTENSIONS` changes discovery behavior for
    every command, not just these two rules, so it deserves its own spec.
  - `docs/roadmap.md`'s "Engine-backed lint rules" row (line ~65) says "10
    shipped rules" — now 11 after `format/animated-input`. Its "Animated
    output" row (line ~70) still frames the tool as flattening animations
    "silently" — after this spec, the flatten is no longer silent (it warns),
    though the animated-*output* capability itself is still unbuilt exactly as
    that row describes. Neither line is load-bearing for this spec's
    acceptance criteria; left for the ship-cycle doc pass rather than touched
    here, matching this repo's practice of not scope-creeping into roadmap
    prose mid-build.
  - `docs/research/proj-002-design-lint.md`'s rule table (a PROJ-002 design
    snapshot, not a maintained catalog) still shows `format/animated-gif`'s
    old `fix:` — not updated; it is historical record, not a live contract.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** The fixture
   guidance ("build them natively... or declare untested") did not anticipate
   that `image` 0.25 has NO encode-side APNG or animated-WebP API at all (only
   decode) — so "native" fixture construction meant hand-assembling RIFF/PNG
   container chunks byte-by-byte against the `image-webp`/`png`-crate parsers'
   exact acceptance rules, which took real reverse-engineering (reading
   `image-webp` 0.2.4's `decoder.rs`/`encoder.rs` and the APNG chunk spec) that
   the prompt's one-line pointer ("the way `rules.rs:361` already does") did
   not prepare for — that precedent is GIF-only, where `image` DOES have an
   encoder. This was the single largest time sink in the build.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** The `IMAGE_EXTENSIONS`-missing-`webp` gap (above) — it directly
   shaped how AC-7b's WebP test had to be written, and nothing in the spec or
   its Notes anticipated that `lint <directory>` and `lint <file>` could
   behave differently for the exact format this spec adds coverage for.
3. **If you did this again, what would you do differently?** Read
   `avif-parse`'s `read_avif` and `image-webp`'s `decoder.rs`/`encoder.rs`
   source FIRST, before writing any detection or fixture code — both the AVIF
   finding and the APNG/WebP fixture feasibility turned entirely on exact
   upstream-crate wire-format details that no amount of API-surface reading
   (docs, signatures) would have surfaced; only reading the parser bodies did.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
