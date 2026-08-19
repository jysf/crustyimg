---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-094
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-17
supersedes: null
superseded_by: null

# The AVIF encode call is the site this governs: `sink::encode_to_bytes*`
# constructs the encoder without `with_num_threads`, which is what leaves the
# tile count to the OS. `quality::encode_candidate_bytes_with` must stay in step
# with it (DEC-019/DEC-068), so it is in scope too.
affected_scope:
  - src/sink/**
  - src/quality/mod.rs
  # Added at verify: the lockfile is where this record's consequence bites.
  # `[env]` cannot express "same machine" (see the Consequences section), so
  # anyone touching the lock contract needs this decision surfaced.
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

# DEC-094: AVIF thread *settings* never reach the encoder — the machine's core count does

## Decision

Record the measured verdict, which is **Call 3's third branch** of SPEC-123:
**the encoder ignores the thread setting.** Neither `RAYON_NUM_THREADS` nor
`--jobs` changes a byte of crustyimg's AVIF output, because `ravif` is compiled
**without its `threading` feature** — so it never consults a rayon pool, encodes
**serially**, and takes its tile count from `std::thread::available_parallelism()`.

Two riders, both measured, and both more consequential than the null:

1. **The knob the encoder *does* read — the machine's core count — changes the
   bytes.** Same input, same version, same features, same OS: 1 tile vs 14 tiles
   produced different SHA-256s on both corpus inputs. crustyimg's AVIF output is
   therefore **not** portable across differently-cored machines, and core count
   is **not** in the `[env]` block the lockfile records (`crustyimg_version`,
   `target`, `features`).
2. **crustyimg is on the wrong end of that trade.** The tile count is the machine's
   core count, so the shipped binary pays the **full multi-tile compression penalty
   and collects none of the parallelism**: measured **+1.5 %** bytes on the photo and
   **+47.9 %** on the graphic versus a 1-tile encode, at **5.7× / 4.4× the wall clock**
   of the same tile count encoded in parallel.

This decision records the measurement and its scope. It does **not** change
behaviour — SPEC-123 ships none. The fix is a follow-up (see Consequences).

## Context

`docs/backlog.md` said "**Measure before claiming either way**", and three shipped
things assume AVIF output is byte-stable on one machine: `build --frozen`, the
lockfile's `hash` (`src/build/lock.rs:32-37`), and the DEC-058 cache key
(`src/cli/build.rs:294-301`). Thread count is in none of their qualifying lists.

### The mechanism, read out of source and then confirmed behaviourally

`src/sink/mod.rs:679` builds `AvifEncoder::new_with_speed_quality(&mut cursor, s, q)`
and never calls `with_num_threads`, so `ravif` 0.13.0 resolves the count itself
(`av1encoder.rs:651-655`):

```rust
let tiles = {
    let threads = p.threads.unwrap_or_else(rayon::current_num_threads);
    threads.min((p.width * p.height) / (p.speed.min_tile_size as usize).pow(2))
};
```

⚠ **Which `rayon` that is depends on a Cargo feature, and ours is off.** `ravif`'s
`threading` feature is reachable only through `image`'s `rayon` feature
(`image` 0.25.10 Cargo.toml: `rayon = ["dep:rayon", "ravif?/threading", "exr?/rayon"]`),
and crustyimg declares `avif = ["image/avif"]` and nothing else (`Cargo.toml`,
`[features]`). With `threading` off, `ravif` swaps in its own shim (`lib.rs:33`):

```rust
mod rayoff {
    pub fn current_num_threads() -> usize {
        std::thread::available_parallelism().map(|v| v.get()).unwrap_or(1)
    }
    pub fn join<A, B>(a: impl FnOnce() -> A, b: impl FnOnce() -> B) -> (A, B) {
        (a(), b())          // sequential
    }
}
```

So `current_num_threads` is **not rayon's pool size** — it is the OS core count,
which no environment variable and no CLI flag moves. And `join` is sequential, so
the encode is single-threaded no matter what `tiles` says.

**Mechanical check — ⚠ REPLACED at verify (2026-08-17). The original was
non-discriminating.** It read `cargo tree -e features` for `feature "threading"`
→ 0 lines, `image feature "rayon"` → 0, `ravif` → 1 bare node, with `image
feature "jpeg"` → 1 as a positive control.

**Verify ran the negative control it lacked: on the PROBE tree, where threading is
provably ON, all three sub-checks return identical values**, and the string
`threading` appears in neither tree. The `jpeg` control proves only that the
pattern *form* can match. `cargo tree` cannot see this feature at all, so it was
never evidence [[a-deps-documented-default-is-a-claim-about-a-feature-set]].

**The discriminating check is the build fingerprint**, and it is unambiguous:

```
shipped  ravif → features: []
probe    ravif → features: ["threading"]
```

That is what establishes the shipped encode is serial, and separately what proves
leg E's probe genuinely enabled the variable rather than merely being labelled so.

Because `tiles` is a **bitstream-level** partitioning — tile boundaries reset
entropy-coding contexts — a different tile count is a different file by
construction, before `rav1e`'s nondeterminism bug (#2781) enters the picture.

## The measurement

Harness: `scripts/spec123_avif_thread_determinism.py`, committed, re-runnable.
Host: Darwin arm64, 14 cores, crustyimg 0.7.0 release, default features.
Corpus: `bench/corpus/photo_forest_cc0.jpg` (800×532),
`bench/corpus/graphic_large.png` (512×512).

**Which quality, measured rather than assumed.** Pinning `--format avif` drops all
three verbs to the sink default **80** (`sink::AVIF_DEFAULT_QUALITY`): `web --format
avif` and `optimize --format avif` are byte-identical to `convert -q 80`
(100344 B, `db798cfaec702270`) and not to `convert -q 85` (125548 B,
`1c5ed3f11c6f72e3`). Their **auto** path — no pin, the surface users actually hit —
encodes at **85** (`sink::FAST_LOSSY_QUALITY`): `web`/`optimize --json` both report
`quality: 85`, and the output is byte-identical to `convert -q 85`.

⚠ **So the pinned matrix is three verbs making one identical encoder call** — a
weaker triple than AC-3's wording implies. Leg A2 was added to drive the auto path
as well, so the answer does not rest on one encode wearing three hats.

At speed 6, both 80 and 85 land under ravif's `high_quality` threshold
(`quality_to_quantizer(80.) = 121`; q80 → 121, *not* > 121; q85 → 99), so
`min_tile_size` is **128** either way and the size term is **25** (photo) and
**16** (graphic). Both exceed 14, so **the thread term binds across the whole range
and no row in the main matrix is clamped.**

### Leg A — shipped binary, `RAYON_NUM_THREADS` ∈ {1, 4, 14}

Hashes are the first 16 hex digits of the SHA-256 of the output bytes.

| verb | input | threads | sha256 | bytes | wall s | cpu/wall |
|---|---|---|---|---|---|---|
| convert | photo | 1 / 4 / 14 | `db798cfaec702270` ×3 | 100344 | 0.530 / 0.529 / 0.529 | 1.00 |
| convert | graphic | 1 / 4 / 14 | `5ad74a803a7ce1aa` ×3 | 1272 | 0.140 / 0.139 / 0.140 | 0.99 |
| web | photo | 1 / 4 / 14 | `db798cfaec702270` ×3 | 100344 | 0.534 / 0.535 / 0.534 | 1.00 |
| web | graphic | 1 / 4 / 14 | `5ad74a803a7ce1aa` ×3 | 1272 | 0.144 / 0.143 / 0.144 | 0.99 |
| optimize | photo | 1 / 4 / 14 | `db798cfaec702270` ×3 | 100344 | 0.530 / 0.529 / 0.528 | 1.00 |
| optimize | graphic | 1 / 4 / 14 | `5ad74a803a7ce1aa` ×3 | 1272 | 0.140 / 0.141 / 0.140 | 0.99 |

**18/18 cells invariant. `cpu/wall` never leaves 0.99–1.00 — one core's worth of
CPU for the whole wall clock, on every leg. The encode is serial and the lever
did not move the work.**

### Leg A2 — the AUTO-decision path, and an in-process control

`--format avif` is a pin, and the pinned path is not the one users hit. Driving
`web` and `optimize` with **no pin** (they pick AVIF themselves, at quality 85):

| verb | input | threads | sha256 | bytes | wall s | cpu/wall |
|---|---|---|---|---|---|---|
| web (auto) | photo | 1 | `1c5ed3f11c6f72e3` | 125548 | 0.630 | 1.00 |
| web (auto) | photo | 4 | `1c5ed3f11c6f72e3` | 125548 | 0.628 | 1.01 |
| web (auto) | photo | 14 | `1c5ed3f11c6f72e3` | 125548 | 0.632 | **1.17** |
| optimize (auto) | photo | 1 / 4 / 14 | `1c5ed3f11c6f72e3` ×3 | 125548 | 0.550 / 0.547 / 0.547 | 1.00 |

Invariant, and byte-identical to `convert -q 85` — so the auto path is the same
encoder call at a different quality, and it is equally blind to the setting.

⚠ **But look at `web (auto)` at 14 threads: `cpu/wall` 1.17, against 1.00 at one
thread.** Monotone in the thread count and reproduced on **three** independent
runs (1.16, 1.17, 1.17). That is `web`'s own rayon work — the full-resolution
content analysis its `--help` calls out — actually consuming the extra threads.

**This is the control that closes the last hole in the null.** A sceptic can say
"the hashes did not move because `RAYON_NUM_THREADS` never took effect in that
process at all." Here it demonstrably did: the same env var, the same process, the
same run, moved ~17 % more CPU through the analysis stage **while the output bytes
stayed fixed**. The variable reached the program. It did not reach the encoder.

### Leg B — `optimize --jobs N`, batch size 1

`--jobs` builds a scoped pool and `install`s the batch (`src/cli/optimize.rs:177-181`);
batch size 1 isolates the encoder's pool from file fan-out. ⚠ `--jobs` is
**silently ignored** by `convert` and five other serial verbs (STAGE-042), so it
is only meaningful here.

| jobs | sha256 | bytes | wall s | cpu/wall |
|---|---|---|---|---|
| 1 | `db798cfaec702270` | 100344 | 0.528 | 1.00 |
| 4 | `db798cfaec702270` | 100344 | 0.528 | 1.00 |
| 14 | `db798cfaec702270` | 100344 | 0.528 | 1.00 |

Invariant, and identical to leg A's bytes. The scoped pool reaches the batch and
not the encoder — consistent with `ravif` never asking rayon anything.

### Leg C — run-to-run at a fixed thread count (SPEC-123 Call 4)

10 repeats per verb at `RAYON_NUM_THREADS=14`, photo input:

| verb | repeats | distinct hashes | stable |
|---|---|---|---|
| convert | 10 | 1 | yes |
| web | 10 | 1 | yes |
| optimize | 10 | 1 | yes |

**Stable.** This is the claim the lockfile actually makes, and it holds — which
also means a `with_num_threads(Some(N))` pin would be *sufficient*, not merely
narrowing: there is no residual run-to-run nondeterminism underneath the tiling.

### Leg D — the lean build (`--no-default-features`)

`convert --format avif` exits **4** with `error: avif support is not built;
rebuild with --features avif`, and writes no file. The AVIF encoder is not
compiled in (DEC-004/DEC-020), so this leg cannot produce the artifact at all —
that is the correct behaviour, not a gap in the matrix.

### Leg E — the positive control, and why the null is earned

A probe binary built with `--features image/rayon` (i.e. `ravif/threading` **on**;
isolated `CARGO_TARGET_DIR`, never a shared one) run over the same matrix:

| input | threads | sha256 | bytes | wall s | cpu/wall |
|---|---|---|---|---|---|
| photo | 1 | `8440985ac135b877` | 98847 | 0.528 | 1.00 |
| photo | 4 | `f5a10e84cf2a522f` | 99303 | 0.184 | 2.91 |
| photo | 14 | `db798cfaec702270` | 100344 | 0.093 | 7.09 |
| graphic | 1 | `79673ecb15623ed5` | 860 | 0.154 | 0.99 |
| graphic | 4 | `0006a371c3cad433` | 1017 | 0.053 | 2.90 |
| graphic | 14 | `5ad74a803a7ce1aa` | 1272 | 0.032 | 5.21 |

**The bytes move, the clock moves, and the CPU/wall ratio moves.** So the harness,
the corpus, the verbs and the lever are all capable of registering a thread-count
change; leg A's null is a property of the shipped build, not of the measurement
[[a-control-you-never-verified-applied-is-not-a-control]].

### Leg F — the cross-check that BOUNDS the shipped tile count

| input | shipped | probe @ 14 | |
|---|---|---|---|
| photo | `db798cfaec702270` | `db798cfaec702270` | **identical** |
| graphic | `5ad74a803a7ce1aa` | `5ad74a803a7ce1aa` | **identical** |

⚠ **Corrected at verify (2026-08-17) — the claim that stood here was false.** It
read *"the shipped bytes land exactly on the probe's 14-tile point and nowhere
else … a tile count of 16 was equally available and did not match."* Verify swept
N ∈ {1, 2, 4, 8, 12, 13, 14, 15, 16, 20, 25, 26, 30} and measured:

- **graphic** matches the shipped bytes at **every N ≥ 12** — including 16, the
  count the original text says did not match. Byte-identical over 3 repeats;
  negative control at N = 8 differs.
- **photo** matches at **N = 13–20 only** (not 12, not ≥ 25).

`rav1e` quantizes a requested tile count to a legal tile grid, so a range of
requests collapses onto one layout. **Leg F therefore BOUNDS the shipped tile
count to a band — intersection ≈ 13–20 — and is not a positive identification of
14.** The verdict does not rest on it: it rests on the build fingerprint, the
`rayoff` shim, and `cpu/wall ≈ 0.99`.

**⚡ Rider — core-count sensitivity is QUANTIZED, and it changes the filed fix.**
A 14-core and a 16-core host emit identical AVIF bytes here; an 8-core and a
14-core host do not. So recording the **raw core count** in the lockfile's `[env]`
would be the wrong key — it would churn `[env]` between machines whose output
actually agrees. Whatever lands must key on the resulting tile grid, or on nothing.

Two further things fall out of the match at 14: the byte differences in leg E are
attributable to the **tile count** and not to threading nondeterminism (same
tiles ⇒ same bytes, serial or parallel), and `rav1e`'s #2781 did not fire in any
run here.

### Leg G — the clamp, demonstrated rather than asserted

`min_tile_size` doubles to 256 when the quantizer clears `ravif`'s `high_quality`
gate (`av1encoder.rs:544/584`) — which, following the maths rather than the
identifier, means **quality below 80**. At `convert -q 50` the graphic's size term
drops from 16 to **4**, on the probe build:

| threads | tiles | sha256 | bytes | wall s |
|---|---|---|---|---|
| 1 | 1 | `cd9ceefe56119779` | 819 | 0.097 |
| 4 | 4 | `ef950c25db145e7d` | 1002 | 0.032 |
| 14 | **4 (clamped)** | `ef950c25db145e7d` | 1002 | 0.032 |

Two legs byte-identical *while the encoder is fully thread-sensitive*. A hash
table without its clamp column reads that as "deterministic above 4 threads".
It is the `.min(..)`.

### Reproducibility (AC-8)

The harness was run twice, end to end, on an otherwise idle machine. All **43**
hash occurrences across the two runs form an identical multiset (**9** distinct
values), compared mechanically rather than by eye. Every table above is the first
of those two runs; the wall-clock figures are from that run and are the only
numbers here that are not bit-exact between them.

## Alternatives Considered

- **Option A — report "deterministic across thread counts".**
  Rejected: it is the answer most likely to be right for the wrong reason. The
  hashes are identical *because the setting never arrives*, and SPEC-123's Call 3
  reserves a separate branch for exactly this. Calling it "deterministic" would
  license threading work that the moment it lands makes the output vary.

- **Option B — report "non-deterministic" on the strength of the core-count result.**
  Rejected as an answer to the question asked. The question is thread *count as a
  setting*; the setting provably does nothing. The core-count dependence is real
  and is recorded here as a rider, but folding it into the headline would overstate
  what the thread axis showed.

- **Option C (chosen) — the third branch, with the core-count finding attached.**
  It is what the table says: the settings are invisible to the encoder today, and
  become live the moment anyone enables `image/rayon` or calls `with_num_threads`.

## Consequences

- **Positive — the narrow claim the lockfile makes survives.** Byte-identical
  run-to-run within a machine held over 30 runs (leg C), and no thread setting a
  user can reach perturbs it. `build --frozen`, the `hash` field and the DEC-058
  cache key are safe **on one machine** exactly as written.

- **Negative — the `[env]` caveat list is incomplete, and `diff` inherits the gap.**
  `src/build/lock.rs:32-37` qualifies the output hash with arch/OS/codec version,
  and `[env]` records `crustyimg_version` + `target` + `features`. **Core count is in
  neither.** Two machines with the same `target` and `features` but different core
  counts write different AVIF bytes, and `diff` treats an output-hash change under
  the **same** `env.target` as *a real regression* — so this is a live false-positive
  path, not a theoretical one. **Correcting that prose is a follow-up, not this spec:**
  SPEC-123's AC-7 forbids a `src/` edit, so the finding is filed rather than smuggled in.

- **Negative — crustyimg pays for tiling and gets nothing back.** Serial encode at
  core-count tiles is the worst cell of the matrix. Against a 1-tile encode of the
  same input: **+1,497 B (+1.5 %)** on the photo and **+412 B (+47.9 %)** on the
  graphic. Against the same 14 tiles encoded in parallel: **0.530 s → 0.093 s (5.7×)** on the photo and **0.140 s → 0.032 s (4.4×)** on the graphic.
  The proportional cost is far larger on small/graphic content, where 14 tiles over
  a 512×512 image is close to `ravif`'s own "inefficiently tiny tiles" caveat.

- **Corrects STAGE-042's premise.** The stage note said "Today the encoder already
  takes every core via the default". Measured: it does not — `cpu/wall ≈ 0.99` on
  every shipped leg. Whoever scopes the `with_num_threads` pin should scope it
  against these numbers, and note that **enabling `image/rayon` is the performance
  lever; the pin is the determinism lever**, and they are separable.

- **Neutral — this is the encode path, not DEC-077's.** DEC-077 pins AVIF *decode*
  to one thread in `src/image/avif.rs` for a `re_rav1d` data race, and establishes
  that decode output is thread-independent (dav1d is conformant). None of that
  transfers: encode output is thread-count-dependent by construction, the crate is
  different, and the reason is tiling rather than a race. Do not conflate them.

- **Neutral — the follow-ups this unblocks.** Encoder threading and `par_iter
  run_pixel_op` (SPEC-091's follow-up) were both gated on this answer. They are
  unblocked *with a constraint*: any change to the ambient pool size on a path that
  encodes AVIF will change the output bytes, unless `with_num_threads(Some(N))` is
  pinned first.

## Validation

- Two independent full runs of `scripts/spec123_avif_thread_determinism.py`
  reproduce all **43** hashes (AC-8). ⚠ *Corrected at verify: this said 37. The
  spec's AC ledger said 43 and was right — 43 table occurrences over 9 distinct
  values (39 JSON `sha256` rows + 4 leg-F restatements), multisets identical across
  two runs.* Verify rebuilt all three binaries from scratch and reproduced every
  hash in every leg bit-for-bit, including leg E's probe hashes, leg F's identity
  and leg G's clamp.
- The null is earned by leg E (the probe moves the bytes) and by leg F **bounding**
  the shipped tile count to a band that excludes the low counts, not by the absence
  of a difference. *Corrected at verify — see leg F; the original wording claimed a
  point identification the data does not support.*
- The serial-encode claim rests on `cpu/wall ≈ 0.99` measured per run **and on the
  build fingerprint** (`features: []` shipped vs `["threading"]` probe). ⚠ *This
  bullet previously counted the `cargo tree` feature check as the second
  independent kind of evidence. It is not evidence at all — verify's negative
  control showed it returns identical values on a tree where threading is ON.*
- `scripts/decisions-audit.sh` flags this record as overlapping **12** existing
  decisions on `src/sink/**` / `src/quality/**` (DEC-003, -004, -006, -007, -016,
  -019, -021, -022, -023, -027, -035, -044). Checked: none is contradicted. They
  govern *what* the encoder is and at what quality/speed it is called; this one
  records *what the thread count does to the bytes* and asserts no new constraint
  on any of them. Nothing is superseded.

**Revisit if:** anyone enables `image/rayon` or calls `with_num_threads` (the
verdict flips to "non-deterministic across thread counts" the same day); `ravif`
changes the `rayoff` shim or its tile formula; `image` reworks how `avif` reaches
`ravif`'s features; or a machine with a different core count is added to the
release matrix (a differently-cored host is the one leg this measurement could not
run — see below).

**Not measured here:** a *second machine* with a different core count. The
core-count dependence is established by driving `tiles` directly on one host, plus
the source path that feeds `available_parallelism()` into the same slot — not by
observing two hosts disagree. On this host `hw.ncpu` and `hw.physicalcpu` are both
14, so it also cannot distinguish which of the two readings
`available_parallelism()` returns.

## References

- Related specs: **SPEC-123** (this measurement), SPEC-091 (`par_iter
  run_pixel_op`, unblocked with a constraint), SPEC-120 (the harness shape),
  SPEC-066 (the lockfile).
- Related decisions: **DEC-077** (AVIF *decode* single-thread policy — different
  code path, different crate, different reason), **DEC-058** (the cache key whose
  component list omits thread/core count), DEC-059 (lockfile), DEC-019/DEC-068
  (the sink/quality AVIF encode contract that keeps `src/quality/mod.rs` in step),
  DEC-020/DEC-004 (AVIF output is feature-gated → the lean build's exit 4),
  DEC-081 (AVIF encode in the default feature set).
- External: `ravif` 0.13.0 `src/av1encoder.rs:513` (`quality_to_quantizer`),
  `:544`/`:584` (`high_quality`, `min_tile_size`), `:651-655` (`tiles`), `:690`;
  `src/lib.rs:33` (the `rayoff` shim); `image` 0.25.10
  `src/codecs/avif/encoder.rs:66-91` and its `rayon` feature; rav1e #2781.
