# SPEC-065 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — the injective source→output guarantee (STAGE-022's unblocker; discharges DEC-057's
  recorded blocker). Reject, at `run_build`'s prepare phase (after all targets resolved, **before**
  `Cache::open` / any write / any `.crustyimg/`), a build whose targets map two inputs to the same output
  path — global across all targets, typed `CliError::OutputCollision` → exit 2. Failing Tests:
  detects-first-duplicate / no-collision-when-distinct / order-preserving (pure lib) + exit-2 map (cli) +
  colliding-stems-rejected-no-write / disambiguating-template-builds / cross-target-collision /
  non-colliding-unaffected (integration). Implementation Context: pure `find_output_collision` in
  `src/build`; collision key = `out` dir + `expand_template` with **`{ext}` normalized to a sentinel**
  (the output ext needs a decode — DEC-058 stores it in the cache entry — so the pre-decode check is
  **conservative: over-detect, never under-detect** — an input-ext proxy would silently miss a real
  format-transforming collision `a/logo.png`+`b/logo.svg`→`logo.png`). No new dep; no lockfile (SPEC-066).
  Mark DEC-057's injective section RESOLVED at build; **no new DEC**. Framing, 2026-07-09.
- [x] **build** — add `find_output_collision` + `OutputCollision` (pure, `src/build`); insert the global
  check after phase 1 in `run_build`; `CliError::OutputCollision` → exit 2 (+ `exit_code_mapping_is_total`).
  Make all Failing Tests pass. Verify default + lean + `just deny` + clippy + fmt; mark DEC-057 resolved.
  Done 2026-07-09 (PR #71): 637 tests green default + lean, clippy ×2 / fmt / deny clean, no new dep.
  Sentinel is the printable `{ext}` (not NUL) so the collision message reads; out-dir normalization also
  drops `./`, without which two spellings of one dir would slip the cross-target check. No new DEC.
- [ ] **verify** — fresh session. Re-run gates; reproduce on the real binary: a same-stem target exits 2
  before any write / no `.crustyimg/`; a disambiguating template builds; a cross-target collision is
  caught; a normal multi-input build is unaffected (no false positives). Confirm no new dep, DEC-057 marked.
- [ ] **ship** — merge PR; verify + ship cost sessions + totals + reflection; archive to done/; advance the
  STAGE-022 backlog (SPEC-065 shipped → SPEC-066 lockfile next, now unblocked). Update the PROJ-007 brief.
