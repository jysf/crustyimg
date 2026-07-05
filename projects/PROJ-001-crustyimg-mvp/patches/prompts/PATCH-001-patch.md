# PATCH-001 patch prompt — `--out-dir` auto-creates the target directory

Start a **fresh session**. You are the IMPLEMENTER for **PATCH-001** in the `crustyimg`
repo (cwd is the repo root). This is a **PATCH** (DEC-043 lightweight lane): a bounded fix
to shipped behavior — **design + build collapsed into one pass** (write the failing tests
AND the fix together). No new feature, no new flag, no scope creep. Open a PR and STOP for
an independent verify.

## Read first
1. `projects/PROJ-001-crustyimg-mvp/patches/PATCH-001-out-dir-auto-create.md` — the whole
   patch (Problem / Fix / Failing Tests / Notes).
2. `decisions/DEC-044-out-dir-auto-create.md` — the decision (auto-create in `Sink::Dir`;
   new `OutDirCreate` error; DEC-035 guards unchanged).
3. `decisions/DEC-035-*.md` and `src/sink/mod.rs` (`safe_join`, `SinkError`, the
   `Sink::Dir` write path) + `src/cli/mod.rs:2992` (`run_responsive`'s existing
   `create_dir_all` — the precedent to dedupe).

## What to do (test-first, in one pass)
1. **Write the failing tests** (per the patch's Failing Tests):
   - `tests/sink.rs`: `out_dir_is_created_when_missing`, `out_dir_creates_nested_parents`,
     `out_dir_creation_failure_is_typed` (file-at-path → `SinkError::OutDirCreate`, exit 5).
   - `tests/cli.rs`: `batch_out_dir_created_end_to_end` (`resize … --out-dir <fresh-dir>`
     → exit 0, outputs written).
2. **Add `SinkError::OutDirCreate { path: String, source: std::io::Error }`** (or similar)
   with a clear message `"could not create output directory {path}: {source}"`. It maps to
   exit 5 via the existing `CliError::Sink(_) => 5` — no change to `code()` needed.
3. **Create the dir in the `Sink::Dir` write path** (`src/sink/mod.rs`): before opening the
   output file under `dir`, `std::fs::create_dir_all(dir)` mapped to `OutDirCreate` on
   error. Idempotent (runs per write; harmless if the dir exists).
4. **Dedupe `run_responsive`** (`src/cli/mod.rs:2992`): remove its now-redundant explicit
   `create_dir_all(out_dir)` (the sink covers it). Keep only if removal changes behavior —
   confirm the responsive tests still pass.
5. **Do NOT touch `safe_join` or the DEC-035 output-name traversal/symlink guards.** They
   still validate every file written into the dir; the auto-create must not weaken them.

## Hard rules
- Bounded fix only — no new flags/commands, no unrelated refactor.
- DEC-043 + DEC-044 are already authored — do NOT create a new DEC.
- The security boundary (DEC-035) must be provably unchanged: its existing guard tests
  must still pass verbatim.

## Gates (all must pass)
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test                                   # incl. the new dir-create tests + unchanged DEC-035 guard tests
cargo build --no-default-features
cargo deny check advisories bans sources licenses
```
Sanity: `mkdir` NOT needed — e.g. build the binary and run
`target/debug/crustyimg resize <some.png> --max 100 --out-dir /tmp/patch001_new -y` into a
fresh dir → exit 0, file written.

## Git / PR
- Branch `fix/patch-001-out-dir-auto-create` off current `main`. Confirm
  `git branch --show-current` before every commit. Ignore untracked `reports/*.md` /
  `TESTING-WITH-YOUR-PHOTOS.md`.
- PR title: `fix(PATCH-001): --out-dir auto-creates the target directory`.
- PR body: reference DEC-044 (behavior) + DEC-035 (guards unchanged) + DEC-043 (patch
  lane); state plainly it adds no flag/command and the traversal/symlink guards are
  untouched.
- Fill the patch's `## Patch Completion` + the 2 patch-reflection answers; append a patch
  cost session entry (`agent: claude-sonnet-4-6`, numerics null).
- Update `CHANGELOG.md` `[Unreleased] → Fixed` with the line from the patch's Ship section.

## When done
Set the patch's `cycle:` to `verify` (edit the frontmatter by hand — patches have no
`just advance-cycle`), open the PR with `gh`, and **STOP** — an independent verifier
reviews next; the orchestrator pauses for the maintainer before merge.
