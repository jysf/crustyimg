# SPEC-068 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — the STAGE-024 LEAD: a systematic **threat-model / attack-surface review** of
  PROJ-007's five new untrusted-input surfaces (build manifest, recipe files, `.crustyimg/` cache
  store, committed lockfile, `--watch` tree), the SPEC-037 shape for this wave. Deliverable = a
  written threat-model note (`docs/research/proj-007-threat-model.md`: per-surface entry point →
  guards in place → adversarial inputs driven → verdict → residual risk) + inline hardening +
  hostile-input regression tests for any clear small security defect + a **reprioritized STAGE-024
  backlog** (confirm/resize/dismiss the other 6 candidate items) + **DEC-061** (verdicts + accepted
  risks). Grounded in a firsthand surface MAP with file:line anchors + existing guards (verify-and-
  tighten, not re-derive). Suspects to confirm-or-dismiss (NOT verdicts): recipe files lack
  `deny_unknown_fields` (blocked by `#[serde(flatten)]`); cache store/read bound off-by-53 (silent
  miss band); watch roots un-clamped (`source="../.."` escapes the tree); silent `.to_str()→""`
  seams; exit-code `code()` match is ALREADY compiler-exhaustive so that item *resizes*. Failing
  Tests = hostile-FILE drivers (malformed manifest/recipe/lock, hand-corrupted + near-cap cache
  entry, escaping watch root, non-UTF-8 stem) against the real binary — never Rust-constructed
  structs. **No new dep.** Framing, 2026-07-10.
- [ ] **build** — write the threat-model note + DEC-061; drive each surface with hostile input on the
  real binary; apply the small security-relevant tightenings (recipe unknown-key, watch-root clamp,
  reachable `.to_str()→""` seams) with hostile-file regression tests, OR accept-and-document in
  DEC-061; CONFIRM + severity-rank the 6 remaining backlog items (do NOT implement them — each is its
  own spec unless a trivial fold-in). Gates: default + lean + `just deny` (unchanged) + clippy + fmt
  + `just validate`.
- [ ] **verify** — fresh **adversarial** session (the framing architect is biased — attack, don't
  re-read). Re-drive the hostile inputs on the real binary; confirm each note verdict is earned by an
  attack that ran, not asserted; confirm every security-relevant finding is fixed-with-test or
  accepted-in-DEC-061; sanity-check the reprioritized backlog is actionable. Gate matrix green, no new
  dep.
- [ ] **ship** — merge PR; build/verify/ship cost sessions + totals + reflection; archive to done/.
  STAGE-024 backlog: SPEC-068 shipped → the review's reprioritized backlog becomes the stage's
  remaining specs. (STAGE-024 does not complete here — the fuzz gate + confirmed items follow.)
