---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-036
  type: chore                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-037, DEC-018, DEC-009]
  constraints:
    - untrusted-input-hardening
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
  related_specs: [SPEC-001]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  Fourth STAGE-006 hardening item: turns the license-only cargo-deny CI job into
  the full supply-chain gate (advisories + bans + sources + licenses) so a
  vulnerable/unmaintained/yanked or non-crates.io dependency fails CI.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work, not separately metered. Read ci.yml /
        deny.toml / justfile; scoped the gate extension (license-only → full
        advisories+bans+sources+licenses) + authored the spec + DEC-037 (the
        cargo-audit-into-cargo-deny consolidation: deny's advisories check reads
        the same RUSTSEC DB) + the Sonnet build prompt, incl. the findings policy
        (dep-bump or narrow commented ignore, never a whole-check disable) and a
        "validate the deny.toml schema against the real tool" note. No new runtime
        dep; license policy (DEC-018) untouched. Fourth STAGE-006 spec (a chore).
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: 59326
      estimated_usd: 0.32
      duration_minutes: 13
      recorded_at: 2026-06-19
      notes: >
        Real metered subagent on Sonnet 4.6. subagent_tokens=59326,
        duration_ms=8062300 (incl. cargo-deny RUSTSEC fetch + a CI round-trip).
        estimated_usd at Sonnet list ($3/$15 per MTok, ~80/20). supply-chain
        gate: ci.yml cargo-deny command -> check advisories bans sources licenses
        + deny.toml [advisories]/[bans]/[sources] + just deny; cargo-audit
        consolidated into cargo-deny (DEC-037); no new runtime dep. One real
        finding handled per policy: RUSTSEC-2024-0436 (paste/unmaintained, no safe
        upgrade — transitive via little_exif/rav1e) → dated narrow ignore. Fixed an
        in-flight YAML colon bug in the step name. 16/16 CI green; 404 tests pass.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 50000
      estimated_usd: 0.45
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        ORDER-OF-MAGNITUDE ESTIMATE (~50k) — read-only Explore subagent on Opus
        (no metered usage block) + config re-checks. Explore verdict: APPROVED,
        no concerns; adversarially confirmed the gate is the genuine four-check
        gate (not weakened), [licenses] is byte-for-byte unchanged, the single
        RUSTSEC-2024-0436 ignore is narrow + dated + justified (transitive paste,
        no safe upgrade), no check disabled to pass, no new runtime dep; both
        old (`check licenses`) and new full checks exit 0.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "Main-loop ship bookkeeping (merge dance + cost totals + reflection + archive); not separately metered."
  totals:
    tokens_total: 109326
    estimated_usd: 0.77
    session_count: 4
---

# SPEC-036: full supply-chain gate (cargo-deny advisories + bans + sources) in CI

## Context

**The fourth STAGE-006 hardening item — a CI/tooling chore.** SPEC-033/034/035
hardened the runtime (decode, paths, recipes); this closes the **supply-chain**
blind spot. CI already runs `cargo-deny`, but only `check licenses` (DEC-018) —
so a license violation fails CI, but a **known-vulnerable, unmaintained, or
yanked** dependency, a **duplicate/banned** crate, or a **non-crates.io source**
passes unnoticed. `cargo-deny` evaluates all four axes from one config; only the
CI `command` and a few `deny.toml` sections need adding.

Per **DEC-037**, the gate becomes `cargo deny check advisories bans sources
licenses` (and `just deny` mirrors it). `cargo audit` is **not** added: its
RUSTSEC advisory database is exactly what `cargo deny check advisories` reads, so
a separate job would be redundant. Parent: `STAGE-006` (backlog item #4).
Governing: **DEC-037** (the gate + cargo-audit consolidation), **DEC-018** (the
license policy this extends), **DEC-009** (CI). No new runtime dependency.

## Goal

Extend the CI `cargo-deny` job and `just deny` from `check licenses` to the full
`cargo deny check advisories bans sources licenses`, add schema-correct
`[advisories]`/`[bans]`/`[sources]` sections to `deny.toml`, and ensure the full
check is **green** — so a vulnerable/unmaintained/yanked/banned/non-crates.io
dependency fails CI, with any real finding handled by a dep bump or a
narrowly-scoped commented exception.

## Inputs

- **Files to read:**
  - `.github/workflows/ci.yml` — the `licenses` job (`EmbarkStudios/cargo-deny-action@v2`,
    `command: check licenses`) to extend.
  - `deny.toml` — currently `[graph]` + `[licenses]` only; add `[advisories]`,
    `[bans]`, `[sources]`.
  - `justfile` — the `deny` recipe (`cargo deny check licenses`) to update.
  - `decisions/DEC-037` (the policy), `DEC-018` (license policy), `DEC-009` (CI).
- **External APIs:** `cargo-deny` (the `EmbarkStudios/cargo-deny-action@v2` action,
  already used) + the RUSTSEC advisory DB (fetched at check time). No new crate.
  Docs: https://embarkstudios.github.io/cargo-deny/checks/index.html
- **Related code paths:** `.github/workflows/`, `deny.toml`, `justfile`.

## Outputs

- **Files modified:**
  - `.github/workflows/ci.yml` — the cargo-deny job `command:` →
    `check advisories bans sources licenses`; rename the job to reflect the
    broadened scope (e.g. "supply-chain policy (cargo-deny)").
  - `deny.toml` — add `[advisories]` (deny vulnerabilities/unmaintained, `yanked
    = "deny"`, empty `ignore`), `[bans]` (`multiple-versions = "warn"`, empty
    `deny`/`skip`), `[sources]` (`unknown-registry = "deny"`, `unknown-git =
    "deny"`, `allow-registry = ["https://github.com/rust-lang/crates.io-index"]`).
    Use the schema version current `cargo-deny` expects; validate with the tool.
  - `justfile` — `deny` recipe → `cargo deny check advisories bans sources licenses`.
  - `docs/api-contract.md` is NOT touched (no CLI surface change).
- **Files created:** none.
- **New exports / Database changes:** none.

## Policy (PINNED — DEC-037)

- **CI command:** `check advisories bans sources licenses` (explicit list, not the
  implicit `check`, for auditability).
- **advisories:** fail on RUSTSEC vulnerabilities + unmaintained; `yanked =
  "deny"`. `ignore = []` (any future advisory exception is a commented, dated entry
  with a revisit note — never a blanket disable).
- **bans:** `multiple-versions = "warn"` (visibility without breaking the build on
  unavoidable duplicate transitive versions); `deny = []`, `skip = []` for now.
- **sources:** only crates.io is allowed (`unknown-registry`/`unknown-git` =
  `"deny"`; `allow-registry` = the crates.io index). No git/path/alt-registry deps.
- **licenses:** UNCHANGED (DEC-018 `allow`/`exceptions`/`confidence-threshold`).
- **`cargo audit` is NOT added** (redundant with `cargo deny check advisories`,
  DEC-037).
- **Findings policy:** a real advisory/ban is fixed by a **dependency bump**; only
  if no fix exists, a **narrowly-scoped, commented** `ignore`/`skip` with a
  revisit trigger (mirrors the license `exceptions` discipline).

## Acceptance Criteria

- [ ] CI's cargo-deny job runs `check advisories bans sources licenses` (verified
  by reading `.github/workflows/ci.yml`) and is **green** on the PR.
- [ ] `cargo deny check advisories bans sources licenses` passes locally (the
  build agent runs it; if an advisory/ban surfaces, it is resolved per the
  findings policy, NOT by disabling a whole check).
- [ ] `deny.toml` has schema-correct `[advisories]`, `[bans]`, `[sources]`
  sections and the existing `[licenses]` policy is unchanged (same `allow` +
  `exceptions`).
- [ ] `just deny` runs the same full check (local == CI).
- [ ] The existing license-only behavior is preserved (a copyleft dep would still
  fail, now alongside advisory/ban/source failures); the rest of CI is unchanged.
- [ ] No new top-level runtime dependency is added (cargo-deny is CI tooling).

## Failing Tests

This is a CI/config chore — the "tests" are the gate itself, not Rust unit tests.

- **`cargo deny check advisories bans sources licenses` (run locally in build)** —
  must exit 0. This is the executable acceptance check; the build agent runs it
  and pastes the result into Build Completion.
- **`.github/workflows/ci.yml` (assert by inspection)** — the cargo-deny step's
  `command:` is `check advisories bans sources licenses` (not `check licenses`).
- **`just deny` (run locally)** — runs the full check and exits 0.
- **CI on the PR** — the (renamed) supply-chain job is green across the workflow.
- *No Rust `#[test]` is added* (there is no library/CLI behavior change to unit-test).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-037` — the full-gate policy + the `cargo audit` consolidation. **Implement
  exactly the four checks; do NOT add a separate cargo-audit job.**
- `DEC-018` — the permissive license policy; **do not change** `[licenses]`
  (`allow`/`exceptions`/`confidence-threshold`) — only ADD the other sections.
- `DEC-009` — CI conventions (no secrets; minimal permissions).

### Constraints that apply

- `untrusted-input-hardening` — the supply chain is part of the threat surface.
- `no-agpl-default-deps` — already enforced by `[licenses]`; unchanged.
- `no-new-top-level-deps-without-decision` — no new runtime dep is added here.

### Prior related work

- `SPEC-001` (shipped) — the CI workflow + matrix (DEC-009); the `licenses` job
  (DEC-018) is the one extended here.

### Out of scope (for this spec specifically)

- A separate `cargo audit` job (redundant with `cargo deny check advisories`,
  DEC-037).
- A scheduled/cron advisory job (the blocking PR gate is enough for the MVP; a
  non-blocking nightly advisory scan is a possible later addition).
- Bumping dependencies for reasons OTHER than a surfaced advisory/ban (out of
  scope; this is a gate-wiring chore, not a dependency-upgrade pass).
- The threat-model verification pass + `/security-review` (backlog item #5).

## Notes for the Implementer

- **Validate against the real tool.** `cargo-deny`'s `deny.toml` schema has
  changed across versions (e.g. `[advisories]` once used
  `vulnerability`/`unmaintained` keys that are now defaults; modern schema may want
  `version = 2` under a section). Write the sections, then run `cargo deny check
  advisories bans sources licenses` and fix any schema deprecation warnings/errors
  it reports until it is clean. The tool is the source of truth, not this spec's
  example keys.
- **If an advisory surfaces** on a transitive dep with a fix available, bump it
  (`cargo update -p <crate>`); if NO fix exists, add a commented, dated
  `ignore = ["RUSTSEC-XXXX-YYYY"]` entry with a one-line reason + revisit trigger.
  Do the same for an unavoidable duplicate under `[bans] skip`. **Never** switch a
  whole check to `"allow"`/remove it to make CI pass — that defeats the spec.
- **Keep `[licenses]` byte-for-byte** as it is (DEC-018). The diff should ADD the
  three sections + change the CI `command` + the `just deny` recipe, nothing else.
- The cargo-deny CI job needs no toolchain step (the action is self-contained); do
  not add a `dtolnay/rust-toolchain` step to it.
- Record the exact `cargo deny check …` output (clean) in `## Build Completion` so
  verify can confirm the gate actually passed, not just that the YAML changed.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-036-supply-chain-gate`
- **PR (if applicable):** opened (see PR URL in session)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none (DEC-037 was written at design; no new decisions needed)
- **Deviations from spec:**
  - RUSTSEC-2024-0436 (`paste`, unmaintained) surfaced with "no safe upgrade available"; added a narrowly-scoped, commented, dated `ignore` entry per the findings policy. No whole-check disable.
- **Follow-up work identified:**
  - Monitor upstream `little_exif` and `rav1e` for dropping `paste`; remove the `ignore` entry when a safe path exists.

### Exact `cargo deny check advisories bans sources licenses` output (clean)

```
warning[duplicate]: found 2 duplicate entries for crate 'getrandom'
warning[duplicate]: found 2 duplicate entries for crate 'r-efi'
warning[duplicate]: found 2 duplicate entries for crate 'wit-bindgen'
advisories ok, bans ok, licenses ok, sources ok
```

Exit 0. The three duplicate warnings are expected (transitive version skew from
`getrandom 0.3/0.4` via `rav1e`/`tempfile`); `multiple-versions = "warn"` keeps
them visible without blocking the build.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing significant. The note "validate against the real tool" was exactly right — the schema just needed one run to confirm the `ignore` entry format (`{ id = ..., reason = ... }`) and that there were no deprecated keys to fix in this version.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraints. The spec correctly anticipated that `paste` (RUSTSEC-2024-0436) might surface and covered the exact handling path (dated `ignore` entry). The duplicate-version warnings for `getrandom`/`r-efi`/`wit-bindgen` could have been mentioned as expected noise, but they didn't block anything.

3. **If you did this task again, what would you do differently?**
   — Run `cargo deny check` before writing any config to see the advisory baseline first, then write the config to match. Saved one round-trip here by running immediately after the first draft, which was fast enough.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Two things the spec got right that paid off: (a) telling the build to
   **validate the `deny.toml` schema against the real tool** rather than trusting
   my example keys — cargo-deny's schema has drifted across versions, and the
   build adjusted accordingly; (b) a **strict findings policy** (dep-bump or a
   narrow, dated, commented ignore — never a whole-check disable). A real advisory
   *did* surface (RUSTSEC-2024-0436, `paste` unmaintained, no upstream fix), and
   because the policy was pinned, the build handled it correctly (one scoped
   ignore + revisit trigger) instead of weakening the gate. The honest read: this
   is the first STAGE-006 item whose value is *ongoing* — the gate will go red on
   future advisories with no code change, which is the point.

2. **Does any template, constraint, or decision need updating?**
   — No template change. DEC-037 records the gate + the cargo-audit
   consolidation; DEC-018 (licenses) is unchanged. One durable note: a CI step
   `name:` containing a bare colon broke YAML parsing on first push (the build
   caught + fixed it) — worth remembering for any future workflow edits. The
   `paste` ignore in `deny.toml` is now a standing item to clear when
   `little_exif`/`rav1e` drop it.

3. **Is there a follow-up spec I should write now before I forget?**
   — The final STAGE-006 item: **backlog #5 — the threat-model verification pass
   + `/security-review`** on the cumulative diff, being designed next. It is the
   stage capstone and absorbs the two tracked op/path follow-ups (the `resize`
   upscale-bomb op-param bound; the `edit --save-recipe` raw-write symlink-guard
   parity). After #5 ships, STAGE-006 — the MVP exit gate — is complete.
