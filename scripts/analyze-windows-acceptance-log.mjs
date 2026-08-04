import { readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
function argument(name, fallback = null) {
  const prefix = `--${name}=`;
  const inline = process.argv.find((item) => item.startsWith(prefix));
  if (inline) return inline.slice(prefix.length);
  const index = process.argv.indexOf(`--${name}`);
  return index >= 0 ? process.argv[index + 1] ?? fallback : fallback;
}
const contractPath = resolve(argument("contract", join(root, "contracts/windows-acceptance-contract.json")));
const logArgument = argument("log");
const profileName = argument("profile", "full");
if (!logArgument) {
  console.error("缺少 --log <football-runtime-*.jsonl> 参数");
  process.exit(2);
}
const logPath = resolve(logArgument);
const contract = JSON.parse(readFileSync(contractPath, "utf8"));
const profile = contract.profiles?.[profileName];
if (!profile) {
  console.error(`未知验收档案：${profileName}`);
  process.exit(2);
}
const lines = readFileSync(logPath, "utf8").split(/\r?\n/).filter(Boolean);
const entries = [];
const invalidLines = [];
for (let index = 0; index < lines.length; index += 1) {
  try { entries.push(JSON.parse(lines[index])); }
  catch (error) { invalidLines.push({ line: index + 1, error: String(error) }); }
}
const completedOperations = new Set();
for (const entry of entries) {
  if (entry?.event === "operation_completed" && typeof entry?.details?.operation === "string") {
    completedOperations.add(entry.details.operation);
  }
}
function evaluateGroup(group) {
  const allOf = Array.isArray(group.all_of) ? group.all_of : [];
  const anyOf = Array.isArray(group.any_of) ? group.any_of : [];
  const alternatives = Array.isArray(group.alternatives) ? group.alternatives : [];
  const allPassed = allOf.every((operation) => completedOperations.has(operation));
  const anyPassed = anyOf.length === 0 || anyOf.some((operation) => completedOperations.has(operation));
  const alternativePassed = alternatives.length === 0 || alternatives.some((alternative) =>
    Array.isArray(alternative) && alternative.every((operation) => completedOperations.has(operation))
  );
  const passed = allPassed && anyPassed && alternativePassed;
  const expectedOperations = new Set([...allOf, ...anyOf, ...alternatives.flat()]);
  const expected = [
    ...(allOf.length ? [`全部：${allOf.join(", ")}`] : []),
    ...(anyOf.length ? [`任一：${anyOf.join(", ")}`] : []),
    ...(alternatives.length ? [`路径：${alternatives.map((item) => item.join(" + ")).join(" 或 ")}`] : []),
  ];
  return { id: group.id, label: group.label, required: group.required !== false, passed, expected, matched: [...expectedOperations].filter((operation) => completedOperations.has(operation)).sort() };
}
const coverage = profile.required_operation_groups.map(evaluateGroup);
const forbiddenLevels = new Set(contract.forbidden_runtime_levels ?? ["error", "critical"]);
const rawErrors = entries.filter((entry) => forbiddenLevels.has(String(entry?.level ?? "").toLowerCase()));
const errorMap = new Map();
for (const entry of rawErrors) {
  const details = entry.details ?? {};
  const message = details.technical_message ?? details.user_message ?? details.error ?? details.context?.error ?? null;
  const signature = JSON.stringify({
    operation: details.operation ?? null,
    message: message ?? `${entry.subsystem ?? "unknown"}:${entry.event ?? "unknown"}`,
  });
  if (!errorMap.has(signature)) errorMap.set(signature, { count: 0, first_sequence: entry.sequence ?? null, last_sequence: entry.sequence ?? null, sample: entry });
  const item = errorMap.get(signature);
  item.count += 1;
  item.last_sequence = entry.sequence ?? item.last_sequence;
}
const errors = [...errorMap.values()];
const missingRequired = coverage.filter((item) => item.required && !item.passed);
const missingRecommended = coverage.filter((item) => !item.required && !item.passed);
const status = invalidLines.length || errors.length || missingRequired.length ? "blocked" : missingRecommended.length ? "warning" : "pass";
const reportPath = resolve(argument("report", join(dirname(logPath), `${basename(logPath, ".jsonl")}.acceptance.json`)));
const report = {
  schema: contract.report_schema,
  contract_id: contract.contract_id,
  contract_version: contract.contract_version,
  generated_at: new Date().toISOString(),
  profile: profileName,
  status,
  source_log: logPath,
  session_ids: [...new Set(entries.map((entry) => entry.session_id).filter(Boolean))],
  app_versions: [...new Set(entries.map((entry) => entry.app_version).filter(Boolean))],
  entry_count: entries.length,
  invalid_line_count: invalidLines.length,
  invalid_lines: invalidLines,
  completed_operations: [...completedOperations].sort(),
  coverage,
  missing_required_groups: missingRequired.map((item) => item.id),
  missing_recommended_groups: missingRecommended.map((item) => item.id),
  runtime_errors: errors,
};
writeFileSync(reportPath, JSON.stringify(report, null, 2) + "\n", "utf8");
console.log(`运行日志验收：${status.toUpperCase()} · ${entries.length} 条记录 · ${completedOperations.size} 个完成操作`);
for (const item of coverage) console.log(`${item.passed ? "[通过]" : item.required ? "[缺失]" : "[建议]"} ${item.label}`);
if (invalidLines.length) console.error(`日志包含 ${invalidLines.length} 行无效 JSON`);
if (errors.length) console.error(`日志包含 ${errors.length} 类 error/critical 事件`);
console.log(`验收报告：${reportPath}`);
process.exit(status === "blocked" ? 1 : 0);
