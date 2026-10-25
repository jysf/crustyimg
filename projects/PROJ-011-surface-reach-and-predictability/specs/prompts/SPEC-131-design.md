# SPEC-131 — DESIGN prompt: typed per-operation parameter structs

Cycle: **design**. Fresh CLI session, own worktree (`just wt docs/spec-131-design`). Starts from
the stub the frame session pushed to `main` (9cccf69). **Reserved: SPEC-131, DEC-102.**
STAGE-050 backlog item: "[M] **Typed per-operation parameter structs**".

**Read the stub's `## Frame` section first.** It corrected this prompt's original premise, and
it is your binding scope. This prompt was rewritten 2026-10-25 to match it.

## What this spec is for (as the frame found it)
The backlog said a malformed recipe fails late, on input 400 of 500. **That is wrong**: `apply` and
`build` build the recipe's pipeline before reading any input (`src/cli/optimize.rs:136`,
`src/cli/build.rs:298`), and bad required params already fail with exit 1 before writing.
The real defect is that **bad keys are silently accepted**:
- **Unknown keys are ignored on every op**, the `optimize` marker included (`bogus = "x"` → exit 0,
  same output as without it). Pinned by `from_toml_tolerates_unknown_step_param_by_design`
  (`src/recipe/mod.rs:597`).
- **Watermark's optional keys fall back to defaults when misspelled or wrong-typed**
  (`opactiy = 0.5`, `opacity = "0.5"`, `margin = -5`, `tile = "yes"`, `size = "big"` all exit 0
  with default output), because the accessors return the same `None` for a missing key and a
  wrong-typed one and every call site does `.unwrap_or(default)` (`src/operation/mod.rs`
  ~1337-1380). That is a silent wrong output.
- **The errors that do fire name neither the step nor the key** (a misspelled `widht` is reported
  as "requires 'width'").

When this ships: **an unknown, wrong-typed or out-of-range step value is refused when the recipe
is loaded, and the error names the step index, the op and the key.**

## Maintainer rulings (2026-10-25 — settled, do not re-open)
1. **Unknown step keys become a load-time error.** This retires DEC-061's **accepted risk #1**
   ("Recipe step params stay tolerant"), which rested on an unknown key being "inert … never a
   wrong output", which watermark has since made false, and which explicitly deferred strict
   per-step validation as a feature. **DEC-102 supersedes that accepted risk** (not the whole of
   DEC-061): mark the clause, follow `decisions/_template.md` and how DEC-101 amended DEC-058.
   The pin test flips on purpose.
2. **No "new in 0.8.0" hint on the error.** A **Breaking** line in the CHANGELOG is enough.

## Design calls that are yours
- **Validate at load, keep the map as the stored form** — the frame's recommendation, and the
  reason this stays M. If you find that impossible and typed structs must become the serialized
  form, stop: that is the L split ("reject at load" here vs "typed storage" as a follow-up), and
  it goes in your report before you write it. Either way, the per-op accepted-key/type/range table
  must be published through the registry (the mechanism DEC-061 said this needs); say how, so the
  `.cube` LUT op (decided, unscheduled) fits without another seam widening. Do not build the LUT op.
- **Round-trip under DEC-005, measured.** With the map kept, `to_toml()` should stay byte-identical
  for every recipe that still parses. Pin it with a test over the corpus (`recipes/` + test
  fixtures; state the boundary). `to_toml()` feeds the build cache key and lockfile (DEC-058,
  DEC-101; SPEC-129's AC-4 pinned that a watermark-free recipe hashes exactly as before).
- **Error wording and exit code** per surface. Unknown key vs wrong type vs out of range: one
  shape or three? "Did you mean…?" suggestions are your call.
- **Surfaces:** `apply --recipe` (single + batch), `build`, and wasm **`transform`**. ⚠ There is no
  wasm `validate()` export (that is roadmap item #6; give it a backlog line, do not build it), and
  wasm `transform` **decodes the image before building the pipeline**, so "refused before any
  image is decoded" is false there today: either check the recipe first in `transform`, or word
  that surface's AC to what it actually guarantees.
- `version = "1"` and `"2"` recipes (SPEC-127 / DEC-099) are validated the same way; the terminal
  `optimize` marker is validated too.
- **Size the verification.** The frame counted 5 ops, 14 keys, ~30 condition rows. All three
  surfaces go through `Recipe::build_pipeline`, so plan table-driven unit tests per op plus one
  integration test per surface plus the round-trip pin, not op × condition × surface. Count the
  nouns in each AC against that list.

## Overlap the orchestrator needs to know about
SPEC-131 and SPEC-133 (watermark damage rule) both edit `Watermark` in `src/operation/mod.rs`, so
**SPEC-131 builds first** and SPEC-133's build follows it; any recipe key SPEC-133 adds goes in
under your strict checks, so make adding a key to the table a one-line change and say where. Your
build may run in parallel with SPEC-132 and SPEC-134 if your file list stays off theirs, so make
the Implementation Context list complete. Frame's estimate: `src/operation/mod.rs`,
`src/operation/registry.rs`, `src/recipe/mod.rs`, `src/wasm.rs`, maybe `src/cli/mod.rs`; tests
`recipe_round_trip`, `recipe_watermark`, `registry_seam`, `build`, `apply_batch`, `wasm_roundtrip`
+ a new strict-params file; docs `recipes/README.md`, `docs/api-contract.md`, `CHANGELOG.md`.

## DECs to read
DEC-005, DEC-031 (why `src/operation/**` cannot touch the filesystem — wasm32), **DEC-061**
(accepted risk #1 and its "Alternatives" entry), DEC-064, DEC-099, DEC-100, DEC-101, DEC-058.

---

## Rules every 0.8.0 design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The frame stub for your spec must now be in
`projects/PROJ-011-surface-reach-and-predictability/specs/` at your reserved id with
`cycle: design`. If it is missing or still `cycle: frame`, stop and report — do not create it. A separate session builds it
from your spec alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15). The orchestrator (another session) sequences builds; it does not design.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Those traps are binding here.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md` — your
   item's backlog entry holds measured evidence and maintainer rulings. A ruling is settled; do not
   re-open it. An open question the ruling left to the spec is yours to settle.
4. `projects/PROJ-011-surface-reach-and-predictability/brief.md`, `/guidance/constraints.yaml`,
   and every DEC this prompt names.
5. A recent shipped spec as the shape to match:
   `projects/PROJ-011-surface-reach-and-predictability/specs/done/SPEC-129-build-cache-hashes-watermark-asset-content.md`.

### How to design well here
- **Read the code before writing the spec.** SPEC-128's backlog title was wrong about the code; a
  design that only read the backlog item would have specced the wrong thing. Cite `file:line`.
- **Drive the current behaviour** on a release build of `main` before you describe it. **Measure
  the scenario, not the control**: the row that carries an argument must use the feature the
  argument is about. Give every measurement a negative control (self-diff = 0) and say what it is.
- **Ask what the user actually sees** — the TOML they write, the stderr line, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing (SPEC-128 shipped "overlay/font" with only overlay tested).
- **Size the verification, not just the code.** If the AC matrix is L-shaped, the spec is L —
  then split it (AGENTS §2) and tell the orchestrator before writing two specs.
- Every AC names the verb surface it applies to (AGENTS §15, "An AC may not transfer between
  verbs"); the Failing Tests section lists each test by name and says why it is RED on `main` today.
- List, in Implementation Context, **every source file the build will touch** (best estimate, by
  path). The orchestrator uses this list to keep two concurrent builds off the same files.
- Name the negative control per independent condition (AGENTS §15) so verify knows what to revert.
- **Everything in PROJ-011 ships as ONE release (0.8.0).** Do not plan a version bump, a release,
  or a `CACHE_SCHEMA_VERSION` change unless your item genuinely needs one — and if it does, say so
  as a design call with its reason.
- Comments and user-facing text read behaviour-first, with no spec/DEC ids in them.

### Ids are reserved — use exactly these
The frame session already created your spec and timeline at the reserved SPEC id; do not run
`just new-spec`. Its `## Frame` section (verdict, sizing, scope in/out, rough file list) is your
binding scope. If you find it wrong, say so in your report rather than silently widening or
narrowing it. Fill Acceptance Criteria, Failing Tests and Implementation Context; keep `## Frame`.
Highest DEC on `main` is DEC-101. Write any DEC with the **block-list** `affected_scope` form
(an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/cli/ops.rs
```
Whether a DEC is written at design or at build is your call; say which in the spec.

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR. You are in your own worktree:
1. Set the spec's `cycle:` to `build` (`just advance-cycle SPEC-NNN build`).
2. In the STAGE-050 backlog, change **only your item's** marker (the frame left it as
   `- [ ] **SPEC-NNN** (framed <date>)`) to `(designed <date>)`. Touch no other item — other design sessions are
   editing their own entries in the same file right now.
3. `git commit -s` (**DCO sign-off is mandatory** — CI fails without it; it has bitten this repo
   four times), conventional message `docs(SPEC-NNN): design <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the rebase
   conflicts in STAGE-050, keep every other session's edits and re-apply only yours.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. **Identify it by content, not
recency**: `grep -l "SPEC-131 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by `.message.id`**: Claude Code writes one
JSONL line per *content block*, and lines sharing an id repeat identical `input_tokens` /
`cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens` grows — take those
three once per id and the **MAX** output. Summing every line over-counts ~2×; taking the first
line's output under-counts ~11×; both mistakes are on record. Price per component at the model you
actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 input, cache_read ×0.10
input), never a flat rate. Append a `design` entry to the spec's `cost.sessions` with real numbers
and a `tokens_breakdown`, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: spec id, path, and the commit SHA on `main`. Then: AC count and failing-test count;
each design call and how you settled it; the file list the build will touch; anything that needs a
**maintainer** ruling (one line each, with your recommendation); your measured cost. Keep it short —
the orchestrator reads the spec itself.
