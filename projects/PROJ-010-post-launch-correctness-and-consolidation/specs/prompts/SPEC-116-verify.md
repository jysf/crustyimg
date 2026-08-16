# SPEC-116 — VERIFY prompt

Cycle: **verify**. **New session — do not continue from the build session.** You are reviewing
someone else's work, and a continuation session cannot see the drift a fresh one can.

**What shipped:** `build`, on a Decide-plan target, now emits the truncated-JPEG warning `apply`
already emitted. PR [#171](https://github.com/jysf/crustyimg/pull/171), branch
`feat/spec-116-build-threads-truncated-jpeg-warning`, 16/16 applicable CI checks green.

**Your verdict is one of:** ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-116-build-threads-the-truncated-jpeg-warning.md`,
   in full, **including `## Build Completion`**. 9 ACs, 3 settled design calls.
2. **The build prompt** — `prompts/SPEC-116-build.md`, so you can see what the builder was told
   and judge deviation against instruction rather than against your own taste.
3. **The diff** — `git diff origin/main...origin/feat/spec-116-build-threads-truncated-jpeg-warning`.
   It is small: 16 lines in `src/cli/build.rs`, 14 in `src/cli/optimize.rs`, 282 in `tests/build.rs`.
4. **DEC-085** (unconditional gating) and **DEC-087** (which named this follow-up).
5. **`/AGENTS.md` §15** — the verify cycle's own rules.

## Work already done for you — confirm, don't redo

The orchestrator checked these at handoff. **Confirm each rather than re-deriving it**, and say so
if you disagree — a handed conclusion is a claim, not a result.

- **The three design calls were followed.** Emit lives in `build_one`, not the wrapper; the wrapper
  widened to `(String, Vec<u8>, bool)`; the label is the input's display path; the emit is
  unconditional and carries the sibling's comment. The trap (copying the `--quiet`-gated cache
  warning four lines below) was **not** hit.
- **No new fixture.** `truncated_jpeg_fixture()` reads the committed
  `tests/fixtures/hostile/truncated.jpg`.
- **The cost readout is arithmetically correct.** `412 + 130,633 + 392,886 + 28,248,268 =
  28,772,199` ✓, and priced per-component at Sonnet anchors ($3/$15, cache-write ×1.25,
  cache-read ×0.10) gives **$11.9085**, recorded as `$11.91` ✓. The model in the readout matches
  the model in `agents.implementer`. **Do not re-audit the cost; audit that the entry is in
  `cost.sessions` and that the build cycle is not null.**
- **AC-3's test is genuinely strong** — it drives `apply`, captures its actual `warning:` line, then
  asserts `build`'s stderr contains a line **string-equal** to it. That is the real claim, not two
  independent substring checks, and it empirically proves the two label constructions agree.

## The two things worth your attention

Everything above is clean. These are not.

### 1. AC-6's test does not test AC-6 as written — decide whether that matters

**AC-6 says:** *"`build`'s output for a clean input is **byte-identical to `main`'s**."*

**`build_output_bytes_unchanged_for_a_clean_input` asserts:** `build`'s bytes ==
`apply --recipe web`'s bytes, **both on this branch.**

Those are different claims. The committed test is a *same-branch cross-verb* check; AC-6 asked for
a *cross-version* one. A change that perturbed encoding identically on both paths would leave this
test green — the oracle is built from the code under test
[[fixtures-from-the-code-under-test-cannot-fail]].

**This is a judgment call, not an automatic reject.** Arguments on both sides:

- *For accepting:* the diff is provably diagnostic-only — the sole behavioural change is an
  `eprintln!` — so byte drift is not mechanically possible here. And `build == apply` is arguably a
  **better standing invariant** than a frozen golden, because it keeps pinning something after
  `main` moves on.
- *Against:* AC-6 as written is then **untested**, and the spec's completion table claims it met.
  A criterion nobody checks is the shape this project has been bitten by repeatedly
  [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]].

**What to do:** drive the cross-version check yourself, once, out of band — build `main`'s binary
and the branch's binary, run both on the same clean JPEG through `build`, compare hashes. That
settles AC-6 factually in about ten minutes. Then rule on whether the committed test's
different-but-defensible invariant stands as-is, needs its doc comment corrected to say what it
actually pins, or needs a second assertion. **Any of those three can be the right answer; say
which and why.**

### 2. The follow-up finding has no reader — it will go quiet at archive

Build Completion records a real, confirmed second defect:

> `OutputFormatPlan::Preserve` and `OutputFormatPlan::Pinned` both route through `encode_one`
> (`src/cli/common.rs`), which has **no truncation check of its own** — a truncated JPEG through
> either arm stays silent on `build`.

Correctly reported rather than fixed; AC-7 and the spec's Out-of-scope section put it out of
bounds, and the build prompt told the builder to report it. **That part was done right.**

**But it is filed only in `## Build Completion`, and `just archive-spec` moves this spec to
`done/` at ship.** A grep of `projects/*/stages/STAGE-042-*.md` for `encode_one`, `Preserve` or
`Pinned` returns **nothing**. So the finding survives as archived prose that no command surfaces —
the same failure mode that cost this repo three measured defects sitting invisible in
`docs/backlog.md` on 2026-08-15 [[a-document-is-not-a-backlog-unless-tooling-reads-it]].

**Punch-list item:** it needs a bullet on **STAGE-042**'s `## Spec Backlog` in the form
`- [ ] (not yet written) — [S] <summary>`, so `just backlog` sees it. Confirm the defect is real
first (drive a truncated JPEG through a `Preserve` and a `Pinned` target and observe the silence)
— **do not file a finding you have not reproduced.**

## The rest of the checklist

Standard verify questions, none of which the above replaces:

- **Every AC met?** Walk all nine. Diff the spec's completion table against the actual test
  files — a checked box is a claim [[verify-test-existence-not-just-gate-count]].
- **AC-8's negative control.** Build Completion says the revert produced the predicted RED/green
  split with a changed binary hash. **Re-run it**, or state that you accepted the builder's record
  and why. One emit site means one revert is the right granularity here — unlike SPEC-117, which
  has two.
- **AC-9's matrix.** The builder reports 873→879, 853→859, 879→885, `+6` on every leg, clippy and
  fmt clean. **Establish your own baseline** — a count quoted in a completion note is not a
  measurement [[a-number-from-an-unproven-path-is-not-a-measurement]]. Fresh per-leg
  `CARGO_TARGET_DIR`, sequential, through `rtk proxy` from the first leg.
- **The extra test.** `build_pinned_format_arm_is_untouched_by_the_decide_arm_fix` was added beyond
  the five the spec named, to give AC-7 a committed assertion. Reasonable, but check it is not
  vacuous — does it actually exercise the `Pinned` arm, or does it pass because nothing ran?
  [[a-harness-that-exercises-nothing-reports-green]]
- **Decision drift.** Run `./scripts/decisions-audit.sh --changed main` — **pass the base ref.** A
  bare `--changed` scopes to uncommitted changes, and a clean checkout has none, so it reports
  "No changed files in scope" and exits 0. **That green cannot go red.**
- **Constraints.** `clippy-fmt-clean`, `test-before-implementation` (two tests confirmed RED on
  unmodified `main` — verify that claim), `one-spec-per-pr`.
- **New DECs?** Build says none, because DEC-085 already governs the gating. Agree or don't.
- **`cost.sessions`** has a design entry (null-with-note, correct) and a build entry (measured).

## Two things you can skip

- **Re-pricing the cost.** Already cross-checked to the cent, above.
- **Chasing the `Input::Stdin` arm** in the new label `match`. `build` rejects stdin at manifest
  validation (`src/cli/build.rs:164`), so that arm is unreachable from this path. It mirrors the
  existing idiom at `:251` and `:628` and is defensive, not dead-in-a-bad-way. *(If you want a
  finding out of it: the repo now constructs this label in four places — `ops.rs:332` via
  `path()/stem()`, and three `match` copies in `build.rs`. That is a tidy-up note at most, and
  explicitly **not** something to fix in this PR.)*

## Guardrails

- **Own git worktree**, off the PR branch. Do not work in the primary checkout.
- **Do not fix what you find.** Verify reports; a punch list goes back to build. The one exception
  is verify/ship bookkeeping, which per AGENTS §13 lands on **`main`**, not the feature branch.
- **Do not merge the PR. Do not bump the version.**
- `git commit -s` (DCO). macOS has no `timeout(1)`. **A piped command reports the pipe's exit
  code** — redirect and read `$?`.
- **`rtk` can silently corrupt grep counts and truncate `git log`** — cross-check anything
  load-bearing with `/usr/bin/git` or raw `grep`, plus a positive control.
- **Budget:** this is a small, green, well-documented diff. If you are past ~60 minutes and have
  not started the matrix, stop and report. Do not spend 14 hours re-deriving a green CI run.

## When you finish, in this order

1. Append a verify cost session entry to `cost.sessions` (see below).
2. Run `just advance-cycle SPEC-116 ship`, and **CONFIRM it moved** — `git diff` on the spec should
   show the `cycle:` line change. It reports success even when it changes nothing.
3. Give the verdict: **✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED**, with the AC-6 ruling stated
   explicitly and the STAGE-042 filing either done or listed.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl` in the directory."** Price per component at
the anchors of the model `.message.model` actually reports (you are expected to be Opus: $5/$25 per
MTok; cache_creation ×1.25 input, cache_read ×0.10 input). State the anchors next to the agent.

**Measure at session end, not mid-session** — a mid-session readout undercounts by ~40%.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
