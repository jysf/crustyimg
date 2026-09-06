# SPEC-127 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** `Recipe` gains `format` and `quality`, gated behind `version = "2"`, honoured
by one precedence rule across `apply`, `build` and `wasm::transform` — with `version = "1"` still
valid, still serialising as `"1"`, and no other verb's bytes moving.

## ⚠ The PR is OPEN and must NOT merge

**PR #188, branch `feat/spec-127-recipe-format-quality`, head `a626ec0`. Review the BRANCH.**
Base ref for every diff and for `decisions-audit` is **`main`** (merge-base `a0cda1f`).

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec127-verify feat/spec-127-recipe-format-quality
```

⛔ **Do not merge, do not bump the version, do not cut a release.** Batches with the rest of
STAGE-050 as PROJ-011's single lockfile migration.

**Make no commits.** Your verdict and `## Cost readout` go in the return message (AGENTS §13).

## Already settled — do NOT re-derive

Checked by the orchestrator against the branch. Don't spend budget re-confirming:

1. **PR #188 is OPEN, MERGEABLE/CLEAN, head `a626ec0`, CI 16 SUCCESS + 6 release-only skips**,
   nothing failing or pending. Snapshot taken directly at the true head. **Never poll CI.**
2. **16 files changed**, matching the handback exactly — including `docs/api-contract.md` **and**
   `docs/data-model.md` (the build did more than the spec required here).
3. **DEC-099 uses the block-list `affected_scope`** — 7 paths, both docs included. The audit can
   read it. (An inline array would be silently dropped; that's why the prompt demanded this form.)
4. **All seven named failing tests exist**, in the files the spec named. `cycle: verify`, timeline
   build entry `[x]`.

---

## Six specific things

### 1 — ⚡ THE COST FIGURE DOES NOT SURVIVE A SANITY CHECK. This is the item.

The build recorded **$38.49 / 120,553,651 tokens**, and the per-component arithmetic is *exact*
($0.0015 + $0.1067 + $2.4143 + $35.9707). **The arithmetic being right is not the question.** The
inputs are.

`output` is recorded as **7,116 tokens** — for a cycle that produced **1505 insertions across 16
files**, plus DEC-099, plus commits. 7,116 output tokens is roughly 5 KB of text; the DEC alone
exceeds that. Against this project's own three prior metered cycles:

| cycle | output | cache_read | ratio |
|---|---:|---:|---:|
| SPEC-126 build (Sonnet) | 262,007 | 104,206,098 | 398:1 |
| SPEC-126 verify (Opus) | 104,013 | 22,292,121 | 214:1 |
| SPEC-126 re-approve (Opus) | 136,135 | 18,626,651 | 137:1 |
| **SPEC-127 build (Sonnet)** | **7,116** | **119,902,211** | **16,850:1** |

**42× off the established pattern, in the direction that says the build barely wrote anything —
while it wrote more code than SPEC-126 did.**

**Two hypotheses. Drive them; do not pick one by argument.**

- **H1 — `cache_read` is over-counted.** If `usage.cache_read_input_tokens` is *cumulative per
  call* rather than per-call incremental, summing it across 254 calls inflates it enormously. The
  build's own note is evidence for this: it reports the **last call's cumulative usage as 645,167**
  and the Agent tool's `subagent_tokens` as **649,005** — within 0.6 %. If cache_read is
  cumulative, the true total is nearer **649 K**, the real cost is nearer **$0.65**, and
  `subagent_tokens` was right all along.
- **H2 — `output` is under-counted**, e.g. the dedup by `.message.id` kept one `usage` line per
  message and dropped output content blocks.

⚠ **Whichever it is, this is not a SPEC-127 problem — it is potentially a PROJECT-WIDE one.** The
same summing method produced SPEC-126's `cost.totals` ($68.82) and every cost figure in
PROJ-010's eighteen shipped specs. It feeds `just cost-audit`, the `cost-data` CI job, and both
reports. **And the build's conclusion may be exactly backwards:** it flagged `subagent_tokens` as
unreliable and rejected it, when its own numbers are equally consistent with `subagent_tokens`
being correct and the sum being the error.

**What to do:** read the transcript structure yourself and determine whether `cache_read` is
cumulative or incremental — one positive control settles it (compare two adjacent calls' figures
against the delta). Then state which hypothesis holds. **Do not silently accept $38.49, and do not
silently replace it either** — say what the number should be and on what evidence. If it is
project-wide, say so; it needs its own filed item, not a quiet edit here.

### 2 — AC-7's negative controls: one revert per independent condition

Call 1 (version gate), Call 2 (precedence) and Call 3 (wasm) were declared independent. **Drive
all three reverts yourself**, one at a time, and confirm each flips **only** its own tests. AGENTS
§15: the evidence is the **behavioural flip**, never a hash — a debug rebuild from byte-identical
source already produces a different binary.

⚠ SPEC-126's re-approve found that a combined test can hide this: it died at the first failing
arity, so a "the other sub-case stayed green" claim was true only by assertion order. Check the
new tests are split where the conditions are independent.

### 3 — ⚡ Call 1's strand guard is the highest-consequence line in the spec

`to_toml` **must still emit `version = "1"`** for a recipe using neither new field. Emitting `"2"`
unconditionally strands every existing recipe on the next `--save-recipe` — and it would **look
like it worked**, because the new binary reads both.

Drive it: round-trip a v1 recipe through `--save-recipe` on the branch binary and read the emitted
version. Then confirm `v1_still_round_trips_and_stays_v1` actually fails if that behaviour is
reverted.

### 4 — AC-8's scope, and what the call graph says it should be

The spec asked for eight verbs. The orchestrator narrowed it to six on diff evidence: in
`optimize.rs` **every hunk is inside `run_apply`**, so `web`, `optimize` and `responsive` have no
changed function on their path; in `ops.rs` the hunks are in `output_format_for` and
`run_pixel_op`, which resize/thumbnail/watermark/convert/auto-orient **do** share.

**Confirm that reading against the actual diff**, then check the record states its corpus boundary
rather than implying a whole-surface sweep — SPEC-126's verify had to add exactly that to DEC-098.
And confirm the positive control is real: a case known to differ, shown differing.

### 5 — The wasm rung is real code with NO required-matrix coverage

Call 3 made `out_format` win over `recipe.format` in `wasm::transform`. `tests/wasm_roundtrip.rs`
carries 6 tests — but **`just wasm-test` runs in no CI job**, so those 16 green checks say nothing
about them. The build flagged this honestly; your job is to (a) run them yourself, and (b) rule
whether shipping a behaviour change whose only guard is un-run is acceptable here, or whether it
needs the STAGE-053 CI leg first.

Also confirm `src/recipe/` still compiles for wasm32 — `just wasm-check`. It is an **engine**
module (DEC-064): no `std::fs`, no `clap`.

### 6 — Precedence, both directions, and the carve-out

`--format` > `-o` ext > `recipe.format` > preserve source; `-q` > `recipe.quality`. Drive the
overrides **in both directions**, not just the happy one. Then the terminal-`optimize` carve-out
(AC-6): a bundled recipe with an explicit `format` must skip the auto-decision, and **the same
recipe without one must still auto-decide with output byte-identical to `main`** — that second
half is what stops this from having quietly changed the flagship `web` path.

---

## Also check

- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref**, or a
  clean checkout reports "No changed files in scope" and exits 0 on a green that cannot go red.
  ⚠ Note it **cannot see DEC-015** (inline `affected_scope`, filed PROJ-013 STAGE-047), and
  DEC-015 governs `docs/api-contract.md`. Check that file by reading, not by trusting the audit.
- **`docs/api-contract.md` and `docs/data-model.md`** — every sentence added is a testable claim
  about a shipped binary. Drive the ones that are. Documentation has no green.
- **DEC-099's confidence is honest** (AGENTS §17), and its Validation section states what was and
  was not swept.
- **Every file the diff touches is listed** in Build Completion, from `git diff --name-only`.
- **AC-9's matrix** — default, `--no-default-features`, `--features webp-lossy`, fresh
  `CARGO_TARGET_DIR` each, sequential; clippy and `fmt --check` on each.
- **No `unwrap`/`expect`/`panic!` added on recoverable paths** in `src/` (constraint).

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge. Do not bump the version.
- **Budget ~200 exchanges.** This spec's verification surface is genuinely larger than its code —
  the orchestrator under-sized it as M and said so. Blowing the budget openly beats rushing a
  control.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal — redirect stdout,
  do not "fix" it. A piped command reports the **pipe's** exit code — redirect and read `$?`.
- zsh does **not** word-split unquoted parameters — use `while IFS= read -r`. Use `/usr/bin/grep`.
  macOS has no `timeout(1)`.

## When you finish — and this time the cost block is inlined, not referenced

The build prompt referenced `cost-snippet.md` instead of inlining it, and the cost was
reconstructed post-hoc as a result. That is the orchestrator's defect, corrected here:

```
Measure your own cost — do not estimate it, and do not leave it for someone else.

Your session transcript records per-message `usage`:
  ~/.claude/projects/<cwd-slug>/<session-id>.jsonl
Each line with `.message.usage` contributes input_tokens, output_tokens,
cache_creation_input_tokens and cache_read_input_tokens. Take duration from the
first and last `timestamp`, and the model from `.message.model`. The session id is
the last path component of your scratchpad directory.

⚠ Given item 1, ALSO state whether those fields are per-call or cumulative, and
show the check you ran. That is part of this cycle's deliverable.

If the transcript is genuinely unreadable, say so and write tokens_total: null —
a stated gap is fine, a made-up number is not.
```

Price **per component** at the anchors `.message.model` reports (DEC-083): Opus $5/$25 per MTok,
cache_creation ×1.25 input, cache_read ×0.10 input. **Never a flat rate.**

End with **✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED**, leading with **item 1's ruling** — whether
this project's cost figures are sound, overstated, or unknown. That question is bigger than this
spec, and this is the cycle that can answer it.
