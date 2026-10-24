import { existsSync, readFileSync, readdirSync } from "node:fs";
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

// R12-01 composition boundary BEGIN
{
  const tokens = source => (source.match(/\/\/[^\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[A-Za-z_$][A-Za-z_0-9$]*|[^\s]/g) ?? [])
    .filter(token => !token.startsWith("//") && !token.startsWith("/*"));
  const code = source => tokens(source).join("");
  const count = (source, value) => source.split(value).length - 1;
  check(code(bootstrapMain) === code('import { startApplication } from "./startApplication"; void startApplication();'), "R12-01 唯一入口只能导入并实际调用一次startApplication，不能空启动或夹带其他职责");
  const modules = [
    ["src/bootstrap/main.ts", bootstrapMain, ["./startApplication"], []],
    ["src/bootstrap/startApplication.ts", startApplication, ["./createApplication", "./applicationHandle"], ["errorMessage", "escapeHtml", "renderStartupFailure", "startApplication"]],
    ["src/bootstrap/createApplication.ts", createApplication, ["./applicationHandle", "./registerApplicationModules"], ["createApplication"]],
    ["src/bootstrap/registerApplicationModules.ts", registerModules, ["./applicationHandle"], ["registerApplicationModules"]],
  ];
  for (const [file, source, imports, functions] of modules) {
    const actualImports = [...source.matchAll(/\bfrom\s*["']([^"']+)["']/g), ...source.matchAll(/\bimport\s*["']([^"']+)["']/g)].map(match => match[1]);
    check(JSON.stringify(actualImports) === JSON.stringify(imports), `R12-01 启动职责不得直接依赖Feature/服务/数据或另一个状态owner：${file}`);
    const dynamicImports = [...source.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)].map(match => match[1]);
    check(JSON.stringify(dynamicImports) === JSON.stringify(file.endsWith("registerApplicationModules.ts") ? ["../main"] : []) && (source.match(/\bimport\s*\(/g) ?? []).length === dynamicImports.length, `R12-01 动态依赖只能由注册模块加载原真实应用：${file}`);
    check(JSON.stringify([...source.matchAll(/\bfunction\s+([A-Za-z_$][A-Za-z_0-9$]*)\s*\(/g)].map(match => match[1])) === JSON.stringify(functions), `R12-01 启动模块的职责函数集合漂移：${file}`);
    for (const forbidden of ["fetch(", "XMLHttpRequest", "localStorage", "sessionStorage", "addEventListener(", "setTimeout(", "setInterval(", "api."]) check(!code(source).includes(forbidden), `R12-01 启动组合根不得吸收业务I/O或监听器生命周期：${file}/${forbidden}`);
  }
  const creation = code(createApplication);
  const creationOrder = ['documentRoot.querySelector<HTMLDivElement>("#app")', 'if(!root)thrownewError("缺少 #app 根节点")', "newApplicationHandle(root)", "registerApplicationModules(handle)", "returnhandle;"];
  let previous = -1;
  for (const step of creationOrder) { const index = creation.indexOf(step, previous + 1); check(index > previous, `R12-01 必须依次解析真实根、校验、创建唯一handle、注册、返回：${step}`); previous = index; }
  check(count(creation, "newApplicationHandle(") === 1 && count(creation, "registerApplicationModules(") === 1 && count(creation, "returnhandle;") === 1, "R12-01 应用工厂不得创建或注册多份handle");
  const registration = code(registerModules);
  check(JSON.stringify([...registerModules.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)].map(match => match[1])) === JSON.stringify(["../main"]), "R12-01 必须唯一延迟载入原真实应用，不能导入mock/副本或多个运行时");
  check(registration.indexOf("exportfunctionregisterApplicationModules") < registration.indexOf("letruntime:"), "R12-01 runtime必须归本次handle注册闭包，不得变为全局状态");
  const registrationOrder = ['name:"browser-application"', 'awaitimport("../main")', "module.createBrowserApplicationModule(handle.root)", "awaitruntime.start()", "awaitruntime?.destroy()", "runtime=null", "handle.register(browserApplication)"];
  previous = -1;
  for (const step of registrationOrder) { const index = registration.indexOf(step, previous + 1); check(index > previous, `R12-01 真实模块start/destroy必须按原顺序接通：${step}`); previous = index; }
  check(count(registration, "handle.register(") === 1 && count(registration, "createBrowserApplicationModule(") === 1 && count(registration, "awaitruntime.start()") === 1, "R12-01 单个handle只能注册/创建/启动一次真实应用");
  const startup = code(startApplication);
  const startupOrder = ["handle=createApplication()", "awaithandle.start()", "returnhandle;", "catch(error:unknown)", 'renderStartupFailure(handle?.root??document.querySelector<HTMLElement>("#app"),error)', "returnnull;"];
  previous = -1;
  for (const step of startupOrder) { const index = startup.indexOf(step, previous + 1); check(index > previous, `R12-01 顶层启动必须等待真实start、成功返回原handle、失败投影后返回null：${step}`); previous = index; }
  check(startup.includes('escapeHtml(errorMessage(error))') && startup.includes('if(!root)return;') && startup.includes("error.errors[0]"), "R12-01 顶层错误须保留原聚合首错、HTML转义及无根节点处理");
  const moduleScripts = [...indexHtml.matchAll(/<script\b[^>]*\btype\s*=\s*["']module["'][^>]*>/g)].map(match => match[0]);
  check(moduleScripts.length === 1 && moduleScripts[0].includes('src="/src/bootstrap/main.ts"'), "R12-01 index.html只能载入一个真实组合根入口");
  const collect = directory => readdirSync(resolve(root, directory), { withFileTypes: true }).flatMap(entry => {
    const file = `${directory}/${entry.name}`;
    return entry.isDirectory() ? collect(file) : entry.isFile() && entry.name.endsWith(".ts") ? [file] : [];
  });
  const startupSites = collect("src").flatMap(file => [...tokens(read(file)).filter(token => !token.startsWith('"') && !token.startsWith("'")).join(" ").matchAll(/\bstartApplication \(/g)].map(() => file)).sort();
  check(JSON.stringify(startupSites) === JSON.stringify(["src/bootstrap/main.ts", "src/bootstrap/startApplication.ts"]), "R12-01 startApplication只允许原定义与唯一入口调用，不能旁路或重复自启动");
}
// R12-01 composition boundary END

if (failures.length > 0) {
  console.error("浏览器组合根验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("浏览器组合根验证通过：唯一入口、模块注册、生命周期 owner、失败边界和销毁链均已建立。");
