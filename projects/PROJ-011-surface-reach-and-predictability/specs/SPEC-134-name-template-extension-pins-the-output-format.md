---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-134
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null                # chosen at build dispatch
  created_at: 2026-10-24

references:
  decisions:
    - DEC-015
    - DEC-048
    - DEC-058
    - DEC-087
    - DEC-098
    - DEC-099
  constraints:
    - metadata-not-via-pixel-encode
    - ergonomic-defaults
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-111
    - SPEC-126
    - SPEC-127
    - SPEC-129
    - SPEC-131
    - SPEC-133

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's predictability outcome: a file named `.png` contains a PNG. Today a literal
  `--name-template` extension is silently ignored on every verb except `build`'s terminal-`optimize`
  targets, so the same recipe + template writes different bytes under `apply` and `build`.

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
      tokens_total: 6090335
      tokens_breakdown:
        input: 100
        output: 24680
        cache_creation: 127288
        cache_read: 5938267
      estimated_usd: 4.38
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive frame session in its own worktree (docs/spec-134-frame). Measured from the
        session's transcript (9c638b37-….jsonl, identified by content), deduped by `.message.id`
        (50 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read
        x0.10 ($0.50). Measured before the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-134: a `--name-template` extension pins the output format

## Context

STAGE-050 backlog item "[S] **A literal extension in `--name-template` does not pin the output
format**" (surfaced by SPEC-126's verify, 2026-08-23). **Maintainer ruling, 2026-10-24 — settled:
honour the template.** A recognised extension in `--name-template` pins the output format exactly
as a recognised `-o` extension already does. Byte-changing for a template whose extension differs
from the output format chosen today; rides 0.8.0's single release.

What the user sees today (release build of `main` at `8471fb1`, JPEG source `in.jpg`, recipe
`r.toml` = one `resize mode="max" width=64` step; bytes identified by magic, not by name):

```console
$ crustyimg apply --recipe r.toml in.jpg --out-dir a1 --name-template '{stem}_w.png'   # exit 0
a1/in_w.png: JPEG          ← a file named .png holds a JPEG
$ crustyimg web in.jpg in2.jpg --out-dir wb3 --name-template '{stem}.png'              # exit 0
wb3/in.png: AVIF           ← and stderr says "jpeg → avif" while naming the file .png
```

The same recipe + template is already pinned by `build` and ignored by `apply`:

```toml
[[target]]                 # crustyimg build → out/in.png: PNG  (pinned since SPEC-111 / DEC-087)
source = "../bsrc/*.jpg"
recipe = "web"
out    = "out"
name   = "{stem}.png"
```
while `apply --recipe web in.jpg in2.jpg --out-dir aw1 --name-template '{stem}.png'` writes
**AVIF** into `aw1/in.png`. One rule, two answers, depending on the verb.

## Goal

A recognised literal extension in an output name template (`--name-template`, or a `build`
target's `name`) pins the output format on every verb that encodes, so `{stem}_w.png` always
writes a PNG and the stderr summary names the format the file actually holds. A template using
`{ext}` (including the default `{stem}.{ext}`) keeps today's behaviour exactly.

## Frame

**Verdict: GO. Size: M** (the item was filed [S]). Framed 2026-10-24 on `main` at `8471fb1`.

### Premise check — confirmed, and wider than the backlog row

Driven on `cargo build --release` of `main`, JPEG source made with
`crustyimg convert --format jpeg tests/fixtures/classify/color_photo_fuji.png -o in.jpg` (and a
copy `in2.jpg` for two-input runs), outputs sniffed by magic bytes (`ffd8ff` JPEG, `89504e47` PNG,
`RIFF….WEBP`, `….ftypavif`). **Negative control:** the `apply … {stem}_w.png` row re-run into a
fresh dir and `cmp`'d — 0 bytes differ, so the runs are deterministic and a byte change after the
build is the change's, not noise.

| # | command (all `--out-dir <d>`) | file written | bytes |
|---|---|---|---|
| 1 | `apply --recipe r.toml in.jpg --name-template '{stem}_w.png'` | `in_w.png` | **JPEG** ❌ (stage row 2 — reproduced) |
| 2 | `apply --recipe r.toml in.jpg --name-template '{stem}_w.jpg'` | `in_w.jpg` | JPEG ✅ (stage row 1 — reproduced) |
| 3 | `apply --recipe r.toml in.jpg in2.jpg --name-template '{stem}_w.png'` | `in_w.png`, `in2_w.png` | **JPEG** ❌ |
| 4 | `resize --max 64 in.jpg --name-template '{stem}.png'` | `in.png` | **JPEG** ❌ |
| 5 | `resize --max 64 in.jpg in2.jpg --name-template '{stem}.png'` | 2× `.png` | **JPEG** ❌ |
| 6 | `thumbnail in.jpg --name-template '{stem}.webp'` | `in.webp` | **JPEG** ❌ |
| 7 | `auto-orient in.jpg --name-template '{stem}.png'` | `in.png` | **JPEG** ❌ |
| 8 | `convert --format webp in.jpg --name-template '{stem}.png'` | `in.png` | **WEBP** — `--format` wins (precedence call) |
| 9 | `meta strip in.jpg --name-template '{stem}.png'` | `in.png` | **JPEG** — container lane, never re-encodes |
| 10 | `optimize in.jpg [in2.jpg] --name-template '{stem}.png'` | `in.png` | **AVIF** ❌ (auto-decided) |
| 11 | `web in.jpg --name-template '{stem}.webp'` / 2 inputs | `in.webp` | **AVIF** ❌ |
| 12 | `web in.jpg in2.jpg --name-template '{stem}.png'` | 2× `.png` | **AVIF** ❌ |
| 13 | `web in.jpg in2.jpg` (no template — control) | 2× `.avif` | AVIF ✅ |
| 14 | `apply --recipe web in.jpg in2.jpg --name-template '{stem}.png'` | 2× `.png` | **AVIF** ❌ |
| 15 | `apply --recipe r.toml in.jpg -o o.png` (the `-o` precedent) | `o.png` | PNG ✅ |
| 16 | `resize --max 64 --format png in.jpg --name-template '{name}'` | `in.jpg` | **PNG** — `{name}` carries the *source's* extension |
| 17 | `build`, plain `r.toml`, `name = "{stem}_w.png"` | `in_w.png` | **JPEG** ❌ |
| 18 | `build`, plain `r.toml`, `name = "{stem}"` / `"{stem}.txt"` | `in` / `in.txt` | JPEG, exit 0 |
| 19 | `build`, `recipe = "web"`, `name = "{stem}.png"` | `in.png` | **PNG** ✅ already pinned (DEC-087) |
| 20 | `build`, `recipe = "web"`, `name = "{stem}.{ext}"` | `in.avif` | AVIF ✅ (decide) |

So the bug is not "single-input `apply` made it visible" — **every encoding verb ignores a template
extension**, and on the auto-decide verbs (rows 10–12, 14) the mislabel is a *different codec*, not
just the source format. Row 19 vs row 14 is the sharpest form: same recipe, same template, `build`
pins and `apply` doesn't.

**`web` / `optimize` batch case (the design call the ruling left open) — today's behaviour,
driven, not decided:** `web --out-dir d/ --name-template '{stem}.webp'` auto-decides and writes
**AVIF bytes into `in.webp`** (row 11), with the stderr line `in.jpg: jpeg → avif · … · ssim 81.5`.
So "keep auto-decide" is not a neutral option — it keeps writing a codec the filename denies. The
repo already has a precedent on exactly this question: `build`'s terminal-`optimize` targets
treat a literal template as a pin that **skips** the decision (DEC-087, `src/cli/build.rs:63-69`,
row 19). ⚠ For design: pinning `web` routes it through the pinned path that PROJ-013 / STAGE-037's
`-o` item says silently moves quality 85→80 and drops the report — a template pin would inherit
that, so the AC must say what the stderr line is on a template-pinned `web`.

### Reach — the roster that sets the size

`--name-template` is a global arg (`src/cli/mod.rs:80-82`). Every read of it, and where the format
is resolved without it:

| surface | where the template is read | where the format is resolved (template not consulted) |
|---|---|---|
| `resize`, `thumbnail`, `auto-orient`, `watermark`, `edit`, `convert` (all via `run_pixel_op`) | `src/cli/ops.rs:390-393` (1 input), `:434-437` (N) | `output_format_for`, `src/cli/ops.rs:200-228`, called at `:377` and `:475`; `convert` forces via `forced_format` (`src/cli/optimize.rs:586`) |
| `apply`, plain recipe | `src/cli/common.rs:259-263` (`build_sink`, 1 input), `src/cli/optimize.rs:198-201` (N) | `src/cli/optimize.rs:177` (1), `:212-215` (N) |
| `apply --recipe web` (terminal `optimize`) | as above | pin check `src/cli/optimize.rs:87-94` (`-o`/`--format`/`recipe.format` only) |
| `optimize` | `src/cli/optimize.rs:1594-1597`, `:1612-1615` (`run_optimize_autodecide`) | pin check `src/cli/optimize.rs:691-695` |
| `web` | same autodecide fan-out | pin check `src/cli/optimize.rs:830-834` |
| `build` (manifest `name`, not the flag) | `Target::name` / `Target::template`, `src/build/mod.rs:180-189` | plain recipe: `src/cli/build.rs:283-289` (Preserve unless `recipe.format`); terminal-`optimize`: already pins, `:272-279` |
| `meta strip/clean/copy/set` (container lane) | `src/cli/ops.rs:709-712`, `:740-743` | none — writes the input's own container bytes (`metadata-not-via-pixel-encode`) |
| `responsive` | **never read** — `--name-template` is accepted and silently ignored (`responsive in.jpg --widths 32 --out-dir rp --name-template '{stem}.png'` → `rp/in-32w.jpg`, exit 0) | n/a |

Template expansion itself is `crate::sink::expand_template` (`src/sink/mod.rs:364`); `{name}`
expands to the input's full file name, extension included (row 16).

### Sizing — M, and why not S or L

The code is small: one "does this template name a recognised literal extension?" helper (build
already has it: `target_format_plan`, `src/cli/build.rs:63`) and a new rung at ~6 resolution
sites. But the **verification** is not S:
- **Nine encoding surfaces × two arities** (resize, thumbnail, auto-orient, watermark, edit-single
  only, convert, apply-plain, apply-web, optimize, web, build-plain), each needing its own AC row
  (AGENTS §15: an AC may not transfer between verbs) — and the 1-input and N-input branches are
  separate code at every site above.
- **Three precedence interactions**: template vs `--format` (row 8 shows today's answer), vs
  `recipe.format` (DEC-099), vs `-o` (mutually exclusive in practice — `-o` wins in
  `build_sink` — but the AC should say so).
- **Two behaviour-preserving negatives that are easy to break**: `{ext}` templates (incl. the
  default) must be byte-identical to today on every verb, and an unrecognised / missing literal
  extension on a non-`build` verb or a plain `build` target (`{stem}`, `{stem}.txt`, row 18 — exit
  0 today) must keep working. Reusing `target_format_plan` as-is would turn row 18 into exit 4.
- **`build`'s cache key**: moving a plain-recipe target from `Preserve` to `Pinned` changes
  `target_recipe_hash` (`src/cli/build.rs:107-140`) — which the 0.8.0 single migration tolerates
  for templates whose bytes change, but a template that pins *the format it already produced*
  (`{stem}.jpg` on a JPEG source) would also re-key. Design must decide whether that is acceptable
  churn and pin it with a test either way (SPEC-129's AC-4 is the shape).
- **Two design calls** (web/optimize, meta lane) plus a doc reversal and a DEC.

That is a table-driven matrix over one rule, not an L-shaped one: every row asserts the same
predicate (sniffed format == the template's extension), so one parameterised integration test can
carry it. Splitting by verb would leave 0.8.0 with the surface disagreeing with itself, which is
the defect. **No split.**

### Scope

**In:**
- A recognised literal extension in `--name-template` pins the output format on every encoding
  verb above, at both arities.
- A plain-recipe `build` target's literal `name` extension pins the same way (the terminal-
  `optimize` target already does — keep it, regression-guard it).
- `web`, `optimize` and `apply --recipe web`: the design call on whether a template pin skips the
  auto-decision (evidence above; DEC-087 is the in-repo precedent).
- Precedence of the template vs `--format` / `recipe.format` / `-o`, matching the `-o` rung.
- What a non-recognised or absent literal extension, and `{name}`, do (recommendation: unchanged —
  names the file only; `{name}` is not a literal extension).
- The `meta` lane's answer (see questions).
- `docs/api-contract.md:490` reversed and `:565` reconciled; the new DEC (reserved DEC-105).
- `build`'s cache-key consequence, decided and pinned by a test.

**Out (and where each lives):**
- The `-o`/template pin silently moving `web` quality 85→80 and dropping the report — PROJ-013 /
  STAGE-037's `-o` item (cross-referenced there 2026-10-24). This spec states what the
  template-pinned stderr line is; it does not change pinned-path quality.
- `responsive` silently ignoring `--name-template` — **needs filing** (STAGE-050 or STAGE-037):
  "`responsive` accepts `--name-template` and ignores it; refuse it (exit 2) or honour it."
- A warning when `--format` and a template extension disagree, if design picks "`--format` wins"
  without one — follow-up, not this spec, unless design finds it free.
- `build` cleaning orphaned outputs when a target's extension changes — pre-existing, named in
  DEC-087 Consequences; unchanged.
- wasm (`src/wasm.rs`) — no output naming there; untouched.

### Rough build file list

- `src/cli/ops.rs` — `output_format_for` gains a template rung; the single/multi call sites in
  `run_pixel_op`; possibly the `meta` lane (`run_metadata_lane`) if design picks refuse.
- `src/cli/optimize.rs` — `run_apply` (single, multi, terminal-`optimize` pin), `run_optimize` and
  `run_web` pin checks; `run_convert` only if the `--format`-vs-template precedence differs from
  `output_format_for`'s.
- `src/cli/common.rs` — `build_sink` / `apply_one` / `encode_one` signature if the template has to
  reach `encode_one`.
- `src/cli/build.rs` — `prepare_target`'s plain-recipe branch; `target_format_plan` split into a
  strict (terminal-`optimize`, exit 4 on `{stem}.txt`) and a lenient form; `target_recipe_hash`
  tests.
- `src/sink/mod.rs` — a shared "literal extension of a template" helper (or reuse
  `format_from_extension` on the un-expanded template, as `build.rs:67` does).
- Tests: `tests/apply_batch.rs`, `tests/build.rs`, `tests/build_cache.rs`, `tests/optimize.rs`,
  `tests/sink.rs`, plus a new table-driven `tests/name_template_pins_format.rs`.
- Docs: `docs/api-contract.md` (`:490`, `:565`, the global `--format` help line),
  `src/cli/mod.rs:88` `--format` help text ("else inferred from `-o` extension or kept" — now also the
  template), `CHANGELOG.md`, `decisions/DEC-105-*.md` (new).

**Overlap:** `src/cli/ops.rs` / `src/cli/optimize.rs` / `src/cli/common.rs` are also SPEC-131's and
SPEC-133's likely territory if either routes errors through the CLI; SPEC-131's frame expects no
overlap with this spec unless it touches `common.rs`. **Recommend sequencing SPEC-134's build after
SPEC-133's** (both edit `run_pixel_op`'s neighbourhood) or confirming at design that SPEC-133
stays inside `src/operation/**`.

### Corrections to `prompts/SPEC-134-design.md`

1. **DEC-087 does not contain the rule being reversed.** DEC-087 says the opposite: "a `build`
   target's name template is its format pin" (terminal-`optimize` targets only). The "names the
   FILE, does not pin the FORMAT" sentence lives only in `docs/api-contract.md:490` (written by
   SPEC-126) and was mis-attributed to DEC-087 in SPEC-126's verify notes, SPEC-127, and the
   STAGE-050 entry; no DEC holds it (`grep` of `decisions/` for "does not pin" / "names the file":
   0 hits). So DEC-105 **extends** DEC-087's rule from one `build` plan to every verb, and reverses
   the api-contract paragraph — it is not a reversal of DEC-087. Setting
   `superseded_by: DEC-105` on DEC-087 would misdescribe it; design should pick the
   "amends/extends" form and say so.
2. The prompt's verb list (`ops.rs`, `common.rs`, `optimize.rs`, `build.rs`) misses that `meta`
   reads the template (`ops.rs:709`) and that `responsive` accepts but ignores it.
3. "What counts as recognised — reuse the exact set `-o` uses": correct, but note `-o` falls
   through silently on an unrecognised extension (`ops.rs:213-217`) while `build`'s
   terminal-`optimize` template errors (exit 4). Two existing answers; design picks per surface.

### Open maintainer questions

1. **`web` / `optimize` / `apply --recipe web` with a literal template: pin and skip the
   auto-decision?** Recommend **yes** — it is what `build` already does with the identical
   template (DEC-087), and the alternative keeps writing AVIF into `.webp`/`.png` files.
   (Ruling leaves this to design; asked here only because it changes `web`'s report.)
2. **`meta` with a template naming a different format** (`meta strip in.jpg --name-template
   '{stem}.png'`): it cannot pin without a pixel re-encode (`metadata-not-via-pixel-encode`).
   Recommend **refuse, exit 4, before writing** ("meta keeps the container; the template names
   .png but the input is JPEG"), rather than silently writing JPEG into `.png`.

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

- Pinned-path `web` quality (85→80) and its dropped report — PROJ-013 / STAGE-037's `-o` item.
  This spec only states what a template-pinned `web` prints.
- `responsive` accepting and ignoring `--name-template` — needs filing: "refuse it (exit 2) or
  honour it".
- A `--format`-vs-template disagreement warning — follow-up unless design finds it free.
- `build` removing orphaned outputs after an extension change — pre-existing (DEC-087
  Consequences), unchanged.
- `src/wasm.rs` — has no output naming; untouched.
- Any version bump or `CACHE_SCHEMA_VERSION` change — 0.8.0 is one release; a re-keyed build
  target rides it.

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
