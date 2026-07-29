# Process findings for the template maintainer — from PROJ-010, 2026-07-26/27

Everything here surfaced while running one project (PROJ-010) through frame → design → build →
verify → ship on the spec-driven scaffold. It is written to be **forwarded as-is**: each item
states the defect, how it was found, the blast radius, and whether it is fixed here or still open.

Repo: `crustyimg`, variant `claude-only`. Nothing below is crustyimg-specific unless marked.

**The one-sentence summary:** five of the defects are **new-project failures** — dormant while a
project is mid-flight, guaranteed to fire the first time someone frames the next one. That pattern
is the most valuable thing in this document, and it argues for a `project-open` preflight more than
for any individual fix.

---

## A. New-project failures (the cluster)

### A1. `just status` aborts on a project with no shipped specs — FIXED HERE

`scripts/status.sh` counted specs with `find "$dir" -name "SPEC-*.md" | wc -l`. `find` exits 1 on
a missing directory, the script runs under `set -euo pipefail`, and a freshly framed project has
no `specs/done/` until its first spec ships. The report aborted just before its Summary block,
exiting 1 — while appearing to have printed successfully.

**Fix:** a `count_files` helper in `_lib.sh` that returns 0 for a missing directory; four call
sites in `status.sh` routed through it.

**Still open in the template:** the identical pattern remains in `scripts/report_daily.sh:90-91`
and `scripts/weekly-review.sh:39,45,68`. `just report-daily` breaks on a new project the same way.

### A2. `next_id` mints duplicate SPEC and STAGE ids on a new project — FIXED HERE

`new-spec.sh` called `next_id SPEC "${PROJECT_DIR}/specs"` and `new-stage.sh` called
`next_id STAGE "${PROJECT_DIR}/stages"` — scoped to one project. But **SPEC and STAGE ids are
globally unique across the repo** (PROJ-010's stages start at 034 because PROJ-008's ended at 033).
On a brand-new project the per-project scan finds nothing and restarts at 001.

**Found by using it:** `just new-spec "…" STAGE-034 PROJ-010` produced **SPEC-001**, colliding with
PROJ-001's. A scan across all projects confirmed both the convention and the damage: 101 distinct
SPEC ids, 37 distinct STAGE ids, exactly one duplicate — the one the command had just created.

**Fix:** both callers scan `${REPO_ROOT}/projects`; `next_id`'s doc comment now states the ID space
explicitly, since the old comment ("or within a project, for SPEC/STAGE/HANDOFF") actively invited
the bug.

### A3. `just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md` — STILL OPEN

A known `find_spec` glob bug. Archiving has to be done by hand with `git mv` every time. It has
been worked around rather than fixed for long enough that the workaround is now folded into cycle
prompts, which is the wrong place for it.

### A4. `just status` counts `-timeline.md` files as specs — STILL OPEN

After archiving one spec, `status.sh` reported **"Shipped (archived): 2"** and listed both
`SPEC-109-….md` and `SPEC-109-…-timeline.md` under the `ship` cycle. `cost-audit` already excludes
timelines (`case "$f" in *-timeline.md) continue ;;`); `status.sh` does not. Every shipped count is
inflated ~2× from the first archive onward.

---

## B. Cost tracking — a rule that made fabrication the compliant answer

### B1. The measurement instruction was unsatisfiable — FIXED HERE (unmerged)

`projects/_templates/prompts/cost-snippet.md` said:

> `tokens_total: <leave null here>` — the ORCHESTRATOR fills the real number … from your Agent
> result at ship. **Do NOT invent token numbers.**

But a cycle run **interactively** rather than as a dispatched subagent has no `subagent_tokens`,
and `just cost-audit` **fails any shipped spec whose metered cycles lack a positive
`tokens_total`**. The two rules together left exactly one way to satisfy both: invent a plausible
number. **That is what happened** on this project's first build cycle, and it was caught only
because a human pushed back on the figure.

**This is the most important item in this document.** A rule whose only compliant path is
fabrication is worse than a missing rule, because the output looks conformant.

**Fix (DEC-083):** the cycle measures itself from its own session transcript — per-message `usage`
is present regardless of dispatch mode — and closes its return with a `## Cost readout` block. The
orchestrator's job changes from *sourcing* the number to *checking* it. Where the cycle did run as
a subagent, there are now two independently derived numbers to compare.

### B2. The pricing formula overstates by more than its own stated tolerance — FIXED HERE (unmerged)

`AGENTS.md` §4 and `docs/cost-tracking.md`: `estimated_usd = tokens_total × list rate`, ~80/20
input/output, **no cache discount**, self-described as "order-of-magnitude".

Measured on two consecutive cycles of one spec:

| cycle | tokens | cache-read share | flat rule | by component | overstatement |
|---|---|---|---|---|---|
| build | 65,339,132 | **98.7%** | $588 | **$43.21** | **13.6×** |
| verify | 21,152,459 | **96.3%** | $190.37 | **$17.76** | **10.7×** |

Cache reads price at 0.10× input; on a long agentic cycle they dominate volume. **14× is more than
one order of magnitude**, so the rule was outside the tolerance it advertised for itself — not
merely imprecise.

**Fix:** price each component (input, output, `cache_creation` ×1.25, `cache_read` ×0.10) and
record the rate anchors used.

**⚠ Consequence for any repo already using the flat rule:** existing entries are in a different
unit. This repo has **317 non-zero `estimated_usd` entries summing $897.98** and *does not know*
how inflated they are — it depends on whether the `subagent_tokens` they derived from counted cache
reads, which has not been established. Restating may be impossible in principle if per-component
breakdowns were never preserved. **Do not sum across the methodology boundary, and do not publish
the aggregate** — spend-per-spec is exactly the figure that reads as precise and gets repeated.

---

## C. The two process gaps behind most of the above

### C1. There is no `project-open` preflight

Items A1–A4 are all dormant-until-new-project. Suggested check, run once when a project is framed:

- `specs/` and `specs/done/` exist
- `just validate`, `just cost-audit`, `just status` each exit 0
- `next_id SPEC` and `next_id STAGE` return globally-free ids *(regression test for A2)*
- **templates are self-consistent** — the check that would have caught B1, where the snippet
  contradicted `AGENTS.md` and `cost-audit` simultaneously
- no dangling references to the previous project's stages
- tree clean, on `main`, synced with origin

### C2. There is no `stage-close` cleanup — and this one is felt as *slowness*

Two things drift during a stage and nothing catches them:

**Dangling cross-references.** Re-homing stages between projects left **four** live pointers to the
old numbers (a project brief's stage plan and count, and two sites in a tooling backlog) — found
only by a deliberate mechanical sweep with a positive control.

**Reading-load creep.** Evidence accumulated in stage files until the prescribed reading for a
single build cycle reached **~16,900 words / ~23k tokens across six documents** — 43% of it written
in the preceding two days. The maintainer noticed this as *"we've gotten much slower in the last
4–7 days"* without a visible cause. Rewriting one prompt to inline the ~8 facts it actually needed
cut it to **~5,760 words / ~8k tokens, a 66% reduction**, with no loss of content.

**Suggested checks at stage close:**
- every `STAGE-0NN` / `SPEC-0NN` cross-reference resolves
- shipped specs archived; backlog counts match reality
- **a reading-load budget** — warn when a spec's prescribed reading exceeds ~8k tokens
- evidence moved to research docs; the stage file keeps decisions and links, not measurements

The reading-load budget is the highest-value item here, because it is the one that manifests as
unexplained slowdown rather than as a visible failure.

---

## D. Method lessons worth folding into the cycle prompts

- **A mutation table needs a must-fail arm.** Verify ran `PHOTO_ENTROPY_STRONG` at 4.0 / 5.5 / 3.2
  *and* at 7.0 — a value that **must** fail. Without it, "green before the change" cannot be
  distinguished from an edit that never got compiled. The 7.0 arm was the executor's own idea; it
  should be standard.
- **Reverting a mutation does not rebuild the binary.** After a control was reverted, the compiled
  artifact was still the mutant and reported *every* photo fixture as the wrong class. Restoring
  source leaves `git status` clean and lying. Rebuild and confirm compilation before measuring; a
  result that dramatic is a build-state symptom.
- **A value measured from one specimen is not a property of its class.** A decision record here
  asserted a threshold ceiling, was corrected, and **the correction repeated the same error at
  smaller scale** — a second specimen refuted it too. Record measurements as *"the maximum observed
  across corpus X"*, never as *"the ceiling"*.
- **Don't let one session do branch surgery in a shared working tree.** Switching branches while
  another session ran gave it a ~7-minute window where the files it was testing did not exist.
  Worktree-per-session needs to be a documented default with a helper, not advice in a prompt.
- **Never read past a `remote:` warning.** Two pushes to a protected `main` succeeded with
  `Bypassed rule violations` printed, because `enforce_admins: false`. The output was scanned for
  "did it succeed" rather than read. Worth an explicit line in the git conventions.

---

## E. What is fixed here vs still open

| item | state |
|---|---|
| A1 `status.sh` pipefail | fixed (`count_files` in `_lib.sh`) |
| A1 `report_daily.sh`, `weekly-review.sh` | **open** — same bug, untouched |
| A2 `next_id` global ids | fixed (both callers) |
| A3 `advance-cycle` / `archive-spec` glob | **open** |
| A4 `status.sh` counts timelines as specs | **open** |
| B1 cost-snippet unsatisfiable | fixed, **unmerged** (branch `chore/cost-measurement-methodology`) |
| B2 flat pricing formula | fixed, **unmerged** (DEC-083, same branch) |
| B2 historical corpus (317 entries / $897.98) | **open question**, deliberately not investigated |
| C1 `project-open` preflight | **not built** |
| C2 `stage-close` cleanup + reading-load budget | **not built** |

The fixes in A1 and A2 are small and portable — `count_files`, and passing the projects root to
`next_id`. B1 and B2 are the ones worth taking upstream in full, because their failure mode is
silent conformance rather than an error.
