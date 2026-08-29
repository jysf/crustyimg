# SPEC-126 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** `apply` resolved its output format two different wrong ways at two different
arities — one input with no `--format` defaulted to **PNG** (a generic `Sink::Dir` fallback firing,
not a deliberate choice), and N inputs **silently ignored `--format` entirely**. Both are now the
one rule every other pixel verb uses (`--format` > `-o` ext > preserve source), reused from
`ops::output_format_for` rather than reimplemented. `build` does not move. **Byte-changing on a
shipped verb.**

## ⚠ The PR is OPEN and must NOT merge

**PR #187, branch `fix/spec-126-apply-and-build-agree`, head `f8deb55`. Review the BRANCH.**
Base ref for every diff and for `decisions-audit` is **`9b4fb80`**.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec126-verify fix/spec-126-apply-and-build-agree
```

⛔ **Do not merge, do not bump the version, do not cut a release.** This batches into PROJ-011's
single lockfile migration with STAGE-050 (Call 3). A ⚠ or ❌ here is cheap — nothing has shipped.

**Make no commits.** Emit your `## Cost readout` and verdict in the return message (AGENTS §13).

> **Do not be confused by `just status` / `just backlog`.** They read the working tree, and the
> `cycle: verify` advance plus `## Build Completion` live **only on the branch** — `main` still
> reports `cycle: design` and will until this merges, weeks from now. The timeline on `main` is the
> authoritative trace. Nothing is wrong.

## Read in order

1. **The spec** — 7 ACs, 4 settled design calls, 4 failing tests, `## Build Completion` claiming
   **zero deviations**.
2. **DEC-098** (new, confidence 0.9) — carries the measured six-path table and, in its
   `## Validation`, the **entire AC-5 and AC-6 evidence record**. AC-5 and AC-6 were driven
   manually and **not committed as tests**, so that section is the only artifact; it is the thing
   most worth attacking.
3. **DEC-015** (the `--format` > `-o` ext > preserve-source precedence), **DEC-058** (why the
   byte change must batch its migration).
4. `src/cli/optimize.rs` (`run_apply`), `src/cli/common.rs` (`build_sink`, `apply_one`),
   `src/cli/ops.rs` (`output_format_for`), `tests/apply_batch.rs`.

---

## Already settled — do NOT re-derive

Four things the orchestrator drove against the branch. Don't spend budget re-confirming them.

1. **Cost.** `$38.16` / 60.8 min / 105,258,757 tokens, Sonnet. Re-derived component-wise and it is
   **exact to the cent** ($0.002 input + $3.93 output + $2.96 cache_creation + $31.26 cache_read),
   and the four components sum to `tokens_total`. It was taken **after** CI settled, correcting a
   mid-build reading that under-reported by 26 %. **Measure your own; do not audit this one.**
2. **The blast radius is structurally contained — I checked the call graph, not the claim.**
   `build_sink` has **exactly one caller** (`run_apply`'s single-input branch). `apply_one` has
   **two** (both inside `run_apply`). `build.rs` calls `encode_one` **directly**, and `encode_one`'s
   signature did not change. So `build` genuinely cannot move. Confirm by reading if you like, but
   the corpus run is not the only evidence and you needn't rebuild two binaries to re-establish it.
3. **⚡ `apply --recipe web` (terminal-`optimize`) never reaches the changed code.** It `return`s
   early in `run_apply`, above `reject_audit_without_autodecide` and far above the single-input
   branch. The flagship AVIF-decision path is untouched. I was worried about this; it is closed.
4. **CI is green on `f8deb55`** — 16 checks SUCCESS, the rest skipped release-only jobs. One
   snapshot if you want it. **Never poll.**

---

## Five specific things

### 1 — ⚡ An unnamed exit-code change on a documented surface. This is the item.

`build_sink` changed signature from `Result<Sink, CliError>` to `Sink`, and now **always** passes
`format: Some(fmt)`. It used to pass through `Option`, which could be `None`.

That means `apply <one input> -o -` **with no `--format`** used to build `Sink::Stdout { format:
None }` and fail `SinkError::UnknownFormat` → **exit 4**. On this branch it resolves to the source
format and **succeeds, exit 0**, writing bytes to stdout.

That is very likely the *correct* consequence of Call 1 — arguably a second bug fixed. **But
nobody named it.** No AC covers stdout. `## Build Completion` says "Deviations: None". And
**`docs/api-contract.md` is not in the diff**, while it documents this verb and this repo treats
exit codes as a contract (its `apply` entry says only "Single input → `-o`/`--out-dir`/stdout as
before" — which is now false).

**Drive it both ways at `9b4fb80` and at `f8deb55`**: `apply` one input `-o -` with and without
`--format`, and check the exit code and the bytes. Then rule:

- Is the new behaviour right? (I think yes — state it plainly either way.)
- Is a **4 → 0 exit-code change on a published CLI** an undocumented contract change that must be
  written into `docs/api-contract.md` before this ships?
- Does it deserve a deviation entry, given the spec claims none?

This is the difference between "one defect, two arities" and "one defect, two arities, plus a
third surface that moved silently."

### 2 — AC-6's corpus omits four verbs, and its SCOPE is a claim

The blast-radius run covered `resize`, `thumbnail`, `watermark`, `build`. Not `optimize`, not
`web`, not `convert`, not `responsive` — and `web` is the flagship.

I have already established (above) that the shared seams cannot carry a change to them. So the
question is not "is there a hole" but **"does the record admit what it did not test?"**
[[mechanical-sweeps-need-a-mechanical-check]] — a sweep's scope is as much a claim as its result.
Check that DEC-098's Validation section states the corpus boundary rather than implying the whole
surface was swept. If it does not, that is a punch-list item on the record, not on the code.

The corpus itself is 4 files / 2 formats. Ask whether that is enough to catch a format-dependent
divergence, and note that it carried a **real positive control** (`apply` flipping `.png`→`.jpg`),
which is the right shape [[a-plausible-test-result-is-not-a-checked-one]].

### 3 — AC-5's independence record makes a claim a test may not be able to make

DEC-098 says that with Call 2 reverted, `apply_honours_format_at_every_arity` goes RED on its
**multi-input sub-case** while "the single-input sub-case inside the same test stays green."

A test is one unit. "A sub-case stayed green inside a failing test" is only observable if the
assertions are ordered so the passing one runs first — otherwise it is a statement of intent
dressed as a measurement [[a-claim-that-a-test-is-vacuous-needs-driving-too]].

**Drive both reverts yourself** (AGENTS §15: one revert per independent condition, and the evidence
is the **behavioural flip**, never a hash). Then rule on whether AC-1's two arities should have
been **two tests** rather than one, so the independence is actually observable by the suite instead
of by a human watching output.

### 4 — AC-3 asserts a property between two paths. Check it cannot go green while both are wrong.

Call 4 was explicit: the test asserts `apply` and `build` **AGREE**, not that `apply` writes
`.jpg`. Right instinct. But two paths agreeing is exactly the shape that stays green if both move
together [[a-self-referential-control-cannot-detect-a-broken-pipeline]].

Confirm AC-2's preserve-source test independently pins **what** the shared answer is, so AC-2 and
AC-3 together are not circular. If AC-2 were deleted, could AC-3 still detect a regression?

### 5 — `#[allow(clippy::too_many_arguments)]` is a suppression, not a fix

`apply_one` went to 8 parameters and the build silenced the lint. Small, but this repo runs
`-D warnings` deliberately. Rule whether 8 positional args threaded through two call sites wants a
params struct, or whether the suppression is proportionate here. Either answer is fine; an
unremarked suppression is not.

---

## Also check

- **Decision drift:** `./scripts/decisions-audit.sh --changed 9b4fb80` — **pass the base ref**, or a
  clean checkout reports "No changed files in scope" and exits 0 on a green that cannot go red.
- **DEC-098's `affected_scope`** is `src/cli/optimize.rs`, `src/cli/common.rs`, `src/cli/ops.rs` —
  confirm that covers everything it governs, and that confidence 0.9 is honest (AGENTS §17).
- **Every file the diff touches is listed** in `## Build Completion` — 6 per
  `git diff --name-only 9b4fb80..f8deb55`. The entry hedges about which files "count"; check the
  list is complete rather than rhetorically complete.
- **AC-7's matrix** — three legs, fresh `CARGO_TARGET_DIR` each, sequential. `fmt --check` was run
  **once** (claimed feature-independent) rather than per leg; rule whether that is acceptable.
- **The "no second instance" sweep.** Build Completion claims every other `format: None` site in
  `src/cli/` writes pre-encoded bytes via `.write_bytes()`, where format is structurally unused.
  That is a mechanical claim — **cite the grep and check its scope** (did it look outside
  `src/cli/`?).
- **Nothing filed to the stage backlog.** If item 1 lands, something probably should be — check it
  reads back in `just backlog`.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge. Do not bump the version.
- **⚡ NEVER POLL CI.** The branch is already green at `f8deb55`.
- **Budget ~150 exchanges.** Five consecutive cycles have blown their budget without the checkpoint
  firing. Take your cost reading **once, at the end**, and note that a cycle structurally cannot
  count the messages that write its own cost block — say so if you snapshot early.
- macOS has no `timeout(1)`. A piped command reports the **pipe's** exit code — redirect and read
  `$?`. zsh does **not** word-split unquoted parameters — use `while IFS= read -r`. Use
  `/usr/bin/grep`, not the shell's aliased one [[rtk-can-silently-corrupt-grep-counts]].
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal — redirect stdout;
  do not "fix" it.
- **Another session may be active in this repo.** Work in your own worktree, and check
  `git branch --show-current` before anything.

## When you finish

1. **No commits.** 2. Emit `## Cost readout` (`cost-snippet.md`; price at the anchors
`.message.model` actually reports, per component — never flat, DEC-083). 3. Verdict — ✅ APPROVED /
⚠ PUNCH LIST / ❌ REJECTED, with **item 1's ruling stated explicitly and first**: whether the
stdout exit-code change is an accepted fix that needs documenting, or an unannounced contract change
that has to be named before this batches into the release.
