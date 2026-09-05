---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-107
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: critical
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-035
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the framing is tight and names
                                   # the crux (11 AC, 13 pre-written tests, F1 decided).
                                   # Verify stays on Opus. Sonnet's known weakness is
                                   # mechanical-sweep thoroughness, which AC-3 defuses
                                   # by making the harness enumerate the corpus itself.
  created_at: 2026-07-28

references:
  decisions:
    - DEC-034
    - DEC-062
    - DEC-063
    - DEC-077
    - DEC-082
    - DEC-085
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - no-unwrap-on-recoverable-paths
    - one-spec-per-pr
    - every-public-fn-tested
  related_specs:
    - SPEC-033
    - SPEC-069
    - SPEC-070
    - SPEC-094
    - SPEC-101
    - SPEC-103

value_link: >
  STAGE-035's whole reason to exist: move the launch-readiness hostile-input item
  from "hold natively; confirm in the browser" to a driven, recorded outcome —
  and fix what driving it found.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4 — design/ship are not
        separately metered). Included a debug + release drive of a 28-case probe
        corpus across `info`/`web`/`optimize`/`convert`/`resize`, a CLI drive of
        the whole committed fuzz crash corpus, and one green `just wasm-test`
        run (26/26, 30.17 s).
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 218031873
      duration_minutes: 1241
      recorded_at: 2026-07-30
      tokens_breakdown:
        input: 1292
        output: 435915
        cache_creation: 2673195
        cache_read: 214921471
      estimated_usd: 81.04
      note: >
        MEASURED — summed .message.usage across every line with usage in this
        session's own transcript (claude-sonnet-5 throughout; one <synthetic>
        line carried zero usage and was excluded). Priced per-component at
        Sonnet anchors ($3/$15 per MTok in/out; cache_creation x1.25 input;
        cache_read x0.10 input) — 98.57% cache reads, so the flat-rate shortcut
        would badly overstate this (DEC-083). duration_minutes is wall-clock
        (first-to-last transcript timestamp), which includes several
        multi-minute waits on `cargo build`/`cargo test` full-recompiles run
        in the background per-leg for AC-11 — not continuous active compute.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 27201122
      duration_minutes: 100
      recorded_at: 2026-07-30
      tokens_breakdown:
        input: 347
        output: 158050
        cache_creation: 466771
        cache_read: 26575954
      estimated_usd: 20.16
      note: >
        MEASURED — summed .message.usage over 184 usage-bearing lines in this
        session's own transcript (claude-opus-5 throughout). Priced per-component
        at Opus anchors ($5/$25 per MTok in/out; cache_creation x1.25 input;
        cache_read x0.10 input) — 97.7% cache reads (DEC-083). Excludes the
        final return message, which is not yet in the transcript when summed.
        duration_minutes is wall-clock first-to-last timestamp and includes long
        waits on nine clean full-matrix legs (three on the branch, three on
        `main`, three re-confirming) plus three `just wasm-test` runs — not
        continuous active compute.
        Ordered BEFORE the punch-list build below: verify ran between the two
        build sessions and is what sent the spec back.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 95596984
      duration_minutes: 901
      recorded_at: 2026-07-30
      tokens_breakdown:
        input: 960
        output: 293464
        cache_creation: 2260020
        cache_read: 93042540
      estimated_usd: 40.79
      note: >
        SECOND build session (PUNCH LIST pass, not a replacement for the first
        session above — both are real spend on this spec). MEASURED — summed
        .message.usage across every line with usage in this session's own
        transcript (claude-sonnet-5 throughout; no <synthetic> lines) — refreshed
        to the session's final total after the commit/push work, matching this
        spec's own established practice for the first build session.
        Priced per-component at Sonnet anchors ($3/$15 per MTok in/out;
        cache_creation x1.25 input; cache_read x0.10 input) — 97.33% cache
        reads, so the flat-rate shortcut would badly overstate this
        (DEC-083). duration_minutes is wall-clock (first-to-last transcript
        timestamp), which includes several multi-minute background waits on
        the AC-11 matrix's per-leg `cargo test`/`cargo clippy` full-recompiles
        (through `rtk proxy`, fresh `CARGO_TARGET_DIR` each) plus real gaps
        between when this session was opened and when work on it actually
        started — not continuous active compute.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered orchestrator main-loop cycle (AGENTS §4). Two pieces of
        orchestrator work on this spec are likewise OUTSIDE the metered total
        below, and are named here rather than silently omitted: (a) the
        two-line Windows CI fix on `tests/hostile_inputs.rs` after PR #127 went
        red on both windows-latest runs, and (b) a focused AC-6 re-verify spot
        check on Opus (~$3.10, a LABELLED ESTIMATE, not a transcript sum) that
        drove three mutation families — a replacement panic incl. a
        length-matched one, an additive coexisting leak in both orderings, and
        six header perturbations after the path normalisation — all RED where
        required, with two GREEN controls proving attribution.
  totals:
    tokens_total: 340829979
    estimated_usd: 141.99
    session_count: 5
    note: >
      Sum of the three METERED cycles only: build $81.04 (Sonnet anchors) +
      verify $20.16 (Opus anchors) + punch-list build $40.79 (Sonnet anchors).
      Each was independently reconciled against its own component breakdown at
      the anchors of the model recorded in `agent` (DEC-083) — all three
      reproduce exactly, and the SPEC-108 anchor-mismatch is not repeated here
      because `implementer` was pinned before the build ran (PR #126).
      `session_count` counts all 5 entries; design and ship are main-loop and
      null by convention, so the dollar figure covers 3 of them.
      As on every spec here, `tokens_total` sums per-message `cache_read`, which
      re-counts the same cached prefix once per message (97–99% of volume across
      these cycles). It is a faithful sum of the usage records and NOT a measure
      of distinct work.
---

# SPEC-107: hostile / edge input confirmation pass

## Context

`docs/launch-readiness.md` carries an open blocker — **"Hostile / edge inputs in the
browser … (Hold natively; confirm in the browser.)"** — and the native half of that
sentence has never been driven either. The claim "holds natively" is an assumption
about a shipped binary that a launch audience is about to test for us.
[[a-plausible-test-result-is-not-a-checked-one]]

Parent stage: **STAGE-035** (launch-gating, sequenced after STAGE-034's classifier
fix so the engine is on a known-correct baseline). This is the stage's only spec.

### What the design cycle drove, and what it found

The roster in STAGE-035 was driven against a release **and** a debug binary before
this spec was written, because the honest scope of a confirmation pass depends
entirely on what is already covered. Two of the roster's seven items turned out to
be **already driven** on the surface that matters, one is **not the gate the stage
thought it was**, and one is a **real, live defect on the flagship `web` verb**.

**Measured, `target/release/crustyimg` at `f4c9d22` (current `main`), 60 s timeout
per case:**

| input | verb | exit | secs | stderr |
|---|---|---|---|---|
| zero-byte `.png` | `info` | 4 | 0.00 | `unsupported or undetectable image format` |
| text renamed `.jpg` | `info` | 4 | 0.01 | `unsupported or undetectable image format` |
| text renamed `.png` | `info` | 4 | 0.01 | `unsupported or undetectable image format` |
| truncated AVIF (½) | `info` | 1 | 0.01 | `could not decode image: avif: container box size exceeds input` |
| **truncated JPEG (⅓)** | `info` | **0** | 0.01 | **(empty)** |
| **truncated JPEG (⅓)** | `web` | **0** | 0.02 | **(empty)** |
| **truncated JPEG (⅓)** | `resize` | **0** | 0.01 | **(empty)** |
| truncated PNG (⅓) | `info` | 1 | 0.01 | `could not decode image: unexpected end of file` |
| forged 20000×20000 PNG | `info` | 1 | 0.01 | `image exceeds decode limits: image 20000x20000 declares 400000000` |
| forged 70000×1 PNG | `info` | 1 | 0.00 | `image exceeds decode limits: Image size exceeds limit` |
| missing path | `info` | 3 | 0.00 | `input not found or unreadable: missing.png` |
| empty directory | `info` | 3 | 0.01 | `input not found or unreadable: out` |
| no extension, text bytes | `info` | 4 | 0.00 | `unsupported or undetectable image format` |
| `bad_parser_state.avif` | `info` | 1 | 0.00 | `could not decode image: avif: container box size exceeds …` |
| `container_box_size_bomb.avif` | `info` | 1 | 0.00 | `could not decode image: avif: container box size exceeds …` |
| `meta_parser_state.avif` | `info` | 1 | 0.00 | `could not decode image: avif container: unread box content or bad parser sync` |
| `pixel_bomb.nef` | `info` | 1 | 0.01 | `image exceeds decode limits: raw: embedded preview exceeds …` |
| `no_preview.cr2` | `info` | 1 | 0.00 | `could not decode image: raw: no decodable embedded JPEG preview` |
| `oversize_preview.dng` | `info` | 0 | 0.25 | (empty — extracts, see F5) |

**No hang, no panic, no OOM anywhere.** Nothing on release exceeded **0.25 s**. That
is the headline confirmation the launch board is waiting for, and it is now a driven
result rather than an assumption. What follows is what the same run found wrong.

**F1 — a truncated JPEG succeeds silently (exit 0, empty stderr), on every verb.**
The user is handed a partially-grey image and never told. Truncated PNG and truncated
AVIF both error correctly; JPEG is the outlier because the decoder tolerates
truncation by design. This is the one finding that fails STAGE-035's own acceptance
bar ("a clear message on every input") on the **flagship `web` path**, and it is the
substance of this spec rather than a footnote.

*Maintainer decision (design cycle, 2026-07-28): **warn on stderr, still exit 0.***
Detect the missing end-of-image marker at the container level and print a one-line
warning; still produce the output. Rejected alternatives: exit 1 (breaks a workflow
every image viewer supports, and changes a frozen exit-code surface), and
document-only (would close a launch-gating item while knowingly leaving a silent
corruption path on `web`).

The detection is a two-byte container check, **not** a decoder change: a well-formed
JPEG ends with the EOI marker `FF D9`. Verified on the probe pair —
`real.jpg` (5694 B) ends `ffd9`; `truncated.jpg` (1898 B) ends `2f69`. This keeps the
change in the container lane (DEC-003) and touches no codec.

**F2 — exit 4's documented meaning does not cover the case it is actually used for.**
`docs/api-contract.md:59` defines `4` as *"Unsupported format / codec not built … The
message names the feature to rebuild with."* But a zero-byte file and a text file both
land on `4` via `src/image/mod.rs:417`
(`reader.format().ok_or(ImageError::UnsupportedFormat)`), and that message names no
feature because there is no feature to name — the bytes are not an image at all. The
**code is right and deliberate** (`src/cli/mod.rs:720`, pinned by
`exit_code_mapping_is_total`); the **doc is narrow**. Post-CLI-freeze (STAGE-030),
changing the exit code for no user benefit is the wrong half to change. Fix the doc.

**F3 — the debug profile leaks a raw Rust panic message; release does not.** On
`tests/fixtures/fuzz/avif_decode/meta_parser_state.avif`:

```
$ target/debug/crustyimg info …/meta_parser_state.avif
thread 'main' (60084631) panicked at …/avif-parse-2.1.0/src/lib.rs:921:9:
assertion `left == right` failed: bad parser state bytes left
note: run with `RUST_BACKTRACE=1` …
error: could not decode image: avif: decoder panicked on malformed input
RC=1
```

```
$ target/release/crustyimg info …/meta_parser_state.avif
error: could not decode image: avif container: unread box content or bad parser sync
RC=1
```

The exit code and the final message are correct on both. The panic text is upstream
`avif-parse` noise on the `debug_assertions` profile, caught by `decode_avif`'s
`catch_unwind` (`src/image/avif.rs:127`) exactly as designed. **This is a recorded
divergence, not a defect to fix** — but it is load-bearing for the harness: a
stderr-cleanliness assertion written under `cargo test` (which is the debug profile)
would assert the wrong thing. AC-6 pins the divergence so a later session does not
"discover" it and try to fix a non-user-facing thing.

**F4 — the 60 MP gate is wasm-only, so the stage's roster item is mis-aimed.**
`MAX_RAW_PREVIEW_MEGAPIXELS = 60` lives in `src/wasm.rs:536` — it is the *demo*
ceiling (DEC-082). The native CLI has no 60 MP gate; native RAW preview is bounded by
DEC-063's **64 Mpix** budget. So "a RAW just under and just over the 60 MP gate" is a
**wasm** case, and the same file on native tests a different, larger cap. Both are
worth having; conflating them would produce a row in the results table that no code
path claims. [[a-number-from-an-unproven-path-is-not-a-measurement]]

**F5 — `oversize_preview.dng` exits 0 natively** (0.25 s release, 6.14 s debug). Not a
defect: the native path has no 60 MP gate (F4), so extracting its preview is correct
behaviour. Recorded because the same fixture is *rejected* on wasm, and a results
table that showed both without the explanation would read as an inconsistency.

### Coverage matrix — what is already driven, and where the real gaps are

Built by reading the tests, not by assuming. [[read-whole-function-before-asserting-a-gap]]

| roster item | library (`cargo test`) | native CLI (exit + stderr) | headless wasm |
|---|---|---|---|
| truncated AVIF | ✅ `fuzz_regressions.rs` corpus sweep | ❌ **gap** (measured 1, clean) | ⚠ partial — `avif_input_errors_not_panics` covers *trimmed decode*, not truncation |
| truncated JPEG | ❌ **gap** (`image_load.rs:62` is PNG) | ❌ **gap + F1 defect** | ❌ **gap** |
| `.txt` → `.jpg` | ✅ `image_load.rs:53` | ❌ **gap** (measured 4) | ❌ **gap** |
| zero-byte file | ❌ **gap** | ❌ **gap** (measured 4) | ❌ **gap** |
| decompression-bomb PNG | ✅ DEC-063 unit tests | ⚠ partial — `cli.rs:3751` covers the *per-dimension* cap (70000×1); the *pixel-count* bomb (20000×20000) is a **gap** | ✅ `optimize_detailed_rejects_oversize_without_panic` (`wasm_roundtrip.rs:631`) |
| RAW under/over 60 MP | ✅ `largest_declared_preview_pixels_straddles_a_60mp_boundary` | ⚠ **n/a — wrong gate** (F4); native 64 Mpix cap is covered by `input_raw.rs:216` | ✅ `raw_preview_rejects_over_threshold_before_decode_and_extracts_under_it` (`wasm_roundtrip.rs:815`), **both sides** |
| empty-OBU AVIF | ✅ `empty_alpha_obu_is_typed_error_not_abort`, `empty_primary_obu_is_typed_error` (`src/image/avif.rs`) | ❌ **gap — and no file fixture exists** | ❌ **gap — same reason** |

Two consequences that shape this spec:

1. **The wasm half is in better shape than the stage assumed.** The 60 MP gate (both
   sides), the forged-header bomb, module-survival-after-rejection, and message
   hygiene are already driven — `just wasm-test` is green at **26/26 in 30.17 s**,
   confirmed this cycle. The wasm work here is filling four named gaps, not building a
   harness.
2. **The empty-OBU AVIF has no committed file.** It is synthesised in-process by
   `build_avif_with_empty_alpha` inside `src/image/avif.rs`'s test module, so neither
   the CLI nor wasm can drive it. Committing those bytes as a fixture is a
   precondition for two of this spec's acceptance criteria, not an extra.

## Goal

Drive a committed hostile/edge corpus through the **native CLI** and the **headless
wasm** build, assert exit code and message on every input, and fix the one defect the
pass found (F1) — leaving the launch-readiness item closed with a driven result
instead of an assumption.

## Inputs

- **Files to read:**
  - `src/image/mod.rs:400-430` — the load path and the `UnsupportedFormat` mapping at
    `:417` that F2 documents.
  - `src/image/avif.rs:280-360` — `decode_obus` and the SPEC-094 empty-OBU guard;
    `:127` the `catch_unwind` boundary F3 describes.
  - `src/image/avif.rs` test module, `build_avif_with_empty_alpha` — the byte builder
    whose output AC-4 must commit as a fixture.
  - `src/cli/mod.rs:711-770` — `code()`, the exit-code mapping; `:1056`
    `exit_code_mapping_is_total`.
  - `src/wasm.rs:530-600` — `MAX_RAW_PREVIEW_MEGAPIXELS`, the two approved messages,
    `raw_preview`.
  - `tests/cli.rs:3745-3785` — `info_on_oversized_image_exits_1_not_panic`, the
    existing CLI-level shape to copy for the new cases.
  - `tests/wasm_roundtrip.rs:625-680, 810-870` — the existing wasm hostile tests, so
    the new ones match their style and do not duplicate them.
  - `tests/fuzz_regressions.rs` — the library-level corpus sweep the CLI half mirrors.
  - `docs/api-contract.md:50-95` — the exit-code table F2 corrects.
  - `docs/launch-readiness.md` — the board item to close, and the stale line 34.
- **Related code paths:** `src/image/`, `src/cli/`, `src/wasm.rs`, `tests/`.

## Outputs

- **Files created:**
  - `tests/fixtures/hostile/` — the committed corpus (see AC-1 for the roster). Kept
    separate from `tests/fixtures/fuzz/`, which is the *crash-reproducer* corpus with
    its own provenance rule (one file per fuzzer finding); these are hand-built
    edge inputs and must not be mistaken for fuzz findings.
  - `tests/fixtures/hostile/README.md` — one line per file: what it is, how it was
    built, what it is expected to do. A corpus whose provenance is not written down
    decays into "some bytes".
  - `tests/hostile_inputs.rs` — the native CLI half of the harness.
  - `decisions/DEC-NNN-*.md` — the F1 decision (warn-on-truncation, container-level
    EOI check, exit stays 0).
- **Files modified:**
  - `src/image/mod.rs` (or the container lane) — the truncation warning for F1.
  - `tests/wasm_roundtrip.rs` — the four wasm gaps.
  - `src/image/avif.rs` — emit the empty-alpha AVIF bytes to the new fixture and have
    the existing unit tests read the committed file, so the fixture and the unit test
    cannot drift apart.
  - `docs/api-contract.md` — F2: widen exit 4's documented meaning; document the F1
    warning.
  - `docs/launch-readiness.md` — close the hostile-input item with the driven result;
    correct the stale line 34 (see Notes).
- **New exports:** none expected. Any truncation-check helper stays
  `pub(crate)`/`pub(super)`.

## Acceptance Criteria

- [ ] **AC-1.** A committed corpus under `tests/fixtures/hostile/` holds, at minimum:
      a zero-byte file; a text file renamed `.jpg`; a text file renamed `.png`; a
      truncated JPEG; a truncated PNG; a truncated AVIF; a forged-header
      decompression-bomb PNG **at the pixel-count cap** (20000×20000, ~69 B); an AVIF
      with an empty alpha OBU (AC-4). Every file is accompanied by its line in
      `README.md`. Each is **≤ 4 KB** — a corpus that is expensive to keep is a corpus
      that gets deleted.
- [ ] **AC-2.** `tests/hostile_inputs.rs` drives **every** file in
      `tests/fixtures/hostile/` through the real binary and asserts, per file: the
      **exact expected exit code**, that the process terminated (no hang), and that
      stderr is non-empty **and** free of the words `panicked`, `RUST_BACKTRACE`, and
      `unwrap` — subject to the F3 profile carve-out in AC-6. The expected code is
      declared per-file in the test, not derived from the run.
      [[fixtures-from-the-code-under-test-cannot-fail]]
- [ ] **AC-3.** The harness **enumerates the fixture directory** and fails if any file
      in it has no declared expectation. Adding a file to the corpus without an
      expectation must turn the suite red, not be silently skipped.
      [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-4.** The empty-alpha-OBU AVIF is committed as a real file, and
      `src/image/avif.rs`'s existing `empty_alpha_obu_is_typed_error_not_abort` reads
      **that file** rather than rebuilding the bytes in-process — so the fixture the
      CLI and wasm drive is provably the same artifact the SPEC-094 guard is pinned
      against. Driven through the CLI it must exit 1, and the `debug_assertions`
      profile is the leg that proves the guard (a `debug_abort()` is not an unwind —
      [[a-thread-boundary-does-not-catch-abort]]).
- [ ] **AC-5.** **F1 fixed.** A truncated JPEG through `info`, `web`, `convert`, and
      `resize` prints a one-line stderr warning naming truncation, still writes its
      output, and still exits **0**. A well-formed JPEG through the same four verbs
      prints **no** such warning — the negative control, without which the warning is
      unfalsifiable. [[a-plausible-test-result-is-not-a-checked-one]]
- [ ] **AC-6.** **F3 recorded, not fixed.** The harness's stderr-cleanliness assertion
      is profile-aware: it carves out the upstream `avif-parse` panic text on
      `cfg!(debug_assertions)` **by name**, with a comment stating that release is
      clean and why. A blanket "skip stderr checks in debug" is not acceptable — the
      carve-out must be narrow enough that a *new* panic still fails the suite.
- [ ] **AC-7.** The four wasm gaps are closed in `tests/wasm_roundtrip.rs`: zero-byte,
      `.txt` bytes, truncated JPEG, and the empty-OBU AVIF each return a `JsError`
      (never a panic that kills the module), and — following the established
      `optimize_detailed_rejects_oversize_without_panic` pattern — **a subsequent
      ordinary call still succeeds**, proving the module survived. Do not duplicate
      the four cases the matrix marks ✅.
- [ ] **AC-8.** **F2 fixed in the doc.** `docs/api-contract.md`'s exit-4 row covers
      "the bytes are not a recognisable image" alongside "codec not built", and drops
      the universal claim that the message names a feature. The F1 warning is
      documented on the affected verbs. [[documentation-has-no-green]]
- [ ] **AC-9.** `docs/launch-readiness.md`'s hostile-input blocker is closed with the
      **driven** outcome and a pointer to the harness, and the genuinely
      browser-specific remainder (does the demo *surface* these errors legibly; how a
      phone behaves on the big ones) is named and left on the board, not silently
      dropped. The stale line 34 is corrected in the same pass (see Notes).
- [ ] **AC-10.** A **negative control** proves the harness is load-bearing: reverting
      the AC-5 truncation warning must turn at least one test **red**, and deleting a
      fixture file must turn AC-3's enumeration **red**. Record both in Build
      Completion — a harness nobody has seen fail is not evidence.
      [[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]
- [ ] **AC-11.** Clean **full-matrix** green from a fresh per-leg `CARGO_TARGET_DIR`:
      default, `--no-default-features`, `--features webp-lossy`; `clippy -D warnings`
      on each; `fmt --check`; plus `just wasm-test`. Confirm each log says
      `Compiling crustyimg` — an incremental build false-greens here.
      [[a-stale-incremental-build-is-a-false-green]] Reference totals on `main`:
      **lean 784 / default 803 / webp-lossy 810**.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is to make
these pass. Expected to FAIL against current `main` except where noted.

- **`tests/hostile_inputs.rs`** (new)
  - `"every_hostile_fixture_has_a_declared_expectation"` — AC-3. Enumerates
    `tests/fixtures/hostile/`, fails on any file absent from the expectation table.
    **Fails today** (neither directory nor table exists).
  - `"hostile_corpus_exit_codes_are_as_declared"` — AC-2. Drives every fixture through
    `info` and asserts the declared code. **Fails today.**
  - `"hostile_corpus_stderr_is_non_empty_and_not_a_panic"` — AC-2 + AC-6, with the
    narrow `debug_assertions` carve-out. **Fails today.**
  - `"empty_alpha_obu_avif_exits_1_through_the_cli"` — AC-4. **Fails today** (no
    fixture).
  - `"truncated_jpeg_warns_on_stderr_and_still_exits_0"` — AC-5, across `info`, `web`,
    `convert`, `resize`. **Fails today**: measured exit 0 with *empty* stderr.
  - `"well_formed_jpeg_emits_no_truncation_warning"` — AC-5's negative control.
    **Passes today** vacuously; it is the guard against a warning that fires on
    everything, and must be written anyway.
  - `"pixel_count_bomb_png_exits_1"` — the 20000×20000 gap the matrix names.
    **Fails today** only as a *missing* test; the behaviour is already correct
    (measured exit 1) — write it as coverage, and say so rather than implying a fix.
- **`src/image/avif.rs`** (unit, modified)
  - `empty_alpha_obu_is_typed_error_not_abort` — AC-4: re-pointed at the committed
    fixture. **Passes today** against in-process bytes; must keep passing against the
    file, which is the actual assertion (same bytes, one source of truth).
- **`tests/wasm_roundtrip.rs`**
  - `"wasm_rejects_zero_byte_input_without_panicking"` — AC-7. **Fails today.**
  - `"wasm_rejects_non_image_bytes_without_panicking"` — AC-7. **Fails today.**
  - `"wasm_truncated_jpeg_does_not_kill_the_module"` — AC-7. **Fails today.**
  - `"wasm_empty_obu_avif_is_an_error_not_an_abort"` — AC-7. **Fails today.**
- **Negative controls** (AC-10, run and recorded, not committed as tests)
  - Revert the AC-5 warning → `truncated_jpeg_warns_on_stderr_and_still_exits_0` must
    go RED.
  - Delete one fixture file → `every_hostile_fixture_has_a_declared_expectation` must
    go RED.

## Implementation Context

### Decisions that apply

- `DEC-034` — per-dimension and allocation decode caps. The 70000×1 case is this one;
  do not conflate it with DEC-063's pixel budget.
- `DEC-062` — the decoder fuzz gate. `tests/fuzz_regressions.rs` is the **library**
  half of "never panics"; this spec is the **binary and wasm** half. Mirror its
  structure, do not merge into it — its corpus has a provenance rule (one file per
  fuzzer finding) that hand-built edge inputs would violate.
- `DEC-063` — the 64 Mpix peak-decode-memory budget, checked against *declared*
  dimensions before allocation. This is what the forged-header bomb exercises.
- `DEC-077` — the AVIF single-thread policy that makes `decode_obus` spawn its
  ample-stack thread; relevant only as context for why the empty-OBU guard sits where
  it does.
- `DEC-082` — the 60 MP demo ceiling. **wasm only.** See F4.
- **A new DEC is required** for F1 (warn on a truncated JPEG, container-level EOI
  check, exit stays 0), recording the two rejected alternatives so they are not
  re-proposed.

### Constraints that apply

- `test-before-implementation` (**blocking**) — the Failing Tests above go in first.
- `clippy-fmt-clean` (**blocking**) — on every leg of AC-11, including wasm.
- `no-unwrap-on-recoverable-paths` (**blocking**) — the truncation check runs on
  untrusted bytes; a two-byte tail read on a possibly-empty buffer is exactly where an
  index panic hides. Use slice methods that cannot panic on a short input.
- `one-spec-per-pr` (**blocking**) — SPEC-110 and SPEC-111 are separate PRs. If the
  pass surfaces a defect in `convert`'s orientation handling, that is SPEC-110's, not
  this spec's.
- `every-public-fn-tested` — applies to any helper the F1 fix introduces.

### Prior related work

- `SPEC-033` / `SPEC-070` (shipped) — the decode caps this pass drives from outside.
- `SPEC-069` (shipped, DEC-062) — the fuzz gate and the committed crash corpus. This
  spec drives that same corpus **through the CLI** for the first time; the library
  already sweeps it.
- `SPEC-094` (shipped) — the empty-OBU `debug_abort()` guard. AC-4 is its
  binary-and-wasm confirmation.
- `SPEC-101` (shipped) — closed the mobile cross-browser gate; the reason line 34 of
  the launch board is stale.
- `SPEC-103` (shipped) — the demo's RAW preview and the 60 MP gate (DEC-082).

### Out of scope (for this spec specifically)

- **The browser half.** Whether the demo *surfaces* these errors legibly, and how a
  phone behaves on the large ones, needs a real device and a human looking at a
  screen. It stays on the launch board as the maintainer's mobile test
  ([[never-drive-the-maintainers-live-browser]]). This spec covers the wasm build
  driven headlessly, which is where the engine behaviour lives.
- **Platform-aware RAW gating.** Whether the 60 MP demo gate should ever become
  device-dependent is decided by the mobile test, not here.
- **Changing any exit code.** F2 is a documentation fix. The mapping is deliberate and
  the CLI surface is frozen (STAGE-030).
- **Fixing F3.** The upstream `avif-parse` panic text on the debug profile is recorded
  and carved out, not suppressed. Suppressing an upstream panic message would hide
  future real ones.
- Any engine redesign, classifier work, new codecs, or CLI surface changes.

## Notes for the Implementer

- **Build the corpus deterministically and commit the builder.** The forged-header
  PNGs used in the measurement above were generated with a ~15-line Python
  `struct`+`zlib` writer (real CRCs, so the decoder reads the header rather than
  bailing on a malformed chunk — the same shape as
  `wasm_roundtrip.rs`'s `png_header_declaring`). Prefer generating them in Rust in a
  `#[test]`-guarded builder or a small `tests/common/` helper, so the corpus can be
  regenerated and reviewed rather than being opaque committed bytes.
- **The truncation check belongs in the container lane, not the decoder.** JPEG's EOI
  is `FF D9`; the check is on the input bytes, before or beside the decode, and must
  not re-read the file. Do not reach into the codec.
- **Do not assert on the exact wording of upstream decoder messages.** Several
  measured strings above (`Format error decoding Png: IDAT or fDAT …`,
  `Image size exceeds limit`) come from the `image` crate and will change under it.
  Assert on the **exit code** and on **our** message prefixes
  (`image exceeds decode limits:`, `could not decode image:`), never on upstream text.
- **`CARGO_BIN_EXE_crustyimg` is the debug binary under `cargo test`.** That is what
  makes AC-4's `debug_assertions` leg work — and it is also why AC-6's carve-out is
  needed. Both facts come from the same property; do not "fix" one and break the other.
- **`just wasm-test` needs `wasm-bindgen-test-runner`** (`cargo install wasm-bindgen-cli
  --version 0.2.126`) — present on this machine, confirmed green at 26/26 this cycle.
- **Line 34 of `docs/launch-readiness.md` is stale** and should be corrected while this
  spec is in the file. It reads *"Mobile — ⚠ STILL OPEN, the remaining cross-browser
  blocker"*, but SPEC-101's record shows that gate closed (iOS Safari + DuckDuckGo PASS
  on real devices; Android Chrome untested, accepted on maintainer judgment). The board
  is maintainer-owned, so this is a correction to a stale line, **not** a re-grading —
  but left alone it will make a future session re-open a closed gate.
- **Report what you did not cover.** If a roster item turns out to be unreachable or
  already covered, say so in Build Completion with the evidence. A silent drop reads as
  coverage. [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-107-hostile-input-pass`
- **PR (if applicable):** [#127](https://github.com/jysf/crustyimg/pull/127) — opened, not merged (maintainer's call).
- **All acceptance criteria met?** yes — AC-1 through AC-11, all green (full detail below).
- **Punch-list pass (second build session, 2026-07-30):** verify returned ⚠ PUNCH LIST on this
  PR — AC-6 was not met as worded (the carve-out screened only by `contains()`, not
  additively) and two record corrections were about to be archived as false. This pass:
  - **Fixed the one unmet acceptance criterion.** `meta_parser_state_avif_debug_leak_is_the_only_carve_out`
    now asserts the debug-profile extra stderr lines are *exactly* the known 5-line F3 banner
    (line-by-line), not merely that the output contains its key phrase. Drove both mutations
    myself against the fix: a panic REPLACING the banner (RED, exit `101` not `1`) and a
    panic-shaped leak COEXISTING alongside the real banner (RED, 8 extra lines not 5) — and
    confirmed the coexisting case was GREEN under the original assertion, reproducing verify's
    finding before restoring the fix. See AC-6 below and
    [[test-a-carve-out-additively-not-just-by-replacement]].
  - **Corrected two archived-record errors** verify found: the headline "deviation from spec"
    was backwards (the design's bare-shape recommendation was right for the wasm test's actual
    100000² dims; the IDAT+IEND extension was needed only for the native path's 20000² fixture,
    under the dimension cap) — see the corrected "Deviations from spec" bullet and reflection
    Q1 below; and the follow-up verb list was wrong in both directions (`edit`/`watermark`'s
    primary input/`apply --recipe web` DO warn; `diff`/`responsive`/plain-recipe
    `apply`/`build`/`watermark --image`/`lint`/`meta strip` do not) — corrected below and in
    the timeline, re-driven empirically this pass with positive controls.
  - **Strengthened two wasm assertions** (`tests/wasm_roundtrip.rs`) to match the file's own
    convention (message assertions, not just non-empty) — see AC-7 below.
  - **Fixed doc overclaims**: the hostile corpus README/launch-readiness both claimed the whole
    corpus is driven through native CLI *and* wasm (only 1 of 8 files is); `cli-reference.md`'s
    `--quiet` row was literally false given DEC-085; `api-contract.md` named 5 verbs in a way
    that read as exhaustive. Corrected DEC-085's rationale (decision unchanged, but "every
    other advisory is cosmetic" was wrong — DEC-023/DEC-075 gate non-cosmetic warnings behind
    `--quiet` too). Fixed a nit (an unconfigured test-hang-bound claim) and a cost-note
    rounding error (98.4% → 98.57%, verified against the recorded token breakdown).
  - **Drove what was cheap from "what verify could not check":** `build` (a scratch manifest +
    plain recipe against `truncated.jpg`, confirmed unwarned), release-profile stderr
    cleanliness (swept the full 8-file corpus + `meta_parser_state.avif` on `target/release` —
    clean everywhere), and `view`'s non-tty path (confirmed unwarned, though the real
    interactive tty-render path stays untested — left as verify's gap). Did not attempt
    driving `view`'s interactive path (needs a real tty; verify already found it hangs under
    `script`, and the risk/reward didn't justify retrying).
  - **Re-ran the full matrix** clean from fresh per-leg `CARGO_TARGET_DIR`s through `rtk proxy`,
    sequentially, from the first leg: **lean 797 / default 816 / webp-lossy 823** (all exactly
    matching the prior reference — expected, since this pass rewrote existing test bodies
    rather than adding/removing tests), `clippy --all-targets -D warnings` clean on all three,
    `cargo fmt --check` clean, `just wasm-test` **30/30** (confirms the two strengthened wasm
    assertions compile and pass).
- **New decisions emitted:**
  - `DEC-085` — a truncated JPEG warns on stderr; exit stays 0 (F1). Records both rejected
    alternatives (exit 1; document-only) and the reasoning for making the warning
    unconditional (not `--quiet`-suppressed), which is a choice the spec didn't explicitly
    pin down.
- **Per-AC evidence:**
  - AC-1: `tests/fixtures/hostile/` holds all 8 named fixtures (zero-byte, text-as-.jpg,
    text-as-.png, truncated JPEG/PNG/AVIF, the 20000×20000 pixel-count-bomb PNG, the
    empty-alpha-OBU AVIF), each ≤ 758 B (well under the 4 KB cap), with a README.md line
    each and the exact Rust builder that produced them.
  - AC-2/AC-3: `tests/hostile_inputs.rs` drives every corpus file through the real binary,
    asserts the exact declared exit code + a non-empty, prefix-matching, panic-free stderr,
    and the enumeration test fails on either a corpus file with no expectation OR a
    declared expectation with no file (both directions verified — AC-10).
  - AC-4: `empty_alpha_obu.avif` committed (317 B); `empty_alpha_obu_is_typed_error_not_abort`
    re-pointed at the file via `include_bytes!`; a new CLI test
    (`empty_alpha_obu_avif_exits_1_through_the_cli`) drives it through the binary, exit 1 on
    both profiles (the `debug_assertions` leg is what actually proves the `debug_abort()`
    guard fires).
  - AC-5: fixed in `src/image/mod.rs` (container-level `jpeg_missing_eoi`, no
    `unwrap`/`panic` on a short buffer) and wired at three CLI call sites
    (`report.rs::run_info`; `ops.rs::run_pixel_op`, both single/multi branches — covers
    `resize` always and `convert`/pinned-`web`/pinned-`optimize`; `optimize.rs`'s
    `optimize_decide_one` → `run_optimize_autodecide` — covers default-mode `web`/`optimize`).
    Verified end-to-end on all four named verbs by hand and by test; negative control
    (`well_formed_jpeg_emits_no_truncation_warning`) passes non-vacuously (confirmed via
    AC-10's revert).
  - AC-6 (corrected on the punch-list pass — the assertion below is narrower than what the
    original build shipped, which verify found had a hole): `meta_parser_state_avif_debug_leak_is_the_only_carve_out`
    drives the EXISTING `tests/fixtures/fuzz/avif_decode/meta_parser_state.avif` (not
    duplicated into the new corpus) through the CLI and asserts the carve-out is
    **additive-safe**: on debug, the extra stderr lines before the final typed-error line must
    be *exactly* the known 5-line F3 banner — matched line-by-line (the panic header by
    prefix/substring since it carries a process ID; the assertion text, left/right values, and
    backtrace note by exact string) — not merely `stderr.contains("bad parser state bytes
    left")`, which is what the original build shipped and which cannot detect a SECOND,
    unrelated panic-shaped leak coexisting alongside the known banner on the same input.
    Drove both mutations myself against this fixed assertion and watched both go RED: (a)
    bypassing `catch_unwind` so the panic REPLACES the typed error — exit code
    `Some(101)` instead of `Some(1)`; (b) an `eprintln!` injecting an unrelated panic-shaped
    line that COEXISTS with the real banner — banner-length check fails (8 extra lines, not
    5). Also confirmed (b) stayed GREEN under the ORIGINAL (pre-fix) assertion — reproducing
    verify's finding exactly — before restoring the fix. Also swept release-profile stderr
    cleanliness across the FULL corpus (all 8 `tests/fixtures/hostile/` files, not just
    `meta_parser_state.avif`) on a `target/release` build: every input's stderr is free of
    `panicked`/`RUST_BACKTRACE`/`unwrap`, and `meta_parser_state.avif` itself takes a
    genuinely different path on release (the triggering `debug_assert!` compiles out, so
    `avif-parse` returns a real parse error — "unread box content or bad parser sync" — never
    reaching the panic path at all) — closing the "release-profile stderr cleanliness (only
    debug was driven)" item from "what verify could not check" below.
  - AC-7: four new `#[wasm_bindgen_test]`s in `tests/wasm_roundtrip.rs`
    (`wasm_rejects_zero_byte_input_without_panicking`,
    `wasm_rejects_non_image_bytes_without_panicking`,
    `wasm_truncated_jpeg_does_not_kill_the_module`,
    `wasm_empty_obu_avif_is_an_error_not_an_abort`), each an `Err` + a subsequent ordinary
    `info()` call proving survival, following `optimize_detailed_rejects_oversize_without_panic`'s
    shape. `just wasm-test`: 30/30 (26 baseline + 4 new).
  - AC-8: `docs/api-contract.md`'s exit-4 row widened to name "not a recognisable image at
    all" alongside codec-not-built, and the universal "the message names a feature" claim
    dropped; the F1 warning documented as a new callout paragraph (mirrors the existing
    "Decode resource limits" style) rather than repeated across 4 subcommand sections.
  - AC-9: `docs/launch-readiness.md`'s hostile-input blocker marked `[x]` with the driven
    outcome and a pointer to the two test files; the browser-specific remainder (does the
    demo surface errors legibly; phone behavior on the largest inputs) is named explicitly
    and left open, folded into the existing mobile-device-pass item. Line 34 (Mobile) is
    also corrected per SPEC-101's actual outcome (iOS Safari + DuckDuckGo PASS; Android
    Chrome untested, accepted on maintainer judgment) — both stale-line corrections, not a
    re-grading of any other item.
  - AC-10: both negative controls run and confirmed (not just asserted):
    reverting `truncated_jpeg` to a hardcoded `false` in `Image::from_bytes` turned
    `truncated_jpeg_warns_on_stderr_and_still_exits_0` AND
    `hostile_corpus_stderr_is_non_empty_and_not_a_panic` RED (both, not just one); deleting
    `zero_byte.png` from the corpus turned `every_hostile_fixture_has_a_declared_expectation`
    RED. Also verified the reverse direction (an undeclared extra file in the corpus dir
    also turns it RED) as extra rigor. Both reverts undone before the final matrix run.
  - AC-11: clean from fresh per-leg `CARGO_TARGET_DIR`s, run sequentially (never
    shared-and-parallel), each log confirmed showing exactly one `Compiling crustyimg` line.
    First pass through the ordinary shell hook silently collapsed `cargo test`'s real output
    into a one-line summary with no `Compiling` line at all (a NEW instance of the
    documented `rtk` output-corruption risk, this time on `cargo test`/`cargo clippy`, not
    just `grep`/`git log`) — caught it via the missing positive control, wiped the
    already-run target dir, and re-ran every leg through `rtk proxy` for genuine raw output
    before trusting any count:
    lean **797** (795 passed / 0 failed / 2 ignored, was 784, +13), default **816** (814/0/2,
    was 803, +13), webp-lossy **823** (821/0/2, was 810, +13) — the +13 delta is identical
    on all three legs and reconciles exactly against what this spec added (9 in
    `tests/hostile_inputs.rs` incl. 1 ignored generator, 3 new unit tests in
    `src/image/mod.rs`, 1 ignored generator in `src/image/avif.rs`). `clippy --all-targets
    -D warnings` clean on all three legs; `cargo fmt --check` clean; `just wasm-test` 30/30.
- **Deviations from spec (corrected on the punch-list pass — the design's recommended shape
  was right; the original wording of this bullet was not):**
  - Verify drove the full shape × dimension cross product and found the bare
    signature-+-IHDR-+-CRC shape (`wasm_roundtrip.rs`'s private `png_header_declaring`) is
    **sufficient** at the wasm test's actual dims, 100000×100000
    (`optimize_detailed_rejects_oversize_without_panic`): a second, unnamed cap explains why —
    the upstream per-dimension limit (`MAX_IMAGE_DIMENSION = 65_535`, DEC-034) fires **inside
    the header read itself**, before any chunk-boundary peek matters, so the bare shape at
    100000² already reaches a real `LimitsExceeded("Image size exceeds limit")`.
    `optimize_detailed_rejects_oversize_without_panic` is **not vacuous** — it was already
    exercising a genuine `LimitsExceeded` path before this spec touched anything.
  - `ImageReader::into_dimensions()` needing to see the chunk boundary past IHDR (else the peek
    fails early with a generic `Decode("unexpected end of file")`, before `check_pixel_budget`
    ever runs) is real, but only bites **below** the dimension cap: `tests/hostile_inputs.rs`'s
    native CLI corpus fixture `pixel_count_bomb.png` is 20000×20000 — under
    `MAX_IMAGE_DIMENSION`, so the only cap that can catch it is the pixel-count budget
    (`check_pixel_budget`, DEC-063), which needs the full header parse to compute total pixels.
    Extending `tests/common::png_header_declaring` (a new copy, not touching
    `wasm_roundtrip.rs`'s private original) to append a real empty `IDAT` + `IEND` **was**
    genuinely necessary — but only for that 20000² native-path fixture, not for the wasm test's
    100000² case as the original wording of this bullet implied.
  - The real residual, which is the opposite of what this bullet originally claimed: **no wasm
    test currently reaches `check_pixel_budget` at all** — `optimize_detailed_rejects_oversize_without_panic`
    hits the dimension cap (DEC-034), not the pixel-count budget (DEC-063), even though its
    docstring advertises both. Filed as a follow-up candidate (add a wasm case using the
    extended shape at dims under `MAX_IMAGE_DIMENSION`, e.g. 20000×20000, to actually exercise
    `check_pixel_budget` on the wasm target); not fixed here (`one-spec-per-pr`).
  - The design's measured truncation ratio (real.jpg 5694 B → truncated.jpg 1898 B, ≈⅓)
    does not transfer across images: empirically, a 96×96 gradient JPEG truncated to ⅓
    hits a hard decoder error (`Not enough bytes...`), not F1's silent-success case; the
    boundary on that specific fixture is between 48% and 50%. Used 60% for
    `tests/fixtures/hostile/truncated.jpg` (comfortable margin on the decodes-OK side) and
    documented the finding in the corpus builder's comment and `README.md`. The wasm
    AC-7 test needs the OPPOSITE property (a truncation aggressive enough to force an
    `Err`, since AC-7's wording is "each return a JsError") — so it uses a separate,
    much-more-truncated JPEG (3% of a larger photo fixture), not the CLI corpus's file.
  - AC-6's carve-out is proven against `tests/fixtures/fuzz/avif_decode/meta_parser_state.avif`
    (the pre-existing fuzz fixture, read in place) rather than a new file in
    `tests/fixtures/hostile/` — none of the 8 AC-1 fixtures happen to trigger the F3
    debug-only panic-leak, so the carve-out logic would otherwise be untested dead code.
    This is additive beyond the literal Failing Tests list (which names one combined
    AC-2+AC-6 test) but was the only way to make AC-6's "narrow enough that a new panic
    still fails" claim actually checked rather than assumed.
  - The F1 warning is **unconditional** (not suppressed by `--quiet`), unlike this crate's
    other CLI advisory warnings. The spec didn't pin this down explicitly; DEC-085 records
    the reasoning (gating a silent-corruption warning behind `--quiet` would reopen the
    exact loophole this fix closes).
- **Follow-up work identified (corrected on the punch-list pass — this list was wrong in both
  directions):**
  - The F1 warning is wired on exactly the four named verbs (`info`/`web`/`convert`/`resize`)
    plus `optimize` (shares `run_pixel_op`/`optimize_decide_one`) and, incidentally,
    `thumbnail`/`auto-orient` (both delegate to `run_pixel_op` too). Verify drove 16
    invocations and found `edit` and `watermark` (its *primary* input) **do** warn — both route
    through `run_pixel_op` (`run_edit`/`run_watermark`, `src/cli/ops.rs`) — as does `apply
    --recipe web`, since a recipe ending in the terminal `optimize` step dispatches through the
    same fast auto-decide path as `web` (`src/cli/optimize.rs::run_apply`).
  - It is **not** wired on: `diff` (both inputs — `run_diff` calls `Image::load` directly for
    each side), `responsive` (`run_responsive` decodes once via `Image::load`, bypassing
    `run_pixel_op`), `apply` with a **plain pixel recipe** — no terminal `optimize` step — for
    both single input and the rayon batch path (both go through the shared `encode_one` worker
    in `src/cli/common.rs`, which calls `Image::load`/`from_bytes` directly), `build` (its
    cache-miss path shares that same `encode_one` worker), `watermark --image` (the *overlay*
    only — `watermark_overlay` loads it via a direct `Image::load`, separate from the
    `run_pixel_op` call that covers the primary input/inputs), and two verbs the original list
    omitted entirely: `lint` (`LintTarget::build` decodes via `Image::decode_path` directly) and
    `meta strip` (`run_metadata_lane` never decodes to pixels at all —
    `metadata-not-via-pixel-encode` — so there is no decode seam to warn from).
  - Every claim above was re-driven empirically on this punch-list pass (not left as source
    inspection): all of `diff`/`responsive`/`apply` (single + batch)/`build`/`watermark --image`/
    `lint`/`meta strip` against the committed `truncated.jpg` fixture produced **empty stderr**,
    while `edit` and `apply --recipe web` (positive controls, driven the same way) both printed
    the warning — proving the empty-stderr results are real negatives, not a broken harness.
    `build` specifically: a scratch `crustyimg.build.toml` + a plain (non-`optimize`-terminal)
    recipe against `truncated.jpg`, `crustyimg build --no-cache`, exit 0, stderr only the
    build-progress lines — closing the "verify's manifest syntax was wrong" item from "what
    verify could not check" below. `view` is also structurally unwarned by the same direct-
    `Image::load` pattern (`run_view`, `src/cli/ops.rs`) — driven on the non-tty path (`Sink::
    Display`'s `is_terminal()` check fires before any render attempt, so a piped invocation
    returns a clean `SinkError::NotATty` fast, no hang, no warning on stderr) — but the real
    interactive tty-render path remains untested; that specific gap stands as verify left it.
  - AC-5 only requires the four named verbs, so this is in-scope-complete, but the underlying
    defect is generic to any JPEG-decoding verb that bypasses `run_pixel_op`. Worth a small
    follow-up spec (a "warning coverage sweep" over the remaining load sites, scoped from the
    corrected list above, not the original) rather than expanding this one (`one-spec-per-pr`).
    Verify's sweep scope, stated as a claim: `/usr/bin/grep -rn "Image::load\|Image::from_bytes"
    src/` — 36 hits, 14 in `src/cli/`, each resolved to its enclosing fn; it would miss a load
    reached through an untraced helper.
  - Not filed as a spec, but worth noting for whoever designs SPEC-110/111: this pass did
    not find any NEW defects on `convert`'s orientation handling or elsewhere — the roster
    was exactly F1 (fixed) and F2 (doc-only), as the design cycle predicted.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?** (corrected on the punch-list pass —
   the first half of this answer was wrong; the design matrix's recommended shape was right.)
   — One thing needed empirical discovery the spec's Notes didn't fully anticipate: at the
   native CLI corpus fixture's dims (20000×20000, under `MAX_IMAGE_DIMENSION`), reaching the
   pixel-count budget (`check_pixel_budget`, DEC-063) needs `ImageReader::into_dimensions()`
   to see the chunk boundary past IHDR, which the bare signature-+-IHDR-+-CRC shape doesn't
   provide — it fails earlier with a generic `Decode("unexpected end of file")` without an
   empty IDAT+IEND appended. That is specific to sitting under the dimension cap, though: the
   wasm test's own dims (100000×100000) are well *over* `MAX_IMAGE_DIMENSION`, so the bare
   shape it was already using reaches a real `LimitsExceeded` straight from the dimension cap
   (DEC-034) — "reuse `png_header_declaring`'s shape" was correct as written for that test; it
   only needed extending for the native-path fixture. Second, the design's measured JPEG
   truncation ratio (~⅓) is specific to that probe's `real.jpg`, not a general property of
   "JPEG truncated to ⅓" — it depends on where the entropy-coded scan data lands relative to
   the cut, which varies by image content and size. Both took a few minutes of binary-search
   probing (via a temporary scratch `#[test]`, deleted before commit) to pin down rather than
   being derivable from the spec text alone.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Whether the F1 warning should respect `--quiet`. The spec says "warn on stderr, still
   exit 0" but doesn't address suppression, and this codebase has an established
   `--quiet`-gates-all-advisories convention that a literal reading would have pointed
   toward — which would have quietly reopened the exact silent-corruption gap this spec
   exists to close, for anyone running with `-Q`. Filed as an explicit DEC-085 call rather
   than a silent default.

3. **If you did this task again, what would you do differently?**
   — Run the two empirical probes (PNG-bomb chunk shape, JPEG truncation boundary) as the
   very first step, before writing any of the harness or fixture-builder code, rather than
   discovering them mid-build. Both were cheap to check (a few `Image::from_bytes` calls in
   a throwaway test) and would have let the corpus generator get the shapes right on the
   first pass instead of two. Separately: run the AC-11 matrix through `rtk proxy` from the
   very first leg, not the second — the summarized-output failure mode was already a
   documented lesson ([[rtk-can-silently-corrupt-grep-counts]]) for `grep`/`git log`; this
   session is the evidence it also applies to `cargo test`/`cargo clippy`, and that memory
   is worth widening rather than re-discovering per-command.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — **Drive the roster on more than one platform when the finding IS a platform artifact.**
   F3 is a finding whose entire subject is platform-specific stderr text, and all three
   cycles characterised it on macOS alone — design captured it there, build reproduced it
   there, verify drove it there. It stayed harmless only while the assertion was loose;
   the punch-list pass made it exact (correctly, to close verify's additive-leak hole) and
   that instantly turned a shared blind spot into a red Windows CI leg. The fix was two
   lines, but nobody would have needed them if the design cycle had asked "is this string
   host-shaped?" once. Generalised: **an exact assertion on a panic message, a path, or
   formatted stderr is a platform assertion wearing a correctness assertion's clothes.**

   Second, smaller: the design cycle's measured JPEG truncation ratio (⅓) was reported as
   if it were a property of "truncated JPEG" when it is a property of *that* file — the
   build found the real boundary sits near 50% on its own fixture. Measurements taken on
   one artifact should be stated as such in the spec, not as a general constant a builder
   can lift.

2. **Does any template, constraint, or decision need updating?**
   — No template or constraint change. Two decisions are now on the record from this spec:
   **DEC-085** (a truncated JPEG warns on stderr, unconditionally, exit stays 0), whose
   rationale was corrected during the punch-list pass after verify found it overstated the
   `--quiet` convention it departs from — DEC-023 and DEC-075 both gate advisories behind
   `--quiet`, so the departure is *more* notable than DEC-085 originally claimed, not less.
   The decision itself stands unchanged.

   Worth flagging for whoever touches `docs/api-contract.md` next: this spec widened exit
   4's documented meaning to cover "the bytes are not a recognisable image at all" and
   dropped the universal claim that its message names a feature. The exit-code *mapping*
   was never wrong; only its description was.

3. **Is there a follow-up spec I should write now before I forget?**
   — **Yes — a warning-coverage sweep, and it should be framed from the corrected list.**
   The F1 truncation warning is wired on `info`, `web`, `convert`, `resize`, `optimize`,
   `thumbnail` and `auto-orient` (the last three incidentally, via `run_pixel_op`). It is
   NOT wired on `diff` (both inputs), `responsive`, `apply`/`build` with a plain pixel
   recipe, `watermark --image` (overlay only), `lint`, or `meta strip` — each decodes JPEGs
   directly without passing through `run_pixel_op`. AC-5 only ever required the four named
   verbs, so this spec is in-scope-complete, but the underlying defect is generic to any
   JPEG-decoding verb.

   **Frame it from verify's list, not the build's.** The build's original follow-up list was
   wrong in both directions — it named `edit` and `watermark` (which do warn) and omitted
   `lint` and `meta strip` (which do not). Verify drove 16 invocations to correct it. A spec
   scoped from the uncorrected list would have sent real work at the wrong files.

   Second, smaller follow-up: **no wasm test reaches `check_pixel_budget`** (DEC-063),
   although `optimize_detailed_rejects_oversize_without_panic`'s docstring advertises both
   DEC-034 and DEC-063. Its 100000² fixture is caught by the per-dimension cap first. The
   test is not vacuous — verify refuted that — but its advertised reach exceeds what it
   drives. [[a-guards-advertised-reach-is-a-claim]]
