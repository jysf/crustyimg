#!/usr/bin/env bash
# Classify changed paths for the `changes` job in .github/workflows/ci.yml.
#
# Reads file names (one per line) on stdin and prints `true` when any of them can
# affect a build, a test or a lint result, otherwise `false`. The one regex lives
# here so the workflow and its self-test cannot drift apart.
#
#   git diff --name-only base...HEAD | scripts/ci-code-paths.sh
#   scripts/ci-code-paths.sh --self-test
set -euo pipefail

# integrations/ddphotos/ is the edit page: a change to its code must run the heavy
# jobs, including the native-vs-wasm parity check of its recipes.
CODE_PATHS='^(src/|tests/|examples/|benches/|Cargo\.(toml|lock)|rust-toolchain|\.github/workflows/|justfile|scripts/|integrations/ddphotos/)'

classify() {
  if grep -qE "$CODE_PATHS"; then echo true; else echo false; fi
}

if [ "${1:-}" = "--self-test" ]; then
  failed=0
  expect() {
    local want="$1" path="$2" got
    got=$(printf '%s\n' "$path" | classify)
    if [ "$got" = "$want" ]; then
      echo "PASS $path -> $got"
    else
      echo "FAIL $path -> $got, expected $want"
      failed=1
    fi
  }
  expect true integrations/ddphotos/edit/edit.js
  expect false docs/x.md
  expect false projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-x.md
  expect true src/lib.rs
  exit "$failed"
fi

classify
