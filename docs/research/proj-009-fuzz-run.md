# PROJ-009 decoder fuzz run — SPEC-069 run record

The re-runnable record of the decoder fuzz gate: toolchain, per-target budget
achieved, seed corpus, findings (input → root cause → disposition), and the exact
repeat recipe. A future run **appends** a dated section rather than overwriting.

Gate policy (run mechanism, triage rules, durability, CI, HEIC scope) lives in
**DEC-062**; the durable per-PR half is `tests/fuzz_regressions.rs` (regressions +
`fuzz_corpus_never_panics` smoke), run by ordinary `cargo test` on all 3 CI OSes.

---

## Run 1 — 2026-07-10/11 (SPEC-069 build cycle)

### Toolchain & host

| | |
|---|---|
| Host | macOS (Darwin 25.5), Apple Silicon (aarch64), 32 GB RAM |
| Rust | `rustc 1.99.0-nightly (375b1431b 2026-07-10)` via **rustup** |
| cargo-fuzz | `0.13.2` |
| Sanitizer | AddressSanitizer (cargo-fuzz default), aarch64-apple-darwin |
| libheif | `1.23.1` (Homebrew, incl. libde265 backend) |

**Setup note (deviation from the spec's assumed env).** The dev host had a
**Homebrew** Rust (stable `1.94.1`), *not* rustup — so `cargo +nightly` did not
exist. Installed rustup via the official installer with `--no-modify-path`
(leaves the user's shell PATH and Homebrew Rust untouched; rustup's nightly lives
under `~/.rustup`, its shims under `~/.cargo/bin`). Then `cargo install
cargo-fuzz`. The repo's normal `cargo`/`just` still resolve to Homebrew stable;
only the fuzz commands use the rustup nightly shim explicitly.

### Budget floor & configs

- **Budget floor:** `-max_total_time=600` (≈10 min wall-clock) per target, the
  DEC-062 floor. (`-runs=1000000` is the alternative floor; wall-clock was chosen
  for predictability and is what `just fuzz` uses.)
- **Two build configs matter** (this is the key finding of the run):
  - **default** = cargo-fuzz's default: `--release` **plus debug-assertions +
    overflow checks on**, ASAN on. Strictest; catches integer overflow in *our*
    code — but also fires upstream `debug_assert!`s that are **compiled out in a
    real release build**.
  - **`-O`** = `--release` with **debug-assertions off**, ASAN on. This matches
    the **shipped binary** (what real users run). `just fuzz` uses `-O`.

### Seed corpus

One valid sample per format under `tests/fixtures/{avif,svg,raw,heic}` (committed
verbatim), plus the minimized crash reproducers under `tests/fixtures/fuzz/`.
libFuzzer's RW corpus is the gitignored `fuzz/corpus/<target>/`, seeded from
those fixtures by `just fuzz`. Coverage grew well from the single seeds (no
seed-starvation observed — see per-target `cov:`), so the corpus was not grown.

---

### `avif_decode` — 3 findings, all upstream `avif-parse` 2.1.0; 2 fixed at our boundary, 1 documented

AVIF is decoded via `avif-parse` (container) + `re_rav1d` (AV1), both third-party
over fully untrusted bytes. The gate surfaced three issues, **all in `avif-parse`
2.1.0's container parser** — none in `re_rav1d` or our YUV→RGB glue.

**Budget achieved.**
- **default config:** *cannot* complete a clean budget — `avif-parse` trips its
  own `check_parser_state` `debug_assert!` on malformed input within a few hundred
  execs, and libFuzzer's panic hook **aborts on any panic regardless of a
  downstream `catch_unwind`** (verified empirically: our guard is on the abort
  stack yet the process still aborts). These asserts are **compiled out in
  release**, so this config is not representative of the shipped binary. Recorded
  as a documented upstream limitation, not a crustyimg crash.
- **`-O` config (production-representative):** ran to ~5.5k execs
  (`cov: 6988 ft: 12058 corp: 264`), exercising the container parse **and** the
  `re_rav1d` decode / YUV→RGB path, until the upstream meta-box over-allocation
  (F-AVIF-3) tripped libFuzzer's 2 GB malloc limit.

**Findings.**

| id | class | input (committed?) | root cause | disposition |
|---|---|---|---|---|
| F-AVIF-1 | panic (bucket c) | `bad_parser_state.avif` (32 B) ✓, `meta_parser_state.avif` (235 B) ✓ | `avif-parse` `check_parser_state` `debug_assert_eq!(0, limit, "bad parser state bytes left")` (`src/lib.rs:1398`, reached from `:784` and `:921`) fires on unread box content. Debug-only; `--release` returns a clean `Err`. | **Fixed at boundary.** `box_sizes_fit` rejects the size-overrun variant before `avif-parse`; `catch_unwind` in `decode_avif` converts any residual upstream panic to `ImageError::Decode` under debug-assertions (`cargo test`). 3 regressions + smoke. Reported upstream. |
| F-AVIF-2 | OOM (bucket b/c) | `container_box_size_bomb.avif` (286 B) ✓ | `ftyp` box size field = `0xB8000018` ≈ **3.09 GB**; `avif-parse` reads a box's declared size into a buffer before validating it against the input → 3 GB `malloc` inside `read_avif`, before any of our caps run. | **Fixed at boundary.** `box_sizes_fit(bytes)` walks the top-level ISOBMFF boxes and rejects any whose declared size overruns the buffer. Regression + smoke; mutation-checked under the fuzzer (original OOMs, fixed is clean). |
| F-AVIF-3 | OOM (bucket c, upstream) | recorded not committed — sha256 `f1f8d36a…`, 286 B | `avif-parse` `read_iinf`/`read_iloc`/`read_meta` call `TryVec::with_capacity(count)` on an attacker-controlled count field **inside** the `meta` box (`src/lib.rs:977/1285/1324/1381`) → ~**4.26 GB** reservation. Passes `box_sizes_fit` (top-level sizes are valid). | **Documented upstream.** Not fixable at our boundary without vendoring/forking `avif-parse` (out of scope: no new deps, no structural rewrite). **Mitigated by `avif-parse`'s fallible `TryVec`**: on a memory-constrained heap the reserve returns `Error::OutOfMemory` → our `map_parse_err` → typed `ImageError::Decode` (no UB/panic); on a big-RAM host it is a transient DoS. **Deliberately NOT added to the always-on smoke** (a multi-GB allocation could OOM-kill CI). Reported upstream. |

**Defense-in-depth also added:** `frame_size_limit(limits)` passes dav1d's
`frame_size_limit` (derived from the DEC-034 pixel budget) into `re_rav1d`, so a
crafted OBU that declares a huge frame is rejected at header-parse time (obu.rs
enforces it) instead of allocating planes before our post-decode `check_caps`.
This closes a latent decode-stage gap even though F-AVIF-1/2/3 were all
container-stage.

**Upstream report (recommended text):** *avif-parse 2.1.0 — malformed input:
(a) `check_parser_state` uses `debug_assert!` for a recoverable "unread box
content" condition (panics under debug-assertions; the `else` branch already
returns `Error::InvalidData`); (b) box/extent/count size fields are used to size
allocations (`read_into_try_vec`, `TryVec::with_capacity`) before being bounded by
the remaining input, enabling multi-GB allocations from tiny files. Suggest
bounding every declared size/count by the bytes actually available.*

---

### `svg_decode` — CLEAN

SVG rasterizes via `resvg`/`usvg`/`tiny-skia` with the SPEC-060 hardening
(`resources_dir=None`, no external file/URL refs) + the DEC-034 caps.

**Budget achieved (default config, debug-assertions on — the strictest):**
`Done 2390202 runs in 601 second(s)` — **no crash / panic / OOM / hang**.
`cov: 12296  ft: 60745  corp: 3639`, exec/s ~4–10k. Coverage grew strongly from
the single 336-byte text seed (past the XML parse into the render path), so the
seed corpus was not grown.

**Findings:** none. A clean run is a first-class, sufficient outcome (DEC-062).
The seed fixture flows through the always-on `fuzz_corpus_never_panics` smoke.

---

### `raw_preview` — no panic/crash; one documented memory-amplification residual (F-RAW-1)

RAW extracts the largest embedded JPEG preview: a byte-scan for `FF D8 FF`
markers, a plausible-marker prune, and decode of each candidate through the
DEC-034-capped `image` JPEG decoder, bounded by `MAX_PREVIEW_CANDIDATES` (SPEC-061).

**Budget achieved (default config, debug-assertions on):**
`Done 1812513 runs in 602 second(s)` — no panic/crash. `cov: 2626  ft: 14490
corp: 2024`, exec/s ~3k. Coverage is lower than AVIF/SVG by design — the surface is
our marker-scan/prune logic plus `image`'s JPEG decoder, not a full container parser
— and grew steadily from the 1.3 KB synthetic seed. **This debug run did not mutate
to the F-RAW-1 bomb within budget; the canonical `-O` `just fuzz` gate does (below).**

**Findings:** no panic/crash/UB — the contract holds. But **one memory-amplification
residual (F-RAW-1)**, the same class as F-AVIF-3, surfaced by the canonical `-O`
`just fuzz raw_preview` gate and confirmed on the **shipped release binary**:

| id | class | disposition |
|----|-------|-------------|
| F-RAW-1 | transient memory DoS (bucket b — a cap gap, not a crash) | ✅ **CLOSED by SPEC-070 / DEC-063** (2026-07-11) — see below. *(Original disposition: documented residual, filed.)* A < 800 B `.nef` whose embedded JPEG's SOF declares **16384×9776** drives the `image` JPEG decoder to a **~1.9 GB peak working set** (`crustyimg info` on the 782 B reproducer → `dimensions: 16384x9776`, `peak memory footprint 1.93 GB`; ≈2470× amplification). It **passes the DEC-034 caps** (16384 < 65535; 480 MB RGB output < 512 MB `max_alloc`) because `image::Limits.max_alloc` bounds a **single allocation**, not the **cumulative/peak** working set. No panic/abort/UB — a valid decode that just costs ~1.9 GB transiently — so the contract holds, but the `-O` gate straddles libFuzzer's default 2048 MB `rss_limit` and nondeterministically OOM-aborts the fuzzer (found in ~60 s in one run; a clean-seed 100 s run peaked 903 MB and passed). **Pre-existing and not RAW-specific:** the same crafted dimensions balloon the plain `.jpg` decode path to ~1.45 GB too; not introduced by this branch. **Root gap:** `max_alloc` bounds single-alloc, not peak/cumulative decode memory, across *all* JPEG decode. |

**F-RAW-1 closure (SPEC-070, DEC-063).** The root gap is fixed: a **declared-pixel
budget** (`MAX_IMAGE_PIXELS` = 64 Mpix, from a 1 GiB peak budget ÷ a measured ~4×
amplification over the RGBA output) is now checked on the **declared** dimensions at
**every** decode seam before the allocation — the generic `ImageReader` path and the
RAW SOF peek (which had no pre-decode dimension check at all), plus AVIF/SVG/HEIC,
which already had dims and are now aligned to the same cap. On the real binary the
reproducer's peak RSS drops **1.93 GB → 8.7 MB** and it exits **1**
(`LimitsExceeded`) instead of 0; a legitimate 24 MP photo still decodes. The
tradeoff (images > 64 Mpix are rejected) is stated in DEC-063.

The minimized reproducer is now committed at
`tests/fixtures/fuzz/raw_preview/pixel_bomb.nef` (sha256 `d4276ee7…`) and — because
it is rejected cheaply at the header rather than decoded — it has **graduated into
the always-on `fuzz_corpus_never_panics` smoke**, which SPEC-069 could not do (a
~2 GB-alloc input risked OOM-killing CI). It also has a dedicated regression
(`raw_pixel_bomb_is_limits_exceeded_not_multi_gb_decode`). The seed fixtures and
every other committed reproducer flow through the same smoke.

⚠️ **F-AVIF-3 is NOT closed by this.** It is an over-allocation inside `avif-parse`
during **container parsing** (`read_avif_meta`), *before* frame dimensions exist to
check — unreachable by a dimension peek without vendoring the parser. It remains the
separately-filed upstream item.

---

### `heic_decode` — RAN (best-effort); no libheif finding; bounded by the shared avif dispatch

HEIC decodes via C `libheif` behind the `heic` feature (DEC-052). The host had
system libheif `1.23.1` (Homebrew, libde265 backend), so the target built
(`--features heic`, `PKG_CONFIG_PATH=/opt/homebrew/opt/libheif/lib/pkgconfig`) and
ran with ASAN — **not skipped**.

**Budget achieved.**
- **default config:** `#618256  cov: 1108`, exec/s ~18k, then aborted on an
  `avif-parse` `debug_assert!` (`src/lib.rs:987`, the same `check_parser_state`
  class as F-AVIF-1).
- **`-O` config:** reached the libheif decoder (`INITED cov: 1047`), explored to
  `#107352  cov: 1274`, then OOM'd (`malloc(2214626834)` ≈ 2.21 GB). The backtrace
  is `avif_parse::read_avif_meta` — the **same F-AVIF-3** meta over-allocation.

**Finding:** the `heic_decode` target routes through `Image::from_bytes`, which
**content-sniffs and dispatches AVIF before HEIC** (both are ISOBMFF/`ftyp`
containers). So mutations of the HEIC seed that look AVIF-branded reach
`decode_avif`, and both runs were ultimately bounded by `avif-parse`'s
already-documented container issues (F-AVIF-1 debug-assert / F-AVIF-3
over-allocation) — **not** a HEIC/libheif defect. **No libheif/HEIC-specific
crash, panic, or OOM was found** in the exercised decode path. Per DEC-052, HEIC's
memory safety is libheif's (a C dep); our Rust contract ("no panic; default build
answers `.heic` with `CodecNotBuilt`→exit 4") holds, and the `#[cfg(feature =
"heic")]` corpus smoke drives the valid HEIC fixture through libheif with no panic.
The `-O` OOM reproducer is recorded (sha256 `f0f1e327…`) but **not committed** to
the smoke (it is F-AVIF-3, a multi-GB alloc). **Follow-up:** a dedicated
HEIC-only decode entry (bypassing the AVIF sniff) would let `heic_decode` fuzz
libheif in isolation instead of being diverted to the AVIF path — noted for the
STAGE-024 backlog.

---

## Repeat recipe

Prerequisites (one-time): `rustup toolchain install nightly` and
`cargo install cargo-fuzz`. For HEIC: `brew install libheif` (with the libde265
plugin) and build `--features heic`.

```sh
# From the repo root. `just fuzz` seeds the gitignored fuzz/corpus/<target> from
# tests/fixtures and runs in -O (release, debug-assertions off = production).
just fuzz avif_decode          # 600s floor
just fuzz svg_decode 300       # custom budget (seconds)
just fuzz raw_preview
# HEIC (best-effort; needs system libheif):
cargo +nightly fuzz run -O --features heic heic_decode fuzz/corpus/heic_decode -- -max_total_time=600

# Reproduce / minimize a crash artifact:
cargo +nightly fuzz run  -O <target> fuzz/artifacts/<target>/<crash>
cargo +nightly fuzz tmin -O <target> fuzz/artifacts/<target>/<crash>
```

The always-on durable guard (no fuzzer, runs every PR on 3 OSes):

```sh
cargo test --test fuzz_regressions   # regressions + fuzz_corpus_never_panics smoke
```
