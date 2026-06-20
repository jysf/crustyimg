---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-006                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: 2026-06-19

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    The MVP exit gate. Hardens every untrusted-input surface (images,
    recipes, paths) and verifies it against the threat model so the binary
    can ship to brew/crates.io without exposing users who run it on
    arbitrary files to decode bombs, path traversal, or malicious recipes.
  delivers:
    - "Decode resource limits on image load (image::Limits — bound dimensions/allocation, no decompression bombs)"
    - "Path/symlink traversal hardening + tests across Source and Sink (tightening the SPEC-004 glob escape-check gap)"
    - "Security-grade recipe validation (version + unknown-op rejection, hardened)"
    - "`cargo audit` / `cargo deny` wired into CI"
    - "A threat-model verification pass against SECURITY.md + a /security-review on the diff"
  explicitly_does_not:
    - Add new user-facing image features (this is a hardening/assessment stage)
    - Change the codec policy or add native codecs (DEC-004 stands)
    - Defer any hardening the constraint requires be done as-built earlier
---

# STAGE-006: hardening and security assessment

## What This Stage Is

The MVP exit gate. Earlier stages harden their own surfaces as they build
(constraint `untrusted-input-hardening`, severity warning); this stage
**consolidates, completes, and verifies** that hardening so the binary is
safe to ship to people who will run it on arbitrary, untrusted files. It
sets decode resource limits on `Image` load (`image::Limits` to bound
dimensions and allocation — closing a known decompression-bomb gap),
hardens path/symlink traversal across the Source and Sink lanes with tests
(including tightening the defensive glob escape-check gap noted in DEC-010 /
SPEC-004), brings recipe validation up to security grade (version + unknown-
operation rejection), wires `cargo audit`/`cargo deny` into CI, and runs a
threat-model verification pass against `SECURITY.md` plus a `/security-review`
on the cumulative diff. When this ships, the MVP is shippable.

## Why Now

It is last by design: a meaningful security assessment needs the full
attack surface present — image decode (STAGE-002/003), the metadata lane
(STAGE-004), and recipe + path/batch handling (STAGE-005). Doing it now
verifies the as-built hardening actually holds end to end rather than in
isolation, and gives the project a single, auditable exit gate
(brief Success Criteria: "passes a hardening/security assessment before
ship"). It adds no new user features — it makes the existing ones safe.

## Success Criteria

- `Image` load sets `image::Limits` bounding dimensions/allocation; a
  crafted decompression-bomb fixture is rejected with a typed error, not a
  panic or OOM.
- Output sinks reject name-template/path values that escape the target dir
  and never overwrite without `--yes` (typed error, exit 5); tests cover
  `../` escapes and absolute-path templates.
- Directory/glob sources do not follow symlinks out of the tree; the
  SPEC-004 escape-check is tightened and covered by a traversal test.
- Recipe load rejects an unsupported version and any unknown operation with
  a typed error (hardened beyond the STAGE-005 baseline).
- `cargo audit` (and `cargo deny`) run in CI and are clean.
- A threat-model verification pass against `SECURITY.md` is recorded and a
  `/security-review` on the diff surfaces no unresolved high-severity finding.

## Scope

### In scope
- Decode limits on `Image::load` via `image::Limits` (decompression-bomb defense).
- Path/symlink traversal hardening + tests on Source and Sink; tighten the SPEC-004 glob escape-check defensive gap (DEC-010).
- Security-grade recipe validation (version + unknown-op rejection).
- `cargo audit` / `cargo deny` in CI.
- Threat-model verification pass vs `SECURITY.md` + `/security-review` on the diff.

### Explicitly out of scope
- New image operations or user-facing features.
- Codec-policy changes / adding native codecs (DEC-004 unchanged).
- Performance work beyond what the resource limits require.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-033 (shipped 2026-06-19, PR #37) — decode resource limits on the canonical load path (`image::Limits`: per-dimension ≤ 65 535 + alloc ≤ 512 MiB, reject-not-clamp → typed `ImageError::LimitsExceeded`, exit 1) at the one `decode_with_format` choke point — closes the known decompression-bomb gap; DEC-034; no new dep
- [x] SPEC-034 (shipped 2026-06-19, PR #38) — path/symlink traversal hardening across Source + Sink: reject a symlinked output destination even under `--yes` (Sink `write`/`write_bytes`, all 4 file arms) + always-anchor the glob escape-check (close the SPEC-004 `root_opt=None` bypass, DEC-010); DEC-035; no new dep. (Follow-up surfaced: `edit --save-recipe` raw write is unguarded → fold into the threat-model pass below.)
- [x] SPEC-035 (shipped 2026-06-19, PR #39) — security-grade recipe validation: recipe resource limits (text ≤ 64 KiB, ≤ 1024 steps, typed `RecipeError::TooLarge`/`TooManySteps`, exit 1) at the `from_toml` choke point + a CLI pre-read file-size guard, on top of the existing version/unknown-op rejection; DEC-036; no new dep. (Op-param bounds — e.g. resize upscale bomb — deferred to the threat-model pass.)
- [x] SPEC-036 (shipped 2026-06-19, PR #40) — extend the CI cargo-deny gate from `check licenses` to the full `check advisories bans sources licenses` (+ `deny.toml` sections + `just deny`); `cargo audit` consolidated into cargo-deny's RUSTSEC advisories check (DEC-037), not a separate job; no new runtime dep. (RUSTSEC-2024-0436 `paste`/unmaintained handled with a dated narrow ignore.)
- [x] SPEC-037 (shipped 2026-06-19, PR #41) — STAGE-006 capstone: `edit --save-recipe` symlink-destination guard (reuse the Sink's `reject_symlink_destination` — DEC-035) + resize output cap tightened to 512 MiB for decode-symmetry (DEC-038 — *finding:* resize was already bounded by SPEC-010's `MAX_EDGE`/`MAX_AREA`, so this tightens rather than adds) + a `SECURITY.md` threat-model verification table; the verify cycle ran an adversarial security review over the cumulative STAGE-006 surface — **no unresolved finding**. No new dep.

**Count:** 5 shipped / 0 active / 0 pending  (STAGE-006 COMPLETE — the MVP hardening exit gate. SPEC-033 decode limits · SPEC-034 path/symlink · SPEC-035 recipe limits · SPEC-036 supply-chain CI · SPEC-037 capstone + threat-model verification.)

## Design Notes

- Anchored by constraint `untrusted-input-hardening` (added 2026-06-14):
  decode limits, sink path-escape rejection, source no-symlink-out, recipe
  validation — all surfaced as typed errors, never panics (DEC-007). Earlier
  stages do these as-built; this stage is the consolidation + verification.
- Decode limits use `image::Limits` on the canonical `Image` load path
  (DEC-002) — the primary decompression-bomb defense, a known gap until now.
- Path/symlink work builds on DEC-010: directories are non-recursive via
  `std::fs::read_dir` and symlink-escape entries are skipped — this stage
  tightens and tests that escape-check (the SPEC-004 defensive gap) and the
  Sink's name-template escape rejection together.
- Recipe validation hardens the STAGE-005 baseline (DEC-005): the recipe
  carries a version and the registry rejects unknown ops.
- The metadata lane (DEC-003) and codec policy (DEC-004) are part of the
  attack surface reviewed but are not changed here.
- `cargo audit`/`cargo deny` may add CI tooling, not top-level runtime deps;
  confirm whether a DEC is warranted (`no-new-top-level-deps-without-decision`
  governs runtime deps).

## Dependencies

### Depends on
- STAGE-002..005 — the full untrusted-input attack surface (decode, metadata, recipes, paths/batch) must exist to assess it.
- STAGE-001 (DEC-010) — Source/Sink whose traversal behavior is hardened here.
- External: `cargo audit`, `cargo deny`; the `SECURITY.md` threat model.

### Enables
- PROJ-001 ship — this is the exit gate; passing it clears the MVP to release.
- A documented security baseline that post-MVP waves (docs/backlog.md) inherit.

## Stage-Level Reflection

*Filled in at ship (2026-06-19).*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes — the MVP exit
  gate is met.** Every untrusted-input surface is bounded with typed errors (never
  a panic/OOM) and verified end-to-end: **decode** (`image::Limits` — dims ≤ 65 535,
  alloc ≤ 512 MiB, SPEC-033); **paths/symlinks** (`safe_join` + symlinked-destination
  rejection on every output write incl. `--save-recipe`, always-anchored source
  escape checks, SPEC-034/037); **recipes** (size ≤ 64 KiB + ≤ 1024 steps + the
  resize output cap, SPEC-035/037); **supply chain** (CI `cargo deny check advisories
  bans sources licenses`, SPEC-036). A `SECURITY.md` `## Verification` table maps each
  of the six threats → mitigation → spec/DEC, and an adversarial review over the
  cumulative diff surfaced **no unresolved high-severity finding**. The MVP is
  shippable.
- **How many specs did it actually take?** **5 (SPEC-033…037), exactly the planned
  backlog** — one per axis (decode, paths, recipes, supply-chain, capstone+verify).
  No splits or extra specs were needed; the as-built hardening from earlier stages
  (DEC-010 source guards, SPEC-005 `safe_join`, SPEC-010 resize caps) meant several
  items were *verify-and-tighten* rather than *build-from-scratch*.
- **What changed between starting and shipping?** Almost nothing in plan — the
  surprise was how much was **already hardened as-built**, so the stage was more a
  *verification* than a *construction* (the resize "bomb" and the recipe-validation
  baseline were already defended; we tightened/consolidated rather than added).
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - No template/constraint change. Four security-policy DECs (DEC-034 decode,
    DEC-035 traversal, DEC-036 recipe, DEC-037 supply-chain, DEC-038 resize) record
    the limits; one standing item (the `paste` RUSTSEC ignore) to clear upstream.
  - **Reinforced:** the lean (`--no-default-features`) build + (now) the full
    `cargo deny check` belong in every verify
    ([[verify-includes-lean-no-default-features-build]]).
  - **New, worth promoting:** for a *verification/hardening* spec, **read the whole
    target function and audit what already exists before asserting a gap** — the
    SPEC-037 design wrongly assumed resize was unbounded (SPEC-010 had hardened it);
    the verify pass caught it, but a full read at design would have saved the
    round-trip.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - **Harden at the single choke point, reuse the typed-error + exit-code mapping,
    and make the verify cycle run an adversarial bypass/panic grep** — this pattern
    (every spec) both proved completeness AND surfaced each next item (the
    `--save-recipe` raw write, the resize bound). Carry it into any future hardening.
  - **The verify-as-security-review (cumulative, not just per-spec)** is what makes
    an exit-gate record trustworthy — run one explicit cumulative adversarial pass at
    the end of a hardening stage.
  - **Model split validated across the whole stage:** 5/5 clean Sonnet builds, each
    cleared by an independent Opus verify, ~$0.32–0.48 per build.

**Stage cost (recorded):** SPEC-033 $0.98/144k + SPEC-034 $0.91/131k + SPEC-035
$0.97/142k + SPEC-036 $0.77/109k + SPEC-037 $1.00/142k = **$4.63 · 668k** across 5
shipped specs — the MVP hardening exit gate.
