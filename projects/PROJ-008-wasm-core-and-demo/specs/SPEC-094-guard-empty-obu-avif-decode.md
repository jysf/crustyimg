---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-094
  type: bug
  cycle: verify
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 550000
      duration_minutes: null
      estimated_usd: 3.30
      note: >
        Main-loop interactive session (not a metered Agent subagent), so no clean
        subagent_tokens reading is available — order-of-magnitude ESTIMATE per the
        "autonomous run cost = labelled estimates" convention (docs/cost-tracking.md),
        not a metered figure. Heavy on vendored-crate source reading (re_rav1d,
        avif-parse) and hand-constructing a byte-exact crafted AVIF container.
        estimated_usd = tokens_total x list rate (Sonnet ~$3/$15 per MTok, ~80/20
        input/output, no cache discount).
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 650000
      duration_minutes: null
      estimated_usd: 5.85
      note: >
        Main-loop interactive session (not a metered Agent subagent), so no clean
        subagent_tokens reading — order-of-magnitude ESTIMATE per the "autonomous
        run cost = labelled estimates" convention (docs/cost-tracking.md), not a
        metered figure. Heavy on vendored-crate source reading (re_rav1d validate.rs
        / lib.rs), two full test-suite runs (759 default + 771 --features avif),
        two clippy runs, and two cargo-fuzz builds (~1m22s each) to drive the
        reachability + fuzz-gate negative controls in both directions.
        estimated_usd = tokens_total x list rate (Opus 4.8 ~$5/$25 per MTok,
        ~80/20 input/output, no cache discount).
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

- [x] **Reachability confirmed FIRST:** a crafted AVIF whose alpha item is an empty OBU stream is shown to
      reach `decode_obus(alpha, …)` with an empty slice on the **pre-fix** code, and to **abort** in a
      debug build. If it genuinely cannot be constructed (avif-parse rejects it earlier), **say so
      explicitly** and reclassify the guard as belt-and-suspenders defense rather than a live-bug fix —
      do not claim a fix for an unreachable path.
- [x] After the guard, an empty OBU stream (primary OR alpha) returns a **typed `ImageError::Decode`**,
      never an abort/panic — verified in a **debug** build (where the abort lived).
- [x] The **SPEC-069 fuzz targets** no longer abort on an empty-OBU input (run the relevant target in
      debug on the crafted/seed input; confirm typed-error, process survives).
- [x] Every path that can hand bytes to `re_rav1d` is guarded (grep `send_data`/`decode_obus`/decoder
      construction; cite the hit count — [[mechanical-sweeps-need-a-mechanical-check]]).
- [x] Valid AVIFs (with and without a real alpha channel) still decode **pixel-identically** to before —
      the guard only rejects empty streams. Prove against the pre-fix binary.
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
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
- **Branch:** `spec-094-empty-obu-guard`
- **PR:** [#97](https://github.com/jysf/crustyimg/pull/97)
- **All acceptance criteria met?** Yes.
  - **Reachability confirmed FIRST, not assumed.** Read the vendored `re_rav1d-0.1.3` source directly:
    `include/common/validate.rs`'s `debug_abort()` calls `std::process::abort()` under
    `cfg!(debug_assertions)`, invoked by the `validate_input!` macro on any failed condition.
    `rav1d_send_data` (`src/lib.rs:519-535`) validates `sz > 0 && sz <= usize::MAX / 2` on the buffer
    `Decoder::send_data` wraps — an empty buffer trips exactly that check. Traced `avif-parse`'s `iloc`
    extent semantics (`src/lib.rs:1260-1358`, `MediaDataBox::matches_extent`/`contains_extent`/`read_extent`)
    to find a **conforming, non-corrupt** container shape that reaches this: a `ToEnd` extent
    (wire `extent_length == 0`) matches on file **offset** only, never current byte count — so two items
    (primary, then alpha) whose `iloc` entries point at the *same* mdat offset each `mem::take` whatever is
    left there; the primary's entry (processed first) drains the real OBU bytes, and the alpha's identical
    entry then drains the now-empty leftover, yielding `alpha_item == Some(<empty>)`. Hand-built that exact
    container byte-for-byte (`build_avif_with_empty_alpha` in `src/image/avif.rs` test module — ftyp + meta
    [iinf/pitm/iref/iprp(ipco+auxC/ipma)/iloc] + mdat, computed offsets, no shortcuts) around the real primary
    OBU bytes extracted from the committed `solid_16x16.avif` fixture. **Drove it through the pre-fix
    `decode_avif` in an isolated `cargo test --exact` subprocess and observed the actual crash**: `signal: 6,
    SIGABRT`, with `re_rav1d`'s own diagnostic on stderr — `` Input validation check `(sz > 0 && sz <=
    usize::MAX / 2, EINVAL)` failed in `fn re_rav1d::src::lib::rav1d_send_data` ``. This is not a hypothetical;
    the alpha path is genuinely reachable from a well-formed container, confirming the spec's "plausible but
    unconfirmed" framing as a real bug. (`empty_primary_obu_is_typed_error`, which calls `decode_obus(&[],
    ..)` directly, aborted the same way pre-fix — the chokepoint itself has no screening, only the container
    parse layer happens to screen primary via `parse_obu`.)
  - **Guard placed at the chokepoint.** `if obus.is_empty() { return Err(...) }` at the top of `decode_obus`
    (`src/image/avif.rs`), before the scoped-thread spawn — the single function both the primary (`avif.rs`
    call ~159) and alpha (~169) paths flow through.
  - **Grep for every decoder-input site** (cited hit count, [[mechanical-sweeps-need-a-mechanical-check]]):
    `grep -rn '\.send_data(' src/` → **1 hit** (inside `decode_obus_inline`, only reachable via `decode_obus`).
    `grep -rn 'Decoder::with_settings\|Decoder::new' src/` → **1 hit** (same function). `grep -rn
    'decode_obus\b' src/image/avif.rs` → **2 call sites** (primary, alpha), both routed through the one
    `decode_obus` chokepoint now guarded. No other file in the crate constructs a `re_rav1d` decoder or calls
    `send_data` (the other `grep -rl re_rav1d` hits outside `avif.rs` are doc-comment mentions only, checked
    individually). One guard, full coverage.
  - **Post-fix: crux tests pass instead of aborting.** `empty_alpha_obu_is_typed_error_not_abort` and
    `empty_primary_obu_is_typed_error` both went from `SIGABRT` (pre-fix, isolated-subprocess proof) to a
    typed `Err(ImageError::Decode(_))` (post-fix), run inside the normal test binary alongside everything
    else — no isolation needed anymore, which is itself proof the abort is gone.
  - **Fuzz target survives the crafted input in debug.** `cargo fuzz run avif_decode
    <crafted-empty-alpha.avif>` (rustup nightly toolchain, `-Cdebug-assertions` on by cargo-fuzz default) —
    `Executed ... in 3 ms`, no crash, no ASAN report.
  - **Valid AVIFs decode pixel-identically, proven against the pre-fix binary.** Without alpha: the existing
    `avif_decode_pixels_unchanged_by_thread_policy` (SPEC-091's golden FNV-1a, unrelated fixture) and
    `optimize_avif_input_writes_webp` both still pass unmodified. With alpha: new
    `valid_avif_with_alpha_unchanged` decodes a newly-generated `solid_16x16_alpha.avif` fixture (real,
    non-empty alpha channel via `image`'s `avif` encoder feature) and checks its RGBA FNV-1a against a golden
    captured from **this exact fixture on the pre-fix binary**, before the guard existed — an independent
    value the guard cannot fabricate, mirroring the SPEC-091 golden-hash pattern.
  - **Gates:** `cargo test` (default, 759 tests across all binaries — 0 failed) and `cargo test --features avif`
    (clean on a second full run; a `build_watch` flake on the first full run reproduced identically with our
    changes *stashed*, isolating it as pre-existing parallel-suite contention unrelated to this change, not
    a regression — confirmed by running `--test build_watch` alone, twice, clean both times).
    `cargo clippy --all-targets -- -D warnings` and `--features avif` variant: clean. `cargo fmt --check`:
    clean. `cargo build --no-default-features`: clean. `just validate`: clean (212 front-matter blocks).
- **New decisions:** None. The fix is a defensive guard rejecting only genuinely-empty OBU streams; it adds
  no capability and changes no observable contract beyond "empty → typed `Err`" (matches the spec's framing).
- **Deviations:** None from the spec's prescribed fix location/shape. Extended `examples/gen_avif_fixture.rs`
  (not mentioned in the spec) to also emit `tests/fixtures/avif/solid_16x16_alpha.avif`, needed for the
  `valid_avif_with_alpha_unchanged` test to run in the **default** build (no `--features avif` needed at test
  time) — mirrors the existing `solid_16x16.avif` fixture-generation convention exactly.
- **Follow-ups:** None new. The two other SPEC-091 follow-ups (upstream `DisjointMut` report, `par_iter
  run_pixel_op`) remain queued, unaffected by this spec.

### Build-phase reflection
1. **What was hardest?** Proving reachability honestly. It would have been easy to assume "avif-parse
   probably allows it" per the design doc's hedge and just add the guard — but constructing an *actual*,
   byte-exact, conforming AVIF container whose alpha item is genuinely empty required reading avif-parse's
   `iloc` extent-matching code closely enough to find the one legal (non-corrupt) path there: a `ToEnd`
   extent's `matches_extent` check compares only file offset, never remaining byte count, so two items
   pointing at the same mdat offset can each drain it via `mem::take` — first one gets real bytes, the
   second gets nothing. That's a genuine edge case in the ISOBMFF `iloc` semantics, not a parser bug, which
   is exactly what made it worth confirming instead of assuming.
2. **What would I do differently?** Nothing structural — but I'd reach for the `cargo fuzz` nightly-toolchain
   PATH fix (`rustup run nightly` alone doesn't propagate to `cargo-fuzz`'s own internal `rustc`/`cargo`
   subprocess calls; prepending the nightly toolchain's `bin/` to `PATH` does) sooner; it cost one wasted
   2-minute timeout figuring out why `-Z` flags were being rejected by a "nightly" invocation.
3. **Any drift from the design?** No — the design's exact ask (confirm reachability first, guard at
   `decode_obus`, no DEC, three named failing tests) was followed as specified. The only addition beyond the
   spec's letter was the parse-layer-only `crafted_container_yields_empty_alpha_item` test, kept as a
   documentation/regression aid for the container-shape exploit itself, separate from the crux
   process-behavior test.

---

## Verify (✅ APPROVED — 2026-07-18, claude-opus-4-8)

Independent verify on Opus (build was on Sonnet — the model-comparison referee stays constant). Every
acceptance criterion re-derived from behavior, not from the Build Completion prose. Both crux items proven
with **negative controls driven in both directions**.

**1. Reachability proof (the crux) — CONFIRMED, not conflated.** Regenerated the crafted container
in-process (`build_avif_with_empty_alpha` over the real `solid_16x16.avif` primary OBU), then temporarily
removed the guard and ran `empty_alpha_obu_is_typed_error_not_abort` in a debug build: the process died with
`signal: 6, SIGABRT`, and re_rav1d printed its own diagnostic —
`` Input validation check `(sz > 0 && sz <= usize::MAX / 2, EINVAL)` failed in `fn re_rav1d::src::lib::rav1d_send_data` `` at `re_rav1d-0.1.3/src/lib.rs:523`. This is a *container-driven* abort (a conforming AVIF whose
alpha item parses to `Some(<empty>)`), not a direct empty-slice call, so it proves genuine **reachability**,
not just that the guard fires. Verified the vendored mechanism directly: `validate.rs::debug_abort()` calls
`std::process::abort()` under `cfg!(debug_assertions)`, reached via the `validate_input!` macro from
`rav1d_send_data` — release compiles it out, debug (tests + fuzz) aborts. `catch_unwind` demonstrably did
**not** save it (the process died despite `decode_avif`'s boundary).

**2. Post-fix typed error — CONFIRMED.** With the guard restored, `empty_alpha_obu_is_typed_error_not_abort`
and `empty_primary_obu_is_typed_error` both pass inside the normal (non-isolated) test binary in debug —
`Err(ImageError::Decode(_))`, process survives.

**3. Fuzz gate (the real motivation) — CONFIRMED in BOTH directions.** Dumped the crafted bytes to a file
and ran `cargo fuzz run avif_decode <crafted>` on the nightly toolchain (cargo-fuzz builds with
`-Cdebug-assertions`, so `debug_abort` is live). Post-fix: `Executed ... in 2 ms`, no crash, no ASAN report.
Guard-removed (negative control): libFuzzer reports **`deadly signal`**, backtrace
`crustyimg…avif::decode_obus → re_rav1d::rav1d_send_data → std::process::abort`, punching through the
`catch_unwind` frame (#11) — exactly the uncatchable-abort this spec closes. (Toolchain note: the Homebrew
`cargo`/`rustc` on PATH shadow rustup; prepend `~/.rustup/toolchains/nightly-*/bin` — matches the build's
reflection.)

**4. Every caller guarded — CONFIRMED.** Re-ran the greps: `rg '\.send_data\('` → **1** (in
`decode_obus_inline`), `Decoder::with_settings|Decoder::new` → **1** re_rav1d site (the other hit is
`GifDecoder::new` in `lint/rules.rs`, a different codec), `decode_obus\b` call sites → **2** (primary
`avif.rs:159`, alpha `:169`). Every re_rav1d mention outside `avif.rs` (error.rs, source, cli, image/mod,
sniff, wasm) is a doc-comment only; wasm doesn't compile the decoder at all. One guard at the chokepoint =
full coverage.

**5. Valid AVIFs decode unchanged — CONFIRMED.** The guard is a pure early-return on `is_empty()`, so for
any non-empty stream control flow is byte-identical to pre-fix — valid decode is a *definitional* no-op.
The new `solid_16x16_alpha.avif` fixture genuinely carries a real alpha aux item (verified: it declares
`auxC` + `urn:mpeg:mpegB:cicp:systems:auxiliary:alpha`), so `valid_avif_with_alpha_unchanged` exercises the
alpha `decode_obus` path and matches its pre-fix golden FNV-1a; SPEC-091's no-alpha golden tests still pass.
Fixture is synthetic/license-clean (466 B, generated by crustyimg's own encoder via `gen_avif_fixture.rs`).

**6. Gates — ALL GREEN.** `cargo test` (default) **759 passed**; `cargo test --features avif` **771 passed**;
`cargo clippy --all-targets -- -D warnings` and `--features avif` clean; `cargo fmt --check` clean;
`cargo build --no-default-features` clean; `just validate` clean (212 blocks). **PR #97 CI: 27/27 green** —
avif-feature legs pass on the full three-OS matrix (macOS/ubuntu/**windows**), distinguishing a real pass
from the SPEC-091 flake (now fixed); heic, lean, webp-lossy, msrv 1.90 all pass. `mergeStateStatus: CLEAN`.

**Completion-table ↔ acceptance-list diff:** all 6 acceptance rows have a matching Build Completion claim
AND an independent verify confirmation; no orphan criterion.

**Build-quality read (Sonnet build, model experiment):** genuinely sound. Proving reachability required
constructing a *byte-exact conforming* AVIF that exploits avif-parse's `iloc` `ToEnd`-extent `mem::take`
semantics (two items at the same mdat offset; the first drains the real bytes, the second gets the empty
leftover) — a real ISOBMFF edge case, not a parser bug — and then *driving the actual abort* rather than
assuming it. The mechanism claim held under independent tracing. The one nuance a stricter framing would
tighten: `valid_avif_with_alpha_unchanged`'s "pixel-identically to the pre-fix binary" is slightly
over-stated, since the guard cannot change a non-empty decode — the golden confirms the alpha path *works*,
not a pre/post *difference*; a framing nit, not a defect. On the hard parts (the reachability investigation,
the chokepoint choice, the abort-≠-unwind reasoning) this is **indistinguishable from an Opus build**.

**Verdict: ✅ APPROVED.** No punch list. Ready to ship.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
