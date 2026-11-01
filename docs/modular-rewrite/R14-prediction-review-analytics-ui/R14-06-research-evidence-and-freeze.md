# R14-06 P4 研究证据与冻结

状态：DONE。本地适用验证、精确代码CI、官方产物摘要、MSI/NSIS及包内startup均已验收。用户授权收尾01并开始06；沿用持续提交推送授权，不轮询 PR/CI。

分支 `rewrite/r14-prediction-review-analytics-ui`，基线 `39b16555162aed0c619dfdbbbdba0b804479d10e`，开始时工作区干净。R14-01 的 `866ba003` / run `41285690142` 已独立核验官方摘要、双包和 startup，收尾提交为上述基线；不再用01的CI代替06。

## 实际变更与保留的功能

沿用既有研究工作台和 Prediction Feature，没有新增整页、后端命令、Schema、模型实现或概率算法。旧 `src/pages/p4Workbench.ts` 已由研究视图、投影与流程替代；根入口退出两个研究工作区变量和原查询/提交业务实现，只保留装配、DOM取值和共享确认调用。

| 责任 | 当前生产所有者 | 实际调用与约束 |
| --- | --- | --- |
| 比赛、模型、窗口、规则选择 | `predictionContext.ts` / PredictionContext | 沿用唯一共享比赛上下文；研究模块不建立第二份比赛选择状态 |
| 研究查询、任务选择和结果归属 | `research/researchFeature.ts` / ResearchFeature.load、openTask | 实际读取比赛、任务；检查 match/task/readiness 身份；后发请求拥有发布权，比赛、页面、分区和数据库变化使旧返回失效 |
| 冲突决策与版本核验 | researchDecision / prepareDecision、decide | 捕获任务及证据版本、所选来源和说明；确认后重新读真实任务并检查状态、截止时间、冲突和引用，编辑后旧确认失效 |
| 请求恢复 | researchRecovery / recover | 实际数据库分区保存原请求；发送前保存失败则不发送；未知结果只能查回原比赛/任务，不自动重发、不把“没查到”当作“没写入” |
| 展示和只读快照 | researchView、researchSnapshotView、researchPresentation | 保留所有证据、URL、时点、引用、路由、历史任务、事件和完整原始快照；未知不补零，真实0保留 |
| 冻结执行与不可变账本 | 原后端 Research/Prediction Service、Ports、Worker | 前端无强制冻结按钮；未解决冲突或过期许可不能通过人工入口绕过后端门禁 |

仍使用原 `readP4MatchWorkspace`、`readP4TaskWorkspace`、`planP4Horizons`、`resolveP4Conflict`。三个实际计划窗口按比赛、规则包和开球时点查回核验；人工来源选择维持 PROBABLE，接受未知维持 NOT_FOUND 语义。服务端账本及证据不由前端覆盖。

决策“已记录”、研究“门禁已就绪”和“已冻结”分别展示。只有任务为 FROZEN 且嵌套快照 ID、比赛和窗口全部匹配，才显示已冻结并允许打开完整快照。只读详情复用既有应用内详情历史，明确显示实际截止时间、冻结时间、指纹、字段、概率链和原始记录；缺少概率链时明确尚未返回，不伪造结果。

请求处理中切换对象不会将原操作结果写入另一场比赛。数据库范围借用已有只读 Bootstrap 快照及数据范围，而非新增全局计数器；即使切出再切回，也不能沿用旧响应。人工确认采用独立的编辑修订检查，避免因忙碌状态禁用表单而误丢失已确认的来源和说明。

## Figma、排布与组件状态

继续使用已批准 S03 骨架、Noto Sans SC、Inter 数字及既有语义色令牌。先读取对应研究 Frame 与组件上下文，再补齐原设计未覆盖的实际来源选择、说明、确认、未知结果查回和完整来源折叠状态。未替换原批准 Screens。

- Research Windows `490:23601`；Conflict `490:23650`；Resolved `490:23699`；Readonly `490:23748`。
- Frozen `490:23797`；Empty `490:23846`；Dark `490:24238`；Compact `490:24287`；AcceptedUnknown `498:31928`；只读快照参考 `495:31745`。
- 本批补充 `1166:376800`，现有 FIP Radio、Input/Text Field、Panel、Button 和 Confirmation 组成；最终截图已检查单选行对齐、说明在动作之前、恢复区独立归属。证据 `figma-research-supplement-final.png`。

研究内容由唯一主视口滚动；顶部选择器等宽、任务按钮紧凑换行、四指标等宽，表格保留来源和时间并允许长内容换行。标题700、控件/标签500、正文400、指标数字600，沿用既有字体与行高。不同窗口内容量不同可自然增高，不以固定空白填满模块。来源表补充全部证据折叠区，未为了视觉简化删掉原有功能。

## 验证与实际边界

证据根目录 `O:/Project/FOODBALL/logs/r14-06-20261101`。

| 层级 | 结果及证据 | 证明范围 |
| --- | --- | --- |
| 架构、类型/接口和保护资产静态检查 | 完整前端内 PASS；18个Feature、57状态owner、187 CSS及564条受约束选择器 | 研究并入既有 Prediction；BA-15 的研究过渡边实际退出，Runs边仍按R14-07保留；未扩大白名单 |
| 新研究流程行为 | 19项 PASS，verifyResearchWorkflow | 实际生产 Feature、确认版本、编辑失效、截止门禁、任务/比赛错配、请求顺序、重复操作、存储失败、未知结果查回、数据库隔离、真实0及只读快照 |
| 完整 Windows 浏览器挂载 | 26项 PASS，research-mounted/results.json | 真实 main/Shell/事件组合根，只有外部传输为显式QA边界；1440×900、960×720、1280×720、1920×1080及明暗主题；等待/冲突/过期只读/冻结中/失败/已冻结、实际说明提交、接受未知、响应丢失查回及计划窗口 |
| 原入口与全量前端 | 完整96步 exit0，frontend.log | 保留27项Formal生产行为、40项S03挂载、21项导航及全部原检查；TypeScript和Vite构建均通过 |
| 最终构建 | build-final.log exit0 | 旧dist先单项归档，再生成干净产物；保留既有大于500kB分块提示，未调高阈值消除提示 |
| Impeccable | impeccable.json 为[]，exit0 | 本批真实研究源码检测，无配置/内联忽略；不把源码检测当成真实交互证明 |
| Windows原生与startup | native-final/report.json PASS；startup.json PASS | 真实Tauri/WebView2，无QA注入、无业务库写入；18条有效日志、6个完成操作、0无效/错误日志；从界面进入研究及真实目录读取，实际空目录保持空 |
| 实际P4/联网研究/冻结及PG并发 | 未验证成功，不能记PASS | 当前库没有可选比赛，P4提供器实际返回运行时不可用；没有人为创建比赛或伪造冻结。真实研究服务还需独立配置，不能把模型可用等同于联网研究可用 |

R3-06冻结检查脚本的过期 `snapshot_projection.rs` 和 `process_next.rs` 单文件锚点已更新到真实拆分后的 features/probabilities/workflow/dispatch 源码；原有 Port、固定路由、投影、不可变快照和双职责委托断言全部保留并通过，未改算法或关闭检查。

纯前端批次复用已验收01代码 `866ba00399b5e8930530df75e71e1609ab47a61f` 的同树Rust证据，本批未宣称重跑Rust检查/测试。当前及原受测树完全相同：crates `e96159c867e4a011f2bf3b18a481f683fac78546`，src-tauri `1c35e0d2be3925d0fea34489f3c54f34f90b8fe7`，Cargo.lock `0cb8b1abcf72d29fb4f0be6641e253b673a60a8a`；本机Node v24.16.0、Rust/Cargo1.88.0沿原环境。最终候选仍须R18门禁。

## 遗留项、回退与下一批

- BA-02：两项相关 PostgreSQL 集成夹具/精度修订仍在原受测树，未在授权新测试库重新运行；R17-06逐项实证。
- BA-08：实际冻结仍为模型执行→快照写入→任务Frozen转换的跨Port顺序。前端保持部分结果与未知写入的可恢复状态，本批不声称完成后端事务原子化；故障/并发由R17-06实证。
- BA-13：私有P4/P7资产缺席，实际成功和固定回归由R17-06承接；不以QA概率或公开壳启动抵销。
- BA-15：研究边已实际退出并通过新调用图验证；`prediction.ts → runs.ts` 仍由R14-07处理。

TEMP/TMP 使用既有 `O:/Project/FOODBALL/project-temp.ps1` 配置到 `O:/Project/FOODBALL/tmp`。必要日志、最终设计/浏览器/原生截图、生产测试与可复用依赖保留；无用profile、旧构建和一次性辅助文件按明确清单归档old，删除数0。具体路径和结果见 temporary-cleanup-manifest.json / temporary-cleanup-result.json。

代码回退点为本批基线39b16555；不回写用户数据，不修改后端账本、数据库、私有资产或R12完成状态。本批CI/双包/包内startup尚未验收前保持VERIFYING。

下一项：**R14-07 运行记录与血缘**，待06适用出口验收后由用户启动。

## 2026-11-01 推送交接

代码 `5171f779fc487def6445f69f5d1cc4a7343f60c0` 已推送到本批R14分支。28路径：12新增、15修改，仅退出已由研究视图/投影替代的 `src/pages/p4Workbench.ts` 一个旧路径；没有额外删除。

仅单次查询该精确SHA、R14分支及push事件，匹配 [Public Platform CI 41297701399](https://github.com/uniquenesssta/123/actions/runs/41297701399)；当次 `in_progress`，conclusion为空。未轮询PR/CI，没有把01的CI或后续文档提交当作06代码CI。06的MSI/NSIS、官方摘要和包内startup尚未下载验收，本批保持VERIFYING。

`ci-once.json` 保存原响应，`local-verification.json`保存本地分层证据与摘要；102项已完成临时产物已按清单归档，另将本批一次性staging目录单项归档old，均不删除。必要最终截图、原生日志、验证结果、实际源码、干净dist及可复用依赖保留。下一项仍为 **R14-07 运行记录与血缘**。

## 2026-11-01 正式收尾

代码5171f779、R14分支、push事件与run41297701399精确匹配；Windows job124821946104所有步骤success。官方artifact13081061987，52,331,600字节，ZIP SHA256 c7ecc769416f532653c7293b5ed2bf4aee831b9b6bb994d40d3560a6ef8abdf2，整体下载摘要一致。没有轮询PR/CI。

- MSI：18,001,920字节，OLE头有效，SHA256 2131929d9d23e220161b708efd27892125ab6a1d5e9ffece69569624ccaf3f25。
- NSIS：15,641,787字节，MZ头有效，SHA256 313e270a3aaae5d1f4c173ad7cfa0f3827018bf893f436bd297f9db0d63f4204。
- 包内windows-acceptance-20261101-103044.json：startup PASS，7条有效记录，bootstrap/read_workspace_state/save_workspace_state；0运行错误、0无效行、0缺失必需组。未据此声称Full或本地重新安装。

证据：O:/Project/FOODBALL/logs/r14-06-ci-41297701399/acceptance.json。06适用出口正式DONE；BA-02/08/13真实外部项仍按原批准R17-06承接。用户已授权开始R14-07运行记录与血缘。后续临时归档与清理的唯一目录为O:/Project/FOODBALL/tmp/old，沿用户最新纠正，不使用上级old。
