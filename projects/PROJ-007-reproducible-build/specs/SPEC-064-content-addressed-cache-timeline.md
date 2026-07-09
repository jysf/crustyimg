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
  duplicate**). Load-bearing probe **deferred to build**: pick + license-check the hasher (BLAKE3 vs
  sha2/xxhash — permissive, pure-Rust-default, `just deny` green with no exception) → **DEC-058** (hasher
  dep + cache-key + store). Encoder-determinism experiment (this session) retired the nondeterminism
  risk. Injective source→output constraint (DEC-057) is NOT resolved here — cache keys on byte-identity,
  not path. Framing, 2026-07-08.
- [ ] **build** — implement `src/build/cache.rs` + wire `run_build`; extract `encode_one`; add
  `--no-cache`; extended summary; one hasher dep + DEC-058. Run the hasher probe FIRST (`just deny` +
  lean + Windows/no-nasm). Make all Failing Tests pass; keep `apply` byte-identical. Verify default +
  lean + `just deny` (no new exception) + clippy + fmt.
- [ ] **verify** — fresh session. Re-run all gates; reproduce against the real binary: a no-change
  re-run is a full cache hit (0 rebuilt), a one-input edit rebuilds only that output, a hit restores a
  deleted output byte-for-byte, a corrupted entry rebuilds cleanly (no panic), `--no-cache` bypasses,
  the store is local. Confirm `encode_one` didn't change `apply`, and no new `just deny` exception.
- [ ] **ship** — merge PR; append verify + ship cost sessions + totals + reflection; archive to done/;
  advance STAGE-021 (single-spec stage → shipped) and update the PROJ-007 brief; STAGE-022 (lockfile)
  next — carrying the injective source→output blocker (DEC-057).
