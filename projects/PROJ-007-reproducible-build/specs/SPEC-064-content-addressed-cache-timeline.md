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
- [ ] **verify** — fresh session. Re-run all gates; reproduce against the real binary: a no-change
  re-run is a full cache hit (0 rebuilt), a one-input edit rebuilds only that output, a hit restores a
  deleted output byte-for-byte, a corrupted entry rebuilds cleanly (no panic), `--no-cache` bypasses,
  the store is local. Confirm `encode_one` didn't change `apply`, and no new `just deny` exception.
- [ ] **ship** — merge PR; append verify + ship cost sessions + totals + reflection; archive to done/;
  advance STAGE-021 (single-spec stage → shipped) and update the PROJ-007 brief; STAGE-022 (lockfile)
  next — carrying the injective source→output blocker (DEC-057).
