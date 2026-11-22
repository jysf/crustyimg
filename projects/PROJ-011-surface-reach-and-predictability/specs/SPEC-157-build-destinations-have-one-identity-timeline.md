# SPEC-157 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-157-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO, [M]. Both review P1s reproduced on `main` (`out = "src"`
  replaced the source, exit 0; `alias/../dist` gave one file, two lock records, and a `--check` that
  passed). Wider class driven: links, `..` past a link, case, `out = "."`, recipe file and watermark
  asset overwritten, cross-target source, and a `--watch` self-rebuild loop through a linked `out`.
- [ ] **design** — the resolver, the overlap list, the symlink ruling and the new DEC; ACs and failing
  tests with the mutation rows from the frame. Starts after the maintainer rules on the frame's questions.
- [ ] **build** — after SPEC-154 has merged.
- [ ] **verify** — full OS matrix (Windows leg for `\\?\` and the root strip), `--no-fail-fast` red set.
- [ ] **ship**
