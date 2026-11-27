// Headless Chrome, driven over the DevTools Protocol: find a binary, start it, and a
// minimal client. Shared by the checks that load a page in a real browser
// (tests/demo_smoke.mjs and tests/ddphotos_page.mjs). No Puppeteer or Playwright: a
// check whose whole point is "no toolchain, no install" should not need a 300 MB
// browser driver.

import { spawn } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { delimiter, join } from "node:path";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ── find Chrome ───────────────────────────────────────────────────────────────

/// A Chrome or Chromium binary, or undefined. `CHROME` wins, as a path or as a name on
/// PATH; one that does not exist is reported (undefined), never handed to `spawn`,
/// where its ENOENT would crash the process with a stack instead of a message.
export function findChrome() {
  const named = process.env.CHROME;
  if (named) {
    if (named.includes("/")) return existsSync(named) ? named : undefined;
    return (process.env.PATH ?? "")
      .split(delimiter)
      .map((dir) => join(dir, named))
      .find((p) => existsSync(p));
  }
  const candidates = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
  ];
  return candidates.find((p) => existsSync(p));
}

// ── a minimal DevTools Protocol client ────────────────────────────────────────

export class CDP {
  #ws;
  #next = 1;
  #pending = new Map();
  handlers = new Map();

  static async attach(wsUrl) {
    const cdp = new CDP();
    cdp.#ws = new WebSocket(wsUrl);
    cdp.#ws.addEventListener("message", (ev) => cdp.#onMessage(JSON.parse(ev.data)));
    await new Promise((res, rej) => {
      cdp.#ws.addEventListener("open", res, { once: true });
      cdp.#ws.addEventListener("error", rej, { once: true });
    });
    return cdp;
  }

  #onMessage(msg) {
    if (msg.id !== undefined) {
      const p = this.#pending.get(msg.id);
      this.#pending.delete(msg.id);
      if (msg.error) p?.rej(new Error(`${msg.error.message} (${JSON.stringify(msg.error.data)})`));
      else p?.res(msg.result);
      return;
    }
    // Events from an attached target carry its sessionId; events from the page do
    // not. The handlers below are keyed by METHOD, so a worker's console error and
    // a page's console error land in the same list — which is what we want.
    this.handlers.get(msg.method)?.(msg.params, msg.sessionId);
  }

  /// `sessionId` addresses an auto-attached target — the engine's Web Worker is one
  /// (SPEC-078), and it is a genuinely separate target with its own Runtime and its
  /// own Network domain. Without this, the worker's traffic (including the .wasm it
  /// fetches) is invisible from the page's session.
  send(method, params = {}, sessionId) {
    const id = this.#next++;
    this.#ws.send(JSON.stringify(sessionId ? { id, method, params, sessionId } : { id, method, params }));
    return new Promise((res, rej) => this.#pending.set(id, { res, rej }));
  }

  on(method, fn) {
    this.handlers.set(method, fn);
  }

  close() {
    this.#ws.close();
  }

  /// Evaluate an expression in the page and return its value. `await`s a promise
  /// result, and turns a page-side throw into a real rejection here.
  async eval(expression) {
    const r = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) {
      throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    }
    return r.result.value;
  }
}

// ── starting Chrome ───────────────────────────────────────────────────────────

// Chrome startup gets the same budget as every other wait in the checks that use it
// (90 s). It previously had 100 x 100 ms = 10 s — an order of magnitude tighter than
// its siblings, for no stated reason — and a cold Chrome on a loaded CI runner misses
// 10 s routinely. Measured: this was the sole cause of three `pages / build + browser
// smoke` failures on 2026-08-21, including one on a DOCS-ONLY commit and one on
// `main`, each dying at almost exactly 10.03 s after the server came up. The
// identical tree passed on re-run.
const CHROME_STARTUP_MS = 90_000;

/// Start headless Chrome with its profile in `profileDir`, and attach to its page.
/// Returns `{ chrome, cdp, exited, port }` (`port` is the DevTools port, for a second
/// target on the same browser), where `exited` is a promise that resolves, with a
/// description of how it ended, once the process is gone. The caller owns the profile
/// folder: wait for `exited` before deleting it, or Chrome is still writing into it.
/// Throws, with Chrome already stopped, when Chrome never comes up.
export async function launchChrome(chromePath, profileDir) {
  const chrome = spawn(
    chromePath,
    [
      "--headless=new",
      "--disable-gpu",
      "--no-first-run",
      "--no-default-browser-check",
      "--disable-extensions",
      "--remote-allow-origins=*",
      "--remote-debugging-port=0",
      `--user-data-dir=${profileDir}`,
      ...(process.env.CI ? ["--no-sandbox"] : []),
      "about:blank",
    ],
    { stdio: ["ignore", "ignore", "pipe"] },
  );

  // Drain Chrome's stderr. Two reasons, and the second is not cosmetic: an
  // un-drained pipe fills its buffer (~64 KB) and then BLOCKS the writer, so a
  // chatty Chrome could hang before it ever writes DevToolsActivePort. Keep only
  // the tail — the useful line ("Failed to launch", a sandbox refusal, a missing
  // shared library) is always near the end.
  let chromeErr = "";
  chrome.stderr.setEncoding("utf8");
  chrome.stderr.on("data", (d) => {
    chromeErr = (chromeErr + d).slice(-4000);
  });

  // Fail fast if Chrome dies rather than waiting out the whole budget for a
  // port file that can never appear.
  let chromeExit = null;
  let settle;
  const exited = new Promise((r) => (settle = r));
  chrome.on("exit", (code, signal) => {
    chromeExit = signal ? `signal ${signal}` : `exit code ${code}`;
    settle(chromeExit);
  });
  // A binary that cannot be started (a bad path, no permission) emits `error` and
  // never `exit`; left unhandled it would crash this process with a stack.
  chrome.on("error", (e) => {
    chromeExit = `spawn failed: ${e.message}`;
    settle(chromeExit);
  });

  const portFile = join(profileDir, "DevToolsActivePort");
  let devtoolsPort;
  const startupDeadline = Date.now() + CHROME_STARTUP_MS;
  while (!devtoolsPort && chromeExit === null && Date.now() < startupDeadline) {
    await sleep(100);
    if (existsSync(portFile)) devtoolsPort = readFileSync(portFile, "utf8").split("\n")[0].trim();
  }
  if (!devtoolsPort) {
    const why =
      chromeExit !== null
        ? `Chrome exited (${chromeExit}) before writing DevToolsActivePort`
        : `no DevToolsActivePort after ${CHROME_STARTUP_MS / 1000}s`;
    chrome.kill();
    await exited;
    throw new Error(
      `headless Chrome never came up — ${why}\n` +
        `  binary: ${chromePath}\n` +
        `  chrome stderr (tail):\n${chromeErr.trim() || "    (nothing on stderr)"}`,
    );
  }

  try {
    const targets = await (await fetch(`http://127.0.0.1:${devtoolsPort}/json/list`)).json();
    const page = targets.find((t) => t.type === "page");
    if (!page) throw new Error("headless Chrome exposed no page target");
    const cdp = await CDP.attach(page.webSocketDebuggerUrl);
    return { chrome, cdp, exited, port: devtoolsPort };
  } catch (e) {
    chrome.kill();
    await exited;
    throw e;
  }
}
