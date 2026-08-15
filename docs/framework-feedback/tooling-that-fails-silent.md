# The template's tooling fails silent — findings for the template author

Written 2026-08-14, from crustyimg (claude-only variant), after PROJ-010's STAGE-043 and
STAGE-045 shipped. Companion to `process-feedback.md` (the narrative retrospective) and
`signals-harvest.md` (the triage index).

**This is about the template's `scripts/`, not about crustyimg.** Six defects below live in
files dated `Initial commit` (2026-06-13) and ship to every repo built on this template. One
(`specs-index.sh`) is crustyimg's own and is included only because it is the same shape.

---

## The pattern, which matters more than the six bugs

In one working session, **seven separate tools reported success or zero while being wrong.**
Not one crashed. Not one printed a warning. Every single failure mode was a green.

That is a specific hazard for *this* template, because the template's whole premise is that
an agent with no memory of prior sessions reconstructs its situation from the repo and its
tooling. `just backlog`, `just specs-by-stage`, `just status` and `just roadmap` are not
conveniences here — they are how a fresh session learns what work exists. A tool that
under-reports does not inconvenience such an agent; it **lies to it**, and the agent has no
independent way to notice.

The concrete cost in this repo, measured:

- **35 open backlog items were invisible.** `just specs-by-stage` reported *"1 not yet
  written"* as a repo-wide total across 43 stages and 7 projects.
- **Two shipped specs sat at `cycle: design`** through build *and* verify, because
  `advance-cycle` edited the wrong file and printed its success hint anyway.
- **The decision-drift gate never ran** on any verify cycle, in a form that could not fail.
- A maintainer asked why work on a known file "had no spec" — it *did* have a backlog entry,
  and the tool that answers that question had been reporting zero for months.

None of these was found by the tooling. All were found by hand, by accident, while doing
something else.

---

## Finding 1 — `find_spec` matches cycle prompts (`_lib.sh`)

**Structural, not drift. Any repo that uses cycle prompts is affected.**

The template mandates two naming conventions that collide:

| artifact | path | AGENTS ref |
|---|---|---|
| the spec | `projects/*/specs/SPEC-NNN-<slug>.md` | §7 |
| a cycle prompt | `projects/*/specs/prompts/SPEC-NNN-<cycle>.md` | §9 |

`find_spec` globs `SPEC-NNN-*.md` and takes `head -n1`. `find` returns traversal order, so
callers got `prompts/SPEC-NNN-build.md`.

Consequences: `advance-cycle` "updated" a file with no `task:` block and **still printed
`Next: use Prompt 5 (Ship)`**, so specs silently stayed at the wrong cycle. `archive-spec`
would have moved the prompt into `done/` and left the spec behind.

**Fix:** add `-not -path '*/prompts/*'`. One line. `find_stage` has the same shape with *no*
exclusions at all (not even `done/`) — latent today, same class.

## Finding 2 — `decisions-audit --changed` cannot fail where it is prescribed

**Structural.** AGENTS §15 instructs the **verify** cycle to run
`just decisions-audit --changed` as the decision-drift check. `--changed` scopes to
*uncommitted* changes. A verify cycle works from a clean checkout of the branch under
review, so it has none: the command prints `No changed files in scope` and exits 0.

The prescribed invocation is structurally incapable of failing in the situation it is
prescribed for. Every verify that followed the documentation got a green that proved nothing.

**Fix:** pass the base ref (`--changed main`), and have the script fall back to the default
branch — with a warning — when a bare `--changed` finds a clean tree. Correct AGENTS §15.

## Finding 3 — three duplicated prose-keyed counters

`backlog.sh`, `roadmap.sh` and `specs-by-stage.sh` each carried a **private copy** of the
same counter, all matching the literal string `(not yet written)` — the phrasing in
`projects/_templates/stage.md:63`.

The template is internally consistent, so this works until someone rewords a bullet. In
crustyimg the corpus drifted to `(not yet framed)`: **11 occurrences against 3**. All three
counters then reported zero. No warning, exit 0, and three tools agreeing with each other
because they shared a bug rather than a fact.

`specs-by-stage`'s copy had an extra defect: its class `\[[ x~?]\]` counted **closed**
bullets, so a completed backlog would still report outstanding work.

**Fix:** one implementation, keyed on **structure not prose** — an open `- [ ]` row that does
not lead with a bold `**SPEC-NNN**` id. Three call sites, one helper, cannot diverge again.

> The general lesson: **never key a counter on author-written prose.** Prose drifts; the
> checkbox does not. If prose keying is unavoidable, a zero result should warn rather than
> print zero.

## Finding 4 — `get_active_stage_file` sees only the first active stage

The template's own model permits several stages active at once; PROJ-010 legitimately ran
three. The helper returns the **first** and stops, and four scripts depend on it.

Result: `just backlog` printed `(none in active stage)` while a spec was mid-build in the
second active stage.

**Fix:** add a plural `get_active_stage_files`; keep the singular for compatibility.

## Finding 5 — `archive-spec`'s success message invites a wrong conclusion

On archiving the last spec it prints:

> `No active specs remain for STAGE-NNN.`
> `If the stage's Spec Backlog is fully complete, run the Stage Ship prompt.`

That is true and misleading. It means "no spec *files* outside `done/`" and says nothing
about the backlog. Both stages it fired on still had open, unframed backlog items. The
conditional in the second line is doing load-bearing work that the reader skims past.

**Fix:** count the open backlog bullets and say so — *"3 backlog items remain; stage is not
complete."* The tool already has the file in hand.

## Finding 6 — heredoc scripts inherit the caller's locale

crustyimg's own `specs-index.sh`, not template code, but the same shape and worth knowing.

`ruby -EUTF-8 … <<'RUBY'` sets **I/O** encoding, not the **source** encoding of a script read
on stdin, which defaults to US-ASCII when `LANG` is unset. The heredoc held non-ASCII
literals, so ruby aborted parsing its own source — *after* the shell redirect had already
truncated the output file by 356 lines. A failure that destroys the artifact it was
regenerating.

**Fix:** a `# encoding: utf-8` magic comment. Worth auditing any template script that pipes a
heredoc into ruby/python. Related: an em dash inside a `sed` bracket expression becomes a
broken range under a non-UTF-8 locale.

---

## What I would change in the template, in order

1. **Make silence loud.** Any tool that reports "0" or "none" for a countable thing should
   distinguish *"I looked and found none"* from *"my pattern matched nothing."* The second is
   a bug report, not a result. This one change would have surfaced findings 1, 2, 3 and 5.
2. **De-duplicate the three counters into `_lib.sh`.** Three copies of one rule guarantees
   divergence; they had already diverged in behaviour (`specs-by-stage` counted closed rows,
   the others didn't) without anyone noticing, because none of them was producing output
   anyone could check.
3. **Key on structure, never on prose.** Checkboxes, front-matter fields and path shapes are
   stable. Sentences written by humans and agents are not.
4. **Ship a self-test for the tooling.** `scripts/test.sh` exists and is good, but it does not
   assert that the reporting tools report *correct numbers* — only that they exit 0. A single
   fixture project with a known backlog count would have caught findings 3 and 4 on day one.
5. **Fix the `find_spec` / prompts collision**, since the template mandates both conventions.

## What I would not change

The DEC log, the cycle model, and the spec-carries-its-own-context design all held up under
a project that found six live defects in its own shipped binary. The failures here are in the
*instrumentation*, not the method — which is precisely why they went unnoticed for so long:
the method kept working, so nobody had cause to doubt the dashboard.

---

## Provenance

Every claim above was driven, not inferred. Each fix landed in crustyimg with a
before/after measurement:

| finding | evidence |
|---|---|
| 1 | `advance-cycle SPEC-113 ship` wrote nothing before; reports `design → ship` after |
| 2 | clean worktree + committed change: `No changed files in scope` before; surfaces DEC-087 and DEC-089 after |
| 3 | `specs-by-stage` total `1` before, `35` after — cross-checked against an independent parse |
| 4 | `backlog` showed `(none in active stage)` before; shows SPEC-114 in flight after |
| 6 | `env -u LANG -u LC_ALL just specs-index` fails before, succeeds after |

Patches available in crustyimg PRs #157, #160 and #163 if useful as reference implementations.
