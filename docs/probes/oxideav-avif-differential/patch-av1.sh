#!/bin/sh
# Fetch oxideav-av1 0.1.21 into wasm/patched-av1 and stub the one std::time::Instant
# use that panics on wasm32-unknown-unknown. Probe-only: the shim zeroes the stage timings.
set -eu
cd "$(dirname "$0")/wasm"
rm -rf patched-av1 && mkdir patched-av1
curl -sL -A "crustyimg-probe" "https://static.crates.io/crates/oxideav-av1/oxideav-av1-0.1.21.crate" \
  | tar xz -C patched-av1 --strip-components 1
python3 -I - patched-av1/src/decoder/frame_driver.rs <<'PY'
import sys
p = sys.argv[1]; s = open(p).read()
old = "use std::time::{Duration, Instant};"
assert s.count(old) == 1, "upstream changed; re-check whether the shim is still needed"
s = s.replace(old, """use std::time::Duration;
// PROBE SHIM: std::time::Instant panics on wasm32-unknown-unknown; stage timings become zero.
#[derive(Clone, Copy)]
struct Instant;
impl Instant {
    fn now() -> Self { Instant }
    fn elapsed(&self) -> Duration { Duration::ZERO }
}""")
open(p, "w").write(s)
PY
echo "patched: wasm/patched-av1 — now uncomment [patch.crates-io] in wasm/Cargo.toml"
