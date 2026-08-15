---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-116
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-043
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the fix is threading one
                                   # already-returned bool to an existing
                                   # println, and the design call below is
                                   # settled. Verify stays Opus.
  created_at: 2026-08-15

references:
  decisions:
    - DEC-085
    - DEC-087
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-107
    - SPEC-111

value_link: >
  STAGE-043's second and last item. `apply --recipe web bad.jpg` warns that a
  truncated JPEG decoded partially; `build` on the identical input is silent.
  A user who moved from `apply` to `build` silently lost a warning the project
  already decided was worth having.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Read the discard at
        `optimize.rs:1292`, the emitting sibling at `:1472`, and `build.rs:441`;
        settled the label question and the `--quiet` trap from DEC-085 rather
        than deferring either to build.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-116: `build` threads the truncated-JPEG warning

## Context

SPEC-107 made a truncated JPEG announce itself on stderr instead of silently handing back a
partly-grey image. SPEC-111 then gave `build` an auto-decide path through the same seam —
`optimize_decide_one` — but the wrapper it added **discards the signal**.

`src/cli/optimize.rs:1291-1292`:

```rust
let (output, _trace, _score, _truncated_jpeg) = optimize_decide_one(
```

That fourth element is the flag. Two sibling call sites in the same file consume it and warn
(`:1468-1474` and `:1518-1523`); this one drops it on the floor with an underscore. So:

```
crustyimg apply --recipe web truncated.jpg   → warning on stderr
crustyimg build  (same input, Decide plan)   → silent
```

**Filed in DEC-087 as a named follow-up, explicitly not an AC of SPEC-111.** This spec is that
follow-up. It is the last open item in STAGE-043; SPEC-113 closed the other.

### Why it matters more than its size suggests

The warning exists because a truncated JPEG decodes *successfully* into a partly-grey image —
the failure is silent by nature, which is why SPEC-107 made it loud. `build` is the batch verb,
so it is the one most likely to be pointed at a directory of files nobody inspected
individually. Losing the warning there loses it exactly where it is most needed.

## Goal

`build`, on a Decide-plan target, emits the same truncated-JPEG warning `apply` emits for the
identical input. Byte output is unchanged.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs:1286-1305` — `encode_one_optimize_decided`, the discard.
  - `src/cli/optimize.rs:1468-1476` — the emitting sibling. **This is the voice and the
    gating to match**, including the comment explaining why it is unconditional.
  - `src/cli/build.rs:430-445` — the `OutputFormatPlan::Decide` arm and what is in scope there.
  - `src/image/mod.rs:119` — `TRUNCATED_JPEG_WARNING`, and `is_truncated_jpeg`.
  - **DEC-085** — why this warning is not gated on `--quiet`.
- **Related code paths:** `src/cli/build.rs`, `src/cli/optimize.rs`, `tests/build.rs`.
- **Fixture that already exists:** `tests/fixtures/hostile/truncated.jpg`. Do not make a new one.

## The design calls — settled here, not deferred to build

**1. Emit from `build`, not from inside the wrapper.** `encode_one_optimize_decided` is a pure
encode helper returning `(String, Vec<u8>)`; it has no label and no business writing to stderr.
Widen its return to carry the flag and let `build.rs` emit, matching how `run_optimize`'s own
call sites do it.

**2. The label is the input's display path**, the same thing `build` already uses in its other
per-input messages — not the output stem, and not the target name. A user fixing the file needs
to know which *input* is truncated.

**3. It is NOT gated on `--quiet`.** This is the trap. The code immediately below the call site
in `build.rs:447` reads `if !ctx.quiet { eprintln!(...) }` for the cache warning, and copying
that shape would be wrong. DEC-085 and SPEC-107 made this specific warning unconditional
deliberately — `report.rs`'s `run_info` carries the reasoning, and the sibling at `:1470` carries
the comment. **Match the sibling, not the neighbour.**

## Outputs

- **Files modified:** `src/cli/optimize.rs` (widen the wrapper's return), `src/cli/build.rs`
  (consume and emit), `tests/build.rs` (the tests).
- **New fixtures:** none — `tests/fixtures/hostile/truncated.jpg` exists.
- **New exports:** none. The wrapper is `pub(super)`; its signature change is internal.
- **No new DEC expected.** DEC-085 already governs the gating; this spec obeys it.

## Acceptance Criteria

- [ ] **AC-1.** `build` on a Decide-plan target whose input is `truncated.jpg` **warns on
      stderr**, naming the input, with the same `TRUNCATED_JPEG_WARNING` text `apply` uses.
      Assert on the message, not merely on non-empty stderr.
- [ ] **AC-2.** **Exit stays 0 and the output is still written.** The warning is a diagnostic,
      not a failure — a truncated JPEG still produces a valid, if partly-grey, output today and
      must continue to.
- [ ] **AC-3.** **`apply` and `build` agree on the identical input.** Drive both, assert both
      stderr streams carry the warning. This is the actual claim of the spec, and asserting only
      `build` would let the two drift apart again.
- [ ] **AC-4.** **`--quiet` does NOT suppress it.** Pinned by a test, because the adjacent cache
      warning *is* quiet-gated and a future tidy-up will otherwise "make them consistent".
      [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]
- [ ] **AC-5.** **A clean JPEG produces no warning.** The did-not-break-it control — without
      this, "always warn" passes AC-1 and ruins the verb.
      [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-6.** **Bytes are unchanged.** `build`'s output for a clean input is byte-identical to
      `main`'s. This spec adds a diagnostic; it must not perturb encoding.
- [ ] **AC-7.** **The pinned-format arm is untouched.** `OutputFormatPlan::Pinned` goes through
      `encode_one`, not this wrapper, and is out of scope — assert it still behaves as it does on
      `main` so the change is provably confined to the Decide arm.
- [ ] **AC-8.** **A negative control**: revert the emit, confirm AC-1 goes RED and AC-5 stays
      green. Prove the revert reached the **built artifact** (a changed binary hash shows a
      rebuild; driving shows the change took effect). [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-9.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run **sequentially**,
      through `rtk proxy` from the first leg: default, `--no-default-features`,
      `--features webp-lossy`. `clippy --all-targets -- -D warnings` and `fmt --check` each.
      **Establish your own baseline on `main` first** — do not trust a count quoted in a prompt.
      **Then read the CI legs individually.**

## Failing Tests

Written during **design**, BEFORE build. At least one must FAIL on today's `HEAD`.

- **`tests/build.rs`**
  - `"build_warns_on_a_truncated_jpeg_like_apply_does"` — AC-1/AC-3. **FAILS today**
    (`build`'s stderr is silent).
  - `"build_still_writes_output_and_exits_zero_on_a_truncated_jpeg"` — AC-2. **Passes today**;
    pins that the warning did not become a failure.
  - `"build_truncated_jpeg_warning_survives_quiet"` — AC-4. **FAILS today**, and would fail
    again the moment someone quiet-gates it.
  - `"build_does_not_warn_on_a_clean_jpeg"` — AC-5. **Passes today**; the control.
  - `"build_output_bytes_unchanged_for_a_clean_input"` — AC-6. **Passes today**.
- **Negative control** (AC-8, run and recorded, not committed)
  - Revert the emit → `build_warns_on_a_truncated_jpeg_like_apply_does` RED,
    `build_does_not_warn_on_a_clean_jpeg` still green.

## Implementation Context

### Decisions that apply
- **DEC-085** — the truncated-JPEG warning is **unconditional**, not `--quiet`-gated. Binding on
  design call 3.
- **DEC-087** — named this exact follow-up when SPEC-111 shipped. This spec discharges it; no
  amendment needed.

### Constraints that apply
- `test-before-implementation` (**blocking**) — the Failing Tests go in first; at least one red
  on `HEAD`.
- `clippy-fmt-clean` (**blocking**) — every leg of AC-9.
- `one-spec-per-pr` (**blocking**) — SPEC-117 is a separate spec on a different stage. Do not
  fold them, even though both touch `build`.

### Prior related work
- **SPEC-107** — created the warning and the reasoning for its unconditional gating.
- **SPEC-111** — added the wrapper that drops it. Read its Build Completion: the discard was
  known and deferred, not missed.

### Out of scope
- `OutputFormatPlan::Pinned` and `encode_one` (AC-7 pins them as untouched).
- Any other diagnostic `build` might also be swallowing — if you find one, **report it, do not
  fix it here**. That is a finding for STAGE-042's conformance matrix, which exists to catch
  exactly this class mechanically.
- The `wasm` surface.

## Notes for the Implementer

- **The fix is three lines and the test is the work.** Resist widening scope; the value here is
  the pinned agreement between `apply` and `build`, not the plumbing.
- **Match the sibling, not the neighbour** — see design call 3. The wrong pattern is four lines
  below the right one in the same file.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
  [[a-piped-command-reports-the-pipes-exit-code]]
- macOS has no `timeout(1)`. `git commit -s` (DCO). **Own git worktree**, and stay in it — do
  not work in the primary checkout. **Do not merge the PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md` when you finish, including
  `just advance-cycle SPEC-116 verify` — and confirm it actually moved the `cycle:` line.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
