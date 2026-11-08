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
