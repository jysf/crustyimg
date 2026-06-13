---
title: "One agent or two: the two shapes of spec-driven work"
date: 2026-06-03
draft: true
tags: [spec-driven, agents, workflow]
---

> Draft. Written from the project history — edit into your own voice
> before publishing.

The template ships in two variants, and people always ask which to pick.
The honest answer is that they're the same workflow solving the same
problem — **agents don't remember** — from two directions. Understanding
the problem makes the choice obvious.

## The problem both variants solve

An agent's context is a goldfish bowl. Within a session it's brilliant;
across sessions it remembers nothing. Worse, *within* a long session it
accumulates contamination — half-formed ideas, abandoned approaches,
things "I said earlier" that were wrong. Spec-driven work fights this by
making context an artifact: the spec, the decision, the handoff are the
memory, not the chat.

The two variants differ only in how many minds are in the loop.

## `claude-only`: one mind, strict hygiene

Here one agent plays every role — architect, implementer, reviewer. It's
simpler, and it's where I tell everyone to start. But one mind playing
three roles is exactly where contamination bites, so the discipline is
**session hygiene**:

- **A new session per cycle**, especially design → build and build →
  verify. A fresh Verify session catches drift a continuation would
  rationalize away.
- **Never "as I said earlier."** The spec is the source of truth, not the
  scrollback.
- **The weekly review is non-optional.** Without a second agent pushing
  back, drift compounds silently.

Dogfooding taught me the sharp edge of this: *a fresh session is weaker
than it looks with a single agent.* You think you've reset, but the spec
you wrote and the code you're reviewing came from the same head. The
"freshness" is partly an illusion — which is precisely why the written
artifacts have to carry the context, and why the review cadence matters.

## `claude-plus-agents`: two minds, explicit handoffs

The other variant splits the roles: Claude architects (writes the spec,
the failing tests, the design decisions) and a *separate* tool implements
(Kilo Code, Factory Droids, Cursor, whatever). They don't share memory at
all — so the context has to be written down or it doesn't cross.

That's what the `/handoffs/` folder is for. A **handoff** is an
architect-to-implementer delegation document: `HANDOFF-NNN`, with the
spec it covers, the `DEC-*` decisions the implementer must read, and a
`## Completion` section the implementer fills in when done. It's the
contract between two agents that can't talk.

The upside is real separation — the implementer can't lean on the
architect's reasoning, so gaps in the spec surface immediately. The cost
is that you maintain the handoff layer.

## How to choose

Start `claude-only`. It's simpler, and migrating to `claude-plus-agents`
later is about an hour of mechanical work — mostly lifting the
`## Implementation Context` out of each spec into a handoff doc. Reach for
two agents when you want a genuine separation of concerns: a different
tool doing implementation, or a workflow where "the implementer only knows
what the handoff says" is a feature you want to enforce.

Either way, the underlying move is identical: **don't trust memory, trust
the artifact.** One mind or two, the template's job is to make the context
explicit enough that the next session — or the next agent — can pick it up
cold.
