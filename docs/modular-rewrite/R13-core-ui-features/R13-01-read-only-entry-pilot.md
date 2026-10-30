# R13-01 首页与架构只读入口接入试点

## 状态、基线与范围

状态：**DONE（2026-10-31收尾）**。精确代码9cdd1f9b的Windows CI、下载证据包摘要、MSI/NSIS及包内startup报告已核验通过；覆盖原R13-01/R13-10。完整PG/API/Full等原延期不计为本批通过。

本批覆盖原R13-01和R13-10，沿同一记录验收；阶段分支 `rewrite/r13-core-ui-features`，起点 `f0770665babac94ba9f98bebdb7eb9b2367c0f32`。开发前工作区干净。R12按用户确认结束，本批不修改其任务书或索引。

本次只迁首页与已有架构说明的职责和接入位置，不新增视觉方案、运行指标、模型算法或后台命令。已有页面HTML、CSS、字体、图标、文案及交互保持；模型列表直接来自真实Bootstrap DTO，架构说明沿用既有业务说明，不新增一份架构manifest。最近运行继续归Session，原首页未展示的新列表不在本批增造。

## 入口、数据与状态去向

| 原入口/职责 | 实际归属与公开调用 | 数据、命令及副作用 |
| --- | --- | --- |
| `src/pages/dashboard.ts::dashboardPage`、metric/actionCard | `src/features/dashboard/dashboardView.ts`；公开 `src/features/dashboard/index.ts` | main每次render传入当时 `session.bootstrap`，View只拼接HTML |
| 首页健康、统计、估算标识及未知值投影 | `src/features/dashboard/dashboardProjection.ts::projectDashboard` | 从Bootstrap只读推导，不缓存第二份摘要、不查询API、不更改原DTO |
| `src/pages/architecture.ts::architecturePage` | `src/features/architecture/architectureView.ts`；公开 `src/features/architecture/index.ts` | 确定性只读说明与既有锚点，不能写保护清单 |
| main的dashboard/architecture render分支 | main仅组合两个公开Feature入口 | 保留Shell、Workspace捕获/恢复、焦点和主题链 |
| `loadNavigationPage`的只读入口 | 显式无独立加载状态返回 | 不查询其他领域API，不注册全局ApplicationModule |
| 五个首页快捷按钮 `data-page` | 既有根事件委派→activatePage→Router | lineups/prediction/review/analytics/players路由键不变；Feature不自行注册监听器 |
| 连接表单 | 既有connectDatabase→configure_database→Session.refresh→render | 原密码输入/通用失败提示/告警中心保持，Feature未接管数据库写入 |
| Bootstrap/最近运行与生命周期 | 既有AppSession/BootstrapState、Router、Workspace及MountHandle | 原全局实例、取消和销毁不变；无新store、worker或路由框架 |

前批BA-01：受测基线之后至本批仅有任务书变更，已核对；本批更新两个Feature owner及public_entry清单，共享架构门禁通过。BA-16：保持用户最新R12结束结论；最终候选证据仍由R17/R18负责，不从旧状态继承全软件PASS。

## 文件与消费者

- 两个旧TS页面分别移入Feature目录；不保留旧路径转发壳。dashboard旧路径仍有样式目录，CSS职责本批保留，不能因此删除整个目录。
- 新增首页投影、两个公开index；index作为跨模块公开边界，不重复实现View或建立第二个状态。
- main两个import及只读loader显式分支更新；设计浏览器夹具改从公开首页入口导入，扩展工作区守卫读取实际架构View。
- 架构清单更新真实owner，现有边界扫描可识别Feature根和公开index；未增加白名单或放宽保护规则。
- 在现有 `verify-browser-bootstrap.mjs` 中复用TS生产模块加载器补7项行为用例，仅添加SVG raw资源读取以加载真实图标。未新增验证runner、框架、依赖或workflow。

## 验证结果

| 层级/入口 | 实际结果与证据 |
| --- | --- |
| 前端完整门禁 `npm.cmd run verify:frontend` | PASS，96步骤，包括原浏览器检查、TypeScript与Vite生产构建；`O:/Project/FOODBALL/tmp/r13-01-frontend.log` |
| 共享架构 `npm.cmd run verify:architecture` | PASS；`O:/Project/FOODBALL/tmp/r13-01-architecture.log`；273前端源、18Feature契约，原两条prediction过渡边仍按R14归属 |
| 命令/保护资产 | 171命令一致、18保护资产不变；模型aggregate仍 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57` |
| 生产模块行为 | 原24生命周期与18会话测试保持，新增7项Feature行为PASS：未配置/连接失败、缺失与零/估算、刷新/旧响应、快捷路由、模型数据转义、只读架构 |
| 迁移前后HTML | 与f0770665的原View比较，8种首页输入组合和架构说明共9次精确HTML相等；无需更新视觉基线 |
| Windows浏览器真实前端 | 18项检查PASS；真实bootstrap/main、Session、Router、连接操作与View；仅Tauri边界受控响应，非真实PG。证据 `O:/Project/FOODBALL/tmp/r13-01-runtime-JeGsXA/result.json` 与runtime.png |
| Windows桌面构建 | `cargo build --locked -p football-match-model-desktop` PASS，Rust1.88；`O:/Project/FOODBALL/tmp/r13-01-native-build.log` |
| Windows原生实际启动 | 当前开发版及当前前端服务，窗口标题“足球赛事模型平台”；现有analyzer的startup PASS，5条日志/2个完成操作（bootstrap、read_workspace_state）、0错误。`O:/Project/FOODBALL/tmp/r13-01-native-close-59d3d35fbd034f8aa6a436f26a9494e1/startup-report.json` |
| Rust fmt/Clippy/完整workspace tests | 本批未重跑；Rust源、Cargo清单/锁与R12代码基线2f3afc63相同，复用此前记录的同树Rust证据，不算本批新PASS。crates树 `fbfcf4bcdae17aaadd6ac277308db1323c42dfce`，src-tauri/src树 `97162c94b0d218d685ab7fbe393ad01a4209dee9` |
| 当前提交CI、release/MSI/NSIS | 待本提交推送后独立核验，不能用开发版build或历史CI替代 |
| 真实PG/API/Full/私有模型 | 本批未执行，继续前期BA账本/R17-06，未标通过 |

R13-01-V1：生产模块验证未配置/失败的原连接入口与未知统计；浏览器从连接动作走原Session刷新并渲染新统计。连接成功响应为Tauri边界夹具，不是数据库连接实测。

R13-01-V2：浏览器反复首页→管理→架构→首页三轮，五个快捷目标不变，一次快捷操作仅一条实际navigation trace；现有完整门禁继续覆盖焦点/滚动/主题/工作区。View不注册事件，所以反复render不增加监听器。

R13-01-V3：实际架构View确定性只读、原锚点/历史原则与原HTML精确一致，无写保护清单动作；未将“说明文字”冒称运行时依赖图或新Figma验收。

## 失败尝试与本机运行注意

1. 本地辅助编辑脚本首轮有模板字符串语法错误，停在写代码之前；两个页面当时已分别移动，随后修正辅助脚本并继续，生产检查通过。
2. 新测试首轮误期望连接详细错误出现在页面。原组件刻意只显示安全摘要、详细错误交告警中心，已按真实安全边界修正断言；生产错误显示策略没有改变。
3. 浏览器运行夹具首轮传null工作区，引发真实版本保护提示；已改为当前合法工作区DTO并增加无意外issue断言，最终18项通过。首轮不计最终证据。
4. 启动现有run-vite脚本时漏加保留清理产物的本机preload，其依赖预检调用既有recursive rmSync，清理了 `O:/Project/FOODBALL/123 r3/node_modules`。这是一次违反用户禁止批量删除约束的执行失误；没有包装为正常清理。上级 `O:/Project/FOODBALL/node_modules` 公共依赖可用，源码根随后仅见Vite缓存；删除前完整明细未留存，不能证明只删除了缓存。没有Git受控文件因此退出。后续验证直接调用所需工具或加载保留产物设置，不再触发该预检清理。
5. 原生首轮尝试设置APPDATA隔离配置，但Windows原生app_config_dir仍解析到原用户配置目录，故不能声称配置隔离。实际仅运行日志隔离，最终使用真实原生配置只做启动、摘要及工作区读取，并正常关闭；未执行数据库写入或重置。首轮analyzer写报告因沙箱路径权限失败，最终使用授权路径生成上表startup报告。

开发/验证进程TEMP/TMP均按要求指向项目tmp。多数浏览器测试保留临时证据以避免递归删除；没有新建清理脚本。

## 交接、回退与未完成项

只读接入示例：组合层每次传当前Session快照→Feature公开index→本域投影/View→原data-page意图→Router。紧密相关的小函数保留同文件；后续需要查询/编辑状态的Feature再按真实职责建立工作流，不复制本批为固定目录模板。

本批基线f0770665；回退应对本批明确提交审阅后revert，同时恢复两个入口、实际消费者和owner清单，不能只恢复旧页面产生双实现。源码/文档回退不恢复已清理的本机依赖缓存或任何数据库状态。

下一步：核验本提交精确Windows CI、artifact摘要及MSI/NSIS/startup；完成后再按用户指令推进R13-02。原R13-10合并验收不另造DONE节点，整阶段保持IN_PROGRESS，BA账本中的真实环境事项继续带入后续。

## 推送后一次性CI核对

代码提交：9cdd1f9ba2320acd32cfce25e748faa8f79654b2，已推送rewrite/r13-core-ui-features。一次性查询确认 [Windows CI run41085494919](https://github.com/uniquenesssta/123/actions/runs/41085494919) 对应同一SHA和分支，当前in_progress、conclusion=null。未轮询，不能标记CI/安装包通过。此后本记录的纯Markdown提交不改变受测源码；后续核验仍以该精确代码SHA为准。


## 2026-10-31 正式收尾核验

[Windows run41085494919](https://github.com/uniquenesssta/123/actions/runs/41085494919) / job124165444411全步骤completed/success，对应代码 `9cdd1f9ba2320acd32cfce25e748faa8f79654b2`、分支rewrite/r13-core-ui-features。完整日志确认架构、前端/类型/构建、Rust fmt/Clippy/workspace tests、Windows release构建及两种bundle通过；ignored PG保持未验，不能按已执行计数。后续ea054d4f仅Markdown，没有改变受测软件树。

artifact12994712478未过期，52,293,359 bytes；本地下载ZIP SHA-256与GitHub及上传日志三方一致：`0efdba480ff8a16ee9f6a199a9031caf705b581dbeb5d6c49bf9ce0025482b98`。首次gh下载停滞，停止本任务对应下载进程，改用连接器同一产物地址成功，未重跑或轮询CI。

| 产物 | 大小 | 本地SHA-256 / 检查 |
| --- | --- | --- |
| MSI x64 zh-CN | 17,973,248 bytes | `e4e48e3fdc4fd766179ef60c7859ba6bb3d9cdf07ba5b1d0c7bf34f5ba4f9e47`；Compound MSI文件头正确 |
| NSIS x64 setup | 15,609,436 bytes | `ccdfea7508128eb6efd535609e27b33e59dbeb46739ac5cccc355936d532ed76`；MZ文件头正确 |

证据目录 `O:/Project/FOODBALL/tmp/r13-01-ci-41085494919/connector-extracted`。包内 `logs/windows-acceptance-20261030-155833.json`：profile=startup、status=pass，7条记录，bootstrap/read_workspace_state/save_workspace_state三项完成，runtime_errors和missing_required_groups均空。只证明本提交Automated/打包/启动出口，未把其当成Full或额外本机安装操作。

01正式DONE，原10随本批完成；用户已授权开始02，本次先收尾01，不改R12、不重跑同软件门禁；02仍须按自身执行卡实现及验证。上面的VERIFYING/待验段落为当时历史，以本节和顶部当前状态为准。
