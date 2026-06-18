---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-028
  type: decision
  confidence: 0.80
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

created_at: 2026-06-18
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - benches/**
  - justfile

tags:
  - benchmarking
  - criterion
  - performance
  - dev-tooling
  - methodology
---

# DEC-028: criterion for micro-benchmarks; every perf claim gated on equal quality

## Decision

Adopt **`criterion`** (a dev-dependency, `harness = false`) for micro-benchmarks
over crustyimg's hot paths — decode, resize, encode, perceptual score, full
pipeline — runnable via `just bench`, plus a `just bench-cli` recipe that wall-clocks
the release binary with **`hyperfine`** (an external CLI, not a vendored dep; skips
cleanly if absent). The benches live in `benches/` over **in-memory generated
fixtures** (no committed binaries, DEC-009). Establishing principle: **any size or
speed claim crustyimg makes must be gated on equal quality (SSIMULACRA2, DEC-019)** —
a smaller/faster result is only meaningful at equal perceptual quality. Cross-tool
comparison, quality-per-byte tables, `BENCHMARKS.md`, and CI bench tracking are
**deferred** to later specs; this is the local regression net + the methodology.

## Context

STAGE-009's credibility leg needs (a) a cheap regression net for the SIMD/codec hot
paths and (b) a disciplined way to make honest performance claims. The roadmap's
benchmarking plan stages this: micro-benches + CLI wall-clock first (cheap, now),
cross-tool + quality-per-byte + publication later. The decisions to settle now:
which micro-bench framework, what to measure, whether it touches the shipped binary,
and the standard for future comparisons.

Constraints: `no-new-top-level-deps-without-decision` (criterion is a new dep — hence
this DEC), `clippy-fmt-clean` (benches are `--all-targets`), DEC-018 (license gate —
criterion's tree must stay permissive), DEC-009 (native in-memory fixtures, no
shell-out), keep the default build/CI fast (so: no CI bench job yet).

## Alternatives Considered

- **Option A: no benchmark harness yet (defer all of it).**
  - Why rejected: the hot paths (resize SIMD, JPEG encode, SSIMULACRA2) are exactly
    where a silent regression would hide; a cheap criterion net now is high-leverage
    and the roadmap explicitly front-loads it.

- **Option B: `iai-callgrind` / instruction-count benches.**
  - Why rejected: Valgrind has no Apple-Silicon support (the dev machine is darwin),
    and instruction counts are blind to the SIMD hot path that matters here. Skipped
    per the roadmap.

- **Option C: hand-rolled `std::time::Instant` timing loops.**
  - Why rejected: criterion gives statistically sound measurement (warmup, outlier
    handling, regression deltas) for free; reinventing it is lower quality.

- **Option D (chosen): `criterion` micro-benches (dev-dep, harness=false) + a
  `hyperfine` `bench-cli` recipe; equal-quality principle for claims.**
  - Why selected: the de-facto Rust micro-bench framework, dev-only (zero shipped-
    binary/default-build impact), measures the real library hot paths, and pairs with
    hyperfine for end-to-end CLI timing without vendoring a tool. The equal-quality
    rule keeps any future "N% smaller/faster" claim honest from day one.

## Consequences

- **Positive:** a fast local regression net (`just bench`) over decode/resize/encode/
  score/pipeline; end-to-end CLI timing (`just bench-cli`); a written standard that
  forbids quality-blind size/speed claims; no impact on the shipped binary (criterion
  is dev-only) or the default-build/CI speed (no bench job added).
- **Negative:** one more dev-dependency tree (criterion pulls plotters/rayon/etc., all
  permissive); two benchmarking tools to keep in mind (criterion in-repo, hyperfine
  external). `hyperfine` must be installed for `bench-cli` (the recipe degrades
  gracefully).
- **Neutral:** the heavier benchmarking work (cross-tool, quality-per-byte,
  `BENCHMARKS.md`, CI trend tracking) is explicitly later — this DEC only commits to
  the micro-net + the principle.

## Validation

- Right if: `cargo bench` reliably surfaces a regression in the hot paths during
  development, and every published perf claim can point at an equal-quality
  measurement. Revisit when: a CI bench-tracking spec lands (choose
  github-action-benchmark vs CodSpeed), or the cross-tool/quality-per-byte specs need
  to extend the harness, or criterion's tree ever trips the license gate (then scope
  an exception or pin differently).

## References

- Related specs: SPEC-025 (this harness), SPEC-016 (the SSIMULACRA2 metric being
  benched + the equal-quality basis), SPEC-013 (`shrink`, the pipeline shape benched)
- Related decisions: DEC-009 (testing/CI + native fixtures), DEC-019 (SSIMULACRA2),
  DEC-016 (encode quality), DEC-008 (resize backend), DEC-018 (license gate),
  DEC-002 (decode-once)
- External docs: https://docs.rs/criterion , https://github.com/sharkdp/hyperfine
- Roadmap: 2026-06-16 handoff §Benchmarking (staged plan; this is steps 1–2)
