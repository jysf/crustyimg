# SPEC-137 — PUNCH LIST prompt: make the Linux system-font row prove it ran

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Small and mechanical:
one test helper, one CI job and three text fixes. You did not design, build or verify SPEC-137.

```
just wt fix/spec-137-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-137-svg-font-family:refs/remotes/origin/feat/spec-137-svg-font-family
git reset --hard origin/feat/spec-137-svg-font-family
```
You push to the PR branch with `git push origin HEAD:feat/spec-137-svg-font-family`. Draft PR
#217's head was `fb09421` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify passed everything except one item. `an_installed_system_font_is_never_read` (in
`tests/svg_font_family.rs`) proves that a family naming an installed system font still draws in
the bundled Go font.
- **macOS and Windows:** `host_font()` panics if Helvetica or Arial is missing, so a pass there
  proves the row ran.
- **Linux:** a missing font prints a SKIP line and returns. libtest hides stderr for passing
  tests, so the CI log for run `40835877119` shows only `... ok`. There is no way to tell whether
  the row ran or skipped. A silent pass is a defect.

## The fix (settled, do not re-decide)
1. **`host_font()` in `tests/svg_font_family.rs`:** on Linux, when no candidate exists **and** the
   environment variable `CRUSTYIMG_REQUIRE_SYSTEM_FONT` is set (to any value), panic with the
   candidate list, as macOS and Windows already do. Without the variable, keep today's SKIP line.
   - Use a dedicated variable, **not** `CI`. Every Ubuntu job (lean, avif, webp-lossy, heic) runs
     this test and sets `CI`. Not every one of them installs a font, and they must not go red
     because of it.
   - Add one comment saying why the variable exists: CI sets it on the step that must prove the
     row ran.
2. **`.github/workflows/ci.yml`, the `build / test / clippy / fmt` matrix job only:**
   - before `Test`, add a step for `ubuntu-latest` only that runs
     `sudo apt-get update && sudo apt-get install -y fonts-dejavu-core`. It needs the same
     `needs.changes.outputs.code == 'true'` guard as the other steps, combined with
     `matrix.os == 'ubuntu-latest'`;
   - after `Test`, add a step on all three OSes, with the same `code == 'true'` guard, that runs
     `cargo test --test svg_font_family an_installed_system_font_is_never_read -- --exact --show-output`
     with `env: CRUSTYIMG_REQUIRE_SYSTEM_FONT: "1"`. Name it so its purpose reads in the Actions
     UI, e.g. `System-font row (prints the font it used)`;
   - change nothing else in the workflow.
3. **Text cleanups from verify** (no behaviour change):
   - in the spec, the blank-render hash `bb918147…` should read `bd30b081…`. Fix every
     occurrence in the spec only; the timeline lives on `main` and is not yours;
   - the spec gives the mixed-`<tspan>` row on `main` as pixel hash `6893dab3…`. Verify measured
     `1accc9c7…`, using the same recipe that reproduces `6e4d6b08…` for the Go render. Replace it,
     and say in Build Completion that verify measured it;
   - the `opt.font_family` assertion in the unit test `hardened_options_hold_only_the_bundled_face`
     (in `src/image/svg.rs`'s `#[cfg(test)]` module) has no message. Give it one, behaviour-first.

Stay in these files: `tests/svg_font_family.rs`, `.github/workflows/ci.yml`, `src/image/svg.rs`
(test module only) and the SPEC-137 spec. `git diff --stat` must show nothing else.

## Prove it can fail
Run these one at a time and record each result.
- **Without the variable,** on any OS, the row passes as before. Run the full
  `cargo test --test svg_font_family` with stdout redirected to a file, and read `$?`.
- **The Linux panic.** You can't reach the Linux branch from macOS, so use Docker. In a
  `rust:1-slim` container, in a fresh `CARGO_TARGET_DIR` inside the container, with the worktree
  mounted read-only and no fonts installed, run the row twice:
  - without the variable it must print the SKIP line and pass;
  - with `CRUSTYIMG_REQUIRE_SYSTEM_FONT=1` it must panic, naming the candidates.
  Then `apt-get install -y fonts-dejavu-core` in the same container and run it once more with
  the variable set. It must print `using DejaVu Sans from …` and pass. Quote all three lines.
  If Docker is unavailable, say so. Do not skip the control silently.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Before pushing, run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
- **Never poll CI.** Run `gh pr checks 217 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA.
- Then read the new step's log on each of the three matrix legs once, with
  `gh run view <run> --log --job <job>`, and quote the `using …` line from each. If a leg prints
  SKIP or fails, report it as it is. Do not loosen anything.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Delete scratch target dirs and containers when you're done.
- **Budget ~30 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It lists
  the fix, the three control lines and the three CI `using …` lines.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-137 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
    that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
    **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
    output under-counts about 11×.
  - Price each component at the model that actually ran (read `.message.model`), never at a flat
    rate: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and
    cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-137 punch list: PR #217 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat` against `fb09421`;
- the three Docker control lines;
- the `using …` line from each CI leg (ubuntu, macos, windows), with the run id;
- the three text cleanups done;
- anything that did not go as written;
- your measured cost.
