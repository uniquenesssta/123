# R12-07 Workspace

状态：`VERIFYING`。06精确793eea9的Windows run40127756204、全部步骤/完整日志、产物摘要及startup报告已核对通过，详[06收尾](R12-06-warning-center.md)。2026-10-25用户在确认07可开始后明确要求“开始”。本节点基线`793eea904a82072fb828f3023ccff8ac705d99ec`，复用唯一`rewrite/r12-frontend-foundation-ui-system`；08～19 BLOCKED，R12整体IN_PROGRESS。07自身Windows CI待提交后独立验收，不轮询。

## 设计与影响范围

先核对总纲8.0、R12-07任务书、R00命令/保护资产与当前调用方清单；设计分支最新仍为`40109ceec8372391a7680e7e86c1265fda06ab3d`，采用其`docs/FIGMA_PC_APP_SHELL.md`的既有accepted合同。Figma文件`PN0Whgu6HLWIHx4Mv6aHfu`，本节点session `84af80b5-9933-4faf-96dd-e9e5ca0d0b2f`。实施前读取PC App Shell `380:1366`的高保真设计上下文及截图，含Workspace Slot `379:1087`、Workspace History `375:217`与Topbar `378:1109`的组件说明。

复用05已接入的60/154/44外壳、32操作、原Icon Slot与25个本地SVG、06告警入口和既有主题token；本节点没有新增视觉、资源、CSS、Figma画板或变量，没有用QA占位文案替换业务页面。工作区历史仍属于原ModalController和RouteHistory；全局导航清详情栈、主题/折叠保留页面、局部重置不改变业务数据、加载只锁定工作区等合同保持。实际恢复需验证真实DOM，Figma预置历史不算持久化或运行时证据。

## 唯一职责与切换

原`src/app/viewState.ts`同时承载版本迁移、状态、DOM读取/恢复、异步保存及生命周期；07收敛到唯一`src/app/workspace/`：

| 文件 | 唯一职责 |
|---|---|
| workspaceState.ts | 唯一文档及局部变更版本所有者；返回深复制快照，记录读取/清空期间的新编辑 |
| workspaceVersion.ts | schema/UI版本检查、旧UI迁移及不可信字段规范化；未来版本保留文件并报告 |
| workspacePersistence.ts | 唯一adapter、250ms延迟保存和串行I/O队列；清空epoch淘汰旧排队写入 |
| workspaceRecovery.ts | 页面内容区的安全控件、详情、滚动、焦点快照；可取消恢复帧和100ms输入活动合并 |
| workspaceCoordinator.ts | 编排上述模块，具名初始化/停止/销毁与失败回调，不实现业务计算或页面渲染 |
| index.ts | 显式导出公开Coordinator及类型，内部State不作为第二外部写入口 |

主入口、共享工作区组件、球队/球员类型调用及原Shell夹具均改走公开出口；删除单个旧文件`src/app/viewState.ts`，无兼容影子实现或临时转发。`browser.workspace`所有权登记迁到WorkspaceState，仍28个owner；历史`last_updated_task=R3-02`兼容标记保持。验证器跟随实际owner，保留原断言和阈值。

## 行为与失败路径

- 初始化去重，停止后的迟到读取/失败不能恢复、清除、迁移或保存状态。读取失败不再调用clear，不以空默认值覆盖未能读取的文件；使用临时会话状态并通过既有Warning Center/IssueLog报告，明确修复后重启。显式重新初始化成功时保留本次已经编辑的模块。
- 保存按提交次序串行，深复制输入，旧写入不能在新写入后完成。保存失败不丢弃dirty状态，下次保存重试；后台失败进入既有告警，未吞掉失败或冒充写入成功。
- 清空与正在进行的写入串行；排队旧快照失效，清空期间的新编辑不会被迟到响应覆盖，失败保留原状态。数据库彻底清空后的工作区处理也走此唯一通道，然后停止工作区，避免退出捕获重建已删除对象的引用。数据库危险确认、后端重建和命令语义未改。
- 实际input/change/scroll/toggle/focusout触发合并捕获和保存，无周期轮询；start重复调用不会重复监听。退出前捕获移到同步stop之前，destroy最终flush去重；停止后解除监听/计时/恢复帧并拒绝迟到修改。强制结束进程仍不能保证尚未完成的异步IPC写入，未宣称无条件零丢失。
- 恢复只操作页面内容根，保留独立滚动、选中、最多6页签、活动分区和展开状态。控件排除password/file、敏感键、disabled及`data-workspace-persist=false`子树；其焦点同样排除。过期帧、分离DOM、加载/错误快照不能覆盖有效状态，延迟恢复不能抢走Shell/弹窗焦点，局部清除取消待恢复帧。
- 原单选/复选快照继续兼容；多选控件新增可选`controls[id].selected_values: string[]`，恢复时只选择当前仍存在的选项。schema_version仍1、ui_revision仍5，旧字段不改名；此为兼容扩展，旧客户端忽略该可选字段，旧快照继续用value恢复。没有业务DTO、Tauri公共命令或数据库迁移。

## 实际验证

本机Windows，Node24.16.0/npm11.13.0、TypeScript5.8.3/Vite7.3.6、Rust1.88.0；浏览器deviceScaleFactor=1。生产模块、真实Chromium表单与布局执行；受控内存adapter用于可复现磁盘失败/竞态，不替代真实PostgreSQL、强制断电或所有Tauri业务端到端。

| 入口 | 结果 |
|---|---|
| 原verify-workspace-ui.mjs | PASS：16项生产State/Version/Persistence/Coordinator行为，覆盖畸形字段/原型键、旧UI/未来版本、深复制、初始化去重、读取失败保护/重试、迟到结果、迁移写失败、重叠保存、清空竞态/失败、局部重置、销毁与数据库清空退出保护 |
| 原Shell浏览器入口 | PASS：12个960×600/1440×900/2560×1440 × Light/Dark × 展开/折叠组合，每组新增16项Workspace检查，保留原76项，共92×12=1,104项；真实原生输入/轮滚/键盘与详情回归仍执行 |
| npm run verify:frontend | PASS：89个原验证脚本＋TypeScript＋Vite，共91步骤；最终日志仓库上一级r12-07-frontend-final.log |
| npm run verify:architecture | PASS：18 Feature、11 Rust crate、28唯一owner，r12-07-architecture.log |
| cargo fmt --all -- --check | PASS，Rust1.88.0 |
| 原24生命周期/18会话/21路由 | PASS，调用唯一新Workspace模块，不复制实现 |
| 原17截图与136告警 | PASS：最终完整前端内17个原视口与136项告警交互通过，PNG基线和阈值未变 |

初轮新增浏览器结构检查为每组15项/总91项；随后补敏感控件焦点排除，完整回归为16项/总92项。首轮完整前端PASS后复核数据库重置退出回写链，补同一原因的串行清空/停止和第16项并发单测，再复验最终完整门禁。早期Node生命周期测试因新DOM边界需HTMLElement替身而失败；已补其测试环境，原24断言保持后通过。全部失败/运行日志保留；本地预载保护使临时目录不批量清理。

实际Shell截图已对照Figma明暗/展开折叠尺寸，并人工核对960深色与1440浅色；其中内容为验证草稿的测试夹具，不是新业务页面。最终Shell/Workspace截图、结构化检查保留于`../logs/r12-05-shell-1792876677270`及`C:\Users\uniqu\AppData\Local\Temp\football-detail-history-poemnp`；沿原日志artifact上传，不另建runner。最终21条非Markdown差异（含旧文件删除标记）的规范化SHA-256为`cc2f9cf6ed191f54a6ade83c7bf298bf7d82be676b4074567b43a283eaa06485`，完成验证后再次核对。171命令与18项保护资产保持，聚合SHA-256仍为`d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`。

## 剩余验收与回退

07自身Rust Clippy/workspace tests、release/MSI/NSIS、实际客户端启动与日志配对交由原Windows CI独立验收；未用06结果替代。07保持VERIFYING，未开始08。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承删除风险继续按最终封包新库安排待验，公开stub、ignored、内存adapter及startup不能算这些PASS。Vite既有主chunk超过500kB的警告保持。

回退点为793eea9，使用受控Git回退本节点；没有数据库或生产依赖变更。持久化可选多选字段与旧读者兼容，不通过复制旧文件恢复双入口。

## 实际文件清单

相对793eea9共27个路径：9新增、17修改、1删除；无移动/重命名。旧文件逐个明确删除，没有批量清理。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-07-workspace.md`
- `scripts/workspace/verifyWorkspaceBehavior.mjs`
- `src/app/workspace/__tests__/workspaceBrowserFixture.ts`
- `src/app/workspace/index.ts`
- `src/app/workspace/workspaceCoordinator.ts`
- `src/app/workspace/workspacePersistence.ts`
- `src/app/workspace/workspaceRecovery.ts`
- `src/app/workspace/workspaceState.ts`
- `src/app/workspace/workspaceVersion.ts`

### 修改

- `README.md`
- `architecture/state-ownership.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-06-warning-center.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `scripts/verify-browser-bootstrap.mjs`
- `scripts/verify-database-reset.mjs`
- `scripts/verify-dual-level-navigation.mjs`
- `scripts/verify-lineup-scroll-continuity.mjs`
- `scripts/verify-search-query-state.mjs`
- `scripts/verify-workspace-ui.mjs`
- `src/app/shell/__tests__/shellBrowserFixture.ts`
- `src/components/workspace.ts`
- `src/main.ts`
- `src/pages/players.ts`
- `src/pages/teams.ts`

### 删除

- `src/app/viewState.ts`
