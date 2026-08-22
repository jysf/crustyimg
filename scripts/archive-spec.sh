#!/usr/bin/env bash
# scripts/archive-spec.sh — move a shipped spec to done/ and update stage backlog.
# Usage: archive-spec.sh SPEC-NNN

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

SPEC_ID="${1:-}"

if [ -z "$SPEC_ID" ]; then
    die "Usage: just archive-spec SPEC-NNN"
fi

SPEC_FILE=$(find_spec "$SPEC_ID")
if [ -z "$SPEC_FILE" ]; then
    die "Spec not found: ${SPEC_ID}"
fi

# Check cycle is ship
CYCLE=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+cycle:/{print $2; exit}' "$SPEC_FILE" 2>/dev/null || echo "")
if [ "$CYCLE" != "ship" ]; then
    warn "Spec cycle is '${CYCLE}', not 'ship'. Continue anyway? [y/N]"
    read -r answer
    if [ "$answer" != "y" ] && [ "$answer" != "Y" ]; then
        echo "Aborted."
        exit 0
    fi
fi

# Did this spec's cycle field ever move? A spec reaching ship having never been
# marked `build` or `verify` means `advance-cycle` was not run for those cycles.
# Measured across SPEC-110..115: no build prompt ever asked for it, so the field
# carried no signal between design and ship. Not fatal -- the work may well have
# happened -- but silence is how it stayed invisible for six specs.
if command -v git >/dev/null 2>&1 && git -C "$REPO_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    # `-n 1` on git itself, NOT `| head -n1`: under `set -o pipefail`, head exits
    # as soon as it has its line, git takes SIGPIPE, and the whole script dies
    # with 141 once the log is long enough to still be writing. Measured here on
    # SPEC-124/125, whose spec files had accumulated enough history to trigger it
    # — and it is intermittent by nature, so it passed for every earlier spec.
    # `lifetime-report.sh:93-98` already carries this fix as `|| true`; asking git
    # for one record is better still, because `|| true` would also swallow a real
    # git failure. [[a-piped-command-reports-the-pipes-exit-code]]
    seen_build=$(git -C "$REPO_ROOT" log -n 1 --format=%H -S'cycle: build' -- "$SPEC_FILE" 2>/dev/null)
    seen_verify=$(git -C "$REPO_ROOT" log -n 1 --format=%H -S'cycle: verify' -- "$SPEC_FILE" 2>/dev/null)
    if [ -z "$seen_build" ] && [ -z "$seen_verify" ]; then
        warn "${SPEC_ID} reaches ship having never been marked 'build' or 'verify'."
        warn "  advance-cycle was likely never run for those cycles, so the cycle"
        warn "  field carried no signal while the work was in flight."
        warn "  See projects/_templates/prompts/closing-steps-snippet.md."
    fi
fi

SPEC_DIR=$(dirname "$SPEC_FILE")
DONE_DIR="${SPEC_DIR}/done"
mkdir -p "$DONE_DIR"

SPEC_BASENAME=$(basename "$SPEC_FILE")
TARGET="${DONE_DIR}/${SPEC_BASENAME}"

mv "$SPEC_FILE" "$TARGET"
success "Archived: ${SPEC_FILE} → ${TARGET}"

# Co-archive the timeline file if one exists. The timeline is an
# artifact of this spec's cycle history and belongs next to the spec
# it describes.
TIMELINE_FILE=$(find_spec_timeline "$SPEC_ID")
if [ -n "$TIMELINE_FILE" ]; then
    TIMELINE_TARGET="${DONE_DIR}/$(basename "$TIMELINE_FILE")"
    mv "$TIMELINE_FILE" "$TIMELINE_TARGET"
    success "Archived timeline: ${TIMELINE_FILE} → ${TIMELINE_TARGET}"
fi

# Try to update the parent stage's backlog.
# Get the stage ID from the spec's front-matter (project.stage field).
STAGE_ID=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+stage:/{print $2; exit}' "$TARGET" 2>/dev/null || echo "")
if [ -n "$STAGE_ID" ]; then
    STAGE_FILE=$(find_stage "$STAGE_ID")
    if [ -n "$STAGE_FILE" ]; then
        echo ""
        echo "Parent stage: ${STAGE_ID} (${STAGE_FILE})"
        echo "${DIM}Remember to update the stage's Spec Backlog section manually:"
        echo "  - Change '[ ] ${SPEC_ID}' to '[x] ${SPEC_ID} (shipped on $(today))'"
        echo "  - Update the count summary at the bottom of the backlog.${RESET}"
    fi
fi

# If this leaves no active specs under the stage, surface that as an
# observation — NOT as a claim that the stage is complete. The stage's
# `## Spec Backlog` may still list unwritten specs, and we can't
# reliably read that list (it's manually maintained markdown).
if [ -n "$STAGE_ID" ]; then
    REMAINING=$(find "$SPEC_DIR" -maxdepth 1 -name "SPEC-*.md" 2>/dev/null \
                | xargs -I{} awk -v sid="$STAGE_ID" '/^---$/{f=!f; next} f && /^[[:space:]]+stage:/ && $2 == sid {print FILENAME; exit}' {} \
                | wc -l | tr -d ' ')
    if [ "$REMAINING" = "0" ]; then
        echo ""
        # "No active specs remain" means no spec FILES outside done/ -- it says
        # nothing about the backlog, and readers took it as "the stage is done".
        # Both stages it fired on during PROJ-010 still had open, unframed
        # backlog items. Count them and say so, rather than leaving a
        # conditional the reader skims past.
        OPEN_BULLETS=0
        if [ -n "${STAGE_FILE:-}" ]; then
            OPEN_BULLETS=$(count_unpromoted_bullets "$STAGE_FILE")
        fi
        if [ "${OPEN_BULLETS:-0}" -gt 0 ]; then
            echo "${YELLOW}No active specs remain for ${STAGE_ID}, but its backlog is NOT complete:${RESET}"
            echo "  ${OPEN_BULLETS} un-promoted backlog item(s) still open. The stage cannot ship yet."
            echo "  See its ## Spec Backlog, or run: just backlog"
        else
            echo "${GREEN}No active specs remain for ${STAGE_ID}, and its backlog is clear.${RESET}"
            echo "Run the Stage Ship prompt (Prompt 1c) in FIRST_SESSION_PROMPTS.md."
        fi
    fi
fi
