# SPEC-117 — VERIFY prompt

Cycle: **verify**. **New session — do not continue from the build session.**

**What shipped:** two regression-pin tests, no source changes. PR
[#174](https://github.com/jysf/crustyimg/pull/174), branch
`test/spec-117-pin-build-and-apply-adopted-format`, 16/16 applicable CI checks green,
`mergeStateStatus: CLEAN`.

**Your verdict:** ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Read in order

1. **The spec** — `.../specs/SPEC-117-pin-build-and-apply-against-the-adopted-format-defect.md`,
   in full, **including `## Build Completion`**. Note its `## Failing Tests` section: **neither
   test fails on `HEAD`, by design.** This spec pins behaviour that already works.
2. **The diff** — `git diff origin/main...origin/test/spec-117-pin-build-and-apply-adopted-format`.
   Three files: 82 lines in `tests/build.rs`, 74 in `tests/input_svg.rs`, and the spec.
3. **SPEC-115** — the fix these tests pin, and `tests/input_svg.rs`'s existing assertions.
4. **`/AGENTS.md` §15.**

## Work already done for you — confirm, don't redo

The orchestrator checked these at handoff. **Confirm rather than re-derive**, and say so if you
disagree.

- **`src/` is untouched.** `git diff --stat origin/main...<branch> -- src/` is empty. The spec said
  a source edit would turn this from a pin into a fix; it didn't happen.
- **The cost readout is arithmetically correct.** `648 + 171,232 + 491,962 + 62,140,591 =
  62,804,433` ✓, priced per component at Sonnet anchors ($3/$15, cache-write ×1.25, cache-read
  ×0.10) = **$23.0575**, recorded as `$23.06` ✓. **Do not re-price it.**
- **Both tests assert all three claims**, not just the format sniff: `guess_format` on the written
  bytes, `assert_ne!` against the raw SVG fixture, and the reported-format check. The `apply` test
  additionally decodes and asserts 40×30.
- **The exact-`WebP` assertion is house style, not an outlier.** Build Completion's reflection says
  SPEC-115's tests "only assert sniffs as *some* raster format" — that is true of
  `input_svg.rs:232` and `:282`, but **not** of the closest sibling at `:85-86`, which asserts
  `ImageFormat::WebP` exactly. SPEC-117 matches the nearer precedent.

## AC-5 is this spec's load-bearing criterion — and it looks properly done

Neither test is red on `HEAD`, so **the per-verb negative control is what proves they are real
pins rather than vacuous passes.** The build ran two genuinely independent reverts, each
simulating a different failure mode the spec named:

| revert | simulates | build test | apply test |
|---|---|---|---|
| `encode_one_optimize_decided` → raw passthrough | "a wrapper that stops calling through" | **RED** | GREEN |
| `run_apply`'s `pinned` forced `true` | "an early return" | GREEN | **RED** |

That is the correct shape — each test sensitive to its own verb's delegation site, neither riding
on the other. **This is what SPEC-113 failed to do and SPEC-116 did do.** Confirm it, ideally by
re-running at least one of the two reverts yourself.

**One thing to scrutinise:** the build proved the reverts reached the artifact by noting that
`cargo test` recompiled each time, rather than by comparing binary hashes the way SPEC-116's
verify did. Arguably that is *stronger* evidence — the test actually flipped RED, which is direct
behavioural proof the change took effect, whereas a hash only shows a rebuild happened. **Rule on
whether you agree**, and say which standard you think this repo should hold.

## The three things genuinely open

### 1. The AC-6 baseline was established by counting, not by running

Build Completion says the `main` baseline came from
`git show HEAD:tests/*.rs | grep -c '#\[test\]'` — 25→26 in `tests/build.rs`, 7→8 in
`tests/input_svg.rs`. That is a **static count of source attributes**, not a measurement of what
executes. It cannot see a test that exists but does not run (cfg-gated, ignored, or never reached).

The build *did* also run the full suite on all three legs (36/36 suites ok). But the **delta**
claim — "exactly the two new tests, nothing else moved" — rests on the grep.
**Establish your own baseline by running `main`**, per house standard
[[a-number-from-an-unproven-path-is-not-a-measurement]].

### 2. `build`'s AC-4 is satisfied by an interpretation — rule on it

AC-4 says *"the reported format matches the bytes — assert the summary/`--json` names the real
container."* `build` has no per-input summary line, so the build read "reported" as **the written
filename plus the lockfile entry**, and asserted both carry `.webp`.

That is a defensible reading and arguably the only one available for this verb. **But it is an
interpretation of a criterion, not the criterion as written** — decide whether it satisfies AC-4,
needs a `--json` assertion alongside, or should be recorded as a scoped deviation.

### 3. Is the pin durable, or does it encode today's codec race?

The build's own comment says WebP wins because a rasterized SVG's `source_format` reports as `png`,
which `fast_fallback_lossy_entry` does not match, so no lossy candidate ever competes. That is a
correct reading of *today's* shortlist logic.

**The question is what happens when that changes.** If a future encoder change makes AVIF win, both
tests go red for a reason that has nothing to do with the adopted-format defect they exist to
pin — and the next person deletes or loosens them. Consider whether the assertion should be "a
real raster that is not the source container" with the WebP expectation as a separate, clearly
labelled assertion. **Either answer is defensible; the spec asked for "a real WebP" and got it.**
This is the same tradeoff SPEC-116's verify ruled on for the golden-file question, and it deserves
the same explicit ruling rather than silence.

## The rest of the checklist

- **All 6 ACs walked**, completion table diffed against the actual test files.
- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref.** (The
  script now detects a clean tree and falls back to `origin/main...HEAD`, so both forms work; the
  base ref is still the reliable habit.) DEC-089 is the one that governs here.
- **Constraints:** `clippy-fmt-clean`, `one-spec-per-pr`. Note `test-before-implementation`
  **does not apply in its usual form** — see the spec's Failing Tests section. Do not treat the
  absence of a red-to-green transition as a finding.
- **New DECs:** build says none. Agree or don't.
- **`cost.sessions`** has a design entry (null-with-note) and a build entry (measured, verified).

## Guardrails

- **Own git worktree**, off the PR branch. Do not work in the primary checkout — `main` moved
  fourteen times in the last two days.
- **Do not fix what you find.** Verify reports; a punch list goes back to build.
- **Do not merge. Do not bump the version.**
- `git commit -s` (DCO). macOS has no `timeout(1)`. **A piped command reports the pipe's exit
  code** — redirect and read `$?`. **`rtk` can corrupt grep counts** — cross-check with
  `/usr/bin/git` or raw `grep` plus a positive control.
- **Budget: ~60 minutes.** This is two tests, no source changes, and a green PR. If you are past
  that and have not started the matrix, stop and report.

## When you finish, in this order

1. Append a verify cost session entry to `cost.sessions` (see below).
2. Run `just advance-cycle SPEC-117 ship`, and **CONFIRM it moved** — `git diff` on the spec should
   show the `cycle:` line change. It reports success even when it changes nothing.
3. Give the verdict, with the three open items above each explicitly ruled on.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** Price per component at the anchors of
the model `.message.model` reports (you are expected to be Opus: $5/$25 per MTok). **Measure at
session end.**

> **Datum worth having, for your own budgeting.** SPEC-117's build cost **$23.06** for two tests
> and no source changes — nearly **double** SPEC-116's build ($11.91) which was a real fix with six
> tests. Wall clock was 62 minutes against SPEC-116's 104. **Cost did not track time**, because the
> expense is in rebuilds: two per-verb controls plus a three-leg matrix with fresh target dirs.
> Budget your controls, not your minutes.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
