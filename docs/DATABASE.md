# Database

The public shell keeps the PostgreSQL schema used by the platform workflows: catalog data, competitions, teams, players, lineups, imports, reviews, analytics, jobs, AI sessions, model-run metadata, and audit records.

## Setup

Use a dedicated PostgreSQL database and configure it through the desktop application. The application applies the ordered SQLx migrations in `crates/persistence-postgres/migrations/`.

## Public model boundary

Database tables may retain model identifiers, route metadata, snapshots, and audit records because they are platform integration contracts. The repository does not contain the private engine, production parameters, fixed prediction fixtures, or regression outputs required to populate those records.

## Safety

Never commit database URLs, passwords, API keys, exported production data, or generated runtime logs. `.env`, `.env.*`, and `verification-logs/` are ignored.

## 导入行子记录身份

R7-15 将已暂存行的 UUID/批次定位、受检物理行号与重复球队合并收敛到 `workbooks/identity/`。UUID 定位裁决对象，工作表与物理行号用于来源定位；同一物理行允许多个实体/合法子记录。三类预检在同源查找/取消之前验证行号满足现有 `row_number >= 2` 与 PostgreSQL integer 范围，账本插入复用同一检查；不在 Rust 重建子键或按物理行去重。

导入暂存记录的唯一身份为 `batch_id, sheet_name, row_number, entity_type, subrecord_key`。其中能力记录使用 `dimension_code`，动态标签使用 `tag_code`，`player_team_period` 使用 `team_id`、`team_key` 或 `team_name` 按原生成列优先级 `team_id → team_key → lower(BTRIM(team_name))` 形成稳定子记录身份，避免同一物理行中的多条业务记录互相覆盖。能力/标签代码只按原 BTRIM 处理；`subrecord_key` 仍由 0043 生成列和五字段 UNIQUE 判定，重复键由原 INSERT 拒绝并回滚暂存事务，不使用 ON CONFLICT 吞掉合法或重复子记录。
