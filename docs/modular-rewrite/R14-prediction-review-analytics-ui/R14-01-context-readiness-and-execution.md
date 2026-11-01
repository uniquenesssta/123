# R14-01 输入、准入与推演执行

状态：VERIFYING。本地实现和适用验证通过；自身精确 CI、安装包和包内 startup 尚待推送后核验，不能先记 DONE。覆盖原 R14-01～05，下一批 R14-06 未启动。

分支 `rewrite/r14-prediction-review-analytics-ui`，起点 `3d9363fe`。开始时工作区干净；用户已持续授权提交推送，不重复申请、不轮询 PR/CI。R13-09 的 `f5cc82a7` / run `41274320064`、官方摘要、MSI/NSIS 和包内 startup 已验收，收尾提交为 `3d9363fe`。R12 保持 DONE。

## 当前源码核对与交付范围

当前 `execute_prediction` 先调用真实提供器，再调用 `save_successful_run`。PostgreSQL adapter 的 `write.rs` 已将 feature snapshot、model run、明细与审计置于同一事务后 commit；旧账本中“run 先落地、snapshot 后写”的描述不能直接套用当前源码。本批没有改 Rust、事务、模型算法、概率计算、数据库 schema、Wire、依赖或锁文件。

`build_input` 在后端同一 assessed_at 重建并核验实际输入。安全截止时点随时间推进，预检 manifest 与随后实际运行 manifest 可以不同；本批保留实际返回审计，并按返回 run ID、比赛、窗口、模型、规则、实际 manifest 和 snapshot 读回核验，不能把预检 hash 硬写成实际运行 hash。

现有 StoredMatchPredictionCommand 没有幂等请求 key。相近记录不能证明属于同一客户端请求；结果未知时只查原记录，不自动重发。只列出基线之后的相关记录，明确核对实际 ID/指纹后才能采用。500 条查询没有命中也不证明执行失败，不由前端推断或自动解除正式写入的未知状态。已明确在持久化前返回的 ModelError::Unavailable 单独呈现，不伪造概率。

## 状态归属、事件与真实数据流

| 责任 | 实际生产所有者／入口 | 约束 |
| --- | --- | --- |
| 比赛、模型、窗口、规则选择 | `predictionContext.ts` / PredictionContext | 变更同步废弃旧许可、结果、路由；模型变更清显式规则 |
| 门禁、检查、操作锁、正式／影子执行 | `predictionFeature.ts` / PredictionFeature | 原 API 提交前重新检查；SHADOW_ONLY 不升级；重复点击不重复发送 |
| 结果未知及查回 | `predictionRecovery.ts`、`predictionRunRecovery.ts` | 按数据库／角色隔离，只存请求身份和基线 ID，不含密码、原始输入或业务行 |
| 结果投影与完整展示 | `predictionProjection.ts`、`predictionResultView.ts`、`predictionLabels.ts` | 缺失值显示未记录；真实 0 保留；矩阵阈值 0.1% 和累计概率保留 |
| S03 页面与固定动作 | `formalPredictionView.ts`、`pages/prediction.ts`、`predictionWorkspace.css` | 原 Shell/Workspace/控件增强完整挂载；只滚动指定内容区 |
| 装配与原动作名兼容 | `src/main.ts` | 正式、影子、阵容补齐、路由、详情动作保留，root 不再拥有第二份正式选择／readiness／结果 |

API 保留原名和参数：`inspectMatchPredictionReadiness`、`readMatchLineupChain`、`executePredictionFromMatch`、`executeShadowPredictionFromMatch`、`previewRoute`、`listRecentRuns`、`readRun`。实际已返回的正式 run 还须读回完整 snapshot/审计，缺失或空 snapshot 身份不计成功。影子必须保持 nil run ID 和真实类型，不写正式历史。

保留日期／时区、T-N/T-24h/T-6h/T-1h、P4/P7 与具体模型选择、规则入口／参数说明、阵容补齐返回链、P4 研究、历史、临时演练、搜索、工作区恢复、主题、滚动与焦点。P4 与 Runs 旧页面的过渡导入分别登记 R14-06、R14-07，本批不提前迁移。

新增异步隔离覆盖上下文变更、页面与页签 ABA、数据库替换、结果读回、历史刷新期间切换比赛。自检与临时演练保留原命令及实际持久化语义，返回展示独立于正式结果，迟到返回不覆盖正式上下文。数据库比赛计算入口保留原来的自动路由语义，显式规则设为null，避免携带其他页面的隐藏覆盖。客户端局部 phase 是界面操作状态，不复制后端模型或研究状态机。

## 最新 Figma 与同状态接入

权威清单为 `origin/ui-design-system:docs/FIGMA_SCREEN_PREDICTION_STATE.json`，文件 `PN0Whgu6HLWIHx4Mv6aHfu`，Screens 页 `3:5`。正式主状态 `487:22278`；blocked `490:22964`、shadow-only `490:23013`、ready `490:23111`、provider-unavailable `490:23160`、changed `490:23209`、running `490:23405`、result `490:23454`、failure `490:23503`；暗色／紧凑状态 `490:24140`、`490:24189`。

根高保真请求超时后按实际子节点取得 Provider、Settings、Audit、Execution、门禁文本和结果文本上下文。根截图通过 Plugin API 核对；结果概率 `491:32636`、比分 `491:32638` 的具体字体与布局另取得高保真上下文。设计中的演示概率不进入生产。结果区改为身份、胜平负、比分与市场摘要，完整矩阵、模型依据和审计仍由现有详情入口打开。

| 同状态义务 | 实际接入／验证 |
| --- | --- |
| 未检查、无比赛 | 不预填概率；原生当前无可选比赛，正式／影子禁用 |
| checking、ready、blocked、shadow-only | 当前输入绑定；控件锁定／门禁区／真实允许类型 |
| running | 固定底栏可见，禁止重复提交，输入锁定 |
| result | 紧凑摘要和实际窗口／run ID；详情保留矩阵与实际输入 hash |
| failed / unavailable / unknown | 分开呈现；不伪造成功，不盲目重发；精确查回及候选确认 |
| changed、跨页、跨库 | 旧门禁、结果、异步返回不得授权新上下文 |

复用 Noto Sans SC 与当前 token：标题 24/700/32；分区标题和结果概率 14/700/20；字段 13/500/18；输入 13/400/20；正文 12/400/18；说明与身份 11/400/16。设置区三列＋两列，<=1100px 单列；12px 分区间隔、8px 操作间隔、40px 输入、32px 操作、16px 卡片圆角。1440×900、960×720、1280×720、1920×1080，100% 缩放，明暗各一组。固定底栏置于主滚动区之外；路由摘要按容器排两列，避免标签挤成碎字。

设计缺少的必要能力先补了可编辑说明：未知结果恢复 `1155:376788`，页内定位 `1157:376797`，都已截图检查。页内定位保留设置、门禁、概率、判定四个真实目标，默认折叠，选择后关闭、焦点移至实际目标，Escape 返回菜单触发器。无新增图片或网络图标；复用既有 C47 Select 和已映射的 `54:260` chevron、`54:290` 提醒图标。图标呈现仅在推演页增强，通用选择器源码及诊断实现保持原样。

旧静态 UI 检查按实际新 owner 与批准 S03 骨架更新，任务 UI 合同记录明确的批准布局映射；没有关闭或删除检查。功能断言和完整挂载验证继续执行，不能把旧 class 名称当作最新设计验收标准。

## 验证与证据

总证据目录：`O:/Project/FOODBALL/logs/r14-01-20261101`。完整96步入口以exit 0结束，最终27项行为、40项S03完整挂载均通过；精确代码提交及单次CI核对见交接段。最终整页证据已由tmp转存至本目录approved-final，路径映射见approved-location.json。

- 静态／架构：模块、状态、样式边界、保护资产、Wire、原有导航／布局／选择器诊断均通过；19 Feature、11 Rust crate、351 TS 源文件、56 状态所有者。无文件删除。
- 自动行为：27 项实际 PredictionFeature／投影／恢复用例，覆盖迟到 A/B、重复提交、SHADOW_ONLY、nil 影子、已知与未知查回、候选确认、缺失／空 snapshot、存储失败、实际 safe-cutoff hash、已完成禁重、页签 ABA、跨库隐藏等。
- Windows 浏览器：实际 main/Shell/Session/Workspace/控件增强完整挂载，只有 native transport 为明确 QA 边界；40 项 S03 检查，包含八组尺寸／主题、运行态、摘要／详情、固定底栏、定位与焦点、迟到历史刷新。该结果不是私有模型或 PG 成功证明。
- 完整前端：现有 96 步入口（包含类型与 Vite 构建）执行；最终日志 `frontend-final.log`，96步全部完成且exit 0；保留旧搜索提示后末段旧源定位已修复复验。随后最终源码类型与Vite构建再次通过，见 `build-final.log`；Vite保留原有大chunk提醒，没有关闭警告。
- Windows 构建：`cargo build --locked -p football-match-model-desktop` 成功；没有用类型／构建替代运行。
- Windows 原生：`native-final/report.json`；真实 WebView/IPC，无业务 mock。实际配置库、12 个模型登记、0 条近期运行，无可选比赛，按钮禁用、五个选择器、固定底栏与内部滚动正确。只读提供器命令返回“模型运行时不可用”，未写业务数据。`native-startup-final.json`：17 条日志、6 个完成操作、startup PASS。未操作保存窗口、未测试私有模型成功。
- Rust 同树：crates `e96159c867e4a011f2bf3b18a481f683fac78546`、src-tauri `1c35e0d2be3925d0fea34489f3c54f34f90b8fe7`、Cargo.lock `0cb8b1abcf72d29fb4f0be6641e253b673a60a8a` 与 R13-09 已验收代码一致；本批复建原生，不声称重新跑过全部 Rust 测试。
- Impeccable：使用 harden／craft-floor 及实际渲染核对；引擎源码检测无 primary findings。检测不能替代浏览器或原生功能证明。

真实新 PG、事务故障／并发、Full 与私有模型回归继续逐项由原账本 BA-08/BA-13、R17-06 承接；本批不把同树静态检查或 QA 输出说成真实环境已通过。

## 临时目录、风险与下一批

进程 TEMP/TMP 使用 `O:/Project/FOODBALL/tmp`，经既有 project-temp.ps1 配置。必要日志、最终截图、原生证据与依赖保留；本批可确认无用的浏览器 profile／一次性辅助文件按明确清单归档到 old，未批量或递归删除。已按pre/final清单将129个退出的浏览器profile移至 `O:/Project/FOODBALL/old/r14-01-20261101-browser-profiles`，另9项初验/旧失败产物移至同级prior-attempts目录。清单为temporary-cleanup-{pre,final}-{manifest,result}.json及supporting-cleanup.json。全部为归档移动、删除数0。

回退点为本批起点 `3d9363fe`；变更限定于前端推演职责与相关验证／记录，不覆盖用户数据、不改数据库或私有资产。自身 CI／双安装包／包内 startup 尚未核验前保持 VERIFYING，不提前交付 R14-06。下一项：**R14-06 P4研究证据与冻结**。

## 2026-11-01 推送交接

代码提交 `866ba00399b5e8930530df75e71e1609ab47a61f` 已推送到本节所列R14分支：34路径，15新增、19修改、0删除，工作区已核对。完整96步exit 0；27行为、40挂载、最终类型/Vite构建及提交后Windows原生startup通过，受测源码归属对应此提交。

仅单次查询该精确SHA的push事件：[Public Platform CI run 41285690142](https://github.com/uniquenesssta/123/actions/runs/41285690142)。创建于2026-11-01 15:39:57（北京时间）；当次状态in_progress、conclusion为空。未轮询PR/CI，未把已启动说成通过，尚未下载或验收本批MSI/NSIS及包内startup；本批保持VERIFYING。后续收尾须从该代码SHA核对CI和产物，不能使用文档交接提交代替代码CI。

`ci-once.json`保存单次响应，`handoff.json`及`final-hashes.json`保存本地交接和证据摘要。另将本批一次性staging目录移入 `O:/Project/FOODBALL/old/r14-01-20261101-staging`，不删除文件；最终证据、生产测试及可复用依赖保留。下一项仍为R14-06，尚未启动。
