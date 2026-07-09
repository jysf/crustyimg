---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-058
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-08
supersedes: null
superseded_by: null

affected_scope:
  - src/build/cache.rs
  - src/build/mod.rs
  - src/cli/mod.rs
  - Cargo.toml

tags:
  - build
  - cache
  - hashing
  - dependencies
  - reproducibility
  - hardening
---

# DEC-058: `sha2` as the build cache's hasher; the cache-key composition; the store design

## Decision

The `crustyimg build` cache hashes with **`sha2`** (RustCrypto SHA-256 — one new
top-level dependency); its **key** is a domain-separated digest of exactly seven
output-affecting inputs — cache-schema version, crustyimg version, compiled-in feature
signature, canonical recipe hash, encode quality, input **extension**, input **content
hash** — and its **store** is a local, sharded, content-addressed directory
(`.crustyimg/cache/`) of self-describing entries, written atomically (`tmp/` →
`rename`) and **verified on read**, where every structural anomaly degrades to a plain
cache miss.

The output **format is deliberately not keyed** — it is recovered from the entry, which
records its own extension. That inversion is what lets a cache hit skip decode entirely.

## Context

STAGE-020 (SPEC-063, DEC-057) shipped a declared build that re-does every target on
every run. STAGE-021's headline is "a re-run does only the work that changed," so
SPEC-064 wraps each per-input unit of `run_build` in a content-addressed cache.

Three things had to be fixed together, which is why they are one decision:

1. **A hasher.** The repo had none. `no-new-top-level-deps-without-decision` and
   `pure-rust-codecs-default` (DEC-004) both apply.
2. **The key.** This is the correctness core. Miss one output-affecting input and the
   build ships a **stale artifact, silently** — the worst failure mode a build tool has.
3. **The store.** It lives under the user's tree, so on read it is untrusted input
   (`untrusted-input-hardening`), and rayon (DEC-006) writes it concurrently.

The load-bearing risk was retired before design: an encoder-determinism experiment
(2026-07-08, arm64 release) confirmed every encoder — including AVIF/rav1e and lossy
WebP/libwebp — is byte-identical run-to-run and across thread counts on a fixed machine.
A cache is local and per-machine, so cross-arch byte-identity never enters into it (that
stays STAGE-022's problem). What remained was key correctness, not nondeterminism.

## Alternatives Considered

### The hasher

- **Option A: `blake3`**
  - What it is: the fastest widely-used tree hash.
  - Why rejected: its default build compiles C/SIMD (and assembly) via `build.rs`. This
    repo has been bitten twice by exactly that class of dependency on the Windows
    no-nasm CI job (SPEC-058, SPEC-062). Taming it means pinning the `pure` feature —
    at which point the speed argument that justified it is gone. And the argument was
    never strong: the cache's win is skipping a **decode + encode**, which costs orders
    of magnitude more than hashing a few MiB. Hash throughput is immaterial here.

- **Option B: a hand-rolled / non-cryptographic hash (`ahash`, FxHash, CRC)**
  - What it is: no new crypto dependency; hash the inputs with something cheap.
  - Why rejected: a build cache's key is a **collision-is-a-wrong-answer** surface. A
    64-bit non-cryptographic hash over an asset tree has a birthday collision at a few
    hundred million entries and, worse, is trivially collidable on purpose. A stale
    artifact served from a colliding key is silent and undetectable. Cryptographic
    strength is the point, not a nicety.

- **Option C (chosen): `sha2`**
  - What it is: RustCrypto SHA-256. MIT OR Apache-2.0, pure Rust, no `build.rs` C.
  - Why selected: boring, permissive, ubiquitous, and it sails through the standard
    gates — so **no design-time probe was needed**, just `cargo add` plus `just deny` /
    lean-build / CI confirmation. It adds `sha2` plus seven pure-Rust RustCrypto
    transitives (`digest`, `crypto-common`, `block-buffer`, `hybrid-array`, `typenum`,
    `const-oid`, `cpufeatures`), all floored at Rust 1.85 — under our 1.90 MSRV, so the
    floor is unchanged. `just deny` stays green with **no new exception**;
    `deny.toml` is untouched; the lean `--no-default-features` build is unaffected.

If a measured hashing bottleneck ever appears, revisit `blake3` **with its `pure`
feature**, not its default build.

### The key: what is in it

Hashed with a one-byte **field tag** and a `u64` **length prefix** per field, so no two
field values can concatenate into a third (`("ab","c")` must not equal `("a","bc")`):

1. `CACHE_SCHEMA_VERSION` — bump to invalidate the whole cache when the cache logic
   itself changes.
2. crustyimg version (`env!("CARGO_PKG_VERSION")`) — a new binary may encode differently.
3. **Feature signature** — the compiled-in cargo features that can change encode bytes
   (`avif`, `heic`, `webp-lossy`), sorted. Over-inclusion is a *safe* over-invalidation;
   omission serves a stale artifact. This is also what stops an AVIF-enabled build's
   entry from being served to a default build that cannot encode AVIF.
4. **Canonical recipe hash** — the digest of the parsed `Recipe`'s round-tripping TOML
   (DEC-005), not the recipe file's raw bytes. A comment or whitespace edit is therefore
   a hit; a changed param is a miss.
5. **Quality** — with a distinct sentinel tag for `None`, so "no `-q`" and `-q 0` are
   different keys. Keyed even for formats that ignore quality (PNG): safe over-invalidation.
6. **Input extension**, lowercased. Load-bearing, not decoration: crustyimg routes decode
   by extension (RAW preview extraction, SPEC-061/DEC-055), so the same bytes named
   `.nef` and `.jpg` decode to different pixels. Omitting it is a correctness bug.
7. **Input content hash** — the source file's bytes. Content, not mtime/size: an
   mtime/size fast-path is explicitly out of scope (a future optimization *in front of*
   the content hash, never a replacement for it).

### The key: what is deliberately NOT in it

- **The output format / extension.** It is a pure function of (6) + (7), both already
  keyed, so a hit already implies the same format — and computing it up front would
  require exactly the decode that a hit exists to skip. Keying it would be circular.
  Instead the **entry records its own output extension**. This inversion is the pivot of
  the whole design: it is what makes a hit a pure `read + write`, with no decode.
- **The output destination** (`out` dir, `name` template). Identical inputs produce
  identical bytes wherever they land, so one entry materializes to N destinations (max
  reuse). The key identifies the **bytes**; the manifest decides where they go.

  A direct consequence: the cache keys on **output-byte identity, not output path**, so
  it neither fixes nor worsens DEC-057's non-injective source→output (stem-collision)
  hazard. Two colliding-stem inputs still race at the destination exactly as they do
  today. That remains STAGE-022's blocker, untouched here.

### The store

- **Layout:** `.crustyimg/cache/<first-2-hex>/<full-64-hex>`, cwd-relative like every
  other manifest path (DEC-057). Sharded so one flat directory never holds every entry.
  Every path component below the root is **hex only** — no caller-controlled string
  (stem, template, extension) ever reaches a path component, so a key cannot traverse
  out of the store. Rejected: naming entries after the source path (traversal surface,
  and it would break the one-entry-many-destinations property).

- **Self-describing entry**, a single framed file rather than a bytes-file + sidecar:
  `MAGIC | ext_len | ext | payload_len | payload_hash | payload`. One file means one
  `rename` and therefore one atomic commit; a sidecar pair can tear (bytes committed,
  metadata not) and would need its own consistency rule.

- **Atomic write:** stage in `tmp/` under the root, then `rename` into place. The
  staging name carries the pid and an atomic counter, so two rayon tasks storing the
  same key never share a temp path. Content-addressing makes the fan-out safe by
  construction: same key ⇒ identical bytes ⇒ last-writer-wins is harmless. No `fsync` —
  a torn post-crash entry is caught by verify-on-read and rebuilt, so paying an fsync
  per output would buy nothing.

- **Verify-on-read (the corrupt→miss guarantee):** `lookup` refuses a non-regular file
  (a symlink is rejected, never followed), bounds the read at `CACHE_ENTRY_MAX_BYTES`
  (256 MiB), parses the frame, and **re-hashes the payload against the hash the entry
  recorded**. Truncation, a flipped byte, appended garbage, a bad magic, missing
  metadata, an oversize or symlinked entry — every one returns `Ok(None)`, a plain miss.
  The executor answers a miss by rebuilding, which is always correct. The cache never
  panics and never serves an unverified byte.

- **Errors:** `CacheError` is deliberately tiny. `lookup` cannot fail. A failed `store`
  costs the next run a rebuild and is warned about, not raised. Only `Cache::open`
  (cannot create the store root) reaches the CLI boundary → **exit 5** (write refused),
  the same code the sink's write failures use; `--no-cache` is the documented way past it.

- **GC/eviction is deferred.** The cache grows until manually cleared (`rm -rf
  .crustyimg`). A `build --gc`/prune and a `--cache-dir` override are additive follow-ups.

### The executor seam

`apply_one` was split into a **behavior-preserving** `encode_one` (decode → pipeline →
encode, returning `(ext, bytes)`) plus a `write_encoded` (the `Sink::Dir` write, which
carries the create-dir / traversal / symlink / overwrite guards). `apply_one` is now
those two in sequence; `run_build`'s miss path is the same two plus a `store`; the hit
path is `write_encoded` alone. The worker is extracted once, never duplicated, so the
cached and uncached paths cannot drift into producing different bytes — and a cached
byte reaches disk through exactly the guards a fresh one does.

Rejected: copying the worker into the build path. That is how a cache and its
uncached fallback silently diverge.

## Consequences

- **Positive:**
  - The project's headline: a no-change re-run is a full cache hit reporting `(N cached,
    0 rebuilt)` and skipping every decode/encode. Verified end-to-end on an 8-image
    tree: cold `(0 cached, 8 rebuilt)` → warm `(8 cached, 0 rebuilt)`; one edited source
    → `(7 cached, 1 rebuilt)`.
  - A deleted output is restored **from cache, byte-for-byte** — a hit writes, it does
    not merely skip.
  - The key is the contract STAGE-022's lockfile will pin and STAGE-023's `--watch` will
    re-run against.
  - Corrupt store → clean rebuild, exit 0, no panic. The cache is a pure optimization:
    deleting or breaking it can never change what a build produces.

- **Negative:**
  - The cache grows unbounded until cleared (no GC).
  - Every input's bytes are read and hashed **before** decode, even on a miss — where
    the file is then read again by `Image::load`. One extra read of the source on the
    miss path, dwarfed by the decode it precedes; the hit path reads it exactly once.
  - Over-invalidation by design: `-q 90` on an all-PNG build rebuilds everything even
    though PNG ignores quality. Correct, and cheap relative to the alternative.
  - A hit trusts that the machine's encoders are deterministic. Held under experiment;
    it is a *within-machine* assumption, and a wrong one degrades to a stale output. The
    schema version is the escape hatch.

- **Neutral:**
  - `--no-cache` is a global flag (clap `global = true`) and so parses on every
    subcommand, but only `build` reads it. Consistent with `-q`/`--jobs`.
  - A `--no-cache` build creates no `.crustyimg/` directory at all.
  - Build inputs are always `Input::Path` (the manifest rejects `-`, DEC-057). A stdin
    input would fall through to an uncached rebuild rather than panic.

## Validation

Right if STAGE-022's lockfile can pin this key without reshaping it, and if no
"stale artifact" bug is ever traced to a missing key input. Revisit if:

- a profile shows hashing (not decode) dominating a large build → `blake3` with `pure`;
- the store's growth becomes a complaint → `build --gc` / `--cache-dir`;
- an output-affecting input is discovered outside the seven (→ bump
  `CACHE_SCHEMA_VERSION` in the same change, which invalidates every stale entry);
- per-target `format`/`quality` overrides land (STAGE-020 follow-up) → they join the key.

## References

- Related specs: SPEC-064 (this decision's spec), SPEC-063 (the executor it extends),
  SPEC-031 (rayon batch + `apply_one`), SPEC-035/DEC-036 (the size-guard pattern)
- Related decisions: DEC-057 (build manifest + executor; the injective source→output
  blocker this does NOT resolve), DEC-004 (pure-Rust default), DEC-005 (recipes — the
  canonical form hashed here), DEC-006 (rayon), DEC-007 (typed errors), DEC-015
  (partial-batch exit 6), DEC-034 (decode caps, inherited on the miss path), DEC-035
  (symlink-destination guard)
- Stage: `projects/PROJ-007-reproducible-build/stages/STAGE-021-content-addressed-cache.md`
- User docs: `docs/cli-reference.md` (`build [FILE]`, `--no-cache`)
