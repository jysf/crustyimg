# PROJ-010 — orchestration handoff

> ⚠ **SUPERSEDED 2026-08-23 for day-to-day work.** PROJ-011 is the active project; its handoff is
> `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`. **Keep this file** — its measured
> evidence and its traps still hold, and PROJ-010 remains open as the correctness lane with ~24
> actionable items (see `TRIAGE-2026-08-22.md`). What is stale here is the *sequencing*: 0.7.1 has
> shipped, and SPEC-124/125 are both done and archived.


**Rewritten 2026-08-20.** Supersedes the 2026-08-16 version, which predates the STAGE-046 wave.

> **This file deliberately does NOT restate repo state.** `just status`, `just backlog`,
> `just roadmap` and `just specs-by-stage` all report correctly — trust them over any summary,
> including this one. What follows is only what the tooling cannot show.

---

## Read first

`/AGENTS.md`, then `just status` and `just backlog`. Then this file.

**You orchestrate; you do not build.** Build and verify go to separate CLI sessions via a persisted
prompt in `specs/prompts/`, pushed to `main` **before** the branch is cut.

---

## Where the wave stands

**Three specs shipped this session**, all on `main`:

| spec | what | cost |
|---|---|---|
| **SPEC-123** | AVIF thread settings never reach the encoder (DEC-094) | $60.33 |
| **SPEC-121** | ops preserve colour type and bit depth (DEC-095) | $91.17 |
| **SPEC-122** | `resize` resamples in linear light (DEC-095 amended) | $139.61 |

**In flight:** **PR #184 — SPEC-124**, "pin the AVIF encoder's tile count to 1". The build ran and
opened the PR; **its readout has not been processed by an orchestrator yet.** Check the PR, get the
cost readout, re-derive it, then write the verify prompt.

**Framed, not started:** SPEC-125 (lossless WebP silent depth halving), SPEC-118 (conformance
matrix, still parked).

## The release gate — this is the sequencing that matters

**The tag is the pivot: byte-changers before it, measurements after it.**

`cache_key_for` includes `crate::version()`, so the lockfile migration is keyed on a version bump.
SPEC-121/122/124 share **one** migration only if they land in the same release.

```
SPEC-124  →  SPEC-125  →  tag 0.7.1  →  STAGE-041's measured items
```

- **SPEC-125** is reporting-only (no byte change), so it is the flexible one.
- **STAGE-041's benchmark refresh and install verification must wait for the tag.** Its
  `## Amendment (2026-08-16)` table said they waited on STAGE-046 — 121/122 have now shipped, so
  they wait on **124** and the tag instead. Its three ✅ items (publication plan, hostile answers,
  RAW-split correction) need nothing and can run any time.
- **The `U16x4` working-type probe is deliberately NOT in 0.7.1** (maintainer, 2026-08-20). Filed
  on STAGE-046 with the numbers. It is an optimization and may carry its own migration later.

## Open, waiting on the maintainer

- **STAGE-041's real status.** The repo reads `0 in flight, 4 backlog`; three of the four are
  substantially done **outside** the repo. Unreported since 2026-08-16. **Do not re-plan STAGE-041
  against what the repo says.**
- **A code-review batch**, never shared. Outstanding since 2026-08-16.

---

## Traps this session paid for

1. **⚡ NEVER POLL CI. Background it and leave it alone.**
   `Bash(run_in_background: true): gh pr checks <PR> --watch --interval 30`. It burns nothing while
   waiting and re-invokes on exit. **Measured: SPEC-122's build spent ~$60 of $103.60 polling;
   SPEC-123's spent $5.80.** SPEC-122's build prompt carried **no CI instruction at all** — the
   punch-list cycle, run from a prompt that did, polled nothing. Cleanest controlled result here.
2. **A green local matrix does not predict CI.** `main` went red **without a commit** when stable
   floated to 1.98 and added the `chunks_exact` lint. A local matrix runs the toolchain installed;
   CI resolves `stable`. SPEC-122's punch list reported twelve local exit-0s against eight red CI
   legs. Split the CI fix to its own PR (#183), then `update-branch` the spec PR.
3. **State design findings as PRIORS, not conclusions.** SPEC-123's spec and prompt both asserted
   "non-deterministic by construction" from reading `ravif`'s tile formula. Wrong — the feature that
   makes the lever exist is off. The build then had to refute a *stated position* and correct five
   documents. **A measurement spec's cost is set by whether its premise survives**, which is
   unknowable at framing: SPEC-120 held → $8.69; SPEC-123 was wrong → $60.33, same shape.
4. **File findings where `just backlog` reads — a stage's `## Spec Backlog`, as `- [ ]`.**
   Failed **three times** this session: SPEC-121's WebP finding and SPEC-123's AC-7 deferral both
   went to `docs/backlog.md` (read by no command), and the orchestrator put an item in
   `## Design Notes` by inserting before `## Dependencies`. **Run `just backlog` and read it back.**
5. **Orchestrator-scoped sweeps were under-scoped twice.** SPEC-121: 4 live premises, 1 named.
   SPEC-122: 4 locations, 3 named. **Require the grep be enumerated first, and its scope cited.**
6. **An instruction that cannot be satisfied is a defect in the same class as a test that cannot
   fail.** A punch-list item asked for an opaque composite to narrow AND a translucent overlay to
   keep RGBA — impossible for an alpha-less base. The cycle caught it and recorded a deviation.
7. **zsh, twice.** It does **not** word-split unquoted parameters, so `for f in $files` iterates
   once over the whole blob and every per-file check silently passes — use
   `while IFS= read -r`. And `$B:tests/...` eats `:t` as a path modifier — write `"${B}:path"`.
   Both were caught only by a **positive control** whose answer was already known.
8. **Keep the spec's `implementer` in sync with the dispatch.** SPEC-122's prompt said Sonnet while
   the dispatch used `--model opus`; the cycle had to flag the mismatch itself.

## What is working, and worth keeping

- **Verify is READ-ONLY** (fixed 2026-08-17). It makes no commits, emits a `## Cost readout` and a
  verdict; the **orchestrator** applies cost and runs `advance-cycle` on `main` at ship. The old
  template told verify to write from a detached worktree that is never pushed, and had stranded a
  cost block twice.
- **Verify is the cheapest and most valuable cycle, three waves running** — $11.08, $14.16, $15.82
  against builds at $46–$104. **Every substantive defect this session came from a verify pass or a
  punch list, not a build.** Argues for shorter builds and more review.
- **Reserve DEC ids in the prompt.** `next_id` scans only the working tree, so a record on an
  unmerged branch is invisible (SPEC-119 and SPEC-120 both minted DEC-092). Highest is **DEC-096**
  (reserved for SPEC-124).
- **Re-derive every cost readout at ship.** All of this session's matched to the cent. One
  orchestrator arithmetic slip (a token total off by 1,000) was caught this way.
- **Budget prompts in exchanges (~150–250), never minutes.** SPEC-121's build ran 555 against ~250
  and the checkpoint never fired; SPEC-122's ran 608.
