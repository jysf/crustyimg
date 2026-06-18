# SPEC-025 build prompt — benchmark micro-net (criterion) + bench recipes

Start a **fresh session**. You are the IMPLEMENTER for SPEC-025 in the `crustyimg`
repo. The architect (Opus) wrote the spec + DEC-028. This is an **infrastructure**
chore — there are **no failing unit tests**; the build target is "`cargo bench
--no-run` compiles and `just bench` runs all five groups."

## Read first
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-025-benchmark-micro-net-criterion-and-bench-recipes.md`
   — especially `## Outputs`, `## Acceptance Criteria`, and `## Notes for the
   Implementer` (it gives the groups, the Cargo.toml `[[bench]]` block, and the
   justfile recipes).
2. `decisions/DEC-028-benchmarking-criterion-and-equal-quality.md`.
3. The public APIs the bench calls: `crustyimg::image::Image::from_bytes`,
   `crustyimg::operation::{OperationRegistry, OperationParams}`,
   `crustyimg::pipeline::Pipeline`, `crustyimg::quality::score`,
   `crustyimg::sink::encode_to_bytes`. (Normal deps `image` + `toml` are available to
   the bench target — no extra dev-deps beyond criterion.)

## What to build
- `Cargo.toml`: add `criterion = "=<pin exact latest>"` to `[dev-dependencies]` and:
  ```toml
  [[bench]]
  name = "pipeline"
  harness = false
  ```
- `benches/pipeline.rs`: a criterion bench with one `criterion_group!` containing
  five groups — `decode`, `resize`, `encode_jpeg`, `score`, `pipeline` — over an
  in-memory generated detailed RGB fixture (copy a `detailed_rgb`-style generator into
  the bench; do NOT reach the `#[cfg(test)]` helper). Wrap inputs in `black_box`.
- `justfile`: `bench` (`cargo bench`) and `bench-cli` (hyperfine wrapper that skips
  cleanly if hyperfine is absent) — see the spec's Notes for the exact recipes.
- `decisions/DEC-028…` already exists (authored at design) — reference it.

## Gates (all must pass)
```
cargo bench --no-run                 # compiles all 5 groups (the primary target)
just bench                           # runs and emits timings for each group
cargo build && cargo test            # default build/tests unaffected
cargo build --no-default-features    # lean build unaffected
cargo clippy --all-targets -- -D warnings   # benches are an --all-targets target
cargo fmt                            # then `git add -u`
cargo deny check licenses            # criterion's tree must stay permissive
```
If `cargo deny` trips a criterion transitive license, add a scoped exception in
`deny.toml` (note it) or a `guidance/license-watchlist.yaml` entry — and flag it.

## Git / PR
- Branch `feat/spec-025-bench` (or `chore/…`) off current `main`. One spec, one PR.
- Verify `git branch --show-current` before each commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `chore(bench): criterion micro-net + bench recipes (SPEC-025)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-028, DEC-009, DEC-019 /
  Constraints / New decisions — DEC-028).
- Fill the spec's `## Build Completion` + the 3 reflection answers. Do NOT fabricate
  unit tests for the harness.

## Cost
Append a build session to `cost.sessions` (numerics null; orchestrator fills at ship):
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "criterion micro-net: benches/pipeline.rs (5 groups) + [[bench]] + just bench/bench-cli + criterion dev-dep; no shipped-code change"
```

## When done
`just advance-cycle SPEC-025 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
