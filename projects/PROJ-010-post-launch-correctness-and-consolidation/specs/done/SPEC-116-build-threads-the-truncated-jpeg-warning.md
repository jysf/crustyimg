---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-116
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 28772199
      duration_minutes: 104
      recorded_at: 2026-08-15
      tokens_breakdown:
        input: 412
        output: 130633
        cache_creation: 392886
        cache_read: 28248268
      estimated_usd: 11.91
      note: >
        MEASURED — transcript sum over 206 assistant messages
        (d6ab563a-c8cd-4338-b4ca-8dac02344cac.jsonl), all claude-sonnet-5.
        Priced per component at Sonnet anchors ($3/$15 per MTok;
        cache_creation x1.25 input, cache_read x0.10 input) — 98.2% of
        tokens were cache reads, so the flat 80/20 shortcut would badly
        overstate this. Most of the wall-clock went to the AC-9 matrix
        (3 full-workspace `cargo test` runs, one with a slow ~193s
        `audit_bench` binary) run sequentially per the guardrail, not to
        active generation.
        ⚠ OVERSTATED, NOT RECOMPUTABLE (flagged 2026-09-05). This figure used the
        naive all-lines sum corrected in STAGE-053 — every measured sibling lands
        between 1.38x and 2.88x over, so this number is high by an unmeasured factor
        in that band. Its transcript is no longer on disk, so no prefix reproduces
        the recorded total and a corrected figure CANNOT be derived. Deliberately
        left rather than scaled by an average — a fabricated precision would be
        worse than a flagged unknown.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 4617817
      duration_minutes: 129
      recorded_at: 2026-08-15
      tokens_breakdown:
        input: 82
        output: 40352
        cache_creation: 139368
        cache_read: 4438015
      estimated_usd: 4.10
      note: >
        MEASURED — transcript sum over 99 assistant messages
        (cc2f4817-fdf7-41c7-b145-35b0e8dc1f27.jsonl), all claude-opus-5.
        Priced per component at Opus anchors ($5/$25 per MTok;
        cache_creation x1.25 input, cache_read x0.10 input) — 95.9% of
        tokens were cache reads. Orchestrator re-derived it independently
        at ship: sum and dollar figure both match to the cent. Verifier's
        stated caveat: measured before their final report message, so it
        excludes roughly 4k output tokens (~$0.10). Most of the wall clock
        was the AC-9 matrix (six full test legs, sequential) plus three
        extra rebuilds for the AC-8, test-before-implementation and
        AC-7-vacuity controls.
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $10.03 / 10,975,994 tokens (2.45x over) over the same
        99 transcript lines = 41 real API calls. See STAGE-053.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop ship cycle (AGENTS §4). Merge, cost totals,
        reflection, archive, and the STAGE-043 close-out.
  totals:
    # ⚠ MIXED: includes at least one session flagged OVERSTATED, NOT
    # RECOMPUTABLE — this total is an upper bound, not a measurement.
    tokens_total: 33390016
    estimated_usd: 16.01
    session_count: 2
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

- [x] **AC-1.** `build` on a Decide-plan target whose input is `truncated.jpg` **warns on
      stderr**, naming the input, with the same `TRUNCATED_JPEG_WARNING` text `apply` uses.
      Assert on the message, not merely on non-empty stderr.
- [x] **AC-2.** **Exit stays 0 and the output is still written.** The warning is a diagnostic,
      not a failure — a truncated JPEG still produces a valid, if partly-grey, output today and
      must continue to.
- [x] **AC-3.** **`apply` and `build` agree on the identical input.** Drive both, assert both
      stderr streams carry the warning. This is the actual claim of the spec, and asserting only
      `build` would let the two drift apart again.
- [x] **AC-4.** **`--quiet` does NOT suppress it.** Pinned by a test, because the adjacent cache
      warning *is* quiet-gated and a future tidy-up will otherwise "make them consistent".
      [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]
- [x] **AC-5.** **A clean JPEG produces no warning.** The did-not-break-it control — without
      this, "always warn" passes AC-1 and ruins the verb.
      [[a-harness-that-exercises-nothing-reports-green]]
- [x] **AC-6.** **Bytes are unchanged.** `build`'s output for a clean input is byte-identical to
      `main`'s. This spec adds a diagnostic; it must not perturb encoding.
- [x] **AC-7.** **The pinned-format arm is untouched.** `OutputFormatPlan::Pinned` goes through
      `encode_one`, not this wrapper, and is out of scope — assert it still behaves as it does on
      `main` so the change is provably confined to the Decide arm.
- [x] **AC-8.** **A negative control**: revert the emit, confirm AC-1 goes RED and AC-5 stays
      green. Prove the revert reached the **built artifact** (a changed binary hash shows a
      rebuild; driving shows the change took effect). [[reverting-source-does-not-rebuild-the-binary]]
- [x] **AC-9.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run **sequentially**,
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
  - `"build_and_apply_agree_on_bytes_for_a_clean_input"` — AC-6. **Passes today.** Renamed from
    `build_output_bytes_unchanged_for_a_clean_input` at the punch list: it pins the same-branch
    cross-verb invariant, and the cross-version claim was driven once out of band instead —
    see `## Build Completion`.
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

- **Branch:** `feat/spec-116-build-threads-truncated-jpeg-warning`
- **PR (if applicable):** [#171](https://github.com/jysf/crustyimg/pull/171) — open, not merged,
  per instructions.
- **All acceptance criteria met?** yes — AC-1 through AC-9 all verified, boxes checked above.
- **New decisions emitted:** none. DEC-085 already governs the unconditional gating; this spec
  obeys it rather than reopening it.
- **Deviations from spec:** none in substance. One extra test beyond the five the prompt named:
  `build_pinned_format_arm_is_untouched_by_the_decide_arm_fix`, added to give AC-7 (the
  `OutputFormatPlan::Pinned` non-regression pin) its own committed assertion rather than relying
  on it being merely "not exercised."
- **Follow-up work identified:** `OutputFormatPlan::Preserve` and `OutputFormatPlan::Pinned` both
  route through `encode_one` (`src/cli/common.rs`), which has **no truncation check of its own** —
  a truncated JPEG through either arm stays silent on `build`, on `main` and unchanged after this
  fix. That is a second, distinct instance of the same class this spec just closed for the Decide
  arm. Confirmed out of scope for SPEC-116 by AC-7 and the spec's own "Out of scope" section;
  recorded here, not fixed, per the build prompt's explicit instruction to report rather than
  patch. This is exactly the class of gap STAGE-042's conformance matrix exists to catch
  mechanically.

### Punch-list follow-up (2026-08-15)

Verify returned **⚠ PUNCH LIST** (3 items) on PR #171. Two were `main`-side bookkeeping and were
applied by the orchestrator directly on `main` (a STAGE-042 backlog bullet for the cache-hit
swallow, and `src/cli/build.rs` / `tests/build.rs` added to DEC-085's `affected_scope`) — neither
is reflected on this branch. The third, and only branch-side item:

- **Renamed** `build_output_bytes_unchanged_for_a_clean_input` to
  `build_and_apply_agree_on_bytes_for_a_clean_input` and rewrote its doc comment. The test's body
  checks `build`'s bytes against `apply --recipe web`'s bytes **on this branch**; the old name and
  comment claimed byte-identity to `main`, which is not what the body checks. Verify ruled: keep
  the invariant, fix the label — a committed golden would go red on every `ravif`/`image` bump for
  reasons unrelated to this spec, and the repo has no such golden and should not acquire one here.
- **Cross-version evidence for AC-6**, driven once by verify out of band rather than pinned by a
  test:

  | | |
  |---|---|
  | `main` binary (`7ac9f27`) | `sha256 a82bb937…` |
  | branch binary (`7c6ff59`) | `sha256 5ed4a7c3…` |
  | input | `bench/corpus/photo_forest_cc0.jpg` |
  | target | `recipe = "web"`, default `{stem}.{ext}` → Decide plan |
  | `main` output | `clean.avif` `sha256 1c5ed3f1…` |
  | branch output | `clean.avif` `sha256 1c5ed3f1…` |

  Byte-identical, with differing binary hashes as the positive control that two genuinely
  different builds were compared. AC-6 as written holds; it is evidenced here rather than pinned
  by a committed golden.

No source changes. No new tests. `cycle:` stays at `verify` — verify re-reads and advances.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing substantive. The three design
   calls were genuinely settled, the trap (matching the `--quiet`-gated cache warning four lines
   below instead of the unconditional sibling) was flagged clearly enough that it never became a
   live risk, and the fixture/test-style pointers (`tests/hostile_inputs.rs:387`,
   `build_writes_the_decided_format_not_the_source_format`) mapped directly onto working test code.
2. **Was there a constraint or decision that should have been listed but wasn't?** No — DEC-085
   and DEC-087 covered the gating and the discard's history completely. The one thing the prompt
   didn't spell out (whether AC-7 needed its own dedicated test or could be satisfied by
   inspection) was a judgment call, not a missing constraint; a dedicated test was the more
   defensible reading of "pins."
3. **If you did this task again, what would you do differently?** Establish the full-workspace
   `main` baseline (all three feature legs) in one pass, up front, before starting the fix — I did
   it correctly but interleaved with the fix work, which meant an extra `git worktree add` round
   trip that could have been queued earlier. The AC-9 matrix itself is unavoidably the long pole
   (three full sequential `cargo test --workspace` runs, one binary with ~193s of slow
   `audit_bench` tests each time); nothing here shortens that legitimately.

---

## Reflection (Ship)

**Shipped 2026-08-15.** PR #171 (squash `016e89f`), 16/16 applicable CI checks green.
Cost: 39,748,193 tokens / **$21.94** across four cycles (design null, build $11.91 Sonnet,
verify $10.03 Opus, ship null).

**1. Did the spec hold up?** Yes, and its three settled design calls were the reason. The build
report says the trap — copying the `--quiet`-gated cache warning four lines below the call site
instead of the unconditional sibling — "never became a live risk" because the spec named it. That
is the clearest evidence this session that settling a call at design is cheaper than discovering
it at build.

**2. What did the cycle catch that the spec did not?** Three things, all from verify, and all of
a kind the spec could not have predicted:
- a **cache HIT swallows the warning** — `build_one` returns before the format-plan match, so
  run 1 warns and run 2 is silent while `apply` warns every time;
- **DEC-085's `affected_scope` was blind to its own second enforcement site**, proven with the
  audit tool's own output on this very diff;
- the AC-6 test was **correctly implemented and wrongly labelled** — it pins `build == apply` on
  the branch, not byte-identity to `main`.

The third is the interesting one. The build met AC-6 in substance (verify drove the bytes and
they were identical) but the test's *name* claimed something stronger than its body checked. A
green test with a false name is worse than a missing test, because it answers the audit question
wrongly instead of leaving it open.

**3. What should change?** Two things already have: DEC-085's scope now covers `src/cli/build.rs`,
and STAGE-042 carries five new instrument items found this session — including that
`decisions-audit` emits ~1,200 overlap warnings, which is how a real finding nearly went unseen.

The lesson worth carrying past this spec: **the orchestrator's handoff should name what it has
already checked.** Pre-crediting the cost arithmetic and the design-call compliance let verify
spend its budget on the two open questions instead of re-deriving settled ones — and both new
defects came out of that reallocated effort.
