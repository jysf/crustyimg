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

import { cliRecipe, pixelRecipe, planEdit } from "./recipe.js";

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

/// A picker tile. With no `href` it is shown but cannot be opened, and with no
/// `img` it gets an empty placeholder of the same shape.
function tile({ href, img, title, sub }) {
  const li = document.createElement("li");
  const a = document.createElement(href ? "a" : "div");
  if (href) a.href = href;
  else a.className = "tile-off";
  if (img) {
    const i = document.createElement("img");
    i.src = img;
    i.alt = "";
    i.loading = "lazy";
    a.append(i);
  } else {
    const blank = document.createElement("span");
    blank.className = "tile-blank";
    a.append(blank);
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
    // `count` includes videos, which cannot be edited. An album with no stills
    // stays listed, like on the site's home page, but cannot be opened. Older
    // sites may not publish a count; then the album is offered as usual.
    const stills = a.count === undefined ? undefined : a.count - (a.videoCount ?? 0);
    const editable = stills === undefined || stills > 0;
    $("tiles").append(
      tile({
        href: editable ? `?album=${encodeURIComponent(a.slug)}` : null,
        // albums.json cover paths already start with the album folder
        // (index.json paths do not).
        img: a.cover ? albumFileUrl(a.cover, a.coverVersion) : null,
        title: a.title,
        sub: editable ? a.dateSpan : "No photos to edit",
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
  if (!$("tiles").children.length) showError("This album has no photos to edit.", "note");
  $("picker").hidden = false;
}

// Captions may contain HTML; show them as plain text.
function stripHtml(html) {
  return new DOMParser().parseFromString(html, "text/html").body.textContent ?? "";
}

// ── the recipe ───────────────────────────────────────────────────────────────

let photo; // { album, entry, index, width, height }

/// The edit as a crustyimg recipe: the same TOML the CLI reads.
function readEdit() {
  return planEdit(photo, {
    size: $("size").value,
    wmOn: $("wm-on").checked,
    wmText: $("wm-text").value,
    wmGravity: $("wm-gravity").value,
    wmSize: $("wm-size").value,
    wmColor: $("wm-color").value,
    wmOpacity: $("wm-opacity").value,
    format: $("format").value,
  });
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
