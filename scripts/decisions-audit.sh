#!/usr/bin/env bash
# scripts/decisions-audit.sh — audit repo decisions for structural
# integrity and scope conflicts.
#
# This is a native, zero-dependency take on two ideas borrowed from
# LineSpec (https://linespec.dev) "Provenance Records":
#
#   1. Structural lint  — every DEC-* file is well-formed and its
#      supersession links are consistent (no dangling / one-sided
#      supersedes/superseded_by, no duplicate IDs).
#   2. Scope auditing   — using an OPTIONAL `affected_scope:` glob list
#      in a decision's front-matter, flag when two ACTIVE decisions
#      govern overlapping paths (review for contradiction), and, with
#      --changed, tell you which decisions govern the files you're
#      about to commit ("re-read DEC-007 before changing this").
#
# Pure bash + awk + git. No yq, no binary, no embeddings, no network.
# Targets bash 3.2 (macOS default): no mapfile, no associative arrays.
# `affected_scope` is optional: decisions without it are still linted,
# they're just skipped by the scope checks.
#
# Usage:
#   just decisions-audit                # lint + scope-overlap warnings
#   just decisions-audit --changed      # which decisions govern uncommitted changes
#   just decisions-audit --changed main # ...vs a base ref (diff base...HEAD)
#
# Exit status: 1 if structural errors are found (CI / pre-commit hook
# friendly). Scope-overlap and --changed output is advisory (exit 0).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

DECISIONS_DIR="${REPO_ROOT}/decisions"

# ---------------------------------------------------------------------
# Front-matter readers (decision-specific; kept local to this script).
# ---------------------------------------------------------------------

# insight.id — nested one level under `insight:`.
get_insight_id() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^insight:/ { i = 1; next }
        i && /^[a-zA-Z_]/ { i = 0 }
        i && /^[[:space:]]+id:/ { print $2; exit }
    ' "$1"
}

# insight.type — nested one level under `insight:`.
get_insight_type() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^insight:/ { i = 1; next }
        i && /^[a-zA-Z_]/ { i = 0 }
        i && /^[[:space:]]+type:/ { print $2; exit }
    ' "$1"
}

# A top-level scalar (supersedes, superseded_by, created_at). Prints the
# raw value; callers decide whether "null" counts.
get_top_scalar() {
    local file="$1" key="$2"
    awk -v k="$key" '
        /^---$/ { f = !f; next }
        !f { exit }
        $0 ~ "^" k ":" { print $2; exit }
    ' "$file"
}

# affected_scope — a YAML list of globs at the top level. One glob per
# line on stdout. Empty if the key is absent or `[]`.
get_affected_scope() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^affected_scope:/ { s = 1; next }
        s && /^[a-zA-Z_]/ { s = 0 }
        s && /^[[:space:]]*-[[:space:]]/ {
            v = $0
            sub(/^[[:space:]]*-[[:space:]]*/, "", v)
            sub(/[[:space:]]*#.*$/, "", v)       # strip trailing comment
            sub(/^"/, "", v); sub(/"$/, "", v)   # strip double quotes
            sub(/^'\''/, "", v); sub(/'\''$/, "", v)  # strip single quotes
            sub(/[[:space:]]+$/, "", v)
            if (v != "" && v != "null") print v
        }
    ' "$1"
}

# ---------------------------------------------------------------------
# Glob helpers.
# ---------------------------------------------------------------------

# Convert a path glob to a regex body (no anchors). `**` matches across
# `/`, `*` matches within a path segment, `?` matches one char. Uses a
# placeholder token instead of \x01 so it works with BSD sed.
glob_to_regex() {
    printf '%s' "$1" \
        | sed -e 's/\./\\./g' \
              -e 's/\*\*/@@GLOBSTAR@@/g' \
              -e 's/\*/[^\/]*/g' \
              -e 's/@@GLOBSTAR@@/.*/g' \
              -e 's/?/./g'
}

# Literal directory prefix of a glob: everything before the first
# wildcard char, trailing slash removed. Empty for wildcard-leading
# globs (e.g. `**/*.ts`).
glob_prefix() {
    printf '%s' "$1" | sed -E -e 's/[*?[].*$//' -e 's#/+$##'
}

# Heuristic: do two globs plausibly govern a common path? True when one
# literal prefix is a path-prefix of the other. Wildcard-leading globs
# (empty prefix) are skipped here to avoid flooding the overlap report;
# --changed catches those precisely.
prefixes_overlap() {
    local a b
    a=$(glob_prefix "$1"); b=$(glob_prefix "$2")
    [ -z "$a" ] && return 1
    [ -z "$b" ] && return 1
    [ "$a" = "$b" ] && return 0
    case "$a" in "$b"/*) return 0 ;; esac
    case "$b" in "$a"/*) return 0 ;; esac
    return 1
}

# Is a decision "active"? Active = not superseded by another decision.
is_active() {
    local sb
    sb=$(get_top_scalar "$1" superseded_by)
    [ -z "$sb" ] || [ "$sb" = "null" ]
}

# ---------------------------------------------------------------------
# Collect decision files (bash 3.2: no mapfile).
# ---------------------------------------------------------------------

if [ ! -d "$DECISIONS_DIR" ]; then
    die "No decisions/ directory at repo root. Nothing to audit."
fi

DEC_FILES=()
while IFS= read -r f; do
    [ -n "$f" ] && DEC_FILES+=("$f")
done < <(find "$DECISIONS_DIR" -maxdepth 1 -type f -name 'DEC-*.md' \
         -not -name '_template.md' | sort)

if [ "${#DEC_FILES[@]}" -eq 0 ]; then
    info "No DEC-* files in decisions/ yet. Nothing to audit."
    exit 0
fi

# ---------------------------------------------------------------------
# Mode: --changed [BASE] — which decisions govern your pending changes.
# ---------------------------------------------------------------------

if [ "${1:-}" = "--changed" ]; then
    base="${2:-}"
    git rev-parse --is-inside-work-tree >/dev/null 2>&1 \
        || die "--changed needs a git repository."

    CHANGED=()
    if [ -n "$base" ]; then
        while IFS= read -r p; do
            [ -n "$p" ] && CHANGED+=("$p")
        done < <(git diff --name-only "${base}...HEAD" 2>/dev/null | sort -u)
        scope_desc="changes in ${base}...HEAD"
    else
        while IFS= read -r p; do
            [ -n "$p" ] && CHANGED+=("$p")
        done < <( {
            git diff --name-only
            git diff --name-only --cached
            git ls-files --others --exclude-standard
        } 2>/dev/null | sort -u )
        scope_desc="your uncommitted changes"
    fi

    if [ "${#CHANGED[@]}" -eq 0 ]; then
        info "No changed files in scope (${scope_desc})."
        exit 0
    fi

    echo "${BOLD}Decisions governing ${scope_desc}:${RESET}"
    echo

    hits=0
    for file in "${DEC_FILES[@]}"; do
        is_active "$file" || continue
        id=$(get_insight_id "$file")
        # The markdown H1 title; anchored to `# DEC-` so it can't pick
        # up a `#`-prefixed YAML comment inside the front-matter.
        title=$(grep -m1 '^# DEC-' "$file" | sed -E 's/^# //')
        globs=()
        while IFS= read -r g; do
            [ -n "$g" ] && globs+=("$g")
        done < <(get_affected_scope "$file")
        [ "${#globs[@]}" -eq 0 ] && continue

        matched=()
        for path in "${CHANGED[@]}"; do
            for g in "${globs[@]}"; do
                re="^$(glob_to_regex "$g")$"
                if [[ "$path" =~ $re ]]; then
                    matched+=("$path")
                    break
                fi
            done
        done

        if [ "${#matched[@]}" -gt 0 ]; then
            hits=$((hits + 1))
            warn "${BOLD}${id}${RESET} — ${title}"
            echo "      ${DIM}re-read this decision before committing; your change touches:${RESET}"
            for m in "${matched[@]}"; do
                echo "        ${m}"
            done
            echo
        fi
    done

    if [ "$hits" -eq 0 ]; then
        success "No active decision's affected_scope matches these changes."
    else
        echo "${DIM}Advisory only — confirm your change is consistent with each decision above,"
        echo "or supersede the decision if it no longer holds.${RESET}"
    fi
    exit 0
fi

# ---------------------------------------------------------------------
# Default mode: structural lint + scope-overlap warnings.
# bash 3.2 has no associative arrays, so the id->file map is two
# parallel indexed arrays with a linear-scan lookup.
# ---------------------------------------------------------------------

errors=0
warnings=0

MAP_IDS=()
MAP_FILES=()

# Echo the file registered for an id, or nothing.
file_for_id() {
    local target="$1" i
    for ((i = 0; i < ${#MAP_IDS[@]}; i++)); do
        if [ "${MAP_IDS[$i]}" = "$target" ]; then
            printf '%s' "${MAP_FILES[$i]}"
            return 0
        fi
    done
    return 1
}

# Build the map and detect duplicates / filename mismatches / missing
# required fields.
for file in "${DEC_FILES[@]}"; do
    base=$(basename "$file")
    file_id=$(printf '%s' "$base" | sed -E 's/^(DEC-[0-9]+).*/\1/')
    fm_id=$(get_insight_id "$file")
    created=$(get_top_scalar "$file" created_at)
    itype=$(get_insight_type "$file")

    if [ -z "$fm_id" ]; then
        warn "${base}: missing insight.id in front-matter"
        errors=$((errors + 1))
        continue
    fi
    if [ "$fm_id" != "$file_id" ]; then
        warn "${base}: filename id (${file_id}) != front-matter insight.id (${fm_id})"
        errors=$((errors + 1))
    fi
    if existing=$(file_for_id "$fm_id"); then
        warn "${fm_id}: duplicate id — also in $(basename "$existing")"
        errors=$((errors + 1))
    fi
    MAP_IDS+=("$fm_id")
    MAP_FILES+=("$file")

    if [ -z "$created" ] || [ "$created" = "null" ]; then
        warn "${fm_id}: missing created_at"
        errors=$((errors + 1))
    fi
    if [ -z "$itype" ]; then
        warn "${fm_id}: missing insight.type"
        errors=$((errors + 1))
    fi
done

# Supersession integrity: every supersedes/superseded_by must point at a
# real DEC, not at itself, and be mirrored on the other side.
for file in "${DEC_FILES[@]}"; do
    id=$(get_insight_id "$file")
    [ -z "$id" ] && continue
    sup=$(get_top_scalar "$file" supersedes)
    supby=$(get_top_scalar "$file" superseded_by)

    if [ -n "$sup" ] && [ "$sup" != "null" ]; then
        if [ "$sup" = "$id" ]; then
            warn "${id}: supersedes itself"
            errors=$((errors + 1))
        elif ! sup_file=$(file_for_id "$sup"); then
            warn "${id}: supersedes ${sup}, which does not exist"
            errors=$((errors + 1))
        else
            other_supby=$(get_top_scalar "$sup_file" superseded_by)
            if [ "$other_supby" != "$id" ]; then
                warn "${id}: supersedes ${sup}, but ${sup}.superseded_by is '${other_supby:-null}' (expected ${id})"
                errors=$((errors + 1))
            fi
        fi
    fi

    if [ -n "$supby" ] && [ "$supby" != "null" ]; then
        if [ "$supby" = "$id" ]; then
            warn "${id}: superseded_by itself"
            errors=$((errors + 1))
        elif ! file_for_id "$supby" >/dev/null; then
            warn "${id}: superseded_by ${supby}, which does not exist"
            errors=$((errors + 1))
        fi
    fi
done

# Scope-overlap warnings between ACTIVE decisions (heuristic, advisory).
ACTIVE_FILES=()
for file in "${DEC_FILES[@]}"; do
    is_active "$file" && ACTIVE_FILES+=("$file")
done

n=${#ACTIVE_FILES[@]}
for ((x = 0; x < n; x++)); do
    fa="${ACTIVE_FILES[$x]}"
    ida=$(get_insight_id "$fa")
    globs_a=()
    while IFS= read -r g; do
        [ -n "$g" ] && globs_a+=("$g")
    done < <(get_affected_scope "$fa")
    [ "${#globs_a[@]}" -eq 0 ] && continue

    for ((y = x + 1; y < n; y++)); do
        fb="${ACTIVE_FILES[$y]}"
        idb=$(get_insight_id "$fb")
        globs_b=()
        while IFS= read -r g; do
            [ -n "$g" ] && globs_b+=("$g")
        done < <(get_affected_scope "$fb")
        [ "${#globs_b[@]}" -eq 0 ] && continue

        overlap=""
        for ga in "${globs_a[@]}"; do
            for gb in "${globs_b[@]}"; do
                if prefixes_overlap "$ga" "$gb"; then
                    overlap="${ga} ~ ${gb}"
                    break 2
                fi
            done
        done
        if [ -n "$overlap" ]; then
            warn "${ida} and ${idb} both govern overlapping scope (${overlap})"
            echo "      ${DIM}confirm they don't contradict; if one wins, mark the other superseded${RESET}"
            warnings=$((warnings + 1))
        fi
    done
done

echo
if [ "$errors" -gt 0 ]; then
    die "${errors} structural error(s), ${warnings} scope warning(s) across ${#DEC_FILES[@]} decision(s)."
fi
if [ "$warnings" -gt 0 ]; then
    info "0 structural errors, ${warnings} scope warning(s) across ${#DEC_FILES[@]} decision(s)."
else
    success "All ${#DEC_FILES[@]} decision(s) clean: structure valid, no scope conflicts."
fi
