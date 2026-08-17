---
task:
  id: SPEC-123
  type: task
  cycle: design
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5
  created_at: 2026-08-16

references:
  decisions:
    - DEC-058
    - DEC-077
  constraints:
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs:
    - SPEC-120
    - SPEC-091

value_link: >
  The repo's reproducible-build story — `build --frozen`, the lockfile, the
  cache key — rests on byte-stable output. Upstream gives no such guarantee for
  AVIF and has a filed nondeterminism bug. Nobody has measured whether the claim
  we already ship is true.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Framed off the
        `docs/backlog.md` entry that says "measure before claiming either way",
        and because two separate roadmap items (encoder threading, the deploy
        pipeline benchmark) are gated on the answer.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-123: is AVIF output byte-deterministic across thread counts?

## Context

`docs/backlog.md`:

> **AVIF byte-determinism is unbacked upstream.** `aomenc`/`vpxenc` ship
> `-D, --debug` *to become* deterministic; rav1e has no guarantee and a filed
> nondeterminism bug (#2781). **If crustyimg's AVIF is not deterministic across
> thread counts, existing "reproducible" language is a false claim.**
> **Measure before claiming either way.**

That has never been measured, and three shipped things assume the answer:

- **`build --frozen`** and the lockfile. `src/build/lock.rs` records `hash` as
  *"the observed output bytes"*, promised stable *within a machine* (STAGE-021's
  determinism experiment) but not across arch/OS/codec versions. **Thread count is
  not in that list** — it is neither the machine nor the codec version.
- **The cache key** (DEC-058) — `version + features + recipe + quality + input
  ext + input content`. **Thread count is not a component.** If output varies with
  it, two runs on one machine can disagree while the key says they must not.
- **Any future threading work.** Encoder threading is filed as a probe in
  PROJ-010's brief; `par_iter run_pixel_op` is filed as a SPEC-091 follow-up.
  Both are gated on this answer, and neither should be scoped before it.

**This spec ships no behaviour.** Its deliverable is a measurement and a DEC.

## The design calls — settled here

### Call 1 — vary ONE thing, and prove the variable moved

The measurement is: same input, same version, same features, same machine,
**different thread counts** → compare SHA-256 of the output bytes.

**A control is required in both directions.** Prove the thread count actually
changed the work done — an encoder that silently ignores the setting produces
identical bytes and looks like a clean pass
[[a-control-you-never-verified-applied-is-not-a-control]]. Show a wall-clock or
CPU-time difference between the counts, or instrument the thread pool. **Without
that, a "deterministic" verdict is unearned.**

### Call 2 — test what actually ships, at the surface users touch

The claim under test is about **crustyimg's output**, not rav1e's. Drive the
binary (`convert --format avif`, `web`, `optimize`), not a library harness.
Include the **lean build** — `--no-default-features` drops the AVIF encoder, so
the matrix legs differ in what they can even produce.

### Call 3 — three outcomes, and two of them are findings

- **Deterministic across thread counts** → the reproducible language is safe, and
  threading work is unblocked on this axis. Record it.
- **Non-deterministic** → **existing shipped language is false** and that is the
  finding: `RELEASING.md`, the lockfile docs and any "reproducible" claim need
  correcting, and encoder threading needs a determinism story before it is scoped.
- **The encoder ignores the thread setting** → the question is moot *today* and
  becomes live the moment anyone changes it. Record it as such; do not report
  "deterministic".

### Call 4 — also answer the cheaper adjacent question

While the harness exists: is output byte-identical **run-to-run at a fixed thread
count** on one machine? STAGE-021 measured that once; it is the narrower claim the
lockfile actually makes, and re-confirming it costs one extra loop.

## Inputs

- `docs/backlog.md`'s determinism entry.
- `src/build/lock.rs:20-45` — what is promised, and in what terms.
- `src/cli/build.rs:275-302` — the cache-key components.
- **DEC-077 / SPEC-091** — why AVIF *decode* is pinned to one thread. Different
  code path; read it so the two are not conflated in the write-up.
- `bench/corpus/` for inputs; the harness shape from `scripts/spec120_linear_light.py`.

## Outputs

- **A DEC** recording the verdict either way, with the measurements and the
  control. `affected_scope`: `src/sink/**` if the answer constrains encoding,
  `[]` if it is purely a documentation finding.
- **The harness**, committed, so the number can be re-derived rather than trusted.
- **Corrections to any shipped "reproducible" language** if Call 3's second branch
  fires — in the same PR, since a false claim should not outlive its disproof.
- **No `src/` behaviour change.**

## Acceptance Criteria

- [ ] **AC-1.** Output SHA-256 compared across **at least three thread counts**
      (1, a middle value, all cores) on the same input, version, features and
      machine. Report the hashes, not a verdict.
- [ ] **AC-2.** **The control fires** — evidence that thread count changed the
      work (timing delta or instrumentation). A null result without this is
      unearned.
- [ ] **AC-3.** Driven through the **shipped binary** on `convert --format avif`,
      `web` and `optimize`, not a library harness.
- [ ] **AC-4.** **Run-to-run stability at a fixed thread count** re-confirmed
      (Call 4).
- [ ] **AC-5.** A verdict stated as exactly one of Call 3's three outcomes.
- [ ] **AC-6.** If non-deterministic: **every shipped "reproducible" claim
      located and corrected**, with the grep cited
      [[mechanical-sweeps-need-a-mechanical-check]].
- [ ] **AC-7.** **No functional `src/` change** — `git diff` against `main` shows
      none; the shipped test suite untouched and green.
- [ ] **AC-8.** Reproducible from the committed harness — re-run and confirm the
      numbers land in the same place.

## Failing Tests

**None, and that is correct** — this is a measurement. AC-2's control is the
load-bearing criterion in their place, exactly as in SPEC-120.

## Implementation Context

### Out of scope
- **Making** AVIF deterministic. If it isn't, that is a finding, not this spec's
  fix.
- Encoder threading and `par_iter run_pixel_op` — both gated on this.
- Decode threading (DEC-077 settled it).

## Notes for the Implementer

- **Report hashes, not conclusions.** The verdict follows from the table.
- **A "deterministic" answer is the one most likely to be wrong for a boring
  reason** — the setting was ignored. Call 1's control is what separates them.
- **Budget: S.** Past ~2 hours, report what you have.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-123 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
