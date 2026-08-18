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
- [ ] **build** — prompt not yet written; write it once SPEC-122 is at verify. Sonnet, own
      worktree. **DEC id to be reserved in the prompt**, not left to `next_id`.
- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
