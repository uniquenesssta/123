# R0-05 前端与 Rust 基线：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 当前任务状态：IN_PROGRESS
- 起始基线提交：`92976da2372287a66f91d31da6d4f090734dee6c`
- 原始项目基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- 实施分支：`new-A`
- 开始日期：2026-08-04
- 对应任务书：`docs/football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md`

## 2. 原始问题与本节点目标

- 按任务书执行 `npm ci`、`npm run verify:frontend` 和 `npm run verify:rust`。
- 记录真实通过项、失败项、环境阻塞和现有警告。
- 本节点只建立重写前的前端与 Rust 可重复验证基线，不修改业务源码、公共接口、依赖、配置、数据库或模型实现。
- 数据库真实执行按用户当前明确要求推迟到最终统一验证；R0-05 不借此宣称数据库测试通过。

## 3. 当前验证策略

- `new-A` 当前没有本地完整工作树，当前容器无法直接运行仓库级 npm/Cargo 命令。
- 建立 `new-A` → `main` 的 Draft PR，触发现有 `.github/workflows/ci.yml`。
- Actions `frontend` job 直接执行 `npm ci` 和 `npm run verify:frontend`。
- Actions `rust` job 安装 Rust 1.88.0 与 Tauri Linux 系统包，并执行 Cargo.lock、metadata、format、Clippy 和 workspace tests。
- `npm run verify:rust` 与 Actions Rust job 的差异必须单独核对，不能把“等价子命令”静默写成“原脚本已直接运行”。

## 4. 新增文件

| 文件路径 | 唯一职责 | 上游 | 下游 | 新增原因 |
|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/R00-05-前端与Rust基线.md` | 保存 R0-05 的真实执行结果、失败、警告、限制和回退事实 | 任务书、GitHub Actions、仓库验证脚本 | R0-06 和 R00 阶段完成记录 | 节点记录硬门禁 |

## 5. 修改文件

| 文件路径 | 修改前职责 | 本次修改 | 修改后职责 | 修改原因 | 影响范围 |
|---|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/README.md` | R00 状态和阶段门禁索引 | 将 R0-04 记为静态完成、R0-05 记为 IN_PROGRESS | 原职责不变 | 同步用户调整后的执行顺序 | 文档 |
| `README.md` | 项目级验证与模块化重写摘要 | 记录数据库延期和 R0-05 验证入口 | 原职责不变 | 根 README 同步门禁 | 文档 |

## 6. 移动或重命名文件

无。

## 7. 删除文件

无。

## 8. 需保持不变的接口与行为

- 前端公共 Tauri 命令、参数和返回 DTO 继续受 R0-03 的 171 命令契约保护。
- Rust command 注册、application/domain/persistence/model 调用语义保持不变。
- `package.json`、`package-lock.json`、Cargo manifests、`Cargo.lock` 和 Rust toolchain 不在本节点修改。
- 数据库公共接口和 0001–0046 迁移继续以 `main` 基线保持。
- UI、配置、错误语义、日志等级和用户可观察行为不得改变。

## 9. 计划执行的验证

| 验证 | 目标环境 | 当前状态 |
|---|---|---|
| `npm ci` | GitHub Actions Ubuntu | 待运行 |
| `npm run verify:frontend` | GitHub Actions Ubuntu | 待运行 |
| Rust toolchain / Tauri Linux dependencies | GitHub Actions Ubuntu | 待运行 |
| Cargo.lock 与 metadata | GitHub Actions Ubuntu | 待运行 |
| `cargo fmt --all -- --check` | GitHub Actions Ubuntu | 待运行 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | GitHub Actions Ubuntu | 待运行 |
| `cargo test --locked --workspace` | GitHub Actions Ubuntu | 待运行 |
| `npm run verify:rust` 精确脚本差异核对 | GitHub 文件扫描及可用补充验证 | 待运行 |

## 10. 当前阻塞和风险

- 当前本地容器没有完整仓库工作树，不能直接在本机执行三个任务书命令。
- Draft PR 尚未创建，Actions 尚未产生本节点结果。
- Actions Rust job 当前不是通过 `npm run verify:rust` 单入口调用，必须核对其子命令覆盖范围。
- PostgreSQL ignored tests 不属于本节点结果，继续延期到最终统一验证。

## 11. 回退说明

- 回退基线：`92976da2372287a66f91d31da6d4f090734dee6c`。
- 删除本节点记录，并恢复阶段 README 与根 README 的 R0-05 状态。
- Draft PR 如仅用于验证，可关闭而不合并。
- 不涉及业务数据、数据库或生产源码回退。

## 12. 当前结论

- R0-05 已进入实施：是。
- 前端或 Rust 验证是否已有结果：否。
- 是否允许标记 DONE：否。
- 下一步：同步阶段状态并创建 Draft PR 触发 GitHub Actions。
