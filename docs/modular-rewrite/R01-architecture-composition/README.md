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
| R1-01 | 模块边界契约 | VERIFYING | [`R01-01-模块边界契约.md`](R01-01-模块边界契约.md) | JSON 解析与契约自检通过 | 完整工作树门禁待执行 |
| R1-02 | 边界验证脚本 | BLOCKED | 待创建 | 待执行 | 待执行 |
| R1-03 | 浏览器组合根 | BLOCKED | 待创建 | 待执行 | 待执行 |
| R1-04 | Tauri 组合根 | BLOCKED | 待创建 | 待执行 | 待执行 |
| R1-05 | Application 组合根 | BLOCKED | 待创建 | 待执行 | 待执行 |

## R1-01 当前结果

- 已创建 `architecture/module-boundaries.json`，登记 18 个前端 Feature、11 个 Rust workspace 成员、4 个应用端口、32 个 PostgreSQL 适配模块、15 个 Tauri 命令组及 171 命令的权威契约引用。
- 已创建 `architecture/state-ownership.json`，登记 17 个共享状态及其唯一当前所有者、读写边界、禁止所有者和计划迁移目标。
- 未修改业务源码、依赖、锁文件、数据库迁移、公共命令或模型保护文件。
- 因当前执行环境不能建立完整 Git 工作树，`npm run verify:frontend`、Rust 全门禁和现有架构脚本尚未在本提交执行；R1-01 保持 `VERIFYING`，R1-02 不开放。

## 阶段出口

全部节点完成后必须创建 `R01-stage-completion.md`，并实际通过：

- 架构边界验证；
- `npm run verify:frontend`（Windows）；
- `npm run verify:rust`；
- 公共命令契约；
- 模型保护资产指纹。

## 当前唯一可执行任务

`R1-01 模块边界契约：完成完整工作树验证并关闭节点`
