---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-088
  type: story
  cycle: build
  blocked: false
  priority: high
  complexity: M
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-15
references:
  decisions: [DEC-049, DEC-069, DEC-071]
  constraints: [pure-rust-codecs-default, ergonomic-defaults, every-public-fn-tested,
                test-before-implementation, no-new-top-level-deps-without-decision, no-secrets-in-code]
  related_specs: [SPEC-083, SPEC-084, SPEC-085, SPEC-086]
value_link: >
  Makes crustyimg's results measurable and reproducible: a consistent machine-readable report
  (`--json`) + `--timing` across the audit-relevant verbs, and a COMMITTED benchmark harness + a small
  license-clean corpus. This is what BENCHMARKS.md (SPEC-083) — a launch blocker — stands on, and what
  lets a user (or CI) audit "how much smaller, how fast, at what quality" without trusting our word.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop (un-metered, §4).
    - cycle: build
      interface: claude-code
      tokens_total: 470000
      estimated_usd: 4.90
      recorded_at: 2026-07-16
      note: >
        ~50 min, own worktree (spec-088-audit-bench). Main-loop ESTIMATE (no metered subagent, §4 +
        [[autonomous-run-cost-estimates]]): order-of-magnitude tokens at Opus 4.8 list rate, ~80/20.
        Added a gated `Timing` field to the `optimize.explain/v1` schema (additive, byte-identical
        without `--timing`), `--json`/`--timing` on optimize/web/apply routed through the shared
        `write_json`, an auto-decision-only usage-error guard, decode/encode/total `Instant` timing;
        committed `scripts/bench.py` (stdlib, offline, no telemetry) + a 40 KB CC0 synthetic corpus
        (`bench/corpus/` + generator `examples/gen_bench_corpus.rs` + provenance README) + `just bench`
        (criterion → `bench-micro`). 5 spec Failing Tests + 3 decide.rs unit tests. Gates green
        (731 default / 744 avif; clippy; fmt; lean build; validate; bench). Emitted DEC-074.
  totals:
    tokens_total: 470000
    estimated_usd: 4.90
    session_count: 1
---

# SPEC-088: unified audit report (`--json`/`--timing`) + committed bench

## Context

STAGE-030 wants crustyimg's output to be **auditable** — the honesty story is "we measure, you can
check." Today the machine-readable surface is inconsistent: `info --json`, `diff --json`, `optimize
--explain=json` (the `optimize.explain/v1` schema, DEC-049), and `lint`'s own format each differ, and
there is **no `--timing`**. And the benchmark that drove the whole taxonomy freeze lived in a session
scratchpad (`scratchpad/bench/`, now gone) — **nothing reproducible is committed**, yet BENCHMARKS.md
(SPEC-083, a launch blocker) needs exactly that.

This spec (a) gives the size/format/score-reporting verbs a **consistent `--json`** and a **`--timing`**
readout, and (b) commits a **reproducible benchmark harness + a small license-clean corpus** (seeded
from the `scratchpad/bench/` approach; no telemetry, no phone-home).

## Goal

Two deliverables: **(1)** a consistent machine-readable **audit report** — `--json` + `--timing`
(decode / encode / total per image) across the audit-relevant verbs (`optimize`, `web`, `apply`, and
`lint`), reusing the `optimize.explain/v1` schema shape where it fits; **(2)** a **committed bench**:
a harness (a `just bench` recipe + a script) over a **small committed corpus** (license-clean /
synthetic, spanning photo/graphic × a few sizes) that measures savings + time + score for `web` vs
`optimize`, reproducibly and offline — the raw material for SPEC-083.

## Inputs — files to read

- `src/cli/mod.rs` — the existing `--json` sites (`info` ~759, `diff` ~765, `optimize --explain=json`),
  `ExplainFmt`, `run_optimize`/`run_web`/`run_apply`/`run_lint`, the batch fan-out.
- `src/analysis/decide.rs` — `ExplainTrace`/`write_json` (the `optimize.explain/v1` schema, DEC-049) to
  extend consistently, not fork.
- `benches/pipeline.rs` — the existing criterion micro-bench (different layer; the new bench is a
  CLI-level end-to-end harness, not a criterion bench).
- The `scratchpad/bench/` approach (the strategy's `run/run2/bench3/sweep.py`) as the model to commit a
  clean version of; the corpus lives at `~/PSeven/experiments/crustimg_redo_plus/_incoming0` (the
  maintainer's real photos — do NOT commit those; ship a small clean corpus instead).

## Outputs

- **`src/cli/mod.rs`** — a `--timing` flag on `optimize`/`web`/`apply` reporting decode/encode/total
  (human to stderr; folded into `--json`); make `--json` consistent across `optimize`/`web`/`apply`
  (a shared report shape — extend the `optimize.explain/v1` schema additively, versioned, rather than a
  new per-command shape) and `lint`. Keep stdout pipe-clean; no behavior change to the images produced.
- **A committed corpus** — a small `bench/corpus/` (or `assets/bench/`) of **license-clean** images
  (synthetic generators and/or CC0), spanning **photo vs graphic × small/large**, with a README noting
  provenance/license (`no-secrets-in-code` — no real EXIF/GPS from private photos).
- **A committed harness** — `just bench` + a script (Rust bin or a checked-in shell/python, no new
  runtime dep) that runs `web`/`optimize` over the corpus, emits a savings/time/score table + `--json`,
  is **deterministic + offline + no telemetry**, and can also point at an external corpus dir (so the
  maintainer can re-run the real-corpus numbers for SPEC-083).
- **DEC** — records the audit-report unification (the schema-extension decision) + the committed-bench
  design (corpus policy: clean/committed vs external real corpus; no telemetry). Or fold into DEC-069's
  follow-through if small.

## Acceptance Criteria

- [ ] `optimize`/`web`/`apply` accept **`--timing`** and report decode/encode/total per image (human +
      inside `--json`); stdout stays pipe-clean.
- [ ] `--json` is **consistent** across `optimize`/`web`/`apply` (a shared, versioned report shape — the
      `optimize.explain/v1` schema extended additively, not forked); a non-`--json`/non-`--timing` run's
      output is **unchanged** (byte-identical) from before this spec.
- [ ] `lint`'s machine-readable output is reconciled into the same audit story (at minimum documented as
      the audit surface; `--json` where it fits).
- [ ] **`just bench` runs offline over a committed corpus** and prints a savings/time/score table for
      `web` vs `optimize` — deterministic, **no network, no telemetry**; a `--json` mode emits the raw
      numbers.
- [ ] The committed corpus is **license-clean** (synthetic/CC0, provenance documented) and small (repo
      stays lean); the harness also accepts an **external corpus dir** for the real SPEC-083 numbers.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
      `cargo build --no-default-features` pass; `just bench` is green in a smoke form.

## Failing Tests (written at design)

- **`src/cli` / integration**
  - `timing_flag_reports_and_json_includes_it` — `--timing` yields decode/encode/total, and `--json`
    carries them; the numbers are plausible (total ≥ encode).
  - `json_shape_consistent_across_verbs` — `optimize`/`web`/`apply` `--json` share the versioned schema
    (same top-level keys); assert against a golden.
  - `non_json_output_unchanged` — a plain run's stdout/stderr is byte-identical to pre-spec (regression
    anchor, like the SPEC-086 `--verify` byte-identity check).
- **Bench harness**
  - `bench_runs_offline_on_committed_corpus` — `just bench` (or the script) produces a table for the
    committed corpus with **zero network** and no telemetry; the `--json` mode parses.
  - `bench_corpus_is_license_clean` — a check/assertion that the corpus has documented provenance (no
    private-photo EXIF).

## Implementation Context

### Decisions that apply
- `DEC-049` — the `optimize.explain/v1` JSON schema; **extend it additively + versioned**, don't fork a
  new per-command shape. `DEC-069`/`DEC-071` — the score/`--verify` fields already ride this schema;
  `--timing` joins them the same way (gated, non-default output unchanged).
- The honesty guardrails (STAGE-030): the audit report + committed bench are the *proof* behind the
  pitch — they must be reproducible by a skeptic, offline.

### Constraints
- `no-new-top-level-deps-without-decision` — the harness uses the existing toolchain (a Rust bin, or a
  checked-in script) — no new runtime/dev dep without a DEC. `no-secrets-in-code` / privacy — **do not
  commit the maintainer's real photos**; ship a clean corpus, keep the real corpus external.
- `pure-rust-codecs-default` / `ergonomic-defaults` — the audit output must be honest (report negative
  savings, passthrough, the real score) and pipe-clean.

### Out of scope (this spec)
- Authoring BENCHMARKS.md itself (SPEC-083, STAGE-028) — this ships the *harness + numbers* it stands on.
- `meta` group (SPEC-087); `convert --to` (SPEC-089); any telemetry/phone-home (explicitly excluded).
- New quality/decision behavior.

## Notes for the Implementer
- **Two deliverables, keep them coherent:** the `--json`/`--timing` output *is* what the bench harness
  consumes — design the schema first, then have the harness read it, so they can't drift.
- **Extend the schema, don't fork it.** `optimize.explain/v1` already carries score (DEC-071); add
  `timing` the same additive/gated way, and reuse the shape for `web`/`apply`.
- **Corpus = clean + small + external-capable.** Commit a handful of synthetic/CC0 images with a
  provenance README; make the harness accept `--corpus <dir>` so the maintainer runs the *real* numbers
  for SPEC-083 without those photos entering git.
- **Reproducible + offline + no telemetry** is the whole point — a skeptic must be able to re-run it.
- Verify will re-run `just bench` and diff a plain run's output against pre-spec to prove non-audit
  output is unchanged.

---

## Build Completion
- **Branch:** `spec-088-audit-bench`
- **PR:** #92 (opened against `main`; orchestrator handles verify → merge → bookkeeping)
- **All acceptance criteria met?** Yes.
  - `--timing` on optimize/web/apply reports decode/encode/total (human → stderr; folded into `--json`); stdout stays pipe-clean. ✅ (`timing_flag_reports_and_json_includes_it`, `non_json_output_unchanged`)
  - `--json` consistent across optimize/web/apply — the `optimize.explain/v1` schema extended additively + versioned (gated `"timing"` object; `"ssim"` unchanged), NOT forked; a non-`--json`/non-`--timing` run is byte-identical. ✅ (`json_shape_consistent_across_verbs`, `non_json_output_unchanged`, decide.rs unit tests)
  - `lint`'s machine-readable output reconciled into the audit story: documented as the existing `lint --format json` audit surface (no schema fork — findings are a different domain). ✅ (DEC-074, docs)
  - `just bench` runs offline over the committed corpus, printing a savings/time/score table; `--json` mode emits raw numbers; deterministic, no network, no telemetry. ✅ (`bench_runs_offline_on_committed_corpus`)
  - Committed corpus is license-clean (synthetic CC0, provenance README, zero EXIF) and small (~40 KB); harness accepts `--corpus <dir>` for the real SPEC-083 numbers. ✅ (`bench_corpus_is_license_clean`, README)
  - Gates: `cargo test` default (731) **and** `--features avif` (744), `cargo clippy --all-targets`, `cargo fmt --check`, `cargo build --no-default-features`, `just validate`, `just bench` — all green. ✅
- **New decisions:** DEC-074 (audit-report schema extension + committed-bench/corpus policy).
- **Deviations:**
  1. **`just bench` repurposed** from the criterion micro-benches → the committed corpus harness (the spec's named recipe); the criterion recipe moved to **`just bench-micro`** (DEC-028's name shifts, intent kept). Recorded in DEC-074.
  2. **Audit surface is auto-decision-only.** `--json`/`--timing` on a format-pinned (`-o`/`--format`), `--profile preserve`, or plain-pixel-recipe run is a **usage error (exit 2)**, not a silent no-op (there is no decision to report). Chosen over silently ignoring the flag (a known repo footgun). `optimize`'s legacy `--explain` keeps its pre-existing silent-ignore-on-pin behaviour (byte-identity).
  3. **Committed photos honestly pass through (0%).** The smooth synthetic JPEGs are already near-optimal, so `web`/`optimize` correctly never-bigger them; real savings show on the maintainer's `--corpus`. Documented in `bench/corpus/README.md` rather than faking a win.
  4. **Harness is Python 3 stdlib** (`scripts/bench.py`), a checked-in script per the spec's allowance — no new Cargo/dev dep. `just bench` builds `--release --features avif` so the flagship AVIF path is exercised.
- **Follow-ups:**
  - SPEC-083 authors BENCHMARKS.md on top of `just bench --corpus <real>` (out of scope here).
  - Consider a `docs/` note / CLI hint if a user passes `--json`/`--timing` on the pinned path (today: usage error — clear, but a hint could help).

### Build-phase reflection
1. **What surprised me?** The synthetic "photo" corpus refused to compress: smooth gradients are already JPEG-optimal (passthrough), and adding high-frequency noise to force a codec win just bloated the file to 160 KB while *still* passing through (noise is incompressible for AVIF too). The honest resolution — a tiny corpus that legitimately exercises the never-bigger path, with real savings deferred to `--corpus` — is better than a manufactured number.
2. **What was the load-bearing design choice?** Building the schema first and having the harness *consume the CLI's own `--json`* (not re-implement measurement) — the spec's "keep them coherent" instruction. It means the bench can't drift from the report, and the gated-additive discipline (copying `ssim`'s exact pattern) kept every non-audit run byte-identical, which the regression anchor proves.
3. **What would I check first in verify?** That a plain run's stdout/stderr is truly byte-identical to `origin/main` (the anchor test asserts structure, not literal pre-spec bytes), and that `just bench` is green offline on a clean checkout with only `python3` present — plus that `--json`/`--timing` correctly error (not silently pass) on every non-autodecide path (pinned `-o`, `--format`, `--profile preserve`, plain recipe).

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
