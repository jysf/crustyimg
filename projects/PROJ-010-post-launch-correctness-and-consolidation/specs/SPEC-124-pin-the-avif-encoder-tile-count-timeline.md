# SPEC-124 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-124-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-18. 9 ACs, 5 settled design calls, 2 failing tests.
      Framed on the **maintainer's ruling to pin now and ride SPEC-121/122's wave** rather than
      pay a second lockfile migration later. Closes both riders SPEC-123 measured (DEC-094): the
      core-count variance behind STAGE-042's `[env]`/`diff` false positive, and the multi-tile
      compression penalty the shipped build pays for tiles that buy **no parallelism** — `ravif`
      is compiled without `threading`, so the encode is serial.
      ⚡ **N is deliberately NOT settled at design.** The prior is that N=1 may be strictly better
      (same speed, materially smaller files) — but it is stated as a **prior to measure**, not a
      conclusion, because SPEC-123 cost $60.33 largely for a design-time prediction asserted with
      confidence. Call 2 lists what must be driven, including the forward cost of N=1 if
      `image/rayon` is ever enabled.
      ⚠ **Blocked on SPEC-122 merging.** Same wave; the migration is keyed on `crate::version()`,
      so what makes it one migration is landing in the same **release**, not the same PR.
- [ ] **build** — prompt: `prompts/SPEC-124-build.md` (2026-08-20). **Opus**, own worktree, branch
      `fix/spec-124-pin-the-avif-encoder-tile-count`. **DEC-096 reserved in the prompt**, not left
      to `next_id`. The spec's `implementer` was updated to Opus to match — SPEC-122's prompt said
      Sonnet while the dispatch used Opus, and the cycle had to flag the mismatch itself.
      **Unblocked 2026-08-20**: SPEC-121 (`9075bc3`) and SPEC-122 (`2bd74b0`) are both merged.
      ⚠ **Deadline-bound.** The shared lockfile migration only stays *one* migration if this lands
      in the same release as 121/122 — the key is a function of `crate::version()`, so the window
      closes at the next tag.
      Carries three lessons the earlier prompts in this wave lacked: **never poll CI** (SPEC-122's
      build spent ~$60 of $103.60 there, from a prompt that carried no CI instruction at all); **a
      green local matrix does not predict CI** (twelve local exit-0s against eight red CI legs when
      stable floated to 1.98); and **list every file the diff touches** (SPEC-122's Deviations
      claimed `src/operation` + `tests/` only and was wrong by two `scripts/` files, which left
      `affected_scope` blind).
- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
