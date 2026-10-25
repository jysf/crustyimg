# SPEC-133 — DESIGN prompt: `watermark` refuses to damage an image (fit test + 5 % coverage cap)

Cycle: **design**. New session, own worktree. **Reserved: SPEC-133, DEC-104.**
STAGE-050 backlog items — **both, as one spec**: "[S] ⚡ **`watermark --text` overruns the right
edge**" and "[S] ⚡ **`watermark` stops silently doing nothing.**" The fit test *is* the first
item's fix. Read both entries in full; they hold the measured curve and the re-drive of
2026-10-24.

## The ruling (settled — do not re-open)
**Option D (maintainer, 2026-10-24):**
- **Fit (primary, no threshold):** refuse when the rendered text extent plus margin does not fit
  inside the image. Catches the 24 px `--size 200` case (6 px changed, exit 0 today) and clipped
  words (`"© 2026 crustyimg all rights reserved"` on 256 px loses the rights clause at 3.73 %).
- **Coverage (secondary):** refuse when the watermark would change more than **5 %** of the pixels.
- **Refusal behaviour (2026-08-23 ruling):** single explicit input → error, **exit 2**, naming the
  numbers. Batch, recipe, or `build` → **skip, report, continue; never abort a run** (a mixed
  directory of originals and thumbnails is the normal case).
- **Measure geometrically** (the fraction of pixels the mark covers / changes). **Never
  SSIMULACRA2** — it is non-monotonic at small canvases (24 px scores +50.7, 64 px −82.0).

## Design calls the ruling left to you
- **Is 5 % fixed or a setting?** Default: **fixed**. If you make it a knob, justify it.
- **`--tile`** covers the image by design, so the coverage cap cannot apply unmodified. Decide
  (per-tile fit? exempt from coverage? something else) and drive it.
- **Image overlays, not just `--text`.** The evidence is all text. Decide whether and how fit and
  coverage apply to an `image` watermark (scaled overlay larger than the canvas, overlay coverage).
- **What "skip" means in each lane** — is the output not written, or written without the mark?
  How does `build`'s cache and lockfile treat a skipped target, and what does the run's report /
  exit code say when some inputs were skipped? If the ruling's word "skip" is genuinely ambiguous
  between those, **do not pick silently**: put it in your report as a maintainer question with your
  recommendation, and design the spec around your recommendation so it can proceed.
- **Computing the measure cheaply.** Above the clipping boundary the default glyph run is a
  constant 1647 px, so coverage is `glyph_area / canvas_area` once the extent is known; the text
  layer already computes the extent. Decide whether the check runs before rendering (preferred) or
  after, and keep `src/operation/**` filesystem-free (DEC-031).
- **The value rule (`--min-size`, opt-in, settable in a recipe)** is part of the same 2026-08-23
  ruling in the stage file. It is ruled behaviour, not a new idea. Decide whether it belongs in
  this spec; if including it makes the spec L, say so in your report **before** splitting, and the
  orchestrator will decide. Do not silently drop it.
- Every surface that applies a watermark: the `watermark` verb (single + batch), `apply --recipe`
  (single + batch), and `build`. Write ACs against each verb's actual report surface.
- Reproduce the measured table rows the ACs rest on, on a release build of `main`, with the
  negative control the stage entry used (self-diff = 0 px).

## Overlap the orchestrator needs to know about
SPEC-131 (typed per-op params) is being designed in parallel and rewrites how every op, watermark
included, reads its params; SPEC-134 touches CLI output naming. Your build will likely run after
SPEC-131's. Design against the current param API, but **if you add any recipe-visible watermark
parameter, name it explicitly** in the spec so the typed struct can carry it. List every file the
build touches.

## DECs to read
DEC-003 (lanes stay separate), DEC-031, DEC-100, DEC-064, and any DEC governing
`src/operation/**` watermark and `src/text/**`.

---

## Rules every 0.8.0 design session follows

**You are the design cycle only.** Write the spec; do not build it. A separate session builds it
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
`just new-spec` and `next_id` allocate ids by scanning the working tree, and four design sessions
are running in parallel, so the allocator WILL hand you the wrong id. Run
`just new-spec "<title>" STAGE-050 PROJ-011`, and if it allocates anything other than your reserved
SPEC id, `git mv` both files (spec + timeline) to the reserved id and fix the id inside them.
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
2. In the STAGE-050 backlog, change **only your item's** `- [ ] (not yet written)` marker to name
   your spec id (match how SPEC-129's entry reads). Touch no other item — other design sessions are
   editing their own entries in the same file right now.
3. `git commit -s` (**DCO sign-off is mandatory** — CI fails without it; it has bitten this repo
   four times), conventional message `docs(SPEC-NNN): design <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the rebase
   conflicts in STAGE-050, keep every other session's edits and re-apply only yours.

### Measure your own cost
Your transcript is `~/.claude/projects/<cwd-slug>/<session-id>.jsonl`; the session id is the last
path component of your scratchpad directory. ⚠ **Dedup by `.message.id`**: Claude Code writes one
JSONL line per *content block*, and lines sharing an id repeat identical `input_tokens` /
`cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens` grows — take those
three once per id and the **MAX** output. Summing every line over-counts ~2×; taking the first
line's output under-counts ~11×; both mistakes are on record. Price per component at the model you
actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 input, cache_read ×0.10
input), never a flat rate. Append a `design` entry to the spec's `cost.sessions` with real numbers
and a `tokens_breakdown`, before your final commit.

### Report back (your final message)
First line: spec id, path, and the commit SHA on `main`. Then: AC count and failing-test count;
each design call and how you settled it; the file list the build will touch; anything that needs a
**maintainer** ruling (one line each, with your recommendation); your measured cost. Keep it short —
the orchestrator reads the spec itself.
