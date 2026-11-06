#!/usr/bin/env bash
# Helpers for the `wasm-roundtrip` job in .github/workflows/ci.yml.
#
#   scripts/wasm-ci-check.sh lock-version [Cargo.lock]   print the locked wasm-bindgen version
#   scripts/wasm-ci-check.sh runner-version [Cargo.lock] assert the installed runner matches it
#   scripts/wasm-ci-check.sh count <full|slim> <log>     assert the exact passed-test count
#   scripts/wasm-ci-check.sh workflow [ci.yml]           assert the job's shape
#   scripts/wasm-ci-check.sh --self-test                 drive every parser above
#
# Exit 0 ok, 1 check failed, 2 usage. Errors print `::error::wasm-ci-check: ...`
# (GitHub annotates the line; locally it is plain text). CI runs lock-version, runner-version,
# count and workflow, plus --self-test; compare-runner is called only by --self-test.
set -euo pipefail

# How many #[wasm_bindgen_test] tests each feature set must run. Exact, not a minimum: a
# minimum cannot tell the slim set from the full one. Whoever adds or removes a wasm test
# updates these in the same PR; the CI message names them.
FULL_COUNT=49   # just wasm-test       (--features avif,svg)
SLIM_COUNT=47   # just wasm-test-slim  (--features avif): 3 svg tests out, 1 refusal test in

SELF="$0"

# To stderr: lock_version's caller captures stdout with $(...), and an error on stdout would
# vanish into the variable and leave CI silent.
err() { echo "::error::wasm-ci-check: $*" >&2; }

# Print the version of the single `wasm-bindgen` package in a lockfile. Matches the whole
# name line, so wasm-bindgen-shared / -macro / -backend are not counted.
lock_version() {
  local lock="${1:-Cargo.lock}" versions count version
  versions=$(awk '
    /^name = "wasm-bindgen"$/ { want = 1; next }
    want && /^version = "/ { gsub(/^version = "|"$/, ""); print; want = 0; next }
    /^\[\[package\]\]/ { want = 0 }
  ' "$lock")
  count=$(printf '%s' "$versions" | grep -c . || true)
  if [ "$count" -ne 1 ]; then
    err "found $count wasm-bindgen entries in $lock, expected 1"
    return 1
  fi
  version="$versions"
  if ! printf '%s' "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    err "version '$version' is not x.y.z"
    return 1
  fi
  echo "$version"
}

# $1 = what `wasm-bindgen-test-runner --version` printed, $2 = the lock version.
# Compares the last field as a whole string (a prefix compare would pass 0.2.1260).
compare_runner() {
  local output="$1" lock="$2" got
  got=$(printf '%s' "$output" | awk 'NF { last = $NF } END { print last }')
  if [ -z "$got" ]; then
    err "wasm-bindgen-test-runner printed no version"
    return 1
  fi
  if [ "$got" != "$lock" ]; then
    err "wasm-bindgen-test-runner is $got but Cargo.lock pins wasm-bindgen $lock; install the exact lock version"
    return 1
  fi
  echo "wasm-bindgen-test-runner $got matches Cargo.lock"
}

runner_version() {
  local lock="${1:-Cargo.lock}" want out
  if ! command -v wasm-bindgen-test-runner >/dev/null 2>&1; then
    err "wasm-bindgen-test-runner not found on PATH"
    return 1
  fi
  want=$(lock_version "$lock") || return 1
  out=$(wasm-bindgen-test-runner --version 2>&1) || true
  compare_runner "$out" "$want"
}

count_tests() {
  local set="${1:-}" log="${2:-}" expected const lines n
  case "$set" in
    full) expected=$FULL_COUNT; const=FULL_COUNT ;;
    slim) expected=$SLIM_COUNT; const=SLIM_COUNT ;;
    *) err "unknown set '$set' (expected full or slim)"; return 1 ;;
  esac
  lines=$(grep -E '^test result: ' "$log" || true)
  n=$(printf '%s' "$lines" | grep -c . || true)
  if [ "$n" -eq 0 ]; then
    err "$set set: no 'test result:' line in $log"
    return 1
  elif [ "$n" -ne 1 ]; then
    err "$set set: $n 'test result:' lines, expected 1"
    return 1
  fi
  if ! printf '%s' "$lines" | grep -qE '^test result: ok\. [0-9]+ passed; 0 failed;'; then
    err "$set set: the test run did not pass: $lines"
    return 1
  fi
  n=$(printf '%s' "$lines" | sed -E 's/^test result: ok\. ([0-9]+) passed;.*/\1/')
  if [ "$n" -ne "$expected" ]; then
    err "$set set: $n passed, expected $expected ($const in scripts/wasm-ci-check.sh). Update $const if you added or removed a #[wasm_bindgen_test]; otherwise tests were compiled out."
    return 1
  fi
  echo "wasm-ci-check: $set set: $n passed, as expected"
}

# Shape check of the wasm-roundtrip job. Ruby (stdlib yaml) is preinstalled on
# ubuntu-latest, as scripts/validate-frontmatter.sh relies on.
workflow_shape() {
  local file="${1:-.github/workflows/ci.yml}"
  command -v ruby >/dev/null 2>&1 || { err "ruby not found (needed to parse $file)"; return 1; }
  ruby -ryaml - "$file" <<'RUBY'
file = ARGV[0]
doc = YAML.safe_load(File.read(file), aliases: true)
jobs = doc["jobs"] || {}
job = jobs["wasm-roundtrip"] || {}
steps = job["steps"] || []
total = 0
failed = 0
check = lambda do |ok, what, expected|
  total += 1
  if ok
    puts "PASS #{what}"
  else
    failed += 1
    puts "FAIL #{what}, expected #{expected}"
  end
end
uses = ->(prefix) { steps.index { |s| s["uses"].to_s == prefix } }
run_has = ->(*needles) { steps.index { |s| needles.all? { |n| s["run"].to_s.include?(n) } } }
needs = job["needs"]
check.(!job.empty? && !job.key?("strategy") && job["name"] == "wasm round-trip tests (Node)",
       "job wasm-roundtrip exists, non-matrix, named 'wasm round-trip tests (Node)'",
       "non-matrix job wasm-roundtrip named 'wasm round-trip tests (Node)'")
check.(needs == "changes" || (needs.is_a?(Array) && needs.include?("changes")),
       "job needs changes", "needs: changes")
check.(job["if"] == "needs.changes.outputs.code == 'true'",
       "job if is the code gate", "if: needs.changes.outputs.code == 'true'")
tc = steps.find { |s| s["uses"].to_s == "dtolnay/rust-toolchain@stable" }
check.(tc && tc.dig("with", "targets").to_s.include?("wasm32-unknown-unknown"),
       "toolchain step has the wasm32 target", "dtolnay/rust-toolchain@stable with targets wasm32-unknown-unknown")
sn = steps.find { |s| s["uses"].to_s == "actions/setup-node@v6" }
check.(sn && sn.dig("with", "node-version").to_s == "22",
       "setup-node pins node 22", "actions/setup-node@v6 with node-version '22'")
inst_i = uses.("taiki-e/install-action@v2")
inst = inst_i && steps[inst_i]
check.(inst && inst.dig("with", "tool").to_s.start_with?("wasm-bindgen@${{ steps.lock.") &&
         inst.dig("with", "fallback").to_s == "none",
       "install step pins wasm-bindgen from steps.lock with fallback none",
       "taiki-e/install-action@v2, tool wasm-bindgen@${{ steps.lock.… }}, fallback: none")
rv = run_has.("scripts/wasm-ci-check.sh runner-version")
check.(rv && inst_i && rv > inst_i,
       "runner-version step runs after the install step", "a runner-version step after the install step")
rc = steps.find { |s| s["uses"].to_s == "Swatinem/rust-cache@v2" }
check.(rc && rc.dig("with", "key").to_s == "wasm-roundtrip",
       "rust-cache key is wasm-roundtrip", "Swatinem/rust-cache@v2 with key wasm-roundtrip")
check.(steps.any? { |s| s["run"].to_s.strip == "just wasm-check" },
       "a step runs 'just wasm-check'", "a step running 'just wasm-check'")
full = steps.index { |s| r = s["run"].to_s; r =~ /^\s*just wasm-test\s*>/ && r =~ /^\s*scripts\/wasm-ci-check\.sh count full "\$log"\s*$/ }
# The count call must be a whole line: a substring match accepts a comment, `|| true` or `echo`.
check.(!full.nil?, "full test step runs 'just wasm-test' and the full count check",
       "a step with 'just wasm-test' and 'scripts/wasm-ci-check.sh count full'")
slim = steps.index { |s| r = s["run"].to_s; r =~ /^\s*just wasm-test-slim\s*>/ && r =~ /^\s*scripts\/wasm-ci-check\.sh count slim "\$log"\s*$/ }
check.(!slim.nil?, "slim test step runs 'just wasm-test-slim' and the slim count check",
       "a step with 'just wasm-test-slim' and 'scripts/wasm-ci-check.sh count slim'")
check.(rv && full && slim && rv < full && rv < slim,
       "the runner assertion precedes both test steps", "runner-version step before both test steps")
check.(!steps.any? { |s| s.key?("continue-on-error") } && !job.key?("continue-on-error"),
       "no continue-on-error in the job or its steps", "no continue-on-error anywhere in wasm-roundtrip")
slim_if = slim && steps[slim]["if"].to_s.sub(/\A\s*\$\{\{\s*/, "").sub(/\s*\}\}\s*\z/, "").strip
check.(slim_if == "!cancelled() && steps.wasm-check.outcome == 'success'",
       "slim step if is exactly !cancelled() && steps.wasm-check.outcome == 'success'",
       "slim step if: !cancelled() && steps.wasm-check.outcome == 'success'")
check.(!job.key?("env"), "no job-level env", "no env: on the wasm-roundtrip job")
md = (jobs["metadata"] || {})["steps"] || []
check.(md.any? { |s| s["run"].to_s.include?("scripts/wasm-ci-check.sh --self-test") } &&
         md.any? { |s| s["run"].to_s.include?("scripts/wasm-ci-check.sh workflow") },
       "metadata job runs the self-test and the shape check",
       "metadata steps running 'scripts/wasm-ci-check.sh --self-test' and '... workflow'")
puts "wasm-ci-check workflow: #{total} assertions, #{failed} failed"
exit(failed.zero? ? 0 : 1)
RUBY
}

self_test() {
  local rows=0 wrong_v=0 wrong_m=0
  # Global, not local: the EXIT trap runs after this function's locals are gone.
  dir=$(mktemp -d)
  trap 'rm -rf "$dir"' EXIT

  # row <name> <want ok|fail> <message fragment> -- <args to this script...>
  row() {
    local name="$1" want="$2" frag="$3" out rc=0 verdict
    shift 4
    out=$("$SELF" "$@" 2>&1) || rc=$?
    verdict=ok; [ "$rc" -eq 0 ] || verdict=fail
    rows=$((rows + 1))
    if [ "$verdict" != "$want" ]; then
      wrong_v=$((wrong_v + 1)); echo "FAIL $name: verdict $verdict, expected $want ($out)"
    elif ! printf '%s' "$out" | grep -qF -- "$frag"; then
      wrong_m=$((wrong_m + 1)); echo "FAIL $name: message lacks '$frag' (got: $out)"
    else
      echo "PASS $name"
    fi
  }

  local l49='test result: ok. 49 passed; 0 failed; 0 ignored; 0 filtered out; finished in 3.8s'
  printf '%s\n' "$l49" > "$dir/l49"
  printf '%s\n' "${l49/49 passed/1 passed}" > "$dir/l1"
  printf '%s\n' "${l49/49 passed/48 passed}" > "$dir/l48"
  printf '%s\n' "${l49/49 passed/47 passed}" > "$dir/l47"
  printf 'running 0 tests\n' > "$dir/none"
  printf '%s\n%s\n' "$l49" "$l49" > "$dir/two"
  printf 'test result: FAILED. 48 passed; 1 failed; 0 ignored; 0 filtered out; finished in 3.8s\n' > "$dir/failed"
  printf '[[package]]\nname = "wasm-bindgen"\nversion = "0.2.126"\n' > "$dir/lock1"
  printf '[[package]]\nname = "other"\nversion = "1.0.0"\n' > "$dir/lock0"
  printf '[[package]]\nname = "wasm-bindgen"\nversion = "0.2.126"\n\n[[package]]\nname = "wasm-bindgen"\nversion = "0.2.125"\n' > "$dir/lock2"
  printf '[[package]]\nname = "wasm-bindgen"\nversion = "0.2.126"\n\n[[package]]\nname = "wasm-bindgen-shared"\nversion = "0.2.122"\n' > "$dir/lock_shared"

  row "1 count full 49" ok "full set: 49 passed, as expected" -- count full "$dir/l49"
  row "2 count full 1" fail "full set: 1 passed, expected 49" -- count full "$dir/l1"
  row "3 count full 48" fail "expected 49" -- count full "$dir/l48"
  row "4 count slim 49" fail "slim set: 49 passed, expected 47" -- count slim "$dir/l49"
  row "5 count slim 47" ok "slim set: 47 passed, as expected" -- count slim "$dir/l47"
  row "6 count no result line" fail "no 'test result:' line" -- count full "$dir/none"
  row "7 count two result lines" fail "2 'test result:' lines, expected 1" -- count full "$dir/two"
  row "8 count FAILED line" fail "did not pass" -- count full "$dir/failed"
  row "9 count unknown set" fail "unknown set 'fat'" -- count fat "$dir/l49"
  row "10 lock one block" ok "0.2.126" -- lock-version "$dir/lock1"
  row "11 lock no block" fail "found 0 wasm-bindgen entries" -- lock-version "$dir/lock0"
  row "12 lock two blocks" fail "found 2 wasm-bindgen entries" -- lock-version "$dir/lock2"
  row "13 lock with -shared" ok "0.2.126" -- lock-version "$dir/lock_shared"
  row "14 runner matches" ok "matches Cargo.lock" -- compare-runner "wasm-bindgen-test-runner 0.2.126" 0.2.126
  row "15 runner older" fail "is 0.2.125 but Cargo.lock pins wasm-bindgen 0.2.126" -- compare-runner "wasm-bindgen-test-runner 0.2.125" 0.2.126
  row "16 runner prefix" fail "is 0.2.1260 but" -- compare-runner "wasm-bindgen-test-runner 0.2.1260" 0.2.126
  row "17 runner empty" fail "printed no version" -- compare-runner "" 0.2.126

  # Errors go to stderr. `row` merges the streams, so check this one on its own: a failing
  # lock-version must print nothing on stdout, or `version=$(...)` in CI swallows the error.
  local so
  so=$("$SELF" lock-version "$dir/lock0" 2>/dev/null) || true
  rows=$((rows + 1))
  if [ -n "$so" ]; then
    wrong_v=$((wrong_v + 1)); echo "FAIL 18 errors go to stderr: stdout was '$so'"
  else
    echo "PASS 18 errors go to stderr"
  fi

  echo "wasm-ci-check self-test: $rows rows, $((rows - wrong_v - wrong_m)) passed, $wrong_v wrong verdict, $wrong_m wrong message"
  [ $((wrong_v + wrong_m)) -eq 0 ]
}

case "${1:-}" in
  lock-version) shift; lock_version "$@" ;;
  runner-version) shift; runner_version "$@" ;;
  # Self-test entry: the compare step on its own, driven with strings.
  compare-runner) shift; compare_runner "$@" ;;
  count) shift; count_tests "$@" ;;
  workflow) shift; workflow_shape "$@" ;;
  --self-test) self_test ;;
  *) echo "usage: $0 lock-version|runner-version|count <full|slim> <log>|workflow|--self-test" >&2; exit 2 ;;
esac
