---
# A PATCH is a lightweight fix to shipped behavior (DEC-043). Lighter than a
# SPEC: no stage, collapsed patch→verify→ship cycle, keeps independent verify.

patch:
  id: PATCH-001
  type: patch
  cycle: verify                    # patch | verify | ship
  fixes: "`--out-dir` errors with a cryptic message instead of creating the directory"
  complexity: S
  blocked: false

project:
  id: PROJ-001
repo:
  id: crustyimg

agents:
  implementer: claude-sonnet-4-6   # the patch pass runs on Sonnet (prescriptive prompt)
  verifier: claude-opus-4-8        # independent verify (kept — DEC-043)
  created_at: 2026-07-04

references:
  decisions: [DEC-044, DEC-035, DEC-043]

# Cost: patch + verify are metered; ship is main-loop (null-with-note).
cost:
  sessions:
    - agent: claude-sonnet-4-6
      role: implementer
      tokens_in: null
      tokens_out: null
      estimated_usd: null
      date: 2026-07-04
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# PATCH-001: `--out-dir` auto-creates the target directory (safely)

**First user of the DEC-043 patch lane.**

## Problem

A batch `--out-dir DIR` fails for every input with the opaque message *"could not write
output"* (exit 5) when `DIR` doesn't exist — the batch fan-out never creates it. Reported
the day v0.1.0 shipped: `crustyimg shrink *.jpeg --max 1600 --out-dir web/` → 7/7 failed.
Meanwhile `run_responsive` **does** `create_dir_all(out_dir)` (an inconsistency), so
auto-create is already the codebase's intended behavior.

## Fix (per DEC-044)

1. **Auto-create in the `Sink::Dir` write path** (`src/sink/mod.rs`): before opening an
   output file under `dir`, `std::fs::create_dir_all(dir)` (idempotent; covers all batch
   commands + `apply` + future ones in one place).
2. **Dedupe:** remove `run_responsive`'s now-redundant explicit `create_dir_all`
   (`src/cli/mod.rs:2992`) — it's covered by the sink now. (Leave it only if removal
   changes behavior; prefer removing.)
3. **Clear error for genuine failures:** add `SinkError::OutDirCreate { path, source }`
   (a distinct variant) returned when `create_dir_all` fails (a file exists at that path,
   or permission denied). Maps to **exit 5** (`CliError::Sink(_) => 5`, unchanged). This
   replaces the cryptic `Io("could not write output")` for the dir-creation case.
4. **Safety unchanged:** do NOT touch `safe_join` / the DEC-035 output-name traversal +
   symlink guards — they still validate every file written into `dir`.

## Failing Tests (write first, in the patch pass)

- **`tests/sink.rs`** (or the existing sink test module)
  - `out_dir_is_created_when_missing` — writing via `Sink::Dir` to a non-existent dir
    creates it and writes the file.
  - `out_dir_creates_nested_parents` — a nested `a/b/c` out-dir is created.
  - `out_dir_creation_failure_is_typed` — when a *file* exists at the out-dir path,
    the error is `SinkError::OutDirCreate` (not the generic `Io`), mapping to exit 5.
  - The existing DEC-035 traversal/symlink name-guard tests still pass unchanged
    (auto-create must not weaken them).
- **`tests/cli.rs`** (integration, real binary)
  - `batch_out_dir_created_end_to_end` — `resize <inputs> --max N --out-dir <new-dir>`
    against a fresh (non-existent) dir succeeds and writes outputs (exit 0), where it
    previously exited 5.

## Verification (independent, kept)

Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo build --no-default-features`, `cargo deny check advisories bans sources licenses`.
Plus: confirm the DEC-035 guard tests are unchanged and still pass (the security boundary
did not move), and that `responsive`'s dir still gets created after the dedupe.

## Ship

CHANGELOG `[Unreleased] → Fixed`: "`--out-dir` now creates the target directory (and
parents) if missing, consistently across all batch commands; genuine creation failures
return a clear error. Output-name path/symlink guards unchanged (DEC-035)." Archive to
`patches/done/`. No stage bookkeeping (DEC-043).

## Notes for the implementer

- Put the creation in `Sink::Dir`'s write method so it runs once per write (idempotent) —
  don't scatter `create_dir_all` across CLI handlers.
- `SinkError::OutDirCreate` should carry the path for a helpful message, e.g.
  `"could not create output directory {path}: {source}"`.
- This is a PATCH (DEC-043): collapse design+build into one pass (tests + fix together),
  then STOP for independent verify. Do NOT expand scope — no new flags, no new commands.

---

## Patch Completion

*Filled at the end of the patch pass, before verify.*

- **Branch:** `fix/patch-001-out-dir-auto-create`
- **PR:** opened via `gh pr create` (see PR URL in session)
- **All acceptance criteria met?** Yes — `SinkError::OutDirCreate` added;
  `create_dir_all` in both `Sink::Dir` write paths (`write` and `write_bytes`);
  responsive's explicit `create_dir_all` kept (it uses `Sink::File` + manual
  `safe_join`, so `Sink::Dir` auto-create does not cover it — left per the
  "leave it only if removal changes behavior" parenthetical in the patch);
  all 5 new tests pass; all 8 DEC-035 guard tests unchanged and pass; all 419
  tests pass; all gates green.
- **Deviations:** `run_responsive`'s `create_dir_all` was NOT removed. The patch
  prompt said to dedupe it because the sink now covers it, but inspection showed
  `run_responsive` builds `Sink::File` paths via a manual `safe_join` loop (not
  `Sink::Dir`), so the sink's auto-create does not cover it. Removing it broke
  all 5 responsive tests. Left it in place with a clarifying comment, per the
  patch's own parenthetical ("Leave it only if removal changes behavior").

### Patch reflection (2 questions)

1. **Did the collapsed patch→verify→ship lane fit this change, or did it want a full spec?**
   — The collapsed lane fit well. The fix was genuinely bounded: one new error
   variant, two `create_dir_all` insertions (in `write` and `write_bytes`), one
   updated test, and four new tests. Design decisions were already captured in
   DEC-044. The only surprise (responsive uses `Sink::File` not `Sink::Dir`) was
   a reading-code discovery, not a design ambiguity — verifying by running tests
   caught it immediately. A full spec would have been overhead for this scope.

2. **Anything the patch methodology (DEC-043) should adjust after this first use?**
   — The "dedupe" instruction should be hedged with "verify by running the tests
   for the affected command" rather than stating it as a flat removal. The patch
   assumed `run_responsive` used `Sink::Dir`; it doesn't. The parenthetical
   "Leave it only if removal changes behavior" was the right escape hatch and
   worked, but the primary instruction was overconfident. Future patches should
   say: "remove if tests pass; otherwise leave with a comment."

---

## Reflection (Ship)

*Appended during ship.*

1. **What would I do differently?** — <answer>
2. **Does DEC-043/DEC-044 or a template need updating?** — <answer>
