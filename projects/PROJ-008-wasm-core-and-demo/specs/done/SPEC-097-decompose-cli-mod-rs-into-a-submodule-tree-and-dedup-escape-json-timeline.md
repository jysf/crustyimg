# SPEC-097 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-097-<cycle>.md`.

## Instructions
- [x] design — framed 2026-07-19 from the pre-launch Rust audit's one unambiguous structural item.
  Split `src/cli/mod.rs` (6,483 lines) into a `cli/` submodule tree (candidate seams: build / report /
  optimize / ops / common, thin `mod.rs` front door + dispatch + re-exports) and dedup the two
  hand-rolled `escape_json` (`cli/mod.rs:2323` + `lint/report.rs:124`). PURE mechanical, byte-identical
  gate; external contract is just `crustyimg::cli::run()`. Framing on a **design branch off main** —
  **build GATED on maintainer review of the approach + the verification gate** (a 6k-line move). Sonnet
  or Opus build (maintainer's call), Opus verify. `run_optimize` arg-bundling explicitly deferred.
- [x] build — Sonnet, primary checkout. `cli/mod.rs` **6,483 → 1,426** lines, split into
  `build/report/optimize/ops/common` + thin `mod.rs` front door + re-exports; `escape_json` deduped to
  one `pub(crate) cli::report::escape_json` (equivalence proven before merge). Golden harness
  (`scripts/cli-golden.py`, 21 cases) byte-identical after every one-module-per-commit; tests verified by
  name-diff (0 dropped). Deviations: `common.rs` extracted first (dep order); `plural()`→`build.rs`
  (grep-caught it fails the ≥2-caller rule for `common.rs`). No signature changes.
- [x] verify — ✅ CLEAN (Opus, primary checkout, adversarial, independent oracle). Rebuilt the pre-split
  `main` binary + branch, **27/27 golden cases byte-identical** (stdout+stderr+exit+per-file SHA-256) —
  caught the clap `argv[0]`-usage-line subtlety the build's harness missed (invoke both as `crustyimg`),
  extended coverage 21→27, added a **function-body brace-matched check across ~170 fns** for paths the
  commands don't reach (0 dropped). All 11 `pub` paths resolve (compile-proof test); 0 tests dropped
  (leaf-multiset + full-path reconciliation); no widened visibility; no signature changes; 3-way
  `escape_json` equivalence over 55 adversarial inputs. Green: native/lean/wasm/clippy/fmt, test 770 /
  avif 783/0. +2 verify-only hardening commits (harness coverage + re-export compile-proof).
- [x] ship — squash-merged PR #103 (**701b7b0**) 2026-07-19, CI CLEAN. cli/mod.rs is legible (1,426 lines,
  a submodule tree) with byte-identical behavior — the r/rust-facing code-quality win. No DEC. Bookkeeping:
  cycle→ship, 3 cost sessions with `model:` (build Sonnet $6.48 / verify Opus ~$10.5 / ship $0.5 ≈
  **$17.48**), timeline, archive, STAGE-031 backlog. Filed follow-up: strict-JSON `escape_json`
  conformance (byte-identical to main; own behavior spec).
