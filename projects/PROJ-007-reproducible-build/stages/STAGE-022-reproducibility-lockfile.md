---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-022
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-09
shipped_at: null

value_contribution:
  advances: >
    Delivers the "verifiable" leg of the "Makefile for images, verifiable" thesis: a
    committed lockfile that pins a build, plus `build --check` / `--frozen` so CI (or a
    teammate) can assert the build hasn't drifted and review image outputs like code.
    STAGE-021 made the build incremental and correct *within a machine* (the robust half);
    this stage makes it *checkable* — pinning the robust inputs (the shipped cache key) and
    recording outputs as observed, so drift is a reviewable diff, not a silent surprise.
  delivers:
    - "First: an injective source→output guarantee — a build whose targets would collide two inputs to one output path is rejected before any write (the DEC-057 blocker; a lockfile is meaningless without it)"
    - "A committed lockfile (`crustyimg.build.lock`) pinning each output's cache key (inputs → identity) + its observed output hash + the env it was observed under"
    - "`build --check` — recompute and compare against the lockfile; exit non-zero on drift with a clear per-output diff (the CI gate)"
    - "`build --frozen` — refuse to update the lockfile at all (fail if it would change), for locked/offline CI"
    - "A recorded decision (DEC-059) fixing the lockfile format + what it pins vs records + the reproducibility policy (inputs pinned always; output-byte identity is env-scoped, opt-in strict)"
  explicitly_does_not:
    - "Guarantee cross-arch / cross-version output-byte identity — the lockfile pins inputs robustly and records outputs as observed-under-env; hard byte-identity is an opt-in strict mode, and pixel/perceptual (SSIMULACRA2, shipped) is the review-grade check, not encoder bytes"
    - "Watch files / `--watch` (STAGE-023)"
    - "Add a remote / networked lockfile or cache (local only)"
    - "Re-open the cache design (STAGE-021 / DEC-058) — the lockfile pins the cache key, it does not change it"
---

# STAGE-022: the reproducibility lockfile + `build --check` / `--frozen`

## What This Stage Is

The stage that makes a `crustyimg build` **checkable**. STAGE-020 gave us a declared build,
STAGE-021 made it incremental and correct within a machine; this stage lets you **commit a
lockfile** that pins the build and **assert it in CI**. Two moves. First — the prerequisite —
a build must map sources to outputs **injectively**: today two inputs that share a stem in one
target collide to one output path and race the rayon fan-out (DEC-057's blocker), and a
lockfile cannot pin a build whose output paths aren't unique, so this stage opens by rejecting
that at prepare time, before any write. Then the lockfile itself: a committed
`crustyimg.build.lock` recording, per output, the shipped **cache key** (the robust
"these exact inputs" identity from DEC-058) alongside the **observed output hash** and the
environment it was observed under. `build --check` recomputes and diffs against the lockfile —
the CI drift gate — and `build --frozen` refuses to update it at all. The framing that keeps
this honest: **inputs are pinned robustly; output bytes are recorded as observed, not
promised** — because encoder byte-identity holds within a machine (STAGE-021's experiment) but
is fragile across arch/version, so hard byte-identity is an opt-in strict mode and the
review-grade "did the image change" check is perceptual (SSIMULACRA2, already shipped), not raw
encoder bytes.

## Why Now

- **It completes the thesis.** "A Makefile for images, **verifiable**" is aspirational until a
  committed artifact lets CI assert "these sources + recipes + this tool version still produce
  these outputs." The build runs (STAGE-020) and is incremental (STAGE-021); this is the leg
  that makes "reviewed like code" real.
- **The prerequisite is now unavoidable and cheap.** DEC-057 recorded the non-injective
  source→output hazard as this stage's blocker three times over (the collision races the
  fan-out and over-counts). It must be resolved before a lockfile can pin anything, and it's a
  small, well-scoped fix at the prepare phase.
- **The determinism question is already answered enough to design against.** STAGE-021's
  encoder-determinism experiment settled the within-machine case (byte-identical run-to-run and
  across thread counts). That is exactly the boundary the lockfile must respect: pin what's
  robust (inputs / cache keys), record what's fragile (output bytes, env-scoped) — don't
  promise cross-arch byte-identity the encoders can't keep.

## Success Criteria

- A manifest whose resolved targets would write two inputs to the **same output path** is
  **rejected before any output is written**, with a typed error naming the collision (exit 2).
  A non-colliding build is unaffected.
- `crustyimg build` writes/updates a committed **`crustyimg.build.lock`** pinning, per output,
  the cache key + observed output hash + observed-env metadata.
- `build --check` exits **0** when the resolved build matches the lockfile and **non-zero
  (exit 7, DEC-025 "check not satisfied")** on drift, printing a per-output diff (added /
  removed / changed, and *what* changed — an input, the recipe, the tool version).
- `build --frozen` **fails** (exit ≠ 0) if the build would change the lockfile at all — the
  locked/offline CI mode — and is a no-op success when nothing would change.
- The reproducibility policy is explicit: **inputs pinned always; output-byte identity is
  env-scoped and opt-in strict**; the default `--check` asserts on the robust key and treats an
  output-hash mismatch under a *different* env as a warning/informational, not a hard failure
  (exact semantics fixed in DEC-059). Everything stays **local + no network**.
- Pure-Rust default preserved; `just deny` green (a lockfile is serde/toml — likely **no new
  dep**); the lean build unaffected.

## Scope

### In scope
- **SPEC-065 — injective source→output guarantee** (the unblocker): a prepare-phase check that
  rejects a build whose targets would map two inputs to the same output path, before any write.
  Typed `BuildError`, exit 2. Resolves DEC-057's blocker. **(framed here)**
- **SPEC-066 — the lockfile + `build --check` / `--frozen`** (the meat): the
  `crustyimg.build.lock` format (serde/toml), writing/updating it on a normal build, and the
  `--check` (drift gate, exit 7) / `--frozen` (locked, fail-on-change) modes; the reproducibility
  policy (pin inputs, record outputs env-scoped, opt-in strict byte-identity, perceptual as the
  review check). **DEC-059** at build. **(backlog — frame when picked up, after SPEC-065 lands)**

### Explicitly out of scope
- `--watch` (STAGE-023); a remote/networked lockfile or cache; re-opening the cache key/store
  (DEC-058 — the lockfile pins the key, doesn't change it); per-target format/quality overrides
  (STAGE-020 follow-up); GC/`--cache-dir` (STAGE-021 follow-up); the STAGE-021 `CACHE_ENTRY_MAX_BYTES`
  vs read-frame off-by-53 fix (fold into whichever spec next touches the store). Hard cross-arch
  byte-reproducibility is a *non-goal* — the lockfile is honest about env-scoping instead.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-065 (shipped on 2026-07-09) — injective source→output guarantee: reject, at `run_build`'s
  prepare phase (global across targets, before `Cache::open` / any write / any `.crustyimg/`), a build
  mapping two inputs to one output path; pure `find_output_collision` in `src/build`,
  `CliError::OutputCollision` → exit 2. Discharges DEC-057's blocker (its Validation now reads RESOLVED);
  no new dep, no new DEC. PR #71 (bc13c4d), 637 tests default + lean. Conservative on `{ext}` (over-detect,
  never under-detect); one disclosed literal-ext residual → DEC-059 threat model (below).
- [ ] SPEC-066 (design) — the reproducibility lockfile (`crustyimg.build.lock`) + `build --check`
  (drift gate, exit 7) + `build --frozen`/`--locked` + `--strict`; `src/build/lock.rs` (versioned TOML,
  one `[[output]]` per output {path, key=pinned cache key, hash=observed output bytes, bytes} + one
  `[env]`) + an env-aware `diff`. Pins the robust (DEC-058 key = inputs), records the fragile (output
  hash + env); cross-env byte variance informational unless `--strict`; perceptual stays the shipped
  `diff`. **No new dep.** DEC-059 at build. Framed 2026-07-09.

**Count:** 1 shipped / 1 active / 0 pending — SPEC-065 shipped (unblocker); SPEC-066 (lockfile) framed, build-ready.

## Design Notes

- **Why the injective fix is its own spec, and first.** DEC-057 records it as this stage's
  blocker: a target with two same-stem inputs under `{stem}.{ext}` maps both to one output path;
  with `Overwrite::Allow` and the rayon fan-out they *race* (nondeterministic winner) and the
  summary over-counts. A lockfile that pins `source→output` is meaningless if the mapping isn't a
  function. It's a small correctness fix with its own failing test, independent of the lockfile
  format, so it ships first (SPEC-065) and de-risks the stage.
- **The injective check's one real subtlety (SPEC-065).** The output filename expands a template
  over `{stem}`/`{name}`/`{parent}`/`{ext}` — and `{ext}` is the *output* format's extension,
  which crustyimg only knows **after decode** (`img.source_format()`; the cache sidesteps this by
  storing ext in the entry, DEC-058). The prepare-phase check runs *before* decode, so it can't
  compute the real `{ext}`. The safe resolution is **conservative over-detection**: treat two
  inputs as colliding if their `{stem}`/`{name}`/`{parent}` expansions match, regardless of
  `{ext}` — this guarantees injectivity of the real paths (they can differ only by ext, which we
  can't prove differs), at the cost of rejecting the rare "same stem, genuinely different output
  ext" build (the user disambiguates the template). **Under-detection is the unsafe direction**
  (two format-transforming inputs — `a/logo.png` + `b/logo.svg`, both → `logo.png` — is a *real*
  collision an input-extension proxy would miss), so prefer over-detection. A format-sniff
  refinement to cut false positives is a later option; correctness first. Check is **global across
  all targets** (two targets writing the same `out`/name collide too), at the end of prepare,
  before the store opens and before any write.
- **The lockfile design direction (SPEC-066 / DEC-059) — capture now, frame later.** Shaped by the
  design-feedback review (2026-07-08) and STAGE-021's experiment result:
  - **Pin the robust, record the fragile.** The lockfile pins each output's **cache key** (DEC-058's
    domain-separated digest of inputs = tool version + features + canonical recipe + quality +
    input ext + input content) — that's the reproducible "these exact inputs" identity. It
    **records** the observed **output hash** plus the **environment** it was observed under (arch,
    OS, tool version, feature set). Output bytes are *observed, not promised*.
  - **`--check` semantics (the honest gate).** Default `--check` fails (exit 7) on **input drift** —
    the resolved cache key no longer matches the pinned key (a source, recipe, quality, or tool
    version changed) — because that is unambiguous and reproducible. An **output-hash** mismatch
    *under the same env* is also a failure (a real regression); under a *different* env it is
    informational (expected encoder variance), unless **strict** mode is on. This is the split the
    review insisted on: don't fail CI on cross-arch encoder bytes.
  - **Strict / `reproducible` mode (opt-in).** A manifest/flag opt-in that promotes output-hash
    identity to a hard requirement — for shops on a pinned toolchain/arch that *want* byte-identity
    enforced. Off by default.
  - **Perceptual is the review-grade check.** For "did the image actually change," the answer is
    SSIMULACRA2 (already shipped, `diff`/DEC-025, exit 7) on decoded pixels — reuse it, don't
    reinvent an encoder-byte assertion that fragile encoders can't satisfy. A `--check --perceptual`
    (or similar) mode is a strong candidate; decide in DEC-059.
  - **`--frozen`** mirrors `cargo --locked`: fail if the lockfile would change at all (no
    regen/update), for offline/locked CI. Likely **no new dependency** (serde/toml shipped);
    confirm at build.
- **What the lockfile keys on.** Because DEC-058's cache key deliberately excludes the output
  *destination* (one entry → many destinations), the lockfile — which is about `source → output
  path` — must key its entries on the **output path** (now guaranteed unique by SPEC-065) and carry
  the cache key + output hash as the pinned/recorded values. SPEC-065's injectivity is exactly what
  makes the output path a valid primary key.
- **Hardening / conventions inherited:** the lockfile is committed config → size-guard + versioned
  + `deny_unknown_fields` like the manifest (DEC-057) and recipes (DEC-005); cwd-relative paths;
  typed errors, exit-code mapping only at the CLI boundary (DEC-007); DEC-025's exit 7 for a
  "check not satisfied" is the precedent for `--check`.
- **Carried from SPEC-065 (fold into SPEC-066 / DEC-059):**
  - **The literal-extension residual → DEC-059's threat model.** SPEC-065's collision check is
    conservative on the `{ext}` *token*, but a *literal*-extension template (`{stem}.png`) has no
    token to normalize, so a target naming `{stem}.png` and another `{stem}.{ext}` into one `out`
    can still map to the same real path undetected (reproduced: exit 0, "2 outputs", one file). It's
    inherent to a *pre-decode* check and needs an unusual mixed-template build. The lockfile is the
    natural second line of defense — it can refuse to pin two outputs to one real path — and a
    format-sniff would close both this gap and SPEC-065's `{ext}` false positives at once. Decide in
    DEC-059 whether the lockfile catches it structurally or a sniff is added.
  - **`exit_code_mapping_is_total` still omits `CliError::Cache`** (a pre-existing SPEC-064 gap the
    build note wrongly believed closed). A one-line test addition; SPEC-066 touches `src/cli` and
    should fold it in (through its PR, not a bare main edit).

## Dependencies

### Depends on
- STAGE-020 (`run_build` / `prepare_target` / the manifest, DEC-057) and STAGE-021 (the cache key
  `CacheKey` / `compute_key` / `recipe_hash`, DEC-058) — the lockfile pins that key; the injective
  fix lands in the prepare phase.
- DEC-025 (exit 7 "check not satisfied"), DEC-004/006/007, the `untrusted-input-hardening` posture,
  and the shipped SSIMULACRA2 `diff` (for the perceptual review mode).

### Enables
- STAGE-023 (`--watch`) — a watch loop re-runs affected targets against the same lockfile/cache.
- The "reviewed like code" CI story for anyone using crustyimg in a build step.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
