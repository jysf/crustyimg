---
# ═══════════════════════════════════════════════════════════════════════════
# DRAFT / IDEA — NOT COMMITTED WORK. NOT SCHEDULED. NOT ATTACHED TO A STAGE.
#
# This is a research draft in `docs/research/`, which is deliberately OUTSIDE
# the work hierarchy (AGENTS.md §2: a spec belongs to one stage in one project
# and lives in `projects/PROJ-*/specs/`). Nothing here is claimed, assigned, or
# in flight. Follows the `draft-stage-linear-working-buffer.md` precedent.
#
# The task/project/stage ids below are placeholders ON PURPOSE. Do not allocate
# a real SPEC number, and do not file this under a project, until someone
# decides to schedule it. If you are a future session reading this: this is an
# IDEA WITH EVIDENCE, not a backlog item. Treat the measurements as real and
# the plan as a proposal.
# ═══════════════════════════════════════════════════════════════════════════

task:
  id: SPEC-XXX                     # unassigned on purpose
  type: story
  cycle: null                      # not entered; a draft has no cycle
  status: idea                     # NOT one of frame|design|build|verify|ship
  blocked: false
  priority: null
  complexity: L                    # L means split it — see "Splitting" below

project:
  id: PROJ-XXX                     # unassigned on purpose
  stage: STAGE-XXX                 # unassigned on purpose — see Placement
repo:
  id: crustyimg

agents:
  drafted_by: claude-opus-5
  created_at: 2026-08-16

references:
  decisions: [DEC-088, DEC-091, DEC-068, DEC-018, DEC-004]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - decode-once-no-per-op-disk
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: []

cost:
  sessions: []
  totals: { tokens_total: 0, estimated_usd: 0, session_count: 0 }
---

# DRAFT-SPEC: animated input → animated AVIF output

> **Status: IDEA.** Not scheduled, not owned, not attached to a stage. Every number
> below was measured on 2026-08-16 (see `docs/backlog.md` and
> `docs/probes/animated-gif-to-av1/`); the *plan* is a proposal.

## Context

crustyimg currently accepts animated input on the pixel path and silently keeps frame 1.
That defect is recorded in `docs/backlog.md` and is **already owned elsewhere** — this draft
does **not** claim it and does not propose changing anyone's scope. What this draft covers
is the separable capability: **producing animated output**, which is what would eventually
make the linter's advice true.

Three things were measured that make this worth writing down now:

1. **The whole path is pure Rust and patent-clear.** Driven end to end out-of-crate, exit 0:
   `AnimationDecoder` → `Image::from_parts` → crustyimg `Pipeline` (registered op, per frame)
   → `rav1e` → decoded back through `re_rav1d` with the frame count verified. AV1 is
   royalty-free by design; GIF/APNG/WebP carry no patent load. No C, no ffmpeg, no new
   system dependency.
2. **The win is an order of magnitude.** A 308,156 B / 36-frame GIF → **27,564 B at
   SSIMULACRA2 86.7** via rav1e (11.2×). Animated WebP's best measured point was
   172,492 B at 84.1 — **AVIF is 6.3× smaller at higher quality**.
3. **AVIF is also the easier pure-Rust build**, which inverts the going-in assumption:
   `rav1e` already encodes and `mp4-atom` already carries `av01`/`av1C`, whereas **there is
   no pure-Rust animated WebP encoder at all** (`image-webp` 0.2.4 writes `VP8X` at
   `src/encoder.rs:722` but emits no `ANIM`/`ANMF`).

## Goal

Given a multi-frame input, emit an **animated AVIF** that preserves every frame and its
timing, sized by the existing perceptual quality search rather than a fixed quantizer.

## Placement — deliberately unresolved

Under DEC-091 this reads workhorse on both fences (Fence A: quality target, loop count and
frame rate are invariant across a batch; Fence B: the output is a delivery artifact a build
consumes). **That is an argument, not an assignment.** Whoever schedules this decides where
it lands; this draft does not.

## Inputs

- **Files to read:** `src/image/avif.rs` (the existing decode path, incl. `alpha_item`
  handling and the `re_rav1d` thread/stack discipline) · `src/quality/mod.rs:255` (the
  search this must reuse) · `src/sink/mod.rs:48` (`AVIF_SPEED`) · `src/operation/mod.rs:137`
  (`Operation::apply`, pure, by value)
- **New dependency (needs a DEC):** `mp4-atom` 0.15.0, MIT OR Apache-2.0, pure Rust, no
  `-sys` crate. Supplies the ISO-BMFF box tree incl. `av01`/`av1C` and the full sample table.
- **Already in-tree:** `rav1e` 0.8.1 (BSD-2, via `ravif`), `re_rav1d` 0.1.3 (BSD-2).

## Outputs

- **Files created:** an animated-AVIF writer module; an integration test file.
- **Files modified:** the sink/format plumbing to route multi-frame output; `Cargo.toml`.
- **New exports:** an encode entry point taking frames + per-frame durations + loop count.

## Acceptance Criteria

- [ ] **AC-1. Every frame survives.** A GIF with N frames produces an AVIF whose decoded
      frame count is N, asserted with an **independent decoder** (`re_rav1d`), not by
      trusting the encoder's packet count.
      [[verify-wasm-output-with-an-independent-decoder]]
- [ ] **AC-2. The structural assertion runs before any pixel assertion**, and no test may
      substitute a perceptual score for it. Measured: a pipeline that dropped 3 of 8 frames
      scored SSIMULACRA2 **100.0**. A "score stayed high" test is **vacuous by construction**
      here. [[a-self-referential-control-cannot-detect-a-broken-pipeline]]
- [ ] **AC-3. Frame order is asserted**, not just the count — N correct frames in the wrong
      order passes AC-1.
- [ ] **AC-4. Per-frame timing is preserved.** GIF delays are per-frame in 1/100 s; ISO-BMFF
      needs a timescale plus an `stts` that can carry *unequal* durations. A fixture with
      **deliberately unequal** frame delays is required — a uniform-delay fixture cannot
      distinguish a correct `stts` from a hardcoded one.
- [ ] **AC-5. Loop count round-trips** (GIF `NETSCAPE` extension → sequence repeat count).
- [ ] **AC-6. The still fallback is written.** Output carries **both** a `meta` box (still
      primary item) and a `moov` box, with `avis` major brand and `avif`/`mif1` among the
      compatible brands. Verified structurally by walking top-level boxes. **This is not a
      nicety:** browsers exist that support still AVIF but not sequences, and they accept
      `<source type="image/avif">` and then cannot animate — the primary item is what makes
      that degrade to a poster frame instead of breaking.
- [ ] **AC-7. Quality is searched, not pinned.** The existing search at
      `src/quality/mod.rs:255` drives the quantizer per sequence. Measured justification: at
      one fixed quantizer, two real sources scored 86.7 and 76.1 — a pinned value cannot
      serve both.
- [ ] **AC-8. Speed is NOT taken from the still-image knob.** Pin `AVIF_SPEED = 6` for
      sequences and add a test that fails if the wasm speed knob (SPEC-079/DEC-068) reaches
      this path. Measured: at a fixed quantizer, **speed 10 produced a 37% LARGER file at
      lower quality** (38,061 B / 84.4 vs 27,705 B / 86.8) because it guts motion estimation —
      the opposite of DEC-068's still-image finding.
- [ ] **AC-9. Alpha is refused, loudly, not silently flattened.** v1 is opaque-only. A
      transparent animated input must warn (or exit 4 per the `CodecNotBuilt` precedent) and
      must not emit an opaque AVIF that silently discards transparency — that would be the
      same class of defect this capability exists to end. **Not hypothetical:** the APNG
      fixture used in the format sweep decoded to `(0,0,0,0)`.
- [ ] **AC-10. A static input is unaffected**, byte-for-byte, compared against `main`'s
      binary. The did-not-break-it control.
      [[fixtures-from-the-code-under-test-cannot-fail]]
- [ ] **AC-11. Negative controls, run and recorded**: revert the frame-loop so only frame 1
      is encoded and confirm AC-1 goes RED; revert the `stts` duration write and confirm AC-4
      goes RED. Prove each revert reached the built artifact.
      [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-12. Clean full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run sequentially:
      default, `--no-default-features`, `--features webp-lossy`. The lean build has no AVIF
      encoder at all, so the multi-frame path must exit 4 there, not panic.
      [[verify-includes-lean-no-default-features-build]]

## Failing Tests

Written during design, before build.

- **`tests/animated_output.rs`** (new)
  - `"animated_gif_round_trips_every_frame"` — AC-1
  - `"animated_output_frame_order_is_asserted"` — AC-3
  - `"unequal_frame_delays_survive_as_unequal_stts_entries"` — AC-4
  - `"loop_count_round_trips"` — AC-5
  - `"sequence_carries_a_still_primary_item"` — AC-6
  - `"quality_search_picks_different_quantizers_for_different_sources"` — AC-7
  - `"sequence_encode_ignores_the_wasm_speed_knob"` — AC-8
  - `"transparent_animation_warns_and_does_not_emit_opaque_output"` — AC-9
  - `"static_input_bytes_are_unchanged"` — AC-10
  - `"multi_frame_output_exits_4_on_the_lean_build"` — AC-12

## Implementation Context

### Decisions that apply
- **DEC-091** — the two fences; the placement argument above.
- **DEC-088** — tier model; no external process may be spawned to do any of this.
- **DEC-068 / SPEC-079** — the rav1e speed knob this path must *not* inherit (AC-8).
- **DEC-018** — the licence gate `mp4-atom` must clear (`just deny`).
- **DEC-004** — exit 4 for a codec not built in (AC-12).

### Constraints that apply
- `pure-rust-codecs-default` — **satisfied without a feature gate**, which is unusual and
  worth stating: unlike `webp-lossy`/`heic`, no C dependency is needed.
- `no-new-top-level-deps-without-decision` — `mp4-atom` needs a DEC before adoption.
- `decode-once-no-per-op-disk` — frames stay in memory; no per-frame temp files.
- `untrusted-input-hardening` — a frame count from a hostile container is an allocation
  vector. Bound total frames and total decoded pixels **before** decoding, mirroring
  `check_caps` in `src/image/avif.rs`.

### Out of scope for this draft
- Alpha / transparency (AC-9 refuses it; that is v2).
- Animated **WebP** output — blocked on there being no pure-Rust encoder.
- Reading video containers, or anything that decodes a codec crustyimg does not already ship.
- The input-side defect (silently keeping frame 1) — owned elsewhere, not claimed here.
- `lint` rule changes.

### Splitting
Marked complexity **L**, which per the template means split it. A plausible cut:
(a) the muxer + frame/timing round-trip (AC-1..AC-6), (b) the quality search + speed
pinning (AC-7, AC-8), (c) alpha (deferred entirely).

## Notes for whoever picks this up

- **Measured muxer price: ~1,000 lines** for a muxing *driver* on top of a box library
  (`mp4` 0.14's `writer.rs` + `track.rs`), not the 150–250 the in-house RIFF estimate
  suggests. Single-track/no-audio is less, but budget honestly.
- **The colour-range trap.** An AVIF whose quality score is *insensitive to the quality knob*
  has a range/matrix bug, not a codec problem. A near-lossless encode scoring 57.2 was traced
  to `Range: Limited` against full-range input; the same control scored 96.5 after the fix.
  Run a near-lossless control first and require it to score high.
  [[a-flat-quality-curve-means-a-colour-bug-not-a-codec]]
- `mp4-atom` is deliberately low-level — its README says it does "encoding/decoding of the
  binary format without validation or interpretation... You have to know what boxes to
  expect!" It gives you boxes, not a muxer. The sample-table bookkeeping is yours.
- Browser support is a **moving** fact, re-check before shipping: caniuse's 94.65% measures
  **still** AVIF and does not break out sequences, so treat it as an upper bound. Its notes
  [2] and [5] (animation unsupported / behind a pref) apply to older Firefox and iOS Safari
  versions, not to current releases.
