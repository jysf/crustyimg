# SPEC-134 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-134-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-134-frame.md`). **GO, M** (filed [S]). Premise confirmed
      and wider: every encoding verb ignores a template extension, and `web`/`optimize`/`apply
      --recipe web` write AVIF into `.png`/`.webp`; `build` already pins on terminal-`optimize`
      targets (DEC-087), so `apply` and `build` disagree on the same recipe + template. The rule
      being reversed lives in `docs/api-contract.md:490`, not in DEC-087.
- [ ] **design** — `prompts/SPEC-134-design.md`
- [x] **build** — prompt: `prompts/SPEC-134-build.md`. **Opus**, draft PR #202 at `635adfb`,
      $11.43, completed 2026-10-26. 22/22 ACs; 41-row table (28 RED baseline rows); NC-1…NC-10
      flipped; CI 16/16 green. DEC-105 updated at build for AC-22. NC-7 has two readings (verify
      judges).
- [~] **verify** — prompt: `prompts/SPEC-134-verify.md`. **Opus**, read-only.
- [ ] **ship**
