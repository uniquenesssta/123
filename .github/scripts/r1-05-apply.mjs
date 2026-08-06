import { execFileSync } from "node:child_process";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";

const verifiedPayloadCommit = "cdfefb98c617f0362aa5deee3fcb5ff06faa8cab";
const wrapper = execFileSync(
  "git",
  ["show", `${verifiedPayloadCommit}:.github/scripts/r1-05-apply.mjs`],
  { encoding: "utf8" },
);
const match = wrapper.match(/Buffer\.from\("([A-Za-z0-9+/=]+)", "base64"\)/);
if (!match) {
  throw new Error("Unable to locate the verified R1-05 payload");
}

let source = gunzipSync(Buffer.from(match[1], "base64")).toString("utf8");

source = source.replaceAll(
  "model_registry/model_registry.rs",
  "model_registry/registry.rs",
);

function replaceOnce(search, replacement) {
  const index = source.indexOf(search);
  if (index < 0) {
    throw new Error(`R1-05 payload anchor missing: ${search.slice(0, 80)}`);
  }
  source = `${source.slice(0, index)}${replacement}${source.slice(index + search.length)}`;
}

replaceOnce(
  `mod model_registry;

pub use model_registry::ModelRegistry;`,
  `mod registry;

pub use registry::ModelRegistry;`,
);

replaceOnce(
  '  "database.rs",\n  "openai_research.rs",',
  '  "database.rs",\n  "fact_pipeline.rs",\n  "openai_research.rs",',
);
replaceOnce(
  'replaceAllRequired(`${applicationRoot}/database.rs`, "PostgresStore", "PersistenceStore");\n\nreplaceOnce(\n  `${applicationRoot}/openai_research.rs`,',
  `replaceAllRequired(\`${'${applicationRoot}'}/database.rs\`, "PostgresStore", "PersistenceStore");

replaceOnce(
  \`${'${applicationRoot}'}/fact_pipeline.rs\`,
  "use super::{ApplicationError, ApplicationResult, ApplicationService};",
  "use super::{ApplicationError, ApplicationResult, ApplicationService};\\nuse crate::PersistenceStore;",
);
replaceAllRequired(
  \`${'${applicationRoot}'}/fact_pipeline.rs\`,
  "football_persistence_postgres::PostgresStore",
  "PersistenceStore",
);

replaceOnce(
  \`${'${applicationRoot}'}/openai_research.rs\`,`,
);
const escapedTick = "\\`";
replaceOnce(
  `- ${escapedTick}crates/application/src/database.rs${escapedTick}\n- ${escapedTick}crates/application/src/openai_research.rs${escapedTick}`,
  `- ${escapedTick}crates/application/src/database.rs${escapedTick}\n- ${escapedTick}crates/application/src/fact_pipeline.rs${escapedTick}\n- ${escapedTick}crates/application/src/openai_research.rs${escapedTick}`,
);

const temporaryScript = join(tmpdir(), `r1-05-patched-${process.pid}.mjs`);
try {
  writeFileSync(temporaryScript, source);
  await import(pathToFileURL(temporaryScript).href);
} finally {
  rmSync(temporaryScript, { force: true });
}

const compositionVerifierPath = "scripts/verify-application-composition.mjs";
let compositionVerifier = readFileSync(compositionVerifierPath, "utf8");
if (!compositionVerifier.includes("model_registry/model_registry.rs")) {
  throw new Error("application composition verifier model registry path anchor missing");
}
compositionVerifier = compositionVerifier
  .replaceAll("model_registry/model_registry.rs", "model_registry/registry.rs")
  .replaceAll('["mod.rs", "model_registry.rs"]', '["mod.rs", "registry.rs"]');
writeFileSync(compositionVerifierPath, compositionVerifier);

const verifierPath = "scripts/architecture/verifyProtectedImports.mjs";
let verifier = readFileSync(verifierPath, "utf8").replaceAll("\r\n", "\n");

function replaceVerifierOnce(search, replacement) {
  const index = verifier.indexOf(search);
  if (index < 0) {
    throw new Error(`protected import verifier anchor missing: ${search.slice(0, 80)}`);
  }
  verifier = `${verifier.slice(0, index)}${replacement}${verifier.slice(index + search.length)}`;
}

replaceVerifierOnce(
  'const tauriRoot = normalizePath(contract.rust?.rules?.tauri_owner ?? "src-tauri");\n',
  'const tauriRoot = normalizePath(contract.rust?.rules?.tauri_owner ?? "src-tauri");\nconst applicationPersistenceOwner = normalizePath(\n  contract.rust?.rules?.application_persistence_import_owner ?? "",\n);\n',
);
replaceVerifierOnce(
  'const rustFiles = listFiles(["crates", "src-tauri"], { extensions: [".rs"] });\nfor (const file of rustFiles) {\n  const source = readFileSync(join(repositoryRoot, file), "utf8");\n',
  'const rustFiles = listFiles(["crates", "src-tauri"], { extensions: [".rs"] });\nconst applicationPersistenceImporters = [];\nfor (const file of rustFiles) {\n  const source = readFileSync(join(repositoryRoot, file), "utf8");\n  if (file.startsWith("crates/application/") && source.includes("football_persistence_postgres")) {\n    applicationPersistenceImporters.push(file);\n  }\n',
);
replaceVerifierOnce(
  `const applicationManifest = parseCargoManifest("crates/application/Cargo.toml");
const applicationUsesPersistence = applicationManifest.dependencies.some((dependency) => dependency.packageName === "football-persistence-postgres");
if (applicationUsesPersistence) {
  const transition = (contract.transitional_edges ?? []).find((edge) => edge.from === "football-application" && edge.to === "football-persistence-postgres");
  report.check(Boolean(transition), "football-application 仍依赖 persistence-postgres，但契约未登记受控过渡边");
  if (transition) report.note(\`保留已登记过渡边 football-application -> football-persistence-postgres，退出任务 \${transition.exit_task}\`);
}
`,
  `const applicationManifest = parseCargoManifest("crates/application/Cargo.toml");
const applicationUsesPersistence = applicationManifest.dependencies.some(
  (dependency) => dependency.packageName === "football-persistence-postgres",
);
const staleApplicationPersistenceTransition = (contract.transitional_edges ?? []).find(
  (edge) =>
    edge.from === "football-application" &&
    edge.to === "football-persistence-postgres",
);
report.check(
  !staleApplicationPersistenceTransition,
  "football-application -> persistence-postgres 的 R1-05 过渡边仍未退出",
);
if (applicationUsesPersistence) {
  report.check(
    Boolean(applicationPersistenceOwner),
    "football-application 仍声明 persistence-postgres，但未登记组合根具体适配器导入所有者",
  );
  report.check(
    applicationPersistenceImporters.length === 1 &&
      applicationPersistenceImporters[0] === applicationPersistenceOwner,
    \`football-application 的 PostgreSQL 具体导入必须仅位于 \${applicationPersistenceOwner || "已登记组合根文件"}；当前：\${applicationPersistenceImporters.join(", ") || "无"}\`,
  );
  if (
    applicationPersistenceOwner &&
    applicationPersistenceImporters.length === 1 &&
    applicationPersistenceImporters[0] === applicationPersistenceOwner
  ) {
    report.note(
      \`Application 持久化具体适配器导入已收敛到组合根：\${applicationPersistenceOwner}\`,
    );
  }
} else {
  report.check(
    applicationPersistenceImporters.length === 0,
    \`football-application 未声明 persistence-postgres，但仍存在具体导入：\${applicationPersistenceImporters.join(", ")}\`,
  );
}
`,
);
writeFileSync(verifierPath, verifier);
