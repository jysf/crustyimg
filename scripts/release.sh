#!/usr/bin/env bash
# just release X.Y.Z — mechanical release prep + guards (RELEASING.md).
#
# Automates the error-prone step that MUST match the tag: it bumps
# Cargo.toml, refreshes Cargo.lock, and VERIFIES that the version you are
# about to tag equals the crate version and that the CHANGELOG has a section
# for it. It does NOT commit, tag, or push — those stay maintainer-authorized
# (RELEASING.md steps 5–8). Prints the exact next commands.
#
# Why this exists: v0.1.1 was first tagged on an un-bumped 0.1.0 commit, so
# both the crates.io and cargo-dist jobs failed ("already exists" / "nothing to
# Release"). This recipe makes the tag-to-version mismatch impossible to miss.
set -euo pipefail

VERSION="${1:-}"
if ! printf '%s' "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$'; then
  echo "usage: just release X.Y.Z   (semver, e.g. 0.1.2)" >&2
  exit 2
fi

CUR="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
echo "current: $CUR  →  new: $VERSION"

# 1) Bump the [package] version (only the FIRST `version = "..."` line).
awk -v v="$VERSION" '!done && /^version = "/ { sub(/"[^"]*"/, "\"" v "\""); done=1 } { print }' \
  Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# 2) Refresh Cargo.lock (and sanity-compile) so `cargo publish --locked` is happy in CI.
cargo build -q

# 3) Guards — fail loudly on any mismatch.
NEWCUR="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
LOCKV="$(awk '/^name = "crustyimg"$/{getline; print; exit}' Cargo.lock | sed -E 's/version = "(.*)"/\1/')"
[ "$NEWCUR" = "$VERSION" ] || { echo "ERROR: Cargo.toml is $NEWCUR, expected $VERSION" >&2; exit 1; }
[ "$LOCKV" = "$VERSION" ]  || { echo "ERROR: Cargo.lock is $LOCKV, expected $VERSION" >&2; exit 1; }
echo "✓ Cargo.toml + Cargo.lock == $VERSION (this is what the tag must match)"

if grep -qE "^## \[$VERSION\]" CHANGELOG.md; then
  echo "✓ CHANGELOG has a '## [$VERSION]' section"
else
  echo "⚠ CHANGELOG.md has NO '## [$VERSION]' section." >&2
  echo "  Roll [Unreleased] → '## [$VERSION] - $(date +%F)' and add the link refs before tagging:" >&2
  echo "    [$VERSION]: https://github.com/jysf/crustyimg/releases/tag/v$VERSION" >&2
fi

cat <<EOF

Bumped to $VERSION. NOT committed/tagged (maintainer-authorized). Next (RELEASING.md):

  git commit -am "chore(release): v$VERSION" && git push origin main
  git tag -a v$VERSION -m "crustyimg v$VERSION" && git push origin v$VERSION   # fires the pipeline

The tag (v$VERSION) now matches Cargo.toml ($VERSION) — the check that would have
caught the v0.1.1 mis-tag.
EOF
