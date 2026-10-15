# R10-13 Release Acceptance

状态：`DONE`（自身精确4ca43ad / Windows run38591308033完整SUCCESS）。用户2026-10-15“收尾13”；uniquenesssta/123 v0.23.0，继续唯一rewrite/r10-review-analytics-release-backend。基线04b2b4cb95cafc861505a4fa9d13bd35a37fce73为12纯文档收尾。01～13与R10代码/Windows Automated阶段出口均DONE；R11-01仅READY、未启动。下述VERIFYING/预期计数/未建阶段记录为实施历史，以阶段索引和本次完成节为准。

### 1. 目标

明确发布验收编排、检查构造、报告查询及PG事实/不可变事务/读取映射的唯一职责，保留原检查政策、报告身份和哈希。

### 2. 现状与来源

原run_acceptance/checks已有五类规则，summary/report/validation已独立；两个mod.rs仍承担编排和检查构造，list/read独立目录只含查询函数，PG release_acceptance.rs混合事实读取、报告事务与映射。保留已有正确职责，不复制规则或按文件数拆分。

### 3. 目标文件与目录

App沿用既有release/run_acceptance标识：{mod,workflow,validation,summary,report}.rs、checks/{mod,catalog,chain,performance,security,cost,release}.rs；release/queries.rs归并两个原报告查询，release/tests.rs测试。PG既有release_acceptance.rs升级为同名目录{mod,runtime,ledger,read,mapping,tests}.rs，lib.rs原登记保持。任务参考acceptance目录落到既有run_acceptance，不为外观改名或另造并行层。

### 4. 文件职责边界

workflow仅原请求→会话→事实→检查/汇总/hash→保存编排；catalog只原检查顺序/身份/序号/构造默认；queries只原会话获取与列表/单条透传。三个App目录出口仅登记/显式导出。PG runtime原六组事实读取，ledger原报告头/全部检查/审计单事务，read原列表/详情SQL与检查排序，mapping原字段/状态/JSON与六计数映射，mod仅登记。原八个规则、汇总、哈希与请求校验文件精确bytes不变。

### 5. 输入

原ReleaseAcceptanceRequest各字段、会话Future、ReleaseAcceptancePort四方法、limit/run_id；原PG运行事实、完整Run/Checks。无新配置、DTO、日志字段、版本或依赖。

### 6. 输出

原Run/Summary/Checks及公开壳/外部模型边界元数据、report_sha256。公开外部运行时缺席仍Warning，阻断/警告报告仍保存；不能将结构验收当私有模型真实通过。

### 7. 允许依赖

原Domain/ReleaseAcceptancePort、ApplicationResult/Error、Future/chrono/UUID、serde_json/sha2/hex；PG原SQLx/Row/Transaction、审计、PersistenceResult/Error。具体PG仅原composition adapter关联。

### 8. 禁止依赖

不新增Port/Repository wrapper/State、框架/依赖/runner/workflow/target/migration或DB专项，不改模型、参数、Profile、Schema、Golden或自动晋升规则。18保护资产含公开模型守卫本身，全部冻结。

### 9. 状态所有权

原数据库报告/检查/审计表持有状态；App临时报告与测试Fixture不算共享生产State。PG根模块身份/直接38/state19保持，清单仅同名.rs→目录mod.rs。App422/43Ports/19Portfiles，Domain365不新增。

### 10. 副作用边界

原窗口归一化和预算校验先于会话Future被poll；再开始时间/run_id→session→一次事实读取→13检查固定顺序/一基序号→分类汇总/完成时间→原JSON哈希→同一Run DTO→persist成功后直接返回，无隐式回读。PG runtime按health、stage、immutable、chain、performance、cost原顺序，无新增读事务。ledger单tx报告头→逐检查→审计commit；列表/详情独立原只读查询与映射，不重算/覆盖哈希。

### 11. 异常路径

预算非法优先原Validation，不获取会话；会话/事实/hash/保存错误按原?传播，不返回成功、不重试。SQL/检查/序列化/审计/commit失败按原事务释放回滚，不新增冲突成功或补偿。缺记录仍原InvalidState，状态大小写/空白不修剪，未知原错误保留。读取六计数负值仍默认为0，不加强旧数据拒绝。

### 12. 并发/异步/生命周期

原不可变表/触发器及主键、随机Run/Check ID、整套报告单事务保持；重复ID写入错误不变，不新增幂等覆盖。Future保持lazy获取，不添加Repository层UI取消/监听State。实际PG不可变触发器/回滚效果仍最终新库验收，单测Fixture和静态比较不替代实跑。

### 13. 兼容要求

三个facade/Service签名、Port四方法、四PG公共签名、SQL/绑定/错误/JSON/报告键/版本/时间/身份/日志/UI、0.23.0/依赖/迁移保持。原A–I阶段码存在检查、迁移>=27、触发器>=8、性能500/1500及2000/5000严格大于阈值、成本缺预算warning/严格超预算blocked、J/版本检查全部不改。汇总Blocked优先Warning，BTreeMap分类排序；13检查顺序、序号、UUID和非负duration投影保持。

哈希原run_id/app/contract/fixture/status/timestamps/requested_by/counts/categories/performance/cost/checks字段与serde_json精确序列化策略保持，未扩大为整个原请求；原只读查询保留存储哈希，不新增验证或重建。PG list夹紧1..200/完成时间和id倒序、详情check按sequence_no；原字段getter/错误顺序和六i32→u32默认转换不变。

### 14. 实施步骤

核实12精确完整Windows并五文档收尾。移出两个mod的三原函数，合并两个查询，切三Service真实调用；PG迁8原函数并抽详情字段mapper/复用六计数投影，清理旧入口/单文件，保留原纯规则及保护资产。補19原unit，更新两个既有守卫、正式扫描，完成等价/负向/架构/保护与记录后推送原Windows。

### 15. 切换入口

ReleaseService直接run_acceptance::run_release_acceptance和两个具名报告查询，原facade/lazy session/组合adapter保持。checks显式导出catalog的build/check，函数可见性仅原run_acceptance祖先域，支持原summary测试。PG lib原mod release_acceptance指向同名目录，PostgresStore API由新职责唯一实现，未留root execute/PG转发。

### 16. 删除清单

两个App旧list_runs/read_run入口mod.rs及目录、PG release_acceptance.rs删除。run_acceptance/mod和checks/mod的原业务迁入workflow/catalog，旧位置不留函数。逻辑迁移五App/八PG，新增内部run_from_row/report_count；整文件原样移动/重命名无。实际3D，完整A/M/D见下方。

### 17. 最小验证

82/82原只读源码守卫、完整npm run verify:architecture实际exit0、14现存变更Rust独立Rustfmt1.88 --check、18等价、22负向拒绝/bytes恢复、保护/命令/数据库静态基线、gitdiffcheck/23字段/27paths/links审核通过。新增15App/4PG原unit尚待自身Windows，预期Application230=215+15、Persistence259=255+4、Gateway112unit+41contract。

### 18. 阶段回归

沿用原Public Platform CI Windows完整前端/contracts/TypeScript/Vite/17视口、fmt/lockedClippy all-targets-Dwarnings/workspace、release/MSI/NSIS/启动/日志。82源码检查排除Windows进程与五原浏览器入口；未执行非Windows Cargo/check/Clippy/build/test/frontendbuild/browser/loopback，不新增验收设施。

### 19. 失败停止条件

13自身精确完整Windows SUCCESS前VERIFYING，R10IN_PROGRESS、R11未启动；不继承12PASS、不创建阶段完成记录。实际Windows job启动即停止所有CI查询并提示20～30分钟；失败仅修同R10分支。

### 20. 回退点

04b2b4cb95cafc861505a4fa9d13bd35a37fce73（12纯文档收尾，源码同已验6c4243c）。受控revert13源码/守卫/清单/记录，不能复制旧职责造成双owner。

### 21. 根 README 摘要记录

根README记录12DONE、13编排/查询/检查构造与PG不可变事务、原规则与哈希保护、19新增unit及自身Windows待验，链接本节点和索引并列实际计数。

### 22. docs 阶段节点详细记录

本23字段/27路径记录实际run_acceptance取舍、lazy会话/阻断报告、原哈希/映射/事务、验证/风险及回退。索引/任务书/TESTING同步；12收尾五doc单独04b2b4c[skip ci]。18等价含三App原完整入口、两个catalog原函数、八规则/汇总/hash/原summary测试冻结、六PG完整函数及两映射逆切、四PG签名、77生产字面量、Service全文（仅格式尾逗号）、边界/扫描/原targets。22负向覆盖旧owner/mod业务复活、会话/保存/阻断/查询规则、检查顺序/序号/UUID、hash检查字段、窗口/最新用量、冲突/审计/哈希绑定、读取排序/limit、状态/计数/测试登记及新owner模型敏感名单覆盖。

### 23. 完成标准

职责/调用切换/旧实现清理/静态证据和记录完成；自身完整Windows SUCCESS后才能DONE。真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验，ignored/公开stub/static指纹不是这些真实PASS。阶段收尾/R11须之后按用户授权和实际出口门禁进行。

## 测试与证据

15App测试覆盖归一化窗口/原身份/同报告保存后返回，非法含NaN/Infinity预算先于lazy session且日预算错误优先，三入口会话错误，六Port错误事实失败不保存/保存失败不成功不重试，重复报告独立ID及先前回包保持，13检查身份序号/分类排序和公开外部运行时warning，成本缺预算/等额不超限/严格超限，原延迟和P95边界，阻断链/J/安全仍记录，负运行计数原0投影，独立公共DTO哈希投影及字段/证据篡改敏感性，两查询身份/limit/原记录透传及六错误停止。4PG测试覆盖精确三状态、未知含空白/大小写原错误、负/零计数回落0及i32正数全范围。只原unit targets，Fixture不计数据库或私有模型实跑。

初次拟扩展公开模型守卫时触发其冻结指纹，已恢复原文件精确bytes；新workflow/queries/catalog由既有Release守卫读取原敏感名单，包括转义字符串、保持大小写不敏感，并有负向证明。初次名单正则误解析转义符已改为JSON解析原数组，预算获取次数守卫由负向补齐；最终82源码/22负向与全部指纹通过，未改保护清单或削弱原守卫。

实际命令：scratch/r1013_static.py、npm run verify:architecture、verify-release-service.mjs/verify-release-acceptance.mjs、verify-application-ports.mjs --refresh-source-scan、generate-domain-type-inventory.mjs、verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs、独立Rustfmt1.88 --edition2021 --config skip_children=true --check、gitdiffcheck；scratch/r1013_compare.py/r1013_probes.py静态逆切/破坏恢复。r1013-static-checks.json/architecture.log/equivalence.json/negative-probes.json是可复建临时报告，不入repo/不新建验收框架。

扫描App422/43Ports/19Portfiles，PG直接38/state19；Domain365/129声明/301映射、scan1272，sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd，rustUsageDigestae0f4f8d4a1371c324520752be2db969b911c142ee70a69c81ed7d310eb0178c。18assets aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；171命令；46迁移/18PGstatic aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

Mermaid Chart中文实际链区分预算失败不获取会话、所有状态报告写入、独立查询不重算/hash、单事务失败释放；既有API无版本变化，不需重复Context7。Create State稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存交接。附件仅scratch读取，Git制品不另存，无子代理。

## 全部实际文件清单

以04b2b4c基线git diff --no-renames及新增文件生成；职责迁移五App/八PG，整文件原样移动/重命名：无。

### 新增（11）

- `crates/application/src/use_cases/release/queries.rs`
- `crates/application/src/use_cases/release/run_acceptance/checks/catalog.rs`
- `crates/application/src/use_cases/release/run_acceptance/workflow.rs`
- `crates/application/src/use_cases/release/tests.rs`
- `crates/persistence-postgres/src/release_acceptance/ledger.rs`
- `crates/persistence-postgres/src/release_acceptance/mapping.rs`
- `crates/persistence-postgres/src/release_acceptance/mod.rs`
- `crates/persistence-postgres/src/release_acceptance/read.rs`
- `crates/persistence-postgres/src/release_acceptance/runtime.rs`
- `crates/persistence-postgres/src/release_acceptance/tests.rs`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-13-release-acceptance.md`

### 修改（13）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/application/src/services/release/service.rs`
- `crates/application/src/use_cases/release/mod.rs`
- `crates/application/src/use_cases/release/run_acceptance/checks/mod.rs`
- `crates/application/src/use_cases/release/run_acceptance/mod.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-release-acceptance.mjs`
- `scripts/verify-release-service.mjs`

### 删除（3）

- `crates/application/src/use_cases/release/list_runs/mod.rs`
- `crates/application/src/use_cases/release/read_run/mod.rs`
- `crates/persistence-postgres/src/release_acceptance.rs`

实际27路径，14现存变更Rust格式通过；已删除文件不参与Rustfmt。移动/重命名：无。


## 精确 Windows 完成与阶段收尾（2026-10-15）

精确提交 `4ca43ad1c4757ace4c95e126bda34f470d3854ea` / tree `a37d2b14c7748d92600d6d61bc846f87ac057fb9`，[Public Platform CI run38591308033](https://github.com/uniquenesssta/123/actions/runs/38591308033) / Windows job115946614576完整SUCCESS，2026-10-15 00:09:19北京时间完成。实际Application230/230、Persistence259/259、Gateway112/112 unit及41/41 contract；本项新增15App/4PG unit逐项成功。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、客户端启动和运行日志扫描均通过（7条记录、3个完成操作）。

报告 `logs/windows-acceptance-20261014-154132.json` 及同名.txt；未过期artifact11908923021，14,051,269字节，SHA-256 `046186113ac42de27713e3b8633f83253c4844cb2690378da9ab9cd838984afa`。已核对run精确head/分支、全部job步骤、完整日志和artifact；本次纯文档[skip ci]收尾复用该源码树，不重复运行同源码Windows。

用户本次只要求“收尾13”。13已DONE；阶段完成记录汇总全部13节点、真实文件总表、兼容/限制/回退，R10阶段出口DONE。R11仅01 READY、02～10 BLOCKED，未开始源码实施或新分支。

真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险继续最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实项目PASS。
