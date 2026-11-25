# SPEC-149 — VERIFY prompt: no flag is accepted and ignored; `-v` shows warnings

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**. You did not design or build SPEC-149. Read-only toward
the PR under review.

```
just wt verify/spec-149 -- --model opus
```

Put the worktree on the PR's code:
```
git fetch origin feat/spec-149-no-flag-accepted-and-ignored:refs/remotes/origin/feat/spec-149-no-flag-accepted-and-ignored
git reset --hard origin/feat/spec-149-no-flag-accepted-and-ignored
```
Draft PR #256's head was `e53e9f6` when this prompt was written (6 commits; CI green: 20 pass, 12 skipped, 0 fail,
including windows-latest and lean). Record the SHA you verify. The maintainer pastes this prompt into a fresh session
and pastes your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and has SPEC-151 (`pad`, draft PR #255) in flight.** Do not touch
`src/operation/`, `src/recipe/`, `docs/recipes.md`, `docs/api-contract.md`, `docs/data-model.md`, `README.md`,
`CHANGELOG.md`, `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh`. Use your own
`CARGO_TARGET_DIR`; never share one.

## What the build claims
- Spec: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-149-no-flag-is-accepted-and-ignored.md`. Read it
  all (`## Frame` with its premise check, `## Design` calls 1 to 5 and the verb × flag × message table, 11 ACs, FT-1 to
  FT-10, the mutation table M1 to M12 plus M6b, Docs sweep, `## Build Completion`). HANDOFF traps 1 to 17 and rules 1
  to 7 bind you (rule 7: optional cleanups go in one punch list, no re-verify unless a claim changed; trap 17: a bound
  or table worked out on paper is a claim until its maximising input is driven).
- 6 commits, all claimed signed off: red tests, fix, docs sweep, the `build --strict` control fix, Build Completion
  with the verify advance, and a lean-clippy gate (`#[cfg(feature = "svg")]` on `W`, `D`, `count_line`).
- Core: one guard table in `src/cli/global_flags.rs` (8 rows R1 to R8, refusals exit 2, scope by exclusion so a later
  verb is covered); a stderr logger in `src/cli/logger.rs` (`-v` warn, `-vv` info+debug, `-Q` wins, nothing installed by
  default); `WatermarkAssetUnreadable` (exit 3) naming the font or overlay file; `log = "0.4.32"` native-only.
- The build says: RED on `main` for FT-1, 3, 4, 5, 7, 8, 9, 10 and FT-6 green by design; GREEN 9 of 9 (8 of 8 on lean,
  FT-7 compiled out); mutations M1 to M12 plus M6b run locally and each red for its reason; `Cargo.lock` 327 packages
  on both; `cargo deny` clean; `just wasm-check` exit 0.
- Deviations to judge: (a) `tests/build_refuses_output_flags.rs` positive control had run `build --strict` expecting
  exit 0, the behaviour this spec removes, so it now runs a plain build, then `build --check --strict`; (b) the
  watermark argv in `minimal_args` needed `--size 8` on the 32×32 fixture (5% coverage limit); (c) FT-2 is a unit
  test in `src/cli/global_flags.rs`, the test file holds FT-1 and FT-3 to FT-10.
- Not applied by design (SPEC-151 was an open draft): `docs/api-contract.md` lines 36, 37, 39, 192, 591-593 and
  `docs/data-model.md:122`. The orchestrator applies them at ship. Do not flag their absence.

## Checks (do all; report each)
1. **Scope and commit order.** `git diff origin/main...HEAD --name-only`: 16 files, none in the PROJ-017 list above;
   all commits signed off; no mutation in the diff. Say what each of the 16 is.
2. **Merge against current `main`.** In a scratch tree `git merge --no-commit --no-ff origin/main`. If SPEC-151 has
   merged, the merge must be clean, and `tests/cli_flags_honest.rs` must name `pad` in `minimal_args` or FT-1 fails
   naming `pad` (21 commands then, not 20). Run `just check` on the merged tree. Report conflicts, do not resolve.
3. **Read the code.** `global_flags.rs` in full, `logger.rs` in full, the `dispatch`/`run` ordering in
   `src/cli/mod.rs`, `ops.rs` (`watermark_overlay`, the font read), the new error variant and its exit code. Look for:
   a verb the guard cannot see (the `Commands` match: is it exhaustive, or does a `_ =>` arm let a new verb through?);
   a flag spelled two ways (`--frozen`/`--locked` aliases, `--check` abbreviations clap accepts, a short form); a
   refusal that runs after a side effect (cache dir, output dir, lockfile created); `-v -v` versus `-vv`; a logger
   installed twice; the `log` filter leaking third-party `trace`; anything that writes to stdout.
4. **Drive the matrix yourself, not through the build's test.** Build the binary, make a fresh 32×32 fixture
   (say what tool). For **every** leaf verb you find in `crustyimg --help` and each meta subcommand, run each of
   `--keep-gps --check --frozen --locked --strict --no-cache --watch`: expect exit 2, the R-row line exactly, and an
   unchanged `ls -lTR` snapshot of the scratch directory. Report the row count and any miss. Then the positive
   controls on the same binary: `build`, `build --check`, `--frozen`, `--locked`, `--no-cache`, `build --check --strict`
   exit 0; `build --strict` and `build --watch --strict` exit 2 with R4 and the watcher never starts; `build --watch`
   starts (kill it, say how, no `timeout(1)` on macOS). Also put the flags **after** the verb, **before** it, after
   `--`, and as `--flag=value` forms. Does `--keep-gps=false` still hit R6? Does `--no-cache` before the verb still
   work on `build`?
5. **`--keep-gps` in the help and the docs.** Absent from `crustyimg --help` and every verb's `--help`; still parses.
   `grep -rn -e '--keep-gps' README.md docs SECURITY.md AGENTS.md skills integrations` shows only refused-form lines;
   list every hit and say which would still tell a reader the flag works. Confirm the R6 `meta copy` recipe it names
   really works: `resize in.jpg -o out.jpg -y` then the exact `meta copy …` from the message on a JPEG with GPS, and
   read the GPS back with `exiftool` (independent reader). Same for a PNG. Say what it does for WebP.
6. **`-v` / `-vv` / `-Q`.** With the build's SVG fixture, then a second input of your own that makes some other crate
   warn (a truncated PNG or a JPEG with a bad marker, whichever produces a `log` record): quote stderr for no flag, `-v`,
   `-vv`, `-v -v`, `-Q`, `-v -Q`, `-Q -v`, `-vv -Q`. `-v` must show no `debug:` line; `-Q` must win in both orders;
   stdout bytes must be 0 where the verb writes a file; the output file must be byte-identical with and without `-v`
   (`cmp`). Drive `diff x x` and `resize a b --out-dir o -j 4` for the "exactly twice" claim, repeated 10 times: any
   flake? Does a warning interleave mid-line under `-j 4`? Does `--json` output stay valid JSON on stdout with `-vv`?
7. **The lean build.** `cargo build --no-default-features`, run `-vv` on an AVIF fixture: the `avif_parse` debug line
   the spec says appears on the lean leg. Quote it. Confirm FT-7 compiles out there and FT-8 does not.
8. **Font and overlay.** Missing and directory `--font`, missing and directory `--image`, an unreadable file
   (`chmod 000`; say if you are root), a non-image `--image` (exit 4, old message kept), a corrupt PNG `--image`
   (ruling: unchanged, out of scope; quote what it prints), a `--font` that is a valid file but not a font. Exit codes
   and the quoted path with a space, a `"`, and a unicode name. Nothing written in any case (`ls` before and after).
9. **Dependency and wasm.** `cargo tree -i log -e normal --depth 1`; `grep -c '^name = ' Cargo.lock` on `main` and
   the branch (must both be 327); `cargo deny check` last lines; `just wasm-check` exit via `$?`, not a pipe; confirm
   `log` is under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` or equivalent and that the wasm `cargo
   tree -i log --target wasm32-unknown-unknown` is empty for crustyimg itself (other crates may use it).
10. **Re-run the gates and the mutation table.** `just check`, `cargo clippy --all-targets -- -D warnings` (default
    and `--no-default-features`), `cargo fmt --check`. Re-apply in a scratch tree, each with a non-empty `git diff`,
    reverted to empty: M1, M2, M3, M4, M5, M6, M6b, M7, M8, M9, M11, M12. Each must go red for its reason with the
    failing line quoted (trap 13); the control for each is the exact old code restored (trap 14). Note the build
    reports M1 also turned `tests/cli.rs::watch_on_non_build_subcommand_is_usage_error` red and M9 turned
    `no_cache_flag_bypasses_store` red: confirm both.
11. **Mutations the spec did not list** (try at least five): the guard's verb exclusion widened so `meta copy` is
    skipped; `--locked` dropped from R2's flag set while `--frozen` stays; R4's `--check` test inverted so
    `build --check --strict` is refused; `-Q` made to win only when given after `-v`; the logger installed only when
    `verbose > 0` but with the filter never raised for `-vv` (so `-vv` equals `-v`); the font error mapped to exit 4;
    the overlay error losing the path; the guard run after the lockfile read in `build`. Which are caught, which are
    gaps? A gap is a punch-list item, not a rejection, unless it lets a flag be accepted and ignored.
12. **Docs and help strings.** Apply the spec's Docs sweep table sentence by sentence against the diff; the proposed
    CHANGELOG, `docs/recipes.md:123` and `skills/crustyimg/SKILL.md:67-68` text is for the orchestrator, not a gap.
    List every rendered `--help` or error string that still carries a spec or DEC id (SPEC-150 owns that sweep; list
    only, do not fix). Check `-v`'s help text now says what it does, and `-Q`'s says it wins.
13. **The two test-file changes.** Read the `build_refuses_output_flags.rs` change and the `--size 8` change. Does the
    first still prove what the file was written to prove (that `build` refuses output flags) with a positive control
    that exits 0? Does `--size 8` make the watermark row run the real path (file written on a control run)?
14. **CI legs.** Every leg at the head, including windows-latest, macOS, lean, msrv, DCO. Name the 12 skipped and say why.
    On the Windows leg, quote one FT-1 or FT-3 line proving the CLI tests ran there (paths with `\`, exit codes
    unchanged).

## Guardrails
- ⛔ Push nothing to #256. No throwaway PR is needed: every row is local. If you do open one, use a
  `verify-scratch/*` branch, `gh pr close --delete-branch`, and `git ls-remote --heads origin 'verify-scratch/*'` must
  be empty at the end.
- ⛔ Nothing publishes or tags. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
- **Budget ~60 exchanges.** Optional findings go in one punch list; the orchestrator will not re-verify them (rule 7).
  Only a required defect (a flag accepted and ignored, a refusal after a side effect, a wrong exit code, a doc that
  still promises `--keep-gps`) triggers a fix round and a narrow re-verify.

## Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced
by `-`); identify **by content**: search `VERIFY prompt: no flag is accepted and ignored`. Python or `/usr/bin/grep`.
⚠ Dedup by `.message.id`: input, cache_creation, cache_read once per id, **MAX** `output_tokens`. Price per
component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10).

## Report back (your final message)
First line, one of:
- `✅ APPROVED — merge PR #256 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #256 at <SHA> needs: <required fixes>.` (separate required items from optional ones)
- `❌ REJECTED — <reason>.`
Then checks 1 to 14 with evidence (quoted lines), the matrix row count, the merged-tree result, cleanup confirmation
(no `verify-scratch/*` branches), anything only the maintainer can do, and your measured cost.
