# SPEC-116 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `apply --recipe web truncated.jpg` warns on stderr that the JPEG decoded
partially. `build`, on the identical input through a Decide-plan target, is **silent** — the
wrapper `build` calls discards the flag with an underscore. Thread it through and pin the two
verbs to agree.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-116-build-threads-the-truncated-jpeg-warning.md`,
   in full. **9 acceptance criteria, 3 settled design calls, 5 pre-written tests, a negative
   control.** Its `## The design calls` section is binding; do not reopen it.
2. **The code** (line numbers verified at `c39022d`, the commit you branch from):
   - `src/cli/optimize.rs:1292` — the discard, inside `encode_one_optimize_decided`
     (fn starts `:1286`). The fourth element is the flag.
   - `src/cli/optimize.rs:1468-1476` — **the emitting sibling. This is the voice and the gating
     to match**, including the `// F1 (SPEC-107, DEC-085)` comment above it explaining why it is
     unconditional. A second emitter lives at `:1522`.
   - `src/cli/build.rs:395` (`fn build_one`) → `:440-442`, the `OutputFormatPlan::Decide` arm.
   - `src/cli/ops.rs:328-338` — how `apply` forms its label and emits. AC-3 pins agreement with
     this path.
   - `src/image/mod.rs:119` — `TRUNCATED_JPEG_WARNING`; `is_truncated_jpeg` nearby.
3. **DEC-085** — why this warning is not `--quiet`-gated. Binding on design call 3.
4. **DEC-087** — named this follow-up when SPEC-111 shipped. You are discharging it.
5. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15 cycle-specific rules**.

## The verified facts (this repo, `c39022d`, read — not inferred)

```
src/cli/optimize.rs:1292
    let (output, _trace, _score, _truncated_jpeg) = optimize_decide_one(

src/cli/optimize.rs:1471-1474
    // F1 (SPEC-107, DEC-085): unconditional, not gated on `--quiet` — see
    // `report.rs`'s `run_info` for why.
    if truncated_jpeg {
        eprintln!("warning: {label}: {}", crate::image::TRUNCATED_JPEG_WARNING);
    }
```

`tests/fixtures/hostile/truncated.jpg` (758 B) exists. **Do not make a new fixture.**

Reproduce the silence yourself on `main` before changing anything — drive `apply --recipe web`
and `build` on that fixture and diff the two stderr streams. That reproduction is AC-3's
starting point, not a formality.

## The design calls are SETTLED — do not reopen them

1. **Emit from `build.rs`, not from inside the wrapper.** `encode_one_optimize_decided` is a pure
   encode helper returning `(String, Vec<u8>)`; it has no label and no business writing to
   stderr. **Widen its return to carry the flag** and let `build_one` emit — matching how
   `run_optimize`'s own call sites do it. The wrapper is `pub(super)`; the signature change is
   internal, no new export.
2. **The label is the input's display path.** `build.rs` already forms exactly this at `:250` and
   `:627` (`Input::Path(path) => path.display().to_string()`), and `apply` does the same at
   `ops.rs:332-335`. Not the output stem, not the target name — a user fixing the file needs to
   know which *input* is truncated.
3. **It is NOT gated on `--quiet`.**

## The trap that will make you ship the wrong fix

**Four lines below your call site**, at `src/cli/build.rs:447-450`, sits the cache warning:

```rust
if !ctx.quiet {
    eprintln!("warning: could not cache output: {e}");
}
```

Copying that shape is the wrong answer. DEC-085 and SPEC-107 made *this specific warning*
unconditional deliberately. **Match the sibling in `optimize.rs`, not the neighbour in
`build.rs`** — and carry the sibling's comment across, so the next person tidying these two into
consistency reads why before they do it. AC-4 is the test that stops them.

## Test style to mirror

`tests/hostile_inputs.rs:387` (`truncated_jpeg_warns_on_stderr_and_still_exits_0`) and its
control at `:417` (`well_formed_jpeg_emits_no_truncation_warning`) are the established shape for
this exact assertion — read both. **Your new tests land in `tests/build.rs`**, per the spec, but
assert the way those do: on the message content, not on non-empty stderr, and always with the
clean-input control beside the positive case.

`tests/build.rs` is 33 KB and already has Decide-plan targets — `grep` it for
`OutputFormatPlan::Decide` and reuse the existing target-construction helpers rather than
inventing new scaffolding.

## The five tests (from the spec — at least one must be RED on `main` before you write the fix)

| test (in `tests/build.rs`) | AC | on `main` today |
|---|---|---|
| `build_warns_on_a_truncated_jpeg_like_apply_does` | AC-1/AC-3 | **RED** |
| `build_still_writes_output_and_exits_zero_on_a_truncated_jpeg` | AC-2 | green |
| `build_truncated_jpeg_warning_survives_quiet` | AC-4 | **RED** |
| `build_does_not_warn_on_a_clean_jpeg` | AC-5 | green |
| `build_output_bytes_unchanged_for_a_clean_input` | AC-6 | green |

**Drive each one red/green on `main` yourself.** A test you assumed was red is not a red test
[[a-plausible-test-result-is-not-a-checked-one]].

AC-7 additionally pins that `OutputFormatPlan::Pinned` is untouched — it goes through
`encode_one`, not your wrapper. Assert it behaves as it does on `main` so the change is provably
confined to the Decide arm.

## Negative control (AC-8) — run it, record it, do not commit it

Revert the emit → `build_warns_on_a_truncated_jpeg_like_apply_does` goes **RED**,
`build_does_not_warn_on_a_clean_jpeg` stays **green**. Then restore.

**Prove the revert reached the built artifact**: a changed binary hash shows a rebuild, and
driving the binary shows the change took effect. Reverting source does not rebuild the binary
[[reverting-source-does-not-rebuild-the-binary]].

## Verify before handing back (AC-9)

Full matrix, **fresh per-leg `CARGO_TARGET_DIR`**, run **sequentially** (never both-shared-and-
parallel), **through `rtk proxy` from the first leg**:

- default
- `--no-default-features`
- `--features webp-lossy`

`clippy --all-targets -- -D warnings` and `fmt --check` on each leg. Confirm each log says
`Compiling crustyimg`; treat a missing one as a tooling failure first, not a fast build.

**Establish your own baseline on `main` first** — do not trust a test count quoted in any prompt,
this one included. The delta should be exactly your new tests.

**A piped command reports the pipe's exit code.** `cargo test | tail` turns a red leg green —
redirect to a file and read `$?`.

**Then read the CI legs on your PR individually** before claiming green. `gh pr checks <PR>`
returns them in one call; do not hand-grep downloaded job logs.

## Repo guardrails

- **Own git worktree.** Other sessions may be live in this repo — do not work in the primary
  checkout, and do not assume `target/` is yours. Check `git branch --show-current` before any
  commit. Branch: `feat/spec-116-build-threads-truncated-jpeg-warning`, base `main` at `c39022d`.
- **Checkpoint early.** Push a WIP commit as soon as it compiles, **before** the matrix — a
  3-hour cycle with zero commits is unrecoverable if it is stopped.
- **Budget:** if you are past ~90 minutes and the matrix has not started, stop and report what
  you have rather than pressing on.
- `git commit -s` (DCO enforced). Never `git reset --hard`. macOS has no `timeout(1)`.
- **`rtk` can silently corrupt grep counts and truncate `git log`.** Cross-check anything
  load-bearing with `/usr/bin/git` or raw `grep`, plus a positive control.
- **Do not merge the PR. Do not bump the version.**

## Scope discipline

**The fix is a few lines and the test is the work.** Out of scope:

- `OutputFormatPlan::Pinned` and `encode_one` (AC-7 pins them as untouched).
- **Any other diagnostic `build` might also be swallowing** — if you find one, **report it in
  Build Completion, do not fix it here.** That is a finding for STAGE-042's conformance matrix,
  which exists to catch this class mechanically.
- The `wasm` surface.
- SPEC-117 also touches `build`. **Separate spec, separate branch, separate PR**
  (`one-spec-per-pr`). Do not fold them.

**No new DEC expected** — DEC-085 already governs the gating and this spec obeys it. If you
believe the build earned one anyway, write it with `affected_scope` set to the path globs it
governs.

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. Create any `DEC-*` the build earned, with `affected_scope` set to the path globs it governs —
   that is what lets `decisions-audit --changed` surface it later.
4. Run `just advance-cycle SPEC-116 verify`, and **CONFIRM it moved**: the command prints the
   file it wrote, and `git diff` on the spec should show the `cycle:` line change. It reports
   success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. **Identify your transcript by something
only your session emitted — never by "the newest `.jsonl` in the directory."** Price **per
component** at the anchors of the model `.message.model` actually reports (you are expected to be
Sonnet: $3/$15 per MTok; cache_creation ×1.25 input, cache_read ×0.10 input). State the anchors
next to the agent.

**Measure at session end, not mid-session.** A readout written before the last leg undercounts —
SPEC-114's build reported $25.75 and finished at $34.31, 40% low with nothing wrong in the
measurement.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
