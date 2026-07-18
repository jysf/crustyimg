---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-091
  type: story
  cycle: build
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 260000
      duration_minutes: null
      estimated_usd: 2.35
      note: >
        Order-of-magnitude estimate — build ran in the orchestrator main loop, not
        a separately metered subagent (per docs/cost-tracking + the autonomous-run
        cost practice). Opus 4.8 list $5/$25 per MTok, ~80/20 in/out, no cache
        discount. Heavy tool use: re_rav1d source spelunking, a repro harness, and
        before/after throughput + pixel-identity measurement across rebuilt binaries.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 200000
      duration_minutes: null
      estimated_usd: 1.80
      recorded_at: 2026-07-18
      note: >
        Order-of-magnitude estimate — verify ran interactively in the orchestrator
        main loop (not a metered subagent), so tokens are a self-estimate. Opus 4.8
        list $5/$25 per MTok, ~80/20 in/out, no cache discount. Heavy tool use:
        built pre-change + branch binaries (debug + release), independently
        reproduced the DisjointMut panic on the parent (2/4 @100 procs) and the
        negative control (cap→0 panics return; n_threads=4 still flakes 1/4),
        confirmed 0/12 on the branch, re-derived the pixel golden with a hand-written
        PNG decoder, measured single/parallel throughput, ran the full gate set, and
        triaged the Windows CI stack-overflow regression.
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 190000
      duration_minutes: null
      estimated_usd: 1.70
      recorded_at: 2026-07-18
      note: >
        Round 2 — the Windows stack-overflow fix. Order-of-magnitude estimate; ran
        in the orchestrator main loop (not a metered subagent). Opus 4.8 list $5/$25
        per MTok, ~80/20 in/out, no cache discount. Confirmed re_rav1d Picture is
        Send from source, moved the inline decode onto an 8 MiB scoped thread, added
        a small-caller-stack regression test, drove a negative control (inline decode
        overflows/aborts a 1 MiB thread), re-proved the flake gone (6×100 rayon) and
        pixels byte-identical, ran the full local gate set, and iterated against the
        PR #95 Windows CI leg until green.
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
- **Branch:** `spec-091-avif-threads`
- **PR:** (opened against main)
- **All acceptance criteria met?** Yes.
  - **Reliable repro (pre-change):** two independent vehicles. (1) Full
    `cargo test --features avif` suite → panic via
    `json_shape_consistent_across_verbs` (its `apply --recipe web` subprocess
    panics at `disjoint_mut.rs:837: overlapping DisjointMut`, `rav1d-worker`),
    ~**1 in 4** sequential full-suite runs. (2) Fast A/B: *N* concurrent debug
    single-image (256²) decodes — at **100 procs, a panic in ~2 of 3 trials**.
    Captured overlap: `cdef.rs:207` (ThreadId 10) ∩ `lf_apply.rs:124` (ThreadId 8).
  - **Explicit policy in code:** `AVIF_DECODE_THREADS = 1` via `decode_settings`
    (`src/image/avif.rs`), commented; overrides dav1d's `n_threads = 0` default.
  - **Flake gone:** **0 panics in 6×100-proc trials** post-change (same vehicle
    that flaked ~2/3 pre-change) — the negative control. `n_threads = 4` **still
    flakes** (1/3 at 50 procs), so only `1` (n_tc=1, zero workers) eliminates it.
  - **Throughput measured (release, 14-core):** single-image 6 MP decode
    ~60→~230 ms (**3.8× slower**); serial `convert` batch (28×6 MP) ~1.80→~6.76 s
    (**3.8× slower** — `run_pixel_op` is a serial `for`, not rayon); **parallel
    `web`/`optimize`/`apply` batch (rayon 14, 28×6 MP) ~640→~625 ms — a wash.**
    The regression on the non-rayon paths is consciously traded for correctness
    (DEC-077).
  - **Pixels byte-identical:** pre-change vs after-change binaries decode a 6 MP
    AVIF to identical PNG (`sha256 df4a5981…`, graded by `shasum`); committed test
    pins the 128² fixture's RGBA to a pre-change digest; sips confirms the fixture.
  - **Gates:** `cargo test` default + `--features avif` green across ≥5
    consecutive full-suite runs; clippy (both feature sets), fmt, lean build,
    `just validate`/`bench`/`bench-micro` all pass.
- **New decisions:** DEC-077 (AVIF decode single-thread policy).
- **Deviations:** The spec's premise "batch always par_iter across files (DEC-006)"
  holds only for `web`/`optimize`/`apply`; `convert`/`resize`/`edit` decode via a
  **serial** `run_pixel_op` loop, so the cap regresses those ~3.8×. Chose
  `n_threads = 1` (Option A) over a small cap (Option B — measured to still flake)
  and context-awareness (Option C — the race fires under external load too, so it
  can't be bounded to "inside our batch").
- **Follow-ups:** (1) Report the `cdef.rs`/`lf_apply.rs` overlap upstream to
  re_rav1d — the root cause is a port threading bug, not our usage. (2) Own spec:
  par_iter `run_pixel_op` so `convert`/`resize` reclaim the loss (makes the cap a
  wash everywhere). (3) SPEC-083 BENCHMARKS should land after this (decode numbers
  now measured without oversubscription).
### Build-phase reflection
1. **The spec's own premise was partly wrong, and only measurement caught it.**
   "Batch par_iter across files makes per-file all-cores threading redundant" is
   true for the rayon verbs but false for `convert`/`resize`, whose `run_pixel_op`
   is a serial `for` loop — so there dav1d's threading was the *only* parallelism
   and the cap costs 3.8×. Re-driving the claim (reading the actual batch path,
   measuring both) turned a clean "obvious win" into an honest trade-off.
2. **A small cap felt safe but was a gamble; the source proved why.** `re_rav1d`
   spawns workers only when `n_tc > 1` (`src/lib.rs:257`); any `n_threads > 1`
   keeps the race live — confirmed empirically (`n_threads = 4` still flakes). The
   only value that *structurally* removes the overlap is 1. Grounding the choice in
   the spawn gate, not just a passing run, is what makes it defensible.
3. **The negative control did the real work.** A flake "fixed" is only believable
   if you watched it fail reliably first, then stop — same harness, variable moved.
   Reproducing at ~2/3, then 0/6 on the identical vehicle, plus a byte-identical
   pixel proof against a *separately built* pre-change binary, is the evidence; one
   green suite would have proved nothing on a 1-in-4 flake.

---

## Verify (2026-07-18, Opus 4.8, independent session in the primary checkout)

### Verdict: ⚠ PUNCH LIST — returns to build

The core fix is **verified sound and honest** on macOS/Linux, but it introduces a
**consistent, release-blocking regression on Windows** — a required platform in the
three-OS release matrix (DEC-009). The approach (`n_threads = 1`) is correct; the
*implementation* is incomplete because it moves the whole decode inline onto the
caller's thread without ensuring that thread has an adequate stack.

### P1 — BLOCKER: AVIF decode stack-overflows on Windows (default build)

- **Evidence:** PR #95 CI is RED on `build / test / clippy / fmt (windows-latest)`
  on **both** runs (29624638686, 29624624672), failing identically at
  `optimize_avif_input_writes_webp` (`tests/input_avif.rs:37`):
  `thread 'main' … has overflowed its stack`. The parent cd39f17 was **green** on
  Windows — this is a new, deterministic regression, not a flake.
- **Blast radius is the default build, not just `--features avif`.** The failing
  test is a plain `#[test]` (no feature gate) decoding a **16×16** AVIF; `re_rav1d`
  is a default dep (SPEC-058), so **every AVIF decode on Windows now crashes** —
  even a tiny image — breaking a shipped default input capability (PROJ-009) on a
  supported release OS.
- **Root cause (mechanically clear, from source):** `re_rav1d` spawns its
  `rav1d-worker-*` threads only when `n_tc > 1`, and those workers deliberately use
  Rust's default 2 MiB thread stack — the crate's own comment at `src/lib.rs:258`
  reads *"Don't set stack size like `dav1d` does. See …/rav1d/issues/889."* With
  `n_threads = 1` the decode runs `Rav1dContextTaskType::Single` **inline on the
  calling thread**, whose stack on Windows' main thread is only ~1 MB — smaller than
  the decode's frame needs. dav1d avoids this by setting a large worker stack; rav1d
  relies on the 2 MiB default that only exists on spawned threads, which the cap
  removes. So the same mechanism that kills the flake (no worker threads) is what
  overflows Windows.
- **Why the build missed it:** the "green across ≥5 consecutive full-suite runs"
  claim was validated only on the darwin box; the three-OS matrix (AGENTS §5 /
  DEC-009) was never exercised locally, and CI's Windows leg was not checked before
  the completion table was written. Acceptance criterion #6 (all gates pass) is
  therefore **NOT met** on Windows.
- **Fix direction (for build, not prescriptive):** keep `n_threads = 1` but run the
  inline decode on a thread with an adequate stack — e.g. execute `decode_obus` on a
  `std::thread::Builder::new().stack_size(N)` scratch thread, or scope it via a small
  helper — then re-validate on Windows CI. (Whatever is chosen must keep the flake
  gone and pixels byte-identical, both re-checked below.)

### What IS verified sound (so the fix can be finished with confidence)

- **Flake repro + gone + negative control (independent).** Rebuilt the pre-change
  parent (cd39f17) and branch binaries. Fast vehicle = N concurrent 256² real-photo
  decodes:
  - **Parent (all-cores):** panic in **2 of 4** completed trials @100 procs
    (`overlapping DisjointMut`, `cdef.rs:207` ∩ `cdef_apply.rs:56`, two ThreadIds) —
    an independent reproduction (build cited `cdef.rs:207` ∩ `lf_apply.rs:124`; same
    CDEF-worker race).
  - **Branch (`n_threads=1`):** **0 panics in 12×100 = 1200 process-decodes**, and it
    no longer hangs under load.
  - **Negative control:** branch source with the cap flipped back to `0` (all-cores)
    → panic **returns** (1/3 trials) — proving the harness can still fail and that
    the cap specifically suppresses it.
- **Option B rejection substantiated.** `n_threads = 4` **still flakes** (1/4 trials
  @100 procs, same overlap) — only `1` (n_tc=1, zero workers) removes it. Matches the
  DEC's mechanism claim (`n_tc > 1` ⇒ workers ⇒ overlap).
- **Pixel identity (independently re-derived).** Parent and branch decode both the
  committed 128² fixture and a 256² photo to **byte-identical PNG** (sha256 match).
  The pinned golden `PRE_CHANGE_RGBA_FNV1A = 0x0d2b956b63f0cd85` was re-derived from
  the decoded RGBA using a **hand-written standalone PNG decoder** (not the code under
  test) → exact match. `sips` (independent decoder) confirms the fixture is 128×128.
- **Throughput (release, 14-core).** Single-image 6 MP `convert→png`: parent ~101 ms
  vs branch ~323 ms (**3.2×** slower — corroborates the DEC's ~3.8× single-image
  claim). **Flagship parallel path IS a wash:** 14 concurrent 6 MP decodes parent
  512 ms vs branch **466 ms (0.91×)** — the branch actually *wins* the parallel case
  (parent oversubscribes 14×14). DEC's "wash (~2% faster)" is honest, if conservative.
- **DEC-077 honesty: CONFIRMED.** States plainly it is a correctness + throughput fix,
  NOT memory-safety (provenanceless targets ⇒ wrong pixels); flags the upstream
  re_rav1d CDEF/loop-restoration bug as the root cause and a workaround, not a repair;
  records the `run_pixel_op` par_iter follow-up. Does not oversell.
- **Fixture provenance: license-clean.** `tests/fixtures/avif/photo_128.avif` is a
  128² crop of `bench/corpus/photo_forest_cc0.jpg` (Wikimedia Commons / DimiTalen,
  CC0-1.0, own work, metadata-stripped — verified against the Commons API in SPEC-088)
  re-encoded to AVIF; carries only an sRGB profile (no camera/GPS/artist/copyright).
  Same standard as SPEC-088's corpus.
- **Gates (macOS):** `cargo fmt --check`, `clippy` (default + avif), `cargo build
  --no-default-features`, `cargo test` (default, 419+), `cargo test --features avif`
  ×3 clean (761 tests, 0 panics; the 3 new SPEC-091 tests run), `just validate`,
  `just bench`, `just bench-micro` — all green.

### P3 — minor (address alongside the Windows fix)

- The DEC's Consequences table and the completion table say the flagship parallel
  paths are a "wash (~2% faster)"; my measurement has the branch **9% faster** there.
  Not wrong (still a wash-or-better), but the specific "~2%" undersells it slightly —
  fine to leave, noted for accuracy.
- Neither the spec, DEC-077, nor the completion table mentions Windows / the three-OS
  matrix at all — the platform dimension of the acceptance repro was never considered.
  The re-worked DEC should record the Windows stack behavior and how the fix handles it.

---

## Build Completion — Round 2 (Windows stack-overflow fix, PR #95)

Addresses the verify PUNCH LIST: the `n_threads = 1` policy is kept (it is the only
setting that structurally closes the race), but the inline decode is moved off the
caller's OS-defined stack onto one with adequate headroom.

- **Root cause (confirmed):** `n_threads = 1` ⇒ zero `rav1d-worker-*` threads ⇒ the
  decode runs **inline on the caller's thread**. dav1d's decode has a large fixed
  stack frame (size-independent); macOS/Linux main threads (~8 MiB) absorb it, but
  the **Windows main thread (~1 MiB)** overflows — even a 16×16 AVIF did, on both
  windows-latest legs of PR #95. `re_rav1d`'s own workers avoid this by keeping
  Rust's 2 MiB default stack (`src/lib.rs:258`); running inline forfeits it.
- **Fix:** `decode_obus` runs the inline decode on a **scoped thread with an 8 MiB
  stack** (`std::thread::scope` + `Builder::stack_size(AVIF_DECODE_STACK_SIZE)
  .spawn_scoped`), returning the decoded `Picture` across the `join`. `re_rav1d`'s
  `Picture` **is `Send`** (`Arc<InnerPicture>`; `static_assertions::assert_impl_all!
  (Picture: Send, Sync)`), so no decode-to-owned-buffer copy is needed. The scope
  lets the closure borrow `obus`/`limits` without cloning.
- **Stack size = 8 MiB:** matches the macOS/Linux main-thread stack that never
  overflowed and is 4× dav1d's 2 MiB worker default. `stack_size` only reserves
  address space (committed lazily), so the generous figure is ~free. Chosen over the
  untested 2 MiB minimum deliberately; Windows CI green is the evidence.
- **Composition with rayon (DEC-006):** each rayon worker spawns one decode thread
  then **blocks on `join`** — one extra thread per concurrent decode, CPU work done
  by the decode thread while the worker sleeps. No all-cores pool, no re-introduced
  oversubscription; alpha's second decode runs after the primary (never two live).
- **Cost:** one spawn per decode, ~tens of µs vs ms-scale decode — negligible;
  `bench-micro`/`bench` unchanged within noise.
- **Picture was Send?** **Yes** — so the frame is returned, not re-materialized.
- **Flake stays gone (re-proven under the refactor):** the concurrent-decode vehicle
  (100 debug decodes via rayon, each now spawning its own decode thread) → **0
  panics in 6 trials**; full `cargo test --features avif` (762 tests) green; targeted
  `input_avif` ×5 clean. A negative control (decode inline, no re-spawn) reproduced
  the overflow-abort on macOS from a 1 MiB thread, proving the guard bites.
- **Pixels byte-identical:** the committed golden `avif_decode_pixels_unchanged_by_
  thread_policy` still passes (digest unchanged) — the refactor changed *where* the
  decode runs, not the pixels.
- **New regression test:** `avif_decode_survives_a_small_caller_stack` (ungated,
  `input_avif.rs`) decodes from a deliberately ~1 MiB caller thread (the Windows
  main-thread size) using the default 16×16 fixture — passes only because the decode
  re-spawns onto its own stack. Guards the default `cargo test` too, like the
  `optimize_avif_input_writes_webp` test that first caught the overflow on Windows.
- **Local gates (macOS):** `cargo test` default + `--features avif` (762), clippy
  (both feature sets), `cargo fmt --check`, `cargo build --no-default-features`,
  `just validate` / `bench` / `bench-micro` — all green.
- **The gate that was missed in round 1 — Windows CI:** PR #95 `build / test /
  clippy / fmt (windows-latest)` is the acceptance gate (the three-OS matrix,
  DEC-009 — not the darwin box). This section is finalized only after pushing and
  watching `gh pr checks 95` show the windows-latest leg **green** (ubuntu + macos
  green too). ‹CI RESULT recorded on confirmation below.›
- **DEC-077 updated:** new section "Windows: the inline decode needs its own stack"
  records the mechanism, the 8 MiB choice, the rayon composition, and the round-1
  gate gap (darwin-only validation of a change to *where* work runs).

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
