# R6-10 Global Name Search

## 状态

`VERIFYING`

## 基线与范围

- 源码基线：`e79f1becb1595b03f5d1ba4686363dd62fbac9e2`；继续使用唯一分支 `rewrite/r6-entity-catalog-persistence`。
- 唯一目标 owner：`crates/persistence-postgres/src/adapters/catalog/global_search/`。
- 仅迁移查询解析、名称归一化和 PostgreSQL 名称谓词；球队、球员、教练、球队 selector 与 Entity Reference 调用点只切换内部 owner。
- `football.global-name-search.v1`、公共 Port/DTO、Schema、0001–0046 migrations、配置、错误语义、UI 与模型保护资产均不改变。

## 实施结果

- `query.rs`：查询解析与多关键词 token。
- `normalization.rs`：大小写、拉丁重音、标点/空白与 compact 规则。
- `predicate.rs`：SQL QueryBuilder 名称谓词。
- `mod.rs`：只做模块声明和显式导出。
- 旧 `crates/persistence-postgres/src/name_search.rs` 删除，无兼容转发壳。
- 现有 7 个 `NameSearch::parse` 接入点保持 contains、多关键词 AND、正式名/本地化名/别名、中文/英文部分匹配、标点/空白无关与拉丁重音折叠语义。
- 新增 PostgreSQL contract 覆盖 Team Directory + 分页、team selector、Player Directory、Coach Directory 和 team/player/coach Entity Reference；不新增拼音或编辑距离纠错。

## 验证状态

- implementation/minimum gate run `32249610140` 已通过：Global Search owner/contract、Persistence check、5 个 focused unit tests、模型保护、database baseline、Domain inventory、PostgreSQL 16 focused contract 与 diff scope 全部 PASS。
- production owner-switch 后继续运行完整 Windows stage regression 与 R6-01～R6-10 retained PostgreSQL 16 contracts。

## 回退点

- 回退到源码基线 `e79f1becb1595b03f5d1ba4686363dd62fbac9e2`；不保留双实现。
