# R9-09 Session Persistence

状态：`DONE`（09自身精确Windows全SUCCESS；下述VERIFYING为实施历史）。用户2026-10-12 10:14:53北京“收尾08开始09”授权；原分支rewrite/r9-research-ai-backend。08精确7fd7e1e0eeec5f8b1e3a6a516a32ba21e9f067f9 / run38191340582、job114624551223全SUCCESS并DONE；基线06ecdd2172eecc539a4d0b3d9b0e5be76ae1a7e5为仅文档收尾，源码同该成功。09须自身Windows，10～11 BLOCKED、R9 IN_PROGRESS。

### 1. 目标

会话用例与PostgreSQL会话持久化各有唯一目录，完整切换实际消费链并保留数据/事务/公开接口。

### 2. 现状与来源

R0类型/命令/调用方清单与实际链路已核对：Tauri会话命令→ApplicationService facade→AiWorkspaceService→会话用例→ApiWorkspaceSessionPort→原composition adapter→PostgresStore。原8个会话用例分散为叶目录，PG api_workspace.rs混持会话/消息/回答事务/文件/用量和提案状态流转。

### 3. 目标文件与目录

Application use_cases/ai_workspace/sessions含8既有用例文件、仅登记/显式re-export的mod和原unit target内tests。PG adapters/ai_workspace/sessions含catalog/messages/assistant_bundle/generated_files/usage/mapping/validation，两个目录mod仅登记。

### 4. 文件职责边界

Application各用例正文不变，Session出口直接导出原execute别名，不转发。PG catalog负责会话CRUD/详情聚合，messages负责原普通消息写入，assistant_bundle拥有完整回答事务，generated_files读取内容，usage聚合成本，mapping只Row→DTO，validation只原写入载荷约束。旧api_workspace.rs保留提案状态流转及其唯一校验/映射，供回答事务/详情复用。

### 5. 输入

原Session/Message/Operation/File DTO、UUID、limit、attachments及DatabaseSession Future/Ports；PG继续原PostgresStore.pool，不新增连接/配置/字段。

### 6. 输出

原会话/消息/详情/文件/用量DTO、ApplicationResult/PortResult/PersistenceResult；SQL projection、字段名、JSON值及错误文字保持。

### 7. 允许依赖

原Domain、Application Ports/Future/serde_json、原PG sqlx0.8.6/uuid1.23.5/serde_json及crate审计函数。数据库具体依赖继续composition-only；未新增crate/依赖/框架。

### 8. 禁止依赖

Application用例不导入PostgresStore/PersistenceStore/sqlx；PG不依赖Gateway/UI/API发送，不新增DDL/删除数据/全局状态。不复制原提案校验/映射，不提前实施10注册表或11诊断。

### 9. 状态所有权

原AiWorkspaceService、数据库服务、DatabaseSession及PostgresStore.pool仍唯一；无新共享State/缓存/连接。新增SessionFixture为cfg(test)局部mock，不是运行时状态。

### 10. 副作用边界

PG原具名方法独占SQL/事务/审计；Application只调用Port，composition adapter和Tauri入口字节不变。没有新增文件导出/日志/HTTP/数据库迁移副作用。

### 11. 异常路径

预设/必须比赛/空用户输入仍先于数据库Future；Port错误原样ApplicationError::Port。PG消息角色精确匹配和100000字符限制、文件120字符/2MiB字节/MIME/非法名称保持；原空内容与供应hash兼容不擅自强化。会话缺失/已归档、回答assistant角色、提案/文件会话不一致及各原恢复/错误语义保持。

### 12. 并发/异步/生命周期

回答事务原消息→提案身份/校验/hash/插入→文件身份/校验/插入→更新时间→审计→commit→回读顺序、同一&mut *tx与早退生命周期保持。普通消息原插入和更新时间两次autocommit不暗改；读取仍原多查询视图，不擅自升为快照事务。取消注册表/过期结果仍由原调用方负责，真实并发/回滚最终新库待验。

### 13. 兼容要求

8用例字节一致、14个原PG方法正文一致、7原helper正文一致；公开Application/Ports/PostgresStore方法/DTO/Schema/profile/数据/DB迁移/Tauri/UI/模型/版本/依赖保持。只增加两个原提案helper的crate内可见性以复用，三个会话mapper和两个载荷validator为目录内pub(super)。Gateway正式Responses与普通双协议边界均不变。

### 14. 实施步骤

按实际8 Session Port能力移动Application用例、切换Service；迁PG9会话方法及5helpers，保留5提案方法与2helpers唯一owner。补4Application/4Persistence unit于原targets，两个既有门禁/清单/记录同步，无新runner/target/DB专项入口。

### 15. 切换入口

Service直调ai_workspace::sessions显式导出；原composition adapter仍调用同一PostgresStore公开方法，由新PG模块实现。旧8用例目录移除，旧PG9会话方法/5helpers移除，无旧路径转发或重复实现。

### 16. 删除清单

8原叶mod路径为移动，正文不改；独立整文件删除无。旧PG会话职责块删除，原api_workspace.rs仍是实际提案owner，不是兼容空壳。

### 17. 最小验证

83/83既有JS静态、完整architecture、Rustfmt1.88对23变更/移动Rust文件check、等价/8破坏探针/diff通过。预期Application130/Persistence179（各新增4）、Gateway106unit/39contract保持，09自身Windows尚待执行完毕，未计动态PASS。

### 18. 阶段回归

原Public Platform CI/Windows Automated执行frontend/contracts/TypeScript/Vite/17视口、cargo fmt --all -- --check、cargo clippy --locked --workspace --all-targets -- -D warnings、cargo test --locked --workspace、Windows release/MSI/NSIS/启动/日志。不执行非Windows Cargo/check/Clippy/build/test/loopback/浏览器运行时。

### 19. 失败停止条件

自身精确Windows全SUCCESS前09 VERIFYING、10～11 BLOCKED。保护/契约/静态失败先修，不跳测试/抑制Clippy/放宽门禁，08成功不替代09验收。

### 20. 回退点

06ecdd2172eecc539a4d0b3d9b0e5be76ae1a7e5，同08成功源码。受控revert09源码/移动/清单/门禁/记录，不手工覆盖或恢复双owner。

### 21. 根 README 摘要记录

08 DONE、09 VERIFYING，明确会话目录/回答事务边界、真实变更/静态通过/Windows待验；细节引用本记录，未创建阶段完成或额外fix log。

### 22. docs 阶段节点详细记录

本节点含23字段、全部真实新旧路径/移动/清单、契约/验证/工具/失败预审修正/回退/延期；阶段索引链接本记录并同步状态。

### 23. 完成标准

实现、静态和记录完成；自身Windows全SUCCESS后方可DONE。真实数据库/历史保护/账本并发回滚/有效XLSX/Full/私有固定回归继续最终封包新库待验，ignored/公共stub不计真实PASS。

## 行为覆盖与证据

新增4 Application unit：3类输入验证先于数据库factory polling；用户文本trim、附件元数据只存name/media/hash/大小/truncated及原Port错误；create/list/read/archive/file/bundle身份与全部载荷顺序/错误传递；正式用量先于workspace聚合、浮点成本原加法与u64请求数饱和。mock仅实现原Session/Audit Ports，无具体数据库依赖、原用例正文不改。

新增4 Persistence纯unit：三精确合法角色/空内容兼容、非法角色先于长度；Unicode字符100000边界；名称/MIME原大小写/空白/禁字符；120字符名与2MiB字节载荷区分Unicode长度，原提供hash不自行重算。未访问真实PG/API/系统凭据；Row映射和事务用源码等价验证，不能以这些unit冒充真实SQL/回滚执行。

等价报告确认8 Application移动正文与Service逆切入口字节一致、全部14 PG方法字节一致（包括9会话和5提案）、7 helper正文一致/签名仅必要内部可见性与格式、133个原生产字符串出现多重集一致；其余Application/PG源码与原测试、公开lib/Ports/composition/Tauri/Gateway/模型均保持。policy_changes=[]。

8探针：active过滤、limit上界、三历史顺序、回答写入绕出事务、提交提前至审计前、文件字节改字符、旧用例入口、供应hash绑定漂移；均被原AI Workspace门禁拒绝且完全恢复。

本地命令`python /workspace/scratch/eb298ad5cdcb/r909_static.py`、`npm run verify:architecture`、Rustfmt1.88 `--check --edition 2021 --config skip_children=true`23文件、`python /workspace/scratch/eb298ad5cdcb/r909_equivalence.py`、`python /workspace/scratch/eb298ad5cdcb/r909_probes.py`、`git diff --check`；报告r909-static-checks.json/r909-architecture.log/r909-equivalence.json/r909-negative-probes.json/r909-review.json在/workspace/scratch/eb298ad5cdcb。

Domain扫描1141→1152、365/300/声明摘要保持，usageDigest de438d1742d587852c69db2d0f2da9e9cd84cafa97f7820e967e5de8fe5af9b5。Application402→404文件、43Ports/19Port源保持，架构登记PG owner35→36只新增实际顶层ai_workspace adapter，不新增连接/状态；19状态、171命令、46迁移/18PG集成源码及18保护资产保持。保护聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57，迁移/既有18PG源码静态聚合d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

## 工具、预审修正与待验

Context7查询SQLx Transaction drop/rollback仅返回latest文档，未冒充锁版本；原launchbadge/sqlx文件读取失败后改读官方transact-rs/sqlx **v0.8.6 sqlx-core/src/transaction.rs**，确认open事务Drop通过start_rollback排队回滚的原生命周期，保留原commit/早退，不将源码推断记真实回滚PASS。Mermaid展示真实Session Port→adapter→查询/回答事务→审计/提交/回读及共享提案校验，Create State先确认足球稳定ID96f03270-c236-46de-99fa-db85d2fbf4ce。

首轮静态82/83与architecture失败：清单误登记递归Session出口且未更新实际顶层owner计数。按原lib/adapters声明严格扫描修正为实际ai_workspace/mod.rs、35→36，原验证器未改，复跑83/83与完整architecture通过。等价辅助计数初次因新crate内可见性导致格式化补参数尾逗号，改为helper正文逐字与等价签名核对，生产行为未改。

同一分支推送原Windows CI；确认本项精确head/Windows job开始即停止轮询，预计20～30分钟。09 VERIFYING、10～11 BLOCKED。真实PG会话/回答事务、历史四项、账本并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub及静态指纹不计真实执行PASS。

## 完整实际路径清单

相对06ecdd2，含新路径，按`git diff --no-renames`核对为 **20A / 13M / 8D**，41路径；其中8对为正文逐字不变的移动。按Git rename识别为12A/13M/8R100/0独立D，33变更条目。

新增路径（20，其中8为移动目标）：

- crates/application/src/use_cases/ai_workspace/sessions/append_assistant_bundle.rs
- crates/application/src/use_cases/ai_workspace/sessions/append_user_message.rs
- crates/application/src/use_cases/ai_workspace/sessions/archive_session.rs
- crates/application/src/use_cases/ai_workspace/sessions/create_session.rs
- crates/application/src/use_cases/ai_workspace/sessions/list_sessions.rs
- crates/application/src/use_cases/ai_workspace/sessions/mod.rs
- crates/application/src/use_cases/ai_workspace/sessions/read_generated_file.rs
- crates/application/src/use_cases/ai_workspace/sessions/read_session.rs
- crates/application/src/use_cases/ai_workspace/sessions/tests.rs
- crates/application/src/use_cases/ai_workspace/sessions/usage_totals.rs
- crates/persistence-postgres/src/adapters/ai_workspace/mod.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/assistant_bundle.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/catalog.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/generated_files.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/mapping.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/messages.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/mod.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/usage.rs
- crates/persistence-postgres/src/adapters/ai_workspace/sessions/validation.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-09-session-persistence.md

修改（13）：

- README.md
- architecture/application-port-inventory.json
- architecture/domain-type-inventory.json
- architecture/module-boundaries.json
- crates/application/src/services/ai_workspace/service/sessions.rs
- crates/application/src/use_cases/ai_workspace/mod.rs
- crates/persistence-postgres/src/adapters/mod.rs
- crates/persistence-postgres/src/api_workspace.rs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/README.md
- scripts/verify-ai-workspace-service.mjs
- scripts/verify-api-workspace.mjs

移除旧路径（8，均为移动来源，独立删除无）：

- crates/application/src/use_cases/ai_workspace/append_assistant_bundle/mod.rs
- crates/application/src/use_cases/ai_workspace/append_user_message/mod.rs
- crates/application/src/use_cases/ai_workspace/archive_session/mod.rs
- crates/application/src/use_cases/ai_workspace/create_session/mod.rs
- crates/application/src/use_cases/ai_workspace/list_sessions/mod.rs
- crates/application/src/use_cases/ai_workspace/read_generated_file/mod.rs
- crates/application/src/use_cases/ai_workspace/read_session/mod.rs
- crates/application/src/use_cases/ai_workspace/usage_totals/mod.rs

8移动对应：

| 原叶目录mod | Session目标文件 |
|---|---|
| use_cases/ai_workspace/append_assistant_bundle/mod.rs | use_cases/ai_workspace/sessions/append_assistant_bundle.rs |
| use_cases/ai_workspace/append_user_message/mod.rs | use_cases/ai_workspace/sessions/append_user_message.rs |
| use_cases/ai_workspace/archive_session/mod.rs | use_cases/ai_workspace/sessions/archive_session.rs |
| use_cases/ai_workspace/create_session/mod.rs | use_cases/ai_workspace/sessions/create_session.rs |
| use_cases/ai_workspace/list_sessions/mod.rs | use_cases/ai_workspace/sessions/list_sessions.rs |
| use_cases/ai_workspace/read_generated_file/mod.rs | use_cases/ai_workspace/sessions/read_generated_file.rs |
| use_cases/ai_workspace/read_session/mod.rs | use_cases/ai_workspace/sessions/read_session.rs |
| use_cases/ai_workspace/usage_totals/mod.rs | use_cases/ai_workspace/sessions/usage_totals.rs |

## R9-09 完成与10授权（2026-10-12）

精确09提交4ed656d51fac604cf74665009cd340f4d7ddbc59 / treee1ea669d51e98fe11797fc90213df5bb3ccbf48b，[Windows run38196834750](https://github.com/uniquenesssta/123/actions/runs/38196834750) / job114641262453全SUCCESS，2026-10-12 11:10:30北京完成。Application130/130、Persistence179/179、Gateway106/106 unit和39/39 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11724985136，14,040,086字节，SHA-256 00fdea25a74bfc9fc99bf0031a03161e2f2eb5419f158065c4a5f43c335550b7；报告windows-acceptance-20261012-024340.json及同名.txt。09新增4Application/4Persistence unit均执行通过，未增加/运行真实PG专项。

09 DONE；用户2026-10-12“收尾09开始10”授权10 READY，同一rewrite/r9-research-ai-backend分支，11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG会话事务/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。
