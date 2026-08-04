# R0-05 前端与 Rust 基线：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 当前任务状态：DONE
- 起始基线提交：`92976da2372287a66f91d31da6d4f090734dee6c`
- 实际 CI 测试提交：`94f86842db227c5153d4334ad5176159a840e429`
- 原始项目基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- 实施分支：`new-A`
- 完成日期：2026-08-04
- Draft PR：`#1`，仅用于触发验证，已关闭且未合并
- GitHub Actions workflow run：`30910130867`
- 对应任务书：`docs/football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md`

## 2. 原始问题与本节点目标

- 按任务书建立 `npm ci`、`npm run verify:frontend` 和 Rust 工作区验证的真实重写前基线。
- 记录通过项、失败项、未执行项、环境限制和现有警告。
- 本节点的验收目标是完整暴露和冻结现有失败，不是修改业务源码使基线强行变绿。
- 本节点不修改公共接口、DTO、依赖、配置、数据库、模型实现或用户可观察行为。
- 数据库真实执行按用户明确要求推迟到最终统一验证，不属于本节点通过项。

## 3. 实际变更摘要

- 创建本节点实施与验证记录。
- 使用 `new-A` → `main` 的 Draft PR #1 触发现有 `Public Platform CI`。
- 取得 `npm ci`、前端验证和 Rust 工作区验证的真实 GitHub Actions 结果。
- 验证完成后关闭 Draft PR，未合并到 `main`。
- 没有修改任何前端、Rust、Tauri、数据库或模型生产源码。

## 4. 新增文件

| 文件路径 | 唯一职责 | 上游 | 下游 | 新增原因 |
|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/R00-05-前端与Rust基线.md` | 保存 R0-05 的实际 CI 结果、失败、警告、限制和回退事实 | 任务书、GitHub Actions、仓库验证脚本 | R0-06 和 R00 阶段完成记录 | 节点记录硬门禁 |

## 5. 修改文件

| 文件路径 | 修改前职责 | 本次修改 | 修改后职责 | 修改原因 | 影响范围 |
|---|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/R00-04-数据库基线.md` | R0-04 数据库基线事实记录 | 追加用户授权的数据库实跑延期订正 | 原职责不变 | 保留真实验证缺口并调整执行顺序 | 文档 |
| `docs/modular-rewrite/R00-baseline/README.md` | R00 状态、记录链接和阶段门禁索引 | 将 R0-05 标记为 DONE，并把 R0-06 设为唯一 READY | 原职责不变 | 同步节点状态 | 文档 |
| `README.md` | 项目级验证与模块化重写摘要 | 记录 R0-05 真实 CI 通过、失败和未执行项 | 原职责不变 | 根 README 同步门禁 | 文档 |

## 6. 移动或重命名文件

无。

## 7. 删除文件

无。

## 8. 模块、接口与数据流变化

- 新增内容仅为执行记录，不进入生产运行链路。
- 公共 Tauri 命令、前端参数与返回 DTO、Rust command 注册保持不变。
- `package.json`、`package-lock.json`、Cargo manifests、`Cargo.lock`、Rust toolchain 和 GitHub Actions 工作流均未修改。
- 数据库公共接口、0001–0046 迁移及不可变约束继续以 `main` 基线保持。
- 模型保护资产和外部 ModelProvider 边界没有变化。

## 9. 保持不变的行为

- UI、路由、页面状态、错误语义、日志等级和用户可观察行为保持不变。
- 前端仍由 `npm run verify:frontend` 组织静态契约、TypeScript、构建和截图验证。
- Rust 仍使用锁定依赖、Rust 1.88.0、rustfmt、Clippy 和 workspace tests。
- 171 个命令契约、46 个数据库迁移和 18 个模型保护文件没有变化。
- 本节点没有修复、跳过、删除或弱化任何失败检查。

## 10. GitHub Actions 执行环境

- Workflow：`Public Platform CI`
- Workflow run：`30910130867`
- 测试提交：`94f86842db227c5153d4334ad5176159a840e429`
- Runner：GitHub-hosted Ubuntu 24.04
- Node：`22.23.1`
- npm：`10.9.8`
- Rust：`1.88.0`
- 前端 job：`91994529694`
- Rust job：`91994529785`
- PR：Draft PR #1，验证完成后关闭，`merged=false`

## 11. 验证记录

| 验证命令或步骤 | 执行环境 | 结果 | 真实结论 |
|---|---|---|---|
| Checkout | GitHub Actions Ubuntu | 通过 | `new-A` 测试提交成功检出 |
| Node 22 setup | GitHub Actions Ubuntu | 通过 | Node `22.23.1`、npm `10.9.8` |
| `npm ci` | frontend job | 通过 | 安装 20 个包，审计 21 个包 |
| npm audit | `npm ci` 输出 | 警告 | 存在 1 个 moderate vulnerability；本节点不修改依赖 |
| `npm run verify:frontend` | frontend job | 失败 | 失败前的公开模型边界、命令契约及多项静态检查已通过；最终在 UI 截图浏览器启动阶段失败 |
| Chromium 截图启动 | `verify-task-ui-screenshots.mjs` | 失败 | Chromium 未开放调试端口；日志显示 zygote 初始化终止，未进入截图像素或业务断言比较 |
| Tauri Linux 系统依赖安装 | rust job | 通过 | WebKit、AppIndicator、librsvg、patchelf 安装成功 |
| Rust 1.88.0 + rustfmt + clippy | rust job | 通过 | 工具链安装成功 |
| `node scripts/verify-public-model-boundary.mjs` | rust job | 通过 | 公开模型边界通过 |
| `node scripts/verify-cargo-lock.mjs` | rust job | 通过 | Cargo.lock 门禁通过 |
| `cargo metadata --locked --format-version 1` | rust job | 通过 | 锁定依赖元数据可解析 |
| `cargo fmt --all -- --check` | rust job | 失败 | 当前 Rust 工作区存在格式差异；可用连接器未返回完整 diff，不能虚构具体文件 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | rust job | 未执行 | 因前一步 `cargo fmt --check` 失败，workflow fail-fast 跳过 |
| `cargo test --locked --workspace` | rust job | 未执行 | 同上 |
| `npm run verify:rust` | 精确 package script | 未直接执行 | Actions 执行了主要 Rust 子步骤，但没有按 package script 单入口调用 |
| Draft PR 清理 | GitHub | 通过 | PR #1 已关闭且未合并 |

## 12. `npm run verify:rust` 与 Actions Rust job 差异

`npm run verify:rust` 的既有脚本还包含：

- `node scripts/verify-cargo-lock-sync.mjs`
- `node scripts/prepare-cargo-target.mjs`

本次 Actions Rust job没有直接调用上述 npm script，而是另外执行了：

- `node scripts/verify-public-model-boundary.mjs`
- `cargo metadata --locked --format-version 1`

因此本节点只能声明 Actions 中实际执行的子步骤结果，不能声明完整 `npm run verify:rust` 已通过或已完整执行。

## 13. 失败与警告分类

| 项目 | 分类 | 是否由 R0-05 文档变更引入 | 后续处理边界 |
|---|---|---|---|
| Chromium 未开放调试端口 | GitHub Ubuntu 浏览器运行环境失败 | 否；测试提交未修改截图工具或业务源码 | 后续在 Windows/R0-06 或可启动 Chromium 的环境复核 |
| `cargo fmt --check` 失败 | 现有 Rust 源码格式基线失败 | 否；R0-05 未修改 Rust 源码 | 后续独立修复任务处理，不能在基线节点夹带源码格式化 |
| Clippy 未执行 | 上游格式检查失败导致跳过 | 否 | 修复格式后重新执行 |
| workspace tests 未执行 | 上游格式检查失败导致跳过 | 否 | 修复格式后重新执行 |
| 1 个 moderate npm vulnerability | 现有依赖审计警告 | 否；依赖和锁文件未修改 | 后续依赖治理任务评估，不在本节点升级依赖 |
| Actions checkout/setup-node Node 运行时弃用提示 | CI action 平台警告 | 否 | 后续 CI 维护任务处理 |

## 14. 未执行验证与原因

| 未执行项 | 原因 | 已完成替代验证 | 尚未排除风险 | 后续操作 |
|---|---|---|---|---|
| 本地完整工作树 `npm ci` 和验证 | 当前执行容器没有完整 Git 工作树 | GitHub Actions 在干净 Ubuntu runner 执行 | 用户本机和 Windows 环境差异未覆盖 | R0-06 执行 Windows 基线 |
| 精确 `npm run verify:rust` | 现有 CI 以分步 Cargo 命令执行 | Cargo lock、metadata、fmt 已实际执行 | lock-sync、target preparation、Clippy、tests 未形成完整结果 | 修复格式后执行原始 npm script |
| Clippy | `cargo fmt --check` 失败 | 工具链和 metadata 已通过 | 编译警告和 lint 风险未排除 | 后续重新执行 |
| Rust workspace tests | `cargo fmt --check` 失败 | 无可替代运行结果 | 单元和非数据库集成回归未排除 | 后续重新执行 |
| PostgreSQL ignored tests | 用户要求数据库最后统一验证 | R0-04 静态迁移和测试契约已冻结 | 数据库运行风险仍存在 | 最终统一验证执行 |
| Tauri 应用启动 | 不属于本次 Linux CI job | 依赖安装成功 | 桌面运行链路未确认 | R0-06 Windows 实机验证 |

## 15. 关键决策与偏差

- 采用 Draft PR 触发现有 Actions，而不修改工作流或向 `main` 推送。
- R0-05 只冻结真实基线；不为追求绿色状态格式化 Rust 源码、替换截图工具、修改依赖或弱化检查。
- 前端验证失败发生于浏览器启动，不能被描述为 UI 断言失败，也不能被描述为前端全部通过。
- Rust 验证失败发生于格式检查，Clippy 和 tests 没有运行，不能依据工具链安装成功推断其结果。
- Draft PR 完成验证后已关闭且未合并。

## 16. 回退说明

- 回退基线：`92976da2372287a66f91d31da6d4f090734dee6c`。
- 删除本节点记录，并恢复根 README、R00 阶段索引和 R0-04 延期订正前状态。
- Draft PR 已关闭，不需要回退合并提交。
- 不涉及业务数据、数据库、生产源码或依赖回退。

## 17. 当前结论

- `npm ci`：通过。
- `npm run verify:frontend`：失败，阻塞点为 GitHub Ubuntu Chromium 调试端口未建立。
- Rust 工具链、公开模型边界、Cargo.lock、locked metadata：通过。
- `cargo fmt --check`：失败。
- Clippy 和 workspace tests：未执行。
- 精确 `npm run verify:rust`：未直接执行。
- R0-05 是否完整记录当前失败、警告和环境限制：是。
- 是否修改业务源码以掩盖失败：否。
- R0-05 是否完成其“基线记录”目标：是。
- 是否允许下一节点进入 READY：是，`R0-06 Windows 基线` 为唯一下一任务。
