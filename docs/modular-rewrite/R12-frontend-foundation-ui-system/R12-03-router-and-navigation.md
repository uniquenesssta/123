# R12-03 Router 与 Navigation

状态：`VERIFYING`。用户2026-10-24确认“02通过 开始03”，基线为02交付`4b5b107aad3415749adbc818bb2c818c38b2da29`；沿唯一`rewrite/r12-frontend-foundation-ui-system`。01～02 DONE，04～19 BLOCKED，R12 IN_PROGRESS。03自身完整Windows Rust/打包/应用启动/日志待原CI结果；遵守用户“不轮询”要求，提交后不持续查询，不自动开始04。

## 实际实现与所有权

- `route.ts`定义路由及导航元数据类型，复用既有公开`Page`联合类型；`routeRegistry.ts`唯一登记7模块/17入口、默认页和中文标签，初始化校验重复页与模块默认页，冻结公开登记数据。无效入口在修改当前路由前拒绝。
- `router.ts`唯一持有当前页面，编排加载开始、成功/错误与结束；`routeTransition.ts`使用02的`AsyncScope`持有当前加载租约。旧成功、旧错误和旧finally均不得覆盖新页面或清除新加载指示；销毁后不能重新开始。中止上一请求同步触发新导航时，外层旧begin也不能覆盖它。
- `navigationRequest.ts`只描述页面/序号/信号/提交有效性，不存业务数据或服务器任务ID。真实Tauri调用仍由原页面加载器负责；AbortSignal使结果失效，不冒充服务端请求已取消。
- `routeHistory.ts`管理右侧详情栈，`-1`代表基础页面；支持边界、返回/前进、关闭后前进、分支截断和跨页面清栈。它不接浏览器URL/history，也不拥有页面业务状态。
- `ModalController`继续拥有详情内容、确认动作和临时表单/滚动/焦点快照，历史游标与数组已移入唯一`RouteHistory`。重绘前调用`modal.capture()`，修复主题/侧栏等Shell重绘丢失当前详情未保存输入的问题。密码、文件、密钥/凭据相关控件和`data-workspace-persist="false"`不进入历史快照；快照只在内存，不写持久化。
- `navigationContext.ts`只拥有工作流返回的页面与分区，供真实补资料/阵容→推演返回链消费一次。比赛ID、快照类型与原因仍留在原业务工作流；球队到球员的既有筛选/来源持久格式不变。
- `src/main.ts`不再另存`page`或`navigationPendingPage`；真实渲染、日志和跳转读取Router。规则操作原直接写页面改走同一导航入口。通用点击无效或已选中页面为空操作，保留当前详情；程序化同页刷新仍可发起新租约。跨页面跳转仍清详情栈并通过原WorkspaceStateStore捕获/恢复页面内容。
- 真实页面加载分支保持原API/DTO：已有players/teams/presets/lineups/prediction/review/analytics/logs/openai保护沿用新租约；补齐release列表→详情的原子提交，以及AI工作台配置/会话列表/会话详情每个await之后的有效性检查。当前错误仍由原调用边界显示与记录。
- `architecture/state-ownership.json`同步Router、RouteHistory与NavigationContext三个实际所有者，状态清单19→20；Workspace和各业务状态所有权保持。

## 文件清单与入口切换

按`git diff --no-renames --name-status 4b5b107`口径，9A/19M/1D，共29条路径；Git相似度可能将旧导航到注册表识别为重命名，仍以实际职责拆分说明为准。

新增9个：

```text
src/app/router/route.ts
src/app/router/routeRegistry.ts
src/app/router/router.ts
src/app/router/routeTransition.ts
src/app/router/navigationRequest.ts
src/app/router/navigationContext.ts
src/app/router/routeHistory.ts
src/app/router/index.ts
docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-03-router-and-navigation.md
```

修改19个：

```text
src/main.ts
src/app/shell.ts
src/app/modal.ts
architecture/state-ownership.json
scripts/verify-dual-level-navigation.mjs
scripts/verify-inline-workspace-and-resource-scroll.mjs
scripts/verify-api-workspace.mjs
scripts/verify-entity-resource-center.mjs
scripts/verify-lineup-preset-navigation.mjs
scripts/verify-openai-profile-ui.mjs
scripts/verify-stage1-balanced-ui.mjs
scripts/verify-team-package.mjs
scripts/verify-team-player-management.mjs
scripts/verify-workspace-ui.mjs
README.md
docs/TESTING.md
docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md
docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-02-生命周期基础设施.md
```

删除1个：`src/app/navigation.ts`，已拆分登记、类型与请求协调职责，所有实际导入切到`app/router`；没有兼容转发壳。该文件按一个明确路径单独移除，没有批量删除。另8个既有验证器只将导航配置来源改为真实注册表，不削弱原业务断言。未新增验证脚本、runner、workflow或依赖。

## 实际验证

本地Windows，Node24.16.0、TypeScript5.8.3、Vite7.3.6、Rust1.97.1；原CI仍为Node22/Rust1.88.0，版本未改。

| 验证 | 实际结果 |
|---|---|
| `node scripts/verify-dual-level-navigation.mjs` | PASS；既有7/17映射与样式契约保留，新增21项真实模块/页面加载行为测试 |
| `node scripts/verify-inline-workspace-and-resource-scroll.mjs` | PASS；实际Windows浏览器执行真实ModalController/RouteHistory/format源码，详情A/B/基础页、输入/checkbox/多选、437/219/311px滚动、焦点、主题重绘、敏感排除、前进分支和清栈通过 |
| `node scripts/verify-browser-bootstrap.mjs` | PASS；01边界与02原24项行为测试保持 |
| `node scripts/verify-frontend.mjs` | PASS；90步（88个既有验证入口+TypeScript+Vite），17视口基线通过，未重录截图 |
| `npm run verify:architecture` | PASS；完整原架构验证链，20个状态owner清单一致 |
| `cargo fmt --all -- --check` | PASS |
| `node scripts/verify-command-contract.mjs` | PASS；171条命令一致 |
| `node scripts/verify_protected_assets.mjs` | PASS；18个保护文件，聚合SHA-256 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57` |
| `git diff --check` | PASS |

完整前端日志：工作区`R12-03-frontend-validation.log`；架构日志：`R12-03-architecture.log`（均位于仓库上一级，非源码交付）。单独浏览器复验产物保留于本机`C:/Users/uniqu/AppData/Local/Temp/football-detail-history-oqT3Ry/`；全量运行另有自身夹具产物路径写入前端日志。测试通过真实源码执行，平台I/O由测试适配器控制，不等于真实数据库端到端验证。

首次受限浏览器连接出现unsettled top-level await，未算通过；同一既有验证在授权的本机浏览器权限下成功。测试装配曾发现Windows绝对路径重复拼接和跨VM promise阶段时序问题，已改为规范化路径及明确请求进入信号，21项通过。最初类型检查遗漏的页面读取已修正。没有忽略失败或降低像素阈值。

本地沿02的仓库外preload禁止清理临时文件，只拦截删除、不改变页面/断言；不提交该preload。新详情夹具自身保留产物，没有新增递归清理。Vite既有超过500kB chunk提示仍存在，本节点未混入业务分包。

## Figma依据、兼容与未完成范围

设计分支本轮核对仍为`40109ceec8372391a7680e7e86c1265fda06ab3d`；读取该版本[FIGMA_PC_APP_SHELL.md](https://github.com/uniquenesssta/123/blob/40109ceec8372391a7680e7e86c1265fda06ab3d/docs/FIGMA_PC_APP_SHELL.md)，Figma文件`PN0Whgu6HLWIHx4Mv6aHfu`，Shell根`370:2`/组件`380:1366`。采用其7模块/17页、模块默认页、已选中入口空操作、跨页清详情栈、主题保持详情以及过期响应丢弃语义；Back/Forward明确是右侧详情栈。

本节点未改CSS、页面视觉、图标、共享变量或Figma节点，未将原型PASS当应用端验收；没有宣称Shell像素接入、17路由全部真实输入、键盘焦点陷阱、重置确认或错误Blocking Message完整实现，这些继续按05/14/17/19职责实施。浏览器夹具的focus恢复不是完整应用无障碍验收。

无公开Page值、171命令、DTO/Schema、数据库格式、配置键、日志等级、依赖/lockfile、Rust/SQL或模型参数变化；UI行为仅落实上述导航及详情恢复合同。真实PG、历史保护四项、不可变账本并发回滚、XLSX、Windows Full、私有P4/P7固定回归及既有最终环境风险仍待实际验收。公开stub、静态指纹及模拟I/O测试不能冲销这些待验项。

Context7查询标准AbortSignal/Promise取消语义，确认调用方必须主动观察取消与丢弃结果；没有升级依赖。Mermaid Chart更新实际路由→切换租约→页面加载→有效提交，以及独立详情栈/返回位置链。按AGENTS协议保存Create State；Git与本记录仍为交付权威。

## 回退与后续

按03原子提交受控回退到基线4b5b107；没有数据库或持久格式迁移。本节点代码、本地验证及记录已完成，发布后由既有Windows流水线验收03精确提交。遵守用户要求不轮询，不创建自动监控；用户返回结果后再收口03。04未实施，R12阶段完成记录未创建。
