# SPEC-154 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-154-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO, [S]. Premise confirmed: the classifier calls 20 tracked
  build/test inputs in 9 classes docs-only (the review's four plus `docs/github-actions.md`,
  `docs/data-model.md`, `README.md`, `.pre-commit-hooks.yaml`, `demo/`, `bench/corpus/`). Recommends
  inverting to a docs-only allow-list (fails safe; 12 of 600 replayed commits flip to a full run)
  and adding `recipes/**`, `assets/**` to `pages.yml`. Prompt: `prompts/SPEC-154-frame.md`.
- [x] **design** (2026-11-22, Opus) — allow-list `DOCS` (15 alternatives) + `EXCEPT` (README.md,
  docs/github-actions.md, docs/data-model.md); derived include check (35 targets, floor, canonical-path,
  concat!/env! refused) and pages drift check, all in `--self-test` (88 rows). 32 mutations driven red on a
  prototype; replay 12/600 flips. Prompt: `prompts/SPEC-154-design.md`.
- [x] **build** (2026-11-22, Sonnet) — draft PR #266. Both RED baselines matched the spec's lists; the
  script is byte-identical to the spec's block; M1–M32 each matched the table, no wrong cell. The PR's
  Linux `changes` job printed 88 rows, 0 failed (no GNU/BSD difference). Deviation: the id grep finds two
  fixture paths at `scripts/ci-code-paths.sh:161-162`. Prompt: `prompts/SPEC-154-build.md`.
- [x] **verify** (2026-11-22, Opus) — PUNCH LIST at `2a70156`. All 1,303 tracked paths through both
  scripts: 0 code → docs-only, 39 → code (the design's list); no docs-only file read by any gated job.
  Two items, neither changes a verdict: ship's proof edit would turn the pages smoke red (wording), and
  the include check reads only the plain-literal form (assert). Out of spec: the rename hole (STAGE-063),
  quoted non-ASCII paths and the include floor's drift (`guidance/ideas.yaml`). Prompt:
  `prompts/SPEC-154-verify.md`.
- [x] **punch list** (2026-11-22, Sonnet) — `80fc4bb`: ship's proof uses `recipes/gallery.toml`;
  `docs/development.md` and AC-9 narrowed to the plain-literal form; script unchanged. A second session
  from the same prompt found the head moved and stopped with no changes. No re-verify (no claim item).
  Prompt: `prompts/SPEC-154-punchlist.md`.
- [x] **merge** (2026-11-22, maintainer) — PR #266 as `f6e2daa` (2026-11-23T02:34:37Z).
- [x] **ship** (2026-11-23, Sonnet) — merged as `f6e2daa`, PR #266; proof PR #267 (`recipes/gallery.toml`-only draft) ran the full gates and was closed unmerged; AC-18 met; total cost $17.49. Prompt: `prompts/SPEC-154-ship.md`.
