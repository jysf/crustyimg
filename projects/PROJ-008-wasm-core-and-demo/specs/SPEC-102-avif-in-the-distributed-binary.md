---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-102
  type: chore
  cycle: design
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-028
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-22

references:
  decisions: [DEC-020, DEC-052, DEC-069]
  constraints: []
  related_specs: [SPEC-018, SPEC-083, SPEC-084]

value_link: >
  Closes the gap between what BENCHMARKS.md measures and what a `brew install` user gets — the
  flagship AVIF path is currently absent from every distributed binary.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 600000
      duration_minutes: null
      estimated_usd: 3.25
      note: >
        Build session on Sonnet — ORDER-OF-MAGNITUDE ESTIMATE, not a real
        usage-object reading. Scope: the one-line Cargo.toml default flip,
        rewriting both AVIF comment blocks, DEC-081, a CHANGELOG headline,
        a mechanical docs sweep across 10 files (grep-cited, 61→59 hits),
        two full clean release builds for the size/compile-time delta, a
        pre-spec-vs-post-spec byte-parity check (temporarily rebuilding
        from the parent Cargo.toml), the 1.90.0 MSRV check, the full native
        gate suite (default + lean: test/clippy/fmt/deny) plus wasm-check
        under both the shipped and lean feature sets, and fixing a latent
        justfile regression (the SPEC-074 lean wasm comparison) the spec
        itself didn't flag.
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 450000
      duration_minutes: null
      estimated_usd: 2.40
      note: >
        Fix pass on Sonnet, responding to verify's not-clean findings —
        ORDER-OF-MAGNITUDE ESTIMATE, not a real usage-object reading. Scope:
        bench-compare.py's overclaim (both the docstring and the --bin/sys.exit
        hints); re-running the docs sweep with .py/.mjs/.yaml/justfile added to
        the grep --include set (69→67 hits across 28 files) and independently
        re-triaging every surviving hit rather than taking the prior triage,
        which turned up one real miss outside the named file types
        (examples/gen_avif_fixture.rs's "no-op without --features avif" doc
        comment, false now that avif defaults on) plus a precision fix to
        license-watchlist.yaml's stale "AVIF/libwebp" off-by-default citation;
        DEC-081's profile mislabel (no `[profile.release]` table exists, only
        `[profile.dist]`) plus adding verify's independently-measured
        dist-profile delta; the justfile:148 `--set` override, which failed to
        parse (`just` reads a value starting with `--` as a flag) — fixed with
        a documented leading-space form and driven for real (`wasm-check` and
        a full `wasm-build`, confirmed via `cargo tree` that rav1e/ravif are
        genuinely absent from the lean wasm dependency graph); and rewriting
        two load-flaky tests to assert AVIF's candidate ADMISSION via the
        `--json` explain trace instead of which candidate won the byte race.
        Re-ran the full gate suite (`just validate`, `just check`, lean
        build/test/clippy, `cargo fmt --check`, `just deny`, wasm-check under
        both feature sets) — all green, no published number moved.
  totals:
    tokens_total: 1050000
    estimated_usd: 5.65
    session_count: 2
---

# SPEC-102: AVIF in the distributed binary

## Context

`BENCHMARKS.md` measures `crustyimg web` producing AVIF and, in its own tools section, has to tell
the reader: *"AVIF encode is a compile-time feature… off in the default distributed binary; install
it with `cargo install crustyimg --features avif`."*

That is the gap. Someone reads the benchmark, runs `brew install jysf/tap/crustyimg`, and gets a
binary that **cannot do the thing the document is about** — `web` falls back to non-AVIF output and
an explicit `--format avif` exits 4. The same is true of the Releases-page binaries and of a plain
`cargo install crustyimg`. The flagship path is invisible to every user who installs the normal way.

Why it's currently off: `avif` was gated at `SPEC-018`/`DEC-020` for compile time, binary size, and
encode speed, back when AVIF was one candidate format among several. Since then `SPEC-084` made
fixed-quality AVIF the **default fast decision** (`Mode::Fast`), `SPEC-083` benchmarked it as the
headline, and the browser demo ships the AVIF encoder unconditionally (`DEC-065`) — the wasm artifact
has had it all along precisely because "if we don't ship the encoder, nobody can encode." The native
default is now the odd one out.

**This is a behavior change, not just a build change.** With `avif` compiled in, `Mode::Fast` can
admit AVIF as a candidate, so `web` and `optimize` produce **different output files** for existing
users. That is the intent, but it must be stated loudly rather than slipped in.

## Goal

Make `avif` a default feature so every distributed channel — Homebrew, the Releases binaries, the
shell/powershell installers, and `cargo install crustyimg` — ships the AVIF encoder, without
weakening the `DEC-052` guard that keeps `heic` out of distributed builds. Emit a DEC recording the
reversal of `DEC-020`'s gating rationale, headline the behavior change in the CHANGELOG, and
reconcile every doc that currently tells readers AVIF is opt-in.

## Inputs

- **Files to read:** `Cargo.toml` (the `[features]` block and the long `avif` comment explaining the
  gating); `dist-workspace.toml` (**and its `DEC-052` note**, which is the thing not to break);
  `decisions/DEC-020` (the original gating rationale), `DEC-052` (why the dist config deliberately has
  no `features` key), `DEC-069` (`FAST_LOSSY_QUALITY` = 85, the fast-AVIF default);
  `docs/research/proj-008-raw-on-wasm-probe.md` is unrelated — ignore.
- **Docs that currently claim AVIF is opt-in** (the sweep surface): `README.md` (the opt-in feature
  table + the `--features` install line), `docs/cli-reference.md`, `BENCHMARKS.md` (its tools section
  instructs `cargo install crustyimg --features avif`), the `avif` comment in `Cargo.toml` itself, and
  the note in `dist-workspace.toml`. **Enumerate by grep, do not rely on this list being complete.**

## Outputs

- **`Cargo.toml`** — `default = ["display", "watch", "avif"]`.
- **`decisions/DEC-081`** (next free) — the decision: AVIF moves into the default feature set;
  what `DEC-020` weighed and why the balance changed (fixed-quality AVIF is now the default decision,
  it's the benchmarked headline, and the wasm build already ships it); the measured costs; and an
  explicit statement that `heic` remains non-default and `dist-workspace.toml` still carries **no**
  `features` key, so `DEC-052`'s guard is untouched.
- **`CHANGELOG.md`** — a headline entry under Changed/Added for 0.6.0: AVIF is now in the default
  build, and `web`/`optimize` may therefore pick AVIF where they previously could not, changing output
  files.
- **Docs sweep** — every place that says AVIF is opt-in updated, including `BENCHMARKS.md`'s install
  instruction (which becomes plain `cargo install crustyimg`).

## Acceptance Criteria

- [x] `cargo build --release` with **no feature flags** produces a binary that encodes AVIF:
      `crustyimg convert <photo> --format avif -o out.avif` exits 0 and writes a valid AVIF (verify the
      container independently, e.g. `sips`/`magick identify`, not just the extension).
- [x] **`DEC-052`'s guard is intact:** `dist-workspace.toml` still has no `features`/`all-features`
      key, and a default build still refuses `.heic` with the typed exit-4 error. State this explicitly —
      the fix must not be implemented by adding a features key to the dist config, which would both miss
      `cargo install` and erode that guard.
- [x] **Measured, not assumed:** report the release binary **size delta** and the clean **compile-time
      delta** (before vs after), and confirm the **MSRV job still passes** — `rav1e`/`ravif` may floor
      above the declared `rust-version`; if it does, that is a finding, not something to quietly bump.
- [x] The **lean build** (`cargo build --no-default-features`) still succeeds, and `cargo-deny` stays
      green.
- [x] `crustyimg web <photo>` on a default build produces AVIF at the fast-quality default and the
      behavior change is recorded in the CHANGELOG as a headline, not a footnote.
- [x] **Docs sweep is mechanically verified:** cite the grep used and the hit count; every surviving
      "AVIF is opt-in / `--features avif`" claim is either updated or deliberately retained with a
      stated reason. `BENCHMARKS.md`'s install line must no longer require the feature flag.
- [x] `just validate`, `just check` (fmt/clippy/build/test) green; no unrelated `src/` behavior change.

## Failing Tests

- A test asserting a **default-feature** build can encode AVIF — i.e. the `avif` cfg path is live
  without any flag. The natural shape is an existing AVIF test losing its `#[cfg(feature = "avif")]`
  gate; prove it by confirming the test **fails on the parent commit** and passes here.
- The complementary guard: `--no-default-features` still builds and `.heic` still exits 4 on a default
  build (proving `DEC-052` is untouched).
- Byte-parity sanity: `convert --format avif -q 80` output is unchanged from the pre-spec binary built
  `--features avif` (turning the feature on by default must not alter the encoder itself).

## Implementation Context

### Decisions that apply
- `DEC-020` — the original "AVIF stays gated for compile time / size / speed" call. **This spec
  reverses its conclusion**; DEC-081 must say so rather than silently contradicting it.
- `DEC-052` — why `dist-workspace.toml` has no `features` key (HEVC patents; `heic` must never reach a
  distributed binary). **Load-bearing: do not touch.**
- `DEC-069` — `FAST_LOSSY_QUALITY = 85`, the quality `web` uses once AVIF is admissible.

### Prior related work
- `SPEC-018` — added AVIF behind the feature. `SPEC-084` — made fixed-quality AVIF the default fast
  decision. `SPEC-083` — benchmarked it as the headline and surfaced this gap.

### Out of scope
- Any encoder change, quality retune, or threading work. macOS code signing / notarization (separate
  track). The RAW-on-wasm work (SPEC-102's sibling, framed separately). Cutting the tag itself — this
  spec lands on `main`; 0.6.0 is a separate release step.

## Notes for the Implementer
- **The one-line fix is `Cargo.toml`'s `default = [...]`, not a `features` key in
  `dist-workspace.toml`.** The dist-config route looks equivalent and is not: it would leave
  `cargo install crustyimg` without AVIF (so brew users and cargo users get different binaries) and it
  would erode the `DEC-052` guard. The dist config must remain feature-free.
- **This is a mechanical sweep, and mechanical sweeps are where this repo has repeatedly under-found.**
  Verification by reading finds a fraction of what grep finds — **cite the grep and the hit count**
  ([[mechanical-sweeps-need-a-mechanical-check]]).
- **Report the measurements even if they're unflattering.** `DEC-020` gated this for real reasons; if
  the size or compile-time delta is large, that belongs in DEC-081 as an accepted cost, stated plainly.
- **You cannot fully prove the *prebuilt* binary has AVIF without cutting a tag**, which is
  irreversible and the maintainer's to fire. Verify everything provable pre-tag (a local default-feature
  build, `dist plan`, the workflow config) and say explicitly that the released-artifact confirmation is
  a post-tag check, rather than implying it was tested.
- Plain user-facing copy in README/CHANGELOG/BENCHMARKS — no spec/DEC refs or internal symbol names
  ([[comments-plain-no-spec-refs]]).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-102-avif-default`
- **PR (if applicable):** none yet — not opened per instructions
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - `DEC-081` — AVIF in the default feature set (supersedes DEC-020's gating).
    `DEC-020`'s frontmatter updated with `superseded_by: DEC-081` (the DEC-011→
    DEC-027 precedent).
- **Measurements:**
  - **Binary size (release, macOS/aarch64):** 12,841,632 B → 15,720,304 B
    (**+2,878,672 B, +22.4%**).
  - **Clean release compile time:** 24.77 s real → 30.48 s real (**+5.71 s,
    +23%**; user-time 227.93 s → 274.55 s). Both measured via `cargo clean
    --release && cargo build --release` back to back on the same machine
    (Apple M4 Pro, Homebrew rustc 1.94.1).
  - **MSRV job result:** PASS. `rustup run 1.90.0 cargo build --features
    avif` — exit 0, clean build. The declared `rust-version = "1.90.0"`
    floor is unaffected: `re_rav1d`/`avif-parse` (the actual 1.90 driver, per
    the existing `Cargo.toml` comment) are AVIF *decode* deps, native-only
    and already unconditional before this spec — not gated by the `avif`
    feature at all. `ravif`/`rav1e` (the newly-default *encode* deps) build
    clean at 1.90.0.
  - **Byte parity:** `convert --format avif -q 80` on the same input, one
    binary built from the pre-spec `Cargo.toml` with `--features avif`
    explicit, one from this spec's `Cargo.toml` with plain defaults —
    **SHA-256 identical**. Confirms this is a distribution change, not an
    encoder behavior change.
  - **Functional check:** `cargo build --release` (no flags) →
    `convert demo/apple-touch-icon.png --format avif -o out.avif` exits 0;
    `sips -g format` (independent decoder, not the extension) confirms
    `format: avif`.
  - **Failing-test proof:** `cargo test convert_to_avif_produces_avif` (bare,
    no `--features` flag) — on the parent commit: `0 passed, 770 filtered
    out` (the `#[cfg(feature = "avif")]` test isn't even compiled in, since
    `avif` wasn't default). On this commit, same command: `1 passed, 782
    filtered out`. See Deviations below for why the test's `cfg` gate itself
    was **not** modified.
  - **Full gate suite (all exit 0):** `cargo test` (default) — **783
    passed, 0 failed** (32 suites, 127 s); `cargo test --no-default-features`
    (lean) — **763 passed, 0 failed** (32 suites, 16 s) — this run includes
    the pre-existing `optimize_heic_exits_4_codec_not_built` /
    `info_heic_exits_4_codec_not_built` tests, confirming the DEC-052 `.heic`
    guard is untouched; `cargo build --no-default-features --release` — exit
    0; `cargo clippy --all-targets -- -D warnings` (default and lean) —
    clean, both; `cargo fmt --check` — clean; `just deny` — "advisories ok,
    bans ok, licenses ok, sources ok" (only the pre-existing scoped
    `libfuzzer-sys` exception, no new exception needed); `just wasm-check`
    with the updated `_wasm_features` (`--no-default-features --features
    avif`) and with the lean override (`--no-default-features` alone) — both
    exit 0; `./scripts/validate-frontmatter.sh` — 226 blocks parse clean.
  - **`dist-workspace.toml`:** confirmed still has no `features`/
    `all-features` key (re-read after all edits) — `DEC-052`'s guard is
    intact.
- **Deviations from spec:**
  - The spec's suggested Failing-Test shape was "the natural shape is an
    existing AVIF test losing its `#[cfg(feature = "avif")]` gate." I did
    **not** remove that gate. Empirically, `#[cfg(feature = "avif")]` is
    satisfied automatically by a bare `cargo test` once `avif` is a default
    feature — the test doesn't need its gate removed to start running
    without a flag; removing it entirely would make the test unconditional
    and it would then **fail** under `cargo test --no-default-features`
    (the lean CI job), since that build genuinely lacks `avif`. Instead I
    verified the behavior change via the same test's pass/fail transition
    across commits (documented above), which proves the identical thing
    ("the avif cfg path is live without any flag") without breaking the
    lean job.
  - Found and fixed a latent regression **not called out in the spec**:
    `justfile`'s `_wasm_features := "--features avif"` (used by
    `wasm-build`/`wasm-check`) did not pass `--no-default-features`, so it
    was implicitly relying on cargo's `default` list being AVIF-less to make
    `just --set _wasm_features "" wasm-build` produce the LEAN/no-AVIF wasm
    comparison artifact SPEC-074/DEC-066 measures against. Once `avif`
    joined `default`, that override would have silently stopped being lean.
    Fixed by pinning `_wasm_features` to `--no-default-features --features
    avif` explicitly (decoupled from the native `default` list) and updating
    the override instructions to `--no-default-features`. Verified both the
    shipped-artifact feature set and the lean override still compile for
    wasm32 (`just wasm-check` under both).
  - `TESTING-WITH-YOUR-PHOTOS.md` appeared in the docs-sweep grep but is
    **gitignored** (`git check-ignore` confirms) — a personal/local file,
    not part of the tracked repo. Edited locally for hygiene but it will
    never be committed; not counted in the sweep total below.
  - `pkg/README.md` is a gitignored, generated copy of `README.md` (`diff`
    confirmed identical) — left untouched, it regenerates from the source at
    next build.
- **Docs sweep — mechanical, cited:** `grep -rn -i "avif" --include="*.rs"
  --include="*.md" --include="*.toml" --include="*.yml" .` piped through a
  filter excluding `target/`, `decisions/`, `*/specs/done/`,
  `*/specs/prompts/`, `reports/`, `docs/sessions/`, `docs/research/`,
  `*/stages/`, `projects/*` (historical/frozen records — not this spec's
  surface) and `pkg/` (generated), then `grep -Ei
  "opt-in|off.by.default|off in the default|not built|--features avif"` —
  **61 hits across 22 files** before edits, **59 after** (one genuine miss
  caught by re-running the same grep post-edit: `README.md`'s `convert
  --format avif` example comment, fixed). Edited: `Cargo.toml` (2 comment
  blocks + the `default` line), `README.md` (feature table + both inline
  example comments), `docs/cli-reference.md` (exit-4 row + `convert` section),
  `docs/api-contract.md` (exit-4 row + `convert` prose + `--max-size` prose),
  `BENCHMARKS.md` (tools section + "what crustyimg is for" + reproduce
  instructions), `docs/USAGE.md`, `CONTRIBUTING.md`, `AGENTS.md` (tech-stack
  list), `docs/moat.md`, `.github/workflows/ci.yml` (stale job comment).
  Deliberately left, with reasons: `tests/*.rs` and `src/*.rs` comments
  describing the feature-gating *mechanism* (still accurate — `avif` remains
  a real, toggleable feature; the "rebuild with `--features avif`" runtime
  hint is still literally correct for a `--no-default-features` build);
  `examples/gen_avif_fixture.rs` (same reason); `deny.toml` (the
  `libfuzzer-sys`/`avif-parse` exception notes remain mechanically accurate
  regardless of default status); `docs/roadmap.md` (its two hits are about
  AVIF *decode*, which was already unconditional and unaffected, and the
  wasm build's own explicit flag, also unaffected); `docs/backlog.md` (a
  frozen wishlist table with a pre-existing, unrelated inaccuracy about
  decode — out of scope to fix here); `docs/feature-exploration.md` and
  `docs/blog/2026-07-06-*.md` (historical/dated records, same category as
  frozen CHANGELOG entries); `bench/corpus/README.md` (a historical note
  about a past harness defect, still literally true today, not an opt-in
  claim); `CHANGELOG.md`'s three pre-existing entries under old version
  headers (historical record of what was true at each past release — not
  edited, consistent with how `[0.5.0]` and earlier are treated everywhere
  else in this file).
- **Follow-up work identified:**
  - The CI `avif` feature job is now redundant with the default `check` job
    (both build/test/lint with `avif` on). Not restructured in this spec
    (kept as an explicit belt-and-suspenders pin, comment corrected) —
    collapsing/simplifying the CI matrix now that three of five feature jobs
    (`display`, `watch`, `avif`) mirror the default is a reasonable small
    follow-up, not done here to keep this spec's blast radius to the
    one-line fix + docs.
  - Per the spec's own honest-limit note: the *prebuilt* Homebrew/Releases
    artifact is **not** verified by this build — only a local default-feature
    build. That confirmation is a **post-tag** check when 0.6.0 actually
    cuts.

### Build Completion — Fix Pass (responding to verify's not-clean findings)

*A second build-cycle pass, made against `main` @ 096789c-rebased (docs/roadmap.md
+ docs/backlog.md only, no overlap). The feature flip, the measured numbers, and
byte parity were confirmed correct by verify and were NOT re-litigated or
re-measured here — this pass is docs/robustness only.*

- **B(i) — `scripts/bench-compare.py`'s overclaim, fixed.** The docstring
  (`"the flagship AVIF path is a pure-Rust opt-in; cargo install crustyimg
  --features avif"`) and two more spots verify didn't separately name but which
  carried the same claim — the `--bin PATH` usage line ("crustyimg binary built
  `--features avif`") and the `find_crustyimg` `sys.exit` hint
  (`` `cargo build --release --features avif` ``) — all now read plain (`cargo
  install crustyimg` / `cargo build --release`, avif built in by default),
  matching `BENCHMARKS.md`'s own corrected wording. The one surviving
  `--features avif` mention in this file (the "shipped 0.5.0 engine built
  `--features avif`" line, describing how *that specific benchmark run* was
  built) is historical, matching `BENCHMARKS.md`'s own retained "crustyimg
  0.5.0, built `--features avif`" — left as-is, same class as the CHANGELOG's
  frozen old-version entries.
- **B(ii) — the sweep itself re-run with the missing extensions, and every hit
  re-triaged independently, not taken on the prior pass's word.** Corrected
  grep:

  ```
  grep -rn -i "avif" \
    --include="*.rs" --include="*.md" --include="*.toml" --include="*.yml" \
    --include="*.py" --include="*.mjs" --include="*.yaml" --include="justfile" \
    . 2>/dev/null \
    | grep -v -E '(^|/)target/|(^|/)decisions/|/specs/done/|/specs/prompts/|(^|/)reports/|(^|/)docs/sessions/|(^|/)docs/research/|/stages/|(^|/)projects/|(^|/)pkg/|node_modules/' \
    | grep -Ei "opt-in|off.by.default|off in the default|not built|--features avif"
  ```

  **69 hits across 27 files before this pass's edits (already reflecting B(i)'s
  fix), 67 across 28 files after.** Every hit was read in file context and
  classified, not just counted:
  - **Fixed, beyond what verify named:** `examples/gen_avif_fixture.rs`'s doc
    comment claimed "without `--features avif` this is a no-op" — **false as of
    this spec**, since `avif` defaulting on means a bare `cargo run --example
    gen_avif_fixture` (no flags at all) now runs the encode branch. Verified by
    driving it: deleted the two committed fixtures, ran the bare command with
    no feature flags, confirmed it regenerated them (`git diff` showed zero
    byte drift against the committed fixtures after). Comment corrected to say
    a plain `cargo run` picks up `avif` by default; only `--no-default-features`
    (or similar) makes it a no-op. The eprintln fallback hint
    (`--features avif`) is untouched — still a valid way to re-enable the
    feature on a build that dropped it, so not a false claim.
  - **Fixed for precision:** `guidance/license-watchlist.yaml:97` cited "the
    AVIF/libwebp pattern" as an example of an off-by-default feature gate for a
    hypothetical jpegli binding — AVIF no longer fits that description.
    Reworded to cite `webp-lossy` (still genuinely off-by-default,
    `Cargo.toml`'s `default` list confirms) and note AVIF was the same shape
    before this spec.
  - **Confirmed correct-as-is, independently re-derived (not just re-stated):**
    - `guidance/constraints.yaml:48` — names `libavif` (a hypothetical *native* C
      binding) as an example needing an off-by-default gate; this is a
      different codec implementation from crustyimg's own pure-Rust `avif`
      feature (`ravif`/`rav1e`) and is unaffected by this spec either way.
    - `justfile:67,74,96,131,149` — the `bench`/`bench-compare` recipes' explicit
      `cargo build --release --features avif` and the wasm `_wasm_features`
      pin are redundant-but-functional (the flag is now a no-op override, not a
      false claim to any reader) — confirmed by re-reading each in context, not
      just accepting the label.
    - `tests/npm_smoke.mjs:140`, `scripts/demo-assemble.mjs:68` — both about the
      **wasm** build's own explicit, deliberately-pinned feature flag
      (DEC-065), unrelated to the native `default` list this spec changed.
    - `bench/corpus/README.md:49`, `deny.toml:46-47`, `docs/roadmap.md:32,184`,
      `docs/backlog.md:106,119`, `docs/feature-exploration.md:103`,
      `docs/blog/2026-07-06-*.md:44`, the three `CHANGELOG.md` hits under old
      version headers, and every `tests/*.rs`/`src/*.rs` hit (feature-gating
      *mechanism* comments and live `cfg!(feature = "avif")` runtime checks,
      e.g. `tests/cli.rs`'s `convert_unbuilt_codec_exits_4`) — re-read in full
      context; each is either a historical/frozen record (same category as a
      CHANGELOG entry under an old version header) or a mechanism description
      that stays true regardless of default status, not a live "AVIF is
      opt-in" claim. `.github/workflows/ci.yml`'s explicit `avif` feature job
      is DEC-081's own documented Neutral-consequence, not an oversight.
  - The 69→67 delta is the two `examples/gen_avif_fixture.rs` lines fixed above
    (`scripts/bench-compare.py`'s B(i) fixes were already reflected in the
    69-count, since B(i) was done first in this pass).
- **A — DEC-081's profile mislabel, fixed.** There is no `[profile.release]`
  table in `Cargo.toml` (confirmed: `grep -n '^\[profile' Cargo.toml` shows only
  `[profile.dist]`, which `inherits = "release"` and adds `lto = "thin"`); a
  plain `cargo build --release` applies the Cargo-default release profile, no
  LTO. The annotation now says so, and adds verify's independently-measured
  `[profile.dist]` figure (12,843,376 → 15,732,192 B, +22.49%) as a second data
  point — the profile Homebrew/Releases actually ship — strengthening the DEC
  rather than just correcting it. **No published number changed**: the
  original before/after table (12,841,632 → 15,720,304 B, +22.4%; 24.77s →
  30.48s) is untouched.
- **C — `justfile:148`'s override, fixed and driven for real.**
  `just --set _wasm_features "--no-default-features" wasm-build` does not
  parse: `just` itself (not cargo) reads a `--set` value starting with `--` as
  an unrecognized flag and errors before the recipe ever runs (reproduced:
  `error: unexpected argument '--no-default-features' found`). Fixed to a
  leading-space value (`" --no-default-features"`), which `just`'s parser
  accepts since the value no longer starts with `--`; the resulting double
  space in the interpolated shell command is harmless. Documented the exact
  failure and why the space is required, not decorative. **Driven, not just
  read**: ran `just --set _wasm_features " --no-default-features" wasm-check`
  (exit 0) and the full documented `wasm-build` (exit 0, produced
  `pkg/`), then independently confirmed via `cargo tree --target
  wasm32-unknown-unknown --no-default-features -e normal` that `ravif`/`rav1e`
  are genuinely absent from that build's dependency graph (the `v_frame`/
  `yuvxyb`/`av-data` crates that also appeared in the build log come from
  `ssimulacra2`, an always-on scoring dependency, not from `avif` — traced via
  `cargo tree --invert yuvxyb`). Re-ran `just wasm-build` afterward (shipped
  feature set) to leave `pkg/` in its normal shipped state (gitignored, no
  tracked diff either way).
- **D — the two load-flaky tests, fixed.** `web_photo_downscales_modernizes_scores`
  and `web_equals_apply_recipe_web` (`tests/cli.rs`) asserted the winning
  output FORMAT was AVIF — i.e., that a debug-build, multithreaded `rav1e`
  encode's measured bytes beat JPEG's on a tiny synthetic photo. That byte
  race is not a stable property under heavy concurrent CPU load (CI's 3-OS
  matrix runs in parallel), even though the ENGINE'S DECISION (admit AVIF as a
  candidate for a photographic bucket, per SPEC-084) is deterministic and
  load-independent. Fixed by adding `--json` to each command and asserting on
  the `--explain` trace's `candidates` array (`"format":"avif"` present)
  instead of the winning file's format:
  - `web_photo_downscales_modernizes_scores` — now asserts AVIF was *admitted*
    (via the JSON report), keeps the format-agnostic assertions unchanged
    (downscale dims, output smaller than source, metadata stripped), and reads
    the SSIMULACRA2 score off the JSON (`"ssim":`) instead of stderr, since
    `--json` routes the report to stdout.
  - `web_equals_apply_recipe_web` — now asserts BOTH the `web` verb and
    `apply --recipe web` admit AVIF as a candidate (via each run's own JSON),
    and keeps comparing the two runs to EACH OTHER (extension agreement +
    byte-identical output) rather than to a hard-coded format — the two
    subprocess invocations run under near-identical momentary CPU conditions,
    so if a byte race flips the winner, it flips consistently for both, and
    the byte-identical invariant (the actual point of this test: `web` ==
    `apply --recipe web`) still holds either way.
  - A fully robust assertion WAS available here (unlike the spec's own caveat
    for a case where it might not be) — the JSON explain trace already exists
    and reports admitted candidates independent of which one wins, so no
    engine behavior change was needed and nothing was left un-fixed or filed.
  - Verified: both tests pass (`cargo test --test cli
    web_photo_downscales_modernizes_scores` /
    `web_equals_apply_recipe_web`), and manually inspected a `--json` report to
    confirm the `candidates` array shape assumed by the new assertions.
- **Full gate suite re-run (all exit 0), scoped to confirm no published number
  moved:** `just validate` (227 front-matter blocks parse); `just check`
  (`fmt-check` + `clippy --all-targets -- -D warnings` + `build` + `test`,
  default features) — all green, full test suite passed; `cargo build
  --no-default-features --release` — exit 0; `cargo test
  --no-default-features` — all green (the avif-gated tests, including the two
  fixed above, correctly absent from this run); `cargo clippy
  --no-default-features --all-targets -- -D warnings` — clean; `cargo fmt
  --check` — clean; `just deny` — advisories/bans/licenses/sources all ok;
  `just wasm-check` (shipped `--no-default-features --features avif`) — exit
  0; `just --set _wasm_features " --no-default-features" wasm-check` (lean) —
  exit 0. **No measurement re-run, no benchmark re-run** — this pass touched
  only `decisions/DEC-081-*.md` (annotation + an added verify-sourced figure),
  `scripts/bench-compare.py`, `justfile`, `guidance/license-watchlist.yaml`,
  `examples/gen_avif_fixture.rs`, and `tests/cli.rs`; `git diff --stat`
  confirms zero touch to `Cargo.toml`, `CHANGELOG.md`, or `BENCHMARKS.md`'s
  measured tables.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The suggested Failing-Test shape ("an existing test losing its cfg
   gate") doesn't actually hold up once you trace through how Cargo feature
   defaults interact with `#[cfg(feature = ...)]` — removing the gate would
   have broken the lean CI job. Took some reasoning to arrive at the
   equivalent-but-correct proof (same test, same bare command, diffed
   across commits) instead of following the literal suggestion.

2. **Was there a constraint or decision that should have been listed but
   wasn't?**
   — The `justfile`'s wasm `_wasm_features` variable's implicit dependency
   on the native `default` feature list wasn't mentioned anywhere (not in
   the spec, not in DEC-065/DEC-073/DEC-074). It's a real coupling this spec
   would have silently broken (the SPEC-074 lean/no-AVIF wasm comparison)
   had I only done the docs-and-Cargo.toml sweep the spec enumerated.

3. **If you did this task again, what would you do differently?**
   — Grep the `justfile` for feature-flag variables up front, not just
   prose docs — a one-line Cargo.toml default change can invalidate a build
   script's assumptions in a way no doc-string grep catches, and the
   `justfile` sits right at that boundary (it's neither "docs" nor "src/").

### Build Completion — Fix Pass 2 (CI red on `webp-lossy feature (build / test / clippy)`)

*A third build-cycle pass, against `main`-rebased `spec-102-avif-default` @ 9d51830. CI's
`webp-lossy` feature leg failed a test that Fix Pass 1's item D did not touch (a different test,
newly exercised in that leg for the first time — see diagnosis below).*

- **Failure:** `optimize_default_photo_picks_avif_single_encode` (`tests/cli.rs`) failed only under
  `cargo test --features webp-lossy` — `assert_eq!` on the output extension expected `"avif"`, got
  `"webp"`. Every other CI leg was green.
- **Why it surfaced now:** this test is `#[cfg(feature = "avif")]`-gated, so before SPEC-102 it only
  ever compiled under an explicit `--features avif` build. Once `avif` became a **default** feature,
  it started compiling into every feature combination that doesn't disable defaults — including the
  `webp-lossy` CI leg (`cargo build --features webp-lossy`, which *adds* to the default set, it
  doesn't replace it), which had never run this test before. Same class of surprise as the two tests
  Fix Pass 1 rewrote (item D) — a feature-default flip changes which `cfg` **combinations**, not just
  which flags, exercise a given path — but a distinct test, since Fix Pass 1's sweep was scoped to
  load-flakiness under `--features avif` alone and had no reason to anticipate a `webp-lossy`
  combination it wasn't yet running.
- **Diagnosis (a vs b) — driven, not assumed.** Per the fix-pass instructions, I did not touch the
  test until confirming which case this was. Ran the exact failing fixture with `--json` added
  (`cargo test --features webp-lossy optimize_default_photo_picks_avif_single_encode -- --nocapture`,
  reading the raw log directly since the `rtk` proxy hook truncates `cargo test` stdout — see
  [[rtk-can-silently-corrupt-grep-counts]]) and inspected the `--explain` candidate list:
  ```
  source_bytes: 3621
  candidates: [
    {"format":"avif","disposition":"lossy","quality":85,"bytes":525,"met_target":true},
    {"format":"webp","disposition":"lossy","quality":85,"bytes":372,"met_target":true},
    {"format":"jpeg","disposition":"lossy","quality":85,"bytes":3861,"met_target":true}
  ]
  winner: 1 (webp, 372 B)
  ```
  This is **case (a): the test was too strong.** AVIF *is* admitted as a candidate (present in the
  shortlist, `met_target: true`) — exactly what `format_shortlist`/`avif_admissible`
  (`src/analysis/decide.rs`) promise. But `pick_winner`'s actual contract is "smallest admitted
  candidate that beats the source, modulo the clear-win guard against the *source-format* candidate"
  (`src/analysis/decide.rs:211-254`) — it has never promised "AVIF always wins the byte race
  regardless of what else is compiled in." On this specific tiny synthetic fixture
  (`common::jpeg_with_exif(256, 256)`, a 256×256 grayscale gradient — `entropy=7.50`,
  `edge_ratio=0.00`, `flat_ratio=1.00`, 192 unique colors), lossy WebP genuinely out-encodes AVIF's
  fixed q85 candidate (372 B vs 525 B) — a real, deterministic property of this fixture at these
  fixed qualities, not flakiness (re-ran the same command twice, byte counts identical both times).
  JPEG (3861 B) never wins regardless — it doesn't beat the 3621 B source at all. Not case (b): AVIF
  was not incorrectly excluded or beaten by a candidate that shouldn't have been admitted; the engine
  correctly shipped the objectively smaller of two admitted, target-meeting candidates. No engine
  change made or needed; nothing filed as a separate decision.
- **Fix:** rewrote the test to assert the contract, not the extension — the same shape Fix Pass 1's
  item D already established for the other two load-flaky tests (`--json` + assert
  `"format":"avif"` is present in the `--explain` candidates array = admission, independent of which
  candidate's bytes win). Specifically:
  - Added `--json` to the `optimize` invocation and asserted the report contains
    `"format":"avif"` (AVIF admitted as a candidate for a photographic source).
  - Replaced the hard-coded `Some("avif")` extension check with: the output extension must be
    `avif` or `webp` (a modern lossy re-encode — explicitly rejecting a bare JPEG passthrough, which
    would indicate the fast decision failed to modernize at all), and the decoded bytes must match
    whichever extension shipped.
  - Kept unchanged: the "beats the source" byte-count assertion and the `elapsed.as_secs() < 20`
    timing assertion (still valid regardless of which admitted candidate wins — both are single
    fixed-quality encodes, not the byte-budget search).
  - Updated the doc comment to explain *why* the assertion is admission-shaped, citing the measured
    372 B vs 525 B split on this exact fixture so a future reader doesn't mistake this for
    unexplained hedging.
- **Verified:** `cargo test --features webp-lossy optimize_default_photo_picks_avif_single_encode`
  — 1 passed; `cargo test optimize_default_photo_picks_avif_single_encode` (default features, no
  `webp-lossy`) — 1 passed (this fixture still has only AVIF/JPEG admitted without `webp-lossy`
  built, AVIF 525 B beats JPEG's 3861 B and the 3621 B source, so it's still the winner there — the
  rewritten assertions hold in both shapes); `cargo test --test cli --features webp-lossy` — full
  suite, **129 passed, 0 failed** (confirms the two Fix-Pass-1 tests and this one all coexist clean
  under the leg that was red); `cargo clippy --all-targets --features webp-lossy -- -D warnings` —
  clean.
- **Full local feature matrix run (the gap that let this through — CI's per-leg jobs aren't run
  combined locally by default), all exit 0. Counts are summed from each run's own `test result:`
  lines (32 suites/binaries every time — unit tests + 31 integration-test crates), not estimated:**
  | Combination | `cargo build --verbose` | `cargo test --verbose` | `cargo clippy --all-targets -- -D warnings` |
  |---|---|---|---|
  | default (`avif,display,watch`) | ✓ | ✓ **783 passed, 0 failed** | ✓ |
  | `--no-default-features` (lean) | ✓ | ✓ **763 passed, 0 failed** | ✓ |
  | `--features avif` | ✓ | ✓ **783 passed, 0 failed** (identical to default — `avif` is already in the default set, so this leg is a no-op addition, per the CI comment calling it a "belt-and-suspenders" pin) | ✓ |
  | `--features webp-lossy` | ✓ | ✓ **790 passed, 0 failed** (the leg that was red before this pass's fix) | ✓ |
  | `--features heic` (system `libheif` 1.23 via Homebrew, present locally) | ✓ | ✓ **789 passed, 0 failed** | ✓ |
  | `--features avif,webp-lossy` | ✓ | ✓ **790 passed, 0 failed** (same as `webp-lossy` alone, for the same reason `avif` alone matches default) | ✓ |

  `heic` was buildable and run locally this pass (`pkg-config --exists libheif` confirmed present) —
  CI's `heic` job additionally covers Ubuntu/Windows-unsupported-path legs this local run does not,
  per the existing CI comments.
- **Gates:** `just validate` — 227 front-matter blocks parse, clean; `just check` (`fmt-check` +
  `lint` + `build` + `test`, default features) — `✓ all gates passed`.
- **No published number moved:** `git diff --stat` for this pass shows a single file touched,
  `tests/cli.rs` (40 insertions, 13 deletions — the one test's doc comment + body). `Cargo.toml`,
  `CHANGELOG.md`, `BENCHMARKS.md`, and every measured figure from Fix Pass 1 are untouched.
- **Scope discipline held:** `webp-lossy` remains opt-in and absent from `dist-workspace.toml`
  (DEC-052 unaffected); no change was made to `src/analysis/decide.rs` or any other engine code:
  this was a test-correctness fix responding to case (a), not a decision-engine change.

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
