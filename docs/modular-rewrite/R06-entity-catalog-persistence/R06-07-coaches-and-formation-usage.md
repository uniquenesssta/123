# R6-07 Coaches 与 Formation Usage

## 状态
`VERIFYING`

## 基线与范围
- Stage branch: `rewrite/r6-entity-catalog-persistence`；start baseline `7bf9b90bc235bdb397d003bdd6e61458e2e425c9`。
- Coaches 迁入 `adapters/catalog/coaches/`；Formation Directory / Usage / Resolution 迁入 `adapters/catalog/formations/`。
- R6-08 Entity Matching/References 与 R6-09 archive/delete 不提前迁移。

## 实现
- Coach directory、detail、names、team periods、mapping、validation/existence/normalization 按职责拆分；legacy `entity_catalog.rs` 删除对应 owner。
- Formation directory、usage validation/probability/window/read/save/grouping 与 resolution 拆分；旧 `formation_catalog.rs` 删除。
- 公共 Ports、Domain DTO、Schema/migrations、配置、错误语义、append-only Formation observations、smoothing、UNKNOWN fallback、resolution priority、历史引用和用户可观察行为保持不变；无新增生产依赖。
- 新增 `coach_formation_repository_contract.rs` 与 `verify-coach-formation-persistence.mjs`；历史 `verify-formation-usage.mjs` 仅迁移 owner 路径，不弱化断言。

## 验证
- implementation gate 将执行 rustfmt、official Domain inventory、R6-07 verifier、历史 Formation Usage verifier、完整 architecture、Persistence check/unit tests、R6-07 contract 编译和 Application check。
- PostgreSQL 16 contract、Windows frontend/workspace Clippy/workspace tests、保护资产与 clean canonical CI 在后续 hard gate 执行；完成前保持 `VERIFYING`。
- 用户现有 PostgreSQL 数据真实 sample/write 与 Windows Full 人工交互验收未执行，继续保留到最终统一验收。

## 回退
- 回退到 `7bf9b90bc235bdb397d003bdd6e61458e2e425c9`，不得恢复双 owner 或临时转发壳。
