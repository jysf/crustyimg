---
stage:
  id: STAGE-054
  status: active
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-27
shipped_at: null

value_contribution:
  advances: >
    The predictability half of the thesis: what STAGE-050 made a recipe able to
    express now behaves the same on every path, and says so when it cannot.
  delivers:
    - "A `watermark` size or scale that is not a sensible number is refused on the CLI, as it already is in a recipe"
    - "A cached `build` reports the same per-input warnings as the build that produced it"
    - "One rule for how per-input lines name their input, and for which verbs honour `--name-template`"
  explicitly_does_not:
    - "Add new verbs or new recipe operations"
    - "Change which candidate `optimize` picks"
---

# STAGE-054: Recipe Reach Follow-Through

## What This Stage Is

The follow-up work STAGE-050 turned up while it shipped. A recipe can now watermark, pin a
format and validate its parameters; this stage closes the gaps around that: the CLI path that
still draws a nonsense mark at exit 0, a cached `build` that forgets its warnings, the value
half of the watermark ruling (`--min-size`), and the verbs and lines that still disagree with
each other. Every item was filed by a STAGE-050 spec and moved here at its Stage Ship
(2026-10-27).

## Why Now

STAGE-050's Stage Ship judged its criterion 4 ("a bad watermark is refused or skipped, never
silently dropped") only **partly** met: `watermark --size nan` still writes an unmarked image at
exit 0. 0.8.0 headlines that rule, so the maintainer ruled (2026-10-27) that the first item
ships **before 0.8.0**. The rest has no release deadline and must not hold the tag.

## Success Criteria

- `watermark --size nan|inf` and `--scale nan|inf` are refused with a plain message, on one
  input and in a batch; no CLI or recipe path draws an invisible mark at exit 0.
- A text mark's `size` has an upper bound, the same on the CLI and in a recipe.
- A second `build` over unchanged inputs prints the warnings the first one printed.

## Scope

### In scope
- The items in the Spec Backlog below.

### Explicitly out of scope
- SPEC-133's optional test gaps: moved to STAGE-053 (guards, not behaviour).
- Anything in PROJ-013. It is `proposed`, so `just backlog` cannot see it (HANDOFF trap 8).

## Spec Backlog

Ordered. ⚡ Only the first item gates 0.8.0.

- [ ] (not yet written) — [S] ⚡ **Before 0.8.0: a `watermark` size or scale that is not a
  sensible number is refused, never silently drawn as nothing.** One spec for two filings that
  share the same validation code on the same verb (maintainer ruling 2026-10-27, after STAGE-050's
  Stage Ship judged criterion 4 only *partly* met because of the first one). Scope: refuse
  non-finite `--size` and `--scale` on the CLI verb (the recipe path already does, SPEC-131); put
  an upper bound on a text mark's `size`, on the CLI and in the recipe schema; and replace
  `--scale inf`'s `4294967295×4294967295 px mark` message. The two filings, moved verbatim:
  - [S] ⚡ **The `watermark` CLI verb silently drops the mark for
    `--size nan` / `--size inf`, and accepts `--scale nan`.** Filed by SPEC-131's verify
    (2026-10-26), driven on release builds before and after SPEC-131 (identical): `watermark
    --size nan` exits 0 and writes output byte-equal to `--opacity 0`. The CLI twin of what SPEC-131
    fixed for recipes; the CLI builds the op with `new_image`/`new_text`, bypassing the recipe
    schema. ⚠ **Check against SPEC-133 first:** its fit test may already refuse a non-finite mark
    size; if so, this closes with SPEC-133 and needs only a test row.
    **Checked 2026-10-26 (SPEC-133 build + verify): NOT closed.** `--size nan`, `--size inf` and
    `--scale nan` still exit 0 (the mark renders tiny and fits); only `--scale inf` is now refused,
    and its message reads `the 4294967295×4294967295 px mark` — fix that wording here too.
  - [S] **No upper bound on a text watermark's `size`.** Filed by SPEC-131's
    design (2026-10-24): `size = 100000.0` in a recipe is accepted and took 52 s of wall time over 3
    small inputs at exit 0 (the text raster is enormous); `watermark --size` shares it. A
    resource-limit question in DEC-036's family, not a malformed-recipe one, so SPEC-131 refuses
    only `size <= 0` and non-finite values. Open: the cap, and whether it lives in the schema.

- [ ] (not yet written) — [S] **A cached `build` replays no per-input warnings.** Filed by
  SPEC-133's design (2026-10-25, Call 4). A cache hit writes the stored bytes and returns before the
  pipeline (`src/cli/build.rs:531-540`), so three warnings print on the build that produced an
  output and never on a cached re-run: SPEC-133's "watermark skipped, image written without it",
  DEC-085's truncated JPEG, and SPEC-119's animated input (the last two only on the Decide plan's
  rebuild path today — the Preserve/Pinned plans never print them at all, `encode_one` drops the
  flags). The output stays correct and locked; what is lost is the *report*: a second `build`
  says `(N cached, 0 rebuilt)` over images that are unmarked or truncated. Fix shape: the cache
  entry carries the output's warnings (frame field after `ext`, `src/build/cache.rs:455-503`),
  `CACHE_SCHEMA_VERSION` 1 → 2 + new magic, and `build_one` replays them on a hit. A schema bump
  is only a one-time rebuild, so this is not tied to 0.8.0's lockfile migration. SPEC-133 ships
  the limitation documented (`docs/api-contract.md`, `build`) with a test pinning it, which this
  item flips on purpose.

- [ ] (not yet written) — [S] **`watermark --min-size`: the opt-in value rule, settable in a
  recipe.** Split out of SPEC-133 by its frame (2026-10-24); the orchestrator confirms the split.
  It is the second half of the 2026-08-23 ruling above ("Value rule — opt-in `--min-size`,
  settable in a recipe … only the user knows what is worth protecting"), which is settled; it is
  not a new idea. Why separate: it adds a CLI flag on `watermark` and a recipe key on the step, so
  it couples to SPEC-131's strict param schema (land it after SPEC-131 and the key goes straight
  into the schema), and it needs its own single-input rule, which the ruling does not state (below
  the floor on one explicit input: error like the damage rule, or write unmarked?). It reuses
  SPEC-133's batch *skipped* outcome, so build it after SPEC-133. Open for its spec: what "size"
  means (shorter edge? longer edge? area?) and whether `apply --recipe` gets a CLI override.

- [ ] (not yet written) — [S] **`responsive` accepts `--name-template` and silently ignores it.**
  Filed by SPEC-134's design (2026-10-25), driven on `main`: `responsive in.jpg --widths 32
  --out-dir rp --name-template '{stem}.png'` writes `rp/in-32w.jpg`, exit 0 — the flag is a global
  arg (`src/cli/mod.rs:80-82`) that `responsive` never reads. Open: refuse it (exit 2, the flag
  means nothing here) or honour it (it would need a `{w}` token). SPEC-134 makes the template pin
  the format on every other encoding verb, so `responsive` is now the one verb where it does
  nothing.

- [ ] (not yet written) — [S] **`watermark` prints absolute paths in batch lines; `apply` prints
  relative ones.** Filed by SPEC-133's re-verify (2026-10-26): `watermark two/` labels its skip
  warnings with absolute paths, `apply --recipe … two/` with the path as given. Pre-existing
  (identical at `3dd23ff`; `src/cli/ops.rs` unchanged by SPEC-133). Pick one labelling rule for
  every per-input line and use it in `run_pixel_op` too; check the error lines as well as the
  skip lines.

- [ ] (not yet written) — [S] **wasm `validate(recipe_toml)` export returning structured,
  per-step recipe errors.** Filed by SPEC-131's design (2026-10-24). Roadmap row #6's (recipe
  playground) prerequisite: no wasm export checks a recipe without an image today, and
  `transform` decodes its input before it builds the pipeline. Consumes SPEC-131's
  `OperationRegistry::param_specs` and its `recipe step N ('op'): …` errors; the natural home for
  "did you mean …?" key suggestions, which SPEC-131 declined.

- [ ] (not yet written) — [S] **Encoding verbs resolve their inputs twice.** Filed by SPEC-134's
  re-verify (2026-10-26). To decide the format pin before branching, `web`, `optimize` and
  `apply --recipe` call `resolved_input_count` (`src/cli/common.rs`), which runs `source::resolve`
  on every argument; the chosen branch (`run_pixel_op` or `run_optimize_autodecide`) then resolves
  the same `&[String]` again. Same function both times and stdin is never drained by the count, so
  it is correct today; the cost is a second directory walk and two passes that could drift.
  Refactor: resolve once and pass `&[Input]` into both branches. Code quality, not a bug; a
  candidate for the maintainer's queued Rust cleanup.

**Count:** 0 shipped / 0 active / 7 pending

## Design Notes

- The first spec's refusal must reuse SPEC-131's recipe validation (`param_specs`), not add a
  second copy for the CLI, so the two paths cannot drift again.

## Dependencies

### Depends on
- STAGE-050 (shipped 2026-10-27): the watermark op, SPEC-131's param schema, SPEC-133's
  refusal and skip.

### Enables
- 0.8.0's release prep, once the first item ships.
- Roadmap row #6, the recipe playground, through the wasm `validate` export.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
