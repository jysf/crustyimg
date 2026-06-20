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
bench:
    cargo bench

# Wall-clock the release binary with hyperfine. Skips cleanly (exit 0) if hyperfine
# is not installed. Usage: just bench-cli shrink photo.jpg --max 800 -o /tmp/o.jpg
bench-cli *ARGS:
    @command -v hyperfine >/dev/null 2>&1 || { echo "hyperfine not installed; skipping (brew install hyperfine)"; exit 0; }
    cargo build --release
    hyperfine --warmup 2 './target/release/crustyimg {{ARGS}}'

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

# Print repo state: active project, stage, specs by cycle, stale items
status:
    @./scripts/status.sh

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
