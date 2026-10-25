# SPEC-131 — DESIGN prompt: typed per-operation parameter structs

Cycle: **design**. Fresh CLI session, own worktree (`just wt docs/spec-131-design`). Starts from
the stub the frame session pushed to `main`. **Reserved: SPEC-131, DEC-102.**
STAGE-050 backlog item: "[M] **Typed per-operation parameter structs**".

## What this spec is for
`OperationParams` is a `BTreeMap<String, toml::Value>` with hand-rolled `get_str` / `get_u32` /
`get_f32` (`src/operation/mod.rs`, ~39-71), so every op re-implements its own validation and a
malformed recipe is discovered when the op runs — on input 400 of 500. When this ships, **a
malformed recipe fails at parse time, naming the bad key** (and which step), before any image is
decoded, on every surface that loads a recipe: `apply --recipe`, `build`, and the wasm
`validate()` surface in `src/wasm.rs`.

## Design calls that are yours
- **Struct shape and how it generalises.** Watermark (~10 params, image/text modes, the
  `image`/`font` asset keys and DEC-100's `resolved_bytes` side channel, which is deliberately
  invisible to `Serialize`) is the richest op. The `.cube` LUT op is a decided-but-unscheduled
  second asset-bearing customer (STAGE-050, the CLOSED item) — design the shape so it fits without
  another seam widening. Do not build the LUT op.
- **Round-trip under DEC-005, measured.** Recipes round-trip through the parsed form, and
  `to_toml()` feeds the build cache key and lockfile (DEC-058, DEC-101; SPEC-129's AC-4 pinned
  that a watermark-free recipe hashes exactly as before). Decide, and pin with a test, whether
  `to_toml()` of every existing recipe shape stays **byte-identical** (key order, default
  emission, float formatting). If it cannot, say exactly what changes and why that is acceptable
  inside 0.8.0's single lockfile migration. Use the recipes under `recipes/` and the test fixtures
  as the corpus, and state the corpus boundary.
- **Error wording and where it surfaces** (exit code, stderr text naming key + step index), per
  verb. Unknown key vs wrong type vs out-of-range value: which are parse-time, and is anything
  still left to run time (e.g. an asset file that does not exist)?
- `version = "1"` and `"2"` recipes (SPEC-127 / DEC-099) must both keep parsing; say what the
  typed layer does with each.
- The terminal `optimize` marker is not an operation — say how it is treated.

## Overlap the orchestrator needs to know about
Two other STAGE-050 specs are being designed right now and will build **after** yours if they
touch the same files: SPEC-133 (watermark refuses to damage an image — may add watermark checks,
not new recipe params by default) and SPEC-134 (`--name-template` extension pins format — CLI
output naming). Your Implementation Context file list decides the build order, so make it
complete. If you see a hard interaction with either (e.g. SPEC-133 needs a typed watermark field),
note it in the spec and in your report.

## DECs to read
DEC-005, DEC-031 (why `src/operation/**` cannot touch the filesystem — wasm32), DEC-064,
DEC-099, DEC-100, DEC-101, DEC-058.

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
