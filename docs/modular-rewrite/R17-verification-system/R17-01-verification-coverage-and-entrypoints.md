# R17-01 验证覆盖与执行入口收敛

状态：DONE。用户已授权收尾R16-09并开始本批；前置c1574990/CI42125458762、官方artifact13462724756摘要/双包/startup已正式接受，R16关闭文档ad743482f25aa44217422187925335e7edce84f1已推送并清理。当前分支rewrite/r17-verification-system从该干净基线建立。

覆盖原R17-01、02、03、04、05、07、08、09的适用自动验证/覆盖义务；真实PG/Full属于原06/10，不据本批核销。完整对象分配见[覆盖矩阵](R17-01-coverage-matrix.md)，不把报告生成当业务通过。

## 真实缺口与修复

1. 原前端总入口只运行verify-command-contract；较严格的verify_command_contract虽然存在且历史手动执行，却没有进入日常98步前端。三方活集合相同不证明机器command_count、owner、注册顺序与哈希正确。现在在原列表相邻接入机器守卫，97→98步，不删任何原检查。
2. 原verify:all只串release/frontend与Rust，缺CI先执行的完整35项architecture。前端只含其中一部分，不能当完整架构。现在all先执行原verify:architecture，再沿原release与Rust；未创建新的runner/workflow。
3. 原Node调用兼容入口扩充7项实际失败传播/契约回归，私有helper位于scripts/process/verifyVerificationFailures.mjs，唯一职责是既有验证入口的隔离故障。实际Node子进程退出37、启动ENOENT、实际npm架构子项退出43分别使顶层停止；实际Windows Automated在受控npm子项37时最终退出1，不进入构建/应用。原契约拒绝count漂移和退役owner；健康三方检查仍通过。外部子进程边界是明确故障夹具，不冒充软件业务成功。

没有生产src/Rust、DTO/Schema/迁移、API行为、依赖版本、模型资产、样式或PNG变化；package仅改验证脚本字符串，锁文件不变。旧测试的CSS解析和迁入owner均复核，没有以缺路径跳过功能。

## 实际覆盖与证据

证据根O:/Project/FOODBALL/logs/r17-01-20261107。执行/页面/原ID/BA/PG清单和command-shapes提供精确生产/测试位置，便于R18按最终候选再核对：17页、40业务ID、16 BA均有分配；Cargo确认20 target/40 PG ignored，Full只选择其中2，额外18必须显式验收。BA-02四个测试逐名保留。源码标签与静态可达图不冒充执行数量。

| 验收项 | 实际证据与结果 | 层级/限制 |
| --- | --- | --- |
| V1 子项失败→顶层失败 | runner-failures.log；最终98步中再次执行7项；frontend退出37/1、all退出43、Windows退出1，后续步骤未调用 | 原runner实际执行，只有隔离子进程边界故障；没有启动用户应用或连接DB |
| V2 旧owner/扫描范围 | 35项architecture通过；原坏契约路径失败；当前生产owner、171签名/参数表达式与各测试可达入口索引 | 静态不等于全字段序列化或全命令实跑 |
| V3 截图/Mock不能核销真实 | 17 fixture视口与17生产页单独列；真实production workflow和main挂载通过，但PG/API/Full仍逐BA OPEN | synthetic runtime full日志仅分析器单测，不是Full |
| 本地完整前端 | frontend-full.log、frontend-exit.json：98步退出0，严格TypeScript/Vite、原行为/完整挂载/原PNG均通过 | G几何与原具名状态；不声称全部状态×主题×尺寸组合 |
| 本地架构/目标 | architecture-full.log：35项通过；cargo-metadata.json与pg-targets.json逐项一致 | 未运行真实PG |
| Rust同树复用 | accepted-ci-backend-excerpt.log：精确c1574990/CI42125458762实际768通过/0失败/40 PG ignored；当前Rust/Cargo/锁/编译配置不变 | 本批没有重新执行Rust全套；cfg原生宿主另列，ignored不变成通过 |
| Windows/包 | 前置09官方双包/startup及此前同生产树原生操作证据可复用；本批负向实际执行原PowerShell拒绝链 | 自身2101ce14/CI42252996208及官方双包/startup已正式接受；不是本机重新安装或Full |

本地没有额外重复完整运行verify:all正向；其完整架构/前端组件当前通过，Rust按允许的同树证据复用，真实npm失败链另行实跑。大于500kB的既有Vite警告保留，不调阈值。明确未做：真实PG/XLSX提交读回、授权API/系统凭据、私有固定回归、安装/签名和Windows Full。现库B接入契约阻断仍BA-12，不能删项放行。

## 自身CI与官方产物正式收尾（2026-11-07）

精确提交2101ce145d58d1516fdb5b252140b0e16d6edae8、分支rewrite/r17-verification-system、push事件、原ci.yml：[run42252996208](https://github.com/uniquenesssta/123/actions/runs/42252996208)，Windows job128012731560完成success。用户授权收尾时一次性复核，不轮询其他提交或PR。

官方artifact13524072272为52,497,709字节，ZIP SHA256为22f258acb64a49060f70b74aadb2e26ceced8c927e84302175501ad4ffbca043，与官方digest一致。MSI18,079,744字节/SHA256 a44125cc10eaba6228c028eef758594a840fcaa298e02fee6cc2c0d113e2c403；NSIS15,709,031字节/SHA256 28464bc47b32b1125f8886d4ad234058c45e53b6a6d2568b072f65351edabce8。双包文件头与非空内容有效，原日志分析器重验startup：7条记录、0无效行、0 runtime错误、PASS。

官方ZIP、双包、原始报告/运行日志、重验startup.json与acceptance.json保存在O:/Project/FOODBALL/logs/r17-01-ci-42252996208。本项未重新本机安装或核验数字签名；startup不代表Full，未核销R17-06。01为4新增/8修改/0删除；其成功推送后旧临时目录已实际逐文件清理并独立核空。

## 原节点与交接

原01：35架构与新失败传播。02：实际生产工作流/生命周期模块及7项回归。03：现有前端业务/契约与双命令守卫。04：原17 PNG fixture、批准生产View和17页完整挂载按范围保留，未重造fixture或更新基线。05：同后端树768单位/集成结果及全部40真实PG待验target/测试名分开。07：171真实三方集合和机器唯一owner/顺序/分组。08：原TypeScript/Vite、依赖准备及npm入口，构建脚本/产物来源逐层核对。09：原Windows runner实际故障拒绝、正式前置Automated与本批推送后自身验收分开。

R17-01适用出口正式完成。R17阶段不能DONE，R18不能READY。用户已授权开始R17-06；真实PG/XLSX/Full/模型/API等继续按其自身证据逐项验收，不能自动回车、空表或公开stub替代真实闭环。

回退基线ad743482，仅回退本批入口/测试/记录；无业务数据迁移。进程TEMP/TMP沿项目tmp，已单独提醒；必要最终证据留logs，任务临时项按清单归入唯一tmp/old，成功推送后逐文件实际清理并独立核空，不要求用户手删。
