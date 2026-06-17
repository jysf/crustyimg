# Handoff — build SPEC-018 (AVIF output), then continue STAGE-008

Paste the block below into a fresh **Opus** session in the `crustyimg` repo. The
design for SPEC-018 is DONE and pushed to `main`; this session's job is to **build
it** (then verify → pause for the user → ship), and then carry on with the
remaining STAGE-008 specs. Process mechanics are unchanged from the prior handoffs
— read them, don't relearn their lessons.

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git repo,
remote `git@github.com:jysf/crustyimg.git`, `gh` authed).

## Orient (read first)
- `AGENTS.md`; `projects/PROJ-001-crustyimg-mvp/brief.md`;
  `projects/PROJ-001-crustyimg-mvp/stages/STAGE-008-modern-formats-and-quality.md`;
  `docs/api-contract.md`; `guidance/constraints.yaml`.
- The shipped auto-quality core you're extending: `src/quality/mod.rs`
  (`search_threshold` generic core; `search_quality`/`auto_quality` perceptual,
  `search_under_size`/`auto_under_size` byte-budget; `encode_candidate_bytes`
  per-format encode; `LossyFormat::supports_lossy_quality`), `src/sink/mod.rs`
  (`encode_to_bytes`, `format_from_extension`, `SinkError`), `src/cli/mod.rs`
  (`run_shrink`/`run_convert`, `resolve_effective_quality`, `AutoQuality` enum,
  `parse_size`, `CliError::code()`).
- Decisions: **DEC-019** (perceptual+byte-budget search policy), **DEC-016**
  (`-q`→quality), **DEC-015** (format precedence/exit-6), **DEC-004** (codec policy
  / feature-gate + exit 4), **DEC-018** (license gate / `just deny`), and the NEW
  **DEC-020** (AVIF / ravif — the spec you're building).
- The two prior process handoffs (orchestration model + gotchas; strategy):
  `docs/sessions/2026-06-15-stage-003-continue-handoff.md` and
  `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md`.
- Run `just status` and `just specs-by-stage`.

## Where we are (2026-06-16, end of the prior session)
STAGE-001/002/003 shipped. **STAGE-008 "Modern Formats & Quality" is the ACTIVE
stage** — it runs in execution position right after STAGE-003 (it's the 4th wave of
work; **numeric id ≠ execution order** — STAGE-004–007 keep their ids and stay
`proposed`; drive by status/priority, not number). Within STAGE-008:
- **SPEC-016** (perceptual auto-quality, `shrink --target`/`--ssim`) — SHIPPED (PR #18, DEC-019).
- **SPEC-017** (`--max-size` byte budget on shrink/convert) — SHIPPED (PR #20). It
  also **generalized the search to be format-agnostic** (`LossyFormat` trait +
  per-format `encode_candidate_bytes` + format-agnostic entry points) — the seam
  AVIF/WebP plug into.
- **SPEC-018** (AVIF output, feature-gated) — **DESIGNED + pushed to `main`** (commit
  `8608c95`), **DEC-020 emitted**, build prompt ready. **NOT YET BUILT — this is your
  job.**
- Stage backlog: **2 shipped / 1 active (SPEC-018 design) / 2 pending** — the
  `--max-size` **dimension-reduction fallback** and **SPEC-019 WebP**.
`main` is clean; ~240+ tests, 3-OS CI + cost-capture + license jobs green.

## Your immediate task: BUILD SPEC-018
1. Build SPEC-018 strictly per its build prompt:
   `projects/PROJ-001-crustyimg-mvp/specs/prompts/SPEC-018-build.md`
   (the spec: `projects/PROJ-001-crustyimg-mvp/specs/SPEC-018-avif-output-behind-a-feature-gated-ravif-codec.md`).
   It adds an off-by-default `avif = ["image/avif"]` feature (→ `ravif`), a scoped
   `deny.toml` exception for the fuzz-only `libfuzzer-sys` (NCSA), the sink AVIF
   encode (`CodecNotBuilt` → exit 4 without the feature), the quality AVIF arm
   (auto-quality + `--max-size` drive AVIF for free), and a `--features avif` CI job.
   **Already verified in design** (do not re-litigate): `image/avif` builds
   **pure-Rust with no nasm**; the shipped tree is permissive; the only NCSA crate is
   the fuzz-only `libfuzzer-sys` (`cargo tree -e normal --features avif` shows it's
   not in the real build); `AvifEncoder::new_with_speed_quality(w, speed 1-10,
   quality 1-100)` is the API. Scope is **AVIF output only** (decode + `--speed`
   deferred), fixed `AVIF_SPEED = 6`.
2. Gate set is DOUBLE: run the 5 default gates AND
   `cargo build/test/clippy --all-targets --features avif`, plus `just deny` (must
   pass WITH the scoped exception). Verify AVIF output in tests via
   `image::guess_format(&bytes) == Avif` (decode isn't built).
3. Then **verify** (independent code-review, medium effort) and **pause for the user
   before merging**. Then merge + ship-bookkeeping on `main`.

## Orchestration model + this-repo gotchas (do not relearn)
- Per spec: design (Opus — you) → build (Sonnet 4.6 prescriptive prompt) → verify
  (Opus, read-only / `/code-review`) → ship. **Pause for the user before each
  merge/ship.** Design+specs+DECs commit to `main` and **push before dispatching the
  build** (so the build PR doesn't fold the design in).
- **Background subagents can't get Bash** in this environment — a background-dispatched
  build/verify agent fails doing zero work. So **run build/verify yourself in the
  main loop** (the sanctioned fallback, used for SPEC-016/017/018-prep), or try a
  FOREGROUND agent (the user can approve its Bash prompt). Read-only review can be
  delegated (the `/code-review` finders use Read/Grep).
- **Cost capture is now enforced** (CI `cost-data` job / `just cost-audit`, constraint
  `cost-captured-per-cycle`): a SHIPPED spec needs a **positive `tokens_total` on its
  build AND verify cycles**. Since build runs in the main loop, record a **labeled
  order-of-magnitude estimate** for build (no clean per-cycle metering — `/cost` if
  you can); record verify with the **real summed `subagent_tokens`** from the
  `/code-review` finder Agent results (SPEC-016/017 verify came to ~511k). design/ship
  stay `null` (main-loop). Compute `cost.totals` (sum non-null, `0` not null;
  `session_count: 4`). `just cost-audit` must pass before the spec is "shipped." See
  `docs/cost-tracking.md` + `projects/_templates/prompts/cost-snippet.md`.
- **Ship bookkeeping by hand on `main`** (the `just advance-cycle`/`archive-spec`
  helpers mis-glob): flip `task.cycle`, append cost sessions + the `## Reflection
  (Ship)`, mark the timeline verify+ship `[x]`, `git mv` the spec + timeline to
  `specs/done/`, update the STAGE-008 backlog line + Count, `brag add -t … -d … -T
  crustyimg -p crustyimg -k shipped -i claude-opus-4-8`.
- **Merge dance** (branch protection: require-up-to-date ON, auto-merge OFF): if the
  branch is behind `main`, `gh pr update-branch <N>` → wait for CI green → `gh pr
  merge <N> --squash --delete-branch`. A branch that merged `main` and is up-to-date
  (behind 0) squashes cleanly.
- **Verify the working-tree branch before every commit** (`git branch --show-current`).
- **`just deny`** (license gate, DEC-018, all-features) must pass; AVIF needs the
  scoped `libfuzzer-sys`/NCSA exception. No AGPL/GPL; LGPL/other only via a documented
  scoped exception.
- Build prompts: confirm every test in the spec's `## Failing Tests` exists; commit
  incrementally; derive `Debug` on new public types; the verify prompt greps each
  named test independently.

## After SPEC-018 ships — the remaining STAGE-008 backlog
Drive by status, not number. Recommended next: **SPEC-019 WebP output** (lossless
WebP via the pure-Rust `image`/`image-webp` backend, default-able; lossy WebP only
behind a feature-gated libwebp or deferred — decide in its DEC; verify the dep tree
+ `just deny` in design as you did for AVIF). Then the **`--max-size`
dimension-reduction fallback** (it's architecturally heavier — the output pixels
become budget-dependent, breaking the current quality-resolution model — so design
it carefully; reuses the shipped `Resize` op + the byte-budget search). After the
formats land: responsive `<picture>`/srcset sets + blurhash, the `optimize`
one-button command, `diff` (the metric foundation already exists), and the benchmark
suite (held at equal quality via SSIMULACRA2). See the roadmap handoff.

Start by confirming state (`just status`/`just specs-by-stage`), then BUILD SPEC-018
per its build prompt. Pause for the user before merging.
