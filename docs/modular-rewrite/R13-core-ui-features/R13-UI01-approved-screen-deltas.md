# R13-UI01 六页批准设计补差

状态：**VERIFYING：本地验收通过，等待本批精确CI与安装产物核验**。分支 `rewrite/r13-core-ui-features`，实施基线 `ac586e3d`（07收尾文档），代码继承 `855d830b`。07保持DONE；不重开R12或01～07，不启动09。用户已持续授权本分支提交推送。

## 设计依据与边界

权威为Figma `PN0Whgu6HLWIHx4Mv6aHfu` / Screens `3:5`，既有批准分支 `origin/ui-design-system@40109cee`。本批读取完整设计上下文和截图，按相同页面状态比较；设计中的演示数字不写入生产。未复制原型专用“验收目录”到产品。

| 页面 | 批准主Frame | 本批落地 |
|---|---|---|
| S01 首页 | [453:2](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=453-2) | 首行132px/18px标题、后三卡96px/14px标题、指标104px、组间16px与卡间12px；动作使用蓝/绿/紫语义配色 |
| S06 球队 | [527:54630](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=527-54630) | 固定筛选栏、两列卡片、可见选择框、38px头像、文字完整度和下方档案动作；选定身份独立卡片、五列名单 |
| S07 球员 | [550:69407](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=550-69407) | 248px筛选栏、简化字段分组、独立名单标题/搜索/结果计数、36px表头/64px行、短“档案”动作与完整可访问名称 |
| S10 赛事规则 | [589:112453](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=589-112453) | 刷新/推演/路由优先级均接真实入口；透明搜索工具栏、固定体系/地区、具体赛事两列网格独立滚动 |
| S15 数据库 | [673:178191](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=673-178191) | 服务身份/脱敏目标、四指标及业务边界；概览/连接/统计/危险操作分区，危险动作不再与概览同屏 |
| S17 架构 | [693:194738](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=693-194738) | 四层职责、两项原则、运行能力边界及独立模块分区；明确外部运行时未捆绑 |

附读：球队选定/两类速览/身份档案/960紧凑态，球员速览/档案/紧凑态，数据库连接/统计/危险态，架构原则/模块折叠态。全部属于既有批准设计，不增加新视觉方向。

## 真实功能和职责

- `dashboardProjection.ts`继续从Session派生数量、约数和模型，不写死原型结果。
- `TeamCatalog`、`TeamsFeature`仍唯一持有球队查询/分页/选择/详情。首次目录、刷新、筛选和翻页不隐式打开第一队；明确打开和已有有效tab恢复仍可进入原对象。资料缺失显示“未计算”/“—”，不伪造0%可信度。
- 球队名单去掉重复末列档案按钮；姓名→球员速览→完整档案→返回来源球队仍走原Feature和原生API，未删除档案能力。球队目录和球员目录返回后由既有Workspace恢复原对象焦点，不另建对象/焦点store。
- `playerDirectoryView.ts`保留八列、四类筛选、搜索、真实分页、选择、工作包和档案调用。列表刷新仍使用原DOM刷新路径。
- 地区按钮过多或高度不足时，同一地区筛选自动转原生紧凑选择器，全部选项保留；体系/地区本身不滚动，只有赛事结果滚动。20地区、长名称及最后一项筛选单独验证。
- 窄桌面（≤1180px）两类Inspector沿用覆盖抽屉，不给主名单预留第三列；关闭恢复完整名单。960侧栏开/关与抽屉开/关均验证，不隐藏数据列制造通过。未改变Tauri原生最小窗口尺寸。
- `databaseView.ts`仅组合；身份/指标在`databaseOverview.ts`，连接/统计/危险内容在`databasePanels.ts`，真实写入仍由R13-02 `DatabaseFeature`/`DatabaseWorkflow`处理。切换分区原位更新DOM，未提交密码保留在输入控件，永不进入Workspace持久数据；提交失败释放密码，安全参数保留。
- `architectureContent.ts`承载只读职责/历史/路由/能力说明，未声称公共接口或登记规则等于可执行私有引擎。
- `sectionPanelController.ts`只切换可见分区、标题和导航语义；模块active_section仍归Workspace。两个管理页取消旧文字/底色过渡，避免深色切换中间态低对比。
- 数据库/架构CSS分别进入页级目录，并纳入`contracts/style-boundaries.json`唯一样式图；不追加全局逃逸样式或放宽边界检查。

## 验证记录

| 类别 | 已验证结果/范围 |
|---|---|
| 设计静态组合 | 既有`productionFixture.mjs`/`browserFixture.ts`/`verifyApprovedDesign.mjs`扩到10场景、四尺寸与明暗；80场景/1648项通过。保留原五路由回归并补数据库/架构 |
| 完整程序挂载 | 真实main/Shell/Session/Router/Workspace，只有原生传输使用明确QA夹具。126项通过：四尺寸/明暗、速览/档案/返回、筛选/页码/选择/焦点、密集地区、失败连接、未知统计、危险名称确认、模块展开；两个管理页切换后的实际导航文字对比度均≥4.5 |
| 类型和静态边界 | TypeScript、样式图、模块边界、数据库清空契约通过；清空仍要求数据库名称完全一致，未执行真实重置 |
| Impeccable | 已恢复并复用0.1.17引擎，安装摘要经官方digest核对；六页及相关恢复逻辑源码layout扫描退出0、发现0。检测器不代替截图或真实流程 |
| 独立视觉复审 | 两个隔离评估分别负责布局与机械扫描；六页1440以及960抽屉无布局阻塞。深色导航过渡问题已定位并局部修正，最后局部截图复核通过，Layout无剩余阻塞 |
| Windows原生构建 | `cargo build --locked --manifest-path src-tauri/Cargo.toml`通过 |
| Windows实际运行 | 真实WebView/Tauri IPC，无`__fixtures`/`__calls`；六页和管理分区15项通过，40条日志、0错误、0业务写入。startup分析PASS，不等于Full |
| Rust边界 | `crates`树`e96159c867e4a011f2bf3b18a481f683fac78546`、`src-tauri`树`1c35e0d2be3925d0fea34489f3c54f34f90b8fe7`与07代码855d830b完全一致；完整Rust测试复用同树07证据，本批原生重新构建/运行单独验证 |
| 完整前端/生产构建 | 完整96步退出0（94个既有检查+TypeScript+Vite生产构建）；最后导航颜色/截图稳定性与更严格目录返回断言另做126项受影响复验，最终样式/模块边界及源码扫描再次通过 |
| 自身CI/安装包 | 推送后只核精确代码SHA一次；未取得本批CI、MSI/NSIS及包内startup前保持VERIFYING |

测试断言按新真实组合更新：数据库统计卡3→4；危险按钮检查跟随实际panel owner；管理锚点检查跟随真实分区生成；迟到路由详情测试显式选择球队。原断言的真实数据、约数、强确认、迟到响应和保护语义保留。布局runner改为每轮一个浏览器复用，避免80场景各建profile；异步检查有界等待真实条件，截图前恢复测试页前台并设置超时，不能超时记成功。

## 证据、临时文件与未验边界

开发/验收进程级TEMP/TMP为`O:/Project/FOODBALL/tmp`，通过项目`project-temp.ps1`配置，未改系统全局Temp。保留复用的引擎、依赖和构建缓存；最终证据与无用临时文件按清单分开，无批量/递归删除。最终证据在`O:/Project/FOODBALL/logs/r13-ui01-20261101`；`frontend-complete.log`、`approved-design/results.json`、`mounted/results.json`、`native/read-report.json`、`native/startup-report.json`与`mechanical-final.json`分别记录完整门禁、组合视觉、完整挂载、真实运行、startup和源码扫描。目录搬迁仅改变位置，原报告中的采集路径保留，摘要和搬迁对照见本目录manifest。无用中间目录移入`O:/Project/FOODBALL/tmp/old/r13-ui01-20261101`，用户按`cleanup-manifest.json`手动删除；未执行批量/递归删除。

真实PG专用新库重置、Full链路、有效外部XLSX、真实API凭据、私有固定回归仍沿BA账本/R17-06承接；本批无真实危险写入，不能用QA夹具或startup抵销这些义务。

**下一项：R13-09（工作簿）。** UI01自身CI/产物完成验收后按用户指令开始，不另起R13-08，不提前实施09。


UI01-V1～V5对照：V1由80个实际View组合与六页1440完整截图及独立复审覆盖；V2由四尺寸/两主题、960侧栏开关/抽屉、长名称和20地区覆盖；V3验证球队已应用筛选/选择/原卡片焦点及球员第2页/筛选/多选/对象焦点，并保留原Feature；V4由15项DatabaseWorkflow、挂载失败/未知/未配置/强确认及原生只读运行分别覆盖；V5由真实Session指标、公开运行时契约说明与原型专用按钮排除覆盖。


## 提交与精确CI交接

代码提交`5613bdcde344ba3c9fc99da8e3a9791b30bbcf3f`已推送到`rewrite/r13-core-ui-features`；42路径（10新增、32修改、无文件删除）。07收尾文档提交`ac586e3d`同批推送。199个本地文档链接核验有效，最终证据168文件摘要及代码文件清单保存在上述本地证据目录。

仅查询一次[本批CI run41228822906](https://github.com/uniquenesssta/123/actions/runs/41228822906)：push事件、分支与head_sha精确匹配5613bdcd，查询时`in_progress`、`conclusion=null`。**未通过验收的项目是本批CI/双安装包及包内startup，不能把07或本地startup作为本批CI通过。** 不轮询、不取消其他运行。

后续收尾只核该代码SHA的完成状态、成功job、artifact摘要、MSI/NSIS与包内startup，成功后才记UI01 DONE，再由用户启动R13-09。当前后续文档交接提交仅修改Markdown，不是另一次UI代码CI；代码验收始终锚定5613bdcd。
