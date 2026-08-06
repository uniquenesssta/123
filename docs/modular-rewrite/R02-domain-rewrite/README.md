# R02 Domain 完整重写：执行记录索引

## 阶段状态

`IN_PROGRESS`

R1 已完成并关闭。R2 按业务语义拆分 `crates/domain`，保持 Serde、数据库映射、公共导出和模型边界兼容；每个节点必须独立实施、验证和回退。

## 前置基线

- R1 阶段完成记录：[`../R01-architecture-composition/R01-stage-completion.md`](../R01-architecture-composition/R01-stage-completion.md)
- R2 开始前已验证提交：`274d689b9c4a3d3ce83c7006878a5508ca3f31d6`
- R2-01 生成清单基线提交：`9e8d527dd26df3e36f00b7730da320acf216b7bc`
- 目标平台：Windows
- Linux：不属于目标平台、交付或阶段门禁
- 真实 PostgreSQL、Windows Full、用户本机 Windows 实机验收：保留到最终统一验收

## 阶段范围

- `competition/`、`routing/`
- `team/`、`player/`、`coach/`、`formation/`
- `lineup/`、`match_record/`
- `prediction/`、`research/`
- `review/`、`postmatch/`
- `analytics/`、`exchange/`、`ai_workspace/`、`release/`
- `shared/`
- Domain 根出口显式收敛

## 禁止范围

- 不修改 Application 用例、SQL Row、Tauri DTO 或模型内部公式与类型。
- 不修改模型保护区、模型参数、Profile、Schema、fixture 或 Golden Master。
- 不改变 Serde 字段名、枚举表示、默认值、optional 语义、数据库历史 JSON 或公共行为。
- 不新增生产依赖。
- 不使用 glob re-export 掩盖无归属类型；该历史债务只在 R2-08 统一退出。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 当前门禁 |
|---|---|---|---|---|
| R2-01 | 领域类型与契约清单 | VERIFYING | [`R02-01-领域类型-and-契约清单.md`](R02-01-领域类型-and-契约清单.md) | 清单、Serde、架构、前端、Rust 已通过；正式 Windows Automated 待确认 |
| R2-02 | Competition 与 Routing | BLOCKED | `R02-02-competition-and-routing.md` | 等待 R2-01 `DONE` |
| R2-03 | Team / Player / Coach / Formation | BLOCKED | `R02-03-team-player-coach-and-formation.md` | 等待前置节点 |
| R2-04 | Lineup 与 Match | BLOCKED | `R02-04-lineup-and-match.md` | 等待前置节点 |
| R2-05 | Prediction 与 Research 外围领域 | BLOCKED | `R02-05-prediction-and-research-外围领域.md` | 等待前置节点 |
| R2-06 | Review 与 Postmatch | BLOCKED | `R02-06-review-and-postmatch.md` | 等待前置节点 |
| R2-07 | Analytics / Exchange / AI / Release | BLOCKED | `R02-07-analytics-exchange-ai-and-release.md` | 等待前置节点 |
| R2-08 | Domain 根出口收敛 | BLOCKED | `R02-08-domain-根出口收敛.md` | 等待前置节点 |

## R2-01 当前结果

- 已生成 `architecture/domain-type-inventory.json`，登记 365 个公共兼容类型。
- 已扫描 20 个 Domain 来源文件和 139 个 Rust 文件调用范围。
- 已识别 299 个被 PostgreSQL 适配器引用的领域类型。
- 已为每个类型登记当前路径、目标模块、目标任务、SerDe 契约、数据库映射、Domain 调用方、外部调用方和公共兼容级别。
- 目标任务分布：R2-02 共 22 个，R2-03 共 72 个，R2-04 共 19 个，R2-05 共 75 个，R2-06 共 59 个，R2-07 共 118 个。
- 已新增历史 JSON 往返、默认值、optional 语义和枚举线值契约测试，共 6 项，全部通过。
- 本节点未移动或修改任何 `crates/domain/src` 生产类型。
- 生成与全量门禁 run `31077537198`、job `92538743873` 已通过；正式 Windows Automated 需在最终实施提交上确认。

## 阶段出口

R2-01 至 R2-08 全部 `DONE` 后创建 `R02-stage-completion.md`，并实际完成阶段规定的 Domain、Serde、workspace、模型保护及最终延期验收记录。当前不得创建阶段完成记录。

## 当前阶段状态

`R2-01 VERIFYING`；正式 Windows Automated 通过前不开放 R2-02，也不开始任何领域类型迁移。
