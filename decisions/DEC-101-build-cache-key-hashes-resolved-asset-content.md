---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-101
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-09-06
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/build.rs
  - src/build/cache.rs
  - docs/api-contract.md

tags:
  - build
  - cache
  - hashing
  - watermark
  - dec-058
  - dec-100
---

# DEC-101: `build`'s cache key hashes resolved-asset CONTENT, not just the recipe's path; `CACHE_SCHEMA_VERSION` stays unchanged

## Decision

`target_recipe_hash` (`src/cli/build.rs`) now folds the **content** of every resolved asset a
recipe's steps name — today, a `watermark` step's `image`/`font` — into the per-target cache key,
in addition to the canonical recipe TOML and the output-format plan it already hashed. This
**amends DEC-058's clause 4** ("canonical recipe hash"): the seven-input key composition DEC-058
established is unchanged in shape, but the definition of "canonical recipe hash" for a target now
widens to include its resolved-asset content, not just the path a step names.

Composed as: reproduce, byte-for-byte, the exact sequence of bytes `target_recipe_hash` fed its
hasher before this spec (the raw recipe TOML for a plain recipe; the length-prefixed TOML + plan
discriminator for a terminal-`optimize` target) — then, ONLY if the recipe has resolved-asset
bytes attached, append one length-prefixed `(step index, asset-key name, content hash)` triple per
resolved asset, in step order. A recipe with no asset-bearing steps therefore hashes to the
**identical digest** it hashed to before this spec, on every format plan — not an approximation,
a byte-for-byte reproduction, verified directly
(`target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe`, AC-4).

**`CACHE_SCHEMA_VERSION` is deliberately NOT bumped.** DEC-058's own "Revisit if" clause names
exactly this situation — *"an output-affecting input is discovered outside the seven"* — and
says to bump the schema version when it happens, which invalidates every cache entry. This
decision departs from that suggested remedy: the new input (resolved-asset content) applies only
to a **subset** of recipes — asset-bearing ones — so bumping the schema version would invalidate
every plain-recipe cache entry and committed lockfile line in existence to fix a bug that only
ever affected watermark recipes. That is pure over-invalidation with no correctness benefit, and
it would regress DEC-058's own headline ("a no-change re-run is a full hit") for every user who
has never touched a watermark. The measured guarantee that makes the narrower fix sound is AC-4:
a watermark-free recipe's key is provably unchanged, so nothing that worked before regresses.

## Context

SPEC-128 (DEC-100) registered `watermark` in `OperationRegistry::with_builtins()` via a
resolve-at-recipe-IO-boundary seam: `OperationParams::set_resolved_bytes` attaches an overlay's or
font's file bytes to a CLONED recipe, on a side channel the `Serialize` impl never touches — by
design, so the recipe TOML always emits the asset's PATH, never its bytes. That is correct for the
recipe format. It is a correctness *gap* for `build`'s cache: `target_recipe_hash` only ever
hashed `recipe.to_toml()` (plus the format plan) — a digest that cannot see the resolved bytes at
all, because they never touch the TOML by construction. So editing an overlay's pixels in place
(same path, same manifest, same recipe) left the cache key unchanged, and `build` reported a full
hit and shipped the byte-identical PRE-EDIT output. Found during SPEC-128's own build, reproduced
at its verify with a positive control (the exact reproduction SPEC-129's RED baseline reused), and
filed rather than folded into SPEC-128 (whose ACs were about registering the op, not the cache).

## Alternatives Considered

### Which hash

- **A `stat`-based fast path (mtime + size).** Rejected: mtime survives `cp -p`, `rsync` (default),
  `git-lfs` checkout, `touch -r`, and an unpacked archive; size is unchanged by a lossless
  recompression or a pixel-only edit at the same dimensions — precisely the SPEC-128 verify's
  reproduction. A stat-based key would reproduce the bug it exists to fix.
- **Chosen: SHA-256 of the resolved bytes**, via the existing `cache::hash_bytes`/`sha2` (DEC-058) —
  the same treatment the primary input already gets. The bytes are already in memory (the resolver
  read them once), so the added cost is one SHA-256 pass per resolved asset per target, dominated
  by the decode+encode a cache hit exists to skip.

### Where the fold happens

- **Widen `compute_key`'s seven-input signature.** Rejected: `compute_key` (`src/build/cache.rs`)
  is the shape STAGE-022's lockfile and the executor both pin; widening it ripples through every
  call site for a change that is really about ONE of its existing inputs (item 4, "canonical
  recipe hash"), not a new independent one.
- **Chosen: extend `target_recipe_hash`**, which already composes the "recipe hash" `compute_key`
  receives. From `compute_key`'s point of view nothing changed — it still receives one `Hash` for
  "the canonical recipe hash for this target"; the definition of that hash widened upstream of it.

### `CACHE_SCHEMA_VERSION`

- **Bump it, per DEC-058's own suggested remedy for an input found outside the seven.** Rejected,
  for the over-invalidation reason in Decision above — argued the same way at the spec's design
  time and confirmed by AC-4's measurement at build.
- **Chosen: leave it at `1`.** The two cases that follow from this are both correct: a
  watermark-free recipe keeps every existing entry valid (Call 5); a recipe with a resolved asset
  gets a NEW key, and its OLD entries become orphaned (unreachable on disk, harmless — a lookup
  at the new key is a clean miss, DEC-058's own untrusted-input-hardening guarantee) until the
  user reclaims the space by clearing `.crustyimg/cache/`.

## Consequences

- **Positive:** editing a watermark overlay or font on disk (same path) is a `build` cache MISS,
  matching the invariant DEC-058 exists to protect — a hit never serves stale bytes for an
  output-affecting change. Every existing plain-recipe cache entry and committed lockfile line
  stays valid (measured, AC-4).
- **Byte-changing, but only for asset-bearing targets.** A `build --check`/`--frozen` run against a
  lockfile committed BEFORE this spec will report drift on any target with a watermark step and
  want regeneration — the same shape of drift a crustyimg version bump already produces
  (`env!("CARGO_PKG_VERSION")` is in the key, per DEC-058), which this repo has always treated as
  normal. No other target's key changes. Batches into PROJ-011's single lockfile migration; no
  crate version bump, no release cut here.
- **The asset is hashed once per TARGET, not once per input** (AC-7): `target_recipe_hash` is
  called from `prepare_target`, which runs once per target and stores the result on
  `PreparedTarget`; the per-input fan-out in `run_build` reads that field, calling nothing. A
  target with N=10 inputs sharing one overlay triggers exactly one overlay hash regardless of N.
- **`src/operation/**` gains nothing new** (SPEC-128's AC-8 holds unchanged): this spec's whole
  diff lives in `src/cli/build.rs` (plus one `pub(crate)` visibility widening and one `pub(crate)`
  constructor in `src/build/cache.rs`) — no new filesystem dependency anywhere, `just wasm-check`
  green, no code newly reaches wasm.
- **Neutral:** `apply --recipe` is untouched by construction — it has no cache (DEC-058) and
  already re-resolves every asset on every run, so it never had this bug.

## Validation

- **AC-1/AC-3** (an on-disk overlay/font edit misses and rebuilds, ≥2 inputs so the summary
  flips): `tests/build_watermark_cache.rs::build_rebuilds_when_overlay_bytes_change`,
  `::build_rebuilds_when_font_bytes_change`.
- **AC-2/AC-3's bundled-font subcase** (over-invalidation guard — an unchanged asset, or no asset
  to resolve at all, stays a full hit): `::build_hits_when_overlay_bytes_unchanged`,
  `::build_hits_when_bundled_font_is_used`.
- **AC-4** (a watermark-free recipe hashes IDENTICALLY to the pre-SPEC-129 formula, plus the
  complement that resolved bytes are load-bearing):
  `target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe`,
  `target_recipe_hash_changes_when_resolved_asset_bytes_change` (`src/cli/build.rs`).
- **AC-5** (a missing/unreadable asset still fails before hashing, exit 1, not a new error path):
  `tests/build_watermark_cache.rs::missing_overlay_still_fails_before_hashing`.
- **AC-6** (nothing else changes bytes; a positive control that CAN detect a difference): 6
  fixtures (4 PNG, 2 JPEG) × 8 pixel-lane verbs (`resize`, `thumbnail`, `convert`, `auto-orient`,
  `optimize`, `web`, `watermark --image`, `watermark --text`) + `apply --recipe` on a
  watermark-free recipe + `build` on a watermark-free recipe, driven against fresh-`main` and
  branch release binaries (isolated `CARGO_TARGET_DIR` each) — 55/55 byte-identical. Positive
  control: a watermark `build` re-run after mutating the overlay's pixel bytes on disk — `main`
  reports `(1 cached, 0 rebuilt)` and emits byte-identical pre-edit output (the bug, reproduced);
  the branch reports `(0 cached, 1 rebuilt)` and emits genuinely different bytes (the fix); the
  two binaries' post-mutation outputs differ from each other, proving the comparison can detect a
  real difference.
- **AC-7** (hashed once per target, not once per input): `target_recipe_hash`'s one production
  call site is asserted mechanically against this file's own source
  (`target_recipe_hash_hashes_each_asset_once_per_target`) — `prepare_target` calls it once per
  target; `run_build`'s per-input fan-out reads the stored `PreparedTarget::recipe_hash` field,
  calling nothing.
- **AC-8** (negative control — revert the one testable condition, the asset-absorption call in
  `target_recipe_hash`): AC-1/AC-3 flip RED (both `build_rebuilds_when_*` tests and the direct
  unit test `target_recipe_hash_changes_when_resolved_asset_bytes_change`); AC-2/AC-4/AC-5/AC-6
  stay GREEN. Verified by literally commenting out the one call, running the suite, and reverting.
- **AC-9** (`src/operation/**` stays file-free): `just wasm-check` green;
  `/usr/bin/grep -rn 'std::fs\|Image::load\|File::open' src/operation/` empty (one unrelated
  doc-comment mention, not code).
- **AC-10** (clean matrix): default, `--no-default-features`, `--features webp-lossy` — each a
  fresh `CARGO_TARGET_DIR`, run sequentially, `cargo clippy --all-targets -- -D warnings` +
  `cargo fmt --check` clean on each, full `cargo test` green on each (486/479/… passed across
  the three legs, 0 failed); plus `just wasm-check` (green), `just wasm-test` (43/43 green), and
  `just demo-build` (bundle assembled, brotli size within the existing baseline window — this
  spec reaches no wasm-compiled code, so the gate was run as a guard, not because anything moved).

## References

- Related specs: **SPEC-129** (this decision's spec), **SPEC-128**/DEC-100 (the seam whose
  resolved-bytes side channel this spec reads from), **SPEC-064**/DEC-058 (the cache key this
  amends).
- Related decisions: **DEC-058** (the seven-input key composition; this decision amends its
  clause 4 and explicitly departs from its "Revisit if" schema-bump suggestion), **DEC-100** (the
  resolve-at-IO-boundary seam and `OperationParams::resolved_bytes`), **DEC-031** (why
  `src/operation/**` cannot open files — the constraint that keeps this fix confined to
  `src/cli/build.rs`), **DEC-064** (the wasm/native split this fix does not touch), **DEC-005**
  (recipes round-trip through the parsed form — why the canonical TOML, not the raw file bytes,
  is what `target_recipe_hash` has always hashed).
- Code: `src/cli/build.rs` (`target_recipe_hash`, `absorb_resolved_assets`), `src/build/cache.rs`
  (`absorb` widened to `pub(crate)`, `Hash::from_hasher` added).
- Stage: `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md`.
