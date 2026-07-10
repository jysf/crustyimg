---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-068
  type: chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # not a large diff, but a SYSTEMATIC adversarial pass over 5 surfaces + write the note + apply the small tightenings + reprioritize the backlog; the breadth (not any one fix) is the weight

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
  decisions: [DEC-034, DEC-035, DEC-057, DEC-058, DEC-059, DEC-060, DEC-025]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-063, SPEC-064, SPEC-065, SPEC-066, SPEC-067, SPEC-037]

value_link: "STAGE-024's LEAD — the systematic threat-model pass over PROJ-007's new untrusted-input surface; its findings reprioritize the rest of the stage."

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
        Grounded in a firsthand attack-surface MAP of all five new PROJ-007 surfaces (manifest,
        recipe, cache store, lockfile, watch) with file:line anchors + the guards ALREADY in place,
        so the review verifies-and-tightens rather than re-derives (the SPEC-037 lesson: read the
        existing guards before asserting a gap). The map surfaced concrete suspects to confirm or
        dismiss (cache off-by-53, recipe missing `deny_unknown_fields`, un-clamped watch roots,
        silent `.to_str()→""` seams, and that the exit-code map is ALREADY compiler-exhaustive so
        that backlog item is smaller than it looks). Those are recorded below as suspects, NOT
        verdicts — the fresh adversarial session reaches the conclusions (Design Notes bias).
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 320000
      estimated_usd: 3.00
      duration_minutes: 55
      recorded_at: 2026-07-10
      notes: >
        Build cycle run in the ORCHESTRATOR main loop (not a separately-metered subagent), so
        numerics are an ORDER-OF-MAGNITUDE ESTIMATE per AGENTS §4 + the "autonomous run cost =
        labelled estimates" practice, not a metered count. Opus 4.8 list rate ($5/$25, ~80/20
        in/out, no cache discount). Work: firsthand adversarial attack of all five surfaces
        (hand-authored hostile .toml/.lock/cache-entry bytes driven against the release binary)
        + the two cross-cutting seams, the threat-model note, DEC-061, one inline tightening
        (recipe top-level deny_unknown_fields) with hostile-file tests, the cache off-by-53
        boundary test, and the reprioritized backlog.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-068: PROJ-007 threat-model / attack-surface review

## Context

PROJ-007 shipped a declared, cached, verifiable, watchable build (STAGE-020..023). In doing so it
added **new untrusted-input surface** the earlier tool never had: a parsed **build manifest**
(`crustyimg.build.toml`), parsed **recipe** files, a `.crustyimg/` **cache store** it reads back
(committed, hand-editable entries), a committed and hand-editable **lockfile**
(`crustyimg.build.lock`), and the filesystem tree **`--watch`** walks. In CI — the "reviewed like
code" story — these files arrive from a pull request the maintainer did **not** write, so they are
untrusted input, not just local config.

The wave already proved this surface has issues we *stumble on* rather than *enumerate*: the
SPEC-066 lockfile `short()` panic on a non-hex digest was a committed-file hostile-input defect
that all green exit-code tests missed. This spec is the **SPEC-037-shaped** systematic pass — walk
each new surface against `untrusted-input-hardening` + DEC-034/DEC-035, drive it with hostile
input, tighten what's clearly wrong and small, file what's structural, and **reprioritize the rest
of STAGE-024's backlog** from what the review finds. It is the stage's LEAD: it runs first because
its findings feed the other six candidate items (the decoder fuzz gate is one instrument of it).

## Goal

Produce (1) a written **threat-model note** covering all five new surfaces + the cross-cutting
`.to_str()` and exit-code seams — each path's entry point, the guards already in place, the
adversarial inputs driven against it, and a verdict (safe / tightened-here / filed-as-follow-up),
with residual risk stated; (2) **inline hardening + regression tests** for any clear, small,
security-relevant defect the review surfaces that is not already its own backlog item; (3) a
**reprioritized STAGE-024 backlog** — which of the six remaining candidate items are confirmed,
resized, or dismissed, plus any new items, each severity-ranked; and (4) **DEC-061** recording the
posture verdicts (including any *accepted* risks with rationale). No new default dependency.

**This is verify-and-tighten, not build-from-scratch.** The guards below already exist; the job is
to attack them, not re-derive them. Do NOT gold-plate: fix what's real, record what isn't (a
dismissed suspect with a one-line "why it's fine" is a valid, valuable outcome).

## Inputs

- **The attack-surface map (read first — it is the design handoff):** the Implementation Context
  section below lists every entry point + existing guard + suspect with `file:line` anchors,
  grounded in a firsthand read of the post-STAGE-023 tree. Re-confirm anchors against the current
  tree before relying on line numbers.
- **Files to read (the five surfaces):**
  - `src/build/mod.rs` — `load`/`from_toml` manifest parse (`:203`), size caps (`:78`, `:205`),
    `deny_unknown_fields` (`:188`/`:166`), version + target-count gates (`:217`/`:224`), per-target
    `validate` (`:243`), `find_output_collision` injectivity (`:300`).
  - `src/recipe/mod.rs` — `Recipe::from_toml` (`:222`), caps (`:44`/`:224`), version/step gates
    (`:235`/`:244`), unknown-op hard error (`:266`). **Note the asymmetry:** `Recipe`/`RecipeStep`
    do NOT carry `deny_unknown_fields` (blocked by `#[serde(flatten)] params`, `:147`).
  - `src/build/cache.rs` — `read_entry`/`parse_entry` (`:388`/`:431`), `store_bounded`/`write_entry`
    (`:354`/`:415`), the frame format (`MAGIC|ext_len|ext|payload_len|hash|payload`, 53-byte + ext
    header), symlink/non-regular refusal (`:392`), verify-on-read (`:453`), `path_for` hex-only
    (`:323`), and the **store-vs-read bound asymmetry** (`store` bounds payload `:361`; `read_entry`
    bounds the whole frame `:393`,`:402`).
  - `src/build/lock.rs` — `from_toml` (`:221`), caps (`:81`/`:223`), `deny_unknown_fields` (`:173`),
    version gate (`:234`), the **hex validation on `key`/`hash`** (`:248-257`), `short()` (`:391`),
    `diff` (`:429`). (SPEC-066's fix landed here — verify it holds, don't re-do it.)
  - `src/build/watch.rs` — `watch_roots` (`:98`), `is_excluded`/`normalize_abs` (`:144`/`:227`,
    does NOT canonicalize/resolve symlinks by design), `source_root`/`lexical_clean` (`:184`/`:247`,
    `..` kept when it would rise above a relative root → **no containment clamp on watch roots**);
    `src/cli/mod.rs` notify wiring `register_roots` (`:1864`), `watch_impl` (`:1881`).
  - Cross-cutting: the `.to_str()` stem/ext/path seams table in Implementation Context; `CliError`
    (`src/cli/mod.rs:493`), `code()` match (`:632`, compiler-exhaustive, no wildcard),
    `exit_code_mapping_is_total` (`:4737`, hand-listed value assertions); `docs/api-contract.md`
    (the documented exit codes); `fuzz/fuzz_targets/*` (the four decode targets).
- **Posture to check against:** `guidance/constraints.yaml` → `untrusted-input-hardening`;
  `decisions/DEC-034` + `DEC-035` (decode caps — the mitigation these surfaces sit behind);
  `DEC-025` (exit-code semantics). And `SPEC-037` (PROJ-001's threat-model pass — mirror its shape).
- **Related code paths:** `tests/build_cache.rs`, `tests/build_lock.rs`, `tests/build_watch.rs`
  (the temp-project + child-process harnesses to add hostile-input drivers to).

## Outputs

- **Files created:**
  - `docs/research/proj-007-threat-model.md` — the threat-model note. One section per surface
    (manifest / recipe / cache / lockfile / watch) + a cross-cutting section (`.to_str()` seams,
    exit-code totality). Each section: **entry point → guards in place → adversarial inputs driven
    → verdict (safe | tightened here | filed) → residual risk**. End with the **reprioritized
    STAGE-024 backlog** (the six other candidate items, each: confirmed / resized / dismissed +
    severity + a one-line why) and any NEW items the review adds.
  - `decisions/DEC-061-*.md` — the threat-model verdicts: the posture conclusions, every
    **accepted risk** with its rationale (an accepted risk is a decision, not an omission), and any
    hardening rule this establishes. `affected_scope` = the surfaces touched.
  - Regression tests for each inline fix (see Failing Tests) — in the existing `tests/build_*.rs`
    harnesses and/or `#[cfg(test)]` modules, **driving hostile input** (a hand-authored malformed
    file, not a Rust-constructed struct — the wave lesson: unit tests that build the struct in Rust
    never see a hostile serialized file).
- **Files modified (only where the review confirms a clear, small, security-relevant defect):**
  - candidate: `src/recipe/mod.rs` — if the review deems the missing `deny_unknown_fields` a real
    hardening gap, close it in a `#[serde(flatten)]`-compatible way (e.g. a post-parse unknown-key
    check, or document + accept in DEC-061 that a recipe tolerates unknown keys by design).
  - candidate: `src/build/watch.rs` / `src/cli/mod.rs` — if un-clamped watch roots are deemed a
    real escape, clamp watch roots to under the manifest's directory (or document + accept that
    `--watch` is a local dev loop and roots follow the declared manifest).
  - candidate: any `.to_str()→""` seam the review finds reachable to a *silent wrong output* (vs a
    caught collision) — convert to a typed error or a documented safe fallback.
  - **Out of this spec's edits:** the specific backlog items that already have their own candidate
    spec — the cache off-by-53, the pre-decode format sniff, the build-profile cache key, and
    *running* the fuzz gate. This spec CONFIRMS + severity-ranks them in the note; it does not
    implement them (that keeps the LEAD focused and the boundaries clean). If the review finds one
    is a genuine one-line fix with a trivial test AND security-relevant, it may fold it in and say
    so — but the default is file-and-rank.
- **New exports:** none expected (this is a review + small tightenings, not an API change).

## Acceptance Criteria

- [ ] `docs/research/proj-007-threat-model.md` exists and covers **all five** surfaces + the two
  cross-cutting sections, each with entry point / guards / **adversarial inputs actually driven
  against the real binary** / verdict / residual risk. A verdict of "safe, here's why" is
  acceptable and expected for most; each must be *earned by an attack that was run*, not asserted.
- [ ] Every surface was **driven with hostile input against the release binary** (not just
  reasoned about): a malformed/oversized/duplicate-key manifest; a malformed recipe + an
  unknown-key recipe; a hand-corrupted cache entry (bad magic / truncated / wrong hash / oversize /
  symlinked) and the **near-cap payload boundary**; a hostile lockfile (non-hex digest, unknown
  field, wrong version, oversize) — confirming SPEC-066's fix holds (exit 2, no panic); a manifest
  whose `source` escapes the tree (`../..`) under `--watch`. Each run recorded in the note.
- [ ] Any clear, small, security-relevant defect the review surfaces is **fixed inline with a
  hostile-input regression test**, OR explicitly accepted in DEC-061 with a rationale. Nothing
  security-relevant is left merely noted.
- [ ] The **reprioritized STAGE-024 backlog** is written: each of the six remaining candidate items
  is marked confirmed / resized / dismissed with a severity and a one-line why (e.g. the exit-code
  totality item is expected to *resize* — the `code()` match is already compiler-exhaustive, so the
  real gap is only missing *value* assertions, not missing arms). New items added if found.
- [ ] **DEC-061** records the verdicts + every accepted risk with rationale.
- [ ] **No new default dependency** (`just deny` unchanged; any fuzz tooling stays dev/nightly-only
  and is not added here). Full gate matrix green: `cargo test` default + `cargo build
  --no-default-features` + `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` +
  `just deny` + `just validate`. No `unwrap` on recoverable paths in any new/changed code.

## Failing Tests

Written during **design/build**, BEFORE the fixes they guard. These are the *known* adversarial
drivers; the review adds more as it finds defects. Each drives a **hand-authored hostile file or
the real binary**, never a Rust-constructed struct.

- **Manifest (`tests/` or `src/build/mod.rs` `#[cfg(test)]`)**
  - `"hostile_manifest_unknown_field_is_typed_error_not_panic"` — a `crustyimg.build.toml` with an
    unknown top-level/target key → typed parse error (exit 2), no panic. (Confirms `deny_unknown_fields`.)
  - `"hostile_manifest_oversize_is_rejected_before_parse"` — a > `BUILD_MANIFEST_MAX_BYTES` file →
    typed size error, not an OOM/slow parse.
- **Recipe**
  - `"recipe_unknown_key_behavior_is_pinned"` — a recipe with an unknown top-level key AND an
    unknown step param → assert the CURRENT behavior (silently tolerated today), so the review's
    decision (tighten vs accept) flips this test deliberately, not by accident.
- **Cache (`tests/build_cache.rs` / `src/build/cache.rs`)**
  - `"near_cap_payload_round_trips_or_is_a_clean_miss"` — a payload sized in the
    `[max-52-ext .. max]` band: assert it either round-trips (if the bound is fixed) or is a
    *clean miss* with no panic and no stored-but-unreadable silent band. This nails the off-by-53
    boundary the existing `oversize_entry_is_a_miss` (10-byte payload) never exercises.
  - `"hand_corrupted_entry_variants_are_misses_not_panics"` — bad magic / truncated frame / wrong
    payload_hash / `ext_len > MAX_EXT_LEN` / non-regular (symlinked) entry → `None` (miss), never a
    panic. (Verify the shipped guards under hostile *bytes*, not constructed structs.)
- **Lockfile (`tests/build_lock.rs`)**
  - `"hostile_lock_non_hex_digest_is_exit_2_not_panic"` — regression-confirm SPEC-066's fix on the
    real binary (already exists as `non_hex_digest_in_lock_is_exit_2_not_a_panic`; assert it still
    passes and covers both `key` and `hash`).
  - `"hostile_lock_unknown_field_and_bad_version_are_exit_2"` — unknown field / unsupported version
    / oversize → exit 2 before any output write, no panic.
- **Watch (`tests/build_watch.rs`)**
  - `"watch_root_escaping_source_is_clamped_or_documented"` — a manifest with `source = "../.."`:
    assert the review's decided behavior (clamped to under the manifest dir, OR documented+accepted
    that roots follow the manifest). Pin whichever DEC-061 chooses so it can't drift silently.
- **Paths (where a fix lands)**
  - `"non_utf8_or_empty_stem_input_is_typed_error_not_silent_empty"` — an input whose stem/ext is
    not `.to_str()`-representable (or empty) → a typed error or a documented collision-safe
    fallback, never a silent `""` that only the injectivity check catches as a confusing "output
    collision". (Add only for the seam(s) the review confirms reachable to a wrong *output*.)

## Implementation Context

*Read this and re-confirm the anchors against the current tree before starting. Grounded in a
firsthand surface map of the post-STAGE-023 tree. The suspects below are **suspects to confirm or
dismiss**, NOT verdicts — reach the conclusions by attacking the binary (Design Notes bias).*

### The five surfaces — entry points + guards ALREADY in place

1. **Manifest** — load `src/cli/mod.rs:1222` (on-disk cap `:1228`), parse `src/build/mod.rs:203`
   (in-mem cap `:205`, `deny_unknown_fields` `:188`/`:166`, version `:217`, target cap `:224`,
   per-target `validate` `:243` incl. `src=="-"` stdin refusal, injectivity `find_output_collision`
   `:300`). Loader path handling is clean (no `.to_str()`/index). **Well-guarded — attack to confirm.**

2. **Recipe** — load `src/cli/mod.rs:1020` (cap `:1022`), parse `src/recipe/mod.rs:222` (in-mem cap
   `:224`, version `:235`, step cap `:244`, unknown-op HARD error `:266`). **SUSPECT:** `Recipe`
   (`:159`) + `RecipeStep` (`:143`) have **no `deny_unknown_fields`** — blocked by
   `#[serde(flatten)] params` (`:147`) — so unknown keys are silently tolerated, unlike manifest +
   lockfile. Decide: real gap (tighten via post-parse check) or accept + document.

3. **Cache store** — read `read_entry`/`parse_entry` `src/build/cache.rs:388`/`:431`, write
   `store_bounded`/`write_entry` `:354`/`:415`. Guards: symlink/non-regular refusal (`:392`),
   verify-on-read re-hash (`:453`), checked frame splits (no panic), hex-only paths (`:323`).
   **SUSPECT (confirmed asymmetry):** `store` bounds `bytes.len()` (`:361`); `read_entry` bounds the
   whole frame `payload+53+ext` (`:393`,`:402`) → a payload within 53+ext of `CACHE_ENTRY_MAX_BYTES`
   is stored but unreadable (permanent silent miss; correctness, not safety). Existing
   `oversize_entry_is_a_miss` (`:818`, 10-byte payload) never hits the boundary. CONFIRM + rank;
   the one-line fix is its own backlog item (fold in only if trivial + you add the boundary test).

4. **Lockfile** — load `src/cli/mod.rs:1507`, parse `src/build/lock.rs:221` (caps `:81`/`:223`,
   `deny_unknown_fields` `:173`, version `:234`, **hex validation `:248-257`**, `short()` boundary-
   safe `get(..n)` `:391`, `diff` compares opaque strings `:429`, `Display` quotes paths not `{:?}`).
   **This is the surface that already bit us (SPEC-066).** Verify the fix holds under hostile bytes;
   do NOT re-implement it.

5. **Watch tree** — `watch_roots` `src/build/watch.rs:98`, `is_excluded`/`normalize_abs`
   `:144`/`:227` (deliberately does NOT canonicalize / resolve symlinks), `source_root`
   `:184`/`lexical_clean` `:247` (**keeps `..` above a relative root → no containment clamp**),
   notify `register_roots` `src/cli/mod.rs:1864` (an unwatchable root is a warning, not a failure),
   `watch_impl` `:1881`. **SUSPECT:** a manifest `source = "../.."` makes `--watch` observe outside
   the project tree — untrusted in CI. Decide: clamp to under the manifest dir, or accept (local dev
   loop, roots follow the declared manifest) + document. Also sanity-check: can the notify backend
   follow a symlink OUT of a watched root? (`normalize_abs` won't, but the OS watcher might.)

### Cross-cutting

- **`.to_str()` stem/ext/path seams** (safe-but-*silent* `.unwrap_or("")` unless noted): source
  stem `src/source/mod.rs:54` (`""`), source ext `:117` (None→not-image), **sink ext `:186`
  (ERRORS — good)**, sink `{name}` `:273` (falls back to `{stem}.{ext}`), sink `{parent}` `:281`
  (`""`), cache-key input_ext `src/cli/mod.rs:1351` (`""`), plus stems at `:3365`/`:4601`
  (`"output"`/`"image"`). Non-test `.to_str().unwrap()` on a path: **none** (the two in
  `cache.rs:891/898` + `source/mod.rs:475` are test-only). The review decides which silent `""`
  seams are reachable to a *wrong output* (vs a caught collision) and worth a typed error.
- **Exit-code totality** — `CliError` `src/cli/mod.rs:493`; `code()` match `:632` is
  **compiler-exhaustive (no `_` wildcard)** — adding a variant breaks the build, so arm-coverage is
  already guaranteed. `exit_code_mapping_is_total` (`:4737`) is a hand-listed *value*-assertion test;
  the two prior misses (`Cache`, `Metadata`) were missing value assertions, not missing arms. So this
  backlog item should **resize**: the audit is "assert every variant's documented value + keep the
  hand list complete", not "fix a totality hole". Confirm against `docs/api-contract.md`.
- **Fuzz targets** — `fuzz/fuzz_targets/{avif_decode,heic_decode,svg_decode}.rs` → `Image::from_bytes`;
  `raw_preview.rs` → `raw_preview(data)`. All assert "never panic". *Running* them is a separate
  backlog spec (needs nightly/cargo-fuzz, absent from the build env); this LEAD ranks it highest-
  severity and hands it off with the repeat recipe noted.

### Decisions / constraints that apply
- `untrusted-input-hardening` (the posture every surface is checked against), `DEC-034`/`DEC-035`
  (decode caps — the mitigation the cache/decode paths sit behind), `DEC-025` (exit-code semantics),
  `DEC-057`/`DEC-058`/`DEC-059`/`DEC-060` (the shipped machinery — do not re-open unless a defect
  forces it). New: **`DEC-061`** records this review's verdicts + accepted risks.
- `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `no-new-top-level-deps-without-decision`
  (none here), `clippy-fmt-clean`.

### Out of scope (for this spec specifically)
- Implementing the other six backlog items (fuzz run, off-by-53 fix, format sniff, build-profile
  key, unusual-filename hardening as a full sweep, exit-code value-assertion patch) — this LEAD
  CONFIRMS + reprioritizes them; each is its own spec unless a fix is a trivial fold-in. Re-auditing
  PROJ-001's surface (SPEC-037 did it). A full repo-wide external audit / ultrareview. Reworking a
  shipped DEC unless a defect forces it. A message-text test *framework* (add grep-stderr asserts
  per test instead).

## Notes for the Implementer

- **Fresh adversarial eye is the point.** The architect who framed this (and all of PROJ-007) is
  biased toward trusting the code. Attack the binary with hostile files first; read the code to
  explain what the attack showed, not to pre-decide the verdict. A dismissed suspect (with a
  one-line "why it's fine, here's the attack that failed to break it") is a first-class result.
- **Hostile FILES, not Rust structs.** Every surface here parses a serialized/committed artifact;
  the wave's three green-but-broken defects were all invisible to struct-constructing unit tests.
  Hand-author the malformed `.toml`/`.lock`/cache entry and drive the real binary.
- **The note is the deliverable.** Even where nothing needs fixing, the written threat-model +
  reprioritized backlog is the value — it's what makes the remaining STAGE-024 work targeted.
- Emit `DEC-061` with the verdicts; record accepted risks explicitly (an accepted risk is a
  decision, silence is a gap).
- Run `just validate` after the doc/decision edits; `just deny` must stay unchanged (no new dep).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-068-threat-model`
- **PR (if applicable):** #75 (opened against `main`; awaiting 3-OS CI)
- **All acceptance criteria met?** yes
  - Threat-model note (`docs/research/proj-007-threat-model.md`) covers all five
    surfaces + both cross-cutting sections, each with entry point / guards / **hostile
    inputs driven against the real binary** / verdict / residual risk. ✅
  - Every surface driven with hostile input against the release binary (manifest:
    unknown/oversize/duplicate/stdin/traversal; recipe: unknown top-level + step
    param + malformed + bad version; cache: bad magic / truncated / empty /
    flipped-payload / `ext_len` overflow / symlink + the near-cap boundary unit test;
    lockfile: non-hex `key` **and** `hash` + unknown field + bad version + oversize,
    no write, no panic; watch: `../..` escaping source live-confirmed). ✅
  - The one clear, small, security-relevant defect (recipe top-level
    `deny_unknown_fields`) fixed inline with hostile-file regression tests; every
    other risk explicitly accepted in DEC-061. ✅
  - Reprioritized STAGE-024 backlog written (6 items confirmed/resized/dismissed +
    severity; 2 new items). ✅
  - DEC-061 records verdicts + 4 accepted risks. ✅
  - No new default dependency (`just deny` unchanged; `git diff` on
    deny.toml/Cargo.toml/Cargo.lock empty). Full gate matrix green: `cargo test`
    (377 lib + all integration) + `cargo build --no-default-features` + `cargo clippy
    --all-targets -- -D warnings` + `cargo fmt --check` + `just deny` + `just
    validate`. No `unwrap` on recoverable paths in changed code. ✅
- **New decisions emitted:**
  - `DEC-061` — PROJ-007 threat-model verdicts + accepted risks
- **Deviations from spec:**
  - The recipe suspect's premise ("missing `deny_unknown_fields` blocked by
    `#[serde(flatten)] params`") was **imprecise**: the flatten is on `RecipeStep`, not
    `Recipe`. So the tightening is simpler than the spec's suggested post-parse check —
    a plain `#[serde(deny_unknown_fields)]` on `Recipe` closes the (more dangerous)
    top-level gap; the step-param tolerance is the part actually blocked by flatten and
    is the accepted risk.
  - Cache off-by-53: **not folded in** (per the spec's default). It is a
    correctness-not-safety wart on a shipped DEC-058 module, so it does not force a
    re-open; pinned by a boundary test asserting the current clean-miss, fix filed as
    its own spec.
  - No new manifest regression test added: the surface held with no defect and the
    existing `rejects_unknown_field` / `rejects_oversize_manifest` unit tests already
    guard it; the binary runs are recorded in the note (avoiding gold-plate).
- **Follow-up work identified:** (see the note's reprioritized backlog)
  - **High:** run the decoder fuzz gate (AVIF/SVG/RAW/HEIC) — recipe handed off.
  - **Med:** pre-decode format sniff; cache-key build-profile completeness.
  - **Low–Med:** cache off-by-53 read-bound fix.
  - **Low:** unusual-filename hardening sweep; exit-code value-assertion patch;
    **NEW** strict per-step recipe params; **NEW** `--watch` root-containment *warning*.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Very little — the surface map with anchors was accurate and saved time. The one
   snag was the recipe suspect conflating `Recipe` and `RecipeStep`: the map said
   `deny_unknown_fields` was "blocked by flatten," which is only true for the step, so I
   had to confirm `Recipe` has no flatten before trusting the simpler fix.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `untrusted-input-hardening` + DEC-034/035/025 + DEC-057/058/059/060 were the
   right set. The `safe_join` output clamp (the mitigation that made the manifest
   out-of-tree-source risk acceptable) wasn't called out by anchor but was easy to find
   and verify.

3. **If you did this task again, what would you do differently?**
   — Drive `--watch` (a blocking process) via a detached background launch from the
   start — my first two attempts hung the shell (`wait` on a SIGINT'd child, then a
   foreground sleep the harness blocks). A `nohup … &` + separate inspect/kill call is
   the clean pattern for attacking a long-running subcommand.

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
