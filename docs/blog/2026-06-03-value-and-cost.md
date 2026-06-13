---
title: "Two numbers traditional dev hides: value and AI cost per spec"
date: 2026-06-03
draft: true
tags: [spec-driven, value, cost, agents]
---

> Draft. Written from the project history (the v5.2 work) — edit into your
> own voice before publishing.

Traditional development tracks effort: story points, hours, velocity. When
you build with agents, two *other* numbers turn out to matter more, and
neither shows up on a normal board:

1. **Does this work actually advance why the project exists?**
2. **What did the AI cost to produce it?**

The v5.2 release of the template made both of these first-class, in the
front-matter, on every spec. Here's the idea.

## A value chain from project to spec

The unit of "why" is a **thesis**. A project brief states one —
`value.thesis` — plus a short, honest list of `risks_to_thesis` (the 2–4
things that would make the whole project wrong). That's the anchor.

Then it flows down:

- A **stage** has `value_contribution.advances` — one sentence on which
  part of the project's thesis this stage moves.
- A **spec** has a `value_link` — one sentence tying this individual task
  back to its stage's contribution. Even plumbing gets one
  ("infrastructure enabling STAGE-002's measurement core").

The point isn't bureaucracy. It's that at any spec you can walk the chain
upward and answer "why am I building this?" in three hops — and if you
*can't*, that's a signal the work is unmoored. The chain makes
unjustified work visible.

## Cost, self-reported, per cycle

The other half is honesty about spend. Every spec carries a `cost` block.
Each cycle — design, build, verify, ship — appends one entry to
`cost.sessions[]`:

- `interface` — `claude-code | claude-ai | api | ollama | other`
- `recorded_at`, and the cycle it belongs to
- `tokens_input` / `tokens_output` and an `estimated_usd`

At ship, `cost.totals` is computed across the sessions. The design is
deliberately forgiving: null numeric fields are fine — a claude.ai web
session has no token count you can paste — and the reports *skip* nulls in
sums but still *count* the session. You're never forced to fabricate a
number to stay compliant.

## What the reports do with it

`just report-weekly` and `just report-daily` roll these up: tokens and
dollars spent in the window, which specs they came from, alongside what
shipped and which decisions are low-confidence. Suddenly the question
"what did this week of agent-assisted work cost, and what value did it
move?" has an answer you can read in one file — built from markdown
front-matter, no analytics service, no tracking pixel.

## Why bother

Because agent-assisted development makes it very easy to produce a lot of
plausible work quickly, and very easy to lose the thread of whether it
*mattered* and what it *cost*. Effort metrics don't catch either failure —
you can burn tokens at high velocity on work that doesn't advance the
thesis and feel productive the whole time.

Making value and cost first-class, per spec, is a small honesty mechanism.
It won't stop you from doing unmoored or expensive work. It just makes
both impossible to not notice.
