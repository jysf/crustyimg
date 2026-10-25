---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-134
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
    - DEC-064
    - DEC-087
    - DEC-098
    - DEC-099
    - DEC-101
    - DEC-105
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
    - SPEC-132
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 9701360
      tokens_breakdown:
        input: 134
        output: 58395
        cache_creation: 176476
        cache_read: 9466355
      estimated_usd: 7.30
      duration_minutes: null
      recorded_at: 2026-10-25
      note: >
        Interactive design session in its own worktree (docs/spec-134-design). Measured from the
        session's transcript (010b22f1-….jsonl, identified by content), deduped by `.message.id`
        (67 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
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

### Maintainer rulings (2026-10-25) — both questions below are settled
1. **Pin it:** on `web`, `optimize` and `apply --recipe web`, a recognised template extension pins
   the format and skips the automatic choice, as `-o` and `build` (DEC-087) already do.
2. **`meta` refuses, exit 4, before writing**, when the template names a different format than
   the source; it says to convert first.

### Open maintainer questions (as asked at frame)

1. **`web` / `optimize` / `apply --recipe web` with a literal template: pin and skip the
   auto-decision?** Recommend **yes** — it is what `build` already does with the identical
   template (DEC-087), and the alternative keeps writing AVIF into `.webp`/`.png` files.
   (Ruling leaves this to design; asked here only because it changes `web`'s report.)
2. **`meta` with a template naming a different format** (`meta strip in.jpg --name-template
   '{stem}.png'`): it cannot pin without a pixel re-encode (`metadata-not-via-pixel-encode`).
   Recommend **refuse, exit 4, before writing** ("meta keeps the container; the template names
   .png but the input is JPEG"), rather than silently writing JPEG into `.png`.

## Inputs

- **Files to read:** `src/cli/ops.rs` (`output_format_for` :200-228, `run_pixel_op` :340-520,
  `run_metadata_lane` :658-800), `src/cli/optimize.rs` (`run_apply` :60-300, `run_convert` :540-595,
  `run_optimize` :670-720, `run_web` :815-860), `src/cli/common.rs` (`build_sink` :247,
  `apply_one` / `encode_one` :44-110), `src/cli/build.rs` (`target_format_plan` :63-69,
  `target_recipe_hash` :107-140, `prepare_target` :248-318), `src/sink/mod.rs`
  (`format_from_extension` :278, `expand_template` :364), `src/cli/mod.rs` (`GlobalArgs` :70-95,
  `CliError` :572, exit mapping :740-800).
- **External APIs:** none.
- **Related code paths:** `src/build/cache.rs` (`recipe_hash`, `compute_key`, `CACHE_SCHEMA_VERSION`
  — read, not changed), `src/build/lock.rs` (`LockOutput.key`).

## Outputs

- **Files created:** `tests/name_template_pins_format.rs` (the table test);
  `decisions/DEC-105-an-output-names-extension-pins-the-format-on-every-verb.md` (written at design).
- **Files modified:** see Implementation Context → "Files the build touches".
- **New exports:** `pub fn crate::sink::template_literal_format(template: &str) -> Option<ImageFormat>`;
  `pub(super) fn cli::common::name_pinned_format(global: &GlobalArgs) -> Option<ImageFormat>`;
  a new `CliError` variant for the `meta` refusal (exit 4). Names are suggestions; the behaviour
  is the contract.
- **Database changes:** none. No `CACHE_SCHEMA_VERSION` change, no lockfile schema change.

## Acceptance Criteria

Every AC names its verb surface (AGENTS §15). "JPEG source" is a photograph-like JPEG
(Notes → Fixture); a format is identified by **magic bytes**, never by file name.
"Template" is `--out-dir d --name-template T`. "N" is two inputs.

**Pin — the pixel verbs (`run_pixel_op`)**
- [ ] **AC-1** — `resize --max 64`, `thumbnail`, `auto-orient` and `watermark --text hi`, at 1 and
  at N inputs, and `edit --invert` (1 input; `edit` takes one), with T = `{stem}.png` on a JPEG
  source, write **PNG** bytes to `d/<stem>.png`, exit 0.

**Pin — `apply`, plain recipe**
- [ ] **AC-2** — `apply --recipe r.toml` (one `resize` step, no `format`) at 1 and N inputs with
  T = `{stem}.png` writes **PNG**, exit 0.

**Pin — the automatic-choice verbs (ruling 2)**
- [ ] **AC-3** — `apply --recipe web` at 1 and N inputs with T = `{stem}.png` writes **PNG** (not
  AVIF), exit 0.
- [ ] **AC-4** — `optimize` at 1 and N inputs with T = `{stem}.png` writes **PNG**, exit 0; and
  `optimize --profile preserve` (1 input) with T = `{stem}.png` writes **PNG** (as it does with
  `-o x.png` today).
- [ ] **AC-5** — `web` with T = `{stem}.webp` at 1 input writes **WEBP**; `web` with
  T = `{stem}.png` at N inputs writes **PNG**; exit 0 both.
- [ ] **AC-6** — **The report on a template pin**, defined per verb as the per-file stderr summary
  line (`<file>: jpeg → <fmt> · …`): on a template-pinned `web` (1 and N), `optimize` (1 and N)
  and `apply --recipe web` (1 and N), stderr contains **no** decision line (no `→`), exactly like
  the `-o`-pinned path today (driven: `web in.jpg -o C.png` prints nothing). And `--json` with a
  template pin is a **usage error, exit 2**, nothing written, with the existing message
  ("…unavailable with a pinned -o/--format…"; the build may add "or name template" to it) — on
  `optimize` (1 input), `web` (1 input) and `apply --recipe web` (1 input). Today all three exit 0.

**Precedence (match the `-o` rung)**
- [ ] **AC-7** — **`--format` beats the template**, silently, exit 0, on: `resize --format webp`
  (1 input), `apply --recipe r.toml --format webp` (N), and `convert --format webp` (1 and N —
  `convert` requires `--format`, so on `convert` the template can never pin). Output is **WEBP**
  in `d/<stem>.png`. This is today's `-o` answer (driven: `resize --format webp -o A.png` → WebP,
  exit 0).
- [ ] **AC-8** — **The template beats `recipe.format`**, on `apply --recipe r2.toml`
  (`version = "2"`, `format = "webp"`) at 1 and N inputs with T = `{stem}.png`: **PNG**. Matches
  `-o` (driven: `apply --recipe r2.toml in.jpg -o J2.png` → PNG) and `build`'s terminal-`optimize`
  ranking (`src/cli/build.rs:43-45`). Today the template row writes WEBP.
- [ ] **AC-9** — **`-o` beats the template** (the template is not the output's name, so it does
  not pin): `resize --max 64 in.jpg -o o.jpg --out-dir d --name-template '{stem}.png'` writes
  **JPEG** to `o.jpg` and creates nothing under `d`; `web in.jpg -o o.jpg` with the same template
  writes **JPEG** to `o.jpg`.

**Must not change**
- [ ] **AC-10** — **`{ext}` is byte-identical to the default.** With T = `{stem}.{ext}` the output
  bytes equal the no-template run's bytes, on `resize` (N), `apply --recipe r.toml` (N) and
  `meta strip` (1); and, with the AVIF encoder built, on `web` (N, T = `{stem}_w.{ext}`, output
  **AVIF**) and `apply --recipe web` (1, output **AVIF**, stderr has the `jpeg → avif` line), and
  `optimize --json` with T = `{stem}.{ext}` still exits 0.
- [ ] **AC-11** — **No recognised literal extension names the file only**, exit 0, on the
  non-`build` verbs: `resize` with T = `{stem}.txt` → JPEG in `in.txt`; `resize` with T = `{stem}`
  → JPEG in `in`; `resize --format png` with T = `{name}` → PNG in `in.jpg` (frame row 16);
  `resize` (1 input) with a template and **no** `--out-dir` writes JPEG to stdout (the template is
  unused); and, with the AVIF encoder built, `web` with T = `{stem}.txt` → AVIF in `in.txt` (as
  `web -o D.txt` does today).

**`meta` (ruling 3)**
- [ ] **AC-12** — `meta strip`, `meta clean --gps` and `meta set --artist x`, 1 input, JPEG
  source, T = `{stem}.png`: **exit 4**, **no file written** (`d` empty or absent), stderr names
  the input, the format the template names, and says to convert first.
- [ ] **AC-13** — `meta strip` at N inputs **refuses before writing anything**: two JPEGs with
  T = `{stem}.png` → exit 4, nothing written; a mixed batch (`a.png` + `in.jpg`, `a.png` listed
  **first**) with T = `{stem}.png` → exit 4 and `d/a.png` is **not** written either. (Exit 4,
  not 6: the batch is refused whole.)
- [ ] **AC-14** — `meta strip` with T = `{stem}.jpg` on a JPEG (same format) and with
  T = `{stem}_c.{ext}`: exit 0, JPEG, unchanged.

**`build`**
- [ ] **AC-15** — A plain-recipe `build` target (`recipe = "r.toml"`) with `name = "{stem}.png"`
  writes **PNG**; with `recipe = "r2.toml"` (`format = "webp"`) and `name = "{stem}.png"` writes
  **PNG** (template beats `recipe.format`, as on the terminal-`optimize` branch).
- [ ] **AC-16** — A plain-recipe target with `name = "{stem}"` and with `name = "{stem}.txt"` still
  **exits 0** and writes JPEG (frame row 18).
- [ ] **AC-17** — A terminal-`optimize` target (`recipe = "web"`) is unchanged: `name =
  "{stem}.png"` → PNG; `name = "{stem}.txt"` → **exit 4**, nothing written.
- [ ] **AC-18** — **Cache key.** One manifest, two plain targets sharing `r.toml` over the same
  source: `outA` with `name = "{stem}.png"`, `outB` with the default name. The lockfile's two
  `key`s **differ** (identical on `main` today — driven: both `65304eca…`), `outA/in.png` is PNG
  and `outB/in.jpg` is JPEG on a cold run **and** on a second, all-cached run; and `outB`'s key
  equals the key a manifest with only `outB` records (a `{ext}` target's key does not move).
  `CACHE_SCHEMA_VERSION` is unchanged (still `1`) — no test; verify checks it with
  `git diff main -- src/build/cache.rs` → empty.

**Helpers, docs, roster**
- [ ] **AC-19** — `sink::template_literal_format` returns the format for `{stem}.png`,
  `{stem}.PNG`, `{stem}.jpeg`, `{parent}_{stem}.webp`, `{name}.webp`, and `None` for
  `{stem}.{ext}`, `{stem}_w.{ext}`, `{stem}.{ext}.png` (contains `{ext}` — matches
  `target_format_plan`), `{stem}`, `{stem}.txt`, `{name}`. `common::name_pinned_format` returns
  the `-o` extension's format when `-o` is a path; `None` for `-o -`; `None` for an unrecognised
  `-o` even when a template is set; the template's format when `-o` is absent and `--out-dir` is
  set; `None` when neither `-o` nor `--out-dir` is set.
- [ ] **AC-20** — Docs say the new rule and nothing contradicts it: `docs/api-contract.md` :490
  (the "does **not** pin" sentence) replaced by the DEC-105 chain; :565-570 (the exit-4 sentence)
  scoped to a terminal-`optimize` target; the global-options `--format` row (:33);
  `docs/cli-reference.md` global table (:31); `docs/data-model.md` :286-291; the `--format` help
  text (`src/cli/mod.rs:88`); a `CHANGELOG.md` `[Unreleased]` entry (byte-changing; names the new
  exit 2 and the `meta` exit 4). Checked by verify with `grep -n 'does \*\*not\*\* pin'
  docs/api-contract.md` → 0 hits; no comment or user-facing string names a SPEC/DEC id.
- [ ] **AC-21** — The table test asserts its own row count (`ROWS.len() == 41`), and every roster
  surface in Implementation Context → "Roster" maps to at least one row or named test.

## Failing Tests

Written at design; the build makes them pass. **RED** = fails on `main` today (driven), **GUARD**
= green today, must stay green (a regression guard; its negative control is named below).

- **`tests/name_template_pins_format.rs`** (new)
  - `name_template_extension_pins_the_format_on_every_verb` — **RED.** One table of 41 rows,
    each `{ label, args, inputs, template, expect_exit, expect: [(path, Format)], expect_absent:
    [path], stderr_has: Option<&str>, stderr_lacks: Option<&str> }`. Runs every row, collects
    every failing row's label, then asserts the list is empty (so one run reports every RED row,
    not the first). Asserts `ROWS.len() == 41`. Sources: `in.jpg` / `in2.jpg` (see Notes →
    Fixture), `a.png` from `common::detailed_png`. Formats by magic
    bytes. Rows (RED ones marked ✗ — wrong today):
    - AC-1 (9): ✗ resize 1, ✗ resize N, ✗ thumbnail 1, ✗ thumbnail N, ✗ auto-orient 1,
      ✗ auto-orient N, ✗ watermark 1, ✗ watermark N, ✗ edit 1.
    - AC-2 (2): ✗ apply 1, ✗ apply N.
    - AC-3 (2): ✗ apply web 1, ✗ apply web N (each also `stderr_lacks: "→"`, AC-6).
    - AC-4 (3): ✗ optimize 1, ✗ optimize N (both `stderr_lacks: "→"`), ✗ optimize
      `--profile preserve` 1.
    - AC-5 (2): ✗ web 1 `.webp`, ✗ web N `.png` (both `stderr_lacks: "→"`).
    - AC-6 `--json` (3): ✗ optimize, ✗ web, ✗ apply web — exit 2, `expect_absent: d/in.png`.
    - AC-7 (4): resize `--format webp` 1, apply `--format webp` N, convert 1, convert N — WEBP.
      Green today (`--format` already wins); a guard on the new rung's position.
    - AC-8 (2): ✗ apply r2 1, ✗ apply r2 N — PNG (WEBP today).
    - AC-9 (2): resize `-o o.jpg` (+ `expect_absent: d`), web `-o o.jpg` — JPEG. Guard.
    - AC-11 (5, non-AVIF): resize `{stem}.txt`, resize `{stem}`, resize `--format png` `{name}`,
      resize 1 no `--out-dir` (stdout captured, checked as JPEG), resize N `{stem}.{ext}` → JPEG.
      Guard.
    - AC-12 (3): ✗ meta strip 1, ✗ meta clean --gps 1, ✗ meta set --artist 1 — exit 4,
      `expect_absent: d/in.png`, `stderr_has: "convert"`.
    - AC-13 (2): ✗ meta strip N (two JPEGs), ✗ meta strip N mixed (`a.png` first) — exit 4,
      `expect_absent: d/a.png, d/in.png`.
    - AC-14 (2): meta strip `{stem}.jpg`, meta strip `{stem}_c.{ext}` — exit 0, JPEG. Guard.
    - Total 9+2+2+3+2+3+4+2+2+5+3+2+2 = **41**.
  - `name_template_ext_placeholder_is_byte_identical_to_the_default` — **GUARD** (AC-10, ungated
    half): `resize` N, `apply --recipe r.toml` N, `meta strip` 1 — `{stem}.{ext}` bytes `==` the
    no-template run's bytes, compared with `assert_eq!` on the file contents.
  - `name_template_ext_placeholder_keeps_the_automatic_choice` — **GUARD**, gated
    `#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]` like
    `tests/build.rs:587` (AC-10 AVIF half, AC-11's `web {stem}.txt`): `web` N `{stem}_w.{ext}` →
    AVIF and bytes equal the no-template run; `apply --recipe web` 1 `{stem}.{ext}` → AVIF,
    stderr has `jpeg → avif`; `optimize --json` `{stem}.{ext}` → exit 0; `web` `{stem}.txt` →
    AVIF in `in.txt`, exit 0.
- **`tests/build.rs`**
  - `build_plain_target_literal_extension_pins_the_format` — **RED** (AC-15 first half): plain
    `r.toml`, `name = "{stem}.png"`, JPEG source → PNG. JPEG today (frame row 17).
  - `build_plain_target_template_beats_recipe_format` — **RED** (AC-15 second half): `r2.toml`
    (`format = "webp"`), `name = "{stem}.png"` → PNG. WEBP today (driven, `outD`).
  - `build_plain_target_without_a_recognised_extension_still_preserves` — **GUARD** (AC-16):
    `name = "{stem}"` and `name = "{stem}.txt"`, two targets → exit 0, JPEG in `out/in` and
    `out2/in.txt`.
  - `build_terminal_optimize_unrecognised_template_still_exits_4` — **GUARD** (AC-17 second
    half; only a unit test covers it today, `src/cli/build.rs:1205`): `recipe = "web"`,
    `name = "{stem}.txt"` → exit 4, `out` has no file. AC-17's first half is the existing
    `build_honours_a_literal_extension_template_as_a_format_pin` (`tests/build.rs:734`), kept.
- **`tests/build_cache.rs`**
  - `plain_targets_sharing_a_recipe_are_keyed_by_their_template_pin` — **RED** (AC-18): the
    two-target manifest above; asserts distinct lockfile keys (equal today), PNG/JPEG per output
    on the cold run and on a second `build` whose summary reports all outputs cached, and that
    `outB`'s key equals the key from a manifest holding `outB` alone.
- **`src/sink/mod.rs`** (`#[cfg(test)]`)
  - `template_literal_format_reads_only_a_recognised_literal_extension` — **RED** (does not
    compile: the function is new) (AC-19 first half).
- **`src/cli/common.rs`** (`#[cfg(test)]`)
  - `name_pinned_format_follows_the_name_the_output_is_written_under` — **RED** (new function)
    (AC-19 second half).
- **`src/cli/build.rs`** (`#[cfg(test)]`)
  - `plain_target_plan_pins_on_a_recognised_extension_only` — **RED** (new lenient plan function):
    `{stem}.png` → `Pinned(Png)`; `{stem}`, `{stem}.txt`, `{stem}.{ext}` → `Preserve`;
    `{stem}.txt` + `recipe.format = "webp"` → `Pinned(WebP)`; `{stem}.png` + `"webp"` →
    `Pinned(Png)`.
  - Existing `target_format_plan_reads_the_template` (strict, `{stem}.txt` errs) and
    `target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe` (the `Preserve` arm hashes
    exactly `recipe_hash`) stay green unchanged — together they pin "a `{ext}` plain target's key
    does not move".
- **`src/cli/mod.rs`** (`#[cfg(test)]`)
  - existing `exit_code_mapping_is_total` gains the new `meta` refusal variant → 4.
- **`src/cli/ops.rs`** (`#[cfg(test)]`)
  - existing `output_format_for` tests (:1465-1550) are updated for the new parameter and gain one
    case: a template rung below `--format`, above `recipe.format`, ignored when `output_path` is
    `Some`.

Count: **11 new tests** (the table + two `{ext}` guards; four in `tests/build.rs`; one in
`tests/build_cache.rs`; three unit) — **7 RED, 4 GUARD** — and **2 extended existing tests**
(`exit_code_mapping_is_total`, the `output_format_for` unit tests). The 41-row table is one test.

### Negative controls — one revert per independent condition (for verify)

| # | revert (one at a time) | goes RED | must stay green |
|---|---|---|---|
| NC-1 | drop the template rung from `output_format_for` | AC-1, AC-2 1-input, AC-4 (all three), AC-5, AC-8 1-input rows | AC-2/AC-8 N rows (separate site), meta, build |
| NC-2 | drop the template rung from `run_apply`'s N-input `format_override` | AC-2 N, AC-8 N | AC-2/AC-8 1-input |
| NC-3 | restore `run_web`'s `-o`-only pin check | AC-5, AC-6 web rows | optimize, apply web rows |
| NC-4 | restore `run_optimize`'s `-o`-only pin check | AC-4 1/N (they auto-decide again), AC-6 optimize `--json` | `--profile preserve` row (goes through `output_format_for`), web rows |
| NC-5 | restore `run_apply`'s terminal `cli_pin` to `-o`-only | AC-3, AC-6 apply-web `--json` | plain apply rows |
| NC-6 | delete the `meta` pre-flight | AC-12, AC-13 | AC-14 |
| NC-7 | move the `meta` check into the per-input write loop | the AC-13 **mixed** row only (`a.png` gets written) | AC-12, AC-13 two-JPEG row |
| NC-8 | restore `prepare_target`'s plain branch | both `build_plain_*` RED tests, the cache test | AC-16, AC-17 guards |
| NC-9 | make the plain-plan helper strict (error on no recognised extension) | AC-16 guard | AC-15, AC-11 (non-`build` verbs use the sink helper, not this one) |
| NC-10 | hash `Pinned` like `Preserve` in `target_recipe_hash` | the cache test (keys equal; `outB/in.jpg` served PNG bytes on the cached run) | AC-15 cold-run format rows |

The evidence is the test going RED against a rebuilt binary (AGENTS §15), not a changed hash.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Design calls (settled here)

1. **One rule, two small helpers, eight call sites.**
   - `crate::sink::template_literal_format(template) -> Option<ImageFormat>`: `None` if the
     template contains `{ext}`, else `format_from_extension(Path::new(template)).ok()`. Pure; lives
     next to `format_from_extension` in `src/sink/mod.rs`, which compiles on both targets (DEC-064)
     — nothing in wasm calls it.
   - `cli::common::name_pinned_format(global) -> Option<ImageFormat>`: the format the output's own
     name asks for, using the single-input sink's order (`build_sink`): `-o` given → `-o -` is
     `None`, else `format_from_extension(-o).ok()` (an unrecognised `-o` is `None`; the template is
     not consulted because it is not the output's name); else `--out-dir` given → the template's
     `template_literal_format`; else `None` (stdout).
   - Call sites: (a) `output_format_for` rung 2 gains the template: `-o` ext when `output_path`
     is `Some`, else the template the sink will use. Add a `template: Option<&str>` parameter
     (single-input callers pass the template only when the sink is `Dir`; the N-input caller passes
     `global.name_template`). Callers: (b) `run_pixel_op` single, (c) `run_pixel_op` N, (d)
     `run_apply` single. (e) `run_apply` N: `format_override = --format > template > recipe.format`.
     (f) `run_apply` terminal `cli_pin`, (g) `run_optimize`'s `pinned`, (h) `run_web`'s `pinned`:
     replace each inline `-o` check with `name_pinned_format(global)`. (i) `prepare_target`'s plain
     branch. (j) `run_metadata_lane`'s pre-flight. `run_optimize_autodecide`, `encode_one`,
     `apply_one`, `build_sink` and `run_convert` are **not** changed.
   - ⚠ On `web`/`optimize` the pinned branch calls `run_pixel_op` with `forced_format = None`, so
     the pin reaches the bytes through (a); the pin check only routes. Both must see the template
     or the row writes the source format through the pinned branch (NC-3/NC-4 separate them).
2. **`build`: a lenient plain plan, the strict terminal plan kept.** Do not reuse
   `target_format_plan` for plain targets (it errs on `{stem}` / `{stem}.txt`, AC-16). Add
   `plain_target_plan(template, recipe_format) -> Result<OutputFormatPlan, CliError>`:
   `template_literal_format(template)` → `Pinned`; else `recipe.format` → `Pinned`; else
   `Preserve`. Rewrite `target_format_plan` on top of `template_literal_format` so the "what is a
   literal extension" rule exists once: `{ext}` → `Decide`; a recognised literal → `Pinned`;
   otherwise the existing `UnsupportedExtension`/`UnknownFormat` error (exit 4) — keep the error
   type so its message is unchanged.
3. **Unrecognised or missing extension, per surface:** names the file only on every non-`build`
   verb (as `-o` does on those verbs, `src/cli/ops.rs:213-217`; driven `web -o D.txt` → AVIF,
   exit 0) and on a plain `build` target (AC-16); exit 4 on a `build` terminal-`optimize` target
   (DEC-087, AC-17). `{name}` and extensionless templates are unchanged everywhere.
4. **Precedence** — `--format` > output-name extension (`-o`, else the template) > `recipe.format`
   > the verb's default. Disagreement is silent, not an error; that is the `-o` precedent, driven
   (`resize --format webp -o A.png` → WebP). A warning for both `-o` and the template is a
   follow-up, not this spec.
5. **`meta` pre-flight (ruling 3).** In `run_metadata_lane`, before the arity split: if `-o` is
   absent, `--out-dir` is set and `template_literal_format` is `Some(fmt)`, sniff every resolved
   input's container with `::image::guess_format` on its bytes (stdin bytes are already buffered;
   for a path, a header read is enough), and if any input's sniff is not `Ok(fmt)` return the new
   error **before any write**. Compare formats, not extension strings (`.jpeg` vs `.jpg` is the
   same format). An unknown container counts as "different" — it cannot be proven to match. New
   `CliError` variant, mapped to exit **4** (`src/cli/mod.rs` exit table — same family as an
   unsupported format, `docs/api-contract.md:59`). Message shape, behaviour-first:
   `error: in.jpg: the name template names a PNG file, but meta keeps the input's own format
   (JPEG); convert it first, e.g. crustyimg convert --format png`. `meta copy` does not read the
   template and is untouched. `meta … -o x.png` is **not** covered (filed).
6. **`build`'s cache key.** A plain target that pins now takes the existing `Pinned` arm of
   `target_recipe_hash` — no new arm, no change to `target_recipe_hash` itself (plain targets
   with `recipe.format` already take it since SPEC-127). Its key changes; a `{ext}` plain target's
   does not (the `Preserve` arm is untouched). This is required for correctness (AC-18: on `main`
   the `{stem}.png` and default targets share one key, which after this change would serve PNG
   bytes into `in.jpg`). It rides 0.8.0's single lockfile migration: the key already folds in the
   crate version, so every 0.7.x lockfile drifts on 0.8.0 anyway (CHANGELOG `[Unreleased]`,
   "regenerate your `*.build.lock` once"). **No `CACHE_SCHEMA_VERSION` bump** — nothing about the
   entry format changes.
7. **The template-pinned report** is whatever the `-o`-pinned path prints today — nothing per
   file (driven). Pinned `web` quality (85 → 80) and the missing report are PROJ-013 / STAGE-037's
   `-o` item; this spec does not change them (AC-6 pins the current shape so that item changes it
   deliberately).
8. **DEC-105 is written at design** (`decisions/DEC-105-…`), `supersedes: null`: it **extends**
   DEC-087, it does not replace it. DEC-087's References already carry an "Extended
   by DEC-105" line (added at design). The build adds any path it touches that DEC-105's
   `affected_scope` misses. The api-contract sentence being retired was held by no DEC (DEC-105 Context).

### Roster — every surface that reads the template, and its coverage

| surface | code | AC | rows / tests |
|---|---|---|---|
| `resize`, `thumbnail`, `auto-orient`, `watermark` (1, N), `edit` (1) | `run_pixel_op` single/N | AC-1 | 9 rows |
| `convert` (1, N) | `run_pixel_op`, `forced_format` | AC-7 | 2 rows (`--format` wins) |
| `apply`, plain (1, N) | `run_apply` | AC-2, AC-7, AC-8 | 5 rows |
| `apply --recipe web` (1, N) | `run_apply` terminal pin | AC-3, AC-6, AC-10 | 3 rows + gated guard |
| `optimize` (1, N, `--profile preserve`) | `run_optimize` | AC-4, AC-6, AC-10 | 4 rows + gated guard |
| `web` (1, N) | `run_web` | AC-5, AC-6, AC-9, AC-10, AC-11 | 4 rows + gated guard |
| `meta strip` / `clean` / `set` | `run_metadata_lane` | AC-12–14 | 7 rows + `{ext}` guard |
| `build`, plain target | `prepare_target` plain | AC-15, AC-16, AC-18 | 3 + 1 tests |
| `build`, terminal-`optimize` target | `prepare_target` terminal | AC-17 | 1 new + 1 existing |
| `meta copy` | does not read the template | — | — |
| `responsive` | accepts and ignores the template | out of scope | filed (STAGE-050) |

Nine encoding verbs (`resize`, `thumbnail`, `auto-orient`, `watermark`, `edit`, `convert`,
`apply`, `optimize`, `web`) × {1, N} = 18, minus `edit` N (takes one input) = 17 pin/precedence
rows, + 2 `apply --recipe web` rows + 1 `--profile preserve` row = the 20 encoding rows; the other
21 are precedence, `--json`, must-not-change and `meta`.

### Files the build touches (for the orchestrator's overlap check)

- `src/sink/mod.rs` — new `template_literal_format` + unit test.
- `src/cli/common.rs` — new `name_pinned_format` + unit test. **Not** `encode_one`, `apply_one`,
  `write_encoded`, `build_sink`.
- `src/cli/ops.rs` — `output_format_for` (signature + rung), `run_pixel_op` (the two
  `output_format_for` calls only, ~:377 and ~:475), `run_metadata_lane` (pre-flight before the
  arity split), its unit tests.
- `src/cli/optimize.rs` — `run_apply` (terminal `cli_pin` ~:87-94, single-input
  `output_format_for` call ~:177, N-input `format_override` ~:212-215), `run_optimize` (`pinned`
  ~:691-695, plus the `--json` message text if widened), `run_web` (`pinned` ~:830-834). **Not**
  `run_optimize_autodecide`, `run_convert`, `optimize_decide_one`.
- `src/cli/build.rs` — `target_format_plan` (rewritten on the sink helper), new
  `plain_target_plan`, `prepare_target` (plain branch ~:283-289 only), unit tests. **Not**
  `watch_impl`, `target_recipe_hash`, `build_one`, the lockfile phase.
- `src/cli/mod.rs` — `--format` help text (:88), new `CliError` variant + `Display`, exit
  mapping, `exit_code_mapping_is_total`.
- Tests: new `tests/name_template_pins_format.rs`; `tests/build.rs` (+4), `tests/build_cache.rs`
  (+1). (The frame's `tests/apply_batch.rs`, `tests/optimize.rs`, `tests/sink.rs` are **not**
  needed: the table carries those rows. Touch them only if an existing assertion encodes the old
  behaviour — none found by `grep -n "name-template" tests/*.rs` at design beyond `{ext}` uses.)
- Docs: `docs/api-contract.md` (:33, :490, :565-570), `docs/cli-reference.md` (:31),
  `docs/data-model.md` (:286-291), `CHANGELOG.md` (`[Unreleased]`), `decisions/DEC-105-…` (scope fix-ups
  only).

**Overlap.** SPEC-133 (watermark damage rule) shares `src/cli/{ops,optimize,common,build,mod}.rs`.
Functions both touch: `run_pixel_op` (this spec: two format-resolution lines; SPEC-133: the
per-input outcome) and `run_apply` (this spec: three format lines; SPEC-133: single + batch
outcome). Disjoint in `common.rs` (SPEC-133 edits `encode_one`; this spec adds a new function),
`build.rs` (SPEC-133: `build_one`, summary, lockfile phase; this spec: `prepare_target`'s plain
branch and plan helpers) and `mod.rs` (each adds its own variant/mapping — a trivial merge).
**Recommend SPEC-134 builds first:** its edits are ~20 lines inside those functions, so SPEC-133's
rebase over them is small; the reverse order rebases this spec over SPEC-133's larger outcome
changes. SPEC-131 stays out of `src/cli/**` — parallel-safe; shared by append only
(`CHANGELOG.md`, `docs/api-contract.md`). SPEC-132 edits `watch_impl` in `build.rs`; this spec
does not touch `watch_impl`.

### Decisions that apply

- `DEC-087` — the rule this spec extends; its strict terminal-`optimize` plan stays.
- `DEC-105` — this spec's rule (written at design).
- `DEC-099` — the precedence chain the template rung slots into (`-o`'s rung, above `recipe.format`).
- `DEC-098` — `apply` resolves format like the pixel verbs at both arities; resolution stays at
  the call site, never inside `encode_one`.
- `DEC-015` — preserve-source default; exit codes (4 for the `meta` refusal; 6 is not used for it).
- `DEC-048` — `--profile preserve` is the engine-off anchor; with a literal template it pins, as
  with `-o`.
- `DEC-058` / `DEC-101` — cache key composition; the plan already folds in via
  `target_recipe_hash`; no schema bump.
- `DEC-064` — `src/sink` compiles on both targets; the new helper is pure.

### Constraints that apply

- `metadata-not-via-pixel-encode` — why `meta` refuses rather than converts.
- `ergonomic-defaults` — the default `{stem}.{ext}` is untouched.
- `test-before-implementation` — the Failing Tests above.
- `one-spec-per-pr`.

### Prior related work

- `SPEC-111` (shipped) — DEC-087; `target_format_plan`, `target_recipe_hash`'s plan fold.
- `SPEC-126` (shipped) — `apply` single input reuses `output_format_for`; wrote the retired
  api-contract sentence.
- `SPEC-127` (shipped) — `recipe.format` / DEC-099; plain targets with `format` already `Pinned`.
- `SPEC-129` (shipped) — the cache-key test shape (`tests/build_cache.rs`, AC-4 there).

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- Pinned-path `web` quality (85→80) and its dropped report — PROJ-013 / STAGE-037's `-o` item.
  This spec only pins what a template-pinned `web` prints today (AC-6).
- `responsive` accepting and ignoring `--name-template` — filed in STAGE-050 at design.
- `meta … -o x.png` writing the source container into `x.png` — filed in STAGE-050 at design
  (driven: `meta strip in.jpg -o E.png` → JPEG, exit 0).
- A warning when `--format` and an output-name extension disagree (both `-o` and the template).
- `build` removing orphaned outputs after an extension change — pre-existing (DEC-087).
- `src/wasm.rs` — no output naming; untouched.
- `-o` given together with multiple inputs (ignored today in favour of `--out-dir`) — unchanged.
- Any version bump or `CACHE_SCHEMA_VERSION` change.

## Notes for the Implementer

- Write the table so a failing row prints its label, args, exit, stderr and each file's sniffed
  format; collect all failures before asserting.
- **Fixture.** The design drove `tests/fixtures/classify/color_photo_fuji.png` re-encoded to JPEG
  (auto choice → AVIF, `jpeg → avif · ssim 81.5`). Build the test's JPEG from that PNG with the
  `image` crate's encoder (not with crustyimg — a fixture made by the code under test cannot fail),
  or use `common::detailed_jpeg` if it also auto-decides to AVIF. The pin rows assert PNG/WEBP, so
  they are RED today whatever the automatic choice is; only the gated guard depends on AVIF, and it
  asserts the no-template control is AVIF first, which proves the fixture. Default features are
  `display, watch, avif` (no `webp-lossy`), so the gated test runs under plain `cargo test` —
  confirm it is listed as run, not filtered.
- `{stem}.avif` on a build without the AVIF encoder fails the way `--format avif` does on that verb
  and arity; no AC, do not special-case it.
- The `--json` refusal message (`optimize.rs`, "unavailable with a pinned -o/--format…") may gain
  "or name template"; tests match on "unavailable", not the full string.
- Comments and user-facing text read behaviour-first, with no SPEC/DEC ids.

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
