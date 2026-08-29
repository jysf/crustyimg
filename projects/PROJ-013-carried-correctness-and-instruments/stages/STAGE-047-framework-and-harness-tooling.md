---
stage:
  id: STAGE-047
  status: proposed
  priority: medium
  target_complete: null

project:
  id: PROJ-013
repo:
  id: crustyimg

created_at: 2026-08-16
shipped_at: null

value_contribution:
  advances: >
    Not PROJ-010's product thesis — this stage advances the ability to DELIVER it. Every
    item here is a tool that reported success while being wrong, or a convention that
    exists only as prose. They cost real sessions in this project.
  delivers:
    - Framework tooling that fails loudly instead of reporting green
    - Conventions that live in AGENTS.md rather than in one person's memory of a stage close
    - Findings routed upstream to the template author, where they fix every repo on it
  explicitly_does_not:
    - Change crustyimg's behaviour. Nothing here ships to a user of the CLI.
    - Fix the release process itself — that is STAGE-042.
---

# STAGE-047: framework and harness tooling

## What This Stage Is

The stage for defects in **the harness that builds crustyimg**, as opposed to crustyimg
itself. Split out of STAGE-042 on 2026-08-16, when that stage had reached 18 items with
two unrelated focuses and its one genuinely blocking item was sitting in a list next to a
markdown-formatting mismatch.

The unifying property is sharp, and it is the same one `docs/framework-feedback/tooling-that-fails-silent.md`
already named: **these tools report success while being wrong.** None crashes. None warns.

| item | what it reported | what was true |
|---|---|---|
| `next_id` | a fresh spec/DEC id | the id was already taken on an unmerged branch |
| `just validate` | "all front-matter parses" | untracked files were never read |
| `just backlog` | a stage's item count | promoted specs double-counted, because the template documents a format the matcher rejects |
| `decisions-audit` | ~1,200 overlap warnings | one was real; it surfaced by luck |
| a squash merge | merged | a push landing near merge time was dropped silently |
| a wall-clock prompt budget | within budget | the cycle cost 4.3× the wave's cheapest |

## Why Now

**Because they are no longer hypothetical.** Every row above cost this project something
measurable in the last week: `next_id` minted SPEC-116 twice *and* DEC-092 twice; the
squash-strand propagated a known-wrong dependency verdict into two authored documents; the
`decisions-audit` noise nearly buried a real finding; the wall-clock budget was silent
through a $51 cycle.

**And they are not ours alone.** Six of the earlier findings trace to `Initial commit`, so
they ship to every repo on this template. `docs/framework-feedback/` already holds the
write-ups; what it has never had is a **schedulable** home, which is exactly the
"a document is not a backlog unless tooling reads it" failure this stage is full of.

## Success Criteria

- No tool in this list can report success while being wrong — each either fails loudly or
  states its scope.
- The conventions proven in PROJ-010 live in `AGENTS.md` or the templates, not in a stage
  close nobody re-reads.
- Findings that belong to the template author are routed there, with their measurements.

## Scope

### In scope
- `scripts/` and `justfile` recipes that mis-report: `next_id`, `validate`, the shared
  backlog counter, `decisions-audit`.
- Prompt and stage **templates**, where they document a format the tooling rejects.
- Git-workflow hazards the repo cannot currently detect (the squash-strand).
- Process conventions measured in PROJ-010 that are still unwritten.

### Explicitly out of scope
- **crustyimg's behaviour.** Nothing here reaches a CLI user.
- **The release process** — `RELEASING.md`, CI legs, the release-lag signal: STAGE-042.
- Rewriting the framework. These are repairs, not a redesign.

## Spec Backlog

- [ ] (not yet written) — [S] ⚡ **`scripts/decisions-audit.sh --changed` is blind to any
  decision whose `affected_scope` uses the INLINE-ARRAY form — and `decisions/_template.md`
  teaches exactly that form.** `get_affected_scope`'s awk
  (`scripts/decisions-audit.sh:77-91`) sets its collect flag on `/^affected_scope:/` and then
  **`next`s past that same line**, so anything written on it is discarded; only `^\s*-\s`
  block-list items are gathered. Two LIVE decisions are written inline and therefore yield
  **zero** globs on every run, forever: **DEC-015** (`["src/cli/**", "docs/api-contract.md"]`)
  and **DEC-043** (`superseded_by: null`, so still binding).

  **Measured with a positive control 2026-08-23** — the same awk yields 8 globs for DEC-087 and
  2 for DEC-006, both block form. The parser is not broken in general; the failure is specific
  to the inline form, which is why it has never announced itself.

  ⚠ **Root cause is the template.** `decisions/_template.md:33` reads
  `affected_scope: []   # e.g. ["src/lib/log.ts", "src/**/*.ts"]` — its worked example is the
  one form the parser cannot read, so every DEC authored from it inherits the bug. AGENTS §15
  does not specify a form either. Fixing the script without fixing the template just resets the
  clock.

  **Why this is more than tidiness:** SPEC-126's verify ran
  `./scripts/decisions-audit.sh --changed 9b4fb80`, got 17 DECs flagged and **no drift** — while
  **DEC-015, the decision that spec implements, was structurally invisible to the run.**
  DEC-015's scope names `docs/api-contract.md`, and SPEC-126 had in fact failed to update it
  (caught by a human reading the diff, not by the audit). **The instrument ran green over the
  one decision that mattered.** That is the same "a green that cannot go red" failure AGENTS §15
  already documents for a missing base ref — in a second, undocumented place.

  Fix is small (accept both YAML forms, or normalise the template and migrate the two records)
  but must ship with a test asserting a known inline-form DEC yields its globs — otherwise the
  next regression is equally silent.

- [ ] (not yet written) — [S] **A complexity rating is set at framing and never revisited when a
  design call widens scope.** SPEC-119 was framed `[S]`, re-estimated to `[M]` when the design
  sweep found 3 affected formats instead of 1 — and then **not re-estimated again** when Call 4
  was revised post-framing to require a second lint rule. It shipped at **$51.24 / 143.5M
  tokens**, 4.3× the cheapest build in the same wave and the most expensive of four. The rule
  that would have caught it is one line: **if a design call is revised after framing, re-check
  the complexity in the same edit.**

- [ ] (not yet written) — [S] **Prompt budgets are written in wall-clock and cost does not track
  wall-clock.** Measured across four builds in one wave: cost scales with the **square of message
  count**, because every message re-reads the accumulated context (cache reads were 97–99% of
  tokens in all four).

  | spec | msgs | total tokens | msgs² | minutes | $ |
  |---|---|---|---|---|---|
  | SPEC-120 | 0.41× | 0.40× | 0.17× | 0.21× | 8.69 |
  | SPEC-116 | 1.00× | 1.00× | 1.00× | 1.00× | 11.91 |
  | SPEC-117 | 1.57× | 2.18× | 2.47× | 0.60× | 23.06 |
  | SPEC-119 | 2.29× | **4.99×** | 5.23× | **0.59×** | 51.24 |

  **Minutes anti-correlate**: SPEC-116 ran 104 minutes and cost $11.91; SPEC-119 ran 61 and cost
  $51.24. Every prompt in this wave carried a clock budget and not one of them fired. Replace with
  a message-count checkpoint (~250 exchanges) in `cost-snippet.md` or the prompt templates.

- [ ] (not yet written) — [S] **A squash merge can strand a push that lands near merge time, and
  nothing warns.** Happened on PR #170, 2026-08-15: commit `7ca85a2` pushed successfully to the
  branch, GitHub squashed from the head it had captured (`e5aca27`), and the correction vanished
  with **no conflict and no warning**. It then propagated — STAGE-046 and SPEC-119 were authored
  against the resulting `main` and both inherited a dependency verdict that was already known to
  be wrong. Recovered by #172 and `fb61a97`.

  **The detector is cheap and nothing runs it:** for each merged PR compare its `headRefOid`
  (the head GitHub actually merged) against the branch's current tip; where they differ, check
  whether the tail commits' own content reached `main`. A full sweep of every merged PR takes
  seconds via `gh pr list --json number,headRefName,headRefOid`.

  ⚠ **Two wrong methods to skip** — both were tried first and both produce confident nonsense:
  "commits ahead of `main`" flags *every* squash-merged branch (SHAs are rewritten), and "files
  differing between `main` and the branch tip" reports the branch's whole drift — ~80 files for a
  commit that touched one. Scope to the tail commits' files, then assert on content.

  **Swept 2026-08-16: exactly one incident (#170, recovered) across all merged PRs.** #150 flags
  on head-vs-tip but its tail touches no files — an amend, clean.

- [x] **LANDED in AGENTS §15, `da120c1`** — [S] **A negative control's evidence is the BEHAVIOURAL FLIP, not a binary
  hash.** Measured by SPEC-117's verify, 2026-08-16: across a four-state control sequence
  (baseline → revert A → revert B → restore) the **restore produced a different binary from the
  byte-identical baseline source** (`9d4a2871…` vs `097e9526…`). So in this repo's debug profile a
  changed hash proves only that a relink happened — exactly what a "Compiling crustyimg" line
  already proves, and no more strongly. **Neither shows the edit reached the artifact; the test
  going RED does**, because the reverted code is observed executing. SPEC-116's verify used the
  hash instrument and got away with it. Fold into AGENTS §15 alongside STAGE-043's proposed
  "one revert per independent condition" update — they are the same lesson from two directions.
  *Scope caveat from the measurement: one incremental target dir, macOS, debug; a fresh dir and
  the release profile were not tested.*

- [ ] (not yet written) — [M] **`decisions-audit`'s overlap check drowns its own signal.** Measured
  2026-08-15: a full run emits **~1,200 `both govern overlapping scope` warnings** (1,187 counted
  in one complete-enough run, in 2,312 lines of output) and takes **over two minutes**. Every one
  of them is the same shape, and the top pairs are trivially-shared globs:

  | count | glob pair |
  |---|---|
  | 401 | `Cargo.toml ~ Cargo.toml` |
  | 237 | `src/cli/mod.rs ~ src/cli/mod.rs` |
  | 103 | `src/cli/** ~ src/cli/mod.rs` |

  The check pairs every DEC against every other and warns on any prefix overlap, so it is O(n²)
  over decisions sharing a common file — and **every dependency decision "overlaps" every other
  dependency decision**, which is true and useless. The consequence is not cosmetic: the one
  semantically real overlap this week (DEC-088 / DEC-091) sat near line 1,150 of 2,312 and was
  **only noticed because a truncated `tail` happened to land on it**. An instrument that surfaces
  its one real finding by luck is not surfacing it. Options: exclude manifest/module-root globs
  from the pairing, warn only on globs shared by ≤N decisions, or make it a distinct opt-in
  subcommand instead of part of the default lint. **This is a STAGE-042 instrument, so its own
  signal-to-noise is in scope.**

- [ ] (not yet written) — [S] **DEC-091 refines DEC-088 and neither record says so.** The audit
  flags them as governing overlapping scope (`docs/territory.md`) and asks whether they
  contradict. **Checked 2026-08-15: they do not.** DEC-091's own text opens *"DEC-088's
  generalization fence is **kept**, restated at the level it actually operates on
  (parameters)"* — it refines DEC-088's fence and adds a second one, while DEC-088's other half
  (the three tiers of external integration) is untouched. So neither supersedes the other, and
  `supersedes` / `superseded_by` are both `null` on both — correctly, because **the schema has no
  way to express "refines"**. A reader arriving at DEC-088 alone gets the older, looser statement
  of the fence with no pointer to the refinement. Fix is either a prose cross-reference in both
  files (cheap, no schema change) or a new front-matter relation field (a framework change needing
  its own DEC). **No adjudication needed — the relationship is already settled in the text.**

- [ ] (not yet written) — [S] **The stage template documents a bullet format `just backlog`
  does not recognise.** `_lib.sh:212` treats a bullet as *promoted* only when it matches
  `**SPEC-NNN**` — bold. `projects/_templates/stage.md` documents
  `Format: `- [status] SPEC-ID (cycle) — one-line summary`` — no bold. A stage written to its
  own template therefore double-counts every promoted spec as backlog. Hit live on STAGE-046,
  2026-08-15. Fix the template, the matcher, or both; whichever, they must agree.

- [ ] (chore) — **`just validate` silently skips untracked files.**
  `scripts/validate-frontmatter.sh:31` enumerates via `git ls-files`, so a **newly created** spec
  or stage — exactly the file most likely to have malformed front-matter — is invisible to the
  validator until it is staged. It reports success with an unchanged block count, which reads as
  a pass. Found while writing this very stage: `just validate` said "250 blocks ✓" with two new
  stage files on disk, and only said 252 after `git add`. Fix: warn when an untracked
  `*.md`/`*.yaml` sits under `projects/` or `decisions/`, or enumerate the working tree and note
  which files are untracked. **The block count should be part of the output people read**, since
  the count not moving is the only tell. [[a-harness-that-exercises-nothing-reports-green]]
  Complexity **S**.

- [ ] (⏸ **PARKED 2026-08-18 — noted, not scheduled.** Maintainer's call: this is harness tooling
  that generalizes past this repo, so it should land at the template level rather than as crustyimg
  work. Recorded here only so it is not lost.) — [S] **A per-spec ledger: size and cost broken down
  by cycle, with description and outcome.**
  **What already exists:** `just specs-by-stage` gives `spec · status · ship date · complexity ·
  cost (usd · tokens) · description` with per-stage subtotals — four of the six columns wanted.
  **Gap 1 — the per-cycle split is cheap.** `cost.sessions` already carries `cycle`,
  `estimated_usd`, `tokens_total`, `agent` and `duration_minutes` per entry, and
  `_lib.sh:413/434` (`sum_cost_tokens_for_spec`, `sum_cost_usd_for_spec`) already walks that block —
  they just collapse it to one total. A ~60-line reader (front-matter via `pyyaml`, already
  available) produces the table. **Prototyped 2026-08-18 and it works**; the prototype lived in a
  session scratchpad and is gone, but the above is enough to rebuild it in minutes.
  **Gap 2 — outcome is NOT structured, and that is the real work.** The signal is prose in
  `## Build Completion` (*"yes"*, *"yes — AC-1 through AC-11"*, *"Mostly yes — 12 of 13"*), and the
  verify verdict (✅/⚠/❌) is not in front-matter at all — it lives only in timeline prose. **A
  scraper over that would be confidently wrong on exactly the interesting rows**, which is the
  failure class this whole stage exists to eliminate. The fix is a small `outcome:` front-matter
  block written at ship (verdict, ACs met, punch-list count, deviations), backfillable for the 13
  archived specs since the prose is there to read once.
  ⚠ **Two things the prototype surfaced that stand on their own, independent of whether the tool
  is ever built:**
  (a) **verify costs 37% of build** across PROJ-010 — $219.19 against $584.84 — a ratio nobody had
  looked at;
  (b) **complexity barely predicts cost.** `[S]` ran $15.58–$76.16 and `[M]` ran $37.59–$141.99,
  heavily overlapping — SPEC-112 `[S]` cost more than five of the eight `[M]`s. That is direct
  evidence for the complexity-rating item above, and it corroborates SPEC-123's sizing lesson: a
  measurement spec's cost is set by whether its premise survives, which is unknowable at framing.
  ⚠ **Any such report reads working-tree state**, so a spec whose cost sits on an unmerged branch
  reads as $0.00 (SPEC-123 did while PR #179 was open). Whatever ships must say so rather than let
  a reader mistake it for free.

- [ ] (not yet framed, **added 2026-08-15**) — **`next_id` mints duplicate spec IDs.** It scans
  only the WORKING TREE, so any spec living on an unmerged branch is invisible to it. Driven
  live: with SPEC-116 and SPEC-117 sitting in PR #166, `just new-spec` on a branch off `main`
  minted **SPEC-116 again**. `next_id`'s own comment warns about a different scoping failure
  (passing a single project dir restarts at 001), so this one was unanticipated rather than
  accepted. Same family as the counter bugs fixed 2026-08-15: it fails by producing a plausible
  wrong answer, silently. Fix is to consult git refs, or at minimum warn when a higher ID exists
  on another ref. Complexity **S**.

**Count:** 1 closed / **9 pending** — re-derived by grep 2026-08-23.

**Count:** 1 landed / 9 pending (1 of them ⏸ parked — the per-spec cost ledger, template-level)

## Design Notes

- **Several of these are one spec, not eight.** `next_id`, `just validate` and the
  backlog counter are all "a script that reads the wrong set of files"; they likely share
  a fix and a test shape. Group before framing.
- **Route upstream as you go.** `docs/framework-feedback/tooling-that-fails-silent.md` is
  the existing artifact for the template author — extend it rather than starting a new one.
- **Each fix needs a negative control**, per AGENTS §15: prove the tool now fails on the
  input that used to pass silently. That is the whole point of the stage, and it would be
  ironic to close it on unproven claims.

## Dependencies

### Depends on
- Nothing. Every item reproduces today.

### Enables
- Reliable parallelism. `next_id`'s blindness to unmerged branches is the single reason
  this project must run specs serially; fixing it removes a standing constraint that has
  already cost two duplicate-id incidents.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
