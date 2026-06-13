# Contributing

This is a template you can fork and adapt freely. If you want to send
changes back, here's what keeps the template coherent.

## Design principles (non-negotiable)

These are the constraints that make the template what it is. A change
that breaks one of them is almost always the wrong change:

- **Zero runtime dependencies.** Markdown, a `justfile`, and pure bash.
  No package to install to use it. Optional external tools are
  documented in `guidance/recommended-tools.md`, never required.
- **Bash 3.2 compatible.** macOS ships bash 3.2. No `mapfile`/`readarray`,
  no associative arrays (`declare -A`), no `\x01`-style escapes in `sed`.
  Build arrays with `while IFS= read -r`; do id→file lookups with
  parallel arrays + a linear scan.
- **Portable shell.** Scripts run on both BSD (macOS) and GNU (Linux)
  `sed`/`date`/`stat`. When they differ, branch on `uname` (see the
  date helpers in `scripts/_lib.sh`).
- **Escape user input.** Anything user-supplied that gets substituted
  into a file goes through `sed_escape_replacement` first. See
  `SECURITY.md`.
- **Both variants stay in parity.** `claude-only` and
  `claude-plus-agents` are kept in sync. A change to one variant's
  `AGENTS.md`, templates, or docs almost always needs the mirror edit
  in the other.

## Development loop

```bash
just test        # the end-to-end suite (init → full cycle → reports → audits)
```

`just test` scaffolds a throwaway repo in a temp dir and runs the real
commands against it. It must stay green. When you add or change a
recipe:

1. Add coverage to `scripts/test.sh` (assert behavior, not just exit 0).
2. Update the command table in `README.md`.
3. Add a `CHANGELOG.md` entry — one per change, newest at the top, with
   a short version tag.
4. If the change is user-facing, reflect it in the relevant `AGENTS.md`
   sections of **both** variants.

## Style

- No trailing whitespace; every file ends with a newline.
- Comments explain *why*, not *what*. No dead code — delete it.
- Match the surrounding script's idioms (`_lib.sh` helpers, the
  `info`/`warn`/`die`/`success` output helpers, `set -euo pipefail`).
- Commits: conventional prefixes (`feat`/`fix`/`docs`/`refactor`/`test`)
  with a scope, e.g. `feat(decisions): …`.

## Where things live

- `scripts/` — the daily commands; `_lib.sh` is the shared library.
- `variants/claude-only/`, `variants/claude-plus-agents/` — the two
  scaffolds `just init` copies to the repo root.
- `justfile` — recipes; works both before init (`init`, `list-variants`)
  and after (everything else).
- `docs/`, `PROJECTS.md`, `SECURITY.md`, `CHANGELOG.md` — the template
  project's own docs (not copied into generated repos).
