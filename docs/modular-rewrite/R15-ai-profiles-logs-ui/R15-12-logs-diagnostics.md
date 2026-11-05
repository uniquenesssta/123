# R15-12 问题日志与诊断关联

状态：DONE（自身CI、官方双包与startup已验收；真实外部边界仍保留）。用户2026-11-05授权“收尾07开始12”；基线42418cb555de577f50e1b25314e9b66183de4adc，起始工作区干净，继续rewrite/r15-ai-profiles-logs-ui。07自身CI/官方双包/startup已正式收尾。本记录覆盖原12 Logs与13 Diagnostics Bridge；本批自身CI及官方包另验，不轮询。下一项R16-01未启动。

## 复用、迁移与唯一归属

| 原职责 | 当前唯一owner/出口 | 保留与修复 |
| --- | --- | --- |
| main.issueLogs/loadIssueLogs与旧logs页面 | features/logs/LogsFeature、logView、index | 本机读取不依赖业务数据库；读取序列、页面/生命周期校验；空态与失败分离，保留旧快照但禁写 |
| 日志详情/分区/焦点 | LogActions、logPresentation | 原source/severity/时间/次数/操作保留，显示原occurrence_keys；三个真实分区与焦点回入口 |
| 原choose/export/clear事件 | LogOperations | 原系统保存位置选择、取消无写入；快照预读及后端锁内比较；清空后真实读回，不伪造空数组 |
| 原recordClientIssue/userFacingError | platform/diagnostics公开出口与clientIssues | 共享脱敏、原友好提示排除；注入wasLogged复用原transport身份，不重复记账、无循环依赖 |
| transport失败/trace | 原InvokeTrace/InvokeRequest | 原request.id传给occurrence_key；诊断自身失败不递归写问题日志，运行错误仍保留 |
| 问题记录、磁盘与报告 | 原IssueLogStore，分拆aggregation/storage/report | 缺文件才为空；坏文件/权限失败明确报错并保留原文件；先clone、原子落盘后才提交内存 |
| 技术脱敏 | 前端diagnostics/logSummary；Rust diagnostic_redaction | UI/WarningCenter共用原前端出口；Rust运行日志/问题库/导出共用原生边界，无第二套日志系统 |

旧src/pages/logs.ts逐项退出，main仅组合/转发/路由；公共控件、WarningCenter、AsyncScope、workspace/storage、原命令及服务直接复用。新恢复偏好仅白名单分区、展开ID和unknownClear/operationId；不保存正文、路径或敏感值。恢复坏记录保留未知门禁，不悄悄抹除证据。

## 实际修复与契约

- 读损坏JSON原来静默变空；清空写失败原来已抹掉内存。真实文件回归先复现，再修复。失败不回写坏文件；修复外部文件后可重新读取。
- clear/export继续原171命令体系，原返回、错误及output_path保持；仅增加可选expected_entries，旧调用省略时走原clear/export。新UI绑定完整原DTO快照，后端在持锁期间比较并拒绝变化集合，杜绝确认后新记录被误清。没有新命令、Schema或数据迁移。
- 清空前先写必要恢复记录，失败不发出命令；未知回执先刷新核验，不自动重试。用户明确检查后才能对新集合发起新确认；读回不证明上次清空成功。后台扫描仍可产生新日志，清空不修复故障、不清任务/会话/草稿。
- Windows实测发现issue_recorded被通用去重吞掉不同attempt的trace；只对该事件把trace加入去重指纹，同attempt仍抑制、不同attempt即时保留。通用诊断降噪与生命周期trace配对原语义保留；新增Rust真实读回回归。
- 已知敏感形状统一处理Basic/Bearer（含制表符）、Authorization/Cookie、带空格/引号的密钥、URL凭据/查询/片段、用户路径、正文与sk-类令牌。数量字段保留；重复脱敏幂等。任意自然语言中的未知秘密不能据此宣称全部可识别，真实凭据BA-11继续R17-06。

依赖清单/锁文件、workflow、模型资产、46项历史迁移与171条命令名称顺序保持。类型清单由原生成器刷新；TS occurrence_keys补齐既有Rust序列化字段，公共类型数366保持，不新造wire类型。

## 已批准设计接入

使用Impeccable4.7.1 Operate/harden及craft-floor；项目0.1.17引擎实际运行context与最终detect，最终报告为空数组。无PRODUCT.md/DESIGN.md时沿已批准S16，不编造设计或更换视觉方向。

Figma文件PN0Whgu6HLWIHx4Mv6aHfu，批准ui-design-system/docs/FIGMA_SCREEN_LOGS_STATE.json，recordCommit14fba2a424e8fc4c505eb9d13997fffa2732cdb5。本轮取得主683:186758、暗色684:187051、紧凑684:187581、详情684:189313、聚合684:188667、失败684:192164、空态684:192750、加载684:191545、确认686:194165、报告快照686:194414的高保真上下文与截图。30状态/12弹窗是预置设计，不是真实功能验收。

26/700标题、14/700分区、13/500控件、12/400正文、Inter28/600数据，32px动作/12px主间距/8px卡间距。工具栏位于唯一正文滚动区外；聚合机制按批准的两块纵向说明面板，卡片内部长技术文本独立限高滚动。危险/警告/信息使用对应语义表面；共享本地字体、图标、焦点与明暗，不引入临时资产URL。原设计中的review-hub和演示状态选择器不是产品操作，未作为真实功能复制。

完整挂载发现全局module-workspace-page Grid及StatCard字重覆盖，引起巨大空位与数据字重不符；修复日志作用域特异性并加入实际computedStyle/几何断言。1440×900、960×720、1280×720、1920×1080明暗，以及500条/长标题/4000字技术文本实际验证。

## 原ID与验收证据

| 原ID/场景 | 真实执行路径 | 层级与证据 |
| --- | --- | --- |
| 12目录、空/失败、详情、刷新 | 实际LogsFeature与独立外部port | 既有diagnostics入口的23项生产行为；层级16项；原导航21项 |
| 12动作与未知恢复 | 实际LogOperations、原确认与控件 | 完整挂载30项，含取消、迟到、集合变化、未知不重发、偏好无正文；仅外部transport可控 |
| 12持久化/导出/清空 | 原Rust存储、原171注册表、实际WebView | Windows14项：真实系统保存窗口/实际导出文件内容、同attempt去重/新attempt次数、清空取消/确认/落盘/锁内快照拒绝 |
| 13诊断关联与防刷屏 | 原transport、IssueLogStore、RuntimeLogStore | UI catch不重复上报、诊断自身不递归、原trace即时保留、前后端已知秘密脱敏；Rust真实文件回归与原诊断测试 |

证据根O:/Project/FOODBALL/logs/r15-12-20261105。完整适用结果、命令退出码、工具版本与受测源码哈希以validation.json、source-manifest.json为准；frontend-full.log按原96步骤包含类型/构建，Rust格式/严格Clippy/工作区测试、Windows构建/运行和startup分别记录，不能互相代替。

Windows GUI验收复用现有lib test target，只有显式FOOTBALL_NATIVE_LOG_ACCEPTANCE_BUILD=1及--lib构建才启用cfg；生产不出现另一入口。全部可写存储限定O:/Project/FOODBALL/tmp/r15-12-native-config，真实命令与dialog插件，不接业务库、不碰用户历史。Common Controls v6仅加在该显式测试构建，普通应用沿原Tauri清单；无新runner/框架/workflow/依赖。Cargo构建指令依据[官方说明](https://doc.rust-lang.org/cargo/reference/build-scripts.html)。native/report.json、binary-hash.json、原生截图、真实导出TXT与startup均保留。

失败证据不抹除：baseline-rust-failures保留原损坏/写失败/泄漏；browser-first/fixed保留确认宿主与详情测试前置问题；browser-verified/geometry保留真实布局覆盖；native-trace-initial保留不同trace丢失；rust-manifest-initial保留首次验收链接设置影响普通构建的失败。后者已限定显式QA配置，并重新执行普通全量门禁。frontend-guard-initial保留旧静态锚点，守卫改验真实新owner及可选兼容参数，原171命令/唯一注册/入口约束继续执行，没有关闭检查。

## 待验、回退与清理

本批自身CI/官方MSI和NSIS/startup在推送后按精确提交另验，不复用07，不轮询。BA-10/11真实API、计费测试、用户Windows凭据写入/替换/删除及故障、真实PG/会话/取消仍按原计划R17-06关闭；不称Full或真实外部服务全部通过。

回退基线42418cb5仅撤回代码，本地日志格式未迁移；已执行的用户清空无法靠代码回退恢复，因此本次原生验收仅动隔离QA日志。真实用户日志、凭据与普通工作区不清理。必要验证证据留logs，稳定依赖/.cargo-target不删。

进程TEMP/TMP沿project-temp.ps1指向项目tmp，已单独提醒。成功推送后按清单移动本次无用临时产物到唯一O:/Project/FOODBALL/tmp/old，再按长期授权逐文件实际删除、空目录逐项移除、不跟随链接、保留old根并独立复核0；以archive-result.json及old-cleanup-12-push.json实际结果为准，计划不算执行。下一项R16-01，未开始。

最终适用本地门禁：完整96步前端（含类型/构建）、23项生产流程、16项层级、21项原导航、30项完整挂载；Rust格式/严格Clippy、工作区768项测试通过，40项既有外部环境测试未执行；正常Windows构建、隔离原生14项及原startup分析通过。正常EXE清单与原Tauri默认Common Controls v6一致，没有新增requestedExecutionLevel。Impeccable最终detect为0项；现有主bundle大于500kB提示保留，未放宽阈值。

frontend-hygiene-initial保留第二轮完整检查发现的测试模块声明顺序问题，已按原规范把reliability_tests声明移到末尾内联tests之前；守卫未改，最终门禁重新通过。测试声明顺序调整与最后Basic脱敏补齐后，受影响的diagnostics、WarningCenter、日志挂载和Windows原生14项均再次通过；普通构建已恢复默认设置。

## 2026-11-05正式收尾

用户授权“收尾12开始R16-01”后，一次核对精确ed1e892d18bea412f97dc0541cded9edacad3cee、push、rewrite/r15-ai-profiles-logs-ui及ci.yml：[运行42006118669](https://github.com/uniquenesssta/123/actions/runs/42006118669)，job127173177599，全部completed/success。此前VERIFYING与“未开始R16”段落保留历史，不作当前状态。

官方artifact13406729451：ZIP52,489,594字节，SHA256为6b34821e08edb9d60cd4feec036b7c5e1b58edcb15ed1eb9dd5dff618fa04a17，与GitHub digest一致。MSI18079744字节（da24676e4a4fe2fe578f7c2f48558b7b37f931251971c3540c2407c4115e83e8），NSIS15709139字节（bc8e9377a88b0e736aa06fd94398de175627eb73ab690e531c0aef2dfda8218a）；名称、非零长度、OLE/MZ头与摘要均核对。官方runtime按原分析器startup/pass：7条、0非法行、0运行错误。完整官方ZIP、两包、原runtime及acceptance.json保存在O:/Project/FOODBALL/logs/r15-12-ci-42006118669。

没有在本机安装这两个包或验证数字签名，startup不等于Full。BA-10/11仍按原R17-06关闭。R15三个批次适用出口已满足，R16-01已获本轮启动授权；R16-09未启动。
