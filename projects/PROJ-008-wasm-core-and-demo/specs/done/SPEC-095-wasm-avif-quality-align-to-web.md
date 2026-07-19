---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-095
  type: story
  cycle: ship
  blocked: false
  priority: medium
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-18
references:
  decisions: [DEC-068, DEC-069, DEC-070, DEC-064]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation]
  related_specs: [SPEC-079, SPEC-084, SPEC-080, SPEC-085]
value_link: >
  Make the demo a FAITHFUL preview of `crustyimg web`, not an over-flattering one. The wasm AVIF path
  encodes at q80 while native `web` uses q85 — so the demo shows a SMALLER file than the CLI actually
  produces, overstating the savings a visitor gets when they install. Aligning the wasm default AVIF
  quality to native's `FAST_LOSSY_QUALITY` closes DEC-069, lets the demo claim "the same engine and
  quality" instead of "approximates", and is the strongest-product move for the launch: honest, and the
  one place the "same engine" story is currently literally weaker.

cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 900000
      duration_minutes: null
      estimated_usd: 5.4
      note: >
        Estimated order-of-magnitude (no clean subagent metering available for a main-loop build
        session run directly in the primary checkout, per AGENTS.md worktree-per-session guidance) —
        ~80/20 input/output at Sonnet list rate ($3/$15 per MTok), no cache discount.
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 400000
      duration_minutes: null
      estimated_usd: 3.6
      note: >
        Estimated order-of-magnitude (main-loop verify run directly in the primary checkout, not a
        separately-metered subagent, per the autonomous-run cost convention) — ~80/20 input/output at
        Opus 4.8 list rate ($5/$25 per MTok), no cache discount. Heavy compute (3× full --features avif
        runs, wasm rebuilds, demo-smoke, npm-smoke) is wall-clock, not tokens.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.45
      recorded_at: 2026-07-18
      note: >
        orchestrator main loop (un-metered, §4) — ESTIMATE. Framed the spec (grounded the q80-vs-q85
        divergence in the code), dispatched build (Sonnet) → verify (Opus, CLEAN). Spot-checked the
        anchor-not-literal change + native-sink-untouched + demo copy before verify, then confirmed CI
        green on the verify commit and squash-merged (992f797). This time correctly stayed on main /
        out of the repo during the build+verify (the SPEC-080 slip lesson held). Model experiment n=2:
        Sonnet build indistinguishable from Opus on the hard parts.
  totals:
    tokens_total: 1300000
    estimated_usd: 9.45
    session_count: 3
---

# SPEC-095: align the wasm default AVIF quality to native `web` (q80 → q85)

## Context

The demo (SPEC-080) mirrors the flagship `web` flow, but has to hedge that it *"approximates"* it — and
the reason is a single quality constant. Grounded in `src/wasm.rs` (2026-07-18):

- **Native `web`** runs `Mode::Fast` and encodes AVIF at **`FAST_LOSSY_QUALITY = 85`** (`src/sink/mod.rs:80`;
  the eyeball-validated two-regime sweet spot from SPEC-084).
- **The wasm demo path** (`optimize_detailed`, `src/wasm.rs:389`) resolves AVIF for a photo, then in the
  quality match falls into the **`(_, true) => (None, None)` arm** (`~459`) — the no-budget, not-
  perceptually-searchable case, which is exactly AVIF (it has an encoder but no wasm decoder, DEC-065, so
  it cannot be SSIMULACRA2-searched; JPEG takes the perceptual branch at `~454`). `quality = None` →
  `encode_to_bytes_with(..., None, speed)` → **`AVIF_DEFAULT_QUALITY = 80`** (`src/sink/mod.rs:676`).

So the demo encodes AVIF **5 quality points below** what `crustyimg web` produces. The **format decision
already converged** (SPEC-084/079 — Auto picks AVIF the same way on both); only the quality knob diverges.
This is the entirety of **DEC-069**'s native(85)/wasm(80) divergence.

**Why fix it (honesty, not vanity):** at q80 the demo produces a *smaller* AVIF than the CLI. A visitor
who drops a file in the demo, is impressed, installs the tool, and re-runs `crustyimg web` gets a
slightly **larger** file than the demo promised. HN is precisely the crowd that will do that. Matching
q85 makes the demo show **what you actually get** — the savings stay huge (`web` ≈ 98%), and the demo
earns the stronger claim.

**Same constant, other surface:** the older `optimize` binding (`src/wasm.rs:230`) has the identical
`None → q80` for AVIF. The demo uses `optimizeDetailed`, but align **both** for consistency (or document
why not) — the same reasoning applies.

## Goal

Make the wasm surface's **default (no-budget, no-perceptual-search) lossy AVIF quality equal to native
`web`'s `FAST_LOSSY_QUALITY` (85)**, so a demo conversion produces the same-quality AVIF as `crustyimg
web` on the same (downscaled) image. Rebuild the packaged wasm, update the demo's honesty copy from
"approximates" to an accurate "same engine + quality" statement, and resolve DEC-069.

## Inputs — files to read

- `src/wasm.rs` — `optimize_detailed` (~389) and its quality `match` (~447–461); the `(_, true) =>
  (None, None)` arm (~459) is the AVIF-default site. The `optimize` twin (~229–231). `OptimizeResult`
  (~256) — confirm `.quality` surfaces the value once it's `Some(85)`.
- `src/sink/mod.rs` — `FAST_LOSSY_QUALITY = 85` (~80), `AVIF_DEFAULT_QUALITY = 80` (~59), and the
  `encode_to_bytes_with(_, _, quality, speed)` entry (~676) the wasm path calls. **Confirm which constant
  `web`/`Mode::Fast` actually uses** so the wasm value is anchored to the same source, not a hardcoded 85.
- `projects/.../specs/done/SPEC-084-*` (the q85 rationale), `decisions/DEC-069*` (the divergence to close),
  `decisions/DEC-070*` (`web`).
- `demo/` (as merged after SPEC-080) — the "approximates `web`" copy to tighten (index.html + README +
  the funnel line in demo.js).

## Outputs

- **`src/wasm.rs`** — the default lossy-AVIF quality (the `(_, true)` no-search arm in `optimize_detailed`,
  and the AVIF case in `optimize`) uses **`FAST_LOSSY_QUALITY`** (imported from `sink`, not a literal 85),
  so wasm and native `web` share one source of truth. `OptimizeResult.quality` now reports **85** for a
  default photo→AVIF conversion. **No change to `convert`/`AVIF_DEFAULT_QUALITY = 80`** (byte-identity
  contract, DEC-071) and **no change to native** anything — this is a wasm-default change only.
- **Rebuilt wasm** — `just wasm-build` (the size-profiled build, DEC-066); update the demo's vendored
  `pkg/` accordingly. **Confirm the bundle-size + wasm smoke still pass** (the packaged `.wasm` must stay
  the profiled artifact).
- **`demo/`** — replace the "approximates `web`" hedge with an accurate claim: the demo uses **the same
  engine and the same AVIF quality (q85)** as `crustyimg web`. Be precise about what remains honest to
  say (the demo runs `web`'s recipe geometry + q85 in-browser; it is not guaranteed *byte-identical* to
  the CLI because the wasm build is no-asm rav1e vs the native encoder — say "same settings, same engine,"
  not "identical bytes").
- **DEC** — resolve/amend **DEC-069**: the native/wasm AVIF-quality divergence is closed; wasm anchors to
  `FAST_LOSSY_QUALITY`. (Amend DEC-069 in place, or a short new DEC that supersedes its divergence note.)

## Acceptance Criteria

- [x] A default photo→AVIF `optimizeDetailed` on wasm encodes at **q85** (`OptimizeResult.quality == 85`),
      anchored to `FAST_LOSSY_QUALITY` (not a literal) — proven in a wasm roundtrip test.
      (`wasm_default_avif_quality_is_web_fast_quality`, `tests/wasm_roundtrip.rs`.)
- [x] The wasm AVIF output for a given (2048px) image is the **same quality setting** as `crustyimg web`
      on the same image — verify the quality parameter matches; the encoded bytes are close but need NOT
      be byte-identical (no-asm rav1e). Grade the AVIF with an independent decoder (`sips`), confirm valid.
      (Same test byte-compares against an independent q85 encode; `just demo-smoke`'s
      `default_is_web_flow_smaller_avif` independently confirms `sips` reads the demo's real hero output as
      valid AVIF; a manual `convert -q 80` vs `-q 85` pair was also `sips`-graded.)
- [x] **`convert` is byte-identical** to before (native `AVIF_DEFAULT_QUALITY = 80` untouched) — prove
      against the pre-spec binary. Native behavior is entirely unchanged.
      (`convert_avif_default_unchanged`, `tests/wasm_roundtrip.rs`, drives the real binary; the existing
      `sink::tests::convert_avif_bytes_unchanged_at_default` unit test also still holds unmodified.)
- [x] `just wasm-build` succeeds, the packaged `.wasm` is the size-profiled artifact (strip fingerprint
      per SPEC-075's structural guard), and `just wasm-npm-smoke` / the demo browser smoke stay green.
      (Both ran green; `wasm-npm-smoke` confirmed the strip fingerprint and brotli size stayed within the
      DEC-066 baseline.)
- [x] The demo copy states the accurate claim (same engine + q85, not "approximates"; honest that bytes
      aren't guaranteed identical) — driven in the demo smoke if it asserts funnel/README text.
      (`tests/demo_copy.rs`, new; the existing `demo-smoke` does not assert this specific copy so no
      browser-smoke change was needed.)
- [x] DEC-069 resolved/amended; `just decisions-audit` clean.
      (Amended in place — resolution note + `affected_scope` extended to `src/wasm.rs`/`demo/**`; audit
      exits 0, only pre-existing advisory scope-overlap warnings.)
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
      `cargo build --no-default-features`, `just validate` pass.
      (All green; see Build Completion for a note on one transient full-suite flake that did not
      reproduce.)

## Failing Tests (written at design)

- **`tests/wasm_roundtrip` (or the wasm test path)**
  - `wasm_default_avif_quality_is_web_fast_quality` — a default photo→AVIF `optimizeDetailed` reports
    `quality == FAST_LOSSY_QUALITY` (85). **Fails today** (reports 80 / None) — the proof the change bit.
  - `wasm_avif_quality_anchored_not_hardcoded` — the value tracks `FAST_LOSSY_QUALITY` (change the const
    in a thought-experiment / assert the same symbol), not a literal 85 in wasm.rs.
- **Native regression**
  - `convert_avif_default_unchanged` — native `convert` → AVIF is byte-identical to pre-spec (q80 path
    untouched); the `AVIF_DEFAULT_QUALITY` byte-identity test (`sink/mod.rs:961`) still holds.

## Implementation Context

### Decisions that apply
- `DEC-069` — the divergence this closes. `DEC-070` — `web` (the quality target to match). `DEC-071` —
  `convert`'s q80 byte-identity, which must **not** move. `DEC-068`/`DEC-064` — the wasm `optimizeDetailed`
  surface + cfg boundary.
- SPEC-084 — why the fast default is q85 (`FAST_LOSSY_QUALITY`), separate from `convert`'s q80.

### Constraints
- `ergonomic-defaults` — the demo default must be what the CLI actually gives, not a flattering variant.
- `every-public-fn-tested` / `test-before-implementation`.

### Out of scope (this spec)
- Any change to native `convert` / `AVIF_DEFAULT_QUALITY` (q80 byte-identity stays).
- Lossy-WebP-on-wasm, AVIF-decode-on-wasm, threads/SIMD (separate/blocked concerns).
- The SPEC-080 demo structure (shipped) and SPEC-081 score UI — this only tightens the quality + the
  honesty copy.

## Notes for the Implementer
- **Anchor to the constant, don't hardcode 85** — import `FAST_LOSSY_QUALITY` so wasm and native can
  never silently drift again ([[a-plausible-test-result-is-not-a-checked-one]] in spirit: one source of
  truth, not two literals that "happen to match").
- **Prove the native side did NOT move** — `convert` byte-identity is the guardrail; run it against the
  pre-spec binary, not just the unit test.
- **The wasm rebuild is part of the deliverable** — a code change without `just wasm-build` ships the old
  `.wasm` ([[verify-includes-lean-no-default-features-build]] cousin: the packaged artifact is what runs).
- **Sequencing:** build this **after SPEC-080 merges** — it edits the merged demo's honesty copy. (Framed
  2026-07-18 while SPEC-080 was in verify.)

---

## Build Completion
- **Branch:** `spec-095-wasm-q85` · **PR:** (opened against `main`, not yet merged) · **All acceptance
  criteria met?** Yes, all seven. · **New decisions:** none (amended DEC-069 in place, per the spec's
  own guidance — no new DEC file). · **Deviations:**
  - Deleted `default_quality_for` (the `.or_else` fallback in `optimize_detailed`) instead of leaving it
    in place: once the `(_, true)` catch-all arm returns `Some(FAST_LOSSY_QUALITY)` directly, its one
    call site can never see a `None` for AVIF, making the whole function permanently unreachable —
    `no-dead-code` calls for deleting rather than leaving an inert fallback.
  - Split `optimize()`'s single `if disposition == Lossless || !supports_perceptual_quality()` guard into
    two `if`s so only the AVIF (lossy, non-perceptual) case gets `Some(FAST_LOSSY_QUALITY)`; the lossless
    case still passes `None` (lossless formats have no quality knob to move).
  - Added a native regression test (`convert_avif_default_unchanged`) driving the real `convert` binary
    end-to-end, on top of the spec-named unit-level anchor, and a mechanical grep test
    (`wasm_avif_quality_anchored_not_hardcoded`) proving no bare `85` literal survives in `src/wasm.rs`
    outside the `FAST_LOSSY_QUALITY` symbol — both go beyond the spec's named test list but follow
    directly from its "anchor to the constant, don't hardcode 85" instruction.
  · **Follow-ups:** none identified.

### Build-phase reflection
1. **What was the trickiest part?** Proving the fix actually changed the ENCODED BYTES, not just the
   reported `quality` field — the failure mode this spec exists to prevent (report 85, still encode 80)
   is exactly the kind of plausible-but-unchecked claim the project's own memory warns about. The test
   independently re-encodes at `FAST_LOSSY_QUALITY` and asserts byte equality against
   `optimize_detailed`'s output, not just the label.
2. **What surprised you?** `default_quality_for`'s `.or_else` fallback turned out to be fully vestigial
   after the fix — every reachable AVIF path through `optimize_detailed` now populates `quality` directly
   from the match, so the fallback function could never fire even before I deleted it. Worth flagging
   because a reviewer scanning only the diff's `+`/`-` might read the deletion as unrelated cleanup
   rather than a direct consequence of the fix.
3. **What would you do differently?** Would grep for `--yes`/`--format` flag spellings against `--help`
   output before writing the native `convert_avif_default_unchanged` test — I first guessed `--to` (this
   crate's flag is `--format`), which would have been a build-time compile pass but a runtime failure at
   test time; checking `--help` first would have saved a cycle.

**Note on a transient full-suite flake:** one intermediate `cargo test --features avif` run (of several)
showed 8 failures in `tests/cli.rs` (AVIF-feature-detection mismatches) plus, separately, my own new
`convert_avif_default_unchanged` failing once with a bare "must succeed". Both reproduced as CLEAN in
isolation (`cargo test --features avif --test cli`: 126/126; `--test wasm_roundtrip
convert_avif_default_unchanged` run 3×: 3/3) and CLEAN on two subsequent full-workspace reruns (764/764
default, 777/777 avif — twice). Read as environment/build-parallelism flakiness under this session's
concurrent background builds, not a code defect; flagging for verify to weigh rather than silently
omitting.

---

## Verify Verdict — ✅ APPROVED (Opus, 2026-07-18)

Verified in the primary checkout (branch `spec-095-wasm-q85`), driven with negative controls.

1. **The demo's real output is q85, from a fresh-from-HEAD wasm.** `just demo-smoke` rebuilds the wasm
   from HEAD source (the exact path GitHub Pages deploys — `demo-build` → `wasm-build`, and the assembler
   refuses a non-profiled/stale `.wasm`, so the "packaged artifact is what runs" trap is structurally
   impossible: `pkg/` and `demo/vendor/` are gitignored and always regenerated). The hero AVIF read back
   as valid AVIF 2048×1536 by three independent decoders incl. `sips`. `just wasm-test`'s
   `wasm_default_avif_quality_is_web_fast_quality` proves in the **compiled wasm** that the returned bytes
   **byte-equal an independent `FAST_LOSSY_QUALITY` (85) encode** — not just the reported label. Mutating
   the `(_, true)` arm to `AVIF_DEFAULT_QUALITY` (80) makes that test **fail** (confirmed it bites).
2. **q85 is anchored, not coincidental.** Both wasm no-search sites (`optimize` legacy branch +
   `optimize_detailed`'s `(_, true)` arm) encode at `sink::FAST_LOSSY_QUALITY`; independent grep found no
   bare `85` literal in live code and 2 symbol refs (the build's `wasm_avif_quality_anchored_not_hardcoded`
   agrees). The `auto_avif_quality` `AVIF_DEFAULT_QUALITY` is an admission-only `Some(_)` signal whose
   number is discarded — not a missed encode site.
3. **Native `convert` is byte-identical.** Built the pre-spec parent binary (`main`, isolated worktree)
   and `cmp`'d `convert --format avif` output vs the HEAD binary: **byte-identical** (286 B, sips-valid).
   `src/sink/` untouched in the diff; the `convert_avif_bytes_unchanged_at_default` unit test holds
   unmodified. Negative control: `convert -q 80` (4732 B) ≠ `-q 85` (6268 B) on a real photo, both
   sips-valid — so byte-identity is a meaningful assertion and q80≠q85 is a real distinction.
4. **Flake CLEARED.** The build's transient full-suite `--features avif` failures did **not** reproduce:
   3× isolated full runs = 777 passed / 0 failed / exit 0 each; PR #99's CI `avif feature` job is green.
   The failure kind (exit-code assertions in feature-detection tests) is a stale/half-built CLI binary
   from `target/` contention under the build session's concurrent background builds — **not** a SPEC-095
   regression and **not** the SPEC-091 re_rav1d `DisjointMut`/`debug_abort` signature (which aborts the
   process; no abort/SIGABRT/panic markers appeared).
5. **Demo copy honest; DEC-069 accurate.** Copy states "same engine and the same AVIF quality (q85)" and
   explicitly disclaims byte-identity ("no-asm `rav1e`") — no overclaim; no "approximates"/"q80" hedge
   survives anywhere user-facing. DEC-069's divergence note is struck through and resolved to
   `FAST_LOSSY_QUALITY`, `convert` q80 unchanged.

**Gates:** `cargo test` (425+) ✅ · `cargo test --features avif` ✅ ×3 (777/0) · `cargo clippy`
default+avif ✅ · `cargo fmt --check` ✅ · `cargo build --no-default-features` ✅ · `just validate` ✅ ·
`just decisions-audit` (0 structural errors) ✅ · `just wasm-build`/`wasm-test`/`wasm-npm-smoke`
(strip fingerprint + brotli 1.33 MB within DEC-066 baseline) ✅ · `just demo-smoke` ✅ · PR #99 CI full
matrix green.

**Build-quality read (Sonnet build, Opus verify):** high. The build correctly identified the real trap
(report vs encode) and wrote a byte-equality test that bites; the `default_quality_for` deletion is a
sound dead-code consequence of the fix; the anchor-not-literal discipline is followed on both surfaces;
and the flake was flagged honestly for verify rather than buried. Nothing on the hard parts distinguished
it from an Opus build — consistent with the stage's n=1 finding.

---

## Reflection (Ship)
1. **A one-constant fix with an honesty payoff bigger than its diff.** The demo now shows what
   `crustyimg web` actually produces, not a q80 file 5 points smaller — so a visitor who installs the CLI
   gets what the demo promised, and the page earns "same engine + quality" instead of "approximates." This
   is the "strongest product out of the box" instinct paying off: the one place the pitch was literally
   weaker, closed. DEC-069 resolved; the value is anchored to `FAST_LOSSY_QUALITY` (one source of truth),
   not a second literal that "happens to match" — the drift that created the divergence can't recur.
2. **Model experiment n=2 (an engine+wasm change this time, not a pure move): Sonnet held.** The build
   nailed the actual trap (report-vs-encode: prove the *bytes* are q85, not just the label), added a
   byte-equality test that bites, kept anchor-not-literal on both surfaces, deleted dead code as a sound
   consequence, and flagged the transient flake honestly rather than burying it. Verify (Opus): "on the
   hard parts, indistinguishable from an Opus build." Build $5.40 / verify $3.60. The working rule is now
   well-supported across a pure move (089), a live-bug fix (094), and an engine/quality change (095):
   **build on Sonnet when the framing is tight and names the crux; verify on Opus.** ([[builder-model-choice-experiment]] n=2.)
3. **The "packaged artifact is what runs" trap was structurally impossible here — verify proved it, didn't
   assume it.** `pkg/`/`demo/vendor/` are gitignored and regenerated by the deploy path, which refuses a
   non-profiled/stale `.wasm`, so a code change can't ship an old artifact. Worth remembering for any
   future wasm-surface change: confirm the deploy assembles from HEAD, and (verify's discipline) grade the
   *demo's real output* with an independent decoder, not the source. The transient full-suite flake was
   `target/` contention under concurrent builds, NOT the SPEC-091 DisjointMut signature (which aborts, it
   doesn't fail an exit-code assertion) — a clean distinction worth keeping. Next: SPEC-081 (score diff UI).
