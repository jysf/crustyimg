---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-093
  type: decision
  confidence: 0.85
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

created_at: 2026-08-16
supersedes: null
superseded_by: null

affected_scope:
  - "src/image/mod.rs"
  - "src/cli/ops.rs"
  - "src/cli/optimize.rs"
  - "src/cli/build.rs"
  - "src/lint/rules.rs"
  - "src/lint/mod.rs"
  - "tests/animated_inputs.rs"
  - "tests/lint.rs"
  - "tests/common/mod.rs"
  - "docs/api-contract.md"

tags:
  - animated-input
  - gif
  - apng
  - webp
  - exit-codes
  - untrusted-input-hardening
  - lint
---

# DEC-093: animated input warns and proceeds; `lint --max-warnings 0` is the strict gate

## Decision

An animated GIF, APNG, or animated WebP fed to `convert`, `optimize`, `web`, or `build`
prints a one-line `warning: <input>: animated input flattened to a single frame — …` to
stderr, naming the input and saying frames were discarded, but **still exits 0** and
still writes the first frame as output. This is DEC-085's pattern (a per-decode
degradation flag on `Image`, turned into a stderr warning by the CLI layer)
applied to a second, distinct input class — a sibling decision, not a reopening of it.

The warning is printed **unconditionally** — not gated behind `--quiet` — matching
DEC-085's own reasoning: the shipped bytes are silently incomplete (three-quarters of
an animation discarded and reported as a size win), not merely a different size or
shape a script could notice by inspecting its own output.

Detection covers the three formats `image` 0.25.10's `AnimationDecoder` trait is
implemented for in this repo's enabled feature set — GIF, APNG, and animated WebP
(`grep -rn "impl.*AnimationDecoder.*for" image-0.25.10/src/` — `codecs/gif.rs`,
`codecs/png.rs`, `codecs/webp/decoder.rs`). TIFF/ICO/BMP/JPEG have no such impl.

**The strict path — the answer to "warn and proceed still loses data" — already
existed and needed only to be widened, not invented.** `lint --max-warnings 0` already
exits non-zero on any `Warn`-severity finding (`src/lint/mod.rs`'s `Severity`,
`src/cli/report.rs`'s exit-code wiring). `format/animated-gif` already existed as
a `Warn` finding; this decision keeps it GIF-only (existing config/output unchanged)
and adds a sibling rule, `format/animated-input`, for APNG and animated WebP — a
separate id, not an alias, so no config migration is needed. A user who cannot
tolerate a silent flatten runs `lint --max-warnings 0` in CI; `convert`/`optimize`/
`web`/`build` remain the tool that does the work, not the gate.

The lint rules' `fix:` string no longer names `convert --format webp` — the exact
command that causes the loss the finding is warning about. Until crustyimg writes
animated output, there is no correct fix to recommend, so the finding says so
plainly instead.

## Why

**The finding (SPEC-119 design cycle, `docs/backlog.md`).** A 4-frame looping GIF run
through `convert --format webp`, `optimize`, or `web` produced a **static** WebP — zero
`ANMF` chunks, no `VP8X` — reported as "72% smaller · ssim 100.0". The size win is real
only because three-quarters of the content was discarded; the SSIMULACRA2 score
structurally cannot see the loss, since it compares decoded-source to output and both
are frame 1. And the tool's own `lint` had been recommending the destructive command.

**Two alternatives were considered and rejected**, on the same grounds DEC-085 already
established for a materially similar defect:

- **Refuse (typed error, non-zero exit).** Rejected: this is a **frozen exit-code
  surface** (STAGE-030). Turning three shipped verbs that exit 0 today into failures
  would break existing PROJ-007 lockfiles and any pipeline processing a directory that
  happens to contain an animated file the user is fine flattening. Exit 4 ("unsupported
  or undetectable format", `docs/api-contract.md`) would also be the wrong code: an
  animated GIF is recognised, decodable, and names no feature to rebuild with — none of
  exit 4's three senses apply.
- **Warn-and-proceed with no strict alternative at all.** Rejected on its own: warning
  and proceeding still destroys the animation, it just narrates the destruction — a
  script piping `convert` over a directory without reading stderr loses frames exactly
  as before. What answers this is not a second warn-vs-refuse toggle on the pixel
  verbs, but recognizing `lint`'s existing severity/exit-code machinery as the CI-facing
  strict path, and making sure it actually covers what the pixel verbs' warning covers
  (all three formats, not just GIF) rather than leaving APNG/animated-WebP users with
  no strict option.

**Why a separate `format/animated-input` rule id instead of broadening
`format/animated-gif` with an alias.** Both were legitimate build-cycle choices (the
spec left it open). A broadened rule with `format/animated-gif` kept as an alias would
require the config layer (`known_rule_ids`, `select`/`ignore`/`severity` resolution) to
carry alias-resolution machinery that does not exist today, for a benefit — one fewer
struct — that does not offset the risk of a config-migration mistake on a stability
surface (DEC-050). Two sibling rules, each single-format, need zero changes to the
config layer beyond appending one more id to the catalog: `format/animated-gif` keeps
firing exactly as before for GIF (existing `.crustyimg-lint.toml` entries and `--json`/
`--sarif` output are unaffected), and `format/animated-input` is purely additive.

## Consequences

- **Positive:** no shipped verb silently discards animation frames without saying so,
  and the tool's own `lint` no longer recommends the command that does it. A user who
  needs a hard stop on animated input has a documented, driven way to get one
  (`lint --max-warnings 0`) without the pixel verbs' exit-code surface changing.
- **Negative:** identical to DEC-085's — a script that greps stderr for `error:` (not
  `warning:`) and does not also check for a non-empty/matching stderr will not notice.
  This is the intended, minimal-surprise behavior on a frozen exit-code surface; the
  strict path exists precisely for callers who need more than that.
- **Neutral:** `Image` gained one more `bool` field (`animated_input`) and one more
  `pub(crate)` accessor (`is_animated_input`), mirroring `truncated_jpeg`/
  `is_truncated_jpeg` exactly. `optimize_decide_one`'s return tuple grew a 5th element
  (now `#[allow(clippy::type_complexity)]`, matching the existing
  `#[allow(clippy::too_many_arguments)]` precedent elsewhere in the same file) rather
  than printing internally, preserving its "does NOT print" contract.
- **Found, not fixed, during this build:** `src/source/mod.rs`'s `IMAGE_EXTENSIONS`
  allow-list (used by directory/glob discovery for every command, including `lint`)
  does not include `webp`, even though WebP decode is a default-feature, fully
  supported input format. A directory containing only `.webp` files silently resolves
  to zero inputs under `lint <dir>` (and under any other command's directory/glob
  argument). Out of scope here — fixing it changes discovery behavior for every
  command, not just the two rules this decision adds — but it should be filed as its
  own item; `tests/lint.rs`'s SPEC-119 tests work around it by linting the `.webp`
  fixture by its own single-file path (never extension-filtered) rather than by
  directory.
- **AVIF is proven safe for the major-brand-`avis` case, not for AVIF sequences in
  general.** `avif_parse::read_avif` (2.1.0), `src/lib.rs:742-761`, keys the rejection
  on `ftyp` major brand `avis` specifically: that brand is refused with a typed
  `Error::Unsupported("Animated AVIF is not supported. Please use real AV1 videos
  instead.")` **before** any pixel decode — confirmed by reading `read_avif`'s body,
  not inferred. `Image::from_bytes` therefore never constructs an `Image` from an
  `avis`-branded file at all; there is nothing to flatten and nothing this decision's
  warning needs to cover on that path. The same function's `_ => skip_box_content`
  arm means a file whose major brand is the ordinary still-image `avif` but that also
  carries an embedded image sequence (a `moov` box) is **not** rejected — `moov` is
  silently skipped and the file parses as a still. That shape is unproven, not
  covered by the guarantee above, and out of this decision's scope to close.
