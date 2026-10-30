# R13-03 赛事层级、规则包与路由

状态：**VERIFYING**。2026-10-31，用户“收尾02开始03”授权；覆盖原 R13-03 Competitions 与 R13-04 Rules。02 已按 `d167e948` 精确 Windows CI、下载产物、双 bundle 和 startup 收尾，收尾提交 `6f40a742`。03 本地实现与适用验证已完成，提交后仅核查自身精确代码 CI 一次，不轮询；尚未完成自身 CI/安装包验收，不提前标 DONE。05 未启动。

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
| 自动生产行为 | 原 hierarchy/bindings/route-resolution 三个入口接入 `scripts/features/verifyCompetitionRulesWorkflow.mjs`；最终 **10 + 13 + 8 = 31** 项通过，实际加载生产类，只有 API/Session/DOM 边界使用夹具 |
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
