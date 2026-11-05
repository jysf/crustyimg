---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-146
  type: chore                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
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

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- Making `wasm-roundtrip` a required check: a branch-protection setting the maintainer changes
  after the first green `main` run, not part of the PR.
- wasm32 clippy / `-D warnings` on the wasm build (red today on `MANIFEST_DROPPED_WARNING`,
  `src/metadata/mod.rs:129`). Filed on STAGE-053.
- `just check` running the wasm suite. It stays native, and the wasm toolchain stays optional
  for contributors.
- The wasm bundle-size gate in `just check` (STAGE-053's own entry).
- Running `tests/npm_smoke.mjs` on PRs (publish-time only, SPEC-144 / DEC-111).
- Any change to `src/wasm.rs`, `tests/wasm_roundtrip.rs`, or PROJ-017's files: `src/operation/`,
  `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`.

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

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
