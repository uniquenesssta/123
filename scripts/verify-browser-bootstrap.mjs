import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const requiredFiles = [
  "src/bootstrap/main.ts",
  "src/bootstrap/startApplication.ts",
  "src/bootstrap/createApplication.ts",
  "src/bootstrap/registerApplicationModules.ts",
  "src/bootstrap/applicationHandle.ts",
];
for (const path of requiredFiles) check(existsSync(resolve(root, path)), `缺少浏览器组合根文件：${path}`);

const indexHtml = read("index.html");
const bootstrapMain = read("src/bootstrap/main.ts");
const startApplication = read("src/bootstrap/startApplication.ts");
const createApplication = read("src/bootstrap/createApplication.ts");
const registerModules = read("src/bootstrap/registerApplicationModules.ts");
const applicationHandle = read("src/bootstrap/applicationHandle.ts");
const browserApplication = read("src/main.ts");
const diagnostics = read("src/diagnostics/searchableSelectDiagnostics.ts");
const moduleContract = JSON.parse(read("architecture/module-boundaries.json"));
const stateContract = JSON.parse(read("architecture/state-ownership.json"));

check(indexHtml.includes('src="/src/bootstrap/main.ts"'), "index.html 未切换到浏览器组合根");
check(!indexHtml.includes('src="/src/main.ts"'), "index.html 仍使用旧浏览器入口");
check(bootstrapMain.includes("startApplication"), "bootstrap/main.ts 未调用 startApplication");
check(startApplication.includes("renderStartupFailure"), "顶层启动失败未集中处理");
check(createApplication.includes('querySelector<HTMLDivElement>("#app")'), "createApplication 未拥有根节点解析");
check(registerModules.includes('import("../main")'), "模块注册未通过唯一浏览器应用实现入口");
check(applicationHandle.includes("class ApplicationHandle"), "缺少 ApplicationHandle 生命周期所有者");
check(applicationHandle.includes("[...this.startedModules].reverse()"), "ApplicationHandle 未按逆序销毁模块");
check(applicationHandle.includes("AggregateError"), "ApplicationHandle 未保留启动/销毁复合错误");
check(browserApplication.includes("createBrowserApplicationModule"), "旧 main.ts 未转换为可注册浏览器应用模块");
check(!browserApplication.includes("void initializeApplication().catch"), "旧 main.ts 仍自行启动应用");
check((browserApplication.match(/browserLifecycleController\.signal/g) ?? []).length >= 10, "浏览器全局监听器未纳入生命周期信号");
check(browserApplication.includes("workspaceState.destroy()"), "浏览器销毁未关闭工作区状态生命周期");
check(diagnostics.includes("signal?: AbortSignal"), "搜索选择器诊断监听器不支持生命周期解绑");
check(moduleContract.frontend?.entry?.owner === "src/bootstrap/main.ts", "模块边界契约的前端 owner 未切换");
check(moduleContract.frontend?.entry?.status === "active-composition-owner", "模块边界契约的前端组合根未激活");
const lifecycle = stateContract.states.find((state) => state.id === "browser.lifecycle");
check(lifecycle?.owner === "src/bootstrap/applicationHandle.ts::ApplicationHandle", "browser.lifecycle 唯一 owner 未切换");
check(lifecycle?.transition === null, "browser.lifecycle 仍保留未完成 transition");

for (const path of requiredFiles) {
  const source = read(path);
  check(!/from ["']\.\.\/pages\//.test(source), `${path} 不得直接导入 Feature 页面`);
}

if (failures.length > 0) {
  console.error("浏览器组合根验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("浏览器组合根验证通过：唯一入口、模块注册、生命周期 owner、失败边界和销毁链均已建立。");
