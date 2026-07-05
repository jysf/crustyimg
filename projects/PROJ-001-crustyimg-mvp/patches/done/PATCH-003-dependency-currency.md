---
# A PATCH is a lightweight fix to shipped behavior (DEC-043). Lighter than a
# SPEC: no stage, collapsed patch→verify→ship cycle, keeps independent verify.

patch:
  id: PATCH-003
  type: patch
  cycle: ship                      # patch | verify | ship
  fixes: "dependency currency — bump the resize backend + progress bar to latest; add a scheduled advisory audit"
  complexity: S
  blocked: false

project:
  id: PROJ-001
repo:
  id: crustyimg

agents:
  implementer: claude-opus-4-8     # trivial bump — done in the main loop (verified by full gates)
  verifier: claude-opus-4-8        # main-loop verification (see notes); independent pass optional for a clean bump
  created_at: 2026-07-05

references:
  decisions: [DEC-008, DEC-033, DEC-037, DEC-043]

# This is the dependency-currency slice of the planned STAGE-011 (dep hygiene),
# executed as a patch per the maintainer's call. It is NOT a full stage — the
# only STAGE-011 code work turned out to be two version bumps + a CI cron.
cost:
  sessions:
    - cycle: patch
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: >
        Main-loop patch: `fast_image_resize` 5.5.0→6.0.0 (MAJOR) + `indicatif`
        0.18.4→0.18.6 + Cargo.lock refresh. The 6.0 major turned out to be
        API-compatible with our single resize block (operation/mod.rs ~511-531) — the
        crate compiled with ZERO code changes. Added .github/workflows/scheduled-audit.yml
        (weekly `cargo deny check advisories` + workflow_dispatch) for RustSec DB drift.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: >
        Main-loop verification (a version bump with no code change): full suite green
        (16 test suites), incl. the resize-parity test (fast_image_resize Lanczos3 vs
        image::imageops Lanczos3, operation/mod.rs ~1141) which would catch any output
        change from the 6.0 backend — it passed, so resize output is unchanged. clippy
        -D warnings clean, fmt clean, lean build compiles, `cargo deny` green (still 1
        ignore: paste). No new transitive advisory/license/ban issues from 6.0.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: "Main-loop ship (merge PR + archive to patches/done/); folds into the 0.2.1 release."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 3
---

# PATCH-003: dependency currency (resize backend + progress bar) + scheduled advisory audit

**The dependency-currency slice of STAGE-011, executed as a patch (DEC-043) per the
maintainer's call.** Bound for **0.2.1**.

## What

1. **`fast_image_resize` 5.5.0 → 6.0.0** (the SIMD resize backend, DEC-008). A major bump,
   but API-compatible with our usage — the single resize block in `src/operation/mod.rs`
   compiled unchanged. Verified behavior-preserving by the existing resize-parity test.
2. **`indicatif` 0.18.4 → 0.18.6** (batch progress bar, DEC-033) — patch bump.
3. **`Cargo.lock`** refreshed (transitive deps).
4. **Scheduled advisory audit** — `.github/workflows/scheduled-audit.yml`: a weekly
   `cargo deny check advisories` (+ manual `workflow_dispatch`) that catches RustSec DB
   drift between pushes (the retro rec — the 0.1.x advisories all landed post-release with
   no code change on our side). Only `advisories` is scheduled; bans/licenses/sources can't
   drift without a commit, which the push-triggered `supply-chain` job already covers.

## Verification (kept, DEC-043)

`cargo test` (16 suites, incl. the resize-parity test), `cargo clippy --all-targets -- -D
warnings`, `cargo fmt --check`, `cargo build --no-default-features`, and `cargo deny check
advisories bans sources licenses` — all green. `cargo deny` still shows the single documented
`paste` ignore; the 6.0 bump added no new advisory/license/ban issue.

## Ship

CHANGELOG `[Unreleased] → Changed` (folds into 0.2.1). Archive to `patches/done/`. Cut
**0.2.1** via `just release 0.2.1` after merge (maintainer-authorized tag push).

## Note

STAGE-011's third notional item (the scheduled deny job) is included here. STAGE-011 as a
themed stage is effectively subsumed by this patch — the actual work was small enough that a
stage would have been overhead. STAGE-012 (permissive Display sink) remains a proper SPEC
(new subsystem + DEC), not a patch.
