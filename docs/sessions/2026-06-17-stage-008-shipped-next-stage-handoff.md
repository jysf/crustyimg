# Handoff — STAGE-008 shipped; pick & start the next stage (fresh Opus session)

Paste the block below into a fresh **Opus** session in the `crustyimg` repo. The
modern-formats & quality stage (STAGE-008) just shipped end-to-end; this session
**confirms the next stage with the user, then designs/builds its first spec** under
the usual per-spec cycle. Process mechanics are unchanged — read them, don't relearn
their lessons (most are also in auto-memory).

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git repo,
remote `git@github.com:jysf/crustyimg.git`, `gh` authed). `main` is clean at
`7eb37da`.

## Orient (read first)
- `AGENTS.md` (esp. §4 cost, §5/§6 gates, §13 git/PR, §15 build cycle, §18 pointers);
  `projects/PROJ-001-crustyimg-mvp/brief.md`.
- `guidance/constraints.yaml`; `guidance/license-watchlist.yaml` (`just watchlist` —
  capabilities declined for license, with the way back); `guidance/questions.yaml`.
- Decisions `/decisions/` — recent: **DEC-016** (-q→quality), **DEC-017**,
  **DEC-018** (license gate), **DEC-019** (auto-quality search), **DEC-020** (AVIF),
  **DEC-021** (WebP lossless+decode default), **DEC-022** (lossy WebP / libwebp /
  first C dep), **DEC-023** (`--max-size` dimension fallback).
- The stage you're choosing among: `projects/PROJ-001-crustyimg-mvp/stages/` —
  STAGE-004 (compose & metadata), 005 (batch & recipes), 006 (hardening & security),
  007 (release & distribution) are all `proposed`; STAGE-001/002/003/008 are
  `shipped`. Read the **roadmap handoff** for the medium-term plan + the
  differentiator continuation: `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md`.
- The shipped quality/formats core you may extend: `src/quality/mod.rs`
  (`fit_under_size`/`SizeFit`, `auto_quality`/`auto_under_size`, `search_threshold`,
  the `LossyFormat` two-predicate seam), `src/sink/mod.rs` (`encode_to_bytes` per-format
  arms incl. AVIF/WebP, `ensure_codec_built`), `src/cli/mod.rs` (`run_shrink`/`run_convert`/
  `run_pixel_op`, `resolve_effective_quality` → `EncodePlan`).
- Run `just status` and `just specs-by-stage`.

## Where we are (2026-06-17)
**STAGE-008 "Modern Formats & Quality" SHIPPED** — 6 specs (SPEC-016→021), PRs
#18/#20/#21/#22/#23/#24:
- SPEC-016 perceptual auto-quality (`--target`/`--ssim`); SPEC-017 `--max-size` byte
  budget + the format-agnostic search; **SPEC-018** AVIF output (feature-gated,
  pure-Rust ravif); **SPEC-019** WebP lossless + decode (pure-Rust **default**);
  **SPEC-020** lossy WebP (feature-gated **libwebp** — the project's first C dep);
  **SPEC-021** `--max-size` dimension-reduction fallback (fits every format).
- The differentiator is real: *ask for an outcome (perceptual target or byte budget),
  get the smallest file that meets it, in a modern format.*
- Patterns established this stage (reuse them): the **feature-gated-native-codec
  pattern** (own DEC + an empirical design-time probe `build`/`just deny`/pinned API +
  a dedicated CI job — DEC-020/022); the **`LossyFormat` two-predicate split**
  (encode-only vs encode+decode); the **cross-sync contract** (the search probe must
  encode byte-identically to what the sink writes — covers JPEG/AVIF/WebP + resized
  candidates); the **license watchlist**.
- ~250+ tests; CI = 3-OS matrix + `avif` + `webp-lossy` + cargo-deny + cost-audit, all
  green. `image` default features now include `webp`; `avif` and `webp-lossy` are
  off-by-default features.

## Your immediate task: pick the next stage WITH the user, then start its first spec
Drive by status/priority, **not** stage number. The strategic fork (lay it out, then
recommend, then confirm before scaffolding):
- **A — Continue the differentiator** (builds directly on the just-shipped
  formats+quality core; the "why I'd switch" story): the **`optimize`** one-button
  command (auto-pick format + quality/size), responsive **`<picture>`/srcset** sets +
  blurhash, **`diff`** (the SSIMULACRA2 metric foundation already exists), and the
  **benchmark suite** (equal-quality via SSIMULACRA2). This may be a NEW stage (or
  PROJ-002) — see the roadmap handoff.
- **B — Complete MVP breadth** starting with **STAGE-004 (compose & metadata)**:
  `watermark`, `strip`, `clean --gps`, `set`, `copy-metadata` — the container-metadata
  lane (DEC-003) + compositing. Well-scoped, lower-risk; rounds out single-image prep.
- (Then STAGE-005 batch & recipes, STAGE-006 hardening, STAGE-007 release.)

**Recommendation: confirm with the user** — A keeps momentum on the moat and reuses
everything just built; B is the safe breadth play. If they pick A and it's not an
existing stage, create the stage first. Then **scaffold + design the first spec**
(authored by you, Opus, with `## Failing Tests`), emit any DEC, **push design to
`main` before dispatching the build**, then build → verify → **pause for the user
before merge** → ship.

## Orchestration model + this-repo gotchas (do not relearn — also in auto-memory)
- Per spec: design (Opus — you) → build (Sonnet 4.6 prescriptive prompt, or
  orchestrator-direct) → verify (independent `/code-review` or focused Explore
  subagents, read-only) → ship. **Pause for the user before every merge/ship.** Push
  design+DECs to `main` before the build so the build PR doesn't fold the design in.
- **Background subagents can't get Bash here** — run build/verify in the main loop (the
  sanctioned fallback used all of STAGE-008), or a FOREGROUND agent. Read-only
  `/code-review`/Explore finders work as subagents.
- **Cost capture is enforced** (CI `cost-data` / `just cost-audit`): a SHIPPED spec
  needs a **positive `tokens_total` on its build AND verify cycles**. Since build/verify
  run in the main loop here, record **labeled order-of-magnitude estimates** (Opus
  $5/$25 per MTok, ~80/20 in/out); design/ship stay `null`. Compute `cost.totals`
  (sum non-null, `0` for null) + `session_count: 4`. `just cost-audit` must pass.
- **Ship bookkeeping by hand on `main`** (the helpers mis-glob): flip `task.cycle`,
  fill cost sessions + `## Reflection (Ship)`, mark the timeline `[x]`, `git mv` the
  spec + timeline to `specs/done/`, update the stage backlog/Count, `brag add … -T
  crustyimg -p crustyimg -k shipped -i claude-opus-4-8`. **Gotcha:** do the file EDITS
  first, then `git mv`, then `git add` only the NEW (`done/`) paths — adding the old
  paths aborts the `git add` and the edits get left unstaged.
- **Merge dance** (branch protection: require-up-to-date ON, auto-merge OFF): if behind
  `main`, `gh pr update-branch <N>` → wait green → `gh pr merge <N> --squash
  --delete-branch`. A branch branched off current `main` (behind 0) squashes cleanly.
- **`cargo fmt` gotcha** (cost a CI round-trip on SPEC-021): a global `cargo fmt`
  reformats files you may have already committed; re-add EVERY file it touched (or
  `git add -u`) before committing — local `cargo fmt --check` passes (working tree) but
  CI checks the committed code.
- **`just deny`** (license gate, DEC-018, all-features) must pass. AVIF needs the scoped
  `libfuzzer-sys`/NCSA exception (already in `deny.toml`); WebP/libwebp needed none.
  When you decline a dep for its license, add it to `guidance/license-watchlist.yaml`.
- **Verify the working-tree branch before every commit** (`git branch --show-current`);
  an unrelated `reports/daily|weekly/*.md` may appear untracked — don't commit it.
- Build prompts: confirm every test in the spec's `## Failing Tests` exists; commit
  incrementally; derive `Debug` on new public types; verify the metric/decode path in
  design (not just build/license) for any new codec.

Start by confirming state (`just status` / `just specs-by-stage`), then present the
next-stage fork to the user with your recommendation. On their go, scaffold + design
the first spec. **Pause for the user before merging.**
