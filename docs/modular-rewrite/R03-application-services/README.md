# R03 Application Services 重写：执行记录索引

## 阶段状态

`IN_PROGRESS`

R2 已完成并关闭。R3 只重写 Application 编排与 Ports/Services/Use Cases 边界，不修改具体 PostgreSQL SQL、Tauri DTO、前端状态或模型实现。

## 基线

- R3 分支：`new-C`
- R3 起点：`7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f`
- R2 完成记录：[`../R02-domain-rewrite/R02-stage-completion.md`](../R02-domain-rewrite/R02-stage-completion.md)
- 目标平台：Windows

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R3-01 | Application Ports 设计 | VERIFYING |
| R3-02 | Database Service | BLOCKED |
| R3-03 | Competition / Rules Services | BLOCKED |
| R3-04 | Teams / Players Services | BLOCKED |
| R3-05 | Lineups Service | BLOCKED |
| R3-06 | Prediction Service | BLOCKED |
| R3-07 | Research Service | BLOCKED |
| R3-08 | Review / Postmatch / Analytics Services | BLOCKED |
| R3-09 | Exchange / AI / Release Services | BLOCKED |
| R3-10 | ApplicationService 兼容门面 | BLOCKED |

## R3-01 当前结果

- 已从真实源码扫描 PostgreSQL `232` 个公开异步方法，其中 `209` 个当前被 Application 调用；Application 对具体 PostgreSQL crate 的直接导入仍只有 `composition/port_registry.rs` 一处。
- 已建立 15 个 Port 职责域和最小能力 trait，不建立万能 Repository。
- Ports 只使用 Domain、Application、自有 DTO、`football-model-api` 与标准基础类型；禁止 SQL Row、SQLx、PgPool、PostgresStore、PersistenceError 和裸 JSON Value 穿透。
- `football-model-api` 继续作为模型执行边界，不复制一套模型协议。
- 当前 concrete PostgreSQL 导入仍由 R1 组合根暂时持有，后续 R3-02 至 R3-09 按服务迁移逐步接入适配器；R3-01 不改业务行为。
- `verify:application-ports` 已接入 `verify:architecture`，锁定职责目录、trait 集合和过渡导入唯一所有者。

R3-01 完整 Windows 本机门禁通过前保持 `VERIFYING`，不得提前开放 R3-02。
