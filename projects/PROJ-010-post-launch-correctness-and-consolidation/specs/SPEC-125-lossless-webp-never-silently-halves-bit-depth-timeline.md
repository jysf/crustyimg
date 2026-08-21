# SPEC-125 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-125-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-18. 7 ACs, 3 settled design calls, 4 failing tests.
      Promoted from the STAGE-042 backlog item that **SPEC-121's punch-list cycle filed and
      measured** — `convert --format webp` halves a 16-bit source on the **default path, no
      feature flag**, and prints `ssim 100.0` while doing it.
      ⚡ **The second defect is the dangerous one.** SSIM is computed on 8-bit renderings, so the
      scorer **structurally cannot see** the loss it reports on. With SPEC-090's honest-size line
      the output reads as *"86% smaller, pixel-perfect"* at the moment half the depth was thrown
      away — a metric that cannot see a defect converts silence into positive reassurance.
      Call 1 widens SPEC-121's Call 3 diagnostic from a two-format list to the rule *"the target
      cannot hold the source's depth"*, **derived from encoder capability, not hard-coded** — and
      states the `Tiff`/`Png` 16-bit-capable assumption as a **prior to check**, not a conclusion.
      Call 2 forbids a bare `ssim 100.0` across a depth change; AC-6 pins DEC-019's search path so
      the fix cannot perturb `optimize`'s candidate selection.
      ⚠ **Blocked on SPEC-121 merging** — it owns the sink diagnostic this widens.
      📌 **The backlog item this promotes lives on SPEC-121's branch, not `main`.** Mark it
      `[x] **SPEC-125**` and bump STAGE-042's count **at SPEC-121's ship**, not before — editing
      STAGE-042 on `main` now would conflict with PR #181.
- [x] **build** — 2026-08-21, PR #185. **Sonnet**, own worktree,
      branch `fix/spec-125-lossless-webp-never-silently-halves-bit-depth`. Gate cleared (PR #184
      merged before branching). All 7 ACs met — see the spec's own `## Build Completion`.
      Call 1's set widened beyond the spec's own candidate list, MEASURED not assumed: BMP/lossless
      WebP/AVIF warn, PNG/TIFF stay silent (prior held), GIF/ICO excluded for two different
      non-depth reasons — GIF hard-errors instead of downgrading, ICO's own `image`-decoder
      round-trip is broken independent of depth (filed to STAGE-042, not fixed here). Call 2's SSIM
      qualifier lands without crossing the DEC-019 boundary — it reads the already-decoded images'
      own depth, never touches the scorer. DEC-097 records the full measured table. Also caught: the
      spec's own Context repro command was wrong (`convert` never prints an ssim line; that is
      `web`'s report) — corrected in STAGE-042's backlog entry, not restated here.
- [ ] **verify** — prompt: `prompts/SPEC-125-verify.md` (2026-08-21). **Opus**, new session,
      read-only. ⚠ **PR #185 MERGED to `main` at `2735f60` before verify ran** — review `main`
      against the pre-merge base `f35e28a`, not a branch. A finding here is a follow-up commit,
      not a blocked merge, so the prompt asks for plain statements rather than softened ones.
      Five things pre-settled so the cycle does not re-derive them: the build's cost, that
      `tests/colour_type_preservation.rs`'s change is comment-only (SPEC-121's guard is intact,
      diffed), the corrected Context repro, the absence of a lossy/lossless WebP double-warn
      (traced — the lossy arm returns before the fallback), and CI green at 15/15.
      ⚡ **AC-6 is the item the spec lives or dies on** and the prompt puts it first: the spec
      promised reporting-only, and the diff touches `src/cli/optimize.rs` and
      `src/analysis/decide.rs` — the candidate-search surface. If `scored_source_depth` can reach
      candidate ranking, a reporting fix became a silent byte-changer in a release about to be
      tagged. Drive the corpus at both commits and diff bytes AND chosen format per file.
      Also: the JSON contract gained a conditional `ssim_source_depth` key on a **published
      library**; and GIF's and ICO's exclusions each rest on a strong claim about `image`'s own
      behaviour — the ICO one (its decoder cannot read back its own encoder's output for ANY
      colour type) is severe enough to need reproducing.
- [ ] **ship**
