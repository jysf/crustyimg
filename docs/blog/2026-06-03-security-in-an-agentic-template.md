---
title: "Security in an agentic template: when the agent types the title"
date: 2026-06-03
draft: true
tags: [security, agents, bash, spec-driven]
---

> Draft. Written from the project history (the v5.8 audit) — edit into
> your own voice before publishing.

I did a security audit of this template — fourteen bash scripts, a
`justfile`, the CI surface — expecting to find nothing. It's local
tooling: no server, no network calls, no secrets handling. The kind of
code where the honest answer is usually "the threat model is you, on your
own machine, hurting yourself."

I found one real bug. It's small. But *why* it matters is the whole point
of this post, so let me start with the bug and then explain why a
five-character shell quirk is a security finding in 2026.

## The bug

Scaffolding a new spec runs, in effect:

```bash
sed "s|<Short Title>|${TITLE}|g" spec.md
```

`TITLE` is whatever you typed. It's substituted into the `sed` command
**unescaped**. If your title contains the `|` delimiter, it closes the
substitution early — and what follows lands in the *flags* position. GNU
sed has a flag there called `e`:

> If a substitution was made, the command found in the pattern space is
> executed.

So a title like `x|e <command>` doesn't get written to a file. It gets
*run*. That's command injection, hiding in a "name your task" prompt.

## Why it's not a five-alarm fire (today)

Before you panic on my behalf: as the template actually ships, this
isn't reachable as RCE. Two accidents save it.

1. The title placeholder only appears on a markdown H1 line —
   `# SPEC-XXX: <Short Title>`. Even if the `e` flag fired, it would try
   to execute a line starting with `#`, which the shell treats as a
   comment. No-op.
2. macOS ships BSD sed, which has no `s///e` flag at all. A `|` in the
   title just makes sed error out and the command aborts.

I confirmed both with a proof-of-concept. So why fix it?

Because both protections are *accidents*, not defenses. Move the
placeholder to a non-comment line in some future template edit, run it on
a Linux box with GNU sed, and the bug is live. And even today, on every
platform, a title containing `|`, `&`, or `\` silently corrupts the
generated file. "Not exploitable yet, and quietly broken now" is not a
state I want to ship.

The fix is boring and correct: escape the replacement before it touches
`sed`.

```bash
sed_escape_replacement() {
    local s="$1"
    s=${s//\\/\\\\}   # backslash first
    s=${s//|/\\|}     # the delimiter
    s=${s//&/\\&}     # sed's whole-match reference
    printf '%s' "$s"
}
```

Pure bash, no dependency, escapes the three characters that matter. The
title now renders verbatim instead of executing. I added three
regression tests that feed a hostile title through and assert: the
command succeeds, no marker file is created, and the title appears
literally in the output.

## The part that actually changed my mind

Here's the thing that makes this worth writing about. In a normal CLI,
`just new-spec "..."` is typed by *you*. You're not going to inject a
shell command into your own machine through your own task title. The
classic threat model — "untrusted input crosses a trust boundary" — has
no untrusted input. That's why this class of bug sleeps for years in
local tooling.

But this template is **built to be driven by agents.** In the
two-agent variant, Claude reads a brief, decides the work breakdown, and
runs `just new-spec "<title it wrote>"`. The title is no longer something
*you* typed. It's something an agent *generated*, possibly influenced by
text the agent read — a pasted issue, an external brief, a dependency's
README.

That's the boundary. The moment an agent constructs the argument, a
prompt-injection in any document the agent ingested can become a
crafted CLI argument. "Create a spec titled `x|e curl …|sh`" buried in a
file the agent was told to summarize is no longer absurd — it's the
exact shape of the attack. The agent is the untrusted-input pathway the
local-tooling threat model assumed didn't exist.

So the rule I came away with, and wrote into the template's
`SECURITY.md`:

> Treat any repo content as untrusted if it originates outside your team
> — because an agent reads it, and an agent runs commands.

## What the audit *didn't* find

Worth saying, because a good audit reports the clean parts too. No
`eval`, no `curl | sh`, no `sudo`. No GitHub Actions workflows, which
deletes the entire `pull_request_target` script-injection class. The
`advance-cycle` command allowlists its inputs; `archive-spec` is
awk-only with IDs validated against real files; titles are slugified
before they become filenames, so no path traversal. The `.gitignore`
excludes `.env`, `*.pem`, `*.key`. The template is, on the whole, a
calm place.

## The takeaway

The audit found one bug, and the bug was almost nothing — a quoting slip
that BSD sed and a `#` character were quietly covering for. What it
*taught* was bigger: **agentic tooling silently promotes "the operator
types this" inputs into "untrusted input" inputs.** Every place your code
assumed a human at the keyboard is a place to re-check now that the thing
at the keyboard reads its instructions from files.

Escape your replacements. And re-read your "safe because local" code with
the assumption that an agent — and whatever it just ingested — is holding
the keyboard.
