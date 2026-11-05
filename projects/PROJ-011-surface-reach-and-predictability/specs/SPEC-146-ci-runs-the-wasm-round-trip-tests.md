---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-146
  type: chore                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-054
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null                # chosen at build
  created_at: 2026-11-05

references:
  decisions:
    - DEC-009   # the CI matrix; governs .github/workflows/**
    - DEC-064   # wasm target; the runner, and why not `wasm-pack test`
    - DEC-065   # wasm feature set; a CI job must build through the just recipe
    - DEC-066   # wasm size profile (not touched: the tests use the test profile)
  constraints: []
  related_specs:
    - SPEC-072  # created tests/wasm_roundtrip.rs and .cargo/config.toml's runner
    - SPEC-142  # the `changes` job and scripts/ci-code-paths.sh this job joins
    - SPEC-143  # its verify filed the STAGE-054 entry this spec replaces

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-054's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-054 follow-through, and STAGE-053's "a wasm32 CI leg" deliverable: the 47 wasm
  round-trip tests run on every code PR, so a wasm-only regression turns CI red.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5816022
      tokens_breakdown:
        input: 96
        output: 33519
        cache_creation: 126630
        cache_read: 5655777
      estimated_usd: 4.46
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive frame session, worktree docs/spec-146-frame. Measured from transcript
        c1c0637c-82d7-4b4b-93aa-37bb0edf9e18.jsonl, found by the prompt's title, deduped by
        `.message.id` (48 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 2351995
      tokens_breakdown:
        input: 44
        output: 38494
        cache_creation: 113391
        cache_read: 2200066
      estimated_usd: 2.77
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive design session, worktree docs/spec-146-design. Measured from transcript
        7a6d587e-9810-4a4c-830c-3e9126a888e4.jsonl, found by the prompt's title, deduped by
        `.message.id` (21 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-146: CI runs the wasm round-trip tests

## Context

`tests/wasm_roundtrip.rs` holds 47 `#[wasm_bindgen_test]` tests (45 on the slim feature set). They
are the only tests that drive `src/wasm.rs` (the npm package's and the demo's API) inside a real
wasm VM. They run only under `just wasm-test` and `just wasm-test-slim`, which need
`wasm-bindgen-test-runner` installed. **No workflow in `.github/workflows/` runs either**, so a
wasm-only regression merges on green CI. `pages.yml` (demo smoke) and `ci.yml`'s
`ddphotos-parity` exercise the wasm build through one path each and run none of these tests.

Two backlog entries describe the same gap, and this spec folds both: STAGE-054's "No CI job runs
the wasm round-trip tests" (filed by SPEC-143's verify, 2026-10-30) and STAGE-053's "A wasm32 CI
leg running `just wasm-check` + `just wasm-test`" (moved there from STAGE-038 item #8).

## Goal

A new `ci.yml` job, `wasm-roundtrip`, runs `just wasm-check`, `just wasm-test` and
`just wasm-test-slim` on every code-touching PR and push to `main`, with a runner whose version is
checked against `Cargo.lock`. A wasm-only regression turns that named job red.

## Frame

**Verdict: GO, [S].** Framed 2026-11-05 (Opus 5.5, worktree `docs/spec-146-frame`).

### Premise check (run, not read)

- **The gap is real.** No workflow in `.github/workflows/` (7 files) runs
  `wasm-test`; the only mention is a comment at the end of `ddphotos-parity` (`ci.yml:100-102`).
- **Branch protection** (`gh api repos/jysf/crustyimg/branches/main/protection`, readable):
  required checks are exactly the three `build / test / clippy / fmt (<os>)` legs, `strict: true`.
  `ddphotos-parity` is advisory, as its comment says.

### Answers to the nine questions

1. **Does it run today? Yes, all green on both feature sets.** Fresh `CARGO_TARGET_DIR`, rustup
   stable, `wasm-bindgen-test-runner 0.2.126`, Node v22.17.0, macOS arm64:
   - `just wasm-check`: exit 0, `Finished dev profile … in 23.25s`, **one warning**:
     `constant MANIFEST_DROPPED_WARNING is never used` (`src/metadata/mod.rs:129`). See the
     out-of-spec finding below.
   - `just wasm-test`: `test result: ok. 47 passed; 0 failed; 0 ignored; 0 filtered out; finished in 3.82s`
   - `just wasm-test-slim`: `test result: ok. 45 passed; 0 failed; 0 ignored; 0 filtered out; finished in 3.70s`
   - The slim delta, from diffing the two test lists: the slim set drops
     `svg_rasterizes_in_wasm`, `svg_text_in_a_named_family_draws_in_wasm` and
     `svg_text_renders_glyphs_in_wasm`, and adds `slim_wasm_refuses_svg_with_the_feature_hint`
     (47 − 3 + 1 = 45).
   - **Nothing is skipped or environment-dependent.** There is no `wasm_bindgen_test_configure!`,
     so the runner uses Node, not a browser. Fixtures are `include_bytes!`, fonts are the bundled
     ones (no system font lookup), and no test reads time. 0 ignored on both sets.
2. **The runner: use `taiki-e/install-action`, with the version read from `Cargo.lock` and
   asserted after install.**
   - `taiki-e/install-action`'s `manifests/wasm-bindgen.json` (alias `wasm-bindgen-cli`) lists
     **0.2.126** with a sha256 for `x86_64_linux_musl` (`064948d5…d869d`). It installs
     `wasm-bindgen-test-runner` from the upstream GitHub release. Prebuilt, so seconds rather than
     the minutes of `cargo install wasm-bindgen-cli --locked`.
   - `wasm-pack test` is ruled out by DEC-064 and `.cargo/config.toml`: it hardcodes `--tests` and
     drags every native CLI-driving integration test into the wasm build.
   - ⚠ **A version mismatch does NOT fail loudly on its own.** The runner refuses a `.wasm` only
     when the *schema* versions differ (`strings` of the installed runner: "rust Wasm file schema
     version … must exactly match"). `wasm-bindgen-shared`'s `SCHEMA_VERSION` is `"0.2.122"` for
     every release from 0.2.122 to 0.2.127 (read from the crate source for 0.2.126, and from the
     upstream tags for 0.2.123, .125, .127). 0.2.128 and 0.2.130 bump it. So a runner from 0.2.125
     or 0.2.127 would run our 0.2.126 tests silently. **The job must assert
     `wasm-bindgen-test-runner --version` equals `Cargo.lock`'s `wasm-bindgen` version**, and it
     should read that version from `Cargo.lock` rather than hard-code `0.2.126` (which today
     appears in `Cargo.toml`, the `justfile` message and the `ci.yml` comment). The design names
     the mutation rows: a wrong version in the install, and a lock bump without the install.
   - Not driven locally: the auto-mode classifier blocked downloading and running the upstream
     release binaries, so the prebuilt 0.2.126 and the mismatched 0.2.125 were not run here. The
     schema rule above comes from source, not a driven mismatch. Verify should drive one.
3. **Node: ubuntu-latest ships Node 22.23.4** (runner-images `Ubuntu2404-Readme.md`, image
   20261026.354.1). The local runs used Node 22.17.0, so the same major. wasm-bindgen 0.2.126
   documents no Node minimum that 22 misses. **Recommend pinning `actions/setup-node` to 22**
   (one step) so an image default moving to 24 does not change the tests' VM unannounced, and
   printing `node --version` in the log.
4. **New job, `wasm-roundtrip`.** Reasons: a red job named `wasm-roundtrip` says what broke,
   while a red `ddphotos-parity` does not; it can be required while `ddphotos-parity` stays
   advisory; the two run in parallel instead of in series. The cost is a separate
   `Swatinem/rust-cache` entry (its own `key:`); sharing one with `ddphotos-parity` buys little,
   since that job builds release-profile wasm and this one test-profile wasm. It joins the
   `changes` gate. DEC-065: it calls the `just` recipes, not bare cargo.
5. **Runtime and required-ness.**
   - Measured locally (wall, fresh target dir): check 23.3 s, test 22.1 s, slim 19.3 s, so 65 s
     cold in all (CPU `user` 187 + 91 + 80 s). A warm re-run of `wasm-test` took 18.3 s, and the
     mutation runs 10.2 s and 8.8 s.
   - CI comparison from recent `main` runs: `ddphotos-parity` (same runner type, warm cache)
     takes 2.5 min (its harness step 2 min 15 s); `pages` build + smoke 3.2 min; the required
     ubuntu `check` leg ~11 min.
   - **Estimate: 2–4 min warm, 5–8 min cold**, on a 4-vCPU runner. Even cold it finishes inside
     the 11-min `check` leg, so it adds no wall time to a PR. The build records the real cold
     (first PR run) and warm (second run) times.
   - **Recommend making it required** after its first green run on `main`. It is deterministic
     (see Q1) and it is the only guard on the wasm API. Adding a required check is a
     branch-protection setting the maintainer changes, not part of the PR. A job skipped by its
     job-level `if:` reports success to a required check, so a docs-only PR is not blocked; the
     design confirms this, or copies `check`'s always-run, skip-the-steps pattern.
   - **Local gate list:** the "`just check` is a weaker gate than CI" list that STAGE-053 cites
     is not in `CLAUDE.md` (a stub that points at `AGENTS.md`) or in `AGENTS.md`. Only the
     STAGE-053 entry uses the phrase. Recommend that `just check` stays native, since it should
     not need the wasm toolchain, and that `AGENTS.md` §6 gains one line naming
     `just wasm-test` / `wasm-test-slim` and the CI job that runs them.
6. **It guards. Mutation driven:** `src/wasm.rs:152`, `self.has_alpha` → `!self.has_alpha` (the
   `hasAlpha` getter on `ImageInfo`).
   - `src/wasm.rs` is compiled only for wasm32 (`#[cfg(target_arch = "wasm32")] pub mod wasm;`
     at `src/lib.rs:84`), so every native test is unaffected by construction. `hasAlpha` is read
     by no CI smoke: `demo/`, `tests/demo_smoke.mjs` and `tests/ddphotos_parity.mjs` never read
     it. Only `tests/npm_smoke.mjs` reads it, and that runs at publish time.
   - Full set: `test result: FAILED. 46 passed; 1 failed`. The failure is
     `wasm::info_reports_png_dimensions`, `panicked at tests/wasm_roundtrip.rs:513:9: the RGBA fixture carries alpha`.
   - Slim set: `test result: FAILED. 44 passed; 1 failed`, same test, same message.
   - Restored with `git checkout -- src/wasm.rs`; `git diff` empty, and `wasm-test` back to
     `47 passed`.
   - **How CI shows it red:** `cargo test` exits 101 (`error: test failed, to rerun pass
     --test wasm_roundtrip`), so `just` exits non-zero and the step and job fail. `test result:`
     lines go to stdout and reach the log without `--show-output`.
   - ⚠ **Trap 11's "zero tests" shape remains.** If a cfg change ever compiled the `wasm` module
     out of the test, `cargo test` would print `0 passed` and exit 0. The design should make the
     job fail below a test-count floor, or on `0 passed`, and list that as a mutation row.
7. **The slim leg: include it**, as its own step. It adds ~19 s locally (est. 1–2 min CI) and is
   the only proof that the slim build refuses SVG with the `--features svg` hint, and the slim
   build is what the ddphotos edit page ships. Separate steps mean the failing step names the
   feature set.
8. **Docs: these sentences change.**
   - `ci.yml:100-102`: the "A wasm32 test leg adds ONE step here: …" comment block. Delete it,
     since the job now exists.
   - `justfile:233-241`: the `wasm-test` comment gains "CI: the `wasm-roundtrip` job runs this and
     `wasm-test-slim`". The hard-coded `--version 0.2.126` in the install hint should read
     `Cargo.lock`, or name it as the source.
   - `.cargo/config.toml:21` "`just wasm-test` wraps the command." gains "CI's
     `wasm-roundtrip` job runs it".
   - `docs/roadmap.md:210` "(2) **a wasm CI job** — today only a local `just wasm-test` floor
     guards the wasm…" is marked done.
   - `docs/research/proj-008-wasm-build.md:223` "**No wasm CI job yet.**" A research record:
     add a dated one-line note rather than rewriting it.
   - `AGENTS.md` §6: one new line (see Q5).
   - STAGE-053 §1 "no CI job runs either recipe" is stage prose, so the ship session updates it.
   - `RELEASING.md` and `docs/` user-facing pages name no wasm test command; no change.
9. **Size [S]; verification costs more than the change.** The change is about 30 lines of
   workflow YAML, a regex line, and comments. Verification must prove four things:
   - (a) the job **ran the tests**: its log quotes `47 passed` and `45 passed`;
   - (b) a wasm-only mutation **turns it red in CI**, not just locally: one throwaway commit on
     the draft PR, its red run id recorded, then reverted;
   - (c) the version assertion **fails on a mismatch**, driven as a shell step with a wrong
     version, and verify drives a real mismatched runner if the environment allows;
   - (d) a `.cargo/config.toml` change **triggers the job** (see the scope note on
     `ci-code-paths.sh`).

   That is two extra CI runs and no new Rust tests. Still [S], not [M].

### Scope

**In:**
- the `wasm-roundtrip` job in `ci.yml`: toolchain with the wasm32 target, `just`, the runner
  install pinned from `Cargo.lock` plus the version assertion, Node pinned, its own cache key,
  and three steps (`wasm-check`, `wasm-test`, `wasm-test-slim`) with a test-count floor;
- `\.cargo/` added to `scripts/ci-code-paths.sh`'s `CODE_PATHS`, with a self-test row.
  `.cargo/config.toml` holds the wasm test runner, yet a change to it today classifies as
  docs-only and skips every heavy job;
- the doc sentences in Q8;
- removing the `ddphotos-parity` comment sketch.

**Out:**
- making the check required. That is a branch-protection setting; the maintainer flips it after
  the first green `main` run (Q1 below);
- wasm32 clippy or `-D warnings` on the wasm build, which is red today on one dead constant.
  Filed on STAGE-053;
- `just check` running the wasm suite, which stays native (Q5);
- the wasm bundle-size gate in `just check` (STAGE-053's own entry);
- `npm_smoke.mjs` in CI on PRs (publish-time only, SPEC-144);
- anything under `src/operation/`, `src/recipe/`, `docs/recipes.md`,
  `tests/recipe_corpus_output.rs` or `tests/fixtures/recipe_corpus/` (PROJ-017 owns them), and
  any change to `src/wasm.rs` or `tests/wasm_roundtrip.rs`.

### Rough build file list
- `.github/workflows/ci.yml`: the new job, and the ddphotos-parity comment removed
- `scripts/ci-code-paths.sh`: `\.cargo/` in `CODE_PATHS`, plus a self-test row
- `justfile`: the `wasm-test` comment and install hint (comments and message only)
- `.cargo/config.toml`: a comment
- `AGENTS.md`: one line in §6
- `docs/roadmap.md`, `docs/research/proj-008-wasm-build.md`: one line each

None of these is a PROJ-017 path.

### Out-of-spec finding
- **The wasm32 build carries a warning that no gate sees.** `MANIFEST_DROPPED_WARNING`
  (`src/metadata/mod.rs:129`) is dead code on wasm32. No CI job lints wasm32, so adding
  `-D warnings` to this job would make it red on day one. Filed on STAGE-053 as a
  `(not yet written)` entry.
- STAGE-053's bundle-size entry says `CLAUDE.md` has a "`just check` is a weaker gate than CI"
  list. It does not, in this repo (Q5). This is noted here and not edited, because that entry
  belongs to another spec.

### Questions only the maintainer can answer
1. **Make `wasm-roundtrip` a required check?** Recommend: yes, after its first green run on
   `main` (you add it in branch protection).
2. **Should the wasm build fail on warnings (wasm32 clippy)?** Recommend: not in this spec. Take
   the STAGE-053 entry, which first removes the one dead constant.

## Design

Designed 2026-11-05 (Opus 5.5, worktree `docs/spec-146-design`). The frame's rulings 1 to 6 are
binding and are not re-opened here. What the design added, each probed rather than assumed:

- **The zero-test shape is `1 passed`, not `0 passed`.** Driven: `tests/wasm_roundtrip.rs:30`
  `#[cfg(target_arch = "wasm32")]` → `#[cfg(any())]` (compiles the `wasm` module out) and
  `just wasm-test` printed `test result: ok. 1 passed; 0 failed; 0 ignored; 0 filtered out` and
  exited **0**. The one survivor is `strict_step_params::transform_refuses_an_unknown_step_key`
  (`mod strict_step_params`, line 1665, its own `cfg(target_arch = "wasm32")`). So a floor of
  "more than zero" would stay green. Restored with `git checkout`, `git diff` empty.
- **The floor is an exact count**, per feature set, kept in one script. A minimum (`>= 45`) cannot
  see the slim step swapped for the full one (47 ≥ 45); an exact count goes red on it. The cost is
  one edited constant on the rare day a wasm test is added (47 tests in ~8 months), and the red
  message names the constant and its file.
- **One job, two steps, not a matrix.** GitHub: "A job that is skipped will report its status as
  "Success". It will not prevent a pull request from merging, even if it is a required check."
  ([using conditions](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/using-conditions-to-control-job-execution)).
  A non-matrix job skipped by its `if:` therefore reports under its own name, so the maintainer can
  require it without copying `check`'s always-run pattern (which exists because a skipped matrix
  job reports one un-suffixed name, not its legs). Two legs would also compile the shared
  dependencies twice to save ~20 s.
- **Runner install.** `taiki-e/install-action`'s `manifests/wasm-bindgen.json` lists `0.2.126` (and
  0.2.120 to 0.2.129), `rust_crate: wasm-bindgen-cli`, and its `bin` list includes
  `wasm-bindgen-test-runner`. README: "If you want to ensure that fallback is not used, use
  `fallback: none`" ([README](https://github.com/taiki-e/install-action#readme)). The design sets
  `fallback: none`, so a lock bump to a version the manifest does not list fails the install step
  instead of compiling `wasm-bindgen-cli` for minutes. Tag `@v2` (the README's form; latest release
  `v2.90.1`); the repo's `@just` tool tag stays as it is.
- **Node.** `actions/setup-node@v6`, as `publish-npm.yml:67` pins it, with `node-version: '22'` and
  `package-manager-cache: false` (as `publish-npm.yml` sets it; the repo root has no lockfile to
  cache).
- **actionlint** (`/opt/homebrew/bin/actionlint`) on a scratch copy of `ci.yml` with the job below:
  the only finding is the **existing** SC2001 style note at `ci.yml:44` (the `changes` job's
  `sed`), which is the baseline. Removing `needs: changes` adds
  `property "changes" is not defined in object type {}`.
- **`ci.yml` is a code path** (`\.github/workflows/` is in `CODE_PATHS`), so the PR that adds the
  job runs the job. The design pins that with a self-test row.

## Inputs

- **Files to read:** `.github/workflows/ci.yml` (the `changes` job, `ddphotos-parity`, `metadata`),
  `.github/workflows/publish-npm.yml:60-75` (the setup-node pin), `scripts/ci-code-paths.sh`,
  `scripts/validate-frontmatter.sh` (the repo's Ruby-YAML pattern), `justfile:193-248` (the wasm
  recipes), `.cargo/config.toml`, `tests/wasm_roundtrip.rs` (read only, never edited).
- **External:** `taiki-e/install-action@v2` (`tool:`, `fallback:`), `actions/setup-node@v6`.
- **Related code paths:** none in `src/`.

## Outputs

- **Files created:**
  - `scripts/wasm-ci-check.sh`: the version read, the runner assertion, the test-count floor, the
    workflow shape check, and a `--self-test` table for all four.
- **Files modified:**
  - `.github/workflows/ci.yml`: the `wasm-roundtrip` job; the `ddphotos-parity` comment sketch
    replaced; two steps added to the `metadata` job.
  - `scripts/ci-code-paths.sh`: `\.cargo/` in `CODE_PATHS`; self-test rows; a summary line that
    counts each failure kind.
  - `justfile`: the `wasm-test` comment and its install hint (comments and message only).
  - `.cargo/config.toml`: one comment sentence.
  - `AGENTS.md`: one line in §6.
  - `docs/roadmap.md`, `docs/research/proj-008-wasm-build.md`: one sentence each.
- **New exports / database changes:** none.

None of these is a PROJ-017 path (`src/operation/`, `src/recipe/`, `docs/recipes.md`,
`tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`).

## Acceptance Criteria

Each AC names its check (see **Checks** below) and what turns that check red.

- [ ] **AC-1 The job exists, is gated and is shaped as designed.** `ci.yml` has a non-matrix job
  `wasm-roundtrip`, `name: wasm round-trip tests (Node)`, `needs: changes`,
  `if: needs.changes.outputs.code == 'true'`, its own `Swatinem/rust-cache@v2` key
  `wasm-roundtrip`, the toolchain with `wasm32-unknown-unknown`, and the steps in the order of the
  YAML below. Checks: C1 (shape), C2 (actionlint). Red under M1, M2, M3, M5, M9, M10, M11.
- [ ] **AC-2 The runner is installed at `Cargo.lock`'s version and asserted.** The version is read
  from `Cargo.lock` (not hard-coded), installed with `fallback: none`, and
  `wasm-bindgen-test-runner --version` must equal it. Checks: C3 (self-test rows), C4 (stub
  mismatch, local), C5 (live log), C6 (live mismatch). Red under M7, M8.
- [ ] **AC-3 Node is pinned to 22.** Checks: C1 (the `setup-node` step and `'22'`), C5 (the log's
  `node --version` line starts `v22.`). Red under M10.
- [ ] **AC-4 The full feature set runs, and exactly 47 tests pass.** Checks: C5 (the log's
  `test result: ok. 47 passed` and the floor's OK line), C7 (floor mutation). Red under M4, M6.
- [ ] **AC-5 The slim feature set runs, and exactly 45 tests pass.** A separate check from AC-4
  (trap 2: two feature sets, a check per set). Checks: C5 (`45 passed` and the floor's OK line),
  C7, C8 (swap). Red under M4, M5, M6.
- [ ] **AC-6 A wasm-only regression turns the job red, and only the job.** Check: C9. The
  `src/wasm.rs:152` mutation makes both test steps red, quoting
  `wasm::info_reports_png_dimensions` and "the RGBA fixture carries alpha", while every native leg
  on the same commit is green.
- [ ] **AC-7 A cfg that compiles the tests out trips the floor, for its own reason.** Check: C7.
  The floor's message (not a compile error, not a test failure) is the red.
- [ ] **AC-8 The classifier treats `.cargo/` as code.** `.cargo/config.toml` → `true`,
  `docs/.cargo/config.toml` → `false`, `.github/workflows/ci.yml` → `true`; the self-test summary
  counts mismatched, crashed and junk rows separately. Checks: C10, C11. Red under M12, M13, M14.
- [ ] **AC-9 Nothing skips silently (trap 11).** A missing runner fails the assertion step; an
  unlisted version fails the install (`fallback: none`); a `cargo test` failure fails its step
  without a pipe. Checks: C3 (missing-runner row), C1 (`fallback: none`), C9.
- [ ] **AC-10 The CI self-checks run in CI.** The `metadata` job runs
  `scripts/wasm-ci-check.sh --self-test` and `scripts/wasm-ci-check.sh workflow`; the `changes`
  job already runs `scripts/ci-code-paths.sh --self-test`. Check: C1 asserts both `metadata` steps;
  C5 quotes their summary lines from the live log.
- [ ] **AC-11 The docs say the job exists.** Each sentence in "Docs, word for word" is in place
  and the old ones are gone. Check: C12.

## Failing Tests

**Named "Checks", because that reads truer here:** the change is a workflow, two scripts and docs,
with no `src/` and no new Rust test. Every check runs from the repo root on the build branch.
"Today" means `main` at design time.

- **C1 `scripts/wasm-ci-check.sh workflow`** (shape check; runs locally and in the `metadata` job).
  Today: red, the script does not exist. After build: exit 0, prints
  `wasm-ci-check workflow: N assertions, 0 failed`. Each failed assertion prints
  `FAIL <what>, expected <what>` and the summary counts them. Assertions are listed in the script
  spec below.
- **C2 `actionlint .github/workflows/ci.yml`** (verify, local). After build: exactly the one
  baseline finding (`ci.yml:44` SC2001, in the `changes` job, pre-existing); no finding in the new
  job or the `metadata` job. M1 adds `property "changes" is not defined in object type {}`.
- **C3 `scripts/wasm-ci-check.sh --self-test`.** Table rows for every subcommand's parser (rows
  below). Prints `PASS`/`FAIL` per row and a summary
  `wasm-ci-check self-test: N rows, P passed, V wrong verdict, M wrong message`, exit 1 if
  `V + M > 0`. A row passes only if both the verdict (ok/fail) and a key phrase of the message
  match (trap 13).
- **C4 Stub mismatch (verify, local).** A directory holding an executable
  `wasm-bindgen-test-runner` that prints `wasm-bindgen-test-runner 0.2.125`, put first on `PATH`:
  `scripts/wasm-ci-check.sh runner-version` exits 1 and prints
  `::error::wasm-bindgen-test-runner is 0.2.125 but Cargo.lock pins wasm-bindgen 0.2.126; install the exact lock version`.
  With the real 0.2.126 runner: exit 0, `wasm-bindgen-test-runner 0.2.126 matches Cargo.lock`.
  With an empty `PATH` dir holding no runner (e.g. `PATH=/usr/bin:/bin`): exit 1,
  `::error::wasm-bindgen-test-runner not found on PATH`.
- **C5 The live run (build's draft PR, then verify reads it).** The `wasm-roundtrip` job's log,
  quoted with its run id: `Cargo.lock pins wasm-bindgen 0.2.126`,
  `wasm-bindgen-test-runner 0.2.126 matches Cargo.lock`, the `node --version` line (`v22.…`),
  `test result: ok. 47 passed; 0 failed; …`, `wasm-ci-check: full set: 47 passed, as expected`,
  `test result: ok. 45 passed; 0 failed; …`, `wasm-ci-check: slim set: 45 passed, as expected`.
  From the `metadata` job: the two summary lines of C1 and C3. The build records cold (first run)
  and warm (second run) job times.
- **C6 Live mismatch (verify, throwaway commit on the draft PR).** Change the install step's
  `tool:` to the literal `wasm-bindgen@0.2.125` (listed in the manifest). Expected: the
  "Runner matches Cargo.lock" step red with C4's mismatch message; later steps skipped. This drives
  a real mismatched runner, which the frame could not do locally. Never merged; reverted exactly.
- **C7 Floor trips (verify, local first, then a throwaway commit on the draft PR).** Mutation M4:
  `tests/wasm_roundtrip.rs:30` `#[cfg(target_arch = "wasm32")]` → `#[cfg(any())]`. Locally,
  `just wasm-test` exits 0 with `1 passed` (driven at design); then
  `scripts/wasm-ci-check.sh count full <that log>` exits 1 with
  `::error::wasm-ci-check: full set: 1 passed, expected 47 (FULL_COUNT in scripts/wasm-ci-check.sh). Update FULL_COUNT if you added or removed a #[wasm_bindgen_test]; otherwise tests were compiled out.`
  In CI both test steps go red with that message (`slim set: 1 passed, expected 45 (SLIM_COUNT …)`),
  and neither `just` step itself fails. The red must be the floor's message; a compile error or a
  test panic is the wrong reason. Native legs green (the module is wasm32-only).
- **C8 Slim swap (verify, local shape + live).** M5: the slim step's `just wasm-test-slim` →
  `just wasm-test`. C1 red (`FAIL slim test step runs 'just wasm-test-slim'`). Live, if verify
  pushes it: the slim step red with `slim set: 47 passed, expected 45`.
- **C9 Wasm-only regression (verify, throwaway commit on the draft PR).** M6: `src/wasm.rs:152`
  `self.has_alpha` → `!self.has_alpha`. ⛔ **This edit is never merged.** Expected: full step red
  (`46 passed; 1 failed`, `wasm::info_reports_png_dimensions` panicked at
  `tests/wasm_roundtrip.rs:513:9: the RGBA fixture carries alpha`, then
  `::error::just wasm-test exited <non-zero>`); slim step also runs and is red (`44 passed; 1 failed`,
  same test and message). Every native leg on the same commit is green: the three `check` legs,
  `avif`, `webp-lossy`, `heic` ×2, `lean`, `msrv`, `ddphotos-parity`. That pairing is the control
  (trap 14). Verify reverts with a commit that restores the line exactly and checks
  `git diff <pre-mutation SHA> HEAD -- src/wasm.rs` is empty.
- **C10 `scripts/ci-code-paths.sh --self-test`.** Rows added: `.cargo/config.toml` → `true`,
  `docs/.cargo/config.toml` → `false`, `.github/workflows/ci.yml` → `true`. Ends with
  `ci-code-paths self-test: N rows, M mismatched, C crashed, J junk` and exits 1 if any is non-zero.
  Today the first row prints `FAIL .cargo/config.toml -> false, expected true`.
- **C11 Classifier failure kinds (verify, local).** M13 (crash) and M14 (junk) each make C10 exit 1
  with the matching counter at 1, and the other counters at 0.
- **C12 Docs grep.** Each "after" sentence below is present (`grep -F`), and these are absent:
  `A wasm32 test leg adds ONE step here` in `ci.yml`; `--version 0.2.126` in `justfile`;
  `today only a local` in `docs/roadmap.md`. Red if any sentence is not written.

### `--self-test` rows for `scripts/wasm-ci-check.sh` (C3)

| # | Subcommand | Input | Verdict | Message contains |
|---|---|---|---|---|
| 1 | count full | `test result: ok. 47 passed; 0 failed; 0 ignored; 0 filtered out; finished in 3.8s` | ok | `full set: 47 passed, as expected` |
| 2 | count full | same line with `1 passed` | fail | `full set: 1 passed, expected 47` |
| 3 | count full | same line with `46 passed` | fail | `expected 47` |
| 4 | count slim | the 47 line | fail | `slim set: 47 passed, expected 45` |
| 5 | count slim | the line with `45 passed` | ok | `slim set: 45 passed, as expected` |
| 6 | count full | a log with no `test result:` line | fail | `no 'test result:' line` |
| 7 | count full | two `test result: ok. …` lines | fail | `2 'test result:' lines, expected 1` |
| 8 | count full | `test result: FAILED. 46 passed; 1 failed; …` | fail | `did not pass` |
| 9 | count both | unknown set name `fat` | fail | `unknown set 'fat'` |
| 10 | lock-version | a lock fragment with one `name = "wasm-bindgen"` / `version = "0.2.126"` block | ok | `0.2.126` |
| 11 | lock-version | no such block | fail | `found 0 wasm-bindgen entries` |
| 12 | lock-version | two blocks (two versions) | fail | `found 2 wasm-bindgen entries` |
| 13 | lock-version | the block followed by `wasm-bindgen-shared` only | ok | `0.2.126` (the `-shared` block is not counted) |
| 14 | runner-version | output `wasm-bindgen-test-runner 0.2.126`, lock 0.2.126 | ok | `matches Cargo.lock` |
| 15 | runner-version | output `… 0.2.125` | fail | `is 0.2.125 but Cargo.lock pins wasm-bindgen 0.2.126` |
| 16 | runner-version | output `… 0.2.1260` | fail | `is 0.2.1260 but` |
| 17 | runner-version | empty output | fail | `printed no version` |

Row 13 is the derived-vs-naive row (trap 10): a naive `grep wasm-bindgen` counts `-shared`,
`-macro`, `-backend` and returns several versions. Row 16 catches a prefix compare.

## Implementation Context

### The job (write it as given, after `ddphotos-parity`, before `check`)

```yaml
  # The #[wasm_bindgen_test] round-trip (tests/wasm_roundtrip.rs) in Node, on both wasm
  # feature sets: the only tests that drive src/wasm.rs, the npm package's API. One job,
  # not a matrix: a job skipped by its `if:` reports success under its own name, so it can
  # be a required check without `check`'s always-run pattern. Keep `name:` stable; it is
  # the required-check context.
  wasm-roundtrip:
    needs: changes
    if: needs.changes.outputs.code == 'true'
    name: wasm round-trip tests (Node)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: taiki-e/install-action@just
      - uses: actions/setup-node@v6
        with:
          node-version: '22'
          package-manager-cache: false
      - name: Node version
        run: node --version
      - name: Read the wasm-bindgen version from Cargo.lock
        id: lock
        shell: bash
        run: |
          set -euo pipefail
          version=$(scripts/wasm-ci-check.sh lock-version)
          echo "Cargo.lock pins wasm-bindgen $version"
          echo "version=$version" >> "$GITHUB_OUTPUT"
      # The runner must match the lock exactly: a runner whose wasm schema happens to match
      # (0.2.122 to 0.2.127 share one) runs the tests silently on the wrong version.
      - uses: taiki-e/install-action@v2
        with:
          tool: wasm-bindgen@${{ steps.lock.outputs.version }}
          fallback: none
      - name: Runner matches Cargo.lock
        run: scripts/wasm-ci-check.sh runner-version
      - uses: Swatinem/rust-cache@v2
        with:
          key: wasm-roundtrip
      - name: just wasm-check
        id: wasm-check
        run: just wasm-check
      - name: just wasm-test (full feature set)
        shell: bash
        run: |
          set -euo pipefail
          log="$RUNNER_TEMP/wasm-test.log"
          rc=0
          just wasm-test >"$log" 2>&1 || rc=$?
          cat "$log"
          if [ "$rc" -ne 0 ]; then echo "::error::just wasm-test exited $rc"; exit "$rc"; fi
          scripts/wasm-ci-check.sh count full "$log"
      # Runs even when the full set failed, so one red run names both feature sets.
      - name: just wasm-test-slim (slim feature set)
        if: ${{ !cancelled() && steps.wasm-check.outcome == 'success' }}
        shell: bash
        run: |
          set -euo pipefail
          log="$RUNNER_TEMP/wasm-test-slim.log"
          rc=0
          just wasm-test-slim >"$log" 2>&1 || rc=$?
          cat "$log"
          if [ "$rc" -ne 0 ]; then echo "::error::just wasm-test-slim exited $rc"; exit "$rc"; fi
          scripts/wasm-ci-check.sh count slim "$log"
```

**Step by step: what it needs, and what its failure looks like in the log.**

| Step | Needs | Red looks like |
|---|---|---|
| `actions/checkout@v4` | — | checkout error (network) |
| `dtolnay/rust-toolchain@stable` + `wasm32-unknown-unknown` | rustup (on the image) | rustup error; then `just wasm-check` would say the target is missing |
| `taiki-e/install-action@just` | — | install error |
| `actions/setup-node@v6`, `'22'` | — | "Unable to find Node version" |
| Node version | Node on PATH | — (prints `v22.x.y`) |
| Read the wasm-bindgen version | `Cargo.lock` | `::error::wasm-ci-check: found N wasm-bindgen entries in Cargo.lock, expected 1`; the assignment under `set -e` stops the step |
| `taiki-e/install-action@v2`, `fallback: none` | the version output | an unlisted version errors as an unsupported tool version, with no cargo-install fallback |
| Runner matches Cargo.lock | the runner on PATH | C4's messages (mismatch, not found, no version) |
| `Swatinem/rust-cache@v2` | — | never red; a miss only costs time |
| `just wasm-check` | toolchain, `just` | a compile error |
| full / slim test steps | runner, Node, `just` | `cargo test`'s own output, then `::error::just wasm-test exited <rc>`; or the floor's message from C7 |

**The pipe trap.** Each test step runs `just … >"$log" 2>&1 || rc=$?` with no pipe, prints the log,
and exits with `just`'s own code. GitHub's default `run` shell on Linux is `bash -e`, without
`pipefail`, so a `| tee` would report `tee`'s exit code; that form is not used. The `|| rc=$?` is
what keeps `set -e` from ending the step before the log is printed.

**Environment variables (trap 16).** The job sets **no** job-level or step-level `env:`. The steps
read only what the runner provides: `$GITHUB_OUTPUT` (the lock step), `$RUNNER_TEMP` (the two test
steps). The `just` recipes set `PATH`/`RUSTC` inline for their own cargo call, as they do locally.

**Why the slim step's `if:`.** A failing full step would otherwise skip the slim step, and the red
run would not say whether the slim set also broke. `steps.wasm-check.outcome == 'success'` keeps it
from running after an install or compile failure, where it could only add noise.

### `scripts/wasm-ci-check.sh` (new)

`#!/usr/bin/env bash`, `set -euo pipefail`, header comment in the style of
`scripts/ci-code-paths.sh` (what it does, how CI calls it). Exit 0 ok, 1 check failed, 2 usage.
Errors print `::error::wasm-ci-check: …` (GitHub annotates the line; locally it is plain text).

```bash
# How many #[wasm_bindgen_test] tests each feature set must run. Exact, not a minimum: a
# minimum cannot tell the slim set from the full one. Whoever adds or removes a wasm test
# updates these in the same PR; the CI message names them.
FULL_COUNT=47   # just wasm-test       (--features avif,svg)
SLIM_COUNT=45   # just wasm-test-slim  (--features avif): 3 svg tests out, 1 refusal test in
```

Subcommands:
- **`lock-version [lockfile]`** (default `Cargo.lock`): finds every `[[package]]` whose `name` line
  is exactly `name = "wasm-bindgen"`, takes the `version = "…"` line that follows it. Exactly one,
  matching `^[0-9]+\.[0-9]+\.[0-9]+$`, is printed alone on stdout; otherwise the error from row
  11/12 (or `version '<junk>' is not x.y.z`).
- **`runner-version [lockfile]`**: `command -v wasm-bindgen-test-runner` or
  `not found on PATH`; runs `wasm-bindgen-test-runner --version`, takes the last field, compares
  it **as a whole string** to `lock-version`. Messages as C4. Put the compare in a function that
  takes the runner's output and the lock version as arguments, so the self-test drives it with
  strings.
- **`count <full|slim> <log>`**: the lines matching `^test result: `; exactly one; it must match
  `^test result: ok\. ([0-9]+) passed; 0 failed;`; the number must equal the set's constant.
  Messages as rows 1 to 9 and C7. On success prints `wasm-ci-check: <set> set: <N> passed, as expected`.
- **`workflow [file]`** (default `.github/workflows/ci.yml`): a Ruby (stdlib `yaml`) block, as
  `scripts/validate-frontmatter.sh` uses (it fails with a clear message if `ruby` is absent). Loads
  the file and asserts, each as one counted line:
  1. `jobs.wasm-roundtrip` exists, has no `strategy`, and its `name` is `wasm round-trip tests (Node)`;
  2. `needs` is `changes` (string, or an array containing it);
  3. `if` is exactly `needs.changes.outputs.code == 'true'`;
  4. a step `uses: dtolnay/rust-toolchain@stable` with `targets` containing `wasm32-unknown-unknown`;
  5. a step `uses: actions/setup-node@v6` with `node-version` `'22'` (compare as a string);
  6. a step `uses: taiki-e/install-action@v2` whose `tool` starts `wasm-bindgen@${{ steps.lock.`
     and whose `fallback` is `none`;
  7. a step whose `run` contains `scripts/wasm-ci-check.sh runner-version`, after the install step;
  8. a step `uses: Swatinem/rust-cache@v2` with `key: wasm-roundtrip`;
  9. a step whose `run` is `just wasm-check`;
  10. a step whose `run` contains `just wasm-test ` (with the trailing space or newline, not
      `-slim`) and `scripts/wasm-ci-check.sh count full`;
  11. a step whose `run` contains `just wasm-test-slim` and `scripts/wasm-ci-check.sh count slim`;
  12. steps 7 < 10 and 7 < 11 by index (the assertion precedes the tests);
  13. `jobs.metadata` has steps running `scripts/wasm-ci-check.sh --self-test` and
      `scripts/wasm-ci-check.sh workflow`.
  Prints `PASS`/`FAIL <what>, expected <what>` per assertion and
  `wasm-ci-check workflow: N assertions, F failed`; exit 1 if `F > 0`.
- **`--self-test`**: the 17 rows in the table above, writing fixture logs and lock fragments to a
  `mktemp -d` directory removed on exit (`trap`). Calls the subcommands **as the CI calls them**
  (the script itself, in a subshell, capturing stdout+stderr and the exit code), never an inner
  helper that the CI path does not use (trap 14). Summary line as C3.

### `metadata` job: two new steps (after the existing one)

```yaml
      # The wasm-roundtrip job's shape and its helper's parsers. Here, not in that job: a job
      # whose `needs`/`if` broke is skipped, and could not report its own breakage.
      - name: wasm-ci-check self-test
        run: scripts/wasm-ci-check.sh --self-test

      - name: wasm-roundtrip job shape
        run: scripts/wasm-ci-check.sh workflow
```

`metadata` always runs, needs no other job, and is advisory, so a false positive here turns one
advisory job red and skips nothing. That is why the shape check is not in `changes`, whose failure
skips every heavy job.

### `scripts/ci-code-paths.sh`

- `CODE_PATHS` gains `\.cargo/` (after `rust-toolchain|`), and the comment above it gains one line:
  `.cargo/config.toml holds the wasm32 test runner, so a change to it must run the wasm job.`
- The self-test's `expect` captures the classifier's exit code instead of letting `set -e` end the
  run: `got=$(printf '%s\n' "$path" | classify) || rc=$?`. Three failure kinds, counted
  separately: **crashed** (non-zero exit: `FAIL <path> -> classifier exited <rc>`), **junk**
  (output not `true`/`false`: `FAIL <path> -> printed '<got>', expected true or false`),
  **mismatched** (`FAIL <path> -> <got>, expected <want>`, the existing message). Ends with
  `ci-code-paths self-test: N rows, M mismatched, C crashed, J junk`; exit 1 if any is non-zero.
- New rows: `expect true .cargo/config.toml`, `expect false docs/.cargo/config.toml`,
  `expect true .github/workflows/ci.yml`.
- The `changes` job is unchanged: it already runs `--self-test` and already guards a crash and junk
  output on the real classification (SPEC-142).

### Docs, word for word

- **`ci.yml:100-102`** (the "A wasm32 test leg adds ONE step here" block in `ddphotos-parity`):
  replace the three lines with
  `# The wasm round-trip tests (tests/wasm_roundtrip.rs) run in the next job, wasm-roundtrip.`
- **`justfile:239`** `# Needs: cargo install wasm-bindgen-cli --version <the pinned wasm-bindgen version>`
  becomes two lines:
  `# Needs wasm-bindgen-test-runner at the exact wasm-bindgen version in Cargo.lock`
  `# (scripts/wasm-ci-check.sh lock-version). CI's wasm-roundtrip job runs this and wasm-test-slim.`
- **`justfile:241`** the install hint becomes
  `@command -v wasm-bindgen-test-runner >/dev/null 2>&1 || { echo "wasm-bindgen-test-runner not installed (cargo install wasm-bindgen-cli --locked --version $(scripts/wasm-ci-check.sh lock-version))"; exit 1; }`.
  Build checks it prints `--version 0.2.126` with the runner off `PATH`.
- **`.cargo/config.toml:21`** `` cheaper, less breakable half of that trade. `just wasm-test` wraps the command. ``
  becomes `` … `just wasm-test` wraps the command, and CI's `wasm-roundtrip` job runs it. `` (rewrap
  the comment to its existing width).
- **`AGENTS.md` §6**, in the `# test` group of the code block, one line:
  `just wasm-test && just wasm-test-slim   # wasm round-trip in Node; needs wasm-bindgen-test-runner; not in just check, CI's wasm-roundtrip job runs both`
  This is where the local-gate truth lives: `just check` stays native, and this line says what it
  does not run. STAGE-053's citation of a `CLAUDE.md` list is left for that entry's own spec.
- **`docs/roadmap.md:210`** `(2) **a wasm CI job** — today only a local \`just wasm-test\` floor
  guards the wasm build (mirrors the fuzz-gate CI decision, DEC-062);` becomes
  `(2) **a wasm CI job**: done, CI's \`wasm-roundtrip\` job runs \`just wasm-test\` and
  \`wasm-test-slim\` on every code change (SPEC-146);`
- **`docs/research/proj-008-wasm-build.md:223`**: a research record, so add one line after the
  bullet, do not rewrite it:
  `  *Update (SPEC-146, <build date>): CI's \`wasm-roundtrip\` job now runs \`just wasm-test\` and \`wasm-test-slim\` on every code change.*`

### Mutation table (trap 12)

Each row: where it is applied, and every check expected red. "Live" means a CI run on a throwaway
commit of the draft PR. ⛔ None of these is ever merged. Verify pushes live rows one at a time and
waits for each run to finish (watch with `gh pr checks --watch` in the background, never poll):
`ci.yml`'s `cancel-in-progress` cancels the previous PR run on the next push.

| # | Mutation | Location | Expected red | Expected green |
|---|---|---|---|---|
| M1 | remove `needs: changes` | `ci.yml`, `wasm-roundtrip` | C1 (`needs`), C2 (`property "changes" is not defined`) | live: the job would skip silently, which is why C1 runs in `metadata` |
| M2 | remove the "Runner matches Cargo.lock" step | `ci.yml` | C1 (assertions 7, 12) | live job (nothing else notices; C1 is the guard) |
| M3 | remove `scripts/wasm-ci-check.sh count full "$log"` | `ci.yml`, full step | C1 (assertion 10) | live job |
| M4 | `#[cfg(target_arch = "wasm32")]` → `#[cfg(any())]` | `tests/wasm_roundtrip.rs:30` | C7: full step and slim step, each with the floor's message | `just wasm-test` exit code (0), all native legs |
| M5 | slim step `just wasm-test-slim` → `just wasm-test` | `ci.yml`, slim step | C1 (assertion 11); live: slim step, `slim set: 47 passed, expected 45` | full step |
| M6 | `self.has_alpha` → `!self.has_alpha` | `src/wasm.rs:152` | C9: full step and slim step (test panic) | all native legs, `ddphotos-parity`, `metadata` |
| M7 | `tool:` → `wasm-bindgen@0.2.125` | `ci.yml`, install step | C6 live (runner step); C1 (assertion 6) | — |
| M8 | runner compare made a prefix match (`[[ $lock == $runner* ]]` or similar) | `scripts/wasm-ci-check.sh`, runner compare | C3 row 16 | other C3 rows |
| M9 | remove `fallback: none` | `ci.yml`, install step | C1 (assertion 6) | live job |
| M10 | `node-version: '22'` → `'24'`, or remove the setup-node step | `ci.yml` | C1 (assertion 5) | live job |
| M11 | remove the `if:` | `ci.yml`, `wasm-roundtrip` | C1 (assertion 3) | live job (it would run on docs PRs) |
| M12 | drop `\.cargo/\|` from `CODE_PATHS` | `scripts/ci-code-paths.sh` | C10 (`FAIL .cargo/config.toml -> false, expected true`, `1 mismatched`); live: `changes` job red at "Classify changed paths" | the other C10 rows |
| M13 | classifier crashes: `exit 3` as the first line of `classify()` | `scripts/ci-code-paths.sh` | C10/C11 (`crashed` = row count, others 0); live: `changes` red (SPEC-142's assign-first guard) | — |
| M14 | classifier prints junk: `echo maybe` in place of `echo false` | `scripts/ci-code-paths.sh` | C10/C11 (`junk` = the number of `false` rows; `mismatched` and `crashed` 0) | the `true` rows |
| M15 | `SLIM_COUNT=45` → `47` | `scripts/wasm-ci-check.sh` | C3 rows 4 and 5 | rows 1 to 3 |
| M16 | count the first `test result:` line instead of requiring exactly one | `scripts/wasm-ci-check.sh`, count | C3 row 7 | other rows |
| M17 | `lock-version` greps `name = "wasm-bindgen` without the closing quote | `scripts/wasm-ci-check.sh` | C3 row 13 (`found 4 …` or similar) | rows 10 to 12 |

**Verify drives** at least: M1, M5, M12, M13, M14, M15 locally (each reverted, `git diff` empty);
C4 locally; and **three live runs**: M6 (C9), M4 (C7) and M7 (C6), each pushed alone and reverted
by a commit that restores the exact pre-mutation text (`git diff <pre-mutation SHA> HEAD` empty for
that file). Verify records each run id and quotes each red step's message.

### Decisions that apply

- `DEC-009` — the CI matrix; governs `.github/workflows/**`. This job adds a single-OS advisory job
  (as `ddphotos-parity`, `avif`, `lean` are), and does not change the required matrix.
- `DEC-064` — wasm target: the runner is `wasm-bindgen-test-runner` via `.cargo/config.toml`, and
  `wasm-pack test` is ruled out (it builds every test crate for wasm32).
- `DEC-065` — wasm feature set: CI builds through the `just` recipes, never bare cargo, so the
  feature set cannot drift from the shipped one.
- `DEC-066` — size profile: not touched; the tests use the test profile.

No new DEC: every call here is inside this spec's scope and recorded in it.

### Constraints that apply

- `no-secrets-in-code` — the job uses no secrets (`ci.yml`'s header).

### Prior related work

- `SPEC-072` — created `tests/wasm_roundtrip.rs` and `.cargo/config.toml`'s runner.
- `SPEC-142` — the `changes` job, `scripts/ci-code-paths.sh` and its assign-first guard (trap 15).
- `SPEC-143` — its verify filed the STAGE-054 entry this spec closes.

### Out of scope (for this spec specifically)

- Making `wasm-roundtrip` a required check: branch protection, the maintainer's, after the first
  green run on `main` (see below).
- wasm32 clippy / `-D warnings` on the wasm build (red today on `MANIFEST_DROPPED_WARNING`,
  `src/metadata/mod.rs:129`). Filed on STAGE-053.
- `just check` running the wasm suite; it stays native.
- The wasm bundle-size gate in `just check` (STAGE-053's own entry).
- Running `tests/npm_smoke.mjs` on PRs (publish-time only, SPEC-144 / DEC-111).
- The existing actionlint SC2001 note in the `changes` job.
- Any change to `src/wasm.rs`, `tests/wasm_roundtrip.rs`, or PROJ-017's files: `src/operation/`,
  `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`.
  (M4 and M6 touch the first two on throwaway commits only, at verify.)

### What the maintainer sees

- **PR check list:** one new entry, `CI / wasm round-trip tests (Node)`. On a docs-only PR it shows
  "This check was skipped" and does not block.
- **A red run:** the job name says wasm; the failing step names the feature set
  (`just wasm-test (full feature set)` or `… (slim feature set)`) or the runner; the last
  `::error::` line says why.
- **The day a wasm test is added:** the job goes red with
  `full set: 48 passed, expected 47 (FULL_COUNT in scripts/wasm-ci-check.sh)`; the author edits one
  constant (two, if the test runs in both sets) in the same PR.
- **Required check (ruling 5):** after the first green `main` run, the maintainer adds
  `wasm round-trip tests (Node)` to `main`'s required status checks (Settings → Branches, or
  `gh api`). The spec changes no branch protection.

## Notes for the Implementer

- Use your own `CARGO_TARGET_DIR` for any local `just wasm-*` run; never share one.
- Run `scripts/wasm-ci-check.sh --self-test`, `workflow`, `scripts/ci-code-paths.sh --self-test`
  and `actionlint .github/workflows/ci.yml` before pushing; quote their summary lines in Build
  Completion.
- Do not "fix" the baseline actionlint note at `ci.yml:44`.
- `scripts/` is itself a code path, so the PR runs every heavy job; expect a full CI run.
- Record cold and warm `wasm-roundtrip` job times from the draft PR's first two runs.
- Push a WIP commit early (long cycles must checkpoint).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
