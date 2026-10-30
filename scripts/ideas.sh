#!/usr/bin/env bash
# scripts/ideas.sh — list guidance/ideas.yaml, grouped by status.
#
# Usage: ideas.sh [STATUS]
#   STATUS  one of idea | proposed | homed | deferred | declined; limits the list.
#
# Read-only. Parses only the flat keys this file's header defines, so it needs
# no YAML library; `just validate` is what checks the file parses.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
FILE="${REPO_ROOT}/guidance/ideas.yaml"
WANT="${1:-}"

case "$WANT" in
    ""|idea|proposed|homed|deferred|declined) ;;
    -h|--help) sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "Unknown status: $WANT (idea|proposed|homed|deferred|declined)" >&2; exit 2 ;;
esac

[ -f "$FILE" ] || { echo "Missing: $FILE" >&2; exit 1; }

# One record per `  - id:` line. Strip surrounding quotes from values.
awk -v want="$WANT" '
    function val(line) { sub(/^[^:]*:[ ]*/, "", line); gsub(/^"|"$/, "", line); return line }
    function flush() {
        if (id == "") return
        if (want == "" || want == status) {
            n[status]++
            row[status, n[status]] = sprintf("  %-34s %s", id, title)
            where = (home != "" && home != "null") ? home : likely
            if (where != "" && where != "null") row[status, n[status]] = row[status, n[status]] "\n      → " where
            if (gate != "" && gate != "null") row[status, n[status]] = row[status, n[status]] "\n      gate: " gate
        }
        id = title = status = home = likely = gate = ""
    }
    /^  - id:/          { flush(); id = val($0); next }
    /^    title:/       { title = val($0); next }
    /^    status:/      { status = val($0); next }
    /^    home:/        { home = val($0); next }
    /^    likely_home:/ { likely = val($0); next }
    /^    gate:/        { gate = val($0); next }
    END {
        flush()
        split("proposed idea deferred homed declined", order, " ")
        total = 0
        for (i = 1; i <= 5; i++) {
            s = order[i]
            if (!(s in n)) continue
            printf "%s (%d)\n", s, n[s]
            for (j = 1; j <= n[s]; j++) print row[s, j]
            print ""
            total += n[s]
        }
        printf "%d ideas", total
        if (want != "") printf " with status %s", want
        print "  (guidance/ideas.yaml; promote one by filing its - [ ] in a stage first)"
    }
' "$FILE"
