# R12-15 Navigation 与 Layout

状态：DONE。本地实现、独立复审、完整门禁、精确提交CI/打包/startup验收完成。2026-10-29用户授权“已通过收尾14开始15”。基线5886a439cc301a7fa315c1a8adf7b3b33c0b1d87；同一rewrite/r12-frontend-foundation-ui-system分支，O:/Project/FOODBALL/123 r3。15完成后用户另行授权16；17～19未实施。

## 15自身最终验收

2026-10-30收尾：bd9172ad767fc7ad6c574d0d025875d3c26e6309 的 Windows run40896297611 / job123526193561 全步骤success；artifact12911330330，48,801,140 bytes；下载ZIP SHA-256 629dba382314413789e71e9d71d2ae25a92099d67409efe06e4ba557f064d5da与GitHub摘要一致。MSI17,969,152 bytes、NSIS15,606,527 bytes；startup pass，7条有效日志、0无效、0错误，bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务链路。证据 O:/Project/FOODBALL/Temp/r12-15-bd9172ad-evidence.zip；只核验完成结果，没有轮询。

## 14前置验收与回退点

5886a439的Windows run40863418461 / job123412066355全部步骤success；artifact12894043614，48,323,138 bytes，ZIP SHA-256 2c06b3c06ffd25530680029b45fd3efaa69a3afec7d3241c00904abe59da4a5e，下载后独立哈希一致。MSI17,969,152 bytes、NSIS15,605,981 bytes；startup profile pass，7条有效日志、0错误，bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务链路。14与其CI来源修复均DONE。2026-10-29只核验完成结果，没有轮询。

15为可回退原子提交；回退使用该提交的git revert，不手工复制或批量删除目录。Git工作区初始clean；无用户未提交重叠变更。

## 来源、Figma与职责

已核对R0文件/命令与保护资产记录、R12任务书、现有router/shell/workspace以及真实页面调用。Figma session e513978c-2c0d-4c26-a342-7bfcbd24f647；Patterns PC App Shell380:1366，Controls Tabs213:940/209:690、Segmented216:1036、Pagination241:1477。读取高保真上下文、截图及当前图层属性，沿用批准的中性色/紫色语义体系、Noto Sans SC字重、固定间距和独立滚动边界。

| 新唯一职责 | 真实入口与边界 |
|---|---|
| ui/navigation/sidebar、topbar | app/shell将17路由、7模块、配置事实和历史操作映射为展示输入；UI不读取route/session/database |
| section-navigation、keyboard | 13个业务页的现有section/anchor入口；原生方向键/Home/End只移动焦点，Enter/Space由原生button提交；竖排球队档案显式传vertical；EventScope随Shell销毁 |
| pagination | Teams/Players传真实page/has_more，不推算总页数；主链setPlayerPageLoading与refreshPlayerPageDom使用同一原地更新函数，保留节点/列表/筛选/滚动 |
| tabs、segmented-control | 32/40组件契约、单一焦点停靠、真实关闭委托/单选语义；null/缺失/禁用身份不编造选中数据。当前批准Teams/Players界面未挂载旧对象栏，未虚构compare/cards业务能力；后续Patterns组合不提前实施 |
| ui/layout | 应用外框、main槽、内容/详情根、pane toggle；只组合受信任HTML槽，不新增业务状态或I/O |

通用样式由ui目录唯一导入；6个旧样式路径迁移，旧混合components/workspace.ts由分职责模块替代，调用已全部切换；11份旧HTML验收夹具仅修正移动后的CSS引用。shell控制器仍负责业务回调、确认保护、请求失败、详情历史和草稿，WorkspaceCoordinator仍是持久化唯一owner。状态清单只登记Keyboard监听器生命周期，并将last_updated_task更新为R12-15；数据库迁移来源仍是R3-02。

## 布局与独立复审修正

- Teams目录唯一外框退出旧双/三列通用规则，明确占满剩余空间；筛选与结果各有固定空隙，分页组相对结果区居中。增加内容填满可用宽度断言，避免“溢出控件碰巧居中”掩盖40px结果区。
- Players明确单行browser和剩余高度分配，退出960px旧的空540px第二行；筛选组保持可读单列，组列表和表格独立滚动，页脚控件留在可见边界。增加表格高度、非空18行、组宽至少160px和页脚边界检查。
- 删除旧CSS伪元素重复加载文案；仅共享分页状态区显示“正在加载结果…”，覆盖前后两个方向。
- 忙碌开始前把焦点停放到带可访问名称的当前页标记，再原生禁用方向按钮；完成后保留当前位置，不从其他控件夺焦点。原生键盘与真实局部加载助手均验证此策略。
- 已选中Tabs/section焦点使用现有on-action颜色。Figma只修改206:654与208:682两项Focus变体的stroke绑定至既有VariableID:47:5，1px INSIDE；没有改全局变量或调色。原生focus-visible与计算颜色对比至少3:1有断言及截图元数据。
- Tabs当前画布明确是padding12px、13px Medium；这是实时节点对历史padding10令牌的局部覆盖，已登记[R12-15-design-mapping.json](R12-15-design-mapping.json)。仓库映射不冒充已发布Code Connect。

独立Impeccable finish reviewer最终SHIP仅覆盖本批UI修复范围，报告C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r12-15-review.md。CLI上下文引擎缺失/只读缓存，采用已披露的人工来源回退；没有编造CLI detector/build-phase通过。独立documenter另查既有体系一致性，不创建另一套DESIGN或覆盖历史快照。

## 公共接口、数据与保护范围

无生产依赖、IPC命令、DTO/Schema、数据库迁移/数据、模型规则或保护资产变化。新增接口均为前端视图/属性与DOM更新函数；异常用本地参数验证，不改变公共错误码/日志。未新增网络或持久化副作用；页面布局、原生焦点和ARIA按上述批准范围改进。组件360px只表示独立组件弹性；桌面Shell下限960×600、Tauri最小1080×700未变。所有示例数据仅在测试夹具，未进入生产数据链。

## 分项验证

| 类别 | 命令/证据 | 结果 |
|---|---|---|
| 静态类型 | node ../node_modules/typescript/bin/tsc --noEmit | PASS |
| 架构 | npm.cmd run verify:architecture | PASS；O:/Project/FOODBALL/Temp/r12-15-architecture.log |
| 新组件原生Windows浏览器 | 3宽度×2主题，方向键/Enter/Space/Delete、禁用/选中/忙碌、焦点/销毁、尺寸/对齐/边界/编码 | 348项PASS，r12-15-focus-settled.log |
| 真实页面回归 | Teams/Players/profile不同尺寸与主题；实际模板及实际setPlayerPageLoading源码，适配数据仅为夹具 | Teams与profile修复通过；最终Players8视图/168项PASS，18行正向保留；football-approved-design-LmJ7ZP |
| Rust格式 | cargo fmt --all -- --check | PASS；r12-15-rust-fmt.log |
| Rust静态分析 | cargo clippy --locked --offline --workspace --all-targets -- -D warnings | PASS；r12-15-rust-clippy.log |
| Rust自动测试 | cargo test --locked --offline --workspace | 748通过、0失败、39保留跳过；r12-15-rust-test.log |
| 完整前端+构建 | node scripts/verify-frontend.mjs | 最终93步exit0，包含TypeScript与Vite构建；r12-15-frontend-release-candidate.log。348项专项、1104项Shell与64视图/1480项实际模板检查均PASS，17份旧截图基线保留；前轮失败未作为PASS |
| 实际Windows应用 | 隔离标识com.footballmatchmodel.platform.r1215validation的Tauri开发程序+实际123 r3 Vite；启动后分析真实日志 | startup PASS，5条有效记录，bootstrap/read_workspace_state完成、0错误；Temp/r12-15-native-verified/startup-report.json |
| 本批Windows打包与CI | bd9172ad精确run40896297611/job123526193561及artifact12911330330 | 全步骤success；下载哈希、MSI/NSIS与startup已核验，见本记录最终验收；不轮询 |

实际startup不是原生窗口逐页交互验收，也不是Full数据库/模型业务链路；真实PostgreSQL与私有模型回归39项仍未验证。既有Vite分块提示保留，不调高阈值、不关闭检查。完整门禁与构建已经完成，81项最终非Markdown来源哈希在完整验收开始至结束保持不变。2026-10-29发布时曾保持VERIFYING；2026-10-30精确CI验收通过后收尾为DONE。

所有开发/验证脚本先加载O:/Project/FOODBALL/project-temp.ps1，进程级TEMP/TMP指向O:/Project/FOODBALL/Temp。旧脚本递归清理通过retain-test-artifacts.cjs替换为保留产物，仅改变清理行为，不改变断言；没有递归删除。临时Vite和隔离验证进程只停止本任务创建的实例。

## 实际文件清单

下列清单来自git diff --cached --name-status --find-renames，共89个变更条目。5项CSS为R100纯移动；旧foundation/workspaceNavigation.css先逐项移动再扩展，低于Git相似度阈值，因此被表示为一项旧路径退出与新sectionNavigation.css，而非丢弃原13行样式。唯一物理删除文件为已替代src/components/workspace.ts；没有批量删除、目录删除或隐藏必要功能。

### 新增（29项）

- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-15-design-mapping.json
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-15-navigation-and-layout.md
- scripts/navigation/README.md
- scripts/navigation/verifyNavigationLayout.mjs
- src/pages/players/playerWorkspace.css
- src/ui/layout/application-frame/applicationFrameView.ts
- src/ui/layout/content-region/contentRegionView.ts
- src/ui/layout/index.css
- src/ui/layout/index.ts
- src/ui/layout/page-frame/pageFrameView.ts
- src/ui/layout/pane-toggle/paneToggleView.ts
- src/ui/navigation/__tests__/navigationBrowserFixture.ts
- src/ui/navigation/index.css
- src/ui/navigation/index.ts
- src/ui/navigation/keyboard/navigationKeyboardController.ts
- src/ui/navigation/pagination/pagination.css
- src/ui/navigation/pagination/paginationController.ts
- src/ui/navigation/pagination/paginationState.ts
- src/ui/navigation/pagination/paginationView.ts
- src/ui/navigation/section-navigation/sectionNavigation.css
- src/ui/navigation/section-navigation/sectionNavigationTypes.ts
- src/ui/navigation/section-navigation/sectionNavigationView.ts
- src/ui/navigation/segmented-control/segmentedControl.css
- src/ui/navigation/segmented-control/segmentedControlView.ts
- src/ui/navigation/sidebar/navigationItemView.ts
- src/ui/navigation/sidebar/sidebarView.ts
- src/ui/navigation/tabs/tabs.css
- src/ui/navigation/tabs/tabsView.ts
- src/ui/navigation/topbar/topbarView.ts

### 修改（53项）

- README.md
- architecture/state-ownership.json
- contracts/global-visual-system-contract.json
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-ci-fix-state-provenance.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-feedback-and-overlays.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
- scripts/design-refresh/browserFixture.ts
- scripts/design-refresh/fixtureData.ts
- scripts/design-refresh/productionFixture.mjs
- scripts/design-refresh/verifyApprovedDesign.mjs
- scripts/verify-core-workspace-hierarchy.mjs
- scripts/verify-dual-level-navigation.mjs
- scripts/verify-entity-resource-center.mjs
- scripts/verify-extended-workspace-hierarchy.mjs
- scripts/verify-global-visual-system.mjs
- scripts/verify-inline-workspace-and-resource-scroll.mjs
- scripts/verify-stage1-balanced-ui.mjs
- scripts/verify-stage2-responsive-catalog-ui.mjs
- scripts/verify-team-player-management.mjs
- scripts/verify-workspace-ui.mjs
- src/app/shell/application-shell/applicationShellController.ts
- src/app/shell/application-shell/applicationShellView.ts
- src/app/shell/content-region/contentRegionView.ts
- src/app/shell/page-frame/pageFrameView.ts
- src/app/shell/sidebar/sidebarView.ts
- src/app/shell/topbar/topbarView.ts
- src/main.ts
- src/pages/analytics.ts
- src/pages/apiWorkspace.ts
- src/pages/architecture.ts
- src/pages/database.ts
- src/pages/lineups.ts
- src/pages/logs.ts
- src/pages/openai.ts
- src/pages/players.ts
- src/pages/prediction.ts
- src/pages/release.ts
- src/pages/rules.ts
- src/pages/teams.ts
- src/pages/teams/teamWorkspace.css
- src/pages/workbooks.ts
- src/styles/entityCenter.css
- tests/ui/approved-design-refresh.html
- tests/ui/core-workspace-hierarchy.html
- tests/ui/extended-workspace-hierarchy.html
- tests/ui/lineup-preset-editor.html
- tests/ui/reported-analysis-focus.html
- tests/ui/reported-entity-workspace.html
- tests/ui/reported-rules-directory.html
- tests/ui/task-ui-master-detail.html
- tests/ui/task-ui-player-prefill.html
- tests/ui/task-ui-profile-editor.html
- tests/ui/task-ui-review-workflow.html

### 移动/重命名（5项）

- src/app/shell/application-shell/applicationShell.css → src/ui/layout/application-frame/applicationFrame.css
- src/app/shell/content-region/contentRegion.css → src/ui/layout/content-region/contentRegion.css
- src/app/shell/page-frame/pageFrame.css → src/ui/layout/page-frame/pageFrame.css
- src/app/shell/sidebar/sidebar.css → src/ui/navigation/sidebar/sidebar.css
- src/app/shell/topbar/topbar.css → src/ui/navigation/topbar/topbar.css

### Git旧路径退出（2项）

- src/components/workspace.ts
- src/styles/foundation/workspaceNavigation.css


## 本地交付证据（2026-10-29推送前历史记录）

完整93步前端exit0；最终类型/构建、模块/状态/保护资产门禁全部保留。新组件348、Shell1104、64视图1480项检查通过；Vite构建245模块，main JS640.70kB（gzip183.44kB），保留500kB提示。最终Windows startup为5条有效记录、0无效行、0错误，独立运行目录O:/Project/FOODBALL/Temp/r12-15-native-verified。Impeccable reviewer最终SHIP，documenter审计无阻塞性冲突；报告位于本任务visualizations目录的r12-15-review.md与r12-15-design-documentation-audit.md。文档仅随后补结果，没有再改机器来源。该本地验收时间点尚未推送。随后bd9172ad已经公开推送，其精确CI/安装包/startup于2026-10-30独立验收通过，见本记录开头；Full业务链路仍未验证。
