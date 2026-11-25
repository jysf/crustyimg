# SPEC-156 — BUILD prompt: `meta strip` keeps the Adobe APP14 colour-transform marker

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the whole change in code terms:
- two constants and one helper in `src/metadata/mod.rs`, used by both `strip_all`'s loop and the
  test reconstruction at `:1712`;
- a new fixture with an exact recipe and three sha256 hashes;
- 9 failing tests, every message, and a 14-row mutation table;
- the docs and the CHANGELOG line, verbatim.

**All of it was driven at design on a prototype.** The code is small. Most of the work is the
fixture, the tests and the mutations. Be thorough across every row: Sonnet's known weak spot is
sweep thoroughness.

```
just wt feat/spec-156-strip-keeps-app14 -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal.

## What you are building
- **`src/metadata/mod.rs`:**
  - `JPEG_ADOBE_MARKER = 0xEE` and `ADOBE_APP14_MIN_LEN = 12`;
  - `is_adobe_colour_transform(marker, contents)`: the payload starts with `Adobe` and is at least
    12 bytes long. ⚠ It's `Adobe`, **not** `Adobe\0`: the byte after `Adobe` is the high byte of the
    version field. libjpeg and zune-jpeg check only `Adobe` plus a length. Design drove a version byte
    of `0x01`; decoders still read the segment;
  - `strip_keeps_jpeg_segment(marker, contents)`, the one helper;
  - `strip_all`'s JPEG arm becomes `jpeg.segments_mut().retain(|s| strip_keeps_jpeg_segment(s.marker(), s.contents()))`;
  - the reconstruction in `meta_verbs_are_byte_identical_on_unsigned_input` (`:1712`) calls the same
    helper;
  - `strip_all`'s doc comment (`:157-160`) and the module doc (`:11-12`).
- **The fixture** `tests/fixtures/metadata/rgb_adobe_t0_ids123.jpg` (452 B) and
  `tests/fixtures/metadata/README.md`, made with the spec's recipe from tools independent of
  crustyimg.
- **Tests:** FT-1 to FT-8 in `src/metadata/mod.rs` (6 new, 2 rewritten) and FT-9 in
  `tests/metadata.rs`.
- **Docs:** `docs/cli-reference.md:231`, `docs/api-contract.md:501-502`, and the CHANGELOG `### Fixed`
  line, verbatim.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation; say why.
**Don't edit DEC-003 or DEC-029.** Ship applies their text after the merge.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- SPEC-154 has merged (PR #266, `f6e2daa`). It has; this run is the first P1 build after it;
- the spec is at `cycle: build`, and its design commit (`docs(SPEC-156): design keeping APP14 in meta
  strip`) is in your history;
- `cjpeg` and `djpeg` are on `PATH`, and `cjpeg -version` reports libjpeg-turbo. Record the version.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists:
- **#240** deletes `MANIFEST_DROPPED_WARNING` at `src/metadata/mod.rs:150-155`, right where your
  constants go. If #240 has merged by the time you start, put them after `has_manifest`, as the spec
  says. If it merges while you work, expect a textual conflict; keep both sides;
- **#237** touches `CHANGELOG.md`; keep both entries.

SPEC-155's build may run at the same time. It touches `src/sink/`, `src/cli/ops.rs`,
`src/cli/build.rs` and `CHANGELOG.md`, so `CHANGELOG.md` is the only file you share. After every
rebase, run `gh pr list --state open --json number,title,files` and compare; name any new overlap in
Build Completion. If a rebase conflicts in a file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-156-meta-strip-keeps-the-adobe-app14-colour-transform-marker.md`.
   Read it in full:
   - `## Frame` with the maintainer rulings;
   - `## Design`, especially "Design corrections to the frame";
   - every AC, the Failing Tests and the mutation table;
   - `## Implementation Context`: the code, the fixture recipe, the docs, the CHANGELOG, "What does not
     change", the DEC text for ship, the overlap and the verification list;
   - `## Notes for the Implementer`.

   **The design calls are settled**, including the `Adobe`-plus-length signature rule, which the
   orchestrator accepted as the ruling's intent. If one turns out wrong, say so in Build Completion
   and in your report; don't quietly re-decide it.
2. **DECs:** DEC-003, DEC-015 (it governs `docs/api-contract.md`) and DEC-029. Read them yourself.
3. The code the spec cites at `file:line`, before you change it.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10 to 14). Also `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. Keep APP14 only when it is Adobe's colour-transform segment. Any other APP14 is still stripped.
2. The orientation and ICC warnings belong to a separate STAGE-063 entry, not here.
3. No message, help line, comment or doc line names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage.
1. **The fixture.** Run the spec's recipe, steps 1–4, in your scratchpad. Check each sha256 against
   the spec's three hashes. If your `cjpeg` gives different bytes, apply the byte edit only after
   checking that offsets 97/100/103 and 327/329/331 hold `0x52 0x47 0x42` in your file. Record the new
   hashes in the README, and say so in your report. Check step 4's two pixel values with `djpeg`.
   Commit the fixture and the README.
2. **The RED baseline.** Add FT-1 to FT-9 to `main`'s code and run the full suite (`--no-fail-fast`,
   stdout redirected, `$?` read). Each red row must fail **for its reason**, with the messages the
   spec quotes under M1:
   - FT-1 and FT-2: `stripped file decodes differently: …`;
   - FT-4 and FT-7: the APP14 list (`left: []`);
   - FT-9: `pixel (8,8)`.

   A red on a "fixture" precondition is the wrong reason; fix it first. FT-5 and FT-6 pass on `main`,
   because it strips every APP14. Quote each line. Open the PR **as a draft** with this commit.
3. **The fix:** the constants, the helper, the two loops, and the two doc comments. Everything is
   green.
4. **Docs and CHANGELOG,** verbatim from the spec. Each behaviour sentence (B1 to B5) is driven by the
   test the spec names. Also drive `crustyimg diff` of the CMYK fixture against its strip (expect
   `100.0000`), and `djpeg` on the committed fixture and on `meta strip` of it. Quote all of it.
5. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it.** For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected and `$?` read, and read the red set
     **off that run**. Compare it with the table and call out every test the table doesn't list;
   - design drove **M3b** and **M8** on the lib target only. Run them on every target;
   - **M6**'s run was killed during design before the doc-tests. Let yours finish;
   - restore, rebuild, and show it green.

   Paste each mutation's red output, trimmed to the lines that name tests, into Build Completion.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
6. **Clean matrix:**
   - default, `--no-default-features`, and `--features webp-lossy`, one after another, each in a
     fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each;
   - then `cargo build --target wasm32-unknown-unknown --lib`, with cargo from `~/.cargo/bin` (the
     rustup toolchain; `justfile:119`). The only warning should be the existing
     `MANIFEST_DROPPED_WARNING` one; name any other.

## The things most likely to go wrong
- **`Adobe\0`.** Writing the signature as six bytes strips segments decoders read (M3b, FT-4).
- **The length floor.** 12 bytes, not 11 or 13. M5a and M5b red different tests.
- **Expected values from the code under test.** FT-1 to FT-4 and FT-9 compare decoded pixels of the
  input with decoded pixels of the stripped output, and assert the APP14 bytes literally. FT-2 asserts
  the fixture's ID bytes and transform byte, not only its render.
- **Retyped text.** Copy every doc sentence and the CHANGELOG line from the spec byte for byte.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean, Windows and wasm.

## Guardrails
- No version bump, no release, no tag. No change to `Cargo.toml`, `Cargo.lock` or any DEC. Paste
  `git diff --stat origin/main...HEAD` for those paths.
- Don't edit the spec's timeline. The orchestrator keeps it.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout to
  a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each commit
  on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not word-split
  unquoted parameters, so loop with `while IFS= read -r`, and it reads `$R:r…` as a modifier, so brace
  it (`${R}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use
  `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Python that reads files runs with `-I`. Never share a `CARGO_TARGET_DIR` with another session.
  Delete each scratch target dir and the mutation worktree when you're done. Write scratch files in
  your session's scratchpad, never `/tmp` or the system temp dir, and don't leave PR-body files
  behind.
- **Budget ~60 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase on
     what changed; give the count and say whether it includes the spec file;
   - the fixture's hashes and `cjpeg` version;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why;
   - each behaviour sentence's drive;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out;
   - the matrix legs, the wasm build, and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-156 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any `subagents/`
     transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per MTok,
     Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-156 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks it
     ready. Never mark it ready yourself.
   - Title: `fix(meta): strip keeps the Adobe colour-transform segment (SPEC-156)`.
   - Body per AGENTS §13:
     - Summary: `meta strip` removed Adobe's APP14 and changed how CMYK and some RGB JPEGs render (orange
       became green on the CMYK fixture); it now keeps that segment, so the stripped file decodes to the
       same pixels, and is 16 bytes larger for a typical file;
     - Spec metadata;
     - Decisions referenced: DEC-003, DEC-015, DEC-029. New decisions: none (DEC-003's amendment and
       DEC-029's note follow at ship);
     - Constraints checked;
     - Breaking: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-156 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- the fixture's hashes against the spec's, and the `cjpeg` version;
- the RED baseline on `main`, each row with its reason;
- AC completion (as a count, plus any row not met);
- each behaviour sentence's drive;
- each mutation's flips against the table, the edit you applied where it differs from the quoted one,
  and every cell that turned out wrong;
- the matrix result, the wasm build, and the CI legs, including lean and Windows;
- the overlap check after your last rebase;
- files changed against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
