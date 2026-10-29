# 0.8.0 — JYSF.ORG PAGE prompt: bring the crustyimg project page up to 0.8.0

Cycle: **release follow-through** (the fourth public surface). Fresh CLI session, **Sonnet**, in
the **site repo, not crustyimg**.

The page is `src/pages/projects/crustyimg.astro` in the private repo **`jysf/home0`** (Astro). Its
tagline and description come from the `crustyimg` entry in `src/data/projects` (`.ts` or `.js`).
It is live at `https://www.jysf.org/projects/crustyimg/`.

Open the session in your local `home0` checkout. If you have none, run
`gh repo clone jysf/home0 <dir>` somewhere outside the crustyimg repo first. The maintainer pasted
this prompt into a fresh session and will paste your final message back to the crustyimg
orchestrator, so that message must stand on its own.

## Where things stand
- crustyimg **0.8.0** was tagged on 2026-10-28 (`v0.8.0` → `139b381`). Its release notes are the
  `## [0.8.0]` section of crustyimg's `CHANGELOG.md`. Read them at the tag:
  `gh api 'repos/jysf/crustyimg/contents/CHANGELOG.md?ref=v0.8.0' --jq .content | base64 -d`.
  Read crustyimg's `README.md` at `v0.8.0` the same way. The README was checked against the
  binary for this release; **this page was not.**
- This page is in no release checklist, so it drifts. Each sentence on it is a claim about the
  shipped tool.

## Read first
- `home0`'s own `README` / `AGENTS.md` / `CLAUDE.md`, if present, for its build, branch and deploy
  conventions. **Follow them** where they differ from this prompt, and say which ones you followed.
- `src/pages/projects/crustyimg.astro` in full (~209 lines), and the `crustyimg` entry in
  `src/data/projects`.
- crustyimg's `CHANGELOG.md` `## [0.8.0]` and `README.md` at `v0.8.0`, as above.

## Do, in order
1. **Inventory the claims.** List every sentence on the page, and in the `projects` entry, that
   states what crustyimg does, which version it is, or how to install it. Mark each **true at
   0.8.0**, **stale** or **false**, citing the CHANGELOG or README line that decides it. The
   orchestrator expects at least these, but confirm each one; don't trust the list:
   - **the install caption:** "v0.7.1." → `v0.8.0`;
   - **the `edit` / `apply` bullet:** "the same recipe toml also runs in the browser demo." At
     0.8.0 a recipe can carry a `watermark` step, and a recipe that needs an image or font file
     is refused in the browser. Use the wording crustyimg's README uses for this, shortened to
     the page's voice;
   - **the same bullet:** recipes can now also pin the output format and quality
     (`version = "2"`) and include a watermark step. Add one short clause;
   - **the `build` bullet:** at 0.8.0 the cache also rebuilds when a watermark's image or font
     file changes, and `--watch` watches those files. One clause at most;
   - **the `watermark` bullet:** 0.8.0 refuses a mark that would not fit the image. Add it only
     if it fits the list's one-line style;
   - "the full engine, compiled to webassembly" in the demo callout. Check it against the
     README's description of the browser demo, and soften it if the README is narrower.

   Leave historical statements alone if they are still true as history. For example, "v0.7.1
   moved every resample into linear light" is still true, and so are the numbers paragraph and
   the license line.
2. **Edit**, matching the page's style: all lower case, short sentences, no marketing words.
   Change only stale or false sentences. Do not add a changelog to the page; it describes the
   tool, not the release.
3. **Check it.**
   - Build the site the way the repo's README says (for example `npm run build`), and read `$?`.
   - Run the dev or preview server, open `/projects/crustyimg/` with the **built-in browser tool**
     (`preview_start` + `get_page_text`), and paste the rendered text of every section you
     changed. WebFetch returns 403 on jysf.org, so don't use it.
   - If crustyimg 0.8.0 is installed (`crustyimg --version`), run any command line you added or
     changed and record the exit code. If 0.8.0 isn't installed yet, say so; don't run 0.7.1 and
     call it a check.
4. **Land it.**
   - Make a branch and commit, using the repo's own convention for both.
   - Open a **PR**: never push to the default branch, and never deploy. The PR body holds the
     claim inventory table and the before → after of each changed sentence.
   - Don't merge it; the maintainer reviews and merges.

## Guardrails
- ⛔ Touch only the crustyimg page and its `projects` entry. Don't touch blog posts
  (`src/content/blog/crustyimg-*.md`): they are dated history.
- ⛔ No deploy, no merge, no change to the crustyimg repo.
- No spec or DEC ids on the page.
- A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~30 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — JYSF.ORG PAGE prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take input, cache_creation and cache_read once per id, and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the crustyimg orchestrator)
First line: `jysf page: PR jysf/home0#NNN at <head SHA> — build <ok | failed>`. Then, short, **with
the evidence inline**:
- the claim inventory (sentence → true / stale / false → source line);
- each changed sentence, before → after;
- the build exit code and the rendered text of the changed sections;
- which `home0` conventions you followed;
- your cost.
