# Database

The public shell keeps the PostgreSQL schema used by the platform workflows: catalog data, competitions, teams, players, lineups, imports, reviews, analytics, jobs, AI sessions, model-run metadata, and audit records.

## Setup

Use a dedicated PostgreSQL database and configure it through the desktop application. The application applies the ordered SQLx migrations in `crates/persistence-postgres/migrations/`.

## Public model boundary

Database tables may retain model identifiers, route metadata, snapshots, and audit records because they are platform integration contracts. The repository does not contain the private engine, production parameters, fixed prediction fixtures, or regression outputs required to populate those records.

## Safety

Never commit database URLs, passwords, API keys, exported production data, or generated runtime logs. `.env`, `.env.*`, and `verification-logs/` are ignored.

## 导入行子记录身份

导入暂存记录的唯一身份为 `batch_id, sheet_name, row_number, entity_type, subrecord_key`。其中能力记录使用 `dimension_code`，动态标签使用 `tag_code`，`player_team_period` 使用 `team_id`、`team_key` 或 `team_name` 形成稳定子记录身份，避免同一物理行中的多条业务记录互相覆盖。
