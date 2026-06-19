---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-006                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: null

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
- [ ] SPEC-034 (design 2026-06-19) — path/symlink traversal hardening across Source + Sink: reject a symlinked output destination even under `--yes` (Sink `write`/`write_bytes`) + always-anchor the glob escape-check (close the SPEC-004 `root_opt=None` bypass, DEC-010); DEC-035; no new dep
- [ ] (not yet written) — security-grade recipe validation: reject unsupported version + unknown operations with typed errors
- [ ] (not yet written) — `cargo audit` / `cargo deny` wired into CI (dependency-advisory + license/ban gate)
- [ ] (not yet written) — threat-model verification pass against `SECURITY.md` + `/security-review` on the cumulative diff, findings recorded

**Count:** 1 shipped / 1 in design / 3 pending

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

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
