import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { comparePng, resolveChromiumExecutable, temporaryScreenshotDirectory } from "./task-ui-screenshot-tools.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(fs.readFileSync(path.join(root, "contracts/task-ui-contract.json"), "utf8"));
const browser = resolveChromiumExecutable();
if (!browser) {
  console.error("截图回归无法执行：未找到 Chromium、Chrome 或 Edge。可通过 CHROME_PATH 指定浏览器。");
  process.exit(2);
}
const actualDirectory = temporaryScreenshotDirectory();
const capture = spawnSync(process.execPath, [path.join(root, "scripts/capture-task-ui.mjs"), `--output=${actualDirectory}`], { encoding: "utf8", timeout: 180_000 });
if (capture.status !== 0) {
  console.error(capture.stderr || capture.stdout || "截图生成失败");
  process.exit(1);
}
const failures = [];
for (const item of contract.screenshot_cases) {
  const baseline = path.join(root, "tests/ui/baselines", `${item.name}.png`);
  const actual = path.join(actualDirectory, `${item.name}.png`);
  if (!fs.existsSync(baseline)) {
    failures.push(`${item.name}: 缺少基线`);
    continue;
  }
  const result = comparePng(baseline, actual);
  if (!result.passed) failures.push(`${item.name}: ${result.reason}`);
  else console.log(`${item.name}: ${result.reason}`);
}
fs.rmSync(actualDirectory, { recursive: true, force: true });
if (failures.length) {
  console.error("任务型 UI 截图回归失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  console.error("确认设计变更后，可运行 npm run capture:task-ui -- --update 更新基线。");
  process.exit(1);
}
console.log(`任务型 UI 截图回归通过：${contract.screenshot_cases.length} 个视口。`);
