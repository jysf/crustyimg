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
- [x] **build** — 2026-08-17, Opus, PR #179, $46.17 / 215 min / 318 messages.
      Prompt: `prompts/SPEC-123-build.md`. DEC-094 was reserved in the prompt rather than left
      to `next_id` — no collision.
      **Verdict: Call 3's THIRD branch — the encoder ignores the thread setting.** `ravif` is
      compiled without its `threading` feature (reachable only via `image/rayon`, which
      `avif = ["image/avif"]` does not enable), so the encode is **serial** and the tile count is
      `available_parallelism()`. 18/18 cells identical; `--jobs` and `RAYON_NUM_THREADS` reach the
      batch pool on some verbs and the **encoder on none**. DEC-094. No `src/` change (AC-7).
      ⚠ **Two riders outrank the null:** AVIF output varies with the machine's **core count**,
      which is in neither the cache key nor the lockfile's `[env]`/caveat list — so `diff` can call
      a differently-cored machine a regression; and the shipped build takes the worst cell,
      **+1.5% / +47.9%** bytes vs a 1-tile encode at **5.7× / 4.4×** the wall clock of the same
      tiles in parallel. That **splits STAGE-042's pin item**: `image/rayon` is the performance
      lever, `with_num_threads(Some(N))` the determinism lever.
      ⚠ **Design predicted the opposite verdict, twice**, by quoting `image`'s doc comment without
      checking the feature set. Both errors are corrected in place on STAGE-042.
- [ ] **verify** — prompt: `prompts/SPEC-123-verify.md` (2026-08-17). Opus, new session, own
      worktree. Two ACs need an explicit ruling: **AC-6** (the thread axis is not falsified, but
      core-count variance is a cross-machine non-determinism the caveat list misses) and **AC-7**
      (its literal "no `src/` diff" blocked the doc-comment correction AC-6's spirit wanted).
      The prompt warns that the spec's own Inputs and the build prompt carry design's wrong
      predictions — the corrections are themselves under review.
- [ ] **ship**
