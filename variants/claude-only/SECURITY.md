# Security

This repo was scaffolded from a spec-driven template. The security model
below comes with it; adapt the reporting section to your team.

## Threat model

This is local-first tooling — markdown, a `justfile`, and bash. No server,
no network calls, no secret handling beyond keeping secrets out of git.
The realistic risks:

1. **Untrusted arguments.** Commands like `just new-spec "<title>"` take
   free-form strings. User input is escaped before it's substituted into
   files, cycle values are allowlisted, and titles are slugified for
   filenames so they can't traverse paths.
2. **Untrusted repo content + agents.** This workflow is driven by coding
   agents. They **read** specs, decisions, briefs, and handoffs, and they
   **run** `just` commands. Treat any of that content as untrusted *if it
   originates outside your team* — a pasted issue, an external brief, a
   dependency's README. Such text can attempt prompt injection (steering
   the agent) or be passed verbatim into a command. Review what an agent
   proposes to run before you let it run.
3. **Secrets in git.** The shipped `.gitignore` excludes `.env*`, `*.pem`,
   and `*.key`, and `guidance/constraints.yaml` makes "no committed
   credentials" a blocking rule. Keep secrets in environment variables
   referenced via `.env.example`.

## Good habits

- Don't paste untrusted text into a brief/spec and then have an agent act
  on it unreviewed.
- Keep the `no-secrets-in-code` constraint enabled.
- If you add CI (e.g. GitHub Actions), scope `permissions` minimally and
  never interpolate `${{ github.event.* }}` into a `run:` block.

## Reporting a vulnerability

Replace this with your team's process. For a private repo, that's usually:
open a private issue or security advisory, describe the impact without a
public exploit, and coordinate a fix before disclosure.
