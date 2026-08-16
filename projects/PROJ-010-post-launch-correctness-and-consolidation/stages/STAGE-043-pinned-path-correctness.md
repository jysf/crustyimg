---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-043                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: critical
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-10
shipped_at: 2026-08-15

value_contribution:
  advances: >
    PROJ-010's thesis is that a shipped verb should do what its name says on an ordinary
    input. Four stages fixed that for the decide path. This stage fixes the two places
    the PINNED path still does not: `optimize` silently returns a file twice the size of
    an already-compressed source, and `build` swallows the truncated-JPEG warning that
    `apply` prints for the identical input. Both are live on 0.7.0 and both are silent.
  delivers:
    - "`optimize` never silently returns a larger same-format file — it keeps the source and says so"
    - "`build` warns on a truncated JPEG, matching `apply`"
  explicitly_does_not:
    - "Change cross-format conversion. `-o out.png` is a deliberate convert; growing is legitimate there."
    - "Touch the decide path, which already has the never-bigger guarantee (see the correction in Design Notes)"
    - "Take on the orphaned-artifact follow-up — that needs a scope decision, not a fix"
---

# STAGE-043: pinned-path correctness

## What This Stage Is

The last correctness stage of PROJ-010, and the one that closes a gap the project's own thesis
implies but never covered: **every fix so far landed on the decide path; the pinned path kept the
defects.**

Two items, both live on the shipped 0.7.0 binary, both **silent**.

## Why Now

### D-1. `optimize` silently returns a file 2× the source — driven on 0.7.0, 2026-08-10

`optimize` has two modes. **Auto** (no `-o`) chooses the output format and carries the
never-bigger guarantee (`src/analysis/decide.rs:843`); when its best attempt still exceeds the
source it reports that plainly. Driven on an already-compressed JPEG:

```
crustyimg optimize low.jpg --out-dir auto
  low.jpg: jpeg → avif · 41862 → 52713 B (26% larger)
  note: shipped 52713 B, larger than the 41862 B source (26% larger) — the source could not
        ship unchanged (metadata stripped / orientation baked / resized to the requested
        bound), so the smallest correct output was kept
```

**Pinned** (`-o out.jpg`, or `--format`, or `--profile preserve`) branches away from that entirely
at `src/cli/optimize.rs:622` — `if profile == ProfileArg::Preserve || pinned` — and reaches
neither the guarantee nor the reporting:

```
crustyimg optimize low.jpg -o out.jpg
  41,862 B → 84,586 B        (2.02× LARGER)
  exit 0, stderr completely EMPTY
```

**Why this is worse than it looks.** Re-encoding a lossy JPEG cannot recover detail the source
already discarded, so the output is **larger and no better** than the input. Keeping the original
bytes would have been strictly superior on both axes — the correct answer was free, and the tool
did not take it. A user who runs a verb called `optimize` and gets a file twice the size, with no
message, has been handed a worse file by the thing that promised a better one.

**And the shape is ordinary, not contrived.** `optimize photo.jpg -o out.jpg` is the most natural
way anyone tries the verb, and already-compressed JPEGs — anything pulled off a website — are the
most common input. This is the same defect class as STAGE-034's 18.5× blow-up (shipped verb,
ordinary input, much larger output) and in one respect worse: **the 18.5× case at least reported
the size.**

**Prior art, and why it sat.** Measured 2026-07-13 at **2.65×** and recorded in
`docs/roadmap.md`'s Track B as a CLI-quality line item. It has never been in a stage backlog —
verified by grep, 2026-08-10 — so it has never been planned work. The roadmap already names the
fix; this stage is what makes it real.

### D-2. `build` swallows the truncated-JPEG warning `apply` prints

SPEC-107 made a truncated JPEG announce itself on stderr instead of silently handing back a
partly-grey image. SPEC-111 then gave `build` an auto-decide path through the same seam
(`optimize_decide_one`), but its `encode_one_optimize_decided` wrapper **discards the signal**. So
`apply --recipe web bad.jpg` warns and `build` on the identical input does not. Filed in DEC-087
as a named follow-up, not an AC.

Small, but it is a *silent* regression of a defect this project already decided was worth fixing,
which is exactly the pattern D-1 shows on a bigger scale.

## Success Criteria

- **`optimize` never silently ships a larger same-format output.** When the output format equals
  the source format and the re-encode would grow the file, the **source is kept** and a one-line
  note says so. Proven by driving the reproducer above and asserting on **bytes** — output
  byte-identical to the source, not merely "smaller than before".
- **Cross-format is untouched.** `-o out.png` from a JPEG may legitimately grow; that is a
  deliberate conversion, not a failed optimization. Pinned by a test, not by intent.
- **`--profile preserve` gets an explicit decision, recorded** — guarded automatically like the
  pin, or deliberately exempt as the documented engine-off regression anchor (DEC-048/DEC-059).
  Either is defensible; leaving it unstated is not.
- **`build` warns on a truncated JPEG**, matching `apply` on the identical input, asserted on
  stderr.
- **A negative control for each**: revert the guard, watch the reproducer go red.
- Full matrix clean, and **the CI legs read individually**.

## Scope

### In scope
- The same-format never-bigger guard on the pinned path, plus its note.
- The `--profile preserve` decision.
- Threading SPEC-107's truncated-JPEG warning through `build`'s auto-decide path.

### Explicitly out of scope
- Cross-format conversion behaviour.
- The decide path — it already holds the guarantee and reports honestly.
- **The orphaned-artifact follow-up** (a content change flips the decided extension and the stale
  file stays in `out/`). That needs a scope decision — should `build` clean `out/` at all? — and
  `--check` surfaces it loudly today. Parked deliberately, not forgotten.
- Any new capability.

## Spec Backlog

- [x] **SPEC-113** (design 2026-08-11, shipped 2026-08-14, PR #155) — **`optimize` keeps the source when a same-format
  re-encode would grow it.** The guard, the note, the `--profile preserve` decision. The reproducer is committed
  evidence: an already-compressed JPEG (make one with `sips -s formatOptions 15`) at
  41,862 B → 84,586 B. Needs a **committed fixture**, since the defect only appears on a source
  already compressed past crustyimg's own target quality — [[fixtures-from-the-code-under-test-cannot-fail]]
  applies: generate it with an independent tool, not with crustyimg. Complexity **S–M** (the
  never-bigger comparison already exists on the decide path; the work is reaching it from the
  pinned branch without disturbing cross-format).

- [x] **SPEC-116** (framed 2026-08-15, shipped 2026-08-15, PR #171) — **`build` threads the truncated-JPEG warning.**
  DEC-087's named follow-up: `encode_one_optimize_decided` discards the flag at
  `optimize.rs:1292`, so `apply` warns and `build` is silent on the identical input. Asserts
  both verbs agree, and pins that `--quiet` does NOT suppress it (DEC-085) — the adjacent
  cache warning IS quiet-gated, which is the trap. Complexity **S**.

**Count:** 2 shipped (SPEC-113, SPEC-116) / 0 framed / 0 pending — **backlog complete**

## Design Notes

- **Correction, 2026-08-11 — "the decide path reports honestly" was half true.** This stage's scope
  note excluded the decide path on the grounds that it *"already has the never-bigger guarantee and
  reports honestly."* The guarantee half holds. The reporting half does not: driven at `08b367d`,
  `optimize` on an SVG / HEIC / RAW input can pass the **source container through verbatim** —
  vector XML, a HEIF container, a whole camera-raw file — while reporting it as the `png`/`jpeg`
  its decoder *adopted* as a label. On stdin it writes XML into a `.jpg`. **The scope decision here
  is unchanged and still right** — that defect is a different root cause and belongs in its own
  spec — but the reason recorded for it was wrong, and SPEC-113's build should not read this stage
  as evidence that the decide path is clean. Framed as **STAGE-045 / SPEC-115**; the two stages
  meet at `pipeline_altered_source` and should ship in the same release.
- **Ship this before the launch post.** Not because it blocks the demo — it does not, the demo
  never takes the pinned path — but because the post will name `optimize`, and
  `optimize photo.jpg -o out.jpg` returning a silently doubled file is a top comment rather than
  a bug report. STAGE-041 depends on nothing here, so the two can run in either order; the
  constraint is only that this lands before anyone is *invited* to try the tool.
- **Cut 0.7.1 for it**, or fold it into 0.8.0 if STAGE-042's work lands alongside. A patch is
  correct if this ships alone: it is a bug fix, it adds no verb and changes no flag. Note the
  output for the affected inputs *does* change — but toward the documented promise, which is the
  same reasoning STAGE-040 used for the orientation change.
- **The asymmetry is the lesson worth recording.** Every PROJ-010 fix landed on the path that
  makes a decision, because that is the path the flagship `web` verb takes and the path the
  review findings pointed at. The pinned path is the *other* half of the same fork and nobody
  swept it. That is not a new failure mode — it is STAGE-042's unenumerated-cell pattern again,
  with the cells being **decide vs pinned** rather than recipe × entry point. Worth feeding back
  into STAGE-042's matrix design: the conformance matrix should cross entry points with **both
  modes**, not just the default one.

## Dependencies

### Depends on
- STAGE-040 (shipped) — 0.7.0 is where both defects were driven.
- `DEC-087` — D-2's provenance.
- `docs/roadmap.md` Track B — D-1's 2026-07-13 measurement.

### Enables
- STAGE-041's post can name `optimize` without a caveat.
- STAGE-042's matrix gains a second axis (decide vs pinned) from D-1's root cause.

## Stage Ship (2026-08-15)

**Both delivered capabilities are live on `main`.** `optimize` keeps an already-compressed source
rather than silently returning a larger same-format file (SPEC-113, PR #155), and `build` warns on
a truncated JPEG exactly as `apply` does (SPEC-116, PR #171).

### Success criteria, checked against what shipped

| criterion | verdict |
|---|---|
| `optimize` never silently ships a larger same-format output, asserted on **bytes** | ✅ SPEC-113 |
| Cross-format untouched, **pinned by a test** not by intent | ✅ SPEC-113 |
| `--profile preserve` gets an explicit, recorded decision | ✅ SPEC-113 — stays **exempt** as the engine-off regression anchor (DEC-048), recorded rather than left implicit |
| `build` warns on a truncated JPEG, matching `apply`, asserted on stderr | ✅ SPEC-116, and **stronger than asked**: the test captures `apply`'s actual warning line and asserts `build` emits a string-equal one, so the two cannot drift |
| **A negative control for each** | ⚠️ **partial — the one real miss.** SPEC-116's was re-run at verify in three stages with changed binary hashes. SPEC-113's was too coarse: reverting the whole guard removed its inner sniff too, so the RAW test stayed green on both sides and **the control reported success on a vacuous test.** Recorded in SPEC-113's own ship reflection. |
| Full matrix clean, CI legs read individually | ✅ both |

Six criteria, five clean, one honestly partial. **The partial is not a footnote** — it is the
finding this stage should be remembered for.

### `value_link` check

Both specs delivered what they claimed. SPEC-113's `value_link` promised the pinned-path
never-bigger guarantee and shipped it; SPEC-116's promised `apply`/`build` parity on the warning
and shipped it with a stronger assertion than the criterion required. No drift between claim and
delivery.

### Built vs planned, in three sentences

Two specs planned, two shipped, no scope growth and no third spec discovered. Elapsed 2026-08-10 →
2026-08-15, but the calendar is misleading: SPEC-113's build alone ran ~3h/$40 across overlapping
sessions, while SPEC-116 went design → shipped inside a single day. Cost: **$52.45 total** —
SPEC-113 $30.51 / 35.1M tokens over 2 recorded sessions, SPEC-116 $21.94 / 39.7M over 4.

### The emergent thing: this stage is an accidental controlled experiment

Its two specs are the same size, by the same author, against the same subsystem, two weeks
apart — and were worked under **opposite process rules**. That makes the comparison unusually
clean:

| | SPEC-113 | SPEC-116 |
|---|---|---|
| sessions | ≥4 overlapping, 2 live in the **primary checkout simultaneously** | 1 per cycle, own worktree |
| specs in flight | 3, across unmerged branches | 1 |
| commits during build | 0 until a *different* session swept it into WIP 2.5h later | WIP pushed as soon as it compiled |
| outcome | a **vacuous test** shipped; an open question closed by prose that measurement later falsified | 6 tests, 2 proven RED on `main` first; verify found 3 real defects |
| verify's drift check | **could not fail** (bare `--changed` on a clean checkout) | ran against the base ref, 0 contradictions |

**The variable that moved was process, not difficulty.** SPEC-113's own reflection reaches this
conclusion from the inside — *"an open question inherited across a session boundary gets answered
with prose, because the session that could have driven it is gone"* — and SPEC-116 is the control
that shows the alternative works.

### Follow-up

- **No new stage here.** The pinned path is correct; STAGE-042's conformance matrix (SPEC-118) is
  the right instrument for catching the *next* one of this class mechanically, and it already
  exists.
- **Two findings from SPEC-116 are filed on STAGE-042**, not deferred quietly: `encode_one` has no
  truncation check at all (so `Preserve`/`Pinned` stay silent), and a `build` **cache hit** returns
  before the emit, so run 2 is silent where `apply` always warns.
- **DEC-085's `affected_scope` was corrected** during this stage to include `src/cli/build.rs` —
  it had gone blind to its own second enforcement site.

### Proposed updates to AGENTS / guidance

1. **AGENTS §15 (verify) should say a negative control needs one revert per independent
   condition.** "Revert the guard" is insufficient for a guard with an inner predicate — the exact
   hole SPEC-113 fell through. SPEC-115 and SPEC-116 both did it per-condition afterwards; the
   convention is proven and unwritten.
2. **AGENTS §16.5's one-worktree-per-session rule needs a consequence, not just a statement.** It
   was already written when SPEC-113 violated it. The table above is the evidence that it is
   load-bearing rather than tidy.
3. **The `advance-cycle` step now reaches prompts via `closing-steps-snippet.md`** — SPEC-116 is
   the first spec in this project to move through all five cycles with its `cycle:` field correct
   at every step. That fix works; keep it in the template.

---

## Stage-Level Reflection

*Filled in when status moves to shipped.*
