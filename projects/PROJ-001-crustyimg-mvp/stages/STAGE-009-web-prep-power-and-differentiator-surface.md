---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-009                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-17
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Turns the STAGE-008 outcome-driven engine (perceptual auto-quality + byte
    budgets + modern formats) into the user-facing differentiator SURFACE — the
    one-button "just make this web-good" command, perceptual verification, and
    responsive web output. It advances the project thesis ("routine image prep,
    faster and more repeatable than a GUI or ImageMagick incantations") by making
    the moat legible: a single short command does the right thing, and the result
    is provably good.
  delivers:
    - "`optimize` — the one-button web-good command: auto-orient + strip metadata (incl. GPS) + perceptual-target re-encode, format/size-preserving by default (SPEC-022)"
    - "`diff` — a perceptual SSIMULACRA2 score + visual pixel-diff between two images, with a `--fail-under` CI gate (planned)"
    - "responsive `<picture>`/srcset sets — multi-width × multi-format output + a paste-ready HTML snippet, optional blurhash/thumbhash placeholder (planned)"
    - "a benchmark net — criterion micro-benches + hyperfine CLI wall-clock, equal-quality comparisons gated on SSIMULACRA2 (planned)"
  explicitly_does_not:
    - Build the watermark / EXIF-metadata-EDIT container lane (the original STAGE-004 — `set`/`copy-metadata`/selective `clean --gps`); `optimize` strips metadata via the pixel-lane re-encode, which is privacy-positive but is NOT the selective-preserve container work
    - Build cross-format AUTO-NEGOTIATION (try JPEG vs WebP vs AVIF, pick the smallest) — deferred behind AVIF decode (needed to perceptually score AVIF) and a dedicated spec; v1 `optimize` preserves the input format unless the user picks one
    - Build animation / GIF (a later stage)
    - Add any AGPL/GPL default dependency (DEC-018 / `no-agpl-default-deps`)
---

# STAGE-009: web-prep power and differentiator surface

> **Sequencing note (2026-06-17).** Created with the next free numeric id
> (STAGE-009) but runs in **execution position right after STAGE-008** — ahead of
> the lower-numbered `proposed` stages (STAGE-004 compose-and-metadata, STAGE-005
> batch, STAGE-006 hardening, STAGE-007 release), which stay `proposed` and slide
> later. **Numeric id ≠ execution order**; drive by stage `status`/`priority`, not
> by number. This continues the differentiator wave begun in STAGE-008 (the
> formats+quality core); STAGE-004 is picked up after this stage. Rationale: the
> 2026-06-17 next-stage decision (Option A, user-chosen) —
> `docs/sessions/2026-06-17-stage-008-shipped-next-stage-handoff.md` — and the
> medium-term roadmap (Month 2 = web-prep power) in
> `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md`.

## What This Stage Is

The stage that turns crustyimg's outcome-driven engine into the differentiator a
user actually touches. STAGE-008 built the machinery — perceptual auto-quality
(`--target`/`--ssim` via SSIMULACRA2), the `--max-size` byte budget with a
dimension fallback, and modern output formats (AVIF / WebP). This stage makes that
machinery *legible and verifiable*:

1. **One button.** `optimize photo.jpg` does the right thing without flags:
   bake EXIF orientation into pixels, strip all metadata (privacy by default,
   including GPS), and re-encode to a **visually-lossless perceptual target** —
   the smallest file with no visible quality loss. "Set the look, not the number"
   becomes the *default* experience, not an opt-in flag. (SPEC-022)
2. **Proof.** `diff a.png b.png` reports an SSIMULACRA2 score + a highlighted
   pixel-diff image, with `--fail-under <N>` as a CI visual-regression gate —
   reusing the exact metric the auto-quality search optimizes against. (planned)
3. **Web delivery.** A responsive-set generator emits multi-width × multi-format
   variants and a paste-ready `<picture>`/srcset snippet (optional blurhash/
   thumbhash placeholder) — the deliverable a web developer actually ships.
   (planned)
4. **Credibility.** A benchmark net (criterion micro-benches + hyperfine CLI
   wall-clock) that always reports time + peak RSS + output bytes, and gates every
   size/speed claim on equal SSIMULACRA2 quality. (planned)

When this stage ships, the moat is no longer just an engine — it is a command you
run and a number you can trust.

## Why Now

- It is the **highest-leverage reuse** available: `optimize`, `diff`, and the
  benchmark quality basis are mostly *composition* of the just-shipped
  `src/quality` (the perceptual metric, `fit_under_size`, the `LossyFormat` seam)
  and `src/cli` (`run_pixel_op`, `resolve_effective_quality`, the auto-orient op).
  Doing this while the formats core is fresh is the efficient order.
- It is **where the "why I'd switch" story lives.** STAGE-008 delivered the
  engine; this stage delivers the demo and the trust. Shipping breadth (the
  metadata/watermark lane) first would delay the thing that makes crustyimg
  *different*, not merely *good*.
- The roadmap sequenced it here: STAGE-008 = Month 1 (formats+quality core),
  this stage = Month 2 (web-prep power + ergonomics). STAGE-004–007 keep their
  ids and slide later.

## Success Criteria

- `optimize photo.jpg -o out.jpg` produces a JPEG that SSIMULACRA2-scores at/above
  the visually-lossless threshold, is smaller than a max-quality encode, has its
  EXIF orientation baked into pixels, and carries no metadata — in one short
  command with no flags.
- `optimize` honors `--target`/`--ssim` (perceptual override), `--max-size` (byte
  budget), `--max` (optional long-edge bound), and `-o`/`--format` (output format),
  reusing the shipped search/fan-out unchanged.
- `optimize` preserves the input format and dimensions by default (it recompresses
  and reorients; it does not silently resize or change container).
- `diff` reports a stable SSIMULACRA2 score and exits non-zero under `--fail-under`
  when the score is below the gate. (when that spec lands)
- The responsive generator emits valid variants + a correct `<picture>` snippet.
  (when that spec lands)
- `cargo deny check licenses` stays green; the default binary stays pure-Rust,
  zero system deps; every command remains one short, ergonomic invocation; 3-OS CI
  green.

## Scope

### In scope
- **`optimize`** — the one-button command: a fixed pipeline (auto-orient [+
  optional `--max` resize]) + a **default perceptual visually-lossless** re-encode,
  format/size-preserving, with `--target`/`--ssim`/`--max-size` outcome overrides.
  Pure composition of shipped primitives + a command-shape DEC. **(SPEC-022)**
- **`diff`** — perceptual SSIMULACRA2 score + visual pixel-diff + `--fail-under`
  CI gate. Reuses `crate::quality::score`. (planned spec)
- **responsive sets** — multi-width × multi-format variants + a paste-ready
  `<picture>`/srcset snippet; optional blurhash/thumbhash rider. Introduces opt-in
  HTML emission. (planned spec)
- **benchmark net** — criterion micro-benches (`just bench`) + hyperfine CLI
  wall-clock (`just bench-cli`); equal-quality basis via SSIMULACRA2. (planned
  spec; the micro-net can land early as a `chore`)

### Explicitly out of scope
- Cross-format **auto-negotiation** (auto-pick the smallest of JPEG/WebP/AVIF) —
  deferred behind AVIF decode (perceptual scoring of AVIF needs a decoder, DEC-020)
  and a dedicated spec. v1 `optimize` preserves the input format unless the user
  asks for one.
- The watermark + EXIF-metadata-EDIT container lane (the original STAGE-004:
  `watermark`/`set`/`copy-metadata`/selective `clean --gps`) — re-sequenced after
  this stage. `optimize`'s metadata strip is the pixel-lane re-encode, not the
  selective-preserve container work (DEC-003).
- Animation / GIF (a later stage).
- Any AGPL/GPL default dependency (DEC-018).

## Spec Backlog

Ordered by recommended execution (the one-button command first — it is the cleanest
composition and the headline demo); drive by status, not number.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-022 (shipped 2026-06-17, PR #25) — **`optimize` one-button web-good
  command (HEADLINE):** a fixed `auto-orient` pipeline (+ optional `--max` long-edge
  bound) + a **default perceptual visually-lossless** re-encode (SSIMULACRA2),
  format/size-preserving; strips all metadata (privacy incl. GPS) via the pixel-lane
  re-encode; `--target`/`--ssim`/`--max-size` override the outcome, `-o`/`--format`
  pick the output format. Pure composition of the shipped `src/quality` +
  `run_pixel_op` + the `auto-orient` op — **no new dependency**, no change to
  `src/quality`/`src/sink`. **DEC-024** (defaults + deferral of cross-format
  auto-negotiation, which needs AVIF decode). 16 tests (6 unit + 10 integration);
  3-OS + feature CI green. Build had zero deviations from the spec.
- [~] SPEC-023 (design) — **`diff` perceptual comparison + CI gate:** prints the
  SSIMULACRA2 score of `<b>` vs `<a>` (reuses `crate::quality::score`); `--fail-under
  <N>` exits a new dedicated code **7** ("check not satisfied") for CI
  visual-regression gating; `--json`; dimension-mismatch = exit 2. **DEC-025** (adds
  exit 7; defers the visual-diff heatmap to a follow-up). v1 = score + gate + json;
  the highlighted pixel-diff image is the deferred "visual" half.
- [ ] (not yet written) responsive `<picture>`/srcset set generator — multi-width
  × multi-format + a paste-ready HTML snippet; optional blurhash/thumbhash. Opt-in
  HTML emission (a question for design).
- [ ] (not yet written) benchmark net — criterion micro-benches + hyperfine CLI
  wall-clock; equal-quality comparisons gated on SSIMULACRA2. The micro-net may
  land early as a `chore`.

**Count:** 1 shipped / 1 active / 3+ pending  *(+ a deferred `diff` visual-diff
heatmap follow-up, per DEC-025)*

## Design Notes

- **Reuse over rebuild.** The whole stage rides shipped seams: `crate::quality`
  (`score`, `auto_quality`, `fit_under_size`, the `LossyFormat` two-predicate
  split) and `crate::cli` (`run_pixel_op`, `resolve_effective_quality`, the
  `auto-orient` registry op, `parse_size`/`fmt_bytes`). New commands should add a
  clap variant + a thin `run_*` handler that builds a pipeline and delegates to
  `run_pixel_op` — not new pixel/search machinery.
- **`optimize` vs `shrink`** (keep the line honest): `shrink` is **resize-centric**
  (default long-edge bound 1600, fixed q80). `optimize` is **recompress +
  reorient + strip**: it does NOT resize by default, it auto-orients (shrink does
  not), and its default quality mode is **perceptual visually-lossless**, not a
  fixed number. DEC-024 records this so the two don't drift into duplicates.
- **Privacy by construction.** Every pixel-lane encode drops metadata (the `image`
  crate discards it on encode), and the `auto-orient` op drops the metadata bundle
  after baking (DEC-017). So `optimize` is privacy-positive with no new code — but
  this is the strip-everything behavior, NOT the DEC-003 selective-preserve
  (keep orientation/ICC/copyright, drop only GPS) container lane, which is
  STAGE-004. Document the distinction; do not claim selective preservation.
- **Cross-sync contract still binds.** Any new candidate encode added for this
  stage must match the sink's write bytes (the JPEG/AVIF/WebP cross-sync contract
  in `src/quality` / `src/sink`). `optimize` adds none — it reuses the existing
  arms — but `diff`/responsive must respect it if they ever encode.
- Weighty/precedent-setting decisions get a `DEC-*`: **DEC-024** (the `optimize`
  command shape — default perceptual visually-lossless + auto-orient + strip,
  format/size-preserving, and the explicit deferral of cross-format
  auto-negotiation). Later specs may add their own (a `diff` visual-diff-image
  decision; an HTML-emission decision for responsive sets).

## Dependencies

### Depends on
- STAGE-008 — the perceptual metric + searches (`src/quality`), the modern-format
  sinks, the `LossyFormat` seam, and `resolve_effective_quality`/`run_pixel_op`.
- STAGE-003 — the `auto-orient` op (DEC-017) that `optimize` folds in, and the
  `run_pixel_op` fan-out + DEC-015 format precedence.
- DEC-018 — the live license gate constraining any new dep.

### Enables
- The animation / GIF stage (modern formats + the verification/benchmark surface
  are its prerequisites).
- A future cross-format auto-negotiation spec (once AVIF decode lands so the
  perceptual search can score AVIF) — `optimize` is the natural host for it.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run the Stage Ship prompt to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
