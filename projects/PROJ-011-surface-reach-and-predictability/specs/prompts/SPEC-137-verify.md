# SPEC-137 — VERIFY prompt: SVG text in any font family renders in the bundled font

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-137.

```
just wt verify/spec-137 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-137-svg-font-family:refs/remotes/origin/feat/spec-137-svg-font-family
git reset --hard origin/feat/spec-137-svg-font-family
```

Draft PR #217's head was `fb09421` when this prompt was written, with CI green on every leg.
Record the SHA you verify. The maintainer pasted this prompt into a fresh session and will paste
your final message back to the orchestrator, so it must stand on its own. This is the first of
the two specs that open **0.8.1**.

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-137-svg-text-in-any-font-family-renders-in-the-bundled-font.md`.
  Read it all: `## Frame` with its rulings, `## Design` with Calls 1–8, the 10 ACs,
  `## Failing Tests` (7 tests) with the six negative controls, the DEC-054 amendment text, and
  the build's `## Build Completion`.
- 10 of 10 ACs met. AC-7 (wasm) is green under a local `just wasm-test` (46 passed), which is
  **not in CI**.
- Commit 1 is `51156ab`; its only `src/` hunk is inside `src/image/svg.rs`'s `#[cfg(test)]`
  module.
- Baseline on unmodified code: 4 RED, 3 GREEN. The tspan row drew 253 px on `main`. **The wasm
  row was not run on unmodified code**; the build only saw it go RED under negative control 1.
- Every negative control flipped as the spec predicted. The table tests stop at their first
  failing row, so the build names only the first red row per control.
- Row counts 18, 6 and 3. The local matrix is green on all three legs.
- **Not checked by the build:** the system-font row's SKIP line on Linux CI, which font it used
  on Windows CI, and that the pinned rows ran on each leg. The build inferred them from green CI.
- Out-of-spec notes: the spec's blank-render hash `bb918147…` should be `bd30b081…` (text only,
  no assertion uses it); the wasm constants were renamed to avoid a clash; the test reads
  `pixmap.pixels()` alpha because clippy rejected `chunks_exact(4)`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope and commit order.**
   - Run `git diff --stat origin/main...HEAD` and compare it with the spec's file list and Build
     Completion.
   - `git show 51156ab -- src/` must touch only the `#[cfg(test)]` module.
   - Every commit is signed off.
   - The diff contains no `Cargo.toml`, `Cargo.lock` or `CACHE_SCHEMA_VERSION` change, and
     `fontdb` gains no feature. Confirm the last one from the build fingerprint, not from
     `cargo tree`, which cannot see it.
2. **The baseline, including the part the build skipped.** In a scratch worktree at `51156ab`
   (tests added, code unmodified):
   - run the four RED tests and the three GREEN ones, and confirm each is red or green **for the
     spec's reason**;
   - run the wasm row under `just wasm-test`. It must be RED (ink 0) there. If it is green on
     unmodified code, the row is hollow;
   - at the PR head, run `just wasm-test` and confirm the row is green.
3. **CI actually exercised the rows.** Green is not proof that a row ran (a harness that
   exercises nothing reports green). Read each leg's log once with
   `gh run view <run> --log --job <job>`. CI run `40835877119` was the run at `fb09421`; use the
   run at the head you verify. Do not poll.
   - **ubuntu-latest:** `an_installed_system_font_is_never_read` printed a SKIP line, or it found
     a font. Quote the line. A pass with neither is a silent pass, which is a punch item.
   - **windows-latest:** the system-font row ran with Arial, and did not skip. Quote it.
   - **macos-latest:** the row ran with Helvetica.
   - **All three:** `every_family_form_draws_the_go_render` and
     `working_family_forms_keep_their_pinned_bytes` appear as `ok`, so the pinned hash held on
     each OS.
4. **Drive the fix on release binaries, counting ink with a tool the build did not write.** Build
   `main` and the PR head as release binaries, each in a fresh target dir. For every AC-1 form,
   the AC-2 forms and the AC-4 text-free forms, render through each binary with the spec's CLI
   surface and count pixels with alpha > 0 using ImageMagick or Python, never crustyimg:
   - AC-1 forms: `main` draws 0 (the tspan row 253), and the PR draws 738 and is byte-equal to
     the PR's `Go` render;
   - AC-2 forms: both binaries draw 738 and give identical bytes (`cmp`);
   - AC-4 forms: both draw 0.

   A row that gives the same answer on both binaries but is listed as "RED on `main`" is hollow.
   Also drive one other surface on the PR binary, `apply --recipe` or `build`, with a
   `sans-serif` SVG, and confirm it draws.
5. **Re-apply the negative controls yourself, one at a time.** For each: check `git diff` is
   non-empty, rebuild in a **fresh** `CARGO_TARGET_DIR` or after `cargo clean -p crustyimg`
   (reverting source does not rebuild the binary), run the named tests, then restore and show
   them green.
   - The table tests stop at their first failing row, so they can't show the spec's full list of
     red rows. For controls 1 and 4, also **drive the mutated binary over every AC-1 row** (as in
     item 4) and check that exactly the rows the spec predicts go blank, and the rows it says stay
     green still draw. A row that should fail but still draws is a gap.
   - Control 2 must turn **only** the unit row red. Control 4 is macOS-only and temporary.
6. **Mutations the spec did not list.** Apply each, rebuild clean, and name the test that catches
   it, or report a gap:
   - **M-a:** point the serif family at a name that is not Go (`"Gox"`). The CLI rows should go
     RED;
   - **M-b:** load `FONT_DATA` twice. `hardened_options_hold_only_the_bundled_face` should go RED;
   - **M-c:** pin the wrong hash and break the in-run equality at the same time (render the `Go`
     comparison with a different font size). Confirm the two checks fail independently, as
     control 6 claims;
   - **M-d:** in the wasm row's SVG, give the text-free twin a `font-family`. Read whether the
     wasm row could still pass on unmodified code with that change, and whether its positive and
     negative rows really differ only in the text.
7. **Code quality.** Read the whole diff:
   - the five setter lines, `opt.font_family` and the doc comments match Call 1;
   - no `unwrap` or `expect` in `src/**` outside tests;
   - `fontdb` is reached as `usvg::fontdb`, with no direct dependency;
   - comments and messages read behaviour-first, with no spec or DEC ids.
8. **Docs and record**, against the binary:
   - `docs/cli-reference.md` `## Conventions` and `README.md`: each sentence is true when
     driven. "Never reads system fonts" is AC-5. "Bold and italic draw upright in Regular" is
     AC-3: drive one bold and one italic render and compare bytes with Regular;
   - the `CHANGELOG.md` `### Fixed` line names 0.5.0 through 0.8.0. Drive at least one released
     binary yourself (Homebrew 0.8.0 is installed; the 0.7.1 asset is on the GitHub Release) with
     a `sans-serif` SVG and confirm it draws 0 px. Say which versions you drove;
   - DEC-054's amendment matches the spec's text, and `./scripts/decisions-audit.sh` reports no
     errors;
   - list the spec's `bb918147…` hash typo as a cleanup for the orchestrator, not a punch item.
9. **Full suite.** Run the default leg **and** the `--no-default-features` leg once each,
   sequentially, each in a fresh target dir. Use `--no-fail-fast`, redirect stdout to a file, and
   read `$?`. Then run `gh pr checks 217` once at the head you verified.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Never share a target dir between differently-featured builds running at once. Delete each
  scratch target dir and scratch worktree when you are done with it.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-137 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`), never at a flat
  rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
  cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #217 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #217 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–9, one or two lines each with evidence;
- the CI log lines for the system-font row on each OS;
- the negative-control table: control, tests that went RED, the rows driven under controls 1 and
  4, and whether that matches the spec;
- M-a to M-d, each caught (by which test) or a gap;
- optional cleanups, listed separately from punch items;
- your cost.
