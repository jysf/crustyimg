#!/usr/bin/env node
// The ddphotos edit page's recipes must mean the same thing to the native CLI as to
// the page's wasm build.
//
// For each row of a fixed table this builds the recipe with the page's own code
// (integrations/ddphotos/edit/recipe.js, nothing re-typed here), runs it through the
// slim wasm package's `transform` (the real .wasm, in Node) and through native
// `crustyimg apply` on the same input bytes, decodes both outputs with Pillow
// (tests/ddphotos_decode.py, no crustyimg code), and compares the pixels.
//
// Run:  node tests/ddphotos_parity.mjs --pkg <slim pkg dir> --bin <crustyimg> \
//                                      --python <python with Pillow> [--self-test]
// `just ddphotos-parity` builds all three and runs both modes.
//
// There is no skip path: a missing package, binary, decoder or export exits non-zero.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..");
const EDIT_DIR = join(ROOT, "integrations/ddphotos/edit");
const FIXTURES = join(HERE, "fixtures/ddphotos");

// How many pixels a resampling row may differ in, each by at most 1/255 per channel
// (DEC-110). Native and wasm round the resize's linear-light maths differently on a
// few pixels. This is the largest count observed on any resampling row over every
// target the harness has run on. It is measured, so it gets no headroom.
const K = 1;

// The most K may ever be: 0.01 % of the smallest resampling row's pixels (1080×723).
// A row above it is a failure to report, never a reason to raise this (DEC-110).
const K_CAP = 78;

// Whether JPEG bytes are asserted on the rows without a resize. The first x86_64 run
// (AMD EPYC runner) gave the same JPEG bytes as the aarch64 Mac, so they are.
const JPEG_BYTES_ASSERTED = true;

if (K > K_CAP) {
  console.error(`K (${K}) is above the cap (${K_CAP})`);
  process.exit(1);
}

// ── the comparator ───────────────────────────────────────────────────────────

/// The verdict on one row, as a pure function of what was measured.
///
/// `dimsEqual` is whether native and wasm gave the same size, `bytesEqual` whether
/// the encoded files are identical, `px` how many pixels differ, `max` the largest
/// channel difference, and `resamples` whether the recipe has a resize step.
/// Returns `{ ok, reason }`; `reason` is `dims`, `max`, `count`, `exact` or `bytes`.
export function judge({ dimsEqual, bytesEqual, px, max, resamples, format }) {
  if (!dimsEqual) return { ok: false, reason: "dims" };
  if (resamples) {
    if (max > 1) return { ok: false, reason: "max" };
    if (px > K) return { ok: false, reason: "count" };
  } else {
    if (px !== 0) return { ok: false, reason: "exact" };
    const bytesMatter = format === "png" || (format === "jpeg" && JPEG_BYTES_ASSERTED);
    if (bytesMatter && !bytesEqual) return { ok: false, reason: "bytes" };
  }
  return { ok: true, reason: "" };
}

/// Compare two decoded images. Dimensions first: on a mismatch the buffers are never
/// read. `a` and `b` are `{ width, height, rgba }`.
export function compareImages(a, b, bytesEqual, resamples, format) {
  if (a.width !== b.width || a.height !== b.height) {
    return { px: null, max: null, ...judge({ dimsEqual: false, bytesEqual, resamples, format }) };
  }
  let px = 0;
  let max = 0;
  const n = a.width * a.height * 4;
  for (let i = 0; i < n; i += 4) {
    let d = 0;
    for (let c = 0; c < 4; c++) d = Math.max(d, Math.abs(a.rgba[i + c] - b.rgba[i + c]));
    if (d > 0) {
      px++;
      if (d > max) max = d;
    }
  }
  return { px, max, ...judge({ dimsEqual: true, bytesEqual, px, max, resamples, format }) };
}

/// A recipe resamples when it has a resize step. Read off the recipe text, not the
/// edit: `cliRecipe` always resizes, even for an edit that asks for no new size.
export function resamples(recipeText) {
  return /^op = "resize"$/m.test(recipeText);
}

// ── the rows ─────────────────────────────────────────────────────────────────

const WATERMARK_TEXT = "© Jane Doe 2026";

// The page's form defaults, with the size and the watermark switch left to each edit.
const FORM = {
  wmText: WATERMARK_TEXT,
  wmGravity: "southeast",
  wmSize: "0.04",
  wmColor: "#ffffff",
  wmOpacity: "0.6",
  format: "png",
};

// "As published" is the photo's long edge (1600), as the page's size list offers it.
const EDITS = {
  none: { size: "1600", wmOn: false },
  resize: { size: "1200", wmOn: false },
  wm: { size: "1600", wmOn: true },
  both: { size: "1080", wmOn: true },
};

// 1080 is here because 1071 × 0.675 = 722.9 rounds to 723 but floors to 722; 1200
// is the page's first size and the first where native and wasm differ.
// [n, input, edit, recipe, format, output size]
const TABLE = [
  [1, "W", "none", "pixel", "png", [1600, 1071]],
  [2, "W", "resize", "pixel", "png", [1200, 803]],
  [3, "W", "wm", "pixel", "png", [1600, 1071]],
  [4, "W", "both", "pixel", "png", [1080, 723]],
  [5, "W", "none", "cli", "png", [1600, 1071]],
  [6, "W", "resize", "cli", "png", [1200, 803]],
  [7, "W", "wm", "cli", "png", [1600, 1071]],
  [8, "W", "both", "cli", "png", [1080, 723]],
  [9, "W", "none", "preview", "jpeg", [1600, 1071]],
  [10, "W", "resize", "preview", "jpeg", [1200, 803]],
  [11, "W", "wm", "preview", "jpeg", [1600, 1071]],
  [12, "W", "both", "preview", "jpeg", [1080, 723]],
  [13, "W", "both", "pixel", "webp", [1080, 723]],
  [14, "W", "both", "pixel", "avif", [1080, 723]],
  // The rotated JPEG is stored 1600×1071 and shown 1071×1600. The page plans from the
  // shown size (`loadPhoto`), so its prediction is the output's sides as they are.
  [15, "R", "resize", "pixel", "png", [803, 1200]],
  [16, "R", "both", "pixel", "png", [723, 1080]],
].map(([n, input, edit, recipe, format, size]) => ({
  n,
  input,
  edit,
  recipe,
  format,
  size,
  family: "table",
  crop: null,
  form: EDITS[edit],
  id: `${input}/${edit}/${recipe}/${format}`,
}));

const ROW_COUNT = 16;
const NO_RESAMPLE_ROWS = [1, 3, 9, 11];

const INPUTS = {
  W: "photo.webp",
  R: "photo_rot.jpg",
  P: "photo_portrait.webp",
  S: "photo_square.webp",
  N: "photo_near_square.webp",
};
const STORED_SIZE = { width: 1600, height: 1071 };
const EXT = { png: "png", jpeg: "jpg", webp: "webp", avif: "avif" };

// Golden recipe texts: a change to recipe.js that moved both sides together would
// otherwise stay green.
const GOLDEN_PIXEL_BOTH = `version = "2"

[[step]]
op = "auto-orient"

[[step]]
op = "resize"
mode = "max"
width = 1080

[[step]]
op = "watermark"
text = "© Jane Doe 2026"
gravity = "southeast"
size = 43
margin = 22
color = "#ffffff"
opacity = 0.6
`;
const GOLDEN_CLI_BOTH = GOLDEN_PIXEL_BOTH.replace('version = "2"\n', 'version = "2"\nformat = "png"\n');
// Row 12: the preview path, `previewRecipe(pixelRecipe(both))`. Without it a
// previewRecipe that dropped its quality would stay green, native and wasm moving together.
const GOLDEN_PREVIEW_BOTH = GOLDEN_PIXEL_BOTH.replace('version = "2"\n', 'version = "2"\nquality = 88\n');

// ── the run ──────────────────────────────────────────────────────────────────

function fatal(message) {
  console.error(`FATAL: ${message}`);
  process.exit(1);
}

function parseArgs(argv) {
  const args = { selfTest: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--self-test") args.selfTest = true;
    else if (a === "--pkg" || a === "--bin" || a === "--python") args[a.slice(2)] = argv[++i];
    else fatal(`unknown argument ${a}`);
  }
  for (const k of ["pkg", "bin", "python"]) {
    if (!args[k]) fatal(`--${k} is required (there are no defaults that could point at a stale build)`);
  }
  return args;
}

/// Load recipe.js. A failed import is a failure, not a skip.
async function loadRecipes() {
  try {
    return await import(pathToFileURL(join(EDIT_DIR, "recipe.js")).href);
  } catch (e) {
    return fatal(`recipe.js failed to import: ${e.message}`);
  }
}

/// AC-1 and AC-8: the page files parse as modules, import only what recipe.js
/// exports, and do not carry their own copy of the recipe code.
function checkModules(recipes) {
  const exported = new Set(Object.keys(recipes));
  const scratch = mkdtempSync(join(tmpdir(), "ddphotos-modules-"));
  const problems = [];
  const owned = [
    "tomlString",
    "GO_ADVANCE",
    "GO_WIDEST",
    "textEms",
    "watermarkSize",
    "planEdit",
    "pixelRecipe",
    "cliRecipe",
    "withQuality",
    "PREVIEW_QUALITY",
    "aspectSize",
    "followOrientation",
    "presetCrop",
    "swapCrop",
    "cropBox",
    "cropStep",
    "shownAspect",
    "loadPhoto",
    "ORIENTED_BASE",
  ];
  // What the model and the loader need from recipe.js, and the one line the worker's
  // `load` is: the harness runs `loadPhoto` itself, so the worker must run the same one.
  for (const name of ["aspectSize", "followOrientation", "presetCrop", "swapCrop", "cropBox", "loadPhoto"]) {
    if (!exported.has(name)) problems.push(`recipe.js lacks ${name}`);
  }
  for (const file of ["edit.js", "worker.js"]) {
    const text = readFileSync(join(EDIT_DIR, file), "utf8");
    // A .js file may be parsed as CommonJS; the .mjs extension forces a module.
    const copy = join(scratch, `${file}.mjs`);
    copyFileSync(join(EDIT_DIR, file), copy);
    const check = spawnSync(process.execPath, ["--check", copy], { encoding: "utf8" });
    if (check.status !== 0) problems.push(`${file} does not parse as a module: ${check.stderr}`);
    const imports = [...text.matchAll(/import\s*\{([^}]*)\}\s*from\s*"\.\/recipe\.js"/g)];
    if (file === "edit.js" || file === "worker.js") {
      if (imports.length === 0) problems.push(`${file} does not import from ./recipe.js`);
    }
    for (const m of imports) {
      for (const name of m[1].split(",").map((s) => s.trim()).filter(Boolean)) {
        if (!exported.has(name)) problems.push(`${file} imports ${name}, which recipe.js lacks`);
      }
    }
    if (file === "worker.js" && !text.includes("return loadPhoto({ info, transform }, source);")) {
      problems.push("worker.js's load does not return loadPhoto({ info, transform }, source);");
    }
    for (const name of owned) {
      if (new RegExp(`(function|const|let)\\s+${name}\\b`).test(text)) {
        problems.push(`${file} defines ${name}, which belongs to recipe.js`);
      }
    }
  }
  const source = readFileSync(join(EDIT_DIR, "recipe.js"), "utf8");
  for (const forbidden of [/\bdocument\b/, /\$\(/, /\bwindow\b/]) {
    if (forbidden.test(source)) problems.push(`recipe.js reads the page: ${forbidden}`);
  }
  rmSync(scratch, { recursive: true, force: true });
  return problems;
}

/// Load the slim wasm package and return its `transform` and `info`.
async function loadWasm(pkg) {
  const js = join(pkg, "crustyimg.js");
  const wasm = join(pkg, "crustyimg_bg.wasm");
  if (!existsSync(js) || !existsSync(wasm)) {
    return fatal(`wasm package missing or empty: expected ${js} and ${wasm}`);
  }
  try {
    const mod = await import(pathToFileURL(js).href);
    mod.initSync({ module: readFileSync(wasm) });
    return { transform: mod.transform, info: mod.info };
  } catch (e) {
    return fatal(`wasm package failed to initialise: ${e.message}`);
  }
}

/// The page's crop for a row's crop spec: null, a preset (`{ preset: [a, b] }`, with
/// `swap` for the swapped orientation) or a box (`{ box: [x, y, w, h] }`).
function cropFor(recipes, spec, photo) {
  if (!spec) return null;
  if (spec.box) {
    const [x, y, w, h] = spec.box;
    return { box: { x, y, w, h } };
  }
  const preset = recipes.presetCrop(...spec.preset, photo);
  return spec.swap ? recipes.swapCrop(preset, photo) : preset;
}

/// The photo as the page reads it: `loadPhoto` on the input's bytes, the production
/// function, so the oriented size comes the way the worker gets it.
function photoFor(ctx, input) {
  return ctx.recipes.loadPhoto(ctx, readFileSync(join(FIXTURES, INPUTS[input])));
}

/// The recipe text for a row, built only from recipe.js.
function recipeFor(row, ctx) {
  const { recipes } = ctx;
  const photo = photoFor(ctx, row.input);
  const crop = cropFor(recipes, row.crop, photo);
  const edit = recipes.planEdit(photo, { ...FORM, ...row.form }, crop);
  const text =
    row.recipe === "pixel"
      ? recipes.pixelRecipe(edit)
      : row.recipe === "cli"
        ? recipes.cliRecipe(edit)
        : recipes.previewRecipe(recipes.pixelRecipe(edit));
  return { edit, text };
}

/// Run every row through wasm and native, decode, compare. `mutateNative` rewrites
/// the recipe text on the native side only, after it has left recipe.js. Returns the
/// rows' results; nothing here exits on a failing row.
function runTable(ctx, rowList, mutateNative = (t) => t) {
  const { transform, bin, python } = ctx;
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-parity-"));
  const rows = [];
  const files = [];
  for (const row of rowList) {
    const inputBytes = readFileSync(join(FIXTURES, INPUTS[row.input]));
    const base = String(row.n).padStart(2, "0");
    const wasmOut = join(dir, `${base}.wasm.${EXT[row.format]}`);
    const nativeOut = join(dir, `${base}.native.${EXT[row.format]}`);
    const recipeFile = join(dir, `${base}.toml`);
    const result = { row, edit: null, text: null, resamples: false, error: null };
    try {
      // Inside the row's try: a recipe.js that throws on one row fails that row.
      const { edit, text } = recipeFor(row, ctx);
      result.edit = edit;
      result.text = text;
      result.resamples = resamples(text);
      writeFileSync(wasmOut, transform(inputBytes, text, row.format));
      writeFileSync(recipeFile, mutateNative(text));
      const run = spawnSync(
        bin,
        ["apply", "--recipe", recipeFile, join(FIXTURES, INPUTS[row.input]), "-o", nativeOut, "-Q"],
        { encoding: "utf8" },
      );
      if (run.error) fatal(`native binary could not run: ${run.error.message}`);
      if (run.status !== 0) throw new Error(`native apply exited ${run.status}: ${run.stderr.trim()}`);
      files.push(wasmOut, nativeOut);
      result.wasmOut = wasmOut;
      result.nativeOut = nativeOut;
    } catch (e) {
      result.error = e.message;
    }
    rows.push(result);
  }

  const dims = decode(python, files);
  for (const r of rows) {
    if (r.error) {
      r.fail = [`threw: ${r.error}`];
      continue;
    }
    const native = image(dims, r.nativeOut);
    const wasm = image(dims, r.wasmOut);
    const wasmBytes = readFileSync(r.wasmOut);
    r.native = native;
    r.wasm = wasm;
    r.bytesEqual = Buffer.compare(readFileSync(r.nativeOut), wasmBytes) === 0;
    r.sha = createHash("sha256").update(wasmBytes).digest("hex");
    r.nativeSha = createHash("sha256").update(readFileSync(r.nativeOut)).digest("hex");
    r.cmp = compareImages(native, wasm, r.bytesEqual, r.resamples, r.row.format);
    r.fail = [];
    if (!r.cmp.ok) r.fail.push(`comparator: ${r.cmp.reason}`);
    const want = r.row.size;
    if (wasm.width !== want[0] || wasm.height !== want[1]) {
      r.fail.push(`wasm size ${wasm.width}x${wasm.height}, expected ${want[0]}x${want[1]}`);
    }
    if (native.width !== want[0] || native.height !== want[1]) {
      r.fail.push(`native size ${native.width}x${native.height}, expected ${want[0]}x${want[1]}`);
    }
    // The page's own arithmetic predicts the size, from the photo's shown size.
    const predicted = [r.edit.outW, r.edit.outH];
    if (predicted[0] !== want[0] || predicted[1] !== want[1]) {
      r.fail.push(`planEdit predicts ${predicted.join("x")}, expected ${want.join("x")}`);
    }
    // The watermark the page plans, where the row names it: its size and whether it was
    // made smaller to fit.
    if (r.row.mark) {
      const wm = r.edit.watermark;
      const got = wm ? `${wm.size}${wm.capped ? " capped" : ""}` : "none";
      const exp = `${r.row.mark[0]}${r.row.mark[1] ? " capped" : ""}`;
      if (got !== exp) r.fail.push(`planEdit's watermark is ${got}, expected ${exp}`);
    }
  }
  return { dir, rows };
}

function decode(python, files) {
  const run = spawnSync(python, ["-I", join(HERE, "ddphotos_decode.py"), ...files], {
    encoding: "utf8",
    maxBuffer: 1 << 26,
  });
  if (run.error || run.status !== 0) {
    return fatal(
      `the Pillow decoder failed (${python}): ${run.error?.message ?? run.stderr.trim()}`,
    );
  }
  const out = new Map();
  for (const line of run.stdout.split("\n").filter(Boolean)) {
    const j = JSON.parse(line);
    out.set(j.path, j);
  }
  return out;
}

function image(dims, path) {
  const d = dims.get(path);
  if (!d) return fatal(`the decoder printed nothing for ${path}`);
  return { width: d.width, height: d.height, rgba: readFileSync(`${path}.rgba`) };
}

/// Checks that are about the table as a whole, not one row. Returns failure messages.
function tableChecks({ rows: allRows }) {
  const failures = [];
  const rows = allRows.filter((r) => r.row.family === "table");
  const ran = rows.filter((r) => !r.error).length;
  if (rows.length !== ROW_COUNT) failures.push(`P4: ${rows.length} rows ran, expected ${ROW_COUNT}`);
  if (ran !== rows.length) failures.push(`rows-threw: ${rows.length - ran} rows threw`);
  const exact = rows.filter((r) => !r.resamples).map((r) => r.row.n);
  if (JSON.stringify(exact) !== JSON.stringify(NO_RESAMPLE_ROWS)) {
    failures.push(`resize-set: the rows without a resize step are {${exact}}, expected {${NO_RESAMPLE_ROWS}}`);
  }
  const byN = (n) => rows.find((r) => r.row.n === n);
  const [r1, r2, r3, r4, r8, r12] = [1, 2, 3, 4, 8, 12].map(byN);
  if (r4 && r4.text !== GOLDEN_PIXEL_BOTH) failures.push("P5-pixel: row 4's recipe text moved");
  if (r8 && r8.text !== GOLDEN_CLI_BOTH) failures.push("P5-cli: row 8's recipe text moved");
  if (r12 && r12.text !== GOLDEN_PREVIEW_BOTH) failures.push("P5-preview: row 12's recipe text moved");
  if (r2?.wasm && r2.wasm.width === STORED_SIZE.width && r2.wasm.height === STORED_SIZE.height) {
    failures.push("P2: row 2's resize left the size unchanged");
  }
  if (r1?.wasm && r3?.wasm) {
    for (const side of ["wasm", "native"]) {
      if (Buffer.compare(r1[side].rgba, r3[side].rgba) === 0) {
        failures.push(`P3: the watermark drew nothing on the ${side} side (row 3 == row 1)`);
      }
    }
  }
  return failures;
}

function report(table) {
  for (const r of table.rows) {
    if (r.error || !r.native) {
      console.log(`FAIL ${r.row.n} ${r.row.id} ${r.error ?? "did not run"}`);
      continue;
    }
    const verdict = r.fail.length === 0 ? "PASS" : "FAIL";
    const c = r.cmp;
    console.log(
      `${verdict} ${String(r.row.n).padStart(2)} ${r.row.id} ` +
        `native ${r.native.width}x${r.native.height} wasm ${r.wasm.width}x${r.wasm.height} ` +
        `bytes ${r.bytesEqual ? "same" : "differ"} px=${c.px ?? "-"} max=${c.max ?? "-"} ` +
        `${r.resamples ? "resamples" : "exact"} wasm-sha256=${r.sha} native-sha256=${r.nativeSha}`,
    );
    if (r.fail.length) {
      for (const f of r.fail) console.log(`     ${f}`);
      console.log(r.text.replace(/^/gm, "     | "));
    }
  }
}

// ── the self-test ────────────────────────────────────────────────────────────

/// A flat buffer of `n` pixels with the first `differ` pixels changed by `by`.
function flat(w, h, differ = 0, by = 0) {
  const base = new Uint8Array(w * h * 4).fill(100);
  const other = base.slice();
  for (let p = 0; p < differ; p++) other[p * 4] += by;
  return [
    { width: w, height: h, rgba: base },
    { width: w, height: h, rgba: other },
  ];
}

/// B1-B7: the comparator's limbs, driven on synthetic buffers with no binary.
function comparatorCases() {
  const throwing = (w, h) => ({
    width: w,
    height: h,
    get rgba() {
      throw new Error("pixels were read");
    },
  });
  const cases = [
    ["B1 one pixel off by 2", () => compareImages(...flat(8, 8, 1, 2), false, true, "png"), false, "max"],
    ["B2 one pixel off by 1", () => compareImages(...flat(8, 8, 1, 1), false, true, "png"), true, ""],
    [
      "B3 K+1 pixels off by 1",
      () => compareImages(...flat(16, 16, K + 1, 1), false, true, "png"),
      false,
      "count",
    ],
    [
      "B4 exactly K pixels off by 1",
      () => compareImages(...flat(16, 16, K, 1), false, true, "png"),
      true,
      "",
    ],
    [
      "B5 one pixel off by 1, no resize",
      () => compareImages(...flat(8, 8, 1, 1), false, false, "png"),
      false,
      "exact",
    ],
    ["B6 identical, no resize", () => compareImages(...flat(8, 8), true, false, "png"), true, ""],
    [
      "B7 1080x723 vs 1080x722",
      () => compareImages(throwing(1080, 723), throwing(1080, 722), false, true, "png"),
      false,
      "dims",
    ],
    [
      "B8 jpeg bytes differ, no resize",
      () => compareImages(...flat(8, 8), false, false, "jpeg"),
      false,
      "bytes",
    ],
    [
      "B9 jpeg bytes differ, resizes",
      () => compareImages(...flat(8, 8), false, true, "jpeg"),
      true,
      "",
    ],
    [
      "B10 png bytes differ, no resize",
      () => compareImages(...flat(8, 8), false, false, "png"),
      false,
      "bytes",
    ],
  ];
  const results = [];
  for (const [name, run, ok, reason] of cases) {
    let got;
    try {
      got = run();
    } catch (e) {
      got = { ok: "threw", reason: e.message };
    }
    const good = got.ok === ok && got.reason === reason;
    results.push({ name, good });
    console.log(
      `${good ? "PASS" : "FAIL"} ${name}: ${got.ok === "threw" ? "threw" : got.ok ? "green" : "red"}${got.reason ? ` (${got.reason})` : ""}`,
    );
  }
  return results.every((r) => r.good);
}

const N1_EXPECTED_RED = [3, 4, 7, 8, 11, 12, 13, 14, 16];
const N2_EXPECTED_RED = [4, 8, 12, 13, 14, 16];

function selfTest(ctx) {
  let good = comparatorCases();
  const mutations = [
    ["N1 native opacity 0.6 -> 0.61", (t) => t.replace("opacity = 0.6\n", "opacity = 0.61\n"), N1_EXPECTED_RED],
    ["N2 native width 1080 -> 1079", (t) => t.replace("width = 1080\n", "width = 1079\n"), N2_EXPECTED_RED],
  ];
  for (const [name, mutateNative, expected] of mutations) {
    const table = runTable(ctx, TABLE, mutateNative);
    const red = table.rows.filter((r) => r.fail.length > 0).map((r) => r.row.n);
    const same = JSON.stringify(red) === JSON.stringify(expected);
    console.log(`${same ? "PASS" : "FAIL"} ${name}: red rows {${red}}, expected {${expected}}`);
    if (!same) {
      good = false;
      console.log(`     kept ${table.dir}`);
      report(table);
    } else {
      rmSync(table.dir, { recursive: true, force: true });
    }
  }
  if (!orientationSelfTest(ctx)) good = false;
  return good;
}

// ── The alpha rule (a pad with a transparent fill) ───────────────────────────
//
// Not rows of TABLE (the editor's recipe.js has no `pad`, so the text is written here), so
// ROW_COUNT and the self-test's expected-red lists do not move.
//   A. `transform(..., "png")` and native `apply -o A.png` both succeed and decode to the
//      same RGBA bytes. Padded pixels are copied, so the match is exact, not within 1/255.
//   B. `transform(..., "jpeg")` throws the refusal line, and native `apply -o B.jpg` exits 4
//      with that line after `error: `. JPEG has no alpha channel to hold the fill.
const ALPHA_RECIPE = 'version = "2"\n[[step]]\nop = "pad"\nborder = 0.05\ncolor = "transparent"\n';
const ALPHA_JPEG_LINE =
  "recipe step 0 ('pad'): 'color' = \"transparent\" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque";

function alphaRows({ transform, bin, python }) {
  const failures = [];
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-alpha-"));
  const inputPath = join(FIXTURES, INPUTS.W);
  const inputBytes = readFileSync(inputPath);
  const recipeFile = join(dir, "alpha.toml");
  writeFileSync(recipeFile, ALPHA_RECIPE);

  // Row A.
  const wasmOut = join(dir, "A.wasm.png");
  const nativeOut = join(dir, "A.native.png");
  try {
    writeFileSync(wasmOut, transform(inputBytes, ALPHA_RECIPE, "png"));
  } catch (e) {
    failures.push(`alpha-A: transform to png threw: ${e.message}`);
  }
  const apply = spawnSync(bin, ["apply", "--recipe", recipeFile, inputPath, "-o", nativeOut, "-y"], {
    encoding: "utf8",
  });
  if (apply.error) fatal(`native binary could not run: ${apply.error.message}`);
  if (apply.status !== 0) failures.push(`alpha-A: native apply exited ${apply.status}: ${apply.stderr.trim()}`);
  if (existsSync(wasmOut) && existsSync(nativeOut)) {
    const dims = decode(python, [wasmOut, nativeOut]);
    const [w, n] = [image(dims, wasmOut), image(dims, nativeOut)];
    if (w.width !== n.width || w.height !== n.height) {
      failures.push(`alpha-A: sizes differ, wasm ${w.width}x${w.height}, native ${n.width}x${n.height}`);
    } else if (Buffer.compare(w.rgba, n.rgba) !== 0) {
      failures.push("alpha-A: wasm and native RGBA bytes differ");
    }
    // Not vacuous: the corner is fill, so its alpha is 0 on both sides.
    for (const [side, img] of [["wasm", w], ["native", n]]) {
      if (img.rgba[3] !== 0) failures.push(`alpha-A: the ${side} corner alpha is ${img.rgba[3]}, expected 0`);
    }
  }

  // Row B.
  let thrown = null;
  try {
    transform(inputBytes, ALPHA_RECIPE, "jpeg");
  } catch (e) {
    thrown = e.message;
  }
  if (thrown === null) failures.push("alpha-B: transform to jpeg did not throw");
  else if (thrown !== ALPHA_JPEG_LINE) failures.push(`alpha-B: transform threw ${JSON.stringify(thrown)}`);
  const jpegOut = join(dir, "B.jpg");
  const refused = spawnSync(bin, ["apply", "--recipe", recipeFile, inputPath, "-o", jpegOut, "-y"], {
    encoding: "utf8",
  });
  if (refused.error) fatal(`native binary could not run: ${refused.error.message}`);
  if (refused.status !== 4) failures.push(`alpha-B: native apply exited ${refused.status}, expected 4`);
  if (refused.stderr !== `error: ${ALPHA_JPEG_LINE}\n`) {
    failures.push(`alpha-B: native stderr was ${JSON.stringify(refused.stderr)}`);
  }
  if (existsSync(jpegOut)) failures.push("alpha-B: native wrote a JPEG");

  if (failures.length === 0) rmSync(dir, { recursive: true, force: true });
  return failures;
}

// ── An aspect that follows each image's orientation ─────────────────────────
//
// Not rows of TABLE (recipe.js has no crop), so the recipe text is written here and
// ROW_COUNT and the self-test's expected-red lists for TABLE do not move. Each row runs one
// crop or pad step through `transform` and native `apply`, both to PNG, and passes only if both
// sizes equal the row's literal and the decoded RGBA bytes are equal. Nothing resamples, so the
// match is exact.
//
// Inputs: W 1600x1071, P 1071x1600, S 1071x1071, N 1071x1072 (portrait by one pixel), and R, the
// 1600x1071 JPEG tagged Orientation 6. Neither side bakes the tag, so R is landscape unless an
// `auto-orient` step comes first. Every written aspect is 16:9 or 9:16, so no input is already
// at it and every row cuts or grows.
//
// Sizes are `[width, height]` from an independent fraction model (largest fit, the short side
// rounded half up), not read from the engine.
//   [label parts: input, key, written aspect] -> crop size, pad size
const ORIENTATION_ROWS = [
  // input, auto-orient, key, aspect, crop, pad
  ["W", false, true, "16:9", [1600, 900], [1904, 1071]],
  ["W", false, false, "16:9", [1600, 900], [1904, 1071]],
  ["W", false, true, "9:16", [1600, 900], [1904, 1071]],
  ["P", false, true, "16:9", [900, 1600], [1071, 1904]],
  ["P", false, false, "16:9", [1071, 602], [2844, 1600]],
  ["S", false, true, "16:9", [1071, 602], [1904, 1071]],
  ["S", false, true, "9:16", [602, 1071], [1071, 1904]],
  ["S", false, false, "16:9", [1071, 602], [1904, 1071]],
  ["N", false, true, "16:9", [603, 1072], [1071, 1904]],
  ["N", false, false, "16:9", [1071, 602], [1906, 1072]],
  ["R", true, true, "16:9", [900, 1600], [1071, 1904]],
  ["R", true, false, "16:9", [1071, 602], [2844, 1600]],
  ["R", false, true, "16:9", [1600, 900], [1904, 1071]],
];
const ORIENTATION_ROW_COUNT = 26;

function orientationText(op, autoOrient, key, aspect) {
  let t = 'version = "2"\n';
  if (autoOrient) t += '[[step]]\nop = "auto-orient"\n';
  t += `[[step]]\nop = "${op}"\n`;
  if (op === "pad") t += 'color = "#ffffff"\n';
  t += `aspect = "${aspect}"\n`;
  if (key) t += 'orientation = "match"\n';
  return t;
}

/// The 26 rows. `mutate` rewrites the NATIVE side's recipe text only (the self-test's negative
/// control), so a harness that stopped checking the key would go red. Returns failure strings,
/// each `<label> -- <what>` with the label `O-<op>-<input>-<on|off>-<aspect>`.
function orientationRows({ transform, bin, python }, mutate = (t) => t) {
  const failures = [];
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-orientation-"));
  let ran = 0;
  for (const op of ["crop", "pad"]) {
    for (const [input, autoOrient, key, aspect, cropSize, padSize] of ORIENTATION_ROWS) {
      ran += 1;
      const want = op === "crop" ? cropSize : padSize;
      const tag = input === "R" && autoOrient ? "R+ao" : input;
      const label = `O-${op}-${tag}-${key ? "on" : "off"}-${aspect}`;
      const fail = (what) => failures.push(`${label} -- ${what}`);
      const inputPath = join(FIXTURES, INPUTS[input]);
      const text = orientationText(op, autoOrient, key, aspect);
      const id = `${op}-${tag}-${key ? "on" : "off"}-${aspect.replace(":", "x")}`;
      const wasmOut = join(dir, `${id}.wasm.png`);
      const nativeOut = join(dir, `${id}.native.png`);
      const recipeFile = join(dir, `${id}.toml`);
      writeFileSync(recipeFile, mutate(text));
      try {
        writeFileSync(wasmOut, transform(readFileSync(inputPath), text, "png"));
      } catch (e) {
        fail(`transform threw: ${e.message}`);
        continue;
      }
      const apply = spawnSync(bin, ["apply", "--recipe", recipeFile, inputPath, "-o", nativeOut, "-y"], {
        encoding: "utf8",
      });
      if (apply.error) fatal(`native binary could not run: ${apply.error.message}`);
      if (apply.status !== 0) {
        fail(`native apply exited ${apply.status}: ${apply.stderr.trim()}`);
        continue;
      }
      const dims = decode(python, [wasmOut, nativeOut]);
      const [w, n] = [image(dims, wasmOut), image(dims, nativeOut)];
      if (w.width !== want[0] || w.height !== want[1]) {
        fail(`wasm is ${w.width}x${w.height}, expected ${want[0]}x${want[1]}`);
      }
      if (n.width !== want[0] || n.height !== want[1]) {
        fail(`native is ${n.width}x${n.height}, expected ${want[0]}x${want[1]}`);
      }
      if (w.width === n.width && w.height === n.height && Buffer.compare(w.rgba, n.rgba) !== 0) {
        fail("wasm and native RGBA bytes differ");
      }
    }
  }
  if (ran !== ORIENTATION_ROW_COUNT) failures.push(`O-count -- ${ran} rows ran, expected ${ORIENTATION_ROW_COUNT}`);
  if (failures.length === 0) rmSync(dir, { recursive: true, force: true });
  return failures;
}

// N3: strip the key from the native text. The rows where the key changes the size must go red
// and no others: a square, a landscape already in order, and the stored-landscape R without
// `auto-orient` are the same size either way.
const N3_EXPECTED_RED = [
  "O-crop-W-on-9:16",
  "O-crop-P-on-16:9",
  "O-crop-N-on-16:9",
  "O-crop-R+ao-on-16:9",
  "O-pad-W-on-9:16",
  "O-pad-P-on-16:9",
  "O-pad-N-on-16:9",
  "O-pad-R+ao-on-16:9",
];

function orientationSelfTest(ctx) {
  const failures = orientationRows(ctx, (t) => t.replace('orientation = "match"\n', ""));
  const red = [...new Set(failures.map((f) => f.split(" -- ")[0]))].sort();
  const expected = [...N3_EXPECTED_RED].sort();
  const same = JSON.stringify(red) === JSON.stringify(expected);
  console.log(
    `${same ? "PASS" : "FAIL"} N3 the orientation key stripped: red rows {${red}}, expected {${expected}}`,
  );
  if (!same) for (const f of failures) console.log(`     ${f}`);
  return same;
}

// ── main ─────────────────────────────────────────────────────────────────────

const args = parseArgs(process.argv.slice(2));
const recipes = await loadRecipes();
const problems = checkModules(recipes);
if (problems.length) {
  for (const p of problems) console.error(`FAIL modules: ${p}`);
  process.exit(1);
}
console.log("PASS modules: edit.js and worker.js parse, and import only what recipe.js exports");

const { transform, info } = await loadWasm(resolve(args.pkg));
const bin = resolve(args.bin);
if (!existsSync(bin)) fatal(`native binary missing: ${bin}`);
const ctx = { recipes, transform, info, bin, python: args.python };

if (args.selfTest) {
  process.exit(selfTest(ctx) ? 0 : 1);
}

const table = runTable(ctx, TABLE);
report(table);
const failures = [...tableChecks(table), ...alphaRows(ctx), ...orientationRows(ctx)];
const rowsFailed = table.rows.filter((r) => r.fail.length > 0).length;
for (const f of failures) console.log(`FAIL ${f}`);
// Every failure is counted: row lines, then table-level checks (P1-P5 and the rest), each
// named by the label before its first colon.
const tableNames = failures.map((f) => f.split(/: | -- /)[0]).join(", ");
console.log(
  `${table.rows.length} rows, ${rowsFailed} row failures, ${failures.length} table failure${failures.length === 1 ? "" : "s"}` +
    `${failures.length ? ` (${tableNames})` : ""}, K=${K} (cap ${K_CAP})`,
);
// Informational, not an assertion: raising K up to the cap turns nothing red, so this
// line is how a K raised without new evidence shows in review.
const resampled = table.rows.filter((r) => r.cmp && r.resamples && r.cmp.px !== null);
const largest = Math.max(0, ...resampled.map((r) => r.cmp.px));
console.log(`largest differing-pixel count on a resampling row: ${largest} (K = ${K})`);
if (rowsFailed > 0 || failures.length > 0) {
  console.log(`outputs kept in ${table.dir}`);
  process.exit(1);
}
rmSync(table.dir, { recursive: true, force: true });
