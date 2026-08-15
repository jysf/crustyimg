# SPEC-117 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** SPEC-115 stopped `optimize` shipping bytes it cannot name. Four verbs share
that seam; SPEC-115 shipped tests for two. Its verify **drove** the other two — `build` and
`apply --recipe web` — green by hand and pinned neither. Write the two pins.

> **PRECONDITION — check this first.** SPEC-116 also edits `tests/build.rs`. **Do not start until
> SPEC-116's PR has merged to `main`**, then branch from the merged `main`. If SPEC-116 is still
> open, stop and say so rather than branching around it. (`git log --oneline -5 origin/main`)

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-117-pin-build-and-apply-against-the-adopted-format-defect.md`,
   in full. **6 acceptance criteria, 2 pre-written tests.** Read its `## Failing Tests` section
   carefully — it is not the usual shape and the reason is stated there.
2. **The code** — `src/cli/optimize.rs`: `optimize_decide_one` (the fix) and
   `encode_one_optimize_decided` (the delegating wrapper `build` uses, ~`:1286`);
   `src/cli/build.rs:440-442`, the `OutputFormatPlan::Decide` arm.
3. **`tests/input_svg.rs`** — SPEC-115's own tests. **Match their assertion style**: sniff the
   written bytes, do not trust the extension or the summary line.
4. **DEC-089** — the `Image` container-origin model SPEC-115 introduced. Unchanged here.
5. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15 cycle-specific rules**.

## What makes this spec unusual — read before you plan

**Neither test fails on `HEAD`, and that is correct.** This is a regression pin on behaviour that
already works: `encode_one_optimize_decided` delegates *unconditionally* to the fixed function,
so today the two verbs are correct by construction.

That is exactly the situation in which a regression goes invisible. Every existing SVG/RAW/HEIC
test drives `optimize` directly, so a change to the delegation — a wrapper that stops calling
through, an early return, a second code path added for `build` — leaves all of them green while
two shipped verbs go back to writing mislabeled bytes.

So: `test-before-implementation` **does not apply in its usual form**. There is no red-to-green
transition to demonstrate. **Do not manufacture a fake red by breaking the source first.**

**AC-5 is the load-bearing criterion of this spec.** It is what separates a real pin from a
vacuous one, and a reviewer will treat it the way they would normally treat a failing test.

## AC-5 in detail — the criterion the whole spec rests on

Force the delegation to bypass the fix (revert `encode_one_optimize_decided` to a pre-SPEC-115
path, or stub the origin to `Native`). **Both** new tests must go RED. Restore.

**Per-verb, not one coarse revert.** If reverting once turns only one of them red, the other is
**not actually pinned** — and that is the finding, not a rounding error. A fix with N independent
sites owes N controls; one coarse revert cannot distinguish "distinct code path" from "vacuous
test". This exact shortcut shipped a vacuous test on SPEC-113 two specs ago
[[a-harness-that-exercises-nothing-reports-green]].

**Prove each revert reached the built artifact** — a changed binary hash shows a rebuild, driving
the binary shows the change took effect [[reverting-source-does-not-rebuild-the-binary]].

## The two tests

| test | AC | where |
|---|---|---|
| `apply_recipe_web_on_svg_writes_a_real_raster` | AC-1/AC-3/AC-4 | `tests/cli.rs` **or** `tests/input_svg.rs` |
| `build_on_svg_source_writes_a_real_raster` | AC-2/AC-3/AC-4 | `tests/build.rs` |

Put each test where its verb's other tests live. **Say which you chose and why** in Build
Completion — `tests/cli.rs` is 207 KB, so if `input_svg.rs` is the better home for the `apply`
case, take it and justify it.

Each test asserts three things, not one:

- **the bytes are a real WebP** — `image::guess_format` or a decode, **on the written bytes**,
  not the extension (AC-1/AC-2);
- **the output is not byte-identical to the source SVG** (AC-3) — the defect's signature was
  passing the container through verbatim; assert against it directly rather than inferring from
  the format sniff alone;
- **the reported format matches the bytes** (AC-4) — the summary / `--json` names the real
  container, not the adopted `png` label. SPEC-115 fixed the bytes *and* the report; a pin that
  only checks bytes lets half the fix regress.

## Fixtures — do not go hunting

`tests/fixtures/svg/rect_text_40x30.svg` (336 B) exists. **Use it. Do not build a new one.**
SPEC-115's fixture hunt cost most of its overrun; its Build Completion records that
`tight_preview.nef` and a naive noise preview both failed to reproduce. SVG is the family that
reproduces with a committed fixture. Do not repeat that search.

The behaviour verify already drove, for reference:

```
apply --recipe web  <svg fixture>  → a real WebP; summary reads svg → webp · 336 → 444 B
                                      (32% larger), with SPEC-115's fourth reason in the note
build               <svg source>   → rect_text_40x30.webp, RIFF … WebP image
```

## Source changes: NONE expected

**If you find yourself editing `src/`, stop.** That means the behaviour is not actually correct
today — which turns this spec from a pin into a fix, and that is an architect call, not a build
call. Report it in Build Completion and do not proceed.

## Verify before handing back (AC-6)

Full matrix, **fresh per-leg `CARGO_TARGET_DIR`**, **sequentially**, **through `rtk proxy` from
the first leg**: default, `--no-default-features`, `--features webp-lossy`. Clippy
(`--all-targets -- -D warnings`) and `fmt --check` on each.

**Establish your own `main` baseline** — the delta should be exactly the two new tests.

**A piped command reports the pipe's exit code.** Redirect to a file and read `$?`.

**Then read the CI legs on your PR individually** before claiming green (`gh pr checks <PR>`).

## Repo guardrails

- **Own git worktree.** Do not work in the primary checkout. Check `git branch --show-current`
  before any commit. Branch: `test/spec-117-pin-build-and-apply-adopted-format`, base the
  **merged** `main`.
- **Budget: if this takes more than an hour, something is wrong** — re-read the scope. It is two
  tests against an existing fixture. Past ~90 minutes, stop and report what you have.
- `git commit -s` (DCO enforced). Never `git reset --hard`. macOS has no `timeout(1)`.
- **`rtk` can silently corrupt grep counts and truncate `git log`.** Cross-check anything
  load-bearing with `/usr/bin/git` or raw `grep`, plus a positive control.
- **Do not merge the PR. Do not bump the version.**

## Out of scope

- **RAW and HEIC coverage for these two verbs.** SVG is sufficient to pin the delegation, and the
  format-specific behaviour is already covered at the `optimize` seam. Two more families × two
  more verbs is four more tests for no additional signal.
- **Any behaviour change.**
- SPEC-116 — separate spec, separate branch, separate PR (`one-spec-per-pr`).

**No new DEC expected.**

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. Create any `DEC-*` the build earned, with `affected_scope` set to the path globs it governs.
4. Run `just advance-cycle SPEC-117 verify`, and **CONFIRM it moved**: the command prints the
   file it wrote, and `git diff` on the spec should show the `cycle:` line change. It reports
   success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. **Identify your transcript by something
only your session emitted — never by "the newest `.jsonl` in the directory."** Price **per
component** at the anchors of the model `.message.model` actually reports (you are expected to be
Sonnet: $3/$15 per MTok; cache_creation ×1.25 input, cache_read ×0.10 input). State the anchors
next to the agent.

**Measure at session end, not mid-session** — a mid-session readout undercounts by ~40%.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
