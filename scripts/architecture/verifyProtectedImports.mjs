import { readFileSync } from "node:fs";
import { join } from "node:path";
import { parseCargoManifest } from "./lib/cargo.mjs";
import { isPackageImport, parseJavaScriptImports, resolveRelativeImport } from "./lib/imports.mjs";
import {
  listFiles,
  matchesPathPattern,
  normalizePath,
  readJson,
  repositoryRoot,
} from "./lib/repository.mjs";
import { VerificationReport } from "./lib/report.mjs";

const report = new VerificationReport("受保护导入验证");
const contract = readJson("architecture/module-boundaries.json");
const transportOwner = normalizePath(contract.frontend?.transport?.owner ?? "");
const persistenceRoot = normalizePath(contract.rust?.rules?.sqlx_owner ?? "crates/persistence-postgres").split("/src/")[0];
const tauriRoot = normalizePath(contract.rust?.rules?.tauri_owner ?? "src-tauri");
const protectedModelTokens = [
  "football-model-p4",
  "football-model-p7",
  "football_model_p4",
  "football_model_p7",
  "crates/model-p4",
  "crates/model-p7",
  "model-p4",
  "model-p7",
];

function containsProtectedModel(value) {
  const normalized = normalizePath(value).toLowerCase();
  return protectedModelTokens.some((token) => normalized.includes(token));
}

function containsProtectedRustReference(source) {
  const importLines = source
    .split("\n")
    .filter((line) => /\b(?:use|extern\s+crate|mod)\b/.test(line) || /football_model_p[47]::/.test(line));
  return importLines.some((line) => containsProtectedModel(line));
}

const frontendFiles = listFiles(["src"], { extensions: [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"] });
for (const importer of frontendFiles) {
  const source = readFileSync(join(repositoryRoot, importer), "utf8").replaceAll("\r\n", "\n");
  for (const specifier of parseJavaScriptImports(source)) {
    if (isPackageImport(specifier, "@tauri-apps/api/core")) {
      const allowed = importer === transportOwner || importer.startsWith("src/platform/tauri/");
      report.check(allowed, `${importer} 不得直接导入 @tauri-apps/api/core`);
    }

    const resolved = resolveRelativeImport(importer, specifier);
    if (resolved) {
      report.check(!matchesPathPattern(resolved, "src-tauri/**"), `${importer} 不得导入 src-tauri：${specifier}`);
      report.check(!matchesPathPattern(resolved, "crates/**"), `${importer} 不得导入 crates：${specifier}`);
      report.check(!matchesPathPattern(resolved, "migrations/**"), `${importer} 不得导入 migrations：${specifier}`);
    }

    if (containsProtectedModel(specifier)) report.violation(`${importer} 直接导入受保护模型：${specifier}`);
  }
}

const rustFiles = listFiles(["crates", "src-tauri"], { extensions: [".rs"] });
for (const file of rustFiles) {
  const source = readFileSync(join(repositoryRoot, file), "utf8");
  const usesSqlx = /(?:\buse\s+sqlx\b|\bsqlx(?:::|!))/.test(source);
  const usesTauri = /(?:\buse\s+tauri\b|\btauri(?:::|!)|#\[tauri::)/.test(source);

  if (usesSqlx) report.check(matchesPathPattern(file, `${persistenceRoot}/**`), `${file} 使用 SQLx，但 SQLx 只能位于 ${persistenceRoot}`);
  if (usesTauri) report.check(matchesPathPattern(file, `${tauriRoot}/**`), `${file} 使用 Tauri，但 Tauri 只能位于 ${tauriRoot}`);

  const restrictedModelConsumer = matchesPathPattern(file, "src-tauri/**") || matchesPathPattern(file, `${persistenceRoot}/**`);
  if (restrictedModelConsumer && containsProtectedRustReference(source)) report.violation(`${file} 直接引用受保护 P4/P7 模型`);
}

const cargoFiles = ["Cargo.toml", ...listFiles(["crates", "src-tauri"], { extensions: [".toml"] }).filter((file) => file.endsWith("Cargo.toml"))];
for (const manifestPath of [...new Set(cargoFiles)]) {
  const manifest = parseCargoManifest(manifestPath);
  for (const dependency of manifest.dependencies) {
    if (dependency.packageName === "sqlx") {
      const allowed = manifestPath === "Cargo.toml" || matchesPathPattern(manifestPath, `${persistenceRoot}/**`);
      report.check(allowed, `${manifestPath} 声明 SQLx；仅 workspace 版本声明和 ${persistenceRoot} 可使用`);
    }
    if (dependency.packageName === "tauri") {
      report.check(matchesPathPattern(manifestPath, `${tauriRoot}/**`), `${manifestPath} 声明 Tauri；仅 ${tauriRoot} 可使用`);
    }
    if (containsProtectedModel(dependency.packageName) || containsProtectedModel(dependency.path ?? "")) {
      const restricted = matchesPathPattern(manifestPath, "src-tauri/**") || matchesPathPattern(manifestPath, `${persistenceRoot}/**`);
      report.check(!restricted, `${manifestPath} 直接依赖受保护 P4/P7 模型：${dependency.packageName}`);
    }
  }
}

const domainManifest = parseCargoManifest("crates/domain/Cargo.toml");
const infrastructureDependencies = new Set([
  "football-application",
  "football-persistence-postgres",
  "football-research-gateway",
  "football-spreadsheet-io",
  "reqwest",
  "sqlx",
  "tauri",
  "tokio",
]);
for (const dependency of domainManifest.dependencies) {
  report.check(!infrastructureDependencies.has(dependency.packageName), `football-domain 不得依赖基础设施：${dependency.packageName}`);
}

const applicationManifest = parseCargoManifest("crates/application/Cargo.toml");
const applicationUsesPersistence = applicationManifest.dependencies.some((dependency) => dependency.packageName === "football-persistence-postgres");
if (applicationUsesPersistence) {
  const transition = (contract.transitional_edges ?? []).find((edge) => edge.from === "football-application" && edge.to === "football-persistence-postgres");
  report.check(Boolean(transition), "football-application 仍依赖 persistence-postgres，但契约未登记受控过渡边");
  if (transition) report.note(`保留已登记过渡边 football-application -> football-persistence-postgres，退出任务 ${transition.exit_task}`);
}

report.finish(`${frontendFiles.length} 个前端文件、${rustFiles.length} 个 Rust 文件、${cargoFiles.length} 个 Cargo 清单`);
