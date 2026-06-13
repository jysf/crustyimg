# Recommended tools (optional)

This template runs with **zero external dependencies** — markdown, a
`justfile`, and pure-bash scripts. Nothing in this file is required.

It's a catalog of external tools worth reaching for *when a project's
needs outgrow the defaults*. Each entry says when to reach for it and
when to skip it. Adopting any of these is a **project-level** choice:
record it as a `DEC-*` in `/decisions/`, add it to the project's setup
docs, and keep it out of the template defaults so a fresh clone stays
dependency-free.

The bias: prefer the in-repo, text-based, LLM-authorable default; only
escalate to a heavier tool when the payoff is real.

---

## Diagrams

### Mermaid — the default (already in use)

<https://mermaid.js.org>

Diagrams in this repo are authored as fenced ```` ```mermaid ```` blocks
directly in markdown — see the example in `/docs/architecture.md`. This
is the default and it's deliberate:

- **Zero dependency.** It's just text in a `.md` file.
- **Renders where the docs live.** GitHub, GitLab, and most markdown
  viewers render Mermaid natively, so diagrams show up on the repo page
  with no build step.
- **Agents can maintain it.** Claude can write and *update* a Mermaid
  block in the same file it's already editing, so diagrams stay current
  through the design/build cycle instead of rotting.

Keep architecture, data-model, and flow diagrams as Mermaid in `/docs/`,
`/decisions/`, and specs. Update them as part of the work, not after.

### Structurizr — optional, for C4 at scale

<https://structurizr.com>

Structurizr models architecture once (the C4 model) and renders multiple
consistent views — context, container, component — that can't drift apart.

**Consider it when:**
- The architecture is large and long-lived, with many diagrams that must
  stay mutually consistent.
- You want enforced C4 rigor across a team.

**Skip it when:**
- A handful of Mermaid diagrams cover it. Most projects never outgrow
  Mermaid.

**No lock-in either way.** Structurizr's CLI exports to Mermaid, so the
clean path is Mermaid-first, escalating to Structurizr only if a project
genuinely needs C4 — and you can still render Mermaid from it. Note it's
a real dependency: a paid SaaS account or the self-hosted "Lite" server.

---

## Testing / Verify phase

The **Verify** cycle is convention-driven by default: a spec ships when
its acceptance criteria are met, tests pass, and there's no decision
drift (see `AGENTS.md` → "During verify"). The tools below help when a
project's verification needs outgrow in-process tests.

### LineSpec — protocol-level integration tests

<https://linespec.dev>

LineSpec intercepts MySQL, PostgreSQL, HTTP, Kafka, gRPC, and Redis at
the protocol level and drives them from a language-agnostic DSL
(`RECEIVE` / `EXPECT` / `VERIFY` / `RESPOND`), so the tests live outside
your application code.

**Consider it when:**
- A spec's acceptance criteria are about *protocol behavior* —
  request/response shapes, DB queries issued, message contracts — not
  just return values.
- You're verifying across a service boundary and mocks keep drifting
  from the real wire format.
- The implementer and the app are in different languages and you want
  one test suite that doesn't care.

**Skip it when:**
- Unit / integration tests in the app's own framework already cover the
  criteria. Don't add an infra dependency you don't need.
- The app has no meaningful DB/HTTP/queue traffic to assert on.

If you adopt it, reference the `.linespec` files from the relevant
spec's acceptance criteria.

---

## Decisions

### Native `just decisions-audit` — the default

Documenting and enforcing architectural decisions is handled in-repo by
`/decisions/` plus:

```bash
just decisions-audit             # structural lint + scope-conflict warnings
just decisions-audit --changed   # which decisions govern your pending changes
```

The optional `affected_scope:` glob list in a decision's front-matter
powers the scope checks (see `/decisions/_template.md`). For commit-time
enforcement, wire `just decisions-audit --changed` into a pre-commit hook.

### LineSpec Provenance Records — optional

LineSpec's *other* half (Provenance Records) is a binary-backed version
of the same idea — YAML decision records with git hooks and semantic
search over decisions via embeddings. It overlaps with what
`/decisions/` + `just decisions-audit` already do natively, so the
template doesn't depend on it. Reach for it only if you specifically want
embedding-based semantic search across a large decision history.
