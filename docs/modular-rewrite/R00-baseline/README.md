# R00 基线冻结与可重复验收：执行记录索引

## 阶段范围

冻结模块化重写开始前的远端提交、分支、保护资产、公共命令、数据库、前后端与 Windows 验收基线。R00 不实施业务重写，不移动业务源码，不修改模型保护区。

## 当前基线

- 基线分支：`main`
- 起始基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- R0-02 起始提交：`beb00b6f723d75703e9235967c4f49f985d41c4e`
- R0-03 起始提交：`83a7d0bddd83894b4f551725c40b50705470e731`
- R0-04 起始提交：`8cddf452e01770a042b671ed8c11bc29afd2f1b1`
- R0-05 起始提交：`92976da2372287a66f91d31da6d4f090734dee6c`
- R0-05 CI 测试提交：`94f86842db227c5153d4334ad5176159a840e429`
- R0-06 起始提交：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`
- R0-06 Windows 证据提交：`2001154dff611e6326f2182d8e4a0b8aa35ca98b`
- R0-06.1 起始提交：`b165bf5c4a658b63edb698e40b49efebf567b334`
- R0-06.1 Windows 验证提交：`278f1093b90c5c13aed7108535329bd1dc528441`
- 实施分支：`new-A`
- 已完成节点：`R0-01`、`R0-02`、`R0-03`、`R0-04（数据库实跑延期）`、`R0-05`、`R0-06`、`R0-06.1`

## 保护资产

- `architecture/protected-assets.json` 冻结 18 个公开模型边界及校验文件。
- 聚合 SHA-256：`d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- 真实 P4/P7 源码、参数、Profile、专用 Schema、fixture、Golden Master 和私有研究资产继续禁止进入公开仓库。

## 命令契约

- `architecture/command-contract.json` 冻结 171 个公共命令、注册顺序和 15 个 Rust 命令模块。
- 命令集合 SHA-256：`dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669`。
- 命令定义映射 SHA-256：`eb908c4ae31ec8d49e65a1b336c1105cb550e755fcdbae40f4943f99433cc9f5`。

## 数据库基线

- `architecture/database-baseline.json` 冻结 `0001`–`0046` 共 46 个连续迁移。
- 迁移聚合 SHA-256：`d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。
- 数据库公共接口、迁移集合和不可变约束继续以 `main` 基线保持不变。
- 用户于 2026-08-04 明确要求将 PostgreSQL 迁移幂等和集成测试推迟到最终统一验证；未执行项不得描述为通过。

## 前端与 Rust 基线

- R0-05 workflow run `30910130867`：`npm ci` 通过，存在 1 个 moderate npm audit 警告。
- Linux `npm run verify:frontend` 失败于 Chromium 未开放调试端口。
- Rust 工具链、Tauri Linux 依赖、公开模型边界、Cargo.lock 与 locked metadata 通过。
- `cargo fmt --all -- --check` 失败；Clippy 和 workspace tests 因 fail-fast 未执行。

## Windows 基线

- R0-06 证据：workflow run `30912862564`、job `92003616257`、artifact `8894874465`。
- Windows release 构建与 RuntimeOnly startup 通过；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06 精确 Automated 暴露目录联接下依赖安装假通过及 `.cmd` 子进程调用问题。
- Windows Full 未执行：数据库按用户要求最后验证，人工 GUI 链不适合 hosted runner。

## R0-06.1 Windows Node 调用链

- 新增 `scripts/process/execution-context.mjs`，统一直接执行身份的规范物理路径判断。
- 新增 `scripts/process/node-package-cli.mjs`，通过当前 Node 执行本地包 JavaScript CLI，不再直接启动 `.cmd` 包装器。
- 新增 `scripts/verify-node-process-compatibility.mjs`，覆盖目录联接、参数传递、非零退出码和缺失入口拒绝。
- `scripts/ensure-node-dependencies.mjs` 已修复联接路径下未执行却返回成功的问题。
- `scripts/verify-frontend.mjs` 已通过 Node 直接执行 TypeScript 与 Vite CLI。
- Windows workflow run `30919764753` 中，完整 `npm run verify:frontend` 通过；精确 Automated 前端阶段通过并继续进入 Rust 阶段。
- Automated 总体仍在既有 `cargo fmt --check` 处退出 1，不描述为总体通过。
- 证据 artifact `8896665715`，SHA-256：`95e567f917082670390860d2671699a59a7c65018cbbb875f5f37ea05288a16d`。
- Draft PR #2 已关闭且未合并；临时 workflow 已删除。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R0-01 | 工作区状态与基线提交确认 | DONE | [记录](./R00-01-工作区状态与基线提交确认.md) | 远端基线与分支起点核对 | 已纳入出口判断 |
| R0-02 | 模型保护资产指纹 | DONE | [记录](./R00-02-模型保护资产指纹.md) | 指纹、篡改和禁止资产负向验证 | 已纳入出口判断 |
| R0-03 | 命令契约冻结 | DONE | [记录](./R00-03-命令契约冻结.md) | 171 命令三方扫描及负向验证 | 已纳入出口判断 |
| R0-04 | 数据库基线 | DONE | [记录](./R00-04-数据库基线.md) | 46 个迁移静态门禁；真实 PostgreSQL 延期 | 出口缺口保留 |
| R0-05 | 前端与 Rust 基线 | DONE | [记录](./R00-05-前端与Rust基线.md) | Linux 前端和 Rust 现有失败已冻结 | 出口缺口保留 |
| R0-06 | Windows 基线 | DONE | [记录](./R00-06-Windows基线.md) | release 与 RuntimeOnly 通过；精确 Automated 失败已冻结 | 出口缺口保留 |
| R0-06.1 | Windows Node 调用链修复 | DONE | [记录](./R00-06.1-Windows-Node调用链修复.md) | Windows frontend 通过；Automated 到达 Rust 阶段 | Node 调用缺口关闭 |
| R0-06.2 | Windows 路径契约修复 | READY | 待创建 | 待执行 | 待执行 |

## 本阶段累计变化

- 新增模型保护、命令契约和数据库基线清单及只读校验器。
- 新增 R0-01 至 R0-06.1 节点记录。
- 新增三个职责单一的 Windows Node 执行/验证模块。
- 修改仅限验证工具和文档；未修改前端或 Rust 业务源码、依赖、锁文件、公共接口、迁移 SQL、模型实现或用户可观察行为。
- 临时验证 workflow 均已删除，Draft PR #1/#2 均已关闭且未合并。

## 未解决问题

- GitHub Ubuntu Chromium 截图进程未开放调试端口。
- Rust `cargo fmt --check` 失败；Clippy、workspace tests 和完整 `npm run verify:rust` 尚未通过。
- PostgreSQL 迁移幂等、不可变触发器和 18 个集成测试按用户要求延期。
- `验收平台.bat` 传入 PowerShell runner 未声明的 `-LogDirectory`。
- Cargo release 输出目录与 Windows release 查找器路径不一致。
- Windows Full 和用户本机 Windows 10/11 实机验收尚未执行。
- 1 个 moderate npm vulnerability、Vite 大 chunk 警告和 2 个 Rust dead-code 警告未处理。
- 用户设备上的未提交或未跟踪文件不可见；远端操作未覆盖这些本地内容。

## 阶段门禁状态

**BLOCKED。** Windows Node/frontend 调用链缺口已关闭，但 Windows 路径契约、Rust 完整验证、数据库实跑和 Windows Full 尚未完成，因此未创建 `R00-stage-completion.md`。

## 下一 READY 任务

`R0-06.2 Windows 路径契约修复` 是唯一 READY 任务。不得提前进入 R1。
