#!/usr/bin/env bash
# scripts/_lib.sh — shared helpers sourced by other scripts.
# Sources are bash-only. Keep this minimal.

set -euo pipefail

REPO_ROOT="$(pwd)"

# Colors (fall back to no-op if terminal doesn't support color).
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    BOLD=$(tput bold 2>/dev/null || printf '')
    DIM=$(tput dim 2>/dev/null || printf '')
    RED=$(tput setaf 1 2>/dev/null || printf '')
    GREEN=$(tput setaf 2 2>/dev/null || printf '')
    YELLOW=$(tput setaf 3 2>/dev/null || printf '')
    BLUE=$(tput setaf 4 2>/dev/null || printf '')
    RESET=$(tput sgr0 2>/dev/null || printf '')
else
    BOLD=''; DIM=''; RED=''; GREEN=''; YELLOW=''; BLUE=''; RESET=''
fi

die() {
    echo "${RED}ERROR:${RESET} $*" >&2
    exit 1
}

info() {
    echo "${BLUE}•${RESET} $*"
}

success() {
    echo "${GREEN}✓${RESET} $*"
}

warn() {
    echo "${YELLOW}⚠${RESET} $*"
}

# Require that the repo has been initialized (AGENTS.md at root).
require_initialized() {
    if [ ! -f "${REPO_ROOT}/AGENTS.md" ]; then
        die "Repo not initialized. Run 'just init' first."
    fi
}

# Get the active variant (claude-only or claude-plus-agents).
get_variant() {
    if [ -f "${REPO_ROOT}/.variant" ]; then
        cat "${REPO_ROOT}/.variant"
    else
        # Fallback: detect based on whether any project has a handoffs/ folder.
        if find "${REPO_ROOT}/projects" -maxdepth 2 -type d -name handoffs 2>/dev/null | grep -q .; then
            echo "claude-plus-agents"
        else
            echo "claude-only"
        fi
    fi
}

# Read the `status:` field nested under `project:` in a project brief's
# front-matter (e.g. "active", "shipped"). Empty if absent/unreadable.
project_status() {
    awk '
        /^---$/ { f = !f; next }
        f && /^project:/ { inproj = 1; next }
        f && inproj && /^[a-zA-Z_]+:/ { inproj = 0 }
        f && inproj && /^[[:space:]]+status:/ { print $2; exit }
    ' "$1" 2>/dev/null || true
}

# Find the active project directory. Honors the brief's `status:` field: the
# active project is the HIGHEST-numbered project whose status is `active` (the
# current wave). If none are active, falls back to the highest-numbered
# non-example project (the latest wave), then the example.
#
# (Previously this ignored `status:` and returned the alphabetically FIRST
# project, which wrongly pinned "active" to PROJ-001 forever — even after later
# waves became active. Set the ACTIVE_PROJECT env var to override.)
get_active_project() {
    if [ -n "${ACTIVE_PROJECT:-}" ]; then
        echo "${ACTIVE_PROJECT}"
        return
    fi

    local latest="" active=""
    while IFS= read -r p; do
        [ -n "$p" ] || continue
        [ -z "$latest" ] && latest="$p"
        if [ -z "$active" ] && [ -f "$p/brief.md" ] \
           && [ "$(project_status "$p/brief.md")" = "active" ]; then
            active="$p"
        fi
    done < <(find "${REPO_ROOT}/projects" -maxdepth 1 -type d -name "PROJ-*" 2>/dev/null \
             | grep -v "example" | sort -r)

    local chosen="${active:-$latest}"
    if [ -z "$chosen" ]; then
        # No non-example project — fall back to the example if present.
        chosen=$(find "${REPO_ROOT}/projects" -maxdepth 1 -type d -name "PROJ-*" 2>/dev/null \
                 | sort | head -n1)
    fi
    if [ -z "$chosen" ]; then
        die "No projects found in ./projects/. Create one by copying projects/_templates/project-brief.md into projects/PROJ-NNN-<slug>/brief.md (see GETTING_STARTED.md)."
    fi
    basename "$chosen"
}

# Return the next ID for a given prefix (SPEC, STAGE, PROJ, DEC, HANDOFF)
# across the entire repo (or within a project, for SPEC/STAGE/HANDOFF).
# Usage: next_id SPEC ./projects/PROJ-001-foo
next_id() {
    local prefix="$1"
    local search_dir="${2:-$REPO_ROOT}"
    local max
    max=$(find "$search_dir" -type f -name "${prefix}-*.md" 2>/dev/null \
          | sed -E "s|.*/${prefix}-([0-9]+).*|\\1|" \
          | sort -n \
          | tail -n1 || true)
    if [ -z "$max" ]; then
        printf "%s-%03d" "$prefix" 1
    else
        # Strip leading zeros for arithmetic, then reformat.
        max=$((10#$max))
        printf "%s-%03d" "$prefix" $((max + 1))
    fi
}

# Slugify a string. "Foo Bar Baz" -> "foo-bar-baz"
slugify() {
    echo "$1" \
        | tr '[:upper:]' '[:lower:]' \
        | sed -E 's/[^a-z0-9]+/-/g' \
        | sed -E 's/^-+|-+$//g'
}

# Escape a string for safe use as the REPLACEMENT half of a
# `sed "s|...|REPLACEMENT|"` command. Escapes backslash, the `|`
# delimiter, and `&` (sed's whole-match reference). Pure bash
# parameter expansion — no sed-escaping-sed, portable to bash 3.2.
#
# Always run user-controlled text (e.g. a spec/stage title) through
# this before substituting it into a template. Without it, a title
# containing `|` could close the s-command early and a trailing `e`
# would reach GNU sed's execute flag — i.e. command injection. The
# escaped output still renders to the original characters in the file.
sed_escape_replacement() {
    local s="$1"
    s=${s//\\/\\\\}   # backslash first, so we don't double-escape below
    s=${s//|/\\|}     # the s-command delimiter
    s=${s//&/\\&}     # sed's "insert whole match" reference
    printf '%s' "$s"
}

# Find a spec file by ID. Searches all projects. Only returns active
# specs — archived specs under specs/done/ are excluded so callers
# like advance-cycle and archive-spec don't silently operate on an
# already-shipped file. Also excludes `*-timeline.md` so the v5.3
# timeline artifact (which shares the SPEC-NNN-* prefix) doesn't
# masquerade as the spec.
# Uses find's -not -path rather than a grep pipeline: grep returns 1
# on no matches, which trips pipefail and would make this function
# silently abort the caller under `set -e`.
# Usage: find_spec SPEC-001
find_spec() {
    local spec_id="$1"
    find "${REPO_ROOT}/projects" -type f -name "${spec_id}-*.md" \
        -not -name '*-timeline.md' \
        -not -path '*/done/*' 2>/dev/null | head -n1
}

# Find the timeline file paired with a spec. Returns empty if none.
# Usage: find_spec_timeline SPEC-001
find_spec_timeline() {
    local spec_id="$1"
    find "${REPO_ROOT}/projects" -type f -name "${spec_id}-*-timeline.md" \
        -not -path '*/done/*' 2>/dev/null | head -n1
}

# Find a stage file by ID.
find_stage() {
    local stage_id="$1"
    find "${REPO_ROOT}/projects" -type f -name "${stage_id}-*.md" 2>/dev/null | head -n1
}

# Find the "active" stage file for a given project. Heuristic: first
# stage with status: active, falling back to the lexically-first
# stage. Used by backlog and report_daily so they agree on what "the
# active stage" means.
# Usage: get_active_stage_file projects/PROJ-001-foo
get_active_stage_file() {
    local project_dir="$1"
    local stages_dir="${project_dir}/stages"
    [ -d "$stages_dir" ] || return
    local s status
    for s in "${stages_dir}"/STAGE-*.md; do
        [ -f "$s" ] || continue
        status=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+status:/{print $2; exit}' "$s" 2>/dev/null || echo "")
        if [ "$status" = "active" ]; then echo "$s"; return; fi
    done
    for s in "${stages_dir}"/STAGE-*.md; do
        [ -f "$s" ] || continue
        echo "$s"; return
    done
}

# Read a stage file's status: field. Empty string if missing.
get_stage_status() {
    local file="$1"
    [ -f "$file" ] || return
    awk '/^---$/{f=!f; next} f && /^[[:space:]]+status:/{print $2; exit}' "$file" 2>/dev/null || echo ""
}

# Read a stage file's target_complete: field. Empty if null/missing.
get_stage_target() {
    local file="$1"
    [ -f "$file" ] || return
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^[[:space:]]+target_complete:/ {
            v = $2
            if (v != "null" && v != "") print v
            exit
        }
    ' "$file"
}

# Read a stage file's top-level created_at field. Used as a proxy
# for "started_on" in the roadmap.
get_stage_created_at() {
    local file="$1"
    [ -f "$file" ] || return
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^created_at:/ {
            v = $2
            if (v != "null" && v != "") print v
            exit
        }
    ' "$file"
}

# Read a stage file's top-level shipped_at field. Empty if null.
get_stage_shipped_at() {
    local file="$1"
    [ -f "$file" ] || return
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^shipped_at:/ {
            v = $2
            if (v != "null" && v != "") print v
            exit
        }
    ' "$file"
}

# Today's date in YYYY-MM-DD format.
today() {
    date +%Y-%m-%d
}

# Read the repo's ID from .repo-context.yaml (metadata.repo.id).
# Used by scaffold scripts to substitute the __REPO_ID__ placeholder
# in templates. Falls back to "my-app" if the file or key is missing,
# which matches the template default and avoids breaking scaffolding
# on a freshly-cloned repo where the user hasn't replaced values yet.
get_repo_id() {
    local ctx="${REPO_ROOT}/.repo-context.yaml"
    if [ ! -f "$ctx" ]; then
        echo "my-app"
        return
    fi
    local id
    id=$(awk '
        /^metadata:/ { in_meta = 1; next }
        /^[a-zA-Z]/ && in_meta { in_meta = 0 }
        in_meta && /^  repo:/ { in_repo = 1; next }
        in_meta && in_repo && /^  [a-zA-Z]/ { in_repo = 0 }
        in_meta && in_repo && /^    id:/ { print $2; exit }
    ' "$ctx")
    echo "${id:-my-app}"
}

# ---------------------------------------------------------------------
# Report helpers — parse value and cost metadata from front-matter
# and do portable date math. Keep pure bash + awk + date; no yq.
# ---------------------------------------------------------------------

# Find all specs under a project (active AND archived under done/).
# find_spec excludes done/ on purpose; reports need both.
# Usage: find_all_specs projects/PROJ-001-foo
find_all_specs() {
    local project_dir="$1"
    find "${project_dir}/specs" -type f -name "SPEC-*.md" 2>/dev/null
}

# Extract a spec's cycle from front-matter.
# Usage: get_spec_cycle path/to/spec.md
get_spec_cycle() {
    local file="$1"
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^[[:space:]]+cycle:/ { print $2; exit }
    ' "$file"
}

# Sum tokens (input + output) across cost.sessions[] entries.
# Null fields are skipped; prints an integer (0 if empty/missing).
# Session-scalar fields live at 6-space indent; totals (which also
# has tokens_total) lives at 4-space indent, so the indent match
# disambiguates.
sum_cost_tokens_for_spec() {
    local file="$1"
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { in_cost = 1; next }
        in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
        in_cost && /^  sessions:/ { in_sessions = 1; next }
        in_cost && in_sessions && /^  [a-zA-Z_]/ { in_sessions = 0 }
        # Schema is a single combined tokens_total per session (the harness
        # reports one number). Legacy tokens_input/_output are still summed
        # if a session happens to use them (forward-compat).
        in_sessions && /^      tokens_total:/  { v = $2; if (v ~ /^[0-9]+$/) total += v }
        in_sessions && /^      tokens_input:/  { v = $2; if (v ~ /^[0-9]+$/) total += v }
        in_sessions && /^      tokens_output:/ { v = $2; if (v ~ /^[0-9]+$/) total += v }
        END { print total+0 }
    ' "$file"
}

# Sum estimated_usd across cost.sessions[] entries. Null skipped.
# Prints a float with 2 decimal places.
sum_cost_usd_for_spec() {
    local file="$1"
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { in_cost = 1; next }
        in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
        in_cost && /^  sessions:/ { in_sessions = 1; next }
        in_cost && in_sessions && /^  [a-zA-Z_]/ { in_sessions = 0 }
        in_sessions && /^      estimated_usd:/ {
            v = $2; if (v ~ /^[0-9]+(\.[0-9]+)?$/) total += v
        }
        END { printf "%.2f\n", total+0 }
    ' "$file"
}

# Count cost sessions whose recorded_at matches a given date.
# Usage: sessions_recorded_on path/to/spec.md 2026-04-21
sessions_recorded_on() {
    local file="$1"
    local date="$2"
    awk -v d="$date" '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { in_cost = 1; next }
        in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
        in_cost && /^  sessions:/ { in_sessions = 1; next }
        in_cost && in_sessions && /^  [a-zA-Z_]/ { in_sessions = 0 }
        in_sessions && /^      recorded_at:/ {
            if ($2 == d) count++
        }
        END { print count+0 }
    ' "$file"
}

# Count cost sessions total (regardless of date). Null-safe.
count_cost_sessions() {
    local file="$1"
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { in_cost = 1; next }
        in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
        in_cost && /^  sessions:/ { in_sessions = 1; next }
        in_cost && in_sessions && /^  [a-zA-Z_]/ { in_sessions = 0 }
        in_sessions && /^    - cycle:/ { count++ }
        END { print count+0 }
    ' "$file"
}

# --- Cost-capture audit helpers (AGENTS.md §4; docs/cost-tracking.md) ----
#
# Specs that predate the cost-capture process; real per-cycle token counts
# are unrecoverable, so the audit skips them. PROJECT-SPECIFIC — in a fresh
# template instance this list is EMPTY (override via the env var if needed).
COST_AUDIT_GRANDFATHERED="${COST_AUDIT_GRANDFATHERED:-SPEC-001 SPEC-002 SPEC-003 SPEC-004 SPEC-005 SPEC-006 SPEC-007 SPEC-008 SPEC-009 SPEC-010 SPEC-011 SPEC-012 SPEC-013}"

# True if a spec is grandfathered out of the cost audit. Accepts either a
# bare id ("SPEC-014") or a full file stem ("SPEC-014-some-slug") and matches
# on the SPEC-NNN id prefix.
is_grandfathered_cost() {
    local rest="${1#SPEC-}"
    local id="SPEC-${rest%%-*}"
    case " ${COST_AUDIT_GRANDFATHERED} " in
        *" $id "*) return 0 ;;
        *) return 1 ;;
    esac
}

# Print the tokens_total recorded for one cycle's cost session.
# Empty if that cycle is absent or its tokens_total is null/missing.
cycle_tokens_total() {
    local file="$1" want="$2"
    awk -v want="$want" '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { in_cost = 1; next }
        in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
        in_cost && /^  sessions:/ { in_s = 1; next }
        in_cost && in_s && /^  [a-zA-Z_]/ { in_s = 0 }
        in_s && /^    - cycle:/ { cur = $3 }
        in_s && cur == want && /^      tokens_total:/ { print $2; exit }
    ' "$file"
}

# Echo the build/verify cycles of a spec that lack a positive tokens_total.
# Empty output = cost fully recorded. (design/ship are orchestrator main-loop
# cycles and may legitimately be null — they are not checked.)
spec_missing_cost_cycles() {
    local file="$1" out="" c t
    for c in build verify; do
        t=$(cycle_tokens_total "$file" "$c")
        case "$t" in
            ''|null|0) out="${out} ${c}" ;;
        esac
    done
    echo "${out# }"
}

# Extract value_link from a spec's front-matter. Empty string if null
# or missing.
extract_value_link() {
    local file="$1"
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^value_link:/ {
            v = $0
            sub(/^value_link:[[:space:]]*/, "", v)
            # Strip surrounding quotes if present
            sub(/^"/, "", v); sub(/"$/, "", v)
            sub(/^'\''/, "", v); sub(/'\''$/, "", v)
            if (v != "null" && v != "") print v
            exit
        }
    ' "$file"
}

# Extract value.thesis from a project brief. Empty string if null or
# missing. Usage: get_project_thesis projects/PROJ-001-foo
get_project_thesis() {
    local dir="$1"
    local brief="${dir}/brief.md"
    [ -f "$brief" ] || return
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^value:/ { in_val = 1; next }
        in_val && /^[a-zA-Z_]/ { in_val = 0 }
        in_val && /^  thesis:/ {
            v = $0
            sub(/^  thesis:[[:space:]]*/, "", v)
            sub(/^"/, "", v); sub(/"$/, "", v)
            sub(/^'\''/, "", v); sub(/'\''$/, "", v)
            if (v != "null" && v != "") print v
            exit
        }
    ' "$brief"
}

# Extract value_contribution.advances from a stage file. Empty if null
# or missing. Usage: get_stage_value_contribution path/to/STAGE-001.md
get_stage_value_contribution() {
    local file="$1"
    [ -f "$file" ] || return
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^value_contribution:/ { in_vc = 1; next }
        in_vc && /^[a-zA-Z_]/ { in_vc = 0 }
        in_vc && /^  advances:/ {
            v = $0
            sub(/^  advances:[[:space:]]*/, "", v)
            sub(/^"/, "", v); sub(/"$/, "", v)
            sub(/^'\''/, "", v); sub(/'\''$/, "", v)
            if (v != "null" && v != "") print v
            exit
        }
    ' "$file"
}

# Portable date math: print the date N days ago in YYYY-MM-DD.
# macOS uses BSD date (-v), Linux uses GNU date (-d).
days_ago() {
    local n="$1"
    if [ "$(uname)" = "Darwin" ]; then
        date -v -"${n}"d +%Y-%m-%d
    else
        date -d "${n} days ago" +%Y-%m-%d
    fi
}

# Print the ISO 8601 week identifier (YYYY-WNN) for a given date.
# Uses %G-W%V so year rollover at the week boundary is handled
# correctly. Usage: iso_week_number 2026-04-21  →  2026-W17
iso_week_number() {
    local d="$1"
    if [ "$(uname)" = "Darwin" ]; then
        date -j -f "%Y-%m-%d" "$d" +"%G-W%V"
    else
        date -d "$d" +"%G-W%V"
    fi
}

# Print the Monday (start) and Sunday (end) of the ISO week
# containing the given date. Two lines: start, end.
iso_week_bounds() {
    local d="$1"
    if [ "$(uname)" = "Darwin" ]; then
        # BSD date: find weekday (1=Mon..7=Sun), compute offsets.
        local dow
        dow=$(date -j -f "%Y-%m-%d" "$d" +"%u")
        local back=$((dow - 1))
        local forward=$((7 - dow))
        date -j -v -"${back}"d -f "%Y-%m-%d" "$d" +%Y-%m-%d
        date -j -v +"${forward}"d -f "%Y-%m-%d" "$d" +%Y-%m-%d
    else
        local dow
        dow=$(date -d "$d" +"%u")
        local back=$((dow - 1))
        local forward=$((7 - dow))
        date -d "$d - ${back} days" +%Y-%m-%d
        date -d "$d + ${forward} days" +%Y-%m-%d
    fi
}

# Spec mtime as YYYY-MM-DD (portable).
spec_mtime_date() {
    local file="$1"
    if [ "$(uname)" = "Darwin" ]; then
        date -r "$(stat -f %m "$file")" +%Y-%m-%d
    else
        date -d "@$(stat -c %Y "$file")" +%Y-%m-%d
    fi
}

# Update a YAML front-matter scalar in a markdown file.
# Usage: update_frontmatter_scalar path/to/file.md task.cycle verify
# This is a deliberately simple awk-based updater for flat YAML. Requires
# the key to already exist in the front-matter. Preserves inline
# comments (everything from the first '#' onward). Assumes the scalar
# value itself contains no '#' — true for our front-matter (barewords
# like `design`, `active`, etc).
update_frontmatter_scalar() {
    local file="$1"
    local key="$2"        # e.g. task.cycle or handoff.status
    local value="$3"

    # Split the key into top-level and leaf
    local top="${key%%.*}"
    local leaf="${key##*.}"

    # awk script that walks the front-matter (between the first two ---
    # delimiters) and replaces the target key's value while preserving
    # any trailing "# comment".
    awk -v top="$top" -v leaf="$leaf" -v val="$value" '
        BEGIN { in_fm = 0; fm_seen = 0; in_top = 0 }
        /^---$/ {
            if (!fm_seen) { in_fm = 1; fm_seen = 1 }
            else if (in_fm) { in_fm = 0 }
            print; next
        }
        in_fm {
            if ($0 ~ "^" top ":") { in_top = 1; print; next }
            if ($0 ~ "^[a-zA-Z_]+:") { in_top = 0 }
            if (in_top && $0 ~ "^[[:space:]]+" leaf ":") {
                colon_idx = index($0, ":")
                prefix = substr($0, 1, colon_idx)
                tail = substr($0, colon_idx + 1)
                hash_idx = index(tail, "#")
                if (hash_idx > 0) {
                    $0 = prefix " " val "  " substr(tail, hash_idx)
                } else {
                    $0 = prefix " " val
                }
            }
        }
        { print }
    ' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
}
