# SPEC-018 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect. The spec file is your only context. This
> prompt is deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-018 ("AVIF output behind a feature-gated ravif
codec"). You are NOT the architect; the spec is your source of truth. This adds an
OFF-BY-DEFAULT `avif` cargo feature that wires `ravif` (via `image/avif`) into the
Sink + the auto-quality search, so a `--features avif` build produces AVIF and the
DEFAULT build keeps AVIF output at exit 4 (DEC-004). NO default-build behavior
change, NO new top-level dep (the codec is the `image` crate's `avif` feature), NO
AVIF decode (output only), NO `--speed` knob (fixed speed v1). Use ABSOLUTE paths.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-018-avif-output-behind-a-feature-gated-ravif-codec
Confirm `git branch --show-current` shows that branch before committing.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 gates (clippy `cargo clippy --all-targets -- -D warnings`; the gate set
   also includes `cargo deny check licenses` via `just deny`), §11 conventions, §12
   testing, §13 git/PR, §15 build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-018-avif-output-behind-a-feature-gated-ravif-codec.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", "## Notes" EXACTLY.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-020-avif-output-feature-gated-ravif.md
   — the governing decision (adopt ravif feature-gated; the libfuzzer-sys/NCSA
   scoped deny exception; output-only v1; fixed speed). Already on main — no new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-*.md
   (codec policy / exit-4 feature gating) and DEC-018-*.md (the license gate).
5. The SHIPPED code you change (read the real signatures):
   src/sink/mod.rs   — `encode_to_bytes` (add the AVIF arm + the `not(feature)` exit-4
                       path), `format_from_extension`/`extension_for_format`,
                       `SinkError` (add `CodecNotBuilt`).
   src/quality/mod.rs— `encode_candidate_bytes` (add the cfg(avif) AVIF arm),
                       `LossyFormat::supports_lossy_quality` (cfg(avif) add Avif).
   src/cli/mod.rs    — `resolve_format`/`output_format_for`, `run_convert`,
                       `CliError::code()` + `exit_code_mapping_is_total`.
   Cargo.toml [features], deny.toml, .github/workflows/ci.yml, docs/api-contract.md.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. Cargo.toml: add `avif = ["image/avif"]` under `[features]`. Then
   `cargo build --features avif` (it builds PURE-RUST, no nasm — verified in design).
B. deny.toml: add to `exceptions` the SCOPED entry
   `{ name = "libfuzzer-sys", allow = ["NCSA"] }` with a comment (fuzz-only
   transitive of rav1e, permissive NCSA, NOT in the shipped --features avif binary).
   Do NOT add NCSA to the global `allow`. Then `just deny` must pass.
C. src/sink/mod.rs:
   - `format_from_extension`: add `"avif" => Ok(ImageFormat::Avif)`.
   - `SinkError::CodecNotBuilt { codec: &'static str, feature: &'static str }` with
     `#[error("{codec} support is not built; rebuild with --features {feature}")]`.
   - `encode_to_bytes`: AVIF branch — `#[cfg(feature="avif")]` encode via
     `::image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut cursor, AVIF_SPEED,
     quality.unwrap_or(AVIF_DEFAULT_QUALITY))` + `img.pixels().write_with_encoder(enc)`;
     `#[cfg(not(feature="avif"))]` → `Err(SinkError::CodecNotBuilt { codec: "avif",
     feature: "avif" })`. Consts `AVIF_SPEED: u8 = 6`, `AVIF_DEFAULT_QUALITY: u8 = 80`.
D. src/quality/mod.rs: `encode_candidate_bytes` — add `#[cfg(feature="avif")]
   ImageFormat::Avif => { ...AvifEncoder at the SAME AVIF_SPEED... }` (IDENTICAL to
   the sink's encode so probe size == written size); `supports_lossy_quality` —
   `#[cfg(feature="avif")]` include `ImageFormat::Avif`.
E. src/cli/mod.rs: ensure `--format avif` / `.avif` resolve (via format_from_extension);
   map `SinkError::CodecNotBuilt → 4` in `CliError::code()`; extend
   `exit_code_mapping_is_total`. `run_convert` already resolves format up front → a
   single exit 4 for an unbuilt codec.
F. .github/workflows/ci.yml: add an `avif` job (single ubuntu runner, mirror the
   cargo-deny job style): `cargo build --features avif`, `cargo test --features avif`,
   `cargo clippy --all-targets --features avif -- -D warnings`. NO nasm step.
G. docs/api-contract.md: update the `convert` entry per the spec.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (per the spec's ## Failing Tests)
═══════════════════════════════════════════════════════════════════════════
DEFAULT build (run always):
  - src/sink: format_from_extension_recognizes_avif
  - src/cli: exit_code_mapping_is_total extended for CodecNotBuilt → 4
  - tests/cli.rs: convert_avif_without_feature_exits_4 (exit 4; stderr mentions
    `avif` + `--features avif`)
FEATURE build (#[cfg(feature="avif")]; run by the CI avif job + locally with
`cargo test --features avif`); verify AVIF via `image::guess_format(&bytes) == Avif`
(NOT load_from_memory — decode isn't built):
  - src/sink: encode_avif_respects_quality (q30 bytes < q90 bytes; both guess Avif)
  - src/quality: avif_supports_lossy_quality; auto_under_size_avif_is_monotone
  - tests/cli.rs: convert_to_avif_produces_avif, shrink_to_avif_output,
    avif_target_high, avif_max_size_fits (≤ 4000 bytes)
The DEFAULT suite + all 5 gates MUST stay green; NO default-build behavior change.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- AVIF decode / .avif input (needs dav1d/avif-native, a C dep). - A `--speed` knob
  (fixed AVIF_SPEED v1). - WebP (SPEC-019) / the --max-size dimension fallback.
- A new top-level dependency (use `image/avif`). - Any default-build behavior change.
- A new Operation or a new DEC. If you think you need one, STOP and add a question
  to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES — run BOTH the default and the feature build
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  just deny                                       # must pass WITH the scoped exception
  cargo build --features avif                     # pure-Rust, no nasm
  cargo test --features avif
  cargo clippy --all-targets --features avif -- -D warnings

Commit INCREMENTALLY (feature + deny green → sink encode + CodecNotBuilt → quality
arm → CLI/exit-4 + default tests → feature tests + CI job). A green committed
checkpoint must survive an interruption.

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and confirm EACH named test
exists and runs (default ones under `cargo test`, feature ones under `cargo test
--features avif`) — list them and check each off. Confirm `just deny` is green and
the default build has NO behavior change (the existing suite is untouched).

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════
1. Fill ONLY the spec's `## Build Completion`.
2. Append a build cost session to `cost.sessions`:
     - cycle: build
       agent: claude-sonnet-4-6
       interface: claude-code
       tokens_total: null       # leave null — the orchestrator fills the real
       estimated_usd: null      # number from your Agent result at ship
       duration_minutes: null
       recorded_at: 2026-06-16
       notes: "<one line>"
   Do NOT invent token numbers (the orchestrator records the real tokens_total /
   duration / usd from your Agent result at ship; if you run interactively, `/cost`).
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`. Do NOT
   run `just advance-cycle`/`archive-spec`.
4. Mark the timeline build line `[x]` (…/specs/SPEC-018-…-timeline.md) — "PR #N
   opened" (real number), never "merged".
5. Commit (Conventional Commits, e.g. `feat(sink): AVIF output behind the avif
   feature (SPEC-018)`); end EACH commit with
   `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push + open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-018; Decisions referenced [DEC-020 (adopt ravif
   feature-gated + the deny exception), DEC-004 (feature-gate/exit-4), DEC-018 (the
   license gate), DEC-019 (the search reused), DEC-016 (-q→quality), DEC-015
   (format precedence), DEC-012/007]; Constraints checked (one-line evidence each,
   incl. `no-agpl-default-deps` ✅ — `just deny` green with the scoped exception,
   `pure-rust-codecs-default` ✅ — feature builds with no nasm); New decisions: "No
   new DEC during build — DEC-020 already governs". End with the Claude Code footer.
```
