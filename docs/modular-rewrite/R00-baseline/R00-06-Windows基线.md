# R0-06 Windows 基线：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 任务状态：IN_PROGRESS
- 起始基线提交：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`
- 原始项目基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- 实施分支：`new-A`
- 开始日期：2026-08-04
- 对应任务书：`docs/football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md`

## 2. 原始问题与本节点目标

- 在全新 Windows 环境执行本次独立的 Automated 验收基线。
- 条件允许时补充 Full 模式；Full 依赖专用 PostgreSQL 和人工 GUI 操作。
- 保存本次新生成的 Windows acceptance log、runtime log 和 acceptance report，不复用历史日志。
- 记录真实通过项、失败项、环境阻塞、警告和未执行项。
- 不修改业务源码、公共接口、依赖、数据库迁移、模型实现或用户可观察行为。

## 3. 实际变更摘要

- 已核对 Windows 根入口、PowerShell runner、机器契约、运行日志分析器和 package scripts。
- 计划使用一次性 Windows GitHub Actions 验证入口，在 `项目源码` 子目录检出 `new-A`，满足既有 runner 的目录契约。
- 先执行 `Automated`；如其被现有门禁阻断，再独立尝试 release 构建与 `RuntimeOnly`，用于取得新的 startup runtime log 和 acceptance report。
- Full 模式因用户要求数据库最后统一验证，且 GitHub runner 不具备人工 GUI 操作条件，本节点不执行。

## 4. 新增文件

| 文件路径 | 唯一职责 | 上游 | 下游 | 新增原因 |
|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/R00-06-Windows基线.md` | 保存 R0-06 的真实 Windows 执行事实、证据索引、限制和回退信息 | 任务书、Windows runner、验收契约 | R00 阶段完成记录 | 节点记录硬门禁 |
| `.github/workflows/r00-windows-baseline.yml` | 临时触发一次 Windows 基线并上传独立证据 | Draft PR、Windows runner | 本节点记录 | 当前无 Windows workflow dispatch；验证后必须删除 |

## 5. 修改文件

| 文件路径 | 修改前职责 | 本次修改 | 修改后职责 | 修改原因 | 影响范围 |
|---|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/README.md` | R00 状态与门禁索引 | 将 R0-06 设为 IN_PROGRESS | 原职责不变 | 同步节点状态 | 文档 |
| `README.md` | 项目级验证摘要 | 完成后记录实际 Windows 结果 | 原职责不变 | 根 README 同步门禁 | 文档 |

## 6. 移动或重命名文件

无。

## 7. 删除文件

- 临时 `.github/workflows/r00-windows-baseline.yml` 在取得证据后删除，不作为最终交付文件保留。

## 8. 模块、接口与数据流变化

- 新增内容只服务验证与文档，不进入生产运行链路。
- 公共 Tauri 命令、前端 DTO、Rust command 注册、数据库接口和模型边界保持不变。
- 临时 workflow 只调用仓库现有验收入口，不修改其检查强度。
- 数据库迁移、Schema、配置和正式构建入口保持不变。

## 9. 保持不变的行为

- 171 个公共命令契约保持不变。
- 46 个数据库迁移及不可变约束保持不变。
- 18 个公开模型保护文件和私有模型缺席边界保持不变。
- UI、错误语义、日志等级和运行行为不在本节点修改。
- 不删除、跳过或弱化失败检查。

## 10. 实施过程中的关键决策

| 决策 | 采用方案 | 未采用方案 | 原因 |
|---|---|---|---|
| Windows 执行环境 | GitHub-hosted Windows runner | 把 Linux 结果当 Windows 结果 | 任务书要求 Windows 基线 |
| 验收模式 | 先 Automated，必要时独立 RuntimeOnly | 伪造 Full | Full 需要数据库和人工 GUI，且数据库已获准延期 |
| 证据 | 上传本次新生成日志与报告 | 复用仓库历史 logs | 任务书明确禁止使用历史日志 |
| workflow | 一次性创建并在完成后删除 | 永久改变 CI | R0-06 只建立基线，不扩展长期 CI 范围 |

## 11. 验证记录

| 验证命令或操作 | 执行环境 | 结果 | 关键输出/报告路径 |
|---|---|---|---|
| Windows runner 静态契约检查 | GitHub Actions Windows | 待执行 | 待生成 |
| `windows-acceptance.ps1 -Mode Automated` | GitHub Actions Windows | 待执行 | 待生成 |
| release 构建补充验证 | GitHub Actions Windows | 待执行 | 待生成 |
| `windows-acceptance.ps1 -Mode RuntimeOnly` | GitHub Actions Windows | 待执行 | 待生成 |
| 独立日志与报告上传 | GitHub Actions artifact | 待执行 | 待生成 |

## 12. 未执行验证与原因

| 未执行项 | 阻塞原因 | 替代验证 | 尚未排除风险 | 用户后续操作 |
|---|---|---|---|---|
| Full 模式 | 需要专用 PostgreSQL、人工 GUI 操作；数据库按用户要求最后验证 | Automated 与 startup RuntimeOnly | 完整业务纵向链和数据库运行风险仍保留 | 最终统一验证时执行 Windows Full |
| 用户本机 Windows 10/11 | 当前无法访问用户设备 | GitHub-hosted Windows runner | 本机驱动、WebView2、权限和路径差异 | 最终实机验收复核 |

## 13. 问题修复与偏差

- 已发现根 `验收平台.bat` 传入 `-LogDirectory`，但 `scripts/windows-acceptance.ps1` 未声明该参数；本节点先记录真实基线，不在验证节点夹带修复。
- 既有 PowerShell runner 默认要求外层目录包含 `项目源码` 子目录；临时 workflow 将按该契约检出，不修改脚本。
- 其他偏差待实际执行后补充。

## 14. 遗留风险与后续边界

- R0-05 的 Rust 格式失败可能在 Automated 阶段再次阻断后续 release/runtime。
- GitHub-hosted Windows runner 不等价于用户本机 Windows 10/11。
- Full 模式、PostgreSQL、人工业务操作和最终发布验收继续保留到最终统一验证。
- 后续任务不得把未执行或失败项改写为通过。

## 15. 回退说明

- 回退基线：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`。
- 删除本节点记录，恢复阶段 README 和根 README 的 R0-06 状态。
- 临时 workflow 必须删除，Draft PR 必须关闭且不合并。
- 不涉及业务数据、数据库或生产源码回退。

## 16. 文档同步

- 根 README：完成后同步真实 Windows 结果和限制。
- 阶段 README：已进入 IN_PROGRESS，完成后同步结果与阶段门禁。
- 架构清单：无变更。
- 其他文档：本节点记录自身保存完整事实。

## 17. 完成结论

- 原始问题是否解决：实施中。
- 受影响路径是否验证：尚未完成。
- 是否存在未说明失败或警告：当前无；后续按实际结果补充。
- 是否允许下一节点进入 READY：否。
