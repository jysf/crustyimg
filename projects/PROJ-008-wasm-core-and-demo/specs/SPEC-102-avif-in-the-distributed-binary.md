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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

- [ ] `cargo build --release` with **no feature flags** produces a binary that encodes AVIF:
      `crustyimg convert <photo> --format avif -o out.avif` exits 0 and writes a valid AVIF (verify the
      container independently, e.g. `sips`/`magick identify`, not just the extension).
- [ ] **`DEC-052`'s guard is intact:** `dist-workspace.toml` still has no `features`/`all-features`
      key, and a default build still refuses `.heic` with the typed exit-4 error. State this explicitly —
      the fix must not be implemented by adding a features key to the dist config, which would both miss
      `cargo install` and erode that guard.
- [ ] **Measured, not assumed:** report the release binary **size delta** and the clean **compile-time
      delta** (before vs after), and confirm the **MSRV job still passes** — `rav1e`/`ravif` may floor
      above the declared `rust-version`; if it does, that is a finding, not something to quietly bump.
- [ ] The **lean build** (`cargo build --no-default-features`) still succeeds, and `cargo-deny` stays
      green.
- [ ] `crustyimg web <photo>` on a default build produces AVIF at the fast-quality default and the
      behavior change is recorded in the CHANGELOG as a headline, not a footnote.
- [ ] **Docs sweep is mechanically verified:** cite the grep used and the hit count; every surviving
      "AVIF is opt-in / `--features avif`" claim is either updated or deliberately retained with a
      stated reason. `BENCHMARKS.md`'s install line must no longer require the feature flag.
- [ ] `just validate`, `just check` (fmt/clippy/build/test) green; no unrelated `src/` behavior change.

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

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-081` — AVIF in the default feature set (supersedes DEC-020's gating)
- **Measurements:**
  - binary size before/after:
  - clean compile time before/after:
  - MSRV job result:
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
