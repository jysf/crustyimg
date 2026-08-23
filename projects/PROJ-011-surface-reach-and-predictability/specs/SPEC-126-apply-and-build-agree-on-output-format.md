---
task:
  id: SPEC-126
  type: bug
  cycle: design
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
  constraints:
    - clippy-fmt-clean
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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

- **Branch:**
- **PR:**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Files this diff touches** — list every one, built from `git diff --name-only`, not recall:
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
