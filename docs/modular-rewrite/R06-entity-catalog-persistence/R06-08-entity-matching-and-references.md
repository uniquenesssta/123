# R6-08 Entity Matching 与 References

## 状态

`VERIFYING`

## 基线与范围

- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`a44e11aceccdb1c582636ff8d4c49f9d24529baa`（R6-07 已关闭，R6-08 为唯一 READY 节点）。
- 本节点只重写 Entity Matching 与 References persistence；R6-09 Archive/Delete/Force Delete 与 R6-10 Global Name Search 未提前迁移。

## 实现结果

- Entity Matching 已收敛到 `adapters/catalog/entity_matching/`：resolve orchestration、stable-ID existence、external-ID lookup、name candidates、normalization 与 outcome 分责。
- References 已收敛到 `adapters/catalog/references/`：reference directory read/mapper/list、provider validation/read/write/mapper、external-ID validation/write/mapper 分责。
- `resolve.rs` 与 reference `directory/list.rs` 仅编排，不直接执行 SQL；SQL 副作用集中在明确 read/write owner。
- `entity_catalog.rs` 已移除 R6-08 matching/reference-list owner，但继续保留 R6-09 deletion/archive/reference-count owner；`player_catalog.rs` 已移除 provider/external-ID owner。
- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、配置、错误语义与用户可观察行为保持不变；无新增生产依赖。

## 契约保持

- Stable entity ID 优先于 trusted provider external ID，external ID 优先于 normalized name matching。
- Team / Player / Coach canonical name 与 alias matching、country/nationality/date-of-birth disambiguation、exact/no_match/ambiguous 状态和既有 candidate score/reason 文案保持不变。
- Reference Directory 继续复用统一 `NameSearch`，保持中文、英文、别名、重音与多关键词检索；active-only、500 上限和稳定排序不变。
- Data Provider upsert、metadata merge、active re-enable 与 External ID upsert/metadata merge 语义不变。
- R6-09 删除预检、归档及 P4/历史引用保护未提前迁移或改写。

## 当前验证

- implementation/minimum gate：R6-08 ownership verifier、R6-07 retained verifier、历史 Entity Relationships verifier、完整 architecture、rustfmt、Persistence compile/unit tests、R6-08 PostgreSQL contract 编译与 Application compile 通过后才提交本记录。
- Stage Regression、真实 PostgreSQL 16 R6-08 contract 与 clean canonical 尚未执行，因此当前只标记 `VERIFYING`，不得提前标记 `DONE`。

## 未执行与剩余风险

- 用户现有 PostgreSQL 数据库真实 sample/write 与 Windows Full 人工交互验收继续保留到最终统一验收。
- R6-07 已确认的 4 个历史 full PostgreSQL baseline 既有失败不属于本节点；R6-08 不修改对应 Lineup/P4/Review owner。
