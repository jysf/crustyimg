#!/usr/bin/env node
// Copy the ddphotos edit page, plus the built wasm package, into a folder.
//
// Point it at a ddphotos site's config directory and the page lands in
// config/static/edit/, which ddphotos copies to the site root as /edit/:
//
//   just wasm-build
//   node scripts/ddphotos-edit-install.mjs ~/my-ddphotos/config
//
// Any other folder works too; it is used as the edit/ directory itself when it is
// not a ddphotos config directory (no albums.yaml inside).

import { copyFileSync, existsSync, mkdirSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const page = join(repo, "integrations", "ddphotos", "edit");
const pkg = join(repo, "pkg");

const arg = process.argv[2];
if (!arg) {
  console.error("usage: node scripts/ddphotos-edit-install.mjs <ddphotos config dir | target dir>");
  process.exit(2);
}

for (const f of ["crustyimg.js", "crustyimg_bg.wasm"]) {
  if (!existsSync(join(pkg, f))) {
    console.error(`missing pkg/${f}: run 'just wasm-build' first`);
    process.exit(1);
  }
}

const target = resolve(arg);
const isConfigDir = existsSync(join(target, "albums.yaml"));
const out = isConfigDir ? join(target, "static", "edit") : target;

mkdirSync(join(out, "vendor"), { recursive: true });
for (const f of readdirSync(page)) copyFileSync(join(page, f), join(out, f));
for (const f of ["crustyimg.js", "crustyimg_bg.wasm"]) {
  copyFileSync(join(pkg, f), join(out, "vendor", f));
}

console.log(`edit page installed in ${out}`);
if (isConfigDir) console.log("rebuild the site and open /edit/");
