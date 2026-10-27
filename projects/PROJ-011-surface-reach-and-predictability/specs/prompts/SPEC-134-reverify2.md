# SPEC-134 — RE-VERIFY 2 prompt: fix 2 on draft PR #202

Cycle: **verify** (re-verify of a small second fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build, fix or verify SPEC-134 before.

```
just wt verify/spec-134-fix2 -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-134-name-template-pins-format
```
PR #202 is a **draft**; its head was `ca8a961` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## Background
SPEC-134 makes a `--name-template` extension pin the output format (DEC-105). With several inputs,
`-o` is ignored and the template names every output, so `name_pinned_format(global, input_count)`
(`src/cli/common.rs`) reads only the template. `input_count` comes from `resolved_input_count`,
which expands globs and directories. The previous re-verify at `a3547cc` found everything correct
except one gap. Every N-input test row used literal paths, so mutation **M2**
(`resolved_input_count` returns `inputs.len()`) left the suite green. Under M2,
`web 'in*.jpg' -o x.bin --out-dir d --name-template '{stem}.png'` wrote AVIF into `in.png`.

Fix 2 is commit `ca8a961` alone; `1cbc685` before it merges docs-only `main`. It claims:
1. Three rows in `tests/name_template_pins_format.rs` pass one unexpanded glob `in*.jpg` (two
   inputs): `web -o x.bin`, `apply --recipe web -o x.jpg`, `optimize -o x.bin`. Each asserts PNG
   in `d/in.png` and `d/in2.png` and no `x.*`. Plus two `meta strip -o x.png --name-template
   '{stem}.jpg'` rows (literal paths and a glob): `-o` ignored, JPEG written. Row count 46 → 51.
   Under M2, exactly the 3 glob rows went RED.
2. A cleanup with no behaviour change. `refuse_a_name_asking_for_another_format`
   (`src/cli/ops.rs`) now calls `name_pinned_format(global, all.len())` for both one and N
   inputs. `via` is `-o` only when `all.len() == 1 && global.output.is_some()`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report what you find; do not fix it.

## Check, in order
1. **Scope:** `git diff a3547cc 1cbc685 -- src tests` is empty. `git diff --stat 1cbc685
   ca8a961` is the test file, `src/cli/ops.rs` and the SPEC-134 spec only.
2. **M2 yourself:** apply M2 and rebuild in a **fresh** `CARGO_TARGET_DIR` (reverting source does
   not rebuild the binary). Run the table test: exactly the 3 glob rows go RED. Restore, rebuild,
   GREEN. Confirm the glob reaches the binary unexpanded: the test's `Command` has no shell. Then
   check the rows would see the bug: the RED message names the wrong format inside `in.png`.
3. **The `ops.rs` cleanup is equivalent:** compare old and new over every case. One input with
   `-o x.png`, `-o -`, `-o x.bin` (unrecognised), and no `-o` with `--out-dir` + template. N
   inputs with and without a template, and with `-o` set. Same `(format, via)` or `None` each
   time? Drive at least the one-input `-o -` and `-o x.bin` cases on a release binary of `meta
   strip`, and read the refusal wording where one fires.
4. **Nothing else moved:** the full default suite passes (`--no-fail-fast`, fresh target dir).
   The spec's row counts match the test's assertion.
5. **CI:** `gh pr checks 202` at the head you verified, every leg. Do not poll.

## Guardrails
`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to a
file. A piped command reports the pipe's exit code; redirect and read `$?`. zsh does not
word-split unquoted parameters. macOS has no `timeout(1)`. Delete each scratch target dir when
done (a fix session filled the disk). **Budget ~30 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-134 — RE-VERIFY 2 prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by
`.message.id` (input / cache_creation / cache_read once per id, MAX output_tokens), price per
component (Opus $5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat. Report
tokens_total, the breakdown and estimated_usd.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #202 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #202 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then items 1–5, one or two lines each with evidence, then your cost.
