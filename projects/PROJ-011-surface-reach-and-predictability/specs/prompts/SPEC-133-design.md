# SPEC-133 — DESIGN prompt: `watermark` refuses to damage an image (fit test + 5 % coverage cap)

Cycle: **design**. Fresh CLI session, own worktree (`just wt docs/spec-133-design -- --model opus`).
Starts from the stub the frame session pushed to `main` (ca19da3). **Reserved: SPEC-133, DEC-104.**
STAGE-050 backlog items — **both, as one spec**: "[S] ⚡ **`watermark --text` overruns the right
edge**" and "[S] ⚡ **`watermark` stops silently doing nothing.**"

**Read the stub's `## Frame` section first** — it holds the reproduction and is your binding scope.
This prompt was rewritten 2026-10-25 to match it and to carry two new maintainer rulings.

## Rulings (settled — do not re-open)
**Option D (maintainer, 2026-10-24):**
- **Fit (primary, no threshold):** refuse when the rendered mark plus margin does not fit inside
  the image. Frame found **image overlays fail the same way** (a 256 px overlay on a 24 px image
  changes 99.65 % at exit 0), so fit covers **text and image** overlays, measured after `scale`.
- **Coverage (secondary):** refuse when the mark would change more than **5 %** of the pixels.
  **Fixed, not a setting** (frame scope; a user-settable threshold is out).
- **Single explicit input → error, exit 2, naming the numbers**, on every surface including wasm
  `transform`.
- **Measure geometrically** (count changed pixels / glyph area). **Never SSIMULACRA2**.

**What "skip" means (maintainer, 2026-10-25):**
1. **A skipped image is written without the mark**, with the recipe's other steps still applied.
   It is not left unwritten. So a skip is not a failure: it does not count toward `failed`, does
   not produce exit 6, and does not stop `build` writing its lockfile (`src/cli/build.rs` ~823-829
   returns `PartialBatch` and skips the lock when `failed > 0`; a skip must not reach that path).
2. **A batch or `build` whose only problem is skips exits 0.** Each skipped image gets one stderr
   line naming the image and the numbers, and the run's summary carries a skip count. Precedent:
   DEC-085 (a truncated JPEG warns on stderr, exit 0).

## Scope (from the frame — binding)
- **In:** fit check (text + image, after scale); the 5 % cap; a rule for `--tile`; **recipe text
  mode refuses `tile` and `scale`** (the CLI already refuses them; a recipe accepts them today, 52.7 %
  and 56.2 % changed, and would otherwise get round a tile exemption); single-input refusal on
  every surface; batch/`build` skip as ruled above; **a watermark that fits produces output
  byte-identical to `main`**; docs.
- **Out:** `--min-size` (now its own STAGE-050 item; do not build it), a relative `--size`, a
  user-settable threshold, `--watch` (SPEC-132), the strict-params mechanism itself (SPEC-131).

## Design calls that are yours
- **Where the decision is made.** Frame: in `Watermark::apply` (`src/operation/mod.rs` ~1441),
  before anything is drawn, keeping `src/operation/**` filesystem-free (DEC-031). But `apply()` can
  only return an image or an error; there is no "skipped" result today. Design the shape: how the
  op tells its caller "refused, here are the numbers", and how each surface turns that into an
  exit-2 error (single input) or an unmarked write + report (batch/`build`). Keep it one decision
  point, not one per surface.
- **The cap and `--tile`.** Frame's suggestion, yours to settle: the cap applies when the mark's
  size is fixed (text, or an image without `scale`); a scaled image is proportional by
  construction. `--tile` is exempt from the cap, but one tile must still fit. Drive whichever you
  choose on a release build.
- **⚠ A cached image never runs the pipeline** (`src/cli/build.rs` ~531-540 writes the hit and
  returns). With an unmarked write, the output is deterministic and caches normally, so the skip
  line would print on the first `build` and never again. Decide: accept that and document it, or
  carry the skip into the cache entry / lockfile record so a cached run still reports it. If the
  second option needs a `CACHE_SCHEMA_VERSION` or lockfile change, say so as a design call with its
  reason (PROJ-011 ships as one release, so it rides 0.8.0's single migration).
- **Every surface that applies a watermark** — frame found the original list was short:
  `watermark` (single + batch), `apply --recipe` (single + batch), **`apply --recipe` ending in
  `optimize`** (the pinned-format branch and the auto-decide branch are separate loops), `build`,
  and **wasm `transform`** (single input, so error). Write ACs against each verb's actual report
  surface.
- **Size.** Frame: M at the top of M, and it stays M only if the tests **add up rather than
  multiply**: each rule tested once on the op, then one refusal test per surface and one skip test
  per batch surface. If you find every rule must be tested on every surface, it is L. Then stop
  and report before writing, with the frame's fallback split: (a) the rule + single-input refusal
  everywhere, (b) batch/`build` skip handling.
- **Count the nouns.** Fit and coverage are two rules; text and image are two modes; refusal and
  skip are two behaviours. Each AC naming two of them needs a test per thing.
- Reproduce the rows the ACs rest on (24 px `--size 200` → 6 px / 1.04 %, exit 0; the 256 px long
  string → 3.73 %, clipped; the 256 px overlay on 24 px → 99.65 %) on a release build of `main`,
  with the self-diff = 0 px control.

## Overlap the orchestrator needs to know about
- **SPEC-131** (strict recipe step params, designed `63fbfc0`, building now) rewrites how
  `Watermark` reads its params and adds a per-op key table (`WATERMARK_PARAMS`, with an
  unused-by-mode check). **SPEC-133 builds after SPEC-131 merges.** Design against SPEC-131's
  specified API, not today's: read its spec. The text-mode `tile`/`scale` refusal belongs in its
  unused-by-mode mechanism; say exactly where, so SPEC-131's table cannot re-allow it.
- **SPEC-134** (`--name-template` pins format) shares ~7 files with you (`src/cli/{ops,optimize,
  common,build}.rs`, tests, docs). **Not in parallel with SPEC-134's build.** List every file and,
  for the shared CLI files, the functions you touch.
Frame's estimated file list: `src/operation/mod.rs`, `src/cli/{ops,optimize,common,build,mod}.rs`,
maybe `src/pipeline/mod.rs`, `src/build/{cache,lock}.rs`, `src/wasm.rs`; tests: new
`tests/watermark_damage.rs` plus `tests/{recipe_watermark,build,apply_batch,wasm_roundtrip}.rs`;
docs: `docs/api-contract.md`, `recipes/README.md`, `CHANGELOG.md`; DEC-104.

## DECs to read
DEC-003 (lanes stay separate), DEC-031, DEC-100, DEC-064, **DEC-085** (the warn-and-exit-0
precedent), DEC-015 (exit codes), DEC-058/DEC-059/DEC-101 (cache and lockfile, for the cached-skip
call), and any DEC governing `src/operation/**` watermark and `src/text/**`
(`./scripts/decisions-audit.sh`).

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
2. In the STAGE-050 backlog, change **only your two items'** markers (the frame left each as
   `- [ ] **SPEC-NNN** (framed <date>)`) to `(designed <date>)`. Touch no other item — other design sessions are
   editing their own entries in the same file right now.
3. `git commit -s` (**DCO sign-off is mandatory** — CI fails without it; it has bitten this repo
   four times), conventional message `docs(SPEC-NNN): design <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the rebase
   conflicts in STAGE-050, keep every other session's edits and re-apply only yours.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. **Identify it by content, not
recency**: `grep -l "SPEC-133 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by `.message.id`**: Claude Code writes one
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
