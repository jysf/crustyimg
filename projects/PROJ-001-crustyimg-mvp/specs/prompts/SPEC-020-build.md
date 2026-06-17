# SPEC-020 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect. The spec file is your only context. This
> prompt is deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-020 ("Lossy WebP behind a feature-gated libwebp
codec"). You are NOT the architect; the spec is your source of truth. This adds an
OFF-BY-DEFAULT `webp-lossy` cargo feature that wires the `webp` crate (libwebp,
VENDORED, built with cc) so a `--features webp-lossy` build encodes LOSSY WebP and the
DEFAULT build keeps WebP at LOSSLESS only (SPEC-019). NO default-build behavior change,
NO new exception in deny.toml, NO use of the webp crate's `image` feature (feed it raw
to_rgba8() bytes), NO animated WebP. WebP output NEVER exits 4 (lossless is always
available). Use ABSOLUTE paths.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-020-lossy-webp-behind-a-feature-gated-libwebp
Confirm `git branch --show-current` shows that branch before committing.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 gates (clippy `cargo clippy --all-targets -- -D warnings`; the gate set
   also includes `cargo deny check licenses` via `just deny`), §11 conventions, §12
   testing, §13 git/PR, §15 build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-020-lossy-webp-behind-a-feature-gated-libwebp.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", "## Notes" EXACTLY.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-022-lossy-webp-feature-gated-libwebp.md
   — the governing decision (libwebp behind `webp-lossy`; first C dep, opt-in;
   lossy-iff-quality; BOTH LossyFormat predicates for WebP). Already on main — no new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-021-*.md
   (WebP foundation) + DEC-020-*.md (AVIF — the cross-sync contract + the
   perceptual-needs-a-decoder finding WebP satisfies) + DEC-004-*.md (feature-gating).
5. The SHIPPED code you change (read the real signatures):
   src/sink/mod.rs   — `encode_to_bytes` (the JPEG + AVIF arms are the pattern; add the
                       WebP lossy arm: lossy iff `quality.is_some()` AND the feature,
                       else fall through to the existing lossless `write_to`),
                       the `AVIF_*` consts (pattern for `WEBP_DEFAULT_QUALITY`).
   src/quality/mod.rs— `encode_candidate_bytes` (AVIF arm is the pattern; add the
                       cfg(webp-lossy) WebP arm, IDENTICAL to the sink lossy encode),
                       `supports_lossy_quality` AND `supports_perceptual_quality`
                       (add WebP under cfg(webp-lossy) to BOTH — WebP has a decoder).
   src/cli/mod.rs    — `resolve_effective_quality` (NO structural change; confirm the
                       predicates admit WebP for both searches under the feature).
   Cargo.toml [dependencies]/[features], .github/workflows/ci.yml, docs/api-contract.md.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. Cargo.toml: add `webp = { version = "=0.3.1", optional = true, default-features =
   false }` to `[dependencies]`, and `webp-lossy = ["dep:webp"]` to `[features]`. Then
   `cargo build --features webp-lossy` (compiles VENDORED libwebp via cc — verified in
   design) and `just deny` (MUST stay green with NO new exception — do NOT touch deny.toml).
B. src/sink/mod.rs: in `encode_to_bytes`, add a WebP branch BEFORE the default write_to:
     #[cfg(feature = "webp-lossy")]
     if format == ImageFormat::WebP {
         if let Some(q) = quality {
             let rgba = img.pixels().to_rgba8();
             let (w, h) = rgba.dimensions();
             let enc = ::webp::Encoder::from_rgba(rgba.as_raw(), w, h);
             return Ok(enc.encode(q.clamp(1, 100) as f32).to_vec());
         }
     }
   With no feature OR quality == None → the existing `write_to` path writes LOSSLESS
   WebP (SPEC-019). Add `const WEBP_DEFAULT_QUALITY: u8 = 80;` (parity with AVIF; used
   only where a default is needed). NO CodecNotBuilt for WebP.
C. src/quality/mod.rs: `encode_candidate_bytes` — add `#[cfg(feature = "webp-lossy")]
   ImageFormat::WebP => { let rgba = reference.to_rgba8(); ... ::webp::Encoder::from_rgba
   (rgba.as_raw(), w, h).encode(quality.clamp(1,100) as f32).to_vec() ... }` IDENTICAL
   to the sink encode (cross-sync). `supports_lossy_quality` AND
   `supports_perceptual_quality`: under `#[cfg(feature = "webp-lossy")]` include
   `ImageFormat::WebP` in BOTH `matches!`.
D. src/cli/mod.rs: confirm (no code change expected) that with the feature,
   `convert --format webp -q Q` threads Q to the lossy encode, and `--target`/`--ssim`/
   `--max-size` drive WebP via the predicates.
E. .github/workflows/ci.yml: add a `webp-lossy` job (single ubuntu runner, mirror the
   `avif` job): `cargo build --features webp-lossy`, `cargo test --features webp-lossy`,
   `cargo clippy --all-targets --features webp-lossy -- -D warnings`. No extra step (cc
   is preinstalled on ubuntu runners).
F. docs/api-contract.md: WebP entry — with `--features webp-lossy`, `-q`/`--target`/
   `--ssim`/`--max-size` produce/tune LOSSY WebP; a bare `convert --format webp` stays
   lossless; without the feature WebP is lossless only.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (per the spec's ## Failing Tests)
═══════════════════════════════════════════════════════════════════════════
DEFAULT build: confirm `webp_quality_is_ignored` (SPEC-019) still passes (WebP lossless,
`-q` ignored without the feature) — do not break it.
FEATURE build (#[cfg(feature = "webp-lossy")]; run by the CI job + `cargo test --features
webp-lossy`); verify WebP via `image::guess_format == WebP`:
  - src/sink: encode_webp_lossy_respects_quality (q30 bytes < q90 bytes; both guess WebP)
  - src/quality: webp_supports_lossy_and_perceptual (both predicates true);
    auto_under_size_webp_is_monotone; auto_quality_webp_succeeds (perceptual round-trip
    decodes WebP and scores — proves the AVIF contrast)
  - tests/cli.rs: convert_to_lossy_webp_is_smaller (convert detailed png --format webp
    -q 20 → WebP, bytes < common::webp_lossless of the same source); webp_target_high
    (shrink <png> --target high -o out.webp → exit 0, WebP, NO decoder-fallback warning);
    webp_max_size_fits (convert detailed png --format webp --max-size 4KB → WebP, ≤ 4000)
The DEFAULT suite + all 5 gates MUST stay green; NO default-build behavior change.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- Animated WebP / WebP tuning (method/effort) knobs. - The webp crate's `image` feature
  (avoid a second image crate — use from_rgba on to_rgba8() bytes). - A system (non-
  vendored) libwebp link. - Any deny.toml change. - Making WebP lossy by DEFAULT (a bare
  `convert --format webp` stays lossless). - A new Operation or a new DEC. If you think
  you need one, STOP and add a question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES — run BOTH the default and the feature build
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  just deny                                       # green, NO new exception
  cargo build --features webp-lossy               # vendored libwebp via cc
  cargo test --features webp-lossy
  cargo clippy --all-targets --features webp-lossy -- -D warnings

Commit INCREMENTALLY (dep + feature + deny green → sink lossy arm → quality arm +
predicates → feature tests → CI job + docs). A green committed checkpoint must survive
an interruption.

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and confirm EACH named test
exists and runs (default under `cargo test`, feature under `cargo test --features
webp-lossy`) — list them and check each off. Confirm `just deny` is green with NO new
exception and the default build is byte-unchanged (WebP still lossless without the feature).

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
       recorded_at: 2026-06-17
       notes: "<one line>"
   Do NOT invent token numbers (the orchestrator records the real values at ship; if you
   run interactively, `/cost`).
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`. Do NOT run
   `just advance-cycle`/`archive-spec`.
4. Mark the timeline build line `[x]` (…/specs/SPEC-020-…-timeline.md) — "PR #N opened"
   (real number), never "merged".
5. Commit (Conventional Commits, e.g. `feat(sink): lossy WebP behind the webp-lossy
   feature (SPEC-020)`); end EACH commit with
   `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push + open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-020; Decisions referenced [DEC-022 (lossy WebP via libwebp,
   feature-gated), DEC-021 (WebP foundation), DEC-020 (cross-sync / AVIF contrast),
   DEC-004 (feature-gate), DEC-018 (license gate), DEC-019 (search reused), DEC-016
   (-q→quality), DEC-015 (format precedence), DEC-012/007]; Constraints checked (one-line
   evidence each, incl. `no-agpl-default-deps` ✅ — `just deny` green with NO new
   exception, `single-image-library` ✅ — webp is an encode-only sink fed raw bytes,
   `pure-rust-codecs-default` ✅ — default build unchanged, C dep is opt-in); New
   decisions: "No new DEC during build — DEC-022 already governs". End with the Claude
   Code footer.
```
