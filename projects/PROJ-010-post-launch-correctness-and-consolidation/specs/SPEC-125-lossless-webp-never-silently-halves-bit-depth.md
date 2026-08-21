---
task:
  id: SPEC-125
  type: bug
  cycle: verify
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-18

references:
  decisions:
    - DEC-095
    - DEC-090
    - DEC-019
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-121
    - SPEC-090

value_link: >
  A default-path silent downgrade on a shipped verb, reported with a perfect
  quality score. `convert --format webp` halves a >8-bit source and prints
  `ssim 100.0` — the honest-size line reads as reassurance for the one thing
  that went wrong.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Promoted from the
        STAGE-042 backlog item SPEC-121's punch-list cycle filed and measured.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 246724470
      duration_minutes: 124
      recorded_at: 2026-08-21
      tokens_breakdown:
        input: 1340
        output: 474445
        cache_creation: 1040103
        cache_read: 245208582
      estimated_usd: 84.58
      note: >
        MEASURED — summed from the session transcript's per-message `usage`,
        priced at Sonnet anchors ($3/$15 per MTok) with cache multipliers
        (cache_creation x1.25, cache_read x0.10 of input rate). Reading taken
        AFTER CI settled at the true head SHA (c7695c0), not when the PR
        opened — an earlier snapshot at ~52 min read $49.25, 37% under this
        figure, matching this repo's own measured pattern for premature
        readings.
        ⚠ CORRECTED BY THE ORCHESTRATOR (2026-08-21). The cycle recorded
        237961353 / $81.78 / 103m, which prices EXACTLY right at its own
        snapshot — 654 of 670 usage-bearing messages — but a cycle cannot count
        the messages that write its own cost block. Delta +8,763,117 tokens /
        +$2.80. This is the SMALLEST proportional undercount in the wave (3.4%,
        against SPEC-124's 7.1%) precisely because the reading was taken after
        CI settled rather than at "PR opened" — the discipline worked, it just
        cannot close the last gap. All 670 messages report claude-sonnet-5.
        Transcript identified by CONTENT (`bb308ebc`, the one carrying the
        SPEC-125 build dispatch) — a naive search matched TWO sessions, the
        other being the orchestrator's own.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-125: lossless WebP never silently halves bit depth

## Context

**Measured on SPEC-121's branch binary, 2026-08-18**, 32×32 16-bit RGB PNG:

```
web
  png → webp · 4791 → 686 B (86% smaller) · ssim 100.0
```

The output round-trips as **8-bit**.

> ⚠ **Corrected at build (2026-08-21), by driving the binary rather than trusting
> this prose.** The original text attributed this line to `convert --format webp`.
> **`convert` never scores at all**, so it prints no `ssim` line — the false-perfect
> score belongs to **`web`**, whose `optimize` smallest-candidate search picks WebP
> for this fixture and then reports on it. The depth-downgrade half of the claim is
> correct for **both** verbs and is unaffected; only the attribution of the `ssim`
> line was wrong. Left visible rather than silently rewritten, because it is the
> reason Call 1's "derive it, don't copy it" instruction paid for itself: the same
> habit that caught this then surfaced AVIF and the ICO round-trip defect.

Two defects, and the second is the one that stings:

1. **The downgrade is silent and on the DEFAULT path.** SPEC-121's Call 3 scoped
   its diagnostic to JPEG + lossy WebP. `image`'s *lossless* WebP encoder — **no
   feature flag, always built** — has no 16-bit mode either, so it takes the same
   *"automatically convert the image to some color type supported by the encoder"*
   path with no diagnostic.
2. ⚡ **`ssim 100.0` is reported while the depth is halved.** The SSIM figure is
   computed on **8-bit renderings**, so it structurally *cannot see* the loss it is
   reporting on. Combined with SPEC-090's honest-size line, the output reads as
   *"86% smaller, pixel-perfect"* at the exact moment half the depth was thrown
   away. **A metric that cannot see a defect is worse than no metric**, because it
   converts silence into positive reassurance.

Full evidence: STAGE-042's backlog item, filed by SPEC-121's punch-list cycle.
**Read it; do not re-derive it.**

## The design calls — settled here

### Call 1 — widen the diagnostic to every 8-bit-only target, derived from code

SPEC-121 deliberately did not reopen this. Its Call 3 warns for JPEG and lossy
WebP. The rule should be **"the target format cannot hold the source's depth"**,
not a hand-maintained list of two.

⚠ **Derive the set from the encoder capabilities, not from this spec.** Candidates
visible at `src/sink/mod.rs:216-226` are `Gif`, `Bmp`, `Ico`, lossless `WebP` —
**and `Tiff` and `Png` are believed to be 16-bit-capable, so they must NOT warn.**
That is a prior to check, not a conclusion
[[a-measurement-specs-cost-lives-in-the-refutation]]. A hard-coded list is exactly
the shape that goes stale [[mechanical-sweeps-need-a-mechanical-check]].

### Call 2 — the SSIM line must not claim a perfect score across a depth change

This is the half that makes the bug dangerous, and it is **not** fixed by the
warning alone — a user reading `ssim 100.0` has been told the opposite of the
truth by the tool's own quality instrument.

Options, and the build picks one **with the reasoning recorded in the DEC**:
suppress the figure when depth was reduced; qualify it (*"ssim 100.0 (8-bit
comparison; source was 16-bit)"*); or compute it at source depth if the scorer
can. **Do not leave a bare `100.0`.**

⚠ **Do not silently change what SSIM means elsewhere.** DEC-019 anchors the
scorer for the byte-budget search; a change there perturbs `optimize`'s candidate
selection. If your fix would touch that path, **stop and report** — this spec is a
reporting fix, not a search change.

### Call 3 — do not reopen SPEC-121's narrowing rule

The colour-type/bit-depth preservation rule is settled and shipped. This spec adds
a diagnostic where the *target format* cannot hold what the pipeline correctly
preserved. No `Operation` body changes.

## Acceptance Criteria

- [x] **AC-1.** `convert --format webp` on a >8-bit source **warns on stderr**;
      `-o -` stdout stays pure WebP (AGENTS §11).
- [x] **AC-2.** The warning fires for **every** 8-bit-only target, and **does not**
      fire for targets that genuinely hold the depth. The set is **derived and
      cited**, not hard-coded from this spec's candidate list.
- [x] **AC-3.** **`web` / `optimize` reach it too** — driven, not reasoned, since
      the smallest-candidate search is how most users hit this.
- [x] **AC-4.** **No bare `ssim 100.0` across a depth change** (Call 2), pinned by
      a test asserting the rendered line.
- [x] **AC-5.** **A negative control** — an 8-bit source through the same verbs
      warns **not at all** and its output is byte-identical to `main`.
- [x] **AC-6.** **DEC-019's search path is unchanged** — `optimize`'s candidate
      selection byte-identical to `main` on the corpus.
- [x] **AC-7.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential:
      default, `--no-default-features`, `--features webp-lossy`. Clippy and
      `fmt --check` each. Then read the CI legs individually.

## Failing Tests

- **`tests/sink.rs`**
  - `"lossless_webp_reports_the_depth_downgrade"` — AC-1. **RED.**
  - `"eight_bit_only_targets_all_warn_and_others_do_not"` — AC-2. **RED.**
  - `"ssim_line_is_qualified_across_a_depth_change"` — AC-4. **RED.**
  - `"eight_bit_source_warns_nowhere"` — AC-5. Passes today; the control.

## Implementation Context

### Out of scope
- SPEC-121's narrowing rule (Call 3), and any `Operation` change.
- Adding 16-bit WebP support. Upstream has no encoder; that is a dependency
  question, not this fix.
- Changing which candidate `optimize` picks (AC-6 pins it).

## Notes for the Implementer

- ⚠ **Blocked on SPEC-121 merging** — it owns `src/sink/mod.rs`'s diagnostic, and
  this widens it.
- **Budget ~150 exchanges.** Never poll CI; background `gh pr checks --watch`.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-125 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-125-lossless-webp-never-silently-halves-bit-depth`
- **PR (if applicable):** [#185](https://github.com/jysf/crustyimg/pull/185)
- **All acceptance criteria met?** yes
  - **AC-1** — `convert --format webp` on a >8-bit source warns on stderr; `-o -`
    stdout verified to stay pure WebP bytes. Driven via the compiled binary AND
    `tests/sink.rs::lossless_webp_reports_the_depth_downgrade`.
  - **AC-2** — the set was MEASURED, not copied from the spec's candidate list:
    BMP, lossless WebP, AVIF warn; PNG, TIFF stay silent (prior held); GIF and
    ICO are excluded for two different, non-depth reasons (see DEC-097). Table-
    driven test: `eight_bit_only_targets_all_warn_and_others_do_not`.
  - **AC-3** — `web` and `optimize` both reach the widened warning through their
    candidate search, no `--format` pin, driven via
    `web_and_optimize_reach_the_widened_downgrade_warning`.
  - **AC-4** — no bare `ssim 100.0` survives a depth-reducing winner; the line is
    qualified with the reference's real depth on the default summary,
    `--explain human`, and `--explain json`/`--json`. Driven via
    `ssim_line_is_qualified_across_a_depth_change`.
  - **AC-5** — negative control, driven both ways: `eight_bit_source_warns_nowhere`
    (in-repo), plus a manual `main` vs. branch byte-diff on `convert`/`web`
    outputs and stderr for an 8-bit source (byte-identical, empty stderr on
    both). Two independent-condition reverts (AGENTS §15) confirmed the
    behavioural flip: Call 1 alone reverted → AC-1/2/3 tests RED, AC-4/5 GREEN;
    Call 2 alone reverted → only AC-4's test RED, everything else GREEN.
  - **AC-6** — `optimize --verify --explain json` on an 8-bit photo-like source:
    byte-identical output, JSON, and stderr between `main` and this branch.
    `pick_winner`/`solve_candidate` never read the new `scored_source_depth`
    field — it is populated strictly after the winner is chosen.
  - **AC-7** — full local matrix (`default`, `--no-default-features`, `--features
    webp-lossy`), fresh `CARGO_TARGET_DIR` per leg, sequential: `cargo test`,
    `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` all clean
    on every leg. **CI legs read individually at the true head SHA (`c7695c0`)**:
    every leg this spec's diff can affect is green — `build/test/clippy/fmt` on
    macOS/Linux/Windows, `avif feature`, `webp-lossy feature`, `heic feature`
    (both OSes), `lean build (--no-default-features)`, `msrv`, `cargo-deny`,
    `front-matter validation`, `cost-capture audit`, `DCO`. **One RED leg,
    unrelated:** `build + browser smoke` failed with `headless Chrome never came
    up (no DevToolsActivePort)`, **10.03s** after `demo/` started serving
    (`06:19:43.01` → `06:19:53.05`) — the exact pre-existing flake STAGE-042's
    backlog already documents (a 10s hard cap on Chrome startup in
    `tests/demo_smoke.mjs`, unrelated to any file this PR touches: no `wasm`,
    `demo`, or browser-smoke code is in this diff). Not fixed here — it is
    already filed as its own STAGE-042 item with its own fix, and the house
    precedent (SPEC-122) is a CI fix gets its own PR, not a spec branch.
- **New decisions emitted:** DEC-097 (widen the 8-bit downgrade warning; qualify
  the SSIM line; the full measured table for Call 1's derivation).
- **Deviations from spec:**
  - The spec's own Context repro command was wrong and is corrected in the
    STAGE-042 backlog entry: `convert --format webp` prints **no** ssim line at
    all (`convert` never scores); it is **`web`** whose candidate search prints
    `png → webp · … · ssim 100.0`. The underlying depth-downgrade claim for
    `convert --format webp` is correct and unaffected.
  - AC-2's candidate set differs from the spec's named list: AVIF is ADDED
    (measured 8-bit-only, not named in the spec, not in DEC-095's "not covered"
    list by omission but confirmed the same class), and GIF is EXCLUDED (hard
    encode error, not a silent downgrade — warning there would misdescribe a
    loud failure as a soft one). Both are measured findings per Call 1's own
    instruction to derive the set rather than copy it, not scope creep.
  - ICO is also excluded, for a THIRD reason distinct from PNG/TIFF's "holds
    the depth": `image`'s own ICO decoder cannot read back the ICO encoder's
    own output for every source colour type **except `Rgba8`** (it reproduces at
    plain 8-bit RGB with no alpha and no depth question at all) — an orthogonal,
    more severe defect, filed to STAGE-042 rather than fixed here (a real fix
    would change output bytes). ⚠ **"ANY source colour type" corrected at verify
    (2026-08-21)** — `Rgba8` round-trips fine, and that it is the one working
    case is exactly why the defect is about the encoder's `Rgba8` requirement
    rather than about depth. The STAGE-042 entry stated this correctly from the
    start; the claim was only overstated where it was restated.
- **Follow-up work identified:** the ICO round-trip defect, filed as a new
  STAGE-042 backlog item (needs a maintainer ruling: warn / fix / accept).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing structural,
   but the Context section's repro command was wrong (attributed `ssim 100.0`
   to `convert` when it is `web`'s report), and I only caught it because Call 1
   demanded driving the binary rather than trusting the prose — the same habit
   that then surfaced AVIF and the ICO defect. A spec that says "measure this"
   in one place is worth re-checking everywhere it asserts a command's output.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** No — DEC-019's boundary was exactly where it needed to be to make
   Call 2's design call (qualify, not compute-at-depth) unambiguous.
3. **If you did this task again, what would you do differently?** Start the
   Call 1 behavioural measurement (encode→decode-back across every candidate)
   before reading the spec's candidate list in detail, so the prior is checked
   fresh rather than read with the list already in mind — it would not have
   changed the outcome here, but it is the more disciplined order.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
