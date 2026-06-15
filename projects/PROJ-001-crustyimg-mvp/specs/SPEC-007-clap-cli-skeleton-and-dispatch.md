---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-007
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6 (separate session)
  created_at: 2026-06-14

references:
  decisions:
    - DEC-012   # clap (derive) as the CLI framework (NEW — written with this spec)
    - DEC-007   # anyhow at the binary boundary + exit-code mapping
    - DEC-002   # single Image model + Operation/Pipeline the CLI drives
    - DEC-005   # recipe TOML + registry the `apply` path runs
    - DEC-006   # --jobs is a parsed placeholder here (parallel batch is STAGE-005)
  constraints:
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-002   # Image / load
    - SPEC-003   # Operation / Pipeline
    - SPEC-004   # Source
    - SPEC-005   # Sink
    - SPEC-006   # Recipe + registry

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Turns the shipped library (Image/Operation/Pipeline/Source/Sink/Recipe) into a usable `crustyimg` binary with a real subcommand interface and dispatch — completing STAGE-001's 'a runnable binary with a real subcommand interface'."

# Self-reported AI cost per cycle.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-06-14
      notes: "subagent; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-007: clap CLI skeleton and dispatch

## Context

This is the **last spec in STAGE-001** — it completes the foundation stage by
turning the shipped library into a real binary.

SPEC-001 shipped a std-only scaffold with a hand-rolled `argv` match in
`main.rs` that only understands `--version`/`--help`. SPEC-002 through SPEC-006
then shipped the full library core: the canonical `Image` + `load`
(SPEC-002), the `Operation` trait + `Pipeline` decode-once executor
(SPEC-003), the `Source` abstraction (SPEC-004), the `Sink` abstraction
(SPEC-005), and the `Recipe` TOML (de)serialization + operation registry
(SPEC-006). All of these are public APIs with **no caller** — nothing wires
them into a command-line program.

SPEC-007 builds that wiring. It **replaces** the SPEC-001 hand-rolled `main.rs`
with a real `clap` (derive) subcommand CLI under `src/cli/`, plus a thin
`main.rs` that parses arguments, dispatches into `cli`, and maps typed library
errors to the exit codes in `docs/api-contract.md` (DEC-007). This is the
keystone called out in `STAGE-001` Success Criteria: "`crustyimg --help` lists
subcommands; `crustyimg <cmd>` dispatches into the pipeline (commands may be
stubs that report 'not yet implemented')."

`clap` is a **new top-level dependency**, so this spec is designed against the
new **DEC-012** (clap as the CLI framework), which satisfies the
`no-new-top-level-deps-without-decision` constraint.

This stage deliberately ships with **zero real image *operations*** — but the
plumbing must be proven end to end. So SPEC-007 wires **one** command for real
(`apply --recipe`) and stubs the rest. See Goal.

## Goal

Replace the SPEC-001 argv `main.rs` with a `clap`-derive subcommand CLI
(`src/cli/`) covering the full MVP surface + global options from
`docs/api-contract.md`, where `main()` returns `ExitCode` and maps typed
errors to the contract's exit codes (DEC-007). Wire **`apply --recipe <file>
<input>`** as a real end-to-end path (Source → `Image::load`/`from_bytes` →
Recipe → Pipeline → Sink) to prove the plumbing; every other subcommand parses
its documented args and dispatches to a stub handler that returns a typed
"not yet implemented" error mapped to a non-zero exit.

## Inputs

- **Files to read:**
  - `docs/api-contract.md` — THE primary source: the global-options table, the
    full subcommand surface and each command's documented args, stdin/stdout
    (`-`) behavior, and the exit-code table (0–6).
  - `docs/architecture.md` — the `cli/` module + `main.rs` responsibilities and
    the layering rule (cli → recipe/source/sink/pipeline; pixel core stays
    clap-free).
  - `decisions/DEC-012-clap-cli-framework.md` — the new CLI-framework decision.
  - `decisions/DEC-007-error-handling-thiserror-anyhow.md` — anyhow at the
    boundary + exit-code mapping.
  - `decisions/DEC-006-no-async-runtime-rayon-for-batch.md` — `--jobs` is a
    parsed placeholder; no rayon here.
  - `guidance/constraints.yaml` — `ergonomic-defaults`,
    `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, etc.
- **Related code paths (the shipped APIs the real `apply` path calls):**
  - `src/image/mod.rs` — `Image::load(path)`, `Image::from_bytes(&[u8])`.
  - `src/source/mod.rs` — `source::resolve(arg, &mut reader) -> Result<Vec<Input>, SourceError>`,
    `Input::{path, stem}`.
  - `src/recipe/mod.rs` — `Recipe::from_toml(&str)`, `Recipe::build_pipeline(&registry)`.
  - `src/operation/registry.rs` — `OperationRegistry::with_builtins()`.
  - `src/pipeline/mod.rs` — `Pipeline::run(img) -> Result<Image, OperationError>`.
  - `src/sink/mod.rs` — `Sink`, `SinkInput`, `Overwrite`, `Sink::write(...)`.
  - `src/main.rs`, `src/lib.rs`, `Cargo.toml` — current state to modify.

## Outputs

- **Files created:**
  - `src/cli/mod.rs` — the clap-derive types (`Cli`, `GlobalArgs`, the
    `Commands` enum + per-command arg structs), the `run()` entry the binary
    calls, the typed `CliError` (incl. a `NotImplemented` variant), the
    exit-code mapping, and the dispatch logic (real `apply`, stubs for the
    rest).
  - `tests/cli.rs` — integration tests driving the built binary via
    `env!("CARGO_BIN_EXE_crustyimg")` + `std::process::Command` with `tempfile`
    fixtures.
  - `decisions/DEC-012-clap-cli-framework.md` — the new CLI-framework decision
    (already created during design; referenced here).
- **Files modified:**
  - `src/main.rs` — **replace** the SPEC-001 argv match with a thin shell:
    `fn main() -> ExitCode { crustyimg::cli::run() }` (or equivalent). `--version`
    and `--help` continue to work — now via clap, not the hand-rolled match.
  - `src/lib.rs` — add `pub mod cli;` (and a doc line noting SPEC-007).
  - `Cargo.toml` — add `clap = { version = "=4.x.y", features = ["derive"] }`
    (pin an exact patch version, per AGENTS §5). No other new dependency.
- **New exports:**
  - `crustyimg::cli::run() -> std::process::ExitCode` — the binary's single
    entry point.
  - `crustyimg::cli::{Cli, Commands, GlobalArgs, CliError}` (the derive types +
    typed error; public so they are unit-testable per `every-public-fn-tested`).

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] `crustyimg --help` exits 0 and lists **every** MVP subcommand by name:
      view, info, resize, thumbnail, shrink, convert, auto-orient, watermark,
      strip, clean, set, copy-metadata, edit, apply.
- [ ] `crustyimg --version` exits 0 and prints the crate semver
      (`crustyimg::version()` / `CARGO_PKG_VERSION`).
- [ ] Each subcommand parses its documented args from `docs/api-contract.md`
      (e.g. `resize --max`, `convert --format`, `apply --recipe`,
      `copy-metadata --from/--to`) — `crustyimg <cmd> --help` exits 0.
- [ ] The global options parse and are accepted on the command line:
      `-o/--output`, `--out-dir`, `--name-template`, `-j/--jobs`, `--format`,
      `-q/--quality`, `-v/--verbose` (repeatable), `-Q/--quiet`, `-y/--yes`,
      `--keep-gps`.
- [ ] An **unknown subcommand** (e.g. `crustyimg frobnicate x.png`) exits with
      code **2** (clap usage error).
- [ ] The **one real path** works end to end: `crustyimg apply --recipe r.toml
      in.png -o out.png` (with `r.toml` a valid version-"1" recipe of built-in
      ops, e.g. `invert`) loads the input, runs the pipeline, writes
      `out.png`, and exits **0**; `out.png` exists and is a non-empty, decodable
      image.
- [ ] A **stub** subcommand (e.g. `crustyimg resize in.png --max 800 -o
      out.png`) exits with the typed not-implemented non-zero code (**1**,
      generic runtime error) and prints a "not yet implemented" diagnostic to
      **stderr**; it does **not** write an output file and does **not** panic.
- [ ] `apply` with a **missing input** (`apply --recipe r.toml does-not-exist.png`)
      exits **3** (input not found), diagnostic on stderr.
- [ ] `apply` with a **bad recipe version** (a recipe whose `version` ≠ `"1"`)
      exits **1** (generic; recipe parse/version error), diagnostic on stderr.
- [ ] Diagnostics go to **stderr**; `-o -` keeps **stdout** clean (the real
      `apply --recipe r.toml in.png -o -` writes only encoded bytes to stdout,
      logs to stderr).
- [ ] No `unwrap()`/`expect()`/`panic!()` on recoverable paths; the binary
      never panics on bad input — it returns an `ExitCode`.
- [ ] The pixel core (`src/image`, `src/operation`) does **not** import `clap`
      (architecture layering).
- [ ] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, and
      `cargo fmt --check` all pass; the SPEC-001..006 test suite stays green.

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build** is
to make these pass. These are **integration** tests that drive the *built
binary* (`env!("CARGO_BIN_EXE_crustyimg")` + `std::process::Command`); a couple
of small **unit** tests in `src/cli/mod.rs` cover the parser and exit-code
mapping directly.

- **`tests/cli.rs`** (binary-driven; use `tempfile::tempdir()` for fixtures;
  generate input images natively with the `image` crate — NO committed binary
  fixtures, NO ImageMagick; trim trailing whitespace/newlines from captured
  stdout before asserting so Windows `\r\n` does not break comparisons):
  - `"help_lists_all_subcommands"` — run `--help`; asserts: exit code 0, and
    stdout (after trim) contains each of the 14 subcommand names.
  - `"version_prints_semver"` — run `--version`; asserts: exit 0 and stdout
    (trimmed) contains `env!("CARGO_PKG_VERSION")`.
  - `"unknown_subcommand_is_usage_error"` — run `frobnicate x.png`; asserts:
    exit code **2**.
  - `"each_subcommand_help_parses"` — for each of the 14 names, run
    `<name> --help`; asserts: exit 0 (proves every variant + its args is
    declared and parses).
  - `"apply_recipe_runs_end_to_end"` — write a 4×4 PNG to the tempdir via the
    `image` crate; write `r.toml` = `version = "1"\n[[step]]\nop = "invert"\n`;
    run `apply --recipe <r.toml> <in.png> -o <out.png>`; asserts: exit **0**,
    `out.png` exists, is non-empty, and re-decodes via `image::open` to the same
    dimensions.
  - `"apply_to_stdout_keeps_stdout_clean"` — same fixture; run `apply --recipe
    <r.toml> <in.png> -o -` capturing stdout; asserts: exit 0 and the captured
    stdout bytes decode as an image via `image::load_from_memory` (i.e. stdout
    is *only* encoded image bytes, no diagnostics). NOTE: `Sink::Stdout`
    requires a known format — pass `--format png` (no extension to infer from
    when the output is `-`).
  - `"stub_command_returns_not_implemented"` — run `resize <in.png> --max 800
    -o <out.png>`; asserts: exit code **1**, `out.png` was NOT created, and
    stderr (not stdout) contains "not yet implemented" (case-insensitive
    substring).
  - `"apply_missing_input_exits_3"` — run `apply --recipe <r.toml>
    <tempdir>/nope.png -o <out.png>`; asserts: exit code **3**.
  - `"apply_bad_recipe_version_exits_1"` — write `bad.toml` with
    `version = "999"`; run `apply --recipe <bad.toml> <in.png> -o <out.png>`;
    asserts: exit code **1**.
- **`src/cli/mod.rs` `#[cfg(test)] mod tests`** (unit, no process spawn):
  - `"cli_parses_global_and_apply"` — `Cli::try_parse_from(["crustyimg",
    "apply", "--recipe", "r.toml", "in.png", "-o", "out.png"])` is `Ok` and the
    parsed value carries the `apply` variant with `recipe == "r.toml"` and the
    global `output == Some("out.png")`. (Use clap's `try_parse_from` so the test
    does not exit the process.)
  - `"cli_unknown_subcommand_is_err"` — `Cli::try_parse_from(["crustyimg",
    "frobnicate"])` is `Err`, and the error's `kind()` is a usage error (clap
    reports `exit_code() == 2`).
  - `"exit_code_mapping_is_total"` — a unit test over the error→exit-code
    mapping function: each `CliError` variant (or each typed source error it
    wraps) maps to its documented code (1/3/4/5/6) and `NotImplemented` → 1.

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec since
there is no separate receiving agent.*

### Decisions that apply

- `DEC-012` (NEW, governs `src/cli/**`, `src/main.rs`, `Cargo.toml`) — use
  **`clap` 4 derive** as the CLI framework. Subcommand surface + global options
  live in `src/cli/` as derive types; `main.rs` is a thin shell. This DEC is
  what justifies adding `clap` (satisfies `no-new-top-level-deps-without-decision`).
- `DEC-007` (governs error handling) — the **library** returns typed
  `thiserror` enums; the **binary boundary** (`cli`/`main`) maps typed errors to
  the api-contract exit codes and may use `anyhow` for human context. The pixel
  core does NOT depend on `anyhow`. **Where to add `anyhow`:** `cli` is the
  binary boundary, so `anyhow` is allowed there — but add it ONLY if you
  actually use it; a clean typed-error → exit-code mapping that formats
  diagnostics directly to stderr does not require `anyhow`, and is simpler. If
  you add it, it is pre-justified by DEC-007 (no new DEC). Do NOT add it to the
  pixel core.
- `DEC-002` — the CLI drives the single `Image` model + `Operation`/`Pipeline`.
  The real `apply` path goes through them; the pixel core stays clap-free.
- `DEC-005` — the `apply` command runs a **recipe**: `Recipe::from_toml` then
  `Recipe::build_pipeline(&registry)`. Built-in ops are `identity` + `invert`
  (`OperationRegistry::with_builtins()`).
- `DEC-006` — `--jobs` is a **parsed placeholder** in STAGE-001 (default = CPU
  count). Do NOT add `rayon`, do NOT parallelize. Accept the value and thread it
  nowhere. Real parallel batch is STAGE-005.

### Constraints that apply

(see `/guidance/constraints.yaml` for full text):

- `ergonomic-defaults` — the common single-image task is one short command with
  sensible defaults; do not require boilerplate flags. Reflect this in the arg
  design (positional inputs, sane defaults; e.g. `--jobs` defaults to CPU count
  rather than being required; `apply` takes the input positionally).
- `no-unwrap-on-recoverable-paths` — no `unwrap`/`expect`/`panic!` on
  recoverable paths. `main()` returns `ExitCode`; bad input → typed error →
  exit code, never a panic.
- `no-new-top-level-deps-without-decision` — `clap` is the only new dep, covered
  by DEC-012. Do not add others (no `rayon`, no extra parser).
- `clippy-fmt-clean` — passes `cargo clippy -- -D warnings` and
  `cargo fmt --check`; no dead code.
- `every-public-fn-tested` — public functions in `cli` (e.g. `run`, the
  exit-code mapping) get a test (unit or via the binary integration tests).
- `test-before-implementation` — the `## Failing Tests` above are written first;
  build makes them pass.
- `untrusted-input-hardening` (advisory here) — the CLI passes user paths /
  recipe files into `Source`/`Sink`/`Recipe`, which **already** guard them
  (symlink-escape, traversal, overwrite, recipe version/unknown-op). The CLI
  must surface their typed errors as the right exit code, NOT re-implement or
  bypass the guards. Use `Overwrite::Forbid` unless `--yes` is passed.

### Prior related work — the exact APIs to call for the real `apply` path

All shipped on `main`. Signatures (verified against the source):

- **Source** (`src/source/mod.rs`):
  - `pub fn resolve(arg: &str, reader: &mut impl Read) -> Result<Vec<Input>, SourceError>`
    — pass `&mut std::io::stdin().lock()` as the reader in production; the
    reader is only used when `arg == "-"`.
  - `pub enum Input { Path(PathBuf), Stdin { bytes: Vec<u8>, stem: String } }`
    with `Input::path() -> Option<&Path>` and `Input::stem() -> &str`.
  - `pub enum SourceError { NotFound(String), InvalidPattern{..}, Stdin(io::Error) }`
    — `NotFound` → exit **3**; `InvalidPattern` → exit **2** (usage);
    `Stdin(io)` → exit **3**. (See the mapping table below.)
- **Image** (`src/image/mod.rs`):
  - `pub fn load(path: impl AsRef<Path>) -> Result<Image, ImageError>` — for
    `Input::Path`.
  - `pub fn from_bytes(bytes: &[u8]) -> Result<Image, ImageError>` — for
    `Input::Stdin { bytes, .. }`.
  - `pub enum ImageError { Io(io::Error), Decode(String), UnsupportedFormat }`
    — `Io` → **3**, `Decode` → **1**, `UnsupportedFormat` → **4**.
- **Recipe** (`src/recipe/mod.rs`):
  - `pub fn from_toml(s: &str) -> Result<Recipe, RecipeError>` — read the recipe
    file's text yourself (`std::fs::read_to_string`, map io error to exit 3),
    then parse.
  - `pub fn build_pipeline(&self, registry: &OperationRegistry) -> Result<Pipeline, RecipeError>`.
  - `pub enum RecipeError { UnsupportedVersion{..}, UnknownOperation{name}, Parse(String), Serialize(String) }`
    — all → **1** (a bad recipe is a runtime error, not a usage error).
- **Registry** (`src/operation/registry.rs`):
  - `pub fn with_builtins() -> OperationRegistry` — gives `identity` + `invert`.
- **Pipeline** (`src/pipeline/mod.rs`):
  - `pub fn run(&self, img: Image) -> Result<Image, OperationError>` — folds the
    ops over the loaded image. `OperationError::Apply{..}` → **1**.
- **Sink** (`src/sink/mod.rs`):
  - `pub enum Sink { File{path, format}, Dir{dir, template, format}, Stdout{format}, Display }`.
  - `pub fn write(&self, img: &Image, input: &SinkInput, overwrite: Overwrite, out: &mut dyn Write) -> Result<(), SinkError>`.
  - `pub struct SinkInput<'a> { pub stem: &'a str, pub path: Option<&'a Path> }`
    — build from the `Input`.
  - `pub enum Overwrite { Forbid, Allow }` — `Allow` iff `--yes`.
  - `pub enum SinkError { Io(io::Error), Encode(String), UnknownFormat, UnsupportedExtension(String), Traversal(String), AlreadyExists(String), NotATty, Display(String) }`
    — output failures → **5**, EXCEPT `UnsupportedExtension`/`UnknownFormat` →
    **4** (unsupported/undeterminable format). See the table.
  - **Mapping the global options onto a Sink for `apply`:** if `-o <PATH>` and
    `PATH == "-"` → `Sink::Stdout { format }` (format from `--format`, else the
    write returns `UnknownFormat` → 4); if `-o <PATH>` →
    `Sink::File { path, format: --format }`; if `--out-dir <DIR>` →
    `Sink::Dir { dir, template: --name-template or a default like
    "{stem}.{ext}", format }`. For the single-input `apply` test path, only
    `-o <file>` and `-o - --format png` are exercised. Pass
    `&mut std::io::stdout().lock()` as the `out` writer (only the `Stdout`
    variant writes to it). `--format` is a string like `png`/`jpg`; convert it
    to `image::ImageFormat` (e.g. via `sink::format_from_extension` on a
    synthetic `name.<fmt>`, or `ImageFormat::from_extension`).

### The exit-code mapping table (authoritative for this spec)

`main()` returns `ExitCode`. clap owns code **2** (usage errors) automatically.
The `cli` layer maps the **typed library errors** to the rest:

| Exit | Meaning (api-contract) | Mapped from |
|---|---|---|
| 0 | Success | `apply` completed and wrote output |
| 1 | Generic runtime error | `RecipeError::*`, `OperationError::Apply`, `ImageError::Decode`, `CliError::NotImplemented` (all stub commands) |
| 2 | Usage error | clap parse failure (unknown subcommand, bad/missing args), `SourceError::InvalidPattern` — clap exits 2 on its own; the pattern case is rare and not exercised here |
| 3 | Input not found / unreadable | `SourceError::NotFound`, `SourceError::Stdin(io)`, `ImageError::Io`, recipe-file read io error |
| 4 | Unsupported format / codec not built | `ImageError::UnsupportedFormat`, `SinkError::UnsupportedExtension`, `SinkError::UnknownFormat` |
| 5 | Output write failed / refused | `SinkError::{Io, Encode, Traversal, AlreadyExists, NotATty, Display}` |
| 6 | Partial batch failure | (STAGE-005; not reachable in this spec — single-input apply only) |

Implementation shape: a `CliError` enum that wraps the source-module errors
(`#[from] SourceError`, `#[from] ImageError`, `#[from] RecipeError`,
`#[from] OperationError`, `#[from] SinkError`) plus a `NotImplemented(&'static
str)` variant, and a `fn code(&self) -> u8` (returning the table above) that
`run()` turns into `ExitCode::from(code)`. `run()` does the parse + dispatch,
prints any error's `Display` to **stderr**, and returns the `ExitCode`. Keep the
mapping in ONE place and unit-test it (`exit_code_mapping_is_total`).

> Note on clap's exit code: clap's own usage/error exit code is already `2`,
> which matches the api-contract usage-error code — so let clap handle parse
> failures (use `Cli::parse()` in `run`, which prints to stderr and exits 2 on
> error; or `try_parse()` + map the clap error's `exit_code()`). The `cli` layer
> only maps the typed *library* errors (1/3/4/5/6).

### Out of scope (for this spec specifically)

If any of these feel necessary during build, create a new spec rather than
expanding this one.

- The actual operation/command **implementations** — resize, view, info,
  thumbnail, shrink, convert, auto-orient, watermark, strip, clean, set,
  copy-metadata, edit. These are STAGE-002/003/004 and are **stubs** here that
  return `CliError::NotImplemented` (exit 1).
- Real **parallel batch** — `--jobs` is parsed and ignored (DEC-006); no
  `rayon`, no `indicatif`. STAGE-005.
- Glob/dir **batch fan-out** in `apply` — `source::resolve` already returns a
  `Vec<Input>`, but this spec only needs to prove the **single-input** real
  path end to end (process the first/only input; multi-input + exit 6 is
  STAGE-005).
- The **metadata lane** (strip/clean/set/copy-metadata logic) — STAGE-004
  (stubs here).
- **Terminal display correctness** — `Sink::Display` / viuer is gated by the
  `display` feature (SPEC-005/DEC-011); `view` is a stub here.
- `edit --save-recipe` — STAGE-005 (stub here).
- Wiring CLI arg flags into real `Operation`s — STAGE-003+.

## Notes for the Implementer

- **clap derive for many subcommands is the trickiest part.** Define ONE
  `#[derive(Parser)] struct Cli` with `#[command(flatten)] global: GlobalArgs`
  and `#[command(subcommand)] command: Commands`. `GlobalArgs` is a
  `#[derive(Args)]` struct holding the global-option fields. `Commands` is a
  `#[derive(Subcommand)]` enum with one variant per command, each variant a
  struct of that command's args. Keep variant arg structs minimal — only the
  args the api-contract documents for that command. Use `#[arg(short, long)]`
  for the documented short flags (`-o`, `-j`, `-q`, `-y`); set explicit
  `#[arg(short = 'Q', long)]` for `--quiet` (uppercase Q) and
  `#[arg(short, long, action = clap::ArgAction::Count)]` for `--verbose`
  (repeatable). `--version`/`--help` are automatic (clap provides `-V`/`-h`) —
  do NOT redefine them; set `#[command(version, about)]` on `Cli` so
  `--version` prints `CARGO_PKG_VERSION`.
- **Replacing `main.rs` while keeping the SPEC-001 smoke test green.** The
  SPEC-001 integration smoke test (check `tests/` — there may be a `smoke.rs`
  or similar) asserts `--version` prints the semver and `--help` exits 0. Those
  still hold via clap — but the exact stdout *text* of `--help` changes (clap's
  format ≠ the old `USAGE` string). If a SPEC-001 test asserts on the old
  literal usage string, that assertion is now wrong; **update it** to the new
  behavior (exit 0, `--help` stdout lists subcommands; `--version` stdout
  contains the semver) — don't delete the coverage. The `src/lib.rs`
  `version_returns_cargo_pkg_version` unit test is unaffected and must stay
  green. The SPEC-001 `EXIT_USAGE`/`USAGE` constants and the argv match are
  removed.
- **Do not panic.** `main()` returns `ExitCode`; `run()` returns `ExitCode`.
  Handle every error and map it — no `unwrap`/`expect` on user-input paths.
- **Diagnostics off stdout.** Error `Display` and any `-v` logging go to
  **stderr** (`eprintln!`). The ONLY thing on stdout is the encoded image bytes
  when `-o -` (written by `Sink::write` via the `out` writer you pass). The
  "not yet implemented" message for stubs goes to **stderr**.
- **The real `apply` path, concretely** (single input):
  1. `let recipe_text = std::fs::read_to_string(&recipe_path)` (io err → exit 3).
  2. `let recipe = Recipe::from_toml(&recipe_text)?` (RecipeError → exit 1).
  3. `let registry = OperationRegistry::with_builtins();`
  4. `let pipeline = recipe.build_pipeline(&registry)?;`
  5. `let inputs = source::resolve(&input_arg, &mut std::io::stdin().lock())?;`
     (SourceError → 3) — take the first input for the single-input path.
  6. Load: `Input::Path(p) => Image::load(p)`, `Input::Stdin{bytes,..} =>
     Image::from_bytes(&bytes)` (ImageError → 3/1/4).
  7. `let out_img = pipeline.run(img)?;` (OperationError → 1).
  8. Build the `Sink` from `-o`/`--out-dir`/`--format` (see the Sink mapping
     above), `SinkInput { stem: input.stem(), path: input.path() }`, `Overwrite`
     from `--yes`, and call
     `sink.write(&out_img, &sink_input, overwrite, &mut std::io::stdout().lock())?`
     (SinkError → 5/4).
  9. Return `ExitCode::SUCCESS`.
- **`--format` conversion:** clap gives you a `String`. Convert to
  `image::ImageFormat`. Simplest: `image::ImageFormat::from_extension(&fmt)`
  (returns `Option`), `None` → treat as unsupported (exit 4). Reuse the sink's
  helpers where convenient.
- **Pin clap.** Add `clap = { version = "=4.<minor>.<patch>", features =
  ["derive"] }` with an exact patch pin like the other deps in `Cargo.toml`.
  `clap` 4.5.x is current; pin the exact patch you resolve.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-007-clap-cli-skeleton-and-dispatch`
- **PR (if applicable):** PR opened — number and URL filled in once pushed (see timeline).
- **All acceptance criteria met?** yes
- **Gates:**
  - `cargo build` — PASS
  - `cargo test` — PASS (97 tests, 10 suites, all green; SPEC-001..006 suite intact)
  - `cargo clippy --all-targets -- -D warnings` — PASS (0 errors, 0 warnings)
  - `cargo fmt --check` — PASS
- **New decisions emitted:**
  - No new DEC during build — DEC-012 written during design justifies `clap = 4.6.1`.
- **Deviations from spec:**
  - `clap` resolved to 4.6.1 (not 4.5.x as the spec estimated); pinned exactly as `=4.6.1`.
  - Added a `CliError::RecipeIo(std::io::Error)` variant to correctly map the recipe-file
    read I/O error to exit 3 without colliding with `SourceError::Stdin`'s `#[from] io::Error`
    (which would cause a conflicting `From` impl). This is a clean, minimal extension to the
    exit-code mapping — no new DEC needed, consistent with DEC-007.
  - Global args marked `global = true` on each `#[arg]` to allow them to appear after the
    subcommand name (the spec snippet shows the flatten pattern; `global = true` is the correct
    clap 4 mechanism to propagate flattened args into subcommand contexts).
- **Follow-up work identified:**
  - STAGE-002: `view` and `info` real implementations.
  - STAGE-003: `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient`.
  - STAGE-004: `watermark`, `strip`, `clean`, `set`, `copy-metadata`.
  - STAGE-005: `edit`, `apply` batch fan-out + parallel + progress.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec's `GlobalArgs` snippet did not mention `global = true`, which is required in
   clap 4 for flattened args to be recognized after the subcommand. The unit test caught this
   immediately (clap returned "unknown argument -o" for `apply ... -o out.png`), but it took
   one compile-fix cycle to identify the root cause. A note in the spec about `global = true`
   would have prevented this.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The `#[from] std::io::Error` collision between `CliError::Image(ImageError::Io)` and a
   potential `CliError::RecipeIo` was not called out. The spec says "map io err → exit 3" for
   the recipe file read, but both `ImageError::Io` and a bare `io::Error` variant would need
   `#[from] io::Error`, which conflicts. Adding a distinct `CliError::RecipeIo(io::Error)` variant
   (mapping to exit 3) was the clean solution; the spec could have noted this explicitly.

3. **If you did this task again, what would you do differently?**
   — Read the clap 4 docs on `global` args before writing the `GlobalArgs` struct, and add
   the `global = true` attribute from the start. Also run `cargo test -- cli` first (just
   the new CLI unit tests) before running the full suite, to tighten the feedback loop on
   the clap integration.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
