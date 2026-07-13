#!/usr/bin/env node
// A static file server for the demo (SPEC-077) — the local stand-in for GitHub Pages.
//
// It exists for ONE reason: the demo cannot be opened as a `file://` URL. The wasm
// package is built `--target web`, so `init()` fetches `crustyimg_bg.wasm` and hands
// it to `WebAssembly.instantiateStreaming`, which refuses anything not served as
// `application/wasm` — and a `file://` fetch has no MIME type at all (it does not
// even get that far: the request is treated as cross-origin and blocked). So the
// page needs a server, and the server needs the MIME table below.
//
// Node ships no static server, and a demo that made you `npm install` one would
// undercut the "no toolchain" pitch — so this is 60 lines of `node:http` with no
// dependencies. It is a dev/test server: local-only, GET/HEAD, no caching, and it
// refuses to serve anything outside its root.
//
// Run: `just demo-serve` (or `node scripts/serve.mjs <root> [port]`).

import { createServer } from "node:http";
import { createReadStream, statSync } from "node:fs";
import { extname, join, normalize, resolve, sep } from "node:path";

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".map": "application/json; charset=utf-8",
  ".ts": "text/plain; charset=utf-8", // the .d.ts files, if anyone asks for one
  ".wasm": "application/wasm", // ← the load-bearing line
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".avif": "image/avif",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".md": "text/markdown; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
};

/// Start the server on `port` (0 = an ephemeral port) and resolve once it is
/// listening. Returns the server and its base URL.
export function startServer({ root, port = 0, host = "127.0.0.1", onRequest }) {
  const rootDir = resolve(root);

  const server = createServer((req, res) => {
    if (req.method !== "GET" && req.method !== "HEAD") {
      res.writeHead(405, { Allow: "GET, HEAD" }).end();
      return;
    }

    const url = new URL(req.url, "http://localhost");
    let pathname = decodeURIComponent(url.pathname);
    if (pathname.endsWith("/")) pathname += "index.html";
    onRequest?.(pathname);

    // Contain the server in its root: normalize away any `..` BEFORE joining, then
    // confirm the result is still inside. A dev server is still a server.
    const target = join(rootDir, normalize(pathname).replace(/^(\.\.[/\\])+/, ""));
    if (target !== rootDir && !target.startsWith(rootDir + sep)) {
      res.writeHead(403).end("forbidden");
      return;
    }

    let size;
    try {
      const st = statSync(target);
      if (st.isDirectory()) throw new Error("is a directory");
      size = st.size;
    } catch {
      res.writeHead(404, { "content-type": "text/plain" }).end(`404 ${pathname}`);
      return;
    }

    res.writeHead(200, {
      "content-type": MIME[extname(target).toLowerCase()] ?? "application/octet-stream",
      "content-length": size,
      // A demo you are actively rebuilding — never hand back a stale .wasm.
      "cache-control": "no-store",
    });
    if (req.method === "HEAD") {
      res.end();
      return;
    }
    createReadStream(target).pipe(res);
  });

  return new Promise((ok, err) => {
    server.once("error", err);
    server.listen(port, host, () => {
      ok({ server, url: `http://${host}:${server.address().port}` });
    });
  });
}

// Run directly: `node scripts/serve.mjs <root> [port]`.
if (import.meta.url === `file://${process.argv[1]}`) {
  const root = process.argv[2] ?? ".";
  const port = Number(process.argv[3] ?? 8080);
  const { url } = await startServer({ root, port });
  console.log(`\n  crustyimg demo → ${url}\n`);
  console.log("  (it must be served, not opened as file:// — see demo/README.md)");
  console.log("  Ctrl-C to stop.\n");
}
