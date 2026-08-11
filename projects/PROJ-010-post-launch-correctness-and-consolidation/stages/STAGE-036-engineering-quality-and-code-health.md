---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-036
  status: proposed
  priority: medium
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-26
shipped_at: null

value_contribution:
  advances: >
    The continuation of PROJ-008's STAGE-031, which shipped its three adopted audit items
    (SPEC-097/098/099) and closed there. This stage inherits its one unframed tail item
    (strict-JSON escape_json), its shelved-directive record, and its byte-identical
    pre-change-oracle gate — and holds whatever further code-health work the maintainer
    decides is worth the churn. Raises the codebase's bar for the scrutiny a launch brings.
  delivers:
    - "The strict-JSON escape_json follow-up from SPEC-097 (0x7F and >=0x20 controls pass through unescaped today)"
    - "Whatever of the candidate list below survives maintainer triage, each proven byte-identical against a pre-change oracle"
  explicitly_does_not:
    - "Change any user-visible behaviour, CLI surface, or output — except escape_json, which is a deliberate behaviour fix and says so"
    - "Add new features, codecs, or engine capabilities"
    - "Re-raise the audit directives PROJ-008 shelved (D1/D2/D3/D5/D6) — see Design Notes"
    - "Block the launch — sequenced post-launch, run only per maintainer judgment"
---

# STAGE-036: Engineering quality and code health

## What This Stage Is

The continuation of PROJ-008's STAGE-031, which shipped three specs there (SPEC-097 CLI split, SPEC-098 DEC-078, SPEC-099 DEC-079) and is now closed. **This is a continuation, not a re-home** — STAGE-031 stayed in PROJ-008 with its shipped specs and PR provenance, and this stage inherits its unfinished tail and its governing rules.

What is genuinely carried forward, with a source:

- **strict-JSON `escape_json`** — the one unframed SPEC-097 follow-up. `0x7F` and `≥0x20` control characters pass through unescaped.
- **The shelved-directive record** (D1/D2/D3/D5/D6, marked *do not re-raise*) — see Design Notes.
- **The byte-identical pre-change-oracle gate** — the standing gate for every spec here.

Everything else in this stage is a **candidate, not a commitment**, and is labelled as such in the backlog below.

This stage is **post-launch** by design. The classifier fix and hostile-input pass (STAGE-034/035) are the launch gating items; code health is what you do after the launch is safe, to make the codebase ready for the contributors a launch attracts.

## Why Now

Framed now because the tail item needs a home and framing is cheap; **executed deliberately, gated on maintainer triage, not on the launch clock.** Nothing here blocks the launch — sequence after STAGE-035.

The draft's "cost of deferring compounds" argument is not repeated, because each leg of it was unmeasured: `-D warnings` is already a required, currently-green gate, the "5+ minute test suite" figure has no source, and the edition migration is declined-by-default under DEC-009 rather than pending. The honest reason to have this stage is the `escape_json` tail plus a place to triage candidates.

## Success Criteria

- The `escape_json` strictness follow-up (SPEC-097 tail) ships: `0x7F` and `≥0x20` controls are escaped. **This one is a deliberate behaviour change** — byte-identity is explicitly *not* its gate.
- `cargo clippy --all-targets -- -D warnings` is clean and `cargo fmt --check` passes — as they must for every spec in this repo; these are the standing gate, not this stage's achievement.
- Every candidate item the maintainer adopts proves **byte-identical behaviour** against a pre-change oracle. A maintainability change that alters behaviour has failed.
- Every candidate item the maintainer **declines** is recorded as declined, with the reason, so it is not re-proposed. A probe that concludes "not now, because X" is a successful outcome.

## Scope

### In scope
- The `escape_json` strictness fix: escape `0x7F` and `≥0x20` control characters (a known remaining issue after SPEC-097's split).
- Behaviour-preserving structural refactors and internal de-duplication, on the same terms STAGE-031 set.
- Triage of the candidate list below: each item either framed as a spec, or recorded as declined.

### Explicitly out of scope
- Any behaviour/surface/output change other than `escape_json` — byte-identical is the gate for everything else.
- New features or engine capabilities.
- Any classifier, codec, or wasm work.
- **Re-raising D1/D2/D3/D5/D6.** They were evaluated and shelved with reasons; see Design Notes.

## Spec Backlog

### Carried forward with provenance

- [ ] (not yet framed) — **Strict JSON `escape_json`.** SPEC-097 follow-up: escape `0x7F` and `≥0x20` controls through the serialization path. Byte-identity is NOT the gate here — this is a deliberate behaviour change to fix a correctness issue. **This is STAGE-031's entire carried tail.**

**Count:** 0 shipped / 0 active / 1 pending

### Candidates — TRIAGED 2026-08-10. All five DECLINED for now; text kept in full below.

> **Read this before treating anything below as work.** Every candidate in this section was
> triaged on **2026-08-10** and **declined**, with the reason recorded inline against each one.
> Nothing is deleted — the full original text stays exactly as written so the decision can be
> reviewed, and any of the five can be revived by a spec that supplies what it currently lacks.
>
> **Summary of the five decisions:**
>
> | candidate | decision | why |
> |---|---|---|
> | clippy `doc_markdown` sweep + cast audit | **declined for now** — revive post-launch | Measured and real (78 sites), but the required gate is green and 91% are doc-comment backticks. Zero user value; the stage's own note says do it when nothing else is editing those 27 files. |
> | test-speed stratification | **declined** — needs a measurement first | Both its numbers ("50 slowest tests", "~30s dev loop") are unsourced. Revive with an actual `--report-time` distribution. |
> | Rust 2024 edition migration | **declined** — contradicts a live decision | DEC-009 chose 2021 and rejected 2024 explicitly. Reviving needs a DEC that supersedes it, names the compelling feature, and accepts the MSRV rise from 1.90.0. |
> | `pulp` for SIMD | **declined** — premise false | Not in the dependency tree (`grep` returns 0), so it is a **new top-level dep**, not a usage gate: needs `no-new-top-level-deps-without-decision`, a licence review (MIT-only, no explicit patent grant), `deny`, and an MSRV probe. |
> | `zlib-rs` flate2 backend | **declined** — unmeasured | Not in the lockfile, and the "2× faster PNG encode" figure is unsourced. A backend swap's entire justification is a number nobody has produced. |
>
> **Why decline rather than leave them open:** a list of six items where only one is real reads
> as planned work to anyone who has not read the caveats — including a future session. Three of
> the five are not merely unsourced but *unsound as stated* (one contradicts a live DEC, one has
> a false premise, one has an invented number). Recording that as a decision is the honest
> outcome; leaving them ambient is how an unsourced claim becomes a spec.
> [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]

⚠ **None of the items below has provenance in this repo.** They appear only in the untracked PROJ-010
draft, attributed to a readiness-analysis session that left no committed record. They are recorded here
so the ideas are not lost, **not** as a backlog. Each needs a source or a measurement before it becomes
a spec, and "declined" is a legitimate outcome for any of them.

- **Clippy `doc_markdown` / `redundant_clone` / `manual_let_else` sweep + cast audit.** ✅ **MEASURED
  2026-07-26** (clean `cargo clean -p crustyimg` then `--all-targets`, confirmed it actually
  recompiled — the cached second run reports 0 and is a false green,
  [[a-stale-incremental-build-is-a-false-green]]):

  | lint | unique sites |
  |---|---|
  | `doc_markdown` ("item in documentation is missing backticks") | **71** |
  | `manual_let_else` | **5** |
  | `redundant_clone` | **2** |
  | **total** | **78 sites across 27 files** |

  **Verdict: real but cosmetic, and not urgent.** `cargo clippy --all-targets -- -D warnings` — the
  required gate — is **green today**; all 78 come from lints that are not enabled. **91% are
  doc-comment backticks**, and clippy reports most as auto-applicable (`--fix` offers 25 suggestions
  on the lib-test target alone). Only the 2 `redundant_clone` hits have any runtime meaning, and that
  lint is nursery.

  **Do it here, post-launch, not before.** The 27 files span `src/analysis`-adjacent code,
  `src/cli/optimize.rs`, `src/cli/ops.rs` and `tests/cli.rs` — exactly what STAGE-034 and STAGE-039
  are editing. Running a zero-user-value 78-site sweep across those files while two launch-gating
  stages are in flight buys merge conflicts and nothing else.

  Shape when pulled: one auto-fix commit per lint, each with its own oracle run, then register the
  three in `Cargo.toml [lints.clippy]` so the count cannot regrow.

  ⚠ **Note what this sweep would NOT have caught:** rule 6's unreachable dead code
  (`src/analysis/mod.rs:625`), which breaks the *blocking* `clippy-fmt-clean` constraint and stays
  invisible to `-D warnings` because the constants remain syntactically referenced. That is in
  STAGE-034. A green clippy run is not evidence of no dead code.
- **Test-speed stratification.** "The 50 slowest integration tests" and the "~30s dev loop" target are both unsourced numbers. Measure the actual distribution via `cargo test -- -Z unstable-options --report-time` before committing to a shape; the gating mechanism (`#[cfg_attr(not(feature = "ci-full-suite"), ignore)]`) is sound if the measurement supports it.
- **Rust 2024 edition migration.** ⚠ **Contradicts a live decision, and the draft cited neither side.** `DEC-009` chose edition 2021 and explicitly rejected 2024 — *"bumps the minimum stable toolchain for no MVP-required feature"* — with a stated revisit trigger: *"an edition-2024 feature becomes compelling."* The draft named no such feature. It also silently raises the MSRV: it requires stable 1.94.1+ against `Cargo.toml:7`'s `rust-version = "1.90.0"`, a floor derived as max(rust_version) across the locked tree ([[msrv-floor-from-cargo-metadata]]). **Do not frame this as a chore.** It needs a DEC superseding DEC-009 that names the compelling feature and accepts the MSRV cost — or it stays declined.
- **`pulp` for SIMD in quality-metric inner loops.** ⚠ **The draft's premise is false.** `pulp` is **not** in the dependency tree — `grep 'name = "pulp"' Cargo.lock` returns **0** (positive control: `flate2` returns 1). This is a **new top-level dependency**, not "a usage gate", so it triggers the full discipline: `no-new-top-level-deps-without-decision`, a licence check (`pulp` is **MIT-only** — no explicit Apache patent grant), the `deny` gate, and an MSRV probe. Independently corroborated as a reasonable *candidate* by `docs/research/photo-preset-import-and-photographic-ops.md` §36, which also notes `std::simd` is confirmed **not** coming (rust#86656 untouched since 2025-03) — but that document reaches it as a new dependency too. Also note it sits close to the shelved D5/D6 territory below.
- **`zlib-rs` as a flate2 backend.** `zlib-rs` is also **not** in the lockfile (count 0). The draft's "**2× faster PNG encode**" figure is **unmeasured and unsourced** — dropped. If probed, measure it here; a backend swap's whole justification is a number.

**Candidate count:** 5 — **all declined 2026-08-10**, none framed, none committed, none deleted.
Revivable by a spec that supplies what each currently lacks (see the triage table above).

**Stage total after triage: 1 real pending item** (the strict-JSON `escape_json` tail).

## Design Notes

- **This stage continues PROJ-008's STAGE-031; it does not re-home it.** STAGE-031 shipped its three
  adopted items and closed in PROJ-008, where its spec files and PR provenance live. This file holds
  what came after.

- **⚠ The shelved audit directives — do not re-raise.** Source: the pre-launch Rust audit
  (`docs/research/proj-008-rust-directives-audit.md`, landed 2026-07-19). Of 6 directives + 1 structural
  finding, the maintainer adopted two (the `src/cli/mod.rs` split → SPEC-097; the D4 pinning decision →
  SPEC-098/DEC-078, later corrected by SPEC-099/DEC-079) and **shelved the rest with reasons**:
  - **D1** checked-math-on-dimensions — **SATISFIED**; every buffer-sizing multiply is already u64/capped
    (the two plain-`usize` sites are proven-bounded, not defects). Only belt-and-suspenders remained.
    (Note: the real typed cap error is `ImageError::LimitsExceeded`, not `DimensionsTooLarge`.)
  - **D2** zero-allocation-pipeline — **N/A**; per-op allocation is real but not the bottleneck (encode
    dominates); a scratch-buffer rewrite chases a non-cost.
  - **D3** miette CLI diagnostics — **design change** vs DEC-007 (thiserror + exit-code mapping); a real
    UX idea, not pursued.
  - **D5** static-dispatch-in-hot-loops — **N/A (false premise)**; `Box<dyn Operation>` dispatches ~3×
    per image, not per pixel.
  - **D6** tile-based-parallelism — **N/A (false premise)**; rayon parallelizes across images; there is
    no intra-image striping to convert.

  This record is why the `pulp` SIMD candidate needs care: **D5 and D6 were shelved as false premises
  about where time goes in this codebase.** A SIMD probe must first establish that the quality-metric
  inner loops are actually hot, or it repeats a question already answered no.

- **The standing gate for every spec here: byte-identical behaviour, proven against a pre-change
  oracle.** Not "the tests still pass" — an independent oracle. SPEC-097 is the worked example: a
  6,483 → 1,426 line split proven with **27/27 golden outputs plus a function-body diff across ~170
  functions**, 0 tests dropped, no signature or visibility change. That method is what made a refactor
  that size safe to merge, and it is the method to reuse.

- **Signature/API changes are never bundled into a behaviour-preserving move** (e.g. argument-struct
  bundling) — they destroy the byte-identity gate. Separate cosmetic follow-ups, always.

- **Large mechanical diffs need structure.** Any adopted sweep should land as a discrete commit per lint
  category, each with its own `--fix` / `fmt` pass and its own oracle run. The maintainer may decline an
  item on diff size alone — that is an explicit success condition, not a failure.

## Dependencies

### Depends on
- STAGE-034/035 (launch-gating stages) — sequenced after so the launch is not blocked.
- PROJ-008 STAGE-031 (shipped) — this stage's tail item, shelved-directive record, and oracle gate.
- The shipped PROJ-008 codebase.

### Enables
- STAGE-038 (polish and housekeeping) — a clippy-clean, fast-testing codebase is easier to iterate on for the remaining chores.
- Future PROJ-011+ work on a maintained foundation.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
