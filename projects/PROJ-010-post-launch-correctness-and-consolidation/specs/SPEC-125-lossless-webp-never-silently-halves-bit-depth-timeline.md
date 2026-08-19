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
- [ ] **build** — prompt not yet written; write it once SPEC-121 has merged. Sonnet, own worktree.
- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
