# SPEC-110 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-110-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-03. Drove a purpose-built `Orientation=6` fixture (stored 1200×800,
      correct display 800×1200) through every pixel-lane verb on a release build at `d854038`
      before writing anything. **The sweep STAGE-039 framed as a check is where most of the
      defect lives:** `convert` (both formats), `resize`, `thumbnail`, `responsive`, and `edit`
      without its flag — **seven invocations return a sideways image, and every one also drops
      the EXIF**, so the information needed to correct the output is destroyed by the same
      operation that made it wrong. `web`, `optimize`, `auto-orient` and `edit --auto-orient`
      bake correctly. Full table in the spec's Context.
      **The current split is not a design** — nothing distinguishes the two groups except which
      pipeline builder each verb happened to call (`optimize_pipeline()` pins `auto-orient`
      first; every other handler builds its own without it).
      **DEC-003's own falsifiability condition is currently false.** It wrote its success test
      as *"Right if: a resize preserves orientation…"* and asserts *"Orientation/ICC survive
      transforms"*; `AGENTS.md:448` repeats it. A `resize` today neither preserves the tag nor
      bakes it — a decision record that has stopped describing the code, the same decay that
      made the launch board read red for two weeks.
      **Why nothing caught it:** all five callers of the orientation fixture builders outside
      `tests/common` are on verbs that already bake (`auto-orient`, `optimize`) or on lint. **No
      test asserts orientation behaviour on any of the five broken verbs.**
      **Maintainer decision: bake everywhere** — pin the existing `auto-orient` operation first
      on every pixel-lane verb. Rejected: preserve-the-tag (more faithful to DEC-003 on paper,
      but needs per-format container-lane writes and still renders sideways in EXIF-ignoring
      viewers) and split-by-verb-intent (two rules, and the seam is where the next bug hides).
      The "convert must stay byte-faithful" objection is weaker than it looks: `convert` already
      discards all metadata, so it is not faithful in any archival sense today.
      **Sub-decision:** `edit --auto-orient` becomes an accepted, documented no-op — it cannot be
      removed (CLI frozen, STAGE-030) — and **no opt-out flag is added** (filed, not built, on
      DEC-063's `--max-pixels` precedent).
      Wrote 11 acceptance criteria and 9 failing tests plus a negative control. Two traps
      called out explicitly: **a double rotation** is the obvious failure mode (AC-2), and **a
      square fixture would make the whole spec vacuous** (AC-4).
      **Un-metered main-loop cycle** (AGENTS §4): one fixture build, ~15 driven invocations on a
      release binary, plus an audit of the five existing orientation-fixture callers.

- [ ] **build** — run `prompts/SPEC-110-build.md` in a **fresh session**, own git worktree.
      Sonnet. Touches shared pipeline construction, so AC-11's clean full matrix is required —
      and **read the CI legs**, not just the local run (SPEC-107 shipped a red Windows leg
      behind a "matrix clean" claim).

- [ ] **verify** — fresh session, **Opus**. Re-derive the table yourself on your own builds of
      branch and `main`; do not inherit it. Drive every verb rather than reasoning from the call
      graph — SPEC-107's follow-up list was wrong in both directions until verify drove 16
      invocations.

- [ ] **ship** — bookkeeping on `main` after the PR merges: cost totals, reflection,
      `just archive-spec SPEC-110`, stage backlog. STAGE-039 also holds SPEC-111 and a doc
      chore, so shipping this does **not** close the stage.
