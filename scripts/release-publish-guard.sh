#!/usr/bin/env bash
# Decide whether a registry publish job should publish, skip, or refuse (DEC-111).
#
#   scripts/release-publish-guard.sh npm    v0.8.3
#   scripts/release-publish-guard.sh crates v0.8.3
#   scripts/release-publish-guard.sh --self-test
#
# Checks, in this order, so a refusal costs no network call:
#   1. the tag is vMAJOR.MINOR.PATCH                       -> else exit 1
#   2. the tag has no prerelease suffix                    -> else exit 1
#   3. the tag's version equals Cargo.toml's version       -> else exit 1
#   4. the tracked working tree is clean                   -> else exit 1
#   5. ask the registry whether that version exists:
#        present         -> "SKIP: ..."    action=skip     exit 0
#        absent          -> "PUBLISH: ..." action=publish  exit 0
#        could not ask   -> "COULD NOT ASK ..."            exit 3, no action
#
# "Could not ask" is never read as "absent": a curl failure, any HTTP code other
# than 200 (a 404 too: both packages exist, so a 404 means a wrong URL or a
# registry fault), and a body that cannot be parsed all exit 3 and publish nothing.
#
# The verdict goes to $GITHUB_OUTPUT as `action=publish|skip` when that variable is
# set, and to stdout either way. Exit codes: 1 refused, 2 usage, 3 could not ask.
set -euo pipefail

USAGE='usage: release-publish-guard.sh <npm|crates> <tag> | --self-test'
USER_AGENT='crustyimg-release-guard (+https://github.com/jysf/crustyimg)'

refuse() {
  echo "REFUSING: $*" >&2
  exit 1
}

could_not_ask() {
  echo "COULD NOT ASK $registry: $*. Not publishing; re-run this job when the registry answers." >&2
  exit 3
}

verdict() {
  local action="$1"
  echo "action=$action"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "action=$action" >> "$GITHUB_OUTPUT"
  fi
}

guard() {
  if [ "$#" -ne 2 ]; then
    echo "$USAGE" >&2
    exit 2
  fi
  local kind="$1" tag="$2"
  local name url
  case "$kind" in
    npm)
      name=crustyimg-wasm
      registry=npm
      url="${RELEASE_GUARD_NPM_URL:-https://registry.npmjs.org/crustyimg-wasm}"
      ;;
    crates)
      name=crustyimg
      registry=crates.io
      url="${RELEASE_GUARD_CRATES_URL:-https://index.crates.io/cr/us/crustyimg}"
      ;;
    *)
      echo "$USAGE" >&2
      exit 2
      ;;
  esac

  if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    refuse "tag '$tag' is not vMAJOR.MINOR.PATCH."
  fi
  if [[ "$tag" == *-* ]]; then
    refuse "$tag is a prerelease; DEC-111 publishes stable versions only to $registry."
  fi

  local version="${tag#v}" cargo_version
  cargo_version=$(sed -n -E 's/^version[[:space:]]*=[[:space:]]*"([^"]*)".*/\1/p' Cargo.toml | head -n 1)
  if [ "$version" != "$cargo_version" ]; then
    refuse "tag $tag says $version, Cargo.toml says ${cargo_version:-nothing}."
  fi

  if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    refuse "not inside a git checkout; publish from a clean checkout of the tag."
  fi
  if ! git diff --quiet || ! git diff --cached --quiet; then
    refuse "the working tree has uncommitted changes; publish from a clean checkout of the tag."
  fi

  local body code curl_rc jq_rc
  body=$(mktemp)
  # shellcheck disable=SC2064  # expand $body now, while it is set
  trap "rm -f '$body'" EXIT
  code=$(curl -sS --max-time 20 --retry "${RELEASE_GUARD_RETRIES:-2}" --retry-delay 2 \
    -A "$USER_AGENT" -o "$body" -w '%{http_code}' "$url" 2> "$body.err") && curl_rc=0 || curl_rc=$?
  if [ "$curl_rc" -ne 0 ]; then
    local err
    err=$(tr '\n' ' ' < "$body.err")
    rm -f "$body.err"
    could_not_ask "curl exit $curl_rc: ${err:-no message} ($url)"
  fi
  rm -f "$body.err"
  if [ "$code" != "200" ]; then
    could_not_ask "HTTP $code from $url"
  fi

  # jq -e: exit 0 = true (present), 1 = false (absent), anything else = unreadable.
  if [ "$kind" = npm ]; then
    jq -e --arg v "$version" '.versions | has($v)' "$body" > /dev/null 2>&1 && jq_rc=0 || jq_rc=$?
  else
    # The sparse index is one JSON object per line. An empty file, or a line with
    # no "vers", is not an index we can read.
    jq -s -e --arg v "$version" \
      'if length == 0 or any(.[]; has("vers") | not) then error("no versions")
       else any(.[]; .vers == $v) end' "$body" > /dev/null 2>&1 && jq_rc=0 || jq_rc=$?
  fi
  case "$jq_rc" in
    0)
      echo "SKIP: $name@$version is already on $registry; nothing to publish."
      verdict skip
      ;;
    1)
      echo "PUBLISH: $name@$version is not on $registry yet."
      verdict publish
      ;;
    *)
      could_not_ask "unreadable body from $url (jq exit $jq_rc)"
      ;;
  esac
}

self_test() {
  local self tmp repo port_file base
  self="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
  tmp=$(mktemp -d)
  repo="$tmp/repo"
  port_file="$tmp/port"
  # Global, not local: the EXIT trap runs after this function has returned.
  SELF_TEST_SERVER_PID=""
  # shellcheck disable=SC2064
  trap "[ -n \"\$SELF_TEST_SERVER_PID\" ] && kill \"\$SELF_TEST_SERVER_PID\" 2>/dev/null; rm -rf '$tmp'" EXIT

  mkdir "$repo"
  printf '[package]\nname = "fixture"\nversion = "1.2.3"\n' > "$repo/Cargo.toml"
  printf 'fixture\n' > "$repo/README.md"
  git -C "$repo" init -q
  git -C "$repo" add Cargo.toml README.md
  git -C "$repo" -c user.name=self-test -c user.email=self-test@example.invalid \
    -c commit.gpgsign=false commit -q -m fixture

  # A fake registry on 127.0.0.1, port chosen by the OS and written to $port_file.
  python3 -I -c '
import http.server, socketserver, sys
NPM_ABSENT = b"{\"versions\":{\"1.2.2\":{}}}"
NPM_PRESENT = b"{\"versions\":{\"1.2.2\":{},\"1.2.3\":{}}}"
CR_ABSENT = b"{\"name\":\"crustyimg\",\"vers\":\"1.2.1\"}\n{\"name\":\"crustyimg\",\"vers\":\"1.2.2\"}\n"
CR_PRESENT = CR_ABSENT + b"{\"name\":\"crustyimg\",\"vers\":\"1.2.3\"}\n"
ROUTES = {
    "/npm/absent": (200, NPM_ABSENT),
    "/npm/present": (200, NPM_PRESENT),
    "/npm/404": (404, b"{\"error\":\"Not found\"}"),
    "/npm/503": (503, b"unavailable"),
    "/npm/junk": (200, b"not json"),
    "/crates/absent": (200, CR_ABSENT),
    "/crates/present": (200, CR_PRESENT),
    "/crates/503": (503, b"unavailable"),
}
class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        code, body = ROUTES.get(self.path, (500, b"no such route"))
        self.send_response(code)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, *a):
        pass
srv = socketserver.TCPServer(("127.0.0.1", 0), H)
with open(sys.argv[1], "w") as f:
    f.write(str(srv.server_address[1]))
srv.serve_forever()
' "$port_file" &
  SELF_TEST_SERVER_PID=$!
  # Out of the job table, so bash prints no "Terminated" notice when the trap kills it.
  disown "$SELF_TEST_SERVER_PID"
  local i=0
  while [ ! -s "$port_file" ]; do
    i=$((i + 1))
    if [ "$i" -gt 100 ]; then
      echo "FAIL self-test: the fake registry did not start"
      exit 1
    fi
    sleep 0.1
  done
  base="http://127.0.0.1:$(cat "$port_file")"
  local dead="http://127.0.0.1:9/"

  local rows=0 failed=0 f_exit=0 f_msg=0 f_action=0
  # row <id> <want_exit> <want_message> <want_action|-> <kind> <tag> <url>
  row() {
    local id="$1" want_exit="$2" want_msg="$3" want_action="$4" kind="$5" tag="$6" url="$7"
    local out gh_out got_exit got_action bad=0
    gh_out=$(mktemp)
    out=$(cd "$repo" && GITHUB_OUTPUT="$gh_out" RELEASE_GUARD_RETRIES=0 \
      RELEASE_GUARD_NPM_URL="$url" RELEASE_GUARD_CRATES_URL="$url" \
      "$self" "$kind" "$tag" 2>&1) && got_exit=0 || got_exit=$?
    got_action=$(cat "$gh_out")
    rm -f "$gh_out"
    [ "$want_action" = "-" ] && want_action=""
    rows=$((rows + 1))
    if [ "$got_exit" != "$want_exit" ]; then
      echo "FAIL $id: exit $got_exit, expected $want_exit"
      f_exit=$((f_exit + 1)); bad=1
    fi
    if [[ "$out" != *"$want_msg"* ]]; then
      echo "FAIL $id: output lacks '$want_msg'; got: $(printf '%s' "$out" | tr '\n' ' ')"
      f_msg=$((f_msg + 1)); bad=1
    fi
    if [ "$got_action" != "$want_action" ]; then
      echo "FAIL $id: GITHUB_OUTPUT '${got_action}', expected '${want_action}'"
      f_action=$((f_action + 1)); bad=1
    fi
    if [ "$bad" -eq 0 ]; then
      echo "ok $id"
    else
      failed=$((failed + 1))
    fi
  }

  row G1 1 "is not vMAJOR.MINOR.PATCH" - npm main "$base/npm/absent"
  row G2 1 "is a prerelease" - npm v1.2.3-rc.1 "$base/npm/absent"
  row G3 1 "tag v1.2.4 says 1.2.4, Cargo.toml says 1.2.3" - npm v1.2.4 "$base/npm/absent"
  printf 'modified\n' >> "$repo/README.md"
  row G4 1 "uncommitted changes" - npm v1.2.3 "$base/npm/absent"
  git -C "$repo" checkout -q -- README.md
  row G5 0 "PUBLISH: crustyimg-wasm@1.2.3" action=publish npm v1.2.3 "$base/npm/absent"
  row G6 0 "SKIP: crustyimg-wasm@1.2.3 is already on npm" action=skip npm v1.2.3 "$base/npm/present"
  row G7 3 "COULD NOT ASK npm: HTTP 404" - npm v1.2.3 "$base/npm/404"
  row G8 3 "COULD NOT ASK npm: HTTP 503" - npm v1.2.3 "$base/npm/503"
  row G9 3 "COULD NOT ASK npm: curl exit" - npm v1.2.3 "$dead"
  row G10 3 "COULD NOT ASK npm: unreadable body" - npm v1.2.3 "$base/npm/junk"
  row G11 0 "PUBLISH: crustyimg@1.2.3" action=publish crates v1.2.3 "$base/crates/absent"
  row G12 0 "SKIP: crustyimg@1.2.3 is already on crates.io" action=skip crates v1.2.3 "$base/crates/present"
  row G13 3 "COULD NOT ASK crates.io: HTTP 503" - crates v1.2.3 "$base/crates/503"
  row G14 3 "COULD NOT ASK crates.io: curl exit" - crates v1.2.3 "$dead"

  echo "$rows rows, $failed failed (exit: $f_exit, message: $f_msg, action: $f_action)"
  [ "$failed" -eq 0 ]
}

if [ "${1:-}" = "--self-test" ]; then
  if [ "$#" -ne 1 ]; then
    echo "$USAGE" >&2
    exit 2
  fi
  self_test
  exit $?
fi

registry=""
guard "$@"
