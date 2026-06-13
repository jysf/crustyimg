---
title: "The repo is the app: why I built a spec-driven template"
date: 2026-06-02
draft: true
tags: [spec-driven, workflow, agents]
---

> Draft. Written from the project history — edit into your own voice
> before publishing.

Most "AI coding workflow" setups optimize for a single conversation: you
open a chat, describe a feature, get code, move on. That works until the
second feature. By the tenth, the context that made the first one good —
why you chose this database, what the logging convention is, which
trade-offs you already settled — has evaporated. The agent doesn't
remember, and neither, honestly, do you.

So I built a template around a different unit of persistence: **the repo
is the app, and it outlives every project.**

## The hierarchy

```
Repo (the app — persists forever)
 └─ Project (a wave of work: "MVP", "v2", "redesign")
     └─ Stage (a coherent chunk: 2–5 per project)
         └─ Spec (one implementable task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

The key move is that **architecture, conventions, constraints, and
decisions live at the repo level**, not inside any one task. A decision
made while building the MVP — "all logging goes through one module" — is
written down once as a `DEC-*` record and is still there, still binding,
when you start v2 eighteen months later. Projects come and go; the repo
accumulates.

This is the opposite of one-repo-per-feature or one-chat-per-task. Those
throw away context by design. Here, context is the asset.

## Why specs, and why a cycle

Each task is a **spec** — a small markdown file with front-matter (an ID,
a cycle, a complexity, a parent stage) and a body (acceptance criteria,
design notes, implementation context). It moves through five phases:

- **Frame** — why this exists and what "done" means.
- **Design** — the approach and its trade-offs; non-trivial choices
  become decisions.
- **Build** — implement against the spec.
- **Verify** — acceptance met, tests pass, no drift from prior decisions.
- **Ship** — reflect, total the cost, archive.

The cycle isn't ceremony. Each phase produces an artifact the next phase
(or the next project) can rely on. The Verify phase in particular exists
because, with agents, drift is silent: code can quietly contradict a
decision nobody re-read. So one of the template's commands literally asks
"which past decisions govern the files this change touched?" before you
ship.

## Zero dependencies on purpose

The whole thing is markdown, a `justfile`, and pure bash. There's nothing
to install to use it. That's a deliberate constraint, not a limitation I
haven't gotten around to fixing — it means an agent can author and update
every artifact in-place, and it means the template runs identically on a
fresh clone with no setup. When a project genuinely outgrows a default
(diagrams, protocol-level tests), there's a catalog of optional, *opt-in*
tools — but the spine stays dependency-free.

## Two shapes of the same idea

It ships in two variants: `claude-only`, where one agent plays every role
(architect, implementer, reviewer), and `claude-plus-agents`, where Claude
architects and a separate tool implements — with explicit handoff
documents carrying context between agents that don't share memory. Same
hierarchy, same cycle; the difference is how many minds are in the loop.

If there's one idea to take away: **stop optimizing for the conversation,
and start optimizing for the repo that remembers.**
