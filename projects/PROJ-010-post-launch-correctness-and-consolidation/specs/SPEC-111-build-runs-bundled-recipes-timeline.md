# SPEC-111 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-111-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-07. **STAGE-039 framed this as needing no design cycle; it does.**
      Drove the failure on a release build at `3dd8fa7` with a real manifest: `build` exits **1**
      with `unknown operation 'optimize'` on a bundled recipe **both by file path and by name**,
      while two controls pass — `apply --recipe web` writes a real AVIF from the same source, and
      a plain pixel recipe through `build` exits 0. So the fault is precisely the terminal marker.
      **The framing's fix ("wire in the strip helper") is necessary but not sufficient, and
      shipping only that would be a worse bug than the current one.** `encode_one`
      (`src/cli/common.rs:52`) hardcodes `img.source_format()` — *"no --format override in batch
      path v1"* — so stripping alone would run the pixel pipeline and then write the result in the
      **source** format, silently discarding the modernization the recipe exists to perform. The
      current failure at least fails loudly.
      **Design question 1 — what picks the output format?** Answered by copying `apply`'s existing
      rule rather than inventing one: `run_apply` (`optimize.rs:80-102`) skips the decision when
      the format is **pinned**, so that `apply --recipe web hero.jpg -o hero.png` matches
      `web hero.jpg -o hero.png`. **Decision: `build` uses the name template as the pin** — a
      literal extension (`{stem}.png`) pins and skips the decision; `{ext}` (incl. the default)
      lets the decision choose and expands to it. `build.rs:575` already contemplates
      literal-extension templates. One rule across `apply` and `build`, which is the lesson
      SPEC-110 paid three cycles for.
      Encouraging: **`build` already anticipates a post-decode extension** — `EXT_SENTINEL`
      (`build.rs:115`) exists because *"the real output extension is only knowable after a
      decode"*, and `lock_output_path` already takes `ext` as a parameter.
      **Design question 2 — the recipe divergence SPEC-110 introduced.** **Decision: `edit
      --save-recipe` records `auto-orient` explicitly**, rather than giving `apply`/`build` an
      implicit prefix. A recipe is a record of what happened; an implicit prefix would make a
      recipe no longer a complete description of its own behaviour, which is the pattern SPEC-110
      just spent three cycles removing. And the precedent is already in the repo:
      **`recipes/web.toml` names `op = "auto-orient"` as its explicit first step.**
      Wrote 11 acceptance criteria and 10 failing tests plus a negative control. Complexity
      rated **M**, not the S–M the stage assumed.
      **Un-metered main-loop cycle** (AGENTS §4): one manifest fixture, four driven invocations
      with two controls, and a trace of the format path through `encode_one` →
      `lock_output_path` → `EXT_SENTINEL`.

- [ ] **build** — run `prompts/SPEC-111-build.md` in a **fresh session**, own git worktree.
      Sonnet. Touches `encode_one`, which `apply` shares — AC-6 is the guard that `apply` and the
      plain-recipe path are unchanged.

- [ ] **verify** — fresh session, **Opus**. Re-derive the driven table yourself on your own
      builds of branch and `main`. Enumerate every path that builds a pipeline from a `Recipe`
      rather than trusting this spec's list — SPEC-110's roster omitted a verb and it cost a full
      extra build cycle.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-111`, stage backlog. STAGE-039 closes only once the
      `docs/data-model.md` chore also lands.
