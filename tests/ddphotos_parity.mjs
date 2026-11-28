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

/// The lowest PSNR (dB, RGB) a replay may read, per crop form. Measured on the Pillow-made
/// original, the lowest row of each form rounded down to 0.5 dB, with no more headroom: an
/// aspect is centred on each image separately, so its framing differs by up to half a copy
/// pixel by design, and its rows read lower than a box's.
export const PSNR_FLOOR = { aspect: 53.5, box: 59 };

/// The verdict on one replay row: the page's output from the copy against the CLI's from the
/// original. `form` is `aspect` or `box`, `band` is `integer` for a whole-number factor, `2x`
/// for 2 or more and `under-2x` below. `page` and `cli` are `{ width, height }`. The sides may
/// differ by 1 px (2 px for a box below 2×); then the PSNR must reach the form's floor, and a
/// NaN fails. Returns `{ ok, reason }`; `reason` is `replay-dims` or `psnr`.
export function judgeReplay({ form, band, page, cli, psnr: db }) {
  const bound = form === "box" && band === "under-2x" ? 2 : 1;
  const off = Math.max(Math.abs(page.width - cli.width), Math.abs(page.height - cli.height));
  if (off > bound) return { ok: false, reason: "replay-dims" };
  if (!(db >= PSNR_FLOOR[form])) return { ok: false, reason: "psnr" };
  return { ok: true, reason: "" };
}

/// PSNR in dB over the RGB of the region both images share from the top-left, `Infinity` for
/// equal pixels. `a` and `b` are `{ width, height, rgba }`.
export function psnr(a, b) {
  const w = Math.min(a.width, b.width);
  const h = Math.min(a.height, b.height);
  let sum = 0;
  for (let y = 0; y < h; y++) {
    let ia = y * a.width * 4;
    let ib = y * b.width * 4;
    for (let x = 0; x < w; x++, ia += 4, ib += 4) {
      for (let c = 0; c < 3; c++) {
        const d = a.rgba[ia + c] - b.rgba[ib + c];
        sum += d * d;
      }
    }
  }
  if (sum === 0) return Infinity;
  return 10 * Math.log10((255 * 255) / (sum / (w * h * 3)));
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

// ── the crop rows ────────────────────────────────────────────────────────────
//
// Native against wasm on the same input, as TABLE's rows, with a crop. "pub" is as
// published: the size is the crop's own long edge, so there is no `resize` step and the
// match is exact. Every size is a literal from an independent model of the engine's crop
// (`rect_edges`, `aspect_size` and `resize` mode "max"), never read from recipe.js or
// crustyimg. A row that resizes is within the resize tolerance.
//   [id, input, crop, size, watermark on, recipe, format, output size, watermark [size, capped]]
// crop: `{ preset: [a, b] }`, with `swap` for the other orientation, or `{ box: [x, y, w, h] }`
// in pixels of the upright photo.
const preset = (a, b, swap = false) => ({ preset: [a, b], swap });
const BOX_FREE = { box: [160, 214, 800, 643] };
const CROP_ROWS = [
  // W presets, as published.
  ["W", preset(1, 1), "pub", false, "pixel", "png", [1071, 1071]],
  ["W", preset(4, 5), "pub", false, "pixel", "png", [1339, 1071]],
  ["W", preset(3, 2), "pub", false, "pixel", "png", [1600, 1067]],
  ["W", preset(16, 9), "pub", false, "pixel", "png", [1600, 900]],
  // P presets.
  ["P", preset(1, 1), "pub", false, "pixel", "png", [1071, 1071]],
  ["P", preset(4, 5), "pub", false, "pixel", "png", [1071, 1339]],
  ["P", preset(3, 2), "pub", false, "pixel", "png", [1067, 1600]],
  ["P", preset(16, 9), "pub", false, "pixel", "png", [900, 1600]],
  // N presets.
  ["N", preset(1, 1), "pub", false, "pixel", "png", [1071, 1071]],
  ["N", preset(4, 5), "pub", false, "pixel", "png", [858, 1072]],
  ["N", preset(3, 2), "pub", false, "pixel", "png", [715, 1072]],
  ["N", preset(16, 9), "pub", false, "pixel", "png", [603, 1072]],
  // R presets (upright 1071×1600).
  ["R", preset(1, 1), "pub", false, "pixel", "png", [1071, 1071]],
  ["R", preset(4, 5), "pub", false, "pixel", "png", [1071, 1339]],
  ["R", preset(3, 2), "pub", false, "pixel", "png", [1067, 1600]],
  ["R", preset(16, 9), "pub", false, "pixel", "png", [900, 1600]],
  // S presets, both written orders (a square follows nothing).
  ["S", preset(1, 1), "pub", false, "pixel", "png", [1071, 1071]],
  ["S", preset(4, 5), "pub", false, "pixel", "png", [857, 1071]],
  ["S", preset(5, 4), "pub", false, "pixel", "png", [1071, 857]],
  ["S", preset(3, 2), "pub", false, "pixel", "png", [1071, 714]],
  ["S", preset(2, 3), "pub", false, "pixel", "png", [714, 1071]],
  ["S", preset(16, 9), "pub", false, "pixel", "png", [1071, 602]],
  ["S", preset(9, 16), "pub", false, "pixel", "png", [602, 1071]],
  // Swapped presets.
  ["W", preset(3, 2, true), "pub", false, "pixel", "png", [714, 1071]],
  ["P", preset(3, 2, true), "pub", false, "pixel", "png", [1071, 714]],
  // Boxes.
  ["W", BOX_FREE, "600", false, "pixel", "png", [600, 482]],
  ["R", { box: [107, 320, 536, 960] }, "800", false, "pixel", "png", [447, 800]],
  ["W", { box: [0, 0, 1600, 1067] }, "pub", false, "pixel", "png", [1600, 1067]],
  ["W", { box: [600, 0, 400, 1071] }, "pub", true, "pixel", "png", [400, 1071], [40, true]],
  ["R", { box: [348, 0, 375, 1600] }, "pub", true, "pixel", "png", [375, 1600], [37, true]],
  // The free box through the other recipes and formats.
  ["W", BOX_FREE, "600", false, "cli", "png", [600, 482]],
  ["W", BOX_FREE, "pub", false, "preview", "jpeg", [800, 643]],
  ["W", BOX_FREE, "600", false, "pixel", "webp", [600, 482]],
  ["W", BOX_FREE, "600", false, "pixel", "avif", [600, 482]],
  // A following preset through the CLI recipe, with the watermark.
  ["P", preset(3, 2), "1080", true, "cli", "png", [720, 1080], [43, false]],
  // A placed box inside the upright photo's top-left 1071x1071, resized and marked. Read against
  // the stored 1600x1071 it would be a different crop, with a different size and mark.
  ["R", { box: [100, 50, 800, 600] }, "400", true, "pixel", "png", [400, 300], [16, false]],
];
const CROP_TABLE = CROP_ROWS.map(([input, crop, size, wm, recipe, format, want, mark], i) => ({
  n: `C${i + 1}`,
  input,
  crop,
  form: { size: size === "pub" ? String(Infinity) : size, wmOn: wm },
  recipe,
  format,
  size: want,
  mark: mark ?? null,
  family: "crop",
  id: `${input}/${cropLabel(crop)}/${size}/${recipe}/${format}`,
}));

function cropLabel(crop) {
  if (crop.box) return `box(${crop.box})`;
  return `${crop.preset.join(":")}${crop.swap ? "-swapped" : ""}`;
}

const CROP_ROW_COUNT = 36;
// Every crop row but these has no resize step: C26, C27, C31, C33, C34, C35 and C36.
const CROP_NO_RESAMPLE_ROWS = CROP_TABLE.map((r) => r.n).filter(
  (n) => !["C26", "C27", "C31", "C33", "C34", "C35", "C36"].includes(n),
);

const INPUTS = {
  W: "photo.webp",
  R: "photo_rot.jpg",
  P: "photo_portrait.webp",
  S: "photo_square.webp",
  N: "photo_near_square.webp",
  // 64×1600: so narrow that a watermark may not fit it, with no crop at all.
  T: "photo_thin.webp",
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

// The crop rows' golden texts. A box is four fractions of the photo, each the exact pixel
// edge; a preset is its aspect alone on a square photo, and with the key on a photo that
// has an orientation to follow. Neither writes a gravity.
const GOLDEN_CROP_BOX = `version = "2"

[[step]]
op = "auto-orient"

[[step]]
op = "crop"
x = 0.1
y = 0.19981325863678806
width = 0.5
height = 0.6003734827264239

[[step]]
op = "resize"
mode = "max"
width = 600
`;
const GOLDEN_CROP_SQUARE = `version = "2"

[[step]]
op = "auto-orient"

[[step]]
op = "crop"
aspect = "3:2"
`;
const GOLDEN_CROP_CLI = `version = "2"
format = "png"

[[step]]
op = "auto-orient"

[[step]]
op = "crop"
aspect = "3:2"
orientation = "match"

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
    "FIT_HEADROOM",
    "fitWatermark",
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
    "toCopyDelta",
    "moveBox",
    "resizeBox",
    "MIN_CROP",
    "cropReducer",
    "sizeLine",
    "clamp",
    "sameBox",
    "toBox",
    "HANDLE_EDGES",
  ];
  // What the model and the loader need from recipe.js, and the one line the worker's
  // `load` is: the harness runs `loadPhoto` itself, so the worker must run the same one.
  for (const name of [
    "aspectSize",
    "followOrientation",
    "presetCrop",
    "swapCrop",
    "cropBox",
    "loadPhoto",
    "toCopyDelta",
    "moveBox",
    "resizeBox",
    "MIN_CROP",
    "cropReducer",
    "sizeLine",
    "FIT_HEADROOM",
    "fitWatermark",
  ]) {
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
    // The plan runs in the worker, with the engine's own check, and the page asks the worker
    // for it: edit.js must not plan on its own thread with no check.
    const planLine = "return { edit: planEdit(shown, req.form, req.crop, watermarkRefusal) };";
    if (file === "worker.js" && !text.includes(planLine)) {
      problems.push(`worker.js's plan is not ${planLine}`);
    }
    if (file === "edit.js") {
      for (const m of imports) {
        if (/\bplanEdit\b/.test(m[1])) problems.push("edit.js imports planEdit; the worker plans the edit");
      }
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

/// Load the slim wasm package and return its `transform`, `info` and `watermarkRefusal`.
async function loadWasm(pkg) {
  const js = join(pkg, "crustyimg.js");
  const wasm = join(pkg, "crustyimg_bg.wasm");
  if (!existsSync(js) || !existsSync(wasm)) {
    return fatal(`wasm package missing or empty: expected ${js} and ${wasm}`);
  }
  try {
    const mod = await import(pathToFileURL(js).href);
    mod.initSync({ module: readFileSync(wasm) });
    // `watermarkRefusal` is the engine's own fit check, the one the page's worker passes to
    // `planEdit`. It is never replaced by a function written here.
    if (typeof mod.watermarkRefusal !== "function") {
      return fatal("the wasm package has no watermarkRefusal export");
    }
    return { transform: mod.transform, info: mod.info, refuses: mod.watermarkRefusal };
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
  const edit = recipes.planEdit(photo, { ...FORM, ...row.form }, crop, ctx.refuses);
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

/// Checks about the crop rows as a whole. Returns failure messages.
function cropChecks({ rows: allRows }) {
  const failures = [];
  const rows = allRows.filter((r) => r.row.family === "crop");
  if (rows.length !== CROP_ROW_COUNT) failures.push(`crop-count: ${rows.length} rows ran, expected ${CROP_ROW_COUNT}`);
  const exact = rows.filter((r) => !r.resamples).map((r) => r.row.n);
  if (JSON.stringify(exact) !== JSON.stringify(CROP_NO_RESAMPLE_ROWS)) {
    failures.push(`crop-resize-set: the rows without a resize step are {${exact}}, expected {${CROP_NO_RESAMPLE_ROWS}}`);
  }
  const byN = (n) => rows.find((r) => r.row.n === n);
  const [c20, c26, c35] = ["C20", "C26", "C35"].map(byN);
  if (c26 && c26.text !== GOLDEN_CROP_BOX) failures.push("golden-crop-box: C26's recipe text moved");
  if (c20 && c20.text !== GOLDEN_CROP_SQUARE) failures.push("golden-crop-square: C20's recipe text moved");
  if (c35 && c35.text !== GOLDEN_CROP_CLI) failures.push("golden-crop-cli: C35's recipe text moved");
  return failures;
}

// ── the replay rows ──────────────────────────────────────────────────────────
//
// The page's recipe on the published copy (wasm) against the CLI recipe on an original that
// is `factor` times larger (native), the original made by Pillow (tests/ddphotos_upscale.py).
// The sizes are literals and equal on both sides in every row; the framing is checked by the
// PSNR floor of the row's form.
//   [input, crop, size, watermark on, output size, watermark [size, capped], factor]
// The form (aspect or box) is read off the crop; the band off the factor.
// The texts the fit and replay rows draw. The widths in the literals are the engine's, from native
// runs. T67's mark at size 1 is 32 px wide ("Not" for "NOT" would make it 31), so the text is part
// of the literal. T120 is the page's longest text (its `maxlength`).
const T67 = "© 2026 Jane Doe. All rights reserved. NOT for reuse without consent";
const T120 =
  "© 2026 Jane Doe. All rights reserved. This photograph may not be copied, shared or reused in any way without my consent.";
const SMALL = "0.025";
const MEDIUM = "0.04";
const LARGE = "0.06";
const WM_SMALL_CENTRE = { wmText: T67, wmSize: SMALL, wmGravity: "center" };
const WM_120_SMALL_CENTRE = { wmText: T120, wmSize: SMALL, wmGravity: "center" };
const REFUSED_32X2 = "the 32×2 px mark does not fit inside the 31×400 px image";

const REPLAY_ROWS = [
  ["W", preset(1, 1), "pub", false, [1071, 1071], null, 4],
  ["W", preset(4, 5), "pub", false, [1339, 1071], null, 4],
  ["W", preset(3, 2), "pub", false, [1600, 1067], null, 4],
  ["W", preset(16, 9), "pub", false, [1600, 900], null, 4],
  ["P", preset(3, 2), "pub", false, [1067, 1600], null, 4],
  ["N", preset(3, 2), "pub", false, [715, 1072], null, 4],
  ["R", preset(3, 2), "pub", false, [1067, 1600], null, 4],
  ["S", preset(3, 2), "pub", false, [1071, 714], null, 4],
  ["S", preset(2, 3), "pub", false, [714, 1071], null, 4],
  ["W", preset(3, 2, true), "pub", false, [714, 1071], null, 4],
  ["W", BOX_FREE, "600", false, [600, 482], null, 4],
  ["R", { box: [107, 320, 536, 960] }, "800", false, [447, 800], null, 4],
  ["W", { box: [0, 0, 1600, 1067] }, "pub", false, [1600, 1067], null, 4],
  ["W", { box: [600, 0, 400, 1071] }, "pub", true, [400, 1071], [40, true], 4],
  ["R", { box: [348, 0, 375, 1600] }, "pub", true, [375, 1600], [37, true], 4],
  // A factor that is not a whole number: a 6000×4016 original.
  ["W", BOX_FREE, "600", false, [600, 482], null, 3.75],
  ["W", preset(3, 2), "pub", false, [1600, 1067], null, 3.75],
  // The watermark's fit on a small crop, at a fractional original. The copy takes a size-1 mark
  // that the CLI refuses on the original, 1 px smaller, so the page must not offer it: the plan
  // is "blocked". The control is today's recipe on the original: [mark size, the engine's refusal].
  ["W", { box: [333, 0, 77, 976] }, "400", true, [32, 400], "blocked", 3.75, WM_SMALL_CENTRE, [1, REFUSED_32X2]],
  ["P", { box: [0, 0, 126, 1600] }, "400", true, [32, 400], "blocked", 3.75, WM_SMALL_CENTRE, [1, REFUSED_32X2]],
  // A size the CLI takes, because the page chose it with the headroom.
  ["W", { box: [0, 0, 640, 96] }, "400", true, [400, 60], [5, true], 3.75, WM_120_SMALL_CENTRE, null],
].map(([input, crop, size, wm, want, mark, factor, over = {}, control = null], i) => ({
  n: `R${i + 1}`,
  input,
  crop,
  form: { size: size === "pub" ? String(Infinity) : size, wmOn: wm, ...over },
  control,
  size: want,
  mark,
  factor,
  kind: crop.box ? "box" : "aspect",
  band: Number.isInteger(factor) ? "integer" : factor >= 2 ? "2x" : "under-2x",
  id: `${input}/${cropLabel(crop)}/${size}/${factor}x`,
}));
const REPLAY_ROW_COUNT = 20;

/// The replay originals for one run, made once per input and factor in `dir`. `shift` moves
/// their content left by round(factor) px (the framing-shift control).
function replayOriginal(ctx, dir, row, shift, made) {
  const key = `${row.input}@${row.factor}`;
  if (!made.has(key)) {
    const dst = join(dir, `original-${row.input}-${row.factor}.${row.input === "R" ? "jpg" : "png"}`);
    const args = ["-I", join(HERE, "ddphotos_upscale.py"), join(FIXTURES, INPUTS[row.input]), dst, String(row.factor)];
    if (shift) args.push(String(Math.round(row.factor)));
    const run = spawnSync(ctx.python, args, { encoding: "utf8" });
    if (run.error || run.status !== 0) {
      return fatal(`the replay original could not be made: ${run.error?.message ?? run.stderr.trim()}`);
    }
    made.set(key, dst);
  }
  return made.get(key);
}

/// The replay rows. `shift` makes the originals' content move left (the self-test's control).
/// Returns `{ dir, rows }`; a row's `reasons` list names every limb that failed, in order:
/// `threw`, `page-size`, `cli-size`, `plan`, `mark`, then `replay-dims` or `psnr`.
function replayRows(ctx, { shift = false } = {}) {
  const { recipes, transform, bin, python } = ctx;
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-replay-"));
  const made = new Map();
  const rows = [];
  const files = [];
  for (const row of REPLAY_ROWS) {
    const result = { row, reasons: [], notes: [], error: null };
    try {
      // Inside the row's try, so a recipe.js that throws fails this row and not the run.
      const photo = photoFor(ctx, row.input);
      const crop = cropFor(recipes, row.crop, photo);
      const edit = recipes.planEdit(photo, { ...FORM, ...row.form }, crop, ctx.refuses);
      const original = replayOriginal(ctx, dir, row, shift, made);
      if (row.mark === "blocked") {
        // Nothing is drawn: the plan is blocked, and the recipe the page would have offered at the
        // size it asks for today is refused by the CLI on the original (the control).
        Object.assign(result, { edit, blocked: true });
        const got = markOf(edit);
        if (got !== "blocked") {
          result.reasons.push("mark");
          result.notes.push(`planEdit's watermark is ${got}, expected blocked`);
        }
        const [size, literal] = row.control;
        const forced = forcedEdit(recipes, photo, { ...FORM, ...row.form }, crop, size);
        const why = nativeRefusal(bin, dir, row.n, recipes.cliRecipe(forced), original, literal);
        if (why) {
          result.reasons.push("control");
          result.notes.push(`control: ${why}`);
        }
        rows.push(result);
        continue;
      }
      const pageOut = join(dir, `${row.n}.page.png`);
      const cliOut = join(dir, `${row.n}.cli.png`);
      const recipeFile = join(dir, `${row.n}.toml`);
      writeFileSync(pageOut, transform(readFileSync(join(FIXTURES, INPUTS[row.input])), recipes.pixelRecipe(edit), "png"));
      writeFileSync(recipeFile, recipes.cliRecipe(edit));
      const run = spawnSync(bin, ["apply", "--recipe", recipeFile, original, "-o", cliOut, "-y", "-Q"], {
        encoding: "utf8",
      });
      if (run.error) fatal(`native binary could not run: ${run.error.message}`);
      if (run.status !== 0) throw new Error(`native apply exited ${run.status}: ${run.stderr.trim()}`);
      files.push(pageOut, cliOut);
      Object.assign(result, { edit, pageOut, cliOut });
    } catch (e) {
      result.error = e.message;
      result.reasons.push("threw");
    }
    rows.push(result);
  }

  const dims = decode(python, files);
  for (const r of rows) {
    if (r.error || r.blocked) continue;
    const page = image(dims, r.pageOut);
    const cli = image(dims, r.cliOut);
    const want = r.row.size;
    r.page = page;
    r.cli = cli;
    r.psnr = psnr(page, cli);
    if (page.width !== want[0] || page.height !== want[1]) {
      r.reasons.push("page-size");
      r.notes.push(`page is ${page.width}x${page.height}, expected ${want.join("x")}`);
    }
    if (cli.width !== want[0] || cli.height !== want[1]) {
      r.reasons.push("cli-size");
      r.notes.push(`cli is ${cli.width}x${cli.height}, expected ${want.join("x")}`);
    }
    if (r.edit.outW !== want[0] || r.edit.outH !== want[1]) {
      r.reasons.push("plan");
      r.notes.push(`planEdit predicts ${r.edit.outW}x${r.edit.outH}, expected ${want.join("x")}`);
    }
    if (r.row.mark) {
      const got = markOf(r.edit);
      const exp = `${r.row.mark[0]}${r.row.mark[1] ? " capped" : ""}`;
      if (got !== exp) {
        r.reasons.push("mark");
        r.notes.push(`planEdit's watermark is ${got}, expected ${exp}`);
      }
    }
    const verdict = judgeReplay({ form: r.row.kind, band: r.row.band, page, cli, psnr: r.psnr });
    if (!verdict.ok) {
      r.reasons.push(verdict.reason);
      r.notes.push(`${verdict.reason}: PSNR ${r.psnr.toFixed(2)} dB, floor ${PSNR_FLOOR[r.row.kind]}`);
    }
  }
  return { dir, rows };
}

/// Checks about the replay rows as a whole. Returns failure messages.
function replayChecks({ rows }) {
  const failures = [];
  if (rows.length !== REPLAY_ROW_COUNT) {
    failures.push(`replay-count: ${rows.length} rows ran, expected ${REPLAY_ROW_COUNT}`);
  }
  return failures;
}

function reportReplay({ rows }) {
  for (const r of rows) {
    const verdict = r.reasons.length === 0 ? "PASS" : "FAIL";
    const sizes = r.page
      ? `page ${r.page.width}x${r.page.height} cli ${r.cli.width}x${r.cli.height}`
      : r.blocked
        ? "blocked, nothing drawn"
        : "did not run";
    console.log(`${verdict} ${r.row.n} ${r.row.id} ${r.row.kind}/${r.row.band} ${sizes}`);
    if (r.error) console.log(`     threw: ${r.error}`);
    for (const note of r.notes) console.log(`     ${note}`);
  }
}

// ── the fit rows ─────────────────────────────────────────────────────────────
//
// The watermark's fit on a small or thin crop. The page asks the engine, through the real wasm
// `watermarkRefusal` (`ctx.refuses`), whether each size fits, and takes the largest that does.
// Every expected side, size and refusal below is a literal from a native run of `apply`, never
// read from recipe.js or computed here: both sides of this harness load recipe.js, so only a
// number written here can see a recipe.js that is wrong.
//
//   [id, input, box or null, size, text, watermark size, gravity, output size, mark, control]
// mark: `[size, capped]`, or "blocked" when no size fits. control: null, or `[size, refusal]`,
// the same edit with the mark forced to the size the page asked for before the check existed.
// Natively that must be refused with exactly the refusal, and in wasm `transform` must throw it.
const FIT_ROWS = [
  // Coverage: today's size 6 covers 5.05 % of the 400×60 copy, whatever the position.
  ["F1", "W", [0, 0, 640, 96], "400", T120, SMALL, "center", [400, 60], [5, true],
    [6, "the mark would cover 5.05 % of the 400×60 px image (1213 of 24000 px); the limit is 5 %"]],
  ["F2", "W", [0, 0, 640, 96], "400", T120, SMALL, "southeast", [400, 60], [5, true],
    [6, "the mark would cover 5.05 % of the 400×60 px image (1213 of 24000 px); the limit is 5 %"]],
  // The width plus the margin, on a thin strip, in both orders (the margin is on both axes at a corner).
  ["F3", "P", [0, 0, 64, 1600], "400", WATERMARK_TEXT, MEDIUM, "southeast", [16, 400], "blocked",
    [1, "the 9×2 px mark plus its 8 px margin does not fit inside the 16×400 px image"]],
  ["F4", "W", [0, 0, 1600, 64], "400", WATERMARK_TEXT, MEDIUM, "southeast", [400, 16], [1, true], null],
  // The upright frame is read, not the stored one.
  ["F5", "R", [0, 0, 64, 1600], "400", WATERMARK_TEXT, MEDIUM, "southeast", [16, 400], "blocked",
    [1, "the 9×2 px mark plus its 8 px margin does not fit inside the 16×400 px image"]],
  // The square, in both orders.
  ["F6", "S", [0, 0, 64, 1071], "400", WATERMARK_TEXT, MEDIUM, "southeast", [24, 400], [1, true], null],
  ["F7", "S", [0, 0, 1071, 64], "400", WATERMARK_TEXT, MEDIUM, "southeast", [400, 24], [2, true], null],
  // The centre has no margin, so the same box takes a size-1 mark that F3 cannot.
  ["F8", "P", [0, 0, 64, 1600], "400", WATERMARK_TEXT, MEDIUM, "center", [16, 400], [1, true], null],
  // A mark wider than the copy even at size 1.
  ["F9", "P", [0, 0, 64, 1600], "400", T120, MEDIUM, "center", [16, 400], "blocked",
    [1, "the 56×2 px mark does not fit inside the 16×400 px image"]],
  // No crop: nothing changes.
  ["F10", "W", null, "1600", WATERMARK_TEXT, MEDIUM, "southeast", [1600, 1071], [64, false], null],
  ["F11", "P", null, "400", T120, MEDIUM, "southeast", [268, 400], [4, true], null],
  // An uncropped thin photo, at 400 px and as published.
  ["F12", "T", null, "400", WATERMARK_TEXT, MEDIUM, "southeast", [16, 400], "blocked",
    [1, "the 9×2 px mark plus its 8 px margin does not fit inside the 16×400 px image"]],
  ["F13", "T", null, "1600", WATERMARK_TEXT, MEDIUM, "southeast", [64, 1600], [3, true], null],
  // The height alone, in both orders.
  ["F14", "W", [0, 0, 1600, 20], "1600", WATERMARK_TEXT, LARGE, "south", [1600, 20], "blocked",
    [2, "the 16×3 px mark plus its 32 px margin does not fit inside the 1600×20 px image"]],
  ["F15", "P", [0, 0, 20, 1600], "1600", WATERMARK_TEXT, LARGE, "south", [20, 1600], [1, true], null],
];
const FIT_ROW_COUNT = 15;

const FIT_TABLE = FIT_ROWS.map(([n, input, box, size, text, wmSize, gravity, want, mark, control]) => ({
  n,
  input,
  crop: box ? { box } : null,
  form: { size, wmOn: true, wmText: text, wmSize, wmGravity: gravity },
  recipe: "pixel",
  format: "png",
  size: want,
  mark,
  control,
  family: "fit",
  id: `${input}/${box ? `box(${box})` : "whole"}/${size}/${gravity}`,
}));

/// What `planEdit` made of the watermark, in the rows' own words: `<size>`, `<size> capped`,
/// `blocked`, or `none`.
function markOf(edit) {
  if (edit.blocked) return "blocked";
  const wm = edit.watermark;
  return wm ? `${wm.size}${wm.capped ? " capped" : ""}` : "none";
}

/// The edit the page would have planned before the engine's check: the check accepts every size,
/// so the plan keeps the ceiling, and `size` is then forced to the control's literal. A control
/// whose literal is not the ceiling is not a control for today's behaviour: it throws.
function forcedEdit(recipes, photo, form, crop, size) {
  const edit = recipes.planEdit(photo, form, crop, () => undefined);
  if (edit.watermark.size !== size) {
    throw new Error(`the control's size ${size} is not the size asked for today, ${edit.watermark.size}`);
  }
  return edit;
}

/// Run `recipeText` natively on `input` and compare the failure with the engine's refusal. Returns
/// null when `apply` stopped with exactly `error: watermark refused: <literal>`, else the reason.
function nativeRefusal(bin, dir, name, recipeText, input, literal) {
  const file = join(dir, `${name}.toml`);
  writeFileSync(file, recipeText);
  const run = spawnSync(bin, ["apply", "--recipe", file, input, "-o", join(dir, `${name}.control.png`), "-y", "-Q"], {
    encoding: "utf8",
  });
  if (run.error) fatal(`native binary could not run: ${run.error.message}`);
  const want = `error: watermark refused: ${literal}`;
  const got = run.stderr.trim();
  if (run.status !== 0 && got === want) return null;
  return `native apply exited ${run.status} with ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`;
}

/// The fit rows. Each one plans with the engine's own check and asserts the output's sides and
/// the mark as literals. A row that is not blocked runs through wasm and native `apply`, as the
/// table rows do. A blocked row must make `pixelRecipe` throw. A row with a control runs it.
/// Returns `{ dir, ran, rows, failures }`, where a failure reads `fit <id> -- <what>`.
function fitRows(ctx) {
  const { recipes, transform, bin } = ctx;
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-fit-"));
  const drawn = runTable(
    ctx,
    FIT_TABLE.filter((r) => r.mark !== "blocked"),
  );
  const rows = [];
  for (const row of FIT_TABLE) {
    const fail = [];
    const ran = { ok: false };
    try {
      const photo = photoFor(ctx, row.input);
      const form = { ...FORM, ...row.form };
      const crop = cropFor(recipes, row.crop, photo);
      const edit = recipes.planEdit(photo, form, crop, ctx.refuses);
      ran.ok = true;
      const exp = row.mark === "blocked" ? "blocked" : `${row.mark[0]}${row.mark[1] ? " capped" : ""}`;
      if (edit.outW !== row.size[0] || edit.outH !== row.size[1]) {
        fail.push(`planEdit predicts ${edit.outW}x${edit.outH}, expected ${row.size.join("x")}`);
      }
      if (markOf(edit) !== exp) fail.push(`planEdit's watermark is ${markOf(edit)}, expected ${exp}`);
      if (row.mark === "blocked") {
        if (edit.blocked) {
          try {
            recipes.pixelRecipe(edit);
            fail.push("pixelRecipe made a recipe from a blocked edit");
          } catch {
            /* the refusal the row wants */
          }
        } else {
          // The plan offers a mark the engine refuses: run it, so the engine's own refusal is quoted.
          try {
            transform(readFileSync(join(FIXTURES, INPUTS[row.input])), recipes.pixelRecipe(edit), "png");
          } catch (e) {
            fail.push(`threw: ${e.message}`);
          }
        }
      } else {
        // The table run covers the engine's side: both targets succeed at the literal size.
        const r = drawn.rows.find((d) => d.row.n === row.n);
        for (const f of r.fail) if (!f.startsWith("planEdit's watermark is")) fail.push(f);
      }
      if (row.control) {
        const [size, literal] = row.control;
        const forced = forcedEdit(recipes, photo, form, crop, size);
        const text = recipes.pixelRecipe(forced);
        const input = join(FIXTURES, INPUTS[row.input]);
        const why = nativeRefusal(bin, dir, row.n, text, input, literal);
        if (why) fail.push(`control: ${why}`);
        try {
          transform(readFileSync(input), text, "png");
          fail.push("control: wasm transform accepted the refused mark");
        } catch (e) {
          if (!e.message.includes(literal)) {
            fail.push(`control: wasm threw ${JSON.stringify(e.message)}, expected it to contain ${JSON.stringify(literal)}`);
          }
        }
      }
    } catch (e) {
      fail.push(`threw: ${e.message}`);
    }
    rows.push({ row, fail });
  }
  rmSync(drawn.dir, { recursive: true, force: true });
  const failures = rows.flatMap((r) => r.fail.map((f) => `fit ${r.row.n} -- ${f}`));
  if (rows.length !== FIT_ROW_COUNT) failures.push(`fit-count: ${rows.length} rows ran, expected ${FIT_ROW_COUNT}`);
  return { dir, ran: rows.length, rows, failures };
}

function reportFit({ rows }) {
  for (const r of rows) {
    console.log(`${r.fail.length === 0 ? "PASS" : "FAIL"} fit ${r.row.n} ${r.row.id} -> ${r.row.size.join("x")} ${
      r.row.mark === "blocked" ? "blocked" : `${r.row.mark[0]}${r.row.mark[1] ? " capped" : ""}`
    }`);
  }
}

// ── the model's own checks ───────────────────────────────────────────────────
//
// Pure checks against literals, no binary: a recipe.js mutant moves native and wasm together,
// so only a number written here can see it.

// [a, b, width, height, expected width, expected height]: the largest a:b box, the short side
// rounded half up (856.8 is 857, and floors to 856).
const ASPECT_SIZES = [
  [3, 2, 1600, 1071, 1600, 1067],
  [2, 3, 1071, 1600, 1067, 1600],
  [3, 2, 1071, 1071, 1071, 714],
  [2, 3, 1071, 1072, 715, 1072],
  [4, 5, 1071, 1071, 857, 1071],
  [4, 5, 1071, 1072, 858, 1072],
  [9, 16, 1600, 1071, 602, 1071],
  [16, 9, 3, 1, 2, 1],
];

// [a, b, width, height, expected a, expected b]: the engine's own table of how an aspect is
// ordered by the image's shape (src/operation/mod.rs, follow_orientation's test).
const FOLLOW_ROWS = [
  [16, 9, 64, 48, 16, 9],
  [9, 16, 64, 48, 16, 9],
  [16, 9, 48, 64, 9, 16],
  [9, 16, 48, 64, 9, 16],
  // A square keeps the aspect as written, either order.
  [16, 9, 48, 48, 16, 9],
  [9, 16, 48, 48, 9, 16],
  // One pixel decides: no tolerance.
  [16, 9, 48, 49, 9, 16],
  [9, 16, 49, 48, 16, 9],
  [1, 1, 64, 48, 1, 1],
  [1, 1, 48, 64, 1, 1],
  [1, 1, 48, 48, 1, 1],
];

const FRACTION_SIDES = [1071, 1600];
// Every (x, width) on both sides: side · (side + 1) / 2 boxes each.
const FRACTION_BOX_COUNT = (1071 * 1072) / 2 + (1600 * 1601) / 2;

function modelChecks(recipes) {
  const failures = [];
  for (const [a, b, w, h, ew, eh] of ASPECT_SIZES) {
    const got = recipes.aspectSize(a, b, w, h);
    if (got[0] !== ew || got[1] !== eh) {
      failures.push(`model-aspect: ${a}:${b} on ${w}x${h} is ${got.join("x")}, expected ${ew}x${eh}`);
    }
  }
  for (const [a, b, w, h, ea, eb] of FOLLOW_ROWS) {
    const got = recipes.followOrientation(a, b, w, h);
    if (got[0] !== ea || got[1] !== eb) {
      failures.push(`model-follow: ${a}:${b} on ${w}x${h} is ${got.join(":")}, expected ${ea}:${eb}`);
    }
  }
  failures.push(...fractionChecks(recipes));
  failures.push(...minimumSideChecks(recipes));
  failures.push(...engineCheckChecks(recipes));
  return failures;
}

/// With the watermark on, `planEdit` must be given the engine's check, and fails loudly when it
/// is not; with the watermark off it needs none.
function engineCheckChecks(recipes) {
  const failures = [];
  const label = "planEdit without the engine's check";
  const photo = { width: 1600, height: 1071 };
  try {
    recipes.planEdit(photo, { ...FORM, size: "1600", wmOn: true });
    failures.push(`${label} -- planEdit with the watermark on and no check did not throw`);
  } catch (e) {
    if (e.message !== "planEdit needs the engine's watermark check") {
      failures.push(`${label} -- threw ${JSON.stringify(e.message)}`);
    }
  }
  try {
    const edit = recipes.planEdit(photo, { ...FORM, size: "1600", wmOn: false });
    if (edit.watermark !== null || edit.outW !== 1600 || edit.outH !== 1071) {
      failures.push(`${label} -- the watermark-off plan is not 1600x1071 with no mark`);
    }
  } catch (e) {
    failures.push(`${label} -- the watermark-off plan threw ${JSON.stringify(e.message)}`);
  }
  return failures;
}

// `resize` mode "max" keeps each side at least 1 px, so a box thinner than the scale predicts
// a 1 px side, not 0. Pure literals, worked out by hand from the engine's rule (each side is
// its length times min(size / long side, 1), rounded half up, at least 1):
//   1460 x 1 at 400:  scale 0.27397, so 400 x 0.27 -> 400 x 1
//   2 x 1500 at 300:  scale 0.2,     so 0.4 x 300  -> 1 x 300
const MINIMUM_SIDE_ROWS = [
  [{ width: 1600, height: 1071 }, { x: 93, y: 131, w: 1460, h: 1 }, 400, [400, 1]],
  [{ width: 1071, height: 1600 }, { x: 5, y: 100, w: 2, h: 1500 }, 300, [1, 300]],
];

function minimumSideChecks(recipes) {
  const failures = [];
  for (const [photo, box, size, [ew, eh]] of MINIMUM_SIDE_ROWS) {
    const edit = recipes.planEdit(photo, { ...FORM, size: String(size), wmOn: false }, { box });
    if (edit.outW !== ew || edit.outH !== eh) {
      failures.push(
        `model-min: box ${box.w}x${box.h} at ${size} is ${edit.outW}x${edit.outH}, expected ${ew}x${eh}`,
      );
    }
  }
  return failures;
}

/// `planEdit`'s crop text for every box on the sides, read back the way the engine reads it
/// (each edge rounded on its own, in f64): the same four edges, and the box inside the photo.
function fractionChecks(recipes) {
  const failures = [];
  const form = { ...FORM, size: String(Infinity), wmOn: false };
  let count = 0;
  let wrongEdges = 0;
  let outside = 0;
  let example = "";
  for (const side of FRACTION_SIDES) {
    const photo = { width: side, height: side };
    for (let x = 0; x < side; x++) {
      for (let w = 1; x + w <= side; w++) {
        const text = recipes.planEdit(photo, form, { box: { x, y: 0, w, h: side } }).crop;
        const [fx, , fw] = text.split("\n").map((line) => Number(line.split(" = ")[1]));
        count += 1;
        if (Math.round(fx * side) !== x || Math.round((fx + fw) * side) !== x + w) {
          wrongEdges += 1;
          example ||= `${side}: x ${x} w ${w}`;
        }
        if (fx + fw > 1) {
          outside += 1;
          example ||= `${side}: x ${x} w ${w}`;
        }
      }
    }
  }
  if (count !== FRACTION_BOX_COUNT) {
    failures.push(`model-fractions: ${count} boxes checked, expected ${FRACTION_BOX_COUNT}`);
  }
  if (wrongEdges || outside) {
    failures.push(
      `model-fractions: ${wrongEdges} boxes read back to other edges and ${outside} run past the photo (first: ${example})`,
    );
  }
  return failures;
}

/// A preset's box is where the engine puts the aspect: the preset's recipe and a box at
/// `cropBox`'s numbers give the same pixels. The margins are odd, so a centre rounded the
/// wrong way moves the frame by a pixel.
function placementChecks(ctx) {
  const failures = [];
  const { recipes, transform } = ctx;
  const form = { ...FORM, size: String(Infinity), wmOn: false };
  for (const [input, a, b] of [["W", 1, 1], ["W", 4, 5], ["S", 2, 3]]) {
    const label = `placement: ${input} ${a}:${b}`;
    try {
      const photo = photoFor(ctx, input);
      const crop = recipes.presetCrop(a, b, photo);
      const bytes = readFileSync(join(FIXTURES, INPUTS[input]));
      const asPreset = transform(bytes, recipes.pixelRecipe(recipes.planEdit(photo, form, crop)), "png");
      const box = { box: recipes.cropBox(crop, photo) };
      const asBox = transform(bytes, recipes.pixelRecipe(recipes.planEdit(photo, form, box)), "png");
      if (Buffer.compare(Buffer.from(asPreset), Buffer.from(asBox)) !== 0) {
        failures.push(`${label} -- the box at cropBox is not where the aspect is placed`);
      }
    } catch (e) {
      failures.push(`${label} -- threw: ${e.message}`);
    }
  }
  return failures;
}

// ── the crop mode's state and box functions ──────────────────────────────────

// Every expected value below is a literal, not read from recipe.js: both sides of the page
// harness load that file, so a function that is wrong there would agree with itself.

// The crop-mode rows that run: 118 reducer rows, 31 geometry rows and 8 size-line rows.
const MODE_ROW_COUNT = 157;

// The form the crop-mode rows plan with: the harness's defaults, as published, no watermark.
const FORM_NO_WM = { ...FORM, size: "", wmOn: false };

// Reducer rows: [id, photo, actions, crop, preset, drag, same state, crop step text].
// Actions: pA:B a preset, sw swap, ds:H drag-start on handle H, dm:X,Y drag-move, de drag-end,
// dn done, km:X,Y key-move, kr:X,Y key-resize, rs reset. The columns after the actions are what
// the last action leaves: the crop (none, `aspect a:b`, `aspect a:b follow` or `box x,y,w,h`),
// the preset button, the drag (`handle@x,y,w,h`), whether it returned the same state object,
// and the crop step's text for the form above (null for no crop step).
const MODE_ROWS = [
  ["X1W","W","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X1P","P","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X1S","S","p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X1R","R","p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X2W","W","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X2P","P","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X2S","S","p2:3","aspect 2:3","2:3",null,false,"aspect = \"2:3\"\n"],
  ["X2R","R","p2:3","aspect 2:3 follow","2:3",null,false,"aspect = \"2:3\"\norientation = \"match\"\n"],
  ["X3W","W","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3P","P","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3S","S","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X3R","R","p1:1","aspect 1:1","1:1",null,false,"aspect = \"1:1\"\n"],
  ["X4W","W","p3:2 sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X4P","P","p3:2 sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X4S","S","p3:2 sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X4R","R","p3:2 sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X5W","W","p2:3 sw","aspect 2:3","2:3",null,false,"aspect = \"2:3\"\n"],
  ["X5P","P","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X5S","S","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X5R","R","p2:3 sw","aspect 3:2","2:3",null,false,"aspect = \"3:2\"\n"],
  ["X6W","W","p3:2 sw sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X6P","P","p3:2 sw sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X6S","S","p3:2 sw sw","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X6R","R","p3:2 sw sw","aspect 2:3","3:2",null,false,"aspect = \"2:3\"\n"],
  ["X7W","W","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X7P","P","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X7S","S","p3:2 sw p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X7R","R","p3:2 sw p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X8W","W","sw","none",null,null,true,null],
  ["X8P","P","sw","none",null,null,true,null],
  ["X8S","S","sw","none",null,null,true,null],
  ["X8R","R","sw","none",null,null,true,null],
  ["X9W","W","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9P","P","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9S","S","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X9R","R","p1:1 sw","aspect 1:1","1:1",null,true,"aspect = \"1:1\"\n"],
  ["X10W","W","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@130,0,1339,1071",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X10P","P","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@0,130,1071,1339",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X10S","S","p4:5 ds:move dm:0,0","aspect 4:5","4:5","move@107,0,857,1071",true,"aspect = \"4:5\"\n"],
  ["X10R","R","p4:5 ds:move dm:0,0","aspect 4:5 follow","4:5","move@0,130,1071,1339",true,"aspect = \"4:5\"\norientation = \"match\"\n"],
  ["X11W","W","p4:5 ds:move dm:50,50","box 180,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X11P","P","p4:5 ds:move dm:50,50","box 0,180,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X11S","S","p4:5 ds:move dm:50,50","box 157,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X11R","R","p4:5 ds:move dm:50,50","box 0,180,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X12W","W","p4:5 ds:move dm:5000,5000","box 261,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.163125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X12P","P","p4:5 ds:move dm:5000,5000","box 0,261,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.163125\nwidth = 1\nheight = 0.836875\n"],
  ["X12S","S","p4:5 ds:move dm:5000,5000","box 214,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.19981325863678806\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X12R","R","p4:5 ds:move dm:5000,5000","box 0,261,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.163125\nwidth = 1\nheight = 0.836875\n"],
  ["X13W","W","p4:5 ds:move dm:50,50 dm:0,0","box 130,0,1339,1071",null,"move@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X13P","P","p4:5 ds:move dm:50,50 dm:0,0","box 0,130,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.836875\n"],
  ["X13S","S","p4:5 ds:move dm:50,50 dm:0,0","box 107,0,857,1071",null,"move@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X13R","R","p4:5 ds:move dm:50,50 dm:0,0","box 0,130,1071,1339",null,"move@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.836875\n"],
  ["X14W","W","p4:5 ds:move dm:50,50 de","box 180,0,1339,1071",null,null,false,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X14P","P","p4:5 ds:move dm:50,50 de","box 0,180,1071,1339",null,null,false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X14S","S","p4:5 ds:move dm:50,50 de","box 157,0,857,1071",null,null,false,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X14R","R","p4:5 ds:move dm:50,50 de","box 0,180,1071,1339",null,null,false,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X15W","W","p4:5 ds:move dm:50,50 dn dm:100,100","box 180,0,1339,1071",null,null,true,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X15P","P","p4:5 ds:move dm:50,50 dn dm:100,100","box 0,180,1071,1339",null,null,true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X15S","S","p4:5 ds:move dm:50,50 dn dm:100,100","box 157,0,857,1071",null,null,true,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X15R","R","p4:5 ds:move dm:50,50 dn dm:100,100","box 0,180,1071,1339",null,null,true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X16W","W","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X16P","P","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X16S","S","p4:5 ds:move dm:50,50 p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X16R","R","p4:5 ds:move dm:50,50 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X17W","W","p4:5 ds:move dm:50,50 sw","box 180,0,1339,1071",null,"move@130,0,1339,1071",true,"x = 0.1125\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X17P","P","p4:5 ds:move dm:50,50 sw","box 0,180,1071,1339",null,"move@0,130,1071,1339",true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X17S","S","p4:5 ds:move dm:50,50 sw","box 157,0,857,1071",null,"move@107,0,857,1071",true,"x = 0.1465919701213819\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X17R","R","p4:5 ds:move dm:50,50 sw","box 0,180,1071,1339",null,"move@0,130,1071,1339",true,"x = 0\ny = 0.1125\nwidth = 1\nheight = 0.836875\n"],
  ["X18W","W","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18P","P","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18S","S","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X18R","R","p4:5 ds:move dm:50,50 rs","none",null,null,false,null],
  ["X19W","W","rs","none",null,null,true,null],
  ["X19P","P","rs","none",null,null,true,null],
  ["X19S","S","rs","none",null,null,true,null],
  ["X19R","R","rs","none",null,null,true,null],
  ["X20W","W","p4:5 km:1,1","box 131,0,1339,1071",null,null,false,"x = 0.081875\ny = 0\nwidth = 0.836875\nheight = 1\n"],
  ["X20P","P","p4:5 km:1,1","box 0,131,1071,1339",null,null,false,"x = 0\ny = 0.081875\nwidth = 1\nheight = 0.836875\n"],
  ["X20S","S","p4:5 km:1,1","box 108,0,857,1071",null,null,false,"x = 0.10084033613445378\ny = 0\nwidth = 0.800186741363212\nheight = 1\n"],
  ["X20R","R","p4:5 km:1,1","box 0,131,1071,1339",null,null,false,"x = 0\ny = 0.081875\nwidth = 1\nheight = 0.836875\n"],
  ["X21W","W","km:1,1","none",null,null,true,null],
  ["X21P","P","km:1,1","none",null,null,true,null],
  ["X21S","S","km:1,1","none",null,null,true,null],
  ["X21R","R","km:1,1","none",null,null,true,null],
  ["X22W","W","p4:5 kr:-1,0","box 130,0,1338,1071",null,null,false,"x = 0.08125\ny = 0\nwidth = 0.83625\nheight = 1\n"],
  ["X22P","P","p4:5 kr:-1,0","box 0,130,1070,1339",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.9990662931839402\nheight = 0.836875\n"],
  ["X22S","S","p4:5 kr:-1,0","box 107,0,856,1071",null,null,false,"x = 0.09990662931839403\ny = 0\nwidth = 0.7992530345471522\nheight = 1\n"],
  ["X22R","R","p4:5 kr:-1,0","box 0,130,1070,1339",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.9990662931839402\nheight = 0.836875\n"],
  ["X23W","W","p4:5 kr:-5000,-5000","box 130,0,64,64",null,null,false,"x = 0.08125\ny = 0\nwidth = 0.04\nheight = 0.059757236227824466\n"],
  ["X23P","P","p4:5 kr:-5000,-5000","box 0,130,64,64",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.04\n"],
  ["X23S","S","p4:5 kr:-5000,-5000","box 107,0,64,64",null,null,false,"x = 0.09990662931839403\ny = 0\nwidth = 0.059757236227824466\nheight = 0.059757236227824466\n"],
  ["X23R","R","p4:5 kr:-5000,-5000","box 0,130,64,64",null,null,false,"x = 0\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.04\n"],
  ["X24W","W","ds:nw dm:100,100","box 100,100,1500,971",null,"nw@0,0,1600,1071",false,"x = 0.0625\ny = 0.09337068160597572\nwidth = 0.9375\nheight = 0.9066293183940243\n"],
  ["X24P","P","ds:nw dm:100,100","box 100,100,971,1500",null,"nw@0,0,1071,1600",false,"x = 0.09337068160597572\ny = 0.0625\nwidth = 0.9066293183940243\nheight = 0.9375\n"],
  ["X24S","S","ds:nw dm:100,100","box 100,100,971,971",null,"nw@0,0,1071,1071",false,"x = 0.09337068160597572\ny = 0.09337068160597572\nwidth = 0.9066293183940243\nheight = 0.9066293183940243\n"],
  ["X24R","R","ds:nw dm:100,100","box 100,100,971,1500",null,"nw@0,0,1071,1600",false,"x = 0.09337068160597572\ny = 0.0625\nwidth = 0.9066293183940243\nheight = 0.9375\n"],
  ["X25W","W","p4:5 ds:w dm:5000,0","box 1405,0,64,1071",null,"w@130,0,1339,1071",false,"x = 0.878125\ny = 0\nwidth = 0.04\nheight = 1\n"],
  ["X25P","P","p4:5 ds:w dm:5000,0","box 1007,130,64,1339",null,"w@0,130,1071,1339",false,"x = 0.9402427637721755\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.836875\n"],
  ["X25S","S","p4:5 ds:w dm:5000,0","box 900,0,64,1071",null,"w@107,0,857,1071",false,"x = 0.8403361344537815\ny = 0\nwidth = 0.059757236227824466\nheight = 1\n"],
  ["X25R","R","p4:5 ds:w dm:5000,0","box 1007,130,64,1339",null,"w@0,130,1071,1339",false,"x = 0.9402427637721755\ny = 0.08125\nwidth = 0.059757236227824466\nheight = 0.836875\n"],
  ["X26W","W","p4:5 ds:se dm:5000,5000","box 130,0,1470,1071",null,"se@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.91875\nheight = 1\n"],
  ["X26P","P","p4:5 ds:se dm:5000,5000","box 0,130,1071,1470",null,"se@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.91875\n"],
  ["X26S","S","p4:5 ds:se dm:5000,5000","box 107,0,964,1071",null,"se@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.900093370681606\nheight = 1\n"],
  ["X26R","R","p4:5 ds:se dm:5000,5000","box 0,130,1071,1470",null,"se@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.91875\n"],
  ["X27W","W","p4:5 ds:s dm:0,-5000","box 130,0,1339,64",null,"s@130,0,1339,1071",false,"x = 0.08125\ny = 0\nwidth = 0.836875\nheight = 0.059757236227824466\n"],
  ["X27P","P","p4:5 ds:s dm:0,-5000","box 0,130,1071,64",null,"s@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.04\n"],
  ["X27S","S","p4:5 ds:s dm:0,-5000","box 107,0,857,64",null,"s@107,0,857,1071",false,"x = 0.09990662931839403\ny = 0\nwidth = 0.800186741363212\nheight = 0.059757236227824466\n"],
  ["X27R","R","p4:5 ds:s dm:0,-5000","box 0,130,1071,64",null,"s@0,130,1071,1339",false,"x = 0\ny = 0.08125\nwidth = 1\nheight = 0.04\n"],
  ["X30W","W","p3:2 rs","none",null,null,false,null],
  ["X30P","P","p3:2 rs","none",null,null,false,null],
  ["X30S","S","p3:2 rs","none",null,null,false,null],
  ["X30R","R","p3:2 rs","none",null,null,false,null],
  ["X31W","W","p3:2 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X31P","P","p3:2 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X31S","S","p3:2 p3:2","aspect 3:2","3:2",null,false,"aspect = \"3:2\"\n"],
  ["X31R","R","p3:2 p3:2","aspect 3:2 follow","3:2",null,false,"aspect = \"3:2\"\norientation = \"match\"\n"],
  ["X28S","S","p4:5 sw","aspect 5:4","4:5",null,false,"aspect = \"5:4\"\n"],
  ["X29S","S","p5:4 sw","aspect 4:5","5:4",null,false,"aspect = \"4:5\"\n"],
];

function deepFreeze(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.freeze(value);
    for (const v of Object.values(value)) deepFreeze(v);
  }
  return value;
}

function parseAction(token) {
  const [head, rest = ""] = token.split(":");
  const nums = () => rest.split(",").map(Number);
  if (/^p\d+$/.test(head)) {
    const [a, b] = token.slice(1).split(":").map(Number);
    return { type: "preset", a, b };
  }
  switch (head) {
    case "sw":
      return { type: "swap" };
    case "ds":
      return { type: "drag-start", handle: rest };
    case "dm":
      return { type: "drag-move", dx: nums()[0], dy: nums()[1] };
    case "de":
      return { type: "drag-end" };
    case "dn":
      return { type: "done" };
    case "km":
      return { type: "key-move", dx: nums()[0], dy: nums()[1] };
    case "kr":
      return { type: "key-resize", dx: nums()[0], dy: nums()[1] };
    case "rs":
      return { type: "reset" };
    default:
      throw new Error(`unknown action shorthand ${token}`);
  }
}

const boxText = (b) => `${b.x},${b.y},${b.w},${b.h}`;

function cropText(crop) {
  if (crop === null) return "none";
  const keys = Object.keys(crop).sort().join(",");
  if (keys === "box") return `box ${boxText(crop.box)}`;
  if (keys === "aspect,follow") return `aspect ${crop.aspect.join(":")}${crop.follow ? " follow" : ""}`;
  return `odd ${JSON.stringify(crop)}`;
}

const presetText = (s) => (s.preset === null ? null : s.preset.join(":"));
const dragText = (s) => (s.drag === null ? null : `${s.drag.handle}@${boxText(s.drag.start)}`);

// One photo per shape for the whole run: W and P are the two landscape and portrait
// fixtures, S the square, R the EXIF-rotated JPEG, all as the page reads them.
function modePhotos(ctx) {
  return Object.fromEntries(["W", "P", "S", "R"].map((k) => [k, photoFor(ctx, k)]));
}

function reducerRows(ctx, photos) {
  const failures = [];
  const { recipes } = ctx;
  for (const [id, shape, actions, crop, preset, drag, same, step] of MODE_ROWS) {
    const photo = photos[shape];
    const label = `mode-reducer: ${id} "${actions}"`;
    try {
      let state = deepFreeze(recipes.cropReducer(null, { type: "reset" }, photo));
      let last = state;
      for (const token of actions.split(" ")) {
        last = state;
        state = deepFreeze(recipes.cropReducer(state, parseAction(token), photo));
      }
      const miss = (what, got, want) => {
        if (got !== want) failures.push(`${label}: ${what} ${got}, expected ${want}`);
      };
      miss("crop", cropText(state.crop), crop);
      miss("preset", presetText(state), preset);
      miss("drag", dragText(state), drag);
      miss("same state", state === last, same);
      const gotStep = recipes.planEdit(photo, FORM_NO_WM, state.crop).crop;
      if (gotStep !== step) {
        failures.push(`${label}: crop step ${JSON.stringify(gotStep)}, expected ${JSON.stringify(step)}`);
      }
    } catch (e) {
      failures.push(`${label} threw: ${e.message}`);
    }
  }
  return failures;
}

// A box as { x, y, w, h } in that order, so two boxes compare as text.
const boxJson = (b) => JSON.stringify({ x: b.x, y: b.y, w: b.w, h: b.h });
const boxOf = (x, y, w, h) => ({ x, y, w, h });
const W_PHOTO = { width: 1600, height: 1071 };
const BOX_500 = boxOf(100, 100, 500, 400);

// Geometry rows: [id, what it runs, expected]. `run` is given the recipe.js exports and
// returns the value to compare; a row that must throw returns the message.
function geometryRows() {
  const thrown = (fn) => {
    try {
      fn();
      return "did not throw";
    } catch (e) {
      return e.message;
    }
  };
  const resize = (r, box, handle, dx, dy, photo = W_PHOTO) => boxJson(r.resizeBox(box, handle, dx, dy, photo));
  const move = (r, dx, dy) => boxJson(r.moveBox(boxOf(264, 0, 1071, 1071), dx, dy, W_PHOTO));
  const want = (x, y, w, h) => boxJson(boxOf(x, y, w, h));
  return [
    ["G1", (r) => r.toCopyDelta(3, 0.42875), 7],
    ["G2", (r) => r.toCopyDelta(1, 0.234375), 4],
    ["G3", (r) => r.toCopyDelta(-1, 0.4), -3],
    ["G4", (r) => r.toCopyDelta(1, 0.4), 3],
    ["G5", (r) => Object.is(r.toCopyDelta(-0.1, 0.5), 0), true],
    ["G6", (r) => r.toCopyDelta(5, 0), 0],
    ["G7", (r) => r.toCopyDelta(5, NaN), 0],
    ["G8", (r) => r.toCopyDelta(30, 0.42875), 70],
    ["G9", (r) => move(r, 50, 0), want(314, 0, 1071, 1071)],
    ["G10", (r) => move(r, -5000, 0), want(0, 0, 1071, 1071)],
    ["G11", (r) => move(r, 5000, 5000), want(529, 0, 1071, 1071)],
    ["G12", (r) => resize(r, BOX_500, "n", 10, 10), want(100, 110, 500, 390)],
    ["G13", (r) => resize(r, BOX_500, "s", 10, 10), want(100, 100, 500, 410)],
    ["G14", (r) => resize(r, BOX_500, "e", 10, 10), want(100, 100, 510, 400)],
    ["G15", (r) => resize(r, BOX_500, "w", 10, 10), want(110, 100, 490, 400)],
    ["G16", (r) => resize(r, BOX_500, "ne", 10, 10), want(100, 110, 510, 390)],
    ["G17", (r) => resize(r, BOX_500, "nw", 10, 10), want(110, 110, 490, 390)],
    ["G18", (r) => resize(r, BOX_500, "se", 10, 10), want(100, 100, 510, 410)],
    ["G19", (r) => resize(r, BOX_500, "sw", 10, 10), want(110, 100, 490, 410)],
    ["G20", (r) => resize(r, BOX_500, "e", -5000, 0), want(100, 100, 64, 400)],
    ["G21", (r) => resize(r, BOX_500, "n", 0, 5000), want(100, 436, 500, 64)],
    ["G22", (r) => resize(r, BOX_500, "nw", -5000, -5000), want(0, 0, 600, 500)],
    ["G23", (r) => resize(r, BOX_500, "se", 5000, 5000), want(100, 100, 1500, 971)],
    ["G24", (r) => resize(r, boxOf(0, 0, 40, 30), "se", -10, -10, { width: 40, height: 30 }), want(0, 0, 40, 30)],
    ["G25", (r) => resize(r, boxOf(0, 22, 100, 56), "s", 0, -10, { width: 100, height: 100 }), want(0, 22, 100, 56)],
    ["G26", (r) => resize(r, boxOf(0, 22, 100, 56), "s", 0, 10, { width: 100, height: 100 }), want(0, 22, 100, 66)],
    ["G27", (r) => thrown(() => r.resizeBox(BOX_500, "middle", 1, 1, W_PHOTO)), "unknown crop handle middle"],
    ["G28", (r) => r.MIN_CROP, 64],
    ["G29", (r) => thrown(() => r.cropReducer(null, { type: "zoom" }, W_PHOTO)), "unknown crop action zoom"],
    ["G30", (r) => resize(r, BOX_500, "w", 5000, 0), want(536, 100, 64, 400)],
    ["G31", (r) => JSON.stringify(r.cropReducer(null, { type: "reset" }, W_PHOTO)), '{"crop":null,"preset":null,"drag":null}'],
  ];
}

function geometryChecks(ctx) {
  const failures = [];
  const rowList = geometryRows();
  for (const [id, run, want] of rowList) {
    try {
      const got = run(ctx.recipes);
      if (got !== want) failures.push(`mode-geometry: ${id}: ${JSON.stringify(got)}, expected ${JSON.stringify(want)}`);
    } catch (e) {
      failures.push(`mode-geometry: ${id} threw: ${e.message}`);
    }
  }
  failures.ran = rowList.length;
  return failures;
}

// Size-line rows: [id, photo, crop, size, text]. `crop` builds the crop from the photo with
// recipe.js's own crop model, which the reducer rows already pin; the text is the literal.
const SIZE_LINE_ROWS = [
  ["L1", "W", () => null, "1600", "The whole photo, 1600 × 1071 · the copy will be 1600 × 1071"],
  ["L2", "W", (r, p) => r.presetCrop(3, 2, p), "1200", "Crop 3:2, 1600 × 1067 of 1600 × 1071 · the copy will be 1200 × 800"],
  ["L3", "P", (r, p) => r.presetCrop(3, 2, p), "1600", "Crop 2:3, 1067 × 1600 of 1071 × 1600 · the copy will be 1067 × 1600"],
  ["L4", "S", (r, p) => r.presetCrop(4, 5, p), "1600", "Crop 4:5, 857 × 1071 of 1071 × 1071 · the copy will be 857 × 1071"],
  ["L5", "S", (r, p) => r.presetCrop(5, 4, p), "800", "Crop 5:4, 1071 × 857 of 1071 × 1071 · the copy will be 800 × 640"],
  ["L6", "R", () => ({ box: boxOf(0, 314, 1071, 1071) }), "800", "Crop 1071 × 1071 of 1071 × 1600 · the copy will be 800 × 800"],
  ["L7", "W", () => ({ box: boxOf(0, 0, 1067, 711) }), "1600", "Crop 1067 × 711 of 1600 × 1071 · the copy will be 1067 × 711"],
  ["L8", "W", (r, p) => r.swapCrop(r.presetCrop(3, 2, p), p), "1080", "Crop 2:3, 714 × 1071 of 1600 × 1071 · the copy will be 714 × 1071"],
];

function sizeLineChecks(ctx, photos) {
  const failures = [];
  const { recipes } = ctx;
  for (const [id, shape, makeCrop, size, want] of SIZE_LINE_ROWS) {
    try {
      const photo = photos[shape];
      const crop = makeCrop(recipes, photo);
      const edit = recipes.planEdit(photo, { ...FORM_NO_WM, size }, crop);
      const got = recipes.sizeLine(edit, photo, crop);
      if (got !== want) failures.push(`mode-size-line: ${id}: "${got}", expected "${want}"`);
    } catch (e) {
      failures.push(`mode-size-line: ${id} threw: ${e.message}`);
    }
  }
  failures.ran = SIZE_LINE_ROWS.length;
  return failures;
}

/// The crop mode's reducer, box geometry and size line: 157 rows. `failures.ran` is how many
/// ran, which the summary line prints and which must be the count the rows add up to.
function modeChecks(ctx) {
  const photos = modePhotos(ctx);
  const geometry = geometryChecks(ctx);
  const sizes = sizeLineChecks(ctx, photos);
  const failures = [...reducerRows(ctx, photos), ...geometry, ...sizes];
  failures.ran = MODE_ROWS.length + geometry.ran + sizes.ran;
  if (failures.ran !== MODE_ROW_COUNT) {
    failures.push(`mode-count: ${failures.ran} crop-mode rows ran, expected ${MODE_ROW_COUNT}`);
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

/// B11-B19: the replay comparator's limbs, on synthetic input.
function replayCases() {
  const size = (width, height) => ({ width, height });
  const judged = (form, band, page, cli, db) => () => judgeReplay({ form, band, page, cli, psnr: db });
  const pair = (value) => ({ width: 2, height: 1, rgba: new Uint8Array(8).fill(value) });
  return [
    ["B11 box, integer, 1 px", judged("box", "integer", size(600, 482), size(601, 482), 70), true, ""],
    ["B12 box, integer, 2 px", judged("box", "integer", size(600, 482), size(602, 482), 70), false, "replay-dims"],
    ["B13 box, under 2x, 2 px", judged("box", "under-2x", size(600, 482), size(600, 484), 70), true, ""],
    ["B14 aspect, under 2x, 2 px", judged("aspect", "under-2x", size(1600, 1067), size(1600, 1069), 70), false, "replay-dims"],
    ["B15 aspect, PSNR 53.5", judged("aspect", "integer", size(8, 8), size(8, 8), 53.5), true, ""],
    ["B16 aspect, PSNR 53.49", judged("aspect", "integer", size(8, 8), size(8, 8), 53.49), false, "psnr"],
    ["B17 box, PSNR 58.99", judged("box", "integer", size(8, 8), size(8, 8), 58.99), false, "psnr"],
    ["B18 box, PSNR NaN", judged("box", "integer", size(8, 8), size(8, 8), NaN), false, "psnr"],
    // 100 against 101 on every channel: MSE 1, so 10 * log10(255^2) = 48.13 dB.
    ["B19 PSNR of 100 against 101", () => ({ ok: Math.abs(psnr(pair(100), pair(101)) - 48.13) < 0.005, reason: "" }), true, ""],
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
    ...replayCases(),
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

// The table's rows, then the crop rows that carry a watermark or a resize to 1080.
const N1_EXPECTED_RED = [3, 4, 7, 8, 11, 12, 13, 14, 16, "C29", "C30", "C35", "C36"];
const N2_EXPECTED_RED = [4, 8, 12, 13, 14, 16, "C35"];
// One copy pixel of crop width on the free box (C26 and the rows that reuse it): the native
// output is 601 px wide where the page's is 600, so the sizes differ.
const N4_EXPECTED_RED = ["C26", "C31", "C32", "C33", "C34"];

function selfTest(ctx) {
  let good = comparatorCases();
  const mutations = [
    ["N1 native opacity 0.6 -> 0.61", (t) => t.replace("opacity = 0.6\n", "opacity = 0.61\n"), N1_EXPECTED_RED],
    ["N2 native width 1080 -> 1079", (t) => t.replace("width = 1080\n", "width = 1079\n"), N2_EXPECTED_RED],
    [
      "N4 native crop width 0.5 -> 0.500625",
      (t) => t.replace("width = 0.5\n", "width = 0.500625\n"),
      N4_EXPECTED_RED,
    ],
  ];
  for (const [name, mutateNative, expected] of mutations) {
    const table = runTable(ctx, [...TABLE, ...CROP_TABLE], mutateNative);
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
  if (!shiftSelfTest(ctx)) good = false;
  return good;
}

// N5: the originals' content moved left by round(factor) px with the sizes kept, so every size
// still matches and only the framing is off. Every replay row must go red, and on the PSNR
// floor alone. Left, not right: an aspect is centred on the copy with its floor and on the
// original exactly, so the copy's frame always sits half a pixel left of the original's, and a
// shift to the right would cancel that on some rows.
// The blocked rows draw nothing, so there is no framing to move: they stay green.
const N5_EXPECTED_RED = REPLAY_ROWS.filter((r) => r.mark !== "blocked").map((r) => r.n);

function shiftSelfTest(ctx) {
  const replay = replayRows(ctx, { shift: true });
  const red = replay.rows.filter((r) => r.reasons.length > 0).map((r) => r.row.n);
  const drawn = replay.rows.filter((r) => r.row.mark !== "blocked");
  const blocked = replay.rows.filter((r) => r.row.mark === "blocked");
  const onPsnr = drawn.every((r) => r.reasons.length === 1 && r.reasons[0] === "psnr");
  const blockedGreen = blocked.every((r) => r.reasons.length === 0);
  const same = JSON.stringify(red) === JSON.stringify(N5_EXPECTED_RED) && onPsnr && blockedGreen;
  console.log(
    `${same ? "PASS" : "FAIL"} N5 the original's content moved left: red rows {${red}}, expected {${N5_EXPECTED_RED}}` +
      `${onPsnr ? ", each on psnr" : ", NOT each on psnr"}` +
      `${blockedGreen ? `, blocked rows {${blocked.map((r) => r.row.n)}} green` : ", a blocked row went red"}`,
  );
  for (const r of replay.rows) {
    console.log(`     ${r.row.n} ${r.psnr === undefined ? "-" : r.psnr.toFixed(2)} dB (${r.row.kind} floor ${PSNR_FLOOR[r.row.kind]}) ${r.reasons.join(",") || "green"}`);
  }
  if (same) rmSync(replay.dir, { recursive: true, force: true });
  else console.log(`     kept ${replay.dir}`);
  return same;
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
  let ran = 0;
  const dir = mkdtempSync(join(tmpdir(), "ddphotos-alpha-"));
  const inputPath = join(FIXTURES, INPUTS.W);
  const inputBytes = readFileSync(inputPath);
  const recipeFile = join(dir, "alpha.toml");
  writeFileSync(recipeFile, ALPHA_RECIPE);

  // Row A.
  ran += 1;
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
  ran += 1;
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

  if (ran !== ALPHA_ROW_COUNT) failures.push(`A-count: ${ran} rows ran, expected ${ALPHA_ROW_COUNT}`);
  if (failures.length === 0) rmSync(dir, { recursive: true, force: true });
  // The summary line prints what ran, not the constant.
  failures.ran = ran;
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
const ALPHA_ROW_COUNT = 2;

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
  // The summary line prints what ran, not the constant.
  failures.ran = ran;
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

const { transform, info, refuses } = await loadWasm(resolve(args.pkg));
const bin = resolve(args.bin);
if (!existsSync(bin)) fatal(`native binary missing: ${bin}`);
const ctx = { recipes, transform, info, refuses, bin, python: args.python };

if (args.selfTest) {
  process.exit(selfTest(ctx) ? 0 : 1);
}

const table = runTable(ctx, [...TABLE, ...CROP_TABLE]);
report(table);
const replay = replayRows(ctx);
reportReplay(replay);
const fit = fitRows(ctx);
reportFit(fit);
const alpha = alphaRows(ctx);
const orientation = orientationRows(ctx);
const mode = modeChecks(ctx);
const failures = [
  ...tableChecks(table),
  ...cropChecks(table),
  ...modelChecks(recipes),
  ...mode,
  ...placementChecks(ctx),
  ...replayChecks(replay),
  ...fit.failures,
  ...alpha,
  ...orientation,
];
const rowsFailed =
  table.rows.filter((r) => r.fail.length > 0).length + replay.rows.filter((r) => r.reasons.length > 0).length;
for (const f of failures) console.log(`FAIL ${f}`);
// Every failure is counted: row lines, then the other checks (P1-P5 and the rest), each
// named by the label before its first colon.
const tableNames = failures.map((f) => f.split(/: | -- /)[0]).join(", ");
const familyRows = (family) => table.rows.filter((r) => r.row.family === family).length;
console.log(
  `${familyRows("table")} table rows, ${familyRows("crop")} crop rows, ${replay.rows.length} replay rows, ` +
    `${orientation.ran} orientation rows, ${alpha.ran} alpha rows, ${mode.ran} crop-mode rows, ` +
    `${fit.ran} fit rows; ` +
    `${rowsFailed} row failures, ${failures.length} other failure${failures.length === 1 ? "" : "s"}` +
    `${failures.length ? ` (${tableNames})` : ""}, K=${K} (cap ${K_CAP})`,
);
// Informational, not an assertion: raising K up to the cap turns nothing red, so this
// line is how a K raised without new evidence shows in review.
const resampled = table.rows.filter((r) => r.cmp && r.resamples && r.cmp.px !== null);
const largest = Math.max(0, ...resampled.map((r) => r.cmp.px));
console.log(`largest differing-pixel count on a resampling row: ${largest} (K = ${K})`);
for (const r of replay.rows) {
  const db = r.psnr === undefined ? "-" : r.psnr === Infinity ? "inf" : r.psnr.toFixed(2);
  console.log(`replay ${r.row.n} ${r.row.id}: PSNR ${db} dB, floor ${PSNR_FLOOR[r.row.kind]} (${r.row.kind})`);
}
if (rowsFailed > 0 || failures.length > 0) {
  console.log(`outputs kept in ${table.dir}, ${replay.dir} and ${fit.dir}`);
  process.exit(1);
}
rmSync(table.dir, { recursive: true, force: true });
rmSync(replay.dir, { recursive: true, force: true });
rmSync(fit.dir, { recursive: true, force: true });
