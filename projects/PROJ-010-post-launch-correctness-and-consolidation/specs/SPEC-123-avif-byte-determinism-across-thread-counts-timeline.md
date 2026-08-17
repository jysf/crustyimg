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
- [ ] **build** — prompt: `prompts/SPEC-123-build.md` (2026-08-16). Opus, own worktree, branch
      `chore/spec-123-avif-byte-determinism`. **DEC-094 is reserved in the prompt** rather than
      left to `next_id`, which cannot see a record on an unmerged branch. The deliverable is a
      judgment about whether a null result is real or an ignored setting.
      ⚠ The prompt carries a design-time finding the spec does not: **crustyimg never calls
      `with_num_threads`** (`src/sink/mod.rs:679`), so the lever is `RAYON_NUM_THREADS` for the
      serial verbs and `--jobs` for the two that read it — and **`--jobs` is silently ignored by
      `convert`**, which would have measured one thread count three times.
- [ ] **verify** — Opus, new session.
- [ ] **ship**
