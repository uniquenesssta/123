import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) =>
  readFileSync(join(root, path), "utf8").replace(/^\uFEFF/, "").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

function filesIn(path) {
  return readdirSync(join(root, path), { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
}

function listRustFiles(path) {
  const absolute = join(root, path);
  const output = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) {
      output.push(...listRustFiles(relative(root, child)));
    } else if (entry.name.endsWith(".rs")) {
      output.push(relative(root, child).replaceAll("\\", "/"));
    }
  }
  return output;
}

check(
  JSON.stringify(filesIn("crates/application/src/composition")) ===
    JSON.stringify(["application_composition.rs", "mod.rs", "port_registry.rs"]),
  "Application composition 文件集合不正确",
);
check(
  JSON.stringify(filesIn("crates/application/src/service")) ===
    JSON.stringify(["application_service.rs", "mod.rs"]),
  "Application service 文件集合不正确",
);
check(
  JSON.stringify(filesIn("crates/application/src/model_registry")) ===
    JSON.stringify(["mod.rs", "registry.rs"]),
  "Model registry 文件集合不正确",
);

const library = read("crates/application/src/lib.rs");
const composition = read(
  "crates/application/src/composition/application_composition.rs",
);
const portRegistry = read("crates/application/src/composition/port_registry.rs");
const service = read("crates/application/src/service/application_service.rs");
const modelRegistry = read(
  "crates/application/src/model_registry/registry.rs",
);
const moduleContract = JSON.parse(read("architecture/module-boundaries.json"));
const stateContract = JSON.parse(read("architecture/state-ownership.json"));
const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");

for (const declaration of ["mod composition;", "mod model_registry;", "mod service;"]) {
  check(library.includes(declaration), `lib.rs 缺少模块声明：${declaration}`);
}
check(
  library.includes("pub use model_registry::ModelRegistry;"),
  "lib.rs 未保留 ModelRegistry 公共出口",
);
check(
  library.includes("pub use service::ApplicationService;"),
  "lib.rs 未保留 ApplicationService 公共出口",
);
for (const forbidden of [
  "pub struct ModelRegistry",
  "pub struct ApplicationService",
  "struct ActiveDatabase",
  "PublicModelStub::built_in_models()",
  "HashMap<String, Arc<dyn PredictionModel>>",
  "RwLock<Option<ActiveDatabase>>",
]) {
  check(!library.includes(forbidden), `lib.rs 仍承担独立职责：${forbidden}`);
}

check(
  composition.includes("pub(crate) struct ApplicationComposition"),
  "缺少 ApplicationComposition",
);
check(
  composition.includes("PublicModelStub::built_in_models()"),
  "默认模型注册未归属组合根",
);
check(composition.includes("PortRegistry::new()"), "组合根未构造端口注册表");
check(
  composition.includes("AtomicBool::new(false)"),
  "P4 worker 初始状态发生变化",
);
check(
  portRegistry.includes("PostgresStore as PersistenceStore"),
  "持久化适配器未通过端口注册表导入",
);
check(
  portRegistry.includes("pub(crate) struct ActiveDatabase"),
  "活动数据库状态未归属端口模块",
);
check(
  portRegistry.includes("RwLock<Option<ActiveDatabase>>"),
  "端口注册表未持有活动数据库槽位",
);
check(service.includes("pub struct ApplicationService"), "缺少兼容 ApplicationService 门面");
check(
  service.includes("ApplicationComposition::new().into_parts()"),
  "ApplicationService 未委托组合根构造",
);
check(modelRegistry.includes("pub struct ModelRegistry"), "缺少 ModelRegistry");
for (const method of ["pub fn register", "pub fn get", "pub fn descriptors"]) {
  check(modelRegistry.includes(method), `ModelRegistry 缺少兼容方法：${method}`);
}
check(modelRegistry.includes("values.sort_by"), "模型描述符排序语义发生变化");

const rustFiles = listRustFiles("crates/application/src");
const concreteImportOwners = rustFiles.filter((path) =>
  read(path).includes("football_persistence_postgres"),
);
check(
  JSON.stringify(concreteImportOwners) ===
    JSON.stringify(["crates/application/src/composition/port_registry.rs"]),
  `PostgreSQL 具体依赖所有者不唯一：${concreteImportOwners.join(", ")}`,
);
const modelRegistrationOwners = rustFiles.filter((path) =>
  read(path).includes("PublicModelStub::built_in_models()"),
);
check(
  JSON.stringify(modelRegistrationOwners) ===
    JSON.stringify([
      "crates/application/src/composition/application_composition.rs",
    ]),
  `默认模型注册所有者不唯一：${modelRegistrationOwners.join(", ")}`,
);

check(
  moduleContract.rust?.application_composition?.composition_root ===
    "crates/application/src/composition/application_composition.rs::ApplicationComposition",
  "模块边界契约未登记 Application 组合根",
);
check(
  moduleContract.rust?.application_composition?.persistence_adapter_import_owner ===
    "crates/application/src/composition/port_registry.rs",
  "模块边界契约未登记持久化具体依赖所有者",
);
check(
  !(moduleContract.transitional_edges ?? []).some(
    (edge) => edge.exit_task === "R1-05",
  ),
  "R1-05 过渡依赖仍未退出",
);
const modelRegistryState = stateContract.states?.find(
  (entry) => entry.id === "application.model-registry",
);
check(
  modelRegistryState?.owner ===
    "crates/application/src/model_registry/registry.rs::ModelRegistry",
  "模型注册表状态所有者未切换",
);
check(modelRegistryState?.transition === null, "模型注册表仍处于过渡状态");
const activeDatabaseState = stateContract.states?.find(
  (entry) => entry.id === "application.active-database",
);
check(
  activeDatabaseState?.owner ===
    "crates/application/src/service/application_service.rs::ApplicationService.database",
  "活动数据库状态所有者未切换",
);
const p4WorkerState = stateContract.states?.find(
  (entry) => entry.id === "application.p4-worker",
);
check(
  p4WorkerState?.owner ===
    "crates/application/src/service/application_service.rs::ApplicationService.p4_worker_running",
  "P4 worker 状态所有者未切换",
);
check(
  packageDefinition.scripts?.["verify:application-composition"] ===
    "node scripts/verify-application-composition.mjs",
  "package.json 未登记 Application 组合根验证入口",
);
check(
  frontendVerifier.includes('"verify-application-composition.mjs"'),
  "前端聚合验证未接入 Application 组合根门禁",
);

if (failures.length) {
  throw new Error(
    `Application 组合根验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`,
  );
}

console.log(
  `Application 组合根验证通过：3 个职责目录、${rustFiles.length} 个 Rust 源文件、唯一模型注册所有者和唯一 PostgreSQL 具体导入所有者。`,
);
