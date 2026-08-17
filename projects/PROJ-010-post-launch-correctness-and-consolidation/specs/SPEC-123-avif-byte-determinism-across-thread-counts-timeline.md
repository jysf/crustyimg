# SPEC-123 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-123-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-16. 8 ACs, 4 design calls, no failing tests (a measurement).
      Framed because **two roadmap items are gated on it** — encoder threading and
      `par_iter run_pixel_op` — and because three shipped things (`build --frozen`, the
      lockfile's `hash`, the cache key) already assume an answer nobody has measured.
      ⚠ Thread count is **not** a component of the cache key and **not** in the lockfile's
      list of things output stability is qualified against.
- [ ] **build** — prompt not yet written. Opus: the deliverable is a judgment about whether a
      null result is real or an ignored setting.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
