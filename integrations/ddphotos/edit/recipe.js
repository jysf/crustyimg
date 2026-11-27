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

/// The size of the largest `a:b` box that fits a `w` × `h` image, as the engine's
/// `crop` step computes it: the long side is the image's, the other is rounded half up.
export function aspectSize(a, b, w, h) {
  if (w * b <= h * a) return [w, Math.floor((2 * w * b + a) / (2 * a))];
  return [Math.floor((2 * h * a + b) / (2 * b)), h];
}

/// `a:b` written in the order that matches the image's shape: long side first on a
/// landscape image, short side first on a portrait one, as written on a square.
export function followOrientation(a, b, w, h) {
  const long = Math.max(a, b);
  const short = Math.min(a, b);
  if (w > h) return [long, short];
  if (w < h) return [short, long];
  return [a, b];
}

// The aspect a crop shows on `photo`: a following preset takes the photo's shape.
function shownAspect(crop, photo) {
  const [a, b] = crop.aspect;
  return crop.follow ? followOrientation(a, b, photo.width, photo.height) : [a, b];
}

/// A preset crop: the largest centred `a:b` box. It follows the photo's orientation
/// (3:2 on a portrait photo is 2:3), except for 1:1 and on a square photo, which have
/// no orientation to follow.
export function presetCrop(a, b, photo) {
  return { aspect: [a, b], follow: a !== b && photo.width !== photo.height };
}

/// The other orientation of a preset crop, written as shown and with no following,
/// so a recipe that is reused keeps the orientation the user chose.
export function swapCrop(crop, photo) {
  const [a, b] = shownAspect(crop, photo);
  return { aspect: [b, a], follow: false };
}

/// The crop's box in pixels of the oriented photo: the whole photo for no crop, the
/// box as given, or a preset's box centred the way the engine centres it.
export function cropBox(crop, photo) {
  if (!crop) return { x: 0, y: 0, w: photo.width, h: photo.height };
  if (crop.box) return { ...crop.box };
  const [a, b] = shownAspect(crop, photo);
  const [w, h] = aspectSize(a, b, photo.width, photo.height);
  return { x: Math.floor((photo.width - w) / 2), y: Math.floor((photo.height - h) / 2), w, h };
}

// The crop step's keys, as the text of the step. A box is four fractions of the
// photo, each the shortest text that reads back as the same number, so the engine
// finds the same pixel edges. A preset is its aspect, plus the key that makes it
// follow each image's orientation when it does.
function cropStep(crop, photo) {
  if (!crop) return null;
  if (crop.box) {
    const { x, y, w, h } = crop.box;
    return (
      `x = ${x / photo.width}\ny = ${y / photo.height}\n` +
      `width = ${w / photo.width}\nheight = ${h / photo.height}\n`
    );
  }
  const [a, b] = crop.aspect;
  return `aspect = "${a}:${b}"\n` + (crop.follow ? 'orientation = "match"\n' : "");
}

/// The edit for a photo and the form's raw values, as the recipes read it.
///
/// `photo` is `{ width, height }`, the size of the photo as it is shown (after its
/// orientation is applied). `form` is `{ size, wmOn, wmText, wmGravity, wmSize,
/// wmColor, wmOpacity, format }`, the values as the form holds them. `crop` is null,
/// a preset (`presetCrop`) or a box (`{ box: { x, y, w, h } }`). `outW` and `outH` are
/// the output's sides, which the page uses to say what it made; they, the size and the
/// watermark's fit all come from the cropped size.
export function planEdit(photo, form, crop = null) {
  const box = cropBox(crop, photo);
  const longEdge = Math.max(box.w, box.h);
  const target = Number(form.size) || longEdge;
  const outLong = Math.min(target, longEdge);
  // The output's sides, as `resize` (mode "max") computes them: each at least 1 px.
  const s = outLong / longEdge;
  const outW = Math.max(1, Math.round(box.w * s));
  const outH = Math.max(1, Math.round(box.h * s));
  const text = form.wmText.trim().replace(/[\r\n]+/g, " ");
  const wm = form.wmOn && text !== "";
  // Margin scales with the output, like the size.
  const margin = Math.round(outLong * 0.02);
  const fit = wm ? watermarkSize(text, outW, outH, margin, form.wmSize) : null;
  return {
    crop: cropStep(crop, photo),
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

/// The edit as a crustyimg recipe: the same TOML the CLI reads. A crop goes right
/// after `auto-orient`, so it acts on the upright photo, and before `resize`.
export function pixelRecipe(edit) {
  let r = 'version = "2"\n\n[[step]]\nop = "auto-orient"\n';
  if (edit.crop) r += `\n[[step]]\nop = "crop"\n${edit.crop}`;
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

// The photo as the page shows it: only its orientation applied, as the preview's
// JPEG, which is also what the crop will be drawn on.
const ORIENTED_BASE = previewRecipe('version = "2"\n\n[[step]]\nop = "auto-orient"\n');

/// Read a photo for the page. `wasm` is the engine's `{ info, transform }`. The size
/// is that of the photo after its orientation is applied (`info` of the stored file
/// reports the stored size, which is sideways for a rotated JPEG), so every crop and
/// every planned size is in the pixels the user sees. `bytes` is the upright render.
export function loadPhoto(wasm, source) {
  const stored = wasm.info(source);
  const format = stored.format;
  stored.free(); // wasm-side objects are not garbage-collected
  const bytes = wasm.transform(source, ORIENTED_BASE, "jpeg");
  const shown = wasm.info(bytes);
  const photo = { width: shown.width, height: shown.height, format, bytes };
  shown.free();
  return photo;
}

// ── the crop mode ────────────────────────────────────────────────────────────
//
// The state and the box geometry behind a crop control, as plain functions of values:
// the page's pointer and key handlers turn what they read into the arguments here, and
// draw what comes back. Boxes are `{ x, y, w, h }` in pixels of the upright photo, whole
// numbers, and the recipe text still comes only from `cropStep`.

/// The smallest side, in photo pixels, that a box can be resized to. It is the box's
/// usability floor (a handle can still be grabbed), not a promise that a watermark fits.
export const MIN_CROP = 64;

/// A pointer's movement on screen, in CSS pixels, as whole photo pixels. `ratio` is the
/// shown image's width over its natural width. Halves round away from zero on both signs,
/// so dragging left and dragging right by the same distance move the box the same
/// distance, and a drag too small to count is `0`, never `-0`. A ratio that is not a
/// positive finite number (an image not yet laid out) moves nothing.
export function toCopyDelta(dCss, ratio) {
  if (!(ratio > 0) || !Number.isFinite(ratio) || !Number.isFinite(dCss)) return 0;
  const photoPixels = Math.sign(dCss) * Math.round(Math.abs(dCss) / ratio);
  return photoPixels === 0 ? 0 : photoPixels;
}

const clamp = (value, low, high) => Math.min(Math.max(value, low), high);

const sameBox = (a, b) => a.x === b.x && a.y === b.y && a.w === b.w && a.h === b.h;

/// `box` moved by (`dx`, `dy`) and held inside `photo`, with its size unchanged. It
/// expects a box that is already inside the photo, as every box here is.
export function moveBox(box, dx, dy, photo) {
  return {
    x: clamp(box.x + dx, 0, photo.width - box.w),
    y: clamp(box.y + dy, 0, photo.height - box.h),
    w: box.w,
    h: box.h,
  };
}

// The edges each handle moves: n, s, e and w are the sides, the corners are two sides.
const HANDLE_EDGES = { n: "n", s: "s", e: "e", w: "w", ne: "ne", nw: "nw", se: "se", sw: "sw" };

/// `box` with the edges `handle` names moved by `dx` (east and west) and `dy` (north and
/// south); the other edges stay put. An edge stops at the photo, and at `MIN_CROP` from
/// the opposite edge. It never passes the opposite edge: that would change which handle
/// is being dragged part-way through, and a key press has no way to flip. A box already
/// smaller than `MIN_CROP` (a preset on a very small photo) can grow but not shrink.
export function resizeBox(box, handle, dx, dy, photo) {
  if (!Object.hasOwn(HANDLE_EDGES, handle)) throw new Error(`unknown crop handle ${handle}`);
  const edges = HANDLE_EDGES[handle];
  const minW = Math.min(MIN_CROP, box.w);
  const minH = Math.min(MIN_CROP, box.h);
  let left = box.x;
  let top = box.y;
  let right = box.x + box.w;
  let bottom = box.y + box.h;
  if (edges.includes("w")) left = clamp(left + dx, 0, right - minW);
  if (edges.includes("e")) right = clamp(right + dx, left + minW, photo.width);
  if (edges.includes("n")) top = clamp(top + dy, 0, bottom - minH);
  if (edges.includes("s")) bottom = clamp(bottom + dy, top + minH, photo.height);
  return { x: left, y: top, w: right - left, h: bottom - top };
}

// The state with its crop made the box `box`, which was moved from `from`. A box that is
// where the crop already was changes nothing, so a press with no movement leaves a preset
// a preset, and the same state object comes back for the caller to see that.
function toBox(s, box, from) {
  if (!s.crop?.box && sameBox(box, from)) return s;
  if (s.crop?.box && sameBox(box, s.crop.box)) return s;
  return { crop: { box }, preset: null, drag: s.drag };
}

/// The crop mode's state machine. `state` is `{ crop, preset, drag }`:
///   crop    what `planEdit` takes: null, a preset (`presetCrop`, or `swapCrop` once
///           swapped) or a box (`{ box }`);
///   preset  `[a, b]` of the preset button that made the crop, null after a drag, a key
///           press or a reset, so the page can show which button is on;
///   drag    null, or `{ handle, start }` while a pointer is down: `handle` is "move" or
///           a resize handle, `start` the box when the pointer went down.
/// `cropReducer(null, { type: "reset" }, photo)` is the starting state.
///
/// The actions:
///   preset { a, b }     the preset's centred box, from any state
///   swap                the other orientation of a preset; nothing on no crop, a box or 1:1
///   drag-start { handle }
///   drag-move { dx, dy }    photo pixels from where the drag started, not per move
///   drag-end, done          the pointer is up
///   key-move { dx, dy }     the box moved, from the crop's box (the whole photo for none)
///   key-resize { dx, dy }   the box's bottom-right corner moved
///   reset                   no crop
/// An action that changes nothing returns the same object, which is how the page tells
/// that Swap does nothing here. The state is never changed in place.
export function cropReducer(state, action, photo) {
  const s = state ?? { crop: null, preset: null, drag: null };
  switch (action.type) {
    case "preset": {
      const crop = presetCrop(action.a, action.b, photo);
      const same =
        s.crop?.aspect &&
        s.crop.aspect[0] === crop.aspect[0] &&
        s.crop.aspect[1] === crop.aspect[1] &&
        s.crop.follow === crop.follow &&
        s.preset?.[0] === action.a &&
        s.preset?.[1] === action.b &&
        s.drag === null;
      return same ? s : { crop, preset: [action.a, action.b], drag: null };
    }
    case "swap":
      // 1:1 swaps to itself, so there is nothing to change; a box has no aspect to swap.
      if (!s.crop || s.crop.box || s.crop.aspect[0] === s.crop.aspect[1]) return s;
      return { ...s, crop: swapCrop(s.crop, photo) };
    case "drag-start":
      return { ...s, drag: { handle: action.handle, start: cropBox(s.crop, photo) } };
    case "drag-move": {
      if (!s.drag) return s; // a stray move after the pointer went up
      const { handle, start } = s.drag;
      const box =
        handle === "move"
          ? moveBox(start, action.dx, action.dy, photo)
          : resizeBox(start, handle, action.dx, action.dy, photo);
      return toBox(s, box, start);
    }
    case "drag-end":
      return s.drag ? { ...s, drag: null } : s;
    case "done":
      // Leaving the mode also clears the pointer state, so a stray move after it does nothing.
      return s.drag ? { ...s, drag: null } : s;
    case "key-move": {
      const from = cropBox(s.crop, photo);
      return toBox(s, moveBox(from, action.dx, action.dy, photo), from);
    }
    case "key-resize": {
      const from = cropBox(s.crop, photo);
      return toBox(s, resizeBox(from, "se", action.dx, action.dy, photo), from);
    }
    case "reset":
      if (s.crop === null && s.preset === null && s.drag === null) return s;
      return { crop: null, preset: null, drag: null };
    default:
      throw new Error(`unknown crop action ${action.type}`);
  }
}

/// The crop mode's size line: what is cropped, then what the copy will be, from
/// `planEdit`'s output for the same crop (so a size below the crop's long edge reads as
/// the resized size). A preset shows the aspect as shown, so a swap is visible.
export function sizeLine(edit, photo, crop) {
  const of = `${photo.width} × ${photo.height}`;
  const box = cropBox(crop, photo);
  let what = `The whole photo, ${of}`;
  if (crop?.box) {
    what = `Crop ${box.w} × ${box.h} of ${of}`;
  } else if (crop) {
    const [a, b] = shownAspect(crop, photo);
    what = `Crop ${a}:${b}, ${box.w} × ${box.h} of ${of}`;
  }
  return `${what} · the copy will be ${edit.outW} × ${edit.outH}`;
}
