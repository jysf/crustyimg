---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-024                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.85                   # 0.0 - 1.0, honest assessment
  audience:                          # who needs to know?
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-06-17
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/cli/mod.rs

tags:
  - cli
  - optimize
  - perceptual-quality
  - ergonomics
  - command-shape
---

# DEC-024: optimize command shape — default visually-lossless, auto-orient + strip, format/size-preserving

## Decision

`crustyimg optimize <inputs…>` is the one-button "web-good" command: with no flags
it runs a fixed pipeline of **auto-orient** (bake EXIF orientation, then drop the
metadata bundle) and re-encodes each input to a **perceptual visually-lossless
target** (SSIMULACRA2 score 90, the existing `QualityTarget::VisuallyLossless`),
**preserving the input format and dimensions**. `--target`/`--ssim` override the
perceptual target, `--max-size` switches to a byte budget, `--max N` optionally
bounds the long edge, and `-o`/`--format` pick the output format. It is pure
composition of shipped primitives (`auto-orient` op, `run_pixel_op`,
`resolve_effective_quality`, `src/quality`) with **no new dependency**. Cross-format
auto-negotiation (auto-pick the smallest of JPEG/WebP/AVIF) is **explicitly
deferred** to a later spec.

## Context

STAGE-008 shipped the outcome-driven engine (perceptual auto-quality, byte budgets,
modern formats), but it lives as flags spread across `shrink`/`convert`. STAGE-009's
thesis is to make that engine *legible*: the differentiator becomes real when a user
runs one short command and gets the right result (the roadmap's "optimize one-button"
item, 2026-06-16 handoff). The design questions this DEC settles:

1. What does `optimize` do by default, and how is it distinct from `shrink`?
2. Does it resize? Does it change format?
3. Does it auto-negotiate the best format?
4. What exactly does "privacy by default" mean given the metadata lanes aren't built?

Constraints in play: `ergonomic-defaults` (the no-flag case must be right);
`pure-rust-codecs-default` + the AVIF decoder gap (DEC-020 — AVIF has no decoder, so
the perceptual search can't score AVIF round-trips); DEC-003 (the selective-preserve
container metadata lane is unbuilt, STAGE-004); DEC-016/017/019 (quality + auto-orient
+ perceptual policies to reuse).

## Alternatives Considered

- **Option A: `optimize` = `shrink` with a perceptual default.**
  - What it is: alias shrink's resize-centric behavior (default long-edge bound
    1600, fixed q80) but default to a perceptual target.
  - Why rejected: it silently resizes (surprising under the word "optimize" — the
    user may want full resolution at fewer bytes), it does NOT auto-orient (so phone
    photos serve sideways after the orientation tag is stripped), and it makes
    `optimize` a near-duplicate of `shrink`. The two commands need distinct
    identities.

- **Option B: `optimize` auto-negotiates the format (try JPEG/WebP/AVIF, write the
  smallest at equal quality).**
  - What it is: the "format auto-negotiation" gap (#7 in the competitive ranking).
  - Why rejected (for v1, not forever): AVIF/WebP-lossy are feature-gated, so the
    behavior would silently differ by build; the perceptual search **cannot score
    AVIF** (no decoder is built — DEC-020), so equal-quality auto-negotiation across
    AVIF isn't possible yet; and the output-extension/format semantics get murky when
    the user passed `-o photo.jpg`. It deserves its own spec once AVIF decode lands.

- **Option C (chosen): `optimize` = auto-orient + strip + perceptual
  visually-lossless, format/size-preserving, with outcome overrides.**
  - What it is: a fixed `auto-orient` pipeline (+ optional `--max`) and a default
    `AutoQuality::Perceptual(visually-lossless)` re-encode in the input's own format;
    `--target`/`--ssim`/`--max-size` override the outcome.
  - Why selected: it bundles the three differentiators (correctness, privacy,
    perceptual quality) into one no-flag command, is pure composition of shipped and
    tested code, adds no dependency, stays pure-Rust/zero-system-dep, and gives
    `optimize` a clean identity distinct from `shrink` (recompress+reorient+strip vs.
    resize-for-web). It leaves the door open for Option B as an additive follow-up.

## Consequences

- **Positive:** The headline demo ("just make this web-good") is one short command;
  it reuses 100% of the STAGE-008 engine; orientation correctness and metadata strip
  come for free from the existing `auto-orient` op + pixel-lane re-encode; no new dep
  → `cargo deny` stays green with no change.
- **Negative:** "privacy by default" here means **strip everything**, not DEC-003's
  selective preserve (keep orientation/ICC/copyright, drop only GPS) — that richer
  behavior waits for STAGE-004's container lane. We must not market selective
  preservation. Orientation is preserved by *baking it into pixels*, not by keeping
  the tag. The `--keep-gps` global remains a no-op until the container lane exists.
- **Negative:** no cross-format auto-negotiation yet — a user who wants "smallest of
  any format" must pick `--format` themselves for now.
- **Neutral:** for a lossless input/output format (PNG, lossless WebP) the perceptual
  target is silently ignored (encoder default), mirroring how `-q` is ignored there
  (DEC-016) — `optimize photo.png` still auto-orients, strips, and re-encodes.

## Validation

- Right if: `optimize photo.jpg` reliably produces a smaller, correctly-oriented,
  metadata-free, visually-indistinguishable file with no flags, and users reach for
  it instead of memorizing `shrink --target …`.
- Revisit when: AVIF decode lands (unblocks perceptual AVIF → enables the
  auto-negotiation follow-up spec); or if user feedback shows the no-resize default
  surprises people (then consider a documented default `--max` for very large
  inputs); or if the DEC-003 container lane ships (then `optimize` can switch from
  strip-everything to selective-preserve + drop-GPS).

## References

- Related specs: SPEC-022 (this command), SPEC-015 (auto-orient), SPEC-016
  (perceptual auto-quality), SPEC-017/021 (`--max-size` + dimension fallback)
- Related decisions: DEC-019 (SSIMULACRA2 perceptual), DEC-017 (auto-orient drops
  metadata), DEC-016 (quality policy), DEC-015 (output-format precedence + partial
  batch), DEC-003 (metadata dual-lane), DEC-020 (AVIF output-only, no decoder)
- Discussions: 2026-06-17 next-stage decision handoff
  (`docs/sessions/2026-06-17-stage-008-shipped-next-stage-handoff.md`); 2026-06-16
  roadmap handoff (Month 2 web-prep power)
