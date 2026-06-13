# Security

This policy covers this template repository **and any repository
generated from it** — the scripts, the `justfile`, and the conventions
are shared, so the security model is the same downstream.

## Threat model

This is local-first developer tooling. There is no server, no network
calls, no deserialization, and no secret handling beyond keeping secrets
out of git. The realistic risks are:

1. **Untrusted arguments.** Commands like `just new-spec "<title>"` take
   free-form strings and substitute them into files. User input is
   escaped before substitution (see `sed_escape_replacement` in
   `scripts/_lib.sh`); cycle values are allowlisted; titles are
   slugified for filenames so they can't traverse paths.
2. **Untrusted repo content + agents.** This template is designed to be
   driven by coding agents (Claude Code, and in the `claude-plus-agents`
   variant a separate implementer). Agents **read** specs, decisions,
   briefs, and handoffs, and they **run** `just` commands. Treat any of
   that content as untrusted *if it originates outside your team* — a
   pasted issue, an external brief, a dependency's README. Malicious
   text can attempt prompt injection (steering the agent) or can be
   passed verbatim into a command. Review what an agent proposes to run.
3. **Secrets in git.** The shipped `.gitignore` excludes `.env*`,
   `*.pem`, and `*.key`, and the `no-secrets-in-code` constraint in
   `guidance/constraints.yaml` makes "no committed credentials" a
   blocking rule. Keep secrets in environment variables referenced via
   `.env.example`.

## What has been hardened

- **Substitution injection (v5.8).** `new-spec` / `new-stage` route
  user-supplied titles and the repo id through `sed_escape_replacement`
  before `sed` substitution, so a title containing `|`, `&`, or `\`
  cannot corrupt the command or reach GNU sed's `s///e` execute flag.
- **Input validation.** `advance-cycle` allowlists cycle values;
  `archive-spec` resolves IDs against existing files and uses `awk`
  (data, never `eval`).
- **No CI attack surface.** There are no GitHub Actions workflows, so
  there is no `pull_request_target` / script-injection class to manage.
  If you add workflows, scope `permissions` minimally and never
  interpolate `${{ github.event.* }}` into a `run:` block.

## Known, accepted low-severity items

- **Report rendering.** `just report-daily` / `report-weekly` embed spec
  titles and content into fenced markdown. A title containing a code
  fence can break out of the block in the generated report. This is
  cosmetic and local; reports are not executed.

## Reporting a vulnerability

If you find a security issue in the template's scripts or conventions,
please report it privately: open a **GitHub Security Advisory**
(repository → **Security** → **Report a vulnerability**) rather than a
public issue. If advisories aren't available, open an issue that
describes the impact without a working exploit and we'll coordinate a
fix. There is no bounty; thank-yous are sincere.
