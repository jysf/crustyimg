---
# Maps to ContextCore epic-level conventions.

stage:
  id: STAGE-014
  status: on_hold                    # proposed | active | shipped | cancelled | on_hold
  priority: low                      # on_hold pending a decidable gate (see "The gate", 2026-08-15)
  target_complete: null

project:
  id: PROJ-004
repo:
  id: crustyimg

created_at: 2026-07-06
shipped_at: null

value_contribution:
  advances: >
    Adds the "this could be smaller / wrong format" rules — the ones that make the linter more
    than a metadata checker. Each reuses PROJ-002's format-decision engine + the SSIMULACRA2
    probe behind the savings-threshold gate, so a finding is backed by a real measurement, not a
    heuristic.
  delivers:
    - "`format/legacy-format` (warn): proves via an equal-SSIMULACRA2 probe that a modern format
      saves ≥ threshold → fix `optimize --format <fmt>`; never suggests an unbuilt codec"
    - "`quality/excessive-jpeg-quality` (warn): a VL-target re-encode scores ≥ anchor while saving
      ≥ threshold → fix `optimize`"
    - "`format/indexed-png-opportunity` (info, advisory): RGB(A) PNG with few colours → palette
      PNG; stays advisory until a permissive quantizer ships (PROJ-007), interim suggests lossless
      WebP"
  explicitly_does_not:
    - Re-implement any search/decision math — it composes the shipped `src/analysis/decide.rs`
      engine + `src/quality/` search
    - Add a new default dependency
    - Ship indexed-PNG as a fix (needs PROJ-007's quantizer)
---

# STAGE-014: engine-backed rules

> **Deferred below the 1.0 line — demand-gated (2026-07-07). Gate replaced 2026-08-15 because it
> could not be decided; see "The gate" below.** `lint` already shipped as a 10-rule catalog in
> v0.4.0. This stage adds *more* lint breadth (three engine-backed rules), which the adoption-first
> roadmap reconciliation put past 1.0. The stage is cheap to finish (reuses the shipped engine, no
> new deps) and is kept **build-ready in spirit but unwritten** — SPEC-054/055 are *not* yet
> specced. Nothing here is cancelled; it is sequenced, not dropped.

## The gate — replaced 2026-08-15

**The original gate could not turn green.** It read: *further investment in the least-validated
surface waits for a real adoption signal (Action/Eleventy users asking for it)*. Reviewed
2026-08-15:

- `feedback/` holds two files — a bragfile project note and a process analysis. **Neither is a
  user request**, for this or anything else.
- **Nothing instruments the signal.** There is no mechanism that would detect "an Action/Eleventy
  user asked", so its absence is not evidence.
- The roadmap's own Wave 4 names **the maintainer's own Eleventy photo blog** as the dogfood
  testbed — and it will never file a GitHub issue.

So the condition was unfalsifiable in practice: an indefinite hold in a gate's clothing. **Both
halves of its premise have also moved** — the work this was deferred *in favour of* (HEIC/RAW/SVG
input reach, PROJ-009) has **shipped**, and 0.7.0 launched on four channels on 2026-08-11.

**The replacement gate is a measurement, not a wait:**

> Run `lint` over the maintainer's Eleventy photo-blog corpus and record two numbers: **what
> fired**, and **what should have fired but did not** (assets a human judges oversized or in a
> legacy format that the 10 shipped rules stay silent on).
>
> - **If the silent-miss count is non-trivial** → activate this stage, `format/legacy-format`
>   first (see priority below).
> - **If the shipped rules already catch essentially everything** → **close the stage** and record
>   that lint breadth was not the gap. That is a real outcome, not a failure.

This is decidable in an afternoon, needs no new code, and — unlike the original — **can return
"no"**. Until it is run, the stage stays `on_hold`; the difference is that it is now on_hold
pending a *specific action* rather than pending an event nobody is watching for.

### Spec priority within the stage — not all three are equal

`format/legacy-format` (SPEC-054) is the one carrying a **positioning claim**, and should be built
first and possibly alone. `docs/territory.md:39` wins the Page-audit layer on being the
source-file, no-URL Lighthouse; Lighthouse's two headline image audits are `uses-webp-images` and
`uses-optimized-images`, and **crustyimg's 10 shipped rules answer neither** — they are metadata,
privacy, and dimensions. This stage's own summary calls its rules "the rules that give `lint` its
teeth"; legacy-format is the tooth that matters.

`quality/excessive-jpeg-quality` and `format/indexed-png-opportunity` (SPEC-055) are materially
weaker. **Do not treat SPEC-055 as automatically following SPEC-054.**

### Correction 2026-08-15 — the indexed-PNG quantizer blocker is out of date

This stage says `format/indexed-png-opportunity` *"stays advisory until a permissive quantizer
ships (PROJ-007)"*, and lists "indexed-PNG as a fix" as out of scope for that reason. **Checked
2026-08-15: a permissive quantizer is already in the tree.**

- `color_quant` **1.1.0 is in `Cargo.lock`**, pulled by `gif` → `image`. Licence **MIT**
  (112M downloads, latest 2.0.0 published 2026-05-09).
- `image` ships `impl ColorMap for color_quant::NeuQuant`
  (`imageops/colorops.rs:438`), and `imageops::index_colors` + `imageops::dither` are the apply
  path — all compiled into every default native build today.
- `image` does **not** re-export it, so calling NeuQuant directly needs a manifest line and a
  `DEC-*` (`no-new-top-level-deps-without-decision`) — but **zero new compiled bytes**, since it
  is already linked. Applying a *known* palette needs no new dep at all: implement `ColorMap`.

**The honest qualifier:** NeuQuant is not pngquant. `imagequant` (the quality leader) is correctly
declined on licence, and NeuQuant's output is measurably worse. So the blocker is not "no
permissive quantizer exists" but **"is NeuQuant good enough?"** — which is a measurement
(SSIMULACRA2 + byte delta on a palette-friendly corpus), not a dependency wait. Run it as part of
the stage gate above; if NeuQuant clears the savings threshold at an acceptable score, the rule
can ship as a **fix** rather than advisory, and the PROJ-007 dependency drops.

### The counter-argument, kept on the record

The 2026-07-07 reasoning was good and has **not** been refuted: *"cheap to build is exactly why
not to over-invest in the least-validated surface."* Three more rules do not create adoption. If
`lint` is unused, the bottleneck is distribution, not breadth — which is why the measurement above
gates the work, and why `docs/backlog.md`'s MCP-server entry (a new *consumer* for the existing
lint output) may be the higher-value move even if this stage activates.

## What This Stage Is

The stage that gives `lint` its teeth: the rules that answer "could this asset be smaller, or is
it in the wrong format?" — each proven by actually running PROJ-002's format-decision engine + the
SSIMULACRA2 perceptual probe on the file and checking the win against the savings-threshold gate.
No new compression math; these rules are a thin read over the shipped engine. It requires PROJ-002
(shipped) and the STAGE-013 framework (the `Rule`/`Finding` model + the savings-threshold config).

## Why Now

- **It's the Lighthouse-parity half** (`uses-webp-images`, `uses-optimized-images`) — but per-file
  and URL-free. The measurement is real (an actual probe), so it earns a `warn`.
- **It's cheap** — the engine + probe already exist; the rule just interprets the result.

## Success Criteria

- `format/legacy-format` fires only when a real equal-SSIMULACRA2 probe shows a built modern format
  saves ≥ the threshold; the fix names only a codec the running binary can produce (a
  license/capability guard — DEC-004).
- `quality/excessive-jpeg-quality` and `format/indexed-png-opportunity` respect the same
  savings-threshold gate; `indexed-png` stays `info`/advisory.
- No new default dependency; `just deny` green; determinism upheld (same file ⇒ same finding).

## Scope

### In scope
- `format/legacy-format` + the license/capability guard + the savings-threshold wiring. **(SPEC-054)**
- `quality/excessive-jpeg-quality` + `format/indexed-png-opportunity`. **(SPEC-055)**

### Explicitly out of scope
- SARIF / Actions / pre-commit (STAGE-015). Near-dup (v2). ~~Indexed-PNG as a fix (PROJ-007)~~ — **reopened 2026-08-15**, see the correction below: the quantizer is already in the tree, so this is a quality measurement, not a dependency wait.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-054 (not yet written) — `format/legacy-format`: equal-SSIMULACRA2 probe over the
  `src/analysis/decide.rs` engine → "could be <fmt>, saves N%"; savings-threshold gate; codec
  license/capability guard (no `--format avif` suggestion in a non-AVIF build).
- [ ] SPEC-055 (not yet written) — `quality/excessive-jpeg-quality` + `format/indexed-png-opportunity`
  (advisory) over the shipped quality search + format-rec.

**Count:** 0 shipped / 0 active / 2 pending

## Design Notes

- **The seam to watch:** `optimize`'s per-candidate solve (`solve_candidate` in `src/cli/mod.rs`)
  is CLI-private today. `legacy-format` needs the same "encode each candidate, measure bytes at an
  equal perceptual score" logic. Build cycle should factor a small shared helper (in
  `src/analysis/decide.rs` or `src/quality/`) rather than duplicate it — this also keeps the
  future planner (PROJ-003) reuse clean.
- Weighty decision → its own DEC (the legacy-format probe method + the codec-suggestion guard) if
  it goes beyond what DEC-050 already covers.

## Dependencies

### Depends on
- **PROJ-002** — `src/analysis/decide.rs` (format decision) + `src/quality/` (SSIMULACRA2 search).
- **STAGE-013** — the `Rule`/`Finding` framework + the savings-threshold config.
- DEC-004 (codec gating → the suggestion guard), DEC-019/020/021/022.

### Enables
- STAGE-015 — these rules appear in SARIF + the Action's PR annotations.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
