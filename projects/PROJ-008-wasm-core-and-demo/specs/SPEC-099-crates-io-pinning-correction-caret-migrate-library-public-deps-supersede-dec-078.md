---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-099
  type: chore
  cycle: build
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-031
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-19

references:
  decisions: [DEC-078, DEC-011, DEC-013, DEC-041]
  constraints: []
  related_specs: [SPEC-098]

value_link: >
  Corrects a false premise in a just-shipped decision and stops us publishing a consumer-hostile
  library: crustyimg IS on crates.io, so the exact `=` pins bite `cargo add crustyimg` NOW.

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

# SPEC-099: crates.io pinning correction — caret-migrate library-public deps + supersede DEC-078

## Context

**DEC-078 (shipped hours earlier, SPEC-098) rests on a false premise.** It states "crustyimg is not on
crates.io, no downstream Cargo consumer exists, so defer the caret migration to a future publish (#5)."
**That is wrong** — verified 2026-07-19 against crates.io directly + `gh run list`:

- crustyimg **is published on crates.io — latest 0.4.0, `has_lib: true`**, every tag v0.1.0→0.4.0
  auto-published by `.github/workflows/publish-crates.yml` (`cargo publish --locked` on every `v*` tag,
  all runs succeeded; first publish 2026-07-04).
- So the **30 exact `=` pins are live on a published library.** Anyone who runs `cargo add crustyimg`
  gets our lib with exact `=` requirements on ~23 runtime deps → forced version-unification conflicts.
  That is the exact downstream harm DEC-078 called hypothetical; it is real, on 0.4.0, now.

This matters specifically for the **r/rust-first launch**: a reader who `cargo add`s us to poke at the
library and hits pin-hell will say so publicly. DEC-078's *direction* (binary reproducibility via the
lock; library-public deps should be caret) is right — only its premise and its "defer, don't migrate"
conclusion are inverted by the facts.

**How the error slipped through (bank it):** the audit's D4 asserted "not on crates.io" citing stale docs
(DEC-040/041) and told us to treat it as given; the DEC-078 verify checked the DEC *against the audit*,
not against crates.io — graded against the wrong oracle. The one-command check that would have caught it:
`gh run list --workflow=publish-crates.yml`. This is the stage's own recurring lesson reaching the
decision layer.

## Goal

Make crustyimg a good crates.io library citizen and correct the record: caret-migrate the library-public
(runtime) dependency requirements so consumers can unify versions, supersede DEC-078 with a corrected
decision, and de-stale the release docs that claim we haven't published. **Reproducibility is preserved**
— the committed `Cargo.lock` still pins exact versions; only the manifest requirements loosen.

## Inputs

- **Files to read:** `Cargo.toml` (the dependency tables — see the exact map below); `Cargo.lock`
  (confirm it stays byte-unchanged); `decisions/DEC-078-dependency-pinning-strategy.md` (the decision to
  supersede); `AGENTS.md` §5 (the exact-pin convention to refine); `RELEASING.md`,
  `projects/PROJ-001-crustyimg-mvp/stages/STAGE-007-release-and-distribution.md`, `decisions/DEC-041-*.md`
  (the stale "not yet published" claims); `docs/research/proj-008-rust-directives-audit.md` (D4 — the
  wrong claim to annotate); `.github/workflows/ci.yml` (note: CI does **not** use `--locked`).

## Outputs

- **`Cargo.toml` — the caret migration (the only code-adjacent change).** Strip the leading `=` from every
  **runtime** dependency requirement (`"=X.Y.Z"` → `"X.Y.Z"`, which is caret `^X.Y.Z` = `>= tested,
  < next-incompatible`), across these three tables:
  - `[dependencies]` (~line 46): fast_image_resize, thiserror, serde, toml, kamadak-exif, img-parts,
    skrifa, zeno, ssimulacra2, webp (optional), libheif-rs (optional).
  - `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` (~line 130): image, resvg, re_rav1d,
    avif-parse, clap, clap_complete, glob, rayon, indicatif, sha2, viuer (optional), notify (optional).
  - `[target.'cfg(target_arch = "wasm32")'.dependencies]` (~line 212): image, resvg, wasm-bindgen.
  - **Keep the 4 `[dev-dependencies]` rows pinned** (serde_json, tempfile, criterion, wasm-bindgen-test) —
    dev-deps never constrain a consumer's resolution, and keeping them exact preserves test-repro value.
- **`decisions/DEC-079-*.md` (new) — supersedes DEC-078.** Draft below. Set `supersedes: DEC-078` and set
  DEC-078's `superseded_by: DEC-079`.
- **De-stale the "not yet published" docs:** `RELEASING.md` ("before cutting `v0.1.0`" → reflect that
  we've published since v0.1.0 and auto-publish every tag, currently 0.4.0), STAGE-007 (its Count line
  claiming the v0.1.0 tag cut is "the only thing left" / "machinery ARMED, not fired"), and DEC-041 (#5
  "not fired"). Correct to the true state; do not rewrite history, just fix the present-tense claims.
- **Annotate the audit's D4** (`docs/research/proj-008-rust-directives-audit.md`): a one-line correction
  that its "not on crates.io" premise was false (verified 2026-07-19), pointing to DEC-079.
- **`AGENTS.md` §5**: update the exact-pin convention text + repoint from DEC-078 to DEC-079 — runtime
  deps are caret (library-friendly, lock gives reproducibility); dev-deps may stay exact.

### DEC-079 draft (plain, behavior-first)

> **DEC-079 — Dependency pinning: caret for the (published) library, exact lock for reproducibility**
> (supersedes DEC-078)
>
> **Context:** DEC-078 assumed crustyimg was not yet on crates.io and deferred relaxing the dependency
> pins. That premise was false — crustyimg has been published since v0.1.0 (2026-07-04) and auto-publishes
> every tag (`publish-crates.yml`); latest 0.4.0, `has_lib: true`. So the exact `=` pins are live on a
> published library and force version-unification conflicts on anyone who `cargo add`s it.
>
> **Decision:**
> 1. **Runtime (library-public) dependency requirements use caret** (`^x.y.z`, written as the bare
>    version), so consumers can unify. This covers `[dependencies]` and both `[target.*.dependencies]`.
> 2. **Reproducibility comes from the committed `Cargo.lock`**, not from manifest pins — `cargo
>    install --locked` and our release/build flows resolve to the exact locked versions regardless of the
>    caret ranges. The reproducible-build thesis (PROJ-007) is intact.
> 3. **`[dev-dependencies]` may stay exactly pinned** — they never constrain a consumer's resolution, and
>    exact dev-deps keep test reproducibility crisp.
> 4. **This supersedes DEC-078 and refines AGENTS.md §5 / DEC-011 / DEC-013:** exact manifest pins are no
>    longer the policy for runtime deps of a published library; the lock carries reproducibility.
>
> **Consequences:** the next release (e.g. 0.4.1 / 0.5.0) ships a consumer-friendly manifest. No resolved
> version changes now (the lock is unchanged; caret is strictly looser than the current exact reqs).

## Acceptance Criteria

- [ ] Every runtime dependency requirement in the three tables above is caret (no leading `=`); the 4
      dev-dependency rows remain exactly pinned.
- [ ] **`Cargo.lock` is byte-unchanged** — caret is looser than the prior exact reqs, so resolution keeps
      the identical versions (proven by `git diff --stat Cargo.lock` = empty).
- [ ] `decisions/DEC-079-*.md` exists, `supersedes: DEC-078`; DEC-078 gains `superseded_by: DEC-079`.
- [ ] RELEASING.md, STAGE-007, DEC-041, and the audit D4 no longer claim crustyimg is unpublished; they
      state the true present (published, auto-per-tag, 0.4.0). AGENTS.md §5 repoints to DEC-079.
- [ ] **Full CI matrix green** — native, `--features avif`, lean `--no-default-features`, **MSRV
      (1.90.0)**, cargo-deny, wasm build, clippy, fmt. (No `--locked` in CI, so this confirms a fresh
      resolve against the caret reqs still lands on lock-compatible, MSRV-passing versions.)
- [ ] No `src/` change, no behavior change, no feature change.

## Failing Tests

Documentation + a manifest change, so verification is structural, not a unit test:

- `cargo build` / `cargo test` (native + `--features avif` + lean) succeed with **`Cargo.lock`
  unchanged** — capture `git status Cargo.lock` before/after.
- `cargo +1.90.0 build` (MSRV) succeeds (the caret reqs must not let a fresh resolve pull a version that
  breaks MSRV; the committed lock protects this — confirm the lock is what's used, or `cargo update
  --dry-run` shows no MSRV-breaking bump).
- `cargo publish --locked --dry-run` succeeds (the shape a real publish uses; proves the manifest is
  still publishable and `--locked` pins exact).
- **Negative control:** confirm at least one caret req actually loosened — e.g. `cargo tree` / a
  `cargo add crustyimg`-style resolution in a throwaway consumer now permits a newer compatible patch
  than the old `=` would have. (Demonstrates the consumer benefit, not just that CI is green.)

## Implementation Context

### Decisions that apply
- `DEC-078` — the decision being **superseded** (its premise was false).
- `DEC-011` / `DEC-013` — the exact-pin convention this refines (now: lock-based reproducibility, caret
  manifest for runtime deps).
- `DEC-041` — names the release/publish channels; one of the stale "#5 not fired" docs to correct.

### Constraints that apply
- The PROJ-007 reproducible-build thesis must stay intact — **the committed `Cargo.lock` is the
  reproducibility mechanism**; do not delete or loosen the lock, only the manifest reqs.

### Prior related work
- `SPEC-098` (shipped) — created DEC-078; this corrects it.
- The pre-launch Rust audit — D4 is the source of the false premise; annotate it.

### Out of scope
- Any `src/` change, feature change, or actual dependency **version** change (the lock stays put; we only
  loosen requirements). No `cargo update`/upgrades — that's separate maintenance.
- Cutting a release / publishing — the corrected manifest ships with the next normal release; this spec
  does not tag or publish.
- Relaxing dev-dependencies (deliberately kept exact).

## Notes for the Implementer
- **The lock is the safety net and the proof.** The whole point is that stripping `=` → caret cannot move
  resolved versions (caret ⊇ the exact point), so `Cargo.lock` must come out byte-identical. If it
  changes, stop and investigate — something re-resolved unexpectedly.
- **MSRV is the one real risk.** CI has no `--locked`, so a fresh resolve uses the caret reqs; the
  committed lock should still win (versions satisfy caret). Confirm the MSRV job stays green and that no
  transitive bump sneaks in — `cargo update --dry-run` should show nothing forced.
- **Keep the DEC + doc prose plain/behavior-first** ([[comments-plain-no-spec-refs]]); internal DEC
  cross-refs are fine in decision/release docs.
- **Supersede, don't delete** DEC-078 — the error + correction trail is the value (this is a
  "checked-claim" lesson worth preserving).
- Take the next DEC number (DEC-079); re-confirm nothing else claimed it.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
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
