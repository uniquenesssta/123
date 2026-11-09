# 独立赛前采样与初始 α 拟合

本工具解决未校准 P4 无法先产生正式预测快照的首次采样问题。`capture`直接读取原PostgreSQL输入构建；`fit`通过本机私有提供器评估原模型，不执行正式预测、结算或参数晋升。公开源码不包含私有P4公式、参数或默认数据。

工具已具备采样、赛果关联、初始估计与时间留出验证入口。真实校准仍需合格真实资料、合理的拟合设置及人工审核；QA结果不能启用真实Profile。应用内原H/I候选入口保持原边界，尚未增加界面按钮。

## 1. 进程与构建

Windows PowerShell，使用已有Node/Rust依赖，没有新增第三方生产依赖：

```powershell
Set-Location -LiteralPath 'O:\Project\FOODBALL\123 r3'
. 'O:\Project\FOODBALL\project-temp.ps1'
cargo build --locked -p football-persistence-postgres --example capture_initial_calibration
if ($LASTEXITCODE -ne 0) { throw 'Capture producer build failed' }
```

采样连接只从进程环境`FOOTBALL_CALIBRATION_DATABASE_URL`取得；不自动使用日常连接、不执行迁移、不打印URL或把密码写进制品。使用前由操作人配置已授权连接。采样程序通过SQLx解析实际数据库名，测试/QA库只能标记`qa`。

私有拟合提供器由模型宿主构建。本机验证使用`O:\Project\FOODBALL\123\crates\model-p4\examples\initial_calibration_oracle.rs`，输出`O:\Project\FOODBALL\.cargo-target\debug\examples\initial_calibration_oracle.exe`。私有源码/参数保留在本机忽略路径；公开安装包不会自动拥有该提供器。

## 2. 赛前采样

先明确创建项目内的数据目录，再提供实际比赛UUID、参数/Profile路径和来源范围：

```powershell
node scripts/calibration/cli.mjs capture `
  --producer 'O:\Project\FOODBALL\.cargo-target\debug\examples\capture_initial_calibration.exe' `
  --match '<实际数据库比赛UUID>' --horizon T-N --scope qa `
  --parameters 'O:\Project\FOODBALL\123\src-tauri\resources\defaults\p4_parameters.json' `
  --profile 'O:\Project\FOODBALL\123\src-tauri\resources\defaults\p4_profile.json' `
  --directory '<已创建的项目数据目录>'
```

- 只能在真实系统时间早于开球、比赛仍scheduled时采样；完成输入读取时再次检查时点，不能传入伪造“参考时间”或为历史比赛回填快照。
- 复用原阵容/历史/能力输入及其质量说明，采样不调用模型、不要求先存在P4运行。数据库读取不足会按原门禁失败；未知特征的置信度不会被当作已知资料拟合。
- 制品包含比赛/赛事/窗口、实际采样时刻、原输入、数据质量、参数/Profile快照及摘要。新文件使用排他创建和持久化写入；已有文件不可覆盖，半写文件不能通过摘要/结构检查。凭据字段拒绝落盘。
- 每次产生独立UUID采样文件。建立拟合集时每场同窗口择一有效采样；同一场重复进入训练/留出会被拒绝，不能用复制文件增加样本量。

## 3. 关联赛果

完赛后准备核验过的JSON：`match_id`、`status:"finished"`、`scope`、`home_goals_90`、`away_goals_90`、带时区的`confirmed_at`、`source_url`，可附`source_match_id`。缺比分不能补0；真实来源使用不含凭据的HTTPS链接。核验时刻应如实表示何时确认资料，不虚构历史终场时刻。

```powershell
node scripts/calibration/cli.mjs outcome --capture '<采样文件.capture.json>' --result '<已核验赛果.json>'
```

赛果写为独立制品，绑定原采样摘要；不改写赛前快照，也不代替应用正式复盘/结算。

## 4. 初始拟合

设置JSON必须明确`configuration_scope`、`minimum_training_matches`、`holdout_matches`、`lower_bounds`、`upper_bounds`、`initial_steps`、`tolerance`和`max_evaluations`。各向量按提供器公布的维度顺序；没有真实α范围的默认推荐值。QA验收使用的样本数和边界只是工具夹具，不是统计充分性标准。

```powershell
node scripts/calibration/cli.mjs fit `
  --directory '<已配对采样目录>' --settings '<明确的拟合设置.json>' `
  --oracle 'O:\Project\FOODBALL\.cargo-target\debug\examples\initial_calibration_oracle.exe' `
  --output '<新的结果制品.json>'
```

处理顺序：摘要和身份核对→单赛事/窗口/基线/来源分区→同场去重→按开球时间留出→检查训练赛果在留出采样时已经可知→特征秩与显式数值检查→有界坐标搜索→独立留出评分。相同开球时点不得跨分割；模型范围、置信度或矩阵范围不满足时拒绝评估，不静默删样本或改公式。

P4本机适配器使用同一原自动基础λ计算和原full矩阵评分。在隔离的估计输入副本中将计算出的基础λ作为固定输入评估；不修改封存样本、现用参数或Profile启用状态。拟合目标为原精确比分NLL，留出同时比较原1X2 log-loss、Brier及比分NLL。搜索是受边界/预算约束的局部估计，不保证全局最优。

## 5. 结果与启用边界

| 状态 | 含义 |
| --- | --- |
| BLOCKED | 缺采样、身份/时间错误、参数设置无效、特征不可识别等；不生成候选 |
| NOT_CONVERGED | 预算内未达到搜索终止标准；退出2 |
| VALIDATION_FAILED | 留出指标退化；退出2 |
| QA_ONLY | QA工具链验证通过，只能作为QA证据 |
| REQUIRES_MANUAL_REVIEW | 真实来源候选达到本次设置的验证条件，仍待人工/领域审核 |

所有结果保持`alpha_calibrated:false`和`direct_activation_allowed:false`；只给出带原值、候选值、采样/赛果摘要、实际私有程序摘要和指标的参数补丁。不会写回原参数文件、改变CALIBRATED/eligibility、覆盖正式版本、自动晋升或代替Windows Full。真实启用须另完成版本登记、数据/统计适用性审核及既有受控流程。

具体实现和分层验证见[R17-06记录](modular-rewrite/R17-verification-system/R17-06-real-environment-and-full.md)。

## 6. 真实来源审核与 Profile 口径（2026-11-09）

本轮已对实际来源执行逐场审核，证据根`O:/Project/FOODBALL/logs/r17-06-real-data-20261109`，不是新增合格训练样本：

- 已有StatsBomb英超2015/16的380场比赛，与[OpenFootball同季来源](https://github.com/openfootball/football.json/blob/master/2015-16/en.1.json)的日期、双方和终场比分逐场一致，20队身份归一仅去明确FC/AFC后缀且无碰撞。340场通过“仅取较早日期、最近12场中双方各至少4场历史及2场对应主客场”的数量筛查；这只是数据准备潜力，不证明当时可知、不生成特征或校准资格。原始开球时区、首次资料可用时间仍未逐场核验，不能用last_updated充当当年首次发布时间。
- [OpenFootball 2026/27来源](https://github.com/openfootball/football.json/blob/master/2026-27/en.1.json)实际380行，只有45行明确ft结果；55行过去/当天日期没有明确ft，其中5行score为数组，全部保持未知，不自行当0-0或补赛果。280条未来赛程已按实际观察时间、文件摘要留作候选清单，最早2026-11-21；它们尚无官方赛程/时区核验、匹配Profile或赛前模型特征，不自动入库、采样或执行推演。此源最新明确ft日期为2026-09-20，不能因README写自动更新就认定当前赛果完整。
- 已保留[OpenFootball CC0声明](https://github.com/openfootball/football.json/blob/master/LICENSE.md)。StatsBomb按[官方使用/署名说明](https://github.com/hudl/open-data/blob/master/README.md)记录来源，原件仅留本机证据。[Football-Data.co.uk当前说明](https://football-data.co.uk/data.php)明确限制自动化/AI训练用途，本轮仅审阅其说明，不将其加入自动训练采集源；不购买服务或使用激活API凭据。

### 已确认的模型前置问题

当前默认P4 Profile用于中立单场淘汰赛，并非英超联赛；环境置信度仍未知。原算法在中立/无环境信号时会令对应系数没有可识别特征。即使增加比赛数量，也不能使恒定无信号的维度变为可识别。现有四维拟合门禁正确拒绝，不应给未知置信度填正值或任意α来消除拒绝。

### 建议的下一项与停止条件

下一项先审定“目标赛事专属Profile及可识别参数口径”，再执行真实拟合：

1. 若继续英超，建立独立联赛研究Profile的版本和适用范围，不覆盖原中立淘汰赛Profile；训练/留出按实际目标赛事划分。若目标仍是中立淘汰赛，则更换匹配来源，不混用联赛样本取得资格。
2. 复用原赛果历史特征算法；先定义环境/基础进球率的真实来源、可用时点和映射依据，明确哪些量从训练数据估计、哪些固定。源数据无法支持的维度保留“不可识别”，不得虚构观测或绕过原运行门禁。
3. 拟定“无作用维度固定并逐项披露、仅估计可识别维度”的审核方案，与现有必须四维全秩的工具规则对照。该方案尚未实施或批准，不能默默删除维度后宣称四类α全部校准；接受标准、边界和人工审核需一并明确。
4. 首选真实赛前观测。若选择事后历史重建，必须独立标记、核验当时可知信息，并另行审定其适用范围；不能改名写成live_pre_match，不能倒填数据库created_at/finalized_at或解除历史拒绝。

本轮只完成真实来源采集/资格审核和方案整理。未改模型公式、参数、Profile、样本资格或拟合门禁，未生成α。原`fit`入口对这些来源文件实际返回BLOCKED且未产生候选；仍需完成上述前置口径，才有条件继续正式P4/结算/Full。R17-06和R18状态保持。

## 7. 联赛研究 Profile 与估计口径（2026-11-09）

英超采用独立的 `epl-2026-27-research-v1`。参考期取已有双源一致的 2015/16 英超全季，380 场 90 分钟赛果共 1,026 球，固定基率为每队场均 1.35 球。它是基于较早赛季的研究起点，2026/27 的适用性需用真实时序留出检验。参考清单、两份来源的摘要、资料实际观察时间、Profile 建立时间和目标时段均随制品保存。基率在建立 Profile 时固定，之后的训练和留出共用这一基线。参考来源为 [StatsBomb](https://github.com/hudl/open-data/blob/master/data/matches/2/27.json) 与 [OpenFootball](https://github.com/openfootball/football.json/blob/master/2015-16/en.1.json)。

`league-profile` 读取已人工或程序逐场核对的参考清单。清单使用 `football.initial-calibration.league-reference.v1`，须包含 `scope`、`profile_id`、`competition_name`、`reference_season`、`target_season`、`target_from`、`target_until`、`observed_at`、至少两份不同的 `source_evidence`（URL 和 SHA256），以及按唯一 ID 标注日期、双方 90 分钟进球及 `cross_checked` 的 `matches`。这些标记来自上游来源审核，命令负责结构和时序校验、求基率和写入带摘要制品。本机已复算真实参考来源，产物位于 `O:/Project/FOODBALL/logs/r17-06-league-profile-20261109`。

```powershell
. 'O:\Project\FOODBALL\project-temp.ps1'
node scripts/calibration/cli.mjs league-profile --reference '<已核对的参考清单.json>' --output '<新的联赛研究.profile.json>'
```

采样沿用第 2 节，`--profile` 传入上述带摘要制品。采样必须匹配联赛类型、赛事名称、目标时段及来源范围，并具备数据库赛季身份。参考资料观察和 Profile 建立时间必须早于或等于特征截止时间。拟合设置额外填入 `target_competition_id` 和 `target_season_id`，与每场采样的数据库身份一致。

研究 Profile 以中性环境作为固定建模参照，原环境系数固定为基线参数的值。拟合器向原私有 P4 核对训练和留出的环境项均为零，随后检验攻、防、主客场三列在训练集上的秩为 3。三项边界和步长按 `attack_alpha`、`defence_alpha`、`venue_alpha` 顺序提供，原 Profile 路径继续使用全维度设置。联赛候选只包含三项估计补丁，环境系数及固定理由单列。训练赛果还须在最早留出特征截止前已知，留出三项评分允许的浮点误差为 `1e-9`。真实研究成功状态为 `RESEARCH_ONLY`，合成验收成功状态为 `QA_ONLY`。真实样本量和数值边界须在首次真实拟合前按目标数据与模型范围审定。

下一项核验首批官方赛程与时区，补齐所选窗口的赛前输入，绑定数据库赛事和赛季并开始真实采样。可用于拟合的赛前冻结样本与赛果配对当前为 0。

## 8. 首批真实采样准备（2026-11-09）

英超 2026/27 第 11 轮的 10 场比赛已逐场核对。[11 月改期公告](https://www.premierleague.com/en/news/4725574)、[官方全季赛程](https://www.premierleague.com/en/news/4675097/all-380-fixtures-for-the-202627-premier-league-season)和英超官方 FPL 赛程记录的 UTC 开球时间一致。旧候选中有 5 场时间已变更。首场为曼城主场对富勒姆，英国当地 11 月 21 日 12:30 GMT、北京时间 11 月 21 日 20:30，T-1h 采样窗口为北京时间 19:30:00 至 20:29:59。临场重新确认赛程，双方正式阵容在所选窗口内真实观测并记下采集时间。[曼城赛程](https://www.mancity.com/tickets/mens/man-city-v-fulham-21-nov-2026)、[富勒姆比赛中心](https://www.fulhamfc.com/matches/men/2026/g2645302)。

已保存赛季前 10 轮的 100 场历史赛果，共 273 球。90 场在 FPL 源中为已结束，第 10 轮 10 场的源状态仍为暂定，逐场与联赛或俱乐部正式战报复核后保留两类证据及核对时间。首场两队各有 10 场历史赛果、5 场相同主客场历史。官方 FPL 球员名录有曼城 33 人、富勒姆 28 人，记录了来源 ID、位置分组与观测时间；正式首发、具体位置和程序中的球员身份还需临场核对。

证据保存在 `O:/Project/FOODBALL/logs/r17-06-first-live-inputs-20261109/first-live-inputs.json`。制品记录了赛果来源、赛程时区、各场采样窗口、联赛研究 Profile 摘要及尚待完成的数据库绑定；原始快照和生成脚本同目录保留。已复算制品摘要、检查全部比赛身份和时间，并实跑 `fit` 确认该目录在没有赛前冻结制品时被拒绝。真实配对仍为 0。

下一项先打通官方赛果在独立研究库的导入，绑定赛事、赛季和稳定的球队身份，实际读回并验证原历史特征接口在截止时间内使用这些数据。正式复盘继续使用原球员表现校验；官方比分导入只承担可核验的历史赛果。真实赛前采样须等到窗口开放，且双方合格阵容进入原输入链后再执行。

## 9. 独立研究库历史赛果导入（2026-11-09）

`import_calibration_history` 读取第 8 节已核对的英超 2026/27 制品，验证摘要、赛季、来源时点和比赛身份，再用一个事务写入赛事、赛季、球队、赛程及历史比分。比分记录关联来源和导入批次。FPL 暂定比分须有逐场正式战报核对。数据库的赛果可知时间取实际来源观察和战报核对时间中较晚者，入库时间用数据库实际时钟。重复导入保留原时间和身份，既有比分或赛程冲突会回滚整批。球队 ID 按本赛季 FPL ID 绑定。

入口限本机 `127.0.0.1:55438/football_r17_research` 的专用低权限账号。使用现有迁移和赛果表，正式复盘、阵容、球员表现及模型冻结记录仍由各自入口维护。Windows 研究实例及受保护连接保存在本机项目 `research/r17-calibration-postgres`，验收结束后停止。复用时由本机证据目录中的 `research-postgres.ps1` 启动并向当前进程注入 `FOOTBALL_CALIBRATION_DATABASE_URL`。

```powershell
. 'O:\Project\FOODBALL\project-temp.ps1'
cargo build --locked -p football-persistence-postgres --example import_calibration_history
& '<import_calibration_history.exe>' init
& '<import_calibration_history.exe>' import --packet '<已核验的 first-live-inputs.json>'
& '<import_calibration_history.exe>' inspect --match '<导入返回的比赛 UUID>' --cutoff '<已发生的 RFC3339 赛前截止时间>'
```

本机导入 100 场赛果、10 场未来赛程、20 支球队，重启后逐场核对 110 条。首场曼城对富勒姆（FPL 108）绑定为 `b7e98ed5-0078-82ab-a511-31d456ea935d`。原历史特征读回在来源核对后、入库前的截止点每队为 0 场，入库后每队 10 场，其中相同主客场各 5 场，联赛基线覆盖 100 场。重复导入新增 0 条。摘要异常、缺战报的暂定比分、既有比分冲突、非研究库连接和未来截止点均被拒绝。验收记录在 `O:/Project/FOODBALL/logs/r17-06-research-history-20261109/verification.json`。

下一项为曼城、富勒姆的官方球员身份、有效所属和正式阵容导入准备。开球时间若有变动，先依据官方公告核对并调整绑定赛程。到实际 T-1h 窗口且双方首发可用时，再通过原输入链采样。真实冻结配对仍为 0。

## 10. 近期首测改用挪超（2026-11-09）

北京时间 11 月 10 日 02:00，挪超第 26 轮克里斯蒂安松对瓦勒伦加在 Nordmøre stadion 开球。T-1h 采样窗口为 01:00:00–01:59:59。独立研究库中登记的新首测键为 `NFF-ELITESERIEN-2026-8986331`，比赛 UUID 为 `ca281c34-d392-e4fd-06a6-c051c533eb3d`。[挪威足协比赛页](https://www.fotball.no/fotballdata/kamp/?fiksId=8986331)、[改期公告](https://www.fotball.no/turneringer/eliteserien/2026/kristiansund---valerenga-utsatt/)。

挪超 2026 赛季单独绑定赛事、赛季和球队身份。实际导入挪威足协已结束赛果 207 场、球队 16 支及本场赛程，双方各有 25 场历史、相同主客场各 12 场。重新导入后比分行摘要一致。英超记录作为原研究批次保留。来源快照和数据库读回保存在 `O:/Project/FOODBALL/logs/r17-near-term-match-20261109`。

首测阵容待官方发布。配对完成需要在窗口内冻结合格赛前输入，并在赛后关联官方 90 分钟比分。挪超后续 α 研究使用独立的联赛 Profile 和相应数据，当前合格配对为 0。
