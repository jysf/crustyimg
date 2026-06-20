# SPEC-040 build prompt — README user-facing rewrite + clap shell completions

Start a **fresh session**. You are the IMPLEMENTER for SPEC-040 in the `crustyimg`
repo (cwd is the repo root). This is **docs + a small, self-contained code addition**:
rewrite `README.md` into a tool-first landing page, and add a `completions <SHELL>`
subcommand that prints a clap-generated completion script to stdout. **No git tag, no
`cargo publish`, no release — and do NOT create the Homebrew tap.** Open a PR and STOP.
Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-040-readme-rewrite-and-shell-completions.md`
   — the whole spec, especially `## Acceptance Criteria`, `## Failing Tests`,
   `## Notes for the Implementer`.
2. `decisions/DEC-039-clap-complete-shell-completions.md` — the dep + the **exact,
   probe-verified API** (copy it). Subcommand over build.rs; stdout only.
3. `docs/api-contract.md` — the real command surface / flags / `-` pipe / exit codes
   for the README usage examples (use real commands, real flags).
4. `docs/moat.md` — the one-paragraph "what crustyimg is / why" framing.
5. `Cargo.toml` — `version = "0.1.0"`, `license = "MIT OR Apache-2.0"`, the
   `[features]` block (`display` default-on DEC-027; `webp-lossy`/`avif` opt-in).
6. `README.md` (current — the spec-workflow README you are restructuring) and
   `tests/cli.rs` (the binary-invocation harness to mirror for the new tests).

## Part A — shell completions (code, TDD)

Per `test-before-implementation`, **write the failing tests first**, then the handler.

1. **`tests/completions.rs`** (new) — mirror `tests/cli.rs`: drive the real binary via
   `Command::new(env!("CARGO_BIN_EXE_crustyimg"))` (NO `assert_cmd` dep). Tests:
   - `completions_bash_emits_script` — `completions bash` → exit 0, stdout non-empty,
     contains `crustyimg` and `_crustyimg`.
   - `completions_all_shells_succeed` — loop over `bash, zsh, fish, powershell,
     elvish`: each exit 0, non-empty stdout.
   - `completions_rejects_unknown_shell` — `completions klingon` → exit 2 (clap usage).
   - `completions_needs_no_input_path` — `completions zsh` succeeds with NO positional
     path and writes no file.
2. **`Cargo.toml`** — add to `[dependencies]` (default, non-optional), with a short
   comment referencing **DEC-039** in the style of the existing dep comments:
   ```toml
   clap_complete = "=4.6.5"
   ```
3. **`src/cli/mod.rs`** — add the subcommand + dispatch + handler (clap is isolated
   here per DEC-012; keep it here):
   - A `Commands` variant with a `///` doc line like the others:
     ```rust
     /// Generate a shell-completion script (bash, zsh, fish, powershell, elvish) to stdout.
     Completions { shell: clap_complete::Shell },
     ```
   - A `dispatch()` arm: `Commands::Completions { shell } => run_completions(*shell),`
   - The handler (exact API from DEC-039):
     ```rust
     use clap::CommandFactory; // top of file with the other clap imports
     use clap_complete::generate;

     fn run_completions(shell: clap_complete::Shell) -> Result<(), CliError> {
         let mut cmd = Cli::command();
         generate(shell, &mut cmd, "crustyimg", &mut std::io::stdout());
         Ok(())
     }
     ```
     No `GlobalArgs`, no input path, no image/`Sink` work. Write to stdout directly.
   - Optionally extend `tests/cli.rs::help_lists_all_subcommands`' expected list with
     `"completions"` (it uses `contains`, so existing assertions still pass).

## Part B — README rewrite (docs)

Rewrite `README.md` to lead with the **tool**, in this order:

1. **Title + one-line description** (what crustyimg is — from `docs/moat.md`).
2. **Install** — document **cargo**, **prebuilt binary (GitHub Releases)**, and
   **Homebrew**, AND a works-today path. Be **honest**: the tap, crates.io publish,
   and Releases download do **not exist yet** (backlog #3/#4/#5):
   - Works today: `cargo install --git https://github.com/jysf/crustyimg` (or clone +
     `cargo build --release`).
   - Mark the rest clearly, e.g. under "**Once v0.1.0 is published:**":
     `cargo install crustyimg`, `brew install jysf/tap/crustyimg`, download from the
     Releases page. Do NOT claim these work now. Cross-reference `RELEASING.md`.
   - A short **feature note**: `view` (terminal display) is on by default (DEC-027);
     headless/CI build with `cargo install ... --no-default-features`; `webp-lossy`
     and `avif` are opt-in features.
3. **Usage** — a tight quickstart with **≥3 real, correct examples** from
   `docs/api-contract.md`, e.g. `crustyimg view photo.jpg`, `crustyimg shrink in.jpg
   --max 1200 -o out.webp`, `crustyimg info photo.jpg`, and a `-` pipe
   (`crustyimg resize - --max 800 -o - < in.jpg > out.jpg`). Point to `crustyimg
   --help` for the full surface — don't dump every command.
4. **Shell completions** — show `crustyimg completions <shell> > <dest>` for the five
   shells (a couple of concrete examples, e.g. zsh + bash), noting that installing the
   script into the shell's completion dir is the user's step.
5. **Changelog & releases** — keep/relocate the existing pointer to `CHANGELOG.md` /
   `RELEASING.md`.
6. **License** — **correct it**: `crustyimg` is dual-licensed **`MIT OR Apache-2.0`**
   (see `LICENSE-MIT` and `LICENSE-APACHE`). Remove the stale "Licensed under the
   Apache License, Version 2.0. See `LICENSE`." line.
7. **Developing crustyimg** (secondary, near the bottom) — KEEP the spec-driven dev
   process content (hierarchy diagram, the `just` commands, the discipline notes,
   "Where things live"); relocate it here, or condense it with a pointer to
   `AGENTS.md` / `GETTING_STARTED.md`. Do not delete the process docs — just stop
   leading with them.

## Hard rules
- **No outward-facing action.** Do NOT run `git tag`, `cargo publish`, `gh release`,
  or create the `jysf/homebrew-tap` repo. Do NOT install completions into system dirs
  or commit static completion files (stdout only — a `build.rs` approach was rejected
  in DEC-039).
- The only new top-level dep is `clap_complete =4.6.5` (justified by DEC-039). Do not
  add any other dependency. Do not change `image`/codec features.
- The `completions` path must compile and work in **both** the default and the lean
  `--no-default-features` build (clap_complete is non-optional).
- `DEC-039` is already authored by the architect — do NOT create a new DEC. Only set
  its `session_id` if you record one; otherwise leave it untouched.

## Gates (all must pass)
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test                                   # incl. the new tests/completions.rs
cargo build --no-default-features            # lean build: completions still compiles
cargo deny check advisories bans sources licenses   # clap_complete is MIT OR Apache-2.0 — must stay green
```
Sanity: `crustyimg completions zsh | head` prints a script; `git tag` shows NO new tag.

## Git / PR
- Branch `feat/spec-040-readme-completions` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked
  `reports/*.md` and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- If a later `cargo fmt` reformats already-committed files, re-add ALL touched files
  (`git add -u`) before committing, or CI fmt fails though local `--check` passed.
- PR title: `feat(SPEC-040): user-facing README + shell completions`.
- PR body per AGENTS.md §13: Decisions referenced (DEC-039, DEC-012, DEC-027, DEC-038);
  Constraints (no-new-top-level-deps-without-decision satisfied by DEC-039; clippy-fmt
  -clean; every-public-fn-tested; no-agpl-default-deps); New decisions (none — DEC-039
  pre-authored).
- Fill the spec's `## Build Completion` + the 3 build-reflection answers; append the
  build cost session entry below (agent `claude-sonnet-4-6`; leave numerics null —
  the orchestrator fills real `subagent_tokens` at ship).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "code+docs: clap_complete =4.6.5 (DEC-039) + `completions <shell>` subcommand (stdout, 5 shells) with tests/completions.rs; README rewritten tool-first (install cargo/release/brew honestly labeled + works-today path, usage quickstart, completions, License corrected to MIT OR Apache-2.0, dev-process relocated). No tag/publish/tap. fmt/clippy/test/lean/deny green."
```

## When done
`just advance-cycle SPEC-040 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
