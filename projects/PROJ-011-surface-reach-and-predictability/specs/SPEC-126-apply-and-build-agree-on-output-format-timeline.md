# SPEC-126 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-126-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-23. 7 ACs, 4 settled design calls, 4 failing tests. **PROJ-011's entry
      point**, framed from a defect driven on `main` rather than reported.
      ⚡ **Call 1 was settled by measurement, not argument.** Every other pixel verb —
      `resize`, `thumbnail`, `watermark` — plus `build` plus `apply`-at-2-inputs all preserve the
      source format; **`apply` at one input is the sole outlier on the entire surface**, so it is
      the one that moves. The opposite case is arguable (PNG avoids JPEG→JPEG generation loss) and
      loses because consistency across six paths beats a local optimum on one — and because
      changing `build` would invalidate every existing lockfile.
      ⚠ **Byte-changing, and explicitly must not ship alone** — it batches into PROJ-011's single
      migration with STAGE-050.
      📌 **Call 4 is the one most likely to be got wrong:** the test asserts that `apply` and
      `build` AGREE, not that `apply` writes `.jpg`. Pinning the format string pins the answer
      instead of the property, and would go green again the day someone changes the default for a
      good reason.
- [ ] **build** — prompt: `prompts/SPEC-126-build.md` (2026-08-23). **Sonnet**, own worktree,
      branch `fix/spec-126-apply-and-build-agree`. **DEC-098 reserved in the prompt** — `next_id`
      scans only the working tree and has collided here before.
      ⛔ **Byte-changing: the prompt says do not bump the version, do not cut a release.** It
      batches into PROJ-011's single migration with STAGE-050.
      Carries the three traps this repo has paid for most recently: never poll CI (and the
      `--watch` summary line is unreliable — read the direct snapshot at the true head SHA);
      `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
      stdout and do not try to fix it; and list every file from `git diff --name-only`, not recall.
- [ ] **verify** — Opus, new session, read-only.
- [ ] **ship**
