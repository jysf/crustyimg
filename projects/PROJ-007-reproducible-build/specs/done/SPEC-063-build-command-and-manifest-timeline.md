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
- [x] **build** — `src/build/mod.rs` (BuildManifest/Target/SourceSpec serde + BuildError, versioned,
  size-guarded) + `Commands::Build` + `run_build` in cli (loop targets over `apply_one`, rayon, default-file
  discovery, summary, exit codes, `Overwrite::Allow`), `pub mod build` in lib, unit + integration tests,
  DEC-057. No new dep. Verify default + lean + `just deny` + clippy + fmt.
  → PR #69, 2026-07-08. **Green:** 601 tests (10 new unit + 8 new integration), clippy default + lean,
  fmt, lean build, `just deny` unchanged (Cargo.toml/lock untouched — no new dep). Executor deviates
  from the sketch: **prepare ALL targets (recipe parse + pipeline probe + source resolve) before
  executing any**, so a bad target #2 can't strand target #1's outputs; `load_recipe` extracted from
  `run_apply` and shared; `apply_one` reused verbatim. Manifest paths are cwd-relative (recorded in
  DEC-057). Est. 260k tok / ~$2.34 (main-loop estimate).
- [x] **verify** — ✅ APPROVED (fresh Opus session, run from a clean state). Re-ran all gates (default 601,
  lean 601, clippy×2, fmt, `just deny` green + `git diff main -- Cargo.toml/lock` empty → no new dep) and
  REPRODUCED both hazards against the real binary (stem-collision + the manifest size / name-escape
  guards). Confirmed fail-before-write is multi-target, `apply_one` not duplicated (`load_recipe`
  extracted, `run_apply` unbroken), PR #69 24/24 CI green (rows pulled raw, not the summarizing wrapper).
  2 non-blocking notes → carried at ship. 2026-07-08.
- [x] **ship** — squash-merged PR #69 → main (a254fe8); appended verify + ship cost sessions + totals
  (440k, labelled estimates §4), ship reflection, marked cycle ship; archived to done/; STAGE-020 shipped
  (single-spec stage). Added the injective source→output constraint to DEC-057 (STAGE-022 blocker) and
  carried the collision into STAGE-021 framing. 2026-07-08.
