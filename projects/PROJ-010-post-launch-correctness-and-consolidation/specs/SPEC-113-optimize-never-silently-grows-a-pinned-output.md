---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-113
  type: bug                        # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-043
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the design call is settled
                                   # below, the comparison already exists on the
                                   # decide path, and the change is one branch
                                   # condition plus a guard. Verify stays Opus.
  created_at: 2026-08-11

references:
  decisions:
    - DEC-015
    - DEC-048
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-084
    - SPEC-088
    - SPEC-108

value_link: >
  STAGE-043's first item. `optimize` is a headline verb the launch post will name,
  and on its most natural invocation it can silently hand back a file twice the
  size of the source. Fixing it before anyone is invited to try the tool is the
  whole point of PROJ-010.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Drove the defect on the
        released 0.7.0 binary, read the fork at `optimize.rs:616-631`, and
        settled the `--profile preserve` question from each flag's documented
        purpose rather than deferring it to build.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-113: `optimize` never silently grows a pinned same-format output

## Context

**Driven on the released 0.7.0 binary, 2026-08-10.** Source: an already-compressed JPEG
(made with `sips -s formatOptions 15`, an independent tool):

```
crustyimg optimize low.jpg -o out.jpg
  41,862 B  →  84,586 B      (2.02× LARGER)
  exit 0, stderr COMPLETELY EMPTY
```

The **auto** path is honest by contrast — no `-o`, on the same input:

```
low2.jpg: jpeg → avif · 41862 → 52713 B (26% larger)
note: shipped 52713 B, larger than the 41862 B source (26% larger) — the source
      could not ship unchanged (metadata stripped / orientation baked / resized
      to the requested bound), so the smallest correct output was kept
```

So the guarantee and the reporting both exist; the pinned path simply never reaches them.

### Why the output is not merely bigger — it is strictly worse

Re-encoding a lossy JPEG **cannot recover detail the source already discarded**. The output is
therefore larger *and no better* than the input. **Keeping the original bytes would have been
superior on both axes, and was free.** A user who runs a verb called `optimize` and receives a file
twice the size, with no message, has been handed a worse artifact by the thing that promised a
better one.

The shape is ordinary, not contrived: `optimize photo.jpg -o out.jpg` is the most natural way anyone
tries the verb, and already-compressed JPEGs (anything pulled off a website) are the most common
input.

### Where it forks

`src/cli/optimize.rs:616-631`:

```rust
let pinned = resolve_format(global.format.as_deref())?.is_some()
    || global.output.as_deref().is_some_and(|o| o != "-"
        && crate::sink::format_from_extension(Path::new(o)).is_ok());

if profile == ProfileArg::Preserve || pinned {
    // Preserve / pinned: auto quality, per-input format preserved / honored
    // from -o/--format (DEC-015). This is the strict regression anchor.
    …
    return run_pixel_op(pipeline, inputs, global, None, None, Some(auto));
}
run_optimize_autodecide(…)
```

The never-bigger guarantee lives on the other side, in `decide::pick_winner`
(`src/analysis/decide.rs:843` documents it: *"returns passthrough (`None`) when no fixed-quality
candidate beats the source — the never-bigger guarantee"*).

**The pin inherited `preserve`'s exemption by sharing its branch, not by decision.** The comment
even conflates them — *"Preserve / pinned … This is the strict regression anchor"* — which is
precisely why nobody noticed: the sentence is true of `preserve` and was never true of a pin. This
is the same shape as SPEC-110, where nothing distinguished the working verbs from the broken ones
except which pipeline builder each happened to call.

## The design call — settled here, not deferred to build

**`--profile preserve` stays exempt. The pin does not.**

Derived from each flag's documented purpose rather than from convenience:

- **`--profile preserve` is the engine-off regression anchor** (DEC-048; `cli-reference.md:412`
  lists it beside a pinned `-o`/`--format` as a case where *"the engine never auto-decides"*). Its
  job is to reproduce format-preserving behaviour **exactly**, so it can serve as a comparison
  baseline. Giving it a never-bigger guard would hand it an engine behaviour and destroy the very
  property it exists to provide. **Leave it alone, and say so in a comment**, so the next reader
  sees a decision rather than an oversight.
- **A pinned `-o out.jpg` is not a regression anchor.** It is a user saying "give me a JPEG." Honour
  the format; do not honour a size regression they did not ask for.

**Do not implement both behaviours behind a flag.** No new flag. If the build finds a reason this
split is wrong, that is a finding to report, not a second rule to invent.

## Goal

When `optimize` would write a **same-format** output **larger than the source** on the pinned path,
keep the source bytes instead and say so on stderr.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs:584-640` — `run_optimize`, the `pinned`/`preserve` fork, and
    `run_pixel_op`'s call.
  - `src/analysis/decide.rs:843` and `pick_winner` — the existing never-bigger comparison and the
    wording of the note the auto path prints. **Reuse the concept and match the message's voice.**
  - `src/cli/ops.rs` — `run_pixel_op`, where a pinned encode currently lands.
  - `docs/cli-reference.md:140,152,412` — what is documented about never-shipping-larger and about
    `preserve`.
- **Related code paths:** `src/cli/optimize.rs`, `src/cli/ops.rs`, `tests/cli.rs`.

## Outputs

- **Files modified:** `src/cli/optimize.rs` (the fork + the guard), possibly `src/cli/ops.rs`;
  `tests/cli.rs` (or a focused new test file); `docs/cli-reference.md` if a documented claim
  changes.
- **New fixture:** an already-compressed JPEG. **Generate it with an independent tool** (`sips`,
  ImageMagick), never with crustyimg — [[fixtures-from-the-code-under-test-cannot-fail]]. Commit it.
- **New exports:** none.

## Acceptance Criteria

- [ ] **AC-1.** `optimize <already-compressed.jpg> -o out.jpg` **writes the source bytes unchanged**.
      Assert **byte-identity with the source**, not merely "smaller than before" — the whole claim is
      that the original was the correct answer. **Fails today** (2.02× larger).
- [ ] **AC-2.** It **says so on stderr**, naming what happened, and **exit stays 0**. A silent
      correct answer is still a silent tool; the auto path's note is the voice to match. Assert on
      the message, not just on non-empty stderr.
      [[test-the-guard-where-the-criterion-applies]]
- [ ] **AC-3.** **Cross-format is untouched.** `optimize photo.jpg -o out.png` may legitimately grow
      — that is a deliberate conversion, not a failed optimization. Pin it with a test that would
      fail if the guard were applied by size alone rather than by *same-format-and-larger*.
- [ ] **AC-4.** **`--profile preserve` still grows, deliberately.** Same input, `--profile preserve`,
      still writes the larger re-encode — the engine-off anchor is intact. This AC exists so the
      decision above is **pinned by a test rather than by a comment**, and so a future
      "consistency" change fails loudly. [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]
- [ ] **AC-5.** `--format jpeg` (the other spelling of a pin) behaves the same as `-o out.jpg`.
      Both reach `pinned` at `:616`; assert both, because a fix applied to one spelling is exactly
      the unenumerated-cell defect this project keeps finding.
- [ ] **AC-6.** **The guard does not fire when it should not**: an ordinary photo pinned to the same
      format, where the re-encode is genuinely smaller, still writes the **new** bytes. Without this
      the fix could be "always keep the source," which passes AC-1 and destroys the verb.
      [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-7.** **The auto path is byte-identical to `main`.** It already has the guarantee; this
      change must not perturb it. Assert on bytes for a photographic input through `optimize` with
      no `-o`.
- [ ] **AC-8.** **A negative control**: revert the guard, confirm the AC-1 test goes RED, restore.
      Record it, and prove the revert reached the **built artifact** rather than only the source —
      a changed binary hash shows a rebuild, driving shows the change took effect.
      [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-9.** Clean **full matrix** from fresh per-leg `CARGO_TARGET_DIR`s, run **sequentially**,
      **through `rtk proxy` from the first leg**: default, `--no-default-features`,
      `--features webp-lossy`; `clippy --all-targets -- -D warnings` each; `fmt --check`. Confirm
      each log says `Compiling crustyimg`. **Then read the CI legs individually.**
- [ ] **AC-10.** `docs/cli-reference.md:140` says `optimize` *"never ships a larger file"* — **read
      it as text against the new behaviour and against `--profile preserve`'s exemption.** If it is
      now true only with a caveat, state the caveat. Do not make the code match a sentence; make the
      sentence match the code. [[documentation-has-no-green]]

## Failing Tests

Written during **design**, BEFORE build. **At least one must FAIL on today's `HEAD`** — if they all
pass before the fix, they do not cover the bug.

- **`tests/cli.rs`** (or a focused new file)
  - `"optimize_keeps_the_source_when_a_same_format_reencode_would_grow_it"` — AC-1, byte-identity.
    **FAILS today** (writes 84,586 B for a 41,862 B source).
  - `"optimize_says_so_when_it_keeps_the_source"` — AC-2, asserts the stderr message and exit 0.
    **FAILS today** (stderr is empty).
  - `"optimize_cross_format_may_still_grow"` — AC-3. **Passes today**; guards against a
    size-only guard.
  - `"optimize_profile_preserve_still_grows"` — AC-4. **Passes today**; pins the deliberate
    exemption so a later "consistency" change fails loudly.
  - `"optimize_format_flag_pin_behaves_like_output_extension_pin"` — AC-5. **Fails today** for the
    same reason as AC-1.
  - `"optimize_still_writes_the_smaller_reencode_when_it_wins"` — AC-6. **Passes today**; the
    did-not-break-the-verb control.
  - `"optimize_auto_path_is_unchanged"` — AC-7. **Passes today**; byte-identity control.
- **Negative control** (AC-8, run and recorded, not committed)
  - Revert the guard → `optimize_keeps_the_source_when_a_same_format_reencode_would_grow_it` RED.

## Implementation Context

### Decisions that apply
- **DEC-048** — the format-decision engine and its profiles. `preserve` is the engine-off profile;
  this spec keeps it that way and adds a test so the exemption is claimed rather than assumed.
- **DEC-015** — per-input format honoured from `-o`/`--format`. Unchanged: the pin still decides the
  format. This spec changes only whether a *larger same-format* result is written.
- **No new DEC is expected.** The design call above follows from each flag's documented purpose. If
  the build finds a reason to deviate, report it as a finding.

### Constraints that apply
- `test-before-implementation` (**blocking**) — the Failing Tests go in first, and at least one must
  be red on `HEAD`.
- `clippy-fmt-clean` (**blocking**) — every leg of AC-9.
- `one-spec-per-pr` (**blocking**) — SPEC-114 (the `meta` lane) is a separate spec on the same
  stage. Do not fold them.

### Prior related work
- **SPEC-084** — the never-bigger guarantee and the two-regime quality search; the source of the
  comparison being reused.
- **SPEC-088** — why `--json`/`--timing` is a usage error on the pinned path. Same fork, same
  reasoning about what a pin means; useful precedent for treating pin and `preserve` as related but
  distinct.
- **SPEC-108** — the 18.5× blow-up. Same defect class on the decide path; this is its pinned-path
  counterpart, and **worse in one respect: that one at least reported the size.**

### Out of scope (for this spec specifically)
- **`build`'s truncated-JPEG warning** — that is SPEC-114's stage-mate, filed separately.
- **The orphaned-artifact follow-up** — parked in STAGE-043, needs a scope decision.
- **`web`** — it already scores and reports always (SPEC-085); do not touch it.
- **Any new flag**, and any change to cross-format conversion.

## Notes for the Implementer

- **The comparison already exists.** `decide::pick_winner` does source-vs-candidate on measured
  bytes. The work is reaching that judgement from the pinned branch **without** dragging the whole
  decision engine along — a pin has no candidates to compare, only "the re-encode" versus "the
  source."
- **Match the auto path's voice.** It already prints a note explaining *why* the source could not
  ship unchanged. On the pinned path the source **can** ship unchanged, so the message is simpler —
  but it should read like the same tool wrote it. Comments and user-facing text stay plain and
  behaviour-first: **no SPEC/DEC references in strings** ([[comments-plain-no-spec-refs]]).
- **Watch for a metadata trap.** The auto path's note lists reasons the source could not be shipped
  verbatim — *metadata stripped / orientation baked / resized*. If any of those apply, the source
  bytes are **not** a valid output and keeping them would be wrong. Determine whether the pinned
  path can be in that state (`optimize` bakes orientation and drops metadata via
  `optimize_pipeline`), and if so, the guard's condition is narrower than "output > source". **This
  is the trap in this spec — find it before writing the guard, not after.**
- **A piped command reports the pipe's exit code.** Redirect and read `$?`
  ([[a-piped-command-reports-the-pipes-exit-code]]).
- **rtk corrupts output intermittently**, including deleting the `Compiling crustyimg` line and
  mangling binary through `cat`. Run every leg through `rtk proxy` from the first, and use
  `/bin/cat` for binary ([[rtk-can-silently-corrupt-grep-counts]]).
- macOS has no `timeout(1)`. `git commit -s` (DCO enforced). Own git worktree — **two other sessions
  are live in this repo**. Never `git reset --hard`. **Do not merge the PR.**

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
