# SPEC-159 — PUNCH LIST prompt: the exit-6 case and the AVIF sentence in three docs

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Wording only: verify
found no code or test to fix. You did not design, build or verify SPEC-159.

```
just wt fix/spec-159-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-159-pad-transparent:refs/remotes/origin/feat/spec-159-pad-transparent
git reset --hard origin/feat/spec-159-pad-transparent
```
You push to the PR branch with `git push origin HEAD:feat/spec-159-pad-transparent`. Draft PR
#262's head was `920b5ef` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove every message, the format rule on every surface, the widening and the byte pin with
its own decoder, and six mutations. The code and tests hold. Three user-facing sentences say a
refused recipe exits 4, but verify drove the other case: a batch (or `build` target) where each
input keeps its own format fails **that input** with exit 6 and writes the others. The full rule
is already in `docs/api-contract.md:481-483`:

> exit **4** for one input or for a batch with one format (`--format`, a name template's extension
> or `recipe.format`), nothing written; exit **6**, that input only, when each input keeps its own
> format

The sentences to fix (find each by content):
1. `docs/recipes.md:~120`: "so a recipe that would write one is refused for those formats (exit 4,
   nothing written): write PNG, WebP or AVIF."
2. `recipes/README.md:~298`: "is refused for that output (exit `4`); write PNG, WebP or AVIF."
3. `CHANGELOG.md:~22`: "A fill alpha JPEG or GIF can't store is refused for that output (exit 4)."

And one sentence that is true but too loose:
4. `docs/recipes.md:~121`: "AVIF keeps the alpha approximately". Verify read a `transparent` border
   back as alpha 5/255 and `#ff000080` as 116 (not 128) on a 4×4 image; on a 64×48 photo both were
   exact.

## The fixes (settled, do not re-decide)
1. In each of sentences 1–3, keep exit 4 and add the exit-6 case, in that file's voice: exit 4
   (nothing written) for one input or a batch with one format; when each input keeps its own
   format, that input fails and the batch exits 6. Keep each as short as you can; the CHANGELOG
   line should read to a user who never saw the spec.
2. Sentence 4: say AVIF keeps the alpha approximately, and on very small images it can be off by a
   few levels (for example a `transparent` border read back as alpha 5 of 255 on a 4×4 image).
   One clause, no more.
3. No spec, stage or DEC id in any changed line.

Stay in these files: `docs/recipes.md`, `recipes/README.md`, `CHANGELOG.md` and the SPEC-159 spec
(Build Completion only). `git diff --stat 920b5ef` must show nothing else.

**Not yours:** DEC-115 (two wording fixes) and the design's mutation-table cells. Both are on
`main` and ship fixes them. Don't edit them.

## Check it
On a release build of the PR (a fresh `CARGO_TARGET_DIR`), with a recipe
`[[step]] op = "pad"`, `border = 0.25`, `color = "transparent"`:
- `apply` over `src.jpg` + `rgb.png` (4×4 each) with `--out-dir kb` and no format: quote the exit code,
  stderr, and which files exist in `kb`. It should be exit 6 with `rgb.png` written.
- The same two inputs with `--format jpeg --out-dir kj`: exit 4, one line, no `kj`.
- A 4×4 RGB PNG to `-o t.avif`: read the corner alpha back with ImageMagick
  (`magick t.avif -format '%[pixel:p{0,0}]' info:`) and check your sentence 4 still holds. Quote it.
- `cargo test --test docs_ops` green (it resolves doc examples), stdout redirected, `$?` read.
- `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-'` on your changed lines: none.
- No code changes, so no mutation runs and no matrix. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. The only
  CHANGELOG edit is fix 1's sentence.
- Rebase onto `main` only if the push is rejected; never force-push over someone else's commit.
- **Never poll CI.** Run `gh pr checks 262 --watch` once in the background after your push, and
  don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters, and `"$R:recipes/…"` needs braces (`${R}:`) or zsh reads `:r` as a
  modifier.
- Never share a `CARGO_TARGET_DIR` with another session; delete yours when you're done. Write
  scratch files in your session's scratchpad, never `/tmp` or the system temp dir.
- **Budget ~20 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`: the four
  sentences as they now read, the three drives with their quoted output, and the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-159 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25; cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-159 punch list: PR #262 at <head SHA> — CI <green | red: legs>`.

Then, short:
- `git diff --stat 920b5ef`;
- each changed sentence, quoted as it now reads;
- the three drives, with quoted output;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
