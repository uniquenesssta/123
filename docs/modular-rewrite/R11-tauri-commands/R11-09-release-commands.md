# R11-09 Release Commands

状态：`DONE`（自身精确44f2047 / Windows run39986012998完整SUCCESS；下文失败/VERIFYING保留实施历史）。

R11-01～08 DONE（08自身精确858288e / Windows run39974291282完整SUCCESS）；09 Release Commands已实施VERIFYING，10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“通过收尾08开始09”，继续唯一rewrite/r11-tauri-commands；09基线dd01f9d为08五文档[skip ci]收尾，源码同08已验树。

## 目标、来源与责任边界

原`src-tauri/src/commands/release_acceptance.rs`聚合执行、列表、读取三条命令。本节点将三个独立动作归入`commands/release/`，每条具名文件唯一真实命令；`mod.rs`只显式导出原函数及Tauri `__cmd__`、`__tauri_command_name_`双宏，根`commands.rs`只替换模块身份。没有引入转发占位层、dispatcher或新业务规则。

| 命令 | 目标文件 | 原真实委托 |
|---|---|---|
| run_release_acceptance | release/run_release_acceptance.rs | state.service.run_release_acceptance(request).await |
| list_release_acceptance_runs | release/list_release_acceptance_runs.rs | state.service.list_release_acceptance_runs(limit).await |
| read_release_acceptance_run | release/read_release_acceptance_run.rs | 原parse_uuid(&run_id, "验收运行 ID")后read_release_acceptance_run(...).await |

执行请求仍是唯一Domain `ReleaseAcceptanceRequest`；响应仍原Run/RunSummary，错误仍`error.to_string()`。列表limit不在命令层新增修剪/上限，run_id不trim或替换，UUID错误仍原标签。已有正确DTO无需复制，本层没有新增request/response/mapping/error空壳。

## 公共契约、状态与副作用

171公共命令名称、全局注册顺序、三签名/完整正文、DTO/Schema/配置/日志/UI/前端API字节保持。内部物理组从release_acceptance改为release，12组不变；其余11组完整保持。`bootstrap/command_registry.rs`与`lib.rs`冻结，不提前推进10。命令定义映射SHA-256为`c45a2d4b54fc5ad7eeaae3f3b57eda65f58ce03dc6b1d6be3576fb807b1d0696`；公共集合仍`dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669`。

AppState/19状态owner、ApplicationService/ReleaseService与43Ports/19Port源、PG唯一owner全部冻结。命令只借原State并委托；不新增SQL、文件I/O、模型公式、服务实例、线程/锁/令牌/缓存/共享状态或重试。本来无独立取消生命周期的薄命令不套入新取消机制，沿原caller/服务错误生命周期。

真实执行链仍为原ReleaseService→run_acceptance：预算校验/窗口规范化→获取会话→runtime_facts→checks→summary→报告SHA256→persist_run→返回同一报告。Blocked报告也保存；未将公开unavailable stub或ignored计作真实模型/数据库成功。原PG ledger报告/checks/审计单事务、不可变策略与查询的排序/上限/不存在错误保持，命令不新增保存或覆盖逻辑。全部原Application15项与PG4项Release测试正文冻结。

Mermaid Chart已展示以下真实链路，图只表达原生产实现及本次命令入口：

```mermaid
flowchart TD
  API["发布验收页面与原API"] --> RUN["执行命令"]
  API --> LIST["列表命令"]
  API --> READ["读取命令：原UUID校验"]
  RUN --> SERVICE["原ApplicationService / ReleaseService"]
  LIST --> SERVICE
  READ --> SERVICE
  SERVICE --> CHECK["执行用例：预算校验、事实、检查、汇总与哈希"]
  SERVICE --> QUERY["查询用例：原窗口与身份"]
  CHECK --> PORT["原ReleaseAcceptancePort"]
  QUERY --> PORT
  PORT --> TX["执行：报告、检查、审计原单事务"]
  PORT --> PGREAD["查询：原排序、上限与缺失错误"]
  TX --> DB["原PostgreSQL不可变报告"]
  PGREAD --> DB
```

## 入口、清单与原守卫

正式command-contract与module-boundaries切至release/mod.rs及三实际定义文件，删除旧单文件。Domain声明、365类型/129源/301映射保持，仅三类型的实际外部caller由旧聚合路径展开；Rust扫描1491→1494，sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`不变，usageDigest为`58d46aa55e321b266a05039f1dcdbda14716cace4679af9f22b4a71151c59f95`。

两个既有发布守卫改读三个实际定义文件；所有原断言经逆变换精确保持。只在原Release Service守卫补实际State/三签名/完整传输正文/参数/await/错误/唯一helper/纯双宏出口/实际清单与旧owner清零检查。正式command verifier、原82入口、package/CI/依赖/迁移与其余守卫冻结，无新增持续回归、runner、workflow、target或数据库专项入口。

## 实际验证与待验

- 82/82原只读Node源码检查、完整`npm run verify:architecture`通过。
- 5个现存变更Rust文件独立Rustfmt1.88（edition2021、skip_children=true、--check）通过；不是Windows Cargo fmt证据。
- 42项源码等价：三完整正文/签名/原State、root仅身份替换、公共契约/其余11组、唯一边界变化、两原守卫逆变换、全部365Domain项除三实际caller路径、扫描+3和全业务/UI/测试/模型/Ports/依赖/CI/state冻结通过。
- 21项隔离负向全部拒绝，3原守卫恢复正向通过，主250源码/架构/守卫/根/注册指纹不变。覆盖旧owner、根登记/双宏/出口、请求/limit/UUID/标签/trim/方法/await/错误/State/清单，及原下游保存/提交/排序丢失。仅隔离临时副本变异，未破坏主源码。
- 18模型保护资产aggregate `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`与46迁移/18PG静态契约aggregate `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`通过，`git diff --check`通过。

新增单测0、迁移单测0；三个薄命令原完整实现不变，复用现有targets。预期Desktop58/Application230/Persistence259/Gateway112unit+41contract须09自身精确Windows确认。当前没有在非Windows执行Cargo/check/Clippy/tests/前端构建/浏览器/loopback；所有动态验证继续原Windows workflow。需确认完整frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace tests、release/MSI/NSIS/启动与日志；08成功只用于08收尾，不能替代09。

报告`r1109-static.json`、`r1109-architecture.log`、`r1109-equivalence.json`、`r1109-negative.json`、`r1109-primary-fingerprints.json`位于repo外临时工作区，可从源码复建；不新增生产或长期测试runner。既有测试中依赖真实数据库的ignored仍未执行，不是PASS。真实PG报告事务/并发回滚、历史保护四项、不可变账本、有效XLSX、Windows Full、私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；08真实API/凭据/会话取消端到端待验也未消失。

## 切换、偏差与回退

先在原Release Service门禁补本节点传输要求，确认旧结构因缺release/mod.rs失败；再原样迁移三命令、切换唯一根/正式清单、删除旧owner、刷新实际reader/Domain usage。最小命令契约、全静态/架构/格式、等价/隔离探针均通过。使用沿前序精确Windows证明的Tauri2.11.1双宏出口，无新增API或未核实的版本行为，不引入依赖升级；本节点无Context7新问题。

本次没有功能偏差或已知实现失败；初期按任务简称探查release.rs，实际旧owner由源码/正式清单定位为release_acceptance.rs。09自身Windows未完成时保持VERIFYING，10 BLOCKED；实际Windows job首次in_progress后停止全部CI查询，等待用户返回。回退点为本节点前`dd01f9d5cb4983637f25c639835fcfe66a1ad17f`（源码同08已验858288e）；使用Git revert本节点原子提交，不手工复制旧文件，不合入其他工作。

## 实际文件清单

以下为初始dce36ac的历史清单：基于dd01f9d5cb4983637f25c639835fcfe66a1ad17f，no-renames口径5A/10M/1D，共16路径。4新Rust（3命令与1纯出口）；既有Rust仅根登记修改，整文件逐字移动/重命名无，三原函数按职责迁移。

| 状态 | 路径 |
|---|---|
| M | `README.md` |
| M | `architecture/command-contract.json` |
| M | `architecture/domain-type-inventory.json` |
| M | `architecture/module-boundaries.json` |
| M | `docs/TESTING.md` |
| M | `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md` |
| A | `docs/modular-rewrite/R11-tauri-commands/R11-09-release-commands.md` |
| M | `docs/modular-rewrite/R11-tauri-commands/README.md` |
| M | `scripts/verify-release-acceptance.mjs` |
| M | `scripts/verify-release-service.mjs` |
| M | `src-tauri/src/commands.rs` |
| A | `src-tauri/src/commands/release/list_release_acceptance_runs.rs` |
| A | `src-tauri/src/commands/release/mod.rs` |
| A | `src-tauri/src/commands/release/read_release_acceptance_run.rs` |
| A | `src-tauri/src/commands/release/run_release_acceptance.rs` |
| D | `src-tauri/src/commands/release_acceptance.rs` |


## R11-09 unused_imports 修复（2026-10-24）

### 本次失败与根因

精确提交`dce36ac4c6e9d4729f6bf3832961c2e7ba290f92` / tree`3e02fc03f23534f935bdbe18b18cf6d7d1d50dbf`，[Public Platform CI run39981102375](https://github.com/uniquenesssta/123/actions/runs/39981102375) / Windows job120508282038在2026-10-23 23:53:14北京时间以FAILURE完成。已fresh核对head/branch、全部job步骤与完整日志；源码架构、完整frontend/contracts/TypeScript/Vite/17视口通过，Rust1.88.0 Clippy阶段在`src-tauri/src/commands.rs:29:16`报唯一`unused import: crate::bootstrap::AppState`，`-D unused-imports`由`-D warnings`提升为错误，lib与lib test编译被阻断。工作区单测及Windows release/MSI/NSIS/启动/运行日志验收未完成，不能计PASS。

artifact12514082002仅668字节，SHA-256 `416fef090ef69dbe40863d39f8252625ffc189dcbca75b18c861f105d611a818`，未过期；仅为失败过程证据，不是完整验收包。失败过程记录`logs/windows-acceptance-20261023-154810.txt`。

旧release_acceptance.rs是根AppState中转的最后消费者；迁移后所有实际命令直接导入`bootstrap::AppState`，根的`pub(crate) use crate::bootstrap::AppState;`遂失去消费者。上轮遗漏清理这一行，旧bootstrap守卫又只要求该文字存在，无法识别它已经无效。问题在导入清理与过时守卫，不是服务/状态实例或发布业务行为。

### 实际修复与不变量

生产Rust只删除commands.rs这一行；实际AppState定义/7字段/安装、bootstrap公开出口、所有命令的原State签名与导入、全部服务/Port/PG/真实报告流程/DTO/公共171注册和模型资产全部字节保持。未改变任何业务行为，不新增allow/降低Clippy或关闭检查。

只修正原`verify-tauri-bootstrap.mjs`中已经过时的根重导出断言：根不得再出现AppState中转/替代定义；从正式清单逐个读取实际命令定义，使用AppState的命令必须直接借用bootstrap唯一类型。其余bootstrap所有断言经逆变换精确保持，原Release Service与Release Acceptance两守卫均冻结。不是为了让冗余入口通过而恢复消费者。

Domain清单仅rustUsageDigest刷新为`aa4b72f8b9d56d7dfee35efc2293e32e12998bcd72750e2a68d7c15846cffa10`；全部365类型/129源/301映射/1494扫描及全部类型caller/sourceDigest精确保持，未新增文件或状态。

### 实际验证与边界

82/82原只读Node、完整`npm run verify:architecture`、1个commands.rs独立Rustfmt1.88 --check、42项原09等价核对（明确仅归一化这次删除的无用重导出）、原正式command verifier、18保护资产/46迁移18PG静态契约与git diff --check通过。修复额外严格复核生产仅删除此行、其余全部Rust/业务/UI/测试/依赖/工作流冻结、Domain仅usageDigest变化、bootstrap除这一个失效断言更新外精确不变。

6项隔离破坏探针全部拒绝：原重导出、分组写法、换行写法、allow掩盖、命令改用已退役中转、bootstrap唯一状态出口被改写；3原守卫恢复正向通过，主252源码/清单/守卫/根与bootstrap状态指纹不变。只在repo外临时副本变异，主源码无破坏。报告r1109f-static.json、r1109f-architecture.log、r1109f-equivalence.json、r1109f-review.json、r1109f-primary-fingerprints.json在临时工作区。

本次新增/迁移单测0，全部原测试字节冻结；未在非Windows执行Cargo/check/Clippy/tests/前端构建/浏览器/loopback。预期Desktop58/App230/PG259/Gateway112unit+41contract须修复自身完整Windows确认，前序成功或本次静态不替代。继续复用既有Windows workflow；实际job首次in_progress后停止所有CI查询，等待用户返回。

Context7已查询Rust官方lint文档核对-D提升错误语义，但未返回1.88精确重导出条目；官网1.88归档页在工具中不可访问。对应版本直接依据是本次真实Rust1.88.0 Windows诊断；未改依赖/API/版本。单一导入清理不改变架构或真实流程，沿09现有中文Mermaid；Create State记录失败/修复与待验。

09保持VERIFYING、10 BLOCKED，不标记本节点或R11完成。真实PG报告事务/并发回滚、历史保护四项/不可变账本、有效XLSX、Windows Full、私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；08真实API/用户凭据/会话取消端到端仍待环境。ignored、公开stub及静态指纹不计真实PASS。回退本次修复会回到已知失败dce36ac，不是可验收回退；最近完整成功源码仍08的858288e/文档dd01f9d。

### 本次修复真实清单

相对dce36ac4c6e9d4729f6bf3832961c2e7ba290f92，8M/0A/0D；没有移动或重命名。

| 状态 | 修复路径 |
|---|---|
| M | `README.md` |
| M | `architecture/domain-type-inventory.json` |
| M | `docs/TESTING.md` |
| M | `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md` |
| M | `docs/modular-rewrite/R11-tauri-commands/R11-09-release-commands.md` |
| M | `docs/modular-rewrite/R11-tauri-commands/README.md` |
| M | `scripts/verify-tauri-bootstrap.mjs` |
| M | `src-tauri/src/commands.rs` |

相对09开始前dd01f9d的累计清单为5A/11M/1D，共17路径；比初始16路径仅增加既有scripts/verify-tauri-bootstrap.mjs的必要守卫修正。上方初始清单保留历史，当前额外8M如本表。


## R11-09 精确 Windows 完成与10授权（2026-10-24）

精确修复提交`44f2047c8391981618f4442ca1c06164626a2554` / tree`e84d02e1011920ed5dd2591a516a8aabf2c39e6b`，[Public Platform CI run39986012998](https://github.com/uniquenesssta/123/actions/runs/39986012998) / Windows job120525276486完整SUCCESS，run于2026-10-24 00:46:56北京时间完成。实际Desktop58/58、Application230/230、Persistence259/259、Gateway112/112 unit和41/41 contract；完整frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与运行日志均通过（7条记录、3个完成操作）。本节点新增/迁移单测0；首轮dce36ac unused_imports失败已由本次修复自身完整成功关闭，源码等价和静态探针不计动态测试。

报告logs/windows-acceptance-20261023-162241.json及同名.txt；artifact12517606093，14,060,201字节，SHA-256 `ccf89e18048dca8ba08855d84dbc26b15aa1a5cb429eae65010938710a36ffd1`，查询时未过期。已fresh核对精确head/branch、全部job步骤、完整日志与artifact。用户2026-10-24“收尾09开始10”，09 DONE、10 READY、R11 IN_PROGRESS；此次仅5份既有文档[skip ci]收尾，源码同09已验树，不重复同源码构建。R11阶段完成记录尚未创建，10与阶段出口须按自身实际影响验收。

真实PG报告事务/并发回滚、历史保护四项/不可变账本、有效XLSX、Windows Full、私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；08真实API/用户凭据/会话PG取消UI端到端同样待实际环境。ignored、公开unavailable stub和静态指纹不计这些真实PASS。
