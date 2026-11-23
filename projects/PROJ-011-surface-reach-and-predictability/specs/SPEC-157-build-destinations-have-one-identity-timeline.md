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
- [x] **build** (2026-11-23, Sonnet, $5.84 measured before its last commit): draft PR #270, head
  `9733cd8`; CI running at report time (13 passed, 5 in progress, none failed; orchestrator-read).
  Baseline red as predicted (FT-5/19/20/27 green). 24 ACs met; AC-22 rests on the Windows leg.
  Mutations match except M12, which also reddens FT-28 (table cell). Upgrade drift driven once.
- [ ] **verify** — `prompts/SPEC-157-verify.md` (Opus). Applies AGENTS §15's coverage map after the
  fact. ⛔ #270 is not merged until verify (and any re-verify) reports.
- [ ] **ship**
