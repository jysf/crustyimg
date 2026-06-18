# SPEC-024 build prompt — `responsive` <picture>/srcset generator

Start a **fresh session**. You are the IMPLEMENTER for SPEC-024 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-026. Make the
`## Failing Tests` pass with the smallest correct change.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-024-responsive-picture-srcset-set-generator.md`
   — especially `## Command surface (PINNED)`, `## Variant generation (PINNED)`,
   `## HTML snippet (PINNED)`, `## Failing Tests`, and `## Notes for the Implementer`.
2. `decisions/DEC-026-responsive-command-and-html-emission.md`.
3. `src/cli/mod.rs` — `run_convert` (up-front `ensure_codec_built` + forced format),
   `resolve_format`, `shrink_params`, `escape_json`, `DEFAULT_SHRINK_QUALITY`,
   `Commands`, `dispatch`.
4. `src/sink/mod.rs` — `extension_for_format`, `ensure_codec_built`, `safe_join`,
   `Sink::File`, `encode_to_bytes`.
5. `src/operation/mod.rs` — the resize `fit` mode (`fit W×BIG` → width W, no upscale).

## What to build (composition — no new dependency; do NOT touch src/quality)
- `Commands::Responsive { input, widths, out_dir, formats, no_snippet }` + a
  `dispatch` arm → `run_responsive`. (`-q`/`--quiet`/`--yes` are global.)
- Helpers (pure where possible, all unit-tested):
  - `parse_widths(&str) -> Result<Vec<u32>, CliError>` — split on `,`, trim, parse,
    reject empty/0/non-int (exit 2), sort ascending, dedup.
  - `parse_formats(Option<&str>, source_fmt) -> Result<Vec<ImageFormat>, CliError>` —
    default `[source_fmt]`; else `resolve_format` each (unknown → exit 4), order
    preserved.
  - `mime_for_format(ImageFormat) -> &'static str`.
  - `build_picture_html(...) -> String` — pure; single-format ⇒ `<img srcset>`,
    multi ⇒ `<picture>` + `<source>` per format + `<img>` fallback (last format,
    largest width, with width/height).
- `run_responsive`: `Image::load(input)` **once**; `ensure_codec_built` each format
  up front (single exit 4); `create_dir_all(out_dir)`; surviving widths = requested
  `≤ source_width` (skip larger with a stderr warning unless `--quiet`; if none
  survive → exit 2); for each width × format: resize via the `fit` op
  (`mode=fit,width=W,height=1_000_000`) on a clone, read actual `width()/height()`,
  `encode_to_bytes`, write to `safe_join(out_dir, "{stem}-{w}w.{ext}")` (overwrite
  guarded by `--yes`); dedup by actual width; print `build_picture_html(...)` to
  stdout unless `--no-snippet`.
- `docs/api-contract.md` already has the `responsive` entry (added at design).

## Tests — every one in the spec's `## Failing Tests` must exist and pass
- 6 unit tests in `src/cli/mod.rs`; 8 integration in `tests/cli.rs` (use
  `common::detailed_jpeg`). Gate the AVIF-without-feature test with
  `#[cfg(not(feature = "avif"))]` like `convert_avif_without_feature_exits_4`.
- Add `"responsive"` to BOTH subcommand lists in `tests/cli.rs`.
- **Confirm each named test exists** before claiming green.

## Gates (all must pass)
```
cargo fmt            # then `git add -u`
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # no new dependency
```

## Git / PR
- Branch `feat/spec-024-responsive` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before each commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(cli): responsive <picture>/srcset set generator (SPEC-024)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-026, DEC-008, DEC-015,
  DEC-016, DEC-004 / Constraints / New decisions — DEC-026).
- Fill the spec's `## Build Completion` + the 3 reflection answers.

## Cost
Append a build session to `cost.sessions` (numerics null; orchestrator fills at ship):
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "responsive command: Commands::Responsive + run_responsive + parse_widths/parse_formats/mime/build_picture_html + tests; composition over resize fit + sink, no new dep"
```

## When done
`just advance-cycle SPEC-024 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
