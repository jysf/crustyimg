# Security

`crustyimg` is a local-first command-line tool: it reads image files,
transforms them in memory, and writes outputs. No server, no network at
runtime, no secret handling beyond keeping secrets out of git. The realistic
risks come from processing untrusted inputs and from the spec-driven,
agent-run workflow this repo uses.

## Threat model

1. **Untrusted image inputs.** Image decoders parse attacker-controllable
   binary data. A malformed or hostile file could trigger a panic, excessive
   memory/CPU (decompression bombs), or a decoder bug. Mitigations: decode
   through the single `image` stack (DEC-002); **decode resource limits are set
   on the canonical load path** (`image::Limits` — `max_image_width`/
   `max_image_height` = 65 535, `max_alloc` = 512 MiB; SPEC-033 / DEC-034), so
   an over-dimension or over-allocation input is **rejected with a typed
   `ImageError::LimitsExceeded` (exit 1), not a panic or OOM**, before pixels
   are produced; never `unwrap()`/`expect()` on decode paths (constraint
   `no-unwrap-on-recoverable-paths`, DEC-007); failures surface as typed errors
   mapped to non-zero exit codes rather than crashes.
2. **Untrusted recipes.** A recipe is a TOML file describing an operation
   chain. Treat a recipe from outside your team as untrusted: it can only
   express registered operations (no code execution). Mitigations: the loader
   validates the `version` and rejects unknown operations / invalid params with
   typed errors (SPEC-006), and **bounds resources** — a recipe text over 64 KiB
   or with more than 1024 steps is rejected (`RecipeError::TooLarge` /
   `TooManySteps`, exit 1), and the `apply` read path refuses an over-size recipe
   *file* before reading it (SPEC-035 / DEC-036). *Known residual:* individual op
   params are not yet bounded (e.g. a `resize` to enormous dimensions) — tracked
   for the threat-model pass.
3. **Path traversal on output.** Batch output uses name templates
   (`{stem}_web.{ext}`) into `--out-dir`. Mitigations (SPEC-005 + SPEC-034 /
   DEC-035): `safe_join` rejects an expanded name that is absolute, contains a
   path separator, or contains `..`; the destination is **refused if it is a
   symlink** (`SinkError::Traversal`, exit 5) **even with `--yes`**, so a planted
   symlink in `--out-dir` cannot redirect a write outside it; and the tool does
   not overwrite an existing file without `--yes`. On input, directory/glob
   sources are non-recursive (DEC-010) and **skip any entry whose real path
   escapes the root** (symlink-escape check, always anchored — SPEC-034). Inputs
   are read-only unless explicitly targeted as output.
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

- Decode limits are set on load and input dimensions are not trusted
  (SPEC-033 / DEC-034); a future `--max-pixels`/env override can re-admit a
  legitimately huge image deliberately.
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
