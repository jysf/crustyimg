#!/usr/bin/env node
// SPEC-075: the npm package's earned verdict.
//
// `npm pack` the finalized pkg/, install THAT TARBALL into a fresh temp project,
// and run the package from inside it — `init` the wasm, `info` a PNG, `transform`
// it, and decode the results back. A package.json that merely looks right but
// won't instantiate in a real consumer's project is the failure mode this exists
// to catch, so nothing here reads pkg/ directly: every call goes through the
// installed `crustyimg-wasm` in node_modules, resolved by Node the way a user's
// bundler would resolve it.
//
// It also asserts the two things that ARE the pitch (DEC-067):
//   * no native addon, no postinstall build step — pure JS + .wasm;
//   * the packaged .wasm is the size-profiled `just wasm-build` artifact
//     (DEC-066), not a bare `cargo build --target wasm32` (+109 KB on the wire).
//
// Run: `just wasm-npm-smoke`. No npm publish — that is SPEC-076, and gated.

import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, deflateSync } from "node:zlib";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = join(repoRoot, "pkg");

const PKG_NAME = "crustyimg-wasm";

// ── The profile guard ─────────────────────────────────────────────────────────
// What actually needs proving is "this .wasm came through the size-profiled
// `just wasm-build` (DEC-066)". Size alone is only a PROXY for that, and a weak
// one: the profiled build (1,394,631 B brotli) and a stock-profile build
// (1,503,485 B, measured) are just 7.8% apart, so a single hand-picked ceiling
// between them has ~4% of daylight on either side. It would false-trip on any
// legitimate 4% growth — and *misreport the cause* when it did ("a bare cargo
// build would blow this") — while its discriminating power erodes as the artifact
// grows, because a stock build grows with it.
//
// So the profile is asserted STRUCTURALLY, and size is asserted separately:
//
//  1. `strip = true` — one of DEC-066's three levers — is directly observable in
//     the binary: a stripped .wasm has no debug-name table. Measured: the profiled
//     artifact carries a 43 B `name` custom section, a stock-profile one carries
//     980,293 B. Four orders of magnitude — categorical rather than a threshold,
//     and immune to however much the code legitimately grows.
//  2. The wire size is then a plain REGRESSION baseline keyed to the measured
//     profiled artifact, so a real size regression is reported as a size
//     regression instead of being misdiagnosed as a bypassed recipe.
//  3. The floor still asserts the AVIF encoder is actually in (DEC-065) — a lean
//     build is hundreds of KB smaller and quietly ships a package that cannot do
//     the thing the demo is for.
const WASM_NAME_SECTION_MAX = 4_096;
const WASM_BROTLI_BASELINE = 1_394_631;
const WASM_BROTLI_TOLERANCE = 0.05;
const WASM_BROTLI_MAX = Math.round(WASM_BROTLI_BASELINE * (1 + WASM_BROTLI_TOLERANCE));
const WASM_BROTLI_MIN = Math.round(WASM_BROTLI_BASELINE * (1 - WASM_BROTLI_TOLERANCE));

const tmpDirs = [];
let failed = 0;

function ok(msg) {
  console.log(`  ✓ ${msg}`);
}

function check(cond, msg) {
  if (cond) {
    ok(msg);
  } else {
    failed++;
    console.error(`  ✗ ${msg}`);
  }
}

function die(msg) {
  console.error(`\nnpm-smoke: ${msg}`);
  cleanup();
  process.exit(1);
}

function cleanup() {
  for (const d of tmpDirs) {
    rmSync(d, { recursive: true, force: true });
  }
}

function mktemp(prefix) {
  const d = mkdtempSync(join(tmpdir(), prefix));
  tmpDirs.push(d);
  return d;
}

function run(cmd, args, cwd) {
  return execFileSync(cmd, args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
}

// ── A PNG fixture, encoded here in plain JS ────────────────────────────────────
// Deliberately NOT produced by the package under test: an input the crate's own
// encoder wrote would let a broken decode-encode round-trip agree with itself.
// This is a hand-rolled 8-bit RGB PNG (filter 0 scanlines, zlib IDAT) — the
// format's own spec, written by neither Rust nor the `image` crate.
function crc32(buf) {
  let c = ~0;
  for (const b of buf) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
}

function pngChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

function makePng(width, height) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 2; // colour type: truecolour RGB (no alpha)
  // 10..12 = compression, filter, interlace — all 0.

  // A gradient, not a solid fill: a solid image survives almost any bug.
  const raw = Buffer.alloc(height * (1 + width * 3));
  for (let y = 0; y < height; y++) {
    const row = y * (1 + width * 3);
    raw[row] = 0; // filter type 0 (None)
    for (let x = 0; x < width; x++) {
      const p = row + 1 + x * 3;
      raw[p] = (x * 255) / (width - 1);
      raw[p + 1] = (y * 255) / (height - 1);
      raw[p + 2] = 128;
    }
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", deflateSync(raw)),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

// ── The wasm section table ────────────────────────────────────────────────────
// Walk the module's sections. Id 0 is a custom section whose payload opens with a
// LEB128-length-prefixed name; `name` is the debug symbol table that `strip`
// removes. This is how we see the build profile in the artifact itself.
function customSections(buf) {
  const out = new Map();
  let o = 8; // skip the magic + version words
  const leb = () => {
    let r = 0;
    let s = 0;
    let b;
    do {
      b = buf[o++];
      r |= (b & 0x7f) << s;
      s += 7;
    } while (b & 0x80);
    return r >>> 0;
  };
  while (o < buf.length) {
    const id = buf[o++];
    const len = leb();
    const end = o + len;
    if (end > buf.length) break; // malformed; nothing further is trustworthy
    if (id === 0) {
      const nameLen = leb();
      out.set(buf.toString("utf8", o, o + nameLen), len);
    }
    o = end;
  }
  return out;
}

// ── 1. the packaged artifact ──────────────────────────────────────────────────
console.log("\n── the finalized pkg/ ──");

let pkgJson;
try {
  pkgJson = JSON.parse(readFileSync(join(pkgDir, "package.json"), "utf8"));
} catch {
  die("no finalized pkg/package.json — run `just wasm-npm-pkg` first");
}

check(pkgJson.name === PKG_NAME, `package is named ${PKG_NAME} (got ${pkgJson.name})`);
check(pkgJson.type === "module", "package is an ES module");
check(
  pkgJson.exports?.["./crustyimg_bg.wasm"] !== undefined,
  "the .wasm is reachable as a subpath export (bundlers need the URL)",
);
check(
  !pkgJson.scripts || Object.keys(pkgJson.scripts).length === 0,
  "no lifecycle scripts (nothing runs on a consumer's machine at install)",
);
check(
  !pkgJson.dependencies || Object.keys(pkgJson.dependencies).length === 0,
  "no runtime dependencies",
);

const wasmBytes = readFileSync(join(pkgDir, "crustyimg_bg.wasm"));
const brotli = brotliCompressSync(wasmBytes).length;
const raw = wasmBytes.length;
const nameSection = customSections(wasmBytes).get("name") ?? 0;
console.log(
  `    .wasm: ${(raw / 1048576).toFixed(2)} MB raw, ${(brotli / 1048576).toFixed(2)} MB brotli ` +
    `(${brotli} B), \`name\` section ${nameSection} B`,
);

// The load-bearing one: this proves the artifact was BUILT the right way, rather
// than inferring it from how much it weighs.
check(
  nameSection <= WASM_NAME_SECTION_MAX,
  `.wasm is stripped, so it came through the size-profiled \`just wasm-build\` (\`name\` debug ` +
    `section ${nameSection} B <= ${WASM_NAME_SECTION_MAX} — a stock-profile build carries ~980 KB ` +
    `there, DEC-066)`,
);
check(
  brotli <= WASM_BROTLI_MAX,
  `.wasm is within ${WASM_BROTLI_TOLERANCE * 100}% of the ${WASM_BROTLI_BASELINE} B baseline ` +
    `(brotli ${brotli} <= ${WASM_BROTLI_MAX}) — if this growth is deliberate, move ` +
    `WASM_BROTLI_BASELINE and say why`,
);
check(
  brotli >= WASM_BROTLI_MIN,
  `.wasm carries the AVIF encoder (brotli ${brotli} >= ${WASM_BROTLI_MIN} — a build without ` +
    `--features avif would be far smaller, DEC-065)`,
);

if (failed) die(`${failed} check(s) failed before packing`);

// ── 2. npm pack → fresh install ───────────────────────────────────────────────
console.log("\n── npm pack → fresh install ──");

const tarballDir = mktemp("crustyimg-tarball-");
const packOut = run("npm", ["pack", "--json", "--pack-destination", tarballDir], pkgDir);
const packed = JSON.parse(packOut)[0];
const tarball = join(tarballDir, packed.filename);
ok(`packed ${packed.filename} (${(statSync(tarball).size / 1048576).toFixed(2)} MB)`);

const consumer = mktemp("crustyimg-consumer-");
writeFileSync(
  join(consumer, "package.json"),
  `${JSON.stringify({ name: "crustyimg-smoke", private: true, version: "0.0.0", type: "module" }, null, 2)}\n`,
);

// NOT --ignore-scripts: letting npm run whatever the package declares is the
// point. If a postinstall compile step ever sneaks in, this is where it fires.
run("npm", ["install", tarball, "--no-audit", "--no-fund"], consumer);
ok("installed the tarball into a fresh project (no build step ran)");

// ── 3. no native addon ────────────────────────────────────────────────────────
console.log("\n── no native addon ──");

const installed = join(consumer, "node_modules", PKG_NAME);

function walk(dir, out = []) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

// npm's own bookkeeping (`.package-lock.json`, `.bin/`) is not a dependency.
const files = walk(join(consumer, "node_modules"))
  .map((p) => p.slice(join(consumer, "node_modules").length + 1))
  .filter((f) => !f.startsWith("."));
const nativeBits = files.filter((f) => /\.(node|dylib|so|dll|a)$/.test(f) || /binding\.gyp$/.test(f));
check(nativeBits.length === 0, `no .node/.dylib/.so/.dll/binding.gyp anywhere in node_modules`);
if (nativeBits.length) console.error(`    found: ${nativeBits.join(", ")}`);

const installedJson = JSON.parse(readFileSync(join(installed, "package.json"), "utf8"));
check(
  !installedJson.scripts || Object.keys(installedJson.scripts).length === 0,
  "the INSTALLED package.json declares no install/postinstall/prepare script",
);
check(
  files.every((f) => f.startsWith(`${PKG_NAME}/`)),
  `the install pulled in nothing but ${PKG_NAME} itself (zero transitive deps)`,
);
console.log(`    shipped: ${readdirSync(installed).sort().join(", ")}`);

// ── 4. it runs ────────────────────────────────────────────────────────────────
console.log("\n── init + info + transform, from the installed package ──");

writeFileSync(join(consumer, "fixture.png"), makePng(64, 48));

// The consumer runs in its own process, in its own directory, importing the bare
// specifier `crustyimg-wasm` — i.e. exactly what a user's code does.
writeFileSync(
  join(consumer, "consume.mjs"),
  `
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { initSync, info, transform, version } from "${PKG_NAME}";

const require = createRequire(import.meta.url);

// --target web does not auto-instantiate: the .wasm is a separate asset and the
// caller decides when the 1.3 MB lands (DEC-067). In Node, fetch() can't read a
// file:// URL, so we hand initSync the bytes — resolved through the package's
// own subpath export, which is what proves that export is real.
const wasmPath = require.resolve("${PKG_NAME}/crustyimg_bg.wasm");
initSync({ module: readFileSync(wasmPath) });

const src = readFileSync("fixture.png");

const meta = info(src);
const results = {
  version: version(),
  info: { width: meta.width, height: meta.height, format: meta.format, hasAlpha: meta.hasAlpha },
  out: {},
};

const recipe = 'version = "1"\\n\\n[[step]]\\nop = "resize"\\nmode = "exact"\\nwidth = 32\\nheight = 24\\n';

for (const fmt of ["png", "jpeg", "webp", "avif"]) {
  const bytes = transform(src, recipe, fmt);
  // 32 bytes: enough to reach a PNG's IHDR width/height (offsets 16 and 20).
  const entry = { len: bytes.length, head: Buffer.from(bytes.slice(0, 32)).toString("hex") };
  // Feed the output back through the package's own decoder where it HAS one.
  // AVIF is encode-only in this build (DEC-065) — the browser decodes it natively —
  // so its bytes are checked by the driver instead of round-tripped here.
  if (fmt !== "avif") {
    const back = info(bytes);
    entry.back = { width: back.width, height: back.height, format: back.format };
  }
  results.out[fmt] = entry;
}

console.log(JSON.stringify(results));
`,
);

let results;
try {
  const out = run("node", ["consume.mjs"], consumer);
  results = JSON.parse(out.trim().split("\n").pop());
} catch (e) {
  console.error(e.stderr || e.message);
  die("the installed package failed to init or run — this is THE failure this test exists for");
}

const crateVersion = readFileSync(join(repoRoot, "Cargo.toml"), "utf8").match(
  /^version\s*=\s*"([^"]+)"/m,
)[1];
check(results.version === crateVersion, `version() reports the crate version ${crateVersion}`);

check(
  results.info.width === 64 && results.info.height === 48 && results.info.format === "png",
  `info(png) → ${results.info.width}x${results.info.height} ${results.info.format} (expected 64x48 png)`,
);
check(results.info.hasAlpha === false, "info(png) → hasAlpha false (the fixture is RGB)");

// The load-bearing one: the transform's OUTPUT BYTES decode to the resized
// dimensions. Checked twice, once by a decoder we did not write — the PNG header
// is parsed here in plain JS, independently of anything the crate does.
const png = results.out.png;
const head = Buffer.from(png.head, "hex");
const sig = head.subarray(0, 8).toString("hex") === "89504e470d0a1a0a";
const ihdrW = head.readUInt32BE(16);
const ihdrH = head.readUInt32BE(20);
check(sig, "transform(png) → real PNG signature");
check(
  ihdrW === 32 && ihdrH === 24,
  `transform(png) → PNG IHDR says ${ihdrW}x${ihdrH} (independent parse; expected 32x24)`,
);
check(
  png.back.width === 32 && png.back.height === 24 && png.back.format === "png",
  `transform(png) → re-decodes to ${png.back.width}x${png.back.height} ${png.back.format} (expected 32x24 png)`,
);

const jpeg = results.out.jpeg;
check(jpeg.head.startsWith("ffd8ff"), "transform(jpeg) → real JPEG SOI marker");
check(
  jpeg.back.width === 32 && jpeg.back.height === 24 && jpeg.back.format === "jpeg",
  `transform(jpeg) → re-decodes to ${jpeg.back.width}x${jpeg.back.height} ${jpeg.back.format}`,
);

const webp = results.out.webp;
const webpHead = Buffer.from(webp.head, "hex");
check(
  webpHead.subarray(0, 4).toString("ascii") === "RIFF" &&
    webpHead.subarray(8, 12).toString("ascii") === "WEBP",
  "transform(webp) → real RIFF/WEBP container",
);
check(
  webp.back.width === 32 && webp.back.height === 24 && webp.back.format === "webp",
  `transform(webp) → re-decodes to ${webp.back.width}x${webp.back.height} ${webp.back.format}`,
);

// AVIF: encode-only in this build (DEC-065), so the package cannot decode its own
// output and this is a container-level check, not a decode. Naming the limit here
// rather than pretending: the browser's createImageBitmap is the decoder that
// matters, and verify drives an independent one.
const avif = results.out.avif;
const avifHead = Buffer.from(avif.head, "hex");
check(
  avifHead.subarray(4, 8).toString("ascii") === "ftyp" &&
    avifHead.subarray(8, 12).toString("ascii") === "avif",
  `transform(avif) → ISOBMFF ftyp/avif box, ${avif.len} B (rav1e ran in wasm — the headline)`,
);

cleanup();

console.log("");
if (failed) {
  console.error(`npm-smoke: ${failed} check(s) FAILED\n`);
  process.exit(1);
}
console.log(`npm-smoke: ${PKG_NAME}@${crateVersion} packs, installs clean, and runs. ✓\n`);
