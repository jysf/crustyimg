#!/usr/bin/env bash
# scripts/validate-frontmatter.sh — strict-parse every YAML front-matter block
# in tracked .md / .yaml / .yml files, and fail on any that a real YAML parser
# rejects.
#
# Why this exists: the repo's other bookkeeping tooling (status.sh, the reports,
# this very audit's siblings) extracts front-matter fields with grep/sed and
# NEVER strict-parses. So a syntactically-invalid block reads "fine" here while
# a real YAML consumer (an external portfolio / standup tracker, or a stricter
# future tool) silently DROPS the whole block — which once made PROJ-004's
# `status: shipped` disappear from a downstream tracker. This gate is the only
# thing that catches malformed front-matter, so a bad block fails fast in CI
# instead of going unnoticed until something external trips over it.
#
# Parser: Ruby's stdlib Psych (`yaml`) — preinstalled on GitHub-hosted runners
# and macOS/most Linux, so no pip/gem install. Requires `ruby` on PATH.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/_lib.sh
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

if ! command -v ruby >/dev/null 2>&1; then
    die "validate-frontmatter: needs \`ruby\` (stdlib yaml) on PATH. It ships with macOS and GitHub-hosted runners; install ruby to run this check locally."
fi

# Feed the NUL-delimited list of tracked markdown/YAML files to a Ruby sweep.
# The sweep prints one INVALID block per failure (file + parser error) to
# stderr and exits non-zero if any block is invalid; pipefail propagates it.
if git ls-files -z '*.md' '*.yaml' '*.yml' \
    | ruby -EUTF-8 -ryaml -rdate -e '
        invalid = 0
        checked = 0
        $stdin.read.split("\x00").reject(&:empty?).each do |f|
          begin
            txt = File.read(f, encoding: "UTF-8")
          rescue
            next
          end
          next unless txt.start_with?("---")
          parts = txt.split(/^---\s*$/, 3)
          next if parts.length < 3
          checked += 1
          begin
            YAML.safe_load(parts[1], permitted_classes: [Date, Time], aliases: true)
          rescue => e
            invalid += 1
            $stderr.puts "  INVALID  #{f}"
            $stderr.puts "           #{e.class}: #{e.message.lines.first.strip}"
          end
        end
        # Report the checked count on the last line of stderr for visibility.
        $stderr.puts "checked #{checked} front-matter block(s)"
        exit(invalid.zero? ? 0 : 1)
    '; then
    success "validate-frontmatter: all front-matter blocks parse."
else
    die "validate-frontmatter: invalid front-matter above. Quote list items that start with a backtick, and make a \`- Word: \"…\" more text\` item a folded block scalar (>-)."
fi
