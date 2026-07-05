---
# A PATCH is a lightweight fix to shipped behavior (DEC-043). Lighter than a
# SPEC: no stage, collapsed patch→verify→ship cycle, keeps independent verify.

patch:
  id: PATCH-002
  type: patch
  cycle: ship                      # patch | verify | ship
  fixes: "`crustyimg <cmd> --help` leaks internal STAGE/SPEC/DEC jargon + stale \"stub\"/\"placeholder\" text into user-facing descriptions"
  complexity: S
  blocked: false

project:
  id: PROJ-001
repo:
  id: crustyimg

agents:
  implementer: claude-opus-4-8     # doc-wording change done in the main loop (judgment-heavy, tiny)
  verifier: claude-opus-4-8        # independent verify (Explore subagent — kept, DEC-043)
  created_at: 2026-07-05

references:
  decisions: [DEC-043]

# Cost: this patch's fix pass was main-loop (a small judgment-heavy doc reword, not a
# metered Sonnet subagent); verify was a metered Explore subagent; ship is main-loop.
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
        Main-loop patch pass (not a metered subagent): a doc-comment-only reword of the
        user-facing clap help in src/cli/mod.rs — the executor judgment IS the final wording,
        so a prescriptive Sonnet prompt would have added no value. Surveyed all 99 jargon
        hits, cleaned only the user-facing ones (Commands variants + #[arg] fields + GlobalArgs
        + the enum about), left internal source docs (fns, error enums, internal types) citing
        specs/DECs. Verified the RENDERED --help of all 17 subcommands is jargon-free; 206 lib
        + full suite / clippy / fmt / lean all green. 23+/26- in one file.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 46668
      estimated_usd: 0.41
      duration_minutes: 5
      recorded_at: 2026-07-05
      notes: >
        Real metered independent Explore subagent. subagent_tokens=46668, duration_ms=278498.
        Fresh-context check: rendered --help of top-level + all 17 subcommands grepped clean;
        no over-stripping / awkward wording; scope disciplined (only src/cli/mod.rs, doc
        comments only, internal docs preserved); docs/cli-reference.md exists; fmt/clippy/431
        tests/lean all green. VERDICT PASS, no defects.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: "Main-loop ship (merge PR + CHANGELOG Fixed entry + archive to patches/done/); not separately metered."
  totals:
    tokens_total: 46668
    estimated_usd: 0.41
    session_count: 3
---

# PATCH-002: strip internal jargon from user-facing `--help` text

**Third STAGE-010 (advisory-elimination) item — the UX-debt one; the last before 0.2.0.**

## Problem

The shipped clap doc-comments leak internal work-tracking jargon into end-user help:
`crustyimg view --help` reads *"Display an image in the terminal via viuer (STAGE-002; stub
in STAGE-001)"*; `--jobs` says *"placeholder; honored in STAGE-005, DEC-006"*; nearly every
subcommand summary trails a `(STAGE-0XX …)` / `(SPEC-0XX, DEC-0XX)` reference, and `apply`
says *"single-input wired here"*. Found during the v0.1.0 install smoke-test. None of this
means anything to a user installing from crates.io/Homebrew.

## Fix

Reword only the **user-facing** clap strings so help reads for end users, and remove stale
"stub"/"placeholder"/"wired here" wording:
- `GlobalArgs.jobs` → "Number of parallel workers for batch operations."
- Every `Commands` variant summary + `#[arg]` field help → drop the trailing
  `(STAGE/SPEC/DEC …)` parentheticals, keeping the descriptive content (e.g. `convert
  --max-size` keeps "(JPEG target only)", drops "SPEC-017").
- The `Commands` enum about → "The full subcommand surface (see `docs/cli-reference.md`)";
  drop the stale "Stub commands parse their args and return `CliError::NotImplemented`" line.

**Deliberately NOT touched:** internal source docs on functions, `CliError`, `QualityTarget`
/`AutoQuality`, and the module header still cite specs/DECs — that's appropriate developer
documentation and never reaches `--help`. No logic/behavior change.

## Verification (independent, kept — DEC-043)

Rendered `--help` of the top-level command **and all 17 subcommands** grepped for
`STAGE-[0-9]|SPEC-[0-9]|DEC-[0-9]|stub|placeholder|wired here` → zero hits. Gates:
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass —
nothing snapshotted the old help), `cargo build --no-default-features`. Independent Explore
verifier (Opus) confirmed no over-stripping, scope discipline (doc comments only), and the
`docs/cli-reference.md` reference exists. VERDICT PASS.

## Ship

CHANGELOG `[Unreleased] → Changed`: "`--help` text reads for end users — internal
stage/spec/decision references and stale \"stub\"/\"placeholder\" wording removed from
command and option descriptions." Archive to `patches/done/`. No stage bookkeeping (DEC-043).
Completes STAGE-010; 0.2.0 is ready to cut.

## Notes

- Executor was the main loop (Opus), not a Sonnet subagent: the change is a tiny,
  judgment-heavy doc reword where the executor's wording *is* the deliverable, so a
  prescriptive hand-off prompt would have just been me writing the final text. DEC-043's
  independent-verify gate was kept (separate Explore subagent). A small, defensible
  proportionality deviation from the lane's default "patch runs on Sonnet".

---

## Reflection (Ship)

1. **What would I do differently?**
   — Nothing on execution. Worth noting the meta-point: this "trivial" doc patch surfaced
   the same discipline as the meaty specs — verify the *rendered* artifact (actual `--help`
   output), not just the source diff, because the mapping from doc-comment → help text is
   what the user sees. Grepping source `///` lines would have missed nothing here, but
   rendering every subcommand is the honest check.

2. **Does DEC-043 or a template need updating?**
   — Minor: DEC-043 could note that the "patch runs on Sonnet" default is a cost
   optimization, not a rule — a small judgment-heavy change (like a doc reword) is fine to
   execute in the main loop as long as the independent-verify gate is kept. Recorded here;
   not weighty enough to amend DEC-043 unless the pattern recurs.
