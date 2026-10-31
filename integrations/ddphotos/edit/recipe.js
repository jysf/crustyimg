// The edit page's recipes, as plain functions of values.
//
// Nothing here reads the page: the page's form values come in as arguments. That
// keeps one copy of the recipe code, used by the page, its worker, and the test
// that runs the same recipes through the native CLI and the wasm build.

// A TOML basic string. Watermark text is one line, so newlines are refused.
export function tomlString(s) {
  return `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

// Advance widths of crustyimg's bundled watermark font (Go Regular, 2048 units per
// em) for U+0020..U+00FF; 0 marks the unused control range. The engine lays text
// out as the plain sum of these advances (no kerning), so the page can tell how
// wide a mark will be before asking for it. Characters outside this range use the
// font's widest advance, which can only overestimate.
// prettier-ignore
const GO_ADVANCE = [
  569,569,727,1139,1139,1821,1366,391,682,682,1196,1196,648,1196,648,569,1139,1139,1139,1139,1139,1139,1139,1139,1139,1139,627,627,1196,1196,1196,1139,
  2079,1366,1366,1479,1479,1366,1251,1593,1479,817,1015,1366,1139,1706,1479,1593,1366,1593,1479,1366,1251,1479,1366,1933,1366,1366,1251,569,569,569,960,1139,
  682,1139,1139,1024,1139,1139,569,1139,1139,505,519,1024,548,1706,1139,1139,1139,1139,682,1024,579,1139,1024,1479,1024,1024,1024,684,532,684,1196,0,
  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
  569,682,1139,1139,1139,1139,532,1139,682,1509,758,1139,1196,682,1509,1139,819,1196,933,933,682,1139,1100,547,682,933,748,1139,1708,1708,1708,1251,
  1366,1366,1366,1366,1366,1366,2048,1479,1366,1366,1366,1366,817,817,817,817,1489,1479,1593,1593,1593,1593,1593,1196,1593,1479,1479,1479,1479,1366,1366,1251,
  1139,1139,1139,1139,1139,1139,1821,1024,1139,1139,1139,1139,505,505,505,505,1139,1139,1139,1139,1139,1139,1139,1196,1251,1139,1139,1139,1139,1024,1139,1024,
];
const GO_WIDEST = 2240;

/// The width of `text` in ems of the bundled font.
export function textEms(text) {
  let units = 0;
  for (const ch of text) {
    const cp = ch.codePointAt(0);
    units += cp >= 0x20 && cp <= 0xff ? GO_ADVANCE[cp - 0x20] : GO_WIDEST;
  }
  return units / 2048;
}

/// The watermark's font size in pixels for an output of `outW` × `outH`.
///
/// The chosen size (Small, Medium, Large) is a fraction of the long edge, so the
/// mark looks the same at any output size. On a portrait or a narrow photo that
/// can make the text wider than the image, which the engine refuses. So the size
/// is capped to keep the text within 90% of the width (after the margin), and to
/// a tenth of the short edge, which keeps the mark's height and its share of the
/// image well inside the engine's limits. `capped` says the cap applied.
export function watermarkSize(text, outW, outH, margin, sizeFraction) {
  const outLong = Math.max(outW, outH);
  const wanted = Math.max(8, Math.round(outLong * Number(sizeFraction)));
  const fitsWidth = Math.floor((0.9 * (outW - margin)) / textEms(text));
  const fitsHeight = Math.floor(0.1 * Math.min(outW, outH));
  const size = Math.max(1, Math.min(wanted, fitsWidth, fitsHeight));
  return { size, capped: size < wanted };
}

/// The edit for a photo and the form's raw values, as the recipes read it.
///
/// `photo` is `{ width, height }`. `form` is `{ size, wmOn, wmText, wmGravity,
/// wmSize, wmColor, wmOpacity, format }`, the values as the form holds them.
/// `outW` and `outH` are the output's sides, which the page uses to say what it
/// made.
export function planEdit(photo, form) {
  const longEdge = Math.max(photo.width, photo.height);
  const target = Number(form.size) || longEdge;
  const outLong = Math.min(target, longEdge);
  // The output's sides, as `resize` (mode "max") computes them.
  const s = outLong / longEdge;
  const outW = Math.round(photo.width * s);
  const outH = Math.round(photo.height * s);
  const text = form.wmText.trim().replace(/[\r\n]+/g, " ");
  const wm = form.wmOn && text !== "";
  // Margin scales with the output, like the size.
  const margin = Math.round(outLong * 0.02);
  const fit = wm ? watermarkSize(text, outW, outH, margin, form.wmSize) : null;
  return {
    resize: target < longEdge ? target : null,
    outLong,
    outW,
    outH,
    watermark: wm
      ? {
          text,
          gravity: form.wmGravity,
          size: fit.size,
          capped: fit.capped,
          margin,
          color: form.wmColor,
          opacity: Number(form.wmOpacity),
        }
      : null,
    format: form.format,
  };
}

/// The edit as a crustyimg recipe: the same TOML the CLI reads.
export function pixelRecipe(edit) {
  let r = 'version = "2"\n\n[[step]]\nop = "auto-orient"\n';
  if (edit.resize) {
    r += `\n[[step]]\nop = "resize"\nmode = "max"\nwidth = ${edit.resize}\n`;
  }
  if (edit.watermark) {
    const w = edit.watermark;
    r +=
      `\n[[step]]\nop = "watermark"\ntext = ${tomlString(w.text)}\n` +
      `gravity = "${w.gravity}"\nsize = ${w.size}\nmargin = ${w.margin}\n` +
      `color = "${w.color}"\nopacity = ${w.opacity}\n`;
  }
  return r;
}

/// The recipe for the CLI, to run on the original file. The original is usually
/// much larger than the published copy, and the watermark size is in pixels, so
/// this recipe always resizes to the long edge the page made: the result matches
/// what the user saw, built from a better source. `format` is the one the page
/// actually wrote, so "let the engine choose" gives the CLI the same answer. The
/// terminal `optimize` step uses the CLI's own fast quality choice, not the page's
/// similarity search, so the file sizes can differ.
export function cliRecipe(edit) {
  let r = pixelRecipe({ ...edit, resize: edit.outLong });
  if (edit.format) r = r.replace('version = "2"\n', `version = "2"\nformat = "${edit.format}"\n`);
  if (edit.format !== "png") r += '\n[[step]]\nop = "optimize"\n';
  return r;
}

// Preview quality. A preview only needs to look right on screen, so it uses a fast
// fixed-quality JPEG rather than the perceptual search the final copy gets.
export const PREVIEW_QUALITY = 88;

/// A recipe with a top-level quality, for the preview encode.
function withQuality(recipe, quality) {
  return recipe.replace('version = "2"\n', `version = "2"\nquality = ${quality}\n`);
}

/// The recipe for the page's preview: the same steps, as a fast fixed-quality JPEG.
export function previewRecipe(recipe) {
  return withQuality(recipe, PREVIEW_QUALITY);
}
