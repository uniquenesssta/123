# R01 架构契约与空壳组合根：执行记录索引

## 阶段状态

`IN_PROGRESS`

R00 已按 Windows-only 目标范围完成。R1 只建立可机器验证的架构边界和浏览器、Tauri、Application 组合根空壳，不迁移具体业务实现。

## 前置基线

- R00 阶段完成记录：[`../R00-baseline/R00-stage-completion.md`](../R00-baseline/R00-stage-completion.md)
- R00 完成前分支提交：`ab812dc2cef126cc46fe9914a63815e047739d75`
- `new-B` 分支起点：`new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496`
- 目标平台：Windows
- Linux：不属于目标平台、交付或阶段门禁
- PostgreSQL 实跑、Windows Full、用户本机 Windows 实机验收：保留到最终统一验收

## 阶段范围

- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `scripts/architecture/` 边界验证脚本
- 浏览器 bootstrap 空壳
- Tauri bootstrap/state/command registry 空壳
- Application composition/service/model registry 空壳

## 禁止范围

- 不迁移具体 Feature、Domain 或 Repository 实现。
- 不修改模型保护区。
- 不改变 Tauri 命令名称、参数、返回类型、启动顺序、窗口行为或默认模型注册语义。
- 不新增生产依赖。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R1-01 | 模块边界契约 | DONE | [`R01-01-模块边界契约.md`](R01-01-模块边界契约.md) | JSON 解析、契约自检、Windows Automated 通过 | workflow run `30989439570` 通过 |
| R1-02 | 边界验证脚本 | DONE | [`R01-02-边界验证脚本.md`](R01-02-边界验证脚本.md) | `npm run verify:architecture` 通过 | workflow run `31001470224` 通过 |
| R1-03 | 浏览器组合根 | DONE | [`R01-03-浏览器组合根.md`](R01-03-浏览器组合根.md) | 浏览器组合根专项验证通过 | workflow run `31012168809` 通过 |
| R1-04 | Tauri 组合根 | DONE | [`R01-04-tauri-组合根.md`](R01-04-tauri-组合根.md) | 专项契约、保护资产、格式和桌面 crate 编译通过 | workflow run `31037323146` 通过 |
| R1-05 | Application 组合根 | READY | 待创建 | 待执行 | 待执行 |

## R1-01 完成结果

- 已创建 `architecture/module-boundaries.json`，登记 18 个前端 Feature、11 个 Rust workspace 成员、4 个应用端口、32 个 PostgreSQL 适配模块、15 个 Tauri 命令组及 171 命令的权威契约引用。
- 已创建 `architecture/state-ownership.json`，登记 17 个共享状态及其唯一当前所有者、读写边界、禁止所有者和计划迁移目标。
- 未修改业务源码、依赖、锁文件、数据库迁移、公共命令或模型保护文件。
- 已将 `Public Platform CI` 扩展为 Windows 自动交付门禁，覆盖目标分支推送、PR 和手动触发。
- workflow run `30989439570`、job `92251837163` 在提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` 上通过；证据 artifact `8924033934` 的 SHA-256 为 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`。
- 前端、Rust、Tauri Windows release、release 客户端启动和运行日志扫描均通过；真实 PostgreSQL、Windows Full 和用户本机验收仍保留到最终统一验收。
- R1-01 状态为 `DONE`，R1-02 开放为 `READY`；R1-03 至 R1-05 继续 `BLOCKED`。

## 阶段出口

全部节点完成后必须创建 `R01-stage-completion.md`，并实际通过：

- 架构边界验证；
- `npm run verify:frontend`（Windows）；
- `npm run verify:rust`；
- 公共命令契约；
- 模型保护资产指纹。

## R1-02 完成结果

- 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，共用独立的文件遍历、Import 解析、Cargo 解析和报告模块。
- 两份 R1 架构契约状态均为 `ACTIVE`；`npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤均已接入。
- 三条现存耦合只按精确源/目标与退出任务登记：Application→PostgreSQL 在 `R1-05` 退出，Prediction→P4 Workbench 在 `R14-06` 退出，Prediction→Runs 在 `R14-07` 退出。
- workflow run `31001470224` 在提交 `28ec363babe4f3fbccd14693d0261febdc305458` 上完整通过；artifact `8929207011` 的 SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`。
- R1-02 状态为 `DONE`；R1-03 开放为 `READY`，R1-04 与 R1-05 继续 `BLOCKED`。

## R1-03 完成结果

- 已建立浏览器唯一入口、应用创建、模块注册和 `ApplicationHandle` 生命周期模块；现有 Feature 实现未迁移。
- 浏览器入口及生命周期状态契约已切换，专项架构、TypeScript、Vite build 与正式 Windows Automated 均通过。
- workflow run `31012168809`、Windows job `92326905405` 在提交 `a3b61088abaf0c9f052ecab09e040ea77bd8d344` 上通过；artifact `8933800016` 的 SHA-256 为 `4c28e5668b8b330cbab5b54516af1d70fe9f39c8299bb640da06a5b4442667f9`。
- 真实 PostgreSQL、Windows Full 和用户本机 Windows 实机验收继续保留到最终统一验收。
- R1-03 状态为 `DONE`；R1-04 开放为 `READY`，R1-05 继续 `BLOCKED`。

## R1-04 完成结果

- `src-tauri/src/lib.rs` 已收敛为公共薄入口；Builder、状态构造、命令注册和启动错误映射分别由 `src-tauri/src/bootstrap/` 下的具名模块承担。
- `AppState` 已切换到 `bootstrap/state.rs` 唯一拥有；171 条 Tauri 命令的名称、顺序、参数和返回契约保持不变。
- 已新增 Tauri 组合根专项验证和确定性保护资产验证入口，并接入完整前端验证；19 个既有验证器已迁移到新的注册表或状态所有者。
- 启动探针与打包前后 A/B 已排除组合根和 bundle 回归；Windows Automated 使用日志路径集合识别新 session、最多一次有界重试和 stdout/stderr 诊断，连续两次超时仍硬失败。
- workflow run `31037323146`、job `92412650719` 在清理后验证提交 `5cb66fdedbfcaf89c86a7124f8894bdc71a533c9` 上完整通过；artifact `8943939773` 的 SHA-256 为 `7562c9137d52040627a58d9c8e104c4053b9923983a16daae08e4361e9a78f2b`。
- Automated 报告为 PASS：7 条运行记录、0 条无效记录、0 个运行时错误，release 客户端首次启动成功。
- 真实 PostgreSQL、Windows Full 和用户本机 Windows 实机验收继续保留到最终统一验收。
- R1-04 状态为 `DONE`；R1-05 开放为 `READY`。

## 当前唯一可执行任务

`R1-05 Application 组合根`
