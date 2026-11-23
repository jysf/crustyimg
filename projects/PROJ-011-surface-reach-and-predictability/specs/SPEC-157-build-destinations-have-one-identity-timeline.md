# SPEC-157 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-157-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus) — GO, [M]. Both review P1s reproduced on `main` (`out = "src"`
  replaced the source, exit 0; `alias/../dist` gave one file, two lock records, and a `--check` that
  passed). Wider class driven: links, `..` past a link, case, `out = "."`, recipe file and watermark
  asset overwritten, cross-target source, and a `--watch` self-rebuild loop through a linked `out`.
- [x] **design** (2026-11-22, Opus) — [M] holds, no split. One resolver (`out` canonicalized up to its
  longest existing prefix, lexical tail, root-relative, case-folded), one planning pass (out-of-tree link →
  collision → overlap with every file the build reads), `write_planned` re-resolving before each write, and a
  `--watch` read carve-out (a source edit inside `out` was missed on `main`, driven). 34 tests, mutation table
  driven on a prototype. New finding filed: NFC/NFD names are one file on APFS, two identities.
- [ ] **build** — `prompts/SPEC-157-build.md`. After SPEC-154 has merged and SPEC-155 has built.
- [ ] **verify** — full OS matrix (Windows leg for `\\?\` and the root strip), `--no-fail-fast` red set.
- [ ] **ship**
