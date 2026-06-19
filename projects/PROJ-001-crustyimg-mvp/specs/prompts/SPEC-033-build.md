# SPEC-033 build prompt — decode resource limits on image load

Start a **fresh session**. You are the IMPLEMENTER for SPEC-033 in the `crustyimg`
repo (cwd is the repo root). The architect (Opus) wrote the spec, its failing tests,
and **DEC-034** (the limits policy). **No new dependency** — `image::Limits` is in the
already-vendored `image =0.25.10`. Make the spec's `## Failing Tests` pass with the
smallest correct change, then open a PR and STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-033-decode-resource-limits-on-image-load.md`
   — especially `## Limits policy (PINNED)`, `## Failing Tests`, `## Notes for the
   Implementer`.
2. `decisions/DEC-034` (the caps, reject-not-clamp, typed `LimitsExceeded`, exit 1),
   `decisions/DEC-002` (canonical load path), `decisions/DEC-007` (typed errors).
3. `src/image/mod.rs` — `decode_with_format` (the ONE choke point to harden; every
   load entry routes through it), the `#[cfg(test)]` fixtures. `src/error.rs` —
   `ImageError`. `src/cli/mod.rs` — `CliError::code()` image arms.

## What to build
- **`src/error.rs`:** add a typed variant
  `#[error("image exceeds decode limits: {0}")] LimitsExceeded(String)` to `ImageError`
  (distinct from `Decode`).
- **`src/image/mod.rs`:**
  - Module consts: `const MAX_IMAGE_DIMENSION: u32 = 65_535;` and
    `const MAX_ALLOC_BYTES: u64 = 512 * 1024 * 1024;`.
  - `fn decode_limits() -> ::image::Limits` — `let mut l = ::image::Limits::default();
    l.max_image_width = Some(MAX_IMAGE_DIMENSION); l.max_image_height =
    Some(MAX_IMAGE_DIMENSION); l.max_alloc = Some(MAX_ALLOC_BYTES); l`. (The struct is
    `#[non_exhaustive]` — build via `default()` + field assignment; a struct literal
    will NOT compile.)
  - `fn map_image_decode_error(e: ::image::ImageError) -> ImageError` — match:
    `::image::ImageError::Limits(_) => ImageError::LimitsExceeded(e.to_string())`,
    `_ => ImageError::Decode(e.to_string())`.
  - A test seam: `fn decode_with_limits(bytes: &[u8], limits: &::image::Limits) ->
    Result<(DynamicImage, ImageFormat)>` holding the current decode body but with
    `let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()
    .map_err(ImageError::Io)?;` then `let format = reader.format()
    .ok_or(ImageError::UnsupportedFormat)?;` then `reader.limits(limits.clone());`
    then `reader.decode().map_err(map_image_decode_error)?`. (`ImageReader::limits`
    takes `Limits` by value and `&mut self`; `Limits: Clone`.)
  - `decode_with_format(bytes)` becomes a thin wrapper: `decode_with_limits(bytes,
    &decode_limits())`.
- **`src/cli/mod.rs`:** in `CliError::code()`, add
  `CliError::Image(ImageError::LimitsExceeded(_)) => 1,` next to the other
  `CliError::Image(...)` arms.

## Hard rules
- **Smallest correct change.** Harden ONLY `decode_with_format` (one choke point) —
  do NOT touch `Image::{load, from_bytes, from_reader}` (they inherit it), the
  metadata-capture code, or the codec feature set. **No new dependency.**
- Implement DEC-034 caps EXACTLY (65 535 per dimension; 512 MiB alloc; reject, never
  clamp/downscale). No `unwrap`/`expect`/`panic!` on the new non-test paths (DEC-007).
- Native fixtures. The bomb fixture is a **real, cheap** oversized PNG —
  `RgbImage::new(70_000, 1)` encoded to PNG (~210 KB; the decoder rejects it at the
  IHDR dimension check before allocating). **No CRC forgery, no multi-GB buffer.**
- Every named test in `## Failing Tests` must EXIST and PASS:
  - `src/image/mod.rs` unit: `oversized_dimension_png_is_limits_exceeded`,
    `normal_image_decodes_under_production_limits`, `tiny_dimension_limit_rejects_via_seam`,
    `tiny_alloc_limit_rejects_via_seam` (use a 64×64 PNG so the decoded buffer exceeds
    the 16-byte cap), `map_limit_error_to_limits_exceeded` (construct via
    `::image::error::{LimitError, LimitErrorKind}` →
    `LimitError::from_kind(LimitErrorKind::DimensionError)`), `truncated_png_is_decode_not_limits`,
    `from_reader_is_also_limited`.
  - `src/cli/mod.rs` unit: `limits_exceeded_maps_to_exit_1`.
  - `src/error.rs` unit: `limits_exceeded_carries_message`.
  - integration (`tests/cli.rs` or `tests/image_load.rs`):
    `info_on_oversized_image_exits_1_not_panic` — write a `70_000×1` PNG to a tempdir,
    run the binary `info bomb.png`, assert exit **1** + non-empty stderr (no panic/hang).
- Run clippy right after writing doc comments (the SPEC-031 `doc_lazy_continuation`
  lesson).

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check licenses
```

## Git / PR
- Branch `feat/spec-033-decode-limits` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`.
- PR title: `feat(image): decode resource limits on load (SPEC-033)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-034, DEC-002, DEC-007 /
  Constraints checked — `untrusted-input-hardening` / New decisions — "DEC-034 at design").
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "decode resource limits: ImageError::LimitsExceeded + decode_limits()/map_image_decode_error + decode_with_limits seam on the one choke point (decode_with_format); caps 65535/512MiB per DEC-034; reject-not-clamp; exit 1; no new dep"
```

## When done
`just advance-cycle SPEC-033 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
