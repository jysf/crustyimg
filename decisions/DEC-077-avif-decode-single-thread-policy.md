---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-077
  type: decision
  confidence: 0.9
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

created_at: 2026-07-17
supersedes: null
superseded_by: null

# Extends DEC-034's precedent (deliberate decode Settings); composes with DEC-006
# (rayon batch). Governs the AVIF decode Settings.
affected_scope:
  - src/image/avif.rs

tags:
  - avif
  - decode
  - threads
  - correctness
  - concurrency
  - re_rav1d
---

# DEC-077: AVIF decode runs single-threaded (cap dav1d's thread pool)

## Decision

`src/image/avif.rs`'s decode `Settings` set `n_threads = 1` explicitly
(`AVIF_DECODE_THREADS`), instead of inheriting dav1d's default `n_threads = 0`
(= auto = all logical cores). Every AVIF still frame is decoded inline, with no
`rav1d-worker-*` pool.

Because `n_threads = 1` removes those worker threads, the inline decode is **not**
run on the caller's thread (whose stack size the OS fixes — only ~1 MiB on the
Windows main thread). `decode_obus` re-spawns it onto a thread it creates with an
8 MiB stack (`AVIF_DECODE_STACK_SIZE`) and returns the decoded frame across the
`join` (`re_rav1d`'s `Picture` is `Send`, `Arc`-backed). See "Windows: the inline
decode needs its own stack" below — this is the round-2 addition (PR #95).

**This is a correctness + throughput decision, NOT a memory-safety fix.** See
"Not a security fix" below — say this plainly wherever it is cited.

## Context

We decode exactly one AV1 **still** frame per call (`decode_obus`). dav1d's
`n_threads = 0` default resolves to `rav1d_num_logical_processors()` tile
contexts (`re_rav1d-0.1.3/src/lib.rs:150`), so **every decoder instance spawned
its own ~N-core pool**. On a 14-core box a batch that fans out across files
(`web`/`optimize`/`apply`, rayon `par_iter`, DEC-006) reached 14 rayon workers ×
~14 dav1d threads ≈ 196 threads — textbook oversubscription.

Under that contention `re_rav1d`'s **debug-mode** `DisjointMut` overlap checker
(`disjoint_mut.rs:837`, `#[cfg(debug_assertions)]`) panics: a genuine cross-thread
overlap between the decoder's own CDEF and loop-restoration workers. A captured
instance:

```
overlapping DisjointMut:
 current:    & _[4158..4170] on ThreadId(10) at re_rav1d/src/cdef.rs:207:28
existing: &mut _[3904..4160] on ThreadId(8)  at re_rav1d/src/lf_apply.rs:124:39
```

It surfaced in CI on PR #94 (a clean, unrelated change) via
`json_shape_consistent_across_verbs`, whose spawned `apply --recipe web`
subprocess decodes the AVIF winner — failing a **required** `avif` check and
clearing only on a same-commit rerun, so it blocked the merge pipeline, not just
local runs.

### The mechanism (re-verified from source, not the spec prose)

- `Settings::new()` → `dav1d_default_settings` → `Rav1dSettings::default()` sets
  `n_threads: 0` (`re_rav1d-0.1.3/src/lib.rs:115`); `get_num_threads` maps `0` to
  all logical processors (`:150`).
- `re_rav1d` spawns `rav1d-worker-*` threads **only when the tile-context count
  `n_tc > 1`** (`src/lib.rs:257`); `n_tc == 1` uses `Rav1dContextTaskType::Single`
  — the decode runs inline, no thread spawned. Since `n_tc` derives directly from
  `n_threads`, `set_n_threads(1)` ⇒ `n_tc == 1` ⇒ **zero worker threads ⇒ no
  cross-thread access ⇒ the overlap is structurally impossible.**
- `Settings::set_n_threads(u32)` exists and is the only lever
  (`re_rav1d-0.1.3/lib.rs:257`); we already set `frame_size_limit` here (DEC-034),
  so this extends that "set decode Settings deliberately" precedent to threads.

### Repro (pre-change) and the negative control

Two independent reproductions on the pre-change tip (cd39f17):

1. **CI-faithful:** full `cargo test --features avif` suite → panic via
   `json_shape_consistent_across_verbs`. **~1 in 4** sequential full-suite runs
   (matches the design-time 2/5 and 1/3).
2. **Fast A/B vehicle:** *N* concurrent debug single-image (256×256) decodes, each
   spawning its all-cores pool (the same oversubscription). At **100 procs: a
   `DisjointMut` panic in ~2 of 3 trials**; at 60 procs, ~1 in 5.

After the cap, the **same** 100-proc vehicle produced **0 panics in 6 trials**,
and the runs stopped hanging (no worker contention). The negative control — the
harness demonstrably *can* show the panic (it did, pre-change) and stopped once
the variable moved — is what makes "the flake is gone" believable rather than a
lucky green.

## Alternatives Considered

- **Option B — a small cap (`n_threads` = 2–4).** Keeps some tile-threading for
  single-image decode. **Rejected on evidence:** `n_threads = 4` **still flakes**
  (1 panic in 3 trials at only 50 procs). Any `n_threads > 1` spawns workers
  (`n_tc > 1`, `src/lib.rs:257`) that can overlap; only `1` removes them. A small
  cap is a gamble that narrows the window without closing it.

- **Option C — context-aware (all-cores when not inside a rayon batch, 1 when
  inside).** Would preserve single-image speed. **Rejected:** the race also fires
  under *external* CPU load (my fast repro is many separate processes each
  decoding a single image on its own main thread — none is inside our rayon pool,
  yet they panic). The CI failure was itself a **single-input** `apply` subprocess.
  So "am I inside our batch?" does not bound the contention, and a policy that
  leaves all-cores decoding live for single inputs would leave the race — and, in
  release, the wrong pixels — live. It would also fail the acceptance repro.

- **Option A (chosen) — `n_threads = 1`, unconditional.** The only value that
  makes the overlap structurally impossible, at every call site, under any load.

## Consequences

Measured on release, 14-core, corpus = an upscaled CC0 photo (`bench/corpus`):

| Scenario | all-cores (before) | 1-thread (after) | effect |
|---|---|---|---|
| Single-image decode, 6 MP | ~60 ms | ~230 ms | **~3.8× slower** |
| Serial batch — `convert`/`resize`, 28×6 MP | ~1.80 s | ~6.76 s | **~3.8× slower**, cores idle |
| **Parallel batch — `web`/`optimize`/`apply`, 28×6 MP (rayon 14, decode-only)** | ~640 ms | ~625 ms | **wash (~2% faster)** |

- **Positive — the flagship paths are a wash and correct.** The verbs that fan
  out across files with rayon (DEC-006) — `web`, `optimize`, `apply`, the ones the
  CI flake lived on — already fill every core with independent single-threaded
  decodes, so removing the per-file pool costs nothing and ends the
  oversubscription (and the release-mode data race that silently corrupts pixels
  → a wrong SSIMULACRA2 score, the honesty headline we sell). AVIF decode is now
  **deterministic** and its published `--timing` numbers (SPEC-083) are trustworthy.

- **Negative — a real regression on the non-rayon paths, consciously traded for
  correctness.** A **single** AVIF decode, and a **serial** batch, lose dav1d's
  intra-frame (tile/postfilter) threading and run ~3.8× slower. This bites two
  cases: (a) decoding one large AVIF (`convert one.avif …`); (b) decoding *many*
  AVIFs through `convert`/`resize`/`edit`, whose `run_pixel_op` multi-input path
  is a **serial `for` loop** (`src/cli/mod.rs:3229`), not a rayon `par_iter`, and
  ignores `--jobs`. AVIF is primarily an *output* format, so decoding batches of
  AVIF *inputs* is a niche path; and for `web`/`optimize` on a single input the
  ~170 ms extra decode is dwarfed by encode (~2.8 s for 6 MP), ≈5% of total.
  **Follow-up (own spec): par_iter `run_pixel_op`** — that would let `convert`/
  `resize` reclaim the loss the same way `web`/`optimize` already avoid it, making
  the cap a wash everywhere.

- **Neutral — pixels are byte-identical.** dav1d is a conformant decoder; output
  does not depend on thread count. Proven: the pre-change (all-cores) and
  after-change (1-thread) binaries decode a 6 MP AVIF to **byte-identical** PNG
  (`sha256 df4a5981…`, graded by `shasum`, independent of our reader), and the
  committed test `avif_decode_pixels_unchanged_by_thread_policy` pins a 128×128
  fixture's decoded RGBA to a digest captured on the pre-change binary.

### Not a security fix

`re_rav1d`'s own safety contract (`disjoint_mut.rs:78–82`) argues all
`AsMutPtr::Target`s are **provenanceless** (pixel data; no pointers/provenance),
so a release-mode data race from the missing runtime check *"would only result in
wrong results, and cannot result in memory safety."* It is still UB by the Rust
abstract machine, but the realistic consequence is **wrong pixels**, not an
exploitable hole. This decision is about correctness (no wrong pixels) and
throughput (no oversubscription), not memory safety. Do not oversell it.

### Windows: the inline decode needs its own stack (round 2, PR #95)

Setting `n_threads = 1` moves *where* the decode runs: with no `rav1d-worker-*`
threads, `re_rav1d` decodes inline on **the caller's** thread. dav1d's decode has
a large *fixed* stack frame (on-stack tile/context structures) that is independent
of image size, and `re_rav1d` deliberately leaves its workers on Rust's default
2 MiB thread stack rather than shrinking it (`src/lib.rs:258`, "Don't set stack
size like dav1d does", upstream rav1d#889). Running inline forfeits that 2 MiB
headroom and inherits whatever stack the OS gave the caller:

- macOS / Linux **main** threads are ~8 MiB → the inline decode never overflowed,
  so round 1 passed local (darwin) and the ubuntu CI leg.
- The **Windows** main thread is only ~1 MiB. On PR #95, `optimize_avif_input_
  writes_webp` (an ungated `#[test]` decoding a **16×16** AVIF) hit `thread 'main'
  has overflowed its stack` on **both** windows-latest runs — the parent commit was
  green. Even 16×16 overflows, confirming the frame is size-independent and this is
  structural, not a decompression concern. (`frame_size_limit`/DEC-034 bounds
  *heap* plane allocation — a separate axis from stack.)

The lesson: round 1's "≥5 green runs" gate was validated on the darwin box only;
the required Windows CI leg (DEC-009) was never checked. A change to *where* work
runs shifts per-platform stack assumptions, so it must be validated on the whole
required OS matrix, not one box.

**Fix.** `decode_obus` never runs the inline decode on the caller's (OS-defined,
possibly ~1 MiB) stack. It uses a scoped thread —
`std::thread::scope` + `Builder::stack_size(AVIF_DECODE_STACK_SIZE).spawn_scoped` —
so the decode runs on a stack we control and the closure can borrow `obus`/`limits`
without cloning; the frame (`Picture`, `Send`/`Arc`-backed) is returned across the
`join`.

**Stack size = 8 MiB.** It matches the macOS/Linux main-thread stack that has
always decoded these frames without overflow, and is 4× dav1d's own 2 MiB worker
default — ample headroom above the frame that overflowed ~1 MiB. `stack_size`
only *reserves* address space (committed lazily by the OS), so the generous figure
costs effectively nothing. Windows CI green on PR #95 is the evidence that 8 MiB
clears the frame; we chose the known-good-environment figure over the untested
2 MiB minimum deliberately.

**Composition with rayon (DEC-006) — no re-oversubscription.** Inside a batch each
rayon worker calls `decode_obus`, which spawns **one** decode thread and then
**blocks on `join`**. So per concurrent decode there is exactly one extra thread,
and the CPU-bound work is done by the decode thread while its rayon worker sleeps
on the join — no two CPU-bound threads per file, no all-cores pool. Net CPU-active
threads stay ≈ core count, the same as before. The alpha plane's second
`decode_obus` runs after the primary, so a file never has two decode threads live
at once.

**Cost.** One thread spawn per decode is ~tens of µs against a ms-scale decode —
negligible; `bench-micro` and `bench` are unchanged within noise.

**Regression guard.** `avif_decode_survives_a_small_caller_stack` (ungated,
`input_avif.rs`) runs the decode from a deliberately ~1 MiB caller thread — the
Windows main-thread size — using the default 16×16 fixture. It passes only because
the decode re-spawns onto its own 8 MiB stack; a negative control (calling the
inline decode directly, no re-spawn) reproduced the overflow/abort on macOS,
proving the guard bites.

### Root cause is upstream

The overlap is between `re_rav1d`'s *own* CDEF/loop-restoration workers — a
threading bug in the port, independent of how we drive it. `n_threads = 1` is a
**workaround**, not a repair: it sidesteps the buggy threaded path. To be reported
upstream (re_rav1d) with the captured `cdef.rs:207` / `lf_apply.rs:124` overlap.
When upstream fixes it, revisit whether multi-threaded decode can return.

## Validation

- **Flake gone:** 0 `DisjointMut` panics over 6×100-proc trials post-change (same
  vehicle that flaked ~2/3 pre-change); `n_threads = 4` still flakes, `1` does not.
- **Pixels unchanged:** byte-identical before/after on 6 MP (`sha256`), plus the
  committed golden test on the 128×128 fixture; sips (an independent system
  decoder) confirms the fixture decodes to 128×128.
- **Windows overflow fixed (round 2):** the inline decode runs on an 8 MiB spawned
  stack; PR #95's windows-latest leg is **green** (with ubuntu + macos green;
  commit `d87d389`, 27 checks passed / 0 failed) — the three-OS matrix (DEC-009),
  not a single box, is the gate.
- **Tests:** `avif_decode_thread_policy_is_explicit`,
  `avif_batch_decode_does_not_oversubscribe` (unit, `avif.rs`),
  `avif_decode_pixels_unchanged_by_thread_policy` +
  `avif_decode_survives_a_small_caller_stack` (integration, `input_avif.rs`; the
  latter decodes from a ~1 MiB caller thread, mirroring the Windows overflow).
- **Gate:** full PR #95 CI matrix green (ubuntu, macos, **windows**); `cargo test`
  (default and `--features avif`) green across ≥5 consecutive full-suite runs.

**Revisit if:** upstream re_rav1d fixes the CDEF/loop-filter overlap (then
multi-threaded decode can return), or `run_pixel_op` is parallelized (then measure
whether a bounded per-file cap beats 1 thread on the now-parallel serial verbs).

## References

- Related specs: SPEC-091 (this decision), SPEC-058/DEC-058 (native AVIF decode),
  SPEC-088 (`--timing`, the measurement lever), SPEC-083 (BENCHMARKS — lands after
  this so its decode numbers are not measured under oversubscription).
- Related decisions: **DEC-006** (sync core + rayon batch — the concurrency model
  this composes with; parallelism belongs at the file level, not N pools inside
  one batch), **DEC-034** (`frame_size_limit` — the precedent for setting decode
  `Settings` deliberately, extended here to threads), **DEC-053** (the `re_rav1d`
  no-asm pure-Rust choice), DEC-064 (re_rav1d is native-only; not in the wasm build).
- External: `re_rav1d-0.1.3` `src/lib.rs:115/150/257`, `src/disjoint_mut.rs:78–82,837`,
  `lib.rs:257` (`set_n_threads`).
