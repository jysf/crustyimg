---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-135
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
    - DEC-015   # exit codes: 2 usage, 3 missing input, 1 recipe load
    - DEC-045   # skrifa + zeno rasteriser; where the size-100000 panic is
    - DEC-106   # written by the build: CLI flags checked through the op's key table, before IO
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 6193024
      tokens_breakdown:
        input: 96
        output: 53454
        cache_creation: 159114
        cache_read: 5980360
      estimated_usd: 5.32
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Interactive design session in its own worktree (docs/spec-135-design). Measured from the
        session's transcript (941792af-….jsonl, identified by content), deduped by `.message.id`
        (48 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Opus $5/$25 per MTok, cache_creation x1.25
        ($6.25), cache_read x0.10 ($0.50): $0.0005 + $1.34 + $0.99 + $2.99. Measured before the
        commit/push turns, so it slightly under-counts.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 11038237
      tokens_breakdown:
        input: 150
        output: 43550
        cache_creation: 164015
        cache_read: 10830522
      estimated_usd: 4.52
      duration_minutes: null
      recorded_at: 2026-10-27
      note: >
        Build session in its own worktree (feat-spec-135-watermark-flag-checks). Measured from the
        session's transcript (f090942b-….jsonl, identified by content), deduped by `.message.id`
        (73 unique ids, all claude-sonnet-5-5), input/cache_creation/cache_read once per id, MAX
        output_tokens per id. Priced by component at Sonnet $3/$15 per MTok, cache_creation x1.25,
        cache_read x0.10. Measured before the final commit/PR turns, so it slightly under-counts.
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

**After design (2026-10-27, settled):**
4. **The ~50-row matrix stays [S], with no cut.** The font/overlay ordering rows and the `f32`
   documentation ship in this spec.
5. **The two reworded CLI messages are accepted**, because they now come from the shared table:
   `--opacity must be in …` → `--opacity must be a number in …`, and `--size must be > 0` →
   `--size must be a number > 0 and <= 1024`. No test, doc or skill outside `src/cli/ops.rs`
   quotes the old wording (grep of `tests/ src/ docs/ skills/ recipes/ README.md`, 2026-10-27).
6. **`text::render_text` still panics for a direct library caller passing a huge size.** It is
   filed on STAGE-054 for 0.8.1. This build does not fix it.
7. **The build runs on Sonnet; verify on Opus.**

## Design

Design session 2026-10-27 (Opus), on `main` @ `f926440`. Every row below was driven on a release
build of `main` in a fresh `CARGO_TARGET_DIR` (deleted afterwards), and the panic row also on the
**released `v0.7.1` asset** (`crustyimg-aarch64-apple-darwin.tar.xz`, sha256 checked). Inputs:
ImageMagick `gradient:` PNGs 400×300 (`small`), 1200×1000 (`mid`), 2048×1536 (`big`), a 60×30
white `logo`. Changed pixels counted with ImageMagick (control: input vs itself → 0 px).

### What `main` does today (the RED baseline every test below is measured against)

| Command (CLI `watermark` unless noted) | `main` exit | `main` stderr / effect |
|---|---|---|
| `small --text hi --size nan` / `inf` / `1e39` / `4294967296` | **0** | nothing drawn |
| `mid --text . --size 1025` | **0** | drawn (cap does not exist) |
| `mid --text . --size 1024` | 0 | drawn, 16,641 px changed |
| `mid --text . --size 1024.00001` | 0 | **byte-equal** to `--size 1024` (f32 reads 1024.0) |
| `small --text "© crustyimg" --size 100000` | **101** | `thread 'main' panicked at …zeno-0.3.3/src/raster.rs:144:42` |
| same, **released 0.7.1 binary**, on `small` and on `big` | **101** | same panic — the CHANGELOG **Fixed** line is true of the release |
| `small --text hi --size 100000` (control) | 2 | fit refusal, no panic: the panic needs the longer string |
| `small --text hi --size=-1` | 2 | `error: --size must be > 0, got -1` |
| `small --image logo --scale nan` | **0** | 1×1 mark |
| `small --image logo --scale inf` | 2 | `…the 4294967295×2147483648 px mark does not fit inside the 400×300 px image` |
| `big --image logo --scale 4294967296` / `1e30` | 2 | `…the 4294967295×2147483648 px mark does not fit inside the 2048×1536 px image` |
| `small --image logo --opacity 1.5` | 2 | `error: --opacity must be in 0.0..=1.0, got 1.5` |
| `--size nan` on: batch dir / glob→1 / missing input / stdin (empty) | 0 / 0 / 3 / 4 | read before the check |
| `--scale nan` on: batch dir / glob→1 / missing input / stdin (empty) | 0 / 0 / 3 / 4 | read before the check |
| `--text hi --font /no/f.ttf --size nan` | **3** | `could not read image input` (font read first) |
| `--image /no/l.png --scale nan` | **3** | overlay read first |
| `--image /no/l.png --gravity bogus` | **3** | overlay read first |
| `--text hi --font /no/f.ttf --color zzz` | **3** | font read first |
| `apply --recipe` `text="."`, `size = 1025` / `1024.00001` on `mid` | **0** / **0** | loads; the recipe has no cap either |
| `apply --recipe` `text="© crustyimg"`, `size = 100000` | **101** | the same panic, through a recipe |
| `apply --recipe` `image="logo.png"`, `scale = 4294967296` / `1e30` on `big` | 2 | the saturated message |
| CLI vs `apply --recipe`, default size / `size = 1024` / `scale = 0.1` | 0 | `cmp`: byte-equal pairwise |

`1e30` matters: `1e30 × 2048` is past `u64::MAX` too, so computing the mark in `u64` would still
saturate. That settles Call 2.

### Call 1 — how the CLI reaches the table: a public per-key check, not `OperationParams`

The CLI checks `--size`, `--scale` and `--opacity` one flag at a time against their rows in
`WATERMARK_PARAMS`, through three items `src/operation/registry.rs` makes public:

```rust
impl ParamKind {
    /// Whether `value` satisfies this kind (the check `check_params` runs per key).
    pub fn accepts(&self, value: &toml::Value) -> bool { self.check(value).is_ok() }
    /// The phrase after "must be" in an error, e.g. `a number > 0 and <= 1024`.
    pub fn expectation(&self) -> String { /* today's private fn, unchanged */ }
}
/// The row for `key` in `specs`, if there is one.
pub fn param_spec<'a>(specs: &'a [ParamSpec], key: &str) -> Option<&'a ParamSpec>;
```

`check_params` keeps calling the private `check` and keeps its wording byte-for-byte. Why not
build the flags into `OperationParams` and run `check_params` (the `resize` precedent)?
- Its message names the recipe key: `'size' must be …`. The user typed `--size`.
- The CLI holds an `f32`; `toml::Value::Float(f64::from(1.1f32))` prints as
  `got 1.100000023841858`. The per-key check lets the CLI print the value it parsed (`got 1.1`).
- `Watermark` is built directly (DEC-031); the overlay is pixels, not a path, so a params map
  would carry dummy `image`/`text` entries only to satisfy the table's mode rules.

The CLI side, in `src/cli/ops.rs` (new private helpers; names are the builder's):

```rust
/// Check one watermark flag against the key table a recipe step is checked against.
fn check_watermark_flag(flag: &str, key: &str, value: Option<f32>) -> Result<(), CliError> {
    let Some(v) = value else { return Ok(()) };
    let Some(spec) = registry::param_spec(WATERMARK_PARAMS, key) else {
        return Err(CliError::Usage(format!("internal error: no watermark key '{key}'")));
    };
    if spec.kind.accepts(&toml::Value::Float(f64::from(v))) { return Ok(()) }
    Err(CliError::Usage(format!("{flag} must be {}, got {}", spec.kind.expectation(), flag_value_text(v))))
}
/// `nan`, `inf`, `-inf`, or the value as parsed (`1025`, `1.5`).
fn flag_value_text(v: f32) -> String;
```

`f64::from(v)` is exact, so the table's `as f32` re-read is a no-op and the check is exactly the
f32 range check — **ruling 3's residual, by construction**: `--size 1024.00001` parses to
`1024.0` and passes; the recipe's `size = 1024.00001` is an f64 above 1024 and is refused.

**What replaces each hand-written check** (`src/cli/ops.rs` on `main`):
| `main` | replaced by |
|---|---|
| `size <= 0.0` (`:1382-1385`, in `watermark_overlay`, after the font read) | `check_watermark_flag("--size", "size", src.size)`, first line of `run_watermark` |
| `!(0.0..=1.0).contains(&opacity)` (`:1418-1423`, after the overlay load) | `check_watermark_flag("--opacity", "opacity", opacity)`, same place |
| `s <= 0.0` (`:1425-1429`, after the overlay load) | `check_watermark_flag("--scale", "scale", scale)`, same place |
| `--gravity` parse (`:1413-1416`, after the overlay load) | the same `Gravity::from_str`, moved ahead of all IO (not table-routed: the parser is the source of truth for both paths already) |
| `--color` parse (`:1378-1381`, after the font read) | the same `parse_color`, moved ahead of all IO; `watermark_overlay` takes the parsed colour and size |
| `--margin` | unchanged: clap's `u32` already is the table's `UInt { min: 0 }` range |

**Order in `run_watermark` after the change:** the five flag checks → `watermark_overlay` (font
read or overlay load) → op construction → `run_pixel_op` (where inputs, globs, dirs and stdin are
resolved and read). No file is opened before the five checks.

**Exact CLI messages** (stderr is this one line; exit 2):
```
error: --size must be a number > 0 and <= 1024, got nan
error: --size must be a number > 0 and <= 1024, got inf        # also for --size 1e39 (clap parses it to inf)
error: --size must be a number > 0 and <= 1024, got 1025
error: --size must be a number > 0 and <= 1024, got -1
error: --size must be a number > 0 and <= 1024, got 100000
error: --scale must be a finite number > 0, got nan
error: --scale must be a finite number > 0, got inf            # also for --scale 1e39
error: --opacity must be a number in 0.0..=1.0, got 1.5
```
The `--opacity` and `--size -1` wordings change (`must be in` → `must be a number in`; `> 0` →
`a number > 0 and <= 1024`): they now come from the same table phrase a recipe prints.

### Call 2 — the oversized mark: say it does not fit, without a number

`Watermark::mark_size` returns `Option<(u32, u32)>`. It computes exactly today's expressions in
`f32`, and returns `None` when a dimension's rounded `f32` value is not below `2^32`
(`u32::MAX as f32` is exactly `4294967296.0`), i.e. exactly where today's `as u32` saturates (or
the value is NaN):

```rust
let w = (base_width as f32 * s).round();
if !(w < u32::MAX as f32) { return None; }          // NaN fails this too
let target_w = (w as u32).max(1);
let h = ((target_w as f32) * (oh as f32) / (ow as f32)).round();
if !(h < u32::MAX as f32) { return None; }
Some((target_w, (h as u32).max(1)))
```

For every `Some`, the values are bit-for-bit today's (same expressions, an in-range cast), so no
accepted mark changes. `None` in `refusal()` yields a new variant
`Refusal::ScaledMarkTooLarge { image: (u32, u32) }`, displayed as
`the scaled mark does not fit inside the {w}×{h} px image`; `apply` already returns
`OperationError::Refused` from `refusal()` first, and its two composite-time `mark_size` calls
(16-bit and 8-bit branches) take the `None` arm as the same refusal (no `unwrap`/`expect`).
`Refusal` is new in 0.8.0 (SPEC-133, unreleased), so adding a variant breaks no released API.

Why not `u64`: `--scale 1e30` on 2048 px is 2.0e33, past `u64::MAX` — driven above, it prints the
same saturated pair. Why not print the scale: `4294967296f32` displays as `4294967300`, and after
the flag check only finite scales reach here; the image size is the actionable number.

**Exact message**, CLI `watermark --image logo.png --scale 4294967296` on a 64×48 input, exit 2:
```
error: watermark refused: the scaled mark does not fit inside the 64×48 px image
```
`apply --recipe` with `scale = 4294967296` prints the same line. `--scale inf` / `1e39` never get
here now: the flag check (CLI) or the schema (recipe) refuses them first.

### Call 3 — help text and docs (worded now, written at build)

- `--size` help (`src/cli/mod.rs:418`): `/// Font size in px for \`--text\`: above 0, at most 1024 (default 32).`
- `docs/data-model.md:229`: `| \`size\` | number (integer or float) | \`> 0\` and \`<= 1024\` |`, and
  under the table: "The `watermark` CLI checks `--size`, `--scale` and `--opacity` against these
  same rows, before it reads any file. It parses them as 32-bit floats, so a value within one f32
  step of a bound (`--size 1024.00001`, `--opacity 1.00000001`) reads as the bound and is
  accepted on the CLI, while the same value in a recipe is refused."
- `docs/api-contract.md` §`watermark` (`:372-374`, `:382-383`): replace "`≤0` → exit 2" and
  "bad opacity/scale … → exit 2" with: "`--size` (px, default 32) must be above 0 and at most
  1024; `--scale` above 0 and finite; `--opacity` in 0–1. A bad `--size`, `--scale`, `--opacity`,
  `--gravity` or `--color` is a usage error (exit **2**) reported **before any file is read** —
  before the inputs, `--font` and `--image` — so it wins over a missing file (exit 3)." Add the
  f32 residual sentence, and in the refusal list (`:419`) the `the scaled mark does not fit inside
  the W×H px image` form.

### Call 4 — DEC-106, written at build

DEC-106, not an amendment: DEC-102 says *what* the table checks; the new rule is that a CLI verb
which builds its op directly (DEC-031) still validates its flags through that op's table, before
any IO. That is a refinement of both. **Written at build**, in the same PR as the code: a DEC on
`main` describing a check that does not exist yet would be false until the merge, and the build
also has to edit DEC-102's Consequences line, which design may not touch.
- Title: `A CLI verb that builds its op directly checks its flags through the op's key table, before any file is read`.
- Decision: the three-item public per-key check (Call 1); flag messages name the flag; all
  `watermark` flag checks precede all file IO; `size <= 1024` is a schema row (ruling 1); the
  `f32` residual (ruling 3) is accepted and documented; an unrepresentable scaled mark is
  `ScaledMarkTooLarge` (Call 2).
- Alternatives: `OperationParams` + `check_params` (Call 1's three reasons); CLI `f64` parsing
  (ruling 3: changes bytes); `u64` mark size (`1e30`); a DEC-036 resource limit (DEC-036 bounds
  the document, not a key; only the table is read by both paths).
- `affected_scope` (block list): `src/cli/ops.rs`, `src/operation/registry.rs`,
  `src/operation/mod.rs`, `src/cli/mod.rs`, `docs/api-contract.md`, `docs/data-model.md`.
- DEC-102 Consequences, the line "An upper bound on watermark `size` is **not** set here (a
  resource-limit question in DEC-036's family)" becomes: "An upper bound on watermark `size`
  (1024) is set by DEC-106, as a row in this table; …" (keep the "did you mean" clause).
- Revisit if: another verb grows hand-written flag checks; or the CLI moves to `f64` parsing.

### Call 5 — CHANGELOG lines (written at build under `## [Unreleased]`)

**Breaking:**
- A text watermark's `size` is now at most 1024, on the CLI (`--size`) and in a recipe
  (`size = …`). A larger value is refused: exit 2 on the CLI, exit 1 when the recipe loads.
  Before, it was accepted; a mark that size only fits an image of about 34 megapixels or more,
  and much larger sizes could exhaust memory or crash.

**Fixed:**
- `watermark --size nan`, `--size inf` and `--scale nan` exited 0 and wrote the image with no
  visible mark (or a 1-pixel one). They are now refused with exit 2 before any file is read, as
  the same values in a recipe already were.
- `watermark --text "© crustyimg" --size 100000`, and the same `size` in a recipe, crashed
  (exit 101) in 0.7.1. It is now refused before any text is drawn.
- A watermark scaled far past the image (`--scale 4294967296`, or `scale = 4294967296` in a
  recipe) was refused with a meaningless size, `the 4294967295×2147483648 px mark …`. It now
  reads `the scaled mark does not fit inside the W×H px image`.
- A bad `--size`, `--scale`, `--opacity`, `--gravity` or `--color` is now reported before a
  missing `--font` or `--image` (exit 2, was 3), and before any input is read.

### Call 6 — wasm: covered by the schema row, no AC of its own

`wasm::transform` builds through `pixel_recipe.build_pipeline(&OperationRegistry::with_builtins())`
(`src/wasm.rs:232`, `:241`) — the same call `tests/recipe_strict_params.rs`'s `load` makes. There
is no wasm-specific param code, so the cap reaches wasm iff the schema row exists, and AC-1's
library rows fail exactly when wasm would. Verify confirms by reading `src/wasm.rs:225-245` that
the build still goes through `build_pipeline` (a read check, not a test).

### Size

**[S], upper end — no cut applied.** Code: one schema row, three `pub` items, two CLI helpers and a
reorder, one `Option` and one `Refusal` variant (≈ 80 lines). The matrix is ~50 rows (the frame
estimated 29): I added the `--opacity` row (its check is replaced), the `--gravity`/`--color`
ordering rows (moved by the same reorder), the height-only overflow row (trap 10) and a batch
row for the message. 46 of them sit in five table-driven tests. The frame's cut line (font/overlay
ordering, f32 docs) is **not** used: both are already cheap rows here.

## Inputs

- **Files to read:** `src/cli/ops.rs:1320-1460` (`watermark_overlay`, `run_watermark`,
  `text_error`); `src/cli/ops.rs:254-283` (`run_resize`, the registry precedent);
  `src/operation/registry.rs:60-240` (`ParamSpec`, `ParamKind::{expectation, check}`,
  `check_params`); `src/operation/mod.rs:195-240` (`Refusal` + Display), `:1255-1317`
  (`WATERMARK_PARAMS`), `:1391-1404` (`mark_size`), `:1670-1790` (`refusal`, `apply`);
  `src/cli/mod.rs:396-433` (the `Watermark` args); `tests/watermark_damage.rs:81-192` (the golden
  pattern to copy); `tests/recipe_strict_params.rs:56-75`, `:235-272`, `:549-600` (helpers, the
  range table, the CLI shape).
- **External APIs:** none.
- **Related code paths:** `src/wasm.rs:225-245` (read only, Call 6); `src/text/mod.rs` (not
  changed; `render_text` is where the panic happens).

## Outputs

- **Files created:** `tests/watermark_cli_params.rs`; `tests/fixtures/watermark/cli_params.golden`;
  `decisions/DEC-106-<slug>.md`.
- **Files modified:**
  - `src/operation/registry.rs` — `ParamKind::accepts` (new, pub), `ParamKind::expectation` (pub),
    `param_spec` (new, pub fn); a unit test.
  - `src/operation/mod.rs` — `WATERMARK_PARAMS` `size` row `max: Some(1024.0)`;
    `Refusal::ScaledMarkTooLarge` + its Display arm; `Watermark::mark_size` → `Option`;
    `Watermark::refusal` and `Watermark::apply` (both composite branches) handle `None`; the
    doc comments on `new_image`/`new_text` that say validation happens in `run_watermark`
    (still true, reworded to "against `WATERMARK_PARAMS`"); unit tests.
  - `src/cli/ops.rs` — `run_watermark` (flag checks first, gravity/colour moved up),
    `watermark_overlay` (takes the parsed size and colour; its `size <= 0.0` check and colour
    parse removed), new `check_watermark_flag` + `flag_value_text`; one unit test.
  - `src/cli/mod.rs` — `--size` help line.
  - `tests/recipe_strict_params.rs` — rows in the range table, two new tests.
  - `docs/api-contract.md`, `docs/data-model.md`, `CHANGELOG.md`,
    `decisions/DEC-102-recipe-step-params-are-validated-at-load.md` (Consequences line).
  - Not `recipes/README.md`: it states no `size` range (checked).
- **New exports:** `ParamKind::accepts(&self, &toml::Value) -> bool`,
  `ParamKind::expectation(&self) -> String` (now pub),
  `operation::registry::param_spec(&[ParamSpec], &str) -> Option<&ParamSpec>`,
  `Refusal::ScaledMarkTooLarge { image: (u32, u32) }`.
- **Database changes:** none. **No** `Cargo.lock`, `Cargo.toml` version, or
  `CACHE_SCHEMA_VERSION` change (checking never rewrites `OperationParams`, DEC-102).

## Acceptance Criteria

Surfaces: **CLI** = `crustyimg watermark`; **recipe** = `crustyimg apply --recipe`; **lib** = the
library API. "Refused on the CLI" = exit 2, stderr is exactly one line (the one quoted), no
output file written. Inputs are generated in-test: `G(w,h)` an RGB gradient PNG, `logo` a 60×30
opaque RGBA **gradient** PNG (not a flat colour: a resize-filter change must show in AC-12). "RED on `main`" quotes `main`'s exit as driven in `## Design`.

**The schema cap — lib + recipe**
- [ ] **AC-1. `size` is capped at 1024 in `WATERMARK_PARAMS`, inclusive.** Recipe load
      (`Recipe::from_toml` + `build_pipeline`): `size = 1025`, `size = 1024.00001`,
      `size = 4294967296`, and `text = "© crustyimg"` + `size = 100000` are refused, reason
      `'size' must be a number > 0 and <= 1024, got …` (the last without panicking); `size = 1024`
      and `size = 1024.0` build. RED on `main`: the four load (the last panics).
- [ ] **AC-2. `apply --recipe` refuses `size > 1024` before any output.** `text = "."`,
      `size = 1025` on `G(1200,1000)` → exit 1, one `error:` line containing
      `recipe step 0 ('watermark'): 'size' must be a number > 0 and <= 1024, got 1025`, no output.
      `text = "© crustyimg"`, `size = 100000` → exit 1, no panic text in stderr. RED: 0 and 101.

**The per-key check — lib**
- [ ] **AC-3. The public per-key check agrees with `check_params`.** For every `Number` key of
      `WATERMARK_PARAMS` (`size`, `opacity`, `scale`) and each of
      `nan, inf, -inf, 0, -1, 1e-50, 0.5, 1, 1.5, 1024, 1024.00001, 1025, 4294967296, 1e39`
      as a `toml::Value::Float`: `param_spec(..).kind.accepts(v)` equals
      `check_params("watermark", WATERMARK_PARAMS, {key: v}).is_ok()`. `param_spec(.., "nope")`
      is `None`. (New API: RED = does not compile on `main`.)

**The CLI refuses a bad flag, naming the flag, before any IO**
- [ ] **AC-4. Each bad value is a usage error with the table's phrase.** CLI on `G(400,300)`
      (`--text hi`) or `G(1200,1000)` (`--text .`, for 1025) → exit 2 and exactly:
      | args | stderr | `main` |
      |---|---|---|
      | `--text hi --size nan` | `error: --size must be a number > 0 and <= 1024, got nan` | 0 |
      | `--text hi --size inf` | `… got inf` | 0 |
      | `--text hi --size 1e39` | `… got inf` | 0 |
      | `--text . --size 1025` (on 1200×1000) | `… got 1025` | 0 |
      | `--text hi --size=-1` | `… got -1` | 2, other words |
      | `--text "© crustyimg" --size 100000` | `… got 100000` | **101** |
      | `--image logo --scale nan` | `error: --scale must be a finite number > 0, got nan` | 0 |
      | `--image logo --scale inf` | `… got inf` | 2, saturated |
      | `--image logo --scale 1e39` | `… got inf` | 2, saturated |
      | `--image logo --opacity 1.5` | `error: --opacity must be a number in 0.0..=1.0, got 1.5` | 2, other words |
- [ ] **AC-5. Boundary at the f32 the op reads, and the documented residual.** CLI
      `--text . --size 1024` on `G(1200,1000)` → exit 0 and a mark is drawn (output pixels ≠
      input pixels); `--size 1025` refused (AC-4). `--size 1024.00001` → exit 0, output
      **byte-identical** to the `--size 1024` output (the residual, pinned); the same value in a
      recipe is refused (AC-1). RED: `--size 1025` exits 0 on `main`.
- [ ] **AC-6. Input arity does not change the answer.** For `--text hi --size nan` **and**
      `--image logo --scale nan`, each of: one input file; a batch directory of two PNGs
      (`--out-dir`); a quoted glob resolving to one file; stdin `-` fed a valid PNG (`-o out.png`);
      a missing input `/no/such.png` → exit 2, stderr exactly that flag's AC-4 line, nothing
      written. RED on `main`: 0 / 0 / 0 / 0 / 3 for both flags (stdin with a valid PNG: driven,
      exit 0).
- [ ] **AC-7. A bad flag wins over a missing font or overlay.** CLI on `G(400,300)` → exit 2:
      `--text hi --font /no/f.ttf --size nan` (stderr the AC-4 `--size` line);
      `--image /no/l.png --scale nan` (the `--scale` line); `--image /no/l.png --opacity 2`
      (the `--opacity` phrase, `got 2`); `--image /no/l.png --gravity bogus` (stderr contains
      `bogus`); `--text hi --font /no/f.ttf --color zzz` (stderr contains `zzz`). RED: all 3.
- [ ] **AC-8. `--size` help states the cap.** `watermark --help` has a `--size` line containing
      `1024`. RED: it says only "default 32.0".

**The oversized-mark message — lib, CLI, recipe**
- [ ] **AC-9. A scaled mark that cannot be represented is `ScaledMarkTooLarge`.** lib,
      `Watermark::new_image` on `G(64,48)`: (a) 60×30 overlay, `scale 4294967296` and (b)
      `scale 1e30` → `refusal()` is `ScaledMarkTooLarge { image: (64, 48) }` and `apply` returns
      `Refused` with it; (c) **height-only overflow** — 1×100 overlay, `scale 1562500` → width
      `1e8` fits a `u32`, height `1e10` does not → `ScaledMarkTooLarge` (a width-only check would
      print `the 100000000×4294967295 px mark`); (d) 60×30 overlay, `scale 1000` →
      `DoesNotFit { mark: (64000, 32000), .. }`, unchanged.
- [ ] **AC-10. Its Display names no size.** `ScaledMarkTooLarge { image: (64, 48) }` displays
      exactly `the scaled mark does not fit inside the 64×48 px image`.
- [ ] **AC-11. No refusal prints a saturated size, on either path.** On `G(64,48)`, exit 2, stderr
      exactly `error: watermark refused: the scaled mark does not fit inside the 64×48 px image`
      and not containing `4294967295`: CLI `--image logo --scale 4294967296`; CLI `--scale 1e30`;
      recipe `image = "logo.png"`, `scale = 4294967296`; recipe `scale = 1e30`. Batch: CLI over
      two inputs with different names (one name twice collides in `--out-dir`), with
      `--scale 4294967296 --out-dir` → exit 0, two `warning:` lines each ending
      in `the scaled mark does not fit inside the 64×48 px image`, none containing `4294967295`.
      RED: every row prints `4294967295×2147483648`.

**Nothing accepted changes**
- [ ] **AC-12. Accepted marks are byte-identical to `main`, on both paths.** Decoded-pixel
      SHA-256 (the `tests/watermark_damage.rs` `pixel_hash` scheme, applied to the decoded output
      file) equals a golden captured from **unmodified `main`** for: (1) `--text "© crustyimg"` at
      the default size on `G(400,300)`; (2) `--text . --size 1024` on `G(1200,1000)`;
      (3) `--image logo --scale 0.25` on `G(400,300)`; and the same three as one-step recipes via
      `apply --recipe` (each recipe output hash equals its CLI case's golden — driven equal on
      `main`). 3 goldens, 6 runs.
- [ ] **AC-13. Recipes that loaded before still load** (every existing
      `tests/recipe_strict_params.rs` and `tests/watermark_damage.rs` test stays green, unchanged
      except the range table's row count), and no `Cargo.lock` / version / `CACHE_SCHEMA_VERSION`
      change appears in the diff.

**Docs and record** (verify reads these; documentation has no green)
- [ ] **AC-14.** `docs/data-model.md` (range row + CLI/f32 paragraph), `docs/api-contract.md`
      (`watermark` ranges, before-any-IO order, residual, new refusal form), `CHANGELOG.md` (Call 5
      lines), `DEC-106` (Call 4, block-list `affected_scope`), DEC-102's Consequences line. No
      spec or DEC ids in user-facing text (help, messages, CHANGELOG).

## Failing Tests

Written during **design**, BEFORE build. Each is RED on `main` for the reason given.

- **`tests/watermark_cli_params.rs`** (new; drives the real binary like `tests/recipe_strict_params.rs`)
  - `"a bad flag value is a usage error naming the flag"` — AC-4: the 10-row table; asserts exit 2,
    `error_lines(stderr) == [expected]`, no output file. Asserts the row count is 10.
    RED: rows 1–4, 7 exit 0; row 6 exits 101; rows 5, 8–10 print other words.
  - `"size 1024 is accepted and 1025 refused on the cli"` — AC-5: 1024 → exit 0, output pixels ≠
    input; 1025 → exit 2. RED: 1025 exits 0.
  - `"the f32 residual is accepted on the cli"` — AC-5: `--size 1024.00001` exit 0 and its output
    bytes equal `--size 1024`'s. (Green on `main` too: it pins documented behaviour, not a fix;
    its mutation is the CLI moving to `f64`.)
  - `"a bad flag is refused before any input is read"` — AC-6: 2 flags × 5 shapes = 10 rows; exit
    2, exact line, no output (dir empty / `-o` absent). Asserts 10 rows. RED: 8 rows exit 0, 2 exit 3.
  - `"a bad flag is refused before the font or overlay is read"` — AC-7: 5 rows. RED: all exit 3.
  - `"size help states the cap"` — AC-8.
  - `"an oversized scale names no saturated size"` — AC-11: 2 CLI + 2 recipe single-input rows,
    1 batch row. RED: `4294967295` in every row.
  - `"accepted marks are byte-identical to main"` — AC-12: 6 runs vs 3 goldens; asserts the
    golden file has 3 lines. Green on `main` **by construction** (goldens captured there); it is
    the guard that the build changes no accepted pixel.
  - `"bless cli params goldens"` — `#[ignore]`, writes the golden only with `CRUSTYIMG_BLESS=1`
    (copy `tests/watermark_damage.rs:174-187`).
- **`tests/recipe_strict_params.rs`**
  - `out_of_range_is_refused_for_every_ranged_key` — add rows `text="hi"`+`size = 1025`,
    `size = 1024.00001`, `size = 4294967296`, `text="© crustyimg"`+`size = 100000`; count 17 → 21.
    RED: the four load (the last panics).
  - `"the size cap is inclusive"` (new) — AC-1: `size = 1024` and `size = 1024.0` build.
  - `"apply refuses a size past the cap before any output"` (new) — AC-2: both rows. RED: 0 / 101.
- **`src/operation/registry.rs`** (unit)
  - `"per key check agrees with check params"` (new) — AC-3. RED: does not compile.
- **`src/operation/mod.rs`** (unit)
  - `"a scale past u32 refuses as scaled mark too large"` (new) — AC-9 (a)–(c), refusal and
    `apply`. RED: does not compile (new variant); semantically, (c) is the trap-10 row.
  - `"a large representable scale still names its size"` (new) — AC-9 (d). Green on `main`; the
    guard against "always say scaled mark".
  - `refusal_display_names_the_numbers` — extend with AC-10's exact string.
- **`src/cli/ops.rs`** (unit)
  - `"every checked watermark flag has a table row"` (new) — `param_spec(WATERMARK_PARAMS, k)` is
    `Some` and a `Number` for `size`, `scale`, `opacity` (guards a key typo turning into the
    "internal error" branch).

### Negative controls for verify (one per independent condition)

Each is a source mutation on the build branch; rebuild **clean** (`cargo clean -p crustyimg` or
a fresh target dir) before running, and confirm the binary changed.
1. **The CLI bypasses the table** — restore the three hand checks (`size <= 0.0`,
   `!(0.0..=1.0).contains`, `s <= 0.0`) in place of the `check_watermark_flag` calls → RED:
   AC-4 rows (nan, inf, 1e39, 1025, 100000, scale nan), `"size 1024 is accepted and 1025 refused"`.
2. **The cap row is removed** (`max: None` on `size`) → RED: the four new range-table rows,
   `"apply refuses a size past the cap…"`, AC-4's 1025 and 100000 rows, AC-5's 1025, AC-3.
3. **The saturated message is restored** (`mark_size` back to the saturating `as u32`, no
   `None`) → RED: `"an oversized scale names no saturated size"`, AC-9 (a)–(c).
   **3b. Only the width is checked** (drop the `h` guard) → RED: AC-9 (c) only. This is the
   trap-10 pair: the derived height differs from the width check.
4. **The flag checks move back after the IO** (call them after `watermark_overlay`) → RED: AC-7's
   five rows (exit 3). AC-6 stays green under this mutation, correctly: inputs are read in
   `run_pixel_op`, after both positions, so only the font/overlay rows can see this order.
5. **The CLI parses `f64`** (clap `Option<f64>`, `as f32` at the op) → RED: `"the f32 residual
   is accepted on the cli"` (1024.00001 is then refused).
6. **Byte identity** — change the scaled overlay's `FilterType::Lanczos3` to `Triangle` (both
   composite branches) → RED: AC-12 case 3 (CLI and recipe). This needs the gradient `logo`.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-102` — the per-op key table checked at load; this spec adds a row and exposes a per-key
  check. Its Consequences line about the `size` bound is amended (Call 4).
- `DEC-106` — **written by this build** (Call 4).
- `DEC-031` — the CLI builds `Watermark` directly and loads the overlay at the IO boundary; it
  still does, but validates through the table first.
- `DEC-104` — the fit/coverage refusal whose message Call 2 fixes; refusal before drawing.
- `DEC-015` — exit codes: 2 usage, 3 missing input, 1 recipe load error, 6 partial batch.
- `DEC-036` — recipe *document* limits; explicitly **not** where the cap goes.
- `DEC-063` — 64 MP / 1 GiB decode budget; the yardstick for 1024 (frame).
- `DEC-045` — `skrifa` + `zeno` rasteriser; where the panic is. Not changed.

### Constraints that apply

- `untrusted-input-hardening` — a flag value or recipe key is untrusted; refuse, never panic.
- `no-unwrap-on-recoverable-paths` — `mark_size`'s `None` and the missing-table-row branch return
  errors; no `unwrap`/`expect` in `src/**`.

### Prior related work

- `SPEC-131` (shipped) — the schema and `check_params`; trap 9 (f32 boundary) came from its verify.
- `SPEC-133` (shipped) — `Refusal`, fit/coverage, batch skip; `tests/watermark_damage.rs`'s
  golden-hash pattern this spec copies.
- `SPEC-128` (shipped) — watermark in a recipe; `run_resize` as the registry-from-CLI precedent.

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
- **`text::render_text` itself still panics** for a direct library caller passing a huge size
  (the `zeno` index panic). Both shipped surfaces (CLI, recipe/wasm) are capped first. Design
  reports it to the orchestrator for filing; the build does not fix it.
- **The font-read error wording** ("could not read image input" for a missing `--font`).
  Filed on STAGE-054.
- **An upper bound on `--scale`.** Not needed: the fit test already refuses every scale too big
  to fit, in pure geometry. Only its message is in scope.
- **`margin` changes**; routing `--gravity`/`--color` through the table (only their position moves).
- **`--min-size`**, the cached-`build` warnings, and every other STAGE-054 item.

## Notes for the Implementer

- **Commit order matters for AC-12.** Commit 1 adds `tests/watermark_cli_params.rs` with only the
  byte-identity test + bless + `tests/fixtures/watermark/cli_params.golden`, blessed with
  `CRUSTYIMG_BLESS=1 cargo test --test watermark_cli_params -- --ignored bless` **before any
  `src/` change**. Commit 1 must touch no `src/` file (verify checks). Then the source change and
  the rest of the tests. Push a WIP once it compiles, before the full matrix.
- `pixel_hash` decodes the output PNG with `image::open` and hashes width, height, `color()` Debug
  and raw bytes, as `tests/watermark_damage.rs:139-150` does. PNG file bytes are not hashed.
- `stdin` row: spawn with `Stdio::piped()`, write a valid `G(400,300)` PNG, close it.
- Glob row: pass the pattern as one argument (`dir/a*.png`), no shell.
- `error_lines` is in `tests/recipe_strict_params.rs:566`; copy it.
- The 1200×1000 input is needed only where `size` 1024/1025 must otherwise be *accepted*: on
  400×300 a 1025 mark is refused by the fit test on `main` too, so the row would pass hollow.
- The `© crustyimg` string is load-bearing in the 100000 rows: `hi` at 100000 does not panic on
  `main` (it is fit-refused), so it would not be RED.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. Read `$?`, not a pipe's. Commits need `git commit -s`.
- Out-of-spec finding rule (HANDOFF rule 6): record anything else you find under **Out-of-spec
  finding** in Build Completion.

---

## Build Completion

- **Branch:** `feat/spec-135-watermark-flag-checks`
- **PR (if applicable):** draft, see the PR for the number
- **All acceptance criteria met?** yes (AC-1..14; see the table below)
- **New decisions emitted:**
  - `DEC-106` — A CLI verb that builds its op directly checks its flags through the op's key table, before any file is read (DEC-102's Consequences line amended)
- **Commit 1:** `d1484cf` — `git show --stat` lists only `tests/fixtures/watermark/cli_params.golden` and `tests/watermark_cli_params.rs`; no `src/`. Golden has 3 lines, blessed on unmodified `src/`.
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`):
  - `CHANGELOG.md` — one Breaking and four Fixed lines under Unreleased
  - `decisions/DEC-102-recipe-step-params-are-validated-at-load.md` — Consequences line points at DEC-106
  - `decisions/DEC-106-…md` — new; block-list `affected_scope`
  - `docs/api-contract.md` — watermark ranges, before-any-IO order, f32 residual, new refusal form
  - `docs/data-model.md` — `size` row `> 0` and `<= 1024`, CLI/f32 paragraph
  - `src/cli/mod.rs` — `--size` help line
  - `src/cli/ops.rs` — `check_watermark_flag`, `flag_value_text`, flag checks first in `run_watermark`, `watermark_overlay` takes parsed colour and size, three hand checks removed, one unit test
  - `src/operation/mod.rs` — `size` row `max: Some(1024.0)`, `Refusal::ScaledMarkTooLarge` + Display, `mark_size` → `Option` (+ `fits_u32`), `refusal`/`apply` handle `None`, doc comments, unit tests
  - `src/operation/registry.rs` — `ParamKind::accepts`, `ParamKind::expectation` (pub), `param_spec`, one unit test
  - `tests/fixtures/watermark/cli_params.golden` — new, 3 lines
  - `tests/recipe_strict_params.rs` — 4 range rows (17 → 21), `the_size_cap_is_inclusive`, `apply_refuses_a_size_past_the_cap_before_any_output`
  - `tests/watermark_cli_params.rs` — new
  - (the spec, its prompts and timeline are the orchestrator's; `Cargo.lock`, version and `CACHE_SCHEMA_VERSION` untouched)
- **RED baseline on unmodified `src/`** (exit codes driven on `main`): `--size nan/inf/1e39` 0; `--text . --size 1025` on 1200×1000 0; `--size=-1` 2 (old words); `© crustyimg --size 100000` 101; `--scale nan` 0; `--scale inf/1e39` 2 (saturated); `--opacity 1.5` 2 (old words); font/overlay rows (`--size nan`, `--scale nan`, `--opacity 2`, `--gravity bogus`, `--color zzz`) all 3; missing input 3; help says only "default 32.0". Recipe: `size = 1025` loads, exit 0; `size = 100000` panics. New-API unit tests (AC-3, AC-9, AC-10, the CLI row check) fail to compile on `main`. Green on `main` as designed: byte-identity, the f32 residual, `the_size_cap_is_inclusive`, AC-9 (d).
- **Negative controls** (each rebuilt with `cargo clean -p crustyimg`, diff non-empty, restored and shown green):
  1. hand checks restored → RED: AC-4 table, AC-5 (1024/1025), AC-6, AC-7.
  2. cap row `max: None` → RED: range table, `apply_refuses…`, `a_number_past_f32_range_is_refused`, AC-4, AC-5, AC-6, AC-7. **AC-3 stays green** (it compares the table with itself, so it cannot see the row's value; the spec predicted RED).
  3. saturating `as u32` → RED: `an_oversized_scale_names_no_saturated_size`, `a_scale_past_u32_refuses_as_scaled_mark_too_large`.
  3b. height guard dropped → RED: only the unit test, on its `height only` assertion (row (c)).
  4. checks after `watermark_overlay` → RED: AC-7's five rows, and the AC-4 table (its `© crustyimg --size 100000` row panics in the rasteriser before the check). AC-6 stays green, as predicted.
  5. clap `f64` → RED: `the_f32_residual_is_accepted_on_the_cli` only.
  6. Lanczos3 → Triangle → RED: `accepted_marks_are_byte_identical_to_main` (only the `image_scale_quarter` hash differs) and `watermark_damage`'s passing-marks test.
- **Table row counts:** AC-4 10, AC-6 10, AC-7 5, recipe range table 21, AC-3 42 (3 keys × 14 values), AC-12 3 goldens (6 runs).
- **Matrix** (fresh `CARGO_TARGET_DIR` per leg, sequential): default, `--no-default-features`, `--features webp-lossy`; fmt, clippy `-D warnings` and tests all exit 0 (1089 / 1054 / 1094 passed, 0 failed). A first run was red on clippy only (`neg_cmp_op_on_partial_ord`, `type_complexity`), fixed in `48b6409` and the whole matrix re-run.
- **Deviations from spec:**
  - `a_number_past_f32_range_is_refused` (existing) pinned the phrase `a finite number` for watermark `size`; that phrase becomes `a number > 0 and <= 1024`, so its watermark row now carries its own phrase. No other existing test changed.
  - `size_help_states_the_cap` reads the `--size` entry's following lines, since the first line naming `--size` is prose.
  - `accepted_marks_are_byte_identical_to_main` also asserts each recipe hash equals its CLI hash.
  - `fits_u32` helper added in `src/operation/mod.rs` for clippy; same semantics as the spec's `!(w < u32::MAX as f32)`.
  - Not run: the wasm bundle-size gate (CI only).
- **Follow-up work identified:** `text::render_text` still panics for a direct library caller with a huge size (ruling 6, filed for 0.8.1).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. AC-3 was listed under control 2 but cannot flip (self-consistent by construction).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — clippy's `neg_cmp_op_on_partial_ord` rejects the design's literal `!(w < MAX)`; worth noting in future float-guard designs.

3. **If you did this task again, what would you do differently?**
   — Run clippy once before the first matrix; I paid a full three-leg matrix to learn it.

No design call turned out wrong.

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
