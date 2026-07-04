---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-043
  type: chore                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-007
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-07-03

references:
  decisions: [DEC-042, DEC-037]
  constraints:
    - untrusted-input-hardening
    - one-spec-per-pr
  related_specs: [SPEC-041, SPEC-042]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  Repairs the supply-chain gate (red on `main` from ambient RustSec drift) so the
  release stage can proceed — adds three documented, revisit-tracked `deny.toml`
  advisory ignores (DEC-042) with a reachability assessment; no code change.

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
      recorded_at: 2026-07-03
      notes: >
        Main-loop orchestrator work, not separately metered. Discovered during the
        SPEC-042 build that `cargo deny check advisories` had gone red on `main`
        (ambient RustSec DB drift, red since 5a0660d) — 3 advisories with no fix path.
        Did the reachability assessment (crustyimg uses little_exif for binary EXIF
        only; zero XMP/XML in src → quick-xml's XML-reader vuln not on our path; plus
        STAGE-006 bounded inputs; little_exif 0.6.23 pins quick-xml ^0.37 so no
        upgrade). Authored DEC-042 (risk acceptance + revisit triggers) + the spec +
        the Sonnet build prompt. Fix = 3 documented deny.toml ignore entries (the
        established RUSTSEC-2024-0436/paste pattern). No code change.
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-03
      notes: >
        deny.toml: 3 documented advisory ignores (RUSTSEC-2026-0194/-0195 quick-xml via
        little_exif; -0192 ttf-parser via ab_glyph), each with reason + revisit trigger
        per DEC-042. cargo deny advisories now green; no code/dep change; advisories check
        still `deny`.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-043: supply chain advisory response quickxml ttfparser

## Context

**A supply-chain repair, discovered mid-launch.** During the SPEC-042 build,
`cargo deny check advisories` was found **failing on `main`** (red since `5a0660d`) —
not from any code change, but from **RustSec advisory-DB drift**: the advisory database
is time-varying, so a green `cargo deny` run can go red days later with zero code change.
Three advisories landed, **none with an available fix**:

- **RUSTSEC-2026-0194** + **RUSTSEC-2026-0195** — `quick-xml 0.37.5` (quadratic start-tag
  attribute check; unbounded namespace allocation / memory-DoS), transitive via
  **`little_exif`** (the EXIF-write lane, DEC-029). `little_exif 0.6.23` (latest) pins
  `quick-xml ^0.37`, so `≥ 0.41` (the fix) cannot be pulled — **no upgrade path**.
- **RUSTSEC-2026-0192** — `ttf-parser 0.25.1` **unmaintained** (author EOL, informational,
  not a vulnerability), transitive via **`ab_glyph`** (watermark rasterizer, DEC-032).
  **No fix exists.**

This blocks the `supply-chain (cargo-deny)` gate on `main`, the SPEC-042 PR (#46), and the
`v0.1.0` release. Per **DEC-042**, the response is three **documented, revisit-tracked
`deny.toml` ignore entries** (the established `RUSTSEC-2024-0436`/`paste` pattern), backed
by a reachability assessment — not a code or dependency change.

Parent: `STAGE-007` (release readiness). Decision: `DEC-042` (the risk acceptance + the
per-advisory rationale); `DEC-037` (the supply-chain gate this maintains).

## Goal

Make `cargo deny check advisories bans sources licenses` green again by adding three
documented `ignore` entries (RUSTSEC-2026-0194, -0195, -0192) to `deny.toml`
`[advisories]`, each with a one-line reason + a revisit trigger, exactly matching the
existing `paste` entry's style and DEC-042's rationale. No code, no dependency change.

## Inputs

- **Files to read:**
  - `decisions/DEC-042-accept-quickxml-ttfparser-advisories.md` — **authoritative**: the
    three advisories, the reachability/risk assessment, and the revisit triggers to
    encode as the `ignore` reasons.
  - `deny.toml` — the `[advisories].ignore` list; the existing `RUSTSEC-2024-0436`
    (`paste`) entry is the exact pattern to follow.
  - `decisions/DEC-037` — the supply-chain gate + the "no unexplained ignores" discipline.
- **External APIs:** none. (RustSec: https://rustsec.org/advisories/RUSTSEC-2026-0194 ,
  `-0195`, `-0192` — reference only.)
- **Related code paths:** `deny.toml` only. No `src/`.

## Outputs

- **Files created:** none.
- **Files modified:**
  - `deny.toml` — three new entries appended to `[advisories].ignore`, each
    `{ id = "RUSTSEC-2026-0XXX", reason = "..." }` with a preceding comment giving the
    dep chain, the reachability note, and the revisit trigger (mirroring the `paste`
    entry). Reference `DEC-042`.
- **New exports / Database changes:** none.

## Acceptance Criteria

- [ ] `deny.toml` `[advisories].ignore` contains entries for **RUSTSEC-2026-0194**,
  **RUSTSEC-2026-0195**, and **RUSTSEC-2026-0192**, each with a `reason` and a preceding
  comment (dep chain + revisit trigger), matching the existing `paste` entry's style and
  referencing `DEC-042`.
- [ ] `cargo deny check advisories bans sources licenses` **passes** (exit 0) —
  `advisories ok`.
- [ ] Only these three IDs are added; `yanked = "deny"` is unchanged; no other advisory
  is ignored, no `bans`/`sources`/`licenses` policy is loosened, and the check is NOT
  downgraded from `deny` to `warn`.
- [ ] No `src/`, `Cargo.toml`, or dependency change; the rest of the gate suite
  (`fmt`/`clippy`/`test`/lean/msrv) is unaffected and green.

## Failing Tests

Config-only supply-chain change — **no Rust tests**. Verification is the gate itself:

- Before: `cargo deny check advisories` **fails** on RUSTSEC-2026-0194/-0195/-0192.
- After: `cargo deny check advisories bans sources licenses` **passes** (`advisories ok`).
- `git diff` touches **only `deny.toml`** (+ the spec/DEC docs).
- `git grep -c 'RUSTSEC-2026-019' deny.toml` == 3; the check is still `deny` (not `warn`).

## Implementation Context

*Read this section (and DEC-042) before starting the build cycle.*

### Decisions that apply

- `DEC-042` — **authoritative**: accepts these three advisories with revisit triggers;
  contains the exact reachability rationale to put in the `deny.toml` comments (quick-xml
  vuln not on crustyimg's EXIF-only `little_exif` path + STAGE-006 bounded inputs + no
  upgrade since `little_exif` pins `^0.37`; ttf-parser unmaintained/no-fix, font input is
  the bundled font or explicit `--font`).
- `DEC-037` — the supply-chain gate + the discipline that every ignore is a commented,
  dated, revisit-triggered exception (keep the gate strict for everything else).

### Constraints that apply

- `untrusted-input-hardening` — the reachability argument leans on it: crustyimg's inputs
  are bounded (decode/recipe/resize limits, STAGE-006), so the accepted advisories are not
  an unbounded exposure.
- `one-spec-per-pr` — this is ONLY the advisory ignores; do not bundle the SPEC-042
  release-channel work or anything else.

### Prior related work

- `SPEC-036` / `DEC-037` — established the full `cargo deny` supply-chain gate.
- The existing `RUSTSEC-2024-0436` (`paste`) ignore in `deny.toml` — the template.
- Discovered while building `SPEC-042` (PR #46, blocked on this).

### Out of scope (for this spec specifically)

- Any **code / dependency change** — no `quick-xml`/`little_exif`/`ttf-parser`/`ab_glyph`
  upgrade or replacement (none is available; those are the DEC-042 revisit triggers).
- **SPEC-042** (release channels) — a separate PR that rebases on top of this once `main`
  is green.
- Loosening any other `deny` policy or downgrading the advisories check to `warn`.

## Notes for the Implementer

- **Copy the rationale from DEC-042 into the `deny.toml` comments** — dep chain +
  reachability + revisit trigger, one block per advisory, matching the `RUSTSEC-2024-0436`
  entry already in the file. Group the two `quick-xml` IDs together (same dep + same
  revisit trigger), the `ttf-parser` one separately.
- Suggested shape (adapt wording from DEC-042):
  ```toml
  # RUSTSEC-2026-0194 / -0195 — quick-xml 0.37.5 (quadratic attr check; NsReader
  # memory-DoS). Transitive via little_exif (EXIF-write lane). The vuln is in
  # quick-xml's XML reader (XMP); crustyimg drives little_exif for BINARY EXIF only
  # and has no XMP/XML in src, so the path is not reached with untrusted input; inputs
  # are also bounded (STAGE-006). No upgrade: little_exif 0.6.23 pins quick-xml ^0.37.
  # Revisit when little_exif bumps quick-xml (>=0.41) or is replaced. See DEC-042.
  { id = "RUSTSEC-2026-0194", reason = "quick-xml XML-reader vuln via little_exif; EXIF-only path, not reached; no upgrade (little_exif pins ^0.37). DEC-042" },
  { id = "RUSTSEC-2026-0195", reason = "quick-xml NsReader memory-DoS via little_exif; EXIF-only path, not reached; no upgrade (little_exif pins ^0.37). DEC-042" },
  # RUSTSEC-2026-0192 — ttf-parser 0.25.1 unmaintained (author EOL; informational, not
  # a vuln). Transitive via ab_glyph/owned_ttf_parser (watermark text). Font input is
  # the bundled BSD Go font or an explicit --font PATH, not untrusted. No fix exists.
  # Revisit when ab_glyph moves off ttf-parser or it becomes a real vuln. See DEC-042.
  { id = "RUSTSEC-2026-0192", reason = "ttf-parser unmaintained (no fix) via ab_glyph; font input is bundled/--font, not untrusted. DEC-042" },
  ```
- **Verify before/after:** run `cargo deny check advisories` first (see it fail on the
  three IDs), then after the edit run `cargo deny check advisories bans sources licenses`
  and confirm `advisories ok`. Do NOT touch `yanked`, `bans`, `sources`, `licenses`, or
  change the check level.
- No `src/`/`Cargo.toml` change. DEC-042 is already authored — do NOT create a new DEC.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-043-advisory-ignores`
- **PR (if applicable):** opened (see PR)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none — DEC-042 was pre-authored
- **Deviations from spec:**
  - none
- **Follow-up work identified:**
  - none (revisit triggers encoded in deny.toml comments)

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing was unclear. The spec was unusually prescriptive and complete: the
   template block in `## Notes for the Implementer` made the deny.toml edit
   mechanical. The `RUSTSEC-2024-0436`/paste entry provided an exact style target.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-042 already contained all rationale needed for the comments. The
   pre-authored DEC pattern worked well — no design decisions needed at build time.

3. **If you did this task again, what would you do differently?**
   — Nothing significant. The "confirm failure first, edit, confirm pass" order was
   correct and the gates ran fast (config-only, no recompile needed for cargo deny).

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
