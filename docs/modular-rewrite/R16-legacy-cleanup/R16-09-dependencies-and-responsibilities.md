# R16-09 死代码、依赖与职责复核

状态：DONE。2026-11-06用户授权“收尾01 开始R16-09”。分支rewrite/r16-legacy-cleanup；基线3baf4ed2b78c09dedb8b29e1a98c2347f4dbb395，开始时工作区干净。覆盖原R16-09/10。前置01的精确CI42041966141/job127294792894、官方artifact13425857136、摘要、MSI/NSIS与包内startup已核验，正式DONE；本批自身CI/包在推送后另验，不轮询。

## 实际发现与修复

1. **真实值依赖环。** 原prediction/index→pages/prediction→runs/index→runWorkspaceView→prediction/index形成运行时环。推演页现在接收主组合入口传入的原runHistoryMarkup输出，只保留Runs视图类型引用；列表实现仍唯一位于runHistoryView，详情继续消费Prediction原公开结果能力。没有新增公共出口例外、改CSS、复制渲染器或迁移业务状态。main按同一Runs view快照拼装行和pending/locked；Session仍是唯一列表缓存。
2. **原门禁不检查前端值依赖环。** 在原verifyModuleBoundaries入口组合frontendRuntimeGraph，使用项目已安装TypeScript 5.8.3及当前tsconfig推导擦除类型后的值图，复用原图遍历。静态/require循环失败；字面量动态加载单列；无法解析的动态加载明确失败。6组含类型的源码环与真实运行时环分开，不能把全部类型引用机械迁出。14项隔离样例覆盖类型擦除、重导出、side-effect、自环、动态/未知加载、require、注释/字符串及语法错误。
3. **Cargo合法表式依赖漏检。** 隔离的[dependencies.football-application]曾在原门禁错误通过。原cargo解析owner现统一识别行内、独立依赖表、带引号/别名、目标平台、dotted属性和workspace继承；package.metadata.dependencies不误当真实依赖。模块与受保护导入继续共用此owner，不增加白名单。16项原入口样例和9项真实门禁负向验证通过；本机cargo metadata --no-deps --offline --locked独立核对11包/97项声明一致，Rust workspace无环。它是当前清单的交叉核对，不宣称自制解析器实现完整TOML语言；无法理解的相关声明明确报错。

## 消费者、动态边和保留决定

最终AST图覆盖455个前端文件、1454条含类型/资源的导入边；与既有import扫描器无集合差异。擦除类型后的图1005条值/资源边，静态运行时环0；唯一字面量动态import是registerApplicationModules→main，非字面量加载0。SVG ?raw、CSS、import.meta.url资源、HTML入口、DOM action分发、原生注册和注入ports另行核对，不把它们误称为“零依赖”。

17个Page union、实际RouteRegistry与main.render分支完全对应。19个Feature、191个CSS文件/原唯一入口、67项状态owner、274个TS公共声明、366个Domain公共类型和171条命令保持；本批未改这些契约。

从bootstrap出发的含类型可达图431文件；其余24项逐项保留：10浏览器入口、1个SVG声明、2个公共feedback/overlays出口、11个已交付primitive家族文件。既有公共barrel不是重复业务。严格TypeScript未使用导入/参数检查通过，没有凭“当前主程序未调用”删除公共能力。

01提出的14个API候选已复核：ZIP选择/导出两项经main的files:api→MatchImports/MatchExchange实际消费；其余12项均具有原171命令契约中的真实native实现，保留公开兼容能力，当前GUI没有直接消费者的事实不隐瞒。具体方法、命令和实现路径见responsibility-inventory.json，不将它们写成12个当前GUI功能，也不删除接口制造清理成果。

## 唯一职责与副作用边界

| Owner | 职责及实际消费者 |
| --- | --- |
| main / predictionPage | 主组合入口拼装原历史列表；页面只展示传入内容及只读状态，不新增写入或缓存 |
| runs/runHistoryView | 完整历史和紧凑历史的唯一列表渲染，原查看/移除action、未知请求锁保留 |
| RunsFeature / Session | 前者拥有选择、请求与恢复；后者拥有列表缓存；隐藏仍先核原对象并读回 |
| predictionProjection / predictionResultView | 原实时/历史结果投影与只读详情；stored语义和缺失值继续保留 |
| frontendRuntimeGraph | 架构值图/未知加载；不成为生产加载器或新增持续runner |
| cargo | 唯一Cargo声明/继承解析与workspace图，供模块与受保护导入检查复用 |
| API Client / types | 原传输适配与wire声明；179成员、171命令、参数/返回/错误及存储/迁移不变 |

复杂剩余main、5个pages入口及保留兼容编排的具体消费者沿[01记录](R16-01-residual-entries-types-and-compatibility.md)，没有因行数搬运。新增3个架构lib文件分别拥有图推导、图样例和Cargo样例；没有新业务模块、runner、workflow或依赖。

## 场景与分层验证

必要证据根：O:/Project/FOODBALL/logs/r16-09-20261106。

| 场景 | 触发、期望与结果 | 证据及层级 |
| --- | --- | --- |
| V1 禁止依赖 | 用只读覆盖隔离输入运行原真实门禁，生产源码不改。旧Cargo表式漏检先复现；修复后运行时环、Feature别名私有导入、Tauri绕过、Platform逆向、未知动态、Rust禁止方向/目标别名/SQLx/workspace继承共9项均退出1；原树指纹前后一致 | baseline-cargo-missed-table.log、negative-gates.json及9日志；静态负向 |
| V2 动态/未引用 | AST与旧解析器集合一致；动态入口、资源、ZIP注入及24个非主图文件逐项说明，不自动删 | import-audit-before.json / import-audit.json、source-edges.json、runtime-edges.json、responsibility-inventory.json；静态审计 |
| V3 契约与可达 | API/types/Rust/命令/状态/依赖/迁移树与01基线无差异；17路由对应。18种真实生产渲染的空/有值/锁定/未知/错误/断库和formal/history/p4输出逐字节等同原页 | contract-audit.json、source-manifest.json、markup-preservation.json；源码/生产函数，非数据库 |
| 既有行为与构建 | 完整97步通过，含26项Runs、27项Prediction行为，S03完整挂载40项、S05完整挂载55项；查看原ID、取消0写入、成功移除、未知响应读回不重放均保留；严格TypeScript与Vite构建通过 | frontend-full.log / frontend-exit.json、full-approved-design；Windows浏览器使用明确transport边界，不称PG |
| 最终架构差异 | 完整前端运行期间补完Cargo解析回归；随后对最终代码执行完整verify:architecture通过，含28身份、14值图、16Cargo样例及四个指定入口/保护检查 | architecture-full.log、contract-audit.json；不是重复声称全套前端重跑 |
| Windows原生 | 复用未变Rust树的原isolated_log_commands_window宿主，加载本批dist：实际WebView、推演历史→运行页→返回、真实workspace读取与持久化、真实list_recent_runs断库拒绝等9项通过；12条运行日志startup通过，无非法行/运行错误 | native/report.json、native/startup.json、2张截图；原注册、无transport替换，未运行有库读写 |

原宿主硬限定的QA配置目录仍为tmp/r15-12-native-config，只是复用既有隔离测试入口；全部写入都在项目测试目录，不接触用户数据库/日志/凭据。断库拒绝是预期边界，不是成功读出业务记录。Rust/后端/依赖源码与已接受ed1e892d及01一致，复用768项/40 ignored和正式01 CI后端证据；不声称本批重新执行Rust全套或正式安装包验收。大于500kB的既有构建警告保留，未调阈值。

## 0641ef3f 自身CI失败与依赖启动修复

2026-11-06用户报告未通过后单次核对精确0641ef3f/CI42064975063/job127376481447为failure；失败在“Verify R1 architecture boundaries”。新加的TypeScript静态import在项目依赖安装前求值，ERR_MODULE_NOT_FOUND；尚未进入Windows Automated，未产生官方安装包。失败完整日志与片段保存在O:/Project/FOODBALL/logs/r16-09-ci-fix-20261106，不以旧本地通过抵销失败。

原verifyModuleBoundaries现在先调用现有ensureNodeDependencies，再动态加载TypeScript及其两个图模块；依赖准备失败仍直接失败。复用既有锁文件安装、上一级依赖布局及就绪标记，不新增依赖、runner、workflow，不跳过任何架构断言。

在项目tmp建立原CI“项目源码/../node_modules”布局，屏蔽本机更上层TypeScript回退：旧入口退出1复现；修复入口使用真实npm ci --include=dev后完整verify:architecture退出0；再次运行同门禁退出0且就绪标记未变、无重复安装；TypeScript实际为锁定5.8.3。cold-before/cold-fixed/warm-fixed日志及cold-validation.json分别保存。实际123 r3再次执行完整架构通过，见architecture-final.log。

本次只改变验证启动顺序和交接记录，src/Rust/依赖/配置/资源/171命令及状态契约均与0641ef3f无差异。因此复用本记录前述97步前端、18种HTML、完整挂载和9项原生证据，不能将其写成本次重新执行；新修复自身CI/包仍在推送后另验。R16与09继续VERIFYING，下一项仍先收尾09，再R17-01。

## BA承接、发布和回退

BA-01的最终当前门禁与BA-15的实际图/零过渡边在本批复核；没有新增例外。BA-01～16的其余事实与17页义务沿01记录逐项承接；真实PG、多事务/并发/回滚、XLSX数据库读回、授权API/凭据/取消、私有P4和Windows Full仍由R17-06按原约定验收，现库B契约问题不被核销。原40 ignored不变，不能用本批静态/浏览器证据替代。

本批提交推送后只按精确SHA查询自身CI一次；自身CI和官方双包/startup未核完时保持VERIFYING，R16阶段出口不提前DONE，R17不自动开始。下一项：收尾09后R17-01。

回退点3baf4ed2；撤回本批组合接线与守卫/记录即可，不涉及业务数据迁移或回退。必要证据与最终设计回归保留logs；无用任务临时项按清单归入唯一tmp/old，成功推送后逐明确文件实际删除、保留根目录并独立复核0，稳定依赖和缓存保留。


## 正式收尾

本次已按精确c1574990c19c3925722065417ea224d9d26d0c61单次核对CI42125458762/job127588365345全部success；初次0641ef3f的失败保留，不被覆盖。官方artifact13462724756共52,499,791字节，SHA256 bff02c320279bce630f13420e8e0e2f4dc85ef1cfbbadfbbcb1a6be30ef8caf8一致。MSI 18,079,744字节、NSIS 15,707,071字节，原文件签头及摘要通过；包内startup/pass，7条记录、0非法行/运行错误。详O:/Project/FOODBALL/logs/r16-09-ci-42125458762/acceptance.json，不是本机重新安装、数字签名或Full验收。09及原10的适用出口已关闭，原BA真实环境项仍承接R17-06。用户已授权收尾09并开始R17-01。
