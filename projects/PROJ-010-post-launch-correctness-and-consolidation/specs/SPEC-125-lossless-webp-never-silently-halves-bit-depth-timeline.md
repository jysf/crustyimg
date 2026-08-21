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
- [ ] **build** — prompt: `prompts/SPEC-125-build.md` (2026-08-21). **Sonnet**, own worktree,
      branch `fix/spec-125-lossless-webp-never-silently-halves-bit-depth`. **DEC-097 reserved in
      the prompt** — DEC-096 exists only on SPEC-124's unmerged branch, so `next_id` cannot see it.
      **Unblocked**: SPEC-121 merged (`9075bc3`).
      ⛔ **Gated on PR #184 merging, and the prompt says stop-and-report if it has not.** SPEC-124
      edits the same regions of `src/sink/mod.rs` — it inserts `AVIF_TILE_THREADS` immediately below
      SPEC-121's `eight_bit_downgrade_warning` block and rewrites both existing warn call sites.
      ⚠ **The spec's own line references have drifted** (`:216-226` is now the `SinkInput` struct;
      the format list is at `:247-268`) — the prompt corrects them and says to locate by name.
      ⚠ Call 1's `Tiff`/`Png` 16-bit-capable prior **cannot be settled by reading the dep's docs**
      [[a-grep-of-src-cannot-see-a-dependencys-default]]; the prompt requires it be driven
      behaviourally — encode, decode back, read the surviving depth — which is both AC-2's
      derivation and the mechanical check that keeps the set from going stale.
      **Reporting-only: no byte change** (AC-5/AC-6), which is what makes this the flexible spec
      in the release sequence — it does not have to precede the tag.
- [ ] **verify** — Opus, new session, read-only. ⚠ AC-6 (`optimize`'s candidate selection
      byte-identical to `main`) is the one that catches a reporting fix that quietly became a
      search change — drive it, do not read it.
- [ ] **ship**
