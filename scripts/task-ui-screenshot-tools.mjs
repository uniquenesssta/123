import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import zlib from "node:zlib";
import http from "node:http";
import { randomBytes } from "node:crypto";
import { spawn } from "node:child_process";

function existingFile(candidates) {
  return candidates.find((candidate) => candidate && fs.existsSync(candidate)) ?? null;
}

export function resolveChromiumExecutable() {
  const configured = process.env.CHROME_PATH || process.env.CHROMIUM_PATH || process.env.EDGE_PATH;
  const platformCandidates = process.platform === "win32"
    ? [
        configured,
        path.join(process.env.PROGRAMFILES ?? "C:\\Program Files", "Microsoft/Edge/Application/msedge.exe"),
        path.join(process.env["PROGRAMFILES(X86)"] ?? "C:\\Program Files (x86)", "Microsoft/Edge/Application/msedge.exe"),
        path.join(process.env.PROGRAMFILES ?? "C:\\Program Files", "Google/Chrome/Application/chrome.exe"),
        path.join(process.env.LOCALAPPDATA ?? "", "Google/Chrome/Application/chrome.exe"),
      ]
    : process.platform === "darwin"
      ? [configured, "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge", "/Applications/Chromium.app/Contents/MacOS/Chromium"]
      : [configured, "/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/microsoft-edge"];
  return existingFile(platformCandidates);
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function requestJson({ port, path: requestPath, method = "GET" }) {
  return new Promise((resolve, reject) => {
    const request = http.request({ host: "127.0.0.1", port, path: requestPath, method }, (response) => {
      let body = "";
      response.setEncoding("utf8");
      response.on("data", (chunk) => { body += chunk; });
      response.on("end", () => {
        if ((response.statusCode ?? 500) >= 400) reject(new Error(`CDP HTTP ${response.statusCode}: ${body}`));
        else {
          try { resolve(JSON.parse(body)); }
          catch (error) { reject(new Error(`无法解析 CDP 响应：${error instanceof Error ? error.message : String(error)}`)); }
        }
      });
    });
    request.on("error", reject);
    request.end();
  });
}

function inlineFixtureStyles(htmlPath) {
  let html = fs.readFileSync(htmlPath, "utf8");
  html = html.replace(/<link\s+rel=["']stylesheet["']\s+href=["']([^"']+)["']\s*\/?>/gi, (_match, href) => {
    const stylesheetPath = path.resolve(path.dirname(htmlPath), href);
    const css = fs.readFileSync(stylesheetPath, "utf8");
    return `<style data-inline-source="${href.replaceAll('"', "&quot;")}">${css}</style>`;
  });
  return html;
}

function encodeWebSocketFrame(text) {
  const payload = Buffer.from(text, "utf8");
  const mask = randomBytes(4);
  let header;
  if (payload.length < 126) {
    header = Buffer.alloc(2);
    header[1] = 0x80 | payload.length;
  } else if (payload.length <= 0xffff) {
    header = Buffer.alloc(4);
    header[1] = 0x80 | 126;
    header.writeUInt16BE(payload.length, 2);
  } else {
    header = Buffer.alloc(10);
    header[1] = 0x80 | 127;
    header.writeBigUInt64BE(BigInt(payload.length), 2);
  }
  header[0] = 0x81;
  const masked = Buffer.alloc(payload.length);
  for (let index = 0; index < payload.length; index += 1) masked[index] = payload[index] ^ mask[index % 4];
  return Buffer.concat([header, mask, masked]);
}

function connectWebSocket(webSocketUrl) {
  const url = new URL(webSocketUrl);
  return new Promise((resolve, reject) => {
    const request = http.request({
      host: url.hostname,
      port: Number(url.port),
      path: `${url.pathname}${url.search}`,
      headers: {
        Connection: "Upgrade",
        Upgrade: "websocket",
        "Sec-WebSocket-Version": "13",
        "Sec-WebSocket-Key": randomBytes(16).toString("base64"),
      },
    });
    request.once("error", reject);
    request.once("upgrade", (_response, socket, head) => {
      let buffer = head;
      let fragments = [];
      let fragmentOpcode = null;
      const listeners = new Set();
      const emitText = (payload) => {
        const text = Buffer.concat(payload).toString("utf8");
        for (const listener of listeners) listener(text);
      };
      const parseFrames = () => {
        while (buffer.length >= 2) {
          const first = buffer[0];
          const second = buffer[1];
          const fin = Boolean(first & 0x80);
          const opcode = first & 0x0f;
          const masked = Boolean(second & 0x80);
          let length = second & 0x7f;
          let offset = 2;
          if (length === 126) {
            if (buffer.length < 4) return;
            length = buffer.readUInt16BE(2);
            offset = 4;
          } else if (length === 127) {
            if (buffer.length < 10) return;
            const wideLength = buffer.readBigUInt64BE(2);
            if (wideLength > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error("WebSocket 帧过大");
            length = Number(wideLength);
            offset = 10;
          }
          const maskOffset = masked ? 4 : 0;
          if (buffer.length < offset + maskOffset + length) return;
          const mask = masked ? buffer.subarray(offset, offset + 4) : null;
          const payload = Buffer.from(buffer.subarray(offset + maskOffset, offset + maskOffset + length));
          buffer = buffer.subarray(offset + maskOffset + length);
          if (mask) for (let index = 0; index < payload.length; index += 1) payload[index] ^= mask[index % 4];
          if (opcode === 0x8) { socket.end(); return; }
          if (opcode === 0x9) continue;
          if (opcode === 0x1 || opcode === 0x2) {
            fragments = [payload];
            fragmentOpcode = opcode;
          } else if (opcode === 0x0 && fragmentOpcode !== null) fragments.push(payload);
          if (fin && fragmentOpcode === 0x1) {
            emitText(fragments);
            fragments = [];
            fragmentOpcode = null;
          }
        }
      };
      socket.on("data", (chunk) => { buffer = Buffer.concat([buffer, chunk]); parseFrames(); });
      socket.on("error", reject);
      resolve({
        send(text) { socket.write(encodeWebSocketFrame(text)); },
        onMessage(listener) { listeners.add(listener); },
        close() { socket.end(); },
      });
      parseFrames();
    });
    request.end();
  });
}

async function connectCdp(webSocketUrl) {
  const socket = await connectWebSocket(webSocketUrl);
  let nextId = 0;
  const pending = new Map();
  socket.onMessage((text) => {
    const message = JSON.parse(text);
    if (!message.id || !pending.has(message.id)) return;
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(`${message.error.message ?? "CDP 错误"}: ${JSON.stringify(message.error.data ?? {})}`));
    else resolve(message.result ?? {});
  });
  return {
    send(method, params = {}) {
      return new Promise((resolve, reject) => {
        const id = ++nextId;
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    close() { socket.close(); },
  };
}

export async function captureHtmlScreenshot({ browser, htmlPath, outputPath, width, height }) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  const profileDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "football-task-ui-browser-"));
  const args = [
    "--headless=new",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    "--hide-scrollbars",
    "--force-device-scale-factor=1",
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-port=0",
    "--remote-allow-origins=*",
    `--user-data-dir=${profileDirectory}`,
    "about:blank",
  ];
  if (process.platform === "linux" && typeof process.getuid === "function" && process.getuid() === 0) args.unshift("--no-sandbox");
  const child = spawn(browser, args, { stdio: ["ignore", "ignore", "pipe"] });
  let stderr = "";
  child.stderr?.on("data", (chunk) => { stderr += chunk.toString(); });
  const portFile = path.join(profileDirectory, "DevToolsActivePort");
  try {
    for (let attempt = 0; attempt < 150 && !fs.existsSync(portFile); attempt += 1) await sleep(100);
    if (!fs.existsSync(portFile)) throw new Error(`浏览器未开放调试端口。${stderr.slice(-1200)}`);
    const [portText] = fs.readFileSync(portFile, "utf8").trim().split(/\s+/);
    const port = Number(portText);
    const target = await requestJson({ port, path: "/json/new?about:blank", method: "PUT" });
    const cdp = await connectCdp(target.webSocketDebuggerUrl);
    try {
      await cdp.send("Page.enable");
      await cdp.send("Runtime.enable");
      await cdp.send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
      const frameTree = await cdp.send("Page.getFrameTree");
      await cdp.send("Page.setDocumentContent", { frameId: frameTree.frameTree.frame.id, html: inlineFixtureStyles(htmlPath) });
      await cdp.send("Runtime.evaluate", { expression: "document.fonts ? document.fonts.ready.then(() => true) : true", awaitPromise: true, returnByValue: true });
      await sleep(180);
      const screenshot = await cdp.send("Page.captureScreenshot", { format: "png", fromSurface: true, captureBeyondViewport: false });
      fs.writeFileSync(outputPath, Buffer.from(screenshot.data, "base64"));
      await cdp.send("Browser.close").catch(() => {});
    } finally {
      cdp.close();
    }
  } finally {
    for (let attempt = 0; attempt < 20 && child.exitCode === null; attempt += 1) await sleep(50);
    if (child.exitCode === null) child.kill("SIGKILL");
    await sleep(80);
    for (let attempt = 0; attempt < 5; attempt += 1) {
      try { fs.rmSync(profileDirectory, { recursive: true, force: true }); break; }
      catch { await sleep(100); }
    }
  }
}

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
}

export function decodePng(filePath) {
  const data = fs.readFileSync(filePath);
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  if (!data.subarray(0, 8).equals(signature)) throw new Error(`${filePath} 不是 PNG 文件`);
  let offset = 8;
  let width = 0;
  let height = 0;
  let bitDepth = 0;
  let colorType = 0;
  let interlace = 0;
  const idat = [];
  while (offset < data.length) {
    const length = data.readUInt32BE(offset);
    const type = data.toString("ascii", offset + 4, offset + 8);
    const chunk = data.subarray(offset + 8, offset + 8 + length);
    offset += 12 + length;
    if (type === "IHDR") {
      width = chunk.readUInt32BE(0);
      height = chunk.readUInt32BE(4);
      bitDepth = chunk[8];
      colorType = chunk[9];
      interlace = chunk[12];
    } else if (type === "IDAT") idat.push(chunk);
    else if (type === "IEND") break;
  }
  if (bitDepth !== 8 || interlace !== 0 || ![2, 6].includes(colorType)) {
    throw new Error(`${filePath} 使用了不支持的 PNG 格式（bitDepth=${bitDepth}, colorType=${colorType}, interlace=${interlace}）`);
  }
  const channels = colorType === 6 ? 4 : 3;
  const stride = width * channels;
  const inflated = zlib.inflateSync(Buffer.concat(idat));
  const pixels = Buffer.alloc(width * height * channels);
  let inputOffset = 0;
  for (let y = 0; y < height; y += 1) {
    const filter = inflated[inputOffset++];
    const rowOffset = y * stride;
    for (let x = 0; x < stride; x += 1) {
      const raw = inflated[inputOffset++];
      const left = x >= channels ? pixels[rowOffset + x - channels] : 0;
      const up = y > 0 ? pixels[rowOffset - stride + x] : 0;
      const upLeft = y > 0 && x >= channels ? pixels[rowOffset - stride + x - channels] : 0;
      let value;
      if (filter === 0) value = raw;
      else if (filter === 1) value = (raw + left) & 255;
      else if (filter === 2) value = (raw + up) & 255;
      else if (filter === 3) value = (raw + Math.floor((left + up) / 2)) & 255;
      else if (filter === 4) value = (raw + paeth(left, up, upLeft)) & 255;
      else throw new Error(`${filePath} 包含未知 PNG 过滤器 ${filter}`);
      pixels[rowOffset + x] = value;
    }
  }
  return { width, height, channels, pixels };
}

export function comparePng(referencePath, actualPath, options = {}) {
  const reference = decodePng(referencePath);
  const actual = decodePng(actualPath);
  if (reference.width !== actual.width || reference.height !== actual.height) {
    return { passed: false, reason: `尺寸不一致：${reference.width}x${reference.height} / ${actual.width}x${actual.height}` };
  }
  const channelThreshold = options.channelThreshold ?? 28;
  const maxDifferentRatio = options.maxDifferentRatio ?? 0.12;
  const maxMeanDifference = options.maxMeanDifference ?? 8;
  const pixelCount = reference.width * reference.height;
  let differentPixels = 0;
  let totalDifference = 0;
  for (let pixel = 0; pixel < pixelCount; pixel += 1) {
    let maxPixelDifference = 0;
    for (let channel = 0; channel < 3; channel += 1) {
      const refIndex = pixel * reference.channels + channel;
      const actualIndex = pixel * actual.channels + channel;
      const difference = Math.abs(reference.pixels[refIndex] - actual.pixels[actualIndex]);
      totalDifference += difference;
      if (difference > maxPixelDifference) maxPixelDifference = difference;
    }
    if (maxPixelDifference > channelThreshold) differentPixels += 1;
  }
  const differentRatio = differentPixels / pixelCount;
  const meanDifference = totalDifference / (pixelCount * 3);
  return {
    passed: differentRatio <= maxDifferentRatio && meanDifference <= maxMeanDifference,
    differentRatio,
    meanDifference,
    reason: `差异像素 ${(differentRatio * 100).toFixed(2)}%，平均通道差 ${meanDifference.toFixed(2)}`,
  };
}

export function temporaryScreenshotDirectory() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "football-task-ui-"));
}
