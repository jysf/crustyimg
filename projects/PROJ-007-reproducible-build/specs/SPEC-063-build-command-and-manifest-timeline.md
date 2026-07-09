# SPEC-063 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — the `build` command + `crustyimg.build.toml` manifest (PROJ-007 skeleton); Failing
  Tests (parse valid/unknown-field/bad-version/oversize/missing-field, build-runs-all-targets,
  default-file discovery, missing-recipe-fails-before-writing, idempotent-rerun-without-yes, summary,
  partial-failure→exit-6) + full Implementation Context. Load-bearing probe done in design: read
  `run_apply`/`apply_one` → **the executor is `apply_one` looped over targets (pure reuse, no new dep)**;
  a serde/toml probe confirmed the manifest schema (string-or-list `source`, `deny_unknown_fields`,
  `version`). Format contract → **DEC-057** (emit at build). Overwrite-owned-outputs is the deliberate
  difference from `apply`. Framing, 2026-07-08.
- [ ] **build** — `src/build/mod.rs` (BuildManifest/Target/SourceSpec serde + BuildError, versioned,
  size-guarded) + `Commands::Build` + `run_build` in cli (loop targets over `apply_one`, rayon, default-file
  discovery, summary, exit codes, `Overwrite::Allow`), `pub mod build` in lib, unit + integration tests,
  DEC-057. No new dep. Verify default + lean + `just deny` + clippy + fmt.
- [ ] **verify** — fresh session; re-run all gates independently, confirm the executor reuses `apply_one`
  (no duplicated worker), overwrite-owned-outputs + idempotent re-run, partial-batch exit-6, manifest
  hardening (size guard + deny_unknown_fields + version), no new dep, DEC-057 recorded.
- [ ] **ship** — merge PR, cost sessions + totals, ship reflection, archive to done/, update STAGE-020
  backlog + PROJ-007 stage plan (STAGE-021 cache next).
