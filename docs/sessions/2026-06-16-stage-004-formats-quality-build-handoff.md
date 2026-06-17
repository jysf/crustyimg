# Handoff — scaffold STAGE-004 (Modern Formats & Quality) + design its first spec

Paste the block below into a new Opus session in the `crustyimg` repo. The path
is **decided** (user chose "Formats + auto-quality" on 2026-06-16). This session's
job: scaffold the new stage, then design its first spec (**perceptual
auto-quality**) following the orchestration model, and run it through
design→build→verify→ship.

**Process mechanics are unchanged** — read the two prior handoffs and don't
relearn their lessons:
- `docs/sessions/2026-06-15-stage-003-continue-handoff.md` — the full orchestration
  model + the hard-won gotchas (push design before build; check branch before
  commit; verify every named test exists; recover dropped subagents; branch-
  protection merge dance; `just advance-cycle`/`archive-spec` mis-glob → ship by
  hand).
- `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md` — the
  strategic synthesis (frontier scan, competitive gaps, the 6-month roadmap, the
  GIF design, the benchmarking plan, new feature ideas). Background; this doc is
  the concrete next step.

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git
repo, remote `git@github.com:jysf/crustyimg.git`, `gh` authed).

**Orient:** `AGENTS.md`; `projects/PROJ-001-crustyimg-mvp/brief.md`;
`docs/api-contract.md`; `guidance/constraints.yaml`; the shipped
`src/cli/mod.rs`, `src/operation/mod.rs`, `src/sink/mod.rs`; and the decisions —
esp. **DEC-016** (encode quality / the `-q` + `run_pixel_op` path the flagship
extends), **DEC-015** (format precedence + exit-6 fan-out), **DEC-004** (codec
policy — AVIF is feature-gated), **DEC-018** (permissive license policy +
cargo-deny gate), **DEC-014** (op-params), **DEC-003** (metadata lane). Read the
two prior handoffs above. Run `just status` / `just specs-by-stage`.

## Where we are (2026-06-16)

STAGE-001/002/003 SHIPPED — **15 specs (SPEC-001–015)**, 16 DECs (DEC-002…018),
207 tests, 3-OS CI green. `main` clean. Live: `view`, `info`, `resize`,
`thumbnail`, `shrink`, `convert`, `auto-orient`, `apply`. **License gate is now
live** (DEC-018): `cargo deny check licenses` runs in CI and via `just deny` —
**no AGPL/GPL default deps**; permissive only; LGPL only via a documented
exception (currently just `ansi_colours` via viuer/display).

## The decision

**STAGE-004 = Modern Formats & Quality** (user-chosen 2026-06-16 over: compose &
metadata [the original STAGE-004], animation/GIF, geometry/effects). This is the
strategic differentiator wave — "set the look, not the quality number" — and it
unblocks GIF, responsive sets, and the benchmark story later.

## License-shaped codec reality (verified 2026-06-16 — this drives the spec order)

Because the cargo-deny gate (DEC-018) now blocks copyleft, the codec options are
constrained — verified via lib.rs:
- **`ssimulacra2`** (perceptual metric): **BSD-2-Clause, pure-Rust**, deps
  num-traits/rayon/thiserror/yuvxyb (all permissive). ✅ The flagship's metric is
  unblocked.
- **`ravif`** (AVIF encode): **BSD-3-Clause, pure-Rust, LOSSY** ✅ — BUT it pulls
  `rav1e` (~18 MB, ~431k SLoC, slow encode), so keep AVIF **feature-gated** per
  DEC-004 (your `convert --format avif` already exits 4 without it).
- **`image-webp`** (the `image` crate's WebP backend): MIT/Apache but
  **LOSSLESS-encode only** (decodes lossy). The pure-Rust **lossy** WebP encoder
  (`zenwebp`) is **AGPL → blocked by our own gate**. Lossy WebP would need a
  **feature-gated libwebp** (C, BSD) — not pure-Rust-default.
- **Consequence:** the default binary has no permissive pure-Rust *lossy*
  modern-format encoder. So **the flagship (perceptual auto-quality) leads — it
  runs on the already-shipped default JPEG path** and needs only the permissive
  `ssimulacra2` dep. AVIF/WebP follow as feature/format additions.
- **Discipline:** every new top-level dep needs a DEC
  (`no-new-top-level-deps-without-decision`) AND a `just deny` run after adding to
  confirm the gate stays green (catches a copyleft transitive dep immediately).

## Stage backlog (recommended order — flagship first)

1. **SPEC-016 — perceptual auto-quality (FLAGSHIP, first).**
   `shrink --target visually-lossless` (and `--ssim <N>`): binary-search the JPEG
   encoder quality, decoding each candidate and scoring it against the original
   with **SSIMULACRA2** (`ssimulacra2` crate), stopping at the lowest quality whose
   score ≥ the target. Default `visually-lossless ≈ SSIMULACRA2 ≥ ~90` (ship as a
   **tunable constant**, not gospel). Cap iterations (~6–8), cache decodes.
   Builds directly on shipped `shrink` + DEC-016 (the `-q`/`run_pixel_op`/
   `encode_to_bytes` path). Default-buildable, fully permissive.
   - New dep: `ssimulacra2` (BSD-2) → **DEC-019** (adopt it + the metric/threshold/
     search-loop policy). Run `just deny` after adding.
   - Scope: JPEG target for v1 (the common case). It generalizes to AVIF/WebP once
     those land (the search loop is encoder-agnostic).
2. **SPEC-017 — `--max-size <KB>` byte budget.**
   `shrink`/`convert --max-size 200KB`: iteratively reduce quality (and, as a
   fallback, dimensions) until the encoded output is ≤ the budget. The other half
   of "tell me the outcome, not the knob" (size-target vs quality-target). Pure-
   Rust, default-buildable, reuses the SPEC-016 search machinery. Likely no new DEC.
3. **SPEC-018 — AVIF output (feature-gated `avif`).**
   Wire `ravif` behind a new off-by-default `avif` cargo feature so
   `--features avif` enables AVIF encode in `convert`/`shrink` (and the
   auto-quality loop), and without it the codec still exits 4 (current behavior,
   DEC-004). Expose a speed knob (rav1e speed 1–10). Add a `--features avif` CI
   job (mirror the deferred mozjpeg job note in `ci.yml`).
   - New dep: `ravif` (BSD-3, feature-gated) → **DEC-020** (adopt ravif; revisit
     DEC-004's AVIF-gating now that it's pure-Rust — recommend KEEP feature-gated
     for compile-time/binary-size/encode-speed). Run `just deny`.
4. **SPEC-019 — WebP output.**
   Lossless WebP via the `image`/`image-webp` backend (permissive, default-able —
   a real win for graphics/screenshots/alpha vs PNG). Lossy WebP only behind a
   feature-gated libwebp (`webp-lossy`, C/BSD) — OR defer lossy WebP entirely and
   document why (no permissive pure-Rust lossy encoder; DEC-018). Decide in the
   spec's DEC.
5. **(later — likely its own stage) responsive set + `<picture>`/srcset emission**
   + blurhash/thumbhash rider. Depends on AVIF + WebP. The chore-killer
   differentiator; net-new vs the original backlog.

> Order rationale: the flagship (1) is the differentiator AND the most
> self-contained (default JPEG, one permissive dep). (2) reuses its search. (3)/(4)
> add formats behind features. Adjust if you'd rather land a modern format first —
> but (1) is the "why I'd switch" demo and has the cleanest build.

## Scaffolding the stage (this session, step 1)

The original STAGE-004–007 (compose-metadata / batch-recipes / hardening /
release) are *proposed plans*, and no specs reference them yet. Create the new
stage with the next id (`just new-stage "modern formats and quality" PROJ-001` →
STAGE-008) and **drive order by status/priority, not number** — mark it
`active`/`high`, leave the others `proposed`, and in the brief's Stage Plan list
it in execution position (right after STAGE-003) with a one-line note that the
roadmap was re-prioritized 2026-06-16 (so numeric id ≠ execution order). Write
its `value_contribution` (the "set the look, not the number" differentiator + the
modern-format additions) and the 4–5-item spec backlog above. (Renumbering the
old stages for clean numeric order is optional and risky — skip it.)

Then `just new-spec "perceptual auto-quality shrink to a visual target" STAGE-008`
(or the new id) and design **SPEC-016** following the model.

## Orchestration model (unchanged — see the 2026-06-15 handoff for full detail)

- Per spec: design (Opus — you, or a fresh Opus subagent; author directly if
  subagents drop, which they did on overload this session) → build (Sonnet 4.6,
  prescriptive; mirror `specs/prompts/SPEC-014-build.md` / `SPEC-015-build.md`) →
  verify (Opus, read-only) → ship. Pause for the user before each merge/ship.
- Design+specs+DECs commit to `main` and **push before dispatching build**
  (gotcha #1). Build goes through a squash PR. Bookkeeping by hand on `main` after
  merge (the `just` helpers mis-glob).
- **NEW gate:** CI now runs `cargo deny check licenses` (DEC-018). Every build
  prompt must (a) keep it green and (b) for any new dep, state the license and run
  `just deny`. The full gate set is now: `cargo build` · `cargo test` ·
  `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` ·
  `cargo deny check licenses`.
- Build prompts must still: confirm every test named in the spec's `## Failing
  Tests` exists; commit incrementally; derive `Debug` on new public types.

## Cost capture — NEW standing process (this was silently broken)

An audit (2026-06-16) found cost tracking was structurally present but **empty**:
SPEC-001–013 all have `null`/`0` token + USD numerics (only SPEC-014/015 carry
real numbers, backfilled by hand). The build prompts had been instructing "append
a cost session with **null numerics**" — so reports aggregated to ~$0. The real
data was available all along in each `Agent` tool result. Fix it from here:

- **Do NOT write `null` numerics for build/verify cycles.** When you dispatch a
  build or verify subagent, the `Agent` result returns `subagent_tokens` and
  `duration_ms` — **capture both.** (If you build/verify directly instead of via a
  subagent, run `/cost` at the end and use that.)
- **At ship (the orchestrator's by-hand bookkeeping),** write each metered cycle's
  `cost.sessions` entry with the real values: `tokens_total` = `subagent_tokens`;
  `duration_minutes` = `round(duration_ms/60000)`; `estimated_usd` = tokens ×
  list rate (Opus 4.8 $5/$25, Sonnet 4.6 $3/$15 per MTok) — apply a stated mix
  (the SPEC-014/015 entries used ~80/20 input/output at list rates, no cache
  discount) and add a one-line note that it's an order-of-magnitude estimate
  (the harness reports a single combined token metric, no I/O split). Mirror the
  exact note style already in `specs/done/SPEC-014-…md` / `SPEC-015-…md`.
- **Design/ship cycles** are orchestrator main-loop work with no clean per-cycle
  metering — leave their numerics `null` with a "main-loop, not separately
  metered" note (as in SPEC-014/015).
- **Compute `cost.totals`** at ship: `tokens_total` = sum of the metered cycles
  (use `0`, not `null`, for the placeholder), `estimated_usd` = sum,
  `session_count` = 4. (Reports skip null numerics in sums but count
  `session_count`.)
- **Remove the "null numerics" line** from any build/verify prompt you write —
  replace it with "append your cost session; the orchestrator will fill the real
  tokens/usd/duration from the Agent result at ship." Don't copy the old wording
  from the SPEC-013/014 build prompts verbatim.
- `just report-weekly` already aggregates this (by spec / cycle / interface, plus
  total + avg-per-spec + top cost drivers + a "shipped without cost data" flag) —
  so once entries carry real numbers, the report becomes meaningful with no
  further work. SPEC-001–013 stay unmetered history (real numbers aren't
  recoverable; leave null or mark any fill as a labeled estimate).

## First action

1. Scaffold STAGE-004 (Modern Formats & Quality) per "Scaffolding" above; commit
   + push to `main`.
2. `just new-spec` + design **SPEC-016 (perceptual auto-quality)**: fill the spec
   + `## Failing Tests` + Implementation Context, write **DEC-019** (ssimulacra2 +
   metric/threshold/search-loop policy), write the build prompt, commit + push the
   design to `main`, then dispatch the Sonnet build.
3. Pause for the user before merging.

## Constraints recap
- **Permissive only** (DEC-018): verify any new dep's license; `just deny` must
  pass; no AGPL/GPL; LGPL only via documented exception.
- Pure-Rust single binary, zero system deps by default → native codecs (libwebp
  for lossy WebP) and heavy ones (ravif/AVIF) are **feature-gated**, never default.
- `single-image-library`, `no-async-runtime`, `no-unwrap-on-recoverable-paths`,
  `decode-once-no-per-op-disk`, `every-public-fn-tested`, `clippy-fmt-clean`
  (`--all-targets`), `untrusted-input-hardening`, `ergonomic-defaults`,
  `test-before-implementation`.
- Benchmark claims (later) must be held at equal quality (SSIMULACRA2) — the
  metric SPEC-016 adds is also the foundation for the benchmark suite + a future
  `diff`/visual-regression command (see the roadmap handoff).
</content>
