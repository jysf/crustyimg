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
- [x] **punch list** (2026-11-24, Sonnet, $4.08 measured before its last commit) — PR #270 at
  `8a7b03c` (code at `7e1411c`, CI 18 passed / 8 skipped / none failed, orchestrator-read). Fix 1:
  a same-file (device + inode) check, `cfg(unix)` (FT-35/36/37, M15). Fix 2: a listing check for glob and
  folder sources, refused on the first build (FT-38–42, M16), with `READABLE_EXTENSIONS` copied from the
  source filter and pinned by FT-52. Pins FT-43–51 (M17–M19). Exit 6 in `SECURITY.md`; `SKILL.md`
  added to scope. STAGE-063's NFC/NFD entry extended to `ß`/`SS` by the orchestrator on `main`.
- [x] **re-verify** (2026-11-24, Opus, $3.43) — PUNCH LIST at `8a7b03c`, **no claims**. Both fixes
  driven on release builds of both sides (`ß`/`SS`, NFD/NFC, a link through `src`, chaining, `**`, a
  sub-folder manifest, a case-sensitive image); nothing wrongly refused (twice-built, `--watch`, sub-folder
  outputs, pinned webp); stat cost +0.15 s on 2,000 inputs. **Two asserts:** FT-52 misses a removed
  extension; FT-35/36 can skip silently on CI. **Three wording items.** Two out-of-spec findings filed in
  `guidance/ideas.yaml`; the `--check` residual added to STAGE-063's NFC/NFD entry.
- [x] **punch list 2** (2026-11-24, Sonnet, $1.30 measured before its last commit) — PR #270 at
  `afbee0c`, CI 18 passed / 8 skipped / none failed (orchestrator-read); merges cleanly. FT-52 probes a
  literal list (nef, webp, xyz mutants each red); FT-35/36 fail instead of skip under `CI` (macOS log shows
  both ran); residual reworded to "on any build"; comment and FT-37 text fixed. The only non-test
  `src/` change is the comment. **Approved for merge by the orchestrator.**
- [x] **merge** (2026-11-25, maintainer) — PR #270 as 3309cce, head 3dd1b48 (the maintainer's merge of `main`
  at `7e213b2` into the approved head `afbee0c`; `git log --first-parent --no-merges afbee0c..3dd1b48` lists nothing and
  `git show --remerge-diff 3dd1b48` is empty).
- [x] **ship** (2026-11-25, Sonnet) — PR #270 merged as 3309cce. Reflected; DEC-117 written and DEC-061 amended;
  archived. Total cost across all eight sessions: $51.29 (re-measured; the ship entry is measured after the push).
