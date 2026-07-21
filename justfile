# Spec-driven repo template — command runner
#
# This justfile works BOTH before and after `just init`:
# - Before init: only `init` and `list-variants` are expected to work.
# - After init: all the daily commands work (status, new-spec, etc.)
#
# Run `just --list` to see everything.

# Show all commands
default:
    @just --list

# ----------------------------------------------------------------------------
# APP DEVELOPMENT (cargo)  —  build / run / test / lint the crustyimg binary
#
# These wrap the exact commands in AGENTS.md §6. `view` needs the optional
# `display` feature (viuer, DEC-011) to actually render — use the *-display
# recipes for that.
# ----------------------------------------------------------------------------

# Debug build
build:
    cargo build

# Optimized release build → target/release/crustyimg
build-release:
    cargo build --release

# Build with terminal display compiled in (so `view` can render) (DEC-011)
build-display:
    cargo build --features display

# Run the CLI (debug). Usage: just run info photo.jpg  /  just run resize p.jpg --max 800 -o o.png
run *ARGS:
    cargo run --quiet -- {{ARGS}}

# Run the optimized binary. Usage: just run-release resize p.jpg --max 800 -o o.png
run-release *ARGS:
    cargo run --release --quiet -- {{ARGS}}

# Run with terminal display on — required for `view`. Usage: just view photo.png
run-display *ARGS:
    cargo run --features display --quiet -- {{ARGS}}

# Convenience: view an image in the terminal (builds with the display feature)
view IMAGE *ARGS:
    cargo run --features display --quiet -- view {{IMAGE}} {{ARGS}}

# Run all tests (unit + integration)
test:
    cargo test

# Run a single test or module. Usage: just test-one resize_parity
test-one NAME:
    cargo test {{NAME}}

# Run the criterion micro-benchmarks: decode/resize/encode/score/pipeline
# (SPEC-025, DEC-028). Dev-only — not part of the shipped binary.
# (Renamed from `bench` in SPEC-088; `bench` is now the committed corpus harness.)
bench-micro:
    cargo bench

# Committed end-to-end benchmark (SPEC-088, DEC-074): run web/optimize over the
# committed license-clean corpus (bench/corpus), printing a savings/time/score
# table. OFFLINE, deterministic, NO telemetry. `--json` for machine-readable
# output; `--corpus DIR` points at a real corpus (those photos never enter git).
# Built with `--features avif` because one corpus row (photo_forest_cc0.jpg, the
# real CC0 photograph) classifies `photograph` and its AVIF candidate WINS — so
# the flagship encode is regression-tested here. The synthetic rows classify
# `graphic-logo` and never reach AVIF; `web`'s downscale needs a >2048px source,
# which no committed image is. The table prints those limits as a footer.
# Usage: just bench [--json] [--corpus DIR] [--verbs web,optimize]
bench *ARGS:
    cargo build --release --features avif
    python3 scripts/bench.py --bin ./target/release/crustyimg {{ARGS}}

# Cross-tool benchmark (SPEC-083, DEC-080): crustyimg vs sharp / ImageMagick /
# @squoosh/cli / cwebp on size + speed + QUALITY, at MATCHED quality (every
# tool's output scored by `crustyimg diff` / SSIMULACRA2 and driven to the same
# band), over a real --corpus. This is what regenerates BENCHMARKS.md's tables —
# no hand-edited numbers. The competitors are NOT repo dependencies; install them:
#   npm i -g sharp-cli @squoosh/cli      # @squoosh/cli needs Node < 18
#   brew install imagemagick webp
# then point --squoosh-node at a Node < 18 binary (or set $SQUOOSH_NODE), and
# --tools-dir at the node_modules holding them (or $BENCH_TOOLS_DIR). A tool that
# isn't installed is LABELLED "NOT RUN", never silently dropped.
# Usage: just bench-compare --corpus /path/to/photos [--json] [--tools ...]
bench-compare *ARGS:
    cargo build --release --features avif
    python3 scripts/bench-compare.py --bin ./target/release/crustyimg {{ARGS}}

# Wall-clock the release binary with hyperfine. Skips cleanly (exit 0) if hyperfine
# is not installed. Usage: just bench-cli web photo.jpg --max 800 -o /tmp/o.avif
bench-cli *ARGS:
    @command -v hyperfine >/dev/null 2>&1 || { echo "hyperfine not installed; skipping (brew install hyperfine)"; exit 0; }
    cargo build --release
    hyperfine --warmup 2 './target/release/crustyimg {{ARGS}}'

# Lint IMAGE ASSETS with `crustyimg lint` (the format-aware upgrade from
# check-added-large-files). Usage: just lint-images [paths…] (default: cwd).
# Exit 7 on any error-severity finding — the CI-native gate (SPEC-057).
lint-images *paths=".":
    cargo run --quiet -- lint {{paths}}

# ----------------------------------------------------------------------------
# WASM (SPEC-072, DEC-064) — the pure engine compiled to wasm32, no backend
# ----------------------------------------------------------------------------
#
# TOOLCHAIN GOTCHA (this cost the SPEC-072 design probe a debug cycle, so it is
# baked in here rather than left in a doc): a machine can have BOTH a Homebrew
# rust (`/opt/homebrew/bin/rustc`, which ships NO wasm std) and a rustup rust. A
# bare `cargo build --target wasm32-…` invokes `rustc` off PATH, hits Homebrew's
# first, and fails with `error[E0463]: can't find crate for core/std` — which
# reads like a broken dependency but is really the wrong compiler. So every
# recipe below resolves the rustup STABLE toolchain explicitly and pins both
# CARGO and RUSTC to it.
#
# Setup (idempotent; run once):
#     rustup toolchain install stable
#     rustup target add --toolchain stable wasm32-unknown-unknown
#     brew install wasm-pack binaryen        # wasm-pack drives wasm-bindgen; binaryen = wasm-opt
#
# AVIF is ASYMMETRIC in the wasm build (SPEC-073, DEC-065): ENCODE is in — the
# shipped artifact is built `--features avif`, so `rav1e` turns a PNG into an AVIF
# in the browser (the demo's headline, and the reason `_wasm_features` below is not
# empty). DECODE is out — `re_rav1d` does not compile to bare wasm32 (DEC-064), so
# an AVIF *input* returns a typed error. Every other default input format works,
# SVG included.

# The rustup stable toolchain's bin dir — the one that actually has wasm std.
_wasm_bin := `rustup which --toolchain stable rustc | xargs dirname`

# The feature set the SHIPPED wasm artifact is built with (DEC-065). `avif` costs
# +345 KB brotli and buys the demo's headline (PNG → AVIF in the browser). Override
# to build the LEAN artifact — the no-AVIF comparison SPEC-074 measures against:
#     just --set _wasm_features "" wasm-build
_wasm_features := "--features avif"

# The SIZE PROFILE for the wasm artifact (SPEC-074, DEC-066) — fat LTO across the
# whole graph with a single codegen unit, worth ~116 KB brotli at zero cost to
# capability OR speed. Build the wasm artifact THROUGH this recipe; a bare
# `cargo build --target wasm32-…` gets the stock profile and a heavier download.
#
# Passed as CARGO_PROFILE_RELEASE_* env vars rather than written into
# `[profile.release]` in Cargo.toml ON PURPOSE: `[profile.release]` is shared with
# the NATIVE release build (and with `[profile.dist]`, which inherits it), and
# DEC-064 requires the released native binary to stay byte-identical. Cargo has no
# per-target profiles, so the env override — scoped to this recipe — is the only way
# to size-tune the wasm build without touching native.
#
# THREE LEVERS ARE DELIBERATELY ABSENT, each because it was measured and rejected:
#   * `opt-level = "z"` / `"s"` — the tempting one. "z" is worth another 165 KB
#     brotli and makes the AVIF encoder 2.8x SLOWER (350 ms → 956 ms on a 512x384;
#     scale that to a real photo). rav1e is generic, so its encoder monomorphizes
#     into `ravif` — pinning `ravif` back to opt-level 3 restores the speed exactly
#     (348 ms) and hands back 161 KB of the 165, i.e. the size win WAS the slowdown.
#     A one-time 165 KB download is not worth seconds on every conversion, on the
#     path that IS the demo's headline (DEC-065). See DEC-066.
#   * `panic = "abort"` — wasm32-unknown-unknown already defaults to it
#     (`rustc --print cfg` says `panic="abort"`), so it is a no-op.
#
# `strip = true` IS load-bearing (−58 KB brotli) and it very nearly got dropped as a
# no-op: measured against a wasm-opt'd build it looks like 250 B of noise, because
# `wasm-opt` had already stripped the debug sections itself. With wasm-opt off
# (DEC-066) nothing else does, so cargo has to. A lever's value depends on which
# other levers are pulled — measure it in the config you actually ship.
_wasm_profile := "CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_STRIP=true"

# Compile the library to wasm32 (debug). The fast "does it still compile" gate.
wasm-check:
    PATH="{{_wasm_bin}}:$PATH" RUSTC="{{_wasm_bin}}/rustc" \
        "{{_wasm_bin}}/cargo" build --lib --target wasm32-unknown-unknown {{_wasm_features}}

# Build the release .wasm + JS bindings via wasm-pack → pkg/ (the npm-shaped
# artifact STAGE-026 packages). Also reports the size SPEC-074 tunes.
wasm-build:
    @command -v wasm-pack >/dev/null 2>&1 || { echo "wasm-pack not installed (brew install wasm-pack)"; exit 1; }
    PATH="{{_wasm_bin}}:$PATH" RUSTC="{{_wasm_bin}}/rustc" {{_wasm_profile}} \
        wasm-pack build --target web --release --out-dir pkg -- {{_wasm_features}}
    @just wasm-size

# Report the .wasm size: raw, and the two COMPRESSED sizes a real host actually
# serves (a browser downloads the encoded bytes, so gzip/brotli are the honest
# numbers — raw alone overstates what a user waits for). SPEC-074 tunes these.
#
# `wasm-opt` does NOT run (DEC-066): on this binary it trades 340 KB of raw size for
# 36 KB MORE on the wire, and buys no speed. The brotli line is the one to watch.
wasm-size:
    @test -f pkg/crustyimg_bg.wasm || { echo "no pkg/crustyimg_bg.wasm — run 'just wasm-build' first"; exit 1; }
    @echo ""
    @echo "── .wasm size (features: {{ if _wasm_features == '' { 'none (lean)' } else { _wasm_features } }}) ──"
    @awk 'BEGIN{printf "  raw:     %8.2f MB  (%d B)\n", '"$(wc -c < pkg/crustyimg_bg.wasm)"'/1048576, '"$(wc -c < pkg/crustyimg_bg.wasm)"'}'
    @awk 'BEGIN{printf "  gzip:    %8.2f MB  (%d B)\n", '"$(gzip -9 -c pkg/crustyimg_bg.wasm | wc -c)"'/1048576, '"$(gzip -9 -c pkg/crustyimg_bg.wasm | wc -c)"'}'
    @command -v brotli >/dev/null 2>&1 && awk 'BEGIN{printf "  brotli:  %8.2f MB  (%d B)  ← the number that matters\n", '"$(brotli -q 11 -c pkg/crustyimg_bg.wasm 2>/dev/null | wc -c)"'/1048576, '"$(brotli -q 11 -c pkg/crustyimg_bg.wasm 2>/dev/null | wc -c)"'}' || echo "  brotli:  (brotli not installed)"

# Run the #[wasm_bindgen_test] round-trip in Node — the REAL decode → transform →
# encode proof. A green `wasm-check` only says it compiles; this says it works.
#
# Runs through plain cargo (NOT `wasm-pack test`) so that `--test wasm_roundtrip`
# actually scopes the build to the one wasm test target — see .cargo/config.toml for
# why, and for the `wasm-bindgen-test-runner` shim that executes the .wasm in Node.
# Needs: cargo install wasm-bindgen-cli --version <the pinned wasm-bindgen version>
wasm-test:
    @command -v wasm-bindgen-test-runner >/dev/null 2>&1 || { echo "wasm-bindgen-test-runner not installed (cargo install wasm-bindgen-cli --version 0.2.126)"; exit 1; }
    PATH="{{_wasm_bin}}:$PATH" RUSTC="{{_wasm_bin}}/rustc" \
        "{{_wasm_bin}}/cargo" test --target wasm32-unknown-unknown --test wasm_roundtrip {{_wasm_features}}

# Build the FINAL npm package into pkg/ (SPEC-075, DEC-067): the size-profiled
# `wasm-build` above, plus the npm identity wasm-pack cannot know — it names the
# package after the crate (`crustyimg`, which is the CLI) and describes it as a CLI.
#
# Depends on `wasm-build` ON PURPOSE: the packaging step must never be reachable
# without going through the size profile (DEC-066), or the package silently ships a
# stock-profile .wasm, +109 KB on the wire.
wasm-npm-pkg: wasm-build
    @node scripts/wasm-npm-finalize.mjs

# The npm package's earned verdict (SPEC-075): `npm pack` the finalized pkg/, install
# THAT TARBALL into a fresh temp project, and run it from inside — init the wasm, then
# info + transform a real PNG and decode the output back. Also asserts the pitch: no
# native addon, no postinstall build, and a .wasm that is the profiled ~1.33 MB.
#
# Does NOT publish. `npm publish` is outward-facing and effectively irreversible
# (npm's unpublish window is narrow) — it is SPEC-076, gated on maintainer approval.
wasm-npm-smoke: wasm-npm-pkg
    @node tests/npm_smoke.mjs

# ----------------------------------------------------------------------------
# DEMO (SPEC-077) — the wasm engine as a real web page
# ----------------------------------------------------------------------------
#
# `demo/` is a plain static site: index.html + demo.js + demo.css + worker.js,
# importing the wasm package directly. No bundler, no framework, no npm install — the
# only build step is `wasm-build` plus a copy.
#
# THE ENGINE RUNS IN A WEB WORKER (SPEC-078). rav1e (AVIF encode) is serial and takes
# seconds, and a wasm call is synchronous — on the page's thread it would freeze the
# tab. worker.js is a module Worker that init()s the wasm and does every conversion,
# so the page stays responsive. A Worker is a separate THREAD, not shared memory: no
# SharedArrayBuffer, no wasm threads, and therefore no COOP/COEP headers (which
# GitHub Pages could not set anyway).
#
# ⚠ THE DEMO MUST BE SERVED OVER HTTP. Opened as a `file://` URL it cannot work, and
# the reason is EARLIER than the MIME type: demo.js is an ES module, module scripts are
# fetched under CORS, and a file:// origin is opaque — so the browser blocks the module
# before it executes a line (measured in Chrome: demo.js is fetched and refused,
# crustyimg_bg.wasm is never even requested). `init()` is never reached, so the
# `instantiateStreaming`/`application/wasm` problem — real, and the reason this server
# exists — would only bite second. index.html carries a classic (non-CORS-fetched)
# script that survives to say so, rather than leaving the page on "Loading…" forever.
# `demo-serve` below serves it correctly; so does GitHub Pages. See demo/README.md.

# Assemble the demo: build the wasm THROUGH `wasm-build` (the size profile lives in
# that recipe — DEC-066, same discipline as `wasm-npm-pkg`), then vendor pkg/ into
# demo/vendor/. The assembler REFUSES a .wasm that did not come through the profiled
# build, so a demo deploy cannot ship a bare-`cargo build` artifact.
demo-build: wasm-build
    @node scripts/demo-assemble.mjs

# Serve the assembled demo locally, with the wasm MIME type the browser demands.
# Open the URL it prints — do not open demo/index.html from the filesystem.
demo-serve port="8080": demo-build
    @node scripts/serve.mjs demo {{port}}

# The demo's earned verdict (SPEC-077, SPEC-078): serve it, load it in a REAL headless
# Chrome, and drive the actual user path — init() the engine over HTTP, put a PNG in
# the file picker, convert it, and decode the downloaded bytes with an independent
# parser. Also asserts the pitch: the .wasm arrives as application/wasm, and the
# conversion makes zero network requests.
#
# SPEC-078 added the claims a Worker makes possible, each one driven rather than
# asserted: the engine init()s inside a module Worker (its own CDP target, which is
# also where the .wasm fetch now shows up); the main thread keeps running timers AND
# animation frames THROUGHOUT a slow AVIF encode; the PNG → AVIF output is valid AVIF
# per decoders the crate never met (an ISOBMFF parse here, Chrome's libavif, and
# `sips` on macOS — the engine cannot check this one itself, DEC-065); an .avif INPUT
# converts through the browser's decoder; and the readout shows bytes, % saved, and
# the chosen format.
#
# A page that renders perfectly but cannot instantiateStreaming is the failure this
# exists to catch, so it must be a browser — Node cannot see it. Needs Chrome or
# Chromium (set CHROME=/path/to/chrome to point it elsewhere); no browser driver is
# installed — Chrome is driven over the DevTools Protocol directly.
demo-smoke: demo-build
    @node tests/demo_smoke.mjs

# Lint with clippy, warnings as errors (the CI gate, AGENTS §6)
lint:
    cargo clippy -- -D warnings

# Lint including all test/bench targets (stricter; what CI gates once #10 lands)
lint-all:
    cargo clippy --all-targets -- -D warnings

# Format all code in place
fmt:
    cargo fmt

# Check formatting without modifying (the CI gate)
fmt-check:
    cargo fmt --check

# Run every gate the way CI does: fmt-check + clippy + build + test
check: fmt-check lint build test
    @echo "✓ all gates passed"

# Gates with the display feature compiled in (keeps the viuer path green, DEC-011)
check-display:
    cargo fmt --check
    cargo clippy --features display -- -D warnings
    cargo build --features display

# cargo-audit over the dependency tree (security advisories; install: cargo install cargo-audit)
audit:
    cargo audit

# Full supply-chain gate: advisories + bans + sources + licenses (DEC-037/DEC-018;
# install: cargo install cargo-deny). Mirrors the CI `supply-chain` job exactly.
deny:
    cargo deny check advisories bans sources licenses

# Fail if any shipped spec is missing real build/verify cost data
# (AGENTS.md §4 / docs/cost-tracking.md). Same check the CI `cost-data` job runs.
cost-audit:
    @./scripts/cost-audit.sh

# Strict-parse every tracked .md/.yaml/.yml front-matter block; grep-based
# tooling can't see these. Needs `ruby` (stdlib yaml); the CI `metadata` job.
# Fail on any front-matter block a real YAML parser rejects.
validate:
    @./scripts/validate-frontmatter.sh

# Build and open the crate's API docs
doc:
    cargo doc --no-deps --open

# Remove build artifacts (target/)
clean-build:
    cargo clean

# Install crustyimg to ~/.cargo/bin (lean — no terminal display)
install:
    cargo install --path .

# Install with terminal display so `view` renders (pulls viuer, DEC-011)
install-display:
    cargo install --path . --features display

# ----------------------------------------------------------------------------
# DECODER FUZZING (SPEC-069, DEC-062)  —  nightly-only, NOT part of `just check`
#
# The `fuzz/` crate is a detached workspace (its own empty [workspace]); the
# repo-root build and CI never touch it. Fuzzing needs a nightly toolchain +
# cargo-fuzz, a one-time setup:
#     rustup toolchain install nightly
#     cargo install cargo-fuzz
# HEIC additionally needs a system libheif (`brew install libheif`) and
# `--features heic` (best-effort; see DEC-062).
#
# The DURABLE, per-PR guard is `cargo test` (tests/fuzz_regressions.rs — the
# regressions + the fuzz_corpus_never_panics smoke run on every PR, 3 OSes,
# without a fuzzer). THIS recipe is the periodic deep run that finds NEW crashes.
# Targets: avif_decode  svg_decode  raw_preview  heic_decode.
# ----------------------------------------------------------------------------

# Fuzz one decoder target, seeded from its committed corpus. Usage:
#     just fuzz avif_decode          # 600s budget (the DEC-062 floor)
#     just fuzz svg_decode 300       # custom wall-clock seconds
#
# Runs in `-O`/release config so the fuzzed code matches the SHIPPED binary
# (production has debug-assertions off; ASAN stays on). Running with
# debug-assertions on additionally trips upstream `debug_assert!`s that are
# compiled out in release (e.g. avif-parse's parser-state checks) — see
# docs/research/proj-009-fuzz-run.md. New coverage is written to the gitignored
# fuzz/corpus/<target>/; a crash lands in fuzz/artifacts/<target>/ (minimize with
# `cargo +nightly fuzz tmin <target> <artifact>`, then commit under
# tests/fixtures/fuzz/<target>/ and add a regression).
fuzz TARGET SECONDS="600":
    #!/usr/bin/env bash
    set -euo pipefail
    fmt="{{TARGET}}"; fmt="${fmt%_*}"      # avif_decode→avif, raw_preview→raw, …
    mkdir -p "fuzz/corpus/{{TARGET}}"
    # Seed the (gitignored) corpus from the committed seeds + crash reproducers,
    # so a fresh clone starts from a known-good set without polluting fixtures.
    cp tests/fixtures/"$fmt"/* "fuzz/corpus/{{TARGET}}/" 2>/dev/null || true
    cp tests/fixtures/fuzz/{{TARGET}}/* "fuzz/corpus/{{TARGET}}/" 2>/dev/null || true
    echo "seeded fuzz/corpus/{{TARGET}} from tests/fixtures/$fmt (+ crash corpus)"
    cargo +nightly fuzz run -O {{TARGET}} "fuzz/corpus/{{TARGET}}" -- -max_total_time={{SECONDS}}

# ----------------------------------------------------------------------------
# ONE-TIME SETUP
# ----------------------------------------------------------------------------

# Initialize the repo: pick a variant and scaffold files to the root.
init:
    @echo "Spec-driven repo template — init"
    @echo ""
    @if [ -f AGENTS.md ]; then \
        echo "⚠  Already initialized (AGENTS.md exists at repo root)."; \
        echo "   Init is one-shot: it consumes variants/ when it runs."; \
        echo "   To start over, restore the repo from git or re-clone."; \
        exit 1; \
    fi
    @if [ ! -d variants ]; then \
        echo "⚠  variants/ directory is missing."; \
        echo "   This repo was already initialized (or the template was"; \
        echo "   modified). Restore from git or re-clone to re-init."; \
        exit 1; \
    fi
    @echo "Pick a variant:"
    @echo "  1) claude-only         (Claude plays every role; simpler)"
    @echo "  2) claude-plus-agents  (Claude architects, separate agent implements)"
    @echo ""
    @printf "Enter 1 or 2: "
    @read variant_choice && \
    if [ "$variant_choice" = "1" ]; then \
        VARIANT="claude-only"; \
    elif [ "$variant_choice" = "2" ]; then \
        VARIANT="claude-plus-agents"; \
    else \
        echo "Invalid choice: $variant_choice"; exit 1; \
    fi && \
    echo "" && \
    echo "Scaffolding $VARIANT to repo root..." && \
    cp -r "variants/$VARIANT/." . && \
    rm -rf variants/ && \
    echo "$VARIANT" > .variant && \
    echo "" && \
    echo "✓ Done. Your variant: $VARIANT" && \
    echo "" && \
    echo "Next steps:" && \
    echo "  1. Open GETTING_STARTED.md" && \
    echo "  2. Work through the PROJECT FRAME prompt in FIRST_SESSION_PROMPTS.md" && \
    echo "  3. Commit the scaffolded repo:" && \
    echo "       git add . && git commit -m 'chore: initialize spec-driven scaffold'"

# List the available variants (useful before init)
list-variants:
    @echo "Available variants:"
    @echo "  claude-only         — Claude plays every role; no handoff documents"
    @echo "  claude-plus-agents  — Claude architects, separate agent implements; adds /handoffs/"
    @echo ""
    @echo "Run 'just init' to pick one."

# ----------------------------------------------------------------------------
# DAILY COMMANDS (work after `just init`)
# ----------------------------------------------------------------------------

# Print repo state: active project, stage, specs by cycle, stale items.
# `just status --json` emits the same state as JSON (needs ruby).
status *FLAGS:
    @./scripts/status.sh {{FLAGS}}

# Scaffold a new spec. Usage: just new-spec "short title" STAGE-NNN [PROJ-NNN]
new-spec TITLE STAGE_ID PROJECT_ID="":
    @./scripts/new-spec.sh "{{TITLE}}" "{{STAGE_ID}}" "{{PROJECT_ID}}"

# Scaffold a new stage. Usage: just new-stage "short title" [PROJ-NNN]
new-stage TITLE PROJECT_ID="":
    @./scripts/new-stage.sh "{{TITLE}}" "{{PROJECT_ID}}"

# Advance a spec's cycle. Usage: just advance-cycle SPEC-NNN verify
advance-cycle SPEC_ID NEW_CYCLE:
    @./scripts/advance-cycle.sh "{{SPEC_ID}}" "{{NEW_CYCLE}}"

# Archive a shipped spec: move to done/ and update stage backlog.
# Usage: just archive-spec SPEC-NNN
archive-spec SPEC_ID:
    @./scripts/archive-spec.sh "{{SPEC_ID}}"

# Prep a release: bump Cargo.toml + refresh Cargo.lock + guard tag==version + CHANGELOG.
# Does NOT commit/tag/push (maintainer-authorized). Usage: just release 0.1.2
release VERSION:
    @./scripts/release.sh "{{VERSION}}"

# Print the Weekly Review prompt with recent activity pre-loaded
weekly-review:
    @./scripts/weekly-review.sh

# Generate today's daily report under reports/daily/YYYY-MM-DD.md
report-daily:
    @./scripts/report_daily.sh

# Generate this week's weekly report under reports/weekly/YYYY-WNN.md.
# Pass a YYYY-MM-DD to report on the ISO week containing that date.
report-weekly DATE="":
    @./scripts/report_weekly.sh "{{DATE}}"

# Capture today's `just status` output to reports/daily/YYYY-MM-DD-status.md.
# Lighter than report-daily — a snapshot of current state with no curation.
daily-status-report:
    @mkdir -p reports/daily
    @D="$(date +%Y-%m-%d)"; \
        { echo "# Daily status - $D"; echo; ./scripts/status.sh; } > "reports/daily/$D-status.md"; \
        echo "✓ Wrote reports/daily/$D-status.md"

# Spec-grained "what's next?" view: in-flight specs in the active
# stage, un-promoted bullets in the active stage's backlog, and
# counts in upcoming stages. Pass --all to widen scope.
backlog *FLAGS:
    @./scripts/backlog.sh {{FLAGS}}

# Stage-grained "where is this project going" view: one row per
# stage in the active project with status, date range, and (for
# active/upcoming) spec counts.
roadmap:
    @./scripts/roadmap.sh

# Flat ledger of every spec grouped by stage, with ship date and
# complexity. Defaults to ALL projects (history); pass `--active` for
# the current project or a `PROJ-NNN` id for a specific one.
specs-by-stage *FLAGS:
    @./scripts/specs-by-stage.sh {{FLAGS}}

# Audit decisions: structural lint + scope-conflict warnings (zero
# deps; a native take on LineSpec-style provenance auditing). Lints
# front-matter and supersession links across all DEC-* files. Pass
# `--changed [BASE]` to flag which decisions govern your pending changes.
decisions-audit *FLAGS:
    @./scripts/decisions-audit.sh {{FLAGS}}

# Show the license watchlist: capabilities declined for license reasons (plus
# non-license capability gaps) with their permissive alternatives / build paths.
watchlist:
    @echo "License watchlist (guidance/license-watchlist.yaml) — id · status · what:"
    @grep -E '^  - id:|^    capability:|^    status:|^    rejected_dependency:|^    deferred_dependency:|^    revisit_trigger:' guidance/license-watchlist.yaml

# ----------------------------------------------------------------------------
# HELPERS
# ----------------------------------------------------------------------------

# Print the active project and variant
info:
    @./scripts/info.sh

# Run the template's end-to-end happy-path tests (uses a temp dir).
# Intended for template maintainers, not end users. Works from the
# pre-init template root only — after `just init` runs, variants/ is
# gone and this test would fail at the first check. (Renamed from `test`
# so the canonical `just test` runs the app's `cargo test`.)
template-test:
    @./scripts/test.sh
