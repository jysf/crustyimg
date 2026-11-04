#!/usr/bin/env bash
# Structural checks on the release pipeline (DEC-111). Run from the repo root:
#
#   scripts/check-release-pipeline.sh
#
#   C1  release.yml is exactly what `dist generate --mode=ci` produces
#   C2  `dist plan`'s runner map is the expected four, and pr-run-mode is "plan"
#   C3  release.yml's jobs and their `needs` graph
#   C4  each custom publish job's uses / permissions / secrets / with / if
#   C5  announce waits on both custom publish jobs
#   C6  no publish workflow has a push trigger; only the two publish files publish
#   C7  each publish workflow's guard -> publish / dry-run wiring and permissions
#
# The runner lives in `dist plan`'s output, not in release.yml, so C2 runs dist.
# Every check runs even after a failure. Exit 1 if any check failed, 2 on a missing
# tool or the wrong dist version.
# jq filters use $vars that jq, not the shell, expands.
# shellcheck disable=SC2016
set -euo pipefail

WF=.github/workflows
RELEASE="$WF/release.yml"

for tool in dist ruby jq; do
  if ! command -v "$tool" > /dev/null 2>&1; then
    echo "check-release-pipeline: $tool is not installed" >&2
    exit 2
  fi
done
dist_version=$(dist --version 2>&1) || dist_version="(dist --version failed)"
if [ "$dist_version" != "cargo-dist 0.32.0" ]; then
  echo "check-release-pipeline: need cargo-dist 0.32.0 (DEC-040), found '$dist_version'" >&2
  exit 2
fi

checks=0
failed=0
cur_bad=0

begin() { checks=$((checks + 1)); cur_bad=0; }
bad() { echo "FAIL $1: $2"; cur_bad=1; }
end() {
  if [ "$cur_bad" -eq 0 ]; then
    echo "ok $1: $2"
  else
    failed=$((failed + 1))
  fi
}

# YAML -> JSON. YAML 1.1 loads the bare key `on` as boolean true; put it back.
yaml_json() {
  ruby -ryaml -rjson -e '
    d = YAML.load_file(ARGV[0])
    d["on"] = d.delete(true) if d.is_a?(Hash) && d.key?(true)
    puts JSON.dump(d)' "$1" 2> /dev/null
}

# q <json> <jq filter> [jq args...]: one jq query, "<jq error>" if it fails.
q() {
  local json="$1" filter="$2" out
  shift 2
  out=$(printf '%s' "$json" | jq -r "$@" "$filter" 2> /dev/null) || out="<jq error>"
  printf '%s' "$out"
}

# --- C1 -------------------------------------------------------------------
begin
c1_out=$(dist generate --mode=ci --check 2>&1) && c1_rc=0 || c1_rc=$?
if [ "$c1_rc" -ne 0 ]; then
  c1_line=$(printf '%s\n' "$c1_out" | grep -m1 -iE 'out of date|error' || true)
  bad C1 "dist generate --mode=ci --check exit $c1_rc: ${c1_line:-$(printf '%s' "$c1_out" | tail -n 1)}"
fi
end C1 "release.yml matches dist generate --mode=ci"

# --- C2 -------------------------------------------------------------------
begin
plan_json=$(dist plan --output-format=json 2> /dev/null) && plan_rc=0 || plan_rc=$?
if [ "$plan_rc" -ne 0 ]; then
  bad C2 "dist plan exit $plan_rc, expected 0"
else
  runner_map=$(q "$plan_json" '[.ci.github.artifacts_matrix.include[] | {(.targets | join(",")): .runner}] | add' -c)
  for pair in \
    aarch64-apple-darwin=macos-15 \
    x86_64-apple-darwin=macos-15-intel \
    x86_64-unknown-linux-gnu=ubuntu-22.04 \
    x86_64-pc-windows-msvc=windows-2022; do
    target="${pair%%=*}"
    want="${pair#*=}"
    got=$(q "$runner_map" '.[$t] // "nothing"' --arg t "$target")
    if [ "$got" != "$want" ]; then
      bad C2 "$target runs on $got, expected $want"
    fi
  done
  n_targets=$(q "$runner_map" 'length')
  if [ "$n_targets" != "4" ]; then
    bad C2 "the build matrix has $n_targets targets, expected 4 ($runner_map)"
  fi
  pr_mode=$(q "$plan_json" '.ci.github.pr_run_mode // "unset"')
  if [ "$pr_mode" != "plan" ]; then
    bad C2 "pr_run_mode is $pr_mode, expected plan"
  fi
fi
end C2 "dist plan runners are the expected four; pr_run_mode is plan"

# --- C3 -------------------------------------------------------------------
release_json=$(yaml_json "$RELEASE") || release_json=""
begin
if [ -z "$release_json" ]; then
  bad C3 "$RELEASE does not parse as YAML"
else
  jobs=$(q "$release_json" '.jobs | keys | sort | join(",")')
  want_jobs="announce,build-global-artifacts,build-local-artifacts,custom-publish-crates,custom-publish-npm,host,plan,publish-homebrew-formula"
  if [ "$jobs" != "$want_jobs" ]; then
    bad C3 "release.yml jobs are [$jobs], expected [$want_jobs]"
  fi
  for spec in \
    "plan=" \
    "build-local-artifacts=plan" \
    "build-global-artifacts=plan,build-local-artifacts" \
    "host=plan,build-local-artifacts,build-global-artifacts" \
    "publish-homebrew-formula=plan,host" \
    "custom-publish-crates=plan,host" \
    "custom-publish-npm=plan,host" \
    "announce=plan,host,publish-homebrew-formula,custom-publish-crates,custom-publish-npm"; do
    job="${spec%%=*}"
    want="${spec#*=}"
    present=$(q "$release_json" '.jobs | has($j)' --arg j "$job")
    if [ "$present" != "true" ]; then
      bad C3 "job $job missing"
      continue
    fi
    # `needs` may be absent, a string, or a list; compare as a set.
    got=$(q "$release_json" '.jobs[$j].needs // [] | if type == "string" then [.] else . end | join(",")' --arg j "$job")
    got_sorted=$(printf '%s' "$got" | tr ',' '\n' | sort | paste -sd, -)
    want_sorted=$(printf '%s' "$want" | tr ',' '\n' | sort | paste -sd, -)
    if [ "$got_sorted" != "$want_sorted" ]; then
      bad C3 "$job needs [$got], expected [$want]"
    fi
  done
fi
end C3 "release.yml jobs and needs graph"

# --- C4 -------------------------------------------------------------------
# Least privilege: crates.io still publishes with a token, so its job holds no
# id-token. npm publishes by OIDC trusted publishing and needs it.
begin
if [ -z "$release_json" ]; then
  bad C4 "$RELEASE does not parse as YAML"
else
  for spec in \
    'crates={"contents":"read"}' \
    'npm={"contents":"read","id-token":"write"}'; do
    x="${spec%%=*}"
    want_perms="${spec#*=}"
    job="custom-publish-$x"
    present=$(q "$release_json" '.jobs | has($j)' --arg j "$job")
    if [ "$present" != "true" ]; then
      bad C4 "$job missing"
      continue
    fi
    uses=$(q "$release_json" '.jobs[$j].uses // "nothing"' --arg j "$job")
    if [ "$uses" != "./.github/workflows/publish-$x.yml" ]; then
      bad C4 "$job uses $uses, expected ./.github/workflows/publish-$x.yml"
    fi
    perms=$(q "$release_json" '.jobs[$j].permissions // {}' --arg j "$job" -S -c)
    want_perms_sorted=$(printf '%s' "$want_perms" | jq -S -c .)
    if [ "$perms" != "$want_perms_sorted" ]; then
      bad C4 "$job permissions $perms, expected $want_perms_sorted"
    fi
    secrets=$(q "$release_json" '.jobs[$j].secrets // "nothing"' --arg j "$job")
    if [ "$secrets" != "inherit" ]; then
      bad C4 "$job secrets is $secrets, expected inherit"
    fi
    has_plan=$(q "$release_json" '.jobs[$j].with // {} | has("plan")' --arg j "$job")
    if [ "$has_plan" != "true" ]; then
      bad C4 "$job has no with.plan"
    fi
    cond=$(q "$release_json" '.jobs[$j].if // ""' --arg j "$job")
    if [[ "$cond" != *announcement_is_prerelease* ]]; then
      bad C4 "$job if does not mention announcement_is_prerelease: '$cond'"
    fi
  done
fi
end C4 "custom publish jobs: uses, per-job permissions, secrets, with.plan, prerelease if"

# --- C5 -------------------------------------------------------------------
begin
if [ -z "$release_json" ]; then
  bad C5 "$RELEASE does not parse as YAML"
else
  announce_if=$(q "$release_json" '.jobs.announce.if // ""')
  for x in crates npm; do
    if [[ "$announce_if" != *"needs.custom-publish-$x.result"* ]]; then
      bad C5 "announce.if does not check needs.custom-publish-$x.result"
    fi
  done
fi
end C5 "announce waits on both custom publish jobs"

# --- C6 -------------------------------------------------------------------
begin
for x in crates npm; do
  f="$WF/publish-$x.yml"
  if [ ! -f "$f" ]; then
    bad C6 "publish-$x.yml missing"
    continue
  fi
  j=$(yaml_json "$f") || j=""
  if [ -z "$j" ]; then
    bad C6 "publish-$x.yml does not parse as YAML"
    continue
  fi
  has_push=$(q "$j" '.on | if type == "object" then has("push") elif type == "array" then index("push") != null else . == "push" end')
  if [ "$has_push" != "false" ]; then
    bad C6 "publish-$x.yml has a push trigger"
  fi
done
publishers=$(grep -lE 'cargo publish|npm publish|wasm-npm-publish' "$WF"/*.yml 2> /dev/null | sort | paste -sd, - || true)
want_publishers="$WF/publish-crates.yml,$WF/publish-npm.yml"
if [ "$publishers" != "$want_publishers" ]; then
  bad C6 "workflows that publish are [$publishers], expected [$want_publishers]"
fi
end C6 "no push trigger on the publish workflows; only they publish"

# --- C7 -------------------------------------------------------------------
begin
for x in crates npm; do
  f="$WF/publish-$x.yml"
  if [ ! -f "$f" ]; then
    bad C7 "publish-$x.yml missing"
    continue
  fi
  j=$(yaml_json "$f") || j=""
  if [ -z "$j" ]; then
    bad C7 "publish-$x.yml does not parse as YAML"
    continue
  fi
  name="publish-$x.yml"

  v=$(q "$j" '.on.workflow_call.inputs // {} | has("plan")')
  [ "$v" = "true" ] || bad C7 "$name has no on.workflow_call.inputs.plan"
  v=$(q "$j" '.on | type == "object" and has("pull_request")')
  [ "$v" = "true" ] || bad C7 "$name has no on.pull_request"

  v=$(q "$j" '.permissions // {} | has("id-token")')
  [ "$v" = "false" ] || bad C7 "$name grants id-token at workflow level"

  for jobname in guard publish dry-run; do
    v=$(q "$j" '.jobs | has($k)' --arg k "$jobname")
    [ "$v" = "true" ] || bad C7 "$name has no $jobname job"
  done

  # publish: both gates, and per-registry permissions (only npm holds id-token).
  publish_if=$(q "$j" '.jobs.publish.if // ""')
  for gate in "github.event_name == 'push'" "github.ref_type == 'tag'" "needs.guard.outputs.action == 'publish'"; do
    [[ "$publish_if" == *"$gate"* ]] || bad C7 "$name publish.if lacks $gate"
  done
  publish_perms=$(q "$j" '.jobs.publish.permissions // {}' -S -c)
  if [ "$x" = npm ]; then
    want_perms='{"contents":"read","id-token":"write"}'
  else
    want_perms='{"contents":"read"}'
  fi
  [ "$publish_perms" = "$want_perms" ] || bad C7 "$name publish permissions $publish_perms, expected $want_perms"

  guard_if=$(q "$j" '.jobs.guard.if // ""')
  for gate in "github.event_name == 'push'" "github.ref_type == 'tag'"; do
    [[ "$guard_if" == *"$gate"* ]] || bad C7 "$name guard.if lacks $gate"
  done

  for jobname in guard dry-run; do
    v=$(q "$j" '.jobs[$k].permissions // {} | has("id-token")' --arg k "$jobname")
    [ "$v" = "false" ] || bad C7 "$name $jobname holds id-token"
  done

  dry_if=$(q "$j" '.jobs["dry-run"].if // ""')
  [[ "$dry_if" == *pull_request* ]] || bad C7 "$name dry-run.if lacks pull_request: '$dry_if'"
  dry_secrets=$(q "$j" '.jobs["dry-run"] | tojson | test("secrets\\.")')
  [ "$dry_secrets" = "false" ] || bad C7 "$name dry-run names a secret"

  # guard: the script step, then the step that validates its verdict.
  guard_idx=$(q "$j" '[.jobs.guard.steps // [] | to_entries[] | select((.value.run // "") | test("release-publish-guard\\.sh " + $k)) | .key] | first // "none"' --arg k "$x")
  if [ "$guard_idx" = "none" ] || [ "$guard_idx" = "<jq error>" ]; then
    bad C7 "$name guard has no release-publish-guard.sh step"
  else
    next_run=$(q "$j" '.jobs.guard.steps[$i + 1].run // ""' --argjson i "$guard_idx")
    [[ "$next_run" == *"expected publish or skip"* ]] || bad C7 "$name guard step is not followed by the verdict check"
  fi

  if [ "$x" = npm ]; then
    # The dry run's npm wrapper: it must set the dry-run flag and the dead registry
    # for `npm publish`, and come before the step that runs the recipe.
    shim_idx=$(q "$j" '[.jobs["dry-run"].steps // [] | to_entries[] | select((.value.run // "") | test("npm-shim")) | .key] | first // "none"')
    recipe_idx=$(q "$j" '[.jobs["dry-run"].steps // [] | to_entries[] | select((.value.run // "") | test("just wasm-npm-publish")) | .key] | first // "none"')
    shim_run=$(q "$j" '.jobs["dry-run"].steps // [] | map(.run // "" | select(test("npm-shim"))) | first // ""')
    [[ "$shim_run" == *"npm_config_dry_run=true"* ]] || bad C7 "$name dry-run lacks npm_config_dry_run=true in its npm wrapper"
    [[ "$shim_run" == *"npm_config_registry=http://127.0.0.1:9/"* ]] || bad C7 "$name dry-run's npm wrapper does not point publish at http://127.0.0.1:9/"
    if [ "$recipe_idx" = "none" ]; then
      bad C7 "$name dry-run does not run just wasm-npm-publish"
    elif [ "$shim_idx" = "none" ] || [ "$shim_idx" -ge "$recipe_idx" ]; then
      bad C7 "$name dry-run's npm wrapper (step $shim_idx) does not come before the recipe (step $recipe_idx)"
    fi
    for jobname in publish dry-run; do
      node=$(q "$j" '[.jobs[$k].steps // [] | .[] | .with["node-version"] // empty | tostring] | first // "unset"' --arg k "$jobname")
      [ "$node" = "24" ] || bad C7 "$name $jobname node-version is $node, expected 24"
    done
  fi
done
end C7 "publish workflows: triggers, gates, permissions, guard -> verdict check"

echo "$checks checks, $failed failed"
[ "$failed" -eq 0 ]
