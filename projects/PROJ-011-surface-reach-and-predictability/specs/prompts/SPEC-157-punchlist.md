# SPEC-157 — PUNCH LIST prompt: an output that is already a read file, a glob that would read its own outputs, and the missing rows

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. This punch list has two
**claim** items, one **assert** item, and wording items. Because two items are claims, a narrow
re-verify follows your report. You did not design, build or verify SPEC-157.

```
just wt fix/spec-157-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-157-build-destinations:refs/remotes/origin/feat/spec-157-build-destinations
git checkout --detach origin/feat/spec-157-build-destinations
```
Push to the PR branch with `git push origin HEAD:feat/spec-157-build-destinations`. When this was
written, draft PR #270's head was `9733cd8`. If it has moved, stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove AGENTS §15's coverage map against release builds of `main` and the PR:
- every shape `out` can take, against every step that reads it;
- every kind of read file, against the overlap check;
- `{ext}` with and without a pinned format;
- case and Unicode;
- the states between steps.

It also ran FT-29 and FT-30 ten times each (10/10), drove every mutation row, and read every CI leg,
Windows included. All of it was green. Every refusal message matches the spec byte for byte, and every
refusal leaves the tree, the lockfile and `.crustyimg` unchanged. Verify found these problems:

1. **A source is still overwritten on macOS.** `fold_case` is `to_lowercase`
   (`src/cli/build.rs:392`). It maps neither `ß` to `ss` nor NFD to NFC, but APFS treats both pairs as
   one name. Verify drove two cases, and each exits 0 with the source replaced:
   - source `src/ß.png`, `out = "src"`, name `SS.{ext}`;
   - an NFC source and an NFD output name in the same folder.

   Two more spec lines are false for these names:
   - "never overwrites a file the build reads";
   - "Letter case is ignored on every OS".
2. **A glob source writing into its own folder works once, then never again.** The manifest is
   `src/*.png`, `out = "src"`, name `{stem}_w.{ext}`. The first build succeeds. Every later build
   exits 2, because the glob now matches `a_w.png` and its planned `a_w.png` overlaps a source.
   `main` grew `a_w_w.png` instead. This PR introduced the behaviour. Verify found it outside the
   spec, and the maintainer has ruled that it be fixed here.
3. **Three mutants turn no rows red, and several refusals have no row:**
   - **UX1**: `_ => &["png"]` in the extension list;
   - **UX2**: the `.crustyimg` check made case-sensitive;
   - **UX3**: the manifest given by a non-default path.

   The coverage table also lists these refusals with no row:
   - `out` itself an out-of-tree link;
   - a lockfile already present;
   - overlap reached through a link, `..` or a case alias.
4. **`SECURITY.md` and the spec's message list say exit 5** when a destination moves. The build
   aggregates per-input failures, so the driven race exits **6** ("1 of 1 inputs failed").

## The maintainer's rulings (settled, do not re-decide)
- **Item 1 is closed by a same-file check, not by Unicode tables.** No new dependency.
- **Item 2 is fixed in this punch list**: refuse on the **first** build.

## The fixes
1. **An output that already exists and is a read file is refused, by file identity.** In
   `check_destinations`:
   - Build a second index of the read files that exist: device and inode → clause, from
     `std::fs::metadata` (follow links, the way the reads were canonicalized). Use
     `std::os::unix::fs::MetadataExt` (`dev()`, `ino()`), `cfg(unix)` only.
   - For each planned output under each candidate extension: if `std::fs::metadata` of the planned
     absolute path succeeds and its device and inode are in the index, return the same
     `CliError::OutputOverlap` the folded-string check returns, with that read's clause. Run it after
     the folded-string lookup misses, so every existing message stays the same.
   - On Windows, skip the check and say why in a comment. NTFS's case mapping doesn't merge `ß` and
     `SS`, and NTFS doesn't normalize, so the folded string is already the file's identity there.
     State that as reasoned, not driven, in the spec.
   - Planning still writes nothing. A planned name that doesn't exist yet can't overwrite a read file.

   Two outputs that land in one file on a fresh build (`ß`/`SS` or NFC/NFD into `dist`: one file, two
   lock records) **stays a documented residual**. Neither file exists at plan time.
   - Extend the spec's existing "Not covered" sentence and its filed STAGE-063 NFC/NFD follow-up to
     name `ß`/`SS` as well, and to say that only the output-to-output collision is left.
   - Make the same change in the docs, `CHANGELOG.md` and DEC sentences that carry the case rule.
     Keep "never overwrites a file the build reads", which is now true. Narrow "Letter case is ignored
     on every OS" so it no longer claims more than `to_lowercase` does: name the residual in one
     clause.

   Rows, all `cfg(target_os = "macos")` end-to-end in the file that holds FT-1, because only APFS
   merges these names:
   - `ß`/`SS` over its own source refused, exit 2, the source's bytes unchanged;
   - an NFD output name over an NFC source refused, exit 2, the source unchanged.

   Each needs a guard that **skips with a printed reason** when the temp dir's filesystem doesn't
   merge the two names (create one, stat the other). Then a case-sensitive APFS volume reports a skip
   rather than a false green. Also add one `cfg(unix)` unit row that doesn't depend on the filesystem:
   a hard link to a source, planted at the planned output name, is refused. That pins the identity
   check on Linux too.

   Mutation row **M15**: the identity check removed. It must turn every new row in this fix red.
2. **A glob or folder source that would read a planned output is refused on the first build.** In the
   same per-output pass, for each target's source arguments as written in the manifest:
   - a **glob**: refuse when the pattern matches the planned output. Build a `glob::Pattern`, and
     match it with the same options `resolve_glob` uses, against the output spelled the way the glob
     would list it. Today the root-relative path, after `rel_dir`.
   - a **folder**: refuse when the planned output sits directly in that folder (non-recursive, as
     `resolve_directory` lists) and carries an image extension the source filter accepts.

   The refusal is `CliError::OutputOverlap` with a new clause, for example
   `which target #j's source pattern `src/*.png` would read on the next build`. Put the exact text
   in the spec's message list.
   - **Before you write it**, build the shapes table AGENTS §15 asks for, and put it in the spec. It
     sets source spelling (`src/*.png`, `./src/*.png`, `src`, `src/`, a link to `src`, `SRC/*.png`)
     against output spelling (`out = "src"`, `"./src"`, `"src/"`, a link to `src`). Each cell gets a
     row or a one-line reason it needs none. Where the glob's spelling and `rel_dir` differ (a link, a
     `./`, case), normalize the pattern's literal prefix the way `resolve_out_dir` normalizes `out`,
     and say how in the spec.
   - Rows:
     - the verify reproduction refused on the first build, exit 2, the tree unchanged;
     - a folder source;
     - a glob whose pattern does **not** match the output (`src/*.png` with name `{stem}.webp`,
       pinned), which still builds and still builds again.

   Mutation row **M16**: the pattern check removed. It must turn red the first two of those rows.
3. **Rows for the cells verify proved uncovered.** One row each, in the existing test files, named
   after the FT numbering:
   - a non-`png` `{ext}` overlap (UX1), and a pinned-format target whose other extension doesn't
     overlap;
   - `.CRUSTYIMG/x` as an output (UX2);
   - the manifest given by a non-default path, with an output over it (UX3);
   - `out` that is itself a link out of the tree, exit 2, nothing written outside;
   - an output over an existing lockfile;
   - overlap through a link, through `x/../src`, and through `SRC`.

   Drive UX1, UX2 and UX3 again, each alone, and quote the red set. Each must now turn at least one
   row red. Add them to the mutation table as **M17–M19**.
4. **Wording.**
   - Exit **6**, not 5, for a destination that moved: in `SECURITY.md` (line 49) and in the spec's
     message list. Read line 42 too. Its exit 5 is for the sink's link refusal. Check what `build`
     exits in that case (verify drove a link planted at the output: exit 6), and make it say what each
     command does. Add one end-to-end check of the build's exit code beside the unit FT-28. If the race
     can't be driven deterministically in a test, say so and leave FT-28 as the row.
   - The mutation table's M12 cell: add FT-28, and say why it goes red. `build_root()` is the test
     process's working directory, so the mutant returns exit 2 before the exit-5 check.
   - The build reflection's claim that the template name is "one component": `src/a.{ext}` reaches
     `out_dir.join` and gives an overlap refusal (exit 2), and `safe_join` refuses it at write time.
     Make the sentence say that.
   - In every case, update the spec's verbatim copies to match, byte for byte, marking each
     `(corrected at punch list)`.
5. No spec, stage or DEC id in any changed line outside the spec.

Stay in these files:
- `src/cli/build.rs`;
- the test files the PR already changes;
- `SECURITY.md`;
- the docs, `CHANGELOG.md` and DEC sentences that fix 1's narrowing touches (list each one);
- the SPEC-157 spec.

`git diff --stat 9733cd8` must show nothing else. **`Cargo.toml`, `Cargo.lock`, `src/sink/`,
`src/source/`, `src/build/` and `.github/` must not change.** If fix 2 seems to need
`src/source/`, use what it already exposes at `pub` (`looks_like_glob`). If that is not enough,
stop and report rather than widen it.

**Not yours:**
- verify's three pre-existing findings, which the orchestrator files on `main`:
  - a recipe name shadowed by an output;
  - an out dir left behind when the template name has a separator;
  - an empty dir left behind by `create_dir_all`;
- the absolute-path wording in the `OutDirCreate` message;
- the timeline;
- the PR's draft state.

## Check it
- **Before each fix**, drive its new rows against `9733cd8` and quote them red:
  - fix 1: the `ß`/`SS` and NFC/NFD cases exit 0 with the source replaced;
  - fix 2: the second build exits 2;
  - fix 3's rows are pins: their mutants make them red.
- Run M15–M19, each alone, in a scratch worktree with its own `CARGO_TARGET_DIR`:
  `cargo clean -p crustyimg`, then a full `cargo test --no-fail-fast` with stdout redirected, and read
  the red set off the run. Fix 1 and fix 2 add work next to the existing checks, so also re-run **M7**
  and **TRAP14**. Each must still be red where the table says.
- Run the matrix, each leg in a fresh `CARGO_TARGET_DIR`: `fmt --check`,
  `clippy --all-targets -D warnings` and the full test, on default, `--no-default-features` and
  `--features webp-lossy`. Then `just wasm-check`.
- Re-drive the upgrade once: a lock `main` wrote, then `--check` (exit 7, the same two drift lines as
  verify saw), then one build, then `--check` (exit 0), with `main`'s warm cache still hitting.
- Drive each changed doc sentence and quote the result.
- CI runs on push. Read the Windows leg: fix 1 is `cfg(unix)`, fix 2 runs everywhere.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- If the PR is behind `main`, rebase onto `main` before you push. Never force-push over someone
  else's commit: if the PR's head isn't `9733cd8` when you start, stop.
- Push after each fix lands green locally (checkpoints), not only at the end.
- **Never poll CI.** Run `gh pr checks 270 --watch` once in the background after your last push, and
  don't write your own wait loop. Don't use `pgrep -f`. When it exits, take one snapshot at the PR's
  head SHA, every leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters, and it reads `$O:s…` as a modifier, so brace it (`${O}:`). `rtk` can truncate
  `grep`, so use `/usr/bin/grep` for anything you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir. Restore any
  `chmod` before you delete a scratch folder.
- **Budget ~55 exchanges.**

## When you finish
- Add a `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - each fix with its red-before and green-after evidence;
  - fix 2's shapes table;
  - M15–M19, M7 and TRAP14, each with its red set;
  - the matrix;
  - the CI snapshot.
- Set the spec's `cycle` back to `verify`.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-157 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions`, with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.
  - Also record verify's cost there as a `verify` entry, from these numbers: input 186, cache_creation
    429,830, cache_read 13,797,124, output 104,013, total 14,331,153, $12.19, Opus.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-157 punch list: PR #270 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat 9733cd8`, and that the files the scope forbids are unchanged;
- each fix: the code change in one line, the new rows' names, red before and green after;
- fix 2's shapes table;
- M15–M19, M7 and TRAP14, each with its red set;
- each changed doc, CHANGELOG or DEC sentence, quoted as it now reads, with its drive;
- the upgrade re-drive;
- the matrix and the CI snapshot, Windows included;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
