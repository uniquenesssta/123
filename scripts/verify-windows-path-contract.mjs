import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => readFileSync(join(root, relative), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const launcher = read("验收平台.bat");
const runner = read("scripts/windows-acceptance.ps1");
const pathModule = read("scripts/windows/acceptance-paths.psm1");
const cargoConfig = read(".cargo/config.toml");
const cargoTargetPreparation = read("scripts/prepare-cargo-target.mjs");

check(
  launcher.includes("-Mode Full") && launcher.includes("-LogDirectory .\\logs"),
  "根验收入口必须显式传递 Full 模式和相对日志目录",
);
check(/\[string\]\$LogDirectory\s*=\s*""/.test(runner), "PowerShell 验收器未声明 LogDirectory 参数");
check(
  runner.includes('Import-Module -Name $PathModulePath')
    && runner.includes("Resolve-AcceptanceLogRoot -ProjectRoot $ProjectRoot -LogDirectory $LogDirectory"),
  "PowerShell 验收器未通过路径模块解析日志目录",
);
check(
  runner.includes("$RuntimeLogRoot = Join-Path $ProjectRoot \"logs\"")
    && runner.includes("Get-ChildItem $RuntimeLogRoot"),
  "验收日志目录与应用运行日志目录未保持独立职责",
);
check(
  runner.includes("Resolve-CargoTargetRoot -SourceRoot $SourceRoot")
    && runner.includes("Find-AcceptanceReleaseExecutable -CargoTargetRoot $CargoTargetRoot"),
  "PowerShell 验收器未通过 Cargo 目标目录契约查找 release",
);
check(!runner.includes('Join-Path $SourceRoot ".cargo-target\\release"'), "仍在源码目录内查找错误的 Cargo release 路径");

for (const marker of [
  "function Resolve-AcceptanceLogRoot",
  "function Resolve-CargoTargetRoot",
  "function Find-AcceptanceReleaseExecutable",
  '".cargo\\target-location.json"',
  '"..\\.cargo-target"',
  'Join-Path $resolvedTargetRoot "release"',
  "Export-ModuleMember",
]) {
  check(pathModule.includes(marker), `Windows 路径模块缺少：${marker}`);
}
check(
  pathModule.includes("[IO.Path]::IsPathRooted($LogDirectory)")
    && pathModule.includes("Join-Path $resolvedProjectRoot $LogDirectory"),
  "日志目录未同时支持绝对路径和相对项目根目录路径",
);
check(
  /target-dir\s*=\s*"\.\.\/\.cargo-target"/.test(cargoConfig),
  "Cargo 配置的目标目录契约已变化",
);
check(
  cargoTargetPreparation.includes('path.resolve(SOURCE_ROOT, "..", ".cargo-target")')
    && cargoTargetPreparation.includes('"target_root"') === false,
  "Cargo 目标目录准备器结构与路径契约不一致",
);
check(
  cargoTargetPreparation.includes("target_root: path.resolve(TARGET_ROOT)"),
  "Cargo 目标目录登记文件未写入绝对 target_root",
);

if (failures.length) {
  console.error("Windows 路径契约验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("Windows 路径契约验证通过：根入口日志参数、运行日志边界和 Cargo release 查找路径一致。");
