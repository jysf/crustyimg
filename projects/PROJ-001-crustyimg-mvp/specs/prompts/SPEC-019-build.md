# SPEC-019 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect. The spec file is your only context. This
> prompt is deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-019 ("WebP lossless output + WebP decode"). You
are NOT the architect; the spec is your source of truth. This adds WebP as a
DEFAULT, pure-Rust format: `.webp` becomes a readable INPUT, and `convert --format
webp` / `-o x.webp` produce LOSSLESS WebP. NO new top-level dep (WebP is the `image`
crate's own `webp` feature), NO lossy encode (that is SPEC-020, libwebp), NO quality
knob (WebP is not a LossyFormat — `-q`/auto are ignored like PNG, DEC-016). Use
ABSOLUTE paths.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST
═══════════════════════════════════════════════════════════════════════════
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-019-webp-lossless-output-and-decode
Confirm `git branch --show-current` shows that branch before committing.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════
1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — §5/§6 gates (clippy `cargo clippy --all-targets -- -D warnings`; the gate set
   also includes `cargo deny check licenses` via `just deny`), §11 conventions, §12
   testing, §13 git/PR, §15 build-cycle rules.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-019-webp-lossless-output-and-decode.md
   — THE SPEC. Implement its "## Outputs", "## Failing Tests", "## Notes" EXACTLY.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-021-webp-lossless-default-decode-pure-rust.md
   — the governing decision (WebP lossless+decode pure-Rust DEFAULT; no quality knob;
   lossy deferred to SPEC-020). Already on main — no new DEC.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-*.md
   (codec policy / pure-Rust default) and DEC-016-*.md (-q ignored on lossless).
5. The SHIPPED code you change (read the real signatures):
   src/sink/mod.rs   — `format_from_extension` (add the webp arm),
                       `extension_for_format` (already returns "webp" — DO NOT change),
                       `encode_to_bytes` (the default write_to path already does
                       lossless WebP — DO NOT add a WebP arm).
   Cargo.toml [dependencies] image features, docs/api-contract.md,
   tests/cli.rs (the existing `convert_unbuilt_codec_exits_4`), tests/common/mod.rs.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact — follow the spec's ## Outputs)
═══════════════════════════════════════════════════════════════════════════
A. Cargo.toml: add `"webp"` to the `image` dep's `features = [...]` list (it becomes
   part of the DEFAULT build). It is the image crate's OWN feature — NO new
   top-level dep line. Then `cargo build` (pure-Rust, no nasm) and `just deny`
   (MUST stay green with NO new exception — do NOT touch deny.toml).
B. src/sink/mod.rs: `format_from_extension` — add `"webp" => Ok(ImageFormat::WebP)`.
   That is the ONLY src change. Do NOT add a WebP arm to `encode_to_bytes` (the
   existing `img.pixels().write_to(&mut cursor, format)` path encodes lossless WebP).
   Do NOT change `extension_for_format` (already returns "webp"). Do NOT change
   src/quality (WebP is not a LossyFormat) or src/cli (the lossless `-q`-ignored path
   already handles WebP) — confirm via tests, add no code there.
C. tests/common/mod.rs: add `pub fn webp_lossless(w: u32, h: u32) -> Vec<u8>` that
   encodes a small image to WebP via `DynamicImage::write_to(_, ImageFormat::WebP)`
   (reuse the module's `encode` helper). This is a FIXTURE, not a test.
D. docs/api-contract.md: document WebP as supported — readable INPUT (lossy +
   lossless) and lossless OUTPUT (`convert --format webp` / `-o x.webp`); `-q`/
   `--max-size`/`--target` ignored for WebP output (lossless, like PNG, DEC-016);
   note lossy WebP is `--features webp-lossy` (SPEC-020). Update the format table /
   the convert entry. (No CI change — WebP is in the default build.)

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (per the spec's ## Failing Tests) — all DEFAULT build
═══════════════════════════════════════════════════════════════════════════
  - src/sink: format_from_extension_recognizes_webp
  - tests/cli.rs: convert_to_webp_produces_lossless_webp (exit 0; guess_format==WebP;
    re-decode → dims match AND pixels EXACTLY equal the source — use a small solid/
    low-color image so equality is exact, proving losslessness)
  - tests/cli.rs: webp_input_decodes (write common::webp_lossless(..) to a .webp file;
    `convert <that.webp> --format png` → exit 0; output PNG decodes to source dims)
  - tests/cli.rs: shrink_to_webp_output (`shrink <jpg> -o out.webp` → exit 0; WebP)
  - tests/cli.rs: webp_quality_is_ignored (`convert <png> --format webp -q 50` →
    exit 0; WebP; -q ignored, DEC-016 — no error)
  - tests/cli.rs: EDIT `convert_unbuilt_codec_exits_4` — DROP the webp branch (webp
    now succeeds as a built default codec); KEEP the avif branch; update the doc comment.
The default suite + all 5 gates MUST stay green.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════
- Lossy WebP encode (SPEC-020 — needs libwebp, a C dep). - A WebP arm in
  encode_to_bytes or in src/quality (WebP has no quality knob here). - Any deny.toml
  change (no new exception needed). - A new Operation or a new DEC. If you think you
  need one, STOP and add a question to guidance/questions.yaml.

═══════════════════════════════════════════════════════════════════════════
THE GATES
═══════════════════════════════════════════════════════════════════════════
  cargo build
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  just deny                                       # MUST stay green, NO new exception

Commit INCREMENTALLY (feature + deny green → format_from_extension + unit test →
integration tests + drop the webp exit-4 branch → docs). A green committed
checkpoint must survive an interruption.

BEFORE YOU FINISH: re-read the spec's ## Failing Tests and confirm EACH named test
exists and runs — list them and check each off. Confirm `just deny` is green with NO
new exception and `convert --format webp` no longer exits 4 (it produces lossless WebP).

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
   Do NOT invent token numbers (the orchestrator records the real tokens_total /
   duration / usd from your Agent result at ship; if you run interactively, `/cost`).
3. Hand-edit the spec front-matter `task.cycle` from `design` to `verify`. Do NOT
   run `just advance-cycle`/`archive-spec`.
4. Mark the timeline build line `[x]` (…/specs/SPEC-019-…-timeline.md) — "PR #N
   opened" (real number), never "merged".
5. Commit (Conventional Commits, e.g. `feat(sink): WebP lossless output + decode
   (SPEC-019)`); end EACH commit with
   `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`.
6. Push + open a PR on `jysf/crustyimg` (§13 template): Summary; Spec metadata
   PROJ-001/STAGE-008/SPEC-019; Decisions referenced [DEC-021 (WebP lossless+decode
   pure-Rust default), DEC-004 (pure-Rust default), DEC-018 (license gate — green, no
   new exception), DEC-016 (-q ignored on lossless), DEC-015 (format precedence),
   DEC-002, DEC-012/007]; Constraints checked (one-line evidence each, incl.
   `no-agpl-default-deps` ✅ — `just deny` green with NO new exception,
   `pure-rust-codecs-default` ✅ — webp builds with no nasm/system libs,
   `single-image-library` ✅ — webp is the image crate's own backend); New decisions:
   "No new DEC during build — DEC-021 already governs". End with the Claude Code footer.
```
