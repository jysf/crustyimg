# Known Limitations

Things the hardening pass on 2026-04-20 explicitly left unfixed. Each
has a reason — none of these is a surprise bug.

## No `just new-project` command

Creating `PROJ-002`, `PROJ-003`, etc. requires manually creating the
directory and copying `projects/_templates/project-brief.md` into
`projects/PROJ-NNN-<slug>/brief.md`. In practice Claude does this
during the PROJECT BRIEF step (see `GETTING_STARTED.md`), so the gap
rarely shows up in the documented flow.

If you add this command later, give it the same shape as `new-stage`:
accept a title, next-id a PROJ ID, copy the template, substitute
`PROJ-XXX` and `__TODAY__`.

## `new-spec` and `archive-spec` don't auto-update the parent stage backlog

The stage's `## Spec Backlog` markdown list and the `**Count:**` line
are written by a human (or by Claude). The scripts print a reminder
but don't edit the stage file. This is deliberate — markdown list
formatting is judgment-laden and a script that gets it wrong is worse
than one that doesn't try.

## `just init` is interactive-only

The recipe uses `read` to ask which variant to use. This works in a
normal terminal but breaks in CI, piped shells, or any non-TTY
context. The intended use (one human clicks "Use this template" and
runs `just init` once) doesn't hit this, so it's left as-is.

## No cross-platform CI

`just test` runs locally on whoever runs it. There's no GitHub Actions
workflow that exercises both macOS and Linux. The scripts have
`uname = Darwin` branches for `stat` and for `sed -i`, and those are
tested under `just test` on whichever OS runs the test — but drift
between the two branches is possible.

## Scripts assume 3-digit zero-padded IDs

`next_id` formats as `%03d`. If someone manually creates `SPEC-0001`
or `SPEC-10000`, behavior is undefined. The projected worst case at
normal scale is one project with >999 specs, which would mean the
stage-and-spec hierarchy is being misused.

## `get_active_project` uses lexical-first heuristic

When multiple `PROJ-*` directories exist (not counting the example),
`get_active_project` picks the lexically first one. Override with
`export ACTIVE_PROJECT=PROJ-NNN-slug`. This is shown by `just info`
but not prominently documented elsewhere.

## Templates exist in two copies (one per variant)

`claude-only/` and `claude-plus-agents/` share most files. Changing a
shared file means editing it in both places. No symlink or generation
system exists. The user chose not to fix this in the hardening pass;
it's a maintainability hazard to watch over time.
