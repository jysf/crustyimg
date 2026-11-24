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
- [x] **verify** (2026-11-23, Opus, $12.19) — PUNCH LIST at `9733cd8`. The coverage map was driven in
  full; every refusal message matches byte for byte; FT-29/FT-30 10/10; CI green incl. Windows.
  **Two claims:** a source is overwritten on APFS through `ß`/`SS` or NFC/NFD (exit 0), and a glob
  source writing into its own folder builds once, then exits 2 on every later build (new in this PR).
  **One assert:** UX1–UX3 turn no rows red, and six refusals have no row. **Wording:** exit 6, not 5,
  in `SECURITY.md`; the M12 cell; the reflection's "one component". Three pre-existing findings were
  filed in `guidance/ideas.yaml`.
- [ ] **punch list** — `prompts/SPEC-157-punchlist.md` (Sonnet). Maintainer rulings: close the
  overwrite with a same-file (device + inode) check, no Unicode dependency, leaving output-to-output
  `ß`/NFC as a residual; refuse the glob case on the first build.
- [ ] **re-verify** — narrow, after the punch list (two claim items).
  ⛔ #270 is not merged until the re-verify reports.
- [ ] **ship**
