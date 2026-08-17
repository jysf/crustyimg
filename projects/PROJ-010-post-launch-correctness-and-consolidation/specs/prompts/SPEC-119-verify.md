# SPEC-119 — VERIFY prompt

Cycle: **verify**. **New session — do not continue from the build session.**

**What shipped:** PR [#176](https://github.com/jysf/crustyimg/pull/176), branch
`fix/spec-119-animated-input-never-silently-flattened`. 12 files, +1204/−82. 16/16 applicable CI
green. Animated GIF / APNG / animated WebP now warn on stderr instead of silently losing frames,
and `lint` stops recommending the command that loses them.

**Verdict:** ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Read in order

1. **The spec** — `.../specs/SPEC-119-animated-input-is-never-silently-flattened.md`, in full,
   including `## Build Completion`. **11 ACs.** Note that **Call 1 was ruled and Call 4 was
   revised after framing** — judge against the amended spec, not the original.
2. **DEC-093** — `decisions/DEC-093-animated-input-warns-and-proceeds-lint-is-the-strict-gate.md`.
3. **The diff.** `src/image/mod.rs` (+180, the detection and the flag), `src/lint/rules.rs` (+97,
   the shared read and the new rule), `tests/animated_inputs.rs` (new, 255), `tests/lint.rs`
   (+155), `tests/common/mod.rs` (+198, native fixture builders).
4. **`docs/backlog.md`**'s animated-input entry — the driven evidence this fixes.

## Work already done for you — confirm, don't redo

- **Cost is arithmetically correct.** `942 + 341,592 + 919,223 + 142,209,098 = 143,470,855` ✓,
  priced per component at Sonnet anchors = **$51.2365** against `$51.24` recorded ✓. 99.1% cache
  reads. **Do not re-price.**
- **The DEC was renumbered by the orchestrator, not the build.** It shipped as DEC-092, colliding
  with SPEC-120's DEC-092 on a parallel branch — different filenames, so both would have merged
  cleanly into duplicate ids. Renumbered to **DEC-093** in `d66a357` (the DEC file, its `id:` and
  heading, 2 refs in `docs/api-contract.md`, 5 in the spec). **Confirm the renumber is coherent
  and that no surviving `DEC-092` reference on this branch means the animated decision** — the
  SPEC-120 references in `prompts/` and the SPEC-120 timeline are correct and must stay.
- **`IMAGE_EXTENSIONS` is missing `webp`** — the build's out-of-scope finding. Confirmed on `main`
  at `src/source/mod.rs:105-113` and **already filed on STAGE-042**. Do not re-file it; do
  confirm it was right to leave out of this PR.

## What the build did that the spec did not ask for — and it is better

`lint`'s rule now reads **`Image::is_animated_input`** instead of re-decoding
(`gif_is_animated` is gone). The original defect was precisely that *the linter knew the file was
animated and the encoder path did not*. Sharing one flag makes that divergence **structurally
impossible** rather than merely fixed.

**Confirm the claim rather than admiring it:** is there any remaining path where `lint` and the
pixel path could disagree about whether an input is animated?

## The five things to scrutinise

### 1. AC-7b — the strict path must be DRIVEN, not asserted

`lint --max-warnings 0` must exit **non-zero** on all three families, with static counterparts
staying clean. **This is the whole answer to the maintainer's recorded reservation** that
warn-and-proceed still loses data — the ruling was accepted *because* `lint` is the gate.

Drive it yourself for GIF, APNG and animated WebP. **An answer nobody drove is not an answer.**

### 2. The rule-id decision — rule on it

Call 4 left the choice open. The build chose **a separate `format/animated-input` rule** rather
than broadening `format/animated-gif`, and justified it as avoiding a config migration.

Defensible — but check the consequence: a user who set `format/animated-gif = "off"` still gets
warnings from the new rule on APNG/WebP. Is that right? (Probably — different formats, different
rule — but say so.) And confirm **no config surface actually needs a migration note**, which is
what the spec said to stop and report on.

### 3. AC-6 — the assertions must be structural, never the score

SSIMULACRA2 compares frame 1 to frame 1, so any test asserting "the score stayed high" is
**vacuous by construction**. Read `tests/animated_inputs.rs` and confirm the frame-count proof is
structural (`ANMF` chunks, or a decode-and-count). **A single score-based assertion is a punch
list item.**

### 4. AC-9 — three independent negative controls

Revert GIF, APNG and WebP detection **separately**; each family's test goes RED, the static
controls stay green. **The evidence is the behavioural flip, not a binary hash** — measured in
this repo on 2026-08-16, a rebuild from byte-identical source produces a different binary, so a
hash proves only that a relink happened. Re-run at least one control yourself.

### 5. `#[allow(clippy::type_complexity)]` — a deliberate carve-out, so judge it

`optimize_decide_one` now returns a **5-tuple** `(OptimizeOutput, Option<Trace>, Option<f32>,
bool, bool)` and the build suppressed clippy rather than introducing a small struct.

`clippy-fmt-clean` is a blocking constraint, and an `#[allow]` is the one way to satisfy it
without satisfying its intent. Two bools side by side in a positional tuple is also exactly the
shape that invites a future call-site swap. **Rule: acceptable as-is, or a punch-list item for a
named struct?** Either is defensible; silence is not.

## The rest of the checklist

- **All 11 ACs walked**, completion table diffed against the actual test files.
- **AC-8 byte-identity for non-animated inputs** — compare against `main`'s binary, not a sibling
  verb on the same branch.
- **AVIF**: the build settled it by reading `avif-parse`'s source (animated AVIF hard-rejected
  before pixel decode, so no code needed). **Confirm that read** — the spec's alternative was to
  declare AVIF unproven, and "no change needed" is a stronger claim than "not tested".
- **A cost the spec did not consider:** `detect_animated_input` now runs on **every** decode. APNG
  and WebP are header reads, but the GIF path does `into_frames().take(2).count()`, which decodes
  up to two frames. Is that measurable on a large GIF-heavy batch? Not an AC — **report it either
  way**, since nobody asked at design.
- **Decision drift:** `./scripts/decisions-audit.sh --changed main`. DEC-085 governs the
  unconditional gating; note it now has a sibling rule in DEC-093.
- **Constraints:** `clippy-fmt-clean` (see item 5), `test-before-implementation` (several tests
  were red on `main` — verify that), `one-spec-per-pr`, `no-unwrap-on-recoverable-paths` — the
  build uses `is_apng().unwrap_or(false)`, which is compliant; confirm no bare `unwrap` crept in.
- **`cost.sessions`**: design null-with-note, build measured.

## Guardrails

- **Own git worktree**, `--detach` at the PR tip if the build worktree still holds the branch.
- **Do not fix what you find.** Verify reports.
- **Do not merge. Do not bump the version.**
- `git commit -s` (DCO). macOS has no `timeout(1)`. **A piped command reports the pipe's exit
  code.** **`rtk` can corrupt grep counts** — cross-check with `/usr/bin/git` or raw `grep`.
- **Budget: ~250 message exchanges, not minutes.** Measured across this wave: cost scales with the
  **square** of message count and anti-correlates with wall clock — SPEC-116 ran 104 minutes for
  $11.91, this spec's build ran 61 for $51.24. If you pass ~250 exchanges without having started
  the matrix, checkpoint and report.

## When you finish, in this order

1. Append a verify cost session entry to `cost.sessions`.
2. Run `just advance-cycle SPEC-119 ship`, and **CONFIRM it moved** (`git diff` shows the `cycle:`
   line change; it reports success even when it changes nothing).
3. Give the verdict, with items 1–5 each explicitly ruled on.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by content, never
by recency. Price per component at your own model's anchors (expected Opus: $5/$25 per MTok).
**Measure at session end** — mid-session readings in this project have run 40% and 49% low.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.
