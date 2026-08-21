---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-096
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-20
supersedes: null
superseded_by: null

# The pin lands on both AVIF encode arms (DEC-019/DEC-068 lockstep) plus the
# lockfile, since that is where the consequence this decision closes bites.
affected_scope:
  - src/sink/mod.rs
  - src/quality/mod.rs
  - src/build/lock.rs

tags:
  - avif
  - encode
  - threads
  - tiles
  - determinism
  - reproducible-build
  - quality-per-byte
---

# DEC-096: pin the AVIF encoder's tile count to 1 — measured, not assumed

## Decision

Every AVIF encode call — `sink::encode_to_bytes_with` and `quality::
encode_candidate_bytes_with`, which DEC-019/DEC-068 require to stay in
lockstep — now calls `.with_num_threads(Some(AVIF_TILE_THREADS))` with
**`AVIF_TILE_THREADS = 1`**. `ravif`'s tile-count formula
(`threads.unwrap_or_else(rayon::current_num_threads)`) reads `threads`
before ever consulting the fallback, so pinning it makes AV1 tile count a
compile-time constant instead of a value read from the OS core count
(DEC-094). This closes the two riders DEC-094 measured: AVIF output no
longer depends on the building machine, and the shipped build stops paying
a multi-tile compression penalty on an encoder that has never parallelized
(the encode stays serial — `image/rayon` is a separate, later decision,
Call 1).

**Call 2 — N was not settled at design time.** The spec's prior was "N = 1
may be strictly better — same speed, materially smaller files," explicitly
flagged as a prior and not a conclusion, because SPEC-123's design-time
prose (a mechanism asserted from reading source) was wrong at a layer below
where anyone looked. This record measures all three axes Call 2 named, on
this branch, then a fourth the spec did not ask for and the numbers turned
out to need.

## The measurement

Host: Darwin arm64, 14 cores (`sysctl -n hw.ncpu` — unchanged from DEC-094's
host). Corpus: `bench/corpus/photo_forest_cc0.jpg` (800×532),
`bench/corpus/graphic_large.png` (512×512) — DEC-094's own corpus, so the
new numbers are directly comparable to the old ones rather than a fresh
baseline. Harness: `examples/spec124_tile_count_probe.rs` (throwaway
prototype, committed for reproducibility — same convention as SPEC-120's
probe). Best-of-5 wall clock per cell unless noted; every hash below was
cross-checked against DEC-094's own table where the cell overlaps (leg A's
`n=14` row, leg E's `n=1/4/14` rows) and matched exactly, which also
confirms the probe replicates the production encode call correctly.

### 1 — Serial wall clock, 1 tile vs N tiles, on the shipped (non-`threading`) build

Nobody had measured whether tile count itself affects **serial** encode
time — DEC-094 only drove *thread-count settings*, which never reach the
encoder at all on the shipped build.

| input | quality path | n=1 | n=2 | n=4 | n=8 | n=14 |
|---|---|---|---|---|---|---|
| photo | pinned q80 | 0.514s / 98,847 B | 0.516s / 99,065 B | 0.512s / 99,303 B | 0.510s / 99,494 B | 0.519s / 100,344 B |
| photo | auto q85 | 0.520s / 123,030 B | 0.523s / 123,244 B | 0.519s / 123,641 B | 0.539s / 124,308 B | 0.531s / 125,548 B |
| graphic | pinned q80 | 0.142s / 860 B | 0.138s / 983 B | 0.141s / 1,017 B | 0.138s / 1,168 B | 0.132s / 1,272 B |
| graphic | auto q85 | 0.137s / 845 B | 0.141s / 930 B | 0.137s / 1,005 B | 0.137s / 1,185 B | 0.132s / 1,295 B |

**Wall clock is flat within noise across the whole sweep, on both content
classes and both quality paths.** No trend, no monotone cost to N=1 or to
N=14 — confirms the prior's first half: with the encode serial, requesting
fewer tiles does not cost time on content this size. (The `n=14` bytes/hash
match DEC-094's leg A/A2 exactly — `100344`/`db798cfaec702270`,
`1272`/`5ad74a803a7ce1aa`, `125548`/`1c5ed3f11c6f72e3`, `860`→ leg E's
probe-at-1 row `79673ecb15623ed5` — cross-check, not re-derivation.)

**⚠ This does NOT generalize to every content class at every size — see
§4.** A 24 MP *synthetic* worst-case pattern shows a real, repeatable
serial-time cost to N=1. A 24 MP *realistic* photo does not. Both are
reported below rather than picking the one that supports the headline.

### 2 — Compression at 1 tile, re-measured on the branch (AC-4)

Re-run rather than quoted from DEC-094, and extended with the intermediate
N values and the **auto** (q85) path DEC-094's leg A2 established is the
one users actually hit:

| input | quality path | N=1 → N=14 | Δ bytes | Δ % |
|---|---|---|---|---|
| photo | pinned q80 | 98,847 → 100,344 | +1,497 | **+1.51 %** |
| photo | auto q85 | 123,030 → 125,548 | +2,518 | **+2.05 %** |
| graphic | pinned q80 | 860 → 1,272 | +412 | **+47.9 %** |
| graphic | auto q85 | 845 → 1,295 | +450 | **+53.3 %** |

Matches DEC-094's headline (+1.5 %/+47.9 % on the pinned path) and extends it
to the auto path, which is proportionally *worse* on the graphic. **N=1 is
strictly the smallest output at every N swept, on both content classes and
both quality paths, with no exception.**

### 3 — Structural safety on an image too large for one AV1 tile

`rav1e`'s `TilingInfo::from_target_tiles` enforces `MAX_TILE_WIDTH` (4096 px)
/ `MAX_TILE_AREA` (4096×2304 px) — hard AV1 bitstream limits — by clamping
`tile_cols_log2`/`tile_rows_log2` **up** to their structural minimum
regardless of the requested count (`rav1e` 0.8.1 `tiling/tiler.rs:74-90`),
before the requested-tiles search loop (`rav1e` `encoder.rs:246-275`) ever
runs. That clamp is a pure function of frame dimensions, not thread count —
so it cannot reopen the machine-dependence this record closes, and N=1 is
never "1 tile no matter what": it is "as few tiles as the bitstream allows."
Driven directly on a synthetic 6000×4000 (24 MP, over both limits) frame:
no panic, N=1 encodes successfully at both the small corpus and this size.

### 4 — The forward cost of N=1, and the caveat that surfaced while measuring it

**4a — the `image/rayon` forward cost (Call 2's third bullet).** A probe
built with `--features image/rayon` (`ravif/threading` ON — confirmed via
the build fingerprint, `features: []` shipped vs `features: ["threading"]`
probe, the DEC-094-established discriminating check; `cargo tree` cannot see
this feature at all) shows what pinning N=1 forfeits if that separate, later
decision is ever made:

| input | n=1 | n=14 | speedup forgone |
|---|---|---|---|
| photo | 0.510s | 0.078s | **6.5×** |
| graphic | 0.141s | 0.024s | **5.9×** |

Consistent with DEC-094's leg E (5.7×/4.4×). This is a real cost, and it is
explicitly not this spec's to pay for: Call 1 keeps `image/rayon` a
separate decision, crustyimg's primary parallelism lever is file-level batch
fan-out (`rayon`, DEC-006) which this pin does not touch, and whoever makes
that later decision gets to re-measure N fresh against real parallelism
data rather than inherit a guess made before threading existed.

**4b — a caveat found, not assumed, while driving 4a's methodology further.**
Repeating §1's wall-clock sweep at 24 MP surfaced something the small corpus
could not: a **synthetic**, maximally-busy 6000×4000 gradient (`(x%256,
y%256, (x+y)%256)` per pixel — no redundancy anywhere) costs N=1 a real,
repeatable **~17% serial-time regression** (3 runs: 28.9s/29.4s/29.8s at
n=1 vs 24.2s/24.3s/24.5s at n=14) for a negligible **+0.76%** byte saving at
n=14 (471,260 → 474,842 B). Rerun on a **realistic** proxy instead — the
`photo_forest_cc0.jpg` corpus image upscaled to the same 6000×4000 via
crustyimg's own `resize --exact` — the effect vanishes: 21.85s/22.08s (n=1)
vs 21.78s/21.87s (n=14), a wash, while N=1 still saves **2.02%**
(659,917 → 673,238 B).

**Read together, not separately: the serial-time cost is a property of
content complexity, not image size.** A per-pixel-random synthetic pattern
maximizes RDO search cost per superblock; splitting it into more, smaller
tiles genuinely reduces total serial work (a real effect, not noise — three
repeats agree). A photographic upscale, however busy in absolute pixel
count, has the spatial redundancy real photos have, and the effect does not
appear. **crustyimg's own benchmark corpus and BENCHMARKS.md are real
photographs, not synthetic worst-case noise** — so this caveat is recorded
for honesty and for whoever revisits N later, not because it changes the
recommendation for the workload this tool is measured against.

## The choice: N = 1

Every axis measured points the same way for content resembling what
crustyimg actually ships against: no serial-time cost (§1, §4b's realistic
leg), the smallest output at every size and content class measured (§2),
structurally safe on arbitrarily large images (§3), and the one real cost
(§4a) is explicitly deferred to a decision this spec does not make. The one
caveat (§4b's synthetic leg) is a property of adversarial content, not of
this tool's measured workload, and is recorded rather than hidden so a
future `image/rayon` decision — or a future report of the CLI running slow
on some hostile/degenerate input — has this data rather than having to
re-derive it.

## Alternatives Considered

- **N = 1 without measuring** (act on the prior directly). Rejected on
  principle: SPEC-123's own postmortem is that confident source-reading
  reasoning was wrong at a layer below where anyone looked, and this
  spec's own text names that as the reasoning shape to avoid.
- **N = core count (status quo, unpinned).** Rejected — this is the exact
  defect DEC-094 measured: machine-dependent output, a false-positive drift
  path in the lockfile (STAGE-042), and the full multi-tile compression
  penalty for zero parallelism.
- **A fixed N > 1 (e.g. 4 or 8), splitting the difference.** Considered as a
  hedge against §4a's forward cost. Rejected: it is strictly worse than N=1
  on every measured axis *today* (more bytes, no speed benefit, still
  serial), and buys headroom for a decision (`image/rayon`) that is not
  made, not scheduled, and would need its own fresh N measurement against
  real parallelism data regardless of what this spec picks — so hedging now
  trades a certain, measured cost for an uncertain, unscheduled benefit.

## Consequences

- **Positive — AVIF output stops depending on the machine.** A 4-core
  laptop and a 32-core CI box now write identical bytes for the same input,
  closing the specific mechanism behind STAGE-042's `[env]`-cannot-express-
  "same machine" item (Call 5 — see that item, now closed, on STAGE-042).
- **Positive — smaller AVIF output, worst on the content quality-per-byte
  matters most for.** Photos: -1.5% to -2.1%. Small/graphic content: -32%
  to -35% (N=14→1 direction; §2's table states it the other way). On a tool
  whose thesis is quality-per-byte, this is the shipped build moving toward
  that thesis rather than away from it, at no quality cost (same quantizer,
  same speed — only the tile partitioning changes).
- **Negative — forecloses free encode-time parallelism if `image/rayon` is
  ever enabled** (§4a, 5.9×–6.5× on this corpus). Explicitly accepted:
  that decision does not exist yet, gets its own measurement when it does,
  and crustyimg's batch parallelism (DEC-006) is unaffected.
- **Neutral, flagged — a content-dependent caveat on very large synthetic
  input** (§4b). Not a correctness issue (rav1e's own tiling clamp keeps
  N=1 bitstream-legal regardless — §3) and not reproduced on realistic
  content at the same size. Revisit if a real user workload — not a
  synthetic probe — is ever reported slow on a large AVIF encode.
- **Migration.** This changes every AVIF output byte, batched with
  SPEC-121/SPEC-122 into one lockfile migration (STAGE-046's wave, Call 4).
  AC-6 drove the mechanism directly rather than reasoning about it (see
  Build Completion): at the unbumped version, `build --check` wrongly
  reports "up to date" and a plain `build` silently serves stale pre-fix
  bytes from cache — the same-version blind spot STAGE-042 already has
  filed, confirmed live for this specific diff, not newly introduced by it.
  At a version bump (driven with a temporary, uncommitted `0.7.1`,
  discarded afterward — Cargo.toml is unchanged in this diff), `--check`
  correctly reports drift (exit 7), `--frozen` correctly fails (exit 7,
  lockfile untouched), and a plain `build` regenerates the correct new
  bytes with no stale hit.

## Validation

- `tests/avif_tile_pin.rs::both_encode_paths_set_the_thread_count` (AC-1) —
  drives the DEC-019/DEC-068 lockstep at the public API boundary on a
  512×512 graphic fixture (matched to DEC-094's own `graphic_large.png`, the
  size term at which N=1 vs ambient-N never coincidentally converge).
  Negative control run three ways during build (not committed — the
  behavioural flip, not a hash): pin removed from `sink` only → RED; pin
  removed from `quality` only → RED; pin removed from BOTH (full revert) →
  **GREEN** — this test's blind spot by design, since a symmetric absence is
  still lockstep. That gap is exactly why AC-2's test exists as a separate,
  independent check.
- `tests/avif_tile_pin.rs::avif_output_is_identical_across_ambient_core_counts`
  (AC-2/AC-5) — builds a `--features image/rayon` probe once (memoized) and
  sweeps `RAYON_NUM_THREADS` ∈ {1,2,4,8,14} as this repo's available proxy
  for "a differently-cored machine" (DEC-094 itself could not drive a real
  second host — its own "Not measured here"). Negative control: full revert
  → RED, with a five-way hash spread (8 and 14 tie, matching DEC-094's
  leg-F quantization finding); asymmetric reverts (previous bullet) → RED
  (drives through `sink`, so it inherits that defect too); fix restored →
  GREEN.
- AC-6 (migration) driven manually per the Build Completion — not a
  committed test, matching SPEC-121's AC-8 precedent (a real target,
  temporary uncommitted version bump, discarded after).
- Full matrix (`default`, `--no-default-features`, `--features webp-lossy`),
  each in a fresh `CARGO_TARGET_DIR`, sequential: all pass; `cargo clippy
  --all-targets -- -D warnings` and `cargo fmt --check` clean on every leg.
- `scripts/decisions-audit.sh --changed main` — see Build Completion for
  the run and its result.

**Revisit if:** `image/rayon` is scheduled (re-measure N against real
parallelism data, not this record's numbers — they predate threading being
real); a user reports a large real-world AVIF encode as slow (check whether
§4b's caveat is reproducing on real content, not synthetic); `rav1e`
reworks its tiling-clamp formula (§3's safety argument would need
re-deriving); or the bench corpus gains a large real photo (re-run §1/§4b
against it directly instead of an upscale proxy).

## References

- Related specs: **SPEC-124** (this spec), **SPEC-123** (DEC-094, the
  measurement this pin acts on), SPEC-121/SPEC-122 (the wave this rides,
  Call 4).
- Related decisions: **DEC-094** (AVIF thread settings never reach the
  encoder — the core count does; this record's entire empirical basis),
  DEC-019/DEC-068 (the sink/quality AVIF encode lockstep this pin must
  respect), DEC-058 (the cache key whose version component is what makes
  AC-6's migration story sound only at an actual bump), DEC-059 (the
  lockfile), DEC-006 (file-level batch parallelism, unaffected by this).
- External: `ravif` 0.13.0 `src/av1encoder.rs:78-101` (`with_num_threads`),
  `:651-655` (`tiles`); `image` 0.25.10
  `src/codecs/avif/encoder.rs:66-91`/`:90` (`with_num_threads` passthrough);
  `rav1e` 0.8.1 `src/tiling/tiler.rs:21-90` (`MAX_TILE_WIDTH`,
  `MAX_TILE_AREA`, the clamp), `src/encoder.rs:238-275` (the target-tiles
  search loop).
