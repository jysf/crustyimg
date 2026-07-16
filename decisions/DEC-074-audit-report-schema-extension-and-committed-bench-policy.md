---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-074
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-16
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/mod.rs"
  - "src/analysis/decide.rs"
  - "scripts/bench.py"
  - "examples/gen_bench_corpus.rs"
  - "bench/**"
  - "justfile"

tags:
  - audit
  - timing
  - explain-json
  - benchmark
  - corpus
  - privacy
  - no-telemetry
---

# DEC-074: audit-report schema extension (`--timing`/`--json`) + committed-bench policy

## Decision

Two coupled choices for SPEC-088:

1. **`--timing` extends the existing `crustyimg.optimize.explain/v1` schema
   additively and gated** — a trailing `"timing":{"decode_ms","encode_ms",
   "total_ms"}` object emitted **only** under `--timing`, exactly as `"ssim"` is
   emitted only under scoring (DEC-071). No version bump, no per-command fork. The
   consistent audit `--json` is made available on `web` and `apply` (terminal
   `optimize` recipes) by routing them through the same `ExplainTrace.write_json`
   the `optimize` auto-decision already used; `optimize`'s `--json` is a synonym for
   `--explain=json`. The audit surface is **auto-decision-only**: on a format-pinned
   (`-o`/`--format`), `--profile preserve`, or plain-pixel-recipe run there is no
   decision to report, so `--json`/`--timing` is a **usage error (exit 2)**, not a
   silently ignored flag.

2. **The committed benchmark is a checked-in Python-3 stdlib script
   (`scripts/bench.py`) over a small, synthetic, CC0 corpus (`bench/corpus/`)** that
   the CLI's own `--json --timing` report feeds — offline, deterministic (for
   savings/scores), and with **no telemetry**. The harness accepts `--corpus <dir>`
   so the maintainer runs the real SPEC-083 numbers **without those photos entering
   git**. `just bench` is the corpus harness; the prior criterion recipe is renamed
   `just bench-micro`.

## Context

STAGE-030's honesty pitch is "we measure, you can check." Two gaps blocked it: the
machine-readable surface was inconsistent (no `--timing`; `--json` only reachable on
`optimize` via `--explain=json`, absent on `web`/`apply`), and the benchmark that
drove the whole taxonomy freeze lived in a since-deleted session scratchpad —
nothing reproducible was committed, yet BENCHMARKS.md (SPEC-083, a launch blocker)
needs exactly that. The audit report **is** what the bench consumes, so the two must
not drift.

## Alternatives Considered

- **Fork a new per-command JSON shape for web/apply**
  - What it is: a bespoke `web.report/v1` distinct from `optimize.explain/v1`.
  - Why rejected: three schemas to keep in sync; the spec explicitly wants ONE
    versioned shape. The auto-decision path is identical across the three verbs, so
    the trace already fits.

- **Bump the schema to `/v2` for the timing field**
  - What it is: version the schema on every additive field.
  - Why rejected: `ssim` (DEC-071) set the precedent that a *gated, additive,
    optional* field keeps `/v1` — existing consumers of a non-`--timing` run see a
    byte-identical object. A version bump would wrongly signal a breaking change.

- **Silently ignore `--json`/`--timing` on the pinned/preserve/plain path**
  - What it is: accept the flag, emit nothing.
  - Why rejected: a flag that silently does nothing is a known footgun in this repo
    (pinned `optimize --verify` already silently ignores its flag). A usage error is
    honest and enforceable.

- **Commit real photos / a large synthetic corpus for the bench**
  - Why rejected: privacy (`no-secrets-in-code` — real EXIF/GPS) and repo bloat
    (detailed/noisy synthetic photos are incompressible → large files). A small
    smooth-synthetic corpus + an external `--corpus` for the real numbers gives both
    a committed smoke and honest launch figures.

- **A new Rust bench binary / a new dev-dependency (criterion-style, or a JSON crate)**
  - Why rejected: `no-new-top-level-deps-without-decision`. The spec allows a
    checked-in script; Python 3 stdlib (`json`, `subprocess`, `argparse`) parses our
    valid single-line JSON with zero installs and stays offline. (Chosen: option C.)

- **Option C (chosen):** additive gated `timing` on `/v1` + shared `write_json`
  across the three verbs, and a stdlib Python harness over a small CC0 synthetic
  corpus with `--corpus` for real numbers.

## Consequences

- **Positive:** one consistent, versioned audit report across `optimize`/`web`/
  `apply`; a skeptic can re-run `just bench` offline; the real-corpus numbers are
  reproducible without committing private photos; non-audit output stays
  byte-identical (regression-anchored).
- **Negative:** `just bench` now depends on a `python3` interpreter (ambient on dev
  machines/CI, but not a Rust toolchain guarantee) and on `--features avif` for the
  release build it benches; the committed smoke corpus shows honest passthrough
  (0%) on its smooth synthetic photos rather than a dramatic win (documented).
- **Neutral:** `just bench` was repurposed from the criterion micro-benches, which
  move to `just bench-micro` (DEC-028's recipe name shifts, not its intent).

## Validation

Right if: verify re-runs `just bench` offline and green, a plain run's output
diffs byte-identical to pre-spec, and the three verbs' `--json` share one key set.
Revisit if: a consumer needs timing on the pinned path (promote from usage-error to
a decode/encode-only report), or if the Python dependency proves a portability
problem (port the harness to a Rust example that shells out).

## References

- Related specs: SPEC-088 (this), SPEC-083 (BENCHMARKS, consumes the harness),
  SPEC-084/085/086 (the taxonomy freeze the audit proves).
- Related decisions: DEC-049 (the `optimize.explain/v1` schema), DEC-071 (the `ssim`
  additive-gated precedent), DEC-069/DEC-070 (the fast decision + terminal-optimize
  recipe the verbs share), DEC-028 (the criterion bench recipe, renamed).
