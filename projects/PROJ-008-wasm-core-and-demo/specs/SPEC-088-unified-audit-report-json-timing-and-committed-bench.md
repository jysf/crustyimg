---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-088
  type: story
  cycle: verify
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
    - cycle: verify
      interface: claude-code
      tokens_total: 340000
      estimated_usd: 3.40
      recorded_at: 2026-07-16
      note: >
        ~35 min, own worktree (detached at origin/spec-088-audit-bench). Main-loop ESTIMATE (no metered
        subagent, §4 + [[autonomous-run-cost-estimates]]): order-of-magnitude tokens at Opus 4.8 list
        rate, ~80/20. Adversarial pass: built the PRE-SPEC parent binary (913faef) as an oracle and
        byte-diffed 28 real runs; regenerated the corpus from its committed generator; drove the offline
        claim under a network-denying sandbox; re-ran every gate. Verdict ⚠ PUNCH LIST (4 items).
    - cycle: fix
      interface: claude-code
      tokens_total: 400000
      estimated_usd: 4.05
      recorded_at: 2026-07-16
      note: >
        ~45 min, worktree spec-088-audit-bench. Main-loop ESTIMATE (no metered subagent, §4 +
        [[autonomous-run-cost-estimates]]): order-of-magnitude tokens at Opus 4.8 list rate, ~80/20.
        Cleared verify's 4-item punch list + the maintainer's corpus ruling (do both). Guarded the
        `--json` + `-o -` stdout collision at the shared writer (covers all 3 verbs + the pre-existing
        `--explain=json`; DEC-074 §Corrections) + a new test proven to fail without the guard; wrote the
        real `docs/cli-reference.md` §"Audit surface" + DEC-074 #3 so the lint criterion has evidence;
        de-staled DEC-028 (`bench` → `bench-micro`); made the justfile AVIF claim true by adding a real
        CC0 photo (Wikimedia/DimiTalen, CC0-1.0, license read from the API, metadata-stripped) that the
        classifier calls `photograph` — the only row reaching AVIF; renamed the mislabelled
        `photo_*.jpg` → `gradient_*.jpg` (engine says `graphic-logo`); added a `print_table` caveat
        footer. Re-proved byte-identity vs the pre-spec oracle (32/32) + re-ran every gate
        (732/745, +1 test).
  totals:
    tokens_total: 1210000
    estimated_usd: 12.35
    session_count: 3
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
- **Status:** amended by the **fix pass (2026-07-16)** below, which cleared verify's 4-item punch list + the maintainer's corpus ruling. Criteria rows reflect the post-fix state; rows the fix changed are marked ⟳.
- **All acceptance criteria met?** Yes — with one criterion narrowed honestly (see the `web`-vs-`optimize` note under the bench row).
  - `--timing` on optimize/web/apply reports decode/encode/total (human → stderr; folded into `--json`); stdout stays pipe-clean. ✅ ⟳ (`timing_flag_reports_and_json_includes_it`, `non_json_output_unchanged`, **`json_report_refuses_stdout_sink`** — the `-o -` collision verify found is now a usage error on all three verbs *and* on the pre-existing `--explain=json`; DEC-074 §Corrections)
  - `--json` consistent across optimize/web/apply — the `optimize.explain/v1` schema extended additively + versioned (gated `"timing"` object; `"ssim"` unchanged), NOT forked; a non-`--json`/non-`--timing` run is byte-identical. ✅ (`json_shape_consistent_across_verbs`, `non_json_output_unchanged`, decide.rs unit tests; **re-proven against the pre-spec oracle in the fix pass: 32/32 runs identical**)
  - `lint`'s machine-readable output reconciled into the audit story: `lint --format json` (`crustyimg.lint/v1`) documented as a first-class audit surface alongside the decision report, with the schema split + the `--format json` vs `--json` spelling difference explained. ✅ ⟳ (**`docs/cli-reference.md` §"Audit surface"** — a real section, written in the fix pass; **DEC-074 decision #3**. Verify was right that the original claim cited evidence that did not exist.)
  - `just bench` runs offline over the committed corpus, printing a savings/time/score table; `--json` mode emits raw numbers; deterministic, no network, no telemetry. ✅ ⟳ (`bench_runs_offline_on_committed_corpus`; the table now prints a **caveat footer** naming what the corpus cannot show — suppressed under `--corpus <real>`.) **Narrowed honestly:** the table is a *smoke/regression* harness. `web` == `optimize` on every committed row because no committed image exceeds `web`'s 2048px downscale bound, so the `web`-vs-`optimize` contrast the spec's Goal mentions is **not demonstrated by the committed corpus** and must come from `--corpus <real>` (SPEC-083). Stated in the footer, the corpus README, and DEC-074.
  - Committed corpus is license-clean (provenance README, zero EXIF) and small (~200 KB); harness accepts `--corpus <dir>` for the real SPEC-083 numbers. ✅ ⟳ (`bench_corpus_is_license_clean`, README) — now **4 synthetic + 1 real CC0 photograph** (`photo_forest_cc0.jpg`, Wikimedia Commons/DimiTalen, CC0-1.0, metadata-stripped). The photo is load-bearing: every synthetic image classifies `graphic-logo`, so before it **no bench row ever reached AVIF** despite the `--features avif` build. It classifies `photograph` and its AVIF candidate wins (30%, ssim 81.4).
  - Gates: `cargo test` default (**732**) **and** `--features avif` (**745**), `cargo clippy --all-targets`, `cargo fmt --check`, `cargo build --no-default-features`, `just validate`, `just bench`, `just bench-micro` — all green (re-run in the fix pass; +1 test vs the build). ✅
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

## Verify (2026-07-16) — ⚠ PUNCH LIST

Independent adversarial pass in its own worktree. **The two load-bearing claims are PROVEN**, not
taken on trust; four defects sit on top of them, none of which touch the engine.

### Proven clean (driven, not assumed)

- **Byte-identity — proven against the pre-spec ORACLE.** Built the parent commit (`913faef`) as a
  binary and diffed **28 real runs** (`optimize` / `optimize --verify` / `optimize --explain=json` /
  `optimize --max` / `web` / `web --max` / `apply --recipe web` × 4 corpus images): **stdout, stderr,
  exit code, and output image bytes all identical**. The spec's `non_json_output_unchanged` asserts
  *structure* only — the build's reflection said so honestly, and this closes the gap. The pre-existing
  `--explain=json` golden is byte-identical too.
- **Privacy — proven, no real photo ever entered git.** Only four image blobs exist across the whole
  branch history (all `bench/corpus/`). `cargo run --example gen_bench_corpus` **regenerates all four
  byte-identically** — the bytes are a pure function of committed math (`sin`/`cos`, no reads, no
  network, no `include_bytes!`), so no photo can be hiding in them. Zero EXIF/GPS/ICC/XMP by both
  `info` and a raw marker scan. The only `_incoming0` strings are pre-existing prose paths in DEC-069 /
  SPEC-084, not data.
- **Schema additive, `/v1` intact.** `timing` is absent without `--timing`, ordered after `ssim`,
  mirroring DEC-071 exactly. `--json` is `conflicts_with = "explain"` at the clap layer, so the
  pre-existing `--explain=json` cannot be shadowed or diverge; `optimize --json` is a true synonym.
- **Usage-error guard consistent** across all three verbs × pinned `-o` / `--format` / `--profile
  preserve` / plain recipe → exit 2 with an actionable message. Breaks no previously-working
  invocation (the flags are new; control runs still exit 0).
- **Offline — proven by driving.** Ran the harness under `sandbox-exec` with `(deny network*)`, having
  first proven *the blocker blocks* (`curl` exits 6 under the sandbox, 0 outside). Harness green, rc=0.
  Imports are stdlib-only; the sole network strings are comments.
- **Gates re-run here:** `cargo test` 731 default / 744 `--features avif` (matches the build's claim),
  clippy clean, `fmt --check` clean, `--no-default-features` builds, `just validate`, `just bench`,
  `just bench-micro` (compiles). `decisions-audit`: 0 structural errors. No CI job calls `bench`, so
  the rename breaks nothing.

### Punch list

1. **`--json` + `-o -` collides on stdout** (contradicts the spec's "stdout stays pipe-clean").
   The `pinned` guard deliberately excludes `-o -`, which is precisely the case where the report and
   the image contend for stdout. Repro:
   `crustyimg web bench/corpus/photo_large.jpg --json -o - > out` → `out` is a 512-byte JSON line
   followed by the 15449-byte image. **Pre-existing** for `optimize --explain=json -o -` (the oracle
   binary does it identically), but SPEC-088 propagated it to `web`/`apply`, which had no `--json`
   before. `non_json_output_unchanged` covers `-o -` *without* `--json`, so it can't catch this.
   Fix is one condition: treat `-o -` as audit-incompatible (exit 2), or send the report to stderr
   when the sink is stdout.
2. **The `lint` acceptance criterion is claimed ✅ on evidence that does not exist.** The row cites
   "(DEC-074, docs)": **DEC-074 contains zero occurrences of "lint"**, and no doc anywhere documents
   `lint --format json` as the audit surface. The whole SPEC-088 diff contains no lint change. The
   claim is its own only evidence. (`lint --format json` does still work, untouched.) Separately,
   `lint` spells it `--format json` while the new surface spells it `--json` — the criterion's
   "`--json` where it fits" inconsistency is untouched and undocumented. Either write the one
   paragraph, or drop the criterion honestly. A new variant of
   [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]: the row is present but its citation is
   hollow.
3. **DEC-028 is now stale.** Its body still reads "runnable via `just bench`" (lines 44, 92) for the
   criterion micro-bench, which is now `just bench-micro`. DEC-074 records the rename and nothing
   breaks, but DEC-028 carries no pointer or `superseded_by`. One-line fix.
4. **The justfile's AVIF claim is false.** `bench` comments that it builds `--features avif` "so the
   flagship AVIF path (the `web` story) is exercised" — **no corpus row ever produces AVIF** (see the
   assessment below). The extra build cost buys nothing on the committed corpus.

### Assessment — does the committed bench serve its stated purpose?

**Judgment: it is a smoke / regression harness, not a demonstration of the `web`-vs-`optimize` story —
and the gap is wider than `bench/corpus/README.md` admits.**

On all 8 rows, `web` and `optimize` emit **identical** `out_bytes` and savings. STAGE-030's whole
thesis (`web` 98% / size-insensitive vs `optimize` 24%) is invisible. Two causes:

- *Documented:* every image is ≤512px, under the 2048 long-edge bound, so `web`'s downscale — its
  entire point — never fires and `web` ≡ `optimize`.
- *Not documented, and the sharper one:* **crustyimg's own classifier labels all four images
  `graphic-logo`** — including both `photo_*.jpg`. The photo/lossy branch is never entered and **AVIF
  never fires once**. So the README's Contents table calling them "photo" / "lossy-family source" is
  contradicted by the engine, and the justfile asserts the opposite of what happens.

What it does buy is real: it proves the harness wiring, that the bench consumes the CLI's own `--json`
(so the two can't drift), the graphic→lossless-WebP branch, and the never-bigger passthrough path.
That is worth committing. But **every headline number for SPEC-083 must come from `--corpus <real>`** —
this corpus is scaffolding, not evidence.

Is the limitation documented where a skeptic hits it? **Partially, and not where it counts.**
`bench/corpus/README.md` §"What the smoke numbers show (and honestly don't)" is genuinely honest about
the size cap — good. But a skeptic runs `just bench`, not `cat README.md`, and **the table itself
carries no caveat**. Reading "photo_large.jpg web 0%" and `web == optimize` on every row, the
reasonable conclusion is "this tool does nothing" — the exact opposite of the launch pitch.

**Recommendation (cheap, high value):** print a one-line footer from `print_table` — e.g. *"smoke
corpus: ≤512px synthetic, all graphic-class → `web`==`optimize` and AVIF never fires; run
`--corpus <real>` for the launch numbers"* — fix the justfile AVIF comment, and reconcile the README's
"photo"/"lossy-family" labels with the classifier's actual verdict. Adding a ≥2048px synthetic is
**not** an easy win: I generated one (3000px, same pure-math formula) and it also classifies
`graphic-logo`, takes lossless-WebP, and lands **−14% (bigger)** — reinforcing the build's reflection
that synthetic gradients aren't photos.

*Out of scope, flagged for the maintainer:* that −14% is `web` shipping a file **larger than the
source**. The oracle binary does it identically, so it is **pre-existing SPEC-085 behaviour, not a
SPEC-088 regression**, and it is reported honestly ("14% larger"). But it shows `web`'s never-bigger
guarantee does not hold once the downscale forces a re-encode. Worth its own look.

---

## Fix pass (2026-07-16) — punch list cleared

All four punch-list items fixed, plus the maintainer's corpus ruling (do **both**:
add a real CC0 photo *and* fix the false labels + caveat). Scope held to those;
the `web`-ships-14%-larger finding was left alone (pre-existing SPEC-085
behaviour, now framed as **SPEC-090** on main).

### What changed

1. **`--json` + `-o -` no longer collides** (punch list 1). The JSON report and the
   image both target stdout, so the combination is now a **usage error (exit 2)**,
   enforced at `run_optimize_autodecide` — the single writer all three verbs share,
   so `web`/`apply`/`optimize --json` **and** the pre-existing
   `optimize --explain=json` are covered by one rule. `--timing` alone and the human
   `--explain` render to stderr and stay compatible with `-o -` (asserted).
   *Deliberate correction of pre-existing behaviour*, per the maintainer's call and
   recorded in **DEC-074 §Corrections**. Proven, not assumed: the oracle emits
   15,963 bytes that parse as **neither JSON nor JPEG** (image magic at offset 514,
   behind a 514-byte JSON prefix) — output no working pipeline can consume.
   New test `json_report_refuses_stdout_sink` (4 spellings + the `--timing`
   negative); **confirmed it fails without the guard** (exit 0, the collision).
2. **The `lint` criterion now has real evidence** (punch list 2). Verify was right —
   DEC-074 contained zero occurrences of "lint" and no doc described the audit
   surface. Written for real: **`docs/cli-reference.md` §"Audit surface"** (both
   schemas, both spellings, the pinned + pipe-clean rules) and **DEC-074 decision
   #3** (why `crustyimg.lint/v1` stays separate rather than merging into
   `optimize.explain/v1`). Also closes verify's sub-point: the `--format json` vs
   `--json` spelling difference is now explained (`--format` selects
   `human`/`json`/`sarif`; a boolean flag could not) rather than left undocumented.
3. **DEC-028 de-staled** (punch list 3): body now reads `just bench-micro`, with an
   `Amended by DEC-074` pointer in its References (not a supersession — only the
   recipe name moved).
4. **The justfile's AVIF claim is now true** (punch list 4) — and true *because* the
   corpus changed, not because the sentence was softened: `photo_forest_cc0.jpg`
   classifies `photograph` and its AVIF candidate wins, so `--features avif` is
   genuinely regression-tested. The comment now names which row does it and what is
   still not exercised.
5. **Corpus — both halves of the ruling.**
   * **Added a real CC0 photograph.** ["Petite Hesse spruce forest undergrowth,
     Waimes, 2023"](https://commons.wikimedia.org/wiki/File:Petite_Hesse_spruce_forest_undergrowth,_Waimes,_2023.jpg)
     (Wikimedia Commons, DimiTalen, own work, **CC0-1.0**; license read from the
     Commons API's `extmetadata`, `AttributionRequired = false` — verified, not
     assumed from the category). Downscaled 6016×4000 → 800×532, q≈65, **all
     metadata stripped** via `crustyimg meta strip`: 174 KB, bare JFIF header, no
     EXIF/GPS/ICC/XMP (raw segment scan + `info --json`). **Classifier driven, not
     assumed:** `class=photograph`, and it is the *only* corpus row that reaches
     AVIF. Selection was evidence-led — the first CC0 candidate (a landscape)
     classified `ui-screenshot`, and a second classified `photograph` only via the
     EXIF camera prior; this one classifies on **pixel content alone**
     (flat_ratio 0.02 < 0.25, entropy 7.58 ≥ 5.0), which is why it survives metadata
     stripping.
   * **Every label is now true.** `photo_small/large.jpg` → **`gradient_small/large.jpg`**:
     the engine calls them `graphic-logo` (flat_ratio **1.00**), so the `photo_*`
     names and the README's "photo"/"lossy-family" rows were contradicted by the
     tool. Generator doc-comments corrected too (`photo()` → `gradient()`, no longer
     claiming "the lossy (photograph) family"). Every `class` in the README is now a
     value read out of `optimize --json`.
   * **Caveat footer in `print_table`** — a skeptic running `just bench` from a clean
     clone now reads, *in the output*, that `web` == `optimize` on every row (no
     committed image exceeds the 2048px bound), that only one row reaches AVIF, and
     that headline numbers require `--corpus <real>`. Suppressed automatically for a
     real `--corpus` (where it would be a lie); `--json` carries the same fact as
     `"smoke_corpus": true`.

### Proof (driven, not asserted)

- **Byte-identity re-proven against the pre-spec oracle** (parent `913faef`, built
  `--release --features avif`): **32/32 runs identical** — stdout, stderr, exit
  code, and output image bytes — across `optimize` / `--verify` / `--explain=json` /
  `--explain` / `--max` / `web` / `web --max` / `apply --recipe web` × 4 corpus
  images. The only intended divergence is the `-o -` correction above.
- **Every documented claim driven:** the `--json`/`--explain=json` clap conflict, the
  `--timing`→stderr rendering, the pinned-path exit 2, `lint --format json`'s schema
  + summary keys, and `--format sarif` (which does ship — checked rather than
  trusted; and the values are `human`/`json`/`sarif`, **not** `text`, which an early
  draft of the doc got wrong).
- **Gates re-run:** `cargo test` **732** default / **745** `--features avif` (+1 each,
  the new test), clippy `--all-targets` clean, `fmt --check` clean,
  `--no-default-features` builds, `just validate` (205 front-matter blocks),
  `just bench` green + footer, `just bench-micro` compiles, `just decisions-audit`
  0 structural errors.

### Honestly unmet / narrowed

- **No criterion is left unmet**, but one is **narrower than the Goal's wording
  implies** and is now marked so rather than quietly claimed: the committed bench
  does **not** demonstrate `web` vs `optimize`. Every committed image is under the
  2048px downscale bound, so the two verbs converge on every row; a >2048px real
  photo cannot be both committed and lean (measured: 2048px q45 ≈ 620 KB, 3× the
  budget). The corpus proves the *harness* and all three decision *branches*; the
  headline numbers must come from `--corpus <real>` (SPEC-083). Stated in the bench
  footer, `bench/corpus/README.md`, and DEC-074 §Consequences.

### Fix-phase reflection
1. **What surprised me?** That the punch list's own trap caught me mid-fix. Writing
   the corpus provenance I typed a plausible Wikimedia author — and it was the wrong
   person; the API said **DimiTalen**. I was, at that moment, doing exactly what
   punch-list item 2 was about: writing a citation whose only evidence was my own
   confidence. The same thing happened again one paragraph later with `--format
   text` (the real values are `human`/`json`/`sarif`). Both were caught only by
   driving the source. A provenance line and a docs line *look* like prose, not
   claims — which is precisely why they ship unverified.
2. **What was the load-bearing choice?** Putting the `-o -` guard at
   `run_optimize_autodecide` rather than in each verb. Three verbs × future spellings
   is three chances to miss one; the report has exactly one writer, so the invariant
   belongs there. It fixed the pre-existing `--explain=json` case for free — the
   inconsistency the maintainer asked me not to leave.
3. **What would I check first if this were re-verified?** That the corpus README's
   `class` column still matches what the classifier says — it is the one claim here
   that a future thresholds change can silently falsify, and `photo_*.jpg` is proof
   this repo has drifted that way before. A test asserting `photo_forest_cc0.jpg`
   classifies `photograph` (and thus that AVIF is exercised) would make the drift
   loud; I did not add one, because the maintainer scoped this pass to the punch
   list. Worth a follow-up.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
