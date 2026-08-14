---
source: "crustyimg (self-harvest) — PROJ-001 … PROJ-010, 0.7.0 shipped"
captured_at: 2026-08-11
captured_by: claude
status: open
---

# What the crustyimg corpus shows about the spec-driven process

A read of the whole repo as evidence rather than as scaffolding: 498 commits,
107 specs, 85 decisions, 43 active days. Git is the primary source; every
`just`/doc claim below was checked against it, and the disagreements are
recorded as findings.

**This entry is also overdue harvest material.** `docs/harvests/instances.md`
records crustyimg as **last reviewed 2026-07-06**. Everything of interest —
the lesson taxonomy (2026-07-16 → 07-25), all five PROJ-010 defects, the
0.7.0 cut — happened after that date and has never reached the template.

---

## 0. Method, and two corrections to my own measurements

Two of my own numbers were wrong before I caught them, both by the repo's own
lessons:

- A regex parse of the cost blocks gave **$1,927**; a real YAML parse gave
  **$966.06**. The regex double-counted `estimated_usd` occurring inside `note:`
  prose. ([[a-number-from-an-unproven-path-is-not-a-measurement]])
- Bare `wc -l` inside command substitution silently returned empty, producing a
  table of zeroes for `AGENTS.md` growth, and `rtk` mangled `$commit:AGENTS.md`
  into `<sha>GENTS.md`. Every count below was re-run with `/usr/bin/` absolute
  paths against a positive and negative control.
  ([[rtk-can-silently-corrupt-grep-counts]])

**Snapshot point:** HEAD moved during the analysis (497 → 498 commits; `19f5ddd`
framing STAGE-044 landed mid-read). Figures are as of that snapshot.

### Census, verified

| metric | value |
|---|---|
| commits / active days / calendar span | 498 / 43 / 59 days |
| unique specs | 107 (gaps at 054–055, 059, 092, 106) |
| decisions | 85 (86 files − 1 template) |
| Rust | 46,363 (28,230 `src/` + 17,750 `tests/`) |
| process markdown | 85,446 (`projects/` 71,480 + `decisions/` 13,455 + `guidance/` 511) |
| all tracked markdown | 104,505 |

**The ~3.5:1 process-to-code ratio does not survive checking.** It requires
counting *all* markdown — including README, CHANGELOG, BENCHMARKS and `docs/`,
which are shipped product deliverables — against `src/` alone, while excluding
17,750 lines of tests you also authored. Process-only against all authored Rust
is **1.84 : 1**. Still substantial. Not 3.5.

---

## 1. The finding that reframes the rest: slots vs recall

Two subsystems, same repo, same author, same 59 days:

| subsystem | reach | how it is invoked |
|---|---|---|
| **Decision log** | **105 of 107 specs** cite ≥1 DEC (mean 4.25/spec); **75 of 85** DECs referenced; 12 orphans | `references.decisions:` — a structured front-matter field the template ships |
| **Lesson system** | **35 of 107 specs** cite any lesson; **40 of 78** lesson files ever cited; **38 never** | prose in the body, written from recall |

The difference is not effort or discipline. **One is a slot; the other is
memory.** Everything below is downstream of this.

Three independent confirmations of the same mechanism in this repo:

1. **The patch lane was invented here, and died here.** crustyimg's own DEC-043
   established it 2026-07-04; `PATCH-001/002/003` shipped 07-04→07-05. The
   template adopted it as its DEC-003 on 2026-06-27, citing crustyimg's finding
   that *"the two elements that actually bought quality were the DEC log and the
   independent verify, not the ceremony of the four named cycles."*
   **crustyimg then never used it again across specs 044–112**, and `AGENTS.md`
   contains **zero** occurrences of `PATCH-`, "patch lane", or "spike". The lane
   that was never written into the governing document fell out of use in the repo
   that invented it.
2. **`guidance/signals.yaml` already exists in the template and never reached
   crustyimg.** The template ships a typed signals ledger with a codification bar
   (`N=3 same-outcome` / `N=2 paired-opposing`), a status lifecycle
   (`open → watch → codified | dropped`), and a `disposition_at` forcing function.
   crustyimg's `guidance/` has `constraints`, `license-watchlist`, `questions`,
   `recommended-tools`. **No signals.yaml.** The lesson system I was going to
   design is already designed — it just has no delivery mechanism.
3. **There is no template → instance propagation channel.** `docs/harvests/` and
   `feedback/` carry insight *from* instances *to* the template. Nothing carries
   improvements back. The only sync script is `handback-sync.sh`, which is about
   delegated cost handback.

---

## 2. The Good — what demonstrably worked

**G1. The decision log is the crown jewel.** 105/107 specs reference it; DEC-007
cited 39 times, DEC-004 26, DEC-015 24. It is the one artifact that made later
work cheaper — specs inherit settled constraints instead of relitigating them.
If one thing survives, this is it.

**G2. Independent verify earns its cost, and the evidence is specific.**
11 of 107 specs were sent back by verify (punch list or reject) — and **4 of the
last 6**. The proof case is SPEC-110: the build ran the mechanical sweep, *found*
`watermark`, routed it back through the design's measured table and filed it out
of scope. Verify drove `watermark --text hi`, got 1200×800 where 800×1200 was
correct, and refuted both AC-7 and the spec's Goal. Fixed 08-06, merged 08-06.
**I checked every `main` commit touching DEC-086: it never reached `main` in the
false state.** The memory file's claim that it "shipped false" is overstated.

**G3. "Drive the surface, don't reason about it."** This is the strongest *method*
in the corpus and it never became a rule. Every late catch came from executing
the binary against an enumerated surface — SPEC-110's verify classified 17
subcommands from `--help` in minutes; SPEC-107 drove a hostile corpus and found a
truncated JPEG exiting 0 with empty stderr on the flagship path. No catch came
from reading the call graph.

**G4. Model instrumentation is real data.** `agents.architect` / `implementer`
(101 Opus architect; 54 Opus / 36 Sonnet implementer) is what made the
Sonnet-vs-Opus comparison possible. Most process metadata is inert; this is not.

**G5. The repo diagnoses itself accurately.** I independently checked all three
factual claims in STAGE-042: `bundled::names()` is called in exactly one non-test
place (`src/cli/common.rs:171`); `build_runs_each_bundled_recipe_by_name`
iterates a hardcoded `["web","gallery","product"]` rather than `names()`; and no
CI leg runs `just wasm-test` (`pages.yml` runs only `demo-build`/`demo-smoke`).
**3 of 3 held.** Late-stage self-analysis in this repo is trustworthy.

---

## 3. The Bad — real, and fixable

**B1. Lessons do not transfer. The clean disproof:**
`a-stale-incremental-build-is-a-false-green` was first cited **2026-07-25**.
SPEC-109 ran **07-26**, **cited that exact lesson in its own body** (one of six
lesson citations — second-densest spec in the repo), and its verify then measured
a fixture table with a stale `./target/debug/crustyimg` still carrying a reverted
mutation, reporting every photo fixture as `graphic-logo`. The response was to
mint a **new** lesson (`reverting-source-does-not-rebuild-the-binary`, first
cited 07-29) rather than recognise the one already cited. Its own text concedes
it: *"Same class of trap … in reverse."*

**B2. The lessons never became rules.**
- `AGENTS.md`: **516 → 563 lines** over 59 days. Zero occurrences of "negative
  control", "mechanical", "clean build", "incremental", "grep", "claim
  inventory", "blast radius". Zero `[[wiki-links]]`.
- `guidance/constraints.yaml`: **12 → 16 constraints, frozen 2026-06-17** — a
  month *before* the lesson taxonomy existed. None of the 16 concerns evidence
  quality. The nearest, `test-before-implementation` and `every-public-fn-tested`,
  are exactly what SPEC-093 disproves ("the tests ran; they could not fail").

**B3. Flat 4.0 sessions per spec, with zero variance.**

| SPEC range | 000–019 | 020–039 | 040–059 | 060–079 | 080–099 | 100–119 |
|---|---|---|---|---|---|---|
| sessions/spec | 4.00 | 4.00 | 4.00 | 4.00 | 4.21 | 3.75 |

`cycle` is `ship` in all 107 committed specs and `blocked: true` has **never
existed in 498 commits** — so there is no skipped-step cohort and therefore no
internal control group. `documentation-has-no-green` diagnosed the overhead
("~2 cycles were orchestrator overhead: dispatching full verify sessions … to
check three sentences") and nothing changed after.

**B4. Dead fields in the schema.** `blocked` (0 `true` in 498 commits);
`insight.type` (4-value enum, 85/85 use `decision`); `session_id` (1 of 85);
`activity` (absent from all 107 — proposed, never built).

**B5. `affected_scope` is a dead safety net — the worst of the dead fields.**
Populated in **1 of 85** decisions, while the template documents it as driving
`just decisions-audit` to flag decisions whose governed paths a pending change
touches. **That is precisely the mechanism that would have caught DEC-003
drifting away from the code** — the drift SPEC-110 exists to repair.

**B6. Enforcement is narrower than the constraint reads.**
`cost-captured-per-cycle` says *"Every shipped spec must record a real
`tokens_total`."* `cost-audit.sh` calls `find_all_specs "$project_dir"` on **the
active project only** (23 files) minus a 13-spec grandfather list
(`SPEC-001…013`). **84 of 107 specs are permanently outside its scope** — once a
project ships, its cost data is never audited again. Hence `just status`
reporting "missing cost data: (none)" while 26 build/verify sessions lack tokens.

**B7. `just status` vs the brief.** Status lists 11 stages for PROJ-010; the
brief's own `**Count:**` line says 4 shipped / 0 active / 5 pending + 1 on hold
= 10. STAGE-044 landed and the count line is one behind.

---

## 4. The Ugly — structural; will not yield to trying harder

**U1. You cannot answer "what did this cost" from your own records.**
SPEC-014's build logged **130,653** tokens. SPEC-107's build logged
**95,596,984**. That is not 732× harder work — the late figures are ~96%
`cache_read`, the early ones do not count cache at all, and `tokens_breakdown`
exists on **6 of 94** specs with any token data. `estimated_usd` has the same
problem across DEC-083: **$966.06 total, 43% of the 429 sessions unpriced**, and
of the 244 priced only 190 carry `recorded_at` — so **$178.56 (18%) cannot be
placed on either side of the methodology change** (pre $239.30 / post $548.20).
**There is no metric in this corpus comparable across its own length.**

**U2. `confidence` has no signal in any repo.** Across **224 decisions in 7
repos**: means 0.82–0.89, stdev 0.04–0.075, `< 0.7` fired **once** (bragfile000),
`< 0.6` **never**. In crustyimg: n=85, mean 0.854, sd 0.047, min 0.70 — and the
design rule is a strict `< 0.7`, so even the lowest decision does not trip it.
This is not a calibration problem to fix by being more honest; an agent's
self-assessed confidence in a record it just wrote is a constant.

**U3. The cross-product is nobody's job, and structurally cannot be.**
Dated from git:

- `build` shipped **07-08**. `wasm::transform` shipped **07-12**. Bundled
  recipes shipped **07-15** (SPEC-085) — and SPEC-085's acceptance criteria
  enumerate exactly two entry points, `web` and `apply --recipe`. The two older
  consumers were never asked about. → SPEC-111, SPEC-112.
- The pixel-lane verbs shipped **06-14** (SPEC-007). The `auto-orient`
  *operation* shipped **06-15** (SPEC-015), one day later. SPEC-015's twelve
  acceptance criteria are all about the operation and its own verb — **and the
  spec is not deficient.** Its job was "add an op." The property "every
  pixel-lane verb bakes orientation" was invented **2026-08-04**, fifty days
  later. → SPEC-110.

**The escape route is the old side of the cross product.** A spec that adds a
consumer thinks about what it consumes. Nothing asks what already consumes a
newly-added thing, because the specs that would have to ask closed before the
thing existed.

**U4. 85,446 lines of process prose produced 47 lines of governing rules.**
The prose volume is real, largely write-only, and the rule set stopped growing on
day 5 of 59.

---

## 5. Correcting the defect diagnosis

STAGE-042 claims *"four of PROJ-010's five defects escaped the same way — a cell
of a matrix nobody enumerated."* Checked against the specs, **the five are two
classes, not one**:

| class | defects | mechanism | instrument |
|---|---|---|---|
| **A — unenumerated cross product** | SPEC-110, 111, 112 | a new thing × pre-existing consumers nobody listed | exhaustiveness-asserted lists |
| **B — unrepresented input population** | SPEC-107, 108 | every fixture was well-formed / synthetic, so a region of input space had no representative | committed adversarial corpus |

SPEC-107 does **not** belong in class A. Its own finding says the truncated JPEG
succeeded *"on every verb"* because **the JPEG decoder tolerates truncation by
design** — no cell was missed; every cell was equally blind. That is
`[[fixtures-from-the-code-under-test-cannot-fail]]` and
`[[a-harness-that-exercises-nothing-reports-green]]`, the same species as
SPEC-088's bench encoding zero AVIFs.

Note also: a **fuzz gate has existed since 2026-07-11** (SPEC-069) and did not
catch it — because a fuzzer's oracle is *"did it crash"* and the defect was
*"it succeeded quietly."*

**Consequence:** STAGE-042's proposed matrix addresses class A only, on the two
axes already known. STAGE-043's newly-found defect (`optimize x.jpg -o out.jpg`
returning 2.02× larger, exit 0) is a **third axis** — pinned vs decide path —
that the proposed matrix would not catch. The diagnosis is sound; the instrument
is scoped to the last war.

---

## 6. The lesson system as slots and constraints

**Do not design a new system. Adopt `guidance/signals.yaml`, then add the three
pieces the crustyimg evidence shows it still lacks.**

The template's signals ledger already supplies the slot, the lifecycle
(`open → watch → codified | dropped`), the codification bar (`N=3 same-outcome`),
and a forcing function (`disposition_at: stage-close`). What crustyimg proves is
missing:

### Gap 1 — citing a lesson is not discharging it

SPEC-109 cited the lesson and hit it. SPEC-110's AC-7 cited it, required the
sweep, the sweep *ran*, and the finding was overruled. **A reference proves
recall; it does not prove a control ran, and it does not make the control's
output binding.** Three tiers, and only tier 3 retires the lesson:

| tier | artifact | proves |
|---|---|---|
| **1 — referenced** | `references.lessons: [LSN-004]` in front-matter | it was in scope |
| **2 — discharged** | the AC names a command **and pastes its output** | the control ran |
| **3 — mechanized** | a repo test / `constraints.yaml` rule | it cannot recur |

**Binding rule (from SPEC-110):** *the sweep's output outranks the roster it is
checked against.* Every site the check finds is fixed, or named as an exception
in a DEC. Filing is not an option.

### Gap 2 — lessons split by skin, so they proliferate instead of merging

`fixtures-from-the-code-under-test` and `harness-exercises-nothing` are mutually
detecting — apply either cure to the other's case and you catch it. The corpus
went from 0 to 7 green-signal lessons in 10 days, and SPEC-109 minted an eighth
for a case it had already cited.

Add a required closed-vocabulary field naming **where in the pipeline the signal
lied**:

```yaml
mechanism: input | path | treatment | artifact | oracle | no-check | stopping-rule
```

- `input` — the fixture could not embody the failure (SPEC-093, SPEC-107, SPEC-108)
- `path` — the branch was never reached (SPEC-088)
- `treatment` — the independent variable never moved (SPEC-093 byte order)
- `artifact` — the thing measured was not the thing changed (SPEC-105, SPEC-109)
- `oracle` — the reference shared the defect (SPEC-083)
- `no-check` — a claim stood in for a check (SPEC-089, SPEC-110)
- `stopping-rule` — no terminating condition (SPEC-083 docs)

**Constraint: a new lesson whose `mechanism` already exists must extend that
record, not create one.** This alone would have collapsed the seven to six and
prevented the SPEC-109 eighth.

### Gap 3 — nothing retires a lesson

38 of 78 memory files are never cited. Lessons accumulate and nothing prunes.

**Constraint: at `N=3`, a lesson must reach tier 3 or be dropped. It may not
remain a lesson.** This is the promotion path whose absence explains everything
in §3 — and it is the mechanism that turns `constraints.yaml` from a frozen
day-5 artifact into something that grows.

### Concretely, in the template

```yaml
# spec front-matter — mirrors references.decisions
references:
  decisions: [DEC-003]
  lessons:   [LSN-004]          # NEW slot
```

```yaml
# guidance/signals.yaml — added to the existing lesson entry shape
  mechanism: no-check           # NEW, closed vocabulary
  discharge: "rg -n 'auto_orient_prefix' src/ | wc -l"   # NEW, tier-2 command
```

Three new constraints in `constraints.yaml`:

- `lesson-cited-is-lesson-discharged` — an AC naming a lesson must carry a
  fenced command **and its output**.
- `sweep-claims-cite-the-grep` — a completion claim over a set cites the command
  that enumerated the set and its hit count, never "cleaned file X."
- `shipped-list-asserted-exhaustive` — any code-level list of shipped things
  (`bundled::names()`, pixel-lane verbs) has a test asserting completeness
  against its source of truth, proven by negative control.

That third one is the generalisation of STAGE-042 that is **not** scoped to the
last war: it applies to any list, including axes not yet discovered.

---

## 7. The sessions-per-spec short-circuit

The flat 4.0 is defensible and should stay the default — verify caught 4 of the
last 6 specs and the data supports keeping it. The problem is not cycle count; it
is that **design and ship get the same weight as the cycle that actually finds
things**, and that documentation gets the full ceremony.

**Route, do not shrink.** The lanes already exist in the template (DEC-003 patch,
DEC-012 spike). What is missing is a *declared, un-abusable* routing rule:

| lane | cycle | for |
|---|---|---|
| **spec** (default) | frame → design → build → verify → ship | anything touching `src/**` |
| **patch** | patch → verify → ship | bounded fix to shipped behaviour, no new surface |
| **doc** (new) | doc → inline-verify → ship | prose-only diff, no `src/**` change |

Two guardrails that make this safe:

1. **Mechanical, not judgment.** *Any diff touching `src/**` is the `spec` lane,
   period* — checkable in CI, so the short-circuit cannot be used to dodge verify
   on real code.
2. **Verify is never removed, only relocated.** The `doc` lane keeps verify but
   runs it inline in the main loop instead of dispatching a fresh ~$4/~12-min
   session to check three sentences — which is exactly what
   `documentation-has-no-green` prescribes, and it is the only lane where blast
   radius is *reader-falsifiable vs internal audit trail* rather than behavioural.

**The lane must be declared at frame and recorded in front-matter**
(`task.lane`), so that — unlike the 2026-07-04 patch-lane experiment — it is a
slot rather than a remembered option.

---

## 8. Priority list

Ranked by evidence strength × leverage on the next 4–8 projects.

| # | action | why now | size |
|---|---|---|---|
| **1** | **Backport `guidance/signals.yaml` into crustyimg and scaffold it into every new project**, with the three additions from §6 (`mechanism`, `discharge`, tier-3-or-drop at N=3) | The single biggest gap, and the design already exists. 35/107 → the DEC log's 105/107 is the target | M |
| **2** | **Build the template → instance propagation channel.** A `just template-check` that reports which template features (lanes, signals, scripts) the instance lacks | Explains B1, the dead patch lane, and the missing signals.yaml in one mechanism. Without it, #1 works once and rots | M |
| **3** | **`shipped-list-asserted-exhaustive` as a constraint**, plus the `PIXEL_LANE_VERBS` / `bundled::names()` tests, each proven by negative control | Class A defects (3 of 5). Generalises past STAGE-042's two known axes | M |
| **4** | **Add the `doc` lane + the `src/**` routing guardrail; declare `task.lane` at frame** | Removes the ceremony that has no defenders, without touching the flat 4 where it earns its keep | S |
| **5** | **Populate `affected_scope` or delete it** | 1/85 on the one mechanism aimed at the DEC-003-style drift that SPEC-110 exists to repair | S |
| **6** | **Stop summing cost.** Mark everything before DEC-083 as a different unit; publish per-cycle output tokens only | The totals are currently not just imprecise but incommensurable (U1) | S |
| **7** | **Widen `cost-audit` beyond the active project, or narrow the constraint's wording to match** | The constraint reads global and audits 23 of 107 | S |
| **8** | **Delete `blocked`, `insight.type`, `session_id`; retire `confidence` or replace it with something with variance** | 224 decisions across 7 repos say `confidence` measures nothing | S |
| **9** | **Harvest crustyimg into the template** — `instances.md` says last reviewed 2026-07-06 | Every finding in this document is currently un-harvested | S |

---

## 9. How to test this on the next 4–8 projects

The corpus's central weakness is that it has **no internal control group** —
`cycle` is `ship` in all 107 specs and `blocked` was never `true`, so nothing
varied. With 4–8 upcoming projects, that is fixable by design. Adopt the changes
on some and not others, and record:

| metric | how | what it answers |
|---|---|---|
| **lesson reach** | % of specs with ≥1 `references.lessons` entry | does the slot close the 35/107 → 105/107 gap? |
| **discharge rate** | % of cited lessons with tier-2 output pasted | does citing become checking? |
| **promotion rate** | lessons reaching tier 3 vs sitting at `watch` | does the N=3 rule prune, or does the ledger just grow? |
| **re-learn count** | new lessons whose `mechanism` already existed | the direct measure of §6 Gap 2 |
| **lane mix** | spec / patch / doc split, and sessions/spec per lane | does routing beat the flat 4, or was the flat 4 right? |
| **escape class** | each shipped defect tagged A (cross-product) or B (input population) | does #3 actually move class A? |

**Two pre-registered predictions**, so this is falsifiable rather than
retrospective:

1. **`references.lessons` reach will exceed 80% within one project** — because
   `references.decisions` reached 98% on the same authoring pattern. If it does
   not, the slot hypothesis is wrong and the problem is something else.
2. **Class A defects drop and class B defects do not** — #3 targets A only.
   If B also drops, something other than the instrument is responsible and the
   result should not be credited to it.

---

## 10. Limits of this analysis

**Survives n=1:** the cross-product finding (dated mechanically from git; the
mechanism is general to any change-scoped process); the slot-vs-recall gap (two
subsystems, same repo, same operator — an internal control, not a cross-project
claim); the fact that `tokens_total` and `estimated_usd` are not series.

**Strengthened by the other repos:** `confidence` having no variance — 224
decisions across 7 repos with one threshold firing once is not a small sample for
that specific claim.

**Does not survive:** any claim that the process *caused* the quality. 46k lines,
5 escaped defects, 59 days is a good outcome, but with no control group, no
skipped-step cohort, and one operator, it cannot be attributed. The defensible
version: **this analysis locates where the process stops working, and the
boundary is structural rather than a discipline failure.**

**Not done:** a full template-lineage diff across all seven instances (I sampled
front-matter rather than diffing each template version), and any read of
`zany-animal-slots`' 104 specs beyond its front-matter — it ran concurrently with
crustyimg and is the obvious second data point for §9.
