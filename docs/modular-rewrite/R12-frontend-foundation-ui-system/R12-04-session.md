# R12-04 Session

状态：`VERIFYING`。用户2026-10-24要求“收尾03开始04”；03精确`7f67940`/Windows run`40103980479`完整通过，日志及下载产物哈希已核对。04起点为纯文档收尾`1cfd7cea36265b7170b48acdd1b9226b9e5680e0`，同03已验源码树。沿唯一`rewrite/r12-frontend-foundation-ui-system`；01～03 DONE，05～19 BLOCKED，R12 IN_PROGRESS。04代码、本地验证及记录完成，自身完整Windows Rust/打包/启动/日志待原CI；不轮询CI，不自动实施05。

## 实际实现与边界

原`src/main.ts`同时保存启动摘要、主题与独立bootstrap请求域，并在多个动作中直接改写摘要/最近运行列表。本节点迁入唯一会话职责：

- `appSession.ts`编排初始主题、启动摘要读取、最近运行刷新、停止/销毁；I/O通过注入的三个adapter调用，不导入业务页面、Router、Workspace或API client。复用02的MountHandle与AsyncScope，不建立第二套启动/销毁promise状态机。
- `appSessionState.ts`只拥有会话阶段及本次启动错误，MountHandle仍拥有生命周期尝试。重复start/destroy返回同一promise；初始摘要未完成时并发refresh复用该初始读取，避免摘要尚空就报告ready。失败保留原错误，停止后不能重启或回写ready。
- `bootstrapState.ts`唯一持有启动摘要。接收真实BootstrapResponse后先structuredClone，再用WeakSet递归冻结；与adapter返回值隔离，页面沿既有DTO签名借用运行时只读快照，未把公开DTO类型升级宣称成完整TypeScript深只读。最近运行通过具名方法生成新摘要，入口不直接改字段。
- 全摘要刷新与最近运行刷新各有自己的请求域。新全摘要读取使此前最近运行请求失效；若较晚的最近运行读取先提交，全摘要随后提交时保留那份较新的最近列表，同时更新其余摘要字段。过期成功/错误不覆盖当前数据；当前失败保留原摘要并向原调用边界抛出，可再次刷新。
- `themeState.ts`只持有Light/Dark值。`sessionPersistence.ts`只读写原始字符串偏好，仍用`football-model-platform.theme`键；缺失/无效值默认light，不重写其他偏好。不保存启动摘要、路由、表单、API凭据或业务状态。
- localStorage属性读取/访问策略异常或写入配额失败时，主题维持本次会话的内存选择，不阻断原重绘；保存失败不冒充已跨启动保存。主题投影adapter失败则保持原值和原保存偏好。主入口原业务偏好读取失败后的清理也改为可失败清理，避免因removeItem再次抛错；业务偏好键/格式/所有权未迁移。
- `src/main.ts`只实例化一个AppSession，注入原api.bootstrap、api.listRecentRuns(100)及document主题投影。原全局state、currentTheme、readInitialTheme/applyTheme和bootstrapRequests移除；实际页面、Shell、日志和业务读取使用session.bootstrap/theme。原9处全摘要读取入口及5处最近列表写入统一通过会话，业务加载和服务端取消协议保持。
- 真实工作区初始化后启动session，再按原行为渲染；停止即时关闭会话请求，销毁时在工作区最终保存的finally中销毁session。主题切换仍`render({ preserveForm: true })`，沿03详情捕获/恢复，不清空页面草稿、详情历史或工作区滚动。
- 架构清单新增会话阶段、启动摘要、主题三个实际owner，20→23。原main中的球队/球员偏好、路由、工作区和各Feature状态继续由已有owner管理；没有建立通用全局store。

## 实际文件清单

相对基线`1cfd7ce`，`git diff --no-renames --name-status`为7A/10M/0移动/0D，共17路径；没有删除文件或目录。

新增：

```text
src/app/session/appSession.ts
src/app/session/appSessionState.ts
src/app/session/bootstrapState.ts
src/app/session/themeState.ts
src/app/session/sessionPersistence.ts
src/app/session/index.ts
docs/modular-rewrite/R12-frontend-foundation-ui-system/R12-04-session.md
```

修改：

```text
src/main.ts
architecture/state-ownership.json
scripts/verify-browser-bootstrap.mjs
scripts/verify-dual-level-navigation.mjs
scripts/verify-extended-workspace-hierarchy.mjs
scripts/verify-inline-workspace-and-resource-scroll.mjs
README.md
docs/TESTING.md
docs/football-model-platform-modular-rewrite-19-docs/12-R12-frontend-foundation-ui-system.md
docs/modular-rewrite/R12-frontend-foundation-ui-system/README.md
```

旧职责以原位删去重复变量/函数/直接写入的方式收敛，没有转发空壳。目录index只具名导出。没有新增脚本、runner、workflow、测试框架或依赖。03收尾的5份文档变更在先前独立[skip ci]提交，不混称为04源码变化。

## 实际验证

本地Windows，Node24.16.0、TypeScript5.8.3、Vite7.3.6、Rust1.97.1；CI原Node22/Rust1.88.0不变。

| 验证 | 结果与覆盖 |
|---|---|
| `node scripts/verify-browser-bootstrap.mjs` | PASS；原24项生命周期保持，新增18项实际AppSession/SessionPersistence行为，含旧键恢复、无效主题、getter/read/write异常、启动销毁去重/竞态/错误、初始刷新合并、摘要隔离冻结、全摘要与最近列表交错、停止后的响应、当前/旧错误与投影失败 |
| `node scripts/verify-dual-level-navigation.mjs` | PASS；03原21项行为保持，真实main函数夹具的摘要读取接到session.bootstrap |
| `node scripts/verify-inline-workspace-and-resource-scroll.mjs` | PASS；原Windows浏览器夹具装配真实会话/持久/生命周期模块，检查初始dark投影、切为light及原键保存，同时保留详情输入/checkbox/多选/437/219/311px滚动/焦点/主题重绘/敏感排除/历史边界 |
| `node scripts/verify-frontend.mjs` | PASS；最终90步（88既有入口+TypeScript+Vite），17视口基线通过，无重录 |
| `npm run verify:architecture` | PASS；原完整验证链、23个状态owner清单一致 |
| `cargo fmt --all -- --check` | PASS |
| `node scripts/verify-command-contract.mjs` | PASS；171命令一致 |
| `node scripts/verify_protected_assets.mjs` | PASS；18文件聚合SHA-256 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57` |
| `git diff --check` | PASS |

最终完整前端日志为仓库上一级`R12-04-frontend-final-tree.log`，架构日志`R12-04-architecture.log`。实际浏览器最终夹具产物保留于`C:/Users/uniqu/AppData/Local/Temp/football-detail-history-b52iAI/`，单独复验另有`football-detail-history-NKhwRS/`。模拟adapter用于准确控制请求交错与存储错误，不是实际PG/API端到端验收。

首次全量日志`R12-04-frontend-validation.log`在既有扩展工作区检查失败：检查仍期待`rulesPage(state, ...)`，实际已迁为`rulesPage(session.bootstrap, ...)`。已同步到真实owner并保留工作区active_section断言；最小复验及最终完整运行通过，没有弱化功能断言或更新像素阈值。测试先补断言时因目标模块尚不存在的预期红灯不计通过。一次提升权限的shell中独立git diff未定位到仓库，随后在明确仓库目录复验PASS；不将该失败命令算验证通过。

本地沿02/03的仓库外preload保留测试临时产物，只拦截文件清理、不改变页面代码/断言，不提交它；没有批量删除。既有Vite大于500kB chunk提示保持。未在本地重复Rust Clippy/workspace tests或完整打包，04精确提交由既有Windows流水线执行；03的成功只作为前置证据。

## 设计、兼容与尚未覆盖

按R12共同输入，04会话没有独立视觉画板；沿已核设计分支`40109ceec8372391a7680e7e86c1265fda06ab3d`及Shell功能合同的主题保持语义。没有改Figma节点、CSS、页面视觉、图标、路由值或截图基线；浏览器夹具验证真实会话主题投影与详情恢复，不代表05 Shell像素接入或完整无障碍验收。

无新增依赖、lockfile、Rust/SQL、公开DTO/Schema/171命令、配置键、数据库格式、日志等级或模型保护资产变化。内部启动摘要改为不可变快照及原主题写入异常回退是本节点实际行为改进；bootstrap connection_error等原字段与原用户错误呈现语义保留。工作区持久化及会话Storage中的数据库重置提示标记仍归已有模块；本次只保证主题偏好存储异常回退，没有宣称所有存储故障或所有业务异步已处理。

真实PG、历史四项/不可变账本并发回滚、XLSX、Windows Full、私有P4/P7固定回归及继承最终环境待验继续保留，公开stub/静态/模拟I/O测试不算这些PASS。

已用Context7核对localStorage getter/SecurityError与setItem配额异常，以及structuredClone独立深拷贝、Object.freeze浅冻结与WeakSet循环保护；查询基于真实TS5.8.3/Tauri API2.11.1/CLI2.11.4且未改版本。Mermaid Chart展示真实会话→摘要/主题→adapter/有效提交→停止清理链；按AGENTS协议保存Create State，Git与本记录为交付依据。

## 回退与后续

受控回退04原子提交到`1cfd7ce`，无数据库或持久格式迁移。04保持VERIFYING，发布后等待自身原Windows验收；不轮询、不新增自动监控。05 Application Shell未实施，仍须自己的设计/代码/Windows证据；未创建R12阶段完成记录。
