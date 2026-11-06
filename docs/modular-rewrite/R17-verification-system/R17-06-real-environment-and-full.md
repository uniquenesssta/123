# R17-06 真实环境、工作簿与 Windows Full

状态：部分实证完成；BLOCKED，未验收完成。用户授权“收尾01开始06”，长期提交推送授权有效；基线2101ce145d58d1516fdb5b252140b0e16d6edae8，沿用rewrite/r17-verification-system。R17-01自身CI/官方双包/startup已正式收尾。本项不把Automated或本次数据库自动测试当作Windows Full，不进入R18。

## 真实环境与授权

现有127.0.0.1:5432的应用账号没有建库权限；最初CREATE DATABASE被拒绝，没有清空或改写日常football_model。用户随后选择“自动启动临时测试实例”：使用已安装的PostgreSQL 18.4，在O:/Project/FOODBALL/tmp/r17-06-postgres/data建立独立实例，只监听127.0.0.1:55437。既有三个repository契约要求名称包含test，保留这些检查，使用football_r17_test_acceptance_20261107。初始化后在该实例将同名验收库重建为普通账号r17_test_owner所有；最终全部PG及XLSX以该无SUPERUSER/CREATEDB/CREATEROLE/REPLICATION/BYPASSRLS权限的账号通过。初始化进程核对data_directory并记录启动身份/OID，普通账号再核对相同启动身份与库OID，不额外授予系统设置读取权限。每轮沿原destructive_reset测试建立清洁迁移基线。最后实例已停止，必要QA数据和DPAPI保护的随机密码保留以便后续依赖验收；没有明文密码文件残留。

进程TEMP/TMP使用O:/Project/FOODBALL/tmp，已单独提醒。没有新增生产依赖、runner、workflow、迁移或持久生产数据库；临时编排脚本只在项目证据中留存，不进入仓库或公开凭据。

用户指定O:/Project/FOODBALL/123作为私有P4/P7校验位置。实际该目录当前为f060748所在R14分支，同样只注册PublicModelStub；crates/model-p4、crates/model-p7、私有参数/固定回归资产均未找到，本地各分支的模型路径历史也没有命中。该目录保持未修改，不能当作完整私有环境。用户已明确“暂不运行真实API和凭据测试”，本次没有计费API请求或Windows Credential Manager保存/删除测试。

## 确认缺陷与修复

| 问题 | 修复与唯一归属 | 保留边界 |
| --- | --- | --- |
| SQL NULL摘要导致整张运行历史无法读取，Application又强制将历史摘要当作完整模型输出 | persistence/runs/read保留NULL；Application的ModelRunHistorySummary专用于可缺失的历史概率；TS注明summary可空 | 不造0概率；模型执行的PredictionSummary仍严格；错误类型仍拒绝；原界面已用可选链显示缺失，无UI改版 |
| SQL NULL惯用脚使月度导出、球员档案及AI工作包读取失败 | monthly_player/read导出空单元格；catalog player行结构保留Option，既有枚举映射为Unknown | 不改写数据库，不猜测左右脚；原已记录值不变 |
| Full说明中的中文示例路径不存在 | examples目录及三份XLSX恢复实际中文名，README随目录移动；原工作簿字节完全相同 | 4项纯移动，无额外文件删除；真实生产解析器新增2项文件读取回归 |

新测试分别位于原Application组合根测试、历史适配测试、spreadsheet-io测试及既有PG target；私有模型实现、概率公式、171个命令、数据库Schema、Cargo/前端依赖版本、UI和PNG基线未变。

真实执行也暴露旧夹具不符合当前契约：回放断言误把T-1h窗口起点当截止；直接阻塞PID计数漏掉锁队列的间接等待；证据重复归属、任务trace复用及第二次导入复用阵容时间违反唯一约束。现在采用生产返回的实际截止，保留相等/后一微秒的原断言；递归观察同一父锁的全部等待链；给独立任务分配独立研究/trace/冲突/路由，并验证相互决策不串线；并发导入用新版本时间。没有删除测试或修改0014/0018等约束。新增manual_peer夹具独立文件负责构造第二个合法上下文。

原静态Lineups守卫同步要求递归阻塞链及原取消/混合并发断言；类型/端口扫描和数据库测试指纹按审阅后的真实源码更新，保留先前接受记录。没有通过跳过检查获得通过。

## 已执行证据

证据根：O:/Project/FOODBALL/logs/r17-06-20261107。各次失败原日志分别保留在attempt01～06目录，最终结果在根目录，不把较早失败或编译结果改写成成功。

| 层级 | 实际结果 | 范围限制 |
| --- | --- | --- |
| PostgreSQL repository/综合测试 | 20个既有target、40个原ignored真实测试全部通过；另先执行原清空重建测试；46次迁移成功、150张业务/基础表读回 | PostgreSQL18.4临时实例；不是日常库兼容迁移或私有模型成功 |
| BA-02四个测试 | match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent、match_scope_inference_and_lineup_pair_transaction_are_atomic、structured_match_events_are_queryable_and_revision_aware、p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable分别PASS | 原四项逐名实跑，模型相关概率是明确测试夹具 |
| 实际XLSX→Application→PG→重建应用连接→读回 | 原Application lib target显式执行1个ignored场景：1156条球队/球员链路及807条比赛/阵容链路写入；读回4队、72球员、864能力、3比赛、12阵容、216阵容明细 | 读取仓库真实工作簿，经原应用导入/预检/提交；非JSON预览替代、非桌面UI Full |
| 重连及重复提交 | 原应用对象断开并销毁，新应用连接后查回相同batch/source SHA；球员批次重复提交返回原结果、比赛批次重复提交拒绝，业务数量不增加 | 证明该应用层持久化/幂等边界；未冒充Windows进程崩溃或真实网络断开 |
| 本地完整前端 | 98步退出0，原工作流/完整挂载/浏览器检查与TypeScript/Vite通过 | 含历史NULL/部分摘要不造0概率的原工作流回归、TypeScript和Vite；未更新PNG |
| 本地完整Rust | 格式、Clippy -D warnings及工作区773项通过；默认41项ignored | 默认ignored仍列出40个PG及新增1个Application XLSX；它们在本次另有显式真实执行证据，不能把默认ignored改报通过 |
| 架构与保护 | 35项完整架构通过；原保护/迁移/Windows契约及模型边界通过 | 原35项架构、Windows验收契约/迁移兼容/公开模型边界/保护资产检查分列，不代替真实Full |
| Windows/CI/包 | 01的精确官方CI/包/startup已收尾；06自身CI/包推送后一次性核对 | 本批未执行Windows Full、重新安装或数字签名验证；不轮询CI或PR |

真实PG命令统一沿用cargo test --locked -p football-persistence-postgres --test <target> -- --ignored --nocapture --test-threads=1；逐项命令、退出码及原日志SHA见postgres-acceptance.json。Application真实工作簿操作、批次身份及读回见application-workbooks.log和application-workbooks-exit.json。普通Rust测试数与显式真实测试数不相加重复计算。

## BA账本与场景交接

| BA | 本次实证 | 尚未关闭 |
| --- | --- | --- |
| 02 | 四项原综合测试分别通过 | 本项四个数据库回归义务已实证；不连带核销Full |
| 03～06 | 当前删除/引用/强删/配对/锁等待/取消/原子回滚/12插入1结束等原契约真实执行 | 桌面同对象完整操作仍随Full核对 |
| 07 | 两份实际XLSX经应用提交并跨连接读回，重复提交边界实跑 | 其余原生文件对话框、月度/复盘包等同状态Full覆盖仍需按原账本确认，不整组核销 |
| 08 | 运行身份、快照复用、事务故障回滚及任务幂等等原PG断言通过 | 私有正式运行及真实客户端未知响应恢复 |
| 09～11 | 原受控工作流回归与PG范围分开 | 真实API、用户会话取消、Windows凭据由用户明确暂缓，不报通过 |
| 12 | 原综合目标内不可变/回滚和工作流持久状态实跑 | 正式复盘→结算→分析/候选→J报告同对象链及B接入契约未获本次完整证明 |
| 13 | 公开边界保持unavailable，未发现可执行私有资产 | 完整P4/P7及原固定回归资产位置待补；不能以Stub替代 |
| 14 | 20target逐项及应用真实XLSX补验完成 | Windows Full及最终安装候选证据仍必需 |

V1：初始账号无建库权限、专用库名被既有守卫拒绝、指定路径没有私有运行时分别如实记录；没有擅用日常库或放行Stub。V2：原PG并发/取消/回滚、实际工作簿提交及重建应用读回通过，保留各自边界。V3：01的startup和本次自动证据均没有被记成Full；外部必需项仍阻塞R17阶段出口。

## 继续条件与回退

下一步仍为R17-06：确认完整私有模型及固定回归资产的准确位置，在匹配当前r3受测业务树的私有环境完成正式链与Windows Full；真实API/系统凭据待用户允许后补验。现有Windows runner的ProjectRoot/项目源码布局和同一对象身份必须先核对；不能在旧123程序上得到结果后套给当前r3。R18保持BLOCKED。

本批源码可回退至2101ce14；无迁移、用户配置或日常数据变更。临时新库保留测试导入数据，实例停止；后续需要重置仍只限该已批准隔离库。成功推送后仅按本批清单将无用临时文件归入O:/Project/FOODBALL/tmp/old，再逐文件实际清理并独立核空；保留证据、受依赖QA目录和稳定依赖缓存。
