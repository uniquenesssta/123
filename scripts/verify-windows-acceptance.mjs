import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outer = resolve(root, "..");
const read = (relative) => readFileSync(join(root, relative), "utf8");
const json = (relative) => JSON.parse(read(relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const contract = json("contracts/windows-acceptance-contract.json");
const pkg = json("package.json");
const reportSchema = json(contract.report_schema_file);
const powershell = read(contract.powershell_runner);
const analyzer = read(contract.runtime_analyzer);
const launcher = readFileSync(join(root, contract.root_launcher), "utf8");
const releasePackageVerifier = read("scripts/verify-release-package.mjs");
const testing = read("docs/TESTING.md");
const readme = readFileSync(join(root, "README.md"), "utf8");

check(contract.contract_id === "football.windows-end-to-end-acceptance.v1", "Windows验收契约ID错误");
check(contract.platform === "win32", "Windows验收平台边界错误");
check(reportSchema.properties?.status?.enum?.includes("blocked") && reportSchema.required?.includes("coverage"), "Windows验收报告Schema不完整");
check(contract.profiles?.full?.required_operation_groups?.length >= 9, "Full验收链覆盖不足");
for (const stage of ["environment_preflight","frontend_contracts_and_build","rust_fmt_clippy_tests","postgres_integration","tauri_release_build","runtime_smoke","runtime_log_analysis"]) {
  check(contract.automated_stages.includes(stage), `验收阶段缺失：${stage}`);
}
for (const marker of [
  'Invoke-Stage "前端契约、类型、截图与生产构建" "npm.cmd" @("run", "verify:frontend")',
  'Invoke-Stage "Rust 格式、Clippy 与工作区测试" "npm.cmd" @("run", "verify:rust")',
  '"postgres_integration"',
  'Invoke-Stage "Tauri Windows release 构建" "npm.cmd" @("run", "tauri:build")',
  'analyze-windows-acceptance-log.mjs',
  'FOOTBALL_TEST_DATABASE_URL',
  'FOOTBALL_RUNTIME_ROOT',
  'FOOTBALL_PROJECT_ROOT',
]) {
  check(powershell.includes(marker), `PowerShell验收器缺少：${marker}`);
}
check(powershell.includes("AcceptanceContract.database_safety.required_name_pattern") && powershell.includes("AcceptanceContract.minimum_versions.node"), "验收器未从机器契约读取数据库与工具链门禁");
check(!powershell.includes("Write-AcceptanceLog \"测试数据库 URL"), "验收日志不得写入完整数据库URL");
check(analyzer.includes("operation_completed") && analyzer.includes("forbidden_runtime_levels"), "运行日志分析器未校验完成操作或错误等级");
check(launcher.includes("windows-acceptance.ps1") && launcher.includes("-Mode Full") && launcher.includes(".\\logs"), "根目录验收入口未启动Full模式或未使用相对日志目录");
check(pkg.scripts?.["acceptance:windows"] === "powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-acceptance.ps1", "package.json缺少Windows验收执行命令");
check(pkg.scripts?.["verify:windows-acceptance"] === "node scripts/verify-windows-acceptance.mjs", "package.json缺少Windows验收契约命令");
check(pkg.scripts.build.includes("verify-frontend.mjs") && read("scripts/verify-frontend.mjs").includes("verify-windows-acceptance.mjs"), "Windows验收契约未进入正式前端门禁");
check(releasePackageVerifier.includes('"验收平台.bat"'), "发布包洁净度门禁未允许验收入口");
check(testing.includes("Windows 全链路验收") && readme.includes("Windows实机全链路验收阶段 5"), "验收说明或README维护记录缺失");

const temporary = mkdtempSync(join(tmpdir(), "football-windows-acceptance-"));
try {
  const analyzerPath = join(root, contract.runtime_analyzer);
  const fullGroups = contract.profiles.full.required_operation_groups;
  const operations = [];
  for (const group of fullGroups.filter((item) => item.required !== false)) {
    if (group.all_of) operations.push(...group.all_of);
    else if (group.any_of) operations.push(group.any_of[0]);
    else if (group.alternatives) operations.push(...group.alternatives[0]);
  }
  function writeLog(name, selectedOperations, extra = []) {
    const path = join(temporary, name);
    const rows = [
      { timestamp_utc: new Date().toISOString(), session_id: "test-session", sequence: 1, level: "info", subsystem: "application", event: "application_started", app_version: "0.23.0", trace_id: null, details: {} },
      ...selectedOperations.map((operation, index) => ({ timestamp_utc: new Date().toISOString(), session_id: "test-session", sequence: index + 2, level: "info", subsystem: "frontend.operation", event: "operation_completed", app_version: "0.23.0", trace_id: null, details: { operation, duration_ms: 1 } })),
      ...extra,
    ];
    writeFileSync(path, rows.map((item) => JSON.stringify(item)).join("\n") + "\n", "utf8");
    return path;
  }
  const passLog = writeLog("pass.jsonl", [...new Set(operations)]);
  const passReport = join(temporary, "pass-report.json");
  const pass = spawnSync(process.execPath, [analyzerPath, "--log", passLog, "--profile", "full", "--report", passReport], { encoding: "utf8" });
  check(pass.status === 0, `完整合成日志应通过：${pass.stderr || pass.stdout}`);
  if (pass.status === 0) {
    const parsedPassReport = JSON.parse(readFileSync(passReport, "utf8"));
    check(parsedPassReport.status === "warning" || parsedPassReport.status === "pass", "通过报告状态错误");
    for (const required of reportSchema.required) check(Object.hasOwn(parsedPassReport, required), `通过报告缺少Schema字段：${required}`);
  }
  const missingLog = writeLog("missing.jsonl", [...new Set(operations)].slice(1));
  const missing = spawnSync(process.execPath, [analyzerPath, "--log", missingLog, "--profile", "full", "--report", join(temporary, "missing-report.json")], { encoding: "utf8" });
  check(missing.status !== 0, "缺少必需操作的合成日志必须失败");
  const errorLog = writeLog("error.jsonl", [...new Set(operations)], [{ timestamp_utc: new Date().toISOString(), session_id: "test-session", sequence: 999, level: "error", subsystem: "frontend.operation", event: "operation_failed", app_version: "0.23.0", trace_id: null, details: { operation: "execute_prediction_from_match", error: "synthetic" } }]);
  const errorResult = spawnSync(process.execPath, [analyzerPath, "--log", errorLog, "--profile", "full", "--report", join(temporary, "error-report.json")], { encoding: "utf8" });
  check(errorResult.status !== 0, "包含error事件的合成日志必须失败");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}

if (failures.length) {
  console.error("Windows实机全链路验收体系验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`Windows实机全链路验收体系验证通过：${contract.automated_stages.length}个自动阶段、${contract.profiles.full.required_operation_groups.length}组运行链覆盖和三类合成日志反向门禁完整。`);
