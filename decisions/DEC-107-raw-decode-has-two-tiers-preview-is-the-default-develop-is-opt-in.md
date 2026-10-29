---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-107
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-016
repo:
  id: crustyimg

created_at: 2026-10-28
supersedes: null
superseded_by: null

# The decode fork, the adapter, and the enum this adds a variant to all live
# under src/image/. The cache key is where the two tiers must not collide, and
# `feature_signature` is the one line that keeps them apart — without this
# entry nothing surfaces this record when someone edits that allowlist.
affected_scope:
  - "src/image/**"
  - "src/build/cache.rs"
  - "src/cli/optimize.rs"
  - "Cargo.toml"

tags:
  - raw
  - input-reach
  - feature-gate
  - cache-key
  - source-container
  - external-library
---

# DEC-107: RAW decode has two tiers — the embedded preview is the default, development is opt-in

## Decision

**RAW input has two tiers, and the cheap one stays the default.**

- **Tier 1 — the embedded preview** (SPEC-061/DEC-055, shipped): scan the RAW
  container for its largest embedded full-res JPEG and decode that. Permissive,
  pure-Rust, patent-free, no new dependency. **Remains the default for every
  RAW extension, in every build.**
- **Tier 2 — development** (`raw-develop` feature, this project): hand the
  container to `irradiance`, which reads the sensor plane and develops it into
  real pixels. **Off by default**, native-only, and absent from the released
  binary unless someone asks for it at build time.

**The fork is at `Image::decode_path`, and Tier 2 falls back to Tier 1 on any
error.** With the feature compiled in, a RAW extension tries `raw_develop`
first; any `irradiance::Error` — a compressed plane, an unknown bit depth, a
malformed sensor, a camera whose geometry we cannot model — falls through to
`raw_preview`. A RAW file therefore **never** stops working because development
failed; it degrades to exactly the behaviour the default build has.

**This is deliberately NOT the `heic` pattern.** A `.heic` without the `heic`
feature is *detected and rejected* with `ImageError::CodecNotBuilt` (exit 4),
because there is no fallback — its thumbnail is HEVC too. RAW has a real
fallback, so the feature's absence is **silent and correct** rather than an
error. Do not copy `heic.rs`'s refusal arm here.

**The two tiers must not share a cache entry.** They produce different pixels
from byte-identical input, so `raw-develop` is registered in
`cache::feature_signature()`, and `irradiance::DEVELOP_VERSION` is folded into
`compute_key` as its own field. Both are load-bearing; see Consequences.

## Context

### Why this is a decision rather than an implementation detail

DEC-055 recorded Tier 1 and deferred Tier 2 on a cost argument (see the
amendment below). It did not say what the two tiers' *relationship* would be if
Tier 2 ever arrived. Three things had to be settled before SPEC-139 could be
written, and each had a plausible wrong answer:

- **Which tier is the default.** Development produces a better image, so the
  tempting answer is "develop when you can." Rejected: it would change the
  output bytes of every existing RAW pipeline the moment the feature was
  compiled in, and it would make a released binary's behaviour depend on a
  build flag most users never see.
- **What happens when development fails.** `irradiance` covers one camera at
  0.1.0. Erroring would make the feature *worse than not having it* for every
  camera it does not yet model.
- **Whether absence is an error.** `heic` says yes; this says no. The
  difference is the fallback, and stating it stops the next person reaching for
  the nearest precedent.

### Why the cache question is not optional

`cache::feature_signature()` is a **hand-maintained allowlist** — it enumerates
`avif`, `heic`, `webp-lossy` with `cfg!` checks. Its own doc comment states the
rule: *"Over-inclusion is a safe over-invalidation … Omitting one that does
affect bytes serves a stale artifact — so when in doubt, list it."*

A develop-enabled build and a default build produce **different pixels from the
same file**: same content hash, same recipe, same quality, different decode
path. Without the `feature_signature` registration, a cache populated by one
serves the other's entry. That is a silent wrong-image bug, not a staleness
annoyance, and it is one line to prevent.

`DEVELOP_VERSION` is the second half: it distinguishes *algorithm* changes
inside the feature. `irradiance` deliberately makes no byte-stability promise —
freezing the algorithm would be fatal for a developer whose job is to improve —
so `build --frozen` stays meaningful only if the version that produced the
bytes is part of the key. It goes in `compute_key` rather than the lockfile
because `LockOutput` is `deny_unknown_fields`, so a new lockfile field would
force a `SUPPORTED_LOCK_VERSION` bump and a migration for a field nobody reads
directly.

### Why an external library rather than in-house

`irradiance` is a permissive, pure-Rust, zero-runtime-dependency RAW develop
library built for this purpose, which crustyimg is the first consumer of. Under
DEC-088's triage this is the first branch — *is X a permissive Rust crate?* →
an ordinary dependency decision under DEC-018 — not a delegate, so tier 3's
objection does not apply. It participates in the lockfile and the cache key
like any other dependency.

The seam is **bytes in, pixels out, one direction only.** `irradiance` has no
`image` dependency and never will; crustyimg's adapter is the only place its
buffers become a `DynamicImage`. It is a **decoder**, not a transform service —
which is why hosting crustyimg's `.cube` LUT engine there was declined
(`single-image-library` is blocking, and a second pixel-processing library is
what that constraint exists to prevent).

## Alternatives Considered

- **Develop by default once the feature exists.** Rejected — see Context. The
  honest version of "better pixels" is an opt-in flag, not a silent change to
  every RAW pipeline's output.
- **A runtime flag (`--raw develop|preview`) instead of a compile-time
  feature.** Tempting, and probably right *later*. Rejected now because it puts
  `irradiance` in the default dependency graph and the released binary, which
  is a bigger commitment than the capability currently justifies at one
  supported camera. The compile-time gate is the reversible choice; a runtime
  flag can be added on top without undoing it.
- **Error when the feature is absent, like `heic`.** Rejected: RAW has a
  working fallback and HEIC does not. Copying the precedent would turn a
  working default into an error.
- **Error when development fails, rather than falling back.** Rejected for
  0.1.0-era coverage. ⚠ **Revisit this** — a silent fallback means a user who
  asked for development can get a preview and not know. The mitigation is that
  `SourceContainer` records which tier actually ran and `info` reports it, so
  the information is available rather than hidden. If that proves too quiet, a
  `-v` line or a `--raw-develop-strict` is the increment, not a reversal.
- **In-house the develop pipeline inside crustyimg.** Rejected: it is a
  multi-month effort with its own oracle, corpus and provenance discipline, and
  a general-purpose RAW library has consumers beyond crustyimg. Keeping it
  outside is also what keeps `single-image-library` honest.

## Consequences

- **Positive.** The default build and the 0.8.x release link zero `irradiance`
  and are byte-for-byte unchanged. A RAW file cannot regress: the worst case of
  a failed develop is today's behaviour. `build --frozen` stays meaningful
  across algorithm changes in a dependency that explicitly does not promise
  byte-stability.
- **Negative.** Two decode paths for one input family, so every RAW test
  matters twice — once per tier. A silent fallback can mask a develop failure
  from someone who asked for development (mitigated, not eliminated; see
  Alternatives). And the feature's final `Cargo.toml` line cannot be committed
  until `irradiance` 0.1.0 is on crates.io, because an optional `path`
  dependency without a `version` makes `cargo publish` fail for the whole
  crate.
- **Neutral.** `SourceContainer` gains a fifth variant. DEC-089 anticipated
  exactly this: its *"Revisit when: a fourth adopted-label family is added
  (another decoder with no matching `::image::ImageFormat` variant) — it must
  set `SourceContainer` too, not lean on `Native`'s default."* `RawDeveloped`
  is that family, and this record is the revisit.

## Validation

**Right if** a default build's RAW output is byte-identical to 0.8.x; a
develop-enabled build produces real sensor pixels on the Leica Q2 Monochrom
fixture; a camera `irradiance` cannot model still decodes via preview at exit
0; and a `DEVELOP_VERSION` bump produces a cache miss while an unrelated change
does not.

**Wrong if** the fallback turns out to hide failures people needed to see — in
which case the fix is a louder signal, not develop-by-default — or if the
compile-time gate proves so inconvenient that nobody enables the feature and it
rots. The second is the real risk, and it is a reason to revisit the runtime
flag at a second supported camera rather than a reason to skip the gate now.

## References

- `DEC-055` (amended by this record) — Tier 1, and the cost argument that
  deferred Tier 2
- `DEC-089` (extended by this record) — `SourceContainer`; its "Revisit when"
  clause names this case
- `DEC-088` — the external-integration triage; `irradiance` resolves on the
  first branch as an ordinary dependency
- `DEC-018` — the permissive-licence gate for adopting it
- `DEC-058` — the cache key's composition, which `DEVELOP_VERSION` joins
- `DEC-095` — ops preserve colour type and bit depth, which is why the adapter
  can emit 16-bit and have it survive the pipeline
- irradiance `PROJ-001 / STAGE-004` — the cross-repo framing this implements
- `SPEC-139`, `SPEC-140` — the two specs that build it
