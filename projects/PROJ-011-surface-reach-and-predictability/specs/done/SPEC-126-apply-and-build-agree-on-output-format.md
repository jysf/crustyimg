---
task:
  id: SPEC-126
  type: bug
  cycle: ship
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-011
  stage: STAGE-049
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-23

references:
  decisions:
    - DEC-005
    - DEC-002
    - DEC-058
    - DEC-015
    - DEC-087
  constraints:
    - clippy-fmt-clean
    - ergonomic-defaults
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-111
    - SPEC-125

value_link: >
  PROJ-011's entry point. `apply` and `build` must agree on what a recipe's
  output format is before `Recipe` can gain a `format` field — otherwise the
  disagreement gets baked into the schema. Today a `*.build.lock` pins bytes the
  `apply` spelling of the same recipe cannot reproduce.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Framed from a defect
        driven on `main` at `232c9cf` and re-confirmed at `789e4a3`.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 105258757
      duration_minutes: 60.8
      recorded_at: 2026-08-23
      tokens_breakdown:
        input: 762
        output: 262007
        cache_creation: 789890
        cache_read: 104206098
      estimated_usd: 38.16
      note: >
        MEASURED — summed from the session transcript's per-message `usage`
        (381 messages), priced at Sonnet anchors ($3/$15 per MTok,
        cache_creation x1.25, cache_read x0.10) per `.message.model`. Taken
        AFTER PR #187's CI matrix settled fully green (16/16), not at the
        "almost done" point — an earlier mid-build reading of $30.30
        under-reported by ~26%, matching this repo's own measured pattern
        for readings taken before CI settles.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 22698850
      duration_minutes: 21.6
      recorded_at: 2026-08-23
      tokens_breakdown:
        input: 308
        output: 104013
        cache_creation: 302408
        cache_read: 22292121
      estimated_usd: 15.64
      note: >
        MEASURED — summed from the session transcript's per-message `usage`
        (154 assistant messages), priced at Opus anchors ($5/$25 per MTok,
        cache_creation x1.25, cache_read x0.10) per `.message.model` (DEC-083).
        Cache reads are 98.2% of volume, so a flat rate would overstate by ~7x.
        Transcript identified by CONTENT (it contains the SPEC-126 verify
        prompt), not recency. Structurally under-reports: a cycle cannot count
        the messages that write its own cost block. 41% of the build's cost;
        returned a 7-item punch list, 2 unnamed behaviour changes and a
        pre-existing tooling defect.
    - cycle: re-approve
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 19130895
      duration_minutes: null
      recorded_at: 2026-09-03
      tokens_breakdown:
        input: 312
        output: 136135
        cache_creation: 367797
        cache_read: 18626651
      estimated_usd: 15.02
      note: >
        MEASURED — summed from the session transcript's per-message `usage`
        (156 assistant messages), priced at Opus anchors ($5/$25 per MTok,
        cache_creation x1.25, cache_read x0.10) per `.message.model` (DEC-083).
        Cache reads 97.4% of volume; a flat rate would say $95.65, overstating
        6.4x. An EXTRA gate, not one of the five cycles — added because the
        orchestrator applied verify's punch list itself. It paid for itself:
        it caught an overstatement in the orchestrator's own STAGE-047 filing
        (two decisions claimed blind to the decisions-audit parser, only
        DEC-015 actually is), a false universal in `docs/api-contract.md`, and
        a file list stale by its own stated derivation.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop ship cycle (AGENTS §4) — merge, reflection,
        totals, archive.
  totals:
    tokens_total: 147088502
    estimated_usd: 68.82
    session_count: 3
---

# SPEC-126: `apply` and `build` agree on output format

## Context

**Driven on `main`, JPEG source, plain pixel recipe (auto-orient + resize, no terminal
`optimize`), no `--format` unless stated:**

| invocation | output |
|---|---|
| `apply` **1** input, no `--format` | **`src.png`** — the source format is CHANGED |
| `apply` **2** inputs, no `--format` | `src.jpg`, `src2.jpg` — preserved |
| `apply` **1** input, `--format png` | `src.png` ✅ honoured |
| `apply` **2** inputs, `--format png` | **`src.jpg`, `src2.jpg` — the flag is SILENTLY IGNORED** |

**`apply`'s multi-input path does no format resolution at all.** It preserves the source format,
ignoring both the single-input default and an explicit `--format`. Exit 0, no warning.

⚠ **`--name-template` is not the discriminator** — an explicit `{stem}.{ext}` behaves identically.
The discriminator is purely single-input vs many.

**This is one defect, not two.** An earlier audit reported it as F6 (`--format` ignored) and F7
(`apply`/`build` default disagreement); both are the same missing resolution step seen from
different sides.

### Why it matters beyond tidiness

`build` binds sources to a recipe and pins the result in a `*.build.lock` (DEC-058). The `apply`
spelling of that same recipe produces different bytes — so **two commands the docs present as
interchangeable are not**, and a lockfile cannot be reproduced by the other path.

## The design calls — settled here

### Call 1 — the correct default is PRESERVE THE SOURCE FORMAT, and this is measured, not argued

The whole rest of the surface already does it. Driven on `main`, same JPEG source, no `--format`:

| verb | output |
|---|---|
| `resize` | `src.jpg` — preserved |
| `thumbnail` | `src.jpg` — preserved |
| `watermark` | `src.jpg` — preserved |
| `build` | preserved |
| `apply`, **2 inputs** | preserved |
| **`apply`, 1 input** | **`src.png` — the sole outlier on the entire surface** |

**So `apply`-single-input moves, and nothing else does.** ⚠ **State this in the DEC rather than
treating it as obvious** — the opposite case is arguable (a plain pixel recipe has no format
opinion, and PNG avoids JPEG→JPEG generation loss), and the reason it loses is that consistency
across six paths beats a local optimum on one. Changing `build` instead would invalidate every
existing lockfile, which is the more expensive migration for the weaker reason.

### Call 2 — `--format` must be honoured at every arity

Not "warn that it was ignored" — **honoured**. It is honoured at one input today; the multi-input
path simply never asks. There is no behaviour to preserve here, only a missing step.

### Call 3 — ⚠ this is byte-changing, and it does NOT ship alone

Output bytes change for `apply` on a single input with no `--format` (PNG → the source format).
It batches into **PROJ-011's single lockfile migration** with STAGE-050. **Do not cut a release
for this spec.**

### Call 4 — the test asserts agreement, not a format string

A test that asserts *"`apply` writes `.jpg`"* pins the answer and not the property. **Assert that
`apply` and `build` produce byte-identical output** for the same recipe and input — that stays true
if a future spec changes the default for good reason, and goes red the moment the paths diverge
again. Add the `-o` vs `--out-dir` arm too; same class, and cheap here.

## Acceptance Criteria

- [ ] **AC-1.** `apply --format X` is honoured for **1 input and for N inputs**, identically —
      driven for at least two formats, so the test cannot pass by coincidence of the source format.
- [ ] **AC-2.** With no `--format`, `apply` **preserves the source format at every arity**
      (Call 1). Driven on a JPEG source *and* a PNG source, so "preserved" is not indistinguishable
      from "always PNG".
- [ ] **AC-3.** **`apply` and `build` produce byte-identical output** for the same recipe, input
      and settings — asserted on the bytes, not the extension or the summary line.
- [ ] **AC-4.** **`-o` and `--out-dir` agree** for the same `apply` invocation — byte-identical
      output.
- [ ] **AC-5.** **A negative control.** Each of AC-1..AC-3 fails on `main` before the fix, driven
      and recorded. ⚠ **One revert per independent condition** — reverting the multi-input
      resolution must not also disable AC-2's single-input assertion, or the controls are
      co-dependent (AGENTS §15).
- [ ] **AC-6.** **Nothing else changes bytes.** `resize`, `thumbnail`, `watermark` and `build`
      output byte-identical to `main` on the corpus — this spec moves `apply` only.
- [ ] **AC-7.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential: default,
      `--no-default-features`, `--features webp-lossy`. Clippy and `fmt --check` each. Then read
      the CI legs individually.

## Failing Tests

- **`tests/apply_batch.rs`** (exists — extend it)
  - `"apply_honours_format_at_every_arity"` — AC-1. **RED.**
  - `"apply_preserves_source_format_at_every_arity"` — AC-2. **RED** for one input.
  - `"apply_and_build_agree_byte_for_byte"` — AC-3. **RED.**
  - `"apply_output_flags_agree"` — AC-4.

## Implementation Context

### Out of scope
- `Recipe` gaining `format`/`quality` — STAGE-050, which depends on this.
- The `-o`-extension **pin ruling** (PROJ-010). AC-4 asserts the two flags *agree for `apply`*; it
  does not touch how `web`/`optimize` treat a pinned extension.
- Any change to `optimize`'s candidate selection, or to the terminal `optimize` marker.

## Notes for the Implementer

- **Read `src/cli/ops.rs` and `src/cli/optimize.rs`'s `run_pixel_op`** — the multi-input fan-out
  is where the resolution step is missing. `resize`/`thumbnail`/`watermark` go through a path that
  resolves correctly; find why `apply` does not and prefer reusing that path over adding a second.
- ⚠ **Do not "fix" this by warning that `--format` was ignored.** Call 2: honour it.
- **Budget ~150 exchanges.** Never poll CI; background `gh pr checks --watch` and leave it alone.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge the PR. Do not
  bump the version.** ⚠ **Do not cut a release** — this batches with STAGE-050 (Call 3).
- Redirect rather than pipe when reading a gate's result — a pipe reports the pipe's exit code.
  ⚠ **`cargo test` in an interactive terminal fails `display_sink_refuses_non_tty`** (a known,
  filed, environment-dependent test); redirect stdout and it passes.
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-126 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-126-apply-and-build-agree`
- **PR:** [#187](https://github.com/jysf/crustyimg/pull/187) — opened, NOT merged (batches with STAGE-050 per Call 3)
- **All acceptance criteria met?** yes
  - AC-1 (`--format` honoured at every arity, ≥2 target formats) —
    `tests/apply_batch.rs::apply_honours_format_at_every_arity`.
  - AC-2 (no `--format` preserves the source at every arity, ≥2 source
    formats) — `tests/apply_batch.rs::apply_preserves_source_format_at_every_arity`.
  - AC-3 (`apply`/`build` byte-identical) —
    `tests/apply_batch.rs::apply_and_build_agree_byte_for_byte`.
  - AC-4 (`-o`/`--out-dir` agree) —
    `tests/apply_batch.rs::apply_output_flags_agree`.
  - AC-5 (negative controls, one revert per independent condition) — driven
    manually, not committed as tests; see DEC-098's Validation section for
    the full baseline + two-revert record.
  - AC-6 (blast radius: `resize`/`thumbnail`/`watermark`/`build` unchanged) —
    driven manually against `main` on a 4-file/2-format corpus (16 output
    files), byte-identical; a positive control on the same corpus confirmed
    the methodology detects a real diff where one exists (`apply`'s own
    fixed defect). See DEC-098's Validation section.
  - AC-7 (clean full matrix) — `default`, `--no-default-features`,
    `--features webp-lossy`, each in a fresh `CARGO_TARGET_DIR`, run
    sequentially: `cargo fmt --check`, `cargo build`, `cargo test`, `cargo
    clippy --all-targets -- -D warnings` all exit 0 on every leg.
- **New decisions emitted:** DEC-098 (`decisions/DEC-098-apply-single-input-moves-to-preserve-format-not-build.md`).
- **Files this diff touches** — from `git diff --name-only main...HEAD`:
  - `src/cli/common.rs`
  - `src/cli/ops.rs`
  - `src/cli/optimize.rs`
  - `tests/apply_batch.rs`
  - `decisions/DEC-098-apply-single-input-moves-to-preserve-format-not-build.md`
  - `docs/api-contract.md`
  - this spec file
  (Seven files. Corrected twice. An earlier draft called DEC-098 "new/untracked"
  and outside what `git diff --name-only main...HEAD` reports — both wrong, fixed
  at verify. Then the punch-list commit that fixed it added `docs/api-contract.md`
  and did not add it to this list three lines above, so the list showed six where
  its own stated derivation returned seven — caught at re-approve, and the missing
  file was the user-facing one.)
- **Deviations from spec:** **Two, both found at verify, neither reworked — the code is
  right and the record was not.**
  1. **An exit-code change on three single-input `-o` invocations that no AC covers.** Making
     `build_sink` take an already-resolved format means it no longer receives `None`, so
     `apply` at one input with **`-o -` and no `--format`**, with **`-o` at a path with no
     extension**, and with **`-o` at an unrecognised extension** went from `SinkError` **exit
     4** to **exit 0**, preserving the source format. This is a conformance fix, not a
     contract break — `resize` and `thumbnail` already did all three on `main`, and the old
     `4` was outside `docs/api-contract.md`'s own enumeration of that code — but it is a
     documented surface that moved unannounced. Now written into
     `docs/api-contract.md`'s `apply` entry.
  2. **A byte change at single input from a literal-extension `--name-template`.** With one
     input and `--name-template '{stem}_w.jpg'`, `main` ignored the template and wrote **PNG
     bytes into a `.jpg` file** — mislabelled output. The branch writes real JPEG. The
     converse (`'{stem}_w.png'` on a JPEG source) now writes JPEG, matching multi-input
     `apply`, `resize` and `build`, which is the documented rule for a plain pixel recipe
     (`docs/api-contract.md`, DEC-087): a name template's literal extension names the file,
     it does not pin the format. Nothing got worse; one case got strictly better. Filed to
     STAGE-050 as the general question.

  Both design calls (Call 1: `apply`
  single-input moves to preserve-source; Call 2: `apply` multi-input honours
  `--format`) were implemented exactly as settled, reusing
  `ops::output_format_for` (widened to `pub(super)`) rather than
  reimplementing the resolution rule, per the spec's own steer.
- **Follow-up work identified:** None beyond what the spec/stage/project
  already scope out (STAGE-050's `Recipe` format/quality fields, the
  `-o`-extension pin ruling, PROJ-010's other invocation defects). Checked
  specifically for a second instance of the same bug class (a `Sink::Dir`/
  `Sink::Stdout` built with an unresolved `format: None` then passed to
  `.write()` rather than `.write_bytes()`) — every other `format: None` site
  in `src/cli/` (the metadata lane's `run_metadata_lane`, `write_encoded`,
  `write_optimize_output`) writes pre-encoded bytes via `.write_bytes()`,
  where `format` is structurally unused. No second instance found; nothing
  filed to the stage backlog.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing was
   unclear about the two Calls themselves — the spec's own repro table and
   "prefer reusing that path" steer pointed straight at
   `ops::output_format_for`. The one thing that took real investigation was
   locating WHERE each arity's bug actually lived: single-input's PNG
   default turned out to be `Sink::Dir::write`'s generic unset-format
   fallback (not a deliberate `apply` default anywhere), and multi-input's
   silently-ignored `--format` turned out to be `common::apply_one` hardcoding
   `format_override: None` into `encode_one` — whose OWN fallback logic was
   already correct. Neither bug was where a first guess ("apply must have its
   own bad default somewhere") would land; both required reading the actual
   call chain (`run_apply` → `build_sink`/`apply_one` → `Sink::write`/
   `encode_one`) to the bottom.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** DEC-015 (per-input format precedence) and DEC-058 (the lockfile's
   stake in `apply`/`build` agreeing) were exactly the decisions needed —
   **but "both were named" was wrong as written, and verify caught it.**
   DEC-058 was in `references.decisions`; DEC-015 was named only in DEC-098's
   own References, not in this spec's front-matter, which is where AGENTS §10
   requires it. So was DEC-087 (a name template's literal extension does not
   pin the format), and the constraint `ergonomic-defaults` — which this fix
   directly satisfies, since the old behaviour made `--format` boilerplate
   mandatory for the simple case to work at all. All four are now listed.
   ⚠ The miss had teeth: DEC-015's `affected_scope` includes
   `docs/api-contract.md`, which is exactly the file this spec should have
   updated and did not.
3. **If you did this task again, what would you do differently?** Run the
   AC-6 blast-radius comparison (main vs. fixed binary, hash-diffed corpus)
   EARLIER — as a first move right after reading the code, before writing
   the fix — since it doubles as a fast, cheap way to confirm the call-graph
   reading (which functions `resize`/`thumbnail`/`watermark`/`build` do and
   don't share with `apply`) is actually right, rather than only as a
   post-hoc control.

---

## Reflection (Ship)

**1. What went right, and would you do it the same way again?**

The design call was settled by driving all six sibling paths rather than by arguing about
which default is nicer, and that is the only reason the fix is one rule instead of a second
special case. "PNG avoids JPEG→JPEG generation loss" is a genuinely defensible position; it
lost to a measured table showing `apply` at one input was the sole outlier on the entire
surface. Do that again. Reusing `ops::output_format_for` instead of reimplementing followed
directly from the same table — once five paths were shown to share a rule, the only open
question was where to call it from.

**2. What went wrong, and what would you change?**

Every finding came from a review cycle and none from the build — five waves of the same
result. But the sharper version here is that **each review found the previous cycle's
RECORDS overstating what had been measured, never the code being wrong.** The code was
correct at `f8deb55` and stayed correct through two reviews. What kept failing was claims:
"Deviations: None" when three exit codes had moved; "both were named" when DEC-015 was not;
"two live decisions" when only one is; "every other pixel-lane verb" when two measurably
are not.

One pattern produced all four: **a half-verified claim stated with the confidence of a
fully-verified one.** The DEC-043 error is the clearest specimen — its liveness was checked,
its `affected_scope` never was, and the write-up quoted `superseded_by:` for it while
quoting scope contents for DEC-015. The tell was legible in the sentence itself. The fix is
mechanical: when a claim covers N items, quote the *same evidence field* for all N, and if
one of them cannot produce that field, that is the signal it was never checked.

**3. What should the next spec know?**

⚠ **A punch list applied by whoever received it is unreviewed work.** This spec added a
sixth cycle, `re-approve`, for exactly that, and it returned three real items for $15.02 —
22 % of the build. It should not become automatic, but when the orchestrator applies its
own punch list something has to read it, because **CI cannot**: all three findings were in
prose, and all sixteen legs were green throughout.

📌 **`docs/api-contract.md` is the highest-leverage file in a spec like this and the easiest
to forget.** It was not in the original diff at all, and DEC-015 — the decision this spec
implements — names it in `affected_scope`. The audit could not say so, because of the
inline-array parser defect (filed, PROJ-013 STAGE-047): **the one instrument that should
have caught it ran green over the one decision that mattered.**
