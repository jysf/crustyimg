---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-013
  type: decision
  confidence: 0.85                   # honest: kamadak-exif is unambiguously the
                                     # right read-only EXIF crate (DEC-003 already
                                     # blessed it) and its pure-Rust, two-crate
                                     # dependency tree (mutate_once is the only
                                     # transitive dep, also pure-Rust, no build
                                     # script) makes "always-on" safe. The residual
                                     # uncertainty is purely about tag-display
                                     # ergonomics (display_value().with_unit()
                                     # formatting) and graceful handling of
                                     # malformed EXIF — not the choice of crate or
                                     # the always-on call.
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-15
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - Cargo.toml
  - src/cli/**

tags:
  - dependencies
  - metadata
  - exif
  - cli
  - read-only
---

# DEC-013: `kamadak-exif` for read-only EXIF tag reads, added always-on (not feature-gated)

## Decision

The `info --exif` tag dump (SPEC-009) reads EXIF via **`kamadak-exif`** (the
crate is published as `kamadak-exif` but imported as `exif`; pure-Rust, MIT/Apache),
added as a **normal, always-on top-level dependency** — `kamadak-exif = "=0.6.1"`
under `[dependencies]`, **not** behind a cargo feature. Reads go through
`exif::Reader::new().read_from_container(&mut Cursor::new(&bytes))` over the
**full input bytes** (the file contents for a path input, or the captured stdin
bytes), which locates the EXIF segment across JPEG/PNG/WebP/HEIF containers
internally. An image with **no EXIF** (`exif::Error::NotFound`) is **not an
error** — `info --exif` reports "no EXIF" / an empty tag array and exits 0. The
crate is **read-only**: it never writes tags. Tag writing remains the STAGE-004
container lane (`img-parts` / `little_exif`), explicitly out of scope here.

## Context

`docs/architecture.md`, AGENTS.md §5, and DEC-003 all pre-name `kamadak-exif`
as the read-only EXIF crate, but pre-naming is **not** a decision record. The
constraint `no-new-top-level-deps-without-decision` requires a DEC before any
new top-level crate lands in `Cargo.toml`; SPEC-009 (`info --exif`) is the spec
that actually adds `kamadak-exif`, so the DEC belongs here. This mirrors the
DEC-011 precedent (viuer was pre-named yet still got its own DEC when SPEC-005
added it).

The one real design call this DEC makes that DEC-011 did **not**: whether to
**feature-gate** the crate. DEC-011 gated `viuer` behind an off-by-default
`display` feature because viuer pulls a heavy transitive tree (sixel/kitty/iterm
backends) we did not want on the default multi-OS CI build. That reasoning does
**not** apply to `kamadak-exif`:

- **Verified pure-Rust, minimal tree.** `kamadak-exif 0.6.1` has exactly one
  runtime dependency, `mutate_once` (also pure-Rust, a leaf crate with no
  further deps), **no build dependencies, no native/system libraries, no build
  script**. The cost on the default `cargo build` / three-OS CI is negligible.
- **`info --exif` is core read-only inspection**, not an optional convenience.
  Gating it would mean the default binary's `info --exif` silently degrades,
  which is a worse user contract than viuer's (where the whole `view` command is
  an acknowledged convenience). EXIF reading is part of what `info` *is*.

So `kamadak-exif` is added **always-on**. (Confirmed on crates.io 2026-06-15:
latest is `0.6.1`; AGENTS.md §5 pre-named `0.6` — pinning `=0.6.1` is consistent.
Update AGENTS.md §5 if the pinned patch should be recorded there.)

## Alternatives Considered

- **Option A: feature-gate `kamadak-exif` behind an off-by-default feature (mirror DEC-011's viuer treatment)**
  - What it is: `kamadak-exif = { version = "=0.6.1", optional = true }`,
    `[features] exif = ["dep:kamadak-exif"]`, the read call `#[cfg(feature = "exif")]`.
  - Why rejected: the gate's only justification (DEC-011) is transitive-tree
    weight on the default CI build. `kamadak-exif` is pure-Rust with a single
    pure-Rust transitive dep and no build script — there is no CI cost to
    neutralize. Gating would add a second build configuration to keep green and
    would make the default binary's `info --exif` a degraded no-op, for zero
    benefit. Rejected.

- **Option B: `rexiv2` (native gexiv2 bindings) for EXIF reads**
  - What it is: use the `rexiv2`/`gexiv2` native library for tag reading.
  - Why rejected: a native/system dependency breaks the pure-Rust-by-default,
    trivial multi-OS CI invariant (DEC-004, `pure-rust-codecs-default`). DEC-004
    already relegated `rexiv2` to an off-by-default feature. Wrong tool for the
    read path. Rejected.

- **Option C: pull the EXIF write crates (`little_exif` / `img-parts`) now and read through them**
  - What it is: add the STAGE-004 write-lane crates early and read tags via them.
  - Why rejected: those are the **write** lane (STAGE-004, DEC-003). Adding them
    in a read-only stage pulls forward dependencies and surface area this spec
    does not need, and conflates read with write. SPEC-009 is read-only.
    Rejected.

- **Option D: read raw captured `MetadataBundle.exif` bytes via `read_raw` instead of the full container via `read_from_container`**
  - What it is: reuse the already-captured APP1 payload (`Image::metadata().exif`)
    and parse it with `exif::Reader::read_raw(Vec<u8>)`.
  - Why rejected (for the headline path): the captured bytes include the
    `Exif\0\0` signature, which `read_raw` does not expect (it wants raw TIFF), so
    they would need 6-byte trimming and the capture path only scans JPEG/PNG today
    — it would not cover WebP/HEIF. `read_from_container` over the full input bytes
    handles all containers uniformly and matches kamadak-exif's documented entry
    point. (The capture-based path stays available as a future optimization.)

- **Option E (chosen): `kamadak-exif` always-on, read-only, via `read_from_container` over the full input bytes**
  - What it is: `kamadak-exif = "=0.6.1"` in `[dependencies]` (ungated);
    `exif::Reader::new().read_from_container(&mut Cursor::new(&bytes))`;
    `Error::NotFound` → graceful "no EXIF", exit 0; tags formatted with
    `Field::display_value().with_unit(&exif)`.
  - Why selected: the de-facto pure-Rust EXIF reader, already blessed by DEC-003,
    with a negligible always-on cost, a uniform multi-container entry point, and a
    clean read-only boundary that leaves the STAGE-004 write lane untouched.

## Consequences

- **Positive:** `info --exif` reads EXIF tags read-only with no native deps and
  no CI cost; the default binary's `info --exif` works out of the box (no feature
  flag). Single, uniform code path across container formats via
  `read_from_container`. Read/write lanes stay cleanly separated (DEC-003): this
  spec adds only the read crate.
- **Negative:** One more always-on top-level crate (plus its single transitive
  `mutate_once`). EXIF tag-value formatting depends on kamadak-exif's
  `display_value()` rendering, which we surface as-is rather than re-formatting.
- **Neutral:** `read_from_container` re-reads the input bytes to locate the EXIF
  segment rather than reusing the already-captured `MetadataBundle.exif`; for the
  single-image `info` command this duplicate scan is immaterial. The
  capture-based `read_raw` path remains available if a future spec wants to avoid
  the re-read.

## Validation

Right if: `cargo build` / `cargo test` / `clippy` / `fmt` stay green on all three
OSes with `kamadak-exif` in the default graph and no native/system dependency
introduced; `info --exif <jpeg-with-exif>` exits 0 and reports EXIF present (and
any readable tags); `info --exif <plain-png>` exits 0 and reports "no EXIF"
(graceful, `Error::NotFound`); a malformed/partial EXIF segment degrades to
"no/none" without a panic. Revisit if: `kamadak-exif`'s tag-display output proves
too noisy/unstable to assert against (then introduce a thin value-formatting
layer), or if a future need to write tags makes consolidating on a single
read+write crate attractive (would be a superseding DEC alongside the STAGE-004
write-lane decision).

## References

- Related specs: SPEC-009 (`info` command — adds `kamadak-exif` here),
  SPEC-002 (captures raw EXIF/ICC bytes at load), STAGE-004 backlog (EXIF *write*
  lane — explicitly NOT this spec)
- Related decisions: DEC-003 (metadata dual-lane — blesses `kamadak-exif` as the
  read-only crate; parent rationale), DEC-004 (pure-Rust/trivial-default-build
  policy — why `rexiv2` is rejected for the read path), DEC-007 (typed errors at
  the binary boundary), DEC-011 (precedent: a one-crate add with a written DEC;
  this DEC diverges by adding always-on, not feature-gated)
- Constraints: `no-new-top-level-deps-without-decision` (this DEC satisfies it),
  `pure-rust-codecs-default` (kamadak-exif is pure-Rust), `untrusted-input-hardening`
  (malformed EXIF degrades gracefully, never panics)
- External docs: https://docs.rs/kamadak-exif/0.6.1 ,
  https://crates.io/crates/kamadak-exif
- Architecture: `docs/architecture.md` (pre-names kamadak-exif), AGENTS.md §5
