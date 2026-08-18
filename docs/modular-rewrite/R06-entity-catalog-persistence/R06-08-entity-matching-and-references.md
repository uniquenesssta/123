# R6-08 Entity Matching 与 References

## 状态

`DONE`

## 基线与范围

- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`a44e11aceccdb1c582636ff8d4c49f9d24529baa`；生产 owner-switch 提交：`89dfbb101989116b5c858dbb8dc4564c50f86f9d`。
- 本节点只重写 Entity Matching 与 References persistence；R6-09 Archive/Delete/Force Delete 与 R6-10 Global Name Search 未提前迁移。

## 实现结果

- Entity Matching 收敛到 `adapters/catalog/entity_matching/`：resolve orchestration、stable-ID existence、external-ID lookup、name candidates、normalization 与 outcome 分责。
- References 收敛到 `adapters/catalog/references/`：reference directory read/mapper/list、provider validation/read/write/mapper、external-ID validation/write/mapper 分责。
- `resolve.rs` 与 reference `directory/list.rs` 仅编排，不直接执行 SQL；SQL I/O 集中在明确的 adapter read/write/lookup owner。
- `entity_catalog.rs` 已移除 R6-08 matching/reference-list owner，仅保留 R6-09 deletion/archive/reference-count；`player_catalog.rs` 已移除 provider/external-ID owner。
- R6-07 retained verifier、Entity Relationships verifier 与 Global Name Search verifier 仅跟随权威 owner 路径，原契约未删除、跳过或放宽。

## 契约与兼容性

- `EntityReferencePort`、Domain DTO、Schema、0001–0046 migrations、数据格式、配置、日志等级、错误语义、UI 行为均未改变。
- Stable entity ID 优先于 trusted provider external ID，external ID 优先于 normalized name matching。
- Team / Player / Coach canonical name 与 alias matching、country/nationality/date-of-birth disambiguation、`exact` / `no_match` / `ambiguous` 状态及 candidate score/reason 文案保持不变。
- Reference Directory 继续复用统一 `NameSearch`，保持中文、英文、别名、重音、多关键词检索、active-only、500 上限与稳定排序语义。
- Data Provider upsert、metadata merge、active re-enable 与 External ID upsert/metadata merge 语义不变。
- 历史 P4/运行引用保护不变；模型保护资产未变化；无新增生产依赖。

## 文件清单

### 新增

- `crates/persistence-postgres/src/adapters/catalog/entity_matching/existence.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/external_id.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/name_candidates.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/normalization.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/outcome.rs`
- `crates/persistence-postgres/src/adapters/catalog/entity_matching/resolve.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/directory/list.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/directory/mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/directory/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/directory/read.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/entity_type.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/external_ids/mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/external_ids/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/external_ids/validation.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/external_ids/write.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/providers/mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/providers/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/providers/read.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/providers/validation.rs`
- `crates/persistence-postgres/src/adapters/catalog/references/providers/write.rs`
- `crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs`
- `docs/modular-rewrite/R06-entity-catalog-persistence/R06-08-entity-matching-and-references.md`
- `scripts/verify-entity-matching-references-persistence.mjs`

### 修改

- `README.md`
- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/catalog/mod.rs`
- `crates/persistence-postgres/src/entity_catalog.rs`
- `crates/persistence-postgres/src/player_catalog.rs`
- `docs/modular-rewrite/R06-entity-catalog-persistence/README.md`
- `package.json`
- `scripts/verify-coach-formation-persistence.mjs`
- `scripts/verify-entity-relationships.mjs`
- `scripts/verify-global-name-search.mjs`

### 移动/重命名

无。

### 删除

无。旧 R6-08 函数实现从 `entity_catalog.rs` / `player_catalog.rs` 内移除，但最终文件本身继续承担 R6-09 职责，因此不属于文件删除。

## 验证记录

- Implementation / Minimum Gate：R6-08 ownership、R6-07 retained ownership、Entity Relationships、完整 architecture、protected assets、database static baseline、rustfmt、Persistence compile/unit tests、R6-08 contract compile、Application compile 与 `git diff --check` 全部通过后形成生产提交 `89dfbb101989116b5c858dbb8dc4564c50f86f9d`。
- 首轮阶段 hard gate run `32142895608` 在 R6-08 ownership、architecture、protected assets 与 database static baseline 均通过后，因 Ubuntu Chrome zygote 未创建 `DevToolsActivePort` 导致 `verify-task-ui-screenshots.mjs` 环境失败；该轮其后 Rust/PG 门禁被正确跳过，未宣称通过。
- 最终 hard gate run `32143394153` 整体 `SUCCESS`：Windows job `95731216023` 通过 `npm run verify:frontend`、`cargo fmt --all -- --check`、workspace Clippy `-D warnings`、`cargo test --locked --workspace` 与 diff hygiene；PostgreSQL 16 job `95731215987` 通过 official inventory、R6-08/R6-07 retained ownership、Entity Relationships、完整 architecture、protected assets、database baseline/dry-run、R6-03～R6-08 全部真实 PostgreSQL contracts 与 final diff hygiene。
- 临时 R6-08 hard-gate / closeout workflow 与结果文件均已从最终 tree 清理；本节点完成标准要求的最小验证与阶段回归均已实际通过。

## 计划偏差与回退

- 计划偏差仅发生在验证执行环境：Linux 截图门禁受到 Chrome zygote 环境限制，因此阶段回归改回项目既有 Windows runner；PostgreSQL contracts 保持 Ubuntu + PostgreSQL 16 service，未改变任何生产实现或测试断言。
- 回退点：`a44e11aceccdb1c582636ff8d4c49f9d24529baa`；不得手工复制旧实现恢复。

## 未执行与剩余风险

- 用户现有 PostgreSQL 数据库真实 sample/write 验收未执行；本节点只验证隔离 PostgreSQL 16 测试库。
- Windows Full 人工交互验收未执行，继续保留到最终统一验收。
