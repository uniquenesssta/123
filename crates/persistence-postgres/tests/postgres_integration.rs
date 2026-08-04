use chrono::{Duration, Utc};
use football_domain::{
    CompetitionDraft, CompetitionKind, CompetitionProfile, EnqueueJobDraft, EvidenceClaimDraft,
    EvidenceConflictDraft, EvidenceVerificationState, FormationDistributionQuery,
    FormationUsageDistributionDraft, FormationUsageEntryDraft, FormationUsageListQuery, JobStatus,
    LineupDraft, LineupPairDraft, LineupPlayerDraft, LineupType, MatchDraft, MatchResultDraft,
    MatchReviewDraft, MatchReviewPackageComparison, MatchReviewPackageDiffSummary,
    MatchReviewPackagePreview, MatchReviewPackageSnapshotSummary, MatchReviewPackageSummary,
    MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowStatus,
    MatchEventRevisionStatus, MatchEventType, MatchEventVerificationStatus,
    MatchReviewPackageWorkflowStep, MatchStatus, P4Horizon, PrematchSnapshotDraft,
    ResearchRunDraft, RulePackageDraft, RuleRouting, RuleSourceReference,
    SchemaVersionDraft, SeasonDraft, SnapshotFeatureDraft, SnapshotProbabilityDraft,
    SnapshotSourceKind,
    SourcePolicyDefinition, SourcePolicyVersionDraft, SourceTierDefinition, SourceTierRule,
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetParsedWorkbook,
    SpreadsheetRawRow, TeamDraft, PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT,
};
use football_model_api::ModelDescriptor;
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::sync::Mutex;
use uuid::Uuid;

static DATABASE_TEST_LOCK: Mutex<()> = Mutex::const_new(());
const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
            panic!(
                "运行忽略测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接专用 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行全部数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立集成测试校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

async fn insert_match_event_fixture(
    pool: &PgPool,
    match_id: Uuid,
    event_key: &str,
    sequence_no: i32,
    event_type: &str,
    team_id: Option<Uuid>,
    player_id: Option<Uuid>,
    minute: i16,
    home_score: Option<i16>,
    away_score: Option<i16>,
    verification_status: &str,
    revision_status: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO review.match_events (
            id, match_id, event_key, sequence_no, event_type,
            team_id, player_id, minute, period, home_score, away_score,
            verification_status, revision_status, verified_at, confidence
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'normal_time',$9,$10,$11,$12,now(),1.0)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(match_id)
    .bind(event_key)
    .bind(sequence_no)
    .bind(event_type)
    .bind(team_id)
    .bind(player_id)
    .bind(minute)
    .bind(home_score)
    .bind(away_score)
    .bind(verification_status)
    .bind(revision_status)
    .execute(pool)
    .await
    .expect("写入 D2 结构化比赛事件");
}

#[tokio::test]
#[ignore = "会彻底清空专用 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn destructive_reset_rebuilds_an_empty_migrated_database() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("reset-team-{token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建清空前测试球队");

    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM football.teams")
        .fetch_one(&database.pool)
        .await
        .expect("读取清空前球队数量");
    assert!(count_before >= 1);

    database
        .store
        .reset_to_pristine()
        .await
        .expect("彻底清空并重新执行迁移");

    let team_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM football.teams")
        .fetch_one(&database.pool)
        .await
        .expect("读取清空后球队数量");
    let migration_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM public._sqlx_migrations WHERE success")
            .fetch_one(&database.pool)
            .await
            .expect("读取重建后的迁移账本");
    let position_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM football.positions")
            .fetch_one(&database.pool)
            .await
            .expect("读取重建后的内置位置目录");

    assert_eq!(team_count, 0);
    assert!(migration_count > 0);
    assert!(position_count > 0);
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn prediction_input_audit_fields_are_persisted_and_immutable() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let definition_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    let parameter_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let manifest_sha = "a".repeat(64);
    let input_sha = "b".repeat(64);

    sqlx::query(
        "INSERT INTO model.definitions (id, model_key, display_name) VALUES ($1, $2, $3)",
    )
    .bind(definition_id)
    .bind(format!("d1-audit-{token}"))
    .bind("D1 audit integration model")
    .execute(&database.pool)
    .await
    .expect("创建 D1 测试模型定义");
    sqlx::query(
        r#"
        INSERT INTO model.versions (
            id, model_id, version, engine_version, input_schema_version, output_schema_version
        ) VALUES ($1, $2, '1.0.0', 'integration', 'integration-input', 'integration-output')
        "#,
    )
    .bind(version_id)
    .bind(definition_id)
    .execute(&database.pool)
    .await
    .expect("创建 D1 测试模型版本");
    sqlx::query(
        r#"
        INSERT INTO model.parameter_sets (
            id, model_version_id, parameter_version, name, definition, definition_sha256
        ) VALUES ($1, $2, 'p1', 'D1 integration parameters', '{}'::jsonb, $3)
        "#,
    )
    .bind(parameter_id)
    .bind(version_id)
    .bind("c".repeat(64))
    .execute(&database.pool)
    .await
    .expect("创建 D1 测试参数");
    sqlx::query(
        r#"
        INSERT INTO model.runs (
            id, match_key, model_version_id, parameter_set_id, snapshot_type,
            route_reason, status, input_payload, input_sha256,
            input_audit_version, input_readiness_level, input_readiness_score,
            input_manifest, input_manifest_sha256, completed_at
        ) VALUES (
            $1, $2, $3, $4, 'T-1h',
            '{}'::jsonb, 'succeeded', '{"fixture":"d1"}'::jsonb, $5,
            'prematch-input-audit-v1', 'formal_ready', 100,
            '{"fixture":"d1"}'::jsonb, $6, now()
        )
        "#,
    )
    .bind(run_id)
    .bind(format!("D1-AUDIT-{token}"))
    .bind(version_id)
    .bind(parameter_id)
    .bind(&input_sha)
    .bind(&manifest_sha)
    .execute(&database.pool)
    .await
    .expect("保存 D1 输入审计运行");

    let row = sqlx::query(
        r#"
        SELECT input_audit_version, input_readiness_level, input_readiness_score,
               input_manifest_sha256, input_sha256
        FROM model.runs
        WHERE id = $1
        "#,
    )
    .bind(run_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取 D1 输入审计运行");
    assert_eq!(
        row.try_get::<String, _>("input_audit_version").unwrap(),
        "prematch-input-audit-v1"
    );
    assert_eq!(
        row.try_get::<String, _>("input_readiness_level").unwrap(),
        "formal_ready"
    );
    assert_eq!(row.try_get::<i16, _>("input_readiness_score").unwrap(), 100);
    assert_eq!(
        row.try_get::<String, _>("input_manifest_sha256").unwrap(),
        manifest_sha
    );
    assert_eq!(row.try_get::<String, _>("input_sha256").unwrap(), input_sha);

    let mutation = sqlx::query(
        "UPDATE model.runs SET input_payload = '{\"fixture\":\"changed\"}'::jsonb WHERE id = $1",
    )
    .bind(run_id)
    .execute(&database.pool)
    .await;
    assert!(mutation.is_err(), "输入载荷必须受不可变触发器保护");

    sqlx::query(
        "UPDATE model.runs SET history_hidden_at = now(), history_hidden_reason = 'integration' WHERE id = $1",
    )
    .bind(run_id)
    .execute(&database.pool)
    .await
    .expect("非输入身份字段仍允许更新");

    sqlx::query("DELETE FROM model.runs WHERE id = $1")
        .bind(run_id)
        .execute(&database.pool)
        .await
        .expect("清理 D1 测试运行");
    sqlx::query("DELETE FROM model.parameter_sets WHERE id = $1")
        .bind(parameter_id)
        .execute(&database.pool)
        .await
        .expect("清理 D1 测试参数");
    sqlx::query("DELETE FROM model.versions WHERE id = $1")
        .bind(version_id)
        .execute(&database.pool)
        .await
        .expect("清理 D1 测试模型版本");
    sqlx::query("DELETE FROM model.definitions WHERE id = $1")
        .bind(definition_id)
        .execute(&database.pool)
        .await
        .expect("清理 D1 测试模型定义");
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn structured_match_events_are_queryable_and_revision_aware() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let home_id = Uuid::new_v4();
    let away_id = Uuid::new_v4();
    let scorer_id = Uuid::new_v4();
    let booked_id = Uuid::new_v4();
    let match_id = Uuid::new_v4();
    let review_id = Uuid::new_v4();

    for (team_id, name) in [
        (home_id, format!("D2 home {token}")),
        (away_id, format!("D2 away {token}")),
    ] {
        sqlx::query(
            "INSERT INTO football.teams (id, canonical_name, normalized_name) VALUES ($1,$2,$3)",
        )
        .bind(team_id)
        .bind(&name)
        .bind(name.to_lowercase())
        .execute(&database.pool)
        .await
        .expect("创建 D2 测试球队");
    }
    for (player_id, name) in [
        (scorer_id, format!("D2 scorer {token}")),
        (booked_id, format!("D2 booked {token}")),
    ] {
        sqlx::query(
            "INSERT INTO football.players (id, canonical_name, normalized_name) VALUES ($1,$2,$3)",
        )
        .bind(player_id)
        .bind(&name)
        .bind(name.to_lowercase())
        .execute(&database.pool)
        .await
        .expect("创建 D2 测试球员");
    }
    sqlx::query(
        r#"
        INSERT INTO football.matches (
            id, external_key, home_team_id, away_team_id, kickoff_time, status
        ) VALUES ($1,$2,$3,$4,now() - interval '2 hours','finished')
        "#,
    )
    .bind(match_id)
    .bind(format!("D2-EVENT-{token}"))
    .bind(home_id)
    .bind(away_id)
    .execute(&database.pool)
    .await
    .expect("创建 D2 测试比赛");
    sqlx::query(
        r#"
        INSERT INTO football.match_results (
            match_id, home_goals_90, away_goals_90, finalized_at
        ) VALUES ($1,1,0,now())
        "#,
    )
    .bind(match_id)
    .execute(&database.pool)
    .await
    .expect("创建 D2 正式赛果");
    sqlx::query(
        r#"
        INSERT INTO review.match_reviews (
            id, match_id, review_version, data_coverage, conclusions,
            status, calculation_version, result_snapshot, prediction_evaluation, finalized_at
        ) VALUES ($1,$2,$3,1.0,'{}'::jsonb,'finalized','integration-d2',
                  '{"home_goals_90":1,"away_goals_90":0}'::jsonb,'{}'::jsonb,now())
        "#,
    )
    .bind(review_id)
    .bind(match_id)
    .bind(format!("d2-{token}"))
    .execute(&database.pool)
    .await
    .expect("创建 D2 正式复盘");

    insert_match_event_fixture(
        &database.pool,
        match_id,
        "goal:1",
        1,
        "goal",
        Some(home_id),
        Some(scorer_id),
        12,
        Some(1),
        Some(0),
        "verified",
        "active",
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        match_id,
        "goal:cancelled",
        2,
        "goal",
        Some(home_id),
        Some(scorer_id),
        24,
        Some(2),
        Some(0),
        "verified",
        "cancelled",
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        match_id,
        "card:1",
        3,
        "yellow_card",
        Some(away_id),
        Some(booked_id),
        42,
        None,
        None,
        "disputed",
        "active",
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        match_id,
        "legacy:hidden",
        4,
        "other",
        None,
        None,
        50,
        None,
        None,
        "unverified",
        "superseded",
    )
    .await;

    let events = database
        .store
        .list_match_events(match_id)
        .await
        .expect("查询当前结构化事件");
    assert_eq!(events.len(), 3, "superseded 历史事件不应进入当前查询");
    assert_eq!(events[0].event_type, MatchEventType::Goal);
    assert_eq!(
        events[1].revision_status,
        MatchEventRevisionStatus::Cancelled
    );
    assert_eq!(
        events[2].verification_status,
        MatchEventVerificationStatus::Disputed
    );

    let detail = database
        .store
        .read_match_review(review_id)
        .await
        .expect("读取带事件摘要的正式复盘");
    assert_eq!(detail.event_summary.total_count, 3);
    assert_eq!(detail.event_summary.effective_count, 2);
    assert_eq!(detail.event_summary.cancelled_count, 1);
    assert_eq!(detail.event_summary.disputed_count, 1);
    assert_eq!(detail.event_summary.latest_home_score, Some(1));
    assert_eq!(detail.event_summary.latest_away_score, Some(0));
    assert_eq!(detail.event_summary.event_type_counts.get("goal"), Some(&1));
    assert_eq!(
        detail.event_summary.event_type_counts.get("yellow_card"),
        Some(&1)
    );

    sqlx::query("DELETE FROM review.match_reviews WHERE id=$1")
        .bind(review_id)
        .execute(&database.pool)
        .await
        .expect("清理 D2 复盘");
    sqlx::query("DELETE FROM football.matches WHERE id=$1")
        .bind(match_id)
        .execute(&database.pool)
        .await
        .expect("清理 D2 比赛");
    sqlx::query("DELETE FROM football.players WHERE id = ANY($1::uuid[])")
        .bind(vec![scorer_id, booked_id])
        .execute(&database.pool)
        .await
        .expect("清理 D2 球员");
    sqlx::query("DELETE FROM football.teams WHERE id = ANY($1::uuid[])")
        .bind(vec![home_id, away_id])
        .execute(&database.pool)
        .await
        .expect("清理 D2 球队");
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn team_package_player_team_period_subrecords_are_distinct() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let batch_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO catalog.import_batches (id, import_type, status, metadata)
        VALUES ($1,'team_package','pending','{}'::jsonb)
        "#,
    )
    .bind(batch_id)
    .execute(&database.pool)
    .await
    .expect("创建球队资料包预检批次");

    for (id, team_name) in [
        (Uuid::new_v4(), "Algeria"),
        (Uuid::new_v4(), "Manchester City"),
    ] {
        sqlx::query(
            r#"
            INSERT INTO catalog.import_rows (
                id, batch_id, sheet_name, row_number, entity_type,
                requested_action, status, payload
            ) VALUES ($1,$2,'球员与评分',2,'player_team_period','update','ready_update',$3)
            "#,
        )
        .bind(id)
        .bind(batch_id)
        .bind(json!({"team_name": team_name}))
        .execute(&database.pool)
        .await
        .expect("同一球员物理行应允许国家队与俱乐部两条效力记录");
    }

    let keys = sqlx::query_scalar::<_, String>(
        r#"
        SELECT subrecord_key
        FROM catalog.import_rows
        WHERE batch_id=$1
        ORDER BY subrecord_key
        "#,
    )
    .bind(batch_id)
    .fetch_all(&database.pool)
    .await
    .expect("读取效力子记录身份");
    assert_eq!(keys, vec!["algeria".to_string(), "manchester city".to_string()]);

    let duplicate = sqlx::query(
        r#"
        INSERT INTO catalog.import_rows (
            id, batch_id, sheet_name, row_number, entity_type,
            requested_action, status, payload
        ) VALUES ($1,$2,'球员与评分',2,'player_team_period','update','ready_update',$3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(batch_id)
    .bind(json!({"team_name": "ALGERIA"}))
    .execute(&database.pool)
    .await;
    assert!(duplicate.is_err(), "相同球队效力记录仍必须受唯一约束");

    sqlx::query("DELETE FROM catalog.import_batches WHERE id=$1")
        .bind(batch_id)
        .execute(&database.pool)
        .await
        .expect("清理球队资料包预检批次");
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn formation_catalog_usage_history_and_resolution_are_consistent() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let team = database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("formation-team-{token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建阵型测试球队");
    let formations = database
        .store
        .list_formations(true)
        .await
        .expect("读取阵型目录");
    assert!(formations.len() >= 17);
    let primary = formations
        .iter()
        .find(|item| item.code == "4-2-3-1")
        .expect("4-2-3-1");
    let secondary = formations
        .iter()
        .find(|item| item.code == "4-3-3")
        .expect("4-3-3");
    let today = Utc::now().date_naive();
    let draft = FormationUsageDistributionDraft {
        scope_type: "team".to_string(),
        team_id: Some(team.id),
        coach_id: None,
        competition_id: None,
        window_preset: "custom".to_string(),
        window_start: Some(today - Duration::days(30)),
        window_end: Some(today),
        observed_matches: 10,
        confidence: 0.8,
        alpha: 3.0,
        source_document_id: None,
        metadata: json!({"integration_test": true}),
        entries: vec![
            FormationUsageEntryDraft {
                formation_id: primary.id,
                usage_count: 6,
            },
            FormationUsageEntryDraft {
                formation_id: secondary.id,
                usage_count: 3,
            },
        ],
    };
    database
        .store
        .save_formation_usage_distribution(&draft)
        .await
        .expect("首次保存阵型观察");
    database
        .store
        .save_formation_usage_distribution(&draft)
        .await
        .expect("再次保存应追加历史");
    let history = database
        .store
        .list_formation_usage_distributions(&FormationUsageListQuery {
            team_id: Some(team.id),
            coach_id: None,
            competition_id: None,
            limit: 20,
        })
        .await
        .expect("读取阵型历史");
    assert!(history.len() >= 2);
    assert!(history[0]
        .entries
        .iter()
        .any(|item| item.formation_code == "UNKNOWN"));
    let sum: f64 = history[0]
        .entries
        .iter()
        .map(|item| item.smoothed_probability)
        .sum();
    assert!((sum - 1.0).abs() < 1e-9);
    let resolved = database
        .store
        .resolve_formation_distribution(&FormationDistributionQuery {
            match_id: None,
            team_id: team.id,
            coach_id: None,
            competition_id: None,
            as_of: Some(Utc::now()),
        })
        .await
        .expect("解析阵型分布");
    assert_eq!(resolved.source_level, "team");
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn monthly_workbook_rebinds_stale_formation_id_by_code() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let team_id = Uuid::new_v4();
    let stale_formation_id = Uuid::new_v4();
    let expected_formation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM football.formations WHERE code='4-2-3-1' AND is_active",
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取当前数据库的 4-2-3-1 阵型ID");
    assert_ne!(stale_formation_id, expected_formation_id);

    let workbook = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.into(),
        source_file_name: format!("stale-formation-{token}.xlsx"),
        source_sha256: format!("stale-formation-{token}"),
        rows: vec![
            SpreadsheetRawRow {
                sheet_name: "球队基础资料".into(),
                row_number: 2,
                entity_type: SpreadsheetEntityType::Team,
                action: SpreadsheetAction::Add,
                values: json!({
                    "team_id": team_id.to_string(),
                    "official_name": format!("stale-formation-team-{token}"),
                    "team_type": "club",
                    "country_code": "ZZ",
                    "is_active": "true",
                    "source_urls": "https://example.test/stale-formation-team"
                }),
            },
            SpreadsheetRawRow {
                sheet_name: "教练与阵型".into(),
                row_number: 2,
                entity_type: SpreadsheetEntityType::FormationUsage,
                action: SpreadsheetAction::Add,
                values: json!({
                    "scope_type": "team",
                    "team_id": team_id.to_string(),
                    "team_name": format!("stale-formation-team-{token}"),
                    "formation_id": stale_formation_id.to_string(),
                    "formation_code": "4-2-3-1",
                    "window_preset": "custom",
                    "window_start": "2026-07-01",
                    "window_end": "2026-07-22",
                    "observed_matches": "1",
                    "usage_count": "1",
                    "confidence": "0.8",
                    "alpha": "3",
                    "observed_at": "2026-07-22T00:00:00Z"
                }),
            },
        ],
    };

    let preview = database
        .store
        .preview_team_monthly_import(&workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检包含过期阵型ID的球队工作簿");
    assert_eq!(preview.counts.error, 0);
    assert_eq!(preview.counts.conflict, 0);
    let formation_row = preview
        .rows
        .iter()
        .find(|row| row.entity_type == SpreadsheetEntityType::FormationUsage)
        .expect("阵型预检行");
    let expected_formation_id_text = expected_formation_id.to_string();
    assert_eq!(
        formation_row.payload["_resolved_formation_id"].as_str(),
        Some(expected_formation_id_text.as_str())
    );
    assert!(formation_row
        .message
        .as_deref()
        .is_some_and(|message| message.contains("已按阵型代码 4-2-3-1 重新绑定")));

    database
        .store
        .commit_team_monthly_import(preview.batch_id)
        .await
        .expect("提交重新绑定后的阵型观察");
    let stored_formation_id: Uuid = sqlx::query_scalar(
        "SELECT formation_id FROM feature.formation_usage_observations WHERE team_id=$1 ORDER BY observed_at DESC LIMIT 1",
    )
    .bind(team_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取已提交阵型观察");
    assert_eq!(stored_formation_id, expected_formation_id);
    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn monthly_workbooks_preview_commit_clear_and_idempotency_are_consistent() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let team_id = Uuid::new_v4();
    let old_coach_id = Uuid::new_v4();
    let new_coach_id = Uuid::new_v4();
    let team_name = format!("monthly-team-{token}");
    let old_coach_name = format!("monthly-coach-old-{token}");
    let new_coach_name = format!("monthly-coach-new-{token}");
    let team_rows = vec![
        SpreadsheetRawRow {
            sheet_name: "球队基础资料".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::Team,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "official_name": team_name,
                "short_name": "MTH", "team_type": "club", "country_code": "ZZ",
                "city": "Original City", "founded_year": "1999", "stadium": "Original Stadium",
                "is_active": "true", "data_confidence": "0.88",
                "source_urls": "https://example.test/team", "verified_at": "2026-07-17T00:00:00Z"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "球队别名".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::TeamName,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "team_name": team_name,
                "name_value": "Monthly Alias", "language_code": "en",
                "source_urls": "https://example.test/team-alias"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "教练目录".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::Coach,
            action: SpreadsheetAction::Add,
            values: json!({
                "coach_id": old_coach_id.to_string(), "official_name": old_coach_name,
                "nationality_code": "ZZ", "coach_status": "active"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "教练目录".into(),
            row_number: 3,
            entity_type: SpreadsheetEntityType::Coach,
            action: SpreadsheetAction::Add,
            values: json!({
                "coach_id": new_coach_id.to_string(), "official_name": new_coach_name,
                "nationality_code": "ZZ", "coach_status": "active"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "教练任期".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::TeamCoachPeriod,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "team_name": team_name,
                "coach_id": old_coach_id.to_string(), "coach_name": old_coach_name,
                "role": "head_coach", "valid_from": "2025-01-01",
                "is_interim": "false", "confidence": "0.8"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "教练任期".into(),
            row_number: 3,
            entity_type: SpreadsheetEntityType::TeamCoachPeriod,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "team_name": team_name,
                "coach_id": new_coach_id.to_string(), "coach_name": new_coach_name,
                "role": "head_coach", "valid_from": "2026-01-01",
                "is_interim": "false", "confidence": "0.9"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "阵型使用".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::FormationUsage,
            action: SpreadsheetAction::Add,
            values: json!({
                "scope_type": "team", "team_id": team_id.to_string(), "team_name": team_name,
                "formation_code": "4-2-3-1", "window_preset": "last_10",
                "window_start": "2026-01-01", "window_end": "2026-03-01",
                "observed_matches": "10", "usage_count": "6", "confidence": "0.8",
                "alpha": "3", "observed_at": "2026-03-02T00:00:00Z"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "阵型使用".into(),
            row_number: 3,
            entity_type: SpreadsheetEntityType::FormationUsage,
            action: SpreadsheetAction::Add,
            values: json!({
                "scope_type": "team", "team_id": team_id.to_string(), "team_name": team_name,
                "formation_code": "３–４–１–２", "window_preset": "last_10",
                "window_start": "2026-01-01", "window_end": "2026-03-01",
                "observed_matches": "10", "usage_count": "4", "confidence": "0.8",
                "alpha": "3", "observed_at": "2026-03-02T00:00:00Z"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "战术画像".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::TeamTacticalObservation,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "team_name": team_name,
                "coach_id": new_coach_id.to_string(), "coach_name": new_coach_name,
                "window_start": "2026-01-01", "window_end": "2026-03-01",
                "build_up_style": "short", "pressing_intensity": "high",
                "confidence": "0.75", "source_urls": "https://example.test/tactics",
                "observed_at": "2026-03-02T00:00:00Z"
            }),
        },
        SpreadsheetRawRow {
            sheet_name: "球队能力观察".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::TeamAbilityObservation,
            action: SpreadsheetAction::Add,
            values: json!({
                "team_id": team_id.to_string(), "team_name": team_name,
                "observed_at": "2026-03-02T00:00:00Z",
                "window_start": "2026-01-01", "window_end": "2026-03-01",
                "attack_rating": "72.5", "defence_rating": "69.0",
                "sample_size": "10", "methodology": "integration-test",
                "confidence": "0.7", "source_urls": "https://example.test/ability"
            }),
        },
    ];
    let team_workbook = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.into(),
        source_file_name: format!("team-{token}.xlsx"),
        source_sha256: format!("team-{token}"),
        rows: team_rows,
    };
    let preview = database
        .store
        .preview_team_monthly_import(&team_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检球队月度工作簿");
    assert_eq!(preview.counts.error, 0);
    assert_eq!(preview.counts.conflict, 0);
    assert_eq!(preview.counts.ready_end_previous, 2);
    let custom_formation_preview = preview
        .rows
        .iter()
        .find(|row| {
            row.entity_type == SpreadsheetEntityType::FormationUsage && row.row_number == 3
        })
        .expect("自定义阵型预检行");
    assert_eq!(
        custom_formation_preview.payload["formation_code"],
        "3-4-1-2"
    );
    assert!(custom_formation_preview
        .message
        .as_deref()
        .is_some_and(|message| message.contains("登记为自定义阵型")));
    let committed = database
        .store
        .commit_team_monthly_import(preview.batch_id)
        .await
        .expect("提交球队月度工作簿");
    assert_eq!(committed.inserted_count, 10);
    assert_eq!(committed.ended_previous_count, 1);
    let custom_formation_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.formations WHERE code='3-4-1-2' AND NOT is_builtin AND is_active",
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取自动登记的自定义阵型");
    assert_eq!(custom_formation_count, 1);
    let repeated_preview = database
        .store
        .preview_team_monthly_import(&team_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("重复预检应复用批次");
    assert_eq!(repeated_preview.batch_id, preview.batch_id);
    let repeated_commit = database
        .store
        .commit_team_monthly_import(preview.batch_id)
        .await
        .expect("重复提交应返回原结果");
    assert_eq!(repeated_commit.inserted_count, committed.inserted_count);

    let profile = sqlx::query(
        "SELECT team_type, city, stadium, data_confidence FROM football.team_profiles WHERE team_id=$1",
    )
    .bind(team_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取球队资料");
    assert_eq!(profile.try_get::<String, _>("team_type").unwrap(), "club");
    assert_eq!(
        profile
            .try_get::<Option<String>, _>("city")
            .unwrap()
            .as_deref(),
        Some("Original City")
    );
    assert_eq!(
        profile
            .try_get::<Option<String>, _>("stadium")
            .unwrap()
            .as_deref(),
        Some("Original Stadium")
    );
    assert!((profile.try_get::<f64, _>("data_confidence").unwrap() - 0.88).abs() < 1e-9);
    let old_period_end: Option<chrono::NaiveDate> = sqlx::query_scalar(
        "SELECT valid_to FROM football.team_coach_periods WHERE team_id=$1 AND coach_id=$2",
    )
    .bind(team_id)
    .bind(old_coach_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取旧教练任期");
    assert_eq!(
        old_period_end,
        Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
    );

    let clear_workbook = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.into(),
        source_file_name: format!("team-clear-{token}.xlsx"),
        source_sha256: format!("team-clear-{token}"),
        rows: vec![SpreadsheetRawRow {
            sheet_name: "球队基础资料".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::Team,
            action: SpreadsheetAction::Clear,
            values: json!({
                "team_id": team_id.to_string(), "clear_fields": "stadium",
                "source_urls": "https://example.test/team-clear"
            }),
        }],
    };
    let clear_preview = database
        .store
        .preview_team_monthly_import(&clear_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检球队清空动作");
    assert_eq!(clear_preview.counts.ready_update, 1);
    database
        .store
        .commit_team_monthly_import(clear_preview.batch_id)
        .await
        .expect("提交球队清空动作");
    let cleared_profile = sqlx::query(
        "SELECT team_type, city, stadium, data_confidence FROM football.team_profiles WHERE team_id=$1",
    )
    .bind(team_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取清空后的球队资料");
    assert_eq!(
        cleared_profile
            .try_get::<Option<String>, _>("stadium")
            .unwrap(),
        None
    );
    assert_eq!(
        cleared_profile
            .try_get::<Option<String>, _>("city")
            .unwrap()
            .as_deref(),
        Some("Original City")
    );
    assert_eq!(
        cleared_profile.try_get::<String, _>("team_type").unwrap(),
        "club"
    );
    assert!(
        (cleared_profile
            .try_get::<f64, _>("data_confidence")
            .unwrap()
            - 0.88)
            .abs()
            < 1e-9
    );

    let player_id = Uuid::new_v4();
    let player_name = format!("monthly-player-{token}");
    let player_workbook = SpreadsheetParsedWorkbook {
        format_version: PLAYER_MONTHLY_FORMAT.into(),
        source_file_name: format!("player-{token}.xlsx"),
        source_sha256: format!("player-{token}"),
        rows: vec![
            SpreadsheetRawRow {
                sheet_name: "球员基础资料".into(),
                row_number: 2,
                entity_type: SpreadsheetEntityType::Player,
                action: SpreadsheetAction::Add,
                values: json!({
                    "player_key": "P1", "player_id": player_id.to_string(),
                    "official_name": player_name, "birth_date": "2000-01-02",
                    "nationality_code": "ZZ", "preferred_foot": "right",
                    "height_cm": "181", "player_status": "active",
                    "source_urls": "https://example.test/player", "confidence": "0.9"
                }),
            },
            SpreadsheetRawRow {
                sheet_name: "球队履历".into(),
                row_number: 2,
                entity_type: SpreadsheetEntityType::PlayerTeamPeriod,
                action: SpreadsheetAction::Add,
                values: json!({
                    "player_key": "P1", "team_name": format!("placeholder-team-{token}"),
                    "valid_from": "2026-01-01", "registration_status": "registered",
                    "source_urls": "https://example.test/period"
                }),
            },
        ],
    };
    let player_preview = database
        .store
        .preview_spreadsheet_import(&player_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检球员月度工作簿");
    assert_eq!(player_preview.counts.error, 0);
    assert_eq!(player_preview.counts.conflict, 0);
    let player_commit = database
        .store
        .commit_spreadsheet_import(player_preview.batch_id)
        .await
        .expect("提交球员月度工作簿");
    assert_eq!(player_commit.inserted_count, 2);
    let repeated_player_preview = database
        .store
        .preview_spreadsheet_import(&player_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("重复预检球员工作簿");
    assert_eq!(repeated_player_preview.batch_id, player_preview.batch_id);
    let repeated_player_commit = database
        .store
        .commit_spreadsheet_import(player_preview.batch_id)
        .await
        .expect("重复提交球员工作簿应幂等");
    assert_eq!(repeated_player_commit.inserted_count, 2);

    let player_clear = SpreadsheetParsedWorkbook {
        format_version: PLAYER_MONTHLY_FORMAT.into(),
        source_file_name: format!("player-clear-{token}.xlsx"),
        source_sha256: format!("player-clear-{token}"),
        rows: vec![SpreadsheetRawRow {
            sheet_name: "球员基础资料".into(),
            row_number: 2,
            entity_type: SpreadsheetEntityType::Player,
            action: SpreadsheetAction::Clear,
            values: json!({
                "player_id": player_id.to_string(),
                "clear_fields": "nationality_code,height_cm",
                "source_urls": "https://example.test/player-clear"
            }),
        }],
    };
    let player_clear_preview = database
        .store
        .preview_spreadsheet_import(&player_clear, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检球员清空动作");
    assert_eq!(player_clear_preview.counts.ready_update, 1);
    database
        .store
        .commit_spreadsheet_import(player_clear_preview.batch_id)
        .await
        .expect("提交球员清空动作");
    let player = sqlx::query(
        "SELECT canonical_name,date_of_birth,nationality_code,preferred_foot,height_cm,status,metadata FROM football.players WHERE id=$1",
    )
    .bind(player_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取清空后的球员");
    assert_eq!(
        player.try_get::<String, _>("canonical_name").unwrap(),
        player_name
    );
    assert_eq!(
        player
            .try_get::<Option<String>, _>("nationality_code")
            .unwrap(),
        None
    );
    assert_eq!(player.try_get::<Option<i16>, _>("height_cm").unwrap(), None);
    assert_eq!(
        player.try_get::<String, _>("preferred_foot").unwrap(),
        "right"
    );
    assert_eq!(player.try_get::<String, _>("status").unwrap(), "active");
    assert!(player
        .try_get::<Option<chrono::NaiveDate>, _>("date_of_birth")
        .unwrap()
        .is_some());
    let metadata: serde_json::Value = player.try_get("metadata").unwrap();
    assert_eq!(metadata["monthly_workbook"], true);

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn migrations_are_idempotent_and_health_is_connected() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;

    database
        .store
        .migrate()
        .await
        .expect("重复执行迁移应保持幂等");
    let health = database.store.health().await.expect("读取数据库健康状态");
    assert!(health.connected);
    assert!(
        health.migration_count >= 22,
        "迁移数量不足：{}",
        health.migration_count
    );

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn p4_stage_e_source_policy_is_idempotent_versioned_and_immutable() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let draft = SourcePolicyVersionDraft {
        policy_key: format!("p4-stage-e-policy-{token}"),
        version: "1.0.0".to_string(),
        competition_profile_id: None,
        definition: SourcePolicyDefinition {
            schema_version: "football.p4-source-policy.v1".to_string(),
            default_tier: "unclassified".to_string(),
            tiers: vec![
                SourceTierDefinition {
                    key: "unclassified".to_string(),
                    rank: 100,
                },
                SourceTierDefinition {
                    key: "official".to_string(),
                    rank: 500,
                },
            ],
            domain_rules: vec![SourceTierRule {
                domain: "example.test".to_string(),
                tier: "official".to_string(),
            }],
        },
        metadata: json!({"integration_test": true}),
    };
    let first = database
        .store
        .register_source_policy_version(&draft)
        .await
        .expect("登记接入E来源策略");
    let retry = database
        .store
        .register_source_policy_version(&draft)
        .await
        .expect("相同来源策略应幂等复用");
    assert_eq!(first.id, retry.id);

    let mut changed = draft.clone();
    changed.definition.tiers[1].rank = 450;
    assert!(matches!(
        database
            .store
            .register_source_policy_version(&changed)
            .await,
        Err(PersistenceError::InvalidState(_))
    ));

    let mut transaction = database.pool.begin().await.expect("开启不可变性校验事务");
    let mutation = sqlx::query(
        "UPDATE research.source_policy_versions SET version = 'tampered' WHERE id = $1",
    )
    .bind(first.id)
    .execute(&mut *transaction)
    .await;
    assert!(mutation.is_err(), "接入E来源策略版本必须拒绝更新");
    transaction.rollback().await.expect("回滚不可变性校验事务");

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn p4_stage_g_manual_override_ledger_is_unique_and_immutable() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;

    let contract_hash: String = sqlx::query_scalar(
        r#"
        SELECT content_sha256
        FROM platform.integration_contracts
        WHERE contract_key = 'p4-single-match-workbench'
          AND contract_version = '1.0.0'
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取接入G契约登记");
    assert_eq!(
        contract_hash,
        "8ffcc0634d126bcf1ad7dc21a72778c2950b4cc130b338b41dcf871f01feb337"
    );

    let unique_constraints: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM pg_constraint constraint_record
        JOIN pg_class table_record ON table_record.oid = constraint_record.conrelid
        JOIN pg_namespace schema_record ON schema_record.oid = table_record.relnamespace
        WHERE schema_record.nspname = 'research'
          AND table_record.relname = 'manual_route_overrides'
          AND constraint_record.contype = 'u'
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取人工决策唯一约束");
    assert!(
        unique_constraints >= 3,
        "人工决策账本缺少幂等、路由或冲突唯一约束"
    );

    let immutable_trigger_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM pg_trigger trigger_record
            JOIN pg_class table_record ON table_record.oid = trigger_record.tgrelid
            JOIN pg_namespace schema_record ON schema_record.oid = table_record.relnamespace
            WHERE schema_record.nspname = 'research'
              AND table_record.relname = 'manual_route_overrides'
              AND trigger_record.tgname = 'manual_route_overrides_immutable'
              AND NOT trigger_record.tgisinternal
        )
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取人工决策不可变触发器");
    assert!(immutable_trigger_exists);

    let validation_trigger_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM pg_trigger trigger_record
            JOIN pg_class table_record ON table_record.oid = trigger_record.tgrelid
            JOIN pg_namespace schema_record ON schema_record.oid = table_record.relnamespace
            WHERE schema_record.nspname = 'research'
              AND table_record.relname = 'manual_route_overrides'
              AND trigger_record.tgname = 'manual_route_overrides_validate_insert'
              AND NOT trigger_record.tgisinternal
        )
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取人工决策截止与状态门禁触发器");
    assert!(validation_trigger_exists);

    let validation_function: String = sqlx::query_scalar(
        "SELECT pg_get_functiondef('research.validate_manual_route_override_insert()'::regprocedure)",
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取人工决策数据库门禁函数");
    assert!(
        validation_function.contains("original_route_id")
            && validation_function.contains("blocked_conflict")
            && validation_function.contains("route_selected_evidence_ids")
            && validation_function.contains("evidence_conflict_members")
            && validation_function.contains("claim.research_run_id")
            && validation_function.contains("claim.value = NEW.selected_value"),
        "数据库门禁必须锁定原路由、冲突成员、研究运行和事实值归属"
    );

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn external_model_provider_artifact_is_registered_once_and_immutable() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;

    let artifact_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM model.engine_artifacts WHERE engine_key = 'external-model-provider' AND artifact_version = '1.0.0'",
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取外部模型提供器制品账本");
    assert_eq!(artifact_count, 1, "外部模型提供器制品必须幂等登记且仅保留一条");

    let mut transaction = database.pool.begin().await.expect("开启不可变性校验事务");
    let mutation = sqlx::query(
        "UPDATE model.engine_artifacts SET release_version = 'tampered' WHERE engine_key = 'external-model-provider' AND artifact_version = '1.0.0'",
    )
    .execute(&mut *transaction)
    .await;
    assert!(mutation.is_err(), "外部模型提供器制品账本必须拒绝更新");
    transaction.rollback().await.expect("回滚不可变性校验事务");

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("P4C-{token}"),
            name: format!("P4接入C集成测试-{token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建P4接入C测试赛事");
    let home = create_team(&database.store, &format!("P4C主队-{token}")).await;
    let away = create_team(&database.store, &format!("P4C客队-{token}")).await;
    let kickoff = Utc::now() - Duration::days(1);
    let target = create_match(
        &database.store,
        &competition.id,
        &format!("P4C-MATCH-{token}"),
        home.id,
        away.id,
        kickoff,
        MatchStatus::Scheduled,
    )
    .await;

    let schema_draft = SchemaVersionDraft {
        schema_key: format!("p4-stage-c-integration-{token}"),
        version: "1.0.0".to_string(),
        schema_kind: "integration_test".to_string(),
        schema_body: json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "additionalProperties": false
        }),
        description: Some("P4接入C幂等与不可变集成测试".to_string()),
        metadata: json!({"integration_test": true}),
    };
    let schema = database
        .store
        .register_schema_version(&schema_draft)
        .await
        .expect("登记测试Schema版本");
    let schema_retry = database
        .store
        .register_schema_version(&schema_draft)
        .await
        .expect("重复登记相同Schema应幂等");
    assert_eq!(schema.id, schema_retry.id);

    let data_cutoff_at = kickoff - Duration::hours(24);
    let trace_id = Uuid::new_v4();
    let research_draft = ResearchRunDraft {
        match_id: target.id,
        horizon: P4Horizon::T24h,
        data_cutoff_at,
        trace_id,
        idempotency_key: format!("research:{token}:t24h"),
        planner_version: Some("integration-v1".to_string()),
        prompt_version_id: None,
        schema_version_id: schema.id,
        request_payload: json!({"missing_fields": ["lineup"]}),
        metadata: json!({"integration_test": true}),
    };
    let research = database
        .store
        .create_research_run(&research_draft)
        .await
        .expect("创建研究任务");
    let research_retry = database
        .store
        .create_research_run(&research_draft)
        .await
        .expect("相同研究任务重试应返回同一记录");
    assert_eq!(research.id, research_retry.id);
    let mut changed_research = research_draft.clone();
    changed_research.request_payload = json!({"missing_fields": ["lineup", "injury"]});
    assert!(matches!(
        database.store.create_research_run(&changed_research).await,
        Err(PersistenceError::InvalidState(_))
    ));

    let claim_time = data_cutoff_at - Duration::minutes(30);
    let claim_a_draft = EvidenceClaimDraft {
        match_id: target.id,
        entity_type: "team".to_string(),
        entity_id: Some(home.id),
        field_key: "home_lineup_status".to_string(),
        value: json!({"status": "probable"}),
        verification_state: EvidenceVerificationState::Probable,
        source_tier: "official_club".to_string(),
        source_document_id: None,
        source_url: Some(format!("https://example.test/{token}/a")),
        source_title: Some("测试来源A".to_string()),
        source_domain: Some("example.test".to_string()),
        published_at: Some(claim_time),
        observed_at: claim_time,
        effective_at: Some(claim_time),
        retrieved_at: claim_time + Duration::minutes(1),
        timezone: "UTC".to_string(),
        independent_source_count: 1,
        conflict_group_id: None,
        research_run_id: research.id,
        prompt_version_id: None,
        prompt_version: None,
        schema_version_id: schema.id,
        schema_version: "1.0.0".to_string(),
        idempotency_key: format!("evidence:{token}:a"),
        metadata: json!({"integration_test": true}),
    };
    let claim_a = database
        .store
        .append_evidence_claim(&claim_a_draft)
        .await
        .expect("追加证据A");
    let claim_a_retry = database
        .store
        .append_evidence_claim(&claim_a_draft)
        .await
        .expect("重复追加相同证据应幂等");
    assert_eq!(claim_a.id, claim_a_retry.id);
    let mut changed_claim = claim_a_draft.clone();
    changed_claim.value = json!({"status": "confirmed"});
    assert!(matches!(
        database.store.append_evidence_claim(&changed_claim).await,
        Err(PersistenceError::InvalidState(_))
    ));

    let mut claim_b_draft = claim_a_draft.clone();
    claim_b_draft.value = json!({"status": "not_available"});
    claim_b_draft.verification_state = EvidenceVerificationState::Conflict;
    claim_b_draft.source_url = Some(format!("https://example.test/{token}/b"));
    claim_b_draft.source_title = Some("测试来源B".to_string());
    claim_b_draft.idempotency_key = format!("evidence:{token}:b");
    let claim_b = database
        .store
        .append_evidence_claim(&claim_b_draft)
        .await
        .expect("追加证据B");
    let conflict_draft = EvidenceConflictDraft {
        match_id: target.id,
        entity_type: "team".to_string(),
        entity_id: Some(home.id),
        field_key: "home_lineup_status".to_string(),
        conflict_key: format!("conflict:{token}:lineup"),
        evidence_ids: vec![claim_a.id, claim_b.id],
        trace_id,
        metadata: json!({"integration_test": true}),
    };
    let conflict = database
        .store
        .create_evidence_conflict(&conflict_draft)
        .await
        .expect("建立证据冲突组");
    let conflict_retry = database
        .store
        .create_evidence_conflict(&conflict_draft)
        .await
        .expect("重复建立同一冲突组应幂等");
    assert_eq!(conflict.id, conflict_retry.id);

    let descriptor = ModelDescriptor {
        model_id: format!("p4-stage-c-model-{token}"),
        display_name: "P4接入C测试模型".to_string(),
        engine_version: "integration-engine-v1".to_string(),
        supported_competitions: vec![CompetitionKind::League],
        input_schema_version: "integration-input-v1".to_string(),
        output_schema_version: "integration-output-v1".to_string(),
    };
    let package = rule_package_draft(
        &format!("p4-stage-c-rule-{token}"),
        &descriptor.model_id,
        "P4接入C测试规则包",
        &format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
        &format!("integration://p4-stage-c/{token}"),
        json!({"provider_parameters": "opaque"}),
    );
    let registered = database
        .store
        .register_rule_package(&descriptor, &package)
        .await
        .expect("登记P4接入C测试模型版本");
    let ids = sqlx::query(
        r#"
        SELECT model_version_id, parameter_set_id, competition_profile_id
        FROM model.rule_packages WHERE id = $1
        "#,
    )
    .bind(registered.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取快照版本外键");
    use sqlx::Row;
    let model_version_id: Uuid = ids.try_get("model_version_id").expect("模型版本ID");
    let parameter_set_id: Uuid = ids.try_get("parameter_set_id").expect("参数版本ID");
    let competition_profile_id: Uuid = ids
        .try_get("competition_profile_id")
        .expect("赛事Profile版本ID");

    let features = (1_u8..=31)
        .map(|field_order| SnapshotFeatureDraft {
            field_order,
            field_key: format!("field_{field_order:02}"),
            value: if field_order == 1 {
                json!({"status": "probable"})
            } else {
                json!(null)
            },
            verification_state: if field_order == 1 {
                EvidenceVerificationState::Probable
            } else {
                EvidenceVerificationState::NotFound
            },
            evidence_ids: if field_order == 1 {
                vec![claim_a.id]
            } else {
                Vec::new()
            },
            metadata: json!({}),
        })
        .collect::<Vec<_>>();
    let probabilities = ["primary", "secondary"]
        .into_iter()
        .map(|chain_key| SnapshotProbabilityDraft {
            chain_key: chain_key.to_string(),
            home_win: 0.4,
            draw: 0.3,
            away_win: 0.3,
            btts: Some(0.5),
            over_2_5: Some(0.45),
            clean_sheet_home: Some(0.3),
            clean_sheet_away: Some(0.2),
            matrix_sha256: "a".repeat(64),
            matrix_cell_count: 1,
            metadata: json!({}),
        })
        .collect::<Vec<_>>();
    let snapshot_draft = PrematchSnapshotDraft {
        match_id: target.id,
        match_key: target.external_key.clone(),
        horizon: P4Horizon::T24h,
        data_cutoff_at,
        frozen_at: data_cutoff_at + Duration::minutes(5),
        model_version_id,
        parameter_set_id,
        competition_profile_id,
        research_run_id: Some(research.id),
        schema_version_id: schema.id,
        schema_version: "1.0.0".to_string(),
        trace_id,
        idempotency_key: format!("snapshot:{token}:t24h"),
        source_kind: SnapshotSourceKind::Real,
        quality_score: 0.8,
        input_payload: json!({"integration_test": true}),
        features,
        probabilities,
        metadata: json!({"integration_test": true}),
    };
    let snapshot = database
        .store
        .freeze_prematch_snapshot(&snapshot_draft)
        .await
        .expect("冻结P4赛前快照");
    assert!(snapshot.created);
    let snapshot_retry = database
        .store
        .freeze_prematch_snapshot(&snapshot_draft)
        .await
        .expect("相同快照重试应返回同一记录");
    assert_eq!(snapshot.id, snapshot_retry.id);
    assert!(!snapshot_retry.created);
    let bundle = database
        .store
        .read_prematch_snapshot(snapshot.id)
        .await
        .expect("读取完整不可变快照");
    assert_eq!(bundle.features.len(), 31);
    assert_eq!(bundle.probabilities.len(), 4);

    let mut transaction = database.pool.begin().await.expect("开启不可变性校验事务");
    let mutation = sqlx::query(
        "UPDATE feature.snapshots SET input_payload = '{\"tampered\":true}'::jsonb WHERE id = $1",
    )
    .bind(snapshot.id)
    .execute(&mut *transaction)
    .await;
    assert!(mutation.is_err(), "冻结后的赛前载荷必须拒绝更新");
    transaction.rollback().await.expect("回滚快照更新校验事务");

    let mut transaction = database.pool.begin().await.expect("开启明细删除校验事务");
    let deletion = sqlx::query(
        "DELETE FROM feature.snapshot_features WHERE snapshot_id = $1 AND field_order = 1",
    )
    .bind(snapshot.id)
    .execute(&mut *transaction)
    .await;
    assert!(deletion.is_err(), "冻结快照的31字段明细必须拒绝删除");
    transaction.rollback().await.expect("回滚明细删除校验事务");

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn failed_rule_package_registration_rolls_back_all_partial_writes() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let first_source_hash = format!("{token}{token}");
    let second_token = Uuid::new_v4().simple().to_string();
    let second_source_hash = format!("{second_token}{second_token}");
    let package_key = format!("integration-rule-{token}");
    let model_id = format!("integration-model-{token}");
    let descriptor = ModelDescriptor {
        model_id: model_id.clone(),
        display_name: "集成测试模型".to_string(),
        engine_version: "integration-engine-v1".to_string(),
        supported_competitions: vec![CompetitionKind::League],
        input_schema_version: "integration-input-v1".to_string(),
        output_schema_version: "integration-output-v1".to_string(),
    };
    let base_parameters = json!({"integration_parameter": 1.0});
    let first = rule_package_draft(
        &package_key,
        &model_id,
        "初始规则包",
        &first_source_hash,
        &format!("integration://rule/{token}/first"),
        base_parameters.clone(),
    );
    database
        .store
        .register_rule_package(&descriptor, &first)
        .await
        .expect("首次规则包注册应成功");

    let conflicting = rule_package_draft(
        &package_key,
        &model_id,
        "冲突规则包",
        &second_source_hash,
        &format!("integration://rule/{token}/conflict"),
        base_parameters,
    );
    let error = database
        .store
        .register_rule_package(&descriptor, &conflicting)
        .await
        .expect_err("相同规则包版本但内容不同必须拒绝");
    assert!(
        matches!(error, PersistenceError::InvalidState(_)),
        "应返回业务状态错误，实际为：{error}"
    );

    let leaked_source_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM catalog.source_documents WHERE content_sha256 = $1",
    )
    .bind(&second_source_hash)
    .fetch_one(&database.pool)
    .await
    .expect("检查失败事务是否遗留来源文档");
    assert_eq!(
        leaked_source_count, 0,
        "规则包注册失败后不应留下事务内先写入的来源文档"
    );

    let package_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM model.rule_packages WHERE package_key = $1 AND version = '1.0.0'",
    )
    .bind(&package_key)
    .fetch_one(&database.pool)
    .await
    .expect("检查规则包版本数量");
    assert_eq!(package_count, 1, "冲突注册不得生成第二条规则包记录");

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn historical_snapshot_excludes_results_ingested_after_the_cutoff() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("IT-{token}"),
            name: format!("历史截止测试-{token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建测试赛事");
    let home = create_team(&database.store, &format!("截止主队-{token}")).await;
    let away = create_team(&database.store, &format!("截止客队-{token}")).await;
    let valid_opponent = create_team(&database.store, &format!("有效对手-{token}")).await;
    let late_opponent = create_team(&database.store, &format!("晚录对手-{token}")).await;

    let target_kickoff = Utc::now() - Duration::days(2);
    let valid_kickoff = target_kickoff - Duration::days(30);
    let late_kickoff = target_kickoff - Duration::days(15);
    let target = create_match(
        &database.store,
        &competition.id,
        &format!("IT-TARGET-{token}"),
        home.id,
        away.id,
        target_kickoff,
        MatchStatus::Scheduled,
    )
    .await;
    let valid_history = create_match(
        &database.store,
        &competition.id,
        &format!("IT-VALID-{token}"),
        home.id,
        valid_opponent.id,
        valid_kickoff,
        MatchStatus::Finished,
    )
    .await;
    let late_history = create_match(
        &database.store,
        &competition.id,
        &format!("IT-LATE-{token}"),
        home.id,
        late_opponent.id,
        late_kickoff,
        MatchStatus::Finished,
    )
    .await;

    let valid_finalized_at = valid_kickoff + Duration::hours(2);
    let late_finalized_at = late_kickoff + Duration::hours(2);
    sqlx::query(
        r#"
        INSERT INTO football.match_results (
            match_id, home_goals_90, away_goals_90, finalized_at, metadata
        ) VALUES ($1, 2, 0, $2, $3), ($4, 5, 0, $5, $3)
        "#,
    )
    .bind(valid_history.id)
    .bind(valid_finalized_at)
    .bind(json!({"integration_test": true}))
    .bind(late_history.id)
    .bind(late_finalized_at)
    .execute(&database.pool)
    .await
    .expect("写入有效和赛后补录赛果");

    // 只有第一条记录能证明在目标 T-1h 截止前已经入库；第二条保留默认 now()。
    sqlx::query("UPDATE football.match_results SET created_at = $2 WHERE match_id = $1")
        .bind(valid_history.id)
        .bind(valid_finalized_at + Duration::minutes(5))
        .execute(&database.pool)
        .await
        .expect("设置有效赛果的真实入库时间");

    seed_valid_match_lineups(
        &database,
        MatchLineupSeed {
            match_id: target.id,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff: target_kickoff,
            snapshot_type: "T-1h",
            lineup_type: LineupType::Confirmed,
        },
    )
    .await;

    let prepared = database
        .store
        .prepare_match_prediction_input(target.id, "T-1h", "p4")
        .await
        .expect("构建历史 T-1h 推演输入");
    assert_eq!(
        prepared.data_quality["run_mode"].as_str(),
        Some("historical_replay")
    );
    assert_eq!(
        prepared.data_quality["home"]["team_features"]["history_match_count"].as_u64(),
        Some(1),
        "截止时间之后入库的高比分赛果不得进入历史球队特征"
    );
    assert_eq!(
        prepared.data_quality["home"]["team_features"]["baseline_match_count"].as_i64(),
        Some(1),
        "赛事进球基准也必须使用相同的入库截止条件"
    );
    assert_eq!(
        prepared.data_quality["away"]["team_features"]["history_match_count"].as_u64(),
        Some(0)
    );

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_scope_inference_and_lineup_pair_transaction_are_atomic() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("PAIR-{token}"),
            name: format!("双方阵容事务-{token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建双方阵容测试赛事");
    let kickoff = Utc::now() + Duration::days(10);
    let season = database
        .store
        .create_season(&SeasonDraft {
            competition_id: competition.id,
            name: format!("自动赛季-{token}"),
            starts_on: Some((kickoff - Duration::days(30)).date_naive()),
            ends_on: Some((kickoff + Duration::days(30)).date_naive()),
            status: "active".to_string(),
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建自动匹配赛季");
    let home = create_team(&database.store, &format!("事务主队-{token}")).await;
    let away = create_team(&database.store, &format!("事务客队-{token}")).await;
    let target = create_match(
        &database.store,
        &competition.id,
        &format!("PAIR-TARGET-{token}"),
        home.id,
        away.id,
        kickoff,
        MatchStatus::Scheduled,
    )
    .await;
    assert_eq!(
        target.season_id,
        Some(season.id),
        "比赛应自动匹配开球日期所在赛季"
    );

    let formation = database
        .store
        .list_formations(true)
        .await
        .expect("读取阵型目录")
        .into_iter()
        .find(|item| item.code == "4-2-3-1")
        .expect("内置 4-2-3-1 阵型");
    let home_players = create_lineup_player_drafts(
        &database,
        home.id,
        kickoff,
        &format!("pair-home-{token}"),
    )
    .await;
    let mut away_players = create_lineup_player_drafts(
        &database,
        away.id,
        kickoff,
        &format!("pair-away-{token}"),
    )
    .await;
    let valid_away_first = away_players[0].player_id;
    away_players[0].player_id = Uuid::new_v4();
    let captured_at = kickoff - Duration::hours(5);
    let pair = LineupPairDraft {
        home: LineupDraft {
            match_id: target.id,
            team_id: home.id,
            lineup_type: LineupType::Expected,
            snapshot_type: "T-6h".to_string(),
            formation: Some(formation.code.clone()),
            formation_id: Some(formation.id),
            coach_id: None,
            captured_at,
            source_document_id: None,
            source_urls: vec!["https://example.test/pair".to_string()],
            quality_score: Some(0.9),
            metadata: json!({"integration_test": true}),
            players: home_players,
        },
        away: LineupDraft {
            match_id: target.id,
            team_id: away.id,
            lineup_type: LineupType::Expected,
            snapshot_type: "T-6h".to_string(),
            formation: Some(formation.code.clone()),
            formation_id: Some(formation.id),
            coach_id: None,
            captured_at,
            source_document_id: None,
            source_urls: vec!["https://example.test/pair".to_string()],
            quality_score: Some(0.9),
            metadata: json!({"integration_test": true}),
            players: away_players,
        },
    };

    database
        .store
        .create_lineup_pair(&pair)
        .await
        .expect_err("客队球员外键失败时双方事务必须回滚");
    let count_after_failure: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1",
    )
    .bind(target.id)
    .fetch_one(&database.pool)
    .await
    .expect("统计失败后的阵容数量");
    assert_eq!(count_after_failure, 0, "任一侧失败后不得保留另一侧阵容");

    let mut valid_pair = pair;
    valid_pair.away.players[0].player_id = valid_away_first;
    let created = database
        .store
        .create_lineup_pair(&valid_pair)
        .await
        .expect("双方阵容应在一个事务中提交");
    assert_eq!(created.home.team_id, home.id);
    assert_eq!(created.away.team_id, away.id);
    let count_after_success: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1 AND status='active'",
    )
    .bind(target.id)
    .fetch_one(&database.pool)
    .await
    .expect("统计成功后的阵容数量");
    assert_eq!(count_after_success, 2);

    let first_home_id = created.home.id;
    let mut replacement_home = valid_pair.home.clone();
    replacement_home.captured_at = captured_at + Duration::minutes(10);
    let replacement = database
        .store
        .create_lineup(&replacement_home)
        .await
        .expect("创建主队替代阵容");
    let deleted = database
        .store
        .remove_lineup_history(replacement.id, Some("集成测试删除未引用当前版本"))
        .await
        .expect("未引用版本应允许物理删除");
    assert_eq!(deleted.removal_mode, "deleted");
    assert_eq!(deleted.restored_lineup_id, Some(first_home_id));
    let restored = database
        .store
        .read_lineup(first_home_id)
        .await
        .expect("读取自动恢复的上一版本");
    assert_eq!(restored.status, "active");

    replacement_home.captured_at = captured_at + Duration::minutes(20);
    let latest = database
        .store
        .create_lineup(&replacement_home)
        .await
        .expect("再次创建主队替代阵容");
    assert_ne!(latest.id, first_home_id);
    let archived = database
        .store
        .remove_lineup_history(first_home_id, Some("集成测试归档已引用版本"))
        .await
        .expect("已被替代链引用的版本应归档");
    assert_eq!(archived.removal_mode, "archived");
    assert_eq!(archived.restored_lineup_id, None);
    let visible = database
        .store
        .list_lineups(Some(target.id), 20)
        .await
        .expect("读取可见阵容历史");
    assert!(visible.iter().all(|item| item.id != first_home_id));
    assert!(visible.iter().any(|item| item.id == latest.id));

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("LC-{token}"),
            name: format!("阵容闭环-{token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建阵容闭环赛事");
    let home = create_team(&database.store, &format!("闭环主队-{token}")).await;
    let away = create_team(&database.store, &format!("闭环客队-{token}")).await;
    let kickoff = Utc::now() - Duration::days(1);
    let target = create_match(
        &database.store,
        &competition.id,
        &format!("LC-TARGET-{token}"),
        home.id,
        away.id,
        kickoff,
        MatchStatus::Scheduled,
    )
    .await;

    let expected = seed_valid_match_lineups(
        &database,
        MatchLineupSeed {
            match_id: target.id,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff,
            snapshot_type: "T-24h",
            lineup_type: LineupType::Expected,
        },
    )
    .await;
    let confirmed = seed_valid_match_lineups(
        &database,
        MatchLineupSeed {
            match_id: target.id,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff,
            snapshot_type: "T-6h",
            lineup_type: LineupType::Confirmed,
        },
    )
    .await;

    let chain = database
        .store
        .read_match_lineup_chain(target.id, "T-6h")
        .await
        .expect("读取 T-6h 数据窗口阵容链");
    assert!(chain.ready_for_model);
    assert_eq!(chain.home.selected_lineup_id, Some(confirmed.0.id));
    assert_eq!(chain.away.selected_lineup_id, Some(confirmed.1.id));
    assert!(chain
        .home
        .versions
        .iter()
        .any(|item| item.id == expected.0.id));
    assert!(chain
        .home
        .versions
        .iter()
        .any(|item| item.id == confirmed.0.id));

    let latest = seed_valid_match_lineups(
        &database,
        MatchLineupSeed {
            match_id: target.id,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff,
            snapshot_type: "T-N",
            lineup_type: LineupType::Confirmed,
        },
    )
    .await;
    let latest_chain = database
        .store
        .read_match_lineup_chain(target.id, "T-N")
        .await
        .expect("读取 T-N 最新赛前阵容链");
    assert!(latest_chain.ready_for_model);
    assert_eq!(latest_chain.home.selected_lineup_id, Some(latest.0.id));
    assert_eq!(latest_chain.away.selected_lineup_id, Some(latest.1.id));
    let fixed_chain_after_latest = database
        .store
        .read_match_lineup_chain(target.id, "T-6h")
        .await
        .expect("T-6h 数据窗口应读取窗口内最新 T-N 阵容");
    assert_eq!(
        fixed_chain_after_latest.home.selected_lineup_id,
        Some(latest.0.id)
    );
    assert_eq!(
        fixed_chain_after_latest.away.selected_lineup_id,
        Some(latest.1.id)
    );

    let prepared = database
        .store
        .prepare_match_prediction_input(target.id, "T-6h", "p4")
        .await
        .expect("T-6h 窗口内最新阵容进入模型输入");
    assert_eq!(
        prepared.match_input["team_a"]["lineup"]["lineup_id"].as_str(),
        Some(latest.0.id.to_string().as_str())
    );
    assert_eq!(
        prepared.match_input["team_a"]["lineup"]["formation_id"].as_str(),
        latest
            .0
            .formation_id
            .map(|value| value.to_string())
            .as_deref()
    );
    let first_player_id = latest.0.players[0].player_id.to_string();
    assert_eq!(
        prepared.match_input["team_a"]["lineup"]["player_contributions"][0]["player_id"].as_str(),
        Some(first_player_id.as_str())
    );

    let invalid_home = seed_team_lineup(
        &database,
        TeamLineupSeed {
            match_id: target.id,
            team_id: home.id,
            kickoff,
            snapshot_type: "T-1h",
            lineup_type: LineupType::Confirmed,
            starter_count: 10,
            label: "invalid",
        },
    )
    .await;
    assert!(!invalid_home.model_eligible);
    assert_eq!(invalid_home.model_validation_status, "invalid");
    assert!(invalid_home
        .validation_errors
        .iter()
        .any(|item| item.contains("11 名首发")));

    database.close().await;
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn concurrent_workers_claim_once_and_restart_recovery_is_bounded() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;

    // 该测试要求专用数据库；清空任务队列可避免旧失败运行干扰 SKIP LOCKED 断言。
    sqlx::query("DELETE FROM platform.jobs")
        .execute(&database.pool)
        .await
        .expect("清空专用测试任务队列");
    let token = Uuid::new_v4().simple().to_string();
    let future_p4 = database
        .store
        .enqueue_job(&EnqueueJobDraft {
            job_type: "p4_horizon_freeze".to_string(),
            payload: json!({"integration_test": token, "kind": "future"}),
            idempotency_key: Some(format!("integration:p4-future:{token}")),
            available_at: Some(Utc::now() + Duration::minutes(10)),
            priority: i32::MAX,
            max_attempts: 3,
        })
        .await
        .expect("排队未到期P4冻结任务");
    let due_p4 = database
        .store
        .enqueue_job(&EnqueueJobDraft {
            job_type: "p4_horizon_research".to_string(),
            payload: json!({"integration_test": token, "kind": "due"}),
            idempotency_key: Some(format!("integration:p4-due:{token}")),
            available_at: None,
            priority: i32::MAX,
            max_attempts: 3,
        })
        .await
        .expect("排队已到期P4研究任务");
    let queued = database
        .store
        .enqueue_job(&EnqueueJobDraft {
            job_type: "data_quality_scan".to_string(),
            payload: json!({"integration_test": token}),
            idempotency_key: Some(format!("integration:{token}")),
            available_at: None,
            priority: 1,
            max_attempts: 3,
        })
        .await
        .expect("排队集成测试任务");

    let (first, second) = tokio::join!(
        database
            .store
            .claim_next_job_by_types(&["data_quality_scan"]),
        database
            .store
            .claim_next_job_by_types(&["data_quality_scan"])
    );
    let claims = [
        first.expect("Worker A 领取任务"),
        second.expect("Worker B 领取任务"),
    ];
    let claimed = claims.into_iter().flatten().collect::<Vec<_>>();
    assert_eq!(claimed.len(), 1, "同一排队任务只能被一个 Worker 领取");
    assert_eq!(claimed[0].id, queued.id);
    assert_eq!(claimed[0].status, JobStatus::Running);
    assert_eq!(claimed[0].attempts, 1);

    let not_due = database
        .store
        .claim_next_job_by_types(&["p4_horizon_freeze"])
        .await
        .expect("检查未到期P4冻结任务");
    assert!(not_due.is_none(), "available_at未到期的任务不得领取");
    let claimed_p4 = database
        .store
        .claim_next_job_by_types(&["p4_horizon_research"])
        .await
        .expect("领取已到期P4研究任务")
        .expect("已到期P4研究任务应可领取");
    assert_eq!(claimed_p4.id, due_p4.id);
    database
        .store
        .complete_job(claimed_p4.id, json!({"integration_test": true}))
        .await
        .expect("完成P4研究任务");
    assert_eq!(
        database
            .store
            .read_job(future_p4.id)
            .await
            .expect("读取未到期任务")
            .status,
        JobStatus::Queued
    );

    let recovered = database
        .store
        .recover_interrupted_jobs()
        .await
        .expect("恢复第一次中断任务");
    assert_eq!(recovered, 1);
    let requeued = database
        .store
        .read_job(queued.id)
        .await
        .expect("读取重排任务");
    assert_eq!(requeued.status, JobStatus::Queued);
    assert!(requeued.finished_at.is_none());

    let second_claim = database
        .store
        .claim_next_job()
        .await
        .expect("第二次领取任务")
        .expect("恢复后的任务应可重新领取");
    assert_eq!(second_claim.id, queued.id);
    assert_eq!(second_claim.attempts, 2);

    sqlx::query(
        "UPDATE platform.jobs SET attempts = max_attempts, status = 'running', finished_at = NULL WHERE id = $1",
    )
    .bind(queued.id)
    .execute(&database.pool)
    .await
    .expect("模拟达到最大尝试次数后的进程中断");
    let failed_count = database
        .store
        .recover_interrupted_jobs()
        .await
        .expect("恢复达到上限的中断任务");
    assert_eq!(failed_count, 1);
    let failed = database
        .store
        .read_job(queued.id)
        .await
        .expect("读取失败任务");
    assert_eq!(failed.status, JobStatus::Failed);
    assert!(failed.finished_at.is_some());
    assert!(failed
        .error_message
        .as_deref()
        .is_some_and(|message| message.contains("最大尝试次数")));

    database.close().await;
}

fn rule_package_draft(
    package_key: &str,
    model_id: &str,
    display_name: &str,
    content_sha256: &str,
    source_uri: &str,
    parameters: serde_json::Value,
) -> RulePackageDraft {
    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: package_key.to_string(),
        version: "1.0.0".to_string(),
        display_name: display_name.to_string(),
        competition_profile: CompetitionProfile {
            profile_id: format!("{package_key}-profile"),
            name: "集成测试联赛".to_string(),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({"integration_test": true}),
        },
        routing: RuleRouting {
            model_id: model_id.to_string(),
            model_version: "1.0.0".to_string(),
            parameter_version: "1.0.0".to_string(),
            priority: 100,
            activate_as_type_default: false,
            supported_snapshot_types: vec![
                "T-24h".to_string(),
                "T-1h".to_string(),
                "T-N".to_string(),
            ],
        },
        parameters,
        feature_requirements: json!({}),
        output_contract: json!({}),
        source_document: Some(RuleSourceReference {
            title: Some(display_name.to_string()),
            source_uri: Some(source_uri.to_string()),
            content_sha256: Some(content_sha256.to_string()),
            notes: Some("PostgreSQL 事务回滚集成测试".to_string()),
        }),
        metadata: json!({"integration_test": true}),
    }
}

struct MatchLineupSeed<'a> {
    match_id: Uuid,
    home_team_id: Uuid,
    away_team_id: Uuid,
    kickoff: chrono::DateTime<Utc>,
    snapshot_type: &'a str,
    lineup_type: LineupType,
}

struct TeamLineupSeed<'a> {
    match_id: Uuid,
    team_id: Uuid,
    kickoff: chrono::DateTime<Utc>,
    snapshot_type: &'a str,
    lineup_type: LineupType,
    starter_count: usize,
    label: &'a str,
}

async fn seed_valid_match_lineups(
    database: &TestDatabase,
    seed: MatchLineupSeed<'_>,
) -> (football_domain::LineupRecord, football_domain::LineupRecord) {
    let home = seed_team_lineup(
        database,
        TeamLineupSeed {
            match_id: seed.match_id,
            team_id: seed.home_team_id,
            kickoff: seed.kickoff,
            snapshot_type: seed.snapshot_type,
            lineup_type: seed.lineup_type,
            starter_count: 11,
            label: "home",
        },
    )
    .await;
    let away = seed_team_lineup(
        database,
        TeamLineupSeed {
            match_id: seed.match_id,
            team_id: seed.away_team_id,
            kickoff: seed.kickoff,
            snapshot_type: seed.snapshot_type,
            lineup_type: seed.lineup_type,
            starter_count: 11,
            label: "away",
        },
    )
    .await;
    (home, away)
}

async fn create_lineup_player_drafts(
    database: &TestDatabase,
    team_id: Uuid,
    kickoff: chrono::DateTime<Utc>,
    label: &str,
) -> Vec<LineupPlayerDraft> {
    let mut players = Vec::new();
    for index in 0..11usize {
        let player_id = Uuid::new_v4();
        let player_name = format!("{label}-player-{index}-{player_id}");
        sqlx::query(
            "INSERT INTO football.players(id,canonical_name,normalized_name,status,metadata) VALUES($1,$2,$3,'active',$4)",
        )
        .bind(player_id)
        .bind(&player_name)
        .bind(player_name.to_lowercase())
        .bind(json!({"integration_test": true}))
        .execute(&database.pool)
        .await
        .expect("创建阵容球员");
        sqlx::query(
            "INSERT INTO football.player_team_periods(id,player_id,team_id,valid_from,registration_status,metadata) VALUES($1,$2,$3,$4,'registered',$5)",
        )
        .bind(Uuid::new_v4())
        .bind(player_id)
        .bind(team_id)
        .bind((kickoff - Duration::days(30)).date_naive())
        .bind(json!({"integration_test": true}))
        .execute(&database.pool)
        .await
        .expect("创建球员球队履历");
        players.push(LineupPlayerDraft {
            player_id,
            position_code: Some(if index == 0 { "GK" } else { "CM" }.to_string()),
            role_code: None,
            is_starter: true,
            shirt_number: Some((index + 1) as i16),
            expected_minutes: Some(90),
            actual_minutes: None,
            sequence_no: (index + 1) as i16,
            bench_order: None,
            availability_status: None,
            starting_probability: Some(1.0),
            membership_override: false,
            source_urls: vec!["https://example.test/lineup".to_string()],
            metadata: json!({"integration_test": true}),
        });
    }
    players
}

async fn seed_team_lineup(
    database: &TestDatabase,
    seed: TeamLineupSeed<'_>,
) -> football_domain::LineupRecord {
    let formation = database
        .store
        .list_formations(true)
        .await
        .expect("读取阵型目录")
        .into_iter()
        .find(|item| item.code == "4-2-3-1")
        .expect("内置 4-2-3-1 阵型");
    let mut players = create_lineup_player_drafts(
        database,
        seed.team_id,
        seed.kickoff,
        seed.label,
    )
    .await;
    for (index, player) in players.iter_mut().enumerate() {
        player.is_starter = index < seed.starter_count;
        player.expected_minutes = Some(if player.is_starter { 90 } else { 20 });
        player.bench_order = if player.is_starter {
            None
        } else {
            Some((index - seed.starter_count + 1) as i16)
        };
        player.starting_probability = Some(if player.is_starter { 1.0 } else { 0.0 });
    }
    let offset = match seed.snapshot_type {
        "T-24h" => Duration::hours(23),
        "T-6h" => Duration::hours(5),
        "T-1h" => Duration::minutes(50),
        "T-N" => Duration::minutes(30),
        other => panic!("未知测试时点：{other}"),
    };
    let captured_at = seed.kickoff - offset;
    let lineup = database
        .store
        .create_lineup(&LineupDraft {
            match_id: seed.match_id,
            team_id: seed.team_id,
            lineup_type: seed.lineup_type,
            snapshot_type: seed.snapshot_type.to_string(),
            formation: Some(formation.code.clone()),
            formation_id: Some(formation.id),
            coach_id: None,
            captured_at,
            source_document_id: None,
            source_urls: vec!["https://example.test/lineup".to_string()],
            quality_score: Some(0.9),
            metadata: json!({"integration_test": true}),
            players,
        })
        .await
        .expect("创建阵容版本");
    sqlx::query("UPDATE football.lineups SET created_at=$2, updated_at=$2 WHERE id=$1")
        .bind(lineup.id)
        .bind(captured_at)
        .execute(&database.pool)
        .await
        .expect("固定阵容版本创建时点");
    database
        .store
        .read_lineup(lineup.id)
        .await
        .expect("回读阵容版本")
}


#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_review_package_workflow_capabilities_follow_persisted_transitions() {
    let _guard = DATABASE_TEST_LOCK.lock().await;
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("RW-{token}"),
            name: format!("复盘状态机-{token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建复盘状态机赛事");
    let home = create_team(&database.store, &format!("复盘主队-{token}")).await;
    let away = create_team(&database.store, &format!("复盘客队-{token}")).await;
    let kickoff = Utc::now() - Duration::hours(3);
    let target = create_match(
        &database.store,
        &competition.id,
        &format!("RW-TARGET-{token}"),
        home.id,
        away.id,
        kickoff,
        MatchStatus::Finished,
    )
    .await;
    let package_id = Uuid::new_v4();
    let summary = MatchReviewPackageSummary {
        output_path: format!("C:/integration/{package_id}.xlsx"),
        package_id,
        match_id: target.id,
        match_key: target.external_key.clone(),
        lineup_count: 0,
        player_count: 0,
        content_sha256: "a".repeat(64),
        pre_match_snapshot: MatchReviewPackageSnapshotSummary::default(),
        export_database_snapshot: MatchReviewPackageSnapshotSummary::default(),
    };
    let exported = database
        .store
        .register_match_review_package_export(&summary)
        .await
        .expect("登记资料包导出");
    assert_eq!(exported.status, MatchReviewPackageWorkflowStatus::Exported);
    assert!(exported
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::PreviewImport));
    assert_eq!(
        exported.next_action,
        Some(MatchReviewPackageWorkflowAction::PreviewImport)
    );

    let actual_lineup = |team_id| LineupDraft {
        match_id: target.id,
        team_id,
        lineup_type: LineupType::Actual,
        snapshot_type: "actual".to_string(),
        formation: None,
        formation_id: None,
        coach_id: None,
        captured_at: Utc::now(),
        source_document_id: None,
        source_urls: vec!["https://example.test/review".to_string()],
        quality_score: Some(1.0),
        metadata: json!({"integration_test": true}),
        players: Vec::new(),
    };
    let mut preview = MatchReviewPackagePreview {
        source_path: format!("C:/integration/{package_id}-filled.xlsx"),
        source_file_name: format!("{package_id}-filled.xlsx"),
        source_sha256: "b".repeat(64),
        format_version: "football.match-review-package.v1".to_string(),
        package_id,
        match_id: target.id,
        match_key: target.external_key.clone(),
        home_team_name: home.canonical_name.clone(),
        away_team_name: away.canonical_name.clone(),
        lineup_pair: LineupPairDraft {
            home: actual_lineup(home.id),
            away: actual_lineup(away.id),
        },
        review: MatchReviewDraft {
            match_id: target.id,
            review_version: None,
            data_coverage: 1.0,
            source_run_id: None,
            result: MatchResultDraft {
                match_id: target.id,
                home_goals_90: 1,
                away_goals_90: 0,
                home_goals_extra_time: None,
                away_goals_extra_time: None,
                home_penalties: None,
                away_penalties: None,
                finalized_at: Utc::now(),
                source_document_id: None,
                metadata: json!({"integration_test": true}),
            },
            substitutions: Vec::new(),
            events: Vec::new(),
            player_observations: Vec::new(),
            notes: None,
        },
        events: Vec::new(),
        comparison: MatchReviewPackageComparison::default(),
        diff: MatchReviewPackageDiffSummary::default(),
        warnings: Vec::new(),
        errors: vec!["integration blocker".to_string()],
        home_player_count: 0,
        away_player_count: 0,
        home_starter_count: 0,
        away_starter_count: 0,
        substitution_count: 0,
        observation_count: 0,
        ready: false,
    };
    let blocked = database
        .store
        .record_match_review_package_preview(package_id, &preview)
        .await
        .expect("记录阻断预检");
    assert_eq!(
        blocked.status,
        MatchReviewPackageWorkflowStatus::PreviewBlocked
    );
    assert!(blocked
        .completed_steps
        .contains(&MatchReviewPackageWorkflowStep::CompleteExternalData));
    assert!(!blocked
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::ConfirmImport));

    preview.ready = true;
    preview.errors.clear();
    let valid = database
        .store
        .record_match_review_package_preview(package_id, &preview)
        .await
        .expect("记录有效预检");
    assert_eq!(valid.status, MatchReviewPackageWorkflowStatus::PreviewValid);
    assert!(valid
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::ConfirmImport));

    let confirmed = database
        .store
        .confirm_match_review_package_workflow(package_id, Some("integration"), None)
        .await
        .expect("确认资料包");
    assert_eq!(confirmed.status, MatchReviewPackageWorkflowStatus::Confirmed);
    assert!(confirmed
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::CommitFacts));
    assert!(database
        .store
        .record_match_review_package_preview(package_id, &preview)
        .await
        .is_err());

    let committed = database
        .store
        .mark_match_review_package_facts_committed(package_id)
        .await
        .expect("标记事实已写入");
    assert_eq!(
        committed.status,
        MatchReviewPackageWorkflowStatus::FactsCommitted
    );
    assert_eq!(
        committed.next_action,
        Some(MatchReviewPackageWorkflowAction::GenerateReview)
    );
    assert!(database
        .store
        .mark_match_review_package_facts_committed(package_id)
        .await
        .is_err());

    let review_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO review.match_reviews (
            id, match_id, review_version, data_coverage, conclusions
        ) VALUES ($1,$2,$3,1.0,$4)
        "#,
    )
    .bind(review_id)
    .bind(target.id)
    .bind(format!("integration-{token}"))
    .bind(json!({"integration_test": true}))
    .execute(&database.pool)
    .await
    .expect("创建状态机复盘记录");

    let review_created = database
        .store
        .mark_match_review_package_review_created(package_id, review_id)
        .await
        .expect("标记正式复盘已生成");
    assert_eq!(
        review_created.status,
        MatchReviewPackageWorkflowStatus::ReviewCreated
    );
    assert!(review_created
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::InspectSettlementReadiness));
    assert!(review_created
        .allowed_actions
        .contains(&MatchReviewPackageWorkflowAction::SettleReview));
    let by_review = database
        .store
        .read_match_review_package_workflow_by_review(review_id)
        .await
        .expect("按复盘读取工作流")
        .expect("复盘应绑定资料包工作流");
    assert_eq!(by_review.package_id, package_id);

    let settled = database
        .store
        .mark_match_review_package_settled(review_id)
        .await
        .expect("标记正式结算")
        .expect("结算应返回资料包工作流");
    assert_eq!(settled.status, MatchReviewPackageWorkflowStatus::Settled);
    assert_eq!(
        settled.next_action,
        Some(MatchReviewPackageWorkflowAction::OpenAnalytics)
    );
    assert!(settled
        .completed_steps
        .contains(&MatchReviewPackageWorkflowStep::SettleReview));
    let settled_again = database
        .store
        .mark_match_review_package_settled(review_id)
        .await
        .expect("重复结算保持幂等")
        .expect("重复结算仍返回资料包工作流");
    assert_eq!(settled_again.status, MatchReviewPackageWorkflowStatus::Settled);

    database.close().await;
}

async fn create_team(store: &PostgresStore, name: &str) -> football_domain::TeamRecord {
    store
        .create_team(&TeamDraft {
            canonical_name: name.to_string(),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建测试球队")
}

async fn create_match(
    store: &PostgresStore,
    competition_id: &Uuid,
    external_key: &str,
    home_team_id: Uuid,
    away_team_id: Uuid,
    kickoff_time: chrono::DateTime<Utc>,
    status: MatchStatus,
) -> football_domain::MatchRecord {
    store
        .create_match(&MatchDraft {
            external_key: external_key.to_string(),
            competition_id: Some(*competition_id),
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id,
            away_team_id,
            kickoff_time,
            status,
            venue: Some("Integration Stadium".to_string()),
            metadata: json!({"integration_test": true}),
        })
        .await
        .expect("创建测试比赛")
}
