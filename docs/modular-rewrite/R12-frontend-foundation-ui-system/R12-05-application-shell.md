# R12-05 Application Shell

状态：`DONE`。05已于2026-10-25按精确CI、日志及下载产物收尾。用户此前明确确认“04已通过，没问题开始05”；05实现基线为`75e3981d544498a0a76e17b85e472860c293e431`，沿唯一`rewrite/r12-frontend-foundation-ui-system`。当前05已完成，06另按用户授权实施，R12整体IN_PROGRESS。

## Windows验收收尾（2026-10-25）

精确提交`c880d04952285e2ac05282a82b50d1a3f1642e9e`的[run40121131372](https://github.com/uniquenesssta/123/actions/runs/40121131372)、job120952691419均SUCCESS，attempt=1。2026-10-25收尾时一次性核对head/分支、全部步骤、完整job日志及下载产物，没有轮询。artifact12571986845未过期，15,433,702字节ZIP的SHA-256为`976e5adc0155d48910cab48f38e3b29e235746b8e74183c4652590276d581505`，与GitHub摘要一致。前端/17视口、Rust检查/测试、release/MSI/NSIS、应用启动及日志检查通过。startup报告PASS，7条有效运行记录、0错误/无效行；会话0fc2c04c-a2da-44b0-a0f2-8032bd3430e8内3组trace开始/完成配对，对应bootstrap、read_workspace_state、save_workspace_state。此报告仅为startup profile，真实PG/XLSX/Full/私有固定回归等最终环境待验继续保留。下载ZIP位于仓库上一级R12-05-windows-evidence.zip。

用户授权“05ci通过，开始06”，06沿当前R12分支实施；07～19仍BLOCKED。

## 依据与真实设计映射

依据R12任务书、总纲8.0节及设计分支`ui-design-system`的`40109ceec8372391a7680e7e86c1265fda06ab3d`已确认记录，特别是`docs/FIGMA_PC_APP_SHELL.md`、`FIGMA_DIALOG.md`及`FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md`。Figma文件`PN0Whgu6HLWIHx4Mv6aHfu`，本次读取session `0d3892ba-b787-4617-b25f-e39667b3a597`。先读取真实设计上下文及截图，再实现代码；没有写入或修改Figma文件，也没有把原型预置动作当成运行时验收。

| 已确认节点 | 本节点接入 |
|---|---|
| PC App Shell `380:1366`；Primary Rail `376:855`；Secondary Navigation `377:1511` | 固定60一级栏、可隐藏154二级栏、最小960×600桌面布局；7模块/17路由复用03注册表 |
| Primary/Secondary Item `374:183` / `374:321`；Brand `375:221` | 52×48一级按钮、138×40二级按钮；FIP文字标记32；选中/悬停/按下/禁用/焦点保持几何 |
| Topbar `378:1109`；Icon Action `373:79`；Workspace History `375:217` | 顶栏44、操作32、16/20图标；面包屑单行省略；工作区详情历史与全局导航分离 |
| Service Status `374:340`；Workspace Slot `379:1087` | 数据库配置存在/缺失/载入状态；真实Feature内容替换工作区插槽，未把设计说明样例复制成产品页面 |
| Blocking Message `304:4348`，Error `304:4246` | 当前页面读取失败的阻断展示；重新读取与问题日志绑定明确目标，不重试业务写入 |
| Dialog/Confirmation `280:2814`，Danger/Default `280:2594` | 480确认弹窗、46最小头部、9/11垂直内边距、Cancel默认焦点、关闭/取消/Esc及局部重置 |
| Icon Slot `37:44`；Button/Icon `123:209`；Button/Primary、Secondary、Danger | 唯一受控资源目录与最小按钮依赖；25份Figma SVG本地打包，未手绘或替换成近似图标 |

FIP Color Light/Dark、Size及Control Scale直接读取并解析真实变量别名：采用60/154/44、32操作、7圆角、4/8/12/16间距、0.42禁用透明度、1px内侧焦点线。弹窗遮罩Light `rgb(25 39 47 / 46%)` / Dark `rgb(2 7 10 / 82%)`，阴影分别0/10/24与0/12/28；深浅主题的文字、描边、强调、危险色均来自本次解析值。沿用原Inter/系统字体回退，不安装字体或依赖。

## 职责、调用流与状态归属

- `application-shell`组合视图、管理一次性委托监听器、解释Shell动作并通过具名adapter协调现有owner。入口仅组装Router/Session/Workspace/详情回调和原日志追踪；旧Shell动作switch已迁出main，其他Feature动作保持原时序。重复start不重复绑定，destroy解绑且不能重新启动。
- `sidebar`渲染双层菜单并解释模块/路由入口；具体Figma路由图标映射属于Shell，通用Icon Slot不依赖Page或Router。只折叠二级栏，一级栏和主题入口持续可用；折叠与恢复转移焦点到可见按钮。
- `topbar`渲染位置、配置状态、历史和重置入口。数据库字段只证明“数据库已配置”，不再宣称“已连接”或“服务正常”；未提供版本时显示“版本未提供”，不虚构版本号。
- `page-frame`承载真实内容区并保存/恢复Shell控件焦点；`content-region`提供独立滚动、工作区inert/aria-busy、紧凑载入进度及读取失败展示。移除旧多层全局Shell样式覆盖，五个目录各自拥有CSS，消除旧布局缩小按钮、顶栏和字号的断点。
- Router仍唯一拥有页面及请求代次，Session仍唯一拥有主题/启动摘要，WorkspaceStateStore仍唯一拥有旧工作区快照，ModalController/RouteHistory仍拥有右侧详情草稿与历史。新Shell不复制上述业务或持久状态。
- 新增两个短期UI owner：ContentRegionState只保存读取失败展示标志；ConfirmationDialogState只保存尚未处理的局部UI确认。状态清单23→25；确认弹窗不进入详情历史、不写入工作区持久数据。
- 保留原主题键、工作区schema_version=1/ui_revision=5。工作区新增对根滚动区域的捕获，复用原internal_scrolls映射；载入/错误根用data-workspace-capture=suspended暂停写快照，避免空错误面板或尚未恢复的加载页覆盖上次可用草稿。恢复仍由原Workspace owner执行。
- 重置先通过已确认的Figma弹窗确认。取消/关闭/Esc保留原详情、输入及滚动；确认仅清除当前工作区快照和原有当前页选择，先清详情历史并将renderedPage置空，阻止旧DOM再次回写已清空的快照。数据库内容、其他页快照、主题和全局折叠选择保持。
- 确认使用原生showModal使同文档其他节点不可交互，Cancel默认焦点；显式处理首尾Tab并拦截Escape传播，再在cancel捕获阶段关闭和恢复焦点。MDN已通过Context7核对showModal与不冒泡的cancel事件：[showModal](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/showModal)、[cancel](https://developer.mozilla.org/en-US/docs/Web/API/HTMLDialogElement/cancel_event)。

## 受控切换与最小共享依赖

唯一公开入口为`src/app/shell/index.ts`，只显式导出。原`src/app/shell.ts`已逐个明确路径删除，没有转发旧实现；五个职责目录是Shell内部模块。原详情控制器未改动。父EventScope统一释放子控制器的委托监听器；无持久状态的sidebar/topbar/page-frame不创建虚假的State文件。

R12共同输入允许的最小依赖落在后续节点唯一规划目录：`src/ui/tokens`颜色/尺寸子集，`src/ui/primitives`图标及反馈动作32，`src/ui/feedback/blocking-message`的Error展示，`src/ui/overlays/confirmation-dialog`的同步本地重置确认，以及`src/ui/icons`/`src/assets`的Shell图标。其余按钮/字段/异步提交弹窗/完整A11y/样式边界仍由10～19在这些实现上继续完成，未另建临时设计系统，也未将这些节点提前标DONE。Feature原有通用图标与控件继续原路径，本次只切换Shell消费者。

## 实际文件清单

相对75e3981，git diff --no-renames --name-status：61A / 27M / 1D，共89路径；移动/重命名无。以下清单包含04文档收尾和05自身记录，全部为真实仓库差异。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-05-application-shell.md`
- `src/app/shell/__tests__/shellBrowserFixture.ts`
- `src/app/shell/application-shell/applicationShell.css`
- `src/app/shell/application-shell/applicationShellController.ts`
- `src/app/shell/application-shell/applicationShellState.ts`
- `src/app/shell/application-shell/applicationShellView.ts`
- `src/app/shell/content-region/contentRegion.css`
- `src/app/shell/content-region/contentRegionController.ts`
- `src/app/shell/content-region/contentRegionState.ts`
- `src/app/shell/content-region/contentRegionView.ts`
- `src/app/shell/index.ts`
- `src/app/shell/page-frame/pageFrame.css`
- `src/app/shell/page-frame/pageFrameController.ts`
- `src/app/shell/page-frame/pageFrameView.ts`
- `src/app/shell/sidebar/sidebar.css`
- `src/app/shell/sidebar/sidebarController.ts`
- `src/app/shell/sidebar/sidebarIcons.ts`
- `src/app/shell/sidebar/sidebarView.ts`
- `src/app/shell/topbar/serviceStatusView.ts`
- `src/app/shell/topbar/topbar.css`
- `src/app/shell/topbar/topbarController.ts`
- `src/app/shell/topbar/topbarView.ts`
- `src/assets/icons/alert-circle.svg`
- `src/assets/icons/arrow-left.svg`
- `src/assets/icons/arrow-right.svg`
- `src/assets/icons/chart.svg`
- `src/assets/icons/chat.svg`
- `src/assets/icons/close.svg`
- `src/assets/icons/contrast.svg`
- `src/assets/icons/dashboard.svg`
- `src/assets/icons/database.svg`
- `src/assets/icons/history.svg`
- `src/assets/icons/info.svg`
- `src/assets/icons/issue-log.svg`
- `src/assets/icons/lineup-preset.svg`
- `src/assets/icons/match-center.svg`
- `src/assets/icons/panel-left.svg`
- `src/assets/icons/panel-right.svg`
- `src/assets/icons/plug.svg`
- `src/assets/icons/prediction.svg`
- `src/assets/icons/release-acceptance.svg`
- `src/assets/icons/reset.svg`
- `src/assets/icons/settings.svg`
- `src/assets/icons/sheet.svg`
- `src/assets/icons/shield.svg`
- `src/assets/icons/spinner.svg`
- `src/assets/icons/users.svg`
- `src/assets/svg.d.ts`
- `src/ui/feedback/blocking-message/blockingMessage.css`
- `src/ui/feedback/blocking-message/blockingMessageView.ts`
- `src/ui/icons/iconSlot.css`
- `src/ui/icons/iconSlot.ts`
- `src/ui/overlays/confirmation-dialog/confirmationDialog.css`
- `src/ui/overlays/confirmation-dialog/confirmationDialogController.ts`
- `src/ui/overlays/confirmation-dialog/confirmationDialogState.ts`
- `src/ui/overlays/confirmation-dialog/confirmationDialogView.ts`
- `src/ui/primitives/button/button.css`
- `src/ui/primitives/icon-button/iconButton.css`
- `src/ui/primitives/icon-button/iconButtonView.ts`
- `src/ui/tokens/colors.css`
- `src/ui/tokens/dimensions.css`

### 修改

- `README.md`
- `architecture/state-ownership.json`
- `contracts/global-visual-system-contract.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-04-session.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/task-ui-screenshot-tools.mjs`
- `scripts/verify-api-workspace.mjs`
- `scripts/verify-browser-bootstrap.mjs`
- `scripts/verify-dual-level-navigation.mjs`
- `scripts/verify-entity-resource-center.mjs`
- `scripts/verify-extended-workspace-hierarchy.mjs`
- `scripts/verify-global-visual-system.mjs`
- `scripts/verify-inline-workspace-and-resource-scroll.mjs`
- `scripts/verify-openai-profile-ui.mjs`
- `scripts/verify-stage1-balanced-ui.mjs`
- `scripts/verify-stage2-responsive-catalog-ui.mjs`
- `scripts/verify-team-package.mjs`
- `scripts/verify-team-player-management.mjs`
- `scripts/verify-workspace-ui.mjs`
- `src/app/viewState.ts`
- `src/main.ts`
- `src/styles/entityCenter.css`
- `src/styles/layout.css`
- `src/styles/visualSystem.css`
- `src/styles/workspacePanels.css`

### 删除

- `src/app/shell.ts`

## 契约与兼容性

公共171条Tauri命令、注册顺序、DTO字段、数据库/迁移/配置格式、日志等级、模型保护资产和生产依赖均未变。新增的data-action只在前端处理。历史`team-player-management-contract.json`原始内容与登记哈希保持，验证器将其中已迁移的Shell历史路径映射到实际新owner，未修改0020或其他迁移。全局视觉验证契约只更新Shell源码/样式位置映射，原密度、截图基线及阈值保持。

## 实际验证

本机Windows，Node24.16.0、npm11.13.0、TypeScript5.8.3、Vite7.3.6；项目Rust工具链1.88.0。浏览器使用既有Chromium解析与截图工具，真实进程执行原生文本输入、滚轮、Enter/Space/Tab/Escape。数据adapter受控以复现竞态和失败，不计为真实PostgreSQL/API或Tauri WebView全业务端到端验收。

| 入口 | 结果 |
|---|---|
| npm run verify:frontend | PASS：88个现有验证脚本＋TypeScript＋Vite，共90步骤。最终日志r12-05-frontend-verified.log。 |
| node scripts/verify-inline-workspace-and-resource-scroll.mjs | PASS：12组960×600/1440×900/2560×1440 × Light/Dark × 展开/折叠；每组76检查，共912。并保留03/04原详情与会话检查。 |
| node scripts/verify-browser-bootstrap.mjs | PASS：24项真实生命周期与18项会话行为，旧组合根边界保持。 |
| node scripts/verify-dual-level-navigation.mjs | PASS：7模块/17路由及21项真实路由/历史/页面加载行为。 |
| npm run verify:architecture | PASS：18个Feature、11个Rust crate、状态清单25个唯一owner及原边界/持久化/应用门禁。 |
| node scripts/verify-task-ui-screenshots.mjs | PASS：原17视口按原阈值通过；未更新PNG基线或比较阈值。存在阈值内像素差异，不宣称逐像素相同。 |
| cargo fmt --all -- --check | PASS，项目工具链1.88.0。 |
| node scripts/verify-command-contract.mjs | PASS：171命令完全一致。 |
| node scripts/verify-protected-assets-deterministic.mjs | PASS：18文件，聚合SHA-256 d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57。 |
| git diff --check / 源文件差异复核 | PASS；crates、src-tauri、依赖清单、Cargo.lock及历史球队契约无差异。 |

Shell检查覆盖几何/完整视口、选中与默认路由、禁用不激活、折叠恢复焦点、主题与输入/滚动保持、详情A/B/基础页、取消/确认局部重置、读取失败/重试/日志入口、加载/错误快照保护、重复start/destroy、原生输入/独立滚动/键盘和减少动画。确认弹窗还覆盖480尺寸、默认Cancel、原生Tab循环和Escape取消/焦点恢复；深浅主题及可见截图已人工核对。

产物：

- 最终Shell最小验证日志：仓库上一级`r12-05-shell-verified.log`。
- Shell HTML、12张外壳截图、12张重置弹窗截图、原详情截图和shell-results.json：`C:\Users\uniqu\AppData\Local\Temp\football-detail-history-qodWCZ`。
- 同批PNG/结构化结果已复制至`C:\Users\uniqu\Documents\ChatGPT\项目代码模块性优化 足球\logs\r12-05-shell-1792866435715`；CI沿相同../logs布局进入原日志artifact，未新增workflow或runner。
- 完整架构日志：仓库上一级`r12-05-architecture.log`；17视口日志`r12-05-screenshots.log`。其后25-owner及最终前端内架构门禁再验通过。

早期全量被旧Shell路径/旧布局片段断言拦截；现已跟随唯一生产模块及Figma布局，历史SQL契约保持原哈希。浏览器夹具还修正了隐藏按钮选择、绑定已销毁测试实例、原生按键参数及滚轮绘制等待；原生弹窗Tab边界已补齐。默认沙箱只读诊断中的浏览器启动出现unsettled top-level await，最终用允许的本机进程/端口权限复验，不算这些失败轮次通过。日志和临时产物保留，未通过降低门槛掩盖失败。

## 剩余环境验收与回退

05自身Rust Clippy、workspace tests、release/MSI/NSIS、实际应用启动及日志配对现已由上述精确Windows运行核实通过；未用04结果替代。后续节点独立验收。Vite仍提示既有主chunk超过500kB，不在本节点夹带拆Feature或放宽警告阈值。

真实PostgreSQL、历史保护四项、不可变账本并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险继续按原最终封包/新库流程待验；公开stub、静态指纹和模拟adapter不算这些真实PASS。完整表单控件、异步/危险业务弹窗、平台能力、全局Warning Center及完整无障碍仍属后续节点。

回退点为75e3981，使用Git受控回退本05提交；没有数据库或持久格式迁移，不通过复制旧文件维持双入口。根README、TESTING、任务书、04完成记录和R12索引已同步；没有创建阶段完成记录。
