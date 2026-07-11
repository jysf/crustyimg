---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-062
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-11
supersedes: null
superseded_by: null

affected_scope:
  - fuzz/**
  - tests/fuzz_regressions.rs
  - tests/fixtures/fuzz/**
  - justfile
  - src/image/avif.rs
  - docs/research/proj-009-fuzz-run.md

tags:
  - fuzzing
  - untrusted-input-hardening
  - decoders
  - pre-1.0-gate
---

# DEC-062: decoder fuzz-gate policy — run mechanism, budget, triage, durability, CI

## Decision

The decoder fuzz gate (`fuzz/` targets `avif_decode`, `svg_decode`,
`raw_preview`, `heic_decode`) is run **manually/periodically** on a nightly
`cargo-fuzz` toolchain in **`-O`/release config** (matching the shipped binary),
to a floor of **`-max_total_time=600` per target**, via **`just fuzz <target>
[seconds]`**; every crash/OOM/panic/hang is triaged into (a) our-bug-fix,
(b) tightened DEC-034/035 cap, or (c) upstream-guarded/documented, and **every
finding is made catchable without the fuzzer** by a deterministic regression in
`tests/fuzz_regressions.rs` plus the always-on `fuzz_corpus_never_panics` smoke —
which, **not** a per-PR fuzz job, is the required per-PR protection. A clean run
to budget is a first-class, sufficient outcome.

## Context

PROJ-009 shipped default pure-Rust decode of AVIF/SVG/RAW + opt-in HEIC, each with
a `fuzz/` target that was **never run** (no nightly/cargo-fuzz in the build/verify
envs). The roadmap names running them a pre-1.0 gate; SPEC-068's threat model
ranked it #1 (the one untrusted-binary surface a read-only review can't close).
SPEC-069 runs the gate and — the point of the spec — makes both the findings and
the gate itself repeatable and catchable by ordinary CI, so "fuzzed once" becomes
"stays fuzzed." Constraints: `untrusted-input-hardening`, `no-new-top-level-deps`
(fuzz tooling stays in the detached `fuzz/` workspace + dev toolchain),
`no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, repo-root build/CI untouched.

## Alternatives Considered

- **Option A: a required per-PR `cargo fuzz` CI job.**
  - What it is: gate every PR on a short fuzz run.
  - Why rejected: fuzzing is time-boxed and non-deterministic — a poor fast gate
    (flaky, slow, needs nightly + ASAN). The durable per-PR floor is the
    regression tests + corpus smoke (deterministic, fast, 3-OS). Fuzzing finds
    *new* crashes; it is not a pass/fail PR check.

- **Option B: fuzz in the default (debug-assertions-on) config.**
  - What it is: cargo-fuzz's default (`--release` + debug-assertions + overflow
    checks).
  - Why rejected as the *primary* config: it fires upstream `debug_assert!`s that
    are **compiled out of the shipped release binary** (e.g. `avif-parse`'s
    `check_parser_state`), and libFuzzer's panic hook aborts on any panic
    regardless of our `catch_unwind`, so the run can't reach a clean budget on a
    non-bug. It does not represent what users run. (Kept as an occasional extra —
    it can catch overflow in *our* code.)

- **Option C (chosen): manual/periodic `-O` runs + `just fuzz` + regression
  conversion + optional non-blocking CI.**
  - What it is: run in `-O` (release, debug-assertions off = production) via
    `just fuzz`, convert every finding to a normal-suite regression + committed
    minimized fixture, keep an always-on corpus smoke, and (optionally) a
    non-blocking `workflow_dispatch`/scheduled nightly CI job for visibility.
  - Why selected: fuzzes the code that actually ships; keeps PRs protected by a
    deterministic, fast, cross-OS guard; makes the gate one command; leaves the
    slow/nondeterministic part off the PR path.

## Consequences

- **Positive:** The untrusted decode surface is fuzzed in the shipped config;
  findings are pinned by deterministic tests that 3-OS CI runs every PR without a
  fuzzer; the gate is one command (`just fuzz`); no new default dependency
  (repo-root `Cargo.toml`/`Cargo.lock`/`deny.toml` untouched — the harness lives
  in the detached `fuzz/` workspace + dev toolchain).
- **Negative:** New crashes are only found when someone runs the gate (no
  automatic per-PR fuzzing). Upstream-owned issues we can't fix at the boundary
  (see below) remain until the dependency is patched or replaced.
- **Neutral:** `-O` config trades away debug-assertion/overflow checks in *our*
  code for production fidelity; ASAN (memory safety — the real threat) is on in
  both configs.

### Triage rules (each finding → exactly one)

1. **Our bug** (panic/`unwrap`/index in `src/image/*` or a caller) → fix at the
   boundary + regression.
2. **Missing/loose cap** (OOM/hang within our code) → tighten DEC-034/DEC-035;
   the cap *is* the fix, the crashing input *is* the test.
3. **Upstream library crash** (`avif-parse`/`resvg`/`image`/`libheif`) → guard at
   our call site if we can (`catch_unwind` for a panic; a structural pre-check or
   a cap for an over-allocation), report upstream, and pin a regression asserting
   *our* contract (typed error, no panic). If genuinely unfixable-by-us without
   vendoring/forking (out of scope) → **document + report upstream** and, when
   safe, add the reproducer to the corpus; never silently drop it. **Do not** add
   an input that provokes a multi-GB allocation to the always-on smoke (it could
   OOM-kill CI) — record it in the run record instead.

### Durability rule (the point of the gate)

Every crash becomes a **normal-suite** regression: the **minimized** crashing
bytes committed under `tests/fixtures/fuzz/<target>/`, fed to the real entry point
(`Image::from_bytes` for AVIF/SVG/HEIC, `raw_preview` for RAW) → assert a typed
`Err` / no panic, each failing before its fix and passing after. The always-on
`fuzz_corpus_never_panics` smoke sweeps every seed fixture + committed reproducer
through the matching entry point every `cargo test`. This is what keeps the
surface fuzzed after the fuzzer stops.

### CI decision

The **required** per-PR protection is `cargo test` (regressions + smoke), already
on all 3 CI OSes — **no** per-PR fuzz job. An **optional, non-blocking**
`workflow_dispatch` (+ optional weekly `schedule`) job on the nightly toolchain
running a short `-O` smoke budget per target is sanctioned for visibility; if it
proves flaky or costly it can be dropped without weakening the floor. (Not wired
up in this build — recorded as the sanctioned shape.)

### HEIC best-effort scope (DEC-052 lineage)

HEIC decodes via C `libheif` behind the off-by-default `heic` feature. Fuzz it
when the env builds it (system libheif present); its memory safety is libheif's,
so findings are "guard at our Rust boundary / report upstream," not necessarily a
crustyimg fix. Our contract is "no panic on the Rust side + the default build
answers `.heic` with `CodecNotBuilt`→exit 4." The three default pure-Rust targets
are the required bar; HEIC and an ASAN HEIC run are best-effort. Note: pinned at
the `v1_17` API floor, libheif's own `set_security_limits` is unreachable, so our
DEC-034 handle-dimension pre-check is the only pre-decode bound (roadmap follow-up
to bump to `v1_19` and wire `heif_context_set_security_limits`).

### The 1.0-gate contract

Before cutting 1.0, each shipped decoder with a `fuzz/` target must have been run
to at least the budget floor in `-O` config, with every finding either fixed or
documented-upstream + pinned by a regression, and the run recorded (append to
`docs/research/proj-009-fuzz-run.md`). OSS-Fuzz onboarding and a
`cargo fuzz` target for the non-decode parsers (manifest/lock/cache — already
driven with hostile files by SPEC-068) are noted future options, out of scope
here.

## Validation

Right if: (1) `cargo test` on 3 OSes catches a re-introduced decoder crash via the
regressions/smoke without a fuzzer; (2) `just fuzz <target>` reproduces a run from
a clean checkout; (3) the repo-root `Cargo.toml`/`Cargo.lock`/`deny.toml` stay
untouched by the gate. Revisit if: a dependency's upstream over-allocation/panic
class becomes fixable (patch/replace the dep), OSS-Fuzz is adopted, or a per-PR
fuzz job becomes cheap/deterministic enough to add.

## References

- Related specs: SPEC-069 (this gate), SPEC-058/060/061/062 (the decoders),
  SPEC-068 (threat model that ranked this #1), SPEC-064 (mutation-checked caps
  precedent).
- Related decisions: DEC-034 (decode caps), DEC-035 (resource guards),
  DEC-052 (HEIC feature-gate), DEC-053/054/055/056 (AVIF/SVG/RAW/HEIC decode).
- Run record: `docs/research/proj-009-fuzz-run.md`.
- Harness: `fuzz/` (detached workspace), `tests/fuzz_regressions.rs`,
  `just fuzz`.
