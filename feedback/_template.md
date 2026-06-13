# Feedback entry template

Capture downstream user feedback with structured front-matter so it's
searchable and status can be tracked over time. Copy this file and
rename to `YYYY-MM-DD-<short-slug>.md`.

---
source: "name of downstream project / user"
captured_at: YYYY-MM-DD
captured_by: human          # human | claude | <agent>
status: open                # open | addressed | deferred
---

# One-line title

## The issue

What the user reported, in their words where possible.

## Context

Where they were in their workflow when this came up.

## Priority (their assessment)

<priority and reasoning>

## Resolution

Filled in when status changes from open. Link to the fix commit, the
spec that addressed it, or the reason for deferral.
