# 足球模型平台公开外壳

## R13-03/04 赛事层级、规则与路由（2026-10-31，VERIFYING）

原代码12ba495c的CI失败在依赖安装前加载TypeScript。现保留全部31项行为，移到安装依赖后的原前端门禁；无依赖源码副本已复现并验证完整架构门禁修复。完整96步正在复验，修复提交CI/安装包待验。04已按任务书并入03，本轮核对规则包、绑定和路由义务，不重复实施；真实PG/Full仍BA-14/R17-06待验，05未启动。详[03/04记录](docs/modular-rewrite/R13-core-ui-features/R13-03-competition-rules-and-routing.md)。

## R13-02 数据库连接与危险操作（2026-10-31收尾 DONE）

连接/断开使用同一串行工作流，重置确认绑定当前摘要身份；失败或未知结果不清工作区、不自动重试，临时凭据用后释放，未知统计不补0。96步前端、15项生产行为、21项Windows浏览器及原生startup通过；d167e948精确Windows CI、证据摘要、MSI/NSIS及startup已核验通过。详[02记录](docs/modular-rewrite/R13-core-ui-features/R13-02-database-connection-and-reset.md)。03当前结果见本页及阶段索引。

## R13-01 首页与架构只读接入（2026-10-31收尾 DONE）

两个只读页面接入Feature公开入口，首页统计由当前Session摘要投影，快捷入口沿原Router；9种原HTML对比完全一致。96步前端、7项新行为、18项Windows浏览器生产运行和当前桌面开发版startup通过；精确9cdd1f9b的Windows CI、下载证据摘要、MSI/NSIS与startup报告已核验通过，不轮询。详[实施记录](docs/modular-rewrite/R13-core-ui-features/R13-01-read-only-entry-pilot.md)与[阶段索引](docs/modular-rewrite/R13-core-ui-features/README.md)。02的当前实施状态见阶段索引。

## 后续任务书订正（2026-10-30）

按当前代码与软件要求修订总纲中后续执行要求及R13～R18：原67项能力完整映射到24个交付批次，按职责复用/迁移/修复，测试随批交付，最终真实环境验收保留。排除UI改版；用户已明确R12结束，本轮不调整R12任务书或索引，R13开发尚未启动。详[审计与任务去向](docs/modular-rewrite/task-plan-review-20261030.md)。

本轮进一步把24批补成源码对照执行卡，明确状态归属、数据流、原子步骤、异常/竞态场景、现有检查入口和停止/回退条件。执行前阅读[共享执行约定](docs/modular-rewrite/remaining-execution-contract.md)，再进入对应阶段任务书；文档中的验收场景是待实施要求，不是软件已通过记录。

R13启动前已补做[R1～R12遗留项基线核对](docs/modular-rewrite/prior-stage-baseline-20261030.md)：12阶段覆盖、16项承接账本、9项当前静态检查通过。已知风险须在对应业务批次交付前处理，必需真实待验最迟R17-06关闭；不把静态通过当成前期问题已全部修复。

## R12-18 样式入口与边界（2026-10-30，VERIFYING）

17精确CI、产物与startup已收尾；18迁移单一生产CSS入口、具名样式职责与页面作用域，保留最新批准视觉。181份CSS单一可达、222受限选择器与15负向检查通过；64组36,682,704项计算样式值0差异、17截图0像素差、Rust与实际Windows startup通过；最终96步门禁已通过，独立复审两项修复均resolved（ship），独立设计文档审计完成，详[18记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-18-样式入口-and-边界.md)。19未实施。

## R12-17 Accessibility、Icons 与 Assets（2026-10-30收尾 DONE）

16精确fa77c8d7的CI、产物摘要、MSI/NSIS与startup已独立核验收尾。17接通六项无障碍能力、37项统一图标和59份受控本地资产；保留批准的字体、字重、配色、布局及真实业务owner。1,950项Windows浏览器专项、59份资源摘要、Rust格式/Clippy/748项测试及隔离Windows startup已通过；最终95步前端门禁通过、Impeccable32路径检测0项发现、独立复审ship及设计文档审计完成，详见[17记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-17-accessibility-icons-and-assets.md)。按用户后续推送长期授权交付；本批a4ce5177精确Windows run41017231141、下载产物摘要、MSI/NSIS和startup已独立核验；18随后授权推进，19未实施。

## R12-16 UI Patterns（2026-10-30收尾 DONE）

15精确bd9172ad的CI、产物摘要、MSI/NSIS与startup已独立核验收尾。16完成六类交互结构的真实接入，保留批准的Figma视觉、数据和写入门槛；94步前端/构建、234项新浏览器检查、64视图1480项页面检查、Rust748项与实际Windows startup通过；独立UI复审SHIP，独立文档审计完成，本批精确Windows run41000941767、MSI/NSIS、产物摘要及startup已独立核验通过。详[16记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-16-ui-patterns.md)。17随后按用户授权推进，18～19未实施。

## R12-15 Navigation 与 Layout（2026-10-30收尾 DONE）

14修复提交5886a439cc301a7fa315c1a8adf7b3b33c0b1d87：Windows run40863418461 / job123412066355 全步骤成功；artifact12894043614，ZIP SHA-256 2c06b3c06ffd25530680029b45fd3efaa69a3afec7d3241c00904abe59da4a5e。MSI 17,969,152 bytes、NSIS 15,605,981 bytes；startup profile pass，7条有效日志、0错误，bootstrap/read_workspace_state/save_workspace_state完成。startup不等于Full业务链路。2026-10-29独立核验，不轮询。

15按已批准Figma收敛通用导航与布局，修正目录宽度/高度和筛选滚动；348项新组件、64视图1480项页面检查、完整93步前端/构建、Rust748项及Windows startup通过，独立UI复审SHIP。保留路由、工作区、异步和领域唯一owner；本批精确Windows run40896297611、产物摘要、MSI/NSIS和startup已于2026-10-30核验通过；详[15实施记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-15-navigation-and-layout.md)。

## R12-14 Feedback 与 Overlays（2026-10-29收尾 DONE）

14的Feedback/Overlays及来源修复已收尾：5886a439精确Windows run40863418461全部步骤成功；下载产物哈希一致，MSI/NSIS和startup报告通过。Full业务链路未验证。见[14记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-feedback-and-overlays.md)与[CI来源修复](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-14-ci-fix-state-provenance.md)。用户已授权推进15；16～19未实施。

## 已批准 Figma 重新接入（2026-10-28）

按用户确认的最新画布更新已有前端：共享明暗颜色、随包字体及字重层级统一；首页、双方阵容、球队目录/档案/速览和赛事目录采用当前排布与滚动边界，原有数据、表单、历史和业务门禁继续接通。最新 UI 接入批次不代替 R12-14～19。用户随后授权本批提交到既有分支，后续本地开发改用 `O:\Project\FOODBALL\123 r3`；CI 单独验收，不轮询。验证范围、截图与 Windows 原生窗口验收限制见[接入记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/approved-design-integration-20261028.md)。

## R12-13 Data Display（2026-10-26；2026-10-28收尾 DONE）

八类Data Display及本地验证已完成；13精确dac0e9c的Windows run40218221375、证据ZIP摘要、Rust测试/Clippy、MSI/NSIS和startup报告已核验，13为DONE。startup不等同于Windows Full；14按用户另行授权推进，15～19未实施。详[13记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-13-data-display.md)。

## R12-12 Forms（2026-10-25，DONE）

11自身Windows证据已独立核验收尾。12完成七类Forms，真实数据库连接字段、名称确认及确认动作区统一复用11控件；字段错误保留原生输入与焦点，摘要定位字段，名称门槛支持大小写敏感trim匹配、输入法边界和生命周期清理，业务/后端二次校验保持。272项Windows浏览器表单检查、91步骤前端、架构、18项保护资产和Rust fmt通过；12自身精确7807534/run40194352653全部步骤、ZIP摘要与startup trace已核验；用户授权13。详[12记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-12-forms.md)。

## R12-11 UI Primitives（2026-10-25，DONE）

10自身精确Windows证据已核验收尾。11在唯一primitives目录完成14类控件及公开出口，现有Shell/反馈/确认按钮统一接入；原生表单值、键盘、禁用/只读、加载宽度、下拉定位和销毁保持明确边界。868项Windows控件检查、91步骤完整前端、架构和Rust fmt通过；11精确085097a/run40188280192自身Windows全部步骤、ZIP摘要与startup trace已核验；用户授权12。详[11记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-11-ui-primitives.md)。

## R12-10 Design Tokens（2026-10-25，DONE）

09自身Windows验收、产物摘要和启动日志已核对收尾。10在原FIP目录按10类职责补齐193个设计变量、13个文字样式和6个阴影样式，保留明暗/尺寸/色调模式，Shell与反馈控件接入唯一CSS入口。1,344项原生计算值、91步骤完整前端、1,104项Shell/工作区、136项告警及17视口通过；自身精确e3dcf5d/run40174915845的Windows、下载产物摘要与startup日志已核验，11按本次授权实施。详[10记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-10-design-tokens.md)。

## R12-09 Platform 能力（2026-10-25，DONE）

08自身Windows验收、产物摘要和启动日志已核对收尾。09将原生文件选择、偏好存储、有界脱敏摘要、剪贴板、时钟/防抖/请求序列迁入各自平台目录，接通原调用方，保留存储键/格式、对话框选项和业务命令。新增25项生产平台行为检查，91步骤完整前端回归通过；09自身精确98dc43e/run40145377194的Windows、产物摘要和startup日志核对通过；10按本次用户授权实施。详[09记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-09-files-storage-diagnostics-clipboard-and-timing.md)。

## R12-08 Tauri Transport（2026-10-25，DONE）

07自身Windows验收已核对并收尾。08将Tauri命令调用、错误、追踪、响应和本地取消收敛到唯一`platform/tauri`；171个命令及原API方法签名/参数/返回保持，退出先完成工作区保存再停止请求。24项生产模块测试与命令/类型门禁通过；91步骤完整前端、1,104项Shell/工作区、136项告警与17截图通过，自身精确c16f2c0/run40137847917的Windows、产物与启动日志核对通过，09按本次用户授权实施。详[08记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-08-tauri-transport.md)。

## R12-07 Workspace（2026-10-25，DONE）

06自身Windows验收已核对并收尾。07将工作区收敛到唯一`app/workspace`，分离状态、版本、串行持久化与DOM恢复；读取失败保留原文件，修复重叠保存/清空竞态并接通输入事件自动保存。沿既有Figma外壳和工作区设计，重启、输入、滚动与焦点检查已通过；91步骤完整前端、16项状态/竞态测试、12组1,104项Shell/工作区检查、136告警和17截图通过；自身精确b0b26d6/run40134011943的Windows、产物摘要及启动日志核对通过。08另按用户授权执行。详[07记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-07-workspace.md)。

## R12-06 Warning Center（2026-10-25，DONE）

05已按精确Windows结果收尾。06接通统一顶栏告警入口、日期分组、未读徽标与弹窗；连接错误和持续错误进入唯一会话告警记录。通知采用单槽、去重及悬停/焦点暂停，关闭提示保留任务、输入和业务门禁；任务提示与ARIA同步。已补齐Figma明暗组合稿，清理旧反馈DOM/样式，并复用原生弹窗焦点机制。91步骤前端、136项告警交互、原912项Shell和17视口通过；自身精确793eea9/run40127756204的Windows、产物摘要与启动日志核对通过。07另按本次授权实施。详[06记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-06-warning-center.md)。不轮询CI；交接时核对精确提交的一次性验收证据。

## R12-05 Application Shell（2026-10-25，DONE）

05精确c880d04/run40121131372已核对全部步骤、job日志、产物哈希和startup报告，Windows自动验收通过。05 DONE，06按用户授权实施。设计与原实现详[05记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-05-application-shell.md)。

## R12-04 Session实施记录（2026-10-24，验收前）

03已完成Windows验收并收尾。04已将会话协调、启动摘要、主题与原主题偏好持久化迁入唯一`app/session`，接通真实启动/刷新/停止/销毁；摘要以独立深冻结快照供既有页面读取，最近运行列表统一更新并丢弃过期响应。原验证入口新增18项会话行为检查，实际浏览器验证主题与详情输入/滚动/焦点保持；不新增依赖或验证入口。实际清单与验证见[04记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-04-session.md)。01～03 DONE，04 VERIFYING，05～19 BLOCKED；04自身完整Windows验收待结果，不轮询CI。

## R12-03 Router 与 Navigation（2026-10-24，DONE）

03精确`7f67940`的[Windows run40103980479](https://github.com/uniquenesssta/123/actions/runs/40103980479)完整成功：前端/17视口、24生命周期/21路由行为、Rust、打包、启动与日志通过，下载产物哈希及报告已核对。01～03 DONE；用户要求收尾03开始04，Session现READY，05～19 BLOCKED。详[03记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-03-router-and-navigation.md)和[R12索引](docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md)。遵守不轮询CI；真实PG/XLSX/Full/私有固定回归等最终环境待验继续保留。

当前版本 **0.23.0**。本仓库是平台、数据库、数据准备、路由、工作台与外部模型调用入口的公开版本；真实 P4/P7 预测引擎、参数、Profile、固定比赛、私有研究提示词及模型专用固定回归资产不随仓库分发。

R1–R6 累计补审覆盖 43 个任务节点。根据用户决定，R7-01 后新增 R7-02～06 审计整改，原 R7-02～10 顺延为 R7-07～15；优化拆分、验证与记录约束，沿用现有 Windows 验证和最终封包新库流程，不新增持续回归体系。R7-01 的 A1～A7 修订已通过 Windows CI run `36678914535`，节点 DONE；R7-02 外部 ID 身份保护、R7-03 普通删除与历史引用保护均已通过 Windows CI 并为 DONE。R7-04 清单、验证入口和历史记录已通过 Windows CI run `36706905645` 并为 DONE；R7-05 新增 20 个关键 Application 行为测试，精确 Windows CI run `36712015030` 已通过并 DONE；R7-06 的夹具/时间精度与账本修订已通过 Windows CI run `36734194083` 并 DONE；R7-07 阵容创建与共同并发锁已通过 Windows CI run `36757339619` 并 DONE；R7-08 阵容链/历史已通过 Windows CI run `36801488090` 并 DONE；R7-09 阵容预设已通过 Windows CI run `36808609584` 并 DONE；R7-10 两类导入的批次/行账本职责与冲突计数同步已通过 Windows CI run `36815568401` 并 DONE；R7-11 球员工作簿唯一职责已通过 Windows CI run `36824027121` 并 DONE；R7-12 球队资料包共享职责与冲突计数修订已通过 Windows CI run `36831302483` 并 DONE；R7-13 月度只读聚合与缺口唯一职责已通过 Windows CI run `36854202029` 并 DONE；R7-14 比赛阵容工作簿唯一职责已通过 Windows CI run `36860224760` 并 DONE；R7-15 行/子记录身份收敛及行号前检修订已通过 Windows CI run `36871154039` 并 DONE，R7 节点及 Windows Automated 阶段收口完成。PG/XLSX/Full 单独留到最终封包新库验收。详见 [R7 任务书](docs/football-model-platform-modular-rewrite-19-docs/07-R7-match-lineup-workbook-persistence.md) 与 [阶段索引/累计审计](docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md)。以下逐节点叙述保留实施历史，当前状态见最新 R8-12 及 R8 阶段索引。

R8 阶段唯一分支 `rewrite/r8-prediction-p4-orchestration`，阶段起点 `90680bf`。R8-01 受审计输入构建已独立且 DONE，精确 `cba72fd` / Windows run `36881256338` 实际通过 Application 61/Persistence 135、17 视口、Rust/前端、release/MSI/NSIS/启动。详见 [01 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-01-match-prediction-input-builder.md)。

R8-02 历史读取已收尾，精确 `cc2b0fe` / Windows run `36891488571` 通过 Application 61/Persistence 141、六项新增历史特征测试、17 视口及 Windows 构建/打包/启动，详见 [02 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-02-historical-feature-reader.md)。

R8-03 就绪审计已收尾，精确 `47ba3de` / Windows run `36902069546` 通过 Application 69/Persistence 141、八项新增审计测试、17 视口和 Windows 构建/打包/启动，详见 [03 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-03-readiness-audit.md)。

R8-04 输入清单已收尾，精确 `59c5679` / Windows run `36966815323` 通过 Application 77/Persistence 141、八项新增清单测试、17 视口和 Windows release/MSI/NSIS/启动。详见 [04 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-04-deterministic-input-manifest.md)。

R8-05 路由与模型请求已收尾：精确修复 `2aa99a7` / [Windows run `36971419476`](https://github.com/uniquenesssta/123/actions/runs/36971419476) 通过 Application 85/Persistence 141、十项路由请求测试、17 视口、Rust/前端、Windows release/MSI/NSIS/启动。原 shared/routing 已删除，唯一 route_model_request 持有原选择/校验/上下文/请求职责；首轮测试专用导出引起的 Clippy 失败已修复并通过。详见 [05 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-05-route-and-model-request.md) 与 [R8 索引](docs/modular-rewrite/R08-prediction-p4-orchestration/README.md)。05 收尾文档 CI `37737677787` 亦已通过；真实 PG/私有固定回归/Full 及继承历史删除风险仍最终新库待验。

R8-06 已收尾为 `DONE`：注册查找、supports、predict、完整错误转换和原毫秒计时收拢到唯一模型适配器，原执行器与默认 dry run 直接复用；默认 dry run 保留无额外 supports/validate 的行为。精确 `0969331` / [Windows run `37752995641`](https://github.com/uniquenesssta/123/actions/runs/37752995641) 通过 Application **92** / Persistence **141**、七项新增边界测试、17 视口、Rust/前端、Windows release/MSI/NSIS 及启动验收。详见 [06 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md)。后续当前状态见下段与 R8 索引；本段五份文档收尾提交使用 `[skip ci]`，不代表 07 已验证。

R8-07 Run Persistence 已通过自身 Windows 门禁并收尾为 `DONE`；08已通过并收尾，09当前状态见下段和R8索引。07从已验源码的文档基线 `eadb49d` 开始，旧 `crates/persistence-postgres/src/model_runs.rs` 已删除，原保存、输入审计/快照前检、模型明细、历史读取和隐藏事务分别迁入 `adapters/prediction/runs/{write,input,details,read,visibility}.rs`，`mod.rs` 只登记/导出。公开 PostgresStore 方法及 ModelRunListItem 路径保持；正式保存/影子 nil 继续由原执行用例编排。九项生产函数体、原 SQL、四个公开签名及原测试比较保持一致，无整文件移动/重命名。完整 7 新增、15 修改、1 删除文件及取舍见 [07 实施记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md)。

07 实施时本地实际通过 83/83 现有源码门禁、完整 `npm run verify:architecture`、Rustfmt、18 保护资产/171 命令/46 迁移与 18 PG 静态契约、`git diff --check`；六项破坏探针均拒绝后恢复。新增七项 Persistence 边界单测，原 Application 保存失败测试扩为六类错误；预期 Persistence **148** / Application **92**，须本项精确 Windows CI 实跑，不继承 06 PASS。原 PG target 补共同回滚、快照复用、读回与隐藏不可变断言，仍 ignored，真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 与继承历史删除风险继续最终封包新库待验。沿用原 Windows workflow，启动后停止轮询；无 Linux/macOS Cargo/客户端动态验收或新基础设施。

R8-08 P4 Evidence Ledger 已收尾为 `DONE`：证据追加与冲突建组从旧 p4_records 迁入 `adapters/p4/evidence_ledger/{claims,conflicts,input,references,row}.rs`，mod 仅登记/导出，原证据状态投影和 P4 共用幂等实现各保持唯一 owner。旧文件保留版本登记/研究任务/快照职责，账本实现与原测试迁出，无空转发；10 原生产函数、原测试、31 SQL 和2公开签名核对等价，原文件其余34函数保持。新增9个Rust文件和 [08 实施记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md)，累计10新增/11修改，无整文件移动或删除。

08 实施时实际本地通过83/83原源码检查、完整architecture、Rustfmt、18保护资产/171命令/46迁移及18PG静态契约、git diff --check，六项破坏探针均拒绝恢复。原Persistence target新增8项边界测试，预期 **156** / Application **92**；原PG Stage C扩原文读回/纳秒指纹、引用/SQL失败无残留、并发同键重试与单条审计、去重冲突/单opened事件及append-only，仍ignored。本项须自身Windows CI，启动后停止轮询；真实PG/历史四项/账本/XLSX/Windows Full/私有Golden Master及继承删除风险仍最终封包新库待验。实施时09～12 BLOCKED；当前09状态见下段，无新runner/workflow/target/数据库/依赖。

08 首轮 `91e9585` / [Windows run `37804265878`](https://github.com/uniquenesssta/123/actions/runs/37804265878) 在 Clippy 因 `p4_records.rs` 的测试专用 `EvidenceVerificationState` 顶层导入失败；前端契约、类型、生产构建与17视口已通过，Rust测试/打包/启动未完成。现将导入移入原 `#[cfg(test)]` 模块并同步唯一 Domain 使用摘要，完整架构、Rustfmt 1.88、源码卫生及差异检查通过；业务函数和原测试体保持。修复提交时本项仍 `VERIFYING`，其后取得的精确Windows结果见下段。

08精确修复 `52f23ab` / [Windows run `37817443918`](https://github.com/uniquenesssta/123/actions/runs/37817443918) 全SUCCESS，前端/17视口、Rust fmt/Clippy/workspace tests、Windows构建打包与启动全部通过，首轮Clippy失败已关闭。详见 [08完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md)；用户已授权09开始，10～12仍BLOCKED。收尾只同步五份文档并复用精确CI证据；真实PG/XLSX/Full/私有固定回归继续最终新库待验。

R8-09 Fact Pipeline 已收尾为 `DONE`：目录出口中的命令/编排/事实准备已提取为具名职责，Postgres混合fact_pipeline_records已退出，原接口/Ports及来源/实体/时间/冲突/路由政策保持。精确 `4025781` / [Windows run `37825126808`](https://github.com/uniquenesssta/123/actions/runs/37825126808) 全SUCCESS，Application99/Persistence163、14新增边界测试、17视口、Rust/前端、Windows构建打包与启动全部通过。完整变更与证据见 [09完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-09-fact-pipeline.md)。用户已授权10开始，11～12仍BLOCKED；真实PG/XLSX/Full/私有固定回归及继承历史删除风险继续最终新库待验。

R8-10 Horizon Orchestration 已收尾为 `DONE`：规划预检、时点身份、PLANNED恢复与主流程各有唯一职责，原后台领取/结算与分派分离，任务读写/事件/指纹/投影归 `adapters/p4/horizon/`。精确修复 `481bfcb` / [Windows run `37886029199`](https://github.com/uniquenesssta/123/actions/runs/37886029199) 全SUCCESS，Application114/Persistence169、21项新增测试、17视口、Rust/前端、Windows构建打包与启动通过。完整变更与证据见 [10完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-10-horizon-orchestration.md)。用户授权11开始，12BLOCKED；真实PG/Full/XLSX/私有固定回归及继承删除风险继续最终新库待验。

10实施时本地83源码/完整架构、Rustfmt、保护资产/命令/数据库静态契约与差异检查通过；六破坏探针拒绝恢复。新增15 Application/6 Persistence行为测试，预期 **114/169** 须自身精确Windows CI确认，原PG Stage C扩事务/并发/回滚覆盖仍待实际数据库执行。沿用原Windows workflow，启动后停止轮询；当时11～12 BLOCKED，现10精确通过并授权11。真实PG/XLSX/Full/私有固定回归与继承历史删除风险继续最终新库待验。

R8-10 首轮 `db5fd51` / [Windows run `37884818742`](https://github.com/uniquenesssta/123/actions/runs/37884818742) 因保留的 `p4_orchestration.rs` readiness 使用 `ResearchRunStatus` 却缺少模块导入而报 E0433；前端/类型/生产构建和17视口已通过，Rust测试/打包/启动未完成。修复补回该生产导入，原五函数体、SQL和规划行为保持；既有 Prediction 门禁增加该依赖检查，移除导入的探针已拒绝并恢复，Domain使用摘要同步。修复提交时保持VERIFYING；现精确修复Windows全SUCCESS，10完成证据见上段及完成记录。

R8-11 Workbench Reads 已收尾为 `DONE`：工作台读取归比赛、任务汇总、研究、证据与冲突职责，原四SQL、顺序/错误/NULL与人工覆盖政策保持。精确 `47bda23` / [Windows run `37891346613`](https://github.com/uniquenesssta/123/actions/runs/37891346613) 全SUCCESS，Application118/Persistence169、四项新增测试、17视口、Rust/前端、Windows构建打包与启动通过。完整变更及证据见 [11完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-11-workbench-reads.md)。用户授权12开始，R8仍IN_PROGRESS，不启动R9；真实PG/Full/XLSX/私有固定回归及继承删除风险继续最终新库待验。

R8-12 Freeze Transaction 已收尾为 `DONE`，R8-01～12 阶段代码/节点及 Windows Automated 出口完成。精确 `af3c98c` / [Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) 全 SUCCESS，Application **126/126**、Persistence **175/175**、17 视口、Rust/前端及 Windows release/MSI/NSIS/启动通过；本项8+6新增测试和原快照测试均通过。详见 [12完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-12-freeze-transaction.md) 与 [R8阶段完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-stage-completion.md)。R8任务书到12结束；本次R8收尾时后续 [R9-01 Shared Transport](docs/modular-rewrite/R09-research-ai-backend/README.md) 为 READY，编号待厘清；当前R9状态见下段。真实 PostgreSQL、历史四项数据库验收、账本/并发/回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master，以及继承的 model.runs/0041 历史删除风险继续最终封包新库待验；ignored、公开 unavailable stub 和静态保护指纹均不计真实执行 PASS。 本次仅文档收尾，复用上述精确源码证据并提交 `[skip ci]`。

R9-01 Shared Transport 已收尾为 `DONE`。精确实现 `f3da62d4b84235044c7483399f6f8aa145d9438d` / [Windows run `37933341557`](https://github.com/uniquenesssta/123/actions/runs/37933341557) / job `113829199317` 全 SUCCESS（2026-10-09 21:24北京时间完成）；Gateway **23/23**、contract **16/16**、Application **126/126**、Persistence **175/175**，本项8个新增测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动全部通过。证据artifact `11617684270`，14,032,903字节，SHA-256 `a2178dd81182ec5839db1e0d53b57cc0a40c00802a3f553c159164124ae19c48`。 原传输契约/HTTP/响应解码唯一owner、公开接口与协议保持；详见 [01完成记录](docs/modular-rewrite/R09-research-ai-backend/R09-01-shared-transport.md)。本次仅五份文档收尾 `[skip ci]`，用户已授权同一R9分支开始02 Credentials与Redaction，03～11 BLOCKED。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。

R9-02 Credentials与Redaction已收尾为 `DONE`。精确修复 `05055d1fd90e9a19ee4bc791e769018ca700dd14` / [Windows run `37954536795`](https://github.com/uniquenesssta/123/actions/runs/37954536795) / job `113901577714` 全 SUCCESS，2026-10-10 00:15:32北京时间完成。Gateway **35/35**、contract **16/16**、Application **126/126**、Persistence **175/175**，新增12项及原凭据/示例/传输测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条记录/3完成操作均通过。证据artifact `11628233815`，14,037,356字节，SHA-256 `9a738defde297f2ff10bc0218885efa72d9e7f2c9ad4ff999bcf237cafc0e45a`。完整职责、四项安全修复与文件清单见 [02完成记录](docs/modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md)。用户授权03开始，04～11 BLOCKED；真实数据库/Full/私有固定回归继续最终新库待验。

R9-02 Credentials与Redaction实施时`VERIFYING`，同一R9分支从01收尾b80f09e起。旧credentials.rs删除，密钥生命周期/提供者/目标管理/WindowsIO/解码/错误/模板脱敏各有唯一owner，六原公开接口保持；补齐失败分支及UTF16临时清理，已提取compatible密钥在嵌套模板中脱敏。9A/9M/1D、原4凭据/3示例测试保留，新12项进入原unit target，预期Gateway35/contract16、Application126/Persistence175须自身Windows。83现有静态/完整architecture/Rustfmt/保护资产及8破坏探针通过；完整记录见 [02实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md)。原UI/配置/协议/依赖/DB保持，contract仅更新实际制品路径；03～11BLOCKED。CI开始后停止轮询；真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。

R9-02 首轮 `98d542f` / [Windows run `37944445186`](https://github.com/uniquenesssta/123/actions/runs/37944445186) 在新增 provider Debug 测试被 Clippy `uninlined_format_args` 拦截；前端/类型/构建和17视口已通过，Rust单测/打包/启动未执行。已改为内联参数，提供者生产实现、原测试断言与35项测试数量保持；既有源码卫生检查增加该回归检查，原写法/换行/尾逗号三探针均拒绝并恢复。83/83静态、完整架构、Rustfmt与保护资产通过；当时VERIFYING；现精确修复Windows已通过，该失败已关闭。

R9-03 Retry/Circuit/Cancel实施时`VERIFYING`，基线59af3f2，同一R9分支。原重试预算/退避/可取消等待、单一熔断state与本地token各归resilience具名职责，旧cancellation.rs及client重复实现删除；协议fallback/工具兼容/远端取消与Semaphore原scope保持。5A/10M/1D，无API/DTO/Schema/依赖/DB/UI/生产策略变化。原unit新增12、contract新增5，源码预期Gateway47/contract21、Application126/Persistence175，须自身Windows实跑；83/83静态/完整architecture/Rustfmt/18保护资产/171命令/46迁移18PG静态与6破坏探针均PASS，原生产体/重路由client/251literal与原16contract等价。详见 [03实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-03-retry-circuit-breaker-and-cancel.md)。CI开始后停止轮询，04～11BLOCKED；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承删除风险仍最终新库待验。

R9-03 已收尾为 `DONE`。精确实现 `d58aa38f88c2b59e3ee3b6f568b54d5430843b29` / [Windows run `37960534145`](https://github.com/uniquenesssta/123/actions/runs/37960534145) / job `113921936420` 全 SUCCESS，2026-10-10 01:05:49北京时间完成。Gateway **47/47**、contract **21/21**、Application **126/126**、Persistence **175/175**，新增12单测/5契约及原测试全部通过；完整前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动验收均通过。artifact `11632067812`，14,041,285字节，SHA-256 `71b4a13385cb2f77cb773e0181f2d1987d533148220b657317ad802507450d5b`。 用户授权04正式研究请求开始，05～11BLOCKED。详见 [03完成记录](docs/modular-rewrite/R09-research-ai-backend/R09-03-retry-circuit-breaker-and-cancel.md)。

R9-04 Formal Request已实施、`VERIFYING`。正式输入/Schema校验、Responses载荷及请求前预算分别归唯一职责；原连接测试/Plain等复用唯一Token投影与定价政策。9A/9M/0D，无API/DTO/Schema/配置/依赖/DB/UI或生产策略变化；7原函数及其余Gateway编排、247literal/原47unit/21contract保持。新增13单测/4契约，源码预期Gateway60/contract25、Application126/Persistence175，须本项Windows实跑。83现有静态/完整architecture/Rustfmt/保护资产/171命令/46迁移18PG静态及7破坏探针通过。详见 [04实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-04-formal-research-request.md)。CI开始后停止轮询，05～11BLOCKED；真实PG/Full/XLSX/私有固定回归及继承删除风险仍最终新库待验。

R9-04 首轮 `16b5d4e` / [Windows run `38021521371`](https://github.com/uniquenesssta/123/actions/runs/38021521371) 的新增协议拒绝契约仅切换Chat Completions，却保留Responses显式端点，Gateway初始化报InvalidConfiguration，未到达该测试断言。前端/类型/构建/17视口及fmt/Clippy通过，Gateway60、Application126、Persistence175通过；contract24/25，release/MSI/NSIS/启动未执行。修复只补该夹具的匹配Chat端点，生产实现及全部断言/测试数量保持；既有兼容传输门禁增加夹具顺序与原断言守卫，缺失/错协议/构造后配置三探针均拒绝恢复。83/83静态、完整架构、Rustfmt/保护资产/命令/数据库静态和等价复核通过；仍VERIFYING，05～11BLOCKED，必须取得修复自身Windows完整SUCCESS。

R10-01 Review Package Workflow 已实施、`VERIFYING`；唯一阶段分支 `rewrite/r10-review-analytics-release-backend`，从 R9 收尾 `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立。Application workflow 独立持有编排、身份门禁与人工确认元数据，ReviewService 五入口已切换；PG 十个工作流方法迁入对应适配器，SQL、审计及事务正文逐字保持，旧两 owner 删除。原 Domain 八状态/九步界面、171 命令、Ports/DTO/错误/模型保护区保持；不自动接受候选或晋升参数。新增 8 项原 Application unit 待自身 Windows，预期 Application146 / Persistence179 / Gateway112 unit、41 contract。84/84 既有源码门禁、完整架构、Rustfmt、等价复核与 8 破坏探针通过；CI 开始后停止查询。R10 IN_PROGRESS，02～13 BLOCKED，真实 PG/历史四项/账本并发回滚/有效 XLSX/Windows Full/私有 P4/P7 固定回归及继承 model.runs/0041 删除风险仍最终封包新库待验。 完整实际文件清单、边界与回退见 [01实施记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-01-review-package-workflow.md)。

R10-02 Structured Match Facts 已实施 `VERIFYING`，基线ab43aee，继续唯一分支rewrite/r10-review-analytics-release-backend。Application真实事实阶段迁入facts coordinator，ReviewService与组合工作流直接复用；PG facts目录独占校验、事件身份、事务写入、查询/摘要、行映射和事实提交，旧review.rs的14职责及事实写入正文移除。原比赛锁、修订归属、SQL/审计/同事务上下文与十个公开方法签名保持，不改变报告/候选/参数人工门禁。新增8 PG unit待自身Windows，预期Persistence187 / Application146 / Gateway112 unit、41 contract；84/84静态、完整架构、19 Rust文件Rustfmt、等价与8破坏探针通过。01自身Windows已DONE，02 VERIFYING、03～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7回归与继承model.runs/0041删除风险仍最终新库待验。 详见 [02实施记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-02-structured-match-facts.md)。

R10-03 Review Engine Adapter 当前 `DONE`：普通/资料包生成及PG准备、上下文、纯引擎调用和报告事务已迁入唯一owner，原公式/公开契约及候选人工门禁保持。精确 `99b15ad` / [Windows run38259438025](https://github.com/uniquenesssta/123/actions/runs/38259438025) 全SUCCESS，Application151/Persistence187、Gateway112 unit/41 contract、新增5项及17视口/打包/启动通过。01～08 DONE，09 READY、10～13 BLOCKED，R10 IN_PROGRESS；真实PG/Full/私有固定回归及继承删除风险仍最终新库待验。完整文件、证据与回退见 [03记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md) 和 [阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。

## 公开边界

- 保留 `crates/model-api`、模型 ID、路由、规则包入口、预测页面和历史数据结构。
- 使用 `crates/model-stub` 注册外部模型入口。未接入 ModelProvider 时，预测明确返回“运行时未分发”，不会静默回退、生成伪结果或使用隐藏默认参数。
- 公开规则包只保存外部提供器标识与通用输入输出契约；参数生成、校准、晋升和真实运行由私有或独立部署的 ModelProvider 负责。
- 私有资产由 `.gitignore` 和 `scripts/verify-public-model-boundary.mjs` 双重阻断。

## 构建与验证

```powershell
npm run setup
npm run verify:frontend
npm run verify:rust
node scripts/verify_protected_assets.mjs
node scripts/verify_command_contract.mjs
node scripts/verify_database_baseline.mjs
npm run tauri:dev
```

PostgreSQL 数据库基线必须使用名称包含 `test` 的专用、允许彻底清空的测试数据库：

```powershell
$env:FOOTBALL_TEST_DATABASE_URL="postgres://user:password@127.0.0.1:5432/football_test"
node scripts/run_database_baseline.mjs
```

Windows 可使用：

```text
启动平台.bat
验证平台.bat
验收平台.bat
```

`verify:frontend` 包含公开模型边界、Domain 类型清单漂移、Node 调用链兼容、Windows 路径契约、TypeScript、静态契约、截图和 Vite 生产构建。TypeScript 与 Vite 使用当前 Node 执行包内 JavaScript CLI，不直接启动 Windows `.cmd` 包装器。Windows 验收器从 `.cargo/target-location.json` 解析实际 Cargo target，并支持相对于项目根目录的 `LogDirectory`；应用 runtime 日志写入运行时根目录的 `logs`，开发态 runtime root discovery 可能解析为源码根目录上一级。`verify:architecture` 包含模块边界、状态所有权、受保护导入和 Domain 类型清单漂移门禁。`verify:rust` 包含 Cargo.lock 一致性、格式检查、Clippy 与工作区测试。`verify_protected_assets.mjs` 校验模型公开边界文件指纹、保护目录精确集合以及私有 P4/P7 资产缺席状态。`verify_command_contract.mjs` 校验前端调用、Rust 命令定义和 `generate_handler!` 注册集合一致，并拒绝缺失、重复、孤立或未授权动态命令。`verify_database_baseline.mjs` 校验 0001–0046 迁移连续性、内容指纹、SQLx 迁移入口、PostgreSQL 集成测试集合和关键不可变约束。`run_database_baseline.mjs` 在静态门禁通过后依次执行已有 Match Catalog 契约与 PostgreSQL 集成测试，并拒绝数据库名不含 `test` 的连接。
`Public Platform CI` 是 Windows 自动交付门禁：对 `main`、`new-*`、`rewrite/**` 的推送、Pull Request 和手动触发执行架构契约检查及 `scripts/windows-acceptance.ps1 -Mode Automated`，并保存验收日志和 release bundle 证据。云端 Automated 不替代最终真实 PostgreSQL、Windows Full 交互和用户本机验收。

Node 开发依赖固定安装和读取自源码根目录上一级的 `../node_modules`，npm 缓存固定使用 `../.npm-cache`；仓库根目录不再保存 Node 依赖目录。Cargo target 继续使用 `../.cargo-target`。

## 模块化重写执行记录

### Figma 设计接入任务书补充（2026-10-08）

- [总纲第 8.0 节](docs/football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md#80-figma-设计接入依据2026-10-08-补充)纳入 `ui-design-system` 分支 `docs/` 的设计依据与可追溯基线 `460eb0d`：已确认 Figma 决定新视觉，源码/契约提供功能和数据语义。R12 承接现有组件与 Patterns，R13～R15 补齐页面到节点的映射；设计路由不机械等同代码模块。
- S01～S13 的 13 / 17 路由仅为视觉、代表性状态及有限原型验收。未开始的 S14 配置、S15 数据库、S16 日志、S17 架构分别登记在对应 UI 节点，不阻塞 R8～R11 和 R12 基础工作；R17 沿用现有 Windows fixtures/截图/交互验收，保留真实运行时和最终新库待验边界。
- 本次仅补充总纲、R12～R15/R17 任务书与本记录；未实施 UI，R8-06 仍 READY。文档差异、41 个设计文件引用、6 个本地链接/锚点及节点编号/状态检查通过；复用原源码验证证据，纯文档提交标记 `[skip ci]` 避免重复构建，不修改 CI workflow、数据库入口或依赖。

### R7-15 Row 与 Subrecord Identity（2026-10-01，DONE）

- 首次 Windows run `36866936459` 在 Clippy `-D warnings` 因迁移后原 `team_package/identity.rs` 遗留未使用的 `PersistenceError` 导入失败；架构、前端类型/构建与 17 视口已通过，Rust tests/打包/启动未执行完成。已删除无调用导入并同步 Domain 使用指纹，直接受影响的原门禁、源码格式通过；无业务改动或警告抑制，修订 `a928c8b` / Windows run `36871154039` 已全通过，该 CI 失败已关闭。

- 将 8 个原重复球队行合并函数与 5 个原 inline 测试迁入共享 `workbooks/identity/teams`，提交及来源关联切换唯一职责；正常合并、名称/来源、显式总览优先、国家/类型隔离及原事务保持。`identity/row` 保留行 UUID/批次/工作表/物理行/实体定位，子记录键继续由原数据库生成列和 UNIQUE 判定，Rust 不重复计算或误去重。
- 修复未经检查的行号整数转换：三类预检在同源查找/取消和暂存之前拒绝不能按原 >=2/integer 约束落库的行号，避免异常输入取消旧 pending 批次后才失败；原载荷/行定位、账本及事实/审计/回滚边界保持。
- 新增原生产模块 inline 3 项，Windows Persistence 135/Application 55 已实际通过；原 PG 子记录用例扩同物理行多实体/维度/标签/双球队、跨行/表、UUID 读回、重复键回滚、三类前检与旧批次保留。83 项现有源码、架构及 Rustfmt 通过，六项破坏探针拒绝并恢复；本项精确 Windows run `36871154039` 的 17 视口、Rust/前端、release/MSI/NSIS/启动全通过。R7-01～15 均 DONE，见 [15 完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-15-row-and-subrecord-identity.md) 与 [R7 阶段完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-stage-completion.md)；PG/历史四项/账本/有效 XLSX/Full 最终新库待验，未宣称最终封包验收通过；该 R7 收尾时 R8 尚未启动；当前状态见上方 R8 索引。无新 target/runner/workflow/数据库/依赖或迁移。

### R7-14 Match Lineup Workbook（2026-10-01，DONE）

- 将 48 个原函数按导出、AI 上下文、预检、冲突、提交、事实写入、身份、校验与载荷迁入唯一 `adapters/workbooks/match_lineup/`；旧根 owner/注册删除，五个公开签名与全部原函数体、SQL/字段映射保持，原 Port 六入口显式分派。
- 原账本/两阶段冲突裁决、唯一整批事务、共同比赛锁、历史替代与角色继承、阵容门禁/计数/审计、回滚与重复提交拒绝保持。原 pair 要求成对，工作簿合法主队替换继续支持；模型截止与 actual 隔离继续复用原模型链。导出/AI 包保留原活动历史、引用时间、去重、角色和 cutoff。
- 现有 PG pair/workbook 夹具补只读快照、导出/AI 投影、空/未知比赛、预检只暂存及行身份、错误主客身份阻断；原回滚/裁决/并发回归保留。五个现有 verifier、职责/SQL/事务门禁与清单更新，83 项源码/架构/Rustfmt 通过，六项破坏探针拒绝并恢复；精确 `bb00120` / [Windows run 36860224760](https://github.com/uniquenesssta/123/actions/runs/36860224760) 已全通过，Persistence 132/Application 55、17 视口、构建/打包/启动通过，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-14-match-lineup-workbook.md)；14 DONE，15 已启动。PG/XLSX/Full 最终封包新库待验，无新 target/runner/workflow/数据库或依赖/迁移。

### R7-13 Monthly Workbook（2026-10-01，DONE）

- 剩余球队聚合与两类缺口从 monthly_workbooks.rs 迁入唯一 monthly_team/read、monthly_team/gaps、monthly_player/gaps，球员共享月度导出从 player_catalog/export 迁至 monthly_player/read；共享缺口映射在 monthly_gaps。旧根文件/注册和旧导出位置删除，无转发壳。五个原函数体、四个公开签名和 18 条查询/映射保持；默认值、连接、排序、历史/未来记录、当前关系包含边界及 90/120 天缺口判定不变，导入继续复用 11/12。
- 两类原 Port 导出/缺口调用显式分派；原 PG 月度夹具增加空资料默认/空时间、clear 值、元数据、名称/任期/阵型/能力聚合、只读快照、当前关系日期/90 天观察/未来资料非负天数、上月/下月及同一球员多球队履历断言。不新增镜像单测或测试目标；现有 inline 数量仍 132/55，本项精确 `78138db` / [Windows run 36854202029](https://github.com/uniquenesssta/123/actions/runs/36854202029) 已全通过，Persistence 132 项/Application 55 项；见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-13-monthly-workbook.md)。真实 PG/XLSX/Full 最终封包新库待验。
- 原月度/Exchange/资料包/管理门禁路径切到实际 owner；月度门禁增加唯一职责/旧 owner 删除/显式分派/无写入或写锁/共享缺口映射/原 SQL 集合与编排检查。83 项现有源码/契约、架构及 Rustfmt 通过，六项破坏探针拒绝并恢复。直接模块 36，Domain 365/300 及声明摘要保持，使用清单/原 PG 指纹刷新；工作流/依赖/锁文件/迁移/模型保护区冻结。13 DONE，14 已启动，15 BLOCKED。

### R7-12 Team Package（2026-10-01，DONE）

- 将资料包和旧球队月度共用的真实球队链迁入唯一 `adapters/workbooks/team_package/`，按预检、裁决、提交、事实写入、名称、身份合并/关联、阵型、校验、载荷划分职责。批次/行 SQL 和严格 Team codec 归既有 batch_ledger，MonthlyWorkbookPort 四入口显式分派；95 个原函数体和 7 个公开签名保持。`monthly_workbooks.rs` 剩余三项月度数据/缺口读取留给 13，无重复导入框架。
- 修复球队人工 skip/候选裁决后 pending skipped/error 计数未同步的问题，行与计数在同一父锁事务提交。预检只写暂存，提交唯一事务内规范化、合并重复球队、绑定关联、写事实/行/计数/原审计。保留原文名/中文主名/简称和覆盖率；完整包保持原“球队成功后提交球员，球员失败保留球队并可重试”双链边界。
- 原 22 个 inline 测试迁至实际职责，新增 Persistence 5 项/Application 2 项，预期 Windows 132/55 项，尚未执行。原 PG 月度夹具补重复预检身份、末行失败整条球队链回滚/原批次恢复、一次审计、三类名称、非法候选拒绝及裁决计数；沿用最终封包新库待验。83 项现有源码/契约门禁、架构聚合及 Rustfmt 通过，六项破坏探针拒绝并恢复；精确 `fe2f7eb` 的 [Windows CI run 36831302483](https://github.com/uniquenesssta/123/actions/runs/36831302483) 全通过，Persistence 132 项/Application 55 项，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-12-team-package.md)；13 已启动，14～15 BLOCKED。无新测试目标、runner、workflow、依赖、迁移或数据库设施。

### R7-11 Player Workbook（2026-10-01，DONE）

- 删除旧根 `spreadsheet_exchange.rs`，59 个生产函数按预览、冲突裁决、提交、导出、身份匹配、字段校验、载荷规范化迁入唯一 `adapters/workbooks/player_catalog/`；原函数体和 8 个内联测试保持，SpreadsheetExchangePort 六入口显式分派。复用 R7-10 账本和共享外部 ID 写入，保留候选边界、同源 ID 不改绑、月度/资料包调用、物理行及子记录身份、依赖顺序、幂等/clear/整批回滚与共同事务审计。
- 增加 6 个现有生产文件内联测试，覆盖重名候选/模式/动作、外部 ID 已绑和新目标、包内延迟关联及载荷解析边界；原 PG 月度用例补重复只读预览的 UUID/工作表/物理行/载荷、无业务写入和无提交审计。原多效力期、外部 ID 竞争/末行回滚/重复导入回归保留；真实 PG/XLSX/Full 最终封包新库待验。
- 现有导入/资料包/角色/身份验证器切换实际职责；Exchange 加唯一 owner、显式分派、预检只暂存、冲突父锁/候选/改绑拒绝/计数审计共同事务检查。模块清单直接模块 37，Domain 365 声明/300 映射分类保持，使用摘要和既有 PG 指纹刷新；无新测试目标、runner、workflow、依赖或迁移。83 项现有源码/契约检查、架构聚合与 Rustfmt 源码检查通过，六项破坏探针拒绝并恢复；精确提交 `310c46b` 的 [Windows CI run 36824027121](https://github.com/uniquenesssta/123/actions/runs/36824027121) 全通过，Persistence 127 项/Application 53 项通过，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-11-player-workbook.md)；12 已启动，13～15 BLOCKED。

### R7-10 Spreadsheet Batch Ledger（2026-10-01，DONE）

- 两类导入账本唯一职责、人工冲突解决后的 pending 计数同步与调用方共同事务已通过精确提交 `cc0d34d` 的 [Windows CI run 36815568401](https://github.com/uniquenesssta/123/actions/runs/36815568401)。Persistence 121 项/Application 53 项、前端类型/17 视口/构建、fmt/Clippy/workspace tests、安装包/启动全部通过；见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-10-spreadsheet-batch-ledger.md)。18 broad PG 仍 ignored，历史四项/账本/有效 XLSX/Full 最终封包新库待验。

### R7-08 Lineup Chain / History（2026-10-01，VERIFYING）

- 按时点选择/版本链/窗口/门禁迁入唯一 `adapters/lineups/chain/`，阵容列表/详情/球队历史/映射/历史变更迁入 `history/`；旧根 chain 删除，player_catalog 仅保留引用数据聚合。创建、工作簿、预测和 Application Port 调用方已切换，正常 API/错误/角色时点、actual 隔离及 1..=200 列表语义保持。
- 历史删除/恢复先锁比赛再锁后重读版本，与创建/导入按相同顺序串行；引用保护、隐藏/物理删除、前驱恢复和成功审计仍单事务。现有 chain/pair 测试补等时/一微秒边界、优先级、稳定排序/limit、隐藏追溯和删除/创建父锁并发，原窗口/枚举单测迁移并补边界断言；无新测试目标、runner、workflow 或数据库设施。
- 原架构/角色/链门禁更新并增强，五项破坏探针被拒绝并恢复，聚合及受影响静态检查通过；Domain 使用路径及 integration 指纹同步，365 声明与 300 项映射分类保持，模型/依赖/锁文件/迁移保持。本项 Windows CI 待验，真实 PG/XLSX/Full 仍最终封包新库待验；R7-09 及以后 BLOCKED，实施与回退见阶段索引。
- 首轮 Windows CI run `36764662888` 失败于阵型验证器旧路径；同类复查修订历史删除旧路径及链验证器对 Rustfmt 换行的误拒绝。阵型检查分别验证当前 read/mapping 的 ID/代码/名称关联，原历史/窗口契约保持；四项破坏探针仍拒绝缺失映射、历史恢复与 String 转换。修订只涉及原验证器和记录，业务源码不变；83 个源码/契约检查与架构聚合通过，5 个浏览器交互/截图及 Windows 编译/构建/打包待新 CI。

### R7-07 Lineup Pair Transaction（2026-10-01，DONE）

- 单侧/双方创建唯一 owner 迁入 `adapters/lineups/pair_transaction/`，结构校验与事务写入按职责共置，旧创建/私有写入实现删除。读取/历史留给 R7-08；原角色规则、时间模型门禁、actual 隔离、公开接口/错误保持。
- 单侧和双方在 READ COMMITTED 下共用比赛 FOR UPDATE 锁，工作簿修改同一版本链时复用该锁与原批次事务；业务、明细及单侧/双方审计共同提交，保留 R7-06 ended_previous_count。Application Port 显式分派到原公开入口；commit 后读回语义保持。
- 精确提交 `3ff21cb` 的 [Windows CI run 36757339619](https://github.com/uniquenesssta/123/actions/runs/36757339619) 全通过，Persistence 102 项/Application 53 项全部通过；见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-07-lineup-pair-transaction.md)。真实 PG 并发/取消/回滚、有效 XLSX/Full 仍最终封包新库待验。

### R7-06 历史数据库失败与账本问题收口（2026-10-01，DONE）

- 修订原 chain 的合法 11 人/缺阵型及 actual 隔离夹具、pair 的有效 T-6h 窗口、events 的完整 MatchResultRecord、P4 亚微秒比较与首建返回时间。PostgreSQL 精度下精确比较，不使用容差，原输入指纹保持；保留冻结不可变/幂等断言，补齐原四概率链夹具。
- 导入在原事务持久化 ended_previous_count；现有 pair 测试复用真实 preview/commit，覆盖末行失败整批回滚、恢复后业务/账本/审计一致及成功批次重复拒绝无额外效果。18 项 broad 测试集合不变，不新建 target/runner/workflow/库。
- 原验证器扩充并经三项破坏探针确认；架构及受影响静态门禁通过，runtime baseline 仅同步合法变更的 integration 指纹，365 类型不变。精确提交 `4496399` 的 [Windows CI run 36734194083](https://github.com/uniquenesssta/123/actions/runs/36734194083) 全通过，Persistence 99 项/Application 53 项实际通过，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-06-database-contract-closeout.md)。四项历史失败和账本仍“已修改、最终封包新库待验”，不记实跑关闭；R7-07 DONE，R7-08 VERIFYING，R7-09 及以后 BLOCKED。

### R7-05 关键 Application 用例验证（2026-09-30，DONE）

- 在现有 Prediction/P4 orchestration 单测和 Exchange 原用例内新增 20 个行为测试，验证真实编排的调用顺序、参数、正式/影子历史写入差异、错误阻断、取消/失败终态只读、已有快照恢复与重复执行、研究入队/状态登记失败的幂等键重试、损坏输入及导出失败副作用边界。
- 共享 fake 只在 cfg(test) 构建可见，未选中 Port 调用立即失败；模型 fake 只验证编排，真实公开 provider 缺席仍断言返回错误且不保存成功历史。队列 fake 不代替数据库 exactly-once 或有效 XLSX 往返验收。
- 生产逻辑、公开接口、Port/DTO、依赖/锁文件、迁移、模型资产与工作流保持；无新测试文件/target/框架/专项。原 Domain inventory 更新测试调用面，365 类型/声明摘要不变。架构及受影响静态检查通过，精确提交 `02e56bad` 的 [Windows CI run 36712015030](https://github.com/uniquenesssta/123/actions/runs/36712015030) 全通过，Application 实际 53 项全部通过；PG/XLSX/Full 仍最终封包新库待验。证据与覆盖边界见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-05-application-use-case-tests.md)。R7-06 DONE，R7-07 DONE，R7-08 VERIFYING，R7-09 及以后 BLOCKED。

### R7-04 架构清单、验证器与执行记录对齐（2026-09-30，DONE）

- 原 Ports 验证器递归发现全部公开声明并与清单双向核对，补齐 5 个遗漏；19 个 Port 文件均检查禁止依赖，当前 43 个 trait 为源码扫描结果。sourceScan 验证当前文件/摘要/数量，原 209/232 调用面统计明确保留为 R3 历史基线。Analytics 私有 JSON 转换与 Review Package 既有 read_source_run JSON 返回采用明确 owner/指纹兼容登记，阻止未经登记的扩张。
- PostgreSQL 模块清单由旧根模块名更新为 lib.rs/adapters/mod.rs 直接登记的 38 个真实 owner，拒绝缺失、额外、重复和失效路径；补接 R6-03 至现有 architecture/frontend，保留 R5 间接导入。R3-06/07 以原阶段 README 对应章节为权威详细记录，R6-09 复检表述订正并链接 R7-03 修复证据。
- 架构、受影响静态契约及 11 项反向检查通过；精确提交 `9bdb849` 的 [Windows CI run 36706905645](https://github.com/uniquenesssta/123/actions/runs/36706905645) 全通过，已正式 DONE，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-04-architecture-inventory.md)。没有改业务 Rust/公开协议/迁移/依赖/保护资产或新增 runner/workflow；真实 PG/XLSX/Full 继续最终封包新库待验。详情见 [阶段索引](docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md)。

### R7-03 普通删除与历史引用保护（2026-09-30，DONE）

- Player/Team 普通删除在同一 READ COMMITTED 事务中，取得 FOR UPDATE 锁后复用完整引用计数与裁决，再删除实体、外部 ID 并写审计。并发新增历史不会被事务外的过期预检放行；Team 两项局部复检改为完整复检，正常归档提示/bulk 结果和 force-delete 确认/墓碑保持。
- 扩充已有 permanent-delete contract：通过实际锁等待验证 dynamic tag/lineup preset 并发提交或回滚、实体/引用/外部 ID 保持、单次审计与重复删除。现有 verifier 防止删除复检、弱化锁或陈旧隔离级别回归；无新 runner/target/workflow，迁移/依赖/公开 DTO 不变。
- 首轮 Windows CI [run 36693857568](https://github.com/uniquenesssta/123/actions/runs/36693857568) 在现有球队/球员管理契约失败：验证器仍要求在删除写入文件直接出现比赛/复盘 SQL，未适配共享引用 owner。本次修复原验证器，检查 Team 事务复检→共享计数/裁决→拒绝引用链路及真实比赛/复盘 SQL，四项临时破坏均被拒绝。
- 修订提交 `073d155` 的 [Windows CI run 36695374298](https://github.com/uniquenesssta/123/actions/runs/36695374298) 全通过：架构、前端契约/类型/截图/构建、Rust fmt/Clippy/tests、Windows 打包及启动烟测完成，测试库名称 guard 实际通过。R7-03 正式 DONE，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-03-safe-entity-deletion.md)。真实 PG 删除/并发测试仍 ignored，PG/XLSX/Full 保留最终封包新库待验；R7-04 READY、尚未实施。

### R7-02 Windows 收尾与 R7-03 启动（2026-09-30）

- `1e4da04` 的 [Windows CI run 36686343584](https://github.com/uniquenesssta/123/actions/runs/36686343584) 全通过，R7-02 正式 DONE，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-02-external-id-integrity.md)。身份/并发/导入事务 PG 测试仍 ignored，真实 PG/XLSX/Full 保留最终封包新库待验。
- 用户已启动 R7-03，修复普通删除事务内完整引用裁决及并发保护；R7-04 及后续仍 BLOCKED。

### R7-02 外部 ID 身份保护（2026-09-30，VERIFYING）

- 公开添加与工作簿提交共用一个原子身份写入 owner：同 provider/type/external ID 只允许同实体合并 metadata，跨实体返回冲突并保留原绑定；导入失败沿用原事务整批回滚。
- 扩充现有 references contract，覆盖重试、竞争绑定、metadata 合并、命名空间隔离、预检后竞争与业务/批次/审计回滚；现有 verifier 防止恢复覆盖写入。没有新增框架、数据库入口或修改迁移/生产依赖。
- 架构聚合、受影响导入/保护资产/命令/迁移静态检查与反向门禁通过；Windows CI 待本次提交运行，真实 PG/XLSX/Full 留到最终封包新库。R7-03 仍 BLOCKED，详细结果见 [阶段索引](docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md)。

### R7-01 Windows 收尾与 R7-02 启动（2026-09-30）

- `c826dd3` 的 [Windows CI run 36678914535](https://github.com/uniquenesssta/123/actions/runs/36678914535) 全通过：架构、前端、Rust fmt/Clippy/tests、打包及启动烟测；R7-01 标记 DONE，见 [完成记录](docs/modular-rewrite/R07-match-lineup-workbook-persistence/R07-01-match-catalog.md)。
- PG/XLSX/Full 未实跑，继续登记最终封包新库待验。R7-02 按用户指令开始外部 ID 稳定身份整改，后续节点仍 BLOCKED。

### R7-01 Match Catalog 问题修复（2026-09-30）

- Application MatchCatalogPort 显式调用 Persistence 固有 `read_match`，移除已删除的方法调用并避免 trait 递归；自动赛季解析、层级校验、比赛 upsert 和返回映射共用一个事务，主客相同在事务前拒绝，后续失败回滚赛季新增/metadata 更新。
- 官方生成器同步 Domain 使用清单：365 个 Domain 类型/声明摘要保持，Rust 扫描 966→975。Lineups Service、Match Workflow、History Scoreline 验证器改读当前 owner，保留原保护断言；已有 R7 verifier 覆盖 Application/事务并接入架构和前端门禁。
- 已有 `match_catalog_repository_contract` 补 URL/实际数据库名双重 test 前检、失败断言后的本次夹具清理，以及 scope 冲突、赛季回滚、600 行列表上下限/状态/排序、研究引用阻止删除、AI/外部 ID 释放与单次删除审计。不可变共享 schema 和审计证据保留，不禁用触发器或清空全库。现有数据库基线与 Windows Full 接入该目标，不新增 runner/workflow；清除已完成迁移的两个一次性 R7 脚本。
- 已通过架构聚合、受影响静态契约、迁移/保护资产/命令检查和数据库入口 dry-run/反向门禁。源码按 Rust 1.88 整理格式；未执行 Windows 编译、Clippy、Rust tests、PostgreSQL 或真实 XLSX/Full，未宣称动态通过。当前 R7-01 仍 VERIFYING、R7-02 仍 BLOCKED，验证详情见阶段索引。

### Windows 验证范围与 R7 审计

- 2026-09-30 起仅要求 Windows 编译、测试、打包和运行验收；Linux/macOS 验证不再作为门禁。当前 CI 已是 Windows-only；真实 PostgreSQL/XLSX、事务与模型/迁移保护要求保留，详见总纲与 `docs/TESTING.md`。
- R7-01 已迁移 Match Catalog，但仍为 `VERIFYING`：存在 Application 读取旧方法、Domain 调用方清单未同步、三个历史验证器路径未迁移和专项数据库测试未接入的缺口。本次只更新验证范围与审计记录，未修复业务代码、刷新清单或改变验收状态。详细证据及建议见 [`R07 执行索引`](docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md)。



### R6 Stage Complete

- R6-01～R6-10 与阶段出口收尾均完成；Entity Catalog persistence 已收敛到 `adapters/catalog/`，旧 `entity_catalog.rs` 删除，`player_catalog.rs` 只保留 R7 Match/Lineup 职责。阶段记录：`docs/modular-rewrite/R06-entity-catalog-persistence/R06-stage-completion.md`。
- 最终生产代码 `809cfb429ec31c165616e65e1b6169f928ee4dcb` 经 run `32276040092` 验证：Windows job `96143592950` 官方 Automated acceptance 与 PostgreSQL 16 job `96143592734` 的 12 个 retained contracts 均 `SUCCESS`。公共契约、0001–0046 migrations、UI、模型保护资产与生产依赖保持兼容；R6 正式 `DONE`，R7 开放。

### R6-10 Global Name Search

- 统一名称搜索已收敛到 `adapters/catalog/global_search/{query,normalization,predicate}`；球队目录、球员目录、球队 selector、教练目录与 Entity Reference 只切换内部 owner，旧 `name_search.rs` 已删除。
- `football.global-name-search.v1` 的中文/英文/别名/重音/标点与空白无关、多关键词 AND、contains 与分页语义保持不变；公共 DTO、Schema、0001–0046 migrations、配置、UI、错误语义和生产依赖不变。production `fbe013b96f38d3c61e7b536e1ef67a89109c57c4` 的 minimum gate run `32249610140` 与 stage verification run `32249902897` 均通过；Windows job `96058363125`、PostgreSQL 16 job `96058363037` 均 `SUCCESS`，R6-10 正式 `DONE`。

### R6-09 Archive / Delete / Force Delete

- R6-09 deletion persistence 已按职责收敛：AT1 `preflight/archive`、AT2 `bulk_delete/safe_delete/delete_write`、AT3 `force_delete/{operation,targets,counts,execute}`；各协调器保持 SQL-free，引用读取、事务写入、强制清除目标闭包、影响统计与破坏性执行均由明确 I/O owner 承担。
- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、配置、错误语义、安全确认、历史 P4/运行引用保护、普通永久删除和 force purge 用户可观察行为均未改变；无新增生产依赖。AT1/AT2/AT3 均已独立验证并关闭；AT3 生产提交 `07a1d421a9903cc588b500aab2b9f6eb8d9a4bc7` 的 Minimum Gate、focused PostgreSQL 16 contracts、完整 Windows Stage Regression 与 R6-01～R6-09 retained PostgreSQL 16 contracts全部通过（recovery workflow run `32231046269`；Windows job `96000683932`、PostgreSQL job `96000683845` 均 `SUCCESS`）。R6-09 正式 `DONE`，R6-10 开放为 `READY`；用户现有 PostgreSQL 数据真实 sample/write 与 Windows Full 人工交互验收继续保留到最终统一验收。

### R6-08 Entity Matching 与 References

- Entity Matching persistence 已收敛到 `adapters/catalog/entity_matching/`，References persistence 已收敛到 `adapters/catalog/references/`；stable-ID/external-ID/name candidate read、reference directory、provider 与 external-ID writes 均按职责拆分，协调器保持 SQL-free。`entity_catalog.rs` 继续只持有 R6-09 deletion/archive/reference-count，`player_catalog.rs` 不再持有 provider/external-ID owner。
- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、数据格式、配置、错误语义、日志等级、统一 NameSearch、matching priority/candidate score/reason、历史 P4/运行引用、UI 行为和模型保护资产保持不变；无新增生产依赖。
- 生产 owner-switch 提交 `89dfbb101989116b5c858dbb8dc4564c50f86f9d` 的 implementation/minimum gate 已通过。首轮阶段 hard gate run `32142895608` 仅因 Ubuntu Chrome zygote 截图环境失败而停止，后续门禁未被误记为通过；最终 hard gate run `32143394153` 整体 `SUCCESS`，Windows job `95731216023` 已通过 frontend、rustfmt、workspace Clippy `-D warnings` 与 workspace tests，PostgreSQL 16 job `95731215987` 已通过 official inventory、R6 retained ownership、完整 architecture、模型保护/database baseline 及 R6-03～R6-08 真实 PostgreSQL contracts。临时 gate/closeout 文件已清理；R6-08 正式 `DONE`，R6-09 开放为 `READY`。用户现有 PostgreSQL 数据真实 sample/write 与 Windows Full 人工验收继续保留到最终统一验收。
### R6-07 Coaches 与 Formation Usage

- Coaches persistence 已收敛到 `adapters/catalog/coaches/{directory,detail,names,team_periods,mapping}/`；Formation persistence 已收敛到 `adapters/catalog/formations/{directory,usage,resolution}/`。Formation resolution/window/save 的 SQL 读取与写入进一步拆入明确 `read.rs` / `write.rs` owner，协调器保持 SQL-free；旧 `formation_catalog.rs` 删除，`entity_catalog.rs` 仅保留后续 R6-08/R6-09 职责。
- `CoachCatalogPort` / `FormationPort`、Domain DTO、Schema/0001–0046 migrations、配置、错误语义、Formation append-only/smoothing/UNKNOWN fallback/resolution priority、历史 P4/运行引用和用户可观察行为保持不变；无新增生产依赖。Global Name Search / Entity Relationships / Formation Usage 历史 verifier 仅跟随新 owner，原断言未删除或放宽。
- Windows 本地 Minimum Gate 与 Stage Regression 已实际通过完整 frontend、TypeScript/Vite、rustfmt、workspace Clippy `-D warnings`、workspace tests、95/95 Persistence unit tests、完整 architecture 与模型保护。R6-07 hard-gate run `32125684434` 除额外 full frozen PostgreSQL baseline 外其余 R6-07/retained contracts、Clippy/tests 均通过；focused PostgreSQL final gate run `32128507369` / job `95684279881` 为 `SUCCESS`。额外 full baseline 在当前 R6-07 tree run `32127380673` 与节点起点 reference run `32127587771` 均出现相同 14/18 PASS、4 FAIL，确认不是本节点回归，未通过放宽测试掩盖。R6-07 正式 `DONE`，R6-08 Entity Matching 与 References 开放为 `READY`；用户现有 PostgreSQL 数据真实 sample/write 与 Windows Full 人工交互验收继续留到最终统一验收。

### R6-06 Abilities 与 Dynamic Tags

- Abilities persistence 已拆入 `adapters/catalog/abilities/{dimensions,observations}/`；Dynamic Tags 已拆入 `adapters/catalog/dynamic_tags/{definitions,tags,contribution}/`，旧 `dynamic_tags.rs` 单文件 owner 删除，Player Detail ability read 复用统一 typed Row/mapper。
- `PlayerSignalPort`、Domain DTO、Schema、0001–0046 migration、配置、错误语义、战术角色/位置映射、模型保护资产和生产依赖保持不变。
- implementation gate run `32043516067` 已实际通过 R6-06 ownership、retained R6 ownership、完整 architecture、模型保护、数据库基线、rustfmt、Persistence unit tests、Application check 与 PostgreSQL 16 contract；最终阶段 hard gate run `32044944449` 整体 `SUCCESS`：Windows job `95430654653` 已通过完整 frontend、rustfmt、workspace Clippy `-D warnings` 与 workspace tests，PostgreSQL/architecture job `95430654659` 已通过 R6 retained ownership、完整 architecture、模型保护/数据库基线及 R6-03～R6-06 PostgreSQL 16 contracts。历史角色继承 verifier 仅将已删除 `dynamic_tags.rs` 的检查跟随到 `dynamic_tags/contribution/{calculate,scoring}.rs` 权威 owner，原断言未删除或放宽。clean canonical Public Platform CI run `32046081931` / Windows job `95434232104` 已在 HEAD `9fe61a55627a51862670624d53d3feec688d1973` 上整体 `SUCCESS`；artifact `9293689776`，SHA-256 `1c84dfa91806ef66cf632e8ce952fdc2dce5601f27b2eae521917e1b268aa5e1`。R6-06 正式关闭为 `DONE`，R6-07 Coaches 与 Formation Usage 开放为 `READY`；用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收继续保留到最终统一验收。


### R6-05 Team Periods 与 Availability

- Player Team Periods / Availability 写职责已从 `crates/persistence-postgres/src/player_catalog.rs` 收敛到 `adapters/catalog/players/team_periods/` 与 `adapters/catalog/availability/`；输入策略、typed Row、Domain mapper 与 SQL owner 分责，Player Detail 读取继续复用相同 Row/mapper owner。
- `PlayerCatalogPort::add_player_team_period`、`PlayerSignalPort::add_availability`、Domain DTO、Schema、0001–0046 migration、错误语义、历史 P4/运行引用和用户可观察行为保持不变；未新增生产依赖。
- 临时实施 gate run `32026919056` 已通过 R6-05 ownership、R6-03 retained ownership、完整 architecture、模型保护/数据库基线、rustfmt、Persistence unit tests、R6-05 PostgreSQL 16 contract 真实执行及 Application check，随后生成生产提交 `6925455224973b08ab697605db182135f58ef1dd`。
- 阶段 hard gate 首轮 run `32027350874` 仅因临时 Windows job 未安装声明的 Node 开发依赖而在 frontend gate fail-fast；补齐既有 `npm run setup` 环境后，最终 run `32027535476` 整体 `SUCCESS`：Windows job `95379993915` 的 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests 全部通过；PostgreSQL/architecture job `95379993984` 的 R6 ownership、完整 architecture、模型保护/数据库迁移兼容门禁及 R6-03/R6-04/R6-05 PostgreSQL 16 contracts 全部通过。临时 workflow 已清理；clean canonical Public Platform CI run `32028645837` / Windows job `95383445833` 为 `SUCCESS`，artifact `9288762564`，SHA-256 `ee8f511b55c03b70c38f77c21937195fa228ca0f318c88837e7cf774c746b644`。R6-05 已正式关闭为 `DONE`，R6-06 Abilities 与 Dynamic Tags 开放为 `READY`；用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收继续保留到最终统一验收。

### R6-04 Player Names 与 Positions

- Player Names / Positions 写职责已从 `crates/persistence-postgres/src/player_catalog.rs` 收敛到 `adapters/catalog/players/{names,positions}/`，输入策略、typed Row、Domain mapper 与 SQL/事务 owner 分责；Player Detail 读取继续保留在 `detail/`，并复用新的 Names/Positions Row/mapper owner。
- `PlayerCatalogPort`、Domain DTO、Schema、0001–0046 migration、错误语义、默认战术角色/位置映射和用户可观察行为保持不变；未新增生产依赖。
- 临时实施 gate run `31969378628` 已实际通过 R6-04 ownership verifier、official Domain inventory 生成与校验、完整 architecture、rustfmt、Persistence unit tests、R6-04 PostgreSQL contract 编译及 Application check。
- 阶段 hard gate run `31969619201` 已整体 `SUCCESS`：Windows job `95219904073` 的 `npm run verify:frontend`、rustfmt、workspace Clippy `-D warnings` 与 workspace tests 全部通过；PostgreSQL/architecture job `95219904082` 的 R6-01～R6-04 ownership、完整 architecture、模型保护/数据库迁移兼容门禁、R6-03 retained PG16 contract 与 R6-04 PG16 contract 全部通过。PR #35 fixed head `9d979d627d3e2495788b4e6873ff4293f191a9d4` 的 clean canonical run `31970247969` / Windows job `95221509718` 为 `SUCCESS`；PR 已 squash merge 为 `40f83e72deabf6cdbd927d3460926f3419e9097d`。merged-stage canonical run `32019769202` / Windows job `95356807701` 为 `SUCCESS`；artifact `9285658232`，SHA-256 `0a01715e1bfe6ca2bf092e6f00c1c1e4847e74b7475cd7832f39ebd9882bc58f`。R6-04 已正式关闭为 `DONE`，R6-05 Team Periods 与 Availability 开放为 `READY`。用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收仍未执行。


### R6-03 Player Directory 与 Detail

- Player `create/update/list/detail` persistence 已从旧 `player_catalog.rs` 收敛到 `crates/persistence-postgres/src/adapters/catalog/players/{directory,detail,record}/`；Directory、Detail coordinator、单用途读取模块、typed Row 与 Domain Mapper 已分责，旧 owner 不再承担这四项生产职责。
- Dynamic Tag 保持既有独立 owner `crates/persistence-postgres/src/dynamic_tags.rs`；R6-04 Names/Positions writes、R6-05 Team Periods/Availability writes、R6-06 Abilities/Dynamic Tags writes 与 R6-09 deletion 未提前迁移。公共 `PlayerCatalogPort`、Tauri command/DTO、Domain shape、Schema、0001–0046 migration、配置、错误语义、用户可观察行为、模型保护资产与生产依赖保持不变。
- 旧 owner PostgreSQL 16 contract run `31929515802` 为 `SUCCESS`；Player Directory owner-switch run `31929916934` 与 Player Detail owner-switch run `31930614713` 均为 `SUCCESS`。official Domain inventory refresh run `31932154320` 为 `SUCCESS`，只更新 `architecture/domain-type-inventory.json`。
- legacy verifier 在 Global Name Search、Player Role Inheritance、Database Reset、Entity Resource Center、Stage E1 Follow-up 上依次发现 owner-path 漂移；均只将原有契约检查跟随到真实新 owner，未删除、跳过或放宽门禁。
- 最终 stage hard gate run `31932648476`：Windows job `95129707185` 与 PostgreSQL/architecture job `95129707240` 均 `SUCCESS`；`npm run verify:frontend`、`cargo fmt --all -- --check`、workspace Clippy `-D warnings`、workspace tests、R6 ownership、完整 architecture、模型保护、database baseline 与 PostgreSQL 16 contract 全部实际通过。临时 hard-gate 与 inventory refresh workflow 已清理。
- clean PR canonical run `31933320602` / Windows job `95131335957` 为 `SUCCESS`；artifact `9260226890`，SHA-256 `46c628502db8d9a1bd86a1459c8c7d955b13b7118eed950d292ee85f0d6bf01d`。PR #34 fixed head `b00d5cb3144bdb634b0d8f48343e7a0e3f897f5e` 已 squash merge 为 `8f285d2b3b539d0fdcfa6c02b1ee0fc801567786`。merged-stage canonical run `31937248702` / Windows job `95140999446` 为 `SUCCESS`；artifact `9261315055`，SHA-256 `38bae9097b67cf743a749b90a659d70e569ce01ee1de34f563d865b91ea236c0`。R6-03 已正式关闭为 `DONE`，R6-04 Player Names 与 Positions 开放为 `READY`。用户现有 PostgreSQL 数据库真实数据写入/sample 验收与 Windows Full 人工交互验收未执行。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-03-player-directory-and-detail.md`。

### R6-02 Team Names 与 Profiles

- Team Names 与 Team Profiles 写入已从旧 `team_catalog.rs` 收敛到 `crates/persistence-postgres/src/adapters/catalog/teams/{names,profiles}/`；validation/policy、typed Row、Domain Mapper 与 SQL/transaction owner 已分责，旧重复 helper/实现已删除。
- Team 名称规范化收敛到 `names/normalization.rs` 唯一 owner，R6-01 create/update 复用该规则；R6-09 删除职责继续留在 `team_catalog.rs`，未提前跨节点迁移。
- 公共 `TeamCatalogPort`、Tauri command/DTO、Domain shape、Schema、0001–0046 migration、配置、错误文案、审计语义、用户可观察行为、模型保护资产与生产依赖保持不变；Profile head-coach 投影保护与 metadata merge 保持原语义。
- 旧 owner PostgreSQL 16 baseline run `31897078340` / job `95041920033` 为 `SUCCESS`；owner-switch run `31897403363` / job `95042724731` 为 `SUCCESS`。
- 最终阶段 hard gate run `31897727309`：Windows job `95043538718` 与 PostgreSQL/architecture job `95043538723` 均 `SUCCESS`，覆盖 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、完整 architecture、模型保护、database baseline 与真实 PG16 contract。
- clean PR canonical run `31898334326` / Windows job `95044999641` 为 `SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。PR #33 已按 fixed head `f04923425f9d660f2ce1275da2b689ca681f3500` squash merge 为 `334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。merged-stage canonical run `31925219003` / Windows job `95111615032` 为 `SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。R6-02 已正式关闭为 `DONE`，R6-03 Player Directory 与 Detail 开放为 `READY`。未对用户现有 PostgreSQL 数据库执行写入/真实数据 sample 验收，也未宣称 Windows Full 人工交互验收完成。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md`。


### R6-01 Team Directory 与 Detail

- Team `create/update/list/options/detail` persistence 已从旧 `player_catalog.rs` / `team_catalog.rs` 收敛到 `crates/persistence-postgres/src/adapters/catalog/teams/{directory,detail}/`；directory、detail coordinator、单用途 SQL reader、typed Row 与 Domain Mapper 已分责。
- R6-02 Team Names/Profile writes 与 R6-09 deletion 继续保留原 owner，没有提前跨节点迁移；Application `TeamCatalogPort`、Tauri command/DTO、Schema、0001–0046 migration、配置、错误语义、用户可观察行为、模型保护资产与生产依赖保持不变。
- 旧 owner PostgreSQL 16 contract run `31888078703` / job `95020173411` 为 `SUCCESS`；最终 owner switch run `31888483586` / job `95021118552` 为 `SUCCESS`，生产切换提交 `5056917bd5c0e81aaaa96d1ef70c7362ef874820`。
- ownership/inventory run `31888800801` / job `95021851053` 为 `SUCCESS`：R6-01 ownership、official Domain inventory、模型保护、完整 architecture、rustfmt、Persistence/Application check、Persistence unit tests 与同一 PostgreSQL contract 全部通过；verified HEAD `b2999e5dce982bddf2093f0596eec33a7dbd0333`。
- 阶段级 hard gate run `31889555412` 已通过：Windows job `95023694944` 的 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 Ubuntu job `95023694946` 的完整 architecture、模型保护、database baseline freeze、PostgreSQL 16 contract 均 `SUCCESS`。
- clean PR canonical run `31890099520` 为 `SUCCESS`；PR #32 已 squash merge 为 `a54df5ca2695297ea5866a3efd74643239568825`。merged-stage canonical run `31892105125` / Windows job `95029805339` 为 `SUCCESS`，artifact `9249127034`，SHA-256 `972be4e32272e700da265f6e82ce96bc5e56aa356491bb217d6a1076e1b00087`。
- R6-01 已正式关闭为 `DONE`；未对用户现有 PostgreSQL 数据库执行写入/真实数据 sample 验收，也未宣称 Windows Full 人工交互验收已完成。R6-02 Team Names 与 Profiles 已开放为 `READY`。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-01-team-directory-and-detail.md`。


### R5-06 Model Run Identity Reads

- Model Run identity read 已收敛到 `crates/persistence-postgres/src/adapters/competition/model_run_identity/`，并按 typed Row、record mapper、单次 identity SELECT 拆分；`model_runs.rs::read_run` 不再直接拥有模型版本、参数版本、规则包与 binding identity JOIN。
- R5-05 后残留在 legacy `routing.rs` 的 `ModelRegistration`、`register_model` / `register_model_in_tx` 及 definition/version/parameter-set SQL 已同步迁入 `model_run_identity/registration/`；Rule Package registration 已切换到新唯一 owner，`routing.rs` 已删除且不保留转发壳。
- 公共 `ModelRegistration`、`PostgresStore::register_model`、`read_run` JSON identity 字段、错误语义、migration/Schema、配置、模型与路由算法均保持不变；未修改 `list_recent_runs`。
- PostgreSQL 16 专项 run `31881761988`：Model Run Identity contract job `95005200186` 与 Rule Package registration regression job `95005200191` 均 `SUCCESS`。
- clean source HEAD `dc6d2baf0f6092f67200214b5b45e37c528394ad` 的 canonical Public Platform CI run `31881842368` / Windows job `95005434692` 为 `SUCCESS`；artifact `9246499189`，SHA-256 `74bc2a687b8499f798c0a91a28522fa74df0237ee24d005197067b1c97a1e452`。
- R5-06 已正式关闭为 `DONE`；fixed clean HEAD `058884e82f3c584f87751dec3bb5f9b6531a151e` 的 push canonical run `31883239072` / job `95008635990` 与 PR #31 canonical run `31883703954` / job `95009730520` 均为 `SUCCESS`。
- PR #31 已按 fixed head squash merge 为 `acb0491003b365b3f775780d8c98ecfdf1e80104`；merged-stage canonical Public Platform CI run `31884882480` / Windows job `95012628426` 为 `SUCCESS`，artifact `9247274708`，SHA-256 `4c6b107922787a732537a83b58b005bb364ff10298146c87f0dff40f8bdf5a26`。
- 详细节点记录：`docs/modular-rewrite/R05-competition-routing-persistence/R05-06-model-run-identity-reads.md`。

### R5 阶段完成

- R5-01～R5-06 均已完成并通过各自专项契约、阶段回归、clean PR 与 merged-stage canonical gate；Competition / Routing PostgreSQL persistence 已按职责收敛到 `crates/persistence-postgres/src/adapters/competition/` 与 `adapters/rules/`。
- 旧 `crates/persistence-postgres/src/competitions.rs` 与 `crates/persistence-postgres/src/routing.rs` 已删除，不保留双实现或长期转发壳；公共 Application Port、Tauri 命令/DTO、Schema、0001–0046 migration、配置、错误语义、路由/模型算法与生产依赖保持兼容。
- R5 阶段完成记录：`docs/modular-rewrite/R05-competition-routing-persistence/R05-stage-completion.md`。R6-01 `Team Directory 与 Detail` 为下一阶段唯一 `READY` 节点；R6 源码实施必须以本次 closeout tree 的 canonical CI 成功为进入基线。



- `new-A` 已从 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 建立。
- `new-B` 已从 `new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496` 建立；R1-01 已创建模块边界与状态所有权契约并完成 Windows 自动化门禁，状态为 `DONE`，R1-02 已开放为 `READY`。
- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤；状态为 `DONE`，R1-03 已开放为 `READY`。
- R1-03 已建立 `src/bootstrap/` 浏览器组合根并切换 `index.html` 唯一入口；`src/main.ts` 仅保留既有业务实现并暴露受控生命周期。Windows workflow run `31012168809`、job `92326905405` 在提交 `a3b61088abaf0c9f052ecab09e040ea77bd8d344` 上通过，artifact `8933800016` 大小 `14117539` 字节，SHA-256 为 `4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9`；状态为 `DONE`，R1-04 已开放为 `READY`。
- R1-04 已建立 `src-tauri/src/bootstrap/` Tauri 组合根，拆分 Builder、全局状态、171 条命令注册和启动错误映射；状态为 `DONE`。
- R1-05 已建立 `crates/application/src/` 下的 Application 组合根、兼容服务门面、模型注册表和持久化端口注册入口；默认模型注册与 PostgreSQL 具体导入均已收敛到唯一所有者，公共 API 和行为不变。专项、架构、前端、Rust 及正式 Windows Automated 全部通过；workflow run `31073166446`、job `92525208547` 在提交 `08803725dcd9f403ffc25552c27d2a9c0d3acd2d` 上通过，artifact `8956912712` 大小 `14117884` 字节，SHA-256 为 `495d3c5e29f2b474e97b98f89dd64b6175cc6e9496dd77681c0a567e00c60016`。状态为 `DONE`，R1 阶段已关闭。
- R2-01 已建立可机器复算的 Domain 类型与调用链清单、目标模块归属策略和 Serde 契约测试。清单登记 365 个公共兼容类型、20 个 Domain 来源文件、139 个 Rust 扫描文件和 299 个 PostgreSQL 映射类型；生成与全量门禁 run `31077537198`、job `92538743873` 已通过。正式 Windows Automated run `31078483578`、job `92541654912` 在最终实施提交 `2a6b9ea96a88168d6a751ebf48c2030512edaf24` 上通过；artifact `8959079579` 大小 `14118091` 字节，SHA-256 为 `6b75cc3abe2067472476cd0e7811b9fd9ee6f689f17cbc0eb346030775d0c9e2`。本节点未迁移或修改任何 `crates/domain/src` 生产类型，状态为 `DONE`，R2-02 已开放为 `READY`。
- R2-02 已将 14 个 Competition 类型和 8 个 Routing 类型从 Domain 根文件迁移到职责目录；正式 Windows Automated run `31088698579`、job `92574240109` 已通过，artifact `8963219366` 大小 `14117627` 字节，SHA-256 为 `5bc1481807b8fd378e8845c1a813e0cca3981681a5f5d89c572a54e74c973124`。状态为 `DONE`。
- R2-03 已将 Team 17、Player 21、Coach 9、Formation 8、Shared 17 共 72 个类型从 Domain 根文件迁移到职责目录；根级类型名、Serde、数据库映射、Application、Tauri DTO、模型边界和生产依赖保持不变。实施 run `31100515822` 已通过；正式 Windows Automated run `31110013068`、job `92645025258` 在与实施提交 `038ebd7096a78f7202d9c98e66e17d32701d343c` 同源码树的触发提交 `594940dca4c57aabfebdd768755ec27006ecaeb5` 上通过，artifact `8972168972` 大小 `14119500` 字节，SHA-256 为 `6ef2e064638cd17b214c66bbdea5ed752a08a1f0dc32002940e0f97d094cae5f`；运行报告为 PASS，7 条记录、3 个完成操作。状态为 `DONE`，R2-04 已开放为 `READY`。
- R2-04 已将 Lineup 16 个类型和 Match 3 个类型迁移到职责目录并删除旧 `crates/domain/src/lineup_chain.rs`；根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令、生产依赖与模型保护边界保持不变。实施 workflow run `31151412918` 已通过专项 Serde 9/9、架构、前端、Rust、Clippy、workspace tests、精确变更集、legacy 删除、README 契约、transient 清理、提交后工作树和 push 后远端 HEAD 校验，并生成最终实施提交 `0aafe42d7ed08f8e78d71d44ccb6f8f58c425999`。正式 Windows Automated run `31153982572`、job `92789397631` 已在该最终提交上通过；artifact `8984980586` 大小 `14118155` 字节，SHA-256 为 `1e7224f4e7f713b0339e97fd114fa6dea2c0b2ecc9400789613fe872d660938c`。R2-04 状态为 `DONE`，R2-05 已开放为 `READY`。
- R2-05 已按两阶段迁移 Prediction 48 个类型与 Research 27 个类型。Prediction 独立提交 `2cd685b8057a1bce2f75e4c7f5b56aed1bf3d142` 的专项门禁通过；首次 run `31158780693` 中 Research 迁移与专项 Serde 11/11、完整 frontend 均通过，但完整 Rust 在 Clippy `-D warnings` 因 `prediction/orchestration/planning.rs` 两个未使用 import 停止，未提交 Research。已直接删除两个无效 import，不增加抑制；恢复 run `31159821513` 完成 Research 27 类型迁移、类型清单、架构、保护资产、frontend 与 Rust 全量回归。旧 6 个职责混合源文件均已删除，根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、生产依赖和模型保护边界保持不变。正式 Windows Automated run `31171082098`、job `92842834091` 已在包含最终 R2-05 源码树的提交 `e328b4aa5a7737e6bb378abf8b891cd953b99f62` 上通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。R2-05 状态为 `DONE`，R2-06 已开放。
- R2-06 已将 Review 48 个类型与 Postmatch 11 个类型迁移到 `review/`、`postmatch/` 职责目录并删除 5 个旧职责混合源文件；根级公共类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令、生产依赖和模型保护边界保持不变。staged 与 Windows Automated 验收已通过，用户随后使用原有 PostgreSQL 数据库连接成功；数据库兼容修复保留历史数据与不可变资产，不清库、不覆盖旧版本、不削弱 fail-closed 或内容指纹保护。R2-06 状态为 `DONE`。
- R2-07 已将 Analytics 39、Exchange 54、AI Workspace 16、Release 9 共 118 个公共兼容类型迁移到职责目录，并删除 7 个旧职责混合根文件；Windows 本机格式、17/17 Serde、365 类型清单与架构门禁通过，用户确认状态为 `DONE`。
- R2-08 已将 `crates/domain/src/lib.rs` 收敛为 17 个模块声明、365 个显式公共兼容类型 re-export，并补齐 34 个既有公共根常量的显式兼容出口；根级 glob export 已清零，三个私有默认值实现迁入 `shared/defaults.rs`，确定性根出口生成/验证门禁已接入 `verify:architecture`。Windows 专项 run `31236344727` 已通过根出口、球队资料包和保护资产门禁并生成最终实现提交 `62b1f622b9c14b33dbaac850812a49c063ccb090`；用户本机阶段回归未见报错，上传 runtime 日志共 58 条且全部为 `info`，启动 `connection_error=null`，球队、阵容、分析、Postmatch 与 API 工作区读取均正常完成。R2-08 状态为 `DONE`，R2 阶段已关闭。
- R3 已从 R2 完成提交 `7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f` 建立独立分支 `new-C`。R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 均已完成。R3-04 已将 35 个球队/球员/教练/实体引用职责拆入 Teams / Players Services；后续累计完整 frontend / Rust 回归与 2026-08-09 用户 Windows `tauri:dev` 非破坏性烟测已补齐，`update_team`、`update_player`、教练读取及 team/player/coach 实体引用均成功，runtime JSONL 209/209 条为 `info`，状态正式关闭为 `DONE`。R3-05 已删除旧 `crates/application/src/player_catalog.rs`，将剩余 19 个阵型/比赛/阵容/阵容预设职责迁入 `services/lineups/` 与 19 个对应 Use Cases，并以 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports 保持公共 Application/Tauri 契约；`read_match_exchange` 仅提升 workspace 可见性，SQL、参数、返回结构与数据库行为不变。clean 实施提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过完整 Windows Automated；用户随后在最终分支完成 clean 工作区、rustfmt、Lineups 专项、architecture、Application 33/33、完整 frontend、完整 Rust/Clippy/workspace tests 与 `tauri:dev`。本机 runtime JSONL 共 280 条，除 3 条预期输入校验与 3 条公开模型运行时未分发的既有错误外，无 Lineups、SQL、migration、panic 或连接失败；预设保存、应用预检、双方阵容原子创建与 `ready_for_model=true` 阵容链均实际跑通。R3-05 状态为 `DONE`。R3-06 Prediction Service 已完成并关闭为 `DONE`；R3-07 Research Service 已完成并关闭为 `DONE`。R3-08 Review / Postmatch / Analytics Services 已正式关闭为 `DONE`；Review Core、Match Review Package、Postmatch 与 Analytics 四个 Atomic Tasks 均为 `DONE`，R3-09 Exchange / AI Workspace / Release Services 已正式关闭为 `DONE`（AT1 Match Lineup / AI Match Package Exchange、AT2 Spreadsheet Exchange、AT3 AI Workspace、AT4 Release 均已关闭）；R3-10 ApplicationService 兼容门面已关闭为 `DONE`，R3 阶段已正式关闭，详见 `docs/modular-rewrite/R03-application-services/R03-stage-completion.md`。
- R3-10 最终 clean Public Platform CI run `31593758268` / Windows Automated job `94104353199` 已全部通过，artifact `9140937975`，SHA-256 `a67e78ee1272d9a953432292ee284118cffcc17325a9aefedf4367cec451ee75`；PR #21 merge commit 为 `f400740a36e29ab5cf154728c5f240ee018707c6`。R3 最终为 15 个 Application Port 职责域、38 个最小 trait，`ApplicationService` 仅保留兼容门面；18 个专用 PostgreSQL 集成测试与 destructive reset 仍未执行。
- R4-01 已将 Persistence Store/Error/Pool/Migration/Health/Statistics 从旧 `lib.rs`/`connection.rs` 拆入职责目录；历史 0001–0046 migration SQL、公共 Persistence API、DatabaseOptions Serde、错误语义与生产依赖保持不变。严格 hard gate run `31615481637` 已通过专项、数据库基线、architecture、persistence crate check/tests/Clippy；最终 clean Public Platform CI run `31615930113` / job `94178594481` 亦为 `SUCCESS`，artifact `9150134498`，SHA-256 `6da5f1750feef5f7fc00233165dea7f7563fefa96d08f8e515a22e16fa4dd5e1`。PR #22 merge commit 为 `ed99ce3bc21da76a55de526a7f912fa363bb84a5`；18 个专用 PostgreSQL 集成测试仍未执行，未执行 destructive reset。R4-01 状态为 `DONE`，R4-02 已开放为 `READY`。
- R4-02 Audit 基础设施已将 AuditEvent/entity ID/payload/SHA-256/审计 SQL writer 收敛到 `crates/persistence-postgres/src/audit/`，并清理 Match Exchange、Player、Spreadsheet、Team Catalog、Team Force Delete 的重复/直写审计路径；`create_team` 的业务行与审计行改为同一事务提交。strict hard gate run `31629430245` 已通过专项、数据库冻结契约、architecture/frontend 与 Rust 全量回归；最终 clean Public Platform CI run `31630618699` / job `94228122818` 为 `SUCCESS`，artifact `9155761101`，SHA-256 `82dbb88b6dd974ad43c2d86bfb291aca8831faa9d956212bdc04de7110d87dee`。PR #23 已按固定 HEAD `6598065e6edf671e8806fc77901d083bad910542` 合并，merge commit `bc4154044f194e5b1505b4ebb308ba51d6208663`；18 个专用 PostgreSQL 集成测试仍未执行，未执行 destructive reset。R4-02 状态为 `DONE`，R4-03 已开放为 `READY`。
- R4-03 Row 映射基础规范已建立 `mapping/time.rs`、`uuid.rs`、`json.rs`、`optional.rs`、`invalid_state.rs` 五类基础标量职责；共享 CompetitionKind 解析迁入独立 `competition_kind.rs`，根模块仅保留兼容 re-export，未建立万能动态 mapper。strict hard gate run `31674700550` 已通过 mapping 专项、R4-01/R4-02 回归、数据库冻结/保护资产、完整 architecture/frontend、Persistence check/tests、workspace Clippy `-D warnings` 与 workspace tests；PR #24 clean Public Platform CI run `31675727990` / job `94369762967` 为 `SUCCESS`，PR 按固定 HEAD `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56` 合并，merge commit `0e5a68e09c6c06204b926f4d30c45262740d983b`。合并后 stage Public Platform CI run `31677600876` / job `94375513281` 同样为 `SUCCESS`；artifact `9172855279` 大小 `13909093` 字节，SHA-256 `31719ff04f00eb944c84fcd37dbbda3fa6252f7d55bc42a1a8af3d903cad3544`。18 个专用 PostgreSQL 集成测试仍未执行，未执行 destructive database reset。R4-03 状态正式关闭为 `DONE`，R4-04 已开放为 `READY`。
- R4 Persistence 基础设施阶段已正式关闭为 `DONE`。R4-04 已由 PR #25 squash merge `b97587c9d20165018f80040dc2a2c098dbbec177` 收口；hard gate `31712193150`、PR CI `31717901248` / job `94507174688`、post-merge stage CI `31724131556` / job `94528167665` 均 `SUCCESS`。真实 PostgreSQL broad diagnostic `31728096953` 执行 18 个 ignored tests 为 14/18，并安全通过临时 test DB destructive reset；4 个失败如实保留为 3 个既有过期业务夹具与 1 个 R4 排除范围内的既有 P4 timestamp 精度问题。R4 专项 PostgreSQL gate `31729577225` / job `94546316946` 在空库真实验证 0001–0046、health/stats、audit 失败回滚与成功提交原子性，1/1 PASS。临时数据库/runner-only test/workflow 已清理，未触及用户数据库。最终 closeout HEAD `615dc952491d5e0e21d4979292cbf5170eedece6` 的 canonical Public Platform CI run `31768264193` / Windows automated delivery gate job `94668577704` 为 `SUCCESS`。
- R5-01 Competitions Repository 已正式关闭为 `DONE`：`create/read/list/delete competition` 从旧 `crates/persistence-postgres/src/competitions.rs` 拆入 `adapters/competition/directory/` 与 `detail/`，typed Row 与 Domain Mapper 分离，多表 soft delete 使用具名 transaction 子目录，旧文件只保留 R5-02/R5-05 尚未迁移的 hierarchy/context 职责。未修改 Application Port、Tauri DTO/命令、Schema、0001–0046 migration、路由算法、model identity、Cargo 依赖或模型保护资产。旧实现 PostgreSQL contract baseline run `31772436658` / job `94680947724` 为 `SUCCESS`；新实现 Persistence 80/80、Application 33/33、PostgreSQL contract 1/1、完整 architecture、Windows frontend run `31773777077` / job `94684879652`、Windows workspace Rust run `31773900831` / job `94685243308` 均已通过。PR #26 clean CI run `31775501711` / job `94689993582` 为 `SUCCESS`，固定 head `999949da6b3d5aeff4734b68779d3b15a890abe1` 已 squash merge 为 `2b4668c2c0da45c9f62c1d40c4765cd730653b33`；merged stage CI run `31777130456` / job `94694820128` 亦为 `SUCCESS`，artifact `9210904946`，SHA-256 `1303176d639550a832327c6e3e6c564f843e7eed10b673144ecd6413b7b01d31`。18 个既有 broad PostgreSQL ignored tests 与 destructive reset 未在 R5-01 执行。R5-02 已开放为 `READY`；详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-01-competitions-repository.md`。
- R5-02 Seasons / Stages / Rounds 已正式关闭为 `DONE`：hierarchy persistence 从旧 `crates/persistence-postgres/src/competitions.rs` 收敛到 `adapters/competition/hierarchy/{seasons,stages,rounds}/`，Season/Stage/Round 分别拥有 create/read/list、typed `sqlx::FromRow` Row 与 Domain Mapper；旧文件仅保留 R5-05 context/scope 职责。Application `CompetitionHierarchyPort`、Tauri DTO/命令、Schema、0001–0046 migration、前端行为、路由算法、model identity、依赖与模型保护资产保持不变。旧 owner PostgreSQL contract baseline run `31793408123` 与第四轮 hard gate run `31794402789` / job `94748190282` 均 `SUCCESS`。PR #27 clean CI run `31794818136` / job `94749470982` 为 `SUCCESS`，artifact `9217614243`，SHA-256 `e1adc3016f71f1c563b9fd0d29721a9f04452cccfe5661f6bb5be52924f7601a`；固定 head `89f164821e8f8157ab8a4804cf1d026a40ab8932` 已 squash merge 为 `baf307fcb733659385f83f187ee343184946ee9d`，merged stage CI run `31796688330` / job `94755204665` 亦为 `SUCCESS`，artifact `9218314804`，SHA-256 `80e836c4012524ac66a21b085b86bebcd4f3fdf4661594d636050fb753f98d65`。18 个既有 broad PostgreSQL ignored tests 与 destructive reset 未在本节点执行，未触碰用户数据库。R5-03 已开放为 `READY`；详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-02-seasons-stages-and-rounds.md`。
- R5-03 Rule Packages 已正式关闭为 `DONE`：`register/list rule package`、source document upsert 与 RulePackage Row mapping 已从旧 `crates/persistence-postgres/src/routing.rs` 收敛到 `adapters/rules/packages/`，事务编排、package INSERT、existing conflict read、profile attach、source upsert、typed `sqlx::FromRow` Row 与 Domain Mapper 均按职责分离；旧文件不保留 R5-03 转发壳，R5-04 Binding、R5-05 Route Resolution、R5-06 Model Run Identity 未提前迁移。旧/new owner PostgreSQL contract、official inventory 与 hard gate 均已通过；PR #28 clean CI run `31812316772` / job `94805647551` 为 `SUCCESS`，artifact `9224448312` SHA-256 `00cf61b683b7780a354ad5e35c59438d6d26ce1a6d0c17dd86edbaaa9e9ae127`；固定 head `d087108d5cc722cc9ebecd788de6c07c1ae5dc3d` 已 squash merge 为 `3d609ace7cbe1db3a3caec18a6477fb41153cc88`，merged stage CI run `31824523536` / job `94845378301` 为 `SUCCESS`，artifact `9229038667` SHA-256 `9c07c35abcd8b229b8044b0373f7ce3aecd6a03d58b1431c4f738e22131c7262`。18 个既有 broad PostgreSQL ignored tests 与 destructive reset 未在本节点执行，未触碰用户数据库。R5-04 已开放为 `READY`；详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-03-rule-packages.md`。
- R5-04 Competition Bindings 已正式关闭为 `DONE`：Binding create/list/detail/type-default 与 package route metadata 已从旧 `crates/persistence-postgres/src/routing.rs` 收敛到 `adapters/competition/bindings/`，typed `sqlx::FromRow` Row、Domain Mapper、validation、单目的 SQL helper 与具名 transaction 均按职责分离；旧文件不保留 Binding 转发壳，R5-05 `resolve_route`/competition context 与 R5-06 model registration 未提前迁移。旧/new owner PostgreSQL 16 contract、official inventory 与 hard gate 均已通过；PR #29 clean CI run `31863321636` / job `94960259474` 为 `SUCCESS`，artifact `9241585461` SHA-256 `d61179bab3a46f4457636d411c9f09325145fa4cc3e69110d166f7814a2e4a5a`；固定 head `e63d3adbed3ddf8cce5d7486a47d2ce6cc4d9bb9` 已 squash merge 为 `eabf3939216f43dbd91839223c1e3c34c3872406`，merged stage CI run `31864520861` / job `94963336422` 为 `SUCCESS`，artifact `9241894657` SHA-256 `d3b3a43eff2f2a574fd4847533bb43b20eee961086a13ad2a3f17d34848bbe4e`。Domain/Application/Tauri/Schema/0001–0046 migration、配置/错误/日志、前端行为、route result/model identity、Cargo manifests/Cargo.lock、生产依赖与模型保护资产保持不变。18 个既有 broad PostgreSQL ignored tests 与 destructive reset 未在本节点执行，未触碰用户数据库。R5-05 已开放为 `READY`；详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-04-competition-bindings.md`。
- R5-05 Route Resolution Reads 已正式关闭为 `DONE`：competition context 与 route resolution persistence 已收敛到 `adapters/competition/route_resolution/{context,route}/`，legacy `competitions.rs` 已删除，legacy `routing.rs` 仅保留 R5-06 model registration；未提前创建 `model_run_identity/`。old/new owner PostgreSQL contract、R5 ownership、official Domain inventory 与 stage hard gate 均已通过。PR #30 首轮 clean canonical run `31870124821` / job `94977359038` 因 typed `RouteRow.competition_kind` 未使用而在 Windows workspace Clippy `-D warnings` 失败，失败 tree 未合并；修复仅删除未消费 Row 字段及冗余 SELECT 返回列，保留 `b.competition_kind` WHERE 过滤与 route specificity/result，official inventory refresh run `31870479459` / job `94978235169` 为 `SUCCESS`。同一修复源码 Windows canonical run `31870519567` / job `94978336960` 为 `SUCCESS`；最终 fixed clean HEAD `879dd724e9bace058d86387b04e97cbdb67843b7` 的 PR canonical run `31873912016` / job `94986637325` 为 `SUCCESS`，artifact `9244488927` SHA-256 `4cc15be2916477ea56a80080090c18b033c8eab1e6cfd2a4f4da948d1b24b754`。PR #30 已按 expected head squash merge 为 `d6b2d692dadbe546220eb0cb533bd9d4e207d230`；merged-stage canonical run `31875035071` / job `94989384122` 为 `SUCCESS`，artifact `9244773760` SHA-256 `f342644fd91e617331060360d47513a076594aa379f1e9e9493b287e2df97b08`。Ubuntu 专项 fix gate `31870518531` 受 runner 缺 `glib-2.0` 系统库阻塞，未把未完成的 workspace Clippy/PG contract 记为通过。18 个 broad PostgreSQL ignored tests 与 destructive reset 未在本节点执行，未触碰用户数据库。R5-06 已开放为 `READY`；本 closeout tree 仍必须通过一次 canonical Public Platform CI，成功后才是 R5-06 的有效起始基线。详见 `docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md`。
- 开发依赖布局已外置：Node 依赖固定到 `../node_modules`、npm 缓存固定到 `../.npm-cache`，Cargo 构建输出继续使用 `../.cargo-target`；仓库根目录不保存依赖目录。
- R1-04 前置校正将 `crates/application/src/model_shell/mod.rs` 恢复为 Rust 1.88 标准排版，并只同步更新该文件的保护指纹与派生聚合值；导出集合、模型行为和保护范围均未变化。
- R1-04 已同步迁移 19 个既有验证器读取新的 Tauri 命令注册表或状态所有者，消除旧 `lib.rs` 路径造成的伪失败；产品代码和公共契约未改变。
- R1-04 Windows Automated 启动烟测改为按启动前日志路径集合识别新 session，并在首次 45 秒超时时最多重启一次；每次启动保留 stdout/stderr，连续两次超时仍硬失败。打包前后 EXE 的 A/B 运行均正常，产品入口、bundle、命令和业务行为未改变。
- R1-04 workflow run `31037323146`、job `92412650719` 在清理后代码树提交 `5cb66fdedbfcaf89c86a7124f8894bdc71a533c9` 上通过；artifact `8943939773` 大小 `14119217` 字节，SHA-256 为 `7562c9137d52040627a58d9c8e104c4053b9923983a16daae08e4361e9a78f2b`。Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误，release 客户端首次启动即建立日志。
- R1-02 最终 workflow run `31001470224`、job `92291121763` 在提交 `28ec363babe4f3fbccd14693d0261febdc305458` 上通过；artifact `8929207011` 大小 `14117150` 字节，SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 截图启动工具仅对 Chromium `DevToolsActivePort` 的 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容执行最长 15 秒的有界重试；其他错误立即失败，截图差异阈值与门禁强度未放宽。
- `Public Platform CI` 现支持推送到 `main`、`new-*`、`rewrite/**`、Pull Request 和 `workflow_dispatch`，以 `windows-2025` 执行架构契约、前端、Rust、Tauri Windows release、release 客户端启动和运行日志扫描，并上传验证证据。 2026-10-25 起，仅新增、修改或删除 Markdown 文档（`**.md`）的直接推送跳过自动 CI；混合代码、依赖、构建或 CI 配置改动仍触发，Pull Request 与手动运行保持。发布包规则入口检查现存 `AGENTS.md`，不再强制包含已移除的两份旧规则文档。
- R1-01 验证运行 `30989439570`、job `92251837163` 在提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` 上通过；artifact `8924033934` 大小 `14115361` 字节，SHA-256 为 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 新增 `.gitattributes` 固定文本 LF 和二进制排除规则；相关验证器统一按 LF 规范读取冻结合同，避免 Windows 检出换行导致伪失败。冻结合同、迁移哈希、锁文件、生产依赖、公共命令、数据库结构和模型保护资产均未改变。
- R0-01 已冻结远端分支起点并建立 `docs/modular-rewrite/R00-baseline/` 节点记录。
- R0-02 已新增 `architecture/protected-assets.json` 和 `scripts/verify_protected_assets.mjs`，冻结 18 个公开模型边界及校验文件，聚合 SHA-256 为 `d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- R0-03 已新增 `architecture/command-contract.json` 和 `scripts/verify_command_contract.mjs`，冻结 171 个公共命令、15 个 Rust 命令模块和前端调用边界。
- R0-04 已新增数据库静态基线与安全执行入口，冻结 0001–0046 共 46 个迁移，聚合 SHA-256 为 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。真实 PostgreSQL 验证按用户要求留到最终统一验证。
- R0-05 workflow run `30910130867` 中，Linux `npm ci` 通过；Chromium 启动失败。Rust locked metadata 通过，但 `cargo fmt --check` 失败，Clippy 与 workspace tests 未执行。
- R0-06 workflow run `30912862564` 建立 Windows 基线。Windows release 构建和 RuntimeOnly startup 通过；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06 精确 Automated 暴露 Windows Node 调用问题及验收路径契约问题。
- R0-06.1 新增 `scripts/process/execution-context.mjs`、`scripts/process/node-package-cli.mjs` 和 `scripts/verify-node-process-compatibility.mjs`，关闭目录联接依赖同步与 `.cmd` 子进程调用缺口。Windows workflow run `30919764753` 中完整 frontend 通过，Automated 到达 Rust 阶段。
- R0-06.2 新增 `scripts/windows/acceptance-paths.psm1` 与 `scripts/verify-windows-path-contract.mjs`，支持根入口既有 `LogDirectory` 参数，并按 Cargo target 登记文件查找 release EXE。
- R0-06.2 Windows workflow run `30922384735` 中，完整 frontend、release 构建与 RuntimeOnly runner 均实际通过。release EXE 从项目根目录 `.cargo-target\release` 启动；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06.2 证据 artifact 为 `8898312587`，SHA-256 为 `d6ed06066aab354686f86938ec7c55f2c1f740e11a37e42a6a1b5edbbd53df63`。临时 workflow 的 job 最终因产品验证结束后的一条辅助中文日志精确匹配未命中而显示 failure；证据文件已复核，未将 workflow 总体描述为通过。
- R0-07 使用 Rust 1.88.0 rustfmt 对 42 个已诊断 Rust 文件进行纯格式规范化，实施提交为 `9e7be511ae2d97a0782fee1a2bea5e25d910d10d`；未触碰模型保护文件、依赖、锁文件、迁移或公共接口。
- R0-07 精确 workflow run `30961535208` 中，Cargo.lock 门禁、Cargo target 准备和 `cargo fmt --all -- --check` 通过。完整 `npm run verify:rust` 随后在 16 个 Clippy 错误处以退出码 `101` 结束，workspace tests 因 fail-fast 未执行。
- R0-07 精确验证 artifact 为 `8913160029`，SHA-256 为 `47712408cb9fbd37088f42cab92e71565b0c982d5c0492a78fb6c4ef2e53ad49`。
- R0-08 以 11 个 Rust 文件白名单关闭 Clippy 门禁，实施提交为 `919d62a2eaf95ade5ba1efa18924a9d578ef3f63`；没有添加 `allow`、放宽 `-D warnings`、修改依赖、迁移、模型边界或公共接口。
- R0-08 workflow run `30965687503` 中，持久化库测试 73/73、Tauri runtime log 专项 7/7、`cargo clippy --locked --workspace --all-targets -- -D warnings` 均通过。表格库完整测试为 11/12，通过替代验证确认除既有空白行用例外其余 11 项通过。
- R0-08 精确 workflow run `30966064295` 中，Cargo.lock、rustfmt 和 Clippy 连续通过；workspace tests 首个失败为 `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret`，因此完整 `npm run verify:rust` 以退出码 `101` 结束。
- R0-08 最终应用 artifact 为 `8914718704`，SHA-256 为 `b8e75726c6ad53bdb4932ceb0bb3d35ff4554f306179178e6a566187723c6c60`；精确验证 artifact 为 `8914844238`，SHA-256 为 `05ee24344468b9613bf18c139ff7d3aabecb92e005f93afb1f9037ed7f21cede`。
- R0-09 先以 `--no-fail-fast` 完整扫描 workspace tests，确认失败集合只有 OpenAI 提示词大小写断言和表格稀疏测试工作簿两项。
- R0-09 仅修正两个测试契约，共新增 2 行；实施提交为 `50daa258af8ac8e09f8e4f5f428249fe670f2dd2`，未改变生产逻辑、公共接口、数据、配置、依赖、迁移或模型保护边界。
- R0-09 workflow run `30967448070` 中两个专项测试、185 个 workspace 测试以及精确 `npm run verify:rust` 全部通过；18 个 PostgreSQL 集成测试保持忽略并按用户要求留到最终统一验证。
- R0-09 最终 artifact 为 `8915431192`，SHA-256 为 `a745bded71179bb6542d3a06b5c65f61cdf48845b8f88193dc7ef0ac5c8fcadc`；临时 workflow、patch 与 Draft PR #6 均已清理或关闭。
- 用户于 2026-08-05 明确将目标平台收敛为 Windows，Linux Chromium 不再属于交付或阶段门禁；PostgreSQL 实跑、Windows Full 与用户本机实机验收统一延期到最终验收。R00 已完成并开放 R1，详见 `docs/modular-rewrite/R00-baseline/R00-stage-completion.md`。
- R0-01 至 R0-06.2 未修改前端或 Rust 业务源码；R0-07 只改变 Rust 排版；R0-08 仅实施行为等价的私有参数收束、Copy/借用修复、无效私有代码清理和测试编译补全；R0-09 只修改测试构造与断言。
- 当前执行环境未建立本地 Git 工作树，用户设备上的未提交与未跟踪文件不可见；远端分支操作不会覆盖这些本地内容。

## 0.23.0 变更记录

- 公开重写后的内置 P4 evidence、prematch snapshot、research output Schema 与研究 Prompt 使用独立不可变内容版本（`1.0.0+public.1` / `2.0.0+public.1`）；旧数据库中的同名旧版本继续保留，启动时不覆盖历史内容或放宽指纹校验。
- 删除真实 P4/P7 模型 crate、参数、Profile、固定比赛、模型专用契约、Schema、研究资源和验证脚本。
- 新增外部 `ModelProvider` Stub、通用模型边界契约、公开研究资源和明确不可用错误语义。
- 移除公开代码中的固定模型矩阵拓扑、固定比分单元数、校准常量、时间前推参数算法和自动参数候选生成。
- 保留赛事推演、模型路由、规则包、快照、复盘、分析和数据库入口；未连接外部提供器时不会执行预测。
- 新增 GitHub Actions 自动验证，减少本机重复下载和手工测试。

## 历史兼容记录

以下标题用于保留原项目的可追溯历史契约；不表示本次公开拆分重新执行了所有历史验收。

## 0.22.0 变更记录

接入点 H 的不可变赛后结算、证据评分和监控工作流保留。既有 UI 不重复重构；公开版本只调整模型执行边界。

## 0.19.0 变更记录

工作区 UI、双层导航和页面状态保持历史兼容。

## 0.15.0 变更记录

实体关系、球队与球员管理链路保持历史兼容。

## 0.14.0 变更记录

API 协作工作台、OpenAI Profile 与运行日志链路保持历史兼容。

## 0.13.5 变更记录

API 传输与诊断契约保持历史兼容。

## 0.13.4 变更记录

球队资料、导入和历史记录能力保持历史兼容。

## 0.13.3 变更记录

球队与球员管理修复保持历史兼容。

## 0.13.2 变更记录

球队与球员管理基础链路保持历史兼容。

## 历史能力索引

- 默认战术角色全链路：Excel、档案、阵容、复盘与来源审计。
- 球队完整资料包与 P4 输入就绪度历史工作流；历史数据记录包含 1248 条世界杯球员俱乐部关系增量补录。
- 强制删除全部资料使用 `football.force_purge` 审计边界；永久删除预检会处理陈旧标签页。
- 导入行子记录身份修复保留同一物理行内多实体身份。
- Windows 实机全链路验收阶段 5 由 `验收平台.bat` 启动。

## 验证事实与限制

R0-06.1 已关闭 Windows 目录联接依赖同步和 `.cmd` 子进程调用缺口；R0-06.2 已关闭 `LogDirectory` 参数和 Cargo release 查找路径缺口。Windows 完整 frontend、release 构建与 RuntimeOnly startup 均已有真实通过证据。R0-07 已关闭 Rust `cargo fmt --check` 阻塞；R0-08 已关闭 workspace all-targets Clippy `-D warnings` 阻塞；R0-09 已关闭全部已枚举 workspace tests 失败，精确 `npm run verify:rust` 现已通过。

R00 阶段已按 Windows-only 目标范围标记为 **DONE**。Linux Chromium 历史失败仅保留为非目标平台记录，不再阻塞后续重写。

PostgreSQL 实跑、Windows Full 和用户本机 Windows 10/11 实机验收仍保留最终统一验收；R2-06 节点已额外使用原 PostgreSQL 数据库完成连接验证。R3-02 已使用原数据库完成非破坏性 `tauri:dev` 运行时烟测，但真实 destructive reset 仍只允许在专用测试数据库执行。另保留 1 个 moderate npm vulnerability 和 Vite 大 chunk 警告。

已创建 `R00-stage-completion.md`、`R01-stage-completion.md` 与 `R02-stage-completion.md`。R1、R2 阶段均已关闭；R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services、R3-05 Lineups Service 与 R3-06 Prediction Service 状态为 `DONE`，R3-04 Teams / Players Services 与 R3-07 Research Service 状态均为 `DONE`，R3-08 Review / Postmatch / Analytics Services 为 `DONE`（AT1 / AT2 / AT3 / AT4 均已关闭）；R3-09 Exchange / AI Workspace / Release Services 为 `DONE`（AT1 Match Lineup / AI Match Package Exchange、AT2 Spreadsheet Exchange、AT3 AI Workspace、AT4 Release 均已关闭）；R3-10 ApplicationService 兼容门面已完成 AT1/AT2 源码迁移并进入 `VERIFYING`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。


## R3-06 Prediction Service（DONE）

- R3-06 在独立分支 `rewrite/r3-06-prediction-service` 上实施，未修改 `new-C` 的 R3-05 已验收基线。Atomic Task 1 已将 Prediction Core 的推演执行、readiness、route preview、formal/shadow stored-match execution、dry-run 与运行历史职责迁入 `services/prediction/`、`use_cases/prediction/`，并通过既有 Ports 保持 ApplicationService / Tauri 公共调用语义；模型执行继续只经 `football-model-api` 边界，不修改或复制模型实现。
- Atomic Task 2A 仅迁移 Prediction 所属的 P4 horizon planning、freeze task list/read/events、freeze readiness、match/task workspace 只读职责；`resolve_p4_conflict`、联网 Research 执行、Evidence/Fact 写入和 Research artifact 写入仍保留给 R3-07，不因旧文件混合职责而提前迁移。
- 2A 专项 Windows run `31266144950` / job `93124468057` 已通过 Application Ports、完整 architecture、rustfmt、`cargo check --locked -p football-application` 与 `cargo test --locked -p football-application`，Application tests 33/33 通过。
- 模块化删除旧 `crates/application/src/prediction.rs` 后，确认并修复 3 个历史验证器的旧 owner 路径：默认战术角色、比赛工作流、历史比分验证器均改读当前 Prediction Service / Use Case 权威模块；原业务断言未删除或放宽，其中比赛工作流与历史验证改为递归扫描完整 Prediction 模块树。
- 2A 编译期确认的未使用 import 已直接清理，不增加 lint 抑制。warning-cleanup Windows run `31266871976` / job `93126329974` 已通过 Application Ports、architecture、rustfmt、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 与 Application tests；测试专用 `P4Horizon` / `is_p4_model` 仅移入 `#[cfg(test)]` 作用域。
- Atomic Task 2A 已正式关闭为 `DONE`。最终验收提交 `443286b269cc6f34318bcf9ea60a86697f7a64a8` 的 Public Platform CI run `31268125289` / Windows Automated job `93129475772` 已全部通过：architecture、frontend、17 个截图回归视口、TypeScript、Vite production build、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建与 release 客户端运行日志验收均为 PASS；Application tests 33/33 通过。artifact `9025087726` 大小 `14245255` 字节，SHA-256 `f69a988b6832c5af18af661ea3e436ffeb48212d9a7f67c049676356376180ae`。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行。2A、warning cleanup、inventory refresh 与 Clippy 修复使用的临时 workflow / 脚本均已清理；R3-06 继续为 `IN_PROGRESS`，R3-07 继续为 `BLOCKED`。
- Atomic Task 2B 已正式关闭为 `DONE`。实施提交 `0d691114e67116fb9f03e4cd0fb04c6a819d4254` 已将 P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service，OpenAI Research、Evidence/Fact 写入与冲突人工覆盖继续保留在 R3-07 边界。专项 Windows hard gate run `31289363055` / job `93183820380` 已通过 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。clean 源码树验收提交 `b0fe2fe5cf3a24e7a8894b8f44061570afa3e35d` 的 Public Platform CI run `31289854065` 已全部通过：architecture job `93185076242` 与 Windows Automated job `93185076247` 均为 SUCCESS，完整 frontend、17 个截图回归视口、TypeScript、Vite production build、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建及 release runtime 日志验收均为 PASS；Application tests 33/33 通过，runtime 自动验收为 7 条日志 / 3 个完成操作。artifact `9031315604` 大小 `14249898` 字节，SHA-256 `438a352f8d81cee9b044d9c1d9a36682f1df435fa513eb7768bbd875322e89bb`。18 个真实 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 按既有安全设计保持 `ignored`，未记为已执行。2B 临时 workflow / Python 脚本均已清理；R3-06 继续为 `IN_PROGRESS`，R3-07 继续为 `BLOCKED`。
- Atomic Task 2C 已正式关闭为 `DONE`。Windows hard gate run `31292152981` 已通过 Prediction Service、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests；公开 P4 Snapshot 写入/读取 API 已从旧 `p4_persistence.rs` 迁入 Prediction Service / `use_cases/prediction/p4_snapshot/`，`P4FreezeExecutionPort` 同时承担不可变快照写入与读取，ApplicationService 方法名、参数和返回类型保持兼容。clean-tree 验收 HEAD `5e60f0b16c907d38f1a827a8705dde59c626b045` 的 Public Platform CI run `31292509918` / Windows Automated job `93192012861` 已全部通过：完整 frontend、17 个截图回归视口、TypeScript、Vite production build、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建与 release runtime 日志验收均为 PASS；Application tests 33/33 通过，runtime 自动验收为 7 条日志 / 3 个完成操作。artifact `9032172319` 大小 `14249704` 字节，SHA-256 `8a14b02d4687a75839497ed07b443e92be5960f49c1b98ad9ea5db6d0bae8450`。18 个真实 PostgreSQL 集成测试因未配置专用可写 `FOOTBALL_TEST_DATABASE_URL` 按既有安全设计保持 `ignored`，未记为已执行。终审确认旧 `p4_orchestration.rs` 仅保留 R3-07 Research worker 且 freeze 分支已经委托 Prediction Service，`p4_workbench.rs` 与 `p4_persistence.rs` 剩余公开写职责均属于 Research；2C 临时 workflow / 脚本均已清理。R3-06 已关闭为 `DONE`，R3-07 Research Service 开放为 `READY`。


## R3-07 Research Service（DONE）

- Atomic Task 1 建立 Research Service / Use Case / Ports 骨架，并迁移原 `p4_persistence.rs` 的 7 个公开 Research Artifact / Ledger API：schema、prompt、赛事配置版本、research run、run event、evidence claim、evidence conflict。数据库初始化中的内置 P4 schema 注册同步改经 ResearchService；旧 `p4_persistence.rs` 删除，不保留空转发层。
- 新增 `ResearchEvidenceLedgerPort`，并扩展既有 `ResearchArtifactPort` 的赛事配置版本和 run-event 返回契约；具体 PostgreSQL 仍仅由 `composition/adapters/research.rs` 适配，Research Service / Use Case 不直接依赖 PersistenceStore、PostgresStore、SQLx 或 PgPool。
- `verify:research-service` 已接入 `verify:architecture` 与 `verify:frontend`。Atomic Task 1 Windows hard gate run `31295528438` 只有在 Research 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 全部通过后才形成 clean 提交。OpenAI Research、Fact Pipeline、P4 Research worker 和人工冲突裁决留给后续 R3-07 Atomic Tasks。
- 正式 Public Platform CI run `31295931710` 在 Database Service verifier 检出旧 `p4_persistence.rs` owner 断言；产品源码和 AT1 Rust 门禁未失败。验证器已改为检查 Database facade -> ResearchService -> Artifact Catalog -> ResearchArtifactPort 的完整内置 P4 artifact 注册链，不删除或放宽初始化断言。修复门禁 run `31296085324` 通过后才提交本记录。
- 第二次正式 Public CI run `31296120912` 已确认 Database verifier 通过，随后 Prediction verifier 因直接读取已删除的 `p4_persistence.rs` 触发 ENOENT。该验证器已改为确认旧 owner 不存在，并直接检查 `use_cases/prediction/p4_snapshot` 继续持有两个快照职责；原 Prediction facade/service/port/adapter 与 Research conflict 边界断言均保留。修复门禁 run `31296198981` 通过后提交本记录。
- 第三次正式 Public CI run `31296232974` 已确认旧文件读取问题消失，但发现 verifier 将 `p4_snapshot` 内部 `freeze/read` 误按公共 Application 方法名断言。已改为检查真实内部 `freeze/read` 入口及 `P4FreezeExecutionPort` 调用，同时继续由 facade/service 断言公共 `freeze_p4_prematch_snapshot/read_p4_prematch_snapshot` 契约。修复门禁 run `31296338770` 通过后提交本记录。
- Atomic Task 1 已正式关闭为 `DONE`。clean 正式 HEAD `ad53bd6c8cdbd93c9e58a38642ae4d22c7d32df7` 的 Public Platform CI run `31296372108` / Windows Automated job `93201993026` 已全部通过：architecture、完整 Windows automated acceptance 与 validation evidence upload 均为 SUCCESS；此前 AT1 hard gate run `31295528438` 已通过 Research Service 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。最终 artifact `9033265259` 大小 `14252405` 字节，SHA-256 `730bc98ba24dca2955c7100975ed79fd10a66919fd74a713ec061bddd3b44b59`。AT1 施工和 verifier 修复临时 workflow / 脚本均已清理；R3-07 继续为 `IN_PROGRESS`，OpenAI Research、Fact Pipeline、P4 Research worker 与人工冲突裁决留给后续 Atomic Tasks。
- Atomic Task 2 迁移 Fact Pipeline：旧 `crates/application/src/fact_pipeline.rs` 删除，`process_p4_research_evidence` 经 ResearchService / `use_cases/research/fact_pipeline/`；实现按协调器、实体解析、时间审计、来源策略、证据持久化、冲突裁决、路由、验证与共享类型拆分，不保留单文件巨型 owner。`FactPipelinePort` 按真实持久化能力收敛并由 `composition/adapters/research.rs` 适配；冲突事件通过序列化新类型穿越 Port，Ports 不暴露裸 JSON Value。OpenAI Gateway、P4 Research worker 与人工冲突裁决仍留给后续 Atomic Tasks。
- Atomic Task 2 已正式关闭为 `DONE`。Windows hard gate run `31298524184` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 提交 `1fb4c7b05573ef75eb48903eea25bc8b2072c9de`。该 HEAD 的 Public Platform CI run `31298887231` / Windows Automated job `93208343460` 已全部通过，validation evidence upload 为 SUCCESS；artifact `9034165649` 大小 `14273283` 字节，SHA-256 `761fa51e8f811d9fd85bf09b5ff798b9862613d7206185993dfff227cdaf160b`。AT2 施工 workflow / generator / fix script 均已清理；R3-07 继续为 `IN_PROGRESS`，下一 Atomic Task 进入 OpenAI Research Gateway execution。
- R3-07 Atomic Task 3 已正式关闭为 `DONE`。OpenAI Research Gateway execution 已从旧 `crates/application/src/openai_research.rs` 迁入按 artifacts / gateway / attempt audit / references / execution / validation 职责拆分的 `use_cases/research/openai_gateway/`；公共 `execute_p4_openai_research` / `OpenAiResearchCommand` 契约与错误语义保持不变，`ResearchGatewayAuditPort` 仅补齐既有 attempt-number offset 能力并由 `ActiveDatabase` 复用现有 PostgreSQL gateway records。迁移后无调用者的 `PreparedDatabaseConnection::transition_store` 与 `ResearchService::register_fact_pipeline_artifacts` 过渡桥接已移除。Windows hard gate run `31312515543` / job `93242388772` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 实施提交 `d818276bc93c1dfb8fd7c1c5fbab71a97e6cd5e2`。与该 clean tree SHA 完全一致的验收提交 `ad12e8d8d574ee3df8c3c1e83cf7705e718d0c49` 的 Public Platform CI run `31313043256` / Windows Automated job `93243752378` 已全部 SUCCESS，validation evidence upload 成功；artifact `9038223561` 大小 `14263377` 字节，SHA-256 `611f79b5b5789fce242d1429c1060415187f7f07f5253d03eb478bb7deed5bd3`。AT3 临时 workflow / generator / fix / CI trigger 均已清理；R3-07 继续为 `IN_PROGRESS`，下一 Atomic Task 进入 P4 Research worker，人工冲突裁决保持后续边界。
- R3-07 Atomic Task 4 已迁移 P4 Research worker：Research task 状态机、联网动态上下文、research run 恢复/执行、Partial/Blocked/Missed/ReadyToFreeze 收口与 freeze job 安排按职责拆入 `use_cases/research/p4_worker/`，并由 ResearchService 经既有 `PredictionWorkflowPort`、`ResearchArtifactPort`、`ResearchGatewayAuditPort`、Fact Pipeline 与 `JobQueuePort` 协作；根 `p4_orchestration.rs` 仅保留跨 Research/Prediction 的 job claim/complete/fail、dispatcher 与 worker loop，Prediction freeze 执行仍委托 PredictionService。`p4_workbench.rs` 的人工冲突裁决逻辑未迁移，仅将成功后的 Research 收口改为复用 ResearchService。公共 API、job type、状态语义、SQL、Schema、迁移、生产依赖与模型边界未改变。AT4 已正式关闭为 `DONE`：clean implementation `89a3c68ad50f7766d6db5d214c0fa5a39c1a6c72` 的同源码树验收提交 `75e76dea9a9ab7980f64374b4f27410ee221f8f9` 已通过 Public Platform CI run `31316144230` / Windows Automated job `93251603683`，validation evidence upload 成功；artifact `9039129007` 大小 `14264776` 字节，SHA-256 `fe3ef81501cb2c7d57302f8e03e9b0753f82138d8dc54356e39fc5d0ab31f68f`。Atomic Task 5 已正式关闭为 `DONE`：人工冲突裁决已迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，只经既有 Prediction/Job/Research Ports 协作；SQL、Schema、迁移、生产依赖、公共 `resolve_p4_conflict` 契约、截止时间、幂等、append-only 与状态机语义保持不变。Windows hard gate run `31318631427` / job `93257903560` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean implementation `c1549738041c0e6d3cb046c16fa193c47a14553f`。该 clean HEAD 的 Public Platform CI run `31319176935` / Windows Automated job `93259283555` 已整体 `SUCCESS`，validation evidence upload 成功；artifact `9039991499` 大小 `14210568` 字节，SHA-256 `e48cf0649449fc74488a60ea560a43a224c3f49fe3f9723ad1e039cc06f79d75`。AT5 临时 workflow / generator / fix / marker 已清理；R3-07 五个 Atomic Tasks 全部完成，Research Service 状态正式关闭为 `DONE`。

## R2-04 Lineup 与 Match

- Lineup 16 个类型和 Match 3 个类型已迁移到职责目录，旧 `lineup_chain.rs` 已删除。
- 根级类型路径、Serde、数据库映射、Application、Tauri DTO、公共命令和模型保护边界保持不变。
- 实施 workflow run `31151412918` 已通过；正式 Windows Automated run `31153982572` 已在最终实施提交 `0aafe42d7ed08f8e78d71d44ccb6f8f58c425999` 上通过，artifact `8984980586` 的 SHA-256 为 `1e7224f4e7f713b0339e97fd114fa6dea2c0b2ecc9400789613fe872d660938c`。R2-04 状态为 `DONE`，R2-05 为 `DONE`。

## R2-05 Prediction 与 Research

- Prediction 48 个类型与 Research 27 个类型已迁移到职责目录，旧 6 个职责混合源文件已删除。
- 正式 Windows Automated run `31171082098`、job `92842834091` 已通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。
- R2-05 状态为 `DONE`，R2-06 已开放。

## R2-06 Review 与 Postmatch

- Review 48 个类型与 Postmatch 11 个类型已迁移到 `review/`、`postmatch/` 职责目录，旧 5 个职责混合源文件已删除。
- staged 与 Windows Automated 验收已通过；原 PostgreSQL 数据库兼容链已在保留历史数据、不可变资产与 fail-closed 保护的前提下完成验证。
- R2-06 状态为 `DONE`，R2-07 已开放。

## R2-07 Analytics、Exchange、AI 与 Release

- Analytics 39、Exchange 54、AI Workspace 16、Release 9 共 118 个公共兼容类型已迁移到职责目录，旧 7 个职责混合根文件已删除。
- Windows 本机格式、Serde 17/17、365 类型清单与架构门禁已通过；用户确认 R2-07 状态为 `DONE`。

## R2-08 Domain 根出口收敛

- `crates/domain/src/lib.rs` 仅保留 17 个业务模块声明、365 个显式公共兼容类型 re-export 与 crate 内默认值兼容转发，不再承载领域定义或默认值实现。
- 根级 `pub use module::*` 已全部删除；新增确定性生成器和静态验证器，`verify:architecture` 会拒绝 glob 回归、遗漏/重复出口和根文件业务实现。
- 现有 `football_domain::TypeName`、Serde、数据库映射、DTO、模型保护边界和生产依赖未改变。Windows 本机完整阶段回归已通过，R2-08 状态为 `DONE`，R2 阶段已关闭。


## R3-08 Review / Postmatch / Analytics Services（DONE）

- Atomic Task 1 启动 Review Core 重写：将 `generate_match_review`、复盘列表/读取、可复盘比赛查询及能力候选查询/裁决 6 个公开 Application 职责迁入 `services/review/`、`use_cases/review/`，具体 PostgreSQL 调用仅由 `composition/adapters/review.rs` 实现 `MatchReviewPort`。公共 ApplicationService/Tauri 方法、DTO、SQL、Schema、配置、错误和用户可观察行为保持不变；Match Review Package、Postmatch、Analytics 不在本 Atomic Task 修改范围。

- R3-08 AT1 正式 CI 首次失败：Public Platform CI run `31328591975` 在 `verify:architecture` 的 Domain 类型清单漂移门禁停止；Review Core 业务代码、AT1 专项 hard gate 与完整 staging frontend/Rust 回归此前均已通过。根因是 Review owner 拆分新增/删除 Rust 文件后 `architecture/domain-type-inventory.json` 的源码调用面快照未同步。已按既有生成器重新计算清单，不修改 Domain 类型、Serde、数据库映射、公共接口、SQL、Tauri DTO 或模型保护区，也不放宽任何门禁。

- R3-08 AT1 第二次正式 CI run `31349803381` 已确认 Domain inventory 与既有 Application 架构门禁均通过，失败点收敛为 `scripts/verify-review-service.mjs` 的跨行普通字符串语法错误；修复仅改为显式 `\n` 输出，并新增 Node 22 `node --check` 前置验证，不修改 Review Core 业务代码、公共接口、SQL、Tauri DTO、模型边界或生产依赖。

- R3-08 AT1 Review Core 已正式关闭为 `DONE`：修复 Review 专项验证器语法后，Node 22 parse gate、完整 architecture、frontend/Rust 回归与正式 Public Platform CI run `31350677129` / Windows Automated job `93340716563` 全部通过；artifact `9049246515`，SHA-256 `62f0bcfdb8f83ce0a715de58ee14f8a0a87b1f3192c938e841d65400b8ecb7ef`。Review Core 业务接口、SQL、Tauri DTO、模型边界和生产依赖未改变；R3-08 下一 Atomic Task 为 Match Review Package。

- R3-08 AT2 Match Review Package 已完成实现并进入正式 CI 验收前 `VERIFYING`：旧 `crates/application/src/match_review_package.rs` 已删除，不保留转发壳；7 个既有 ApplicationService/Tauri 入口保持原名、参数和返回 DTO。导出组装、导入预检、确认/事实写入/正式复盘生命周期、共享快照/身份校验与 XLSX I/O 已拆入 `services/review/`、`use_cases/review/package/`；PostgreSQL 具体调用只位于 `composition/adapters/review.rs`。R3-01 冻结 `MatchReviewWorkflowPort` 已原样恢复，AT2 低层工作流状态持久化接口明确为 `MatchReviewPackageStatePort`，并新增 `MatchReviewPackageSourcePort` / `MatchReviewPackageFactsPort`；阵容写入复用 `LineupPort`，正式复盘复用 AT1 `MatchReviewPort`。初始 hard gate `31359686297` 已通过专项、Application Rust、完整 frontend/Rust 与 scope，但 final-tree 暴露 Domain inventory 未固化；fresh-checkout 诊断 `31360521486` 同时确认冻结 WorkflowPort 被误删导致 37-Port 契约下降为 36。最终 recovery run `31362128833` 已恢复 37-Port、刷新确定性 inventory，并通过 Review/Package 专项、完整 architecture、保护资产、Application Rust、完整 frontend/Rust、精确 scope、clean commit 与 clean-tree。当前未修改 `postmatch.rs`、`analytics.rs`、Tauri、PostgreSQL SQL/migration、Domain 契约、模型保护区、生产依赖或用户可观察业务语义；AT2 已在 canonical 提交 `23997cde7d3ba46db34fdd2e0577075555391986` 的 Public Platform CI run `31406717077` / Windows Automated job `93514736356` 成功后正式关闭为 `DONE`。

- R3-08 AT2 首次 canonical Public Platform CI run `31363234205`：完整 architecture 已通过，但 Windows Automated 在 `verify-stage-a-architecture.mjs` 因仍读取已删除的旧 `crates/application/src/match_review_package.rs` 而 `ENOENT` 停止；AT2 业务专项、37-Port、Domain inventory、Review/Package architecture 均已在同一 clean checkout 中通过。修复仅将 Stage-A 应用层断言读取源迁移到新 authoritative `use_cases/review/package/preview.rs` 与 `lifecycle.rs`，原 Preview/Confirm/Commit/Generate 门禁断言保持，不修改生产源码、公共接口、SQL/Schema、Tauri、Domain、Postmatch/Analytics、模型或依赖。另：AT2 canonical 发布过程中曾出现连接器误操作造成 README 瞬时被清空/缩减的文档提交；最终 HEAD 已通过 fast-forward 恢复为已验证完整 publication tree，生产源码从未被该误操作修改，相关异常 run 不作为验收证据。

- R3-08 AT2 第二次 canonical Public Platform CI run `31377744818` 已通过 architecture，但 Windows Automated 在 `verify-match-event-facts.mjs` 仍读取已删除的旧 `crates/application/src/match_review_package.rs` 时因 `ENOENT` 停止。修复提交 `23997cde7d3ba46db34fdd2e0577075555391986` 只将该验证器读取源迁移到 authoritative `use_cases/review/package/shared.rs`，OwnGoal 身份校验等原断言未删除或放宽，业务源码、公共 API、SQL/Schema、Tauri、Domain、Postmatch/Analytics、模型和生产依赖均未改变。PR clean CI run `31404194850` / job `93506381047` 已完整 `SUCCESS`；artifact `9069809674` 大小 `14156070` 字节，SHA-256 `0109171ab7791609e0c92dcdeba6797f651e1ea20175d9967a89f3bc7d256c37`。随后 canonical run `31406717077` / job `93514736356` 同样 `SUCCESS`，architecture、完整 Windows automated acceptance 与 evidence upload 均通过；artifact `9070811986`（`windows-automated-delivery-evidence-23997cde7d3ba46db34fdd2e0577075555391986`）大小 `14156296` 字节。完整 frontend、17 个截图回归视口、TypeScript/Vite、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Windows release 构建和 release runtime 启动验收均通过。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行。AT2 正式关闭为 `DONE`；R3-08 整体继续 `IN_PROGRESS`，Postmatch 与 Analytics 尚未实施。

## R3-08 AT3 Postmatch Service（DONE）

- 旧 `crates/application/src/postmatch.rs` 的 7 个公共 Application 职责已迁入 `services/postmatch/` 与 7 个独立 `use_cases/postmatch/<use-case>/`；旧 owner 删除，不保留转发壳。结算、证据评分与监控继续通过既有 `PostmatchSettlementPort` / `PostmatchMonitoringPort`，具体 PostgreSQL 调用只位于 `composition/adapters/postmatch.rs`。
- `settle_postmatch_review` 保留 Match Review Package 的 `SettleReview` 状态机门禁；AT3 只扩展既有 `MatchReviewPackageStatePort` 的按 review 读取与 settled 状态推进能力，继续复用 AT2 的唯一 workflow 状态 owner。
- 7 个 ApplicationService/Tauri 公共入口、参数、返回 DTO、错误语义和用户可观察行为保持不变；PostgreSQL SQL/migration/Schema、Tauri、Domain、Analytics、模型保护区和生产依赖未修改。受 owner 删除影响的 Postmatch contract、Review/Stage-A/Postmatch verifier 只迁移 authoritative 读取位置，原断言未删除或放宽。
- Windows hard gate：run `31419165233`：Postmatch 专项、Application Ports、完整 architecture、保护资产、rustfmt、Application check/tests、完整 frontend 与完整 Rust 回归均 `SUCCESS`。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试若未配置专用测试库继续保持 `ignored`，不得记为已执行。
- canonical 提交 `b551ac8acc4030f05d93100812b316469dc7ea83` 的 Public Platform CI run `31420385032` / Windows Automated job `93559423579` 已整体 `SUCCESS`：architecture、完整 Windows automated acceptance 与 validation evidence upload 均通过；artifact `9075930419` 大小 `14152359` 字节，SHA-256 `39fab48a3f2d62e1538276fb8f19fb811f1c72c79380de55897e12c82e39f582`。
- AT3 Postmatch Service 正式关闭为 `DONE`；该时点 Analytics AT4 随即开放。AT4 后续已完成并通过正式验收，R3-08 的最终当前状态为 `DONE`，R3-09 为 `READY`。


## R3-08 AT4 Analytics Service（DONE）

- 旧 `crates/application/src/analytics.rs` 已删除，不保留转发壳；唯一 `AnalyticsService` 通过 `services/analytics/` 聚合相邻用例，21 个公共 Analytics 用例均按 `use_cases/analytics/<use-case>/` 独立目录收口。后台 worker 与 Parameter Lifecycle 指标只作为明确内部协调/共享职责存在，不与公共用例混写。
- `AnalyticsPort`、`JobQueuePort`、`ParameterLifecyclePort` 保持分责；具体 PostgreSQL 能力仅由 composition adapters 实现，Service / Use Case 不直接依赖 SQLx、PostgresStore、PersistenceStore 或 PostgreSQL crate。公共 ApplicationService/Tauri 方法、参数、返回 DTO、错误语义、数据格式和用户可观察行为保持兼容；未新增生产依赖。
- Stage Regression run `31461057749` 已通过完整 `verify:frontend`、完整 workspace `verify:rust`（含 rustfmt、Clippy `-D warnings`、workspace tests）、protected assets 与最终 clean-tree。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试继续按安全设计保持 `ignored`，未记为已执行。
- clean publication commit `fcf5f3df29b477baf7e1c3aeebcf5ed6f459b8a8` 的 PR #16 Public Platform CI run `31461760837` 已全绿；canonical Public Platform CI run `31463538968` / Windows Automated job `93691679605` 同样整体 `SUCCESS`，architecture、Windows automated acceptance 与 validation evidence upload 均通过。canonical artifact `9091238488` 大小 `14092031` 字节，SHA-256 `ae08e7eeccb09f4b625885716391f88de2ad4ce3902d59608c4978bc63f29a72`。
- R3-08 四个 Atomic Tasks 已全部关闭，R3-08 正式为 `DONE`；下一节点 R3-09 Exchange / AI Workspace / Release Services 开放为 `READY`。


## R3-09 Exchange / AI Workspace / Release Services（IN_PROGRESS）

- AT1 Match Lineup / AI Match Package Exchange 已正式关闭为 `DONE`；8 个公共入口由唯一 `ExchangeService` 编排，AT1 的 canonical Public Platform CI run `31473739191` / Windows Automated job `93722595150` 已整体 `SUCCESS`。
- AT2 Spreadsheet Exchange 已正式关闭为 `DONE`：旧 `crates/application/src/spreadsheet.rs` 已删除；16 个既有 Spreadsheet 公共入口迁入 `services/exchange/` 与独立 `use_cases/exchange/<use-case>/`，Exchange facade / service / composition adapter 按 Match Lineup 与 Spreadsheet 两条职责拆分；R3-01 `SpreadsheetExchangePort` / `MonthlyWorkbookPort` 仅补齐真实能力并由 `ActiveDatabase` 适配。AT1 + AT2 共 24 个 Exchange 公共用例继续由唯一 `ExchangeService` 编排。
- ApplicationService/Tauri 方法、参数、返回 DTO、Excel/JSON 格式、错误文本及错误优先级保持兼容；球队完整资料包预检 JSON 继续无需数据库连接。PostgreSQL SQL/migration/Schema、Domain 公共契约、前端产品源码、模型保护区、配置与生产依赖未改变。6 个历史 Spreadsheet 专项验证器仅迁移到新 authoritative owner，原断言未删除或放宽；Domain inventory 使用项目官方生成器按最终源码刷新。
- Windows hard gate run `31512091398` / job `93848182746` 已通过最终 rustfmt、官方 Domain inventory、完整 `verify:architecture`、workspace Clippy `-D warnings` 与 workspace tests；Application tests 35/35、Domain Serde 17/17、Spreadsheet IO 12/12 等均无失败。最终 clean Public Platform CI run `31513432237` / Windows Automated job `93852598090` 在 clean HEAD `8719008ba63241d313fe02ce4328ee8d0a727e9d` 上整体 `SUCCESS`，validation evidence artifact `9110944721` 大小 `14040458` 字节，SHA-256 `f0628be97bbfb13a765da3e4799bd7e8abf549352e391cb79ef1111cb9522a9f`。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行；未执行破坏性数据库验证。AT2 已关闭为 `DONE`。
- AT3 AI Workspace 已完成 Application 层源码迁移并进入 `VERIFYING`：旧 `crates/application/src/api_workspace.rs` 已删除，不保留转发壳；12 个既有 ApplicationService 公共入口迁入唯一 `AiWorkspaceService` 与独立 `use_cases/ai_workspace/<use-case>/`。Session、Context、Operation、Presets、Attachments 已按职责拆分，Apply Operation 进一步拆为 orchestration / dispatch / payload / metadata；公共导出继续以原名称从 `lib.rs` 暴露。
- R3-01 `ApiWorkspaceSessionPort` / `ApiWorkspaceOperationPort` 已按真实持久化能力补齐并由 `ActiveDatabase` 适配；第一轮架构门禁发现 Port 暴露裸 `serde_json::Value` 后，改为显式 `SerializedApiWorkspaceOperationResult` 序列化边界，由 adapter 唯一还原 JSON，没有放宽 Ports 禁止裸 JSON 的门禁。两个历史 AI Workspace 相关验证器仅迁移 authoritative owner 读取位置，原断言未删除或弱化。
- AT3 Windows hard gate run `31553249625` / job `93980269528` 已整体 `SUCCESS`：官方 Domain inventory、完整 `verify:architecture`、37 个最小 Port trait、AI Workspace 专项、`cargo check --locked -p football-application`、Application tests 33/33、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 与 formatter scope 均通过；临时 hard-gate workflow 已自删除。PostgreSQL SQL/migration/Schema、Domain 公共契约、Tauri 产品源码、前端产品源码、Release/AT4、配置与生产依赖未改变。最终 clean Public Platform CI run `31553867879` / Windows Automated job `93982110497` 已整体 `SUCCESS`；validation evidence artifact `9125791161` 大小 `13985122` 字节，SHA-256 `f5662c9e11d1fbd9da12e39efce9b44645144fce33bf6fe674011c2b7c1af0a6`。PR #19 已合并，merge commit `3d925671ccd424e25965ba9409189f32f920bc4f`；AT3 正式关闭为 `DONE`，AT4 Release 开放为 `READY`。


## R3-09 AT4 Release Service（DONE）

- 已删除旧 `crates/application/src/release_acceptance.rs`，不保留转发壳；3 个既有 Release Acceptance ApplicationService/Tauri 入口继续保持原名、参数和返回 DTO，由唯一 `ReleaseService` 编排。
- `run_release_acceptance`、`list_release_acceptance_runs`、`read_release_acceptance_run` 已迁入 `services/release/` 与 `use_cases/release/`；运行验收进一步按请求校验、chain/performance/security/cost/release 检查、汇总和报告 SHA-256 哈希拆分职责。
- R3-01 既有 `ReleaseAcceptancePort` 已最小对齐真实 runtime-facts/persist/list/read 能力，具体 PostgreSQL 调用仅由 `composition/adapters/release.rs` 的 `ActiveDatabase` 适配；SQL、migration、Schema、Tauri 产品命令、前端产品行为、0.23.0 发布契约和生产依赖均未改变。
- 既有窗口 clamp、预算 Validation 文案、A–I/J 门禁、27 条迁移门槛、性能阈值、8 个不可变触发器门槛、成本预算语义、检查顺序、汇总字段和报告哈希字段保持不变；历史 `verify-release-acceptance.mjs` 与公开模型边界验证器仅迁移 authoritative owner 路径，原业务/保护断言未删除或放宽。
- 最终严格 hard gate run `31568170298` / job `94024298468` 已整体 `SUCCESS`：Release Service 专项、历史发布验收契约、完整 `verify:architecture`、官方 Domain inventory、rustfmt、`cargo check --locked -p football-application`、Application tests 33/33、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 全部通过。
- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未在本 hard gate 执行，未记为通过；本 AT 未执行破坏性数据库验证。临时 hard-gate / inventory-refresh workflow 已清理。
- 首轮最终 clean Public Platform CI run `31568684129` / job `94025845936` 中独立 architecture 已通过，Windows Automated 在完整 frontend 的 deterministic protected-assets 门禁停止：AT4 为迁移旧 Release owner 而合法修改了受保护的 `scripts/verify-public-model-boundary.mjs` 权威扫描路径，但 `architecture/protected-assets.json` 尚未同步该验证器的新指纹；这不是 Release 业务、编译或契约失败，且该轮未被记为通过。
- 按仓库既有 `chore(verify): refresh public-boundary fingerprint` 机制，只刷新该受保护验证器的 Git blob / fingerprint 与聚合 SHA；refresh workflow 在提交前实际执行 `verify-protected-assets-deterministic.mjs` 并通过。新 blob 为 `2e5adc250dba986b3b0441c7f26e762f8b9ef6f4`，fingerprint 为 `13f9ea8c98624e156208c836f837875f2a54104be15e6115f5e7f547d4f491e0`，聚合 SHA-256 为 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；保护文件集合、禁止私有资产规则和验证逻辑未放宽。
- 恢复后的最终 clean Public Platform CI run `31569072962` / Windows Automated job `94027017727` 已在无临时 workflow 的 HEAD `316f7b055817f85e086090bebc3613c577ceae9a` 上整体 `SUCCESS`；architecture、完整 Windows automated acceptance 与 validation evidence upload 均成功。artifact `9131083580` 大小 `13994757` 字节，SHA-256 `03b34d9ba85f0cd82b2a90f6b8cd2a5db3d8a59cb323b0439a7886c2cb6a0f87`。
- PR #20 已正式合并到阶段分支，merge commit `fd1b8eb2b726338ec588df7c1e8cd87ef202ff51`。AT4 正式关闭为 `DONE`；R3-09 四个 Atomic Task 全部关闭，R3-09 正式为 `DONE`，R3-10 ApplicationService 兼容门面开放为 `READY`。


## R3-10 ApplicationService 兼容门面（VERIFYING）

- R3-10 从 R3-09 已关闭阶段树开始实施，目标是让 `ApplicationService` 只保留稳定公共兼容入口、数据库 session 获取和单一 Service/Use Case 委托；公共方法名、参数、返回 DTO、Tauri 调用面、错误语义和数据库行为保持不变。
- Atomic Task 1 删除旧根级 `crates/application/src/p4_orchestration.rs`（136 行），建立 `services/p4_orchestration/` 与 `use_cases/p4_orchestration/`；后台 job claim/complete/fail 通过新增最小 `P4OrchestrationQueuePort` 穿越边界，P4 worker 运行状态由 `P4OrchestrationService.running` 单一持有。Application Port inventory 从 37 个增至 38 个最小 trait，仍为 15 个职责域。
- AT1 同时删除已失去调用方的 `active_store`、私有 freeze/research worker facade helper 与 `PersistenceStore` 内部 re-export，不使用 `#[allow(dead_code)]` 掩盖；Prediction/Research/P4 freeze、Application Composition、state ownership 与 Port 历史验证器均迁移到新 authoritative owner，原断言未删除或放宽。
- Atomic Task 2 新建 `use_cases/application_facade/`，将 bootstrap、database connect/initialize/reset 生命周期编排从兼容 facade 下沉；Research 多 Port access 组装进入 `ResearchService` 单-session compatibility 边界，Prediction recent-run `ModelRunListItem` 转换进入 `services/prediction/compatibility.rs`。Database bootstrap/facade 分别净减少 67/60 行，Prediction facade 净减少 15 行，Research facade 净减少 27 行；新增 lifecycle/bootstrap use cases 共 155 行并按职责分文件。
- 新增 `scripts/verify-application-service-facade.mjs` 并接入 `verify:architecture`，扫描当前 22 个 `impl ApplicationService` 文件，拒绝根对象业务异步方法/运行状态、facade 中具体 PostgreSQL/SQLx/HTTP 实现、Database 生命周期编排、Research 多 Port 组装、Prediction DTO 映射与 P4 job orchestration 回流。
- AT1 最终严格 hard gate run `31591061641` / job `94095867258` 已整体 `SUCCESS`：rustfmt、官方 Domain inventory、P4 freeze/Prediction/Research、38-Port、Application Composition、完整 architecture、Application check、Application tests 33/33 与 Clippy `-D warnings` 全部通过；AT1 提交为 `2bc98df768ed36bfe9e9712b4eae9cc54f83b1c6`。
- AT2 最终严格 hard gate run `31592634518` / job `94100834403` 已整体 `SUCCESS`：新的 ApplicationService 兼容门面专项、Database/Competition-Rules/Research/Prediction/Application Composition、完整 architecture、Application check、Application tests 与 Clippy `-D warnings` 全部通过；AT2 提交为 `dd0aa8e32c8459acbbb68805b6fa099d92d17f91`。实施过程中被历史 owner 路径或源码变换精确匹配阻塞的 recovery runs 均在提交前硬停，未把失败树提交为生产结果。
- 文档后的完整 `verify:frontend` 首轮发现 `verify-database-reset.mjs` 仍从 Database facade 检查自动重连/P4 worker，已将原断言迁到 `database_lifecycle/reset.rs` 与 P4 facade；恢复 run 随后通过该契约与 17 个截图回归，但因临时 runner 未先执行仓库 Node setup，在 `typescript` 包解析处停止。该环境失败未记为通过，后续使用仓库既有 `npm run setup` 后重跑完整 frontend。
- 相对 R3-10 基线，当前源码/验证变更共 42 个文件、`+740/-389`；旧 P4 单文件 owner 清零，新增职责文件均保持小型化。未修改 PostgreSQL SQL/migration/Schema、Tauri 产品命令、前端产品源码、模型实现/私有资产、配置或生产依赖。
- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未在 AT1/AT2 hard gate 执行，未记为通过；未执行破坏性数据库验证。PR #21 当前保持 Draft / Open / 未合并；R3-10 在 clean Public Platform CI 与正式合并/收口前保持 `VERIFYING`，不得提前标记 `DONE`。

R8-07 精确 `58b390a` / [Windows run `37796909083`](https://github.com/uniquenesssta/123/actions/runs/37796909083) 全 SUCCESS：Application 92/Persistence 148、七项新增边界测试、17 视口、fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条3操作通过。详见 [07 完成记录](docs/modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md)。真实 PG/Full/私有固定回归与继承删除风险仍最终新库待验；07 DONE，08 按用户指令开始，当前状态见 [R8 索引](docs/modular-rewrite/R08-prediction-p4-orchestration/README.md)。


## R9-04 精确 Windows 收尾（2026-10-11，DONE）

精确修复提交 `cb3d2cadfb8caa0c7dc3f4eb7c34faedd49f79bc` / tree `8da6d35f383939d61ef4278b6d575f8c8eca1e34`，同一分支rewrite/r9-research-ai-backend，[Public Platform CI run `38055651688`](https://github.com/uniquenesssta/123/actions/runs/38055651688) 与 [Windows job `114223464446`](https://github.com/uniquenesssta/123/actions/runs/38055651688/job/114223464446) 均 **SUCCESS**；完成于2026-10-10 21:53:28北京。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及运行日志覆盖/错误扫描通过；实际Gateway60/contract25、Application126/Persistence175均通过，首轮失败契约已执行成功。

交付artifact `11671897216` / `windows-automated-delivery-evidence-cb3d2cadfb8caa0c7dc3f4eb7c34faedd49f79bc`，14,040,701 bytes，SHA-256 `b8928c736b0d462199c251a39ff1a9bb821cc23242e36874dcb86d6558275dd3`。验收报告 `D:\a\123\123\logs\windows-acceptance-20261010-132633.json`，过程记录同名.txt；Windows制品为原0.23.0 x64 zh-CN MSI和NSIS setup。原18 broad PG仍ignored；真实数据库、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险仍最终封包新库待验，ignored与公共stub不计PASS。

首轮失败由该修复自身精确Windows关闭。04 **DONE**；用户“收尾04开始05”授权同一R9分支05 **READY**，06～11BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，源码/门禁/依赖保持，同源码树复用上述Windows成功。此前VERIFYING叙述为实施历史，不创建R9阶段完成记录。


R9-05 Formal Response/Schema/Citation已实施、`VERIFYING`，基线53e49f8，同一R9分支。正式严格解析/解码/公开输出校验/引用关联各归唯一职责，共用引用wire/用量/错误映射仍唯一且兼容调用方不依赖Formal；URL/domain/time原政策留06。12A/12M/0D，无公开API/DTO/Schema/配置/依赖/DB/UI/生产政策变化，17原函数/657生产literal/原60unit和25contract保持。新增22unit/4contract，源码预期Gateway82/contract29、Application126/Persistence175须自身Windows；83/83静态/完整architecture/Rustfmt/保护资产/171命令/46迁移18PG静态及6破坏探针通过。 详见 [05实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-05-formal-response-schema-and-citation.md)。CI开始后停止轮询，06～11BLOCKED；真实PG/Full/XLSX/私有固定回归及继承删除风险仍最终新库待验。

## R9-05 精确 Windows 收尾（2026-10-11，DONE）

精确提交 `aa70d2b4464e86254eecfcace11621d1b03aad51` / tree `e3581a2ee6264999ab5c3afb37be695e70f6e9bb`，同一分支rewrite/r9-research-ai-backend，[Public Platform CI run `38072987214`](https://github.com/uniquenesssta/123/actions/runs/38072987214) 与 [Windows job `114274071102`](https://github.com/uniquenesssta/123/actions/runs/38072987214/job/114274071102) 均 **SUCCESS**，2026-10-11 02:04:42北京完成。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/客户端启动及运行日志覆盖/错误扫描通过；实际 **Gateway82 / contract29 / Application126 / Persistence175** 均通过。

交付artifact `11677353373` / `windows-automated-delivery-evidence-aa70d2b4464e86254eecfcace11621d1b03aad51`，14,042,360 bytes，SHA-256 `ae05f14fdbdbe18f49bc34de912ab2b0e63ef928f74ff5409db585f6d0cbbb47`。报告 `D:\a\123\123\logs\windows-acceptance-20261010-174601.json`，过程记录同名.txt。原18 broad PG仍ignored；真实PG/历史四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险保持最终封包新库待验，ignored与公开stub不计PASS。

05 **DONE**；用户2026-10-11“收尾05开始06”授权06 **READY**，07～11BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，源码/门禁/依赖保持，复用上述同源码Windows成功；此前VERIFYING叙述属于实施历史，不创建R9阶段完成记录。

R9-06 Source/Time Policy已实施、`VERIFYING`，基线b6636fe，同一R9分支。URL/domain与禁用字段/内容归source_policy，三可选事实时间归time_policy；Schema保留身份/主体/值形状/覆盖，Citation保留关联，删除旧root validation。4A/11M/1D、16文件；原公开API/DTO/Schema/SourcePolicy配置/协议/依赖/DB/UI/错误/生产策略不变。原82unit/29contract保留，新增8unit/2contract；源码预期Gateway90/contract31、Application126/Persistence175须06自身Windows。83/83静态、完整architecture、Rustfmt、657生产literal/原规则与组合等价、保护资产/171命令/46迁移18PG静态、7破坏探针均通过。 详见 [06实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md)。Windows精确开始后停止轮询，07～11BLOCKED；真实PG/Full/XLSX/私有固定回归及继承删除风险仍最终新库待验。

R9-06 初版f714acb的Windows run38108158888/job114377958564因SourcePolicy字段提示触发Clippy uninlined_format_args失败。现将普通变量格式参数改为内嵌field_key，错误文字/类别/恢复建议/政策保持；原源码卫生门禁新增该回归守卫（旧源码拒绝、修复后通过），Domain使用摘要同步。83/83现有静态、完整architecture、Rustfmt和差异/提示等价通过；Gateway90/contract31仍是待本轮Windows验证的预期，06 VERIFYING、07～11BLOCKED。 详见 [06修复记录](docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md#首轮-windows-失败修复2026-10-11)。

R9-06 第二轮 `3981b9d` / [Windows run38109741446](https://github.com/uniquenesssta/123/actions/runs/38109741446) / job114382644048 已通过fmt/Clippy，Gateway单测89/90，新增域名测试错误地把大写WWW配置当作去前缀后的apex。已按原“trim→去*.→去小写www.→小写”顺序修正夹具，补大写/混合前缀拒绝与保留前缀的子域成功断言；生产代码保持。原兼容传输门禁锁定该顺序及夹具，三反向探针均拒绝恢复。 当前节点4A/12M/1D共17文件；本轮8M，06仍VERIFYING、07～11BLOCKED，待修复自身Windows完整结果。详见 [06单测修复记录](docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md#第二轮-windows-单测失败修复2026-10-11)。

R9-06 第三轮43dba5f / [Windows run38135480448](https://github.com/uniquenesssta/123/actions/runs/38135480448) 的Gateway单测90/90通过，contract30/31；新增后台/恢复契约误把完整URL排序后首条引用来源当作搜索来源。已改为比较包含#citation、#search的完整有序列表，保留其他断言与全部生产源码；原兼容门禁补排序/完整列表守卫，四反向探针拒绝恢复。 本轮8M，静态83/83、architecture、Rustfmt通过；06仍VERIFYING、07～11BLOCKED，待修复自身Windows。详见 [06契约修复记录](docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md#第三轮-windows-契约失败修复2026-10-11)。

精确修复提交d996a4d4bea34d53486b14a642fa1f655478bde7 / tree2584ce620b3d9f53595f8d4d49da0b3f5ae42df0，[Windows run38145691538](https://github.com/uniquenesssta/123/actions/runs/38145691538) / job114488432087均SUCCESS，2026-10-11 22:30:11北京完成。Gateway90/90、contract31/31、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7条记录、3个完成操作）。 06 DONE；用户2026-10-12“收尾06开始07”授权07 READY，同一rewrite/r9-research-ai-backend分支，08～11 BLOCKED、R9 IN_PROGRESS。本次只同步五既有文档，使用[skip ci]复用上述同源码Windows成功；此前VERIFYING为实施历史，无R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。

R9-07 Plain Chat Responses已实施、`VERIFYING`，基线9ddb27a，同一R9分支。普通Responses最小载荷/原始文本解码归plain_chat/responses/{request,response}.rs，两个mod只登记/显式导出；共用校验/预算/许可/取消/重试/provider/refusal/回退/用量/attempt仍原owner，原Chat分支留08。5A/11M/0D、16文件：新增上述四Rust模块及07节点记录；修改根README、TESTING、R9任务书/索引、Domain清单、lib/client/response、原contract及两个既有兼容门禁，整文件删除/移动无。原API/DTO/Schema/配置/profile/DB/UI/协议/依赖/错误/生产政策保持。原90unit/31contract全文保持，新增8unit/4contract，预期Gateway98/35、Application126/Persistence175待07自身Windows。83/83静态、完整architecture、Rustfmt、完整共用client/response逆迁移等价、字符串多重集/保护/171命令/46迁移18PG静态与7破坏探针通过。报告r907-static-checks.json/r907-architecture.log/r907-equivalence.json/r907-negative-probes.json/r907-review.json；真实文件、23职责与兼容细节见07记录。Context7/官方serde_json1.0.150源码/Mermaid/足球状态同步；Windows精确开始后停止轮询，预计20～30分钟。06 DONE、07 VERIFYING、08～11BLOCKED，R9 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041风险仍最终封包新库待验，不把ignored/公共stub记PASS。 详见 [07实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-07-plain-chat-responses.md)。

## R9-07 完成与08授权（2026-10-12）

精确07提交3dbfde3c7246a08e91ad8bde6dd324a774f5a4a9 / tree38d8ce7f358eb45e8ce73e1414013b46f7462ced，[Windows run38155453893](https://github.com/uniquenesssta/123/actions/runs/38155453893) / job114517387791全SUCCESS，2026-10-12 00:54:23北京完成。Gateway98/98 unit、35/35 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过。

证据artifact11709500697，14,040,090字节，SHA-256 10c5237fa3aa7713212cfdcd7647b927bd61d60ed5f82878ce03b190c16f8c01；报告windows-acceptance-20261011-162740.json及同名.txt。07新增8unit/4contract全部执行通过。

07 DONE；用户2026-10-12“收尾07开始08”授权08 READY，同一rewrite/r9-research-ai-backend分支，09～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码树与上述成功一致，先前VERIFYING叙述为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。

R9-08 Plain Chat Chat Completions已实施、`VERIFYING`，同一R9分支，基线d065cff。原model/messages载荷与首choice文本提取迁入plain_chat/chat_completions/{request,response}.rs；mod仅登记/显式导出，旧内联与旧函数移除，共用执行/错误/回退/预算/取消/用量/attempt保持。4A/11M/0D、15文件：新增三Chat模块及08节点，修改client/plain_chat根mod/response、原contract、两个既有兼容门禁、Domain清单和README/TESTING/任务书/索引，删除/移动无。原98unit/35contract保留，新增8unit/4contract；83/83静态、完整architecture、Rustfmt、逆迁移等价与8探针通过，106unit/39contract待08自身Windows。API/DTO/Schema/profile/配置/依赖/DB/UI/模型/生产政策保持；07 DONE、08 VERIFYING、09～11 BLOCKED、R9 IN_PROGRESS。 完整文件、兼容/证据与最终新库待验见 [08实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-08-plain-chat-chat-completions.md)。

## R9-08 完成与09授权（2026-10-12）

精确08提交7fd7e1e0eeec5f8b1e3a6a516a32ba21e9f067f9 / treea029512756ee7524367ff1c3daa4ba0ca293688c，[Windows run38191340582](https://github.com/uniquenesssta/123/actions/runs/38191340582) / job114624551223全SUCCESS，2026-10-12 09:45:37北京完成。Gateway106/106 unit、39/39 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11722500937，14,041,703字节，SHA-256 791a0a3414641bdbc70b524c69be0ef16e4b3e57a93078b2b4809bcd2844ff29；报告windows-acceptance-20261012-011909.json及同名.txt。08新增8unit/4contract全部执行通过。

08 DONE；用户2026-10-12“收尾08开始09”授权09 READY，同一rewrite/r9-research-ai-backend分支，10～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。

R9-09 Session Persistence已实施、`VERIFYING`，同一R9分支，基线06ecdd2。8会话用例移动到Application sessions且Service直连出口；9个PG会话方法迁入adapters/ai_workspace/sessions，查询、消息、回答事务、文件、用量、Row映射与载荷校验职责明确，原提案状态流转/校验/映射唯一owner保持。公开接口/DTO/SQL绑定/事务审计顺序/配置/DB迁移/模型/Gateway/UI/依赖不变。新增4Application/4Persistence unit；83/83静态、完整architecture/Rustfmt、14原PG方法字节等价与8探针通过，Application130/Persistence179、Gateway106/39待自身Windows。08 DONE、09 VERIFYING、10～11 BLOCKED，R9 IN_PROGRESS。 完整A/M/D/8移动、验证与最终新库待验见 [09实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-09-session-persistence.md)。

## R9-09 完成与10授权（2026-10-12）

精确09提交4ed656d51fac604cf74665009cd340f4d7ddbc59 / treee1ea669d51e98fe11797fc90213df5bb3ccbf48b，[Windows run38196834750](https://github.com/uniquenesssta/123/actions/runs/38196834750) / job114641262453全SUCCESS，2026-10-12 11:10:30北京完成。Application130/130、Persistence179/179、Gateway106/106 unit和39/39 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11724985136，14,040,086字节，SHA-256 00fdea25a74bfc9fc99bf0031a03161e2f2eb5419f158065c4a5f43c335550b7；报告windows-acceptance-20261012-024340.json及同名.txt。09新增4Application/4Persistence unit均执行通过，未增加/运行真实PG专项。

09 DONE；用户2026-10-12“收尾09开始10”授权10 READY，同一rewrite/r9-research-ai-backend分支，11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG会话事务/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。


## R9-10 取消注册表（2026-10-12）

R9-10 Cancellation Registry已实施、`VERIFYING`，基线8d80ee6，同一R9分支。Application唯一Registry及不可克隆Lease管理活动ID/token，Tauri已切换且旧map/锁操作移除；重复ID不覆盖原token并在profile/会话副作用前拒绝，准备失败/网关结束/任务abort/unwind自动清理。原命令/DTO/错误文案/协议/profile/SQL事务/日志载荷/UI/模型/版本/依赖保持；仅新增两个Rust类型导出与明确取消生命周期修正。8新增Application unit待自身Windows，预期138/179与Gateway106/39；83/83静态、完整architecture/Rustfmt、逆切等价和8破坏探针通过。09 DONE、10 VERIFYING、11 BLOCKED、R9 IN_PROGRESS。 完整4A/12M/0D清单、工具/回退和最终新库待验见 [10实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-10-cancellation-registry.md)。


## R9-10 完成与11授权（2026-10-12）

精确10提交07da8d9a70726fe707fe18fb50e6cbf49d402d49 / tree0ae872b0e41609a989ca4a8fbdc2aaca85cef34e，[Windows run38213432437](https://github.com/uniquenesssta/123/actions/runs/38213432437) / job114692567611整体SUCCESS，2026-10-12 15:05:56北京完成。Application138/138、Persistence179/179、Gateway106/106 unit和39/39 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。10新增8个生命周期unit均实际通过，包括并发同ID、任务abort和panic清理。

证据artifact11731842587，14,039,211字节，SHA-256 b610fb2b906fd93db285eab9c0908dff0cd3e37df73e25f88f16fda9fc798366；报告windows-acceptance-20261012-063850.json及同名.txt。

10 DONE；用户2026-10-12“收尾10开始11”授权11 READY，同一rewrite/r9-research-ai-backend分支、R9 IN_PROGRESS，未进入R10。本次五既有文档[skip ci]收尾，源码同精确成功提交；先前VERIFYING为实施历史。自身11完整Windows未完成前不创建R9阶段完成记录。真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub不计PASS。


## R9-11 运行诊断（2026-10-12）

R9-11 Runtime Diagnostics已实施、`VERIFYING`，基线6f471d0，同一R9分支。Gateway diagnostics唯一持有尝试DTO、异步sink/default、JSON指纹/响应ID/毫秒耗时，根出口与实际client已切换，旧owner正文移除；原八条审计→尝试收集→熔断/重试顺序保持。56公开导出名字、DTO/错误/profile/协议/SQL事务/日志/UI/模型/版本/依赖保持，无新共享状态或IO。新增6 Gateway unit/2双协议contract待自身Windows，预期112/41、Application138/Persistence179；83/83静态、完整architecture/Rustfmt、逐字等价与8破坏探针通过。10 DONE、11 VERIFYING、R9 IN_PROGRESS，R10不启动。 完整7A/11M/0D、工具/回退与最终新库待验见 [11实施记录](docs/modular-rewrite/R09-research-ai-backend/R09-11-runtime-diagnostics.md)。


## R9-11 双协议审计测试修正（2026-10-12）

R9-11 首轮Windows run38218231140 / job114707912576在精确5ff9164失败：Gateway112/112 unit通过，contract39/41，新增plain/structured两测试把协议切为Chat Completions但保留默认Responses显式端点，构造网关即报“兼容 API请求端点与所选协议不一致”。已补两夹具逐协议端点match；原运行诊断门禁锁定两协议覆盖、构造前同步与正确映射，4负向探针均拒绝且恢复。生产源码/公开接口/配置验证/协议/SQL/日志/模型不变，测试数量112/41不变；83/83静态、完整architecture/Rustfmt与夹具逆切等价通过。11仍VERIFYING，R9 IN_PROGRESS，须修复提交自身完整Windows，不启动R10。 完整8M清单与失败证据见 [11节点记录](docs/modular-rewrite/R09-research-ai-backend/R09-11-runtime-diagnostics.md)。


## R9-11 精确 Windows 完成与阶段收尾（2026-10-12）

精确修复提交b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5，[Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812整体SUCCESS，2026-10-12 16:29:01北京完成。Gateway112/112 unit、41/41 contract，Application138/138、Persistence179/179；两项plain/structured双协议审计失败契约已实际通过，首轮39/41失败由本修复自身关闭。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描通过（7记录、3完成操作）。

证据artifact11735687461，14,038,470字节，SHA-256 3e28db7fddd900f59e4655c058df08e7d69c04feaab92e02e641f99900c05a68；报告windows-acceptance-20261012-080203.json及同名.txt。

用户2026-10-12“收尾11”：11 DONE；R9全部01～11及Windows Automated阶段出口DONE，阶段完成记录与实际总表已建立。仅R10-01 Review Package Workflow READY，02～13 BLOCKED；未开始R10实现、未建立第二分支。本次纯文档[skip ci]，源码、门禁与清单同精确成功提交。真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub不计真实PASS；此前VERIFYING/失败叙述保留为实施历史。

R9实际职责、全阶段A/M/D与8对移动、节点证据、兼容修正/最终新库待验和回退见 [R9阶段完成记录](docs/modular-rewrite/R09-research-ai-backend/R09-stage-completion.md)；下一任务见 [R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


### R10-01 Review Package Workflow（2026-10-12，VERIFYING）

R10-01 Review Package Workflow 已实施、`VERIFYING`；唯一阶段分支 `rewrite/r10-review-analytics-release-backend`，从 R9 收尾 `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立。Application workflow 独立持有编排、身份门禁与人工确认元数据，ReviewService 五入口已切换；PG 十个工作流方法迁入对应适配器，SQL、审计及事务正文逐字保持，旧两 owner 删除。原 Domain 八状态/九步界面、171 命令、Ports/DTO/错误/模型保护区保持；不自动接受候选或晋升参数。新增 8 项原 Application unit 待自身 Windows，预期 Application146 / Persistence179 / Gateway112 unit、41 contract。84/84 既有源码门禁、完整架构、Rustfmt、等价复核与 8 破坏探针通过；CI 开始后停止查询。R10 IN_PROGRESS，02～13 BLOCKED，真实 PG/历史四项/账本并发回滚/有效 XLSX/Windows Full/私有 P4/P7 固定回归及继承 model.runs/0041 删除风险仍最终封包新库待验。 详见 [01记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-01-review-package-workflow.md) 与 [R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


## R10-01 精确 Windows 完成与02授权（2026-10-12）

精确提交 `838e39ea35355662c192769a358c56a06131a6ed` / tree `7f42a75620aa8eb44441670a69a379b0c1a7830b`，[Public Platform CI run38229119264](https://github.com/uniquenesssta/123/actions/runs/38229119264) / Windows job114743919870全SUCCESS，run于2026-10-12 17:44:15北京完成。实际Application146/146（新增8 workflow unit均成功）、Persistence179/179、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描全部通过。报告windows-acceptance-20261012-092001.json。

artifact11739056741，14,041,808字节，SHA-256 `2831911b9159911fb03ebc9bf894e03f00a4a01c73001b9ffddc8f9a289a1fa7`。01自身完整成功后标记DONE；用户2026-10-12“收尾01开始02”授权，02 Structured Match Facts READY，同一rewrite/r10-review-analytics-release-backend分支，03～13 BLOCKED，R10 IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同上述成功提交；后续02必须取得自身完整Windows证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验；ignored/公开unavailable stub/静态指纹不计这些真实项目PASS。此前01 VERIFYING及预期计数保留为实施历史，不替代当前DONE状态。


### R10-02 Structured Match Facts（2026-10-12，VERIFYING）

R10-02 Structured Match Facts 已实施 `VERIFYING`，基线ab43aee，继续唯一分支rewrite/r10-review-analytics-release-backend。Application真实事实阶段迁入facts coordinator，ReviewService与组合工作流直接复用；PG facts目录独占校验、事件身份、事务写入、查询/摘要、行映射和事实提交，旧review.rs的14职责及事实写入正文移除。原比赛锁、修订归属、SQL/审计/同事务上下文与十个公开方法签名保持，不改变报告/候选/参数人工门禁。新增8 PG unit待自身Windows，预期Persistence187 / Application146 / Gateway112 unit、41 contract；84/84静态、完整架构、19 Rust文件Rustfmt、等价与8破坏探针通过。01自身Windows已DONE，02 VERIFYING、03～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7回归与继承model.runs/0041删除风险仍最终新库待验。 [02记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-02-structured-match-facts.md) 与 [R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)维护实际边界/文件与状态。


## R10-02 精确 Windows 完成与03授权（2026-10-12）

精确提交 `1ad6e68042e7749caf6671b2eb4756077dbfa17b` / tree `822588ffe13bd392fbe2851af389eb111b2391ad`，[Public Platform CI run38245932270](https://github.com/uniquenesssta/123/actions/runs/38245932270) / Windows job114799659672全SUCCESS，run于2026-10-12 20:18:27北京完成。实际Application146/146、Persistence187/187（新增8事实unit均成功）、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描通过。报告windows-acceptance-20261012-115847.json。

artifact11746489380，14,041,992字节，SHA-256 `378486f022954e5ee7044258e31b09bcd83c8c484c55af6fff11ef71ef1af3d1`。02自身完整成功后DONE；用户2026-10-12“收尾02开始03”授权03 Review Engine Adapter READY，同一rewrite/r10-review-analytics-release-backend，04～13 BLOCKED，R10 IN_PROGRESS。本次只五既有文档收尾[skip ci]，源码同上述已验提交；03必须取得自身精确Windows完整证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub/静态指纹不计真实PASS。此前02 VERIFYING与预期计数保留为实施历史。


### R10-03 Review Engine Adapter（2026-10-12，VERIFYING）

R10-03 Review Engine Adapter已实施 `VERIFYING`，基线93a008e，继续唯一rewrite/r10-review-analytics-release-backend。Application普通/资料包两条实际生成入口迁入engine_adapter，Service与组合工作流已切换；PG六个引擎职责迁入准备、上下文、原计算事务和报告持久化，事实提交复用同一完整准备。纯review-engine源码/公式、原SQL/错误/JSON载荷、公开签名、事务/审计/提交后回读及pending候选人工门禁保持。原8 Application测试正文保持，其中2生成测试迁入新owner、共用原fixture；新增5入口unit待Windows，预期Application151 / Persistence187 / Gateway112 unit、41 contract。84/84静态、完整架构、18 Rustfmt、等价及8负向探针通过；01～02 DONE，03 VERIFYING，04～13 BLOCKED、R10 IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。 [03记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md)及[R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)维护实际边界/清单/回退。


### R10-03 自身Windows验收完成（2026-10-12）

03精确 `99b15ad` / [Windows run38259438025](https://github.com/uniquenesssta/123/actions/runs/38259438025) 全SUCCESS并DONE，Application151/Persistence187/Gateway112 unit、41 contract及新增5项通过。02已收尾、03已实施；01～03 DONE，04～13 BLOCKED，R10 IN_PROGRESS。本次仅五文档补记 [skip ci]；原源码和图中事务链路保持，完整证据与最终新库延期见 [03记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md)。此前VERIFYING叙述为实施历史。


### R10-04 人工能力候选事务（2026-10-12，VERIFYING）

候选列表/详情/映射和人工决定收拢到唯一PG模块，原锁、过期判断、拒绝无能力观察、决策/审计同事务和提交后回读保持；公开接口、SQL、模型公式和参数人工门禁不变。7A/8M/0D，新增8原PG unit待Windows；84/84静态、完整架构、8Rustfmt及等价/负向通过，预期PG195/App151，不能记作动态PASS。03已收尾DONE，04 VERIFYING、05～13 BLOCKED、R10 IN_PROGRESS；详细变化、验证及最终新库延期见[04记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-04-ability-candidate-transaction.md)和[阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


## R10-04 精确 Windows 完成与05授权（2026-10-13）

精确提交 `221d21134c46fa45e280230238153694be415aa1` / tree `5f8234f89eb86a6aeb7c19dd8bfc1376385d31b4`，[Public Platform CI run38271387412](https://github.com/uniquenesssta/123/actions/runs/38271387412) / Windows job114886215505 全SUCCESS，run于2026-10-13 00:03:32北京时间完成。实际Application151/151、Persistence195/195（新增8候选unit均成功）、Gateway112/112 unit和41/41 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、客户端启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-153709.json及同名.txt。

证据artifact11758811423，14,041,670字节，SHA-256 `296e86e05c1a649bfab6cda6cb9a938f8b83d75c4f2b19c71d80a3ad404fec59`。04自身完整成功后DONE；用户2026-10-13“收尾04开始05”授权05 Postmatch Settlement READY，继续唯一rewrite/r10-review-analytics-release-backend，06～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，后续05须自身完整Windows证据。此前04 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。无R10阶段完成记录。


### R10-05 赛后结算（2026-10-13，VERIFYING）

结算编排、readiness/列表和PG幂等事务及直接证据/样本支持迁入唯一settlement模块，三个旧App用例删除；原指纹/键、SQL/错误/版本、单事务审计/提交后回读、资料包独立Port推进及参数人工门禁保持。15A/14M/3D，新增8App/4PG原unit待Windows（预期159/199）；84/84静态、完整架构、18Rustfmt、等价/负向通过。04已收尾DONE，05 VERIFYING、06～13 BLOCKED、R10 IN_PROGRESS；完整文件/验证与最终新库延期见[05记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-05-postmatch-settlement.md)和[阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


## R10-05 精确 Windows 完成与06授权（2026-10-13）

精确提交 `fe3529f93ac1ec6777ccff53da73c0836d310a7f` / tree `950654f5357fb05a162fe4cd1449cb071aa9fd79`，[Public Platform CI run38277864089](https://github.com/uniquenesssta/123/actions/runs/38277864089) / Windows job114907643831 全SUCCESS，run于2026-10-13 00:58:51北京时间完成。实际Application159/159、Persistence199/199、Gateway112/112 unit和41/41 contract；新增8App/4PG unit与迁移的原timeliness unit通过，前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-163146.json及同名.txt。

证据artifact11761067078，14,042,191字节，SHA-256 `65186f8312bf643c3b6547cc36cc7a61669e23f735a82be172128584ecb382f2`。05自身完整成功后DONE；用户2026-10-13“收尾05开始06”授权06 Evidence Scoring READY，继续唯一rewrite/r10-review-analytics-release-backend，07～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，06须自身完整Windows证据。此前05 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。无R10阶段完成记录。


### R10-06 证据评分（2026-10-13，VERIFYING）

证据读取/人工判定/评分与冻结队列归唯一owner，旧App两入口/root证据职责/旧队列位置删除；原说明字节边界、公式/NotVerifiable空分、不可变决定/审计及提交后回读保持，不自动判定或晋升参数。12A/13M/3D，新增6App/8PG原unit待Windows（预期165/207）；84/84静态、完整架构、16Rustfmt、等价/负向通过。05已DONE，06 VERIFYING、07～13 BLOCKED；完整变化、验证及最终新库延期见[06记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md)和[阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


R10-06 已收尾DONE：精确1ed6dbe / [Windows run38290328302](https://github.com/uniquenesssta/123/actions/runs/38290328302) 全SUCCESS，Application165/Persistence207及新增14测试、17视口/打包/启动通过。用户授权07供应商评分，08～13 BLOCKED；完整证据见[06记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md)。


### R10-07 供应商评分（2026-10-13，VERIFYING）

原供应商聚合、评分/指纹、不可变快照写入与最新读取收拢唯一PG owner，App综合监控保持；原公式/分区/顺序/逐组pool写入政策不变。7A/9M/0D，新增9PG原unit待Windows（预期App165/PG216）；84/84静态、完整架构、8Rustfmt、等价/负向通过。06已DONE，07 VERIFYING、08～13 BLOCKED；完整变化、证据与最终新库延期见[07记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)和[阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


R10-07 首轮 `688892d` / [Windows run38296053769](https://github.com/uniquenesssta/123/actions/runs/38296053769) 因提取后的快照键格式触发Clippy失败；已改为内嵌变量，键内容/评分保持，并补回归守卫。前端和17视口首轮通过，Rust单测/打包/启动仍待修复后的完整Windows。07保持VERIFYING、08～13BLOCKED，完整变化和证据见[07记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。


R10-07 第二轮 `8c9ab5d` / [Windows run38346437982](https://github.com/uniquenesssta/123/actions/runs/38346437982) 已通过编译、Clippy和Application165；PG215通过/1失败，原因是新增精度夹具的预期算错。现改为两个能区分提前舍入的夹具并补数值核对，生产评分与原9测试数量保持。07仍VERIFYING、08～13BLOCKED，完整证据与待验项见[07记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。


R10-07 已收尾DONE：精确cdd2797 / [Windows run38350692611](https://github.com/uniquenesssta/123/actions/runs/38350692611) 全SUCCESS，Application165/Persistence216/Gateway112unit+41contract及9新测试、17视口/打包/启动通过，前两轮失败已关闭。用户授权08漂移监控，09～13BLOCKED；完整证据见[07记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。


### R10-08 漂移监控（2026-10-13，VERIFYING）

原监控入口、共享活动分区与漂移样本/准备/幂等事务/读回已形成唯一owner；旧App两入口删除，原SQL/公式/窗口/状态/单事务审计和供应商→漂移→总览顺序保持。18A/13M/2D，新增6App/11PG unit待Windows（预期171/227）；84/84静态、完整架构、22Rustfmt、19等价及10负向通过。07已DONE、08VERIFYING、09～13BLOCKED，完整变化与最终新库延期见[08记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md)和[阶段索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


R10-08 已收尾DONE：精确f834e31 / [Windows run38357781835](https://github.com/uniquenesssta/123/actions/runs/38357781835) 全SUCCESS，Application171/Persistence227/Gateway112unit+41contract及17新测试、17视口/打包/启动通过。用户授权09分析计算，10～13BLOCKED；完整证据见[08记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md)。


### R10-09 分析计算（2026-10-13，VERIFYING）

R10-09 Analytics Calculation已实施 `VERIFYING`，基线3fdb443，同一唯一R10分支。App总览及两实际worker刷新归calculation；PG样本/解码/原事务/子指标/持久化与总览投影/读取各有唯一owner，旧root七职责移除，剩余29函数保持。原纯引擎/SQL/绑定/公开接口/错误/窗口/分组覆盖政策、500批次/单事务审计及提交后回读保持；后台进度/取消/全量质量→查询顺序保持，10未提前开始。新增5App/8PG unit待自身Windows，预期176/235、Gateway112unit+41contract；84/84静态、完整架构、18Rustfmt、15等价与11负向通过。01～08DONE、09VERIFYING、10～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

14A/13M/1D、原总览逻辑迁移1项；完整职责/文件/验证/回退见[09记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)与[R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


R10-09首轮精确656a97c / [Windows run38371099454](https://github.com/uniquenesssta/123/actions/runs/38371099454)因窗口返回类型触发Clippy type_complexity失败。现改为具名内部AnalysisWindows，同步唯一事务调用方和8处原窗口断言，原值、窗口政策、SQL/事务/审计及纯引擎保持；原5App/8PG测试数量不变。84/84源码、完整架构、3Rustfmt、22等价及5新负向通过；09仍VERIFYING，须修复提交自身完整Windows，10～13BLOCKED。完整修复与待验项见[09记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)。


R10-09 已收尾DONE：精确fbfe83a / [Windows run38398255939](https://github.com/uniquenesssta/123/actions/runs/38398255939)全SUCCESS，Application176/Persistence235/Gateway112unit+41contract及13新测试、17视口/打包/启动通过，首轮窗口类型Clippy失败已关闭。用户授权10后台任务，11～13BLOCKED；完整证据与最终新库待验见[09完成记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)。


### R10-10 后台任务（2026-10-13，VERIFYING）

R10-10 Background Jobs已实施 `VERIFYING`，基线4a6fa9b，继续唯一R10分支。App四队列控制入口编组，启动/领取结算/任务执行/请求解码各唯一；PG共享jobs按控制/读取/领取/生命周期/映射/政策重写，旧root及四旧App目录删除。原四类型/六取消checkpoint、状态/错误/重试/到期/优先级/幂等、SKIP LOCKED/状态事件同事务及提交后读解码保持，P4与启动恢复原调用不变。新增13App/6PG unit待自身Windows，预期189/241、Gateway112unit+41contract；84/84源码、完整架构、18Rustfmt、17等价及12负向通过。01～09DONE、10VERIFYING、11～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验。

14A/13M/5D、实际共享队列与原政策/验证/回退见[10记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-10-background-jobs.md)和[R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


R10-10 已收尾DONE：精确3365b2b / [Windows run38411713492](https://github.com/uniquenesssta/123/actions/runs/38411713492)全SUCCESS，Application189/Persistence241/Gateway112unit+41contract及19新测试、17视口/打包/启动通过。用户授权11分析资料包，12～13BLOCKED；完整证据与最终新库待验见[10完成记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-10-background-jobs.md)。


### R10-11 分析资料包（2026-10-13，VERIFYING）

R10-11 Analysis Package已实施 `VERIFYING`，基线5a19c71，继续唯一R10分支。App导出/回包/人工建议六入口按三职责编组；PG资料包数据/元数据/导入/决定/读取/映射/政策各唯一，旧六目录与root15职责移除，剩余14函数保持。原ZIP库/格式/Schema/摘要验证、SQL/错误/身份核验/待审导入/人工候选事务与提交后回读保持，不自动晋升参数。新增11App/8PG unit待自身Windows，预期200/249、Gateway112unit+41contract；84/84源码、完整架构、17Rustfmt、18等价及13负向通过。01～10DONE、11VERIFYING、12～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

14A/11M/6D、实际职责/人工门禁/验证与回退见[11记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-11-analysis-package.md)和[R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


## R10-11 精确 Windows 完成与12授权（2026-10-14）

精确提交 `9d3727c4bdc0ebeb21e83a466c49f97720f1e063` / tree `5606f26bcbddbc74be5ea105fe25faa3cf8c4b6c`，[Public Platform CI run38423742534](https://github.com/uniquenesssta/123/actions/runs/38423742534) / Windows job115387546853完整SUCCESS，2026-10-14 00:03:23北京时间完成。实际Application200/200、Persistence249/249、Gateway112/112 unit及41/41 contract；本项新增11App/8PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-153953.json及同名.txt。

证据artifact11828902351，14,051,522字节，SHA-256 `bf5b6c6f869263510a06e462570b02fef9265c3a4b6aeb664f1f164ac96d7eea`。11自身完整SUCCESS后DONE；用户2026-10-14“收尾11开始12”授权12 Parameter Lifecycle READY，继续唯一rewrite/r10-review-analytics-release-backend，13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验9d3727c；12须自身精确完整Windows，无阶段完成记录。此前11VERIFYING/预期计数为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不计这些真实PASS。

### R10-12 参数生命周期（2026-10-14，VERIFYING）

R10-12 Parameter Lifecycle已实施 `VERIFYING`，基线6da5057，同一R10分支。App九入口按候选/就绪/影子验证/人工绑定四职责编组；PG同名模块升级为职责目录，22原函数迁入，root analytics保留九原函数。指标公式/排序/阈值/哈希、公开提供器拒绝、H精确分区、不可覆盖报告、人工状态门禁/锁/CAS/两绑定事务及提交后回读保持。新增15App/6PG unit待自身Windows，预期215/255、Gateway112unit+41contract；82源码、完整架构、19Rustfmt、19等价及20负向通过。01～11DONE、12VERIFYING、13BLOCKED、R10IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

17A/12M/10D、实际职责/保护与验证见[12记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-12-parameter-lifecycle.md)和[R10索引](docs/modular-rewrite/R10-review-analytics-release-backend/README.md)。


## R10-12 精确 Windows 完成与13授权（2026-10-14）

精确提交 `6c4243c1150ba0bbf2c0139429e52c3477df6d8a` / tree `8b3928901a0e495f211eb0b937322eb4b48541d2`，[Public Platform CI run38508581096](https://github.com/uniquenesssta/123/actions/runs/38508581096) / Windows job115671629414完整SUCCESS，2026-10-14 12:18:42北京时间完成。实际Application215/215、Persistence255/255、Gateway112/112 unit及41/41 contract；本项新增15App/6PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动及日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261014-035521.json及同名.txt。

证据artifact11869757552，14,050,358字节，SHA-256 `ae7631e1aafcfdfe6e230961c04709d0b8c205dc3e1b28213c7df67c5b4e5c1a`。12自身完整SUCCESS后DONE；用户2026-10-14“收尾12开始13”授权13 Release Acceptance READY，继续唯一rewrite/r10-review-analytics-release-backend，R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验6c4243c；13须自身精确完整Windows，无阶段完成记录。此前12VERIFYING为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不是这些真实PASS。

### R10-13 与 R10 阶段收尾（2026-10-15，DONE）

R10-13精确 `4ca43ad` / [Windows run38591308033](https://github.com/uniquenesssta/123/actions/runs/38591308033) 完整SUCCESS，实际Application230、Persistence259、Gateway112unit+41contract；新增15App/4PG unit、前端/17视口、Rust、Windows release/MSI/NSIS/启动与日志全部通过。原检查、报告哈希、不可变事务及模型保护保持。01～13与R10代码/Windows Automated阶段出口DONE，详见[13完成记录](docs/modular-rewrite/R10-review-analytics-release-backend/R10-13-release-acceptance.md)、[阶段完成总表与限制](docs/modular-rewrite/R10-review-analytics-release-backend/R10-stage-completion.md)。

本次“收尾13”只文档[skip ci]；[R11-01仅READY](docs/modular-rewrite/R11-tauri-commands/README.md)，未启动代码或新分支。真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险继续最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实项目PASS。


### R11-01 平台命令（2026-10-15，VERIFYING）

Bootstrap、工作区与日志9条命令已收拢至commands/platform，唯一rewrite/r11-tauri-commands，基线3d43e8e。原保存连接恢复、错误/日志、State与wire DTO保持，171公共命令/注册顺序不变，内部定义清单同步真实位置。新增11原Tauri单测待Windows；82源码、完整架构、24Rustfmt、33等价和16负向通过。01VERIFYING、02～10BLOCKED，完整[01记录](docs/modular-rewrite/R11-tauri-commands/R11-01-bootstrap-workspace-and-logs-commands.md)与[阶段索引](docs/modular-rewrite/R11-tauri-commands/README.md)。真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。


R11-01 首轮 `9e0a907` / [Windows run38689146967](https://github.com/uniquenesssta/123/actions/runs/38689146967) 在恢复连接helper因Arc接收者类型丢失报E0599；前端和17视口通过。现已恢复原Arc借用、同步四个测试夹具并补原守卫，82源码/完整架构/2Rustfmt/35等价/17负向通过；同一R11分支复跑Windows，01仍VERIFYING。详见[01修复记录](docs/modular-rewrite/R11-tauri-commands/R11-01-bootstrap-workspace-and-logs-commands.md)。


R11-01已收尾DONE：精确e56d21f / [Windows run38724727036](https://github.com/uniquenesssta/123/actions/runs/38724727036)完整SUCCESS，Desktop38/App230/PG259及完整前端/17视口、Rust、Windows打包/启动/日志通过，原E0599关闭。用户2026-10-23授权02 READY，沿同一R11分支；[01完成记录](docs/modular-rewrite/R11-tauri-commands/R11-01-bootstrap-workspace-and-logs-commands.md)。真实PG/私有回归/XLSX/Full与继承删除风险仍最终新库待验。


### R11-02 数据库命令（2026-10-23，VERIFYING）

配置、断开和重置命令已按桌面生命周期职责拆分，原加密配置/凭据、名称预检、失败补偿和工作器恢复顺序保持；171公共命令不变。新增8原Tauri单测待Windows；82源码/完整架构/10Rustfmt/32等价/17负向通过，02VERIFYING、03～10BLOCKED。详[02记录](docs/modular-rewrite/R11-tauri-commands/R11-02-database-commands.md)与[R11索引](docs/modular-rewrite/R11-tauri-commands/README.md)；真实PG重置/重连及原最终新库项目仍待验。

R11-02已收尾DONE：精确a2341de / [Windows run39924367412](https://github.com/uniquenesssta/123/actions/runs/39924367412)完整SUCCESS，Desktop46（新增8项逐项OK）/App230/PG259及完整前端/17视口、Rust、Windows打包/启动/日志通过。03已获授权READY，沿同一R11分支；[02完成记录](docs/modular-rewrite/R11-tauri-commands/R11-02-database-commands.md)。真实PG及原最终新库待验项目保持。

### R11-03 赛事与规则命令（2026-10-23，VERIFYING）

七条赛事/赛季/阶段/轮次/删除/规则包/绑定命令已按职责独立，规则两入口归rules子目录；保留原UUID门槛、Application调用和错误投影，旧competition.rs删除，171公共契约不变。82源码/完整架构/9Rustfmt/38等价/17负向通过；原单测目标保持，03自身Windows待验、04～10 BLOCKED。详[03记录](docs/modular-rewrite/R11-tauri-commands/R11-03-competitions-and-rules-commands.md)；真实PG及原最终新库项目继续待验。

R11-03已收尾DONE：精确849f1a4 / [Windows run39930230184](https://github.com/uniquenesssta/123/actions/runs/39930230184)完整SUCCESS，Desktop46/App230/PG259及完整前端/17视口、Rust、Windows打包/启动/日志通过。04已获授权READY，沿同一R11分支；[03完成记录](docs/modular-rewrite/R11-tauri-commands/R11-03-competitions-and-rules-commands.md)。真实PG及原最终新库待验项目保持。

### R11-04 球队与球员命令（2026-10-23，VERIFYING）

球队/球员/教练/实体35条命令已按职责独立，原19条阵型/比赛/阵容/预设命令为删除混合catalog入口按能力原样归位，05继续验收。54条原签名/正文、批量UUID首错和强删非Send线程/错误保持，171公共契约不变。82源码/完整架构/44Rustfmt/189等价/20负向通过，原动态目标保持，04自身Windows待验。详[04记录](docs/modular-rewrite/R11-tauri-commands/R11-04-teams-and-players-commands.md)；真实PG及原最终新库项目仍待验。

R11-04已收尾DONE：精确7ccb11e / [Windows run39937856913](https://github.com/uniquenesssta/123/actions/runs/39937856913)完整SUCCESS，Desktop46/App230/PG259及完整前端/17视口、Rust、Windows打包/启动/日志通过。05已获授权READY，沿同一R11分支；[04完成记录](docs/modular-rewrite/R11-tauri-commands/R11-04-teams-and-players-commands.md)。真实PG及原最终新库待验项目保持。

### R11-05 阵容、预设与工作簿命令（2026-10-23，VERIFYING）

19条阵容/阵型/比赛/预设命令与24条工作簿/资料包命令已独立，catalog收敛35，删除5个旧owner；内部物理组增至14，171公共命令/注册顺序/原参数与错误保持。最终82源码/完整架构/55Rustfmt/161等价/20隔离负向通过，05自身Windows待验。详[05记录](docs/modular-rewrite/R11-tauri-commands/R11-05-lineups-presets-and-workbooks-commands.md)；真实PG/XLSX与原最终新库项目仍待验。


R11-05已收尾DONE：精确b9040df / [Windows run39946211886](https://github.com/uniquenesssta/123/actions/runs/39946211886)完整SUCCESS，Desktop46/App230/PG259、完整前端/17视口、Rust、Windows打包/启动/日志通过。用户授权同一R11分支06 READY，07～10 BLOCKED；详[05完成记录](docs/modular-rewrite/R11-tauri-commands/R11-05-lineups-presets-and-workbooks-commands.md)。真实PG/XLSX/Full/私有固定回归和继承删除风险仍最终新库待验。


### R11-06 推演、研究与运行记录命令（2026-10-23，VERIFYING）

17条原命令已按推演6、运行记录3、P4任务/工作台7、研究冲突1归入唯一commands/prediction目录，4纯出口接通原Application能力，旧prediction.rs/orchestration.rs删除。171公共命令/注册顺序/原签名与正文保持，内部物理组14→13；82源码/完整架构/22Rustfmt/69等价/19隔离负向与保护门禁通过。06自身完整Windows待验，07～10 BLOCKED，R11 IN_PROGRESS。 详[06记录](docs/modular-rewrite/R11-tauri-commands/R11-06-prediction-research-and-runs-commands.md)与[R11索引](docs/modular-rewrite/R11-tauri-commands/README.md)。真实PG/Full/私有固定回归及原最终新库项目仍待验。


R11-06已收尾DONE：精确faa6f72 / [Windows run39954995571](https://github.com/uniquenesssta/123/actions/runs/39954995571)完整SUCCESS，Desktop46/App230/PG259、完整前端/17视口、Rust、Windows打包/启动/日志通过。用户授权同一R11分支07 READY，08～10 BLOCKED；详[06完成记录](docs/modular-rewrite/R11-tauri-commands/R11-06-prediction-research-and-runs-commands.md)。真实PG/XLSX/Full/私有固定回归与继承删除风险仍最终新库待验。


### R11-07 复盘、赛后与分析命令（2026-10-23，VERIFYING）

41条原命令按复盘6、赛后14、分析21拆入各自唯一职责目录及能力子目录；10个纯出口接通原Application，旧review.rs/postmatch.rs/analytics.rs删除。后台任务失败问题投影沿原共享owner与原调用顺序；171公共命令/注册顺序/签名/正文保持，13物理组不变。82源码/完整架构/51Rustfmt/154等价/26隔离负向与保护门禁通过；07自身完整Windows待验，08～10 BLOCKED，R11 IN_PROGRESS。 详[07记录](docs/modular-rewrite/R11-tauri-commands/R11-07-review-postmatch-and-analytics-commands.md)与[R11索引](docs/modular-rewrite/R11-tauri-commands/README.md)。真实PG/XLSX/Full/私有固定回归与原最终新库项目仍待验。


R11-07已收尾DONE：精确5b779b9 / [Windows run39962382506](https://github.com/uniquenesssta/123/actions/runs/39962382506)完整SUCCESS，Desktop46/App230/PG259、完整前端/17视口、Rust、Windows打包/启动/日志通过。用户授权同一R11分支08 READY，09～10 BLOCKED；详[07完成记录](docs/modular-rewrite/R11-tauri-commands/R11-07-review-postmatch-and-analytics-commands.md)。真实PG/XLSX/Full/私有固定回归与继承删除风险仍最终新库待验。


### R11-08 AI问答与兼容API配置命令（2026-10-23，VERIFYING）

13条原AI问答/配置命令归入唯一commands/ai_workspace，发送流程分离纯校验、会话绑定、内容、网关/日志及回答投影；取消仍原唯一租约，配置仍原凭据存储。旧api_workspace.rs/openai.rs删除，171公共命令/注册/DTO保持，物理组13→12，配置owner不变且刷新实际五个writer。新增12原Tauri单测待Windows；82源码/完整架构/30Rustfmt/61等价/30隔离负向与保护门禁通过。08自身Windows待验，09～10 BLOCKED，R11 IN_PROGRESS。 详[08记录](docs/modular-rewrite/R11-tauri-commands/R11-08-ai-workspace-and-openai-profiles-commands.md)与[R11索引](docs/modular-rewrite/R11-tauri-commands/README.md)。真实API/凭据/会话端到端、PG/XLSX/Full/私有固定回归与原最终新库项目仍待验。


R11-08首轮ffbaedc / [Windows run39971471528](https://github.com/uniquenesssta/123/actions/runs/39971471528)在追踪ID格式化处被Clippy uninlined_format_args拦截；前端/17视口通过。现内联session_id/request_id，追踪内容及原发送/取消/凭据行为保持，补原源码卫生守卫；82源码/完整架构/1Rustfmt/61显式等价与3隔离负向通过。修复自身Windows待验，08 VERIFYING、09～10 BLOCKED；详[08修复记录](docs/modular-rewrite/R11-tauri-commands/R11-08-ai-workspace-and-openai-profiles-commands.md)。


R11-08已收尾DONE：精确858288e / [Windows run39974291282](https://github.com/uniquenesssta/123/actions/runs/39974291282)完整SUCCESS，Desktop58（新增12项AI发送unit）/App230/PG259、完整前端/17视口、Rust、Windows打包/启动/日志通过，首轮Clippy失败关闭。同一R11分支09 READY、10 BLOCKED；详[08完成记录](docs/modular-rewrite/R11-tauri-commands/R11-08-ai-workspace-and-openai-profiles-commands.md)。真实API/凭据/会话取消端到端、PG/XLSX/Full/私有固定回归与继承删除风险仍待实际验收。


R11-09发布验收命令已按执行/列表/读取拆分，旧入口删除，真实报告生成与查询继续原服务；171命令与公共DTO不变。源码/架构/格式/等价及隔离回归检查通过，自身完整Windows待验，10未开始。详[09记录](docs/modular-rewrite/R11-tauri-commands/R11-09-release-commands.md)。


R11-09首轮Windows被无用AppState重导出阻断，现删除遗留中转，并修正组合根守卫为实际命令直接使用唯一状态类型；发布命令与真实服务/报告行为不变。源码/架构/格式/等价与隔离检查通过，修复自身Windows待验。详[09修复记录](docs/modular-rewrite/R11-tauri-commands/R11-09-release-commands.md)。


R11-09已收尾DONE：精确44f2047 / [Windows run39986012998](https://github.com/uniquenesssta/123/actions/runs/39986012998)完整SUCCESS，Desktop58/App230/PG259/Gateway112+41、完整前端/17视口、Rust、Windows打包/启动/日志通过，首轮unused_imports失败关闭。同一R11分支10 READY，阶段尚未完成；详[09完成记录](docs/modular-rewrite/R11-tauri-commands/R11-09-release-commands.md)。真实PG/Full/XLSX/私有固定回归及继承风险继续待实际验收。


R11-10已锁定唯一命令注册入口与171个实际命令文件，现有守卫能拦截注册表混入实现、重复handler、组合根绕过与命令重新聚合。生产Rust/真实服务/公共契约不变，静态与隔离检查通过，自身完整Windows待验；详[10记录](docs/modular-rewrite/R11-tauri-commands/R11-10-command-registry-收敛.md)。R11阶段尚未完成，R12未开始。


R11已收尾DONE：171命令归入12职责目录，唯一注册边界收敛；最终ed11637 / [Windows run39996999630](https://github.com/uniquenesssta/123/actions/runs/39996999630)完整SUCCESS，前端/17视口、Rust测试、Windows打包/启动/日志通过。[阶段完成记录](docs/modular-rewrite/R11-tauri-commands/R11-stage-completion.md)汇总真实清单和待验；下一项R12-01 READY（R11仅10项）。最终新库/Full/私有固定回归等真实待验未关闭。


R12-01已补强启动组合根校验：唯一入口实际调用、真实根与handle、原应用延迟注册/启动/销毁、等待与失败投影均接通原实现，能拦截空启动和旁路。生产TypeScript/业务/视觉保持，源码与隔离检查通过，自身Windows待验；详[01记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-01-bootstrap-组合根.md)。


### R12-01 Windows验收收尾（2026-10-24，DONE）

R12-01 Bootstrap组合根DONE：精确f460fbf / Windows run40061447839于2026-10-24 12:42:31北京时间完整SUCCESS，Desktop58/App230/PG259/Gateway112unit+41contract、前端/17视口、Rust、Windows打包/启动/日志通过。R12-02生命周期基础设施READY（尚未实施），03～19 BLOCKED，R12 IN_PROGRESS，唯一rewrite/r12-frontend-foundation-ui-system。本次仅文档收尾，生产与守卫同已验树；[01完成记录](docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-01-bootstrap-组合根.md)包含日志、报告和产物校验值。真实PG/Full/XLSX/私有固定回归及既有最终环境待验保持。
