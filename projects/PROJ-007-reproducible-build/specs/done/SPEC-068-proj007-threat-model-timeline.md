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
- [x] **build** — wrote the threat-model note (`docs/research/proj-007-threat-model.md`) + DEC-061;
  drove all 5 surfaces + 2 cross-cutting seams with hostile FILES on the real binary. Inline tightening:
  recipe **top-level** `deny_unknown_fields` (corrected the framing's premise — the `#[serde(flatten)]`
  is on `RecipeStep` not `Recipe`; closed the zero-step silent-passthrough footgun) with hostile-file
  tests; cache off-by-53 boundary test (pinned + filed); confirmed/resized/dismissed the 6 backlog items
  (exit-code map already compiler-exhaustive → resized). Watch un-clamped roots + `.to_str()→""` seams
  accepted-and-documented in DEC-061. → **PR #75**, 26/26 3-OS CI green, no new dep. Est. ~320k tok / ~$3.15.
  2026-07-10.
- [x] **verify** — fresh adversarial session. Re-drove every surface + invented new attacks
  (payload_len u64::MAX → clean miss; hostile lock paths opaque). Recipe fix + 4/5 surfaces sound — but
  **found a SHIP-BLOCKER the design + build both missed:** an unclamped target `out` is a path-traversal
  WRITE-escape reachable via `build --check` (a hostile manifest wrote bytes outside the tree while the
  note claimed writes were clamped). Punch list (fix vs accept). → user chose **FIX/clamp**. Punch fix
  pass: clamped `out` at `Target::validate` (lexical, exit 2, prepare-phase) + hostile-file regression
  test (commit a888ba2). Est. ~150k + ~60k tok. 2026-07-10.
- [x] **re-verify** — scoped adversarial re-check of the clamp. Every lexical bypass rejected
  (buried/mixed `..`, absolute; contained `dist/../dist2` still allowed), happy path + `--watch` intact,
  gates green, no new CliError variant — **code ready.** Caught a **doc-honesty blocker** (same overclaim
  class as SPEC-066): the note/DEC claimed the write-time `safe_join`/DEC-035 "second layer" catches a
  symlinked out dir; re-verify DROVE it and bytes escaped at exit 0. Doc-only punch list. → user chose
  **accept + document**: scoped the claims to `../`+absolute, recorded the symlinked-out-dir as an
  accepted, un-caught residual, filed backlog #10 (commit f421dce, CI green). Est. ~90k tok. 2026-07-10.
- [x] **ship** — squash-merged **PR #75** → main (**b8283bb**); filled the verify/re-verify/ship cost
  sessions + `cost.totals` (690k tok / ~$6.33, 6 sessions, labelled estimates §4) + ship reflection;
  timeline [x]; STAGE-024 backlog gains **#10** (canonicalize-contain out) + SPEC-068 marked shipped;
  archived spec+timeline to `done/`; `just cost-audit` + `just validate` green; brag + memory updated.
  **SPEC-068 SHIPPED — STAGE-024 LEAD done.** The reprioritized backlog is now the stage's queue; the
  decoder fuzz gate (#1, High) is next to frame. PROJ-007 does NOT close yet — STAGE-024 continues. 2026-07-10.
