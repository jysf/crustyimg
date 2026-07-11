---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-069
  type: chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # the run + triage + regression-conversion + recipe + record is bounded; the UNKNOWN is what the fuzzer finds (S if all clean, L if it surfaces real decoder bugs). Toolchain (nightly + cargo-fuzz) is a real setup step.

project:
  id: PROJ-007
  stage: STAGE-024
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-10

references:
  decisions: [DEC-034, DEC-035, DEC-052, DEC-053, DEC-054, DEC-055, DEC-056, DEC-004]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - every-public-fn-tested
    - clippy-fmt-clean
  related_specs: [SPEC-058, SPEC-060, SPEC-061, SPEC-062, SPEC-068]

value_link: "STAGE-024's #1 (High) — the pre-1.0 decoder fuzz gate the roadmap requires but that was never run; the one untrusted-binary surface this wave's review could not close by reading."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-10
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Grounded in a firsthand read of the existing `fuzz/` harness (detached workspace, four
        targets, run+seed recipes already in `fuzz/Cargo.toml`), the per-format seed fixtures
        (`tests/fixtures/{avif,svg,raw,heic}`), the decoder entry points (`Image::from_bytes` /
        `raw_preview`), and the roadmap's explicit pre-1.0 fuzz-gate lines. Key finding: the
        scaffolding EXISTS and was never run (no nightly/cargo-fuzz in prior build/verify envs), so
        this spec is "run it, triage, make findings catchable per-PR, make it repeatable" — not
        build a harness. The durability move (convert every crash into a deterministic regression
        test in the NORMAL suite) + the CI decision are set here; SPEC-068 ranked this #1.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-069: run the decoder fuzz gate (AVIF / SVG / RAW / HEIC)

## Context

PROJ-009 gave crustyimg its input reach — default pure-Rust decode of AVIF (`re_rav1d`, SPEC-058),
SVG (`resvg`, SPEC-060), and RAW embedded-preview extraction (SPEC-061), plus opt-in HEIC
(`libheif`, SPEC-062). Each shipped with a `fuzz/` target and corrupt-input + cap unit tests — but
the **fuzz targets were never actually run**: there was no nightly toolchain / `cargo-fuzz` in the
build+verify envs. The roadmap names this a **pre-1.0 gate** in plain terms ("anything with a fuzz
target must have it actually *run* before 1.0"; *"the long pole is the corpus + fuzzing, not the
code"*).

SPEC-068's threat-model review ranked this **#1, High**: it is the one untrusted-binary surface a
read-only review cannot close — a decoder either panics/hangs/OOMs on a crafted file or it doesn't,
and only fuzzing tells you. The decode path is where crustyimg touches the least-trusted input (raw
bytes from a file it did not write), behind the DEC-034/DEC-035 caps. This spec finally runs the
gate, fixes anything it finds, and — crucially — makes both the findings and the gate itself
**repeatable and catchable by ordinary CI**, so "fuzzed once" becomes "stays fuzzed."

## Goal

Run the four fuzz targets (`avif_decode`, `svg_decode`, `raw_preview`, `heic_decode`) on a nightly
toolchain via `cargo fuzz`, seeded from the committed fixtures, for a **documented budget**; triage
every crash/panic/hang/OOM into a fix, a tightened cap, or a documented upstream-guarded case
(nothing left unaddressed); **convert each finding into a deterministic regression test in the
normal test suite** (driving `Image::from_bytes` / `raw_preview` with the minimized crashing bytes →
typed error, never a panic) so 3-OS CI catches a regression without the fuzzer; add a `just fuzz`
recipe + a committed seed corpus + a written run record; and emit **DEC-062** fixing the fuzz-gate
policy (run mechanism, budget, triage rules, the regression-conversion durability move, the CI
decision, HEIC's best-effort scope). **A clean run — zero crashes for the budget — is a first-class,
sufficient outcome**; do not manufacture findings. No new default dependency.

## Inputs

- **The existing harness (read first — it is 90% built):**
  - `fuzz/Cargo.toml` — the **detached** workspace (empty `[workspace]` so the repo-root build/CI
    never touch it), `libfuzzer-sys` dep, the four `[[bin]]` targets, and the **run + seed recipes
    already written as comments** (`cargo +nightly fuzz run <target> -- -runs=100000`; seed dir
    `../tests/fixtures/<fmt>`; HEIC adds `--features heic`).
  - `fuzz/fuzz_targets/{avif_decode,svg_decode,raw_preview,heic_decode}.rs` — each is `#![no_main]`
    + `fuzz_target!(|data: &[u8]| { let _ = <entry>(data); })`. AVIF/SVG/HEIC route through
    `crustyimg::image::Image::from_bytes`; RAW calls `crustyimg::image::raw_preview` directly
    (RAW is extension-routed in `Image::load`, so `from_bytes` never reaches it). **Contract: never
    panic — `Ok`/`Err` both acceptable; a panic/abort/hang/OOM is the only failure.**
  - `tests/fixtures/{avif/solid_16x16.avif, svg/rect_text_40x30.svg, raw/synthetic_preview.nef,
    heic/solid_64x48.heic}` — the seed corpus (one valid sample each; grow if a target needs more
    coverage to get past the format sniff).
- **The decoders under test + their caps:** `src/image/avif.rs`, `src/image/svg.rs`,
  `src/image/raw.rs`, `src/image/heic.rs`; `Image::from_bytes` (content-sniff dispatch) and
  `raw_preview`; `DEC-034` (decode dimension/alloc caps) + `DEC-035` (symlink/resource guards) —
  the mitigations a crash would mean are insufficient. `MAX_PREVIEW_CANDIDATES` bounds RAW decode
  work; `resvg` uses `resources_dir=None` + no external refs (SPEC-060).
- **Where regressions land:** `tests/input_avif.rs`, `tests/input_svg.rs`, `tests/image_load.rs`
  (per-format hostile-input tests already live here) — add a `tests/fuzz_regressions.rs` or extend
  these; commit each minimized crash input under `tests/fixtures/fuzz/<target>/`.
- **Toolchain (the real prerequisite):** nightly Rust (`rustup toolchain install nightly`) +
  `cargo install cargo-fuzz`. Runs on the dev **macOS** host (libFuzzer works on macOS; Apple-silicon
  ASAN is best-effort). HEIC needs a system `libheif` (`brew install libheif`, incl. the libde265
  plugin) built with `--features heic`.
- **Roadmap gate lines:** `docs/roadmap.md` (the pre-1.0 "fuzz target must actually run" section,
  ~L120-128) — this spec discharges them.

## Outputs

- **Files created:**
  - `docs/research/proj-009-fuzz-run.md` (or `proj-007-fuzz-run.md`) — the **run record**: toolchain
    version, per-target budget actually achieved (execs + wall time), seed corpus used, findings
    (each: input, root cause, disposition), and the **exact repeat recipe**. This is the artifact
    that turns "ran once" into "re-runnable"; a future run appends.
  - `decisions/DEC-062-*.md` — the fuzz-gate policy: the run mechanism + budget floor, the triage
    rules, the **regression-conversion durability rule** (every crash → a normal-suite test +
    committed fixture), the CI decision (below), HEIC's best-effort scope, and the 1.0-gate contract.
  - `tests/fuzz_regressions.rs` (+ `tests/fixtures/fuzz/<target>/*`) — one deterministic test per
    crash found, feeding the minimized bytes to the real entry point and asserting a typed error /
    no panic. **If a target is clean, it has no regression test — that is expected**; instead add the
    cheap always-on smoke below.
  - `just fuzz` recipe (in the `justfile`) — wraps the run/seed commands (e.g. `just fuzz <target>
    [runs]`), documented, so the gate is one command not a remembered incantation.
  - (decision in DEC-062) an optional **non-blocking** CI workflow (`workflow_dispatch` +/or
    scheduled, nightly toolchain, short smoke budget) — see below; the *required* per-PR protection
    is the regression tests + smoke, NOT a per-PR fuzz job (fuzzing is time-boxed, not a fast gate).
- **Files modified (only if the fuzzer finds something):**
  - the relevant `src/image/*.rs` decoder or a `DEC-034`/`DEC-035` cap — the minimal fix/guard for
    each real crash; an upstream-library crash we can't fix gets a boundary guard + an upstream report
    noted in the run record (or a documented, justified `#[ignore]` if truly unfixable by us).
  - `docs/roadmap.md` — tick the pre-1.0 fuzz-gate item as run (with a pointer to the run record).
- **New exports:** none (a gate + tests + docs, not an API change).

## Acceptance Criteria

- [ ] The **three default-path targets** (`avif_decode`, `svg_decode`, `raw_preview`) each ran on a
  nightly `cargo fuzz` build, **seeded from `tests/fixtures/<fmt>`**, for a documented budget
  (floor: `-runs=1000000` OR `-max_total_time=600` per target, whichever the run records — bump if a
  target is trivially clean fast), with **no crash surviving** at the end. The budget actually
  achieved is recorded, not just the target.
- [ ] `heic_decode` is run **best-effort**: if the local env builds `--features heic` (system
  libheif present), run it to the same budget; if not, the run record documents why it was skipped.
  HEIC findings are triaged as "guard at our boundary / report upstream" (its memory safety is
  libheif's, a C dep — DEC-052), not necessarily a crustyimg code fix.
- [ ] **Every crash/panic/hang/OOM found is addressed** — a decoder fix, a tightened DEC-034/035 cap,
  or a documented upstream-guarded case — AND **pinned by a deterministic regression test in the
  normal suite** (the minimized bytes → typed error / no panic), with the input committed under
  `tests/fixtures/fuzz/`. No finding left as a note only. (The SPEC-068 lesson: a finding without a
  driven regression is unverified.)
- [ ] A **clean run is accepted as complete** — if a target surfaces nothing for its budget, the
  deliverable is the recorded clean run + the reproducible recipe + the seed corpus; no invented fix.
- [ ] The gate is **repeatable**: `just fuzz <target>` runs it; the run record documents the exact
  toolchain + commands; the seed corpus is committed. A cheap always-on smoke — the seed fixtures +
  any committed crash inputs decode-or-typed-error **without panic** through `Image::from_bytes` /
  `raw_preview` — runs in the **normal** `cargo test` (so 3-OS CI exercises the crash corpus every PR
  even though the fuzzer itself does not).
- [ ] **DEC-062** records the policy (mechanism, budget floor, triage rules, regression-conversion,
  the CI decision, HEIC best-effort, the 1.0 gate). `docs/roadmap.md`'s fuzz-gate item is ticked.
- [ ] **No new default dependency** (`libfuzzer-sys`/`cargo-fuzz` stay in the detached `fuzz/`
  workspace + dev toolchain; the repo-root `Cargo.toml`/`Cargo.lock`/`deny.toml` are untouched —
  `git diff main -- Cargo.toml Cargo.lock deny.toml` empty). Full gate matrix green incl. lean build;
  no `unwrap` on recoverable paths in any fix.

## Failing Tests

Written as findings appear (the crashes are unknown until the fuzzer runs) + one deterministic
always-on smoke that can be written now.

- **`tests/fuzz_regressions.rs`** — one per crash the fuzzer finds: feed the **minimized** crashing
  bytes (committed under `tests/fixtures/fuzz/<target>/`) to `Image::from_bytes` (AVIF/SVG/HEIC) or
  `raw_preview` (RAW) → assert a typed `Err` (or `Ok`), and **no panic**. Each must fail before its
  fix/guard and pass after (mutation-check the guard the way SPEC-064 did).
- **Always-on corpus smoke (write during design/build, runs in normal `cargo test`):**
  `"fuzz_corpus_never_panics"` — iterate every file in `tests/fixtures/{avif,svg,raw,heic}` and
  `tests/fixtures/fuzz/**`, run it through the matching entry point, assert no panic (Ok/Err both
  fine). This is the per-PR guard that the crash corpus stays non-panicking on all 3 OSes without the
  fuzzer. (HEIC entries are gated behind `#[cfg(feature = "heic")]`.)

## Implementation Context

*Read this and re-confirm anchors against the current tree. The harness is built; the work is
running it, triaging, making findings durable, and recording it.*

### The run (macOS dev host)
```
rustup toolchain install nightly
cargo install cargo-fuzz
# from fuzz/ (detached workspace):
cargo +nightly fuzz run avif_decode  ../tests/fixtures/avif  -- -runs=1000000   # seed + run
cargo +nightly fuzz run svg_decode   ../tests/fixtures/svg   -- -runs=1000000
cargo +nightly fuzz run raw_preview  ../tests/fixtures/raw   -- -runs=1000000
cargo +nightly fuzz run --features heic heic_decode ../tests/fixtures/heic -- -runs=1000000  # best-effort
```
A crash writes `fuzz/artifacts/<target>/crash-*`; reproduce with `cargo +nightly fuzz run <target>
<artifact>`; **minimize** with `cargo +nightly fuzz tmin <target> <artifact>` before committing it as
a fixture. libFuzzer stops on the first crash — loop: fix/guard → re-run to clear that crash → until
the budget completes clean.

### Triage policy (each crash → exactly one)
1. **Our bug** (a panic/`unwrap`/index in `src/image/*.rs` or a caller) → fix at the boundary.
2. **Missing/loose cap** (an input that OOMs/hangs within our code because a dimension/alloc/candidate
   bound is absent or too high) → tighten DEC-034/DEC-035; the fix is the cap, the test is the bomb.
3. **Upstream library crash** (inside `re_rav1d`/`resvg`/`image`/`libheif`) → guard at our call site
   if we can (catch/limit), report upstream (note the issue in the run record), and pin a regression
   that asserts *our* contract (typed error, no panic) — or, if genuinely unfixable-by-us and
   non-reachable in practice, a documented + justified `#[ignore]`. Never silently drop it.

### Durability (the point of the spec)
The fuzzer runs on nightly, off the per-PR path. So every crash becomes a **normal-suite** regression
(committed minimized input → `from_bytes`/`raw_preview` → no panic) that 3-OS `cargo test` runs every
PR. The `fuzz_corpus_never_panics` smoke sweeps the whole committed corpus. This is what keeps the
surface fuzzed after this spec ships.

### The CI decision (make it in DEC-062)
Recommended: a **manual/scheduled** `workflow_dispatch` (+ optional weekly `schedule`) job on the
nightly toolchain running a **short** smoke budget per target — visibility without gating PRs on a
slow, nondeterministic job. NOT a required per-PR check. If CI nightly proves flaky/costly, the
regression tests + corpus smoke are the durable floor and the CI job is optional — decide and record.

### HEIC scope (DEC-052 lineage)
HEIC is opt-in, decodes via C `libheif`. Fuzz it if the env builds it, but its crashes are largely
libheif's; our contract is "no panic on the Rust side + the default build answers `.heic` with
`CodecNotBuilt`→exit 4." Don't block the spec on libheif internals or an ASAN HEIC run (bonus if it
works). The three default pure-Rust targets are the required bar.

### Constraints that apply
- `untrusted-input-hardening` (this is its highest-severity instrument), `no-unwrap-on-recoverable-
  paths` (any fix), `no-new-top-level-deps-without-decision` (none — fuzz tooling is detached/dev),
  `every-public-fn-tested`, `clippy-fmt-clean`. The repo-root build/CI must stay green and untouched
  by `fuzz/`.

### Out of scope (for this spec)
- OSS-Fuzz onboarding (a bigger, separate effort — note it as a future option in DEC-062); structural
  decoder rewrites; fuzzing non-decode surfaces (the manifest/lock/cache parsers — SPEC-068 already
  drove those with hostile files; a `cargo fuzz` target for them is a possible future item, not here);
  a corpus-minimization/coverage-tracking pipeline beyond seeding + `tmin`.

## Notes for the Implementer

- **Install the toolchain first and confirm a target builds** (`cargo +nightly fuzz build`) before a
  long run — nightly + `cargo-fuzz` + (for HEIC) `brew install libheif` is the real setup cost.
- **A clean run is success.** If nothing crashes for the budget, record it and ship the gate + recipe
  + smoke. Don't gold-plate or invent a fix. (Mirror SPEC-068's "a dismissed suspect is a result.")
- **Minimize before committing** a crash input (`cargo fuzz tmin`) — a 4-byte reproducer is a better
  fixture than a 40 KB one, and it makes the regression test legible.
- Keep everything out of the repo-root workspace; verify `git diff main -- Cargo.toml Cargo.lock
  deny.toml` is empty at the end.
- Emit `DEC-062` with `affected_scope` covering `fuzz/`, `tests/fuzz_regressions.rs`, `justfile`,
  `docs/`, and any `src/image/*.rs` a fix touched.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-069-fuzz-gate`
- **PR (if applicable):** _(opened at end of build; see PR link)_
- **All acceptance criteria met?** **yes**, with one documented upstream deviation
  (F-AVIF-3, below). The 3 default targets ran seeded to the budget floor; every
  finding is fixed-at-boundary or documented-upstream + pinned by a regression /
  the corpus smoke; `just fuzz` + run record + DEC-062 + roadmap tick landed;
  `git diff main -- Cargo.toml Cargo.lock deny.toml` is **empty**.
- **Toolchain installed:** host had Homebrew Rust (stable 1.94.1), **no rustup** →
  installed rustup (`--no-modify-path`) + nightly `rustc 1.99.0-nightly`, then
  `cargo install cargo-fuzz` (`0.13.2`). ASAN on aarch64-apple-darwin. libheif
  `1.23.1` (Homebrew) for HEIC. See run record for the setup note.
- **New decisions emitted:**
  - `DEC-062` — decoder fuzz-gate policy (run mechanism = `-O` release config +
    `just fuzz`, budget floor `-max_total_time=600`, triage rules, the
    regression-conversion durability rule, the non-blocking CI decision, HEIC
    best-effort scope, the 1.0-gate contract).
- **Findings (per target):** (full detail: `docs/research/proj-009-fuzz-run.md`)
  - **avif_decode** — budget: `-O` ran to ~5.5k execs (cov 6988) before an upstream
    OOM; default config can't reach a clean budget (upstream debug-asserts abort
    libFuzzer regardless of `catch_unwind`). **3 findings, all upstream `avif-parse`
    2.1.0** (none in `re_rav1d`/our glue): (1) `check_parser_state` `debug_assert!`
    panic → **fixed** (`box_sizes_fit` + `catch_unwind`), 2 regressions; (2) 3.09 GB
    box-size-bomb OOM → **fixed** (`box_sizes_fit`), regression; (3) ~4.26 GB nested
    meta `with_capacity` over-allocation → **documented upstream** (bucket c,
    reproducer recorded not committed — see deviation). Plus `frame_size_limit`
    decode-stage hardening.
  - **svg_decode** — **CLEAN**: 2,390,202 runs / 601 s, no crash (cov 12296).
  - **raw_preview** — **CLEAN**: 1,812,513 runs / 602 s, no crash (cov 2626).
  - **heic_decode** — **ran best-effort** (system libheif, ASAN): libheif decode
    exercised (~107k execs, cov 1274), **no HEIC/libheif finding**; both configs
    bounded by the shared `from_bytes` AVIF-first dispatch reaching avif-parse's
    documented issues.
- **Deviations from spec:**
  - **F-AVIF-3 not converted to a committed regression / smoke input.** The nested
    `avif-parse` meta over-allocation provokes a multi-GB allocation; committing it
    to the always-on `fuzz_corpus_never_panics` smoke risks OOM-killing CI. It is
    instead recorded by sha256 in the run record and reported upstream. Its class is
    covered by the committed `box_sizes_fit` regressions for the fixable sibling.
  - **Two build configs, not one.** The spec's recipe implies the default cargo-fuzz
    config; the shipped binary is `-O`/release (debug-assertions off). AVIF's upstream
    `debug_assert!`s abort libFuzzer in the default config regardless of our
    `catch_unwind`, so `just fuzz` and the run use `-O` (production-representative) as
    the canonical config, with the distinction documented in DEC-062 + the run record.
- **Follow-up work identified:**
  - Report the two `avif-parse` 2.1.0 robustness classes upstream (debug-assert on
    recoverable state; size/count fields sizing allocations before bounding by input).
  - A **dedicated HEIC-only decode fuzz entry** (bypass the AVIF content-sniff) so
    `heic_decode` fuzzes libheif in isolation instead of being diverted to the AVIF
    path (STAGE-024 backlog).
  - Optional: wire the sanctioned non-blocking `workflow_dispatch`/scheduled nightly
    fuzz-smoke CI job (DEC-062); OSS-Fuzz onboarding.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — The spec assumed a rustup host with a single fuzz config; the real host was
   Homebrew-Rust (no rustup), and — more consequentially — cargo-fuzz's **default**
   config runs with debug-assertions ON, which fires upstream `avif-parse`
   `debug_assert!`s that are compiled out of the shipped release binary and that
   libFuzzer aborts on *regardless* of a downstream `catch_unwind`. Realizing the
   canonical run must be `-O` (matching production) took an empirical rebuild-and-
   reproduce cycle.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — A note that the three ISOBMFF decoders (AVIF, HEIC) share the `from_bytes`
   sniff/dispatch, so the `heic_decode` target cannot be fuzzed in isolation from the
   AVIF path — and guidance on whether an upstream-dependency OOM that is only
   reachable as a multi-GB allocation should be committed to the always-on smoke
   (decided here: no — record it, don't risk CI).

3. **If you did this task again, what would you do differently?**
   — Run each target in `-O` first (production config) to separate real
   memory-safety findings from upstream debug-assert noise, and probe the crash's
   *stage* (container-parse vs decode) before writing a fix — my first AVIF fix
   (`frame_size_limit`) targeted the decode stage while the OOM was in the container
   parser; a one-line stage probe would have pointed straight at `box_sizes_fit`.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
