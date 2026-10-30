# R12-17 Accessibility、Icons、Assets

状态：DONE。用户2026-10-30授权“收尾16开始17”，并明确后续已授权范围内验证通过直接提交推送，不逐批确认。既有rewrite/r12-frontend-foundation-ui-system分支，O:/Project/FOODBALL/123 r3；基线fa77c8d73126f56282801f4e3b306ded6f008d5d。用户随后授权18，19未实施。17自身精确CI/MSI/NSIS、产物摘要与startup已于2026-10-30独立核验，不轮询。

## 16前置验收与收尾

2026-10-30独立核验：fa77c8d73126f56282801f4e3b306ded6f008d5d的Windows run41000941767 / job123884917625全步骤success；artifact12955471415，50,607,082 bytes，下载ZIP SHA-256 98d90a8931d732fece52fda0955284cdd873947959881b8e6b5fad4f9e87db49与GitHub摘要一致。MSI17,969,152 bytes、NSIS15,606,556 bytes；startup pass，7条有效、0无效、0错误，bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务。只核验完成结果，不轮询。证据O:/Project/FOODBALL/Temp/r12-16-fa77c8d7-evidence.zip。

## 改动、职责与真实接入

- focusManager和focusTrap收敛DOM焦点资格、模态范围、返回目标及原生Tab顺序。DialogController实际调用；PageFrame和Workspace恢复也采用同一资格判定。Workspace仍是持久化视图快照唯一owner。
- KeyboardShortcutController在组合根的实际生命周期启动/销毁，替代原Ctrl/Meta K分支。IME组合/229、重复、Alt、模态和编辑区域策略明确，CommandMenu仍执行原路由动作。
- LiveRegion由真实ToastController使用，纯文本、重复播报、过期定时器和销毁统一；WarningStore仍持有通知业务状态。
- reducedMotion在实际AI对话和工作区锚点滚动时读取当前媒体偏好；accessibleName由真实primitive、FieldLabel和Dialog标题调用，不改变作者文案和边界转义。
- 37项icon registry和25项兼容AppIcon名称统一到Icon Slot；逐项退出src/components/icons.ts并切换全部实际调用。受控资源目录src/assets已有实体资源保持，src/ui/assets持有59项只读元数据（55 SVG、2字体、2许可证），没有生产运行时下载或文件I/O。

## Figma与视觉依据

沿用用户已确认的明暗、字体字重、排布和间距。Figma file PN0Whgu6HLWIHx4Mv6aHfu，session4cafbf1f-cd99-458b-b8de-f0a586be8e81；Icon Slot37:44，Search54:208、Refresh88:643、Compare88:684、Cards88:706、Detail88:727、More88:747、Calendar139:212上下文/截图已读取。7份原始SVG原样下载，路径和根尺寸不重绘；Plus/Calendar根24.875px、外描边-0.875px由24px载体缩放兼容。Calendar两组笔画均随主题currentColor。没有Figma画布写入、新token/style或生产依赖；完整来源摘要及真实调用见[R12-17-design-mapping.json](R12-17-design-mapping.json)。

## 验证与证据

| 类别 | 命令/证据 | 结果 |
|---|---|---|
| 静态 | tsc --noEmit；完整前端内架构、单一owner、保护资产/公开边界门禁 | 冻结版95步内全部通过；591份源码/脚本/架构摘要复核一致 |
| 最小行为 | node scripts/accessibility/verifyAccessibility.mjs | 1,950项Windows Chromium检查通过，360/960/1440宽度×明暗6组，每组325项；59份长度、SHA-256、SVG根尺寸和完整目录一致 |
| 完整前端与构建 | node scripts/verify-frontend.mjs | 冻结版95步全部通过（exit0），包含93脚本、TypeScript和Vite生产构建；日志O:/Project/FOODBALL/logs/r12-17-frontend-frozen.log |
| 页面保留 | production构造器与CSS，64 views、1480 checks | 通过；首轮O:/Project/FOODBALL/tmp/football-approved-design-LIJFNj，合成夹具仅用于测试，不进入生产数据流 |
| Rust | cargo fmt --all -- --check；cargo clippy --workspace --all-targets --locked --offline -- -D warnings；cargo test --workspace --locked --offline | 通过；748 passed、39 ignored、0 failed；fmt/Clippy以本次独立终端执行exit0为证，test日志O:/Project/FOODBALL/logs/r12-17-rust-final.log仅覆盖cargo test；ignored不作实际PG/外部环境通过 |
| Windows实际运行 | 隔离标识com.footballmatchmodel.platform.r1217validation，build.devUrl1421，独立cargo target；真实Vite /src/bootstrap/main.ts | startup PASS；7有效、0无效、0错误；bootstrap/read_workspace_state/save_workspace_state；O:/Project/FOODBALL/Temp/r12-17-native-verified/startup-report.json |
| 视觉与独立复审 | Impeccable，实际截图与fresh finish reviewer | fresh finish reviewer结论ship（五部分完整，无必改项）；普通扩展沿用R12-16映射、approved-design-integration-20261028.md及当前tokens/代码；仓库无PRODUCT/DESIGN，不新建第二套设计系统 |
| 本批公开CI/安装包 | 推送后精确SHA独立验收 | a4ce5177精确run41017231141/job123936218536/artifact12962007254全步骤success；ZIP摘要及MSI/NSIS和startup核验通过；不轮询 |

专项最终截图与accessibility-results.json在O:/Project/FOODBALL/logs/r12-17-accessibility-1793341313662；另一次完整门禁专项在r12-17-accessibility-1793341527075；冻结版最终专项目录O:/Project/FOODBALL/logs/r12-17-accessibility-1793342087800，结果见最终前端日志。独立复审报告C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r12-17-finish-review.md（ship，覆盖本批冻结源码与指定截图，不等于阶段DONE）。覆盖：原生Tab/ShiftTab、正tabindex、checked radio、disabled fieldset、hidden/inert/visibility/aria-hidden焦点返回、嵌套菜单Tab/Escape、模态快捷键隔离、一次绑定和销毁、LiveRegion重复/过期/销毁、原生media切换、所有实际按钮AX名称、图标尺寸及明暗笔画。合成composition/229事件只证明快捷键guard，不代表完整真实输入法操作；AX树不代表NVDA/Narrator人工语音验收。

初始tsc暴露未用constructor属性及测试导入，已修正。截图发现Calendar暗色内部笔画和动画中途裁切，分别修复CSS继承、补笔画计算值断言与等待动画结束后截图。Impeccable0.1.17引擎初始缺失且默认缓存不可写。改项目Temp缓存已可写；通过既有Git代理取到20,285,328 bytes引擎，但官方摘要侧文件和API摘要请求持续TLS/EOF失败。用户恢复代理后，官方侧文件成功下载，实际SHA-256 EDB62BFF4EE48DC59B6B34F5788B9052FF0EFC73019A4E4FC3B1067A4EBF94EE与官方一致，engine-probe确认0.1.17；使用位置参数扫描32个本批AM生产TS/CSS路径（排除测试夹具），exit0、JSON数组[]。context初始未成功且未重复执行；detect已经补跑通过。未跳过校验、使用第三方镜像、忽略检测规则或修改全局代理/技能缓存；读已有设计并作实际渲染与独立复审。默认1420开发窗口和可执行文件正被占用，最初直接构建拒绝访问；改独立target/1421成功，未关闭用户窗口，验收仅关闭本次创建的隔离进程。

独立设计文档审计报告C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r12-17-design-documentation-audit.md；已核对16 ZIP、实际清单、59实体资产、Figma映射、冻结591份文件和最终日志，并补核官方引擎摘要、版本与32路径检测证据。没有新增或重建PRODUCT/DESIGN、token/style；没有代码或设计必改项。

官方引擎仅保存在O:/Project/FOODBALL/Temp/impeccable-runtime/manual-0.1.17/impeccable.exe；后续开发进程如需复用，设置IMPECCABLE_BIN到该路径及IMPECCABLE_HOME=O:/Project/FOODBALL/Temp/impeccable-runtime。检测原始JSON：O:/Project/FOODBALL/logs/r12-17-impeccable-detect.json。

## 兼容性、限制与回退

公共Tauri命令、DTO、Schema、数据库/配置格式、日志等级及模型保护资产不变；没有API/repository新owner，无业务假数据和空实现。仅通用DOM临时状态新增具名owner，architecture两份清单同步。Vite既有大chunk提示未放宽，未承诺全业务Full或真实PG/私有模型/外部API通过；18/19需另行授权。开发脚本进程级TEMP/TMP继续通过O:/Project/FOODBALL/project-temp.ps1配置到项目tmp目录，产物保留不批量清理。

回退以基线fa77c8d7为准，通过Git审阅/逆向本批提交恢复；不手工复制原图标或删除业务能力。源冻结摘要O:/Project/FOODBALL/Temp/r12-17-source-freeze.json，提交前再次核对。

## 实际文件清单

以下清单由本批git diff --name-status和git ls-files --others --exclude-standard生成，最终git diff --cached --name-status --find-renames逐项核对一致。新增24，修改26，退出1。没有批量删除，唯一退出路径逐项替换；没有纯移动。

### 新增

- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-17-accessibility-icons-and-assets.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-17-design-mapping.json`
- `scripts/accessibility/verifyAccessibility.mjs`
- `src/assets/icons/calendar.svg`
- `src/assets/icons/cards.svg`
- `src/assets/icons/compare.svg`
- `src/assets/icons/detail.svg`
- `src/assets/icons/more.svg`
- `src/assets/icons/refresh.svg`
- `src/assets/icons/search.svg`
- `src/ui/accessibility/__tests__/accessibilityBrowserFixture.ts`
- `src/ui/accessibility/accessibleName.ts`
- `src/ui/accessibility/focusManager.ts`
- `src/ui/accessibility/focusTrap.ts`
- `src/ui/accessibility/index.ts`
- `src/ui/accessibility/keyboardShortcut.ts`
- `src/ui/accessibility/liveRegion.ts`
- `src/ui/accessibility/reducedMotion.ts`
- `src/ui/assets/assetManifest.ts`
- `src/ui/assets/assetRegistry.ts`
- `src/ui/assets/index.ts`
- `src/ui/icons/appIconView.ts`
- `src/ui/icons/iconRegistry.ts`
- `src/ui/icons/index.ts`

### 修改

- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-16-ui-patterns.md`
- `docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md`
- `README.md`
- `scripts/verify-frontend.mjs`
- `scripts/verify-workspace-ui.mjs`
- `src/app/router/route.ts`
- `src/app/shell/page-frame/pageFrameController.ts`
- `src/app/workspace/workspaceRecovery.ts`
- `src/main.ts`
- `src/pages/apiWorkspace.ts`
- `src/pages/lineupPresets.ts`
- `src/pages/lineups.ts`
- `src/pages/players.ts`
- `src/pages/release.ts`
- `src/pages/teams.ts`
- `src/pages/workbooks.ts`
- `src/ui/feedback/toast/toastController.ts`
- `src/ui/forms/field-label/fieldLabelView.ts`
- `src/ui/icons/iconSlot.css`
- `src/ui/icons/iconSlot.ts`
- `src/ui/layout/pane-toggle/paneToggleView.ts`
- `src/ui/overlays/dialog/dialogController.ts`
- `src/ui/overlays/dialog/dialogView.ts`
- `src/ui/primitives/controlAttributes.ts`

### 退出的原路径

- `src/components/icons.ts`

### 移动/重命名

无；src/components/icons.ts职责由src/ui/icons/appIconView.ts和iconRegistry.ts承接，原25项公开名称保持。

## 2026-10-30 精确Windows验收收尾

a4ce5177c95e70df5bda5c082de641228e0263e6，run41017231141/job123936218536全步骤success；artifact12962007254，52,285,163 bytes，下载ZIP SHA-256 d2fc3196eb0d044f42c98fea74454f300d93babdf1689a8ee348dcb5a9f53058与GitHub摘要一致。MSI17,973,248 bytes、NSIS15,610,491 bytes；logs/windows-acceptance-20261030-070007.json：startup pass，7有效、0无效、0错误；bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务。下载包O:/Project/FOODBALL/Temp/r12-17-a4ce5177-evidence.zip。只读取完成结果、不轮询。17 DONE，18按用户本轮授权推进，19未实施。
