---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-091
  type: story
  cycle: design
  blocked: false
  priority: high
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-17
references:
  decisions: [DEC-006, DEC-034, DEC-053, DEC-058]
  constraints: [pure-rust-codecs-default, every-public-fn-tested, test-before-implementation]
  related_specs: [SPEC-083, SPEC-088]
value_link: >
  Set an explicit AVIF-decode thread policy. Today every decoder instance spawns dav1d's auto thread
  pool (all cores) to decode a SINGLE still frame — inside a rayon batch that already parallelizes
  across files. That oversubscription is very likely what trips re_rav1d's DisjointMut overlap check
  (a CI flake today, wrong pixels in release), and it makes any decode timing we publish suspect —
  including SPEC-083's BENCHMARKS numbers, measured via the `--timing` SPEC-088 just shipped.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-091: AVIF decode thread policy (cap dav1d's threads; kill the DisjointMut flake)

## Context

SPEC-088's fix pass surfaced a **pre-existing** `--features avif` test flake: `re_rav1d`'s
`DisjointMut` overlap check panics under load (`disjoint_mut.rs:837`), reached because `web` always
scores → decodes the AVIF winner. It is load-dependent (measured 2/5 and 1/3 on the pre-fix tip;
passes 5/5 in isolation), so it is **not** a SPEC-088 regression.

**Investigation (design-time, 2026-07-17 — grounded, but re-verify each claim at build):**

1. **We ship it.** `re_rav1d = "=0.1.3"`, a **default native dep** (`Cargo.toml` ~162), no-asm
   (`default-features = false`, `bitdepth_8`/`bitdepth_16`), per DEC-053. It is in every native binary.
2. **The check is debug-only, and disjointness is unchecked in release.** `disjoint_mut.rs` ~59:
   *"This disjointness is unchecked in release mode…"*; the checker is `#[cfg(debug_assertions)]`. So a
   panic there means a **real overlap was detected**, and in release the same overlap is silent.
3. **Severity is bounded — this is NOT a memory-safety hole.** The same contract (~78–82) argues all
   `AsMutPtr::Target`s are **provenanceless** (pixel data; no internal pointers/provenance), therefore a
   release-mode data race *"would only result in wrong results, and cannot result in memory safety."*
   It is still UB by the Rust abstract machine, but the realistic consequence is **wrong pixels**, not an
   exploitable safety hole. **Do not frame this as a security fix.** The correctness angle is what bites:
   wrong decoded pixels → a wrong SSIMULACRA2 score, which is the honesty headline we sell.
4. **The likely mechanism — we never cap threads.** `src/image/avif.rs:260` (`decode_obus`) does
   `Settings::new()` and sets only `set_frame_size_limit` (DEC-034). `Settings::new()` calls
   `dav1d_default_settings()`, whose `n_threads = 0` = **auto = all cores**. So **each decoder instance
   spawns its own dav1d thread pool** — and the overlap fires across *dav1d's internal* threads, not our
   rayon. `Settings::set_n_threads(u32)` exists (`re_rav1d-0.1.3/lib.rs:257`) and we simply never call it.
5. **Capping is plausibly a perf WIN, not a cost.** A single AVIF **still frame** gains nothing from
   *frame*-threading (there is one frame). Worse, batch runs `par_iter` across files (DEC-006) and *each*
   file's decoder then spawns N-core threads → textbook oversubscription.
6. **Not reproduced locally** by the orchestrator in 3 suite runs (load-dependent). **The build must
   establish a reliable repro first** — a fix you cannot see fail is a fix you cannot prove.

## Goal

Decide and implement an explicit **AVIF-decode thread policy** (rather than inheriting dav1d's
all-cores default), backed by measurement: does capping threads (a) kill the DisjointMut flake, and
(b) help, hurt, or not move decode throughput — single-image and in a rayon batch? Emit a DEC.

## Inputs — files to read

- `src/image/avif.rs` — `decode_obus` (~258–280), `frame_size_limit` (DEC-034), and every `Settings`
  touchpoint. Note we create a **decoder per image**.
- `re_rav1d-0.1.3/lib.rs` — `Settings::set_n_threads` (~257), `set_max_frame_delay` (~265),
  `Settings::new`/`dav1d_default_settings` (~245).
- `re_rav1d-0.1.3/src/disjoint_mut.rs` — the safety contract (~50–85) and `check_overlaps` (~828–838).
- `src/cli/mod.rs` — the rayon batch fan-out (`ThreadPoolBuilder`/`par_iter`, ~1250, ~1789) and `--jobs`;
  `DEC-006` (sync core + rayon for batch) is the concurrency model this must fit.
- `benches/`, `scripts/bench.py`, `just bench-micro` — where a decode-throughput measurement belongs.

## Outputs

- **`src/image/avif.rs`** — an explicit, commented thread policy on the decode `Settings`. Candidates
  (measure, don't assume): **(A)** `set_n_threads(1)` for still-image decode — recommended prior: one
  frame can't use frame-threading, and it composes correctly under rayon; **(B)** a small cap (e.g. 2–4)
  if tile-threading measurably helps a large still; **(C)** thread count derived from whether we're
  inside a batch (more complex; only if the numbers justify it).
- **A reliable repro + a regression test.** Whatever kills the flake must be provably the thing that
  killed it — a test/harness that reproduced the panic *before* the change and does not *after*.
- **Measurements** (the decision's basis): single-image AVIF decode wall-clock at the current default
  vs the chosen policy, and a **batch** (`par_iter`, several files, `--jobs` default) at both — the
  oversubscription case is where a win is most likely. Reuse `--timing` (SPEC-088).
- **`DEC-075`+ (next free at build)** — the AVIF-decode thread policy: what was chosen, the measured
  numbers, the explicit note that this is a **correctness + throughput** decision and **not** a
  memory-safety fix (provenanceless targets), and its relationship to DEC-006.
- If the overlap survives a thread cap, **escalate honestly**: it is then a genuine re_rav1d/rav1d bug
  independent of our usage → file/point to an upstream issue and record it; do not paper over it.

## Acceptance Criteria

- [ ] A **reliable repro** of the `DisjointMut` panic exists on the pre-change code (documented: how to
      trigger, hit-rate). If it genuinely cannot be reproduced, say so explicitly and STOP — do not
      "fix" an unobserved bug.
- [ ] The chosen thread policy is **explicit in code** (not inherited from `dav1d_default_settings`) and
      commented with the rationale.
- [ ] The flake is **gone** under the repro (state the post-change hit-rate over a stated number of runs).
- [ ] **Measured** single-image AND batch decode throughput, before vs after; the DEC records both. A
      regression is acceptable ONLY if consciously traded for correctness and stated in the DEC.
- [ ] AVIF decode output is **byte-identical** (or pixel-identical) to before on the corpus — a thread
      policy must not change decoded pixels. Prove it against the pre-change binary.
- [ ] `cargo test` (default **and** `--features avif`) is green **across ≥5 consecutive full-suite runs**
      (the flake's whole tell is that one green run proves nothing); clippy, fmt,
      `cargo build --no-default-features`, `just validate`, `just bench`, `just bench-micro` pass.

## Failing Tests (written at design)

- **`src/image/avif.rs` / integration**
  - `avif_decode_thread_policy_is_explicit` — the decode `Settings` set a thread count deliberately
    (guards against silently regressing to dav1d's all-cores default).
  - `avif_decode_pixels_unchanged_by_thread_policy` — decoded pixels identical to the pre-change path.
  - `avif_batch_decode_does_not_oversubscribe` — a batch decode under rayon doesn't spawn ~N-cores
    threads *per file* (assert the observable proxy the build settles on; if unassertable, say so and
    cover it with the measurement instead).

## Implementation Context

### Decisions that apply
- `DEC-006` — sync core; **rayon** for batch parallelism across files. The decode thread policy must
  compose with it: parallelism belongs at the file level, not N pools inside one batch.
- `DEC-034` — `frame_size_limit` bounds decoder allocation; it is the existing precedent for setting
  decode `Settings` deliberately. **This spec extends that pattern to threads.**
- `DEC-053` — the `re_rav1d` no-asm pure-Rust choice (and its "serves the Wave-3 WASM demo" note, which
  the STAGE-025 probe already corrected). `DEC-058` — native AVIF decode shipped.

### Constraints
- `pure-rust-codecs-default`; `test-before-implementation`; `every-public-fn-tested`.

### Out of scope (this spec)
- The AVIF **encode** path (rav1e) and its `AVIF_SPEED` (DEC-016/019/020).
- Swapping or forking `re_rav1d`, or vendoring a patch — if the overlap survives a thread cap, this
  spec **reports** it upstream; it does not re-home the dep.
- The wasm build (re_rav1d isn't in it — DEC-064).
- SPEC-083's BENCHMARKS authoring (this spec makes its decode numbers trustworthy; it doesn't write them).

## Notes for the Implementer
- **Repro first, then fix.** Establish the panic reliably before changing anything — the whole risk here
  is "fixed" something that was never observed, then declaring a flake dead because one run was green.
  Green once proves nothing on a 1-in-3 flake ([[a-harness-that-exercises-nothing-reports-green]] and
  [[test-the-guard-where-the-criterion-applies]] both apply: run it enough times to see it fail).
- **Do not oversell it.** Upstream's own contract says provenanceless targets ⇒ wrong results, not
  memory unsafety. The DEC must say that plainly ([[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]).
- The investigation above is **design-time grounding, not proof** — re-drive the claims (auto-thread
  default, `set_n_threads` availability, the oversubscription) rather than trusting this prose. The
  citation lesson has bitten this stage three times ([[a-citation-looks-like-prose-not-a-claim]]).
- Land this **before SPEC-083 BENCHMARKS** — otherwise published decode timings are measured under
  oversubscription.

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
