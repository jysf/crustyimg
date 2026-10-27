# SPEC-134 — RE-VERIFY prompt: the punch-list fix on draft PR #202

Cycle: **verify** (re-verify of a punch-list fix). Fresh CLI session, own worktree, **Opus**,
**read-only**, small budget. You did not build SPEC-134, write its punch list, or fix it.

```
just wt verify/spec-134-fix -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-134-name-template-pins-format
```
PR #202 is a **draft**; its head was `a3547cc` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## What was verified before, and what the fix claims
SPEC-134 (a `--name-template` extension pins the output format; DEC-105) was verified at
`635adfb`. The code was sound on every check; the punch list was three doc fixes plus one code
fix the maintainer ruled in. Read `## Build Completion` in
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-134-name-template-extension-pins-the-output-format.md`
and the verify line in its timeline.

The fix is commit `a3547cc` alone. Before it is `d78d5d4`, a merge of `main`. It claims:
1. CHANGELOG: every literal-extension template on `web`, `optimize` and `apply --recipe web`
   takes the pinned encode, even when the extension matches the automatic choice.
2. `docs/cli-reference.md` (two places) and the `web` help text name a `--name-template`
   extension as an exit-2 trigger, alongside `-o` / `--format`.
3. `docs/api-contract.md`: the false sentence is now scoped to one input; at N inputs `-o` is
   ignored and the template names the file and pins its format.
4. **Code:** `name_pinned_format(global, input_count)` (`src/cli/common.rs`) reads only the
   template above one input; one input keeps `-o` > template. A new
   `resolved_input_count(inputs)` resolves every input argument to count them before the real
   resolution runs. Call sites: three in `optimize.rs` (`run_apply`, `run_optimize`, `run_web`)
   and one in `ops.rs` (passes `1`). Five new rows in `tests/name_template_pins_format.rs` (41 →
   46). Four were RED before the fix; the `web` N-input `-o x.jpg` row was already green. Plus a
   CHANGELOG Fixed entry and a DEC-105 Consequences bullet.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope:** `git diff 33912eb d78d5d4 -- src tests` is empty (the merge brings in only `main`'s
   docs), and `git diff --stat d78d5d4 a3547cc` is the files the fix names. Any file outside
   the fix prompt's boundary (`optimize.rs`, `ops.rs`) changes only what it needs to pass the
   count.
2. **The bug is gone, and only where it should be:** release binary, fresh `CARGO_TARGET_DIR`,
   sniff every output with `file`. Run on both `main` (`origin/main`) and the PR:
   - `apply --recipe web in.jpg in2.jpg -o x.jpg --out-dir d --name-template '{stem}.png'`
   - `web` and `apply --recipe web` with the same arguments but `-o x.bin`
   On `main`: JPEG or AVIF inside `in.png`. On the PR: PNG in both outputs. Then check the
   one-input rows kept today's order (`-o x.jpg` with a `.png` template writes JPEG to `x.jpg`).
3. **The new counting function, `resolved_input_count`:** it resolves inputs twice, once to
   count and once for real. Drive these edges and say what each does:
   - **a glob or directory that resolves to exactly one file**, with `-o x.jpg` and a `.png`
     template: is it treated as one input in both the count and the write? A mismatch would
     bring the mislabel back.
   - **stdin (`-`)**: the count passes an empty reader. Confirm stdin is read once, by the real
     resolution, and a piped image still works.
   - **a missing path**: the count treats it as one input. Confirm the real resolution still
     reports the same error and exit code.

   Then judge the design as a code-quality call (the maintainer's bar is "code quality is a
   must"). Could the pin decision move after the one real resolution, so nothing resolves
   twice? Name the concrete obstacle if not. Also: `refuse_a_name_asking_for_another_format`
   in `ops.rs` still has its own N-input template branch. Could it now call
   `name_pinned_format(global, all.len())`? Report these as ⚠ only if they are wrong, or likely
   to drift into a bug. Otherwise list them as optional cleanups.
4. **Mutation:** revert only the N-input branch in `name_pinned_format`, rebuild in a **fresh**
   target dir (reverting source does not rebuild the binary), confirm exactly the 4 rows go RED.
   Restore and rebuild: green. Separately, make `resolved_input_count` return `inputs.len()` and
   check whether any test notices. If none does, say whether a glob/directory row should exist.
5. **Docs:** items 1–3 are true as written. Check each sentence against the binary, not against
   the code. The `web --help` text matches `docs/cli-reference.md`, and other specs' CHANGELOG
   entries are untouched.
6. **CI:** `gh pr checks 202` at the head you verified, every leg. Do not poll.

## Guardrails
`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to a
file. A piped command reports the pipe's exit code; redirect and read `$?`. zsh does not
word-split unquoted parameters. macOS has no `timeout(1)`. The fix session filled the disk once
with target dirs: delete each of your scratch target dirs when you are done with it. **Budget ~50
exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-134 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Opus
$5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat. Report tokens_total, the
breakdown and estimated_usd.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #202 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #202 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then items 1–6, one or two lines each with evidence; optional cleanups listed separately; then
your cost.
