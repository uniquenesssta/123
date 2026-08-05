import { readFileSync } from "node:fs";
import { join } from "node:path";
import { findCycles, loadWorkspaceGraph, parseWorkspaceMembers } from "./lib/cargo.mjs";
import { parseJavaScriptImports, resolveRelativeImport } from "./lib/imports.mjs";
import {
  listFiles,
  matchesPathPattern,
  normalizePath,
  pathExists,
  readJson,
  repositoryRoot,
} from "./lib/repository.mjs";
import { VerificationReport } from "./lib/report.mjs";

const report = new VerificationReport("模块边界验证");
const contract = readJson("architecture/module-boundaries.json");
const stateContract = readJson("architecture/state-ownership.json");

report.check(contract.contract_id === "football.module-boundaries.v1", "模块边界 contract_id 不正确");
report.check(contract.status === "ACTIVE", `模块边界契约状态必须为 ACTIVE，当前为 ${contract.status}`);
report.check(contract.policy?.default === "deny-unlisted", "模块边界默认策略必须为 deny-unlisted");
report.check(contract.policy?.state_contract === "architecture/state-ownership.json", "模块边界未引用状态所有权契约");
report.check(stateContract.contract_id === "football.state-ownership.v1", "状态所有权契约引用无效");

const features = contract.frontend?.features ?? {};
const featureEntries = Object.entries(features);
const featureOwners = new Map();
for (const [featureName, feature] of featureEntries) {
  const owner = normalizePath(feature.owner ?? "");
  report.check(Boolean(owner), `Feature ${featureName} 缺少 owner`);
  report.check(pathExists(owner), `Feature ${featureName} owner 不存在：${owner}`);
  report.check(!featureOwners.has(owner), `Feature owner 重复：${owner}`);
  featureOwners.set(owner, featureName);
}
report.check(featureEntries.length === contract.counts?.frontend_features, "前端 Feature 数量与 counts.frontend_features 不一致");

const targetFeatureRoot = "src/features";
function featureForPath(filePath) {
  const normalized = normalizePath(filePath);
  const currentOwner = featureOwners.get(normalized);
  if (currentOwner) return { name: currentOwner, publicEntry: null };

  if (!normalized.startsWith(`${targetFeatureRoot}/`)) return null;
  const [, , featureName] = normalized.split("/");
  if (!features[featureName]) return null;
  const publicEntries = ["index.ts", "index.tsx", "index.js", "index.mjs"].map((entry) => `${targetFeatureRoot}/${featureName}/${entry}`);
  return { name: featureName, publicEntry: publicEntries.includes(normalized) ? normalized : null };
}

const frontendTransitions = contract.frontend?.transitional_imports ?? [];
const observedFrontendTransitions = new Set();
function transitionKey(transition) {
  return [
    transition.from_feature,
    normalizePath(transition.importer ?? ""),
    transition.to_feature,
    normalizePath(transition.target ?? ""),
  ].join("|");
}

for (const transition of frontendTransitions) {
  report.check(Boolean(features[transition.from_feature]), `过渡导入起点 Feature 不存在：${transition.from_feature}`);
  report.check(Boolean(features[transition.to_feature]), `过渡导入终点 Feature 不存在：${transition.to_feature}`);
  report.check(pathExists(transition.importer ?? ""), `过渡导入 importer 不存在：${transition.importer}`);
  report.check(pathExists(transition.target ?? ""), `过渡导入 target 不存在：${transition.target}`);
  report.check(Boolean(transition.reason), `过渡导入缺少 reason：${transitionKey(transition)}`);
  report.check(Boolean(transition.exit_task), `过渡导入缺少 exit_task：${transitionKey(transition)}`);
}

function isRegisteredFrontendTransition(importer, sourceFeature, targetFeature, resolvedTarget) {
  const key = [sourceFeature, normalizePath(importer), targetFeature, normalizePath(resolvedTarget)].join("|");
  const transition = frontendTransitions.find((candidate) => transitionKey(candidate) === key);
  if (!transition) return false;
  observedFrontendTransitions.add(key);
  report.note(`保留已登记过渡导入 ${importer} -> ${resolvedTarget}，退出任务 ${transition.exit_task}`);
  return true;
}

const frontendFiles = listFiles(["src"], { extensions: [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"] });
for (const importer of frontendFiles) {
  const sourceFeature = featureForPath(importer);
  if (!sourceFeature) continue;
  const source = readFileSync(join(repositoryRoot, importer), "utf8").replaceAll("\r\n", "\n");

  for (const specifier of parseJavaScriptImports(source)) {
    const resolvedTarget = resolveRelativeImport(importer, specifier);
    if (!resolvedTarget) continue;

    const targetFeature = featureForPath(resolvedTarget);
    if (targetFeature && targetFeature.name !== sourceFeature.name) {
      if (!targetFeature.publicEntry && !isRegisteredFrontendTransition(importer, sourceFeature.name, targetFeature.name, resolvedTarget)) {
        report.violation(`${importer} 直接导入 Feature ${targetFeature.name} 内部文件：${specifier} -> ${resolvedTarget}`);
      }
      continue;
    }

    for (const forbidden of contract.frontend.feature_rules?.forbidden ?? []) {
      if (forbidden.startsWith("@")) continue;
      if (matchesPathPattern(resolvedTarget, forbidden)) {
        report.violation(`${importer} 导入禁止路径 ${forbidden}：${specifier} -> ${resolvedTarget}`);
      }
    }
  }
}

for (const transition of frontendTransitions) {
  report.check(observedFrontendTransitions.has(transitionKey(transition)), `登记的过渡导入未在源码中出现，应删除或修正：${transitionKey(transition)}`);
}

const contractCrates = contract.rust?.crates ?? {};
const workspaceMembers = parseWorkspaceMembers();
const contractRoots = Object.values(contractCrates).map((item) => normalizePath(item.root)).sort();
report.check(JSON.stringify(workspaceMembers) === JSON.stringify(contractRoots), `Cargo workspace members 与契约不一致：workspace=${workspaceMembers.join(", ")} contract=${contractRoots.join(", ")}`);
report.check(Object.keys(contractCrates).length === contract.counts?.workspace_members, "Rust workspace 数量与 counts.workspace_members 不一致");

const { manifests, graph } = loadWorkspaceGraph(contractCrates);
for (const [packageName, definition] of Object.entries(contractCrates)) {
  const manifest = manifests.get(packageName);
  report.check(Boolean(manifest), `${packageName} 缺少 Cargo.toml`);
  if (!manifest) continue;
  report.check(manifest.packageName === packageName, `${manifest.path} package.name=${manifest.packageName}，契约为 ${packageName}`);

  const allowed = new Set(definition.allowed_workspace_dependencies ?? []);
  for (const dependency of graph.get(packageName) ?? []) {
    report.check(allowed.has(dependency), `${packageName} 存在未登记 workspace 依赖：${dependency}`);
  }
  for (const dependency of allowed) {
    report.check(Boolean(contractCrates[dependency]), `${packageName} 的允许依赖不存在于契约：${dependency}`);
  }
}

report.check((graph.get("football-domain") ?? []).length === 0, "football-domain 不得依赖任何 workspace crate");
for (const cycle of findCycles(graph)) report.violation(`检测到 Rust workspace 依赖环：${cycle.join(" -> ")}`);

const transitionalEdges = contract.transitional_edges ?? [];
for (const edge of transitionalEdges) {
  report.check(Boolean(contractCrates[edge.from]), `过渡依赖起点不存在：${edge.from}`);
  report.check(Boolean(contractCrates[edge.to]), `过渡依赖终点不存在：${edge.to}`);
  report.check((graph.get(edge.from) ?? []).includes(edge.to), `登记的过渡依赖当前不存在：${edge.from} -> ${edge.to}`);
  report.check(Boolean(edge.exit_task), `过渡依赖缺少 exit_task：${edge.from} -> ${edge.to}`);
}

report.finish(`${featureEntries.length} 个 Feature、${Object.keys(contractCrates).length} 个 Rust crate、${frontendFiles.length} 个前端源码文件`);
