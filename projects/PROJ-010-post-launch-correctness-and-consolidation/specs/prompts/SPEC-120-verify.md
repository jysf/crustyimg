# SPEC-120 — VERIFY prompt

Cycle: **verify**. **New session — do not continue from the build session.**

**What shipped:** a measurement, not behaviour. PR
[#175](https://github.com/jysf/crustyimg/pull/175), branch
`chore/spec-120-measure-linear-light-premise`. Five files: DEC-092, `docs/backlog.md`, the spec,
and two harnesses (`scripts/spec120_linear_light.py`, `examples/spec120_linear_probe.rs`).
**No `src/`, no `tests/`.**

**Verdict:** ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

**The build's own verdict:** *premise holds, spec the fix* — plus a refutation of the same backlog
entry's second half.

## Read in order

1. **The spec** — `.../specs/SPEC-120-measure-the-linear-light-premise.md`, in full, **including
   `## Build Completion`**. Note it has **no failing tests by design**; AC-2's positive control
   plays that role.
2. **DEC-092** — the decision this spec exists to produce.
3. **The harnesses.** `scripts/spec120_linear_light.py` and `examples/spec120_linear_probe.rs`.
   **These are the artifact.** A measurement nobody can re-derive is an anecdote.
4. **`docs/backlog.md`**'s linear-light entry, before and after.

## Work already done for you — confirm, don't redo

- **Cost is arithmetically correct.** `168 + 80,568 + 179,916 + 11,104,778 = 11,365,430` ✓, priced
  per component at Opus anchors = **$8.6919** against `$8.69` recorded ✓. **Do not re-price.**
- **AC-7 holds.** `git diff --stat origin/main...<branch> -- src/ tests/` is empty. The prototype
  lives in `examples/`, which the spec permits and which cargo compiles without touching the
  library.
- **The alpha refutation is verified at the dependency source**, not just taken from the readout:
  `fast_image_resize-6.0.0/src/resizer.rs:52-60` — `impl Default for ResizeOptions` sets
  `mul_div_alpha: true`, and `:63-64` shows `new()` *is* `Default::default()`. **The premultiplied
  half of the backlog entry is genuinely false.**
- **DEC-092 is correctly numbered** (highest across all refs was DEC-091).

## The four things genuinely open

### 1. Is the reference independent enough to carry the verdict? ← the one that matters

The reference is **ImageMagick 7.1.2-29 Q16-HDRI**, `-colorspace RGB -filter Lanczos … -colorspace
sRGB`. The spec demanded independence and got a different program — good.

**But the build flagged its own weakness, and it is the right one to scrutinise:** *"the prototype
scores ~100 partly because it is the same algorithm as the reference; the load-bearing number is
today's score against a correct reference, not the delta's size."*

So: the ~100 and the Δ column are **partly self-fulfilling** — a linear-light Lanczos prototype
scored against a linear-light Lanczos reference should agree. **The claim that survives is the
first number: today's shipped path scores 70.45 and 84.45 against an independent correct
reference.** Rule on whether the verdict rests only on that, and whether DEC-092 and the backlog
entry state it that way rather than leading with the flattering delta.

### 2. The two metrics disagree in magnitude — confirm the explanation, don't accept it

On `graphic_large.png` the mean signed luminance error is **−0.44%** while the perceptual penalty
is **29.55 points**. The build's explanation: the error concentrates at edges (max local 0.213 vs
mean absolute 0.0023, ~90×), so a *mean* understates the defect on exactly the content class the
premise says is worst hit.

That is plausible and it is the honest way round. **Check it**: the harness should let you confirm
the ~90× locality claim directly. If it holds, it is a finding worth promoting — it says the
physical metric the spec added as a safety net is the weaker of the two on realistic content.

### 3. AC-6 reproducibility — re-derive it yourself

The build reports the harness re-run twice with byte-identical output (`diff` exit 0). The
synthetic worst case is **generated at run time, not committed** — deliberately, because
`bench/corpus/` is enumerated by `just bench` and scanned by `bench_corpus_is_license_clean`.
That reasoning is sound, but it makes reproducibility depend on the generator being deterministic.

**Run the harness yourself and confirm the numbers land in the same place.** If ImageMagick is not
installed in your environment, say so and state which rows you could not re-derive.

### 4. Does AC-2's positive control actually establish what it claims?

−88.07% physical error → a **163.85-point** SSIMULACRA2 swing. That is a large, clean signal and
it is the reason the realistic rows are readable rather than an uninterpretable null.

**Confirm the synthetic case is worst-case-*like*, not degenerate.** A control so extreme that no
real image resembles it would prove the metric can see *something*, not that it can see *this
defect at realistic amplitude*. The realistic rows (70.45, 84.45) sit far from both ends, which is
reassuring — say whether you agree.

## The rest of the checklist

- **All 8 ACs walked**, against the spec's completion table.
- **AC-4's verdict is one of three permitted outcomes** — the build chose *premise holds*. The
  third option (*"SSIMULACRA2 cannot settle this"*) was live and was closed by AC-2 firing.
  Confirm that reasoning is recorded, not just the conclusion.
- **DEC-092's `affected_scope`** should cover `src/operation/**` (verdict = proceed).
- **Decision drift:** `./scripts/decisions-audit.sh --changed main`. DEC-019 governs here and the
  spec explicitly permitted questioning whether it is the right oracle *for this question* —
  finding *for* it is not drift either way.
- **Constraints:** `clippy-fmt-clean`; `one-spec-per-pr`. `test-before-implementation` **does not
  apply** — do not treat its absence as a finding.
- **`cost.sessions`**: design null-with-note, build measured.
- **CI legs read individually.** This PR touches `examples/`, which is Rust — so the heavy jobs
  run and are *not* skipped as docs-only.

## A drift the measurement surfaced — worth a line in your report

**`AGENTS.md` §5 says `fast_image_resize` 5. The lockfile says 6.0.0**, and DEC-092's argument
depends on 6.0.0's `Default`. The doc is stale. Not this spec's job to fix, but confirm the
version your reading relies on and note the drift so it gets filed.

## Guardrails

- **Own git worktree**, off the PR branch (`--detach` at the tip if the build worktree still holds
  the branch).
- **Do not fix what you find.** Verify reports.
- **Do not merge. Do not bump the version.**
- `git commit -s` (DCO). macOS has no `timeout(1)`. **A piped command reports the pipe's exit
  code.**
- **Budget: ~45 minutes.** This is five files, no source changes, and the expensive part is
  re-running one harness.

## When you finish, in this order

1. Append a verify cost session entry to `cost.sessions`.
2. Run `just advance-cycle SPEC-120 ship`, and **CONFIRM it moved** (`git diff` shows the `cycle:`
   line change; it reports success even when it changes nothing).
3. Give the verdict, with items 1–4 above each explicitly ruled on.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by content, never
by recency. Price per component at your own model's anchors. **Measure at session end** — measured
twice in this project, mid-session readings ran 40% and 49% low.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
