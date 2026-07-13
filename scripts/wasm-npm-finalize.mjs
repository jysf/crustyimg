#!/usr/bin/env node
// Turn `just wasm-build`'s pkg/ into the npm package we would actually publish
// (SPEC-075, DEC-067).
//
// wasm-pack emits a nearly-right package.json, but it can only know what is in
// Cargo.toml: it names the package after the CRATE (`crustyimg` — the CLI) and
// describes it as "A fast Rust CLI to view and transform images", which is the
// wrong artifact and the wrong audience for someone typing `npm install`. This
// script merges the npm identity from npm/package.overrides.json and swaps the
// copied 13 KB CLI README for the npm-facing one.
//
// It runs AFTER wasm-pack, never instead of it: pkg/ is regenerated from scratch
// on every build (so a hand-edit there cannot survive), and the .wasm must come
// through the size-profiled `just wasm-build` (DEC-066) — a bare `cargo build
// --target wasm32` silently ships +109 KB on the wire.
//
// Publishing is NOT this script's job and never will be (DEC-067): it stops at a
// finalized pkg/. `npm publish` is gated on maintainer approval (SPEC-076).

import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = join(repoRoot, "pkg");
const pkgJsonPath = join(pkgDir, "package.json");

function die(msg) {
  console.error(`wasm-npm-finalize: ${msg}`);
  process.exit(1);
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (e) {
    die(`cannot read ${path}: ${e.message}`);
  }
}

// wasm-pack must have run first, and it must have produced the .wasm — not just
// the JS shim.
for (const f of ["package.json", "crustyimg.js", "crustyimg.d.ts", "crustyimg_bg.wasm"]) {
  try {
    readFileSync(join(pkgDir, f));
  } catch {
    die(`pkg/${f} is missing — run \`just wasm-build\` first`);
  }
}

const pkg = readJson(pkgJsonPath);
const overrides = readJson(join(repoRoot, "npm", "package.overrides.json"));
delete overrides._comment;

// The npm version is the crate version, in lockstep, with no independent line to
// maintain (DEC-067). wasm-pack already copies it from Cargo.toml — this only
// guards the invariant, so a future override or a stale pkg/ can't quietly drift.
const cargoToml = readFileSync(join(repoRoot, "Cargo.toml"), "utf8");
const crateVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!crateVersion) die("could not read `version` from Cargo.toml");
if (pkg.version !== crateVersion) {
  die(
    `pkg/package.json version ${pkg.version} != Cargo.toml version ${crateVersion} — ` +
      `pkg/ is stale; re-run \`just wasm-build\``,
  );
}
if ("version" in overrides) die("npm/package.overrides.json must not set `version` (DEC-067)");

// A publish must be a deliberate act, so the package may not carry a lifecycle
// script that could run one (or compile anything) on a consumer's machine.
const merged = { ...pkg, ...overrides };
if (merged.scripts && Object.keys(merged.scripts).length > 0) {
  die(`package.json must carry no lifecycle scripts, found: ${Object.keys(merged.scripts).join(", ")}`);
}

writeFileSync(pkgJsonPath, `${JSON.stringify(merged, null, 2)}\n`);
copyFileSync(join(repoRoot, "npm", "README.md"), join(pkgDir, "README.md"));

console.log(`✓ pkg/ finalized as ${merged.name}@${merged.version} (target: web, DEC-067)`);
