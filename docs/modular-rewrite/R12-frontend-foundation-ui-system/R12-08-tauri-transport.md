# R12-08 Tauri Transport

状态：`VERIFYING`。2026-10-25用户授权“07通过，开始08”。07精确b0b26d6/run40134011943及全部步骤、完整日志、产物摘要与startup报告已独立核验，详[07收尾](R12-07-workspace.md)。本节点基线`b0b26d6bbe8c99041eb284bc1811fd48ed7fbcfe`，沿唯一`rewrite/r12-frontend-foundation-ui-system`。09～19 BLOCKED，R12整体IN_PROGRESS；08自身Windows待提交后独立验收，不轮询。

## 范围与依据

依据R12-08任务书、总纲执行订正、R00冻结的171命令/DTO/调用方清单及当前源码。沿TypeScript/DOM/CSS与已安装Tauri API2.11.1，直接核对该版本`core.d.ts`和实际invoke实现：支持命令、参数和InvokeOptions，未提供AbortSignal撤销接口。

08为内部传输改造，没有UI视觉、Figma资源、图标、样式或业务页面变化；继续沿05～07已接入的设计与交互。没有新增Figma稿件、框架、生产依赖、持续runner、workflow或数据库验证入口。`plugin-dialog`文件选择仍由既有API门面承担，按09范围后续处理。

## 唯一职责与入口

原`src/api/client.ts`混合类型化命令门面、SDK调用、错误、日志摘要和追踪；08保留全部原API方法体，将传输能力迁入唯一`src/platform/tauri/`：

| 文件 | 责任 |
|---|---|
| tauriAdapter.ts | 唯一`@tauri-apps/api/core`导入与默认运行实例；提供原invoke/诊断调用出口 |
| invokeRequest.ts | 编排请求ID、当前租约、原生命令、成功/失败与销毁；业务命令并发策略保持 |
| invokeResponse.ts | 原值/原对象交付检查，响应诊断摘要；不增加返回信封或改写DTO |
| invokeError.ts | 原LoggedInvokeError/issueLogged语义和错误文案；具名本地取消错误 |
| invokeTrace.ts | 原日志phase/字段、每请求开始与单一终结顺序、UI/导航/诊断与问题日志例外 |
| traceSummary.ts | 有界、非抛出的日志投影与脱敏，不变更实际业务参数/结果 |
| cancellationRegistry.ts | 唯一活动请求租约、AbortSignal监听器及停止边界，所有出口清理登记 |
| commandName.ts | 由R00命令合同生成的171项只读类型/运行时集合，按原注册顺序逐项验证 |
| index.ts | 显式导出公开接口与类型，不承载业务实现 |

API门面改走公开出口，原四个日志/错误识别函数以原名称再导出。主应用destroy先完成Workspace最终保存与Session销毁，在finally停止Transport；全局命令调用入口没有复制或保留旧包装器。原文件继续承担类型化业务门面，未删除或另造替代文件。

架构登记SDK owner、公开出口和实际源码列表；直接SDK导入从“整个platform/tauri目录可用”收紧为唯一adapter，Feature禁止绕过数据边界直接导入transport。新增`browser.invoke-requests`具名owner，共29个；历史`last_updated_task=R3-02`兼容字段保持，未夹带其余契约格式整理。

## 兼容与异步边界

- 原API方法签名、命令字符串、参数对象及返回声明相对基线逐字比对一致；171公共Tauri命令、注册顺序、Rust DTO、数据库/迁移、配置、IssueLog字段及运行JSONL格式不变。源码位置更新仅由现有验证器追随登记owner，冻结command-contract未改。
- 每请求独立`ipc-`ID与租约，允许原有并发；成功交付保留null/undefined/数组/原始值与对象身份。错误保留原字符串归一化、LoggedInvokeError名字和issueLogged识别，新增cause仅供内部追溯。bootstrap及record_issue继续不二次记录Issue；日志写失败不覆盖业务成功或原错误，issueLogged沿用原“已处理记录尝试”标记，不冒充磁盘必然成功。
- 可选AbortSignal用于本地结果取消：已取消请求不派发，取消等待及时拒绝，迟到成功/失败被消费且不再交付；所有退出路径移除监听和登记，stop幂等并拒绝新请求。重复ID不替换活动租约，不按当前页面猜测请求归属。
- 本地取消不能撤销已经发送的原生命令，不保证事务回滚，不自动重试；既有AI业务`cancel_api_workspace_request(requestId)`仍按原业务ID调用后端。Transport没有把自己的trace ID冒充AI请求ID，也没有新增后端取消命令。
- started先于单一completed/failed。取消使用既有failed阶段和`context.cancelled=true`，没有新增后端日志phase；已完成原生命令进入completed日志后发生本地取消，只丢弃交付，保留该唯一终结。开始日志尚未完成时取消，不再派发业务命令；日志不记录递归日志命令链。
- 继续40键/8数组项/4深度/512字符的摘要边界，正文以长度/省略标记记录。敏感键、Bearer、URL凭据、API token文本在前端日志投影和Issue副本脱敏，公开拒绝消息及真正的业务参数保持。循环值、抛错getter等诊断投影失败不把成功响应变成业务错误，删除结果数量/ID摘要保持可诊断。

## 实际验证

本机Windows，Node24.16.0/npm11.13.0、TypeScript5.8.3/Vite7.3.6、Rust1.88.0。Transport测试执行实际生产模块及原API门面，原生边界注入可控返回/错误/挂起以复现竞态；没有把该adapter测试当成真实Tauri、网络API或PostgreSQL端到端。

| 入口 | 结果 |
|---|---|
| 原verify-api-runtime-diagnostics.mjs | PASS：24项生产请求/响应/错误/追踪/取消测试；原Rust诊断合同断言保留 |
| 两套原命令检查器 | PASS：171命令、原顺序、调用/定义/注册集合与冻结哈希一致；源码集合跟随实际owner，动态SDK参数规则保持 |
| TypeScript | PASS，原全部业务API调用类型通过 |
| npm run verify:architecture | PASS：18 Feature、11 Rust crate、29唯一owner；新增Feature传输禁止边界另专项通过 |
| cargo fmt --all -- --check | PASS，Rust源码未修改 |
| npm run verify:frontend | PASS：89个原验证脚本＋TypeScript＋Vite，共91步骤；原1,104 Shell/工作区、136告警与17视口、24生命周期/18会话/21路由继续通过，日志仓库上一级r12-08-frontend-final.log |

24项覆盖命令全集/非法名、参数/返回身份、null等原值、原错误/marker/cause、bootstrap/record_issue例外、日志/Issue失败、并发独立性、预取消/本地取消/迟到响应、慢started/terminal日志、stop、完成后取消、敏感摘要/循环/getter、原门面与UI诊断桥、AI业务取消分离、监听器四出口恰好释放及重复ID。

首轮完整回归在旧可搜索选择器诊断检查失败：仍只读取client中的phase声明；检查现已追随新Trace owner，原diagnostic断言/真实浏览器检查保持，最终完整91步骤复验通过。早期独立检查漏用保留产物预载，触发了原脚本对自身临时测试目录的清理；已明确告知用户，后续完整验证启用保留保护，未改清理脚本或弱化门禁。失败日志`../r12-08-frontend-1.log`保留。

最终21条非Markdown差异的规范化SHA-256为`ceab9ed5e15af3201c5b80573588cc628ba106f60ddce935b90c846e07cba330`，验证后再比对一致。Shell/工作区截图与结构化产物沿原入口保留在`C:\Users\uniqu\Documents\ChatGPT\项目代码模块性优化 足球\logs\r12-05-shell-1792879727809`，原17截图报告及告警证据由最终日志定位；没有更新PNG基线或阈值。保护资产18项聚合SHA-256仍`d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`。Rust源码、171命令合同、src/types.ts、依赖/锁文件、UI样式和图标均无差异。构建主CSS仍为`main-CMIfZmKw.css`，606.76kB主JS的既有超500kB提示保持；相对07增加的传输模块已如实进入包，没有放宽门槛。

## 待验与回退

08自身Rust Clippy/workspace tests、release/MSI/NSIS、实际客户端启动与运行日志配对交由原Windows CI独立验证；未用07结果替代08通过，当前保持VERIFYING。真实API/凭据/完整会话取消UI端到端及真实PG/XLSX/历史保护四项/不可变账本并发回滚/Windows Full/私有P4/P7固定回归等按已有最终封包新库安排保留待验；ignored、公开stub、注入adapter及startup不代表这些PASS。

回退点b0b26d6，使用受控Git回退本节点，无数据库/持久格式迁移。未开始09；Vite既有主chunk超过500kB警告保持，没有放宽阈值。

## 实际文件清单

相对b0b26d6共27路径：12新增、15修改、0删除；无移动/重命名。旧传输实现从原API文件内移除，类型化门面保留。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-08-tauri-transport.md`
- `scripts/tauri/transportSources.mjs`
- `scripts/tauri/verifyTransportBehavior.mjs`
- `src/platform/tauri/cancellationRegistry.ts`
- `src/platform/tauri/commandName.ts`
- `src/platform/tauri/index.ts`
- `src/platform/tauri/invokeError.ts`
- `src/platform/tauri/invokeRequest.ts`
- `src/platform/tauri/invokeResponse.ts`
- `src/platform/tauri/invokeTrace.ts`
- `src/platform/tauri/tauriAdapter.ts`
- `src/platform/tauri/traceSummary.ts`

### 修改

- `README.md`
- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-07-workspace.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/architecture/verifyProtectedImports.mjs`
- `scripts/verify-api-runtime-diagnostics.mjs`
- `scripts/verify-command-contract.mjs`
- `scripts/verify-entity-deletion.mjs`
- `scripts/verify-searchable-select-diagnostics.mjs`
- `scripts/verify_command_contract.mjs`
- `src/api/client.ts`
- `src/main.ts`

### 删除

无。
