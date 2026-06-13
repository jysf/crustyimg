---
title: "Zero-dependency tooling: decisions-audit and the reporting trio"
date: 2026-06-02
draft: true
tags: [spec-driven, bash, tooling]
---

> Draft. Written from the project history — edit into your own voice
> before publishing.

When you build developer tooling, there's a constant temptation to reach
for a real tool: a binary, a service, a DSL with a parser. Sometimes
that's right. But for a template whose whole premise is "runs on a fresh
clone with nothing installed," every dependency is a tax on every future
user. So the rule I held to is: **pure bash and markdown until the payoff
of a heavier tool is undeniable** — and even then, make it opt-in.

Two features show what that buys you.

## decisions-audit: provenance without a binary

The template records architectural decisions as `DEC-*` markdown files.
The obvious next step is a tool that keeps them honest. There's a good
commercial one — LineSpec — that does exactly this with a binary, git
hooks, and embedding-based semantic search. I looked at it and built the
native version instead:

```bash
just decisions-audit             # lint structure + warn on scope conflicts
just decisions-audit --changed   # which decisions govern your pending edits
```

The default mode checks that every record is well-formed: filename
matches its id, no duplicate ids, required fields present, and
`supersedes` / `superseded_by` links are consistent in *both* directions
(no dangling or one-sided supersession). It exits non-zero on a real
problem, so it drops straight into CI or a pre-commit hook.

The `--changed` mode is the interesting half. Add an optional
`affected_scope:` glob list to a decision — the paths it governs — and
the audit will tell you, before you commit, which decisions cover the
files you just touched. "You're editing `src/lib/log.ts`; re-read
DEC-001 before you ship." That's the LineSpec *provenance audit* idea,
done in a few hundred lines of bash with no dependency and no network.

The trade-off is honest: I don't get embedding-based semantic search. If
a project's decision history gets big enough to need that, the catalog
points at LineSpec. But most repos never get there, and the default
shouldn't make everyone pay for the case that rarely happens.

## The reporting trio (plus one)

The other place this philosophy shows up is in *seeing* the work. There
are four read-only views, each answering a different question, all parsing
the same markdown front-matter:

- `just status` — current state: active project/stage, specs by cycle.
- `just backlog` — spec-grained "what's next" (forward-looking).
- `just roadmap` — stage-grained "where is this going" (counts per stage).
- `just specs-by-stage` — a flat ledger of *every* spec by stage, with
  ship date and complexity, across all projects.

No database, no indexer — just `find`, `awk`, and front-matter readers
shared from one `_lib.sh`. Each view is a different lens on the same
files, which means there's nothing to keep in sync and nothing to
corrupt.

## The unglamorous constraint that shaped all of it

macOS ships bash 3.2. That single fact rules out `mapfile`, associative
arrays, and a pile of conveniences. It's annoying — and it's also a
forcing function. You build arrays with `while IFS= read -r`, you do
id→file lookups with parallel arrays and a linear scan, you branch on
`uname` when BSD and GNU `sed` disagree. The result is tooling that runs
the same on a developer's Mac and a Linux CI box with zero setup.

That's the quiet thesis here: **constraints that look like limitations
are often what make a tool portable, legible, and honest.** A binary
would have been faster to write. The bash version is faster to *adopt* —
and adoption is the only thing that matters for a template.
