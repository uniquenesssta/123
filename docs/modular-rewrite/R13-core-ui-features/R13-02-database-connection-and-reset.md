# R13-02 数据库连接与危险操作

状态：**DONE（2026-10-31收尾）**。用户已授权R13-02，并补充要求先收尾01；01已核对精确9cdd1f9b的Windows CI、产物和startup，于beecbf22正式收尾后才实施02。本批分支 `rewrite/r13-core-ui-features`，起点 `beecbf22`，实施前工作区干净。03及后续未启动。

## 职责与真实调用链

| 原职责/入口 | 当前唯一owner与调用方 | 保留边界 |
| --- | --- | --- |
| `src/pages/database.ts::databasePage` | `src/features/database/databaseView.ts`，公开index导出；main和Data Display浏览器夹具改用公开入口 | 继续从Session当前Bootstrap读取健康、统计、迁移与配置摘要，不新增健康缓存 |
| main的connectDatabase及DOM读取 | `connectionDraft.ts`读取一次提交快照，`DatabaseFeature.connect`负责现有UI接入 | URL仅暂存于输入/必要请求；非敏感字段仍由原Workspace保留；不新增持久化草稿 |
| 全局连接变更、迟到响应、重复提交 | `databaseWorkflow.ts::DatabaseWorkflow` | 配置/断开按实际后端唯一活动连接顺序执行，旧请求不发布新摘要；同一待办连接参数去重，Bootstrap仍只由Session发布 |
| main的requestDatabaseReset/executeDatabaseReset | Feature控制器接原Modal确认；Workflow拥有确认票据、当前revision/快照、提交锁和失败复核标记 | 仍需输入数据库名，并在真实API调用前独立验证；不信任DOM dataset提供的expected |
| 成功重置后的应用恢复 | main组合层的completeReset端口 | 实际reset成功后才清Workspace、停止退出回写、逐项清原三个目录偏好、置完成标记并重载；保留配置和后端自动重连/worker逻辑 |
| 退出/销毁 | 原应用stopResources调用 `databaseFeature.stop()` | 不执行排队新连接、不发布迟到结果；不声称取消已经开始的后端事务 |

实际链：页面/内联连接表单→Feature→串行工作流→main注入的原api.configureDatabase/disconnectDatabase/resetDatabase→原171命令及Application/PG owner；成功读取原Session.refresh→render。串行和确认属于前端编排，没有改变wire参数、后端事务或Schema。

架构清单迁移database owner/public_entry；新增 `browser.database-workflow` 唯一owner登记，状态清单38→39。此状态仅含临时任务/确认/生命周期，不能成为第二个连接配置或Bootstrap存储。旧页面路径单独移动，未保留旧路径壳；样式入口和视觉组件保持。

## 修复的已确认风险

- 连接调用会在后端激活连接，不能只丢弃前端A响应而允许A最后激活。现在串行执行实际变更，B在A落定后执行；未开始的过期请求跳过。相同待办参数不重复提交。
- 重置票据绑定提交时的同一Session快照、工作流revision和不可变数据库名。连接/断开请求立即使旧票据失效；摘要替换也要求重新确认，即使新旧数据库同名也不沿用旧许可。
- 配置成功而摘要刷新失败时，旧UI不能当作当前身份。新增未核实状态，此时禁止清空；成功显式重连并读回摘要后才解除。
- 重置提交到本地清理完成期间，禁止切连接/断开/重复重置。后端失败或结果未知时不清Workspace、不设置成功标记、不重载，也不自动重试；必须核查实际结果并显式重新连接后再次确认。
- 请求结束后释放临时URL；失败清除原敏感输入，保留连接数/超时等非敏感字段。日志继续走唯一Platform脱敏；技术连接URL增加同一脱敏函数。未知健康/统计使用“—”，不把缺失结果补成真实0；实际0及估算前缀保持。

确认票据只在原确认回调使用；取消确认由原组件销毁回调，零reset调用。使用现有typed、pending、failed、requires-review显示，不新增UI布局或重做Figma。近期运行/主题/工作区/路由及其他业务仍归原owner。

## 验证与证据

| 层级 | 结果 |
| --- | --- |
| 完整前端门禁 | 96步PASS（含类型与生产构建）；日志 `O:/Project/FOODBALL/tmp/r13-02-frontend-final.log` |
| 生产工作流行为 | 15项PASS，接入原verify-database-reset入口；覆盖串行/去重、旧失败、断开顺序、旧确认/新快照/错误名称、提交锁、失败核查、销毁、草稿/脱敏/未知值及刷新失败禁止清空 |
| Windows浏览器生产运行 | 21项PASS，真实bootstrap/main、Router、Session、Feature、表单与Modal；仅Tauri边界受控响应。`O:/Project/FOODBALL/tmp/r13-02-browser-akMsxt/result.json`及database.png |
| 架构/状态/契约/模型保护 | 完整verify:architecture、39个状态owner、277前端源、171命令及18保护资产通过；`O:/Project/FOODBALL/tmp/r13-02-architecture.log`；TypeScript最终单独复核通过 |
| Windows桌面构建 | cargo build --locked -p football-match-model-desktop通过；Rust1.88；`O:/Project/FOODBALL/tmp/r13-02-native-build.log` |
| Windows原生startup | 当前开发版/当前源码服务：startup PASS、5记录、bootstrap/read_workspace_state完成、0错误；`O:/Project/FOODBALL/tmp/r13-02-native-LJz4ly/startup-report.json`；最终源码复验通过 |
| Rust完整单测/Clippy/格式 | 本批Rust树、Cargo及依赖不变，复用01精确CI run41085494919的同树证据；本批不是新跑这些项目，不将开发版build替代workspace tests |
| 本提交CI/MSI/NSIS | 待推送后单独核验精确提交，不轮询；不以01产物代替02 |
| 真实配置/重置/PG/Full | 本批未对任何真实库执行配置或重置。专用新测试库重置及失败/未知提交读回仍按BA-14、R17-06，不能把Tauri夹具视为实际数据库效果 |

R13-02-V1：生产测试证明A阻塞时B不并发激活，A完成后只发布B；浏览器真实按钮同样只顺序发出slow_test/fast_test。旧失败不会归到新连接。

R13-02-V2：测试绑定同名连接换快照后的票据失效、错误名称拒绝、待连接拒绝重置；浏览器取消确认reset调用为0。

R13-02-V3：测试/浏览器重置响应丢失时不清本地引用，实际reset仅一次；错误确认界面禁止直接重试，关闭后仍需显式重连核实。真正后端事务结果必须由后续授权新库证明。

## 失败尝试与约束

第一轮新View测试使用错误StatCard类名，按真实ui-stat-value纠正；第一轮浏览器暴露全局任务提示会拒绝第二次请求，已把配置/断开的任务提示移至队列中实际执行位置，未放宽原全局任务互斥；后续21项通过。原Forms/Overlays/数据库reset源码守卫仍读main旧函数，已迁到实际Feature并保留名称确认及后端命令断言；完整门禁未以删除断言放行。

原生启动端口1420已有Vite服务，未停止该进程；读取该服务提供的main确认包含当前Feature调用/stop后复用。只隔离runtime日志，配置仍是Windows原生配置路径，仅执行启动/摘要/工作区读取，不操作真实库。开发脚本进程TEMP/TMP指向项目tmp，使用保留产物preload避免递归清理；未执行批量删除、未改依赖预检脚本。

## 交接与回退

原BA-01/14按前期账本承接：01前置已核验，当前共享结构门禁通过；真实PG/Full仍非PASS。后续02只有在自身精确CI与产物验收后才能正式DONE；03待用户后续指令。

代码回退应针对本批提交审阅后revert，成组恢复入口/owner/原守卫消费者；不同时保留旧页面与Feature。回退代码不恢复数据库，真实重置不可用Git撤销。外部进程/其他客户端修改数据库连接不由本前端队列提供跨进程原子保证，后端原预检/事务边界仍为权威；若未来要求跨进程强绑定，需单列后端契约设计，不能本批虚报已实现。

## 推送后一次性CI核对

代码提交d167e948cb39bbaff0c95c88b5a6c23d8c0f4530已推送既定R13分支；[Windows run41098940648](https://github.com/uniquenesssta/123/actions/runs/41098940648) head_sha/head_branch与本批一致。一次查询时in_progress、conclusion=null，未轮询，不标CI或安装包通过。其后纯文档提交不改变软件受测树，后续核验以d167e948为准。


## 2026-10-31 正式收尾

run41098940648和job124211928625均completed/success，精确head_sha为d167e948cb39bbaff0c95c88b5a6c23d8c0f4530，分支rewrite/r13-core-ui-features；run更新时间2026-10-30T18:15:15Z（北京时间2026-10-31 02:15:15）。作业步骤列表仍返回旧in_progress/pending快照，因此不把该列表冒称全步骤success；完整日志已逐项确认前端、15行为、Rust fmt/Clippy/tests、release双bundle及startup成功，且产物/报告与run总状态相互印证。未轮询。

artifact13001455633，52,288,909 bytes，未过期；下载ZIP摘要与GitHub及上传日志一致：`70a21b78c54ef11a854184fff4628d70f3d1e1554674a327f5ddf375a31f4085`。证据目录O:/Project/FOODBALL/tmp/r13-02-ci-41098940648/extracted。

| 产物 | 大小 | SHA-256与格式 |
| --- | --- | --- |
| MSI x64 zh-CN | 17,977,344 bytes | `081916a7d078f784ee1a74205c008330d6f4f2bae66b5fd4d854360c6ff3ea44`；Compound MSI头正确 |
| NSIS x64 setup | 15,608,796 bytes | `319e1cd4974afb76be966577227db8c776b835353b617b4d0b0ffde2a21e13b0`；MZ头正确 |

包内logs/windows-acceptance-20261030-172830.json：profile=startup、status=pass，7条记录，bootstrap/read_workspace_state/save_workspace_state三项完成；无runtime_errors或missing_required_groups。不把打包/启动结果扩大为额外本机安装或真实新库重置/Full通过。02正式DONE；03（合并原04）已获用户授权，按执行卡推进。此前VERIFYING/待CI文字为历史，以本节和顶部为准。
