# R14-08 复盘选择、事实与包提交

状态：VERIFYING。本地适用实现与验收完成；本批精确提交的 CI、官方安装包与包内 startup 仍待推送后验收，不轮询。2026-11-01 开始，11-02 完成本地收尾，日志沿用 `O:/Project/FOODBALL/logs/r14-08-20261101`。

用户授权“收尾07开始08”，并已长期批准提交推送。分支 `rewrite/r14-prediction-review-analytics-ui`；起点 `f0899a5ed4379d4a6fe9ef74b6d8bea725b346bb`，开始时工作区干净。07 已按修复代码 `f38b52dd` 的 CI `41314470020`、官方 digest、MSI/NSIS 与包内 startup 正式收尾；本批不再查询07或重开R12。08覆盖原08/09/10/11；下一主交付是 **R14-12 赛后结算与监控**，尚未启动。

## 最终归属与真实调用

| 职责 | 当前唯一实现及入口 |
| --- | --- |
| 目录、比赛选择、实际阵容、工作流和结果读回 | `src/features/review/reviewFeature.ts`；`reviewQueries.ts` 核验比赛/包/SHA/复盘ID，并保留原 actual→confirmed→expected 读取优先级 |
| 手工赛果、球员量化事实、确认字段和展开状态 | `reviewFacts.ts`，按数据库身份与比赛隔离原始草稿；该子树退出通用 Workspace 表单持久化 |
| 不明结果的窄恢复引用 | `reviewRecovery.ts`；仅保存原步骤、对象、包、SHA、路径或版本，不保存密码，不建立前端业务账本 |
| 九步和预检证据 | `reviewSteps.ts`、`reviewWorkflowView.ts`，只消费后端 completed_steps/allowed_actions/blocking_reasons/next_action |
| 手工录入、结果和事件 | `reviewManualView.ts`、`reviewResultView.ts`、`matchEventTimelineView.ts`；保留真实阵容补录、全部量化字段、角色来源、事件双状态轴、完整预检/复盘/候选证据 |
| 共享控件实际交互 | `reviewControls.ts` 挂载共享 Select/NumberInput 控制器；每次重绘及退出销毁，避免重复事件与计时器 |
| 页面及样式 | `reviewPage.ts`、`reviewViewParts.ts`、`review.css`；公开入口 `src/features/review/index.ts` |

根入口 `src/main.ts` 只连接真实 Feature/Facts、现有确认弹窗、导航和阵容补录；原8项可写复盘全局状态及 pageLoaders 中两套加载器已退出。`src/pages/review.ts`、`src/pages/review/index.ts` 逐项退出；旧事件文件迁入Feature，Data Display真实消费端也改用公开入口。没有保留空壳、复制状态机、扩张边界例外或关闭检查。18个Feature仍按实际调用图核对；状态所有权由58项增加为60项。

沿用 `src/api/client.ts` 现有查询、文件选择、导出/预检/确认/事实提交/生成/候选决定接口，真实按钮均进入原 wire 命令；API、171命令、Schema、Rust和模型算法未改。R13的工作簿底层接口继续复用。14-12既有结算入口只改读新owner并补对象/数据库/页面迟到保护，本批不实施结算业务重构。

## 设计与排布依据

使用 Impeccable harden、polish/craft-floor 与 Figma design-to-code。上下文采用现有已批准系统，无新依赖或视觉方向。

Figma `PN0Whgu6HLWIHx4Mv6aHfu`，Screens `3:5`：Loaded `505:33973`、PreviewReady `505:34365`、ManualEntry `505:35051`、ResultsEvents `505:34953`。整页上下文请求超时后，按核实ID读取实际 Stepper、CurrentTask、ManualInput/主队、Result、PreviewEvidence、Timeline/PlayerReview等小实例；稀疏父容器不作为实施依据。候选区另外只读核对至实际文本和按钮，不修改Figma。Loaded设计截图在日志 `figma-loaded.png`。

左侧280px步骤轨道、12px模块间隙、12px面板内边距、16px面板圆角；内容视口单独滚动，手工提交栏留在视口外。1440×900作为标准状态，960/1280/1920按容器收缩；紧凑状态隐藏步骤说明，保留可读字号与完整任务详情。共享FIP图标、按钮、Select、NumberInput、Alert、StatCard与Timeline都实际复用。

标题26/700；说明12/400；字段与辅助文字按现有令牌。数字指标使用Inter 28/600/34；“无预测”使用Noto Sans SC 22/700/30；完整长版本标识使用14px文字层级以便换行，不截断身份。已修正局部段落规则对共享指标字号的覆盖。同行数字控件和补充操作中心对齐，展开量化字段及两队列表使用固定间隙。

## 行为与恢复证据

| 场景 | 已接通行为与验证 |
| --- | --- |
| V1 对象和摘要变化 | 通用入口等待明确选择；A/B快速切换、失败B、数据库或页面ABA不复用旧内容。确认绑定原比赛、包、完整SHA、输入及查看版本；文件选择取消、旧确认或预检后换文件均零事实写入 |
| V2 部分成功 | 导出、预检、确认、事实提交和生成分别读回；事实响应丢失后只查回原包的事实时间和实际阵容，生成失败保留事实已提交状态，禁止重放事实。未知写入无法证明时保留恢复引用，不拼成全成功 |
| 文件响应缺失 | 原SHA/包ID未返回时不猜测成功。明确核对当前登记的比赛、包、文件、完整SHA后，可只读采用该状态；不把原请求认定成功、不自动重发，此入口不能清除未知事实写入 |
| V3 拒绝与重复 | 后端许可决定执行能力；查看锁定步骤不解锁。请求锁、一次性确认与新鲜工作流复核阻止重复/并发。刷新或详情失败准确返回失败；输入保留。共享选择器、数字步进和真实键盘输入均实际操作验证 |
| 手工事实 | 两队实际阵容必须齐备；未知比分、分钟、指标、确认时间保持空白，实际0有效；不推断90分钟、不补0、不猜换人。已有事件/换人保留；角色继承/本场覆盖保留来源 |
| V4 结果与候选 | 正式复盘、结算和候选决定分开；只读结果不自动接受候选。接受/拒绝单独确认并核对原候选状态；取消零写入。取消事件保持审计记录与独立核验/修订状态，后端摘要不由前端重算，被替代事件的历史比分仍保留 |

## 分层验证

- 静态：复盘资料包、事件事实、Review service、分析历史、模块/状态/保护边界及命令等原门禁；测试迁移至实际Feature，不保留旧源路径或修改义务来制造通过。
- 完整前端：`frontend-final-3.log`，完整96步、TypeScript、Vite构建退出0；其中生产复盘业务31项、真实挂载浏览器51项通过。
- 最终定向复验：随后修正局部指标字体层级及选中对象刷新返回值；生产业务增至33项，真实复盘浏览器增至55项，21项导航、类型、样式与受影响静态门禁复验通过；新建干净dist复建。最终截图/逐项结果位于 `browser-final-approved-type` 与最终浏览器目录，实际输出见 `validation-summary.json`。原500kB分包提示保留，未调阈值。
- Impeccable：本批使用hardening与polish，检测与最终人工截图复核分别记录；检测器不能代替运行或业务验证。
- Windows原生：`native-final/report.json`，真实Tauri/WebView和真实命令，未装QA边界；数据库已配置，可复盘目录实际0项，与UI一致，9步保留，刷新正常，空态手工提交禁用，无业务库写入。`startup.json`：28条日志、8个完成操作、0无效行/运行错误，startup PASS。没有将此结果写成Full、真实写入或原生文件对话框全链路通过。
- Rust：与已验收 `f38b52dd` 的 crates/src-tauri/Cargo清单对照无变化，复用已核实的fmt/clippy及748通过、40忽略证据；本批未冒称重新运行Rust。精确源码树及旧日志摘要在验证汇总中记录。
- 交付：本批自身CI及官方双包在推送后按精确SHA单次核验；运行中保持VERIFYING，不以07的成功替代08。

## 待验、回退与临时产物

BA-02/07/12/13真实新库/持久化、多Port原子性及并发、私有模型样本仍按已批准R17-06承接，不用QA传输抵销真实证据。后端事实/复盘与工作流登记分开，未知中间结果继续阻断盲重发；本批不以UI恢复掩盖该真实原子性待验。R17-06及R18不得再次延期必须外部验收。

回退以本批基线f0899a5e为参照；代码回退不撤销已成功写入的事实、复盘、结算或能力历史，必须先查真实账本。未知请求不能通过换页面/删本地引用冒充已回滚。

开发进程TEMP/TMP使用 `O:/Project/FOODBALL/tmp`；必要证据留logs，构建/浏览器临时产物按清单归档到唯一正确目录 `O:/Project/FOODBALL/tmp/old`。保留依赖缓存；不批量、递归或循环删除，不操作根目录old或用户Temp。归档清单见本批日志。
