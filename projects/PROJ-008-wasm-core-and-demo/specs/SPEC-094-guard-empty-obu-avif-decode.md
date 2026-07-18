---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-094
  type: bug
  cycle: design
  blocked: false
  priority: medium
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-18
references:
  decisions: [DEC-062, DEC-077, DEC-034]
  constraints: [pure-rust-codecs-default, every-public-fn-tested, test-before-implementation, untrusted-input-is-hostile]
  related_specs: [SPEC-091, SPEC-069, SPEC-058]
value_link: >
  Close an uncatchable-abort hole in AVIF decode. An EMPTY OBU stream reaches re_rav1d's debug-only
  `debug_abort()`, which bypasses both our `catch_unwind` and SPEC-091's scoped-thread `join` (abort ≠
  unwind) — so in a DEBUG build (where the SPEC-069 decoder fuzz gate + all tests run) a crafted AVIF
  crashes the process, violating the DEC-062 "typed error, never a panic/abort on hostile input" contract.
  The alpha decode path is unguarded.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-094: guard the empty-OBU `debug_abort()` in AVIF decode

## Context

Filed by SPEC-091's round-2 verify (2026-07-18) as an out-of-scope P3 observation, and pinned in the
SPEC-091 follow-ups queue. It is **pre-existing** (predates SPEC-091's thread-policy change; the thread
move can't affect abort semantics).

**The hole.** An **empty** OBU byte-slice reaches `re_rav1d`'s debug-only `debug_abort()` — an `abort()`
under `cfg!(debug_assertions)`. `abort()` is **not an unwind**, so it bypasses BOTH:
- `decode_avif`'s `std::panic::catch_unwind` (`src/image/avif.rs:127`), and
- SPEC-091's scoped-thread `join` (`decode_obus`, ~323) — `join` propagates panics, not aborts.

So in a **debug** build the process dies. That matters because the **SPEC-069 decoder fuzz gate and the
entire test suite run in debug** (`debug_assertions` on) — a crafted AVIF whose alpha item is an empty
OBU stream would crash the fuzzer/tests, and it violates the DEC-062 contract (decode of hostile input
returns a typed `ImageError`, never a panic/abort). In release the check is compiled out (no abort), but
feeding an empty stream to the decoder is still wrong.

**Reachability (verify's note — CONFIRM FIRST, do not assume).** The **primary** decode path is guarded
by a `parse_obu` metadata pre-check. The **alpha** path is NOT: `decode_avif_inner` (`avif.rs:168-169`)
does `if let Some(alpha) = &parsed.alpha_item { decode_obus(alpha, limits)? }`, and `avif-parse` can
reportedly set `alpha_item = Some(<empty>)`. Whether a real crafted AVIF actually yields an empty alpha
slice was **plausible but unconfirmed** at SPEC-091 verify. This spec's first job is to confirm it.

## Goal

Ensure **no** empty OBU stream can reach `re_rav1d` from any decode path, so hostile input yields a typed
`ImageError`, never an abort — in debug (fuzz/tests) or release. Confirm the alpha-path reachability
first; guard at the shared chokepoint so every caller is covered.

## Inputs — files to read

- `src/image/avif.rs` — `decode_avif` + `catch_unwind` (~123–135), `decode_avif_inner` (~138): the
  **primary** call (~159) and the **alpha** call (~168–169); `decode_obus` (~323, the scoped-thread
  chokepoint BOTH paths flow through) → `decode_obus_inline` (~342) → `send_data`. The `parse_obu`
  metadata pre-check that guards the primary path (find it; confirm it does NOT cover alpha).
- The vendored `re_rav1d-0.1.3` source: `debug_abort()` / `validate_input!` sites and their
  `cfg!(debug_assertions)` gate — confirm empty-input is what trips it (grep `debug_abort`, `abort`).
- `DEC-062` (typed-error / no-panic-no-abort decode contract), `DEC-077` (the scoped-thread decode),
  `fuzz/` (SPEC-069 — the debug fuzz targets this protects).

## Outputs

- **`src/image/avif.rs`** — an `if obus.is_empty() { return Err(ImageError::Decode(...)) }` guard **at
  `decode_obus`** (the single chokepoint, BEFORE the scoped-thread spawn / `send_data`), so the primary
  AND alpha paths are both covered by one guard. If any *other* code path constructs a `re_rav1d` decoder
  or calls `send_data` directly (grep to be sure), guard it too — the contract is "no empty stream reaches
  the decoder from ANY caller" ([[image-extensions-expose-every-decode-caller]]).
- **Tests** proving the guard and its reachability (below).
- **No DEC expected** — a defensive guard that adds no capability and changes no public contract; note it
  in the spec + the STAGE-030 stage doc. (Emit one only if the fix turns out to change observable
  behavior beyond "empty → typed Err".)

## Acceptance Criteria

- [ ] **Reachability confirmed FIRST:** a crafted AVIF whose alpha item is an empty OBU stream is shown to
      reach `decode_obus(alpha, …)` with an empty slice on the **pre-fix** code, and to **abort** in a
      debug build. If it genuinely cannot be constructed (avif-parse rejects it earlier), **say so
      explicitly** and reclassify the guard as belt-and-suspenders defense rather than a live-bug fix —
      do not claim a fix for an unreachable path.
- [ ] After the guard, an empty OBU stream (primary OR alpha) returns a **typed `ImageError::Decode`**,
      never an abort/panic — verified in a **debug** build (where the abort lived).
- [ ] The **SPEC-069 fuzz targets** no longer abort on an empty-OBU input (run the relevant target in
      debug on the crafted/seed input; confirm typed-error, process survives).
- [ ] Every path that can hand bytes to `re_rav1d` is guarded (grep `send_data`/`decode_obus`/decoder
      construction; cite the hit count — [[mechanical-sweeps-need-a-mechanical-check]]).
- [ ] Valid AVIFs (with and without a real alpha channel) still decode **pixel-identically** to before —
      the guard only rejects empty streams. Prove against the pre-fix binary.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
      `cargo build --no-default-features`, `just validate` pass.

## Failing Tests (written at design)

- **`src/image/avif.rs` / integration**
  - `empty_alpha_obu_is_typed_error_not_abort` — the crux: a crafted AVIF with an empty alpha OBU decodes
    to a typed `Err` (post-fix). **Must abort/fail on the pre-fix code** — that's the proof the guard
    bites and the reachability is real.
  - `empty_primary_obu_is_typed_error` — an empty primary OBU slice → typed `Err` (belt-and-suspenders,
    even though `parse_obu` guards it upstream).
  - `valid_avif_with_alpha_unchanged` — a normal alpha-bearing AVIF decodes pixel-identically (the guard
    doesn't reject real streams).

## Implementation Context

### Decisions that apply
- `DEC-062` — the decode contract this restores: hostile input → typed `ImageError`, **never** a
  panic/abort. An `abort()` that dodges `catch_unwind` is exactly the failure this forbids.
- `DEC-077` — the scoped-thread decode; the guard goes BEFORE the spawn (no point spawning a thread to
  abort). `DEC-034` — `frame_size_limit` is the precedent for a pre-decode guard on `Settings`/input.

### Constraints
- `untrusted-input-is-hostile` / `test-before-implementation` — the fuzz gate is the point; a guard with
  no debug-mode abort test is unproven ([[a-thread-boundary-does-not-catch-abort]]).

### Out of scope (this spec)
- The re_rav1d **DisjointMut threading race** (SPEC-091's other follow-up — a separate upstream bug).
- Any change to valid-stream decode behavior, the thread policy, or the parse layer beyond the guard.

## Notes for the Implementer
- **Confirm reachability before writing the fix** — the whole risk is "fixing" an unreachable path and
  declaring a bug closed that never existed ([[a-plausible-test-result-is-not-a-checked-one]]). Build the
  empty-alpha AVIF, drive the pre-fix debug binary, watch it abort.
- **An abort is not an unwind** — `catch_unwind` and the scoped `join` do NOT catch it; the ONLY fix is
  to stop the empty stream *before* it reaches `re_rav1d` ([[a-thread-boundary-does-not-catch-abort]]).
- **Guard the chokepoint, not just the path you reproduced** — put it in `decode_obus` so primary + alpha
  + any future caller are all covered; grep for other decoder-construction sites.
- Small fix; the *reachability proof + the debug-mode abort test* are the real deliverable.

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
