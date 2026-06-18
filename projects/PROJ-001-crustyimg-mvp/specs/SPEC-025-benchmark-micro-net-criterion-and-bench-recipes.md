---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-025
  type: chore                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L

project:
  id: PROJ-001
  stage: STAGE-009
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build cycle (or orchestrator-direct main loop)
  created_at: 2026-06-18

references:
  decisions: [DEC-028, DEC-009, DEC-019, DEC-016, DEC-008, DEC-002]
  constraints:
    - clippy-fmt-clean
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
  related_specs: [SPEC-016, SPEC-013]

value_link: "Delivers STAGE-009's credibility leg — a cheap criterion regression net for the resize/encode/decode/score/pipeline hot paths, and the equal-quality basis every future size/speed claim must rest on."

cost:
  sessions:
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null      # main-loop build (orchestrator-direct); orchestrator fills at ship
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: "criterion micro-net: benches/pipeline.rs (5 groups) + [[bench]] + criterion dev-dep + just bench/bench-cli; no shipped-code change. Main-loop build — order-of-magnitude estimate recorded at ship."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-025: benchmark micro-net (criterion) + `just bench` / `just bench-cli`

## Context

STAGE-009 made the differentiator legible (`optimize`/`diff`/`responsive`); its last
leg is **credibility** — being able to make honest, reproducible performance claims
and catch regressions. The roadmap's benchmarking plan (steps 1–2) is the cheap
foundation: **criterion micro-benches** over the hot paths (decode / resize / encode
/ perceptual score / full pipeline), plus a **hyperfine** CLI wall-clock recipe.
This is explicitly a "do it early, even as a chore" item — a regression net now, the
basis for cross-tool and quality-per-byte comparisons later.

This spec is **infrastructure**: it adds a dev-only `criterion` dependency and a
`benches/` harness; it changes **no shipped code**, no library public API, and the
default binary is unaffected (benches are a dev/CI concern). The strategic
benchmarking decisions (criterion, what to measure, and the equal-quality principle
for any future comparison) are recorded in **DEC-028**.

## Goal

Add a `criterion` micro-benchmark harness covering crustyimg's hot paths
(decode/resize/encode/score/pipeline) runnable via `just bench`, plus a `just
bench-cli` recipe that wall-clock-times the release binary with `hyperfine` — with no
change to the shipped binary and `cargo deny` staying green.

## Inputs

- **Files to read:** `src/lib.rs` (the public surface benches call), `src/image/mod.rs`
  (`Image::from_bytes`, `pixels`), `src/operation/mod.rs` + `src/operation/registry.rs`
  (`OperationRegistry::with_builtins().build`, `OperationParams::from_map`),
  `src/pipeline/mod.rs` (`Pipeline::new/push/run`), `src/quality/mod.rs` (`score`),
  `src/sink/mod.rs` (`encode_to_bytes`). `justfile` (recipe style; `test`/`bench`
  conventions). `Cargo.toml` (`[dev-dependencies]`, `[[bench]]`).
- **Decisions:** DEC-028 (this spec's benchmarking DEC — author during build),
  DEC-009 (testing/CI), DEC-019 (the SSIMULACRA2 metric, benched + the equal-quality
  basis), DEC-016 (encode quality), DEC-008 (resize backend), DEC-002 (decode-once).
- **Note:** normal deps (`image`, `toml`) are available to bench targets, so the
  bench can build `OperationParams` and generate fixtures without new dev-deps beyond
  `criterion`.

## Outputs

- **Files created:** `benches/pipeline.rs` — a criterion bench with groups
  `decode`, `resize`, `encode_jpeg`, `score`, `pipeline`, over a generated in-memory
  fixture.
- **Files modified:**
  - `Cargo.toml` — add `criterion` to `[dev-dependencies]` (pinned, exact) + a
    `[[bench]] name = "pipeline", harness = false` entry.
  - `justfile` — `bench` (`cargo bench`) and `bench-cli` (hyperfine wrapper) recipes.
  - `decisions/DEC-028-benchmarking-criterion-and-equal-quality.md` — the new decision.
- **New exports:** none (no library API change).
- **No shipped-binary impact** (dev-dependency + bench target only).

## Acceptance Criteria

This is **infrastructure**; verification is compile + run + gates, not unit tests
(the bench harness adds no public library functions, so `every-public-fn-tested`
does not apply).

- [ ] `cargo bench --no-run` compiles the `pipeline` bench target (all 5 groups).
- [ ] `just bench` runs criterion and reports timings for `decode`, `resize`,
  `encode_jpeg`, `score`, and `pipeline`.
- [ ] `criterion` is a **pinned** `[dev-dependencies]` entry; `[[bench]]` uses
  `harness = false`.
- [ ] `cargo build` / `cargo test` / `cargo clippy --all-targets -- -D warnings` /
  `cargo fmt --check` stay green (the default + lean builds are unaffected — the
  bench is dev-only).
- [ ] `cargo deny check licenses` stays green (criterion's tree is permissive); if it
  trips a transitive license, record a scoped exception or a watchlist entry per DEC-018.
- [ ] `just bench-cli` runs `hyperfine` against the release binary if installed, and
  prints a clear, non-failing message (skip) when `hyperfine` is absent.
- [ ] The bench fixture is generated in-memory (no committed binary files, mirroring
  the test-fixture rule, DEC-009).

## Failing Tests

This spec is **infrastructure** and intentionally has **no `## Failing Tests`** in
the TDD sense — benchmarks are not behavioral unit tests, and the harness adds no
public library function to test. The executable verification is the Acceptance
Criteria above:

- `cargo bench --no-run` (compiles all five groups), and
- `just bench` (runs and emits timings for each group).

Treat those two commands as the build cycle's "make it pass" target. Do **not**
fabricate unit tests for the bench harness.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- **DEC-028 (NEW — author it)** — adopt `criterion` for micro-benches; the bench
  target set (decode/resize/encode/score/pipeline); `harness = false`; dev-dependency
  only (no shipped-binary/default-build impact); `hyperfine` as an external (not
  vendored) CLI wall-clock tool via `just bench-cli`; and the **principle that any
  future size/speed claim must be gated on equal quality (SSIMULACRA2)** — even
  though cross-tool comparison + quality-per-byte tables + `BENCHMARKS.md` + CI bench
  tracking are deferred to later specs. `affected_scope`: `Cargo.toml`, `benches/**`,
  `justfile`; confidence ~0.8.
- **DEC-009** — testing/CI + native (no shell-out) fixtures: generate the bench image
  in-memory, same as tests.
- **DEC-019 / DEC-016 / DEC-008 / DEC-002** — the real hot paths being measured
  (perceptual score, JPEG quality encode, resize backend, decode-once).

### Constraints that apply

- `no-new-top-level-deps-without-decision` — `criterion` is a new dev-dependency →
  DEC-028 covers it. Run `cargo deny check licenses` after adding.
- `clippy-fmt-clean` — bench code is a target clippy checks under `--all-targets`;
  keep it warning-clean and `cargo fmt` it (then `git add -u`).
- `no-unwrap-on-recoverable-paths` — bench setup may `.expect()` on fixture
  generation (test/bench-only code, like `#[cfg(test)]`); keep it to setup, not a
  library path.

### Out of scope (for this spec specifically)

- **CI bench tracking** (github-action-benchmark / CodSpeed) — deferred; no CI bench
  job in this spec (keeps CI fast). A later spec adds non-blocking trend tracking.
- **Cross-tool comparison** (`bench-compare` vs ImageMagick/vips/sharp/…) and
  **quality-per-byte** tables — later roadmap steps; this spec is only the local
  micro-net + the CLI wall-clock recipe.
- **`BENCHMARKS.md`** publication — lands once cross-tool + quality-per-byte exist.
- **iai-callgrind / instruction-count benches** — skipped (no Apple-Silicon Valgrind;
  blind to the SIMD hot path), per the roadmap.

## Notes for the Implementer

- **Fixture:** generate a detailed RGB `DynamicImage` in-memory in `benches/pipeline.rs`
  (a smooth gradient + a mild checker, like `src/quality`'s `detailed_rgb` test
  helper — copy it into the bench, don't try to reach the `#[cfg(test)]` one).
  Pre-encode it to JPEG bytes once for the `decode` group. A ~256×256 (or 512×512)
  image keeps benches meaningful but fast.
- **Groups (one `criterion_group!`):**
  - `decode` — `crustyimg::image::Image::from_bytes(&jpeg_bytes)`.
  - `resize` — build the `resize` op via `OperationRegistry::with_builtins().build(
    "resize", &OperationParams::from_map(...))` (mode `max`/`fit`), run a
    `Pipeline::new().push(op)` over the decoded image.
  - `encode_jpeg` — `crustyimg::sink::encode_to_bytes(&img, image::ImageFormat::Jpeg,
    Some(80))`.
  - `score` — `crustyimg::quality::score(a.pixels(), b.pixels())` (reference vs a
    re-encoded copy).
  - `pipeline` — the full shrink-shaped chain: `from_bytes` → resize → `encode_to_bytes`.
  - Wrap inputs in `criterion::black_box`.
- **Cargo.toml:** `criterion = "=<latest>"` in `[dev-dependencies]` (pin the exact
  latest, e.g. `0.5.x`/`0.7.x` — record it). Add:
  ```toml
  [[bench]]
  name = "pipeline"
  harness = false
  ```
  Consider `criterion = { version = "=X.Y.Z", default-features = false }` if the
  default features drag a heavy/again-licensed tree — but only if `just deny` or build
  time demands it; otherwise defaults are fine.
- **justfile:**
  ```
  # Run the criterion micro-benchmarks (SPEC-025, DEC-028).
  bench:
      cargo bench

  # Wall-clock the release binary with hyperfine (skips cleanly if not installed).
  bench-cli *ARGS:
      @command -v hyperfine >/dev/null || { echo "hyperfine not installed; skipping (brew install hyperfine)"; exit 0; }
      cargo build --release
      hyperfine --warmup 2 './target/release/crustyimg {{ARGS}}'
  ```
- **Confirm** `cargo bench --no-run` compiles and `just bench` runs all five groups
  before claiming done. **Run `cargo deny check licenses`** after adding criterion.
- **Cost:** append a build session to `cost.sessions` (real `tokens_total`, or a
  labeled estimate if main-loop), per AGENTS.md §4.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-025-bench`
- **PR (if applicable):** (opened during build; number recorded in the timeline)
- **All acceptance criteria met?** yes — `cargo bench --no-run` compiles the
  `pipeline` target; `just bench` runs and reports all five groups (measured:
  decode ~128µs, resize ~163µs, encode_jpeg ~340µs, score ~9.8ms, pipeline ~399µs);
  `criterion =0.8.2` pinned dev-dep with `[[bench]] harness = false`; default +
  `--no-default-features` builds/tests, `clippy --all-targets`, `fmt --check`, and
  `cargo deny check licenses` all stay green (no exception needed for criterion).
- **New decisions emitted:**
  - `DEC-028` — criterion micro-benches + equal-quality benchmarking principle
    *(authored during design, on `main`)*
- **Deviations from spec:** none. Infrastructure only — no shipped-code change, no
  library API change. Used five `bench_function` calls (named decode/resize/
  encode_jpeg/score/pipeline) in one `criterion_group!` rather than nested groups;
  `std::hint::black_box` for the loop barriers.
- **Follow-up work identified:**
  - CI bench trend tracking (github-action-benchmark / CodSpeed) — deferred (DEC-028).
  - Cross-tool `bench-compare` + quality-per-byte tables + `BENCHMARKS.md` — later
    roadmap steps (DEC-028); the `score` bench already shows SSIMULACRA2 (~9.8ms) is
    the dominant cost, which is the equal-quality basis those will build on.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing. The spec pinned the groups, the Cargo.toml `[[bench]]` block, the
   justfile recipes, and the public APIs; the note that normal deps (`image`/`toml`)
   are available to bench targets removed the one real uncertainty.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-028 covered the dep + methodology. Confirmed the useful fact for future
   bench/test work: a bench is a separate crate that can use the crate's **normal**
   deps (image, toml) plus dev-deps — no need to duplicate them.
3. **If you did this task again, what would you do differently?**
   — Nothing of substance. `cargo bench` runs the full ~40s suite; if iteration speed
   matters later, a `just bench` could pass a shorter measurement time, but the full
   run is the honest default.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
