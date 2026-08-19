# R6-10 Global Name Search

## 状态

`DONE`

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
- stage verification run `32249902897` 已通过：Windows job `96058363125` 完成 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 diff hygiene；PostgreSQL 16 job `96058363037` 完成 Global Search / Domain inventory / architecture / protected assets / database baseline 门禁及 R6-01～R6-10 retained contracts。

## 回退点

- 回退到源码基线 `e79f1becb1595b03f5d1ba4686363dd62fbac9e2`；不保留双实现。

## 完成结论

- 生产 owner-switch 提交：`fbe013b96f38d3c61e7b536e1ef67a89109c57c4`。统一名称搜索唯一生产 owner 为 `adapters/catalog/global_search/`，旧 `name_search.rs` 已删除，无双实现或转发壳。
- implementation/minimum gate `32249610140` 与 stage verification run `32249902897` 均已实际通过，R6-10 据此关闭为 `DONE`。
- 本节点未修改 UI。此前人工验收中，球队目录/详情和球队名称/档案写入回读由用户明确确认通过；球员项仅收到“应该OK了”，不扩大记为完整 Windows Full 人工验收。
