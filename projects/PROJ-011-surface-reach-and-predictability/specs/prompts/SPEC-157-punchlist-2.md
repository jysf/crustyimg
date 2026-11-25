# SPEC-157 — PUNCH LIST 2 prompt: two loose rows and three sentences

Cycle: **build (second punch list)**. Fresh CLI session, own worktree, **Sonnet**. The re-verify
drove both claim fixes on release builds and found them right. It also checked what they must not
refuse, and found nothing wrongly refused. It left two **assert** items and three **wording** items,
and no claims. So no re-verify follows: the orchestrator checks your report against the PR, and ship
comes next. You did not design, build, verify or punch-list SPEC-157.

```
just wt fix/spec-157-punch-2 -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-157-build-destinations:refs/remotes/origin/feat/spec-157-build-destinations
git checkout --detach origin/feat/spec-157-build-destinations
```
Push with `git push origin HEAD:feat/spec-157-build-destinations`. Draft PR #270's head was `8a7b03c`
when this was written. If it has moved for any reason other than a maintainer merge from `main`,
stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## The fixes (settled, do not re-decide)
1. **FT-52 must catch an extension removed from the copy** (assert).
   - Evidence: the re-verify removed `nef` from `READABLE_EXTENSIONS`, and FT-52
     (`readable_extensions_match_the_source_filter`) stayed green. The test only writes files for
     extensions in the copy or in `OUTPUT_EXTENSIONS`, so it can't see an extension missing from both.
   - Fix: have the test probe a list it doesn't take from the copy. Use a literal list of the 28 source
     extensions in the test itself, plus `OUTPUT_EXTENSIONS`. For each one, assert that the real source
     filter's verdict equals the copy's verdict.
   - Drive three mutants on the copy, each alone, and quote the result. Each must turn FT-52 red:
     1. remove `nef`;
     2. remove `webp`;
     3. add `xyz`.
2. **FT-35 and FT-36 must not pass silently when they skip** (assert).
   - Evidence: both tests skip, with an `eprintln!`, when the temp dir's filesystem doesn't merge the
     two names. CI runs plain `cargo test`, which hides a passing test's output. So the macOS log can't
     show whether they ran.
   - Fix, without touching `.github/`: when the `CI` environment variable is set (GitHub Actions sets
     `CI=true`), a filesystem that doesn't merge the names is a **failure**, not a skip, with a message
     that says why. Local runs still skip, with the reason printed.
   - Drive it locally:
     - with `CI=true`, both pass on APFS;
     - force the probe to "not merged" (a temporary edit, reverted) with `CI=true`, and both fail;
     - the same forced edit without `CI`, and both skip.

     Quote all three.
3. **"one new file" understates the residual** (wording).
   - Evidence: two outputs whose names APFS merges (`a/ß.png` and `b/SS.png` into `out = "dist"`) stay
     uncaught on warm builds too, when the file is no longer new. `--check` also exits 0 while the lock
     holds two paths.
   - Fix: in `docs/cli-reference.md`, `CHANGELOG.md` and the spec's copies, say that the check compares
     outputs with the files the build reads, never two outputs with each other by file identity. Then
     two outputs whose names only a Mac's filesystem merges (`ß`/`SS`, NFC/NFD) are not caught, on any
     build.
4. **A comment names a test that doesn't exist** (wording).
   - Evidence: `src/cli/build.rs:604` says `source_extensions_match_the_source_filter`.
   - Fix: change it to `readable_extensions_match_the_source_filter`.
5. **The spec's punch-list text** (wording, spec only).
   - It says FT-35/36/37 each "exit 0 with the source replaced". On `9733cd8`, FT-37's hard-link case
     exits 0 with the source **unchanged**, because the staged rename breaks the link. Say FT-37 was red
     only on the exit code.
   - Remove the `*(corrected at punch list)*` markers from inside the "verbatim" CHANGELOG code fence.
     Put each marker after the fence instead, so the fenced text equals `CHANGELOG.md` byte for byte.
   - Then re-check every verbatim copy against its source mechanically, normalizing only line wrapping
     where the spec rewraps on purpose, and quote the result.

Stay in these files:
- `src/cli/build.rs`, the comment only;
- `tests/build_destinations.rs`;
- `docs/cli-reference.md`;
- `CHANGELOG.md`;
- the SPEC-157 spec.

`git diff --stat 8a7b03c` must show nothing else. `.github/` and every other `src/` file must not
change.

**Not yours:**
- the re-verify's three out-of-spec findings, which the orchestrator files on `main`;
- the timeline;
- the PR's draft state.

## Check it
- Run the matrix, each leg in a fresh `CARGO_TARGET_DIR`: `fmt --check`,
  `clippy --all-targets -D warnings` and the full test, on default, `--no-default-features` and
  `--features webp-lossy`.
- CI runs on push. Read the macOS leg's log and confirm FT-35 and FT-36 ran under `CI=true` and
  passed. With fix 2 in place, a pass now means they ran.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Never force-push over someone else's commit.
- **Never poll CI.** Run `gh pr checks 270 --watch` once in the background after your last push, and
  don't write your own wait loop. If it returns before the checks register, run it once more. Then
  take one snapshot at the PR's head SHA, every leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh reads `$O:s…` as a
  modifier, so brace it (`${O}:`). Use `/usr/bin/grep` for anything you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- One fresh `CARGO_TARGET_DIR` per leg or mutant, never shared with another copy of your own script.
- Write scratch files in your session's scratchpad, never `/tmp`.
- **Budget ~25 exchanges.**

## When you finish
- Add a `### Punch list 2 (re-verify)` subsection under `### Punch list (verify)`, with each fix's
  evidence, the three FT-52 mutants, the three FT-35/36 drives, the matrix and the CI snapshot.
- Leave `cycle: verify`.
- **Cost.** Append to the spec's `cost.sessions`:
  - a `reverify` entry from these numbers: input 76, cache_creation 110,280, cache_read 3,884,821,
    output 31,811, total 4,026,988, $3.43, Opus. It was measured before its final message.
  - your own `punchlist-2` entry. Find your transcript by content:
    `/usr/bin/grep -l "SPEC-157 — PUNCH LIST 2 prompt" ~/.claude/projects/<slug>/*.jsonl`, where
    `<slug>` is your working directory's absolute path with every `/`, `.` and `_` replaced by `-`.
    Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
    `output_tokens`. Price at Sonnet's $3/$15 per MTok, cache_creation ×1.25, cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is
`SPEC-157 punch list 2: PR #270 at <head SHA> — CI <green | red: legs>`. Then:
- `git diff --stat 8a7b03c`;
- each fix in one line, with its evidence (the FT-52 mutants, the three FT-35/36 drives, each changed
  sentence quoted as it now reads, the verbatim-copy check);
- the matrix and the CI snapshot, with the macOS log lines for FT-35 and FT-36;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
