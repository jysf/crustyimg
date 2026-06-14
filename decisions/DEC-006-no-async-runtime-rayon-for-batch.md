---
insight:
  id: DEC-006
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-13
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - src/**

tags:
  - concurrency
  - performance
  - dependencies
---

# DEC-006: No async runtime; batch parallelism via `rayon`

## Decision

`crustyimg` is a **synchronous** CLI. There is **no** async runtime
(`tokio`/`async-std`). Batch work parallelizes with **`rayon`** data
parallelism across input files (landed for `apply` in STAGE-005); the
abstractions are sync from STAGE-001.

## Context

The prototype declared `#[tokio::main]` and used async for nothing — pure
overhead and startup cost. Image processing is CPU-bound and embarrassingly
parallel per file; that is exactly `rayon`'s sweet spot, not async I/O.
CLI startup must feel instant (feature-exploration.md § "Technical
considerations" — Async: none).

## Alternatives Considered

- **Option A: `tokio` runtime (prototype's choice)**
  - Why rejected: no async I/O to justify it; adds binary size and startup
    latency; the work is CPU-bound.

- **Option B: Manual `std::thread` pool**
  - Why rejected: `rayon` gives a tuned work-stealing pool, `par_iter`, and
    `--jobs` control for free; rolling our own is needless risk.

- **Option C (chosen): sync core + `rayon` for batch**
  - Why selected: instant startup, simple code, ideal fit for per-file
    parallelism, trivial `--jobs` (`ThreadPoolBuilder`).

## Consequences

- **Positive:** Fast startup, simpler code, no async coloring. Memory is
  bounded by capping concurrency (≈ W×H×4 bytes per decoded image).
- **Negative:** No streaming/overlapped network I/O — irrelevant for a
  local tool. If a future feature needs network fetch (placeholder APIs),
  it'll use blocking I/O or a scoped runtime, not a global async core.
- **Neutral:** `--jobs` in STAGE-001 is a parsed placeholder; honored in
  STAGE-005.

## Validation

Right if: startup is instant and `apply` scales across cores without an
async runtime. Revisit if: a future project needs genuine async network I/O
heavy enough to justify a runtime (scope it locally, don't make the core async).

## References

- Related specs: SPEC-006 (`--jobs` placeholder), STAGE-005 (parallel apply)
- Related decisions: DEC-002 (sync pipeline)
- External docs: https://docs.rs/rayon
