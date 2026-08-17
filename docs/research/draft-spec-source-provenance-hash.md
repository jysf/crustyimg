---
# ═══════════════════════════════════════════════════════════════════════════
# DRAFT / IDEA — NOT COMMITTED WORK. NOT SCHEDULED. NOT ATTACHED TO A STAGE.
#
# Research draft in `docs/research/`, deliberately OUTSIDE the work hierarchy
# (AGENTS.md §2: specs live in `projects/PROJ-*/specs/`). No justfile recipe
# scans this directory; confirmed absent from `just status` and `just backlog`.
# Follows the `draft-stage-linear-working-buffer.md` precedent.
#
# The task/project/stage ids are placeholders ON PURPOSE. Do not allocate a real
# SPEC number until someone decides to schedule it.
# ═══════════════════════════════════════════════════════════════════════════

task:
  id: SPEC-XXX                     # unassigned on purpose
  type: story
  cycle: null
  status: idea
  blocked: false
  priority: null
  complexity: S                    # tier 1 only; M if tier 2 is folded in

project:
  id: PROJ-XXX
  stage: STAGE-XXX
repo:
  id: crustyimg

agents:
  drafted_by: claude-opus-5
  created_at: 2026-08-16

references:
  decisions: [DEC-058, DEC-059, DEC-078, DEC-024, DEC-003]
  constraints:
    - metadata-not-via-pixel-encode
    - test-before-implementation
    - every-public-fn-tested
  related_specs: [SPEC-114]   # shipped; corrects the spike's C2PA framing

cost:
  sessions: []
  totals: { tokens_total: 0, estimated_usd: 0, session_count: 0 }
---

# DRAFT-SPEC: record the source image's own digest, for provenance

> **Status: IDEA.** Not scheduled, not owned, not attached to a stage.

## Context

**The question:** when `optimize` or `web` writes an artifact, can it record a hash of
the *original* so the output can be traced back to where it came from?

**What exists today, verified in the tree:**

- The primitive is already public — `hash_bytes()` (SHA-256) and `Hash::to_hex()`
  (`src/build/cache.rs:151-178`).
- **`build` is already 90% there.** `crustyimg.build.lock` (SPEC-066 / DEC-059) records per
  output: `source` (the path — whose doc comment reads *"(provenance, for review)"*,
  `src/build/lock.rs:154-155`), `recipe`, `key`, `hash` (SHA-256 of the *written* bytes) and
  `bytes`.
- **`optimize` / `web` / `apply` record nothing that identifies the original.** Their shared
  `--json` audit report (`crustyimg.optimize.explain/v1`, `src/cli/mod.rs:322-323`) carries
  `source_format` and `source_bytes` — a **length**, not a digest.

**The gap, stated precisely.** The lockfile's `key` is *not* a source identity. It is
`SHA-256(schema ‖ version ‖ features ‖ recipe_hash ‖ quality ‖ input_ext ‖ input_hash)`
(`src/build/cache.rs:245-252`). You cannot recover the source's own digest from it, and it
changes when crustyimg's version or the recipe changes **even though the source did not**. It
answers *"will this rebuild identically"* — a reproducibility token. It does not answer
*"which original is this"* — an identity.

Those are different properties and want different fields.

## Goal

Record the SHA-256 of the source file's **bytes as read**, plus its locator, in the
machine-readable outputs — so an artifact can be tied back to the original it came from.

## The conceptual point that shapes the whole spec

**A bare digest is not provenance.** It lets you *verify* a link you already suspect; it does
not let you *discover* one. You can only check `sha256(candidate) == recorded` if you already
hold the candidate. Provenance is **digest + locator**, and a spec that ships only the digest
produces artifacts that can prove they came from something nobody can find.

## Scope — tier 1 only

| tier | what | this spec? |
|---|---|---|
| **1** | `source_sha256` + `source_path` in the `--json` audit report | **yes** |
| 2 | a standalone `source_hash` in `crustyimg.build.lock` | deferred — see Notes |
| 3 | embed provenance in output metadata (XMP) so it survives file movement | **no** — see Anti-goals |

## Acceptance Criteria

- [ ] **AC-1. The digest is of the source bytes AS READ FROM DISK** — before decode, before
      `auto-orient` bakes, before RAW preview extraction, before SVG rasterization. `Image::load`
      already does `std::fs::read` then `decode_path(path, &bytes)` (`src/image/mod.rs:187-192`),
      so the bytes are in hand and hashing is one pass over an in-memory buffer.
- [ ] **AC-2. The digest is STABLE ACROSS crustyimg VERSIONS.** This is the property that
      distinguishes it from the cache key and it must be tested, not assumed: the same input file
      must produce the same `source_sha256` under a different tool version, feature set, recipe
      and quality. **A test that only re-runs the same binary cannot see this** — vary the inputs
      that `compute_key` mixes in and assert the digest does *not* move.
- [ ] **AC-3. Output image bytes are unchanged.** Tier 1 is sidecar-only. Compare against
      `main`'s binary for every format. [[fixtures-from-the-code-under-test-cannot-fail]]
- [ ] **AC-4. RAW and SVG hash the CONTAINER, not the derived pixels.** A `.dng` whose pixels
      come from an embedded JPEG preview (`raw_preview`, DEC-055) records the digest of the
      `.dng`. Same for a rasterized `.svg`. The recorded identity is *the file the user gave us*.
- [ ] **AC-5. A truncated JPEG hashes the truncated bytes.** `Image::from_bytes` sets
      `truncated_jpeg` and still decodes (SPEC-107 / DEC-085). The digest records what we were
      actually handed — that is the honest answer, and it must be asserted rather than left to
      chance.
- [ ] **AC-6. stdin is handled explicitly, not accidentally.** `-` has bytes but no path
      (`src/source/mod.rs:158`, DEC-088 tier 2). Emit `source_sha256` and a **null/absent**
      `source_path`, and pin that shape in a test — this is the case where digest-without-locator
      is unavoidable, and the schema should say so rather than emit an empty string.
- [ ] **AC-7. The schema version is bumped**, `optimize.explain/v1` → `/v2`, and the bump is
      asserted. The report is a published machine surface consumed by `--json`; adding fields
      silently would break the contract it exists to provide.
- [ ] **AC-8. Both new fields appear on all three verbs** that share `--json` — `optimize`,
      `web`, `apply` — each with its own test. One verb's report is not evidence about another's.
      [[a-guards-advertised-reach-is-a-claim]]
- [ ] **AC-9. Negative control:** revert the hashing call and confirm the AC-1 test goes RED;
      revert the `/v2` bump and confirm AC-7 goes RED. Prove each revert reached the built
      artifact. [[reverting-source-does-not-rebuild-the-binary]]
- [ ] **AC-10. Clean full matrix**, per-leg `CARGO_TARGET_DIR`s, run sequentially: default,
      `--no-default-features`, `--features webp-lossy`.

## Failing Tests

- **`tests/provenance.rs`** (new)
  - `"explain_report_carries_source_sha256_of_the_bytes_as_read"` — AC-1
  - `"source_digest_is_stable_across_version_features_recipe_and_quality"` — AC-2
  - `"output_image_bytes_are_unchanged_by_provenance_recording"` — AC-3
  - `"raw_and_svg_hash_the_container_not_the_derived_pixels"` — AC-4
  - `"truncated_jpeg_hashes_the_truncated_bytes"` — AC-5
  - `"stdin_emits_a_digest_with_an_absent_source_path"` — AC-6
  - `"explain_schema_is_v2"` — AC-7
  - `"optimize_web_and_apply_each_emit_both_fields"` — AC-8

## Anti-goals

- **Do NOT route this through C2PA** — and note the reason has *changed* since the spike, so do
  not repeat the spike's framing verbatim.

  `docs/research/c2pa-provenance-spike.md` (driven 2026-08-10/11 against 0.7.0, validated with an
  independent `c2patool` 0.27.9) found two defects. **One is fixed.** `SPEC-114 meta-lane-never-
  emits-a-broken-manifest` is **shipped** (`specs/done/`, `cycle: ship`, under the shipped
  `STAGE-044-metadata-lane-provenance-safety`); its goal is that *"`meta set`, `meta clean` and
  `meta copy` can never write a file carrying a manifest that no longer matches its own bytes"*,
  achieved by **detecting and dropping** the manifest with a warning. So the spike's
  *"`meta set --artist` is a manifest-forger"* line is **no longer true** and must not be quoted
  as current.

  The other observation stands and was never in SPEC-114's scope (which is `src/metadata/mod.rs`
  and the `meta` verbs): the **pixel lane drops manifests** — `web`, `optimize`, `convert`,
  `auto-orient` re-encode and nothing survives.

  **So the current state is: both lanes drop, neither forges.** That is coherent, and it is why
  C2PA is out of scope here — not because anything is broken, but because *preserving* a manifest
  across a re-encode would itself be forgery (the manifest is signed over bytes that no longer
  exist). The only correct C2PA behaviour for a transcoder is to **sign a new manifest asserting
  "derived from X"**, which needs a signing identity, a certificate chain, and `c2pa-rs`. That is
  a project, not a field in a JSON report. **This spec's sidecar digest is deliberately the
  unsigned, dependency-free 5% of that.**
- **Do NOT embed anything in the image** in this spec (tier 3). It changes the output bytes,
  which touches the reproducibility story — the lockfile's `hash` field is *recorded as observed*
  precisely because encoders are not byte-stable across environments (DEC-078, `lock.rs:31-36`).
  A provenance tag would be deterministic given the source, so it is not fatal — but it is a
  decision, not a freebie, and it belongs to its own spec with its own DEC.
- **Do NOT claim tamper-evidence.** An unsigned digest in a sidecar proves *association*, not
  authenticity — anyone can rewrite both. Signing is C2PA's job, which is the anti-goal above.

## Implementation Context

### Decisions that apply
- **DEC-058** — the cache key whose composite nature is the reason a separate field is needed.
- **DEC-059** — the lockfile this extends in tier 2.
- **DEC-078** — the determinism/pinning story that tier 3 would touch and tier 1 does not.
- **DEC-055** — RAW preview extraction, the reason AC-4 exists.

### Constraints that apply
- `metadata-not-via-pixel-encode` — tier 1 writes no metadata at all, so it is trivially
  satisfied; it becomes load-bearing the moment tier 3 is picked up.
- `every-public-fn-tested` — `hash_bytes` is already `pub` and tested; the new emitters are not.

### Notes on tier 2 (deferred, and it is NOT free)
`LockOutput` carries **`#[serde(deny_unknown_fields)]`** (`src/build/lock.rs:148-149`). Adding
`source_hash` therefore breaks both directions — new code cannot read an old lockfile, old code
cannot read a new one — so it requires a `SUPPORTED_LOCK_VERSION` bump and a migration story for
committed lockfiles. That is why tier 2 is split out rather than folded in: tier 1 is additive to
a versioned JSON report, tier 2 is a breaking change to a committed file.

## Notes for whoever picks this up

- **Cost is negligible and worth stating so nobody re-litigates it.** `build` already hashes the
  input for the cache key, so it is free there. For `optimize`/`web`/`apply`, `Image::load`
  already reads the whole file into memory, so this is one SHA-256 pass over a buffer that is
  already resident — not a second read.
- **The locator is the half that will get skipped.** Shipping `source_sha256` alone is easy and
  half-useless; `source_path` is what makes it answer "where did this come from." If a reviewer
  trims one field, it should not be that one.
