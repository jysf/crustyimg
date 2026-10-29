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

const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
const pending = new Map();
let nextId = 1;
let engineReady;
const engineBooted = new Promise((resolve, reject) => (engineReady = { resolve, reject }));

worker.onmessage = ({ data }) => {
  if (data.type === "ready") {
    $("engine").textContent = "Engine ready";
    $("engine-version").textContent = `v${data.version}`;
    engineReady.resolve();
    return;
  }
  if (data.type === "failed") {
    $("engine").textContent = "Engine failed to load";
    engineReady.reject(new Error(data.message));
    return;
  }
  const p = pending.get(data.id);
  if (!p) return;
  pending.delete(data.id);
  data.ok ? p.resolve(data) : p.reject(new Error(data.message));
};

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

function showError(text, kind = "error") {
  const el = $("error");
  el.className = kind; // "error", or "note" for information that is not a failure
  el.textContent = text;
  el.hidden = false;
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

/// The edit as a crustyimg recipe: the same TOML the CLI reads.
function readEdit() {
  const longEdge = Math.max(photo.width, photo.height);
  const target = Number($("size").value) || longEdge;
  const outLong = Math.min(target, longEdge);
  const wm = $("wm-on").checked && $("wm-text").value.trim() !== "";
  return {
    resize: target < longEdge ? target : null,
    outLong,
    watermark: wm
      ? {
          text: $("wm-text").value.trim().replace(/[\r\n]+/g, " "),
          gravity: $("wm-gravity").value,
          // Size and margin scale with the output, so the mark looks the same at
          // any size the user picks.
          size: Math.max(8, Math.round(outLong * Number($("wm-size").value))),
          margin: Math.round(outLong * 0.02),
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
    const reply = await ask("preview", { recipe: pixelRecipe(readEdit()) });
    if (seq !== previewSeq) return; // a newer edit superseded this one
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = URL.createObjectURL(new Blob([reply.bytes], { type: "image/jpeg" }));
    $("preview").src = previewUrl;
  } catch (e) {
    if (seq === previewSeq) showError(`Preview failed: ${e.message}`);
  } finally {
    if (seq === previewSeq) $("busy").hidden = true;
  }
}

const EXT = { jpeg: "jpg", webp: "webp", avif: "avif", png: "png", gif: "gif" };

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
    $("result-summary").textContent = parts.join(" · ");

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
    engineBooted,
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

engineBooted.catch((e) => showError(`The image engine could not start: ${e.message}`));
main().catch((e) => showError(e.message));
