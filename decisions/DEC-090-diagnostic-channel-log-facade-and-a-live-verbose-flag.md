---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-090
  type: recommendation              # PROPOSED — flip to `decision` on acceptance
  confidence: 0.75
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-11
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/mod.rs"
  - "src/cli/common.rs"
  - "src/diag/**"
  - "Cargo.toml"
  - "AGENTS.md"
  - "docs/cli-reference.md"
  - "tests/diagnostics.rs"

tags:
  - logging
  - diagnostics
  - observability
  - verbose
  - stderr
  - silent-wrong-output
  - svg
  - no-new-top-level-deps-without-decision
---

# DEC-090: adopt the `log` facade with an in-house stderr sink, and make `-v` real

> **STATUS: PROPOSED.** Written 2026-08-11 from a read-only observability audit.
> Not yet accepted. The three calls flagged **OPEN** below want the maintainer's
> read before this becomes a spec.

## Decision

Take **`log` (the facade only) as a top-level dependency**, write the sink
in-house (~60 lines under `src/diag/`), and install it once at the CLI boundary so
`-v/--verbose` — today a parsed-but-never-read flag — actually controls a
diagnostic level.

**The default level is `Warn`, not `Off`.** A run that is silently degrading its
own output says so on stderr without the user opting in. `-v` = `Info`,
`-vv` = `Debug`, `-vvv` = `Trace`; `--quiet` = `Error` only. Everything goes to
**stderr**; stdout stays pipe-clean.

The point is not the flag. The point is that **12 crates in our tree emit
`log` records today and we install no logger, so every one of them is discarded
at the facade** — including the warnings that would catch silent output
corruption.

## Context

### The flag is dead, and the docs claim otherwise

`GlobalArgs::verbose` is declared at `src/cli/mod.rs:97` as a global repeatable
counter. It is **read zero times in `src/`** — the only other occurrences in the
crate are `verbose: 0` in two test fixtures (`src/cli/ops.rs:1215`,
`src/cli/common.rs:276`). There is no test for it anywhere in `tests/`.

Its own help text is false:

```
-v, --verbose...    Increase verbosity; repeatable (`-vv` for more). Logs to stderr
```

So is the convention in `AGENTS.md:339` — *"Diagnostics go to stderr … gated by
`-v/--verbose` / `--quiet`."* `--quiet` is real and applied consistently.
`-v` is fiction.

Verified with controls against `target/release/crustyimg` at 0.7.0:

```
NEGATIVE  info photo.jpg -vvvvv  vs plain  → stdout AND stderr byte-identical
NEGATIVE  web  photo.jpg -vvv    vs plain  → stderr byte-identical
POSITIVE  web  photo.jpg -Q      vs plain  → stderr 99 B → 0 B
```

The positive control proves the harness can see a stderr difference; `-v` at any
repetition produces none.

### The real cost: ~95 upstream diagnostics on the floor

`log` is already in the tree — transitively, via twelve crates. Nothing ever calls
`log::set_logger`, so every record is dropped at the facade:

| crate | `log::` call sites |
|---|---|
| usvg | 55 |
| notify | 12 |
| resvg | 9 |
| rav1e | 8 |
| rustybuzz / fontdb | 4 each |
| avif-parse | 3 |

**This is not academic — it hides a measured silent-wrong-output bug.** An SVG
carrying a dangling filter reference:

```svg
<rect width="40" height="30" fill="#08f" filter="url(#nope)"/>
```

converts with **exit 0 and zero bytes on stderr**. usvg drops the entire filtered
element. Scored against the same SVG with the dangling ref removed:

```
crustyimg diff warn.png ok.png  →  ssimulacra2: -61.7
```

The blue rect is gone and nothing anywhere tells the user. The diagnostic that
would have caught it is being emitted **right now** and thrown away —
`usvg-0.47.0/src/parser/filter.rs:64`:

```rust
log::warn!("Filter '{}' has an invalid region. Skipped.", node.element_id());
```

DEC-066 named exactly this class a *"silent-wrong-output footgun"* for SVG
`<text>`, and paid **287 KB of wasm bundle** to avoid it. Here the same class of
failure is already detected by a dependency, and we are dropping the evidence for
the cost of one zero-dependency crate.

### Why `Warn` on by default is the house position, not a new one

DEC-085 already settled the shape: a truncated JPEG prints an unconditional
`warning:` to stderr on every decoding verb and **exit stays 0**. Silent
degradation gets an unconditional stderr warning. The dangling-filter case is the
same class with a worse blast radius (`-61.7` vs a partial decode), so defaulting
the level to `Off` and hiding it behind `-v` would contradict a decision this repo
made three weeks ago.

## Alternatives Considered

- **Option A: delete `-v` and add nothing.**
  - What it is: remove the dead flag, fix `AGENTS.md`, ship no logger.
  - Why rejected: cheapest and honest about the flag, but it makes the silent-SVG
    class **permanently undetectable**. The defect is not the flag; the flag is
    the symptom. This closes the paperwork and leaves the bug.

- **Option B: `log` + `env_logger`.**
  - What it is: the conventional pairing; `RUST_LOG` filter syntax for free.
  - Why rejected: `env_logger` 0.11 pulls `env_filter` → `regex-automata` for a
    filter grammar we do not need — we need five levels written to stderr. It is
    a large transitive tail on a crate whose lean build (`--no-default-features`)
    and wasm bundle are both actively defended (DEC-066, `just wasm-size`). Fails
    the "price the parser in lines and compare" gate that sent `little_exif` →
    `tiff.rs` (718 lines) and `ab_glyph` → `skrifa`+`zeno`.

- **Option C: `tracing` + `tracing-subscriber`.**
  - What it is: the modern structured/span-oriented stack.
  - Why rejected: **wrong shape twice over.** First, spans model a distributed or
    async system; this is a single-process, no-network CLI (`SECURITY.md:4`) with
    `no-async-runtime` as a standing constraint. Second — decisively — the whole
    value here is capturing the ~95 **`log`** sites upstream. `tracing` needs
    `tracing-log` to bridge them, so it is strictly more machinery for a strictly
    worse fit. Structured spans are a real want for `--timing` depth; that is a
    different problem (see *Out of scope*), and it does not need this crate.

- **Option D (chosen): `log` facade + an in-house stderr sink.**
  - What it is: `log = "0.4"` as a top-level dep; a `src/diag/` module
    implementing `log::Log` (level filter, `warning:`/`error:` prefixing, input
    attribution, dedup) installed once from `cli::run()`.
  - Why selected: `log` is **MIT OR Apache-2.0, ~3.1k LOC, and has zero required
    dependencies** — it is already a leaf in our tree (`cargo tree -i log` shows
    no children). It is the exact facade the upstream crates target, so adopting
    it captures all ~95 sites with no bridge. The sink is the ~60 lines we would
    have to write anyway to get the prefixing and attribution this codebase's
    output conventions demand. This is the documented house pattern for a small,
    well-understood surface.

## Design calls this settles

1. **Level mapping.** default `Warn` · `-v` `Info` · `-vv` `Debug` · `-vvv` `Trace`
   · `--quiet` `Error`.
2. **Stream.** stderr, unconditionally, at every level. stdout carries payload
   only — the existing discipline that already makes `--json` on an stdout sink a
   usage error (DEC-074).
3. **`-v` with `-Q` is a usage error (exit 2).** Contradictory intent gets
   rejected, not silently resolved — same posture as the `--json`/stdout-sink rule
   rather than a last-wins guess.
4. **Prefixing.** Upstream records render as `warning: <target>: <message>` so a
   usvg complaint is visibly *not* crustyimg's own wording, and `TRUNCATED_JPEG_WARNING`
   (DEC-085) keeps its existing exact text.
5. **wasm installs no logger.** The facade with no logger is a no-op, and
   `release_max_level_off` compiles the call sites out entirely. **The bundle
   delta must be measured through the real artifact, not assumed** — `just
   wasm-size` before/after, per the SPEC-074 lesson. A non-zero delta on wasm
   sends this back for redesign.

## OPEN — needs the maintainer's read

- **OPEN-1 · Batch attribution.** A `log` record carries no input path, and the
  rayon batch path interleaves. Without help, a 500-file run prints
  `warning: usvg: Filter '…' has an invalid region` five hundred times with no way
  to know which file. Proposal: a thread-local "current input" the sink prefixes,
  plus **dedup of identical `(level, target, message)` within one run, reported as
  `(×N)`**. This is the largest chunk of the ~60 lines and the piece most worth
  arguing about.
- **OPEN-2 · Is `Warn`-by-default too loud?** DEC-085's precedent says no, and a
  dirty-SVG asset tree is exactly the population that *should* hear about it. But
  it is a behavior change: runs that were silent now print, and that will show up
  in someone's CI log diff. Alternative is `Off` by default with `-v` = `Warn`,
  which is quieter and, I think, wrong.
- **OPEN-3 · Env configuration.** `NO_COLOR` is cheap and standard — I would take
  it. A full `RUST_LOG` filter grammar is exactly the thing that made Option B too
  heavy; a scalar `CRUSTYIMG_LOG=warn|info|debug|trace` gets 90% of the CI/Docker
  ergonomics for ~5 lines. Worth noting the crate currently reads **no environment
  variable for configuration at all** (`std::env::var` appears only for
  `current_dir` and `env::consts`), so this is a first.

## Out of scope — named so they don't get folded in

Each is a separate spec; this DEC is the **diagnostic channel** only.

- **`--timing` span depth.** Three buckets today, and the code's own doc comment
  concedes what is hidden (`src/analysis/decide.rs:285`: *"total = decode +
  analysis + encode + any score"*). Measured: `decode 4.79 + encode 541.11` against
  `total 629.85` leaves **84 ms unattributed**, and `encode_ms` accumulates across
  candidates (`src/cli/optimize.rs:1048`) so an AVIF-vs-JPEG cost split is
  unavailable. For an optimization engine that is the measurement you most want.
- **Batch aggregate report.** Already filed — `docs/backlog.md:394`, `--report[=path]`,
  maintainer request 2026-07-25. Also `build` has no `--json` at all.
- **`console_error_panic_hook` for wasm.** A Rust panic in the browser is an
  opaque `RuntimeError: unreachable`; typed errors reach the user
  (`demo/worker.js:270` → `demo/demo.js:185`), panics do not.
- **Panic detail.** `src/image/avif.rs:127` correctly converts an unwind to a typed
  error but discards payload and location, while the default hook still prints its
  own `thread panicked at …` first. No `panic::set_hook` anywhere.
- **CI dogfooding.** `lint --format sarif` exists for GitHub code scanning
  (`docs/cli-reference.md:430`) and **no workflow runs `crustyimg lint` or uses
  `upload-sarif`**. No `GITHUB_STEP_SUMMARY` either.
- **Promoting no-network/no-telemetry to a constraint.** `SECURITY.md:4` asserts
  it and DEC-074 tags it, but it is **not among the 16 ids in
  `guidance/constraints.yaml`** — the strongest privacy claim the tool makes is
  prose, not a gate. That is a `guidance/` change, not a DEC.
- **Trace correlation via `TRACEPARENT` (follow-on, wants its own spec).** The
  export boundary belongs to the **harness**, not the tool: a CI pipeline is a
  genuinely distributed system worth tracing, a single `crustyimg` process is not.
  The tool's job is to be joinable, not to be a client. Sketch: read the W3C
  `TRACEPARENT` env var (which `otel-cli` and every CI OTel setup already export),
  echo `trace_id`/`span_id` into the audit report, and let the collector join it.
  ~10 lines, no dependency, **no socket** — so `SECURITY.md:4` survives intact
  while a crustyimg run becomes a first-class row in someone else's trace. Depends
  on the batch `--report[=path]` landing first, since a report on stdout is
  hostile to the artifact/collector path.

## Consequences

- **Positive.** The silent-degradation class becomes visible for one zero-dep
  crate. `-v` stops lying and `AGENTS.md:339` becomes true. Debugging a bad
  decode stops requiring a rebuild. Future upstream diagnostics arrive free.
- **Negative.** A top-level dep (`no-new-top-level-deps-without-decision` — this
  document is that decision). Output changes for runs that were silent, so golden
  stderr assertions in `tests/` need an audit. A global logger is process-wide
  mutable state — `set_logger` must be called exactly once, and never from the
  library path, or an embedding consumer gets hijacked.
- **Neutral.** stdout is untouched, so every pipe, `--json` consumer, and exit
  code keeps its contract. The lean `--no-default-features` build is unaffected
  (`log` is unconditional and dependency-free, not feature-gated).

## Validation

The acceptance test is the reproducer built during the audit, not a synthetic one:

1. **The dangling-filter SVG goes from silent to spoken.** `convert warn.svg
   --format png` must still exit 0 and still write the file, but must now emit a
   `warning:` line naming the dropped filter. The `-61.7` ssimulacra2 gap against
   the clean render is the evidence the warning is *about something real*.
2. **`-v` moves stderr.** The negative control above must flip: `-vvv` vs plain
   must now differ, with `-Q` still driving it to zero. Assert both directions —
   a verbosity test that only checks "more output" passes on a broken filter.
3. **stdout is untouched at every level.** `-vvv` must not change one byte of
   `--json`, `info --json`, or an image on `-o -`.
4. **wasm bundle delta is zero.** Measured via `just wasm-size` before/after, read
   from the artifact — not inferred from the feature flag.

**Revisit if:** the dedup/attribution design (OPEN-1) grows past ~100 lines, at
which point Option B's dep tree starts looking like the cheaper trade; or if a
future need for real span hierarchies in `--timing` makes `tracing` the right
base after all, which would supersede this.

## References

- Related decisions: DEC-085 (silent degradation warns unconditionally, exit stays
  0 — the precedent for `Warn`-by-default), DEC-066 (the "silent-wrong-output
  footgun" framing and the 287 KB paid to avoid it), DEC-074 (stdout stays
  pipe-clean; the audit-report surface), DEC-054 (SVG rasterize, resvg/usvg),
  DEC-018/DEC-036 (dependency gate), DEC-006 (`no-async-runtime`).
- Constraints: `no-new-top-level-deps-without-decision` (this document),
  `no-async-runtime` (`log` is synchronous), `no-agpl-default-deps` (`log` is
  MIT OR Apache-2.0), `clippy-fmt-clean`, `every-public-fn-tested`.
- Code: `src/cli/mod.rs:97` (the dead flag), `AGENTS.md:339` (the false
  convention), `src/analysis/decide.rs:285` (the unattributed timing), `usvg-0.47.0/
  src/parser/filter.rs:64` (the discarded warning).
- Backlog: `docs/backlog.md:394` (batch `--report`, maintainer request 2026-07-25).
