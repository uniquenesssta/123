# R13-03 赛事层级、规则包与路由

状态：**VERIFYING（CI失败已定位修复，待修复提交自身CI/产物验收）**。原代码 `12ba495c583ea25fd32b508aa68cceea9407f9d0` 的 [run41110961979](https://github.com/uniquenesssta/123/actions/runs/41110961979) / job124252955643 已确认为 **completed / failure**，head/分支均匹配。失败在前端依赖安装前的架构门禁：新增运行期行为测试需要 TypeScript；Windows 自动验收被跳过，0 个产物，不能收尾。用户本轮“收尾03开始04”授权继续修复与核验；原04已按任务书并入03，本轮逐项核对，05未启动。

依据：[阶段任务书](../../football-model-platform-modular-rewrite-19-docs/13-R13-core-ui-features.md)、[执行约定](../remaining-execution-contract.md)、[前期遗留账本](../prior-stage-baseline-20261030.md) BA-01/BA-14。保持用户批准的 R12 视觉与页面入口，不修改 R12 任务书、索引或完成记录。

## 实际职责与原功能去向

| 原职责/调用方 | 当前唯一 owner 与保留的数据流 |
| --- | --- |
| `src/pages/rules.ts` 的目录分类、计数、赛事卡片 | `src/features/competitions/catalogueView.ts`；仍读取 Session 摘要，不复制 Bootstrap |
| main 的目录筛选、分类切换及旧 `catalogueFilters.ts` | `features/competitions/catalogueFilters.ts`；沿用搜索、赛制、体系/地区口径及原清除动作 |
| 赛事/赛季/阶段/轮次创建表单 | `competitionView.ts`、`hierarchyOptions.ts`；原四种 DTO、选项身份和公共控件保持 |
| main 的四种 create 与删除确认 | `competitionDrafts.ts`、`CompetitionFeature`；调用原 `api.createCompetition/createSeason/createStage/createRound/deleteCompetition`，失败不刷新或报成功，取消不写入 |
| 目录规则路径说明 | `competitionPathView.ts`；展示已登记范围与候选，不在前端代替后端选出正式路由 |
| 原 rules 页面外壳和四个页签 | `features/rules/rulesView.ts`；从两个 Feature 公共出口组合，仍只有原 `rules` 导航入口 |
| 绑定表单及分组目录 | `bindingView.ts`、`RulesFeature.createBinding`、`RulesWorkflow.saveBinding`；提交原 CompetitionBindingDraft 到 `create_competition_binding`，不新增或伪装已有绑定的更新接口 |
| main 的 `pendingRulePackage` 与异步文件读事件 | `RulePackageImport`；候选/文件读身份唯一 owner；原文件事件只分发到 `RulesFeature.readPackage` |
| 规则包注册 | `RulesWorkflow.register`；提交候选的独立快照到原 `register_rule_package`；完整业务校验、参数/版本与引用语义仍归后端 |
| 路由预览与候选失效 | `RulesWorkflow.resolve/invalidatePreview`；复制提交命令，绑定 Session/请求代次/当前表单身份；结果只来自原 `preview_route` |
| 规则详情、路由卡片 | `features/rules/routeView.ts`；统一展示原 RouteDecision，并转义后端名称；通过公共出口供原预测页面调用 |
| 当前预测表单取值/正式执行 | main 仍保留预测域的 `syncSimpleMatchInput/routeSelection/executePrediction` 及薄预览适配，后续预测批次接收；正式执行继续原 API，未使用预览绕过执行校验 |

实际变更为 **34 个路径：18 新增、14 修改、2 个已替代旧路径退出**；退出路径均有实际实现承接，无额外删除。

两个 Feature 均通过 `index.ts` 暴露具名出口。Workspace 继续唯一持有普通表单/页签恢复，Session 继续唯一持有全局摘要，Modal 继续唯一持有确认 UI。新增状态登记仅 `browser.rules-workflow` 与 `browser.rule-package-import`；总计 41 个状态 owner、19 个 Feature、291 个前端源码文件。未引入框架、生产依赖、持续 runner/workflow、独立数据库或新 DTO。

原 `src/pages/rules.ts`、`src/pages/rules/catalogueFilters.ts` 两个旧路径已分别逐项移动并拆分替代。原 `src/pages/rules/rulesCatalogue.css` 保持原样式图归属；未递归删除目录或批量删除文件。

## 本批修复及恢复语义

- 绑定及临时演练的赛季/阶段同时服从所有上级身份；切换赛事清空赛季和阶段，切换赛季清空阶段。无上级时下级禁选；Workspace 恢复后也先校正过期组合。移除阶段后赛制恢复为当前赛事赛制。提交前再次检查身份，后端仍负责完整有效性。
- 文件读取实行最后一次选择有效；取消、清空或销毁后旧结果不发布，旧读取错误不污染新文件。读取中禁止注册；格式失败保留此前有效草稿，后端注册失败也保留候选。注册 A 成功不会清空期间新选择的 B。
- 形状预检只保护摘要渲染，不复制后端 schema/模型/参数校验。Rust `serde(default)` 允许的省略字段原样透传，不在前端补造默认值，也不因省略 metadata/source_document 等字段拒绝后端支持的文档。
- 创建/绑定写入成功后刷新当前 Session，保留当前页签和当前表单上下文；刷新失败明确提示“已保存、刷新未完成、勿重复提交”，不自动重放写入。后端拒绝和确认取消均不产生虚假成功。删除确认绑定当前 Session 身份，切换摘要后须重新核对。
- 预览绑定当前 Session、请求代次和实际 DOM 表单身份。切换上级、开球时间/模型、页面重建、绑定/规则写入或销毁即失效；迟到成功/错误均不覆盖当前上下文。下一次预览重新调用后端，读取最新绑定。
- 实际浏览器发现原临时演练预览会写入隐藏的另一页签。现将“检查模型”的当前结果直接展示在既有详情面板，确保反馈可见。目录说明、分组候选和预览均不再宣称“可直接开始推演”；正式许可仍由正式执行链重新判定。
- 目录概览只显示赛事级/类型级候选，不把阶段或赛季绑定投影成整项赛事的默认规则；绑定目录按真实作用范围与匹配顺序显示，不把不同范围简单排序成全局“优先采用”。

## 验证及证据

| 验证类别 | 本轮结果/范围 |
| --- | --- |
| 静态与边界 | 完整 `npm run verify:architecture` 通过；最终 routeView 移交后再次通过模块边界/状态 owner。原命令、受控资产等完整前端门禁均保留，未关闭或降低任何检查 |
| 自动生产行为 | 现由安装依赖后的原 `verify-browser-bootstrap.mjs` 入口执行 `scripts/features/verifyCompetitionRulesWorkflow.mjs` 的全部三组；hierarchy/bindings/route-resolution 原静态断言保持；最终 **10 + 13 + 8 = 31** 项通过，实际加载生产类，只有 API/Session/DOM 边界使用夹具 |
| 类型/生产构建 | 完整 **96 步**前端验收通过。最后规则包默认字段兼容与路由展示移交后，再跑 31 项行为、受影响边界、TypeScript 和 Vite build，均通过；其他未变检查沿用本轮完整结果。保留既有 JS chunk 大小提示 |
| Windows 实际浏览器 | 最终生产入口 bootstrap/main/Router/Session/Feature 的 **28 项**检查通过；只替换 Tauri 外部返回，不替换页面或工作流。涵盖四种创建、层级切换、绑定拒绝草稿、删除取消/拒绝、文件乱序/格式/注册失败恢复、预览过期与保存新绑定后重新预览；0 未捕获错误 |
| Windows 原生构建/运行 | `cargo build --locked -p football-match-model-desktop` 通过；最终当前 Vite 生产入口的桌面开发版实际启动，startup PASS，**5 条记录 / bootstrap、read_workspace_state / 0 错误** |
| Rust 完整门禁复用 | 本批 crates、src-tauri、Cargo、Schema/迁移、package/lock 与 workflow 均未改；fmt/clippy/完整 Rust 测试沿用 02 精确代码 `d167e948` 的成功 CI run41098940648。此复用不等于本批又执行了完整 Rust 测试，也不代替本批推送后的 CI/安装包 |
| 真实 PG / Full / 私有模型 | **未验，未记 PASS**。本地浏览器使用显式边界夹具，不作为真实数据库读写、引用保护/事务或 ModelProvider 验收 |

本轮证据位于项目临时目录（本机产物，未上传到公开仓库）：

- `O:/Project/FOODBALL/tmp/r13-03-frontend.log`：96 步完整门禁。
- `O:/Project/FOODBALL/tmp/r13-03-architecture-final.log`：完整架构门禁。
- `O:/Project/FOODBALL/tmp/r13-03-build-final.log`：最终构建。
- `O:/Project/FOODBALL/tmp/r13-03-browser-geCHPt/result.json`、`rules.png`：最终 28 项实际浏览器检查与截图。
- `O:/Project/FOODBALL/tmp/r13-03-native-build.log`、`r13-03-native-bdJrUs/startup-report.json`：Windows 原生构建与最终 startup。

开发/检查进程先加载 `O:/Project/FOODBALL/project-temp.ps1`，TEMP/TMP 为 `O:/Project/FOODBALL/tmp`，已单独提醒用户。现有测试清理通过保留产物 preload 拦截，不改变断言或生产行为；Vite 直接运行包 CLI/服务，绕开会清理依赖目录的开发包装脚本。原生运行只隔离 runtime logs，仍使用真实 Windows 配置目录，未声称配置已隔离；本轮不执行配置、重置或真实业务写入。

## 场景映射、待验与回退

- **R13-03-V1**：生产层级同步和身份断言、实际浏览器切换 A→B、延迟 preview 丢弃均通过；旧子级不会提交到 B，旧响应不会发布。
- **R13-03-V2**：生产测试和实际浏览器均验证注册/绑定后端拒绝保留草稿，引用删除拒绝不删除/不成功提示，取消不调用删除 API；真实 PG 引用/事务保护仍须最终新库验证。
- **R13-03-V3**：预览与绑定写入代次隔离，实际 UI 保存修改后的表单范围后，下一次 preview 再次调用后端并显示新绑定；全程未调用模型执行。原本没有“更新已有绑定”命令，本批保留既有建立绑定语义，不凭空声称实现了更新接口。
- **BA-01**：本批迁移的完整生产链、单 owner、公共出口和原门禁消费者已同步；其他 Feature 按阶段计划继续，不把整个历史账本一次勾完。
- **BA-14 / R17-06**：真实 PG 赛事层级写入/引用停用保护/规则注册与绑定持久化/路由结果、最终专用新库、Full、有效 XLSX 和私有 P4/P7 固定回归逐项承接；此处不关闭。

回退点为 `6f40a742`。未推送时可审阅后逐项恢复本批路径；推送后使用单独 revert 提交，不覆盖远端历史，不批量删除文件或目录。当前只交付 03（含原04），05 待本批收尾及用户授权。

## 本次推送与后续收尾入口

代码批次为 `12ba495c583ea25fd32b508aa68cceea9407f9d0`，已推送 `origin/rewrite/r13-core-ui-features`。单次查询获得 run41110961979，状态运行中。未下载本批产物，也未用02的产物替代03。后续收尾只核此精确代码的最终运行、证据包摘要、MSI/NSIS和startup；完成前保持 VERIFYING，不启动05。此后状态记录提交只修改文档，不改变受测代码。

## 2026-10-31 CI入口修复与原04核对

前一轮本机门禁通过，但本机父目录已有 TypeScript，未覆盖 CI 的全新依赖状态。这是本批测试接线引入的问题，不是环境偶发错误。CI 顺序为：架构静态门禁 → Windows Automated 中的 setup → 前端运行期/构建。将运行期测试挂在三个架构链守卫末尾，导致未安装 TypeScript 时加载失败。

本次只调整四个现有验证入口：三个 Competition/Route 守卫退出运行期导入与调用，原断言不改；`verify-browser-bootstrap.mjs` 在现有依赖安装后的前端门禁中执行原31项（10层级/13绑定与包/8路由），任何一项失败仍使门禁失败。未删除测试、修改生产代码、降低断言、添加依赖或修改CI工作流，96步入口数量保持。

复现与修复证据：从 `1bd4f393` 导出独立副本，确认 `require.resolve("typescript")` 为 MODULE_NOT_FOUND；原完整 `npm run verify:architecture` 重现相同错误。只同步这四个修复脚本后，保持 TypeScript 缺席，完整架构门禁通过。原工程新前端入口的31项生产行为通过；完整96步复验正在运行，完成后追加结果。

日志：`O:/Project/FOODBALL/tmp/r13-03-ci-clean-before.log`（预期失败）、`r13-03-ci-clean-after.log`（通过）、`r13-03-ci-fix-frontend.log`（本轮完整门禁）。独立副本为测试产物，进程TEMP/TMP仍在项目tmp；未清理或删除任何目录。

| 原R13-04义务 | 已核对的实际路径和证据 |
| --- | --- |
| 规则包文件/草稿/校验 | `RulePackageImport` + `RulesFeature.readPackage/registerPackage` → 原 `api.registerRulePackage`；最后文件有效、形状预检、后端默认字段/合法性、失败草稿保留与A注册不清B，原行为及28项浏览器证据覆盖 |
| 绑定与目录 | `bindingView` + `RulesFeature.createBinding` + `RulesWorkflow.saveBinding` → 原 `createCompetitionBinding`；真实ID/作用范围/优先级 DTO 保留，无虚构更新命令 |
| 路由与父级变化 | `RulesWorkflow.resolve` + `hierarchyContext` + `routeView`；Session/DOM/代次失效，后端结果唯一，旧结果不能授权执行；V1/V3及公开视图转义检查覆盖 |
| 失败与危险操作 | 后端拒绝不刷新成虚假成功，取消不写入，当前Session确认保护；V2及注册/绑定失败恢复检查覆盖 |
| UI组合与唯一owner | 原rules导航和四页签组合两个Feature，Workspace/Session/Modal公共owner保留，R12视觉不变；原04不重复创建新模块或新任务编号 |
| 最终验收边界 | 本轮CI失败修复后仍需精确新提交的CI、MSI/NSIS/证据摘要/startup；真实PG/Full仍BA-14/R17-06。原04不得先于合并主交付标DONE |

当前结论：原04已实现并核对，随03共同VERIFYING；下一主交付为05，本轮没有将“开始04”擅自当作05实施授权。
