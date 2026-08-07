import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { isDirectExecution } from "./process/execution-context.mjs";
import { resolveNodePackageCli, spawnNodePackageCli } from "./process/node-package-cli.mjs";

const temporary = mkdtempSync(join(tmpdir(), "football-node-process-"));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

try {
  const physical = join(temporary, "physical");
  const alias = join(temporary, "alias");
  const modulePath = join(physical, "entry.mjs");
  mkdirSync(physical, { recursive: true });
  writeFileSync(modulePath, "export const ok = true;\n", "utf8");
  symlinkSync(physical, alias, process.platform === "win32" ? "junction" : "dir");
  check(
    isDirectExecution(pathToFileURL(modulePath).href, join(alias, "entry.mjs")),
    "直接执行检测未兼容目录联接或符号链接",
  );

  const sourceRoot = join(temporary, "project");
  const packageRoot = join(temporary, "node_modules", "sample-cli", "bin");
  const cliPath = join(packageRoot, "cli.mjs");
  const outputPath = join(temporary, "args.json");
  mkdirSync(sourceRoot, { recursive: true });
  mkdirSync(packageRoot, { recursive: true });
  writeFileSync(
    cliPath,
    "import { writeFileSync } from 'node:fs';\n" +
      "const [output, ...args] = process.argv.slice(2);\n" +
      "if (args.includes('--fail')) process.exit(7);\n" +
      "writeFileSync(output, JSON.stringify(args), 'utf8');\n",
    "utf8",
  );

  check(
    resolveNodePackageCli(sourceRoot, "sample-cli", "bin/cli.mjs") === cliPath,
    "上一级 Node CLI 路径解析错误",
  );
  const success = spawnNodePackageCli({
    root: sourceRoot,
    packageName: "sample-cli",
    executablePath: "bin/cli.mjs",
    args: [outputPath, "alpha", "beta"],
    options: { stdio: "pipe", encoding: "utf8" },
  });
  check(success.error == null && success.status === 0, `Node CLI 成功路径退出异常：${success.error?.message ?? success.status}`);
  check(readFileSync(outputPath, "utf8") === '["alpha","beta"]', "Node CLI 参数传递错误");

  const failure = spawnNodePackageCli({
    root: sourceRoot,
    packageName: "sample-cli",
    executablePath: "bin/cli.mjs",
    args: [outputPath, "--fail"],
    options: { stdio: "pipe", encoding: "utf8" },
  });
  check(failure.error == null && failure.status === 7, "Node CLI 非零退出码未原样保留");

  let missingRejected = false;
  try {
    resolveNodePackageCli(sourceRoot, "sample-cli", "bin/missing.mjs");
  } catch {
    missingRejected = true;
  }
  check(missingRejected, "缺失的 Node CLI 未被拒绝");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}

if (failures.length) {
  console.error("Windows Node 调用链兼容验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("Windows Node 调用链兼容验证通过：目录联接、上一级 Node CLI、参数和退出码均保持一致。");
