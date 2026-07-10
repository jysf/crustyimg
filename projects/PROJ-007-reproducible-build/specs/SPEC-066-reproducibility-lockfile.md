---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-066
  type: story
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: L                    # lockfile module + executor wiring + check/frozen/strict + env-aware policy + DEC-059; the stage's headline spec

project:
  id: PROJ-007
  stage: STAGE-022
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-09

references:
  decisions: [DEC-057, DEC-058, DEC-025, DEC-005, DEC-006, DEC-007, DEC-004, DEC-059]
  constraints:
    - untrusted-input-hardening
    - no-new-top-level-deps-without-decision
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-063, SPEC-064, SPEC-065, SPEC-023]

value_link: "STAGE-022's 'a committed lockfile pins the build; --check fails CI on drift' — the 'verifiable' payoff."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-09
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Grounded in a firsthand read of the shipped post-SPEC-065 executor (run_build /
        check_output_injective / build_one / cache_key_for) + the cache public API (CacheKey/Hash
        to_hex, hash_bytes, compute_key). Confirmed toml serialization is already used (DEC-058
        recipe round-trip) and sha2/serde/toml are shipped → no new dep. Reproducibility policy
        (pin key, record hash+env; env-aware --check) carried from the STAGE-022 Design Notes.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-066: the reproducibility lockfile + `build --check` / `--frozen`

## Context

STAGE-020 gave crustyimg a declared build, STAGE-021 made it incremental, and SPEC-065
guaranteed the build maps sources to outputs **injectively** (so an output path is a valid
key). This spec is STAGE-022's headline — the **"verifiable"** payoff: a committed
`crustyimg.build.lock` that pins the build so a re-run, a teammate, or CI can assert the
outputs haven't drifted, and image outputs get reviewed like code.

The honest framing (STAGE-022 Design Notes; the encoder-determinism experiment): **pin the
robust, record the fragile.** The lockfile *pins* each output's DEC-058 **cache key** — a
hash of the inputs (tool version + features + canonical recipe + quality + input ext +
input content), which is reproducible across machines. It *records* the observed **output
hash** and the **environment** it was observed under, because encoder bytes are
byte-identical within a machine but **not** across arch/version. So `--check` fails on
**input drift** (a key changed — always, unambiguously) and on an **output regression under
the same env**, but treats cross-env output-byte variance as **informational**, not a
failure — unless `--strict`. The review-grade "did the image actually change" check stays
perceptual (SSIMULACRA2, the shipped `diff`, DEC-025), not encoder bytes. See the parent
`STAGE-022-reproducibility-lockfile.md`.

## Goal

Add a `src/build/lock.rs` (the `crustyimg.build.lock` format + an env-aware diff) and wire
`run_build` to it: a normal build **writes/refreshes** the lockfile; `build --check` (and
its `--frozen`/`--locked` aliases) **verifies** the resolved build against the committed
lockfile and exits **7** on drift without modifying it; `--strict` promotes cross-env
output-hash variance to a failure. No new dependency.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_build` (~L1378, the two-phase executor; the lock write/check hooks
    in after execute), `build_one` (~L1314, must surface per-output `(path, key, output-hash)`),
    `cache_key_for` (~L1268, the key must be computed even under `--no-cache` when a lock is
    written/checked), `BuildCtx` (~L1300), `check_output_injective` (~L1237, the injective
    guarantee the lock relies on), `GlobalArgs` (~L58, add `--check`/`--frozen`/`--locked`/`--strict`),
    `CliError` + `code()` (~L460/543; `CheckFailed` → 7 already exists, DEC-025).
  - `src/build/cache.rs` — `CacheKey::to_hex`, `Hash::to_hex`, `hash_bytes` (the output hash),
    `compute_key`; `src/build/mod.rs` (add `pub mod lock;`).
  - `src/build/mod.rs` / `src/sink/mod.rs` — `expand_template` + `safe_join` (the output path a
    lock entry is keyed on) and `Target::template`.
  - `src/recipe/mod.rs` — the `toml::to_string` round-trip already used by `recipe_hash` (proves
    toml serialization is available for writing the lock).
  - `decisions/DEC-057` (manifest/executor + injective), `DEC-058` (the cache key the lock pins),
    `DEC-025` (exit 7 `CheckFailed`), `DEC-005` (the versioned-TOML + size-guard discipline).
- **External APIs:** none. **No new dependency** (serde + toml + sha2 all shipped; toml
  serialization is already exercised).
- **Related code paths:** `tests/build_cache.rs` / `tests/build_injective.rs` for the
  integration-test shape; `crustyimg diff` (SPEC-023) as the perceptual escape hatch.

## Outputs

- **Files created:**
  - `src/build/lock.rs` — the lockfile, library-first + unit-tested:
    - `pub struct BuildLock { pub version: u32, pub env: LockEnv, pub output: Vec<LockOutput> }`
      (`#[serde(deny_unknown_fields)]`, `version` gate, `LOCK_MAX_BYTES` guard mirroring
      `RECIPE_MAX_BYTES`).
    - `pub struct LockEnv { pub crustyimg_version: String, pub target: String, pub features: String }`
      — the env the hashes were observed under (`target` = `"{ARCH}-{OS}"` from
      `std::env::consts`, no dep).
    - `pub struct LockOutput { pub path: String, pub source: String, pub recipe: String,
      pub key: String, pub hash: String, pub bytes: u64 }` — `path` is the primary key
      (injective, SPEC-065); `key` = pinned cache key hex; `hash` = observed output-bytes hex;
      `source`/`recipe`/`bytes` are provenance for a reviewable diff.
    - `BuildLock::from_toml(&str) -> Result<_, LockError>` + `to_toml(&self) -> String` (sorted
      outputs by `path` for a deterministic, review-friendly file).
    - `pub fn diff(committed: &BuildLock, current: &BuildLock, strict: bool) -> LockDiff` — the
      env-aware comparison (see Implementation Context); `LockDiff { drifted: bool, changes: Vec<LockChange> }`.
    - `pub enum LockError` (thiserror) — parse / unknown-field / unsupported-version / too-large.
    - `pub const DEFAULT_LOCK_FILE: &str = "crustyimg.build.lock"`, `SUPPORTED_LOCK_VERSION: u32 = 1`.
  - `tests/build_lock.rs` — integration tests driving the binary (see Failing Tests).
  - `decisions/DEC-059-*.md` — the lockfile format + what it pins vs records + the env-aware
    `--check`/`--frozen`/`--strict` policy (emitted at build).
- **Files modified:**
  - `src/build/mod.rs` — `pub mod lock;`.
  - `src/cli/mod.rs` — `--check` / `--frozen` / `--locked` / `--strict` on `GlobalArgs`;
    `build_one` returns a per-output record; `run_build` collects records → builds a `BuildLock`
    → **writes** it (default) or **diffs** it against the committed lock and returns
    `CliError::CheckFailed` (exit 7) on drift (`--check`/`--frozen`/`--locked`), never writing.
  - `Cargo.toml` — confirm `toml`'s serialization is on (already used); **no new dep**.
- **New exports:** `crustyimg::build::lock::{BuildLock, LockEnv, LockOutput, LockError, diff,
  LockDiff, DEFAULT_LOCK_FILE}`.

## Acceptance Criteria

- [ ] A normal `crustyimg build` writes/refreshes `crustyimg.build.lock` with one `[[output]]` per
  output (path, source, recipe, pinned `key`, observed `hash`, bytes) + one `[env]` block; a
  no-change re-run on the same machine produces a **byte-identical** lockfile (deterministic, sorted).
- [ ] `crustyimg build --check` exits **0** when the resolved build matches the committed lockfile,
  and **7** (`CheckFailed`, DEC-025) on drift — **without modifying** the lockfile.
- [ ] **Input drift** (a source, recipe, quality, or tool version change → the output's cache `key`
  differs, or an output path is added/removed) makes `--check` exit 7, naming what drifted.
- [ ] **Output-hash drift under the SAME env** (key matches, bytes differ, recorded `target` ==
  current) is a failure (exit 7). Under a **different** env it is **informational** (exit 0 with a
  note) — unless `--strict`, which makes it exit 7.
- [ ] `--frozen` / `--locked` behave as `--check` (assert, never write); a **missing** lockfile under
  `--check`/`--frozen` is drift → exit 7 with "no lockfile; run `crustyimg build` to create one".
- [ ] A malformed lockfile (bad TOML / unknown field / unsupported version / oversize) is a typed
  `LockError` → exit 2, before any output is written.
- [ ] The lockfile relies on SPEC-065's injectivity (one entry per output path); **no new dependency**;
  `cargo build --no-default-features` still succeeds; `just deny` unchanged and green.
- [ ] `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` clean; every new public fn tested.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/build/lock.rs`** (`#[cfg(test)] mod tests`)
  - `"parses_valid_lock"` — version + `[env]` + two `[[output]]` → `Ok`, fields populated.
  - `"rejects_unknown_field" / "rejects_unsupported_version" / "rejects_oversize_lock"` — typed `LockError`.
  - `"to_toml_from_toml_roundtrips"` — `to_toml` then `from_toml` equal; outputs are sorted by `path`.
  - `"diff_identical_is_clean"` — same lock vs itself → `drifted == false`.
  - `"diff_added_or_removed_output_is_drift"` — an output present in one only → `drifted`.
  - `"diff_key_change_is_drift"` — same path, different `key` → `drifted` (input drift), regardless of env.
  - `"diff_hash_change_same_env_is_drift"` — same path+key, different `hash`, **same** `target` → `drifted`.
  - `"diff_hash_change_cross_env_is_informational"` — same path+key, different `hash`, **different**
    `target` → NOT `drifted` when `strict == false`; **is** `drifted` when `strict == true`.
- **`tests/build_lock.rs`** (integration; native PNG fixtures)
  - `"build_writes_lockfile"` — a build writes `crustyimg.build.lock` with an `[[output]]` per output
    (path/key/hash/env present) and the outputs themselves.
  - `"lockfile_is_deterministic"` — two clean builds → byte-identical lockfiles.
  - `"check_passes_on_matching_tree"` — after a build, `build --check` → exit 0, lockfile unchanged.
  - `"check_fails_on_edited_source"` — edit one source, `build --check` → exit 7, names the drifted output.
  - `"check_fails_on_added_or_removed_output"` — change the manifest's outputs, `build --check` → exit 7.
  - `"frozen_without_lockfile_fails"` — no lockfile + `build --frozen` → exit 7 with the "create one" hint.
  - `"frozen_matching_passes_and_does_not_write"` — matching tree + `--frozen` → exit 0, lockfile mtime
    unchanged; a drifting tree + `--frozen` → exit 7, lockfile still unmodified.
  - `"cross_env_hash_tolerated_unless_strict"` — hand-edit the committed lock's `[env].target` to a
    foreign arch and one `hash` to a wrong value; `build --check` → exit 0 (informational); `build
    --check --strict` → exit 7. (The env branch can't be exercised by really changing arch in a test.)
  - `"malformed_lockfile_is_exit_2"` — a `crustyimg.build.lock` with an unknown field → exit 2, no writes.

## Implementation Context

*Read this section (and the files it points to) before starting build. The seam was read
firsthand during design against the current post-SPEC-065 tree — re-confirm signatures.*

### Decisions that apply
- `DEC-058` — the cache key the lock **pins**. `CacheKey::to_hex()` is the pinned `key`; the observed
  output hash is `cache::hash_bytes(&output_bytes).to_hex()`. Do NOT reshape the key — the lock consumes
  it. The key already encodes version + features + recipe + quality + ext + content, so key-equality IS
  input-equality; that is why key drift is an unambiguous, cross-machine-reproducible failure.
- `DEC-057` — the executor + manifest; the lock file sits next to the manifest, cwd-relative, and
  inherits the versioned-TOML + `deny_unknown_fields` + size-guard discipline.
- `DEC-025` — exit **7** (`CliError::CheckFailed`) is the established "a check computed but was not
  satisfied" code (shared with `diff --fail-under`). `--check` drift reuses it; do not invent a new code.
- `DEC-005` — the recipe TOML size-guard / version-gate pattern to mirror for `LOCK_MAX_BYTES`.
- **`DEC-059` (NEW — emit at build)** — the `crustyimg.build.lock` format, what it pins (cache key) vs
  records (output hash + env), and the env-aware `--check`/`--frozen`/`--locked`/`--strict` policy.
  Records that perceptual verification is the shipped `diff` (SSIMULACRA2), composable, not baked into
  `--check` here; and that `--frozen`/`--locked` are equivalent (crustyimg has no network).

### The env-aware diff (the load-bearing policy — capture in DEC-059)
Compare committed vs current, keyed on output `path`:
- a `path` in one lock but not the other → **drift**.
- same `path`, `key` differs → **drift** (input changed: source/recipe/quality/version). Always.
- same `path`, `key` equal, `hash` differs:
  - current `target` (`env.target`, `"{ARCH}-{OS}"` from `std::env::consts`) **==** committed `target`
    → **drift** (a real output regression on this machine).
  - `target` **!=** committed → **informational** (expected cross-arch/os encoder variance): NOT drift
    when `!strict`; **drift** when `strict`.
- else clean. `LockDiff` carries a per-change reason so the CLI can print a readable diff.

### Executor seam (post-SPEC-065)
`run_build` already: prepare all targets → `check_output_injective` → open cache → execute (rayon
`build_one` per input) → summary. Add:
1. **Compute the key even under `--no-cache`** when a lock is being written or checked — today
   `cache_key_for` only runs when the cache is `Some`. Factor the key computation so `build_one` can
   produce it whenever `lock_mode` is on. (Reads the source bytes once; the miss path already does.)
2. **`build_one` returns a per-output record** alongside `Built`: `LockRecord { path, source_label,
   recipe, key_hex, out_hash_hex, bytes }`. `path` = `safe_join(out, expand_template(template, stem,
   ext, ...))` using the real `ext` (from the encode, or the cache entry on a hit). `out_hash_hex` =
   `hash_bytes(&bytes).to_hex()` (the bytes it wrote — cache bytes on a hit, fresh on a miss).
3. **After execute**, collect the records (only on success; a partial-batch failure → exit 6 as today,
   no lock written/checked), sort by `path`, build a `BuildLock { version, env: current_env(), output }`.
4. **Default** → `to_toml` → write `crustyimg.build.lock` (atomic temp→rename; overwrite-owned like the
   build's outputs). **`--check`/`--frozen`/`--locked`** → load the committed lock (missing → exit 7),
   `diff(committed, current, strict)`; if `drifted`, print the changes and return `CliError::CheckFailed`
   (exit 7) **without writing**; else exit 0.

### Constraints that apply
- `untrusted-input-hardening` — the lock is committed config: `LOCK_MAX_BYTES` (fs::metadata pre-check
  + string-length check), `deny_unknown_fields`, version gate; hashes/paths are compared as opaque
  strings. `no-new-top-level-deps-without-decision` (none), `no-unwrap-on-recoverable-paths`,
  `every-public-fn-tested`, `clippy-fmt-clean`, `ergonomic-defaults` (default writes the lock; the
  assert modes are opt-in flags; a missing lock under `--check` fails with an actionable message).

### Prior related work
- `SPEC-063`/`DEC-057` (executor + manifest), `SPEC-064`/`DEC-058` (the cache key pinned here),
  `SPEC-065` (injectivity — one lock entry per output path, the property that makes `path` a valid key),
  `SPEC-023`/`DEC-025` (the SSIMULACRA2 `diff` + exit-7 precedent; the perceptual review escape hatch).

### Out of scope (for this spec specifically)
- `--watch` (STAGE-023); a remote/networked lock or cache; reshaping the cache key/store (DEC-058);
  baking perceptual (SSIMULACRA2) verification into `--check` (it stays the composable `diff` command —
  DEC-059 records why); per-target format/quality overrides; a `--check --json` machine-readable drift
  report (a natural follow-up — note it, don't build it); auto-regenerating the lock under `--check`
  (never — that would defeat the gate); the STAGE-021 `CACHE_ENTRY_MAX_BYTES` off-by-53 fix (fold in
  only if the store is touched here).

## Notes for the Implementer

- Keep the format + `diff` in `src/build/lock.rs` (library, unit-tested); keep the executor wiring +
  `LockRecord` collection in `cli`. Reuse `hash_bytes`/`to_hex` from the cache — do not add a second hasher.
- The lockfile must be **deterministic**: sort `[[output]]` by `path` before serializing, so two clean
  builds diff byte-identically and a review diff is minimal. Add a test that pins this.
- Compute the output `path` the SAME way the sink does (`expand_template` + `safe_join`), so a lock entry
  names exactly the file that was written. On a cache **hit** the ext comes from the entry; on a miss from
  the encode — both already available in `build_one`.
- `--frozen`/`--locked` are aliases of the assert behavior (no network in crustyimg); document that in
  DEC-059 rather than implementing a second path.
- The cross-env branch of `diff` is unit-tested (arch can't change within one test binary) — mirror how
  SPEC-064 unit-tested version-invalidation; be explicit in the test comment.
- Emit `DEC-059` with `affected_scope` covering `src/build/lock.rs`, `src/build/mod.rs`, `src/cli/mod.rs`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-059` — the lockfile format + pin-vs-record + env-aware `--check`/`--frozen`/`--strict` policy
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

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
