# R10-12 Parameter Lifecycle

状态：`DONE`。用户2026-10-14“收尾11开始12”授权；uniquenesssta/123 v0.23.0，继续唯一rewrite/r10-review-analytics-release-backend。基线6da505780c366e0f495f4bab9de54b257cbb245a为11五文档[skip ci]收尾，源码同自身完整Windows SUCCESS的9d3727c。01～12DONE、13READY、R10IN_PROGRESS。12自身6c4243c完整Windows SUCCESS；用户2026-10-14“收尾12开始13”授权13。

### 1. 目标

收拢参数候选、就绪检查、影子验证、人工晋升/回滚的实际入口和PG读写/事务所有权，保留原模型和参数保护、指标与不可变账本。

### 2. 现状与来源

App九个独立入口目录，PG参数生命周期单文件混合查询、制品、回放、报告与绑定事务，PG root analytics另持候选四入口/映射。原指标支持已独立为metrics.rs，继续完整保留。

### 3. 目标文件与目录

App既有use_cases/analytics/parameter_lifecycle/{mod,candidates,readiness,validation,bindings,metrics,tests}.rs；PG既有parameter_lifecycle.rs升级为parameter_lifecycle/{mod,readiness,candidates,artifacts,replay,shadow,bindings,decisions,mapping,policy,tests}.rs。沿用合法Rust标识和lib.rs原根模块登记，不另设Port、wrapper或平行adapter。

### 4. 文件职责边界

App candidates负责公开生成拒绝/候选列表/人工审核，readiness负责原公开请求校验，validation负责原完整影子验证工作流及报告列表，bindings负责人工晋升/回滚请求和账本列表，metrics原支持完整不变。PG readiness负责H契约/活动基线/精确样本查询，candidates候选登记/读取/审核，artifacts参数定义读取与三表制品事务，replay事实读取，shadow不可覆盖报告事务与回读，bindings两个人工绑定事务，decisions决策账本读取，mapping字段与原错误，policy原状态投影/人工状态门禁及原UUID/非负数规则。mod只登记/显式导出。

### 5. 输入

原九入口Draft/Request/limit/candidate_id、ModelRegistry、11方法ParameterLifecyclePort；原PG候选/制品/回放事实/报告/人工请求。无新参数、配置、DTO、状态字段或模型版本。

### 6. 输出

原候选、readiness、影子报告、晋升/回滚决策及列表。公开生成继续返回原“未捆绑模型提供器”错误；不虚构私有参数，不将stub当正式模型。

### 7. 允许依赖

原ApplicationResult/Error、ParameterLifecyclePort、Domain、ModelRegistry、football-model-api、原serde_json/chrono/UUID/指标支持；PG原SQLx/Row、Domain、PersistenceResult/Error和审计。Ports与具体PG仅既有组合适配关联。

### 8. 禁止依赖

无新增框架、依赖、runner、workflow、target、migration、DB专项或共享State；不自动晋升，不更改模型/参数/Profile/Schema/Golden，不复制原公式。

### 9. 状态所有权

原数据库候选/制品/报告/晋升账本/绑定表继续拥有状态；PG直接模块38/state19保持，清单仅parameter_lifecycle.rs→同名目录mod.rs。App420/43Ports/19Portfiles；无新生产State、Domain类型或跨层接口。

### 10. 副作用边界

公开readiness先四horizon与样本>=20，再Port。影子验证先读候选/人工队列状态/五身份→直接Port readiness；未就绪只构建blocked报告并保存。就绪后读精确回放→原排序/概率校验/最低样本/时间留出→读参数/原模型验证和predict→原指标及绑定未变门禁→保存报告，不自动触发人工请求。

PG制品单tx依次模型draft、参数draft、候选、审计commit→候选回读；shadow单tx原validation_key冲突DO NOTHING、原候选状态更新、审计commit→按key回读。人工晋升另单tx锁活动精确绑定、写决策/逐绑定快照、CAS替换、版本active/候选promoted、审计commit→决策回读。回滚另单tx读原快照、写rollback账本、CAS恢复、版本deprecated/候选rolled_back、审计commit→决策回读。

### 11. 异常路径

原说明、候选状态、缺身份、H、绑定变化、样本/概率/模型、Port/SQLx/Row错误和优先级保持。错误立即?传播，无新增重试、自动补偿或候选晋升。事务失败按原释放回滚；commit后回读错误不会撤销已提交记录。候选普通登记/人工pending审核仍原单语句后回读，不擅自换事务。

### 12. 并发/异步/生命周期

原报告唯一键/不可变触发器，晋升FOR UPDATE OF binding、逐绑定原值CAS和候选状态受影响行数检查、回滚同类CAS保持，各事务不合并。Repository不新增UI请求ID、取消State或监听器；真实并发/回滚/新库效果仍最终封包待验。

### 13. 兼容要求

九App签名、原21facade、11方法Port及所有PG公共签名、SQL/绑定/错误/JSON/模型请求/日志/UI、0.23.0/依赖/迁移保持。metrics.rs精确bytes冻结，公式、概率容差、留出比例、阈值、round6及validation_key内容/排序均不改。原公开readiness的四horizon/20门槛不额外套入内部候选流程，constraints.minimum_sample_size仍原as_u64或默认50。

说明校验仍trim().len()>=8（原UTF-8字节规则，未改为字符计数），原reviewer和说明值透传。PG候选list夹紧1..500、仅pending可人工接受进入accepted_for_backtest或拒绝；影子状态passed→shadow_passed、failed→shadow_failed、其他→blocked_by_h。重复报告不覆盖已有内容，但仍按原顺序执行候选更新/审计并回读已有报告，未新增“重复即提前返回”语义。UUID含nil仍原处理，负样本拒绝不夹紧；u64写入原as i64保持。

### 14. 实施步骤

核实11自身完整Windows并五文档收尾。App按四职责切九实际调用、删旧入口，PG同名模块升级并迁22原函数、root analytics保留九原函数；抽取四原状态/人工门禁片段为真实policy供原生产调用。补15App/6PG unit、更新两个原守卫和正式清单，完成等价、破坏恢复、架构/保护、文档后推送原Windows。

### 15. 切换入口

AnalyticsService直接调用parameter_lifecycle九具名函数，mod显式导出四职责，未保留execute转发。既有PG adapter继续同名PostgresStore API；lib.rs原mod parameter_lifecycle登记同名目录，所有旧函数由新职责唯一实现。兄弟bindings只需原decision单条回读改pub(super)，无公开接口扩大。

### 16. 删除清单

九App旧入口mod.rs/目录、PG parameter_lifecycle.rs退出；root analytics移出五候选职责与三个无用类型imports，九原函数保留。实际10D，逻辑迁移九App/22PG；没有复制旧版本或整文件原样移动，A/M/D按no-renames口径列于下方。

### 17. 最小验证

82/82原源码守卫、完整npm run verify:architecture、19存在的变更Rust独立Rustfmt1.88 --check、19等价、20负向拒绝/bytes恢复、保护/命令/DB静态基线、gitdiffcheck/字段/清单/links审核通过。新增15App/6PG原unit尚待自身Windows，预期Application215=200+15、Persistence255=249+6、Gateway112unit+41contract。

### 18. 阶段回归

沿用原Public Platform CI Windows完整前端/contracts/TypeScript/Vite/17视口、fmt/lockedClippy all-targets-Dwarnings/workspace、release/MSI/NSIS/启动/日志。无非Windows Cargo/check/Clippy/build/test/frontendbuild/browser/loopback，本地证据不冒充动态验收。

### 19. 失败停止条件

12自身精确完整Windows SUCCESS前VERIFYING、13BLOCKED，不继承11PASS，不创建阶段完成记录；失败修同R10分支。实际Windows job启动即停止所有CI查询并提示20～30分钟。

### 20. 回退点

6da505780c366e0f495f4bab9de54b257cbb245a（11五文档收尾，源码同已验9d3727c）。受控revert12整套源码/守卫/清单/记录，不能手工复制旧文件造成双owner。

### 21. 根 README 摘要记录

根README记录11DONE、12候选/就绪/影子/人工绑定的实际职责、公式与事务保护、21新unit和自身Windows待验，链接本节点/索引并列真实变更计数。

### 22. docs 阶段节点详细记录

本记录23字段与全部39路径；任务书/索引/TESTING同步12VERIFYING。静态19等价含18原PG完整函数及四状态抽取逆切后原事务、292生产字面量、全部公共签名、九App原体、原私有horizon、Service全文（仅原格式尾逗号除外）、metrics精确bytes、保护区/Ports/接口/工作流冻结、扫描和原targets。20负向覆盖旧入口/root候选职责复活、说明/样本/排序/概率门禁、自动晋升、H/分区、报告覆盖、审核状态、晋升/回滚状态、锁/CAS/行数、负数、制品表顺序及测试登记。制品存在性守卫经负向发现并补齐。

### 23. 完成标准

实际职责/调用方/旧实现清理/静态证据/记录完成；12自身完整Windows SUCCESS才能DONE。真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验，ignored/公开stub/static指纹不是这些真实PASS。

## 证据与未执行项

15App测试覆盖公开生成原拒绝、四horizon/20边界/错误优先级、limit0/max与两种人工审核/空白note透传、两账本查询身份、说明原字节规则、晋升readiness/原基线/缺scope/阻断原因顺序、显式回滚、影子队列状态与五身份、H阻断报告/完整原回包、无效概率/样本/外部模型缺席先于写报告以及六种Port错误停止/不重试。6PG测试覆盖人工候选两状态、影子状态精确映射和fallback、仅shadow_passed晋升/仅promoted回滚、原UUID nil/缺失错误及负数拒绝。Fixture不计真实数据库/模型实跑。

源码检查从原前端清单选择82只读守卫，Windows进程测试及五个原浏览器入口留Windows。初次误列的两浏览器入口因无解析器/浏览器失败，未启动浏览器、未计PASS；恢复锁定TypeScript5.8.3只用于原解析器，不改依赖锁。独立Rustfmt初次缓存组件提取不完整导致SIGBUS；从原1.88组件校验恢复后19文件格式通过，未编译源码。

实际命令：scratch/r1012_static.py、npm run verify:architecture、verify-analytics-service.mjs/verify-parameter-lifecycle.mjs、verify-application-ports.mjs --refresh-source-scan、generate-domain-type-inventory.mjs、verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs、独立Rustfmt1.88 --edition2021 --config skip_children=true --check、gitdiffcheck；scratch/r1012_compare.py/r1012_probes.py静态逆切/破坏恢复。r1012-static-checks.json/architecture.log/equivalence.json/negative-probes.json为可复建临时报告，不入repo或另建验收框架。

扫描：App420/43Ports/19Portfiles，PG直接38/state19；Domain365/129声明/301映射，scan1265，sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd，rustUsageDigest57b22b534e916a40c70c9a2b88eca9cb1eb04f110e011bd6f05644b51398df80。18assets aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57、171命令、46迁移/18PGstatic aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

Mermaid Chart中文实际链已更新；既有API/依赖未改变，不需重复Context7。Create State稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce记录交接。附件只读scratch、Git制品不另存、无子代理。

## 全部实际文件清单

以6da5057为基线，git diff --no-renames和新增文件生成；职责迁移九App/22PG，整文件原样移动/重命名：无。

### 新增（17）

- `crates/application/src/use_cases/analytics/parameter_lifecycle/bindings.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/candidates.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/readiness.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/tests.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/validation.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/artifacts.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/bindings.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/candidates.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/decisions.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/mapping.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/mod.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/policy.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/readiness.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/replay.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/shadow.rs`
- `crates/persistence-postgres/src/parameter_lifecycle/tests.rs`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-12-parameter-lifecycle.md`

### 修改（12）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/application/src/services/analytics/service.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/mod.rs`
- `crates/persistence-postgres/src/analytics.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-analytics-service.mjs`
- `scripts/verify-parameter-lifecycle.mjs`

### 删除（10）

- `crates/application/src/use_cases/analytics/parameter_lifecycle/decide_parameter_tuning_candidate/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/generate_parameter_tuning_candidate/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/list_parameter_promotion_decisions/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/list_parameter_shadow_validations/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/list_parameter_tuning_candidates/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/parameter_lifecycle_readiness/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/promote_parameter_candidate/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/rollback_parameter_candidate/mod.rs`
- `crates/application/src/use_cases/analytics/parameter_lifecycle/run_parameter_shadow_validation/mod.rs`
- `crates/persistence-postgres/src/parameter_lifecycle.rs`

实际合计39路径，19存在的变更Rust格式检查通过。移动/重命名：无；已删除文件不参与Rustfmt。


## R10-12 精确 Windows 完成与13授权（2026-10-14）

精确提交 `6c4243c1150ba0bbf2c0139429e52c3477df6d8a` / tree `8b3928901a0e495f211eb0b937322eb4b48541d2`，[Public Platform CI run38508581096](https://github.com/uniquenesssta/123/actions/runs/38508581096) / Windows job115671629414完整SUCCESS，2026-10-14 12:18:42北京时间完成。实际Application215/215、Persistence255/255、Gateway112/112 unit及41/41 contract；本项新增15App/6PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动及日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261014-035521.json及同名.txt。

证据artifact11869757552，14,050,358字节，SHA-256 `ae7631e1aafcfdfe6e230961c04709d0b8c205dc3e1b28213c7df67c5b4e5c1a`。12自身完整SUCCESS后DONE；用户2026-10-14“收尾12开始13”授权13 Release Acceptance READY，继续唯一rewrite/r10-review-analytics-release-backend，R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验6c4243c；13须自身精确完整Windows，无阶段完成记录。此前12VERIFYING为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不是这些真实PASS。
