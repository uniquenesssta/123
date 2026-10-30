# R13～R18 执行约定与验证命令

## 适用范围与事实基线

本约定配套六份阶段任务书的24个交付批次，保留原67项义务，不增加新产品功能或UI改版。R12已由用户确认结束，禁止把本轮任务书补充解释为重开R12。规划订正本身不授权开发；当前执行状态以[R13索引](R13-core-ui-features/README.md)及用户最新指令为准。对应关系见[任务审计](task-plan-review-20261030.md)。

源码盘点基于 `0b1132bd265adbd7d689c5345986c412fffb3ba4`。卡中的函数、路径和检查命令是此基线实际存在的入口；目标职责是待实现设计，不表示相应Feature文件已存在。执行前以最新提交复核，前批迁移后用其交接记录更新后批锚点，不能照抄失效行号。

当前 `src/main.ts` 仍承担页面业务分派；`src/bootstrap/registerApplicationModules.ts` 注册单个 `browser-application`，再调用 `createBrowserApplicationModule`。`ApplicationModule`/`MountHandle` 是已有应用生命周期接口，不是已经完成的逐页面Feature挂载接口。R13-01先完成小范围接入试点，采用符合现有Router和页面生命周期的最小公开接口；禁止假定所有页面都能直接注册为全局模块，或借此重做R12启动架构。

## R13启动前的前期遗留核对

先读取[已完成的R1～R12基线核对与16项承接账本](prior-stage-baseline-20261030.md)，核对其受测提交与当前差异。各批执行卡已绑定具体BA编号；发现的阻塞问题先处理，再继续依赖工作，不能把已知缺陷统一留给R16/R17。仅原已批准的真实新库/外部验证可逐项延期，最迟R17-06关闭。当前基线静态通过不等于全软件已验收，也不自动启动R13。

## 每批开始前的清单

1. 读取对应任务书、前批交接及当前源码，记录分支、基线SHA、工作区已有修改、前置项证据和本批原ID。发现用户修改时保留，不覆盖或回退其内容。
2. 从代表函数追踪所有事件分支、间接调用、异步回调、注册表、DTO消费者和相关脚本。不得把卡上列出的几个函数当成完整迁移范围。
3. 在本批实施记录建立下表，填写真实符号与签名；不为填表新建多份重复文档。尚未决定的目标路径应写“拟定”，实施后替换为实际路径。

| 现有入口/事件/状态 | 实际消费者 | 命令/参数/结果/错误 | 目标唯一owner及公开出口 | 对应验收场景与测试 |
| --- | --- | --- | --- | --- |
| 按源码逐项填写 | 含动态路径和测试 | 引用真实声明及契约 | 明确迁移、复用或保留原因 | 使用执行卡V编号，并补遗漏 |

4. 核对卡中期望行为与当前实现。若当前存在竞态或错误，不把旧行为当作必须保留的契约；先补可失败的行为测试，再修复。涉及业务规则改变、Schema或wire不兼容时，停止扩大范围并说明所需决策。
5. 外部环境缺失时提前登记具体待验项、环境条件、责任批次和阻塞影响；不能到交付才用一句“环境原因”覆盖全部未验内容。

**开发脚本的临时目录需单独提醒用户配置。** 当前已有项目级脚本，后续执行开发/测试前确认 `O:\Project\FOODBALL\tmp` 存在并在该PowerShell进程加载；不修改系统级环境变量，不自动清空用户Temp。

```powershell
Set-Location -LiteralPath 'O:\Project\FOODBALL\123 r3'
. 'O:\Project\FOODBALL\project-temp.ps1'
if ($env:TEMP -ne 'O:\Project\FOODBALL\tmp' -or $env:TMP -ne $env:TEMP) {
    throw 'Confirm project TEMP/TMP before development scripts'
}
```

## 实现边界与分步交付

- 先完成调用盘点和失败场景，再按执行卡顺序迁移读取、写入及界面接入。每步保留可运行路径；批次可以包含多个可回退提交，但不得同时维护两份可写业务状态。
- Feature拥有本域查询、编辑草稿及操作工作流；View只呈现数据和发送操作意图。Router、Session、Workspace、Platform继续使用既有公开能力。跨域调用通过具名公开出口，不读取另一域私有store；不要为了规避守卫增加中转壳。
- 按职责拆分状态、查询、动作、映射和公开出口，只在确有职责或消费者时创建文件/目录。不存在固定文件数、行数或必须套用的目录模板。公共DTO应有一个稳定owner；UI投影、领域概念与wire DTO不可混为同一个共享对象。
- 命令名、参数名、序列化格式、错误码、存储键及模型资产是兼容边界。先从 `src/api/client.ts`、Tauri命令和对应 `contracts/` 核对，再移动调用。不得以“模块化”为由改Schema、概率算法或资产内容。
- 源码路径改变时同步更新真实测试目标与架构扫描范围。保持原断言意图；不能删除必要检查、放宽依赖约束、硬编码结果，或把调用失败转成绿色状态。
- 每个原ID保留可追溯结果。合并批次中的一部分完成，不代表其余原项自动完成；不重复实现已有正确能力，也不把必要能力删掉来收口。

## 异步、确认与恢复规则

1. 发起请求时捕获实际业务身份：按流程选择查询条件/游标、实体ID、比赛/快照/模型、会话/profile、预检批次/文件摘要等必要字段。使用现有生命周期及请求序列工具，避免另建全局取消系统。
2. **每次await结束后、写入当前界面状态之前重新核对身份及生命周期。** await之前缓存的“仍在原页面”不够；列表刷新后还可能已经换会话。过期读响应不可覆盖新对象，原写入结果仍应归属原对象。
3. 取消分为停止本地回填和后端真实取消。销毁View或AbortSignal失效不证明后端任务已终止；任务状态以实际服务响应/读回为准。
4. 关键写入锁按业务对象/请求设定，保持既有合理并发语义。超时或连接中断可能是“结果未知”；先按批次/运行身份读回再决定重试，不自动重复删除、导入、发送或晋升。
5. 删除、重置、导入、冻结、晋升等确认绑定具体对象及必要版本。对象、输入或连接变化后旧确认失效；取消确认不发写入。前端门禁不能代替后端事务、权限或幂等约束。
6. 失败保留可恢复的非敏感草稿和错误归属；不能用全局清空解决竞态。密钥、数据库密码和完整敏感URL不进入草稿持久化、截图、日志或公开报告。

## 验证层级与共享门禁

每批先审查并执行卡中相关脚本，补足场景，再运行受影响的共享门禁。以下命令是项目已有入口，不表示本次文档编辑已运行。所有原生命令立即检查退出码，不能只看最后一条成功。

```powershell
npm.cmd run verify:architecture
if ($LASTEXITCODE -ne 0) { throw 'Architecture verification failed' }
node scripts/verify-command-contract.mjs
if ($LASTEXITCODE -ne 0) { throw 'Command contract verification failed' }
node scripts/verify-protected-assets-deterministic.mjs
if ($LASTEXITCODE -ne 0) { throw 'Protected assets verification failed' }
npm.cmd run verify:frontend
if ($LASTEXITCODE -ne 0) { throw 'Frontend verification failed' }
```

Rust、Tauri、契约或相关构建配置发生变化时，另执行以下现有入口。纯前端批次如引用此前Rust结果，必须写出原受测SHA、当前Rust受测树/锁文件与环境是否相同；最终候选仍按R18门禁核验，不能把复用证据写成本批重新运行。

```powershell
npm.cmd run verify:rust
if ($LASTEXITCODE -ne 0) { throw 'Rust verification failed' }
```

| 层级 | 能证明什么 | 不能代替什么 |
| --- | --- | --- |
| 静态/架构/契约检查 | 扫描范围内依赖、规则及接口一致性 | 真实交互、竞态、数据库效果 |
| 类型检查与构建 | 受测代码可编译/构建 | 功能正确、安装和启动 |
| 自动行为测试 | 实际执行路径在指定输入及调度下符合断言 | 未执行路径、真实外部服务 |
| Windows浏览器交互 | 表单、焦点、滚动、操作链与受控响应 | Tauri原生能力与真实PG事务 |
| Windows安装/实际运行/startup | 指定包能安装、启动及其已执行操作 | Full完整业务链路 |
| 真实DB/API/Full | 受测环境中的真实读写、账本与完整操作证据 | 其他版本、未覆盖模型或测试target |

场景测试可在外部传输边界安排延迟、乱序、失败，必须调用真实生产工作流。不能在测试里重写一遍状态机再断言自己的实现。涉及数据库原子性、历史不可变或幂等时，需实际库读回。优先扩充既有runner/fixture和测试入口；不为增加检查数量而建重复框架。

前端交付须有受影响页面的Windows实际操作证据，包括从界面触发真实命令；仅浏览器模拟无法证明原生功能。允许按已确认的最终封包流程延期的外部项，逐项登记到R17-06，不能称为已通过；R17-06与R18必须闭合必需项。

## Windows Full的实际前提

先阅读 `scripts/windows-acceptance.ps1` 和 `contracts/windows-acceptance-contract.json`。当前脚本从 `ProjectRoot\项目源码` 找源码；仓库目录是 `123 r3`，因此不能把 `O:\Project\FOODBALL` 直接当作可用的ProjectRoot。CI的checkout布局与本地不同。

使用已准备并核对的验收目录，确认其中源码SHA和受测内容等于候选；不要为执行命令擅自移动用户项目或虚构目录。下面 `$acceptanceRoot` 是操作人事先赋值的已核实目录变量，不是现成的项目环境配置。没有合适布局时先明确缺口及最小准备办法，再执行；禁止跳过路径检查制造成功。

```powershell
if ([string]::IsNullOrWhiteSpace($acceptanceRoot)) { throw 'Set a verified acceptance root first' }
$acceptanceSource = Join-Path $acceptanceRoot '项目源码'
if (-not (Test-Path -LiteralPath (Join-Path $acceptanceSource 'package.json'))) {
    throw 'Acceptance source layout is missing'
}
# 执行前核对该源码的提交及未提交差异，并确认专用测试库已授权。
& (Join-Path $acceptanceSource 'scripts\windows-acceptance.ps1') -Mode Full -ProjectRoot $acceptanceRoot
if ($LASTEXITCODE -ne 0) { throw 'Windows Full acceptance failed' }
```

环境预检至少包括Windows工具链、脚本要求的Node/Rust版本、`FOOTBALL_TEST_DATABASE_URL`、授权专用新测试库、有效XLSX及资料包、私有P4/P7固定回归资产、所需API和Windows凭据。数据库名的test/acceptance/qa限制与系统库拒绝规则以实际脚本为准，不能关闭检查，也不能仅凭名字合规就使用未授权数据库。记录资产版本/摘要与可用性，报告只记录脱敏的环境标识。

现有Full入口调用两个PG target：`match_catalog_repository_contract`（单测试线程）与 `postgres_integration`，并使用ignored测试；**它不自动覆盖所有repository contract target**。从 `crates/persistence-postgres/tests` 及待验账本逐项核对额外target，例如实体删除、球队强制删除和运行身份。按实际存在的测试名执行，不能把目录内测试数量当执行数量。

Full会构建并启动客户端，随后提示人工操作：测试库连接→真实球队/球员及比赛阵容导入→就绪检查和正式P4→运行历史→比分/复盘→分析→应用内验收报告。当前脚本将正式结算提示为可继续执行；本任务书仍要求真实结算专项证据，不能因该提示可选而漏验。必须实际操作完成后再继续脚本；不能自动发送回车代替业务验收。最终对照本次runtime日志、数据库读回和报告的同一批对象身份。

API会话隔离/取消、Windows凭据、私有固定回归、历史保护与并发回滚等超出上述自动路径的项目仍需专项证据。缺外部环境记阻塞；公开外壳可启动不代表私有模型运行成功。

## 停止、回退与交接

出现以下任一情况，停止本批扩大迁移范围：接口/Schema/模型资产非预期变化；对象身份串线；写入结果未知且无法读回；原有必要功能失联；测试被弱化；双状态owner未收敛。先保留复现及日志，修复后只重跑受影响验证和必要门禁，不反复轮询CI。

代码回退以本批明确提交和用户已有修改清单为基础，审阅后用针对性revert；不使用破坏用户修改的reset或批量删除。文件退出逐项确认替代实现和所有消费者，一次只处理一个明确路径；需要批量删除时交用户手动操作。**Git回退不会恢复数据库、凭据或工作区数据。** 涉及数据的恢复必须有对应备份/版本兼容和实际恢复验证，不能擅自降级Schema、重置生产库或清空目录。

每批交接至少写出：

- 精确提交/受测树、实际修改范围、旧入口/状态/类型/事件到新owner的对应表；保留项及理由。
- 每个主ID及原ID的结果，逐个V场景的前置数据、操作、预期、实际结果和证据路径。
- 命令、退出码、工具/环境、报告及产物摘要；分列静态、构建、自动测试、Windows运行和真实外部验证。
- 未执行/失败/阻塞项及承接批次；本批回退前提和下一批入口，不写笼统“全部通过”。

后续代码提交/推送沿用用户长期授权，不重复询问。只在已有CI结果可核对时核对精确SHA的一次结果，不轮询，不把其他提交或文档变更的CI当作本批证据。发布标签/安装包必须对应经过验证的候选，不能用最后一次绿色记录替代来源核验。

本约定及执行卡的编辑只做文档一致性、路径/符号/命令引用和Git差异验证；不产生新的软件运行验收结果。
