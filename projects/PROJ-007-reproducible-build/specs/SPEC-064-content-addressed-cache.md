---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-064
  type: story
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # cache-key composition + local content-addressed store + executor seam + one new dep; single spec

project:
  id: PROJ-007
  stage: STAGE-021
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-08

references:
  decisions: [DEC-004, DEC-005, DEC-006, DEC-007, DEC-015, DEC-034, DEC-057, DEC-058]
  constraints:
    - untrusted-input-hardening
    - pure-rust-codecs-default
    - no-new-top-level-deps-without-decision
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-063, SPEC-031, SPEC-035, SPEC-006]

value_link: "STAGE-021's 'a re-run does only the work that changed' — the incremental-rebuild headline."

# Self-reported AI cost per cycle. design/ship = main-loop null-with-note;
# build/verify = real tokens_total (subagent) or labelled estimate. See AGENTS §4.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-08
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Included a firsthand read of the shipped executor (run_build / prepare_target / apply_one /
        load_recipe) + the sink byte-write path (write_bytes / expand_template / safe_join /
        encode_to_bytes) to fix the cache seam; the load-bearing hasher probe is deferred to build
        (pick + license-check BLAKE3 vs sha2, DEC-058). Encoder-determinism experiment (prior this
        session) retired the nondeterminism risk.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-064: the content-addressed cache (incremental rebuild)

## Context

STAGE-020 shipped a declared build (`crustyimg build` over a `crustyimg.build.toml`) that
re-does **every** target on every run. This spec is STAGE-021 — the project's headline:
a **content-addressed cache** so a re-run does only the work that actually changed. It
wraps each per-input unit of `run_build` in a local on-disk cache keyed by a hash of every
output-affecting input; on a hit it materializes the cached output and skips the
decode→pipeline→encode, on a miss it runs the shipped worker and stores the result. A
no-change re-run becomes a full cache hit reporting near-zero work.

This is the **robust** half of "verifiable" — cache-correctness, deterministic *within a
machine* — as distinct from the **fragile** half (cross-arch output-byte reproducibility)
that STAGE-022's lockfile owns. The encoder-determinism experiment (this session) already
retired the encoder-nondeterminism risk; the remaining hard problem is **cache-key
correctness** (miss one output-affecting input → ship a stale artifact). See the parent
`STAGE-021-content-addressed-cache.md` for the probe result and framing, and DEC-057 for
the executor this extends.

## Goal

Add a `src/build/cache.rs` (cache-key over every output-affecting input + a local
`.crustyimg/cache/` content-addressed store) and wire it into `run_build`: per input,
compute the key, on a **hit** materialize the cached output and skip decode/pipeline/encode,
on a **miss** run the shipped worker and store the result — with a `--no-cache` bypass, a
cached/rebuilt summary, corrupt/missing-entry → clean rebuild, and **one** license-probed
hasher dependency (DEC-058).

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_build` (~L1131, the execute phase to wrap), `prepare_target` /
    `PreparedTarget` (~L1089/1045), **`apply_one`** (~L815, the worker to split into
    `encode_one` + write), `load_recipe` (~L877), `run_apply` (~L900, also calls `apply_one`
    — must stay behavior-identical after the split), `GlobalArgs` (~L58, add `--no-cache`), the
    `CliError` enum + `code()` map (~L460/543, add a `Cache` arm if any cache error can reach
    the boundary), `Commands::Build` dispatch (~L617).
  - `src/sink/mod.rs` — `write_bytes` (~L505, materialize a hit), `expand_template` (~L269),
    `safe_join` (~L308), `encode_to_bytes` (public), `Overwrite`, `SinkInput` — the reuse
    surface for both the miss-path write and the hit-path materialize.
  - `src/build/mod.rs` — the manifest module to add `pub mod cache;` to; `Target::template`.
  - `src/recipe/mod.rs` — `Recipe` (the parsed form to hash canonically), `RecipeError`.
  - `src/source/mod.rs` — `Input` (build inputs are always `Input::Path`; `stem`/`path`).
  - `Cargo.toml` — add the ONE hasher dep (DEC-058); confirm the lean build + `just deny`.
- **External APIs:** the chosen hasher crate (BLAKE3 recommended; sha2/xxhash alternatives) —
  license + pure-Rust-default + `just deny` verified firsthand at build (the load-bearing probe).
- **Related code paths:** `src/build/`, `src/cli/`, `tests/` (the `apply`/`build` integration
  tests as the shape for `tests/build_cache.rs`).

## Outputs

- **Files created:**
  - `src/build/cache.rs` — the cache. Suggested surface:
    - `pub struct CacheKey([u8; 32])` (or the hasher's digest) with a hex path form.
    - `pub fn compute_key(version: &str, features: &str, recipe_hash: &Hash, quality: Option<u8>,
      input_ext: &str, input_hash: &Hash) -> CacheKey` — the domain-separated composition of
      the enumerated inputs (see Implementation Context). Plus `hash_bytes(&[u8]) -> Hash` and a
      `recipe canonical hash` helper.
    - `pub struct Cache { root: PathBuf }` with `open(root) -> Result<Cache, CacheError>`,
      `lookup(&CacheKey) -> Result<Option<CachedOutput>, CacheError>` (verify-on-read; any
      anomaly → `Ok(None)`, i.e. a miss), and `store(&CacheKey, ext: &str, bytes: &[u8]) ->
      Result<(), CacheError>` (atomic temp→rename).
    - `pub struct CachedOutput { pub ext: String, pub bytes: Vec<u8> }`.
    - `pub const CACHE_SCHEMA_VERSION: u32`, `pub const DEFAULT_CACHE_DIR: &str = ".crustyimg/cache"`,
      `pub const CACHE_ENTRY_MAX_BYTES: usize` (bound the read), and a compiled-in
      `feature_signature() -> String`.
    - `pub enum CacheError` (thiserror) — I/O opening/creating the store root (the only variant
      that should ever reach the CLI boundary; lookup/store degrade to a miss rather than error).
    - Library-first: all key/store logic + unit tests here; no `clap`, no pixel decode.
  - `tests/build_cache.rs` — integration tests driving `crustyimg build` (see Failing Tests).
  - `decisions/DEC-058-*.md` — hasher dep + cache-key composition + store design (emit at build).
- **Files modified:**
  - `src/build/mod.rs` — `pub mod cache;`.
  - `src/cli/mod.rs` — extract `encode_one` from `apply_one` (behavior-preserving); wire the
    cache into `run_build`'s execute phase (key → lookup → hit materialize / miss encode+store);
    add `--no-cache` to `GlobalArgs`; extend the summary to cached/rebuilt/failed; a `CliError`
    arm for `CacheError` if the store-open error can surface (exit code: 5 write-refused or 1
    generic — decide in DEC-058).
  - `Cargo.toml` — the one hasher dep.
- **New exports:** `crustyimg::build::cache::{Cache, CacheKey, CachedOutput, CacheError,
  compute_key, CACHE_SCHEMA_VERSION, DEFAULT_CACHE_DIR}`. The executor wiring stays in `cli`
  (mirrors `run_build`); the reusable worker `encode_one` stays in `cli` next to `apply_one`.

## Acceptance Criteria

- [ ] A second `crustyimg build` with no changes is a **full cache hit**: every output reported
  `cached`, `0 rebuilt`, the decode→pipeline→encode skipped, outputs byte-identical to run 1.
- [ ] Changing **one source file's bytes** rebuilds only that input's output (a miss); every other
  output stays a hit and is byte-identical.
- [ ] Changing a **recipe param** forces a miss + rebuild of every output that recipe produces; the
  new outputs reflect the new recipe.
- [ ] Changing `--quality` forces a miss (safe over-invalidation even for quality-insensitive formats).
- [ ] The **crustyimg version** is in the key: a different version string yields a different key
  (proven by a `src/build/cache.rs` unit test, since the compiled-in version can't change within one
  test binary).
- [ ] A cache **hit materializes a byte-correct output**: deleting an output file and re-running
  restores it from cache byte-for-byte (not a stale/garbage file).
- [ ] A **corrupt or missing cache entry** → clean rebuild: a truncated / hash-mismatched / missing
  entry produces the correct output on re-run, exit 0, no panic.
- [ ] The cache is **local only** (`.crustyimg/cache/` under cwd; no network path exists); `--no-cache`
  bypasses it (no store reads/writes, every input rebuilt); the per-build summary reports cached /
  rebuilt / failed and `--quiet` suppresses it.
- [ ] **One** new dependency, recorded in **DEC-058**; `just deny` green with **no exception**;
  `cargo build --no-default-features` (lean) still succeeds; `apply`'s behavior is unchanged by the
  `encode_one` extraction.
- [ ] `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` clean; every new public fn tested.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/build/cache.rs`** (in a `#[cfg(test)] mod tests`)
  - `"key_is_stable_for_identical_inputs"` — `compute_key` twice with the same components → equal keys.
  - `"key_changes_with_each_output_affecting_input"` — from a fixed base, changing ANY single
    component — input content hash, input extension, recipe hash, quality, crustyimg version,
    feature signature, or the cache-schema version — yields a **different** key. (The correctness
    core: every enumerated input is load-bearing.)
  - `"store_then_lookup_roundtrips"` — `store(key, "png", bytes)` then `lookup(key)` → `Some`
    with `ext == "png"` and byte-identical `bytes`.
  - `"lookup_unknown_key_is_none"` — a never-stored key → `Ok(None)` (a miss, not an error).
  - `"corrupt_entry_is_a_miss"` — a stored entry whose bytes are then truncated / altered so the
    recorded output-hash no longer matches → `lookup` returns `Ok(None)` (verify-on-read), never panics.
  - `"missing_sidecar_or_metadata_is_a_miss"` — an entry missing its self-describing metadata
    (ext / output-hash) → `Ok(None)`.
  - `"oversize_entry_is_a_miss"` — an entry larger than `CACHE_ENTRY_MAX_BYTES` → `Ok(None)` with a
    bounded read (never an unbounded load).
  - `"store_is_atomic_no_partial_entry"` — after `store`, no temp/partial file is visible as a
    committed entry; a committed entry is complete (temp→rename discipline).
  - `"key_path_is_hex_sharded_and_contained"` — a key's on-disk path is hex, sharded by prefix,
    and contains no user-controlled component (no traversal from a key).
- **`tests/build_cache.rs`** (integration, drive the binary; generate PNG fixtures natively)
  - `"second_run_is_all_cache_hits"` — temp project, one target, ≥2 source PNGs + a resize recipe;
    first `crustyimg build` → all rebuilt; second run (no changes) → summary reports all cached,
    `0 rebuilt`; outputs present and byte-identical across the two runs.
  - `"changing_one_input_rebuilds_only_that_output"` — after a full-hit state, modify ONE source
    PNG; re-run → summary shows exactly 1 rebuilt, the rest cached; that output changed, the others
    byte-identical.
  - `"changing_recipe_param_forces_rebuild"` — edit the recipe (e.g. new resize dims); re-run → the
    affected outputs rebuild and reflect the new recipe (not the cached old bytes).
  - `"changing_quality_forces_rebuild"` — re-run with a different `-q`; the affected outputs rebuild.
  - `"hit_materializes_byte_correct_output"` — after a full build, delete one output file (keep the
    cache); re-run → the output is restored from cache, byte-identical to the first build.
  - `"corrupt_cache_entry_triggers_clean_rebuild"` — corrupt an on-disk cache entry, then re-run →
    the build still produces the correct output (rebuild), exit 0, no panic.
  - `"no_cache_flag_bypasses_store"` — `crustyimg build --no-cache` writes no `.crustyimg/cache/`
    entries and rebuilds every input on a repeat run.
  - `"cache_store_is_local_under_project"` — after a build, `.crustyimg/cache/` exists under the
    project dir and is populated; the build completes with no network access (the store is a local
    directory — there is no remote/networked code path).

## Implementation Context

*Read this section (and the files it points to) before starting build. The seam below was read
firsthand during design against the current tree — re-confirm signatures.*

### Decisions that apply
- `DEC-057` — the build executor + manifest this extends. The cache wraps STAGE-020's per-input
  unit; manifest paths (and so the cache dir) are **cwd-relative**. The **injective source→output
  constraint** (stem collision) is a STAGE-022 blocker, NOT resolved here: the cache keys on
  output-*byte* identity, not the output *path*, so two colliding-stem inputs still race at the
  path exactly as today — the cache neither fixes nor worsens it. Do not attempt to fix it here.
- `DEC-004` / `pure-rust-codecs-default` — the hasher must be permissive + pure-Rust-default +
  `just deny`-green with **no exception**. BLAKE3's default build may pull C/SIMD (a `cc` build
  step) — verify it stays within the pure-Rust posture and the no-nasm/Windows CI (use the crate's
  `pure` feature if needed), or fall back to sha2 (RustCrypto, fully pure-Rust). Record the pick + why.
- `DEC-005` — recipes are versioned TOML parsed to a `Recipe`; hash the **canonical parsed** recipe
  (ops + params in order) so a cosmetic edit doesn't bust the cache but a semantic change does.
- `DEC-006` — rayon fan-out; the store must be concurrency-safe (content-addressing + atomic rename).
- `DEC-007` — typed `CacheError` in the library; exit-code mapping only at the `cli` boundary.
- `DEC-015` — partial-batch (exit 6) is unchanged; a cache hit/miss is orthogonal to a per-output failure.
- `DEC-034` — decode caps inherited (a miss still decodes through the guarded pipeline).
- **`DEC-058` (NEW — emit at build)** — the hasher dependency (+ license/deny/pure-Rust rationale),
  the cache-key composition (the enumerated input set), and the store design (layout, atomic write,
  self-describing entry, verify-on-read, corrupt→miss, GC deferred). `affected_scope`:
  `src/build/cache.rs`, `src/build/mod.rs`, `src/cli/mod.rs`, `Cargo.toml`.

### The cache-key inputs (enumerate exhaustively — this is the correctness core)
Hash, domain-separated (length-prefix or a fixed separator between fields so no concatenation
collision):
1. `CACHE_SCHEMA_VERSION` (const; bump to invalidate the whole cache on cache-logic changes).
2. crustyimg version — `env!("CARGO_PKG_VERSION")`.
3. feature signature — the compiled-in crustyimg cargo features that can change encode bytes
   (`avif`, `webp-lossy`, `heic`, …), sorted. **Over-inclusion is safe over-invalidation.**
4. resolved recipe hash — canonical hash of the parsed `Recipe` (ops + params, in order).
5. `global.quality` (with a distinct sentinel for `None`).
6. input file **extension**, lowercased — captures crustyimg's extension-routed decode (the same
   bytes named `.nef` vs `.jpg` decode differently → the extension is output-affecting). Do NOT omit.
7. input file **content hash** — the source bytes.

**Output format is NOT a key input** — it's a pure function of (6)+(7), so a hit implies the same
format, and computing it pre-decode would be circular. The store's entry **records the output ext**
instead, so a hit materializes to the right path without decoding. This is the pivot that lets a hit
skip decode. `out` dir + `name` template are NOT in the key either (identical bytes → one entry,
materialized to N destinations); the destination is where a hit is written, not part of its identity.

### The store (`.crustyimg/cache/`, cwd-relative)
- Content-addressed, sharded: `.crustyimg/cache/<first-2-hex>/<full-hex-key>…`. Entry names are hex
  only — **no user-controlled path component**, so no traversal from a key.
- **Self-describing entry:** records the output **ext** and a **hash of the stored output bytes**
  (for verify-on-read) alongside the bytes. Exact on-disk encoding is a build detail (a bytes file +
  a small sidecar, or a single framed file) as long as it round-trips and a malformed entry is
  detectable.
- **Atomic write:** write to a `tmp/` file under the cache root, then `rename` into the final path,
  so a crashed/concurrent write never leaves a half-entry a later run trusts.
- **Verify-on-read (the corrupt→miss guarantee):** on `lookup`, re-hash the stored bytes and compare
  to the recorded output-hash; any mismatch / truncation / missing metadata / oversize
  (`> CACHE_ENTRY_MAX_BYTES`) / symlinked entry → return `Ok(None)` (a miss) and let the executor
  rebuild. Never panic, never serve unverified bytes.
- Rayon-safe by construction (same key ⇒ identical bytes ⇒ last-writer-wins is harmless).

### Executor seam (reuse the worker, don't duplicate it)
1. **Extract `encode_one` from `apply_one`** (behavior-preserving): the decode→pipeline→encode half,
   returning `(ext, Vec<u8>)` using `img.source_format()` + `encode_to_bytes`. `apply_one` becomes
   `encode_one` + `sink.write_bytes(bytes, &sink_input, ext, overwrite, out)`. `run_apply` and
   `run_build`'s non-cached fallbacks both keep working (assert `apply`'s tests are unchanged).
2. In `run_build`'s execute phase, per input (all `Input::Path` — the manifest rejects stdin):
   - read the file bytes once; `input_hash = hash_bytes(&bytes)`; `input_ext` = the path's extension.
   - `key = compute_key(version, features, recipe_hash, quality, input_ext, input_hash)`
     (`recipe_hash` computed once per target).
   - `--no-cache` → skip lookup/store, run `encode_one` + `write_bytes`, count as `Rebuilt`.
   - else `cache.lookup(&key)?`:
     - **Hit** → `Sink::Dir { dir: out, template, .. }.write_bytes(&cached.bytes, &sink_input,
       &cached.ext, Overwrite::Allow, &mut stdout)` (inherits safe_join / symlink / create-dir /
       overwrite guards). Count `Hit`. **No decode.**
     - **Miss** → `encode_one` → `write_bytes` → `cache.store(&key, &ext, &bytes)`. Count `Rebuilt`.
   - a per-output decode/encode failure is still collected → exit 6 (unchanged).
3. Open the `Cache` once per build (create `.crustyimg/cache/` if missing) before the execute phase;
   a store-open failure is the only `CacheError` that should surface at the CLI (map it in DEC-058).
4. **Summary:** track `Hit`/`Rebuilt`/`Failed` counts; render `built N targets, M outputs (C cached,
   R rebuilt)[, K failed]`. A no-change re-run → `(M cached, 0 rebuilt)`. `--quiet` suppresses it.

### Constraints that apply
- `untrusted-input-hardening` — the cache dir is under the user's tree: hex-only entry names, bounded
  reads (`CACHE_ENTRY_MAX_BYTES`), refuse symlinked entries, verify-on-read, corrupt→miss (never panic
  or serve garbage). The inherited image/source/sink hardening (decode caps, safe_join, symlink guard)
  is unchanged.
- `pure-rust-codecs-default` / `no-new-top-level-deps-without-decision` — one hasher dep, DEC-058,
  `just deny` green with no exception, lean build unaffected.
- `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `clippy-fmt-clean`, `ergonomic-defaults`
  (cache is on by default; `--no-cache` is the opt-out; no required flags).

### Prior related work
- `SPEC-063` (shipped, DEC-057) — the build executor + manifest this wraps; reuse `run_build` /
  `apply_one` / `load_recipe` / `prepare_target`.
- `SPEC-031` (rayon batch) + `apply_one` — the per-input worker to split into `encode_one`.
- `SPEC-035` / DEC-036 (`RECIPE_MAX_BYTES` size-guard) — the pattern for `CACHE_ENTRY_MAX_BYTES`.
- The encoder-determinism experiment (this session) — retired the encoder-nondeterminism risk.

### Out of scope (for this spec specifically)
- Lockfile + `--check`/`--frozen` (STAGE-022); `--watch` (STAGE-023); the injective source→output
  fix (STAGE-022 blocker — do NOT fix here); a remote/networked cache; automatic eviction/GC (a future
  `build --gc`/prune) and a `--cache-dir` override (additive); a `--dry-run`/plan preview; an
  mtime/size fast-path ahead of content hashing (content hash is the source of truth); per-target
  format/quality keys (STAGE-020 follow-up).

## Notes for the Implementer

- Keep key + store in `src/build/cache.rs` (library, unit-tested); keep the executor wiring +
  `encode_one` in `cli`. Do NOT duplicate the decode→pipeline→encode worker — extract it once.
- The `encode_one` split must be behavior-preserving: run `apply`'s existing tests + the new
  `build_cache` tests; `apply` output must be byte-identical before/after.
- Run the load-bearing hasher probe FIRST (a throwaway `cargo add` + `just deny` + lean build +
  `cargo build` on the 3-OS assumptions): confirm permissive license, no `just deny` exception, no
  nasm/C surprise on Windows, pure-Rust-default. Only then commit the dep and write DEC-058.
- Verify-on-read is the load-bearing correctness guard — write the `corrupt_entry_is_a_miss` unit
  test first and make the store satisfy it, so "corrupt → rebuild" is structural, not incidental.
- Version-invalidation is a UNIT test on `compute_key` (the compiled-in version can't change in one
  test binary) — be explicit about that in the test comment; don't fake it at the integration level.
- Emit `DEC-058` with `affected_scope` covering `src/build/cache.rs`, `src/build/mod.rs`,
  `src/cli/mod.rs`, `Cargo.toml`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-058` — hasher dep + cache-key composition + store design (if emitted as designed)
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
