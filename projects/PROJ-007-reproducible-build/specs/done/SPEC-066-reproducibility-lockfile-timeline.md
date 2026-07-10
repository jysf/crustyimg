# SPEC-066 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — the reproducibility lockfile + `build --check`/`--frozen` (STAGE-022's headline; the
  "verifiable" payoff). `src/build/lock.rs`: `crustyimg.build.lock` (versioned TOML, `deny_unknown_fields`,
  size-guarded) with one `[[output]]` per output {path, source, recipe, **key** (pinned DEC-058 cache key
  hex), **hash** (observed output-bytes hex), bytes} + one `[env]` {crustyimg_version, target=`{ARCH}-{OS}`,
  features}, plus an **env-aware `diff`**. Wire `run_build`: default WRITES the lock; `--check`/`--frozen`/
  `--locked` VERIFY (exit 7 on drift, never write); `--strict` promotes cross-env output-hash variance to a
  failure. **Policy (the DEC-059 core): pin the robust (cache key = inputs → cross-machine reproducible),
  record the fragile (output hash + env).** `--check` fails on input drift (key change / added-removed
  output) and on same-env output regression; cross-env byte variance is informational unless `--strict`;
  perceptual (SSIMULACRA2) stays the composable `diff`, not baked in. Failing Tests: parse/roundtrip/reject
  (unit) + diff identical/added-removed/key-change/hash-same-env/hash-cross-env-strict (unit) +
  writes-lock/deterministic/check-pass/check-fail-edit/check-fail-outputs/frozen-no-lock/frozen-matching-no-write/
  cross-env-tolerated-unless-strict/malformed-exit-2 (integration). **No new dep** (serde/toml/sha2 shipped;
  toml serialization already used by DEC-058 recipe round-trip). Reuses `CacheKey::to_hex`/`hash_bytes`,
  exit 7 (DEC-025), SPEC-065 injectivity (path = valid key). **DEC-059 at build.** Framing, 2026-07-09.
- [x] **build** — `src/build/lock.rs` (BuildLock/LockEnv/LockOutput/LockError + `to_toml`/`from_toml` +
  env-aware `diff`) + wire `run_build` (build_one returns a per-output LockRecord; compute the key even
  under `--no-cache`; write default / verify on `--check`/`--frozen`/`--locked`/`--strict`); `pub mod lock`;
  `--check`/`--frozen`/`--locked`/`--strict` on GlobalArgs; DEC-059. Make all Failing Tests pass. Verify
  default + lean + `just deny` (no new dep) + clippy + fmt.
  → **PR #73** on `feat/spec-066-reproducibility-lockfile`, 2026-07-09. 666 tests green (default + lean;
  11 new in `tests/build_lock.rs`), clippy ×2 + fmt clean, `just deny` unchanged, **no new dep**
  (`git diff main -- Cargo.toml` empty). DEC-059 emitted. Every branch driven on the real binary
  (deterministic lock; `--check` pass / exit-7 on edited source, added, removed; `--frozen` w/o lock → 7;
  cross-env note vs `--strict` → 7; malformed → 2 pre-write). Folded in per the STAGE-022 notes:
  `exit_code_mapping_is_total` now covers `CliError::Cache`, and the literal-`{ext}` residual is closed as
  a *silent* failure (post-encode `find_output_collision` → exit 2, no lock; the race itself still needs a
  pre-decode format sniff — see DEC-059 + follow-ups). Est. ~350k tok / ~$3.15 (main-loop estimate).
- [x] **verify** — fresh session. Re-run gates; reproduce on the real binary: a build writes a deterministic
  lock; `--check` passes on a matching tree and exits 7 on an edited source / added-removed output without
  writing; `--frozen` without a lock exits 7; the cross-env hash tolerance (unit) + `--strict` escalation.
  Confirm no new dep, exit-7 mapping, lock never auto-regenerated under `--check`.
  → **⚠ PUNCH LIST**, 2026-07-09. All 8 acceptance criteria reproduced on the real binary; all gates green
  from clean (666 default + 666 lean, 26 suites; 11 integration + 15 unit lock tests present and running;
  clippy ×2 + fmt clean; `just deny` green; `git diff main -- Cargo.toml Cargo.lock` empty). The load-bearing
  deviation holds: the generated lock carries only relative, `/`-separated paths (no `/Users/…`, no `\`).
  Message prose correct — `--check --strict` prints "failing under --strict", never "use --strict to fail on
  it"; drift lines name the path + what drifted. `--check` never writes/regenerates the lock (mtime+bytes
  unchanged on match and on all three drift cases); `--frozen`/`--locked` alias correctly. Literal-`{ext}`
  residual behaves exactly as DEC-059 discloses (exit 2, no lock, output already raced to disk) — accepted as
  carried, not punch-listed. All six deviations sound and disclosed.
  **One defect (blocks ship, not the design):** `lock::short()` (`src/build/lock.rs:367-370`) byte-slices a
  `key`/`hash` read out of the committed lockfile, so a lockfile whose digest contains a multi-byte char
  panics `--check` at exit 101 instead of returning a typed code — reproduced on both the `key` (KeyChanged)
  and `hash` (HashChanged*) render paths. Fails closed, but violates `untrusted-input-hardening` and the
  module's own "compared as opaque strings" contract. Fix: `hex.get(..n).unwrap_or(hex)` (or validate hex at
  parse), plus a regression test.
- [x] **ship** — **punch-list resolved on the branch** (`c6dc827`): `from_toml` now rejects a non-hex
  `key`/`hash` → exit 2 + `short()` uses `get(..n).unwrap_or(hex)`, with unit (`rejects_non_hex_digest`,
  `short_never_panics_on_multibyte`) + integration (`non_hex_digest_in_lock_is_exit_2_not_a_panic`, drives the
  binary → exit 2, asserts no panic) regression tests. Re-confirmed 26 default + lean suites, clippy ×2, fmt,
  `just deny`, no new dep; **3-OS CI 26/26 green** (DCO fixed after a first miss). Squash-merged PR #73 → main
  (**ce2fc69**); re-applied verify + ship cost sessions + `cost.totals` (470k tok / ~$4.23, 4 sessions —
  build+verify are labelled main-loop estimates §4) + ship reflection on main (stash-pop, §13); archived
  spec+timeline to `done/`; `just cost-audit` green. **STAGE-022 SHIPPED** (SPEC-065 + SPEC-066) → PROJ-007's
  **"verifiable" leg complete**; only STAGE-023 (`--watch`) remains. Brief + stage-ship reflection updated.
  2026-07-09.
