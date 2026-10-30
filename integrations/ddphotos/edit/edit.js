// The edit page for a ddphotos site.
//
// It reads only the files a ddphotos site already publishes:
//   /albums/config.json         site name, copyright owner, encryption flag
//   /albums/albums.json         album list (absent when the whole site is encrypted)
//   /albums/<slug>/index.json   photos, with src.full (1600px WebP) and src.grid
//
// URL: /edit/index.html                          pick an album
//      /edit/index.html?album=<slug>             pick a photo
//      /edit/index.html?album=<slug>&photo=<n>   edit photo n (1-based, like permalinks)

const $ = (id) => document.getElementById(id);

// ── the engine, in a worker ──────────────────────────────────────────────────

// The worker, and so the multi-megabyte .wasm, starts only when the editor opens.
// The album and photo pickers never need it.
let worker = null;
const pending = new Map();
let nextId = 1;

/// Start the engine. Resolves once it is ready to take requests.
function startEngine() {
  $("engine").textContent = "Loading the engine…";
  worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  let engineReady;
  const booted = new Promise((resolve, reject) => (engineReady = { resolve, reject }));
  worker.onmessage = ({ data }) => {
    if (data.type === "ready") {
      $("engine").textContent = "Engine ready";
      $("engine-version").textContent = `v${data.version}`;
      engineReady.resolve();
      return;
    }
    if (data.type === "failed") {
      $("engine").textContent = "Engine failed to load";
      engineReady.reject(new Error(`The image engine could not start: ${data.message}`));
      return;
    }
    const p = pending.get(data.id);
    if (!p) return;
    pending.delete(data.id);
    data.ok ? p.resolve(data) : p.reject(new Error(data.message));
  };
  return booted;
}

function ask(type, payload = {}, transfer = []) {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, type, ...payload }, transfer);
  });
}

// ── reading the site ─────────────────────────────────────────────────────────

async function getJson(path) {
  const res = await fetch(path);
  if (!res.ok) throw new Error(`${path}: ${res.status} ${res.statusText}`);
  return res.json();
}

// Mirrors the site's own URL helper: files are served under /albums/ with ?v= when
// the photo has a version, because regenerated files keep their names.
function albumFileUrl(path, version) {
  const url = `/albums/${path}`;
  return version ? `${url}?v=${encodeURIComponent(version)}` : url;
}

/// Show a message above the page. `from` names what raised it, so the same thing
/// can clear it once it works again (see clearError).
function showError(text, kind = "error", from = "") {
  const el = $("error");
  el.className = kind; // "error", or "note" for information that is not a failure
  el.textContent = text;
  el.dataset.from = from;
  el.hidden = false;
}

/// Hide the message, if `from` raised it.
function clearError(from) {
  const el = $("error");
  if (el.dataset.from === from) el.hidden = true;
}

function tile({ href, img, title, sub }) {
  const li = document.createElement("li");
  const a = document.createElement("a");
  a.href = href;
  if (img) {
    const i = document.createElement("img");
    i.src = img;
    i.alt = "";
    i.loading = "lazy";
    a.append(i);
  }
  const t = document.createElement("span");
  t.className = "tile-title";
  t.textContent = title;
  a.append(t);
  if (sub) {
    const s = document.createElement("span");
    s.className = "tile-sub";
    s.textContent = sub;
    a.append(s);
  }
  li.append(a);
  return li;
}

async function showAlbums(config) {
  $("heading").textContent = "Pick an album";
  if (config.encrypted && !(await fetch("/albums/albums.json")).ok) {
    showError("This site is password-protected. The edit page cannot open encrypted albums yet.");
    return;
  }
  const albums = await getJson("/albums/albums.json");
  const open = albums.filter((a) => !a.encrypted);
  for (const a of open) {
    $("tiles").append(
      tile({
        href: `?album=${encodeURIComponent(a.slug)}`,
        // albums.json cover paths already start with the album folder
        // (index.json paths do not).
        img: a.cover ? albumFileUrl(a.cover, a.coverVersion) : null,
        title: a.title,
        sub: a.dateSpan,
      }),
    );
  }
  if (open.length < albums.length) {
    showError("Password-protected albums are hidden: the edit page cannot open them yet.", "note");
  }
  $("picker").hidden = false;
}

async function loadAlbum(slug, albums) {
  const summary = albums?.find((a) => a.slug === slug);
  if (summary?.encrypted) {
    throw new Error("This album is password-protected. The edit page cannot open it yet.");
  }
  if (albums && !summary) throw new Error(`This site has no album called "${slug}".`);
  return getJson(`/albums/${encodeURIComponent(slug)}/index.json`);
}

function showPhotos(album) {
  $("heading").textContent = `${album.title}: pick a photo`;
  album.photos.forEach((p, i) => {
    if (p.kind === "video") return; // only stills can be edited
    $("tiles").append(
      tile({
        href: `?album=${encodeURIComponent(album.slug)}&photo=${i + 1}`,
        img: albumFileUrl(`${album.slug}/${p.src.grid}`, p.version),
        title: p.description ? stripHtml(p.description) : p.fileName,
      }),
    );
  });
  $("tiles").classList.add("photos");
  $("picker").hidden = false;
}

// Captions may contain HTML; show them as plain text.
function stripHtml(html) {
  return new DOMParser().parseFromString(html, "text/html").body.textContent ?? "";
}

// ── the recipe ───────────────────────────────────────────────────────────────

// A TOML basic string. Watermark text is one line, so newlines are refused.
function tomlString(s) {
  return `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

let photo; // { album, entry, index, width, height }

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
function textEms(text) {
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
function watermarkSize(text, outW, outH, margin) {
  const outLong = Math.max(outW, outH);
  const wanted = Math.max(8, Math.round(outLong * Number($("wm-size").value)));
  const fitsWidth = Math.floor((0.9 * (outW - margin)) / textEms(text));
  const fitsHeight = Math.floor(0.1 * Math.min(outW, outH));
  const size = Math.max(1, Math.min(wanted, fitsWidth, fitsHeight));
  return { size, capped: size < wanted };
}

/// The edit as a crustyimg recipe: the same TOML the CLI reads.
function readEdit() {
  const longEdge = Math.max(photo.width, photo.height);
  const target = Number($("size").value) || longEdge;
  const outLong = Math.min(target, longEdge);
  // The output's sides, as `resize` (mode "max") computes them.
  const s = outLong / longEdge;
  const outW = Math.round(photo.width * s);
  const outH = Math.round(photo.height * s);
  const text = $("wm-text").value.trim().replace(/[\r\n]+/g, " ");
  const wm = $("wm-on").checked && text !== "";
  // Margin scales with the output, like the size.
  const margin = Math.round(outLong * 0.02);
  const fit = wm ? watermarkSize(text, outW, outH, margin) : null;
  return {
    resize: target < longEdge ? target : null,
    outLong,
    watermark: wm
      ? {
          text,
          gravity: $("wm-gravity").value,
          size: fit.size,
          capped: fit.capped,
          margin,
          color: $("wm-color").value,
          opacity: Number($("wm-opacity").value),
        }
      : null,
    format: $("format").value,
  };
}

function pixelRecipe(edit) {
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
function cliRecipe(edit) {
  let r = pixelRecipe({ ...edit, resize: edit.outLong });
  if (edit.format) r = r.replace('version = "2"\n', `version = "2"\nformat = "${edit.format}"\n`);
  if (edit.format !== "png") r += '\n[[step]]\nop = "optimize"\n';
  return r;
}

// ── the editor ───────────────────────────────────────────────────────────────

let previewUrl = null;
let downloadUrl = null;
let previewTimer = null;
let previewSeq = 0;

function schedulePreview() {
  clearTimeout(previewTimer);
  $("result").hidden = true;
  previewTimer = setTimeout(runPreview, 200);
}

async function runPreview() {
  const seq = ++previewSeq;
  $("busy").hidden = false;
  try {
    const edit = readEdit();
    $("wm-capped").hidden = !edit.watermark?.capped;
    const reply = await ask("preview", { recipe: pixelRecipe(edit) });
    if (seq !== previewSeq) return; // a newer edit superseded this one
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = URL.createObjectURL(new Blob([reply.bytes], { type: "image/jpeg" }));
    $("preview").src = previewUrl;
    clearError("preview"); // an earlier preview's failure no longer applies
  } catch (e) {
    if (seq === previewSeq) showError(`Preview failed: ${e.message}`, "error", "preview");
  } finally {
    if (seq === previewSeq) $("busy").hidden = true;
  }
}

const EXT = { jpeg: "jpg", webp: "webp", avif: "avif", png: "png", gif: "gif" };

// The similarity the engine's JPEG search aims for (its default target).
const SIMILARITY_TARGET = 90;

/// Single-quote a string for a POSIX shell.
function shellQuote(s) {
  return `'${s.replace(/'/g, "'\\''")}'`;
}

function kb(n) {
  return n >= 1048576 ? `${(n / 1048576).toFixed(2)} MB` : `${Math.round(n / 1024)} KB`;
}

async function makeCopy(event) {
  event.preventDefault();
  const edit = readEdit();
  const button = $("make");
  button.disabled = true;
  button.textContent = edit.format === "avif" || edit.format === "" ? "Encoding (this can take a few seconds)…" : "Encoding…";
  $("result").hidden = true;
  $("error").hidden = true;
  try {
    const r = await ask("make", { recipe: pixelRecipe(edit), format: edit.format });
    const stem = photo.entry.fileName.replace(/\.[^.]+$/, "");
    const name = `${stem}-${Math.max(r.width, r.height)}.${EXT[r.format] ?? r.format}`;
    if (downloadUrl) URL.revokeObjectURL(downloadUrl);
    downloadUrl = URL.createObjectURL(new Blob([r.bytes], { type: `image/${r.format}` }));
    const link = $("download");
    link.href = downloadUrl;
    link.download = name;
    link.textContent = `Download ${name}`;

    const parts = [`${r.width} × ${r.height}`, r.format.toUpperCase(), kb(r.bytes.byteLength)];
    if (r.encoderQuality !== undefined) parts.push(`quality ${r.encoderQuality}`);
    if (r.score !== undefined) parts.push(`similarity ${r.score.toFixed(1)} / 100`);
    parts.push(`${(r.ms / 1000).toFixed(1)} s`);
    let summary = parts.join(" · ");
    // The JPEG search aims for SIMILARITY_TARGET and stops at quality 100. Say so
    // when even that could not reach it, rather than imply the target was met.
    if (r.format === "jpeg" && r.score !== undefined && r.score < SIMILARITY_TARGET) {
      summary += `. JPEG could not reach the ${SIMILARITY_TARGET} target on this photo, even at its highest quality.`;
    }
    $("result-summary").textContent = summary;

    const recipe = cliRecipe({ ...edit, format: edit.format || r.format });
    $("recipe").textContent = recipe;
    const src = photo.entry.sourcePath || photo.entry.fileName;
    // Without -o, `apply` writes the image to stdout.
    $("cli").textContent = `crustyimg apply --recipe edit.toml ${shellQuote(src)} -o ${shellQuote(name)}`;
    $("result").hidden = false;
  } catch (e) {
    showError(`Could not make the copy: ${e.message}`);
  } finally {
    button.disabled = false;
    button.textContent = "Make the copy";
  }
}

function fillSizes(longEdge) {
  const sel = $("size");
  sel.replaceChildren();
  const add = (value, label) => sel.append(new Option(label, String(value)));
  add(longEdge, `As published (${longEdge}px)`);
  for (const s of [1200, 1080, 800, 400]) if (s < longEdge) add(s, `${s}px`);
}

async function showEditor(album, index, config) {
  const entry = album.photos[index];
  if (!entry) throw new Error(`This album has no photo ${index + 1}.`);
  if (entry.kind === "video") throw new Error("That item is a video. Only photos can be edited.");

  $("heading").textContent = album.title;
  $("back").href = `/albums/${encodeURIComponent(album.slug)}/${index + 1}`;
  $("back").textContent = "← Back to the photo";
  $("caption").textContent = entry.description ? stripHtml(entry.description) : entry.fileName;
  $("editor").hidden = false;

  const year = new Date().getFullYear();
  $("wm-text").value = config.copyrightOwner ? `© ${year} ${config.copyrightOwner}` : `© ${year}`;

  // Show the published image straight away; the engine replaces it with a preview.
  const url = albumFileUrl(`${album.slug}/${entry.src.full}`, entry.version);
  $("preview").src = url;
  $("preview").alt = $("caption").textContent;

  const [bytes] = await Promise.all([
    fetch(url).then((r) => {
      if (!r.ok) throw new Error(`${url}: ${r.status}`);
      return r.arrayBuffer();
    }),
    startEngine(),
  ]);
  const meta = await ask("load", { bytes }, [bytes]);
  photo = { album, entry, index, width: meta.width, height: meta.height };
  fillSizes(Math.max(meta.width, meta.height));

  const form = $("controls");
  form.addEventListener("input", (e) => {
    if (e.target.id === "wm-opacity") {
      $("wm-opacity-out").textContent = `${Math.round(e.target.value * 100)}%`;
    }
    $("wm-fields").classList.toggle("off", !$("wm-on").checked);
    if (e.target.id !== "format") schedulePreview();
    else $("result").hidden = true;
  });
  form.addEventListener("submit", makeCopy);
  $("wm-fields").classList.add("off");
  $("make").disabled = false;
  runPreview();
}

$("copy").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText($("recipe").textContent);
    $("copy").textContent = "Copied";
  } catch {
    $("copy").textContent = "Copy failed: select the text instead";
  }
  setTimeout(() => ($("copy").textContent = "Copy recipe"), 2000);
});

// ── start ────────────────────────────────────────────────────────────────────

async function main() {
  const params = new URLSearchParams(location.search);
  const slug = params.get("album");
  const n = Number.parseInt(params.get("photo") ?? "", 10);

  const config = await getJson("/albums/config.json");
  $("site-name").textContent = config.siteName ?? "";
  document.title = `Edit a copy · ${config.siteName ?? "Photos"}`;

  if (!slug) return showAlbums(config);

  const albums = await fetch("/albums/albums.json").then((r) => (r.ok ? r.json() : null));
  const album = await loadAlbum(slug, albums);
  $("back").href = `/albums/${encodeURIComponent(slug)}`;
  $("back").textContent = `← ${album.title}`;

  if (!Number.isInteger(n) || n < 1) return showPhotos(album);
  return showEditor(album, n - 1, config);
}

main().catch((e) => showError(e.message));
