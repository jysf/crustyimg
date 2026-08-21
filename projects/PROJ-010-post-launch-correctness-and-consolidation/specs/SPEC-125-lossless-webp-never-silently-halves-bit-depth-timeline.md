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
- [x] **build** — 2026-08-21, PR #TODO (opened after this commit lands). **Sonnet**, own worktree,
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
- [ ] **verify** — Opus, new session, read-only. ⚠ AC-6 (`optimize`'s candidate selection
      byte-identical to `main`) is the one that catches a reporting fix that quietly became a
      search change — drive it, do not read it.
- [ ] **ship**
