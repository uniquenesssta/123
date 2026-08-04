import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { captureHtmlScreenshot, resolveChromiumExecutable } from "./task-ui-screenshot-tools.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(fs.readFileSync(path.join(root, "contracts/task-ui-contract.json"), "utf8"));
const updateBaselines = process.argv.includes("--update");
const requestedOutput = process.argv.find((argument) => argument.startsWith("--output="))?.slice("--output=".length);
const outputDirectory = requestedOutput
  ? path.resolve(requestedOutput)
  : path.join(root, "tests/ui", updateBaselines ? "baselines" : "actual");
const browser = resolveChromiumExecutable();
if (!browser) {
  console.error("未找到 Chromium、Chrome 或 Edge。可通过 CHROME_PATH 指定浏览器可执行文件。");
  process.exit(2);
}

for (const item of contract.screenshot_cases) {
  const outputPath = path.join(outputDirectory, `${item.name}.png`);
  await captureHtmlScreenshot({
    browser,
    htmlPath: path.join(root, "tests/ui", item.fixture),
    outputPath,
    width: item.width,
    height: item.height,
  });
  console.log(`已生成 ${path.relative(root, outputPath)} (${item.width}x${item.height})`);
}
