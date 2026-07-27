import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawn } from "node:child_process";

const baseUrl = process.env.MYND_URL ?? "http://127.0.0.1:4280";
const browserBinary = process.env.CHROMIUM_BIN ?? "chromium";
const debuggingPort = 9328;
const smokeMessage = `browser smoke ${Date.now()}`;
const profile = await mkdtemp(join(tmpdir(), "mynd-browser-smoke-"));
let createdId;
const browser = spawn(
  browserBinary,
  [
    "--headless",
    "--disable-gpu",
    "--no-sandbox",
    `--remote-debugging-port=${debuggingPort}`,
    `--user-data-dir=${profile}`,
    baseUrl,
  ],
  { stdio: "ignore" },
);

try {
  const target = await waitFor(async () => {
    const response = await fetch(`http://127.0.0.1:${debuggingPort}/json`);
    const targets = await response.json();
    return targets.find((candidate) => candidate.type === "page" && candidate.url.startsWith(baseUrl));
  });
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });

  let sequence = 0;
  const pending = new Map();
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (!message.id || !pending.has(message.id)) return;
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(message.error.message));
    else resolve(message.result);
  });

  const command = (method, params = {}) =>
    new Promise((resolve, reject) => {
      const id = ++sequence;
      pending.set(id, { resolve, reject });
      socket.send(JSON.stringify({ id, method, params }));
    });
  const evaluate = async (expression) => {
    const result = await command("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  };

  await command("Runtime.enable");
  await command("Page.enable");
  await waitFor(() => evaluate("Boolean(document.querySelector('#new-todo'))"));
  const { installabilityErrors } = await command("Page.getInstallabilityErrors");
  if (installabilityErrors.length) {
    throw new Error(`PWA is not installable: ${JSON.stringify(installabilityErrors)}`);
  }
  await waitFor(() => evaluate("Boolean(document.querySelector('.install-invite'))"));
  await evaluate(`(() => {
    const input = document.querySelector('#new-todo');
    input.value = ${JSON.stringify(smokeMessage)};
    input.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('.capture-panel form').requestSubmit();
  })()`);
  await waitFor(() => evaluate(`document.body.textContent.includes(${JSON.stringify(smokeMessage)})`));
  const todos = await fetch(`${baseUrl}/api/todos`).then((response) => response.json());
  createdId = todos.find((todo) => todo.message === smokeMessage)?.id;
  if (!createdId) throw new Error("Browser-created todo was not persisted");
  await evaluate("document.querySelector('.share-button').click()");
  await waitFor(() =>
    evaluate("document.querySelector('.share-dialog').open && document.querySelector('.qr-frame canvas').width > 0"),
  );
  socket.close();
  console.log("Browser smoke passed: capture, render, mobile dialog, and local QR code.");
} finally {
  if (createdId) {
    await fetch(`${baseUrl}/api/todos/${encodeURIComponent(createdId)}`, { method: "DELETE" });
  }
  if (browser.exitCode === null) {
    browser.kill("SIGTERM");
    await new Promise((resolve) => {
      browser.once("exit", resolve);
      setTimeout(resolve, 2_000);
    });
  }
  await rm(profile, { recursive: true, force: true });
}

async function waitFor(operation, timeoutMs = 10_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const value = await operation();
      if (value) return value;
    } catch {
      // The browser or page may still be starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("Timed out waiting for browser smoke condition");
}
