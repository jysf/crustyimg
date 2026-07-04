# Framework signals harvest — for the template author

A structured harvest of distinct findings mined from **every shipped spec's reflection**
(ship Q2 "does any template/constraint/decision need updating?" + build Q2 "a constraint
that should have been listed?"), **each stage-level reflection**, and the open-questions
register, across PROJ-001 (crustyimg MVP → v0.1.0, 43 specs / 9 stages / 41 DECs).

This is feedback about the **spec-driven template itself**, not about crustyimg. Each
signal is bucketed as `lesson` (a rule earned by recurring evidence), `process-debt`
(framework/tooling friction to fix), `product` (a usage signal for what to build next),
or `risk` (a standing hazard to watch). Companion to `process-feedback.md` (the narrative
retrospective); this doc is the triage-ready index.

Disposition convention: `lesson`s are due at their **stage-close**; everything else at
**project-close** (now). `status: open` = actionable/needs a decision or codification;
`status: watch` = a standing hazard to monitor.

```yaml
signals:
  # ─────────────── LESSONS (coding/process rules earned by recurring evidence) ───────────────
  - id: design-time-probe-load-bearing-crates
    type: lesson
    summary: Probe any unproven external crate/tool against the REAL pinned tree during design and record the verified calls in the dep's DEC, before writing failing tests.
    evidence: {specs: [SPEC-020, SPEC-026, SPEC-041, SPEC-042, "+~13 more mention 'probe'"], n: 17, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-026
    notes: Highest-frequency lesson in the repo (17/43). Caught real dead-ends — clap_complete pin, the cargo-dist "crates.io is not a publish-job" gap, the true MSRV floor. Belongs in AGENTS.md as a named design-cycle step; today it lives only in an auto-memory.

  - id: verify-runs-lean-no-default-features-build
    type: lesson
    summary: Run `cargo build --no-default-features` in BOTH build and verify — default-feature gates (test/clippy/fmt/deny) never exercise the lean path, so feature-default deps slip through.
    evidence: {specs: [SPEC-028, SPEC-029, SPEC-030, SPEC-032, STAGE-004], n: 5, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-028
    notes: A SPEC-028 lean-only flake (ab_glyph's `std`) surfaced it; STAGE-004 asked to fold it into the standard gate set, which never happened at the template level.

  - id: index-verify-before-ship-commit
    type: lesson
    summary: Ship bookkeeping must go stage→`git show :file` (verify index)→commit→verify HEAD, because editor/linter churn re-stages stale content and cost-audit only catches a bad spec AFTER it lands on main.
    evidence: {specs: [SPEC-023, SPEC-024, STAGE-009], n: 3, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-023
    notes: Twice caused stale spec content to be committed. Symptom of the ship-bookkeeping-automation debt below.

  - id: push-design-before-build-branch
    type: lesson
    summary: Push the design commit + DECs to main BEFORE dispatching the build agent, or the squash-merged build PR silently drops the design.
    evidence: {specs: [SPEC-008, STAGE-002], n: 2, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-008

  - id: confirm-every-failing-test-exists
    type: lesson
    summary: A green test COUNT doesn't prove the spec's prescribed `## Failing Tests` were written — add an explicit end-of-build checklist step diffing the spec's tests against the files.
    evidence: {specs: [SPEC-011, "verify-test-existence memory (a dropped build left tests unwritten)"], n: 2, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-011

  - id: adversarial-bypass-grep-in-verify
    type: lesson
    summary: For hardening specs the highest-value verify step is an adversarial "grep every write/open/decode path" gap-hunt — it both confirms completeness and surfaces the next item.
    evidence: {specs: [SPEC-034, SPEC-035, SPEC-037], n: 3, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-034
    notes: STAGE-006-scoped, but generalizes to any security/hardening spec; verify-doubling-as-security-review (SPEC-037) was explicitly called high-value.

  - id: feature-gated-native-codec-pattern
    type: lesson
    summary: A feature-gated native codec = its own DEC (license+build-cost) + an empirical design probe + a dedicated CI job + an up-front `ensure_codec_built` check on EVERY multi-input path (to preserve "single exit 4, not partial-batch 6").
    evidence: {specs: [SPEC-018, SPEC-020, STAGE-008], n: 3, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-018

  - id: msrv-from-cargo-metadata-not-guess
    type: lesson
    summary: Declare `rust-version` as max(rust_version) across the LOCKED dep tree and enforce with a pinned msrv CI job — don't guess and let CI correct.
    evidence: {specs: [SPEC-041], n: 1, pattern: paired-opposing (1.85 guess vs 1.89 real, CI-corrected)}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-041

  - id: bookkeeping-edits-land-on-main
    type: lesson
    summary: Verify/ship bookkeeping edits (cost, cycle, archive) land on main, not the feature branch.
    evidence: {specs: [SPEC-001, STAGE-001], n: 2, pattern: same-outcome}
    status: open
    disposition_at: stage-close
    first_flagged: SPEC-001

  # ─────────────── PROCESS-DEBT (framework/tooling friction) ───────────────
  - id: ship-bookkeeping-not-automated
    type: process-debt
    summary: `archive-spec`/`advance-cycle` don't update the stage backlog list or **Count:**, and cost totals/reflection/git-mv are all manual and mis-glob-prone — the single biggest source of manual error.
    evidence: {specs: [SPEC-001, SPEC-023, SPEC-024, STAGE-009, "template KNOWN_LIMITATIONS.md"], n: 5}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-001
    notes: RE-FLAGGED and the template deliberately punts on it (KNOWN_LIMITATIONS calls markdown-list editing "judgment-laden"). But it directly caused index-verify-before-ship-commit to become a standing rule. Highest-value debt — at least auto-compute cost.totals and update the Count line.

  - id: cost-capture-assumes-metered-subagent
    type: process-debt
    summary: The `cost-captured-per-cycle` constraint + `just cost-audit` assume build/verify run as METERED subagents; when they run in the main loop (subagent blocked/dies), tokens_total is a labeled estimate, not a clean meter.
    evidence: {specs: [SPEC-016, SPEC-017, SPEC-042], n: 3}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-016
    notes: RE-FLAGGED three times over ~6 months, never resolved. Ties directly to the non-Claude-portability question — the whole cost model is welded to Claude's /cost + subagent_tokens.

  - id: scheduled-advisory-ci-missing
    type: process-debt
    summary: `cargo deny` advisories only runs on push/PR, so time-varying RustSec drift goes red with zero code change and stays invisible until the next unrelated push trips it.
    evidence: {specs: [SPEC-043, SPEC-042], n: 2}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-043
    notes: RE-FLAGGED (raised at SPEC-043, re-noted SPEC-042). Actually bit this project — main sat red across several doc-only pushes. Fix is a scheduled/cron advisory job, decoupled from code pushes.

  - id: build-prompt-lacks-language-gotcha-checklist
    type: process-debt
    summary: A recurring class of "the spec should have noted X" build-reflections are Rust/tooling gotchas the spec template doesn't pre-warn about.
    evidence: {specs: ["SPEC-005 non_exhaustive enum wildcard", "SPEC-007 #[from] io::Error collision", "SPEC-013 pub for tested fn", "SPEC-031 doc_lazy_continuation", "SPEC-032 stale test needs update", "SPEC-034 macOS tempdir canonicalize", "SPEC-038 --allow-dirty"], n: 7}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-005
    notes: Each is trivial alone; the pattern says the template wants a per-language "known gotchas" appendix the build prompt links.

  - id: ci-step-name-bare-colon-footgun
    type: process-debt
    summary: A CI step `name:` containing a bare colon broke YAML parsing on first push.
    evidence: {specs: [SPEC-036], n: 1}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-036

  # ─────────────── PRODUCT (usage signals for what to build next) ───────────────
  - id: batch-parallelism-only-in-apply
    type: product
    summary: Per-command multi-input fan-out is sequential; only `apply --recipe` is parallel (`-j`), so large single-op batches get no concurrency.
    evidence: {specs: [SPEC-031, "docs work this session"], n: 2}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-031

  - id: metadata-coverage-asymmetric
    type: product
    summary: strip/clean/set do JPEG+PNG but copy-metadata is JPEG-only; PNG eXIf↔zTXt and WebP/TIFF metadata remain open — the coverage matrix is uneven.
    evidence: {specs: [SPEC-026, SPEC-027, SPEC-028, "open Q metadata-icc-coverage"], n: 3}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-026

  - id: avif-decode-and-perceptual-gap
    type: product
    summary: AVIF is encode-only (no pure-Rust decoder), so .avif INPUT is unsupported and perceptual auto-quality can't score AVIF output.
    evidence: {specs: [SPEC-018, "perceptual-search-needs-a-decoder memory", "watchlist avif-decode"], n: 2}
    status: open
    disposition_at: project-close
    first_flagged: SPEC-018

  - id: heic-and-raw-input-demand
    type: product
    summary: Direct user demand (this session) for HEIC and camera-RAW (Nikon/Canon/Fuji/Leica) INPUT — a future input-formats wave; RAW-via-embedded-preview is the tractable, permissive path.
    evidence: {specs: ["this session's user asks + docs/backlog.md fast-follows"], n: 1}
    status: open
    disposition_at: project-close
    first_flagged: post-0.1.0

  - id: help-text-leaks-internal-jargon
    type: product
    summary: Shipped `--help` command descriptions leak STAGE-00X/DEC-0XX references and a stale "stub" note — user-facing polish.
    evidence: {specs: ["v0.1.0 install smoke-test this session"], n: 1}
    status: open
    disposition_at: project-close
    first_flagged: post-0.1.0

  # ─────────────── RISK (standing hazards to watch) ───────────────
  - id: accepted-advisory-ignores-shipped
    type: risk
    summary: v0.1.0 ships with 3 deny.toml advisory ignores (2 quick-xml vulns via little_exif with no upgrade path; 1 ttf-parser unmaintained) — reachability-assessed as not-reached + STAGE-006-bounded, but live until eliminated.
    evidence: {specs: [SPEC-043, DEC-042], n: 1}
    status: watch
    disposition_at: project-close
    first_flagged: SPEC-043
    notes: Revisit triggers are the 0.2.0 fast-follows (ab_glyph→fontdue; in-house EXIF writer to drop little_exif).

  - id: subagent-session-fragility
    type: risk
    summary: Build/verify subagents die on API overloads mid-cycle and background-dispatched subagents can't get a Bash permission; recovery (orchestrator verifies partial output + finishes in main loop) is unwritten.
    evidence: {specs: [SPEC-042, "background-subagents-cannot-get-bash memory", "this session x2 overloads"], n: 3}
    status: watch
    disposition_at: project-close
    first_flagged: SPEC-016

  - id: operation-not-send-rebuild-invariant
    type: risk
    summary: The `Operation: !Send` rebuild-per-task pattern is the load-bearing concurrency invariant for `apply` parallelism; a future op that's expensive to rebuild breaks it.
    evidence: {specs: [SPEC-031], n: 1}
    status: watch
    disposition_at: project-close
    first_flagged: SPEC-031

  - id: feature-vs-lean-bitrot
    type: risk
    summary: Feature-gated paths (display/avif/webp-lossy) bit-rot if a new feature ships without its own CI job; the lean+feature jobs are the only guard.
    evidence: {specs: [SPEC-028, STAGE-008], n: 2}
    status: watch
    disposition_at: project-close
    first_flagged: SPEC-028

  - id: open-questions-unresolved-at-close
    type: risk
    summary: Two guidance questions remain open at project close — resize-backend-api-stability and metadata-icc-coverage.
    evidence: {specs: ["guidance/questions.yaml"], n: 2}
    status: watch
    disposition_at: project-close
    first_flagged: STAGE-003
```

## meta

**Re-flagged but never adopted or rejected — the highest-value process-debt.** Three
items recur across rounds and were never dispositioned:

- **`cost-capture-assumes-metered-subagent`** — raised at SPEC-016, re-raised at
  SPEC-017, and again at SPEC-042. Every time a build ran in the main loop instead of as
  a metered subagent, `tokens_total` became a labeled guess and the note said "worth
  flagging." It never turned into a template change. The clearest "raised-thrice,
  adopted-never."
- **`scheduled-advisory-ci-missing`** — flagged at SPEC-043, echoed at SPEC-042, and it
  *actually bit the project* (main sat red across several pushes). Still no scheduled job.
- **`ship-bookkeeping-not-automated`** — flagged from SPEC-001 onward and the template's
  own `KNOWN_LIMITATIONS.md` explicitly *decided not to fix it* ("markdown-list
  formatting is judgment-laden"). That's a legitimate rejection — but the cost kept
  compounding (it's the root cause of the `index-verify-before-ship-commit` rule). Worth
  reopening the "at least auto-compute `cost.totals` + the `Count:` line" middle ground,
  which is not judgment-laden.

**Friction a generic template can't encode.** (a) Language/tooling gotchas —
`non_exhaustive` enums, `pub`-for-tested-fn, `#[from] io::Error` collisions,
`doc_lazy_continuation`, macOS tempdir `canonicalize`, `--allow-dirty` — are Rust/cargo
facts; they belong in a per-language appendix the build prompt links, not the core
template. (b) Time-varying external state — RustSec advisory drift breaks a gate with
zero code change; no static template can pre-encode it, only a scheduled job (which is
CI-provider-specific). (c) Agent-runtime reliability — subagent overloads and
Bash-permission gaps are environment facts the template silently assumes away.

**If this ran on a non-Claude agent, what breaks or gets awkward.** The cost model is the
biggest tie-in: `cost.sessions[].agent` hard-codes model IDs (`claude-opus-4-8`,
`claude-sonnet-4-6`), `agents.architect/implementer` are Claude model IDs, and
`cost-captured-per-cycle` + `just cost-audit` assume Claude's `/cost` + `subagent_tokens`
— a different agent has no equivalent and the whole "metered subagent" premise collapses
(see the re-flagged debt above). Build prompts are Claude-Code-shaped in wording ("run on
Sonnet", "fresh Claude session", "background subagents can't get Bash"), and the
Sonnet-build / Opus-design-verify split is a Claude-model-tier optimization with no
analog elsewhere. The deepest coupling is the **session model**: the entire variant rests
on "new Claude session per cycle" with architect and implementer as *separate cheap
sessions* — an agent without cheap fresh contexts would find the ritual awkward (though
the "Implementation Context folded into the spec" design generalizes cleanly, precisely
because it removes the separate-handoff object). A portable version would need: a generic
`agent`/`model` field, a cost hook abstracted from `/cost`, and prompt boilerplate
stripped of Claude-Code-isms.

**What the discipline bought vs. what was ceremonial.** *Bought:* the **DEC log**
prevented re-litigation across 9 stages and repeatedly caught license landmines (AGPL
gifski/imagequant/heic) before they entered the tree; the **independent verify cycle**
caught real defects a self-review misses (the too-low MSRV floor, a dropped build's
missing tests, the advisory reachability judgment); **design-time probes** (17 specs)
caught wrong assumptions before they became public artifacts; the **constraint gates**
caught advisory drift and lean-build flakes; **cost capture** gave honest ROI.
*Ceremonial:* the full four-cycle for trivial changes (SPEC-043 was a 3-line `deny.toml`
edit that still ran design→build→verify→ship); the **Frame** cycle is essentially unused
(0 specs sat in `frame`); stage-level reflections are frequently "no change needed"
boilerplate; and the manual bookkeeping ritual is pure tax. The pattern: value tracked
the *stakes* of the change, not its size — the two elements that actually prevented
errors were the *independent verify* and the *DEC log*, and a stakes-tiered "lightweight
lane" that kept only those two for mechanical changes would shed most of the ceremony
without losing the quality.
