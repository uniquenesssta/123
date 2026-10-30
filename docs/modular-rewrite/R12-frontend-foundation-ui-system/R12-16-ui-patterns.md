# R12-16 UI Patterns

状态：VERIFYING。本地六类Pattern及真实调用完成；完整门禁、构建与实际Windows startup通过；独立UI复审SHIP；独立文档审计完成。本批尚未公开推送，精确CI、MSI/NSIS在发布后另验，不轮询。用户2026-10-30授权“收尾15开始16”；基线bd9172ad767fc7ad6c574d0d025875d3c26e6309，既有rewrite/r12-frontend-foundation-ui-system分支，O:/Project/FOODBALL/123 r3。17～19未实施。

## 15前置验收与回退基线

2026-10-30收尾：bd9172ad767fc7ad6c574d0d025875d3c26e6309 的 Windows run40896297611 / job123526193561 全步骤success；artifact12911330330，48,801,140 bytes；下载ZIP SHA-256 629dba382314413789e71e9d71d2ae25a92099d67409efe06e4ba557f064d5da与GitHub摘要一致。MSI17,969,152 bytes、NSIS15,606,527 bytes；startup pass，7条有效日志、0无效、0错误，bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务链路。证据 O:/Project/FOODBALL/Temp/r12-15-bd9172ad-evidence.zip；只核验完成结果，没有轮询。

## 目标、依据与视觉边界

按R12任务书实现entity-directory、entity-detail、workbench、async-boundary、destructive-action、file-workflow；Pattern只封装交互结构，不接受业务repository或Feature state。沿用批准的[设计接入记录](approved-design-integration-20261028.md)、现有tokens及14/15组件；不替换字体、颜色、字重、图标、间距、CSS或原型。实时Figma session92118bd2-a0c4-42d1-9418-0039553900fe已读取目录527:54630、档案527:57101、页标题404:14589、加载387:11012及确认头1134:6850的高保真上下文和截图；无画布写入、新变量、新样式或新资产。

完整Screens索引和一次use_figma只读查询遇到Transport中断，改为已知节点读取；467:5786仅返回稀疏结构，未用其实现UI。文件工作包只迁移现有已批准实现的组合，未冒充新Figma工作流设计或原型验收。对应[映射记录](R12-16-design-mapping.json)仅为仓库交接，不是已发布Code Connect。

## 唯一职责与实际接入

| Pattern | 负责内容 | 真实调用与保留边界 |
|---|---|---|
| entity-directory | 目录frame、inspector开合布局类、选择命令区位置 | Teams/Players；筛选、结果、游标、对象身份和选择仍由原页面/控制器持有 |
| entity-detail | 当前对象标题、返回动作和编辑主体 | Teams/Players完整档案、工作包及新增；跨页面返回仍使用原来源动作 |
| workbench | 页标题/上下文、固定本地导航、可伸缩任务区 | Teams/Players组合；既有任务页标题和上下文入口统一迁入；退出旧components/taskWorkspace.ts唯一旧路径 |
| async-boundary | 组合保留内容、pending inert/aria-busy、失败恢复 | Shell真实contentRegionView；请求与过期丢弃仍在Router/AsyncScope，草稿和恢复仍在WorkspaceCoordinator |
| destructive-action | 临时确认DOM host与EventScope监听器生命周期 | ModalController；复用14 ConfirmationDialogController/State的名称门槛、busy、错误核对及epoch，不复制待执行请求；成功后关闭对应详情仍由ModalController的身份比较决定 |
| file-workflow | 文件入口、实际预检/空状态组合 | Teams teamPackagePanel与Players spreadsheetPanel；原选择文件、解析/预检、冲突选择、提交、数据库回读链路保持，ready/错误/覆盖阻断规则仍在原业务owner |

受信任HTML槽来自现有页面构造器；文本/属性在其owner或Pattern边界escape。没有接受动态repository、传输器或feature state，也没有新增生产依赖、API/DTO、Schema、配置格式、Rust/模型计算变化。模型保护资产和公共命令检查全部保留。通用样式入口未改；旧taskWorkspace.css还含Feature布局，留给已规划的18，不为制造移动数量整体迁走。

## 验证结果与证据

所有开发进程先加载O:/Project/FOODBALL/project-temp.ps1，TEMP/TMP只在进程中指向O:/Project/FOODBALL/Temp。旧验证脚本递归清理用retain-test-artifacts.cjs改成保留产物，只改变清理行为，不改断言；本次未批量删除目录或文件。

| 验证种类 | 实际命令/证据 | 结果 |
|---|---|---|
| 静态与架构 | 完整前端内的模块、状态、保护资产、服务/命令/Schema契约；tsc --noEmit | PASS；35个状态owner，确认request仍由14持有，只登记新host监听器owner |
| 新Pattern自动测试 | scripts/patterns/verifyPatterns.mjs | 234项原生Windows浏览器检查PASS，360/960/1440×明暗；另11项真实Teams/Players文件视图门槛PASS |
| 现有真实页面 | design-refresh/verifyApprovedDesign.mjs | 64视图、1480项PASS；960/1280/1440/1920×明暗，目录宽度、速览开合、名单/筛选独立滚动、44px阵容行、控件居中、局部加载/焦点和固定分页均保留 |
| Shell及组件回归 | 完整前端内真实Shell、Forms、DataDisplay、Overlays、Navigation/Layout等 | Shell1104项、Navigation/Layout348项及原17份截图基线等全部PASS |
| 完整前端与生产构建 | npm.cmd run verify:frontend；Temp/r12-16-frontend-final.log | 94步exit0；TypeScript PASS；Vite257模块，main JS641.62kB/gzip183.89kB；既有500kB提示未关闭/放宽 |
| Rust格式 | cargo fmt --all -- --check；Temp/r12-16-rust-fmt.log | PASS |
| Rust静态检查 | cargo clippy --locked --offline --workspace --all-targets -- -D warnings；Temp/r12-16-rust-clippy.log | PASS |
| Rust自动测试 | cargo test --locked --offline --workspace；Temp/r12-16-rust-test.log | 748 passed、0 failed、39 ignored；ignored不算真实PG/私有回归通过 |
| 实际Windows应用 | cargo build --locked --offline --package football-match-model-desktop；隔离标识com.footballmatchmodel.platform.r1216validation，真实123 r3 Vite/bootstrap入口 | startup PASS；7条有效、0无效、0错误；bootstrap/read_workspace_state/save_workspace_state；Temp/r12-16-native-verified/startup-report.json |
| 工作包补充浏览器复核 | Temp/r12-16-workbooks/results.json；真实页面与Shell、明确标注的合成预检输入 | 16个初始视图及8个内部滚动视图共24状态PASS；下部提交可见、外层scrollTop=0、冲突提交禁用并保留2候选；合成记录明确标注 |
| 本批精确CI/安装包 | 本地提交尚未公开推送 | 未执行；发布后单独验收；不以15结果替代16，不轮询 |

最终完整前端新Pattern证据：O:/Project/FOODBALL/logs/r12-16-patterns-1793327188911；真实页面截图：O:/Project/FOODBALL/Temp/football-approved-design-VuSeHm；最终Navigation/Layout证据：O:/Project/FOODBALL/logs/r12-15-navigation-1793327014416。独立Impeccable finish reviewer结论SHIP，仅覆盖本批结构迁移；报告C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r12-16-review.md，含五项必需章节。另独立执行基线bd9172ad与当前页面构造器64组合比较，仅归一化新增Pattern标记/类顺序/空白，动作、文本、表单、结构与数据属性不忽略，0差异。独立documenter核对现有设计资料、真实调用、最终清单及原始验证日志，未发现阻塞性设计文档冲突；报告C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r12-16-design-documentation-audit.md。没有创建第二套PRODUCT/DESIGN或改写已有批准快照。

## 失败尝试、检查迁移与限制

- 初次完整前端candidate在旧源码片段顺序检查处失败；测试仍在找页面内展开的标记，而实现已迁移到Pattern。保留该次Temp/r12-16-frontend-candidate.log为失败，不计PASS。
- 自动审批拒绝了最初拟采用的“仅凭Pattern源码存在即可通过”的改法；该脚本未执行。随后采用实际生产页面构造器返回HTML作为原断言输入，完整保留层级、master-detail、详情及筛选/名单/速览顺序要求；全部94步最终通过。没有关闭或降低原门槛。
- 开发初期类型检查发现导入路径和测试表单属性不匹配，架构登记缺少forbidden/count；均按真实接口与契约修正。文件按钮新增测试曾有正则转义错误，修正测试本身后11种真实视图输入通过；没有修改生产禁用门槛来满足测试。
- startup前两次预检分别误查旧入口及后台Vite端口拒绝；客户端均未启动，不计为startup通过。改用受管Vite并核实/src/bootstrap/main.ts后实际客户端通过。只停止本任务专用Vite及应用进程，未清理用户既有进程或数据库。
- Impeccable context沿用前会话已披露的引擎缺失回退；本次detect再次证实engine0.1.17未安装、缓存不可写。未宣称CLI detector/build-phase通过，依赖实际浏览器与独立人工审查；未安装新引擎或修改用户全局配置。
- Windows startup不等于原生窗口逐页完整交互或Full真实数据库/模型/XLSX业务链；真实PostgreSQL、私有固定模型及39项ignored仍未验证。测试预检/页面样例明确是夹具，不是生产硬编码结果。

## 文件清单、状态与回退

最终暂存按git diff --cached --name-status --find-renames核对：46项Git变更（17新增、28修改、1次职责迁移，R073；原workbench共享实现迁入并扩展），对应47个前后路径；没有额外删除。以下为实际清单。阶段保持VERIFYING直至本批精确CI/安装包验收；17～19不推进。按原子提交git revert恢复本批迁移，不手工复制旧实现、不批量删除目录；此前可回退基线bd9172ad已独立验收。


### 新增（17项）

- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-16-design-mapping.json
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-16-ui-patterns.md
- scripts/patterns/renderResourceMarkup.mjs
- scripts/patterns/verifyPatterns.mjs
- src/ui/patterns/__tests__/patternsBrowserFixture.ts
- src/ui/patterns/async-boundary/asyncBoundaryView.ts
- src/ui/patterns/async-boundary/index.ts
- src/ui/patterns/destructive-action/destructiveActionController.ts
- src/ui/patterns/destructive-action/index.ts
- src/ui/patterns/entity-detail/entityDetailView.ts
- src/ui/patterns/entity-detail/index.ts
- src/ui/patterns/entity-directory/entityDirectoryView.ts
- src/ui/patterns/entity-directory/index.ts
- src/ui/patterns/file-workflow/fileWorkflowView.ts
- src/ui/patterns/file-workflow/index.ts
- src/ui/patterns/index.ts
- src/ui/patterns/workbench/index.ts

### 修改（28项）

- README.md
- architecture/module-boundaries.json
- architecture/state-ownership.json
- docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-15-navigation-and-layout.md
- docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
- scripts/verify-core-workspace-hierarchy.mjs
- scripts/verify-entity-resource-center.mjs
- scripts/verify-frontend.mjs
- scripts/verify-player-pagination-team-filter.mjs
- scripts/verify-profile-editor-persistence.mjs
- scripts/verify-task-ui.mjs
- scripts/verify-workspace-ui.mjs
- src/app/modal/modalCoordinator.ts
- src/app/shell/content-region/contentRegionView.ts
- src/pages/analytics.ts
- src/pages/apiWorkspace.ts
- src/pages/architecture.ts
- src/pages/database.ts
- src/pages/lineupPresets.ts
- src/pages/lineups.ts
- src/pages/logs.ts
- src/pages/openai.ts
- src/pages/players.ts
- src/pages/prediction.ts
- src/pages/release.ts
- src/pages/review.ts
- src/pages/rules.ts
- src/pages/teams.ts

### 移动/重命名（1项）

- src/components/taskWorkspace.ts → src/ui/patterns/workbench/workbenchView.ts

### 删除（0项）

无。

最终来源核对：41项代码/配置/测试文件在完整验证与独立审查后未变；交接mapping JSON仅将消费者数量措辞订正为实际14个页面，已重新解析核对，无UI或运行代码变化。42项非Markdown最终摘要记录于C:/Users/uniqu/Documents/ChatGPT/项目代码模块性优化 足球/r12-16-validation-sources.json；哈希覆盖最终working tree与暂存内容。
