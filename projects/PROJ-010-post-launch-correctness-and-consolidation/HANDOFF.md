# PROJ-010 — orchestration handoff

**Written 2026-08-16** at the end of a long orchestration session.

> **This file deliberately does NOT restate repo state.** `just status`, `just backlog`,
> `just roadmap` and `just specs-by-stage` all report correctly now — trust them over any
> summary, including this one. What follows is only what the tooling cannot show: what is
> ready to run, what is waiting on a decision, and the traps this session paid for.

---

## Read first

`/AGENTS.md`, then `just status` and `just backlog`. Then this file.

**You orchestrate; you do not build.** Build and verify go to separate CLI sessions via a
persisted prompt in `specs/prompts/`. Push the prompt to `main` before the build branches.

---

## Ready to run right now

Three specs are framed with no prompt written yet. **Run them serially** — see Trap 1.

| spec | stage | what it is |
|---|---|---|
| **SPEC-121** `[M]` | 046 | ops preserve colour type and bit depth. Fixes the RGBA widening (+12.4% bytes, measured) and 16-bit truncation across three op bodies. |
| **SPEC-122** `[M]` | 046 | `resize` resamples in linear light. Premise measured and confirmed by SPEC-120 (DEC-092). |
| **SPEC-123** `[S]` | 042 | is AVIF output byte-deterministic across thread counts? Gates two roadmap items. |

**SPEC-121 and SPEC-122 are a deliberate pair.** Same function family, **one shared DEC**, one
lockfile migration. Sequencing them together pays that migration once. Build 121 first — 122
touches the same `Resize::apply`.

**Design already removed their biggest assumed risk.** Both backlog entries flagged "this
invalidates every PROJ-007 lockfile". It does not need new machinery: `cache_key_for` includes
`crate::version()` (`src/cli/build.rs:294`), and the lockfile never promised output-hash
stability across versions (`src/build/lock.rs:32-36`). The builds **drive** that (AC-8); they do
not design it, and they stop-and-report if the contract does not hold.

**SPEC-123 is the cheapest and unblocks the most.** Its load-bearing criterion is a control: a
"deterministic" verdict is most likely to be wrong for a boring reason — the encoder ignored the
thread setting. Same shape as SPEC-120's positive control, which is the model to copy.

---

## Open, waiting on the maintainer

- **A code-review batch** was about to be shared when this session ended. It has not been seen.
- **STAGE-041 status.** Three of its four items are substantially done **outside the repo**, and
  the repo still reads `0 in flight, 4 backlog`. The maintainer will report; until then the stage
  understates reality. Its `## Amendment (2026-08-16)` carries a safe-to-start-now vs
  wait-for-STAGE-046 table — **read that before touching any launch item.**
- **The benchmark refresh.** `BENCHMARKS.md` was written at **0.5.0** on a private, uncommitted
  8-photo corpus, and predates thirteen shipped specs including the classifier fix. Half a day to
  a day, needs the maintainer's machine and photos, and **must wait for SPEC-121/122** because
  they move the numbers. Open question when it happens: whether `@squoosh/cli` stays a live row
  (it is archived and needs Node 16) or becomes a labelled historical one.

---

## Sequencing that is decided, and why

- **STAGE-046 precedes STAGE-041.** Maintainer call. Launch content publishes a
  quality-per-byte claim that the STAGE-046 defects contradict.
- **SPEC-118 (conformance matrix) is parked behind STAGE-041** — but that reasoning has weakened.
  The matrix has now missed two findings it exists to catch (`responsive` silently flattening;
  `Preserve`/`Pinned` never warning). **Worth revisiting its position.**
- **Threading order:** SPEC-123 → `par_iter run_pixel_op` → deploy-pipeline benchmark → only then
  consider within-image or wasm threading. `par_iter` reclaims a measured ~3.8× regression with no
  race risk and is also decision drift against DEC-006.

---

## Five traps this session paid for

1. **The ID space is shared even when files are not.** SPEC-119 and SPEC-120 ran in parallel,
   touched no common file — which is what was checked — and both minted **DEC-092**. `next_id`
   scans only the working tree, so a record on an unmerged branch is invisible. **Prefer one spec
   in flight.** If you must parallelize, the thing to check is IDs, not files.
2. **Cost scales with the square of message count, and anti-correlates with wall clock.**
   Measured across four builds: SPEC-116 ran 104 minutes for $11.91; SPEC-119 ran 61 for $51.24.
   **Budget prompts in exchanges (~250), never in minutes** — `cost-snippet.md` now says so.
3. **Mid-session cost readings run 40–49% low.** Measured twice. Always re-measure at session end.
4. **A squash merge can strand a push that lands near merge time.** Happened on #170 with no
   conflict and no warning, and the dropped correction propagated into two authored documents.
   Detector: compare each merged PR's `headRefOid` against its branch tip, then check whether the
   tail commits' content reached `main`. Swept 2026-08-16 — one incident, recovered.
5. **Read output before diagnosing it.** I concluded "the tooling truncates files" from line
   counts; it was a `"$B:path"` shell-variable bug printing a commit instead of a file. `rtk` was
   innocent and I had to retract it from two places. Counting told me one thing; reading seven
   lines of `commit/Merge:/Author:/Date:` told me the truth.

---

## Conventions worth not relearning

- **Verify + ship bookkeeping lands on `main` after the PR merges**, never on the branch
  (AGENTS §13). Verify sessions have twice committed cost blocks to a **detached worktree** that
  was never pushed — transcribe from their readout into the timeline, then apply at ship.
- **Re-derive every cost readout at ship.** All five this session matched to the cent; the check
  is cheap and it is the orchestrator's job per `cost-snippet.md`.
- **AGENTS §15 now carries three measured verify rules** — one revert per independent condition,
  the behavioural flip (not a binary hash) is the evidence, and a test asserting a *property of
  the defect* is not a regression guard.
- **`just archive-spec` now refuses to imply a stage is done** when un-promoted backlog items
  remain. Trust it.
- **A stage bullet is only "promoted" if it matches `**SPEC-NNN**` in bold** (`_lib.sh:212`) —
  the stage template documents the format without bold, so a stage written to its own template
  double-counts. Filed on STAGE-047.
- **Multi-line markdown bullets: rewrite the file, do not `sed`.** Line-oriented `sed` left
  orphaned continuation lines twice.
