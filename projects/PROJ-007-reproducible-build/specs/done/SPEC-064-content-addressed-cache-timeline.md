# SPEC-064 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — the content-addressed cache (STAGE-021 headline): `src/build/cache.rs` (cache-key
  over every output-affecting input + a local `.crustyimg/cache/` content-addressed store, atomic
  temp→rename, self-describing entries, verify-on-read, corrupt→miss) wired into `run_build`
  (hit → materialize via `sink.write_bytes`; miss → `encode_one` + `store`), a `--no-cache` bypass, and
  a cached/rebuilt summary. Failing Tests: key-stable / key-changes-per-input (content, ext, recipe,
  quality, version, features, schema) / store-roundtrip / unknown-miss / corrupt-miss / missing-sidecar
  / oversize-miss / atomic-store / hex-sharded-contained (unit); second-run-all-hits /
  one-input-rebuilds-only-that / recipe-param-rebuild / quality-rebuild / hit-materializes-byte-correct
  / corrupt-entry-clean-rebuild / no-cache-bypass / local-store (integration). Full Implementation
  Context: the enumerated cache-key inputs (**output format deliberately NOT keyed — recovered from the
  self-describing entry; input ext IS keyed for extension-routed decode**), the store design, and the
  executor seam (**extract a behavior-preserving `encode_one` from `apply_one` — reuse, don't
  duplicate**). Hasher = a boring pure-Rust dep (**`sha2` recommended**) — **no probe**, just `cargo add`
  + the standard `just deny`/lean/CI gates → **DEC-058** (hasher dep + cache-key + store); avoid `blake3`'s
  C/SIMD Windows/no-nasm risk for hash speed the cache doesn't need. Encoder-determinism experiment (this
  session) retired the nondeterminism
  risk. Injective source→output constraint (DEC-057) is NOT resolved here — cache keys on byte-identity,
  not path. Framing, 2026-07-08.
- [x] **build** — implemented `src/build/cache.rs` (key + store, 16 unit tests) + `tests/build_cache.rs`
  (10 integration tests); wired `run_build` (hit → materialize, miss → encode + store); extracted
  `encode_one` + `write_encoded` from `apply_one` (behavior-preserving; `apply`'s tests unchanged);
  added `--no-cache` + the `(C cached, R rebuilt)` summary; `cargo add sha2` (=0.11.0, the only new
  dep) → **DEC-058**. Gates green: 627 tests on default AND lean, clippy `-D warnings` on both,
  `cargo fmt --check`, `just deny` (advisories/bans/licenses/sources ok, `deny.toml` untouched, no new
  exception), MSRV unchanged (sha2 tree floors at 1.85 < our 1.90). Drove the release binary on an
  8-image tree: cold `(0 cached, 8 rebuilt)` → warm `(8 cached, 0 rebuilt)` → one source edited
  `(7 cached, 1 rebuilt)` → deleted output restored from cache → all entries corrupted → clean rebuild,
  exit 0, no panic → `--no-cache` bypasses and writes nothing. Deviations + follow-ups in the spec's
  Build Completion. Injective source→output (DEC-057) untouched, as instructed. Branch
  `feat/spec-064-content-addressed-cache`, PR #70. 2026-07-08.
- [x] **verify** — ✅ APPROVED, fresh session, 2026-07-09. All 9 acceptance criteria met. Gates from
  clean: 627 tests pass on default **and** lean (`tests/build_cache.rs` = 10 integration tests, the 8
  spec'd + 2 extras; `src/build/cache.rs` = 16 units, the 9 spec'd + 7 extras — all present and
  running, diffed against the spec's Failing Tests); clippy `-D warnings` clean on both; `cargo fmt
  --check` clean; `just deny` → `advisories ok, bans ok, licenses ok, sources ok` with `deny.toml`
  byte-identical to `main` and no new exception; `Cargo.lock` adds only `sha2` + 7 pure-Rust RustCrypto
  transitives, nothing removed; `rust-version = 1.90.0` untouched. `decisions-audit` → 0 structural
  errors; `--changed main` flags DEC-057/058 (advisory), both consistent — DEC-058 extends the executor
  and explicitly declines the injective fix. Independent end-to-end drive of the release binary on an
  8-image tree (driver written fresh, **not** the build session's buggy script): cold `(0 cached, 8
  rebuilt)` → warm `(8 cached, 0 rebuilt)`, outputs byte-identical → one source edited `(7 cached, 1
  rebuilt)`, exactly one output changed → deleted output restored byte-for-byte → all 9 entries
  corrupted → `(0 cached, 8 rebuilt)`, exit 0, no panic, outputs still correct → `--no-cache` creates no
  `.crustyimg/` at all and rebuilds on repeat → `--quiet` emits zero bytes. Also: `-q 60` forces a miss
  and coexists as a distinct key; a *cosmetic* recipe edit stays a hit while a semantic one misses
  (canonical-hash payoff); a blocked cache root exits **5** with no panic and `--no-cache` gets past it.
  Decode-skip corroborated on a 2600×2600 source (warm 0.029s vs cold 0.094s / `--no-cache` 0.090s) and
  structurally in `build_one` (a hit returns before `encode_one` is reachable). Corrupt→miss proven
  **structural** by a mutation test: deleting the verify-on-read re-hash makes `corrupt_entry_is_a_miss`
  fail — and only that test. `apply` unchanged: no test file but `tests/build_cache.rs` was added, and
  the encode path is the same `encode_to_bytes(out_img, source_format, quality)` +
  `extension_for_format` pair the old `Sink::Dir { format: Some(fmt) }` used. One accepted deviation:
  `encode_one` now encodes *before* the sink's overwrite/traversal/symlink guards — **accepted**, since
  no bytes reach disk before the guards (encode targets an in-memory `Cursor`), the guard set is
  unchanged and still precedes `open()`, and only the reported error *ordering* changes when an input
  would fail both. DEC-057's injective source→output blocker untouched (no duplicate-output-path check
  added to `prepare_target`) — STAGE-022 stays blocked on it, as designed.
- [x] **ship** — squash-merged PR #70 → main (**2c41c06**); re-applied the verify cost session + timeline
  verify mark on main after merge (stash-pop, not on the feature branch, AGENTS §13); appended the ship
  cost session + `cost.totals` (740k tok / ~$14.80, 4 sessions — build+verify are labelled main-loop
  estimates §4) + ship reflection; archived spec+timeline to `done/`; `just cost-audit` green. **STAGE-021
  SHIPPED** (single-spec stage) + PROJ-007 brief stage plan updated. Carried non-blocking follow-ups
  (`--gc`/`--cache-dir`, mtime fast-path, `--dry-run`, double-read removal, the `CACHE_ENTRY_MAX_BYTES`
  off-by-53 read bound) and the **DEC-057 injective source→output blocker** into STAGE-022 (lockfile), next.
  2026-07-09.
