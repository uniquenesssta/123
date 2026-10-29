# R12-14 Feedback 与 Overlays

状态：VERIFYING（本地实现完成，发布批次的 Windows 打包/启动 CI 待提交后单独验收）。2026-10-28 用户授权，在最新 UI 刷新基线 518700f59a7de451afbe9655b91abe8776a32adb 上实施。沿 rewrite/r12-frontend-foundation-ui-system，工作目录 O:/Project/FOODBALL/123 r3。15～19 未实施，不轮询 CI。

## 前置与范围

13 精确 dac0e9cca8006ceabdbe24f845da84c6710a0723 的 Windows run40218221375、job121243891804、artifact12609027463 已核验。ZIP SHA-256 e24a76588b76b4d092c651e7700188845e29ac61b997b26896b1d2c1e5dbb39a，18,989,664 bytes；Rust、MSI/NSIS 和 startup profile pass。13 DONE；startup 不等于 Windows Full。518700f 的公开推送完成，不把其本地结果说成自身 CI。

14 只负责 alert/toast/inline-message、loading/error/success、dialog/confirmation、drawer、popover、command-menu。保留真实 WarningStore、runBusy 并发门禁、详情历史与草稿、业务删除保护和后端名称二次校验；没有新增生产依赖、API/DTO/Schema/迁移或模型规则。

## Figma 先行与视觉权威

文件 PN0Whgu6HLWIHx4Mv6aHfu，本次 session 1fdcfbb2-9ec1-42cc-95de-640d76e23b89。先读取 Dialog279:2337、Toast302:4062、Inline303:4193、Blocking304:4246、Menu258:1755 的高保真上下文与截图，再写应用。库发现和本地搜索没有 Drawer、Popover、Command Menu 等价母版，故使用已有 FIP Header/Button/Field/Menu/Alert/Spinner 实例补齐，不引入另一套设计系统。

- Drawer set1134:6849：Default1134:6846、Busy1134:6847、Error1134:6848。
- Command Menu set1134:7176：Default1134:7172、Empty1134:7173、Busy1134:7174、Error1134:7175。
- Popover set1134:7124：Actions1134:7122、Empty1134:7123。
- Controls review1134:7361：9 状态、18 个 Light/Dark 实例；母版结构审计无可见溢出，补齐中文空状态、圆角裁切与实际浮层阴影。

新扩展记录见 [R12-14-figma-extension.json](R12-14-figma-extension.json)。原237变量/25文字/6 effect历史快照保持；新 radius/panel16（VariableID:1134:6844）、radius/surface12（VariableID:1136:7873）及三项 Light role effects 独立登记，不覆盖旧记录。Modal Light 0 8 32 / .16、Toast Light 0 4 16 / .10、Menu Light 0 4 16 / .12；Dark 继续复用批准 Dialog/Subtle effect。正文12/18/400、反馈标题14/20/700、浮层标题18/26/700；复用 feature 文字令牌、Noto Sans SC 和已有语义色。

## 职责与真实接入

| 唯一 owner | 行为与消费者 |
|---|---|
| ui/feedback | 内容、语义 Alert/Inline、状态、Blocking；Toast 保留现有单槽 identity/3200ms/悬停和焦点暂停；警告/错误/含动作通知持续显示，关闭不取消任务或释放验证门禁 |
| ui/overlays/dialog | 原生模态、可用控件焦点边界、Esc/遮罩策略、焦点恢复；没有请求和业务 I/O |
| confirmation-dialog | 一次请求/epoch、同步与异步结果、忙碌防重复、Typed Input 生命周期、失败保稿；危险结果未知时先核对再重发，禁止原弹窗盲目重试 |
| drawer | 480px 右侧原生模态，12px 画布边距，标题与动作固定，仅 body 滚动；接真实 WarningStore 日期分组、已读和日志目的地 |
| popover | 240px 可见视口夹取、统一16px图标列/8px空隙、1px危险组分隔、方向键/Home/End/禁用/Esc/外部滚动与监听器销毁；在所属 dialog 中挂载，内层 Esc 不关闭父层 |
| command-menu | 480px 原生搜索、结果/禁用/空/忙碌/错误、键盘与 IME、query 保留/epoch；顶栏及 Ctrl+K 接 PRIMARY_NAVIGATION 的真实导航，不提供虚构业务操作 |
| app/context-menu | 仅把真实推演/阵容/比赛身份与影响说明映射给 Popover；原 data-action、后端闭包与保护链继续使用 |
| app/modal | visualRecordView 仅负责详情投影，workspacePanelController 仅负责原 RouteHistory/草稿/滚动/敏感排除，modalCoordinator 分离 transient 确认请求；成功仅关闭未被替换的旧详情，失败和取消保留原详情 |

真实数据库清空改为 typed confirmation，共用名称输入组件。保留全部不可撤销影响说明、大小写敏感 trim、主机执行前 confirmation !== expected、api.resetDatabase(confirmation) 和原后端保护/重连/清理顺序。本轮测试没有对真实数据库执行清空或删除。

反馈和浮层分别通过唯一 index.css 进入 main；旧中心弹窗几何与内联确认职责退出。唯一旧文件 src/app/modal.ts 已拆分替代，逐项删除；没有批量删除文件或目录。原详情页面仍是工作区 region，只有确需保护焦点的确认和浮层使用 native dialog。

## 验证证据

| 层次 | 实际结果/边界 |
|---|---|
| 静态/契约 | TypeScript、模块边界、33状态 owner、数据库清空保护及原历史链已通过；完整92步最终冻结前端门禁PASS（含TypeScript与Vite）；r12-14-frontend-frozen.log |
| Windows Chromium | 342 项（57×3宽度×2主题）通过；360/960/1440，明暗；原生 Tab/ShiftTab/Enter/Esc/输入/滚轮、精确名与 IME、防重复、忙碌关闭锁、失败保稿、未知结果门禁、菜单/嵌套 Esc、真实注册表搜索、过期完成/销毁。fixture 只模拟 UI promise/状态，不伪造生产数据或称为真实后端 E2E |
| 截图 | O:/Project/FOODBALL/logs/r12-14-overlays-1793195177093/，48 张；feedback/lower/confirmation/error/command/drawer/toast/popover ×3宽×2主题 |
| Rust格式 | cargo fmt --all -- --check PASS |
| Rust静态 | cargo clippy --locked --offline --workspace --all-targets -- -D warnings PASS；r12-14-clippy.log |
| Rust自动测试 | cargo test --locked --offline --workspace PASS：748 passed、0 failed、39 ignored。忽略型真实PG/私有固定回归未执行，不记为PASS；r12-14-cargo-test.log |
| Windows构建 | cargo build --locked --offline -p football-match-model-desktop PASS；实际Tauri开发可执行文件，非MSI/NSIS验收 |
| Windows实际启动 | 独立TAURI_CONFIG identifier com.footballmatchmodel.platform.r1214validation，不改生产配置。最终 startup report PASS（5条、bootstrap/read_workspace_state完成、无runtime_errors）；首次7条另含save。FOOTBALL_RUNTIME_ROOT/TEMP/TMP 均指向项目Temp。没有连接真实数据库，不冒充Full |
| Windows窗口交互 | 可读取真实WebView可访问性树，包含最新快速导航等入口；截图采集 FrameArrived timed out，点击 coordinate input geometry is unavailable。没有宣称该部分手动交互/截图通过；键盘和视觉证据来自原生Windows Chromium矩阵 |
| 模型保护 | 原18文件指纹/公开模型边界保持；生产Rust、命令和模型资产未修改。完整门禁再次核验 |

Impeccable 按已有批准系统扩展，CLI 上下文启动失败后使用人工上下文，没有宣称 detector/build-phase CLI gate 通过。独立 reviewer 的首次 recapture 要求已补齐48张。完整审阅发现混合图标错列和危险组分隔缺失，两项一次修复并补断言；同一 reviewer 对两项均评分 resolved，disposition ship 的最终范围限于这两项修复，不能扩大为 CI/打包/真实端到端认证。独立documenter已核对既有系统、设计扩展、源owner、真实adapter、最终矩阵和startup边界；交接报告保留在本任务visualizations目录，未另建DESIGN.md或侧表取代现有权威。。

## 发布、未验项与回退

当前 VERIFYING。14 自身公开发布及精确提交的 Windows CI/MSI/NSIS/启动证据尚未验收，不进15。真实PG、私有P4/P7固定回归和Windows Full仍需对应环境；不以ignored、公开stub、静态指纹代替。

所有日志和脚本过程级 TEMP/TMP 使用 O:/Project/FOODBALL/Temp。后续 PowerShell 先加载 O:/Project/FOODBALL/project-temp.ps1。源码基线518700f；最终以一项可回退原子提交交付，回退采用审阅后的 git revert，不手工复制旧实现或批量删除。

旧 C:/Users/uniqu/Documents/ChatGPT/项目代码模块性优化 足球/R12 已提交并保留。用户要求旧目录删除，但同时明确禁止批量删目录，故目录级删除留给用户手动处理；新开发已全部使用123 r3。


## 实际文件清单

相对基线518700f；移动/重命名：无（旧modal.ts按职责拆为多个文件，不是纯路径改名）。

### 新增（41）

- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-feedback-and-overlays.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-figma-extension.json
- scripts/overlays/browserBundle.mjs
- scripts/overlays/verifyOverlays.mjs
- scripts/verify-overlays.mjs
- src/app/context-menu/contextMenuAdapter.ts
- src/app/context-menu/index.ts
- src/app/modal/index.ts
- src/app/modal/modalCoordinator.ts
- src/app/modal/visualRecordView.ts
- src/app/modal/workspacePanelController.ts
- src/app/modal/workspacePanelTypes.ts
- src/ui/feedback/alert/alert.css
- src/ui/feedback/alert/alertView.ts
- src/ui/feedback/alert/index.ts
- src/ui/feedback/index.css
- src/ui/feedback/index.ts
- src/ui/feedback/inline-message/index.ts
- src/ui/feedback/inline-message/inlineMessageView.ts
- src/ui/feedback/operation-state/index.ts
- src/ui/feedback/operation-state/operationState.css
- src/ui/feedback/operation-state/operationStateView.ts
- src/ui/overlays/__tests__/overlaysBrowserFixture.ts
- src/ui/overlays/command-menu/commandMenu.css
- src/ui/overlays/command-menu/commandMenuController.ts
- src/ui/overlays/command-menu/commandMenuView.ts
- src/ui/overlays/command-menu/index.ts
- src/ui/overlays/confirmation-dialog/index.ts
- src/ui/overlays/dialog/dialog.css
- src/ui/overlays/dialog/dialogView.ts
- src/ui/overlays/drawer/drawer.css
- src/ui/overlays/drawer/drawerView.ts
- src/ui/overlays/drawer/index.ts
- src/ui/overlays/index.css
- src/ui/overlays/index.ts
- src/ui/overlays/popover/index.ts
- src/ui/overlays/popover/popover.css
- src/ui/overlays/popover/popoverController.ts
- src/ui/overlays/popover/popoverView.ts
- src/ui/tokens/radius/overlayRadiusTokens.css
- src/ui/tokens/shadow/overlayShadowTokens.css

### 修改（33）

- README.md
- architecture/state-ownership.json
- contracts/global-visual-system-contract.json
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-13-data-display.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
- scripts/forms/verifyForms.mjs
- scripts/verify-database-reset.mjs
- scripts/verify-global-visual-system.mjs
- scripts/verify-history-scoreline-ui.mjs
- scripts/verify-inline-workspace-and-resource-scroll.mjs
- scripts/verify-stage2-responsive-catalog-ui.mjs
- scripts/verify-warning-center.mjs
- src/app/shell/topbar/topbarController.ts
- src/app/shell/topbar/topbarView.ts
- src/app/warnings/__tests__/warningBrowserFixture.ts
- src/app/warnings/warning-center/warningCenter.css
- src/app/warnings/warning-center/warningCenterView.ts
- src/main.ts
- src/ui/feedback/blocking-message/blockingMessage.css
- src/ui/feedback/blocking-message/blockingMessageView.ts
- src/ui/feedback/feedback-content/feedbackContent.css
- src/ui/feedback/toast/toast.css
- src/ui/feedback/toast/toastView.ts
- src/ui/overlays/confirmation-dialog/confirmationDialog.css
- src/ui/overlays/confirmation-dialog/confirmationDialogController.ts
- src/ui/overlays/confirmation-dialog/confirmationDialogState.ts
- src/ui/overlays/confirmation-dialog/confirmationDialogView.ts
- src/ui/overlays/dialog/dialogController.ts
- src/ui/overlays/dialog/index.ts
- src/ui/primitives/controlAttributes.ts
- src/ui/tokens/index.css

### 删除（1）

- src/app/modal.ts


## 最终冻结收尾

完整92步前端门禁exit0：O:/Project/FOODBALL/Temp/r12-14-frontend-frozen.log。该次实际342项浮层检查和48张截图位于O:/Project/FOODBALL/logs/r12-14-overlays-1793195596412/，与独立审查使用的截图同源；documenter独立核验360 Light Popover截图SHA-256一致。静态检查、构建、自动测试与实际startup结果分别如上，MSI/NSIS、精确提交CI与Full仍待验。保留原有Vite大于500kB分块提示，没有调高阈值或关闭检查。


## 2026-10-29 CI失败订正与修复

3e335ec2的run40734621493在Database Service架构检查失败，后续Windows验收/打包未执行。上一轮92步通过之后修改了机器状态契约，故“最终冻结全量通过”不能作为该提交完整通过的证据。修复将数据库owner的R3-02迁移来源与last_updated_task分开，保留原保护并补正反例；详[CI修复记录](R12-14-ci-fix-state-provenance.md)。14维持VERIFYING，15未开始，不轮询新CI。

CI修复已完成完整架构及93步前端重验（342项浮层检查），相关可执行/契约指纹在验收期间保持不变；机器契约修改后没有沿用此前验证结论。修复提交自身Windows CI/MSI/NSIS仍待验。
