# SPEC-121 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** `Invert`, `Resize` and `Watermark` now widen to work at the input's own bit
depth and narrow back losslessly on write, instead of collapsing everything to RGBA8. `resize`,
`thumbnail`, `edit --invert` and flagship **`web`** preserve colour type and bit depth.

**PR #181**, branch `fix/spec-121-ops-preserve-colour-type-and-bit-depth`, head `4391e06`.
DEC-095 (shared with SPEC-122). Cycle advanced to `verify`.

## Your worktree — detached, read-only

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec121-verify 4391e06
```

Detached so you cannot commit onto the branch under review. **Make no commits.** AGENTS §13 puts
verify bookkeeping on `main` after merge — anything you write here goes nowhere. Emit your cost
readout and verdict in your **return message**; that is the deliverable.

## Read in order

1. **The spec** — `.../specs/SPEC-121-ops-preserve-colour-type-and-bit-depth.md`, all 10 ACs, 5
   design calls, 8 failing tests, and its `## Build Completion`.
2. **DEC-095** — and note it is **shared with SPEC-122**, which amends it next. Scope it for that.
3. **The diff** — `src/operation/mod.rs` (+348/-…), `src/sink/mod.rs` (+31), `src/image/mod.rs`
   (+8), `tests/colour_type_preservation.rs` (new, 356), `tests/sink.rs` (+89).
4. **`/AGENTS.md`** §4, §12, §13, **§15** — especially the three measured verify rules.

## Three specific leads — found by the orchestrator, NOT adjudicated

I checked these before writing the prompt. **Rule on each; do not assume my reading is right.**

### Lead 1 — ⚠ the sweep looks incomplete, and the miss gates future work

AC's sweep required correcting *"crustyimg is 8-bit internally"* **wherever it is written**. The
build corrected `docs/lab-plan-2026-08.md` and `docs/roadmap.md`. But **`docs/backlog.md:700`**
still reads:

> *"the pipeline is 8-bit throughout (`to_rgba8()` at `src/operation/mod.rs:197,396,816,817`), so a
> grade is quantized to 256 levels per channel"*

Read its context. On my reading it is a **live premise**, not a description of the defect: it sits
in the linear-light entry's "second consumer" section, arguing that grading ops (`.cube` LUT,
curves, exposure) have a *correctness* defect — and it explicitly **gates the LUT entry**. If it is
live, this spec has just made it false, and it will mislead precisely whoever picks up the grading
work. **Rule: was it in scope?**

Other hits (the spec itself, the build prompt, `docs/backlog.md:997`, DEC-095's own description)
look like legitimate historical/defect-describing text that should NOT be rewritten. Say which.

⚠ This is Sonnet's one measured weakness in this repo — **sweep thoroughness**. Treat the sweep as
the highest-risk part of this build, and **cite your own grep and its scope**
[[mechanical-sweeps-need-a-mechanical-check]].

Also: **STAGE-046's bullet** (`:246`) says correcting the claim is *"part of the colour-type spec,
not a follow-up"* and is still unchecked. Should it be marked?

### Lead 2 — `src/image/mod.rs` is touched but is not in the spec's Outputs

The spec's Outputs name `src/operation/mod.rs`, `src/sink/mod.rs` and `tests/`. The diff also
carries **8 lines in `src/image/mod.rs`**. Small, possibly necessary — but it is an unlisted
deviation. **Is it justified, and is it recorded in `## Build Completion`'s deviations?**

### Lead 3 — ⚖ AC-8's finding needs a ruling, and it is load-bearing beyond this spec

AC-8 said: drive the migration, and **"if the contract does not hold, stop and report."** The build
reports the safety net *"only fires on an actual version bump"* — and **filed it to STAGE-042
rather than stopping.**

That matters well past SPEC-121: **SPEC-122's Call 5 and SPEC-124 both rest on the same "the
migration already exists" reasoning.** If the contract is weaker than design claimed, three specs
inherit the gap.

Rule on: (a) is "filed, not stopped" the right call, or did AC-8's stop condition fire? (b) what
exactly is the residual exposure between releases? (c) is it filed where **tooling reads** — a
STAGE-042 checkbox that `just backlog` surfaces, not prose? *(SPEC-123's punch list caught exactly
this: a real finding filed in `docs/backlog.md`, which no command reads.)*

## The ACs that carry the most weight

- **AC-3 — the lossless-only control.** An RGBA input with a genuinely translucent pixel must stay
  RGBA. *"Always narrow"* passes AC-1 and AC-2 while destroying real transparency. **This is the
  test that separates the fix from a plausible regression** — confirm it can actually fail.
- **AC-9 — three independent reverts.** The build claims one op body per test, driven via three
  real reverts. Confirm each revert turns **only its own** tests red and leaves the other two
  green — that is what proves independence rather than co-dependence (AGENTS §15, rule 1). **The
  evidence is the behavioural flip, not a binary hash.**
- **AC-4 — `Watermark` both directions.** RGBA only when the overlay contributed non-opaque
  samples; narrows otherwise. Both tested?
- **AC-7 — byte-identical.** `convert`, `optimize`, `auto-orient` unchanged vs `main`. Build your
  own `main` baseline rather than trusting the claim.
- **AC-6** — the byte win asserted, not assumed. **AC-5** — the 8-bit downgrade diagnostic pinned
  by a test asserting the message.
- **AC-10** — clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential. Then **read the CI
  legs individually** at the true head `4391e06`; a green summary is not a matrix.

## Also check

- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref**, or a
  clean checkout reports "no changed files" and exits 0, a green that cannot go red.
- **DEC-095** — `affected_scope` covers `src/operation/**` and `src/sink/**`; confidence honest
  (§17); and **scoped so SPEC-122 can amend rather than mint a second decision.**
- **16-bit → lossy behaviour**: the Call 3 diagnostic goes to **stderr**, not stdout (AGENTS §11 —
  `-o -` must stay clean).
- `cost.sessions` carries design + build; build reflection candid.

## Guardrails

- **Read-only. No commits.** Do not fix what you find — a punch list is an output, not an edit.
- **Do not merge. Do not bump the version.**
- **Budget ~200 exchanges.** ⚠ The build ran **555** against a ~250 budget and cost **$58.50**.
- **Never poll CI** — background it: `gh pr checks 181 --watch --interval 30`. Take your cost
  reading **after** it settles.
- macOS has no `timeout(1)`. A piped command reports the pipe's exit code — redirect and read `$?`.

## When you finish, in this order

1. **Make no commits.**
2. **Emit your `## Cost readout` block** (see `projects/_templates/prompts/cost-snippet.md`).
   Identify your transcript by content, never by "newest `.jsonl`". Price per component at the
   anchors `.message.model` reports (expected **Opus**: $5/$25 per MTok, cache_creation ×1.25,
   cache_read ×0.10 of input).
3. **Verdict** — ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED, with rulings on all three leads stated
   explicitly. They are what the orchestrator cannot decide without you.
