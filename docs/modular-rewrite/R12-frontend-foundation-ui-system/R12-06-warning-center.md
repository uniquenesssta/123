# R12-06 Warning Center

状态：`DONE`。06已按自身精确Windows验收完成收尾；详下。实施基线c880d04952285e2ac05282a82b50d1a3f1642e9e；沿唯一rewrite/r12-frontend-foundation-ui-system。07按本次用户授权开始，R12整体IN_PROGRESS。

## Windows验收收尾（2026-10-25）

06精确提交`793eea904a82072fb828f3023ccff8ac705d99ec`的[Windows run40127756204](https://github.com/uniquenesssta/123/actions/runs/40127756204)、job120972736636、attempt1全部SUCCESS。已一次性核对提交/分支、全部步骤、完整job日志及下载产物；artifact12574033620未过期，ZIP15,880,420字节，SHA-256 `8a22e453ec79d06cd95ef27315624b3ddf4f9243555fe027fafd8ddb9bdbd994`与API摘要一致。前端/136告警/原912 Shell/17截图、Rust fmt/Clippy/workspace tests、release/MSI/NSIS、实际客户端启动与日志通过。报告`windows-acceptance-20261024-200046.json`为startup PASS：7条有效记录、0无效行、0运行错误；会话`45a742de-0f91-4150-a892-46e57c4f22fb`内bootstrap、read_workspace_state、save_workspace_state三组trace均开始/完成配对。原ZIP在仓库上一级`R12-06-windows-evidence.zip`。startup不是Windows Full或真实PG最终环境验收；原待验继续保留。

## 设计与实施范围

先核对总纲8.0、R12-06任务书、当前源码与ui-design-system精确40109ceec8372391a7680e7e86c1265fda06ab3d。沿TypeScript/DOM/CSS，不新增生产依赖，不改具体业务Feature流程或后端。原代码没有独立全局告警中心；连接失败大提示分散于复用连接表单，通知单DOM/旧timer和全局busy在main。业务readiness、字段错误和导入阻断仍属于各Feature，不能用已读/关闭解除。

已读取Figma Toast 302:4210、Task Activity305:4208、Inline Alert303:4193及Dialog Standard/Wide279:2445的高保真上下文、截图和变量。fileKey PN0Whgu6HLWIHx4Mv6aHfu；本次session001d667b-0523-4cab-8097-e827320b7200。现有Patterns缺少日期/未读中心组合，故先在Patterns补齐可编辑组合`897:19519`，再回读高保真上下文/截图后实现。Light未读898:19520、Dark未读898:19687、Light空898:19640、Dark空898:19777、全局入口898:19823。此为本轮新增的受限组合稿，不冒充旧accepted状态，也没有新增业务Screen/S18、修改母版、变量集合或用户已有画稿。

组合复用Dialog Header277:2212、Inline Warning303:4193/Danger303:4226、Button Secondary52:3、Button/Icon123:210、Badge66:196和原图标。采用640宽、54默认header/46最小、9/11/8垂直padding、12水平padding、12分组间距、9外圆角、32动作；Light/Dark使用真实FIP Color与既有4.5%/26%对话框阴影。通知380上限、8×10 padding、5圆角，任务140最小宽、10×13 padding、9间距；右下20、彼此8间隔。成功色#168b69/#58cfa6，feedback surface及Subtle阴影均通过真实变量alias/effect回读。新增check.svg为Figma精确16×16资源，其余Info/Alert/Close/Spinner复用05唯一资源，不复制路径或引用临时URL。

## 职责与真实接入

- warning-dto定义会话告警、严重度、来源、时间及导航目标；在进入展示边界时限制长度、隐藏连接地址/凭据/Bearer/API key并去除堆栈，View再HTML转义。不是新的后端DTO或IssueLog Schema。
- WarningStore唯一拥有最多200条会话记录、首次/最近发生时间、次数、已读/恢复状态、单个通知及全局任务门禁。重现事件合并并重新未读；重复渲染观测不产生新事件。日期按本机日历分组，跨日/DST不使用固定24小时差判断。关闭通知不清空告警记录；全部已读不解除问题。
- WarningCenterState只拥有弹窗显隐。Controller组合Store、独立反馈宿主和弹窗，通过明确navigate adapter去既有问题日志页面；没有I/O/轮询、业务规则或全局数据库读取。main仅保留原toast/runBusy语义入口的委托，已删除busy状态和旧Toast timer/DOM访问。
- 顶栏仅提供data-warning-entry挂载槽；告警模块自己投影徽标。通知/任务宿主位于应用root外，普通重绘不会替换通知、重启timer或清除当前弹窗。Bootstrap连接错误由唯一Session摘要观测；恢复后显式标已恢复。连接表单仍原地可用，只保留紧凑关联说明，详细原因统一进入中心。
- 原toast调用没有携带请求来源，迁移后统一标“客户端”，不根据请求完成时的当前路由猜测原业务归属；有来源的结构化告警显式传source。严重全局异常进入中心，现有recordIssue日志路径仍保留。
- ToastController仅拥有剩余展示时间、hover/focus暂停及可释放timer。Info/Success无动作3200ms；Warning/Danger/带动作持续显示；相同消息合并不重建/重复播报，替换/关闭/销毁清旧timer且校验id。关闭只消提示，不取消任务、不回放写入、不清输入、不解除readiness。非模态通知不监听全局Esc，不抢焦点。
- runTask保持旧“已有任务正在执行”互斥语义，真实操作仍由原Feature执行；任务提示非模态，并同步hidden与aria-hidden。销毁后晚到任务结果不能重建DOM；不伪造后端取消。
- 原生Dialog的模态、Tab边界、Esc和焦点恢复提到ui/overlays/dialog唯一机制，05的Confirmation继续拥有自己的语义State并复用此机制；原详情历史控制器未改。告警列表独立滚动，Header/Actions留在视口；关闭回到触发器，标已读后焦点不落在Disabled控件。关闭/已读不写工作区快照，不改变输入、选中、滚动、主题或业务数据。
- 状态清单25→28：WarningStore、WarningCenterState、ToastController局部计时状态。最小Feedback Content/Toast/Task Activity/Dialog落在后续14等规划唯一目录；没有第二套临时设计系统，后续节点仍BLOCKED。所有依赖出口显式导出。

## 切换与兼容

旧main busy/Toast定时器、Shell的#toast/#busy及busy渲染参数、旧.toast/.task-activity/.busy样式已移除。没有整文件删除或移动。视觉契约1.2→1.3仅更新实际反馈selector/样式owner映射并验证真实main导入；未放宽密度、截图阈值或替换PNG基线。公共171条命令、Rust、DTO、数据库/迁移、日志等级/IssueLog格式、工作区schema_version=1/ui_revision=5、主题键、依赖/锁文件及18项保护资产不变。原生Dialog支持提取属于本任务共享边界所需，05接口/行为保持。

## 验证与证据

Windows本机，Node24.16.0/npm11.13.0、TypeScript5.8.3/Vite7.3.6、Rust1.88.0，浏览器deviceScaleFactor=1。受控adapter用于可复现任务/导航/异常；不是实际PostgreSQL、API或全部Tauri业务端到端。

| 检查 | 结果 |
|---|---|
| npm run verify:frontend | PASS：89个验证脚本＋TypeScript＋Vite，91步骤；最终日志仓库上一级r12-06-frontend-final.log。 |
| verify-warning-center.mjs | Store的身份/重复/恢复/日历/脱敏/只读快照/200条边界检查；实际生产模块4组960×600/1440×900×Light/Dark，37+33+33+33=136交互检查。使用真实3200ms timer、原生Enter/Tab/Shift+Tab/Escape/鼠标悬停/滚轮，验证独立滚动、焦点、门禁、8px placement和销毁后晚到结果。 |
| 原Shell/详情/Session/Router | 912项Shell检查、24生命周期、18会话、21路由行为继续通过，含共享Dialog提取后的取消/确认/Tab/Esc/焦点恢复。 |
| 原截图回归 | 17视口按原阈值PASS，PNG基线未更新。 |
| npm run verify:architecture | PASS：18 Feature/11 Rust crate/28唯一owner；日志r12-06-architecture.log。 |
| cargo fmt --all -- --check | PASS；未更改Rust。 |
| command/protected assets | PASS：171命令；18文件聚合SHA-256 d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57。 |
| git diff --check/差异审阅 | PASS；仅本任务及05收尾记录，无数据库/依赖/业务规则修改。 |

完整回归内warning截图/JSON沿原../logs/**上传入口复制，无新workflow/runner。最终证据为`../logs/r12-06-warning-1792871021284`及`C:\Users\uniqu\AppData\Local\Temp\football-warning-center-0qMcu2`，由r12-06-frontend-final.log定位；前一次完整通过证据位于../logs/r12-06-warning-1792870545978与Temp/football-warning-center-Anqr5Y。同轮旧Shell证据继续进入r12-05-shell目录。产物均保留，没有批量删除。图标局部文件/调用点/16px有效几何、深浅表面、长文案和列表滚动已人工核对。

最终41条非Markdown差异的规范化SHA-256为`4bbce00571da0ee679f48145f65145278e36be6ddd4226a2899c63df324c3327`，验证完成及暂存后再次比对一致。状态契约的历史`last_updated_task=R3-02`为既有Database Service门禁兼容标记，保持原值；06新增的三个owner及28条总数独立验证。曾尝试更新此历史标记的失败日志保留为仓库上一级r12-06-frontend-marker-failure.log，恢复原值后最终91步骤全量通过。

初次完整回归在旧反馈selector的静态门禁失败，已修正唯一新路径映射；没有降低门槛。早期浏览器夹具补齐初始异常回执、聚焦时preventScroll、既有原生按键参数和Page.bringToFront；失败日志保留，未计为通过。视觉回读发现默认预留滚动沟槽使列表右侧空隙偏大，已移除该局部预留并完整复验。Vite既有主chunk>500kB警告仍保留，不在06夹带Feature拆分。

## 尚待验收与回退

06自身Rust Clippy/workspace tests、release/MSI/NSIS、实际客户端启动及运行日志配对已由上述精确Windows运行核对通过；没有继承05结果。真实PG/历史保护四项/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归等最终封包新库待验继续保留；ignored、mock、公开stub与startup不算这些PASS。当前仅会话告警历史，不虚构跨重启持久化；持久问题仍由既有IssueLog负责。完整组件矩阵、异步危险业务弹窗和全部A11y属于后续节点。

回退点c880d04，采用Git受控回退06提交，无数据库或持久格式迁移。06完整Windows证据已核对；07另按本次用户明确授权执行。

## 实际文件清单

相对c880d04，共47路径：22新增、25修改、0删除。移动/重命名无。含05收尾记录与06节点文档；清单由真实git差异与未跟踪清单生成。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-06-warning-center.md`
- `scripts/verify-warning-center.mjs`
- `src/app/warnings/__tests__/warningBrowserFixture.ts`
- `src/app/warnings/index.ts`
- `src/app/warnings/warning-center/warningCenter.css`
- `src/app/warnings/warning-center/warningCenterController.ts`
- `src/app/warnings/warning-center/warningCenterState.ts`
- `src/app/warnings/warning-center/warningCenterView.ts`
- `src/app/warnings/warning-dto/index.ts`
- `src/app/warnings/warning-dto/warningDto.ts`
- `src/app/warnings/warning-store/index.ts`
- `src/app/warnings/warning-store/warningStore.ts`
- `src/assets/icons/check.svg`
- `src/ui/feedback/feedback-content/feedbackContent.css`
- `src/ui/feedback/feedback-content/feedbackContentView.ts`
- `src/ui/feedback/task-activity/taskActivity.css`
- `src/ui/feedback/task-activity/taskActivityView.ts`
- `src/ui/feedback/toast/toast.css`
- `src/ui/feedback/toast/toastController.ts`
- `src/ui/feedback/toast/toastView.ts`
- `src/ui/overlays/dialog/dialogController.ts`
- `src/ui/overlays/dialog/index.ts`

### 修改

- `architecture/state-ownership.json`
- `contracts/global-visual-system-contract.json`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-05-application-shell.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `docs/TESTING.md`
- `README.md`
- `scripts/verify-frontend.mjs`
- `scripts/verify-global-visual-system.mjs`
- `scripts/verify-workspace-ui.mjs`
- `src/app/shell/__tests__/shellBrowserFixture.ts`
- `src/app/shell/application-shell/applicationShellState.ts`
- `src/app/shell/application-shell/applicationShellView.ts`
- `src/app/shell/topbar/topbarView.ts`
- `src/components/databaseSetup.ts`
- `src/main.ts`
- `src/styles/app.css`
- `src/styles/layout.css`
- `src/styles/visualSystem.css`
- `src/ui/icons/iconSlot.css`
- `src/ui/icons/iconSlot.ts`
- `src/ui/overlays/confirmation-dialog/confirmationDialogController.ts`
- `src/ui/primitives/button/button.css`
- `src/ui/primitives/icon-button/iconButton.css`
- `src/ui/tokens/colors.css`

### 删除

无。
