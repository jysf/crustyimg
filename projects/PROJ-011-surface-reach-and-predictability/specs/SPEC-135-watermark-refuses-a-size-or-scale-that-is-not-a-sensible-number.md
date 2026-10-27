---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-135
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-054
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-7
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-10-27

references:
  decisions:
    - DEC-102   # recipe step params validated at load; the schema this spec reuses
    - DEC-104   # watermark damage rule; the fit refusal whose message saturates
    - DEC-036   # recipe resource limits; the family the size cap was filed under
    - DEC-063   # peak-decode memory budget; the yardstick for the cap
    - DEC-031   # CLI builds the watermark op directly, not via the registry
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-131  # the recipe-side schema (shipped)
    - SPEC-133  # the fit/coverage refusal and batch skip (shipped)
    - SPEC-128  # watermark in a recipe (shipped)

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-054's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-054's first deliverable, and the 0.8.0 gate: a watermark size or scale that
  is not a sensible number is refused on the CLI as it already is in a recipe.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4571319
      tokens_breakdown:
        input: 80
        output: 37172
        cache_creation: 127504
        cache_read: 4406563
      estimated_usd: 3.93
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Interactive frame session in its own worktree (docs/spec-135-frame). Measured from the
        session's transcript (fb3cc926-….jsonl, identified by content), deduped by `.message.id`
        (39 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25
        ($6.25), cache_read x0.10 ($0.50): $0.0004 + $0.80 + $2.20 + $0.93. Measured before
        the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-135: watermark refuses a size or scale that is not a sensible number

## Context

STAGE-054's first backlog item, and the only one that gates 0.8.0 (maintainer ruling
2026-10-27). STAGE-050's Stage Ship judged its criterion 4 ("a watermark that cannot sensibly
be applied is refused or skipped, never silently dropped") only **partly** met, because the
`watermark` CLI verb builds its op by hand (`src/cli/ops.rs:1353-1453`, DEC-031) and never sees
the per-op schema SPEC-131 added for recipes (`WATERMARK_PARAMS`, `src/operation/mod.rs:1261`;
checked by `check_params`, `src/operation/registry.rs:216`). Two STAGE-050 filings, merged into
this one spec: the CLI's non-finite `--size`/`--scale`, and the missing upper bound on a text
mark's `size`.

What a user sees today (release build of `main` at `4d6808b`, `crustyimg 0.7.1`, a
2048×1536 PNG; changed pixels counted with ImageMagick, not with crustyimg):

```
$ crustyimg watermark big.png --text "© crustyimg" --size nan -o out.png   # exit 0, 0 px changed
$ crustyimg watermark big.png --text "© crustyimg" --size inf -o out.png   # exit 0, 0 px changed
$ crustyimg watermark big.png --image logo.png --scale nan -o out.png      # exit 0, 1 px changed of 3,145,728
$ crustyimg watermark big.png --image logo.png --scale inf -o out.png      # exit 2
error: watermark refused: the 4294967295×3221225472 px mark does not fit inside the 2048×1536 px image
$ crustyimg watermark big.png --text "© crustyimg" --size 100000 -o out.png  # exit 101
thread 'main' panicked at …/zeno-0.3.3/src/raster.rs:144:42: range start index 8492 out of range …
```

The same values in a recipe are refused at load with exit 1:
`recipe step 0 ('watermark'): 'size' must be a finite number > 0, got nan`.

## Goal

`watermark --size` and `--scale` are checked against the same schema a recipe step is, before any
file is read: `--size nan`, `--size inf` and `--scale nan` exit 2 with one plain line
(`error: --size must be a finite number > 0 and <= 1024, got nan`, wording for design), on one
input, a batch, a glob or stdin alike. A text mark's `size` gets one upper bound (recommended
1024), the same on the CLI and in a recipe, and no refusal message prints a saturated `u32` as a
mark's size.

## Frame

**Verdict: GO.** Size **[S]** (upper S). Premise confirmed on every row; one row is worse than
filed (a panic), and the sweep found no CLI/recipe disagreement outside `--size` and `--scale`.

### Premise check (release build of `main` @ `4d6808b`, fresh `CARGO_TARGET_DIR`)

Inputs made with ImageMagick (`gradient:` 400×300 and 2048×1536 PNGs, a 60×30 white logo).
Pixel counts: `magick A B -compose difference -composite -alpha off -colorspace gray -threshold 0
-format "%[fx:round(mean*w*h)]" info:`.

| Row | Exit | Changed px (of 3,145,728) | Notes |
|---|---|---|---|
| control: input vs itself | – | **0** | the instrument reads 0 when nothing changed |
| control: `--size 48` | 0 | **3,399** | the instrument sees a normal mark |
| `--opacity 0` (reference) | 0 | 0 | |
| `--size nan` | **0** | **0** | `cmp`: byte-equal to `--opacity 0` ✔ filed |
| `--size inf` | **0** | **0** | `cmp`: byte-equal to `--opacity 0` ✔ filed |
| `--scale nan` | **0** | **1** | ✔ filed ("1 pixel of 3.2 M") |
| `--scale inf` | 2 | – | `the 4294967295×3221225472 px mark …` ✔ filed |

**Mechanism (corrects the backlog's "renders tiny and fits"):** a non-finite size lays out no
glyphs, so `render_text` returns its 1×1 *transparent* fallback (`src/text/mod.rs:194-197`); the
mark has 0 visible pixels, passes the fit and coverage tests, and draws nothing. `--scale nan`
reaches `mark_size` (`src/operation/mod.rs:1400`), where `(w as f32 * NaN).round() as u32` is 0,
clamped to 1: a 1×1 mark. `--scale inf` saturates the same cast to `u32::MAX`, which the fit test
refuses and the message prints.

**Where each path validates:**
- CLI: `watermark_overlay` (`src/cli/ops.rs:1353`) reads the font (`:1368-1371`) and only then
  checks `size <= 0.0` (`:1382-1385`); `run_watermark` (`:1399`) loads the overlay first
  (`:1410`), then checks `opacity` (`:1418-1423`) and `scale <= 0.0` (`:1425-1429`), then calls
  `Watermark::new_image`/`new_text` (`:1436-1449`; constructors at `src/operation/mod.rs:1415`,
  `:1442`, which do no checking). `NaN <= 0.0` is false, so NaN passes both hand checks.
- Recipe: `OperationRegistry::build` → `check_params` (`src/operation/registry.rs:216`) →
  `ParamKind::check`'s `Number` arm (`:149-176`: finite as f64 *and* as the f32 the op reads,
  then range), against `WATERMARK_PARAMS` (`src/operation/mod.rs:1261-1317`; `size` `:1275`,
  `scale` `:1299`); `Watermark::from_params` re-runs it (`:1490-1494`).
- Precedent for reuse: CLI `resize` already builds its op through the registry and maps
  `RegistryError` to exit 2 (`src/cli/ops.rs:254-280`). So the Design Note's "reuse
  `param_specs`" is possible; nothing found says it is wrong.

### Sweep: every numeric `watermark` input, CLI vs recipe (`main` @ `4d6808b`)

CLI = `watermark small.png … --flag=V` (exit, px changed). Recipe = `apply --recipe` with the
same key (exit). `1e39` is past f32 range: clap parses it to `inf`. ✗ = the paths disagree.

| Flag | `nan` | `inf` | `-inf` | `0` | `-5` | huge |
|---|---|---|---|---|---|---|
| `size` CLI | **0, 0 px** ✗ | **0, 0 px** ✗ | 2 | 2 | 2 | `1e39`: **0, 0 px** ✗; `4294967296`: **0, 0 px** |
| `size` recipe | 1 | 1 | 1 | 1 | 1 | `1e39`: 1; `4294967296`: **0, 0 px** |
| `scale` CLI | **0, 1 px** ✗ | 2, saturated msg ✗ | 2 | 2 | 2 | `1e39`: 2, saturated ✗; `4294967296`: 2, saturated |
| `scale` recipe | 1 | 1 | 1 | 1 | 1 | `1e39`: 1; `4294967296`: **2, saturated** |
| `opacity` CLI | 2 | 2 | 2 | 0, 0 px | 2 | 2 |
| `opacity` recipe | 1 | 1 | 1 | 0, 0 px | 1 | 1 |
| `margin` CLI | 2 (clap) | 2 (clap) | 2 (clap) | 0 | 2 (clap) | `4294967296`: 2 (clap) |
| `margin` recipe | 1 | 1 | 1 | 0 | 1 | 1 |

Exit 2 on the CLI vs 1 in a recipe is each path's convention (usage error vs recipe load error,
DEC-102), not a disagreement. Every ✗ is in scope. Two agreeing rows are also wrong and are
fixed by this spec's own cap and message: `size = 4294967296` draws nothing on **both** paths
(the cap refuses it), and `scale = 4294967296` prints a saturated size on **both** paths (the
message fix). `opacity` and `margin` agree everywhere; `opacity = 0` drawing nothing is an
agreeing row, out of scope (see below).

One more disagreement, at the edge of the type: the CLI parses `--size`/`--scale`/`--opacity` as
`f32`, so `--opacity 1.00000001` reads 1.0 and is accepted (exit 0) while the recipe refuses
`opacity = 1.00000001` (exit 1, its f64 is past the bound). With a 1024 cap the same holds for
`--size 1024.00001`. This is HANDOFF trap 9's boundary. **Design decides** whether the CLI hands
the schema an f64 (clap `f64`, then the op's `as f32`) — beware that parsing to f64 then rounding
to f32 can differ by one ulp from parsing straight to f32, which would change bytes for an
accepted mark — or documents this residual. My lean: keep `f32` parsing and document it; the
disagreement is confined to values within one f32 ulp of a bound.

### The cap: text `size`

Cost is the text raster, paid once, **before any input is read**, and it grows with
text length × size² — the image's size does not matter (the fit test refuses afterwards).
Release build, `watermark large.png --text "© crustyimg" --size S` (11 chars), `/usr/bin/time -l`:

| `size` | wall | peak RSS | outcome |
|---|---|---|---|
| 32 | 0.03 s | 87 MiB | drawn |
| 512 | 0.02 s | 60 MiB | refused: 2686×480 mark does not fit |
| **1024** | 0.02 s | **106 MiB** | refused: 5370×960 mark does not fit |
| 2048 | 0.04 s | 291 MiB | refused: 10737×1920 |
| 4096 | 0.15 s | **1,031 MiB** | refused: 21474×3840 |
| 8192 | 0.51 s | 3,953 MiB | refused |
| 10000 | 0.77 s | **5,887 MiB** | refused |
| 20000 | 7.6 s | – | refused (400×300 input) |
| 50000 | **> 60 s** (killed) | – | – |
| 100000 | – | – | **panic, exit 101** (`zeno` raster index) |

Same ladder with `--text x` (1 char): 0.17 s at 20 000, 1.5 s at 50 000. SPEC-131's design
measured `size = 100000.0` at 52 s over 3 inputs in a recipe.

**Recommendation: `size <= 1024`, as a schema range row** (`ParamKind::Number { max:
Some(1024.0) }` on `size`), so the one table bounds both paths.
- 1024 is 32× the default (32). An 11-char mark at 1024 is 5370×960 px with about a third of
  that visible (1,661 visible px of a ~5,000 px box at size 32); DEC-104's 5 % rule then needs
  an image of roughly 34 MP to accept it. At 2048 the same mark could only pass on a ~136 MP
  image, past DEC-063's 64 MP decode cap: no decodable image would ever take it.
- Memory at 1024 stays near a tenth of DEC-063's 1 GiB peak-decode budget, leaving headroom for
  longer strings (text length is bounded separately, see out-of-spec). 4096 would spend the whole
  budget on 11 characters.
- It sits far below the panic (between 20 000 and 100 000 for this string), so the cap also
  closes a crash. The same `zeno 0.3.3` and CLI path are in the `v0.7.1` tag, so released 0.7.x
  very likely panics too (code read at the tag, not driven on the released binary): a
  CHANGELOG **Fixed** line.
- **Schema, not DEC-036's family:** DEC-036 bounds the recipe *document* (bytes, steps) at
  `Recipe::from_toml`; a per-key range is what DEC-102's table exists for, and only the table is
  read by both paths. DEC-102's Consequences line ("an upper bound on watermark `size` is not
  set here … DEC-036's family") is what the decision record amends.

**`--scale` needs no upper bound.** The fit test is pure geometry and runs before any resize
allocates: `--scale 4294967296` is refused in milliseconds. Every scale whose mark is wider than
the image is already refused. Only the *message* is wrong: `mark_size`'s `as u32`
(`src/operation/mod.rs:1400`, `:1403`) saturates. Requirement for design: no refusal names
`4294967295` (or any clamped value) as a mark dimension; how (compute in u64, or say "wider than
the image") is design's.

### Exit codes and input arity

`docs/api-contract.md` §Exit Codes: `2` = "Usage error (bad args)". A bad `--size` or `--scale`
is bad for every input, so it is a usage error on the invocation, never a per-image skip (exit 0
with a warning) or a partial failure (exit 6). Recommend:
- **CLI: exit 2 before reading any file**: one input, a batch directory, a glob resolving to one
  file, stdin, and a missing input (which today proves the existing `--scale 0` check runs first:
  `watermark /no/such.png --image logo.png --scale=0` → exit 2, not 3). Measured today for
  `--scale=0`: exit 2 in all five shapes. `--size=nan`, in the same order: 0 / 0 / 0 / 4
  (empty stdin, failed to decode) / 3 — it is checked only after the inputs are read.
- **Including the overlay and font:** today a bad flag loses to a missing `--font` or `--image`
  (`--font /no/font.ttf --size=-1` → exit 3), because the font/overlay load runs before the flag
  checks (`src/cli/ops.rs:1368`, `:1410`). Moving every flag check ahead of all file IO is the
  same code change; recommend it in scope.
- **Recipe: unchanged**, exit 1 at load (SPEC-131), now also for `size > 1024`.

### Bytes

No accepted mark may change. Design must require a byte-identity check against `main`'s binary
(built from `4d6808b` or the merge-base) for at least: a text mark at the default size, a text
mark at `size = 1024` (the new boundary, accepted), an image mark with `--scale`, and the same
three through `apply --recipe`. No lockfile or cache-key change is expected (DEC-102: checking
never rewrites `OperationParams`).

### Sizing

**[S], upper end.** Code: one schema row, one CLI check routed through the table, one message
fix, check order moved ahead of IO. Verification is the larger half. AC matrix: CLI `size`
{nan, inf, 1e39, 1025, 1024 accepted} + `scale` {nan, inf, 1e39} = 8 value rows; arity
{one, batch dir, glob→1, stdin, missing input} on 2 representative flags = 10; overlay/font
order = 2; recipe {size 1025 refused, 1024 accepted, saturated-message row} = 3; byte identity =
6. About 29 rows, most of them one table-driven test. Not M: no new mechanism, no format change.
Cut line if it grows: the f32-boundary question and the font/overlay check order are the two
items that may be dropped to STAGE-054 without leaving a silent-nothing path.

### Scope

**In:** refuse non-finite `--size`/`--scale` on the CLI through `WATERMARK_PARAMS`; the `size`
cap (1024) in the schema, so both paths; the saturated-`u32` refusal message (both paths); every
`watermark` flag checked before any file IO; docs (`api-contract.md`, `data-model.md` range
table, `--size` help text), CHANGELOG (**Fixed**: the CLI non-finite rows and the `--size 100000` panic;
**Breaking**: `size > 1024` now refused), and one decision record.

**Out:** see "Out of scope" below.

### Rough build file list
- `src/operation/mod.rs` — `size` row `max: Some(1024.0)`; `Refusal::DoesNotFit` / `mark_size`
  message (`:211-221`, `:1396-1404`).
- `src/operation/registry.rs` — possibly a public per-key check (or a CLI-flag wording hook) so
  the CLI can name `--size` rather than `'size'`.
- `src/cli/ops.rs` — `watermark_overlay` / `run_watermark`: build the flags into
  `OperationParams` (or call the per-key check), map to `CliError::Usage`, before any IO; drop
  the hand-written `size`/`scale`/`opacity` checks.
- `src/cli/mod.rs` — `--size` help text (default 32, max 1024).
- `tests/` — a CLI test file for the matrix (e.g. `tests/watermark_cli_params.rs`) and a row in
  `tests/recipe_strict_params.rs`; any test pinning today's `--size must be > 0` wording.
- `docs/api-contract.md`, `docs/data-model.md`, `CHANGELOG.md`.
- `decisions/DEC-106-…md` (reserved) **or** an amendment to DEC-102 — design's call; my lean is
  DEC-106, because "the CLI verb validates through the registry's table although it builds its
  op directly" refines DEC-031 as well as DEC-102.

### Maintainer questions
1. **The cap: `size <= 1024`?** Recommend yes (evidence above). 2048 is the alternative if a
   single giant glyph on a 30+ MP image matters; it costs ~3× the memory per character.
2. **`opacity = 0` and other agreeing "draws nothing" rows stay out of 0.8.0?** Recommend yes:
   they agree on both paths, so they are not this item; filed on STAGE-054 for 0.8.1. It means
   0.8.0's note should say "non-finite or oversized size/scale is refused", not "no invisible
   mark at exit 0" — STAGE-054's first success criterion promises the latter.

### Maintainer rulings (2026-10-27, settled)
1. **Cap: `size <= 1024`**, as a range row in the shared schema (`WATERMARK_PARAMS`), so the CLI
   and recipes share one bound.
2. **Marks that draw nothing on both paths stay out of 0.8.0** (`opacity = 0`, blank text, a tiny
   `size` such as 0.01). They are filed on STAGE-054 for 0.8.1. STAGE-054's first success
   criterion is narrowed to match, and 0.8.0's notes say "a non-finite or oversized size or scale
   is refused", not "never an invisible mark".
3. **Keep `f32` parsing on the CLI** and document the residual: a value within one `f32` step of
   a bound (`--opacity 1.00000001`, `--size 1024.00001`) is accepted on the CLI and refused in a
   recipe. Parsing as `f64` could change bytes for marks accepted today.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- **Any other verb.** The sweep covered `watermark` only (frame rule).
- **Marks that draw nothing while both paths agree:** blank text (`--text " "`), a size too
  small to rasterise a glyph (`--size 0.01`), `--opacity 0`. Filed on STAGE-054 (0.8.1):
  "A watermark with no visible pixels is drawn as nothing at exit 0".
- **A bound on the text raster as a whole.** The size cap bounds one glyph; peak memory grows
  with text length × size², and `--text` has no length limit (a recipe's is 64 KiB, DEC-036).
  Filed on STAGE-054: "A text watermark's raster has no memory bound beyond `size`".
- **The font-read error wording** ("could not read image input" for a missing `--font`).
  Filed on STAGE-054.
- **An upper bound on `--scale`.** Not needed: the fit test already refuses every scale too big
  to fit, in pure geometry. Only its message is in scope.
- **`opacity` / `margin` changes.** Both paths already agree on every swept value.
- **`--min-size`**, the cached-`build` warnings, and every other STAGE-054 item.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
