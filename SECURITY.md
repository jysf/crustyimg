# Security

`crustyimg` is a local-first command-line tool: it reads image files,
transforms them in memory, and writes outputs. No server, no network at
runtime, no secret handling beyond keeping secrets out of git. The realistic
risks come from processing untrusted inputs and from the spec-driven,
agent-run workflow this repo uses.

## Threat model

1. **Untrusted image inputs.** Image decoders parse attacker-controllable
   binary data. A malformed or hostile file could trigger a panic, excessive
   memory/CPU (decompression bombs — `image` exposes dimension/byte limits we
   should set), or a decoder bug. Mitigations: decode through the single
   `image` stack (DEC-002), bound resource use, never `unwrap()`/`expect()`
   on decode paths (constraint `no-unwrap-on-recoverable-paths`, DEC-007),
   and surface failures as typed errors mapped to non-zero exit codes rather
   than crashes.
2. **Untrusted recipes.** A recipe is a TOML file describing an operation
   chain. Treat a recipe from outside your team as untrusted: it can only
   express registered operations (no code execution), but validate the
   `version` field and reject unknown operations rather than guessing.
3. **Path traversal on output.** Batch output uses name templates
   (`{stem}_web.{ext}`) into `--out-dir`. Templates must not contain path
   separators that escape `--out-dir`, and the tool must not overwrite an
   existing file without `--yes`. Inputs are read-only unless explicitly
   targeted as output.
4. **Metadata leakage / privacy.** Image metadata can contain GPS location,
   device serials, and personal info. The default-preserve policy (DEC-003)
   **drops GPS** on pixel-lane encodes unless `--keep-gps`; `clean --gps` and
   `strip` exist precisely for privacy. Don't silently retain location data.
5. **Untrusted repo content + agents.** This workflow is driven by coding
   agents that read specs/decisions/briefs and run commands. Treat content
   originating outside your team (a pasted issue, an external brief) as
   untrusted — it can attempt prompt injection. Review what an agent
   proposes to run before letting it run.
6. **Secrets in git.** `.gitignore` excludes `.env*`, `*.pem`, `*.key`, and
   `guidance/constraints.yaml` makes "no committed credentials" a blocking
   rule (`no-secrets-in-code`). `crustyimg` itself needs no secrets.

## Good habits

- Set decode limits and don't trust input dimensions.
- Keep the `no-secrets-in-code` and `no-unwrap-on-recoverable-paths`
  constraints enabled.
- When wiring CI (GitHub Actions, DEC-009), scope `permissions` minimally
  and never interpolate `${{ github.event.* }}` into a `run:` block.
- Don't paste untrusted text into a brief/spec and then have an agent act on
  it unreviewed.

## Reporting a vulnerability

Replace this with the project's process. For now: open a private security
advisory (or private issue) describing impact without a public exploit, and
coordinate a fix before disclosure.
