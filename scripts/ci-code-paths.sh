#!/usr/bin/env bash
# Classify changed paths for the `changes` job in .github/workflows/ci.yml.
#
# Reads file names (one per line) on stdin and prints `false` only when every one
# of them is provably docs-only, otherwise `true`. Anything not provably docs-only
# runs the gates: a path outside the DOCS list below, including a new top-level
# directory nobody has classified yet, counts as code. A docs-shaped file that a
# build or a test reads goes in EXCEPT, which wins over DOCS. Blank lines are
# ignored; an empty list prints `true`, because nothing was proved docs-only.
#
#   git diff --name-only base...HEAD | scripts/ci-code-paths.sh
#   scripts/ci-code-paths.sh --self-test
set -euo pipefail

# Docs and process files: nothing compiles them and no gated job reads them. Every
# alternative is anchored at the repository root.
#   docs/ projects/ decisions/ guidance/ feedback/ reports/  prose, specs, ADRs, process records
#   gitignore_files_maybe/                                   parked notes, read by nothing
#   skills/ .claude-plugin/                                  agent-skill text and its plugin manifest
#   [^/]+\.md$                                               a Markdown file at the root only
#   LICENSE-[A-Z]+$                                          licence texts
#   .repo-context.yaml .variant .gitignore                   repo metadata; no build reads them
#   .github/TEMPLATE_README.md                               template prose, not a workflow
DOCS='^(docs/|projects/|decisions/|guidance/|feedback/|reports/|gitignore_files_maybe/|skills/|\.claude-plugin/|[^/]+\.md$|LICENSE-[A-Z]+$|\.repo-context\.yaml$|\.variant$|\.gitignore$|\.github/TEMPLATE_README\.md$)'

# Docs-shaped files that a test reads, so a change to one must run the gates:
#   README.md               tests/adoption_glue.rs reads it
#   docs/github-actions.md  tests/lint.rs includes it
#   docs/data-model.md      tests/docs_ops.rs reads it
# A file a build or a test includes with include_str!/include_bytes! is checked by
# the self-test below; a file read at run time must be listed here by hand.
EXCEPT='^(README\.md|docs/github-actions\.md|docs/data-model\.md)$'

classify() {
  local paths
  paths=$(grep -v '^$' || true)
  if [ -z "$paths" ]; then
    echo true
  elif printf '%s\n' "$paths" | grep -qE "$EXCEPT"; then
    echo true
  elif printf '%s\n' "$paths" | grep -qvE "$DOCS"; then
    echo true
  else
    echo false
  fi
}

# Every file named by a literal include_str!/include_bytes! under src/, tests/,
# benches/, examples/ or build.rs, as a repository path, one per line. Comment lines
# are skipped. The path is resolved against the including file's folder, as rustc does.
include_targets() {
  local roots=() r
  for r in src tests benches examples build.rs; do
    if [ -e "$r" ]; then roots+=("$r"); fi
  done
  grep -rnE --include='*.rs' 'include_(str|bytes)!\("' "${roots[@]}" | awk '
    {
      file = $0; sub(/:.*/, "", file)
      text = $0; sub(/^[^:]*:[0-9]+:/, "", text)
      if (text ~ /^[ \t]*\/\//) next
      dir = file; sub(/\/[^\/]*$/, "", dir)
      if (dir == file) dir = "."
      while (match(text, /include_(str|bytes)!\("[^"]+"\)/)) {
        lit = substr(text, RSTART, RLENGTH)
        text = substr(text, RSTART + RLENGTH)
        sub(/^include_(str|bytes)!\("/, "", lit); sub(/"\)$/, "", lit)
        n = split(dir "/" lit, part, "/"); depth = 0
        for (i = 1; i <= n; i++) {
          if (part[i] == "" || part[i] == ".") continue
          if (part[i] == "..") { if (depth > 0) depth--; continue }
          out[++depth] = part[i]
        }
        path = out[1]
        for (i = 2; i <= depth; i++) path = path "/" out[i]
        print path
      }
    }' | sort -u
}

# Include forms the scan above cannot resolve: a path built with concat!/env!.
unresolvable_includes() {
  local roots=() r
  for r in src tests benches examples build.rs; do
    if [ -e "$r" ]; then roots+=("$r"); fi
  done
  grep -rnE --include='*.rs' 'include_(str|bytes)!\((concat|env)!' "${roots[@]}" || true
}

# One of pages.yml's two `paths:` lists (push or pull_request), one entry per line.
pages_paths() {
  awk -v want="$1" '
    /^  [a-z_]+:/ { section = $1; sub(/:$/, "", section); inpaths = 0; next }
    section == want && /^    paths:/ { inpaths = 1; next }
    inpaths && /^      - / { e = $0; sub(/^      - /, "", e); gsub(/"/, "", e); print e; next }
    inpaths && /^ *#/ { next }
    inpaths { inpaths = 0 }
  ' .github/workflows/pages.yml
}

if [ "${1:-}" = "--self-test" ]; then
  # The derived checks read the tree; run them from the repository root.
  cd "$(dirname "$0")/.."
  rows=0 mismatched=0 crashed=0 junk=0 failed=0
  expect() {
    local want="$1" path="$2" got rc=0
    rows=$((rows + 1))
    # Assign first, read the exit code after: `set -e` must not end the run on a crash.
    got=$(printf '%s\n' "$path" | classify) || rc=$?
    if [ "$rc" -ne 0 ]; then
      echo "FAIL $path -> classifier exited $rc"
      crashed=$((crashed + 1))
    elif [ "$got" != "true" ] && [ "$got" != "false" ]; then
      echo "FAIL $path -> printed '$got', expected true or false"
      junk=$((junk + 1))
    elif [ "$got" = "$want" ]; then
      echo "PASS $path -> $got"
    else
      echo "FAIL $path -> $got, expected $want"
      mismatched=$((mismatched + 1))
    fi
  }
  # Check rows that are not a single path: print PASS/FAIL with a reason.
  check() {
    local ok="$1" label="$2"
    rows=$((rows + 1))
    if [ "$ok" = "yes" ]; then
      echo "PASS $label"
    else
      echo "FAIL $label"
      failed=$((failed + 1))
    fi
  }

  # Code: the classic roots.
  expect true integrations/ddphotos/edit/edit.js
  expect true src/lib.rs
  expect true .cargo/config.toml
  expect true .github/workflows/ci.yml
  # Code: build and test inputs outside the classic roots.
  expect true recipes/web.toml
  expect true assets/fonts/Go-Regular.ttf
  expect true clippy.toml
  expect true deny.toml
  expect true .pre-commit-hooks.yaml
  expect true demo/index.html
  expect true bench/corpus/README.md
  expect true fuzz/Cargo.toml
  # Code: the docs-shaped test inputs in EXCEPT.
  expect true README.md
  expect true docs/github-actions.md
  expect true docs/data-model.md
  # Code: a top-level directory nobody has classified yet.
  expect true newdir/x.txt
  # Code: DOCS is anchored at the root, and only a root-level *.md is docs.
  expect true src/docs/mod.rs
  expect true tests/fixtures/x.md
  # Docs-only: one path per DOCS alternative.
  expect false docs/x.md
  expect false docs/.cargo/config.toml
  expect false docs/recipes/web.toml
  expect false projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-x.md
  expect false decisions/DEC-001-x.md
  expect false guidance/constraints.yaml
  expect false feedback/x.md
  expect false reports/x.md
  expect false gitignore_files_maybe/x.txt
  expect false skills/crustyimg/SKILL.md
  expect false .claude-plugin/plugin.json
  expect false CHANGELOG.md
  expect false LICENSE-MIT
  expect false .repo-context.yaml
  expect false .variant
  expect false .gitignore
  expect false .github/TEMPLATE_README.md

  # Lists, not single paths.
  got=$(printf '' | classify)
  check "$([ "$got" = true ] && echo yes || echo no)" "empty list -> $got, expected true"
  got=$(printf 'docs/x.md\n\nCHANGELOG.md\n' | classify)
  check "$([ "$got" = false ] && echo yes || echo no)" "docs-only list with a blank line -> $got, expected false"
  got=$(printf 'docs/x.md\nsrc/lib.rs\n' | classify)
  check "$([ "$got" = true ] && echo yes || echo no)" "docs plus one code path -> $got, expected true"

  # Every file a build or a test includes must classify as code.
  INCLUDE_FLOOR=35
  targets=$(include_targets)
  count=$(printf '%s\n' "$targets" | grep -c . || true)
  check "$([ "$count" -ge "$INCLUDE_FLOOR" ] && echo yes || echo no)" \
    "include scan found $count files, expected at least $INCLUDE_FLOOR (lower INCLUDE_FLOOR only if an include was removed on purpose)"
  while IFS= read -r t; do
    [ -n "$t" ] || continue
    if printf '%s\n' "$t" | grep -qE '(^|/)\.\.?(/|$)'; then
      check no "include target $t is not a repository path (the scan did not collapse . or ..)"
    elif [ ! -f "$t" ]; then
      check no "include target $t does not exist (the scan resolved a path wrong)"
    elif [ "$(printf '%s\n' "$t" | classify)" != true ]; then
      check no "include target $t classifies as docs-only; add it to EXCEPT"
    else
      check yes "include target $t -> true"
    fi
  done <<< "$targets"
  bad=$(unresolvable_includes)
  check "$([ -z "$bad" ] && echo yes || echo no)" \
    "no include_str!/include_bytes! builds its path with concat!/env!${bad:+: $bad}"

  # pages.yml's two lists must match, list the demo's inputs, and be code paths.
  push_paths=$(pages_paths push)
  pr_paths=$(pages_paths pull_request)
  check "$([ -n "$push_paths" ] && [ "$push_paths" = "$pr_paths" ] && echo yes || echo no)" \
    "pages.yml push.paths and pull_request.paths are identical and not empty"
  for need in 'recipes/**' 'assets/**'; do
    check "$(printf '%s\n' "$push_paths" | grep -qxF "$need" && echo yes || echo no)" \
      "pages.yml lists $need"
  done
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    sample=$(printf '%s' "$p" | sed 's/\*\*/x/g; s/\*/x/g')
    check "$([ "$(printf '%s\n' "$sample" | classify)" = true ] && echo yes || echo no)" \
      "pages.yml path $p classifies as code"
  done <<< "$push_paths"

  echo "ci-code-paths self-test: $rows rows, $mismatched mismatched, $crashed crashed, $junk junk, $failed failed"
  [ $((mismatched + crashed + junk + failed)) -eq 0 ]
  exit
fi

classify
