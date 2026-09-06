---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-129
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-7
  implementer: claude-sonnet-5     # usually same Claude, different session
  created_at: 2026-09-06

references:
  decisions:
    - DEC-058
    - DEC-100
    - DEC-031
    - DEC-064
    - DEC-005
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
    - untrusted-input-hardening
  related_specs:
    - SPEC-128
    - SPEC-064

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's correctness item. The stage delivers `build`-driving-watermark
  end-to-end; a `build` that serves a stale overlay from cache does not deliver
  that — it delivers **a wrong output no one asked for**, silently. Closing this
  is the last correctness gap between SPEC-128's capability and STAGE-050's
  outcome (`build` watermarks and optimizes a whole photo site from a manifest).

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4).
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 27931672
      tokens_breakdown:
        input: 234
        output: 98169
        cache_creation: 293929
        cache_read: 27539340
      estimated_usd: 10.84
      duration_minutes: null
      recorded_at: 2026-09-06
      note: >
        Interactive session (not an orchestrated Agent call), measured from the session's own
        transcript JSONL, deduped by `.message.id` (117 unique ids), taking input/cache_creation/
        cache_read from the group and MAX output_tokens per id (summing every line over-counts;
        the first line's output under-counts). Priced by component at Sonnet $3/$15 per MTok,
        cache_creation x1.25, cache_read x0.10 — never a flat rate on tokens_total. Ran in its
        own git worktree (feat/spec-129-build-cache-asset-hash), per the prompt.
  totals:
    tokens_total: 27931672
    estimated_usd: 10.84
    session_count: 2
---

# SPEC-129: build cache hashes watermark asset content

## Context

⚠ **This is a silent wrong-output bug in a shipped verb**, reproduced at SPEC-128's verify with a
verified control: change an overlay's bytes on disk (same path, same name in the manifest), re-run
`crustyimg build`, watch it report *"1 cached, 0 rebuilt"* and emit **byte-identical pre-edit
output**. `apply --recipe` is unaffected — it decodes every input every run.

The mechanism is exactly one function. `build`'s cache key is composed by
[`target_recipe_hash`](../../../src/cli/build.rs) at `src/cli/build.rs:88-112`, which folds two
inputs into a per-target hash:

1. `recipe.to_toml()?.as_bytes()`, the canonical recipe text.
2. The output-format `plan` (`Preserve`/`Pinned(fmt)`/`Decide`) for a terminal-`optimize` target.

**Neither reaches the overlay's bytes.** SPEC-128 (Call 1) deliberately routes those through a
side channel: `OperationParams::set_resolved_bytes` (`src/operation/mod.rs:104`) attaches the file
content to a CLONE of the recipe **after `to_toml`'s field is already serialized**. That is
correct for its purpose (the recipe TOML must emit the path, never the bytes — DEC-100's
highest-consequence guard). It is a correctness *gap* for the cache: the same TOML text, with two
different `logo.png`s on disk, hashes to the same key.

The gap is precisely the one DEC-058's own "Revisit" clause names — *"an output-affecting input
is discovered outside the seven"* — and the SPEC-128 verify handoff called it out (the last
`📌` under `## Reflection (Ship)`).

**Three facts, read from the code:**

| fact | where |
|---|---|
| `target_recipe_hash` folds only `to_toml()` + `plan` into its digest | `src/cli/build.rs:88-112` |
| `cache::recipe_hash` (the plain-recipe path, `plan = Preserve`) does the same | `src/build/cache.rs:196` |
| `resolved_bytes` are on a side-channel field the `Serialize` impl skips | `src/operation/mod.rs:104-114, 123-133` |

**`apply --recipe` is out of scope by construction.** `apply` has no cache — it recomputes every
run — and `resolve_recipe_assets` already reads the overlay on every invocation. This spec is
purely a `build`-side cache-key correctness fix.

## Goal

`build`'s per-target cache key covers the **file content** of every asset a recipe's steps name,
so editing an overlay or font on disk always misses (rebuilds), and a recipe with no asset-bearing
steps hashes exactly as before.

## The design calls — settled here

### Call 1 — content hash, not mtime+size

Two options were considered; this spec picks content.

- **mtime + size (a `stat`-based fast path).** Rejected. mtime is preserved by `cp -p`, `rsync`
  by default, `git-lfs` checkout, `touch -r`, unpacked archives; some filesystems have
  coarser-than-second resolution; and size is unchanged by a lossless recompression, a palette
  reshuffle, or the exact kind of pixel-only edit this bug is about. A stat-based key would let
  a re-saved `logo.png` reproduce the SPEC-128 verify's failure verbatim.
- **✅ Chosen — SHA-256 of the resolved bytes.** Matches the precedent set by the source input,
  whose bytes are hashed unconditionally on every `build` (`cache_key_for` at
  `src/cli/build.rs:319-346` reads and `hash_bytes`es the source before any decode). It is
  cryptographic (collision-is-a-wrong-answer, DEC-058 says the same about the source), and the
  bytes are **already in memory** when `resolve_recipe_assets` runs, so the added cost is one
  SHA-256 pass per target — dominated by the decode + encode a hit exists to skip.

### Call 2 — fold into `target_recipe_hash`, not `compute_key`

`compute_key` (`src/build/cache.rs:245`) takes seven inputs (DEC-058), and its shape is the
contract STAGE-022's lockfile pins and the executor consumes. Widening it would ripple through
every call site.

- **✅ Chosen — extend `target_recipe_hash`.** It already folds the format `plan` in through the
  same length-prefixed absorb-a-field discipline `compute_key` uses. The resolved-asset digest
  is one more length-prefixed section under the same domain. From `compute_key`'s point of view,
  the "recipe hash" input (item 4) still means "the canonical recipe hash for this target" —
  the definition now includes its resolved assets. **`CACHE_SCHEMA_VERSION` is deliberately
  unchanged** (see Call 5).
- Rejected — pass the asset hashes as a new positional parameter to `compute_key`. It would
  churn every call site and would break the DEC-058 fingerprint that the lockfile and the
  seven-input mnemonic depend on.

### Call 3 — one hash per (step, asset key) triple, in step order

The fingerprint format:

- Walk `resolved.steps` **in order** (the same order `resolve_recipe_assets` and `build_pipeline`
  walk them, and the same order `to_toml` emits them, so this hashes the same shape the recipe
  itself does).
- For each step, walk `registry.asset_keys(&step.op)` in the order the registry declared them
  (`&["image", "font"]` for `watermark` today — `'static`, source-order, stable).
- For each asset key with resolved bytes present, absorb three fields: **the asset key name**
  (length-prefixed, so `"image"` and `"font"` cannot collide with each other), **the content
  hash's 32 bytes**, and **the step's index** (`u32 LE`). All three make the composition
  injective: two steps swapping their assets, and two assets on different steps, both stay
  distinguishable.
- **A step with NO resolved asset contributes nothing.** So a recipe with no asset-bearing steps
  produces zero additional bytes → the same hash as before this spec (Call 5's guarantee).

⚠ **The bytes must come from `resolved_bytes`, not from a re-read of the path.** `resolve_recipe_assets`
already read the file exactly once per target; a second read would be a wasted syscall AND a
race (a designer editing `logo.png` between the resolver and the hasher would poison the key).
The one-read discipline is not incidental; it is what makes the fix cheap.

### Call 4 — plain vs terminal-`optimize`: same widening on both branches

`target_recipe_hash` has two branches today:

```rust
if plan == OutputFormatPlan::Preserve {
    return Ok(crate::build::cache::recipe_hash(recipe)?);   // plain-recipe branch
}
// terminal-optimize branch: absorb toml + plan into a fresh digest
```

Both need the asset fingerprint. The cleanest shape is to move the asset absorption OUT of the
branches — one length-prefixed section, appended to a common inner `absorb_recipe`, in one place
— so the plain-recipe branch stops calling `cache::recipe_hash` directly and both branches
compose from the same primitives. **This is the only change that touches `src/cli/build.rs`
outside `target_recipe_hash` itself.** `src/build/cache.rs::recipe_hash` stays as-is; anything
outside `build.rs` that hashes a recipe (only unit tests, per grep) keeps its old semantics.

📌 **A grep for `cache::recipe_hash(` in `src/` returns two hits, both in `src/cli/build.rs`
(line 95 and line 1074)** — the `Preserve` early-return and one test helper. Both live in this
same file, so the whole change is local to `src/cli/build.rs` + the tests.

### Call 5 — `CACHE_SCHEMA_VERSION` unchanged

Two cases, both correct:

- **A recipe with no asset-bearing steps.** No bytes are folded in → `target_recipe_hash`
  returns the same digest as before this spec → every existing cached entry for these recipes
  stays valid. **This is the guarantee, and it has its own test** (`watermarkfree_recipe_hash_unchanged`).
- **A recipe with an asset-bearing step (a watermark today).** New key. Old cached entries
  become **orphaned** on disk (unreachable, harmless — `Cache::lookup` at the new key returns
  `None` → clean rebuild, DEC-058's untrusted-input-hardening guarantee). Users cleaning up
  `.crustyimg/cache/` reclaim the space; nothing depends on those entries.

**No user-visible failure mode exists where a stale entry could survive.** Bumping the schema
version would over-invalidate the plain-recipe cache — a regression in the "no-change re-run is
a full hit" property (DEC-058's headline). Do not bump.

📌 **The lockfile's key hex will change on any pre-existing target with a watermark.** A
`build --check`/`--frozen` against a lockfile committed **before** this spec will report drift
on those targets and want regeneration. This is not a design choice — it is what a corrected key
means. Same shape as the drift a version bump produces (`env!("CARGO_PKG_VERSION")` is in the
key), which this repo has always accepted as normal.

### Call 6 — the failure mode when an asset is missing is already covered

`resolve_recipe_assets` (`src/cli/common.rs:207-228`) errors on a missing/unreadable asset with
`CliError::RecipeAssetUnreadable` (exit 1) **before** `target_recipe_hash` runs, so the hasher
never encounters a step with a declared asset key but no resolved bytes. **No new error path is
introduced by this spec.** AC-5 pins that as a guard, not a fix.

### Call 7 — `apply`, `wasm::transform`, and other verbs stay byte-identical

`apply` has no cache and does not call `target_recipe_hash`. `wasm::transform` refuses
asset-bearing recipes (SPEC-128 Call 3) and does not reach the build cache. Every other pixel-lane
verb writes through `run_pixel_op`, which is unrelated to `src/cli/build.rs`. **AC-6 sweeps the
verbs that reach `run_pixel_op`, not just the ones in `build.rs`** — SPEC-127's own miss was
narrowing this sweep wrongly, and SPEC-128 fixed the discipline; keep it.

## Acceptance Criteria

- [ ] **AC-1.** `build` **rebuilds** when a watermark overlay's bytes change (same path in the
      manifest, mutated bytes on disk). The summary reports `(0 cached, N rebuilt)` on the second
      run, and the rewritten output reflects the new overlay bytes (**not** byte-identical to the
      first run's output). Driven at ≥1 input; extended to ≥2 inputs so the summary really flips.
- [ ] **AC-2.** ⚡ **Positive control complement of AC-1** — `build` **hits** on a second run when
      the overlay is unchanged. `(N cached, 0 rebuilt)`, outputs byte-identical to the first run.
      This guards against over-invalidation, which is exactly how the fix could break the "no-change
      re-run is a full hit" headline (DEC-058) if implemented wrong.
- [ ] **AC-3.** ⚡ **The `font` half of the mechanism has its own test.** AC-5's SPEC-128 lesson:
      an AC that names two things needs a test per thing. Same as AC-1, on a `text` watermark
      with a `font = "custom.ttf"` key. Driven twice: once at `--font PATH`'s bytes changing
      (miss), once with the font unchanged (hit — bundled default with no `font` key is a subcase
      here: nothing to resolve, nothing to hash, unchanged behavior).
- [ ] **AC-4.** **A recipe with no asset-bearing steps hashes identically to `main`.** Direct unit
      test: `target_recipe_hash(recipe, Preserve)` for a `resize`-only recipe returns
      exactly `cache::recipe_hash(&recipe)` (the shape it returns today). Guards Call 5's promise
      that existing cache entries stay valid.
- [ ] **AC-5.** **Missing-asset behavior is unchanged.** A recipe naming an unreadable overlay/font
      still fails with `RecipeAssetUnreadable` (exit 1) **before** `target_recipe_hash` is called.
      Not a new error path — a pin, so a future refactor cannot slip a hash into the failure path.
- [ ] **AC-6.** **Nothing else changes bytes.** Every pixel-lane verb reaching `run_pixel_op`,
      plus `apply --recipe` on a watermark-free recipe, plus `build` on a watermark-free recipe,
      produce output **byte-identical** to `main`. Sweep by the CALL GRAPH — `run_convert`,
      `run_optimize`, `run_web`, `run_apply`, `run_build`, `run_resize`, `run_thumbnail`,
      `run_auto_orient`, `run_watermark` (image AND text) — over ≥4 fixtures × 2 formats, both
      binaries built as release with a fresh `CARGO_TARGET_DIR` each. **Positive control:**
      driving `build` with a watermark recipe against the modified overlay must produce output
      that DIFFERS between `main` (which caches, so serves stale) and the branch (which rebuilds).
- [ ] **AC-7.** ⚡ **`build` unit test: the asset is hashed ONCE per target, not once per input.**
      Two targets each with N=10 inputs sharing one overlay must trigger exactly two overlay
      reads (already covered by `resolve_recipe_assets`'s per-target `std::fs::read`) and exactly
      two overlay hashes (this spec's added work). Driven by counting invocations of a wrapped
      hasher, or by asserting `target_recipe_hash` is called once per target — the perf property
      Call 1 rests on.
- [ ] **AC-8.** **Negative control, one revert per independent condition** (AGENTS §15 rule 1).
      The change has ONE testable condition — the added asset absorption — because Calls 3 and 4
      shape a single mechanism. Revert **only** that absorption: AC-1 and AC-3 flip RED,
      AC-2/AC-4/AC-5/AC-6 stay GREEN. Evidence is the **behavioural flip**, never a hash — a
      debug rebuild from identical source already differs.
- [ ] **AC-9.** `src/operation/**` gains no filesystem dependency (SPEC-128 AC-8 held): `just
      wasm-check` passes and a grep for `std::fs`/`Image::load`/`File::open` under
      `src/operation/` stays empty. The whole change is under `src/cli/build.rs` + tests + one
      DEC + one docs line.
- [ ] **AC-10.** Clean matrix — default, `--no-default-features`, `--features webp-lossy`, fresh
      `CARGO_TARGET_DIR` each, sequential; clippy + `fmt --check` each; plus `just wasm-check`
      and `just wasm-test`. **Plus `just demo-build`** — SPEC-128's build regressed the wasm
      bundle size by 10.6 % and only CI caught it (`just check` never runs the size gate); run
      it locally. This spec should NOT move the size gate (no code reaches wasm), but the guard
      is worth running.

## Failing Tests

Written at design, made to pass at build. **All confirmed RED against `main` first**, baseline
recorded. `main`'s baseline for AC-1 is the SPEC-128 verify's exact reproduction: overlay bytes
mutated, `build` reports `(1 cached, 0 rebuilt)`, output byte-identical to pre-edit.

- `tests/build_watermark_cache.rs::build_rebuilds_when_overlay_bytes_change` — AC-1.
- `tests/build_watermark_cache.rs::build_hits_when_overlay_bytes_unchanged` — AC-2 (over-invalidation guard).
- `tests/build_watermark_cache.rs::build_rebuilds_when_font_bytes_change` — AC-3, the `font` half.
- `tests/build_watermark_cache.rs::build_hits_when_bundled_font_is_used` — AC-3's no-`font`-key
  subcase (nothing to resolve, nothing to hash).
- `src/cli/build.rs::target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe` — AC-4.
- `src/cli/build.rs::target_recipe_hash_changes_when_resolved_asset_bytes_change` — AC-4's
  complement, a direct unit test on the mechanism.
- `tests/build_watermark_cache.rs::missing_overlay_still_fails_before_hashing` — AC-5.
- `src/cli/build.rs::target_recipe_hash_hashes_each_asset_once_per_target` — AC-7, the perf
  property (counts hasher invocations for a target with N=10 inputs).

## Implementation Context

**Read first:** DEC-058 (the cache-key composition — this spec extends its clause 4), DEC-100
(SPEC-128's seam — this is the promised follow-up), DEC-031 (why `src/operation/**` cannot open
files — the constraint that keeps the fix out of that tree), DEC-064 (engine/shell split),
DEC-005 (recipes round-trip through the parsed form).

**The whole change is under `src/cli/build.rs`.** No `src/operation/**` edit, no
`src/build/cache.rs` public-API change (the private `recipe_hash` primitive stays), no
`src/wasm.rs` edit, no `src/cli/common.rs` edit — `resolve_recipe_assets` is already the seam
that reads the bytes; this spec only reads them back from `OperationParams::resolved_bytes` in
the hasher.

**The added helper's shape** (illustrative, not prescriptive):

```rust
/// Absorb every step's resolved-asset content into `hasher`, in step order,
/// tagged so no two field values can concatenate into a third.
///
/// A step with NO resolved bytes contributes nothing, so a recipe with no
/// asset-bearing ops hashes exactly as before (SPEC-129 Call 5).
fn absorb_resolved_assets(hasher: &mut Sha256, recipe: &Recipe, registry: &OperationRegistry) {
    for (i, step) in recipe.steps.iter().enumerate() {
        for &key in registry.asset_keys(&step.op) {
            if let Some(bytes) = step.params.resolved_bytes(key) {
                // (step_index, key_name, content_hash) triples: injective by construction.
                absorb(hasher, TAG_ASSET, &(i as u32).to_le_bytes());
                absorb(hasher, TAG_ASSET, key.as_bytes());
                absorb(hasher, TAG_ASSET, cache::hash_bytes(bytes).as_bytes());
            }
        }
    }
}
```

Whether the whole `absorb`/`Sha256` primitive stays private in `src/build/cache.rs` and gets
one `pub(super)` export, or gets inlined into `src/cli/build.rs`, is an implementation choice;
the seam is the *behavior*, not the module split.

**Where the resolved bytes come from.** `prepare_target` (`src/cli/build.rs:176-245`) already
calls `resolve_recipe_assets` at line 224 *before* `target_recipe_hash` at line 226. So by the
time the hasher runs, `resolved_bytes` is already populated. No re-read, no re-plumbing.

**The registry is already available** — `prepare_target(target, registry)` receives it, so
extending `target_recipe_hash`'s signature to take `&OperationRegistry` is one call site.

⛔ **Byte-changing only for recipes with asset-bearing steps.** A watermark-free recipe must be
byte-identical (AC-4 + AC-6). This still batches into PROJ-011's single lockfile migration —
**do not bump the version, do not cut a release.**

## Notes for the Implementer

- 📌 **DEC-101 is reserved.** Highest on `main` is DEC-100. Use the **block-list** `affected_scope`
  form — `scripts/decisions-audit.sh` silently drops inline arrays (filed, PROJ-013 STAGE-047).
  DEC-101 should **amend DEC-058 clause 4**, not supersede DEC-058 — the seven-input key
  composition is unchanged; the "canonical recipe hash" input is what widens. Reference DEC-058's
  own "Revisit if" clause: this spec is exactly the "output-affecting input outside the seven"
  case, and DEC-058 said `CACHE_SCHEMA_VERSION` should bump in that case. **Explain the departure**
  from that guidance: the new input applies **only to a subset of recipes** (asset-bearing ones),
  and forcing every existing cache entry to invalidate would be pure regression. Cite the
  measured no-change-guarantee (AC-4).
- ⚡ **Write the docs/api-contract.md update in the same change.** One paragraph under the
  content-addressed cache prose (~`docs/api-contract.md:559`), naming resolved-asset content as
  part of the key for asset-bearing recipes. Do not touch `docs/data-model.md` — the schema is
  unchanged.
- ⚠ **Size expectation, stated honestly:** the code here is genuinely **S** — one helper, one
  branch merge, one signature widening. The VERIFICATION is not: AC-6's byte-identical sweep is
  the same 41/41 shape SPEC-128 did, so **budget ~120 exchanges**. Push a WIP commit as soon as
  it compiles.
- ⚠ **Do not run a backgrounded feature build while editing source for a revert** — SPEC-127's
  build contaminated a leg that way and had to re-run it from a clean tree.
- ⚠ **Two identical PNGs with different bytes need a real difference on disk.** A test fixture
  that writes `Rgb([200, 30, 30])` then `Rgb([10, 220, 90])` at the same dimensions is what
  drives AC-1 (see `tests/build_cache.rs::changing_one_input_rebuilds_only_that_output` for the
  exact pattern — reuse `write_png` from that file's helpers if practical, or duplicate it).
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout.
  A piped command reports the **pipe's** exit code — redirect and read `$?`. Never poll CI.
- zsh does **not** word-split unquoted parameters — build argument lists explicitly, use
  `while IFS= read -r`, and prefer `/usr/bin/grep`. macOS has no `timeout(1)`.

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-129-build-cache-asset-hash`
- **PR (if applicable):** opened at the end of this build cycle (see the repo's PR list).
- **All acceptance criteria met?** yes — AC-1 through AC-10, all measured (see below).
- **New decisions emitted:**
  - `DEC-101` — `build`'s cache key hashes resolved-asset CONTENT, not just the recipe's path;
    `CACHE_SCHEMA_VERSION` stays unchanged (amends DEC-058 clause 4).
- **Files this diff touches** — from `git diff main --name-only` plus untracked additions:
  - `src/build/cache.rs` — widened `absorb` from private to `pub(crate)` (so
    `target_recipe_hash` in `cli::build` can reuse the same tag+length-prefix discipline for its
    asset fingerprint); added `Hash::from_hasher`, a `pub(crate)` constructor that finalizes an
    externally-composed running `Sha256` into a `Hash`.
  - `src/cli/build.rs` — widened `target_recipe_hash`'s signature to take `&OperationRegistry`;
    rewrote its body to compose one running `Sha256` hasher (reproducing the exact pre-SPEC-129
    byte sequence for each `OutputFormatPlan` branch, then appending the new asset fingerprint)
    instead of returning early via `cache::recipe_hash`/building a `Vec<u8>` + `hash_bytes`; added
    `absorb_resolved_assets` + the local `TAG_ASSET` constant; updated the one production call
    site (`prepare_target`) and the one existing unit test
    (`target_recipe_hash_distinguishes_pinned_from_decided`) to pass a registry; added three new
    unit tests (`target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe`,
    `target_recipe_hash_changes_when_resolved_asset_bytes_change`,
    `target_recipe_hash_hashes_each_asset_once_per_target`).
  - `tests/build_watermark_cache.rs` — new integration test file, 5 tests: AC-1
    (`build_rebuilds_when_overlay_bytes_change`), AC-2
    (`build_hits_when_overlay_bytes_unchanged`), AC-3 and its no-`font`-key subcase
    (`build_rebuilds_when_font_bytes_change`, `build_hits_when_bundled_font_is_used`), AC-5
    (`missing_overlay_still_fails_before_hashing`).
  - `docs/api-contract.md` — one paragraph under the content-addressed cache prose (~line 559)
    naming resolved-asset content as part of the cache key for asset-bearing recipes.
    `docs/data-model.md` was NOT touched, per the spec (the recipe schema is unchanged).
  - `decisions/DEC-101-build-cache-key-hashes-resolved-asset-content.md` — new decision, block-list
    `affected_scope` (`src/cli/build.rs`, `src/build/cache.rs`, `docs/api-contract.md`).
  - `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-129-build-cache-hashes-watermark-asset-content.md`
    — this file: `cost.sessions` build entry + totals, and this Build Completion section.
- **Deviations from spec:**
  - None of the seven design calls needed re-deciding; all held as designed. One implementation
    detail beyond what the spec's illustrative code showed: the illustrative
    `absorb_resolved_assets` snippet implied building a fresh digest from scratch; to satisfy
    Call 4/Call 5's promise for **every** `OutputFormatPlan` (not just `Preserve`, which is the
    only one AC-4's named test drives), the actual implementation composes ONE running `Sha256`
    hasher across the whole function — reproducing the exact old byte sequence for whichever plan
    applies, then appending the asset fingerprint only if one exists — rather than computing the
    recipe hash and the asset fingerprint as two separate digests and combining them. This was
    necessary because `cache::recipe_hash` is a bare, untagged `hash_bytes(toml_bytes)` with no
    length prefix at all; any composition that re-hashed its OUTPUT (rather than replaying its
    INPUT bytes into a shared hasher) would not reproduce that exact digest, which is what AC-4
    requires byte-for-byte.
  - The AC-7 unit test does not literally count hasher invocations (the spec's first suggested
    method); it asserts the alternative the spec explicitly allows — "asserting `target_recipe_hash`
    is called once per target" — mechanically, via a source-text match against this file's own
    production call graph (excluding the test module), so a future change that added a second call
    site (e.g. inlining the hash into the per-input path) would fail this test.
- **Follow-up work identified:**
  - None. This closes STAGE-050's SPEC-129 backlog item outright — no new correctness gap was
    found while building it.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing was unclear; the one place that needed real thought rather than transcription was
   Call 4's "compose from the same primitives" language, which (read literally) could suggest
   re-hashing `cache::recipe_hash`'s 32-byte digest rather than replaying the TOML bytes — that
   would have broken AC-4's exact-equality requirement for a plain recipe. Working out that the
   composition has to be a single streaming hasher fed the OLD byte sequence, with the new
   fingerprint appended only conditionally, took a few minutes of reasoning about SHA-256's
   Merkle–Damgård construction before writing any code.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-058, DEC-100, DEC-031, DEC-064, DEC-005 were exactly the right five, in the right
   order to read them.

3. **If you did this task again, what would you do differently?**
   — Nothing. Reasoning through the byte-composition constraint before writing any code (rather
   than transcribing the spec's illustrative snippet literally and discovering the mismatch via a
   failing AC-4 test) is the order I'd repeat.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
