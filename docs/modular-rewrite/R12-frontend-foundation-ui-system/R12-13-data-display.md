# R12-13 Data Display

状态：`DONE`（2026-10-28收尾）。基线`780753483310ccbc851da34b739007998e6ece46`；唯一rewrite/r12-frontend-foundation-ui-system。13精确dac0e9c的Windows run40218221375、ZIP摘要、Rust、打包与startup报告已核验；详文末收尾。14由用户另行授权，15～19仍BLOCKED。以下验收前描述保留为实施历史。

## 前置核验

12精确提交`780753483310ccbc851da34b739007998e6ece46`的[run40194352653](https://github.com/uniquenesssta/123/actions/runs/40194352653)、job121171885561、attempt1全部步骤SUCCESS。读取完整311,854字符日志：45组Rust结果均ok/0 failed，ignored环境用例仍为ignored。前端91步骤验证与打包各通过一次，272 Forms/868 Primitives/1344 Tokens/1104 Shell/136告警/17视口，Rust fmt/Clippy/工作区测试及release/MSI/NSIS通过。artifact12599627358未过期，ZIP17,863,027字节，下载SHA-256 `0afe169b458a79fccef400991b28cce7e92419ea63f5f330844642dc553777fb`与API及上传日志一致。MSI8,392,704字节，NSIS5,922,437字节。报告`windows-acceptance-20261025-124423.json` startup PASS：7有效条目、0无效行、0运行错误；会话`e0caf002-66f2-4e68-8eb8-b6cb6d875f41`内read_workspace_state/save_workspace_state/bootstrap三组trace配对。原ZIP与完整日志在仓库上一级`R12-12-windows-evidence.zip`、`r12-12-ci-full.log`。一次交接核验，无轮询；startup不替代Windows Full/真实PG/XLSX/私有固定回归。

## 实现与入口切换

八类展示按职责拆到唯一 [data-display](../../../src/ui/data-display/) 目录，各目录的 index.ts 只显式导出；类型、视图、控制器、样式分别归属。唯一 [CSS 入口](../../../src/ui/data-display/index.css) 由 main.ts 导入一次。保持既有 Primitives、Forms、设计变量、图标、EventScope 和任务状态所有者，未新增生产依赖或跨模块状态（仍31 owner）。

| 组件 | 真实调用及行为 |
| --- | --- |
| Data Table | 数据库页的 BootstrapResponse 表统计；原生 table/caption/th/td，默认/加载/空、数字/动作列、混合选择及排序手势。控制器只上报宿主回调；宿主决定行序、选择和请求，本批未增加数据库排序接口。加载保留数据并禁用操作。 |
| Key Value List | 通用详情弹窗，原生 dl/dt/dd；标量格式、技术字段展开、数组限制及 ModalController 原行为保留。复杂值由显式应用渲染器负责，不把不可信文本当 HTML。 |
| Stat Card | 数据库3项、日志4项原统计值；零值保留，指标20/24/600，标签11/16，说明10/14。统计推导与原始前缀表达不变。 |
| Status Badge | 组合既有 Badge 的语义薄层，状态由宿主传入；用于表格和时间线，不在UI猜测领域状态。 |
| Timeline | 复盘事件通过专属 matchEventTimelineView 呈现；verification 与 revision 两个轴分开，9种已接受组合覆盖，DTO 额外 Superseded 明示“已被替代”。保留输入顺序、原描述、参与者、元信息和取消/被替代记录，不重算比分与后端摘要。缺失比分不显示，真实0-0保留。取消仅标题删除线，详情仍可读。 |
| Progress | 任务活动提示接入不确定进度；已知值仅做显示范围约束，不生成业务进度。role/aria、2/4px轨道、减少动态和强制颜色有覆盖，无新增定时器或请求。 |
| Empty State | 日志空状态与时间线空状态；可选操作复用现有 Button，真实调用仍走原宿主动作。表格自身空态遵循独立140px解剖。 |
| Code Block | 日志原始技术消息用转义的 pre/code，保留换行、选择、滚动和11/17 Cascadia字体；无新增剪贴板、日志采集或脱敏职责。 |

旧事件标签/分钟和投影从 review.ts 收敛到 [review 展示适配器](../../../src/pages/review/matchEventTimelineView.ts)；旧合并状态 helper 被双轴映射替代。app.css 中17条旧时间线样式退出，包括取消整行 opacity。详情、数据库和日志的重复展示切换新组件，旧实现未并存；无文件删除、移动或重命名。

main.ts 除唯一 CSS import 外与基线完全一致；ModalController、visualScalar、visualLabel、isTechnicalVisualKey 与基线逐声明一致；databasePage/logsPage 的统计推导逐段一致。Rust、API、DTO、Schema、配置格式、存储、领域计算及171命令均未变；没有新增错误码、外部日志格式或 I/O。

## 设计依据与决策

已核对 ui-design-system 头 `40109ceec8372391a7680e7e86c1265fda06ab3d` 的 accepted 文档及 Figma 文件 PN0Whgu6HLWIHx4Mv6aHfu，读取设计上下文和截图，未写 Figma、未发布 Code Connect。逐族节点、两份原始12px排序 SVG 的根属性与 SHA-256 见 [设计映射](R12-13-component-mappings.json)。没有把独立 Code Block 母版或 Superseded 视觉变体伪造为设计来源：前者沿 LogsLongDetails 技术文字槽，后者组合已有中性 Badge 并保留真实 DTO。

表格头34/行36/横padding9，行内32px动作可容纳；12px排序原导出不以16px近似图标替代。数字列12px/600，元信息说明常规400；键值100px标签列、指标20/24、Timeline64px时间列与16px图轨、Progress2/4、Code11/17均在原生计算值检查。原14族控件、7族表单和token未改设计基线或PNG阈值。

## 验证结果与证据边界

验证环境为 Windows 原生 Chromium、960/1440 × Light/Dark、DPR1、截图高2100；使用原CDP截图 helper，所有浏览器/构建命令保留产物，未执行递归清理。页面级夹具调用真实生产视图，但记录为明确测试数据，不冒充连接真实PG。

| 项目 | 实际结果 |
| --- | --- |
| TypeScript、前端脚本及构建 | `npm run verify:frontend` 最终exit0，原91步骤完整通过；Vite208模块，CSS449.12kB/gzip70.46，主JS623.44kB/gzip178.13，辅助JS6.31kB/gzip2.59。原>500kB警告保留，未放宽门禁。 |
| 新组件行为 | 80项/组 ×4 =320项PASS：结构/尺寸/语义/转义/零与缺失值/加载/空/独立状态轴/真实页面投影；CDP Enter排序回调、Space选择/全选、Tab与方向键滚动、减少动态、强制颜色、销毁重挂载。 |
| 既有回归 | 272 Forms、868 Primitives、1344 Tokens、1104 Shell/Workspace、136告警、17既有截图视口全部PASS；未改基线和阈值。 |
| 截图复核 | 最终 display-960-light.png、display-1440-dark.png 已目视复核，数字字重修正包含在最终320项及截图中。其余两组也保存截图及结构化结果。 |
| 架构与格式 | `npm run verify:architecture`、`cargo fmt --all -- --check` exit0；31共享状态所有者，职责和依赖门禁通过。 |
| 保护资产 | 原18项指纹PASS：`d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`。 |
| 13自身Rust/Windows验收 | Clippy、工作区测试、Windows release/MSI/NSIS、实际Tauri启动及startup日志待本批提交后运行并按精确commit核验；12成功仅是前置。13保持VERIFYING。 |

完整最终日志保存在仓库上一级 `r12-13-frontend-final.log`（exit0）；架构日志 `r12-13-architecture.log`；截图与结构化结果位于原CI产物目录 `../logs/r12-13-display-1792949785342`，临时原件 `football-r12-display-cgGnYi`。初始完整通过日志 `r12-13-frontend-first.log` 为308项，最终增加3项/组字体断言后全量重跑为320项。日志/浏览器profile/辅助脚本均留在仓库外，不提交。

初次验证暴露不存在的token别名、夹具百分比精度与原格式不一致，以及新CDP驱动漏发Enter文本；分别使用已存在的tertiary token、遵守原75.00%格式、对齐既有驱动的回车文本后通过。没有以合成click替代原生键盘，没有关闭/跳过检查。截图发现数字字重与说明文字继承问题，已在本族样式中修复并增加实际计算值断言。

未执行真实PG往返、数据库清空、Windows Full、操作系统IME、屏幕阅读器、所有DPI、大规模表格性能与私有P4/P7固定回归；不可变账本/并发/回滚/有效XLSX/真实API凭据等仍按原计划在对应最终环境验收。测试夹具、CI公共stub和ignored均不能替代这些结果。

## 变更清单与回退

共54个路径：36新增、18修改，无文件删除。48个非Markdown变更文件的路径/字节SHA-256聚合：`296fba228e02f03fe29f8f5c24906d14af337025b5ee9bdd6fc2ff83bba70916`；精确清单和逐文件摘要位于仓库外 `r12-13-source-proof.json`。该审计同时锁定main运行逻辑、Modal控制器/格式化及页面统计推导不变。

### 新增（36）

- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-13-component-mappings.json](R12-13-component-mappings.json)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-13-data-display.md](R12-13-data-display.md)
- [scripts/data-display/verifyDataDisplay.mjs](../../../scripts/data-display/verifyDataDisplay.mjs)
- [src/assets/icons/chevron-sort-down.svg](../../../src/assets/icons/chevron-sort-down.svg)
- [src/assets/icons/chevron-up.svg](../../../src/assets/icons/chevron-up.svg)
- [src/pages/review/index.ts](../../../src/pages/review/index.ts)
- [src/pages/review/matchEventTimelineView.ts](../../../src/pages/review/matchEventTimelineView.ts)
- [src/ui/data-display/__tests__/displayBrowserFixture.ts](../../../src/ui/data-display/__tests__/displayBrowserFixture.ts)
- [src/ui/data-display/code-block/codeBlock.css](../../../src/ui/data-display/code-block/codeBlock.css)
- [src/ui/data-display/code-block/codeBlockView.ts](../../../src/ui/data-display/code-block/codeBlockView.ts)
- [src/ui/data-display/code-block/index.ts](../../../src/ui/data-display/code-block/index.ts)
- [src/ui/data-display/data-table/dataTable.css](../../../src/ui/data-display/data-table/dataTable.css)
- [src/ui/data-display/data-table/dataTableController.ts](../../../src/ui/data-display/data-table/dataTableController.ts)
- [src/ui/data-display/data-table/dataTableTypes.ts](../../../src/ui/data-display/data-table/dataTableTypes.ts)
- [src/ui/data-display/data-table/dataTableView.ts](../../../src/ui/data-display/data-table/dataTableView.ts)
- [src/ui/data-display/data-table/index.ts](../../../src/ui/data-display/data-table/index.ts)
- [src/ui/data-display/data-table/tableCellView.ts](../../../src/ui/data-display/data-table/tableCellView.ts)
- [src/ui/data-display/empty-state/emptyState.css](../../../src/ui/data-display/empty-state/emptyState.css)
- [src/ui/data-display/empty-state/emptyStateView.ts](../../../src/ui/data-display/empty-state/emptyStateView.ts)
- [src/ui/data-display/empty-state/index.ts](../../../src/ui/data-display/empty-state/index.ts)
- [src/ui/data-display/index.css](../../../src/ui/data-display/index.css)
- [src/ui/data-display/key-value-list/index.ts](../../../src/ui/data-display/key-value-list/index.ts)
- [src/ui/data-display/key-value-list/keyValueList.css](../../../src/ui/data-display/key-value-list/keyValueList.css)
- [src/ui/data-display/key-value-list/keyValueListView.ts](../../../src/ui/data-display/key-value-list/keyValueListView.ts)
- [src/ui/data-display/progress/index.ts](../../../src/ui/data-display/progress/index.ts)
- [src/ui/data-display/progress/progress.css](../../../src/ui/data-display/progress/progress.css)
- [src/ui/data-display/progress/progressView.ts](../../../src/ui/data-display/progress/progressView.ts)
- [src/ui/data-display/stat-card/index.ts](../../../src/ui/data-display/stat-card/index.ts)
- [src/ui/data-display/stat-card/statCard.css](../../../src/ui/data-display/stat-card/statCard.css)
- [src/ui/data-display/stat-card/statCardView.ts](../../../src/ui/data-display/stat-card/statCardView.ts)
- [src/ui/data-display/status-badge/index.ts](../../../src/ui/data-display/status-badge/index.ts)
- [src/ui/data-display/status-badge/statusBadge.css](../../../src/ui/data-display/status-badge/statusBadge.css)
- [src/ui/data-display/status-badge/statusBadgeView.ts](../../../src/ui/data-display/status-badge/statusBadgeView.ts)
- [src/ui/data-display/timeline/index.ts](../../../src/ui/data-display/timeline/index.ts)
- [src/ui/data-display/timeline/timeline.css](../../../src/ui/data-display/timeline/timeline.css)
- [src/ui/data-display/timeline/timelineView.ts](../../../src/ui/data-display/timeline/timelineView.ts)

### 修改（18）

- [architecture/module-boundaries.json](../../../architecture/module-boundaries.json)
- [docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md](../../football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-12-forms.md](R12-12-forms.md)
- [docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md](README.md)
- [docs/TESTING.md](../../TESTING.md)
- [README.md](../../../README.md)
- [scripts/verify-global-visual-system.mjs](../../../scripts/verify-global-visual-system.mjs)
- [scripts/verify-match-event-facts.mjs](../../../scripts/verify-match-event-facts.mjs)
- [src/app/modal.ts](../../../src/app/modal.ts)
- [src/main.ts](../../../src/main.ts)
- [src/pages/database.ts](../../../src/pages/database.ts)
- [src/pages/logs.ts](../../../src/pages/logs.ts)
- [src/pages/review.ts](../../../src/pages/review.ts)
- [src/styles/app.css](../../../src/styles/app.css)
- [src/ui/feedback/task-activity/taskActivity.css](../../../src/ui/feedback/task-activity/taskActivity.css)
- [src/ui/feedback/task-activity/taskActivityView.ts](../../../src/ui/feedback/task-activity/taskActivityView.ts)
- [src/ui/icons/iconSlot.css](../../../src/ui/icons/iconSlot.css)
- [src/ui/icons/iconSlot.ts](../../../src/ui/icons/iconSlot.ts)

移动/重命名：无。删除文件：无。

所有代码拟以本批同一可回退提交发布。发布前保留基线 `780753483310ccbc851da34b739007998e6ece46`；发布后需要回退时对该原子提交执行审阅后的 git revert，不手工拷贝旧文件、不批量删除目录。13自身Windows成功前不进入14；按用户要求不轮询CI。


## 自身 Windows 验收收尾（2026-10-28）

精确提交 `dac0e9cca8006ceabdbe24f845da84c6710a0723` 的 [run40218221375](https://github.com/uniquenesssta/123/actions/runs/40218221375) 及 Windows job121243891804 全部步骤 SUCCESS。artifact12609027463 的远端与本地 ZIP 摘要一致：`e24a76588b76b4d092c651e7700188845e29ac61b997b26896b1d2c1e5dbb39a`，18,989,664 bytes。完整日志确认 Rust 工作区、MSI/NSIS 打包通过；ZIP 内 startup report 为 pass、无 runtime_errors，完成 bootstrap/read_workspace_state/save_workspace_state。13 改为 DONE，不能把 startup profile 说成 Windows Full 或真实业务端到端通过。用户随后授权 14；不轮询 CI。
