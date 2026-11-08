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
