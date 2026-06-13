---
title: "Dogfooding: what shipping bragfile taught the template"
date: 2026-06-02
draft: true
tags: [spec-driven, dogfooding, case-study]
---

> Draft. Written from the project history — edit into your own voice
> before publishing.

A template is a theory about how work should go. The only way to find out
if the theory holds is to ship something real with it. So I did:
**bragfile**, a small Go CLI for logging your accomplishments, built
entirely on the spec-driven template and shipped to a Homebrew tap.

## The numbers

bragfile's MVP (`PROJ-001`) came out to **5 stages, 23 specs, and 14
decision records over about four weeks.** Every spec went through the
full Frame → Design → Build → Verify → Ship cycle. Every non-trivial
choice — using a pure-Go SQLite driver, validating required flags in
`RunE` instead of `MarkFlagRequired`, the stdout-is-data / stderr-is-for-
humans contract — landed as a `DEC-*` you can still read in the repo.

That last part is the payoff of the model: the *reasons* are in the repo,
not lost in a chat history.

## The feedback loop is the point

Building bragfile generated a list of friction. I captured it in the
template's own `feedback/` folder, and it drove concrete fixes:

- A false-positive in `just archive-spec`'s "stage shipped" detection —
  cited multiple times in the process feedback, fixed in the template.
- A placeholder-substitution bug in a decision template — *the same bug
  class* I later hardened in `new-spec`/`new-stage`.
- "A fresh session is weaker than it looks with a single agent" — a note
  that fed directly into how the onboarding docs and the two-agent
  variant are framed.

None of these were visible from inside the template. They only showed up
because I was actually trying to ship with it. That's the whole argument
for dogfooding: **the template improves by being used, and the friction
log is where the next version comes from.**

## A feature that flowed backward

The clearest example: bragfile's project repo had a little command that
listed every spec grouped by its stage, with ship dates and complexity —
a flat ledger you could scan to see the whole arc of the project. The
template didn't have it. So I pulled it back upstream as `just
specs-by-stage`.

I didn't copy it verbatim. The downstream version scraped ship dates out
of backlog prose; the upstreamed version derives them from authoritative
front-matter (the cycle, the complexity, the ship cost-session date), so
it stays accurate on the template's data model. The *idea* came from the
project; the *implementation* got rebuilt to the template's standards.
And during that rebuild I found and fixed a latent `set -e` bug the
verbatim port would have carried.

That's the healthy version of dogfooding: real use surfaces the idea,
and bringing it home is its own small act of engineering — not a
copy-paste.

## What I'd tell anyone starting

Pick something you actually want to ship, not a toy. The template's value
shows up exactly where real projects get messy — the tenth decision, the
third stage, the spec you have to send back from Verify. A toy never gets
there. bragfile did, and the template is better for it.
