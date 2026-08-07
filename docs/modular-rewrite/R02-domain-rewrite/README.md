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
- 真实 PostgreSQL、Windows Full、用户本机 Windows 实机验收：保留到最终统一验收；节点内已完成的额外实机验证可作为补充证据记录

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
| R2-01 | 领域类型与契约清单 | DONE | [`R02-01-领域类型-and-契约清单.md`](R02-01-领域类型-and-契约清单.md) | workflow run `31078483578` 通过 |
| R2-02 | Competition 与 Routing | DONE | [`R02-02-competition-and-routing.md`](R02-02-competition-and-routing.md) | workflow run `31088698579`、job `92574240109` 通过 |
| R2-03 | Team / Player / Coach / Formation | DONE | [`R02-03-team-player-coach-and-formation.md`](R02-03-team-player-coach-and-formation.md) | workflow run `31110013068`、job `92645025258` 通过 |
| R2-04 | Lineup 与 Match | DONE | [`R02-04-lineup-and-match.md`](R02-04-lineup-and-match.md) | 实施 run `31151412918`、正式 Windows Automated run `31153982572` 通过 |
| R2-05 | Prediction 与 Research 外围领域 | DONE | [`R02-05-prediction-and-research-外围领域.md`](R02-05-prediction-and-research-外围领域.md) | 正式 Windows Automated run `31171082098`、job `92842834091` 通过 |
| R2-06 | Review 与 Postmatch | DONE | [`R02-06-review-and-postmatch.md`](R02-06-review-and-postmatch.md) | staged 全量通过；Windows Automated run `31200190104` 与用户原库实机连接通过 |
| R2-07 | Analytics / Exchange / AI / Release | READY | `R02-07-analytics-exchange-ai-and-release.md` | R2-06 已关闭，可开始 |
| R2-08 | Domain 根出口收敛 | BLOCKED | `R02-08-domain-根出口收敛.md` | 等待 R2-07 |

## R2-01 当前结果

- 已生成 `architecture/domain-type-inventory.json`，登记 365 个公共兼容类型。
- 已扫描 20 个 Domain 来源文件和 139 个 Rust 文件调用范围。
- 已识别 299 个被 PostgreSQL 适配器引用的领域类型。
- 已为每个类型登记当前路径、目标模块、目标任务、SerDe 契约、数据库映射、Domain 调用方、外部调用方和公共兼容级别。
- 目标任务分布：R2-02 共 22 个，R2-03 共 72 个，R2-04 共 19 个，R2-05 共 75 个，R2-06 共 59 个，R2-07 共 118 个。
- 已新增历史 JSON 往返、默认值、optional 语义和枚举线值契约测试，共 6 项，全部通过。
- 本节点未移动或修改任何 `crates/domain/src` 生产类型。
- 生成与全量门禁 run `31077537198`、job `92538743873` 已通过；正式 Windows Automated run `31078483578`、job `92541654912` 在最终实施提交 `2a6b9ea96a88168d6a751ebf48c2030512edaf24` 上通过，artifact `8959079579` 的 SHA-256 为 `6b75cc3abe2067472476cd0e7811b9fd9ee6f689f17cbc0eb346030775d0c9e2`。

## 阶段出口

R2-01 至 R2-08 全部 `DONE` 后创建 `R02-stage-completion.md`，并实际完成阶段规定的 Domain、Serde、workspace、模型保护及最终延期验收记录。当前不得创建阶段完成记录。

## 当前阶段状态

`R2-06 DONE`；`R2-07 READY`。Review 48 个类型与 Postmatch 11 个类型已从 5 个根级职责混合文件迁移到 `review/` 与 `postmatch/` 职责目录；staged run `31173824393`、job `92851309157` 已通过格式化、类型清单、Serde/模块路径、架构、保护资产、完整 frontend 与完整 Rust 门禁。后续数据库兼容链在不清库、不覆盖历史不可变资产、不放宽指纹保护的前提下完成修复；Windows Automated run `31200190104` 通过，用户在 Windows 上使用原有 PostgreSQL 数据库完成连接与运行日志复核，R2-06 正式关闭。

## R2-02 当前结果

- Competition 14 个类型已按目录职责拆分为 kind、catalog、profile、season、stage、round、rule_package。
- Routing 8 个类型已按 identity、rules、binding、context、route 拆分。
- 根级公共类型名继续通过兼容 re-export 保持不变；glob 根出口债务仍由 R2-08 统一退出。
- `architecture/domain-migration-progress.json` 已将 R2-02 登记为完成迁移范围，清单门禁拒绝类型回退到目标目录之外。
- 本节点未修改数据库迁移、SQL Row、Application、Tauri DTO、模型保护资产或生产依赖。

## R2-03 当前结果

- Team 17、Player 21、Coach 9、Formation 8、Shared 17，共 72 个类型已进入职责目录。
- 根级公共类型名通过兼容 re-export 保持不变；新增业务语义模块路径与类型身份测试。
- `architecture/domain-migration-progress.json` 已登记 R2-03，清单门禁拒绝 72 个类型回退到目标目录之外。
- 实施 workflow run `31100515822` 的专项、架构、前端和 Rust 门禁已通过。
- 正式 Windows Automated run `31110013068`、job `92645025258` 在同源码树触发提交 `594940dca4c57aabfebdd768755ec27006ecaeb5` 上通过；artifact `8972168972` 大小 `14119500` 字节，SHA-256 为 `6ef2e064638cd17b214c66bbdea5ed752a08a1f0dc32002940e0f97d094cae5f`，运行报告为 PASS，7 条记录、3 个完成操作。
- 本节点未修改数据库迁移、SQL Row、Application、Tauri DTO、模型保护资产或生产依赖。

## R2-04 当前结果

- Lineup 16 个类型已按 kind、player、snapshot、preset、chain 拆分。
- Match 3 个类型已按 status、catalog 拆分。
- 旧 `crates/domain/src/lineup_chain.rs` 已删除，正式阵容快照常量与三类链路契约迁入 `lineup/chain.rs`。
- 根级公共类型名继续通过兼容 re-export 保持不变；glob 根出口债务仍由 R2-08 统一退出。
- `architecture/domain-migration-progress.json` 已登记 R2-04，清单门禁拒绝 19 个类型回退到目标目录之外。
- 实施 workflow run `31151412918` 已通过专项、架构、前端、Rust 及最终树提交/push 校验。
- 正式 Windows Automated run `31153982572`、job `92789397631` 已在最终实施提交 `0aafe42d7ed08f8e78d71d44ccb6f8f58c425999` 上通过；artifact `8984980586` 大小 `14118155` 字节，SHA-256 为 `1e7224f4e7f713b0339e97fd114fa6dea2c0b2ecc9400789613fe872d660938c`。
- 本节点未修改数据库迁移、SQL Row、Application、Tauri DTO、模型保护资产或生产依赖。

## R2-05 当前结果

- Prediction 48 个类型已进入 `prediction/` 职责目录；Research 27 个类型已进入 `research/` 职责目录。
- 75 个类型均保留根级公共兼容路径，并新增 `prediction::*` / `research::*` 业务语义路径身份门禁。
- Prediction 独立提交为 `2cd685b8057a1bce2f75e4c7f5b56aed1bf3d142`；首次 run `31158780693` 暴露并停止于两个未使用 import，恢复 run `31159821513` 已在不放宽门禁的前提下完成 Research 与全量回归。
- 正式 Windows Automated run `31171082098`、job `92842834091` 已通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。
- R2-05 状态为 `DONE`，R2-06 已开放。

## R2-06 当前结果

- Review 48 个类型与 Postmatch 11 个类型已进入业务语义职责目录，并保留根级公共兼容类型路径。
- 已删除 `review.rs`、`match_event.rs`、`match_review_package.rs`、`match_review_workflow.rs`、`postmatch.rs` 5 个旧职责混合文件。
- 新增 59 个类型的 `review::*` / `postmatch::*` 模块身份契约，迁移进度与目标模块策略同步登记 R2-06。
- staged run `31173619041` 在类型清单策略缺少新目录时按硬门禁停止；仅补齐目录策略后，recovery run `31173824393`、job `92851309157` 已完整通过格式化、类型清单、Serde、架构、保护资产、frontend 与 Rust 全量门禁。
- Windows 实机 smoke test 发现并修复断库状态比赛页对未渲染 `new-match-competition` 控件的错误初始化。
- 原有 PostgreSQL 数据库验证暴露并修复已知历史 migration checksum、不可变 engine artifact 字段映射、内置规则包及 P4 Schema/Prompt 同版本不同内容冲突；修复均保留历史数据和不可变资产，不清库、不覆盖旧版本、不削弱 fail-closed 与内容指纹保护。
- Windows Automated acceptance run `31200190104` 通过；用户随后使用原有 PostgreSQL 数据库连接成功，最终 runtime 日志无连接错误，启动与工作区读取正常。
- R2-06 状态为 `DONE`，R2-07 已开放。
