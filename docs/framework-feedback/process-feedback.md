# Spec-driven process — feedback after PROJ-001 shipped (crustyimg v0.1.0)

Written at the v0.1.0 release: **PROJ-001 (the crustyimg MVP) shipped end-to-end** —
~43 specs across **9 stages**, **41 DECs**, publicly installable via crates.io, a
Homebrew tap, and cross-platform GitHub Release binaries. This is honest feedback on how
the spec-driven template (the "Claude plays every role" variant) served *this* project.
It is a companion to the same-named doc in the bragfile project, but the conclusions here
are crustyimg's own.

> **Scope:** this is about the *process*, not the product. For what crustyimg does, see
> the README; for the conventions, `AGENTS.md`; for the framework mapping,
> `docs/CONTEXTCORE_ALIGNMENT.md`.

## Outcome assessment: the methodology delivered

crustyimg went from empty repo to a hardened, publicly-installable image CLI without a
death march and without a mid-project rewrite — which is notable given the *prototype it
replaced* died of exactly that (two competing image models, flag-soup, zero tests). The
single-image-library + pipeline architecture held across every feature stage, and the
untrusted-input hardening (STAGE-006) was a coherent gate rather than a scramble. The
process didn't just produce code; it produced code with a *reviewable decision trail*.

## What genuinely worked

- **The DEC log is the highest-value element.** 41 standalone, supersedable decision
  records meant almost nothing got re-litigated across 9 stages. DEC-004 (pure-Rust
  codec policy) silently governed every format spec; the license constraints
  (`no-agpl-default-deps`) caught AGPL/GPL dependencies *repeatedly* (gifski, imagequant,
  and — at release time — the `heic` crate) before they entered the tree. When a
  decision needed to change, supersession (not deletion) kept the history legible.

- **Design-time probes became the signature move — and paid off every time.** The
  template's insistence on a real *design* cycle before build created space to probe
  load-bearing crates against the *actual* pinned tree before writing failing tests. This
  caught concrete problems that would otherwise have surfaced as build/CI failures:
  `clap_complete`'s exact version pin, the fact that **cargo-dist can't publish to
  crates.io** (found before committing to a wrong config), and the **true MSRV floor**
  (a 1.85 guess vs the real 1.89 driven by a transitive dep). This wasn't in the template
  by name; it *emerged* from the design/build separation and is worth codifying.

- **The independent verify cycle caught real defects.** Verify run by a *different* agent
  session (not the builder) is the best quality lever in the variant. It flagged the MSRV
  floor being too low before merge, and — for the security-sensitive advisory acceptance
  — independently re-derived the reachability argument rather than rubber-stamping it.
  Self-review would have missed both.

- **Spec-as-source-of-truth made fresh sessions cheap.** Build could run as a brand-new
  Sonnet session driven only by the spec + a prescriptive prompt, with no "as I said
  earlier." When a build subagent *died mid-cycle* (see friction), the orchestrator could
  pick up from the spec state because the spec, not the session, was authoritative.

- **Constraint gates + cost capture were real, not decorative.** The `cargo deny` /
  clippy-fmt / lean-build / MSRV gates caught actual regressions (including the ambient
  advisory drift). Per-cycle cost capture (~$57.72 total metered build+verify) gave
  honest ROI visibility a vibes-based process never has.

## Friction I actually hit

- **The full cycle is disproportionate for trivial changes.** SPEC-043 was a **3-line
  `deny.toml` edit** and still ran design → build → verify → ship with a dispatched
  subagent. The *judgment* (accepting advisories, with a reachability assessment) was
  real and belonged in a DEC; the *ceremony* around three lines was pure tax. There's no
  lightweight lane.

- **Ship bookkeeping is manual and error-prone.** `advance-cycle` / `archive-spec` don't
  update the parent stage's backlog list or `**Count:**`; cost totals, the ship
  reflection, and the `git mv` to `done/` are all by hand, and the helpers occasionally
  mis-glob. This produced a *standing rule I had to follow every ship* — "verify the git
  index before the ship commit" — because editor/linter churn kept re-staging stale
  content. This is the same gap the template's own `KNOWN_LIMITATIONS.md` calls out; in a
  43-spec project it added up.

- **Subagent sessions are fragile and the template assumes they aren't.** Build subagents
  died on API overloads mid-cycle (twice this run), and background-dispatched subagents
  can't get a Bash permission at all. The recovery — orchestrator verifies the partial
  output and finishes in the main loop — worked, but it's an *unwritten* resilience
  pattern the process should name.

- **Ambient advisory drift broke a gate with zero code change.** The `cargo deny`
  advisories check went red on `main` because the RustSec DB updated underneath us — and
  it stayed red across several doc-only pushes before I caught it, because I only
  re-checked the gates I expected to be affected. The supply-chain gate runs on push/PR,
  not on a schedule, so time-varying advisories are invisible until the next push trips
  them.

- **Docs accreted without a user-vs-contributor split.** `docs/USAGE.md` is titled "How
  to use this *template*" but also holds the CLI batch/dogfood examples — so a *user*
  looking for "how do I process a folder of images" won't find it there. The README was
  tool-first only after a dedicated spec (SPEC-040) rewrote it. The template gives no
  guidance on separating "how to develop this repo" from "how to use the built tool."

## Improvements I'd prioritize (in order)

1. **A lightweight lane for mechanical changes.** For a doc/config one-liner with no
   design surface, allow design+build to collapse into a single step while *keeping the
   independent verify* (the part that actually catches mistakes). SPEC-043 shouldn't cost
   four cycles.
2. **Automate ship bookkeeping.** Have `archive-spec` update the stage backlog line +
   `**Count:**`, and compute `cost.totals` from `cost.sessions`. This is the single
   biggest source of manual error and the reason for the "verify the index" rule.
3. **A scheduled `cargo deny` advisory job.** Run the advisory check on a cron
   (daily/weekly), decoupled from code pushes, so DB drift is surfaced on its own instead
   of ambushing the next unrelated change.
4. **Name the subagent-recovery pattern.** Make "if a build/verify subagent dies mid-
   cycle, the orchestrator verifies its partial output and completes in the main loop" an
   explicit, cost-annotated procedure rather than an ad-hoc save.
5. **Codify the design-time probe.** It's the most valuable habit that isn't written
   down: for any load-bearing external crate/tool, probe it against the real pinned tree
   during design and record the verified calls in the dep's DEC.
6. **A user-vs-contributor docs split.** README + a `docs/` CLI-usage home for the tool;
   `docs/development.md` + `AGENTS.md` for the workflow. Batch/multi-file usage in
   particular should live where a *user* looks.

## What I would NOT change

- **The DEC discipline.** Keep decisions first-class, supersedable, confidence-scored.
- **The independent verify cycle.** A different session reviewing is worth its cost.
- **Design-before-build + spec-as-truth.** This is what made fresh sessions and subagent
  recovery possible.
- **Constraint gates + per-cycle cost capture.** Cheap, and they caught real problems.

## One meta-observation

**The template's value tracked the *stakes* of the change, not its size.** It earned its
overhead handsomely on the hardening specs (where the DEC + verify discipline prevented
real untrusted-input mistakes) and the release-engineering work (where design-time probes
caught wrong assumptions before they became public artifacts). It felt like pure tax on
the mechanical changes. The two things that most prevented actual errors were the
*independent verify* and the *DEC log* — not the ceremony of the four named cycles. If
you kept only those two and made everything else optional-by-stakes, you'd retain most of
the quality for a fraction of the overhead.

---

*This is a living document — append a new dated round at the next major milestone
(e.g. PROJ-002 kickoff or a 0.2.0 that removes the accepted advisories), the way the
bragfile version does.*
