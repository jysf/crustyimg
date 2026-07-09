---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-021
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: 2026-07-09

value_contribution:
  advances: >
    Delivers the project's headline: a content-addressed cache so `crustyimg build`
    does ONLY the work that actually changed. This is the "incremental rebuild" half
    of the "Makefile for images" thesis — a second run with no changes is a full cache
    hit reporting near-zero work, and changing one source / recipe param / quality / the
    crustyimg version rebuilds only the affected outputs. It is the *robust* half of
    "verifiable" (cache-correctness, deterministic-within-a-machine), separate from the
    *fragile* half (cross-arch output-byte reproducibility) that STAGE-022's lockfile owns.
  delivers:
    - "A local, on-disk content-addressed cache under `.crustyimg/cache/` keyed by a hash of every output-affecting input"
    - "`crustyimg build` skips the decode→pipeline→encode for an unchanged (input × recipe × config × version) and materializes the cached output instead"
    - "A per-build summary reporting cached / rebuilt / failed — a no-change re-run says 'all cached, 0 rebuilt'"
    - "A `--no-cache` bypass, and corrupt/missing-entry → clean rebuild (never a panic or a stale artifact)"
    - "A recorded decision (DEC-058) fixing the hasher dependency + the cache-key composition + the store design"
  explicitly_does_not:
    - "Write or check a reproducibility lockfile / `--check` / `--frozen` (STAGE-022) — the cache proves 'same env → same work', not 'cross-arch → same bytes'"
    - "Watch files / `--watch` (STAGE-023)"
    - "Resolve the injective source→output constraint (the stem-collision hazard, DEC-057) — that stays a STAGE-022 blocker; the cache neither fixes nor worsens it"
    - "Add a remote / networked / distributed cache — local only (the no-service, no-CDN guardrail)"
    - "Evict / GC old entries automatically (deferred; the cache grows until manually cleared)"
---

# STAGE-021: the content-addressed cache (incremental rebuild)

## What This Stage Is

The stage that makes `crustyimg build` **incremental**. STAGE-020 shipped a declared
build that re-does every target on every run; this stage wraps each per-input unit of
work in a **content-addressed cache**. Before decoding an input, the executor computes a
key from everything that could change the output — the input file's content + extension,
the resolved recipe, the encode quality, the crustyimg version, and the compiled-in
codec features — and looks it up in a local on-disk store (`.crustyimg/cache/`). On a
**hit** it materializes the cached output bytes to the target path and skips the whole
decode→pipeline→encode; on a **miss** it runs the shipped worker, writes the output, and
stores the result under the key. A no-change re-run becomes a full cache hit that reports
near-zero work; changing exactly one output-affecting input rebuilds exactly the affected
outputs. No lockfile, no `--check`, no watch (those are STAGE-022/023) — this is the
robust, deterministic-within-a-machine "incremental" leg, and the cache-key contract the
lockfile will later key on.

## Why Now

- **It's the project's headline capability.** STAGE-020 gave us a declared build; without
  the cache, "a re-run does only the work that changed" — the top success signal in the
  brief — is unmet. Everything after (lockfile, watch) assumes an incremental build exists.
- **The load-bearing risk is now retired.** The encoder-determinism experiment (this
  session, arm64 release) confirmed every encoder — including AVIF (rav1e) and lossy WebP
  (libwebp) — is byte-identical run-to-run and across thread counts on a fixed machine. So
  the cache is de-risked: same (input × recipe × config × version) → same output bytes on a
  given machine, and a cache is local/per-machine, so cross-arch byte-identity never enters
  into it (that stays STAGE-022's problem). Determinism-*within-env* is all the cache relies
  on, and that held. *(Caveat: rav1e's threading lever may not be `RAYON_NUM_THREADS`, so we
  don't over-claim thread-invariance in general — only that it held under test.)*
- **The remaining hard part is cache-key correctness, and it's a design problem we can
  fully enumerate now** — miss one output-affecting input and the build ships a stale
  artifact silently. This stage's whole discipline is enumerating that input set (below and
  in SPEC-064) and testing that each member forces a miss.

## Success Criteria

- `crustyimg build` run twice with no changes: the second run is a **full cache hit** —
  every output reported `cached`, `0 rebuilt`, and the decode→pipeline→encode is skipped.
- Changing exactly one thing — one source file's bytes, one recipe param, the `--quality`,
  or (proven at the key level) the crustyimg version — forces a **miss on only the affected
  outputs**; everything else stays a hit.
- A cache **hit materializes a byte-correct output** (identical to what a fresh build would
  write) — deleting an output and re-running restores it from cache, byte-for-byte.
- A **corrupt or missing cache entry falls back to a clean rebuild** — never a panic, never
  a stale/garbage artifact served from a bad entry.
- The cache is **local only** (`.crustyimg/cache/`, no network path anywhere); a `--no-cache`
  flag bypasses it; the per-build summary reports cached / rebuilt / failed.
- **One new dependency** (a hasher), gated behind **DEC-058**; `just deny` green with **no
  exception**; the lean build (`--no-default-features`) still succeeds; pure-Rust-default posture
  (DEC-004) preserved.

## Scope

### In scope
- A `src/build/cache.rs` module: the cache-key composition (hash of the enumerated
  output-affecting inputs), a local content-addressed store under `.crustyimg/cache/`
  (layout, atomic writes, self-describing entries, verify-on-read, corrupt→miss), and a typed
  `CacheError` — all library, unit-tested.
- Wiring `run_build` (STAGE-020's executor) to the cache: compute the key per input, hit →
  materialize, miss → run the shipped worker + store. Reuse `apply_one`'s worker (extract a
  behavior-preserving `encode_one` that produces `(ext, bytes)`, shared by `apply` and the
  cache-miss path — do NOT duplicate the worker).
- A `--no-cache` bypass flag; a per-build summary extended to cached / rebuilt / failed.
- The new hasher dependency (license-probed, `just deny` green) recorded in **DEC-058**
  alongside the cache-key + store design. **(SPEC-064)**

### Explicitly out of scope
- Lockfile + `--check`/`--frozen` (STAGE-022); `--watch` (STAGE-023); the injective
  source→output constraint (DEC-057 — the cache is keyed on output-byte identity, not the
  output path, so it neither fixes nor worsens the stem collision; STAGE-022 stays blocked on
  it); a remote/networked cache; automatic eviction/GC (the cache grows until cleared — a
  future `build --gc`/prune, and a `--cache-dir` override, are additive follow-ups); a
  `--dry-run`/plan preview (natural companion — note as follow-up); an mtime/size fast-path
  ahead of content hashing (content-hash is the source of truth this stage).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-064 (shipped on 2026-07-09) — content-addressed cache: `src/build/cache.rs` (cache-key over
  every output-affecting input + local `.crustyimg/cache/` store, atomic temp→rename + self-describing
  entries + verify-on-read + corrupt→miss) wired into `run_build` (hit → materialize via `write_encoded`,
  miss → `encode_one` + store), `--no-cache`, `(C cached, R rebuilt)` summary; **one new dep (`sha2`)** →
  DEC-058. PR #70 (2c41c06), 24/24 CI green, 627 tests on default + lean. Verify APPROVED (fresh session):
  reproduced cold→warm→edit→restore→corrupt→`--no-cache` on the real binary + a mutation test proving
  corrupt→miss is structural. Injective source→output (DEC-057) untouched — carried to STAGE-022.

**Count:** 1 shipped / 0 active / 0 pending — single-spec stage complete.

## Design Notes

- **PROBE RESULT (this session) — the cache is fully de-risked on determinism.** The
  encoder-determinism experiment (arm64 release) confirmed byte-identical output run-to-run
  and across thread counts (1 vs 8) for every encoder incl. AVIF/rav1e and lossy WebP/libwebp.
  A cache is local/per-machine, so *cross-arch* byte-identity is irrelevant here (STAGE-022).
  The cache relies only on determinism-within-env, which held. **The remaining risk is
  cache-key correctness, not encoder nondeterminism.**
- **DEC-058 (at build):** the hasher dependency (**`sha2` recommended** — RustCrypto, pure-Rust, no
  `build.rs` C, permissive, `just deny` green with no exception → **no probe**, just `cargo add` + the
  standard gates; `blake3` only with its `pure` feature if a hash bottleneck ever shows), the
  **cache-key composition** (the enumerated input set below), and the **store design** (layout,
  atomic write, self-describing entry, verify-on-read, corrupt→miss, GC-deferred).
- **Cache-key inputs — the correctness core (enumerate exhaustively; each must force a miss).**
  The key is a hash over, domain-separated:
  1. a **cache-schema version** constant (bump to invalidate the whole cache when the cache
     logic itself changes);
  2. the **crustyimg version** (`env!("CARGO_PKG_VERSION")`);
  3. a **compiled-in feature signature** (the set of crustyimg cargo features that could change
     encode bytes: `avif`, `webp-lossy`, `heic`, … — over-inclusion is a *safe* over-invalidation);
  4. the **resolved recipe** (a canonical hash of the parsed `Recipe`'s ordered ops + params —
     so a comment-only edit doesn't bust the cache, but a param change does; hashing the raw
     recipe file bytes is the conservative fallback);
  5. the **encode quality** (`global.quality`, with a sentinel for `None`);
  6. the **input file extension**, lowercased (captures crustyimg's *extension-routed decode* —
     PROJ-009: the same bytes named `.nef` vs `.jpg` decode differently, so the extension is
     output-affecting and MUST be in the key);
  7. the **input file content hash** (the source bytes).
- **Output format is deliberately NOT in the key — it's recovered from the entry.** The output
  format/ext is a pure function of (input bytes, extension), both already in the key, so a hit
  implies the same format. Computing the format before decode would be circular (it needs the
  decode/routing). Instead the store's **entry is self-describing**: it records the output
  extension, so a hit materializes to the right path *without decoding*. This is the pivot that
  lets a hit skip decode entirely.
- **Key is on output-byte identity, NOT the output path.** `out` dir + `name` template do not
  enter the key — identical source+recipe+config produces identical bytes regardless of
  destination, so one entry materializes to N paths (max reuse). The *destination* is where a
  hit gets written; the *key* identifies the bytes.
- **Store design (`.crustyimg/cache/`, cwd-relative like DEC-057's manifest paths):**
  content-addressed, sharded by a key prefix (`<ab>/<full-hex-key>…`) to avoid one giant dir;
  each entry is **self-describing** (records the output ext + a hash of the stored output bytes)
  and written **atomically** (temp file in a `tmp/` subdir → `rename` into place) so a crashed or
  concurrent write never leaves a half-entry a later run trusts. **Verify-on-read:** re-hash the
  stored bytes against the entry's recorded output-hash; any mismatch, truncation, missing
  sidecar, oversize entry, or symlinked entry → treat as a **miss** and rebuild (never panic,
  never serve the bad bytes). Rayon-concurrency-safe by construction (content-addressing +
  atomic rename; same key ⇒ identical bytes ⇒ last-writer-wins is harmless).
- **Seam into the executor (reuse, don't duplicate):** extract a behavior-preserving
  `encode_one(&Recipe, &registry, &Input) -> Result<(ext, Vec<u8>)>` from `apply_one` (the
  decode→pipeline→encode half); `apply_one` becomes `encode_one` + `sink.write_bytes` (its
  behavior is unchanged, so `apply`'s tests still hold). In `run_build`'s execute phase, per
  input: read+hash the file bytes → build the key → `cache.lookup(key)`; **hit** →
  `Sink::Dir::write_bytes(cached_bytes, …, ext, Overwrite::Allow)` (inherits the sink's
  safe_join / symlink / create-dir / overwrite guards); **miss** → `encode_one` → `write_bytes`
  → `cache.store(key, ext, bytes)`. Build inputs are always `Input::Path` (manifest rejects
  stdin, DEC-057), so the pre-decode file read/hash is always available.
- **Reporting:** each input resolves to `Hit` / `Rebuilt` / `Failed`; the summary gains
  `(C cached, R rebuilt)` — a no-change re-run reads `built N targets, M outputs (M cached, 0
  rebuilt)`, the "zero work" success signal. `--quiet` still suppresses it.
- **Hardening:** the cache lives under the user's tree; entries are named only by hex key (no
  user-controlled path component → no traversal from a key), reads are size-bounded, symlinked
  entries are refused, and every structural anomaly degrades to a rebuild. Typed `CacheError`;
  no `unwrap`/`expect` on recoverable paths.

## Dependencies

### Depends on
- STAGE-020 (shipped): `run_build` + `prepare_target` + `PreparedTarget` + the `apply_one`
  worker + `load_recipe` (`src/cli/mod.rs`, DEC-057); `src/build/` (manifest); `src/sink/`
  (`write_bytes`, `expand_template`, `safe_join`, `encode_to_bytes`, `Overwrite`); `src/recipe/`
  (parsed `Recipe` to hash); `src/source/` (`Input::Path`).
- DEC-004 (pure-Rust default) + `pure-rust-codecs-default` — the hasher must be permissive,
  pure-Rust-default, and `just deny`-green with no exception; DEC-034 decode caps (inherited);
  the `untrusted-input-hardening` posture (applied to cache entries too).

### Enables
- STAGE-022 (lockfile + `--check`) keys on this cache-key contract; STAGE-023 (`--watch`)
  re-runs affected targets against the same cache. Faster large-tree runs generally.

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes — `crustyimg build` is now
  incremental. A no-change re-run is a full cache hit (`8 cached, 0 rebuilt`, decode skipped,
  confirmed both by timing and structurally — a hit returns before `encode_one` is reachable);
  changing one source / recipe param / quality rebuilds only the affected outputs; a deleted output
  is restored byte-for-byte from cache; a corrupt entry rebuilds cleanly (exit 0, no panic — proven
  structural by a mutation test that deleted verify-on-read); the store is local-only with a
  `--no-cache` bypass. PR #70, 24/24 CI green, one pure-Rust dep (`sha2`), `just deny` green with no
  exception, lean build unaffected.
- **How many specs did it actually take?** 1 (SPEC-064), as planned — a clean single-spec stage.
- **What changed between starting and shipping?** The seam landed as three functions
  (`encode_one` / `write_encoded` / `apply_one`) rather than the two the spec sketched — the write
  half is shared verbatim by the cache-hit path, so extracting it beat inlining it twice. One
  accepted behavioral delta: `apply` now encodes before the sink's guards (byte-identical output,
  only the error *ordering* changes on a double-failure path; safe because encode targets an in-memory
  `Cursor`, so no bytes reach disk before any guard).
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **When a spec demands a test proving a compiled-in constant is load-bearing, put that constant
    in the function's parameters** — SPEC-064 asked for a "schema version changes the key" test but
    gave `compute_key` a signature with no schema argument, forcing a private
    `compute_key_with_schema` workaround. A spec-authoring habit, not a template change.
  - **Name the decision behind reused guards.** The Implementation Context leaned on the sink's
    symlink/overwrite guards (DEC-035) and mentioned a new symlinked-cache-entry surface without
    tying either to DEC-035 — cheap to have listed. Also: a spec that adds a `GlobalArgs` field
    should note its hand-built `#[cfg(test)]` initializers (adding `--no-cache` broke the lib tests
    first).
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - The **injective source→output constraint (DEC-057)** is a project-level invariant, not a
    SPEC-064 detail: the cache keys on output-byte identity (not path), so it is neither fixed nor
    worsened here and **STAGE-022 (lockfile) stays blocked on it**. It lives in DEC-057's validation
    list; carried into STAGE-022 framing.
  - One non-blocking defect worth carrying: `store` bounds the payload at `CACHE_ENTRY_MAX_BYTES`
    while `read_entry` bounds the whole frame (payload + 53-byte header), so a near-cap payload is
    stored but never readable (a permanent silent miss — correctness unaffected, disk wasted). A
    one-line fix when the store is next touched.
