# SPEC-143 — VERIFY prompt: a watermark that draws nothing is refused

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-143.

```
just wt verify/spec-143 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-143-refuse-invisible-watermark:refs/remotes/origin/feat/spec-143-refuse-invisible-watermark
git reset --hard origin/feat/spec-143-refuse-invisible-watermark
```

Draft PR #235's head was `c467490` when this prompt was written, with CI green on all 16
non-skipped checks. Record the SHA you verify. The maintainer pasted this prompt into a fresh
session and will paste your final message back to the orchestrator, so it must stand on its own.
This spec opens **0.8.2**.

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-143-a-watermark-that-draws-nothing-is-refused.md`.
  Read it all:
  - `## Frame` with rulings 1–3;
  - `## Design` with Calls 1–8;
  - the 14 ACs;
  - `## Failing Tests` (FT-1 to FT-16) and the mutation table (M1–M10);
  - the build's `## Build Completion`.
  Also read `decisions/DEC-109-…md` (new, written at design) and the DEC-106 line the build added.
- 14 of 14 ACs met. Commits, in order:
  - `2ccbdaf`: the 12-case golden, blessed with no `src/` change;
  - `a028d9d`: FT-1 to FT-12, still on unmodified `src/`;
  - `8a91c83`: the `faded_alpha16` refactor, with FT-6 and `opacity_does_not_change_coverage` green;
  - `fe5d0de`: the fix; `9bb4d6e`: the docs; `c467490`: the head.
- Baseline on `main`: FT-1/2/3/7/8/11/12 RED, FT-4 exits `3,3,3,3,3,0`, FT-9 exits 3; FT-5, FT-6
  and FT-10 GREEN.
- Every mutation went red, with three differences from the table, which the build reports as
  deviations:
  - **AC-3:** `build` prints two `error:` lines (the target, then the step), not one. FT-11
    asserts both.
  - **M5:** FT-15 goes red, though the table expects it green, because FT-15 calls `apply`.
  - **M4a:** FT-13 goes red as well; the table doesn't list it.
- ⚠ **A claim to test:** the build says the golden-floor mutation and M6 turn FT-6 red because the
  predicate refuses `faint_opacity` first, **not because a hash differs**. So nobody has yet shown
  that FT-6's hashes can catch a pixel change on a mark that is still accepted. Item 5 covers it.
- Local matrix green: default 1129, `--no-default-features` 1067, `webp-lossy` 1134,
  `just wasm-test` 47.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope and commit order.**
   - Compare `git diff --stat origin/main...HEAD` with the spec's `## Outputs` and Build
     Completion.
   - `git show 2ccbdaf -- src/` and `git show a028d9d -- src/` are both empty.
   - `git show 8a91c83 -- src/` is the refactor only: `faded_alpha16` and the 16-bit loop calling
     it. No predicate yet.
   - Every commit is signed off.
   - No `Cargo.toml`, `Cargo.lock`, version, `CACHE_SCHEMA_VERSION`, recipe-key or `params()`
     change.
   - `opacity_does_not_change_coverage` is **unedited**: its body diffs empty against `main`.
2. **The baseline.** In a scratch worktree at `a028d9d`, run FT-1 to FT-12 with `--no-fail-fast`.
   Confirm each RED **for the spec's reason** (read each failure message, not only the count),
   and FT-5, FT-6 and FT-10 green. At `8a91c83`, confirm FT-6 green: the refactor moved no pixel.
3. **The predicate, read whole.** Read `faded_alpha16`, `max_alpha16`, the `max_alpha` field in
   **every** constructor, `InvisibleMark` and `invisible_mark()`, and both 16-bit and 8-bit
   composite branches.
   - The 16-bit composite and the check call the same `faded_alpha16`; they cannot drift.
   - The 8-bit branch and `visible_pixels()` are unchanged.
   - The cause order is Call 1's: opacity 0 → colour alpha 0 → (no visible pixel: whitespace →
     blank, other text → too small, image → transparent) → too faint.
   - `InvisibleMark` is `#[non_exhaustive]`. `new_image`/`new_text` keep their signatures, and
     `Refusal` has no new variant.
   - **Can the check accept a mark the composite draws as nothing?** Using Call 1's numbers, show
     that a mark accepted at 16 bits keeps alpha on the 16-bit branch. Then name the case on the
     8-bit branch where an accepted mark changes nothing (the design accepts this under ruling 2),
     and confirm it is the documented one, not a wider gap.
4. **The call sites, and the CLI order.**
   - `from_params` calls `invisible_mark()` at its end, after image bytes are resolved. Is any
     recipe, `build`, `--watch` or wasm path able to build a `Watermark` without going through
     it? Grep every construction site (`new_image`, `new_text`, `from_params`).
   - `run_watermark`: existing flag checks → opacity 0 and colour alpha `00` → the
     `--font`/`--image` read → `invisible_mark()` → the inputs. Each message matches Call 4
     character for character, and maps to exit 2.
5. **Can FT-6's hashes catch a pixel change?** Apply a mutation that changes the pixels of an
   **accepted** mark without triggering a refusal. For example: in `faded_alpha16`, return
   `(a as f32 * op * 0.99).round()`, or change the 16-bit blend for `opacity 0.6`. Rebuild clean.
   - FT-6 must go red on a **hash mismatch**, not on an exit code. Quote the failure.
   - If it stays green, or goes red only through a refusal, the golden is not a byte guard: report
     it as a punch item.
   - Also confirm the golden has 12 lines and that blessing is `#[ignore]`-gated, so CI can't
     re-bless it.
6. **Drive the release binaries, without the test harness.** Build `main`'s merge-base and the PR
   head as release binaries, each in a fresh target dir. Count changed pixels with Python or
   ImageMagick, never crustyimg. Snapshot files with
   `find . -type f | sort | xargs shasum -a 256` before and after every refused row.
   - **All six causes on all four paths:** CLI `watermark`, `apply --recipe`, `build`, and
     `build --watch`. `main` exits 0 and changes 0 pixels; the PR refuses with Call 4's exact
     lines and exit codes, and the snapshot is unchanged. **`--watch` has no FT**: drive it by
     hand, kill the child, and say what it prints and whether it keeps watching.
   - **The boundaries** from Call 5 on a 400×300 black 16-bit base: opaque `0.0000076`/`0.0000077`,
     `ffffff01` `0.0019`/`0.002`, the alpha16-4 overlay `0.1`/`0.13`, text size `0.17`/`0.18`.
     Each refused value gives 0 changed pixels on `main`, and each accepted value changes ≥ 1
     pixel on the PR binary.
   - **Odd characters** on the PR binary: U+0020, U+00A0, tab, U+3000, ZWSP, U+FEFF, soft hyphen
     and an emoji. Only the first two are refused (blank text). The rest draw, and you count their
     pixels.
   - **Messages print as typed:** `--opacity 0.000001`, `0.1` (on a mark that is too faint at
     `0.1`), `-0`, and a recipe `opacity = 1e-50`.
   - **The CLI order:** `watermark /no/such.png --opacity 0` exits 2;
     `--text " " --font /no/f.ttf` exits 3.
7. **Byte identity out of repo (Call 6).** Build `main` and the head in separate fresh target dirs.
   Run the three corpus fixtures (`v1_watermark_image_placement.toml`,
   `v1_watermark_text_font.toml`, `v2_watermark_integer_numbers.toml`) through `apply --recipe` with
   both binaries and `cmp` the outputs. Add CLI controls: text, image, `--opacity 0.6`, `--tile`,
   `--scale 0.1`, a 16-bit base. Negative control: nudge `opacity` or `size` in one fixture and
   show `cmp` differs.
8. **Re-apply the mutation table yourself, M1–M10, one row at a time.** For each one:
   - check `git diff` is non-empty;
   - rebuild **clean** (fresh `CARGO_TARGET_DIR`; reverting source does not rebuild the binary);
   - run every test the row names, on every path it names (CLI, recipe, `build`, wasm via
     `just wasm-test`);
   - record exactly which go red, then restore and show them green.

   Judge the build's three deviations (AC-3's two lines; M5/FT-15; M4a/FT-13): is each a spec
   error, or a sign the code is different from the design? Say which.
9. **Mutations the spec did not list.** Apply each, rebuild clean, and name the test that catches
   it, or report a gap:
   - **M-a:** one of the three constructors leaves `max_alpha` at its default (0 or 65535);
   - **M-b:** a refusal returns a different `CliError` variant, so the CLI exits 1 instead of 2;
   - **M-c:** `from_params` calls `invisible_mark()` **before** the image bytes are resolved;
   - **M-d:** the blank-text check uses `trim().is_empty()` instead of `char::is_whitespace`
     over every char. Does any row tell them apart?
   - **M-e:** delete the `--opacity` help line. FT-7 must catch it.
10. **Code quality.** Read the whole diff:
    - no `unwrap` or `expect` in `src/**` outside tests;
    - no wasm panic path: a refusal through `transform` is a thrown error;
    - comments and messages read behaviour-first, with no spec or DEC ids;
    - test fixtures `black16` and `faint16` assert they decode as 16-bit (and alpha 4).
11. **Docs and record**, against the binary:
    - `recipes/README.md:236-237` no longer promises that `opacity = 0` draws;
    - `docs/cli-reference.md`, `docs/api-contract.md` (refusal list), `docs/data-model.md`,
      `skills/crustyimg/SKILL.md` and `--help` match the binary. Drive each claim;
    - `CHANGELOG.md`: Call 7's entry under `## [Unreleased]` → `### Breaking`, and the released
      0.8.1 lines unchanged;
    - DEC-109 still matches the built code (for example, AC-3's two lines). The DEC-106 line is
      present. `./scripts/decisions-audit.sh` reports no errors.
12. **Full suite and CI.** Run the default leg, the `--no-default-features` leg and
    `just wasm-test` once each, sequentially, each in a fresh target dir. Use `--no-fail-fast`,
    redirect stdout to a file and read `$?`. Then read the CI run at your head once (not polled):
    on ubuntu, macos, windows-latest and the lean leg, FT-1 to FT-16 appear as `ok`, with none
    `ignored`. Run `gh pr checks 235` once.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. In zsh, `$B:t`
  is a modifier, so write `"${B}:path"` for a git rev and path. macOS has no `timeout(1)`.
- A target dir shared across worktrees serves a stale library. Use a fresh one per checkout, and
  never share one between differently-featured builds running at once.
- Kill any `--watch` child you start, and check `pgrep crustyimg` before you finish.
- Delete each scratch target dir and scratch worktree when you are done with it. Keep scratch
  files in a scratch directory, never in `~`.
- **Budget ~75 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "VERIFY prompt: a watermark that draws nothing" ~/.claude/projects/<slug>/*.jsonl`.
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
- `✅ APPROVED — merge PR #235 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #235 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–12, one or two lines each with evidence;
- item 5's result: whether FT-6 catches a pixel change by hash;
- the `--watch` rows from item 6, and the odd-character counts;
- the mutation table as you ran it: mutation, tests that went RED, and whether that matches the
  spec; your ruling on the build's three deviations;
- M-a to M-e, each caught (by which test) or a gap;
- optional cleanups, listed separately from punch items;
- your cost.
