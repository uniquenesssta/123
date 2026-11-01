# R13-09 工作簿导出、预检与提交

状态：**VERIFYING**。本地实现及受影响验证已完成；自身精确提交的 CI、双安装包和包内 startup 尚待推送后验收，当前不能标记 DONE。分支 `rewrite/r13-core-ui-features`；修改基线 `5ee217c2d808e4a835067f0b5de4124b1ab3d2f6`，初始工作区干净。UI01 的精确 CI/产物已独立核验并正式 DONE，收尾提交 `4ff08d7f`。沿用用户提交推送长期授权，不轮询 PR 或 CI，不重开 R12。

## 范围与唯一归属

工作簿页面仅组合既有 Team、Player、Match 公共流程；`WorkbooksFeature`只持有类别确认代次、导出进行状态和反馈，不存业务行、批次副本或第二套账本。类别、选择、焦点、滚动仍由已有 Workspace 持有。

| 格式 / 原入口 | 导出与预检 | 冲突、提交与恢复 / 结果 |
| --- | --- | --- |
| 球队月度 XLSX / S09、球队工作包 | `TeamsFeature.exportFile` → 原 `exportTeamMonthlyTemplate/Data`；`TeamImports.monthlyBatch` → 原 monthly preview | 同一 Team batch 调用原 resolve/commit/read API；完整球队资料包不在此混用 |
| 球员目录及月度 XLSX / S09、球员工作包 | `PlayersFeature` → 原 catalog template/data/preview API，原后端格式识别保留 | `PlayerImports` 实例持有原批次；目录和月度不创建第二套 owner |
| 比赛阵容 XLSX / S09、比赛中心 | `createMatchExchange` → 原 lineup template/data/preview API；数据导出指定当前真实比赛 | `MatchImports` 实例调用原 resolve/commit/read；保留新增、更新、结束旧记录及错误统计 |
| 球队完整资料包 / 球队页 | R13-05 已交付的完整包导出/预览保持；未改为月度表 | 原 Team package workflow 与确认保持，字段归一化及中文名检查继续执行 |
| AI 比赛 ZIP / 比赛中心 | 原 AI 导出与 ZIP 预检入口保持 | 与 XLSX 共用同一个 MatchImports 实例和原 commit/read，不复制导入状态 |
| 复盘包 / 原复盘入口 | 原 package 格式、校验能力保留 | 事实写入与跨阶段调用由 R14-08 联验；不在本批伪造完成 |

`src/app/imports/spreadsheetBatch.ts` 是注入 API 的共享工作流实现，每个领域各自实例化；不直接访问 client。`importRecovery.ts` 复用现有 PreferenceStore，只保存批次 ID、文件 SHA256、模式和文件名；按数据库端点/库名/角色隔离，不保存密码、原文件路径或业务行。

原月度 Team/Player/Match 私有状态迁入各自实例，原 Feature 方法和事件继续调用这些实例。root 只注入现有 Session、Platform 文件选择、Modal 和 Workspace；Wire DTO、Rust、Schema、锁文件和生产依赖未改。module/state/style 契约同步登记：19 Feature、54 个状态 owner、185 份 CSS 单入口，无文件删除。

## 批次安全与实际行为

- 预检保留后端真实行号、对象 ID、原候选、错误及原始批次；摘要展示最多 80 行，完整明细仍可打开。
- 空文件选择不预检、不提交；重新选文件或修改模式使旧确认失效。busy 阻止第二次选择、决议和提交，旧页面/类别/数据库的迟到结果不覆盖当前视图。
- 提交确认绑定类别、文件名、SHA256、批次、导入模式和页面代次；Team → Player → Team 不能复用旧确认。
- 无变更、冲突/错误、已导入、写入中及结果未知各有独立状态。成功消费旧提交资格；重新选择并成功预检的新批次可再次提交。
- 写入前必须保存恢复引用；保存失败不发送 commit。传输中断后保留原引用并禁止盲目重提；返回页面只读原 batch/hash/mode 核查。
- 当前预检 DTO 没有事务终态字段。`imported=0` 不能证明回滚，因此继续保持“结果未核实”；本批不伪造“失败已回滚”。真实账本/事务闭环沿 BA-04、R17-06。
- 比赛现有数据从真实 managed matches 选择；确认框显示实际对阵和稳定 ID，确认及保存窗口均核对原目标。取消保存不写文件、不误报成功；取消和类别返回均保留原选择。

## S09 设计与接入

权威设计为 `origin/ui-design-system` 的 `docs/FIGMA_SCREEN_WORKBOOKS_STATE.json`，Figma 文件 `PN0Whgu6HLWIHx4Mv6aHfu`。Team Idle `579:102496`、Ready `579:105097`、Match Idle `579:104276` / Ready `579:119360` / Blocked `579:120268`、Compact `579:126968` / Ready `579:127689`、Dark `579:125239` / Ready `579:126060`、Rules `579:121209`。

接入分类竖排、范围说明、导出/预检/确认三步、真实表格与固定底部动作。目录宽 220px，主区域间距 8px，面板内部间距 12px，圆角 16px；只滚动主内容，底部在滚动区之外；窄屏步骤纵排，规则面板改覆盖层。复用已有图标，没有新增演示数据或图片填充。

实际计算值已核对：标题/汇总 14px、700、20px 行高；正文 12px、400、18px；标签 13px、500、18px；注释 11px、400、16px；表头 36px、行 56px、图标 16×16。明暗主题和 1440×900、960×720、1280×720、1920×1080 均进入真实 Shell/Session/Workspace 挂载验证。

原比赛数据导出弹窗 `582:112433` 只写“范围在实现时确认”。本批补齐具体规则：可编辑范围说明 [1151:376779](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=1151-376779)，复用批准 Select `61:122` 及原 Noto Sans SC 样式；同步更新原弹窗说明 `I582:112435;278:2270`。新增说明无位图、5 个可编辑文本节点，产品字体断言通过；没有改写原型演示对象为真实数据。最终可编辑范围面板截图已通过（Plugin API screenshot；独立 get_screenshot 超时后采用同节点内联截图）。

## 验证分层

全部本地证据根：`O:/Project/FOODBALL/logs/r13-09-20261101`。测试进程 TEMP/TMP 为 `O:/Project/FOODBALL/tmp`，不是用户系统 Temp。

| 层级 | 已完成的结果与证据 |
| --- | --- |
| 静态 / 类型 | 模块边界、54 状态 owner、185 CSS / 464 受限选择器 / 15 负向变异均通过；TypeScript 通过。没有关闭或降低原检查 |
| 完整前端 | `frontend-verified.log`，现有 96 步退出 0，包括原 80 设计场景 / 1648 检查、UI01 126 完整挂载及当时 46 项 S09 检查 |
| 最终增量行为 | `behavior-scope-final.log`：24 个新生产工作流场景通过；原 Team 42、Player 30、Match 8 项再次通过。调用生产模块，未复制测试专用状态机 |
| 最终 Windows 浏览器 | `mounted-final-selected/results.json`、`browser-selected-final.log`：55 项通过，完整 main / Shell / Session / Workspace，只有原生传输边界为明确 QA fixture；含两主题四尺寸、焦点、取消、原批次恢复、具体比赛导出与选择保持 |
| 计算样式 / 截图 | `typography-final/typography.json` 与上述最终浏览器截图；实际字体层级、间距、表格和图标几何核验通过 |
| 最终构建 | `production-selected-final.log`：受影响修改后的 TypeScript / Vite 通过；既有大块体积提示保留，没有关闭告警 |
| Impeccable | `mechanical-selected-final.json`，0.1.17 source detector 退出 0、无发现；仅证明源码检测，不替代截图和交互 |
| Windows 原生构建 | `native-build.log`，`cargo build --locked --manifest-path src-tauri/Cargo.toml` 退出 0 |
| Rust 测试 | 本批未重复全套 Rust 测试；`crates` 树 `e96159c867e4a011f2bf3b18a481f683fac78546`、`src-tauri` 树 `1c35e0d2be3925d0fea34489f3c54f34f90b8fe7`、Cargo.lock blob `0cb8b1abcf72d29fb4f0be6641e253b673a60a8a` 不变，复用 R13-07 同树 fmt/Clippy/748 passed、40 ignored 证据 |
| Windows 实际原生运行 | 无 QA fixture；Team/Player 真实界面与保存窗口导出完成，原生操作日志有对应完成项。用户停止桌面自动化并要求命令操作后，Match 通过实际原生 IPC 导出；命令响应与文件独立校验。`native-summary.json`、`native-command/export-report.json`、`native-command/startup-report.json` |
| 自身 CI / 安装包 | 尚待推送后精确验收；UI01 的成功不能代替 09 |
| Full / 真实外部链路 | 未宣称通过；未对业务库导入、重置或作其他业务写入 |

完整 96 步在后续目标绑定/取消恢复修正前运行；修正后重新执行上列受影响行为、55 项完整挂载、静态/类型/构建，不能把最终增量数量写成旧完整运行的一部分。早期换行归一化、截图热更新干扰、辅助脚本重复声明和新增取消恢复断言的失败日志保留；实际问题修复并复验，不以放宽断言通过。

三份真实 XLSX 的 ZIP 完整性、工作表目录及摘要见 `xlsx-manifest.json`：

| 文件 | 字节 | SHA256 |
| --- | --- | --- |
| `native-final/team-native-template.xlsx` | 16,733 | `49db5330b1bf2170747ce09c29d56e2c1d8a1231cefce40f11dd8ce184672d33` |
| `native-final/player-native-template.xlsx` | 17,417 | `d521c4167efd889f8077f77129f3d5ac9f6d6821219850cc311c901703096e3b` |
| `native-final/match-native-template.xlsx` | 24,947 | `2fd349fca2fb959653cf8b26c51e9854053e083d8a1e4009b88b4659c5d5fb4e` |

Match 的直接 IPC 不经过前端操作记录器，不能冒称原 UI 日志中有该导出完成项；以原生命令响应和实际工作簿为证。两次用户 Escape 后已停止 Computer Use。此处仅验证模板导出及启动，不是数据导出或 XLSX → PG 往返通过。

## 场景逐项与剩余账本

| 要求 | 本批证据 / 尚未核销部分 |
| --- | --- |
| V1 外部 ID 重复 / 冲突 | 原 row identity、月度、完整包归一化/中文名、复盘包检查由完整前端运行通过；新增决议断言保持原 row/candidate。真实实体与审计一致性仍需 PG，不用 QA 样例核销 |
| V2 替换 A/B / 取消 | 生产行为和挂载确认测试通过，旧 ticket 不能写入，取消不调用 preview/commit/导出 API |
| V3 中断 / 重开 | 原批次引用、读回、禁止重发通过；真实计数、事务回滚及审计一致性按 BA-04、R17-06 继续 |
| D1 类别隔离 | 三类别独立实例，类别 ABA 使旧确认失效；原预检在返回时保持 |
| D2 布局及动作 | 两主题四尺寸、真实表格、内部滚动、固定 footer、焦点与 disabled 状态通过；具体实际 PG 数据截图仍随真实链路补验 |
| BA-02 / BA-04 / BA-07 | 历史四项、实际阵容 ended_previous_count / 账本事务、有效 XLSX → PG 读回，按既有批准安排移交 R17-06；不改写 DONE、不假称已清零 |

## 清理、回退与交接

只保留最终证据、可复用依赖及必要复现助手。可清理的本批浏览器 profile / 构建辅助临时项列出绝对路径清单；不递归或批量删除，不触碰其他任务或用户 Temp。本批日志明确引用的临时目录已按清单可逆归档至 `O:/Project/FOODBALL/tmp/old/r13-09-20261101`，未删除文件、未声称释放磁盘。最终完整设计证据移入本证据根 `approved-design`，最终55项挂载、字体、原生文件与日志保留。`cleanup-manifest.json`记录所有源/目标，旧报告中的原路径可按此追溯；归档项供用户手动删除。

代码回退点为 `4ff08d7f`（仅已验收 UI01 收尾与 09 启动记录）；如需撤销，审阅后针对本批提交 revert。Git 回退不能恢复真实数据库；本批未修改 Schema 或执行业务数据写入。

下一项为 **R14-01 输入、准入与推演执行**（覆盖原 R14-01～05）。须先完成本批精确 CI、安装包与 startup 收尾并取得启动授权；不自动进入下一阶段。
