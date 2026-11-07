# SPEC-150 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-150-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-06, Opus 5.5, `prompts/SPEC-150-frame.md`) — GO, size M, rulings settled 2026-11-06 by the orchestrator on the frame's recommendations (see the design prompt):
  folder-with-no-images stays an error (exit 3, wording only); fold-ins 4, 5, 7, 8 in, 6 out; comments out of scope; DEC-003 amended now; cli-reference cleaned.
  Driven on `main` (0.8.2 debug): 13 folder-taking verbs × 4 folder shapes all say "input not found or unreadable" for a
  readable folder with no top-level image; 44/44 help pages and 4/5 completion scripts carry ids from 8 doc comments.
  Fold-ins 4, 5, 7 in; 6 out; 8 (DEC-003: ICC driven dropped too) in as docs-only, pending ruling.
- [ ] **design** — prompt: `prompts/SPEC-150-design.md`
- [ ] **build** — prompt: `prompts/SPEC-150-build.md`
- [ ] **verify** — prompt: `prompts/SPEC-150-verify.md`
- [ ] **ship** — prompt: `prompts/SPEC-150-ship.md`
