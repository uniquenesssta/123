use chrono::{DateTime, Duration, Utc};
use football_domain::{MatchDraft, MatchStatus, TeamDraft};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions, PgPool};
use std::str::FromStr;
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

fn test_database_options(connection_url: &str) -> Result<PgConnectOptions, &'static str> {
    if !connection_url.starts_with("postgres://") && !connection_url.starts_with("postgresql://") {
        return Err("测试库必须使用 PostgreSQL URL");
    }
    let options = PgConnectOptions::from_str(connection_url).map_err(|_| "测试库 URL 无效")?;
    if !options
        .get_database()
        .is_some_and(|name| name.to_lowercase().contains("test"))
    {
        return Err("已拒绝连接：专用测试数据库名称必须包含 test");
    }
    Ok(options)
}

#[test]
fn match_catalog_rejects_non_test_database_before_connecting() {
    for url in [
        "postgres://localhost/football",
        "postgres://localhost/",
        "sqlite://football_test",
        "not-a-url",
    ] {
        assert!(test_database_options(url).is_err());
    }
    for url in [
        "postgres://localhost/football_test",
        "postgresql://localhost/FOOTBALL_TEST",
    ] {
        assert!(test_database_options(url).is_ok());
    }
}

#[derive(Clone)]
struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
    token: String,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).expect("设置 FOOTBALL_TEST_DATABASE_URL");
        let options = test_database_options(&connection_url).expect("R7-01 测试库前检失败");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .expect("连接专用校验数据库");
        let actual_name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .expect("确认实际数据库");
        assert!(actual_name.to_lowercase().contains("test"));
        let store = PostgresStore::connect(&DatabaseOptions {
            connection_url,
            max_connections: 4,
            connect_timeout_seconds: 10,
        })
        .await
        .expect("连接 R7-01 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        Self {
            store,
            pool,
            token: Uuid::new_v4().simple().to_string(),
        }
    }

    async fn team(&self, side: &str) -> Uuid {
        self.store
            .create_team(&TeamDraft {
                canonical_name: format!("R7 {side} {}", self.token),
                country_code: Some("ZZ".to_string()),
                metadata: json!({"contract": "r7-01", "fixture": self.token}),
            })
            .await
            .expect("创建契约球队")
            .id
    }

    async fn competition(&self, suffix: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO football.competitions (id, code, name, timezone, competition_kind, metadata) VALUES ($1, $2, $2, 'UTC', 'league', $3)",
        )
        .bind(id)
        .bind(format!("R701-{}-{suffix}", self.token))
        .bind(json!({"fixture": self.token}))
        .execute(&self.pool)
        .await
        .expect("创建契约赛事");
        id
    }

    async fn assert_no_season(&self, competition_id: Uuid) {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM football.seasons WHERE competition_id = $1")
                .bind(competition_id)
                .fetch_one(&self.pool)
                .await
                .expect("检查失败写入后的赛季数");
        assert_eq!(count, 0, "失败比赛不得遗留自动创建的赛季");
    }

    async fn cleanup_fixtures(&self) {
        // Only this run's fixtures are removed; immutable schema/audit evidence is retained.
        let mut tx = self
            .pool
            .begin()
            .await
            .expect("fixture cleanup transaction");
        for sql in [
            "DELETE FROM research.runs WHERE metadata->>'fixture' = $1",
            "DELETE FROM ai_workspace.sessions WHERE metadata->>'fixture' = $1",
            "DELETE FROM football.external_entity_ids WHERE metadata->>'fixture' = $1",
            "DELETE FROM football.matches WHERE metadata->>'fixture' = $1",
            "DELETE FROM football.rounds WHERE stage_id IN (SELECT id FROM football.competition_stages WHERE season_id IN (SELECT id FROM football.seasons WHERE competition_id IN (SELECT id FROM football.competitions WHERE metadata->>'fixture' = $1)))",
            "DELETE FROM football.competition_stages WHERE season_id IN (SELECT id FROM football.seasons WHERE competition_id IN (SELECT id FROM football.competitions WHERE metadata->>'fixture' = $1))",
            "DELETE FROM football.seasons WHERE competition_id IN (SELECT id FROM football.competitions WHERE metadata->>'fixture' = $1)",
            "DELETE FROM football.competitions WHERE metadata->>'fixture' = $1",
            "DELETE FROM football.teams WHERE metadata->>'fixture' = $1",
            "DELETE FROM catalog.data_providers WHERE metadata->>'fixture' = $1",
        ] {
            sqlx::query(sql)
                .bind(&self.token)
                .execute(&mut *tx)
                .await
                .expect("清理本次 fixture");
        }
        tx.commit().await.expect("提交 fixture 清理");
    }
}

fn draft(home: Uuid, away: Uuid, kickoff: DateTime<Utc>, token: &str) -> MatchDraft {
    MatchDraft {
        external_key: String::new(),
        competition_id: None,
        season_id: None,
        stage_id: None,
        round_id: None,
        home_team_id: home,
        away_team_id: away,
        kickoff_time: kickoff,
        status: MatchStatus::Scheduled,
        venue: Some("  R7 Test Ground  ".to_string()),
        metadata: json!({"contract": "r7-01", "fixture": token, "phase": "create"}),
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_catalog_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let test_database = database.clone();
    // A panic is joined before cleanup, so failed assertions also release their fixtures.
    let result = tokio::spawn(async move { verify_match_catalog(&test_database).await }).await;
    database.cleanup_fixtures().await;
    database.pool.close().await;
    database.store.close().await;
    if let Err(error) = result {
        std::panic::resume_unwind(error.into_panic());
    }
}

async fn verify_match_catalog(database: &TestDatabase) {
    let home = database.team("Home").await;
    let away = database.team("Away").await;
    let kickoff = Utc::now() + Duration::days(2);
    let base = draft(home, away, kickoff, &database.token);
    let created = database.store.create_match(&base).await.expect("创建比赛");
    assert!(created.external_key.starts_with("MATCH-"));
    assert_eq!(created.home_team_id, home);
    assert_eq!(created.away_team_id, away);
    assert_eq!(created.venue.as_deref(), Some("R7 Test Ground"));
    assert!(matches!(created.status, MatchStatus::Scheduled));
    let read = database
        .store
        .read_match(created.id)
        .await
        .expect("读取比赛");
    assert_eq!(read.id, created.id);
    assert_eq!(read.external_key, created.external_key);

    let mut update = base.clone();
    update.external_key = created.external_key.clone();
    update.kickoff_time += Duration::hours(1);
    update.status = MatchStatus::Postponed;
    update.venue = Some(" Updated Ground ".to_string());
    update.metadata = json!({"phase": "upsert"});
    let updated = database
        .store
        .create_match(&update)
        .await
        .expect("更新比赛");
    assert_eq!(updated.id, created.id);
    assert!(matches!(updated.status, MatchStatus::Postponed));
    assert_eq!(updated.venue.as_deref(), Some("Updated Ground"));
    let metadata: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM football.matches WHERE id = $1")
            .bind(created.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取合并 metadata");
    assert_eq!(metadata["fixture"], database.token);
    assert_eq!(metadata["phase"], "upsert");

    verify_scope_atomicity(database, &base).await;
    verify_lists(database, &base, created.id).await;
    verify_deletion(database, created.id, &created.external_key).await;
}

async fn verify_scope_atomicity(database: &TestDatabase, base: &MatchDraft) {
    let competition = database.competition("scope").await;
    let mut scoped = base.clone();
    scoped.competition_id = Some(competition);
    scoped.external_key = format!("R7-SCOPE-{}", database.token);
    let mut invalid = scoped.clone();
    invalid.away_team_id = invalid.home_team_id;
    assert!(database.store.create_match(&invalid).await.is_err());
    database.assert_no_season(competition).await;

    invalid.away_team_id = Uuid::new_v4();
    assert!(database.store.create_match(&invalid).await.is_err());
    database.assert_no_season(competition).await;
    let failed_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM football.matches WHERE external_key = $1")
            .bind(&scoped.external_key)
            .fetch_one(&database.pool)
            .await
            .expect("检查失败比赛");
    assert_eq!(failed_count, 0);

    let match_record = database
        .store
        .create_match(&scoped)
        .await
        .expect("自动推断赛季");
    let season = match_record.season_id.expect("自动创建赛季");
    assert_eq!(match_record.competition_id, Some(competition));
    let stage = Uuid::new_v4();
    let round = Uuid::new_v4();
    sqlx::query("INSERT INTO football.competition_stages (id, season_id, code, name, stage_kind) VALUES ($1, $2, 'R7', 'R7', 'league')")
        .bind(stage).bind(season).execute(&database.pool).await.expect("创建阶段");
    sqlx::query(
        "INSERT INTO football.rounds (id, stage_id, code, name) VALUES ($1, $2, 'R7', 'R7')",
    )
    .bind(round)
    .bind(stage)
    .execute(&database.pool)
    .await
    .expect("创建轮次");
    let mut round_draft = base.clone();
    round_draft.external_key = format!("R7-ROUND-{}", database.token);
    round_draft.round_id = Some(round);
    let inferred = database
        .store
        .create_match(&round_draft)
        .await
        .expect("按轮次推断层级");
    assert_eq!(inferred.stage_id, Some(stage));
    assert_eq!(inferred.season_id, Some(season));
    assert_eq!(inferred.competition_id, Some(competition));

    for (season_id, stage_id, round_id, competition_id) in [
        (Some(season), Some(stage), Some(round), Some(Uuid::new_v4())),
        (
            Some(Uuid::new_v4()),
            Some(stage),
            Some(round),
            Some(competition),
        ),
        (
            Some(season),
            Some(Uuid::new_v4()),
            Some(round),
            Some(competition),
        ),
        (Some(season), Some(stage), None, Some(Uuid::new_v4())),
        (Some(season), None, None, Some(Uuid::new_v4())),
        (None, None, Some(Uuid::new_v4()), None),
    ] {
        let mut conflict = round_draft.clone();
        conflict.season_id = season_id;
        conflict.stage_id = stage_id;
        conflict.round_id = round_id;
        conflict.competition_id = competition_id;
        assert!(
            database.store.create_match(&conflict).await.is_err(),
            "非法 scope 必须拒绝"
        );
        let unchanged = database
            .store
            .read_match(inferred.id)
            .await
            .expect("读取未改动比赛");
        assert_eq!(unchanged.stage_id, Some(stage));
        assert_eq!(unchanged.season_id, Some(season));
        assert_eq!(unchanged.competition_id, Some(competition));
    }

    // Cover rollback of ON CONFLICT season metadata updates, not only new inserts.
    let rollback_competition = database.competition("rollback").await;
    let sentinel_season = Uuid::new_v4();
    sqlx::query("INSERT INTO football.seasons (id, competition_id, name, status, metadata) VALUES ($1, $2, $3, 'archived', '{\"sentinel\":true}')")
        .bind(sentinel_season).bind(rollback_competition).bind(base.kickoff_time.format("%Y").to_string())
        .execute(&database.pool).await.expect("创建回滚校验赛季");
    let mut rollback = base.clone();
    rollback.competition_id = Some(rollback_competition);
    rollback.away_team_id = Uuid::new_v4();
    assert!(database.store.create_match(&rollback).await.is_err());
    let metadata: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM football.seasons WHERE id = $1")
            .bind(sentinel_season)
            .fetch_one(&database.pool)
            .await
            .expect("读取回滚后的赛季");
    assert_eq!(
        metadata,
        json!({"sentinel": true}),
        "失败必须回滚赛季 metadata 更新"
    );
}

async fn verify_lists(database: &TestDatabase, base: &MatchDraft, postponed_id: Uuid) {
    // More rows than either public cap, with equal kickoffs to exercise ID tie-breaking.
    let ids: Vec<Uuid> = (0..600).map(|_| Uuid::new_v4()).collect();
    sqlx::query(
        "INSERT INTO football.matches (id, external_key, home_team_id, away_team_id, kickoff_time, status, metadata) SELECT id, $2 || ordinal::text, $3, $4, $5::timestamptz + (ordinal / 2) * interval '1 minute', CASE ordinal % 3 WHEN 0 THEN 'cancelled' WHEN 1 THEN 'live' ELSE 'scheduled' END, $6 FROM unnest($1::uuid[]) WITH ORDINALITY AS fixture(id, ordinal)",
    )
    .bind(&ids)
    .bind(format!("R7-LIST-{}-", database.token))
    .bind(base.home_team_id)
    .bind(base.away_team_id)
    .bind(base.kickoff_time)
    .bind(&base.metadata)
    .execute(&database.pool)
    .await
    .expect("创建列表边界夹具");
    for requested in [0, 1, 2, u32::MAX] {
        let upcoming = database
            .store
            .list_upcoming_matches(requested)
            .await
            .expect("upcoming list");
        assert_eq!(upcoming.len(), requested.clamp(1, 250) as usize);
        assert!(upcoming
            .iter()
            .all(|item| matches!(item.status, MatchStatus::Scheduled | MatchStatus::Live)));
        assert!(!upcoming.iter().any(|item| item.id == postponed_id));
        assert!(
            upcoming.windows(2).all(
                |pair| (pair[0].kickoff_time, pair[0].id) <= (pair[1].kickoff_time, pair[1].id)
            ),
            "upcoming 必须按时间、ID 升序"
        );
        let managed = database
            .store
            .list_managed_matches(requested)
            .await
            .expect("managed list");
        assert_eq!(managed.len(), requested.clamp(1, 500) as usize);
        assert!(
            managed.windows(2).all(
                |pair| (pair[0].kickoff_time, pair[0].id) >= (pair[1].kickoff_time, pair[1].id)
            ),
            "managed 必须按时间、ID 降序"
        );
    }
}

async fn verify_deletion(database: &TestDatabase, match_id: Uuid, external_key: &str) {
    let session = Uuid::new_v4();
    let provider = Uuid::new_v4();
    sqlx::query("INSERT INTO ai_workspace.sessions (id, profile_id, preset_key, title, match_id, metadata) VALUES ($1, 'r7-contract', 'r7-contract', 'R7', $2, $3)")
        .bind(session).bind(match_id).bind(json!({"fixture": database.token}))
        .execute(&database.pool).await.expect("创建可空 AI 会话引用");
    sqlx::query("INSERT INTO catalog.data_providers (id, code, name, provider_type, metadata) VALUES ($1, $2, 'R7', 'manual', $3)")
        .bind(provider).bind(format!("R7-{}", database.token)).bind(json!({"fixture": database.token}))
        .execute(&database.pool).await.expect("创建测试 provider");
    sqlx::query("INSERT INTO football.external_entity_ids (id, provider_id, entity_type, entity_id, external_id, metadata) VALUES ($1, $2, 'match', $3, $4, $5)")
        .bind(Uuid::new_v4()).bind(provider).bind(match_id).bind(external_key).bind(json!({"fixture": database.token}))
        .execute(&database.pool).await.expect("创建 external ID 引用");

    // Reusable immutable schema artifact; do not disable its immutability trigger for cleanup.
    let registered_schema: Option<Uuid> = sqlx::query_scalar("INSERT INTO research.schema_versions (id, schema_key, version, schema_kind, schema_body, content_sha256) VALUES ($1, 'r7-match-catalog-contract', '1', 'contract', '{}', $2) ON CONFLICT (schema_key, version) DO NOTHING RETURNING id")
        .bind(Uuid::new_v4())
        .bind("44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a")
        .fetch_optional(&database.pool)
        .await
        .expect("注册契约 schema");
    let schema = match registered_schema {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM research.schema_versions WHERE schema_key = 'r7-match-catalog-contract' AND version = '1'")
            .fetch_one(&database.pool)
            .await
            .expect("读取契约 schema"),
    };
    let research = Uuid::new_v4();
    sqlx::query("INSERT INTO research.runs (id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key, request_fingerprint, schema_version_id, metadata) VALUES ($1, $2, 'T-6h', now(), $3, $4, $5, $6, $7)")
        .bind(research).bind(match_id).bind(Uuid::new_v4()).bind(format!("R7-{}", database.token))
        .bind("0".repeat(64)).bind(schema).bind(json!({"fixture": database.token}))
        .execute(&database.pool).await.expect("创建受保护研究引用");
    assert!(
        database.store.delete_match(match_id).await.is_err(),
        "有研究引用必须保护比赛"
    );
    assert!(database.store.read_match(match_id).await.is_ok());
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit.events WHERE event_type = 'match_deleted' AND entity_id = $1",
    )
    .bind(match_id.to_string())
    .fetch_one(&database.pool)
    .await
    .expect("保护删除审计数");
    assert_eq!(audit_count, 0, "拒绝删除不得写入成功审计");
    let reference: Option<Uuid> =
        sqlx::query_scalar("SELECT match_id FROM ai_workspace.sessions WHERE id = $1")
            .bind(session)
            .fetch_one(&database.pool)
            .await
            .expect("读取保护后的会话引用");
    assert_eq!(reference, Some(match_id));
    sqlx::query("DELETE FROM research.runs WHERE id = $1")
        .bind(research)
        .execute(&database.pool)
        .await
        .expect("清理契约研究引用");

    database
        .store
        .delete_match(match_id)
        .await
        .expect("删除未受保护比赛");
    assert!(database.store.read_match(match_id).await.is_err());
    let reference: Option<Uuid> =
        sqlx::query_scalar("SELECT match_id FROM ai_workspace.sessions WHERE id = $1")
            .bind(session)
            .fetch_one(&database.pool)
            .await
            .expect("读取解除后的会话引用");
    assert_eq!(reference, None);
    let external_count: i64 = sqlx::query_scalar("SELECT count(*) FROM football.external_entity_ids WHERE entity_type = 'match' AND entity_id = $1")
        .bind(match_id).fetch_one(&database.pool).await.expect("读取解除后的外部引用");
    assert_eq!(external_count, 0);
    assert!(
        database.store.delete_match(match_id).await.is_err(),
        "重复删除应报告不存在"
    );
    let audit: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM audit.events WHERE event_type = 'match_deleted' AND entity_id = $1",
    )
    .bind(match_id.to_string())
    .fetch_one(&database.pool)
    .await
    .expect("读取成功删除审计");
    assert_eq!(audit["external_key"], external_key);
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit.events WHERE event_type = 'match_deleted' AND entity_id = $1",
    )
    .bind(match_id.to_string())
    .fetch_one(&database.pool)
    .await
    .expect("删除审计数");
    assert_eq!(audit_count, 1, "成功删除只写一次审计");
}
