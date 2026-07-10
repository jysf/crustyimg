---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-065
  type: story
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # a prepare-phase collision check + typed error + exit-code map; no new dep, no lockfile yet

project:
  id: PROJ-007
  stage: STAGE-022
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-09

references:
  decisions: [DEC-057, DEC-007, DEC-015, DEC-035]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-063, SPEC-064]

value_link: "STAGE-022's prerequisite — a lockfile can only pin a build whose source→output mapping is a function."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-09
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Grounded in a firsthand read of the shipped post-cache executor (run_build / prepare_target /
        build_one / cache_key_for) + DEC-057's injective-constraint section + sink::expand_template.
        No new dep; no lockfile in this spec (that's SPEC-066).
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 115000
      estimated_usd: 2.10
      duration_minutes: 22
      recorded_at: 2026-07-09
      notes: >
        Build cycle — main-loop, not separately metered, so tokens/usd are order-of-magnitude
        ESTIMATES (AGENTS §4). One session: read the seam, added the pure detector + the
        ext-normalizing key, wired the global prepare-phase check, made all 9 failing tests pass.
        No new dep, no new DEC. Gates: 637 tests default + 637 lean, clippy ×2, fmt, deny green,
        Cargo.toml untouched; four collision scenarios driven on the real binary.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 90000
      estimated_usd: 1.65
      duration_minutes: 18
      recorded_at: 2026-07-09
      notes: >
        Verify cycle — fresh session per AGENTS §15, re-derived from the spec + the diff.
        Main-loop, not separately metered → order-of-magnitude ESTIMATES (AGENTS §4).
        Gates from clean: 637 tests default + 637 lean, clippy ×2, fmt, `just deny` green,
        `git diff main -- Cargo.toml Cargo.lock` empty. All 9 spec-named Failing Tests present
        and running (3 build units + 2 cli units + 4 integration; +2 collision-key units beyond
        the spec's list). Four collision scenarios re-driven on the release binary, incl. the
        `dist` vs `./dist/` two-spellings cross-target case; the disclosed literal-ext residual
        reproduced and accepted (carried to DEC-059). DEC-057 confirmed RESOLVED; no new dep,
        no new DEC; cache store untouched. One non-blocking note: `exit_code_mapping_is_total`
        still omits `CliError::Cache` (pre-existing, SPEC-064) despite the build note's claim.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-09
      notes: >
        Ship bookkeeping (squash-merge PR #71 → main bc13c4d; re-apply the verify cost session +
        timeline verify mark on main via stash-pop after merge, §13; ship reflection; cost.totals;
        timeline ship mark; STAGE-022 backlog + brief; archive to done/; `just cost-audit`; carry the
        two verify follow-ups — the `CliError::Cache` exit-map gap and the literal-ext residual) —
        main-loop, not separately metered → null-with-note per AGENTS §4.
  totals:
    tokens_total: 205000
    estimated_usd: 3.75
    session_count: 4
---

# SPEC-065: the injective source→output guarantee

## Context

DEC-057 (SPEC-063) recorded a reproducibility hazard as **STAGE-022's blocker**: a build
target does not guarantee that its expanded output paths are unique. Two inputs sharing a
stem in one target (`a/logo.png` + `b/logo.png` under `{stem}.{ext}`) map to the **same**
output path; with `Overwrite::Allow` and the rayon fan-out they *race* — the winner is
nondeterministic and the summary over-counts (exit 0, "2 outputs", one file). STAGE-021's
cache did not change this: it keys on output-byte identity, not path (DEC-058), so the
collision still races at the destination.

STAGE-022 commits a **lockfile that pins `source → output`**. That is meaningless if the
mapping isn't a function — you cannot pin an output path that two inputs fight over. So this
spec is the stage's prerequisite: **reject a non-injective build before any output is
written.** It is small, has its own failing test, and is independent of the lockfile format
(SPEC-066), so it ships first and unblocks the rest of the stage. See the parent
`STAGE-022-reproducibility-lockfile.md` for the framing.

## Goal

At `run_build`'s prepare phase — after all targets' sources are resolved, before any output
is written and before the cache store is opened — reject a build whose resolved targets would
write **two inputs to the same output path**, with a typed error naming the collision and its
two sources (exit 2). A non-colliding build is unaffected.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_build` (~L1294, the two-phase executor; the check inserts between
    phase 1 "prepare all targets" and phase 2 "execute" / `Cache::open`), `prepare_target` /
    `PreparedTarget` (~L1151/1097; carries `target` + resolved `inputs`), `build_one` (~L1231,
    for how `out`/`template`/`Overwrite::Allow` are used to place an output), the `CliError`
    enum + `code()` map (~L460/543) and `exit_code_mapping_is_total` test (~L4037).
  - `src/sink/mod.rs` — `expand_template` (~L269; `{stem}`/`{ext}`/`{name}`/`{parent}`) — reuse
    it to compute each input's output file name; `SinkInput`, `safe_join` for context.
  - `src/source/mod.rs` — `Input::stem` / `Input::path` (the naming context per input).
  - `src/build/mod.rs` — where the pure collision-detection helper + its typed error land.
  - `decisions/DEC-057-*.md` — the "Injective source→output constraint" section this discharges.
- **External APIs:** none. **No new dependency.**
- **Related code paths:** `tests/build.rs` / `tests/build_cache.rs` for the integration-test shape.

## Outputs

- **Files created:** none (extends existing modules).
- **Files modified:**
  - `src/build/mod.rs` — a **pure, library-first** collision detector + its typed error:
    - `pub struct OutputCollision { pub output: String, pub first: String, pub second: String }`
      (the shared output path + the two source labels).
    - `pub fn find_output_collision(entries: &[(String, String)]) -> Option<OutputCollision>` —
      given `(collision_key, source_label)` pairs, return the first pair whose `collision_key`
      duplicates an earlier one. Order-preserving, deterministic. Unit-tested here.
  - `src/cli/mod.rs`:
    - a helper that, after phase 1, builds the `(collision_key, source_label)` list across **all**
      prepared targets and calls `find_output_collision`; on `Some`, returns a new
      `CliError::OutputCollision { output, first, second }` **before** phase 2 / `Cache::open`.
    - `CliError::OutputCollision { output: String, first: String, second: String }` (thiserror)
      → **exit 2** in `code()`; add the arm to `exit_code_mapping_is_total`.
  - `decisions/DEC-057-*.md` — mark the injective constraint **RESOLVED by SPEC-065** in its
    Validation section (the over-detection approach + tradeoff). **No new DEC** — this discharges a
    constraint DEC-057 already named; DEC-059 stays reserved for the lockfile (SPEC-066).
- **New exports:** `crustyimg::build::{OutputCollision, find_output_collision}`.

## Acceptance Criteria

- [ ] A build whose resolved targets write two inputs to the same output path is **rejected with
  a typed `CliError::OutputCollision`** (exit 2) naming the shared output + both sources.
- [ ] The rejection happens **before any output is written** and before `.crustyimg/` is created —
  the out dir is empty/absent after the failed build (fail-before-write, like a bad recipe).
- [ ] The check is **global across all targets** — two *different* targets writing the same
  `out`/name also collide and are rejected.
- [ ] The collision key is computed **without decoding** (prepare-phase) and is **conservative on
  `{ext}`**: two inputs whose `{stem}`/`{name}`/`{parent}` expansions match collide regardless of
  the (unknowable-pre-decode) output extension — so a real format-transforming collision
  (`a/logo.png` + `b/logo.svg` both → `logo.png`) is caught, at the cost of rejecting the rare
  "same stem, genuinely different output ext" build.
- [ ] A **non-colliding** build is unaffected (exit 0, all outputs written); a template that
  disambiguates (`{parent}_{stem}.{ext}`) turns a would-be collision into a clean build.
- [ ] `find_output_collision` is a pure, deterministic, order-preserving library fn with unit tests;
  no `unwrap`/`expect` on recoverable paths; clippy + fmt clean; **no new dependency**; lean build
  and `just deny` unaffected.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/build/mod.rs`** (in the `#[cfg(test)] mod tests`)
  - `"detects_first_duplicate_collision_key"` — entries with a repeated key → `Some(OutputCollision)`
    naming the two source labels + the shared output; the *first* offending pair, deterministically.
  - `"no_collision_when_all_keys_distinct"` — all-distinct keys → `None`.
  - `"collision_is_order_preserving"` — the reported `first` is the earlier source, `second` the later.
- **`src/cli/mod.rs`** (unit, for the exit-code map)
  - `"output_collision_maps_to_exit_2"` — `CliError::OutputCollision{..}.code() == 2`; and
    `exit_code_mapping_is_total` still covers every variant.
- **`tests/build_injective.rs`** (integration, drive the binary; generate PNG fixtures natively)
  - `"colliding_stems_in_one_target_are_rejected"` — a temp project with `a/logo.png` + `b/logo.png`,
    a target `source = ["a/*.png","b/*.png"]`, recipe, `out`, `name = "{stem}.{ext}"` → `crustyimg
    build` exits 2, stderr names the collision, and **no** files are written to `out` (and no
    `.crustyimg/`).
  - `"disambiguating_template_builds_cleanly"` — the same project with `name = "{parent}_{stem}.{ext}"`
    → exit 0, both `a_logo.png` + `b_logo.png` written.
  - `"collision_across_two_targets_is_rejected"` — two targets both writing `out = "dist"` with the
    same `name` over inputs that share a stem → exit 2, before any write.
  - `"non_colliding_build_unaffected"` — a plain multi-input build with distinct stems → exit 0, all
    outputs present (regression guard that the check adds no false positives).

## Implementation Context

*Read this section (and the files it points to) before starting build. The seam was read
firsthand during design against the current post-cache tree — re-confirm signatures.*

### Decisions that apply
- `DEC-057` — the build executor + manifest; its **Injective source→output constraint** section is
  exactly this hazard. The fix lands where DEC-057 said: "reject duplicate expanded output paths in
  `prepare_target` (detect the collision at prepare time, before executing)." Mark it RESOLVED there.
- `DEC-007` — typed `thiserror`; the collision is a typed `CliError` mapped to an exit code only at
  the CLI boundary. Keep the *detector* pure in `src/build` (library-first, unit-tested).
- `DEC-015` — partial-batch (exit 6) is a per-*output execution* failure; a collision is a
  **config** error detected before execution → exit 2, not 6. Don't conflate them.
- `DEC-035` — the sink already blocks name-template path escapes / symlinked destinations; this spec
  adds *uniqueness*, orthogonal to those guards.

### Where it goes (mirror the two-phase executor)
`run_build` already prepares ALL targets (phase 1) before executing ANY (phase 2), and opens the
cache store between them. Insert the injective check **at the end of phase 1, before `Cache::open`
and before phase 2**, so a collision aborts having written nothing and created no `.crustyimg/`:

```
// phase 1: prepare all targets  (unchanged)
let prepared = ...;
// NEW: injective check across all prepared targets
check_output_injective(&prepared)?;      // -> CliError::OutputCollision (exit 2)
// open cache store, then phase 2: execute  (unchanged)
```

### Computing the collision key (the one subtlety — conservative, pre-decode)
For each input of each prepared target, the collision key is the target's `out` dir joined to the
template expanded over the input's `{stem}`/`{name}`/`{parent}`, with **`{ext}` normalized to a
fixed sentinel** (because the real output ext needs a decode — DEC-058 stores it in the cache entry
for exactly this reason — and this check runs pre-decode). Reuse `sink::expand_template(template,
stem, SENTINEL_EXT, path)` with a sentinel like `"\u{0}ext\u{0}"` that can't occur in a real ext,
then prefix the `out` dir string. Normalize the `out` dir consistently (same string form for the
same directory) so two targets writing the same dir compare equal — a plain `Path`-based
normalization is enough; do **not** `canonicalize` (the dirs may not exist yet). The `source_label`
is the input's path display (or stem for the unreachable stdin case).

**Why conservative on ext (over-detect), not a proxy (under-detect):** ignoring ext rejects the rare
"same stem → different output ext" build (a false positive the user fixes by disambiguating the
template). Using the *input* extension as an ext proxy would **miss** a real collision when two
inputs transform to the same output format (`a/logo.png` + `b/logo.svg` → both `logo.png` via
SVG→PNG, DEC-054) — silent, and the exact failure the lockfile can't tolerate. Over-detection is the
safe direction; SPEC-065 takes it. A format-sniff refinement to cut false positives is a documented
future option, not this spec.

### Constraints that apply
- `untrusted-input-hardening` (the check is over resolved config; no new untrusted surface),
  `no-unwrap-on-recoverable-paths`, `every-public-fn-tested` (the pure detector + the ext-normalizing
  key), `clippy-fmt-clean`, `ergonomic-defaults` (no new flags; the check is always on — a build that
  can't be pinned shouldn't silently race).

### Prior related work
- `SPEC-063` (DEC-057) — the executor + the recorded blocker this discharges.
- `SPEC-064` (DEC-058) — the cache; why the output ext is a post-decode / entry-stored quantity
  (the reason the check must be conservative on `{ext}`).

### Out of scope (for this spec specifically)
- The lockfile / `--check` / `--frozen` (SPEC-066, DEC-059); `--watch` (STAGE-023); a format-sniff to
  narrow the conservative ext over-detection; per-target format overrides; the STAGE-021
  `CACHE_ENTRY_MAX_BYTES` off-by-53 fix. Do not touch the cache key/store (DEC-058).

## Notes for the Implementer

- Keep `find_output_collision` a **pure** fn in `src/build` (library, unit-tested); build the
  `(key, label)` list in `cli` where the resolved `PreparedTarget`s live. Don't reach into the sink
  for anything but `expand_template`.
- Run the check **once, globally**, after phase 1 — not per target — so cross-target collisions are
  caught and the "before any write" guarantee is a single, obvious insertion point.
- Add the `CliError::OutputCollision` arm to BOTH `code()` (→ 2) and `exit_code_mapping_is_total`, or
  the totality test fails to compile — that test is the guard against an unmapped variant.
- Prefer a message like: `output collision: "dist/logo.png" written by both "a/logo.png" and
  "b/logo.png" — two inputs map to one output (disambiguate the name template, e.g. {parent}_{stem})`.
- Mark DEC-057's injective section RESOLVED (approach + tradeoff); do **not** open a new DEC.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-065-injective-source-output`
- **PR (if applicable):** #71
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — as expected. This discharges DEC-057's named constraint; its Validation section is
    marked **RESOLVED by SPEC-065** with the approach + the over-detection tradeoff. DEC-059
    stays reserved for the lockfile (SPEC-066).
- **Deviations from spec:**
  - **The `{ext}` sentinel is the literal string `{ext}`, not `"\u{0}ext\u{0}"`.** The spec
    suggested a NUL-wrapped sentinel; a brace-wrapped one has the same property that matters (no
    real extension can contain `{`/`}`, so it cannot alias a real ext) and it stays **readable
    when it reaches the error message**: `output collision: "dist/logo.{ext}" written by both …`.
    A NUL sentinel would have printed control characters into the user's terminal. For a template
    that omits `{ext}` (e.g. `{name}`), the key is the real path, which reads even better.
  - **Out-dir normalization drops `Component::CurDir`** as well as trailing separators, so `dist`,
    `dist/`, and `./dist` are one directory. The spec asked for "a plain `Path`-based
    normalization"; `components().collect()` alone leaves a leading `./` intact, which would have
    let two targets spell the same directory two ways and slip past the cross-target check.
    Verified on the real binary (a `dist` + `./dist/` pair collides).
  - **The message quotes paths literally, not with `{:?}`.** The spec's suggested message uses
    `Debug`-style quoting; on Windows that escapes the separator and prints `"a\\logo.png"`, which
    is not a path a user can copy back into the manifest. Caught by the Windows CI job, since the
    source label is a `Path` display. Quoted literally instead, with a cross-platform unit test.
  - Added one missing `CliError::Metadata` assertion to `exit_code_mapping_is_total` while adding
    the `OutputCollision` arm — the test was not in fact total.
- **Follow-up work identified:**
  - **Residual under-detection: a template that hardcodes an extension.** Two targets writing the
    same `out` where one names `{stem}.png` and the other `{stem}.{ext}` produce keys
    `dist/logo.png` and `dist/logo.{ext}` — different keys, but potentially the same real path.
    Inherent to any pre-decode check that can't know the output ext; it needs a hardcoded-ext
    template to trigger, so it is rare. A format-sniff (the same refinement that would cut the
    false positives) closes both at once. Worth a line in DEC-059's threat model, not its own spec.
  - Not touched, still open from STAGE-021: the `CACHE_ENTRY_MAX_BYTES` vs read-frame off-by-53.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocked me; the spec's seam pointers were accurate against the post-cache tree and
   `## Notes for the Implementer` pre-answered the two decisions that would otherwise have cost a
   round-trip (global check, not per-target; both `code()` *and* the totality test). The one thing
   the spec asserted without checking was the sentinel: `"\u{0}ext\u{0}"` is correct as an equality
   key but wrong as a *display* string, and the same field is both (`OutputCollision.output` is
   compared and then printed). A design that separates the key from its label would not have had
   the question — but the single field is the simpler API, so the sentinel just has to be printable.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `DEC-058` was listed as context (why the ext is post-decode) but it also *predicted* a result
   worth flagging: because the cache key excludes the output destination, a cold build of two
   targets over the same inputs reports `(2 cached, 2 rebuilt)` — the second target hits the first's
   entries. That is correct and by design, but it surprised me mid-verify and it will surprise
   whoever writes SPEC-066's lockfile tests, since output paths and cache entries are not 1:1.

3. **If you did this task again, what would you do differently?**
   — Write the out-dir normalization test *first*. I reached for `components().collect()` because
   the spec named it, then only found the `./dist` hole when I sat down to write
   `collision_key_normalizes_the_out_dir`. The cross-target criterion is the one that depends on
   normalization, and it is the one a per-target implementation would have passed by accident —
   so the normalization deserved the failing test, not the collision itself. Related: every
   assertion I wrote about a *path* was Unix-shaped, and Windows CI caught two of them (a `/`
   literal in the integration test, and `Debug`-escaped separators in the error message itself).
   Any spec whose output is a path should get one deliberate "what does this print on Windows?"
   pass before the PR, not after — the round-trip cost more than the thinking would have.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — As architect: **never put `{:?}` in a user-facing message that carries a path.** My
   "Notes for the Implementer" suggested `{output:?}`, which is `Debug` on a `String` — a
   Windows path renders `"a\\logo.png"`, un-pasteable into a manifest. The three-OS CI caught
   it (and a mirror `/`-hardcoded bug in the build's own test); a mac-only author wouldn't
   have. Literal quoting is the rule for any message a user is meant to act on. Otherwise the
   spec held — the conservative-on-`{ext}` design and the global-check-after-phase-1 insertion
   point were both borne out.

2. **Does any template, constraint, or decision need updating?**
   — `DEC-057`'s Validation section now reads **RESOLVED by SPEC-065** (approach + the
   over-detection tradeoff); no new DEC (as designed). Two carries recorded in the STAGE-022
   Design Notes, neither blocking: (a) **the literal-extension residual** — a target naming
   `{stem}.png` and another `{stem}.{ext}` into one `out` still collide undetected (the
   `{ext}` token normalizes, a literal `.png` doesn't), inherent to any *pre-decode* check;
   it belongs in **DEC-059's threat model** and a format-sniff would close it and the false
   positives together. (b) **`exit_code_mapping_is_total` still omits `CliError::Cache`** — a
   pre-existing SPEC-064 gap the build note *claimed* was closed but wasn't (only a `Metadata`
   arm was added). Correcting the record here: the test is **still not total**. A one-line
   test addition, but a code/test change → it goes through a PR, not smuggled onto main at
   ship; carried as a follow-up (SPEC-066 touches `src/cli` and will absorb it).

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec — SPEC-065 was the unblocker, and STAGE-022's remaining spec (**SPEC-066**,
   the lockfile + `--check`/`--frozen`, **DEC-059**) is already in the backlog and is the next
   thing to frame, now unblocked. The two carries above fold into it: the literal-ext residual
   is a DEC-059 threat-model line, and the `CliError::Cache` exit-map gap is a one-line test
   fix SPEC-066 will pick up as it works in `src/cli`. STAGE-022 stays **active** (SPEC-066
   pending); no stage-ship yet.
