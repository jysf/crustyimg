# SPEC-110 — VERIFY prompt

Cycle: **verify**. Fresh session, **Opus**, your own git worktree. You are not the builder and
not the architect.

Output one of: **✅ APPROVED** / **⚠ PUNCH LIST** / **❌ REJECTED**.

Under review: **PR #133**, branch `feat/spec-110-orientation`. All 12 CI checks pass, **including
all three OS legs** — the orchestrator confirmed that directly, so you do not need to re-derive
it. The build reports all 11 acceptance criteria met.

## Re-derive, do not inherit

Every number in the build report is a claim. Drive the verbs yourself on your own builds of the
**branch** and of **`main`**. [[a-number-from-an-unproven-path-is-not-a-measurement]]

**Word your own conclusions as "confirm or refute", not "confirm."** The SPEC-107 verify prompt
conceded a contested point to the builder in advance and was wrong to; the design cycle's matrix
turned out to be right. Do not inherit that habit in either direction.

## Item 1 — an unmet criterion, already confirmed by driving

**`watermark` does not bake orientation on this branch.** The orchestrator built the branch and
drove it:

| verb (branch build) | got | expected | |
|---|---|---|---|
| `convert --format png` | 800×1200 | 800×1200 | OK |
| `resize --max 600` | 400×600 | 400×600 | OK |
| `thumbnail --size 300` | 200×300 | 200×300 | OK |
| **`watermark --text hi`** | **1200×800** | **800×1200** | **NOT BAKED** |

`run_watermark` (`src/cli/ops.rs:~1085`) builds `Pipeline::new().push(Box::new(op))` and hands it
to `run_pixel_op`. `run_pixel_op` takes the pipeline as a **parameter** and is not itself
prefixed, so watermark is the one pixel-lane construction site the sweep missed.

**This is AC-7's exact purpose.** That criterion asks for the prefix to be applied via a shared
helper *and* for a mechanical sweep citing "the grep showing every pixel-lane pipeline
construction site, with its scope stated as a claim." A `grep -n 'Pipeline::new()' src/cli/`
finds `run_watermark` immediately. The spec's Goal is that **no shipped verb can hand back a
sideways image**; on this branch one still can.

**Be fair to the build:** it did flag watermark, in Build Completion, as "never driven to check
if it has the same latent bug," and filed it under `one-spec-per-pr` rather than hiding it. Your
call is whether filing was correct. Weigh it against the fact that the fix is one line at the
same kind of site as the six that were fixed, and that shipping this spec while one verb stays
wrong recreates the exact pattern the spec exists to end.
[[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]

**Also on the architect:** the design's measured table omitted `watermark` entirely. That is my
error, and it is *why* AC-7 asked for a mechanical sweep rather than trusting the table — so the
sweep missing it compounds rather than excuses. Check whether **any other** pixel-lane
construction site was missed; cite your own grep and state its scope as a claim.
[[mechanical-sweeps-need-a-mechanical-check]]

## Item 2 — the double-rotation trap (AC-2)

The design named this as the obvious failure mode: `web`/`optimize` already push `orient` via
`optimize_pipeline()`, so a shared prefix applied *on top* would bake twice and return a 90° case
180° off. Confirm on a **non-square** fixture that `web`, `optimize`, `auto-orient` and
`edit --auto-orient` all still produce 800×1200 from the 1200×800 Orientation=6 source — and that
`auto_orient_prefix()` is not being applied in addition to an existing push anywhere. A square
fixture cannot see this (AC-4); check the committed fixtures are non-square.

## Item 3 — AC-3's safety claim, asserted on bytes

The whole argument that this change is safe for the overwhelming majority of inputs is that
Orientation 1 and no-EXIF inputs are **byte-identical** to before. Confirm the test asserts on
output **bytes**, not dimensions, and drive it yourself on both branch and `main`. If it only
compares dimensions, the safety claim is unproven.

## Item 4 — the test-count reconciliation

Branch: **lean 804 / default 823 / webp-lossy 830 passed, 0 failed**; `just wasm-test` 30/30.
The build reports the prompt's reference numbers were "stale by 2 in every leg" — **that is the
architect's error and it is real**: the prompt quoted SPEC-107's *totals* (797/816/823, which
included 2 ignored) as if they were passed counts. Main's passed counts are 795/814/821, so
+9 matches the 9 tests in `tests/orientation.rs` exactly.

**Re-measure both sides yourself anyway** and confirm the arithmetic closes. Nine tests for
eleven acceptance criteria is worth a look — check AC-5 (all eight orientation values) and AC-6
(`edit --auto-orient` still exits 0) are genuinely covered and not folded away.
[[verify-test-existence-not-just-gate-count]]

## Item 5 — AC-9, the record reconciliation

DEC-003 asserted *"Orientation/ICC survive transforms"* and wrote its success test as *"Right if:
a resize preserves orientation…"*. Both were false. Confirm the amendment is **dated, reasoned,
and accurate** — that it says the code now bakes rather than preserves — and that
`AGENTS.md:448`'s glossary line matches. Read the new **DEC-086** as text: does it carry the
measured table and both rejected alternatives, so neither gets re-proposed?
[[documentation-has-no-green]] Also confirm `docs/api-contract.md` no longer calls `convert` a
"pure re-encode … no pixel transform", and that `docs/cli-reference.md` describes
`edit --auto-orient` as now-default.

## Item 6 — the second filed finding

The build also filed: **`edit --save-recipe` does not capture the CLI-level bake as a recipe
step.** That means a recipe round-tripped out of `edit` no longer reproduces what `edit` did — a
recipe-fidelity gap, not just a missing feature. Judge whether that is correctly out of scope or
whether it makes a shipped surface self-inconsistent. Do not fix it; decide and record.

## Also

- Re-run **AC-10's negative control** rather than reading it: revert the prefix on one verb,
  confirm RED, restore. **Reverting source does not rebuild the binary** — prove the revert
  reached the artifact. The build reports it verified this via changed binary MD5; confirm that
  method actually discriminates. [[reverting-source-does-not-rebuild-the-binary]]
- Run your matrix through **`rtk proxy` from the first leg**. The build hit yet another rtk
  corruption — a compressed summary reporting "622/404 passed" for 12 CI checks — and caught it
  only because the numbers were implausible. A plausible-looking corruption would not have been
  caught. [[rtk-can-silently-corrupt-grep-counts]]
- **Cost is already reconciled** by the orchestrator: 61,879,452 tokens and $29.58 at Sonnet
  anchors both reproduce exactly, 96.08% cache reads, `agent` matches the pinned `implementer`.
  Flag only if the spec's `cost.sessions` entry disagrees with the readout.
- `just decisions-audit --changed`, `just validate`, `just cost-audit`.

## Guardrails

Own worktree — `../crustyimg-spec110` is the build's and two other trees are still checked out.
`git commit -s`. Never `git reset --hard`. Cross-check anything load-bearing with `/usr/bin/git`
or `python3` plus a positive control. macOS has no `timeout(1)`.

**Do not merge the PR.**

## When you finish

Append your verify cost session to `cost.sessions` (see
`projects/_templates/prompts/cost-snippet.md` — per component, at the anchors of the model that
**actually ran**, read from `.message.model` in your own transcript). Update the timeline's
`verify` line. Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not check as clearly as what you did.**
