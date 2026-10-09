use chrono::{Duration, Timelike, Utc};
use football_domain::{
    CompetitionDraft, CompetitionKind, CompetitionProfile, ConflictEvaluationDraft,
    ConflictEvaluationStatus, EnqueueJobDraft, EntityResolutionDraft, EntityResolutionStatus,
    EvidenceClaimDraft, EvidenceConflictDraft, EvidenceRouteDraft, EvidenceRouteStatus,
    EvidenceVerificationState, FormationDistributionQuery, FormationUsageDistributionDraft,
    FormationUsageEntryDraft, FormationUsageListQuery, JobStatus, LineupDraft, LineupPairDraft,
    LineupPlayerDraft, LineupType, MatchDraft, MatchEventRevisionStatus, MatchEventType,
    MatchEventVerificationStatus, MatchResultDraft, MatchReviewDraft, MatchReviewPackageComparison,
    MatchReviewPackageDiffSummary, MatchReviewPackagePreview, MatchReviewPackageSnapshotSummary,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowStatus,
    MatchReviewPackageWorkflowStep, MatchStatus, P4FreezeTaskDraft, P4FreezeTaskState,
    P4FreezeTaskTransition, P4Horizon, PrematchSnapshotDraft, ResearchRunDraft, RulePackageDraft,
    RuleRouting, RuleSourceReference, SchemaVersionDraft, SeasonDraft, SnapshotFeatureDraft,
    SnapshotProbabilityDraft, SnapshotSourceKind, SourcePolicyDefinition, SourcePolicyVersionDraft,
    SourceTierDefinition, SourceTierRule, SpreadsheetAction, SpreadsheetEntityType,
    SpreadsheetImportMode, SpreadsheetParsedWorkbook, SpreadsheetRawRow, TeamDraft, TimeAuditDraft,
    TimeAuditStatus, PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT,
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

struct MatchEventFixture<'a> {
    match_id: Uuid,
    event_key: &'a str,
    sequence_no: i32,
    event_type: &'a str,
    team_id: Option<Uuid>,
    player_id: Option<Uuid>,
    minute: i16,
    home_score: Option<i16>,
    away_score: Option<i16>,
    verification_status: &'a str,
    revision_status: &'a str,
}

async fn insert_match_event_fixture(pool: &PgPool, fixture: MatchEventFixture<'_>) {
    let MatchEventFixture {
        match_id,
        event_key,
        sequence_no,
        event_type,
        team_id,
        player_id,
        minute,
        home_score,
        away_score,
        verification_status,
        revision_status,
    } = fixture;
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
    let position_count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM football.positions")
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

    sqlx::query("INSERT INTO model.definitions (id, model_key, display_name) VALUES ($1, $2, $3)")
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
    let result_snapshot = database
        .store
        .read_match_result(match_id)
        .await
        .expect("读取 D2 正式赛果")
        .expect("正式赛果存在");
    sqlx::query(
        r#"
        INSERT INTO review.match_reviews (
            id, match_id, review_version, data_coverage, conclusions,
            status, calculation_version, result_snapshot, prediction_evaluation, finalized_at
        ) VALUES ($1,$2,$3,1.0,'{}'::jsonb,'finalized','integration-d2',
                  $4,'{}'::jsonb,now())
        "#,
    )
    .bind(review_id)
    .bind(match_id)
    .bind(format!("d2-{token}"))
    .bind(serde_json::to_value(&result_snapshot).expect("序列化完整 MatchResultRecord"))
    .execute(&database.pool)
    .await
    .expect("创建 D2 正式复盘");

    insert_match_event_fixture(
        &database.pool,
        MatchEventFixture {
            match_id,
            event_key: "goal:1",
            sequence_no: 1,
            event_type: "goal",
            team_id: Some(home_id),
            player_id: Some(scorer_id),
            minute: 12,
            home_score: Some(1),
            away_score: Some(0),
            verification_status: "verified",
            revision_status: "active",
        },
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        MatchEventFixture {
            match_id,
            event_key: "goal:cancelled",
            sequence_no: 2,
            event_type: "goal",
            team_id: Some(home_id),
            player_id: Some(scorer_id),
            minute: 24,
            home_score: Some(2),
            away_score: Some(0),
            verification_status: "verified",
            revision_status: "cancelled",
        },
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        MatchEventFixture {
            match_id,
            event_key: "card:1",
            sequence_no: 3,
            event_type: "yellow_card",
            team_id: Some(away_id),
            player_id: Some(booked_id),
            minute: 42,
            home_score: None,
            away_score: None,
            verification_status: "disputed",
            revision_status: "active",
        },
    )
    .await;
    insert_match_event_fixture(
        &database.pool,
        MatchEventFixture {
            match_id,
            event_key: "legacy:hidden",
            sequence_no: 4,
            event_type: "other",
            team_id: None,
            player_id: None,
            minute: 50,
            home_score: None,
            away_score: None,
            verification_status: "unverified",
            revision_status: "superseded",
        },
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
    assert_eq!(detail.result.match_id, match_id);
    assert_eq!(detail.result.home_goals_90, 1);
    assert_eq!(detail.result.finalized_at, result_snapshot.finalized_at);
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
    assert_eq!(
        keys,
        vec!["algeria".to_string(), "manchester city".to_string()]
    );

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

    // R7-15：现有生成列继续作为子记录身份的唯一计算来源。
    let team_id = Uuid::new_v4();
    for (entity, payload, expected) in [
        (
            "player_ability",
            json!({"dimension_code":" attack "}),
            "attack".to_string(),
        ),
        (
            "player_ability",
            json!({"dimension_code":"defence"}),
            "defence".to_string(),
        ),
        (
            "player_dynamic_tag",
            json!({"tag_code":" pace "}),
            "pace".to_string(),
        ),
        (
            "player_dynamic_tag",
            json!({"tag_code":"form"}),
            "form".to_string(),
        ),
        ("player", json!({}), String::new()),
        (
            "player_team_period",
            json!({"team_id":team_id,"team_key":"ignored-key","team_name":"ignored name"}),
            team_id.to_string(),
        ),
    ] {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO catalog.import_rows(id,batch_id,sheet_name,row_number,entity_type,requested_action,status,payload) VALUES($1,$2,'球员与评分',2,$3,'skip','skip',$4)")
            .bind(id).bind(batch_id).bind(entity).bind(&payload).execute(&database.pool).await.unwrap();
        let key: String =
            sqlx::query_scalar("SELECT subrecord_key FROM catalog.import_rows WHERE id=$1")
                .bind(id)
                .fetch_one(&database.pool)
                .await
                .unwrap();
        assert_eq!(
            key, expected,
            "生成列保持能力/标签维度、实体及球队 ID 优先语义"
        );
    }

    let token = Uuid::new_v4();
    let mut parsed = SpreadsheetParsedWorkbook {
        format_version: PLAYER_MONTHLY_FORMAT.into(),
        source_file_name: "子记录身份.xlsx".into(),
        source_sha256: format!("subrecord-{token}"),
        rows: vec![],
    };
    // skip 行测试暂存身份，无需创建球员/球队事实；能力、标签和效力记录仍由真实账本插入。
    for (entity_type, values) in [
        (SpreadsheetEntityType::Player, json!({"player_key":"P1"})),
        (
            SpreadsheetEntityType::PlayerAbility,
            json!({"dimension_code":"attack"}),
        ),
        (
            SpreadsheetEntityType::PlayerAbility,
            json!({"dimension_code":"defence"}),
        ),
        (
            SpreadsheetEntityType::PlayerDynamicTag,
            json!({"tag_code":"pace"}),
        ),
        (
            SpreadsheetEntityType::PlayerDynamicTag,
            json!({"tag_code":"form"}),
        ),
        (
            SpreadsheetEntityType::PlayerTeamPeriod,
            json!({"team_key":"national","team_name":"Algeria"}),
        ),
        (
            SpreadsheetEntityType::PlayerTeamPeriod,
            json!({"team_name":" Manchester CITY "}),
        ),
        (
            SpreadsheetEntityType::PlayerPosition,
            json!({"position_code":"ST"}),
        ),
    ] {
        parsed.rows.push(SpreadsheetRawRow {
            sheet_name: "球员与评分".into(),
            row_number: 7,
            entity_type,
            action: SpreadsheetAction::Skip,
            values,
        });
    }
    let mut other_row = parsed.rows[1].clone();
    other_row.row_number = 8;
    parsed.rows.push(other_row);
    let mut other_sheet = parsed.rows[1].clone();
    other_sheet.sheet_name = "其他工作表".into();
    parsed.rows.push(other_sheet);
    let fact_counts_before: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM football.players),(SELECT count(*) FROM football.player_team_periods),(SELECT count(*) FROM audit.events)")
        .fetch_one(&database.pool).await.unwrap();
    let preview = database
        .store
        .preview_spreadsheet_import(&parsed, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    assert_eq!(
        preview.rows.len(),
        10,
        "同物理行的多实体、多能力、多标签和双球队效力子记录全部暂存"
    );
    assert_eq!(preview.counts.skipped, 10);
    let ids: std::collections::HashSet<_> = preview.rows.iter().map(|row| row.id).collect();
    assert_eq!(ids.len(), 10, "暂存行 UUID 不由物理行号复用");
    let reread = database
        .store
        .read_spreadsheet_import_preview(preview.batch_id)
        .await
        .unwrap();
    for row in &preview.rows {
        let stored = reread
            .rows
            .iter()
            .find(|stored| stored.id == row.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(row).unwrap(),
            serde_json::to_value(stored).unwrap(),
            "子记录读回保持原 UUID、工作表、物理行和载荷"
        );
    }
    let subrecords: Vec<(String,String)> = sqlx::query_as("SELECT entity_type,subrecord_key FROM catalog.import_rows WHERE batch_id=$1 AND sheet_name='球员与评分' AND row_number=7 ORDER BY entity_type,subrecord_key")
        .bind(preview.batch_id).fetch_all(&database.pool).await.unwrap();
    assert_eq!(subrecords.len(), 8);
    assert!(
        subrecords.contains(&("player_ability".into(), "attack".into()))
            && subrecords.contains(&("player_ability".into(), "defence".into()))
    );
    assert!(
        subrecords.contains(&("player_dynamic_tag".into(), "pace".into()))
            && subrecords.contains(&("player_dynamic_tag".into(), "form".into()))
    );
    assert!(
        subrecords.contains(&("player_team_period".into(), "national".into()))
            && subrecords.contains(&("player_team_period".into(), "manchester city".into()))
    );
    let ledger_counts_before: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM catalog.import_batches),(SELECT count(*) FROM catalog.import_rows)")
        .fetch_one(&database.pool).await.unwrap();
    let mut duplicate_parsed = parsed.clone();
    duplicate_parsed.source_sha256 = format!("duplicate-subrecord-{token}");
    let mut repeated_dimension = parsed.rows[1].clone();
    repeated_dimension.values["dimension_code"] = json!(" attack ");
    duplicate_parsed.rows.push(repeated_dimension);
    database
        .store
        .preview_spreadsheet_import(&duplicate_parsed, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect_err("重复子记录必须回滚新批次及此前合法行");
    let mut invalid_parsed = parsed.clone();
    invalid_parsed.rows[9].row_number = u32::MAX;
    database
        .store
        .preview_spreadsheet_import(&invalid_parsed, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect_err("非法物理行号不得取消同源 pending 批次");
    invalid_parsed.format_version = TEAM_MONTHLY_FORMAT.into();
    database
        .store
        .preview_team_monthly_import(&invalid_parsed, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect_err("球队预检也须在数据库操作前拒绝非法行号");
    invalid_parsed.format_version = "football.match-lineup.v2".into();
    database
        .store
        .preview_match_lineup_import(&invalid_parsed, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect_err("比赛预检也须在数据库操作前拒绝非法行号");
    let ledger_counts_after: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM catalog.import_batches),(SELECT count(*) FROM catalog.import_rows)")
        .fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        ledger_counts_before, ledger_counts_after,
        "重复子记录整批回滚及非法行号前检不得留下新账本行"
    );
    let pending_status: String =
        sqlx::query_scalar("SELECT status FROM catalog.import_batches WHERE id=$1")
            .bind(preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(pending_status, "pending");
    let retained = database
        .store
        .read_spreadsheet_import_preview(preview.batch_id)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reread).unwrap(),
        serde_json::to_value(&retained).unwrap()
    );
    let fact_counts_after: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM football.players),(SELECT count(*) FROM football.player_team_periods),(SELECT count(*) FROM audit.events)")
        .fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        fact_counts_before, fact_counts_after,
        "身份暂存、失败回滚和前检拒绝不写事实或成功审计"
    );
    sqlx::query("DELETE FROM catalog.import_batches WHERE id=$1")
        .bind(preview.batch_id)
        .execute(&database.pool)
        .await
        .unwrap();

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
    let expected_formation_id: Uuid =
        sqlx::query_scalar("SELECT id FROM football.formations WHERE code='4-2-3-1' AND is_active")
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
        .find(|row| row.entity_type == SpreadsheetEntityType::FormationUsage && row.row_number == 3)
        .expect("自定义阵型预检行");
    assert_eq!(
        custom_formation_preview.payload["formation_code"],
        "3-4-1-2"
    );
    assert!(custom_formation_preview
        .message
        .as_deref()
        .is_some_and(|message| message.contains("登记为自定义阵型")));

    let team_preview_again = database
        .store
        .preview_team_monthly_import(&team_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    assert_eq!(team_preview_again.batch_id, preview.batch_id);
    for row in &preview.rows {
        let repeated = team_preview_again
            .rows
            .iter()
            .find(|other| other.id == row.id)
            .unwrap();
        assert_eq!(repeated.sheet_name, row.sheet_name);
        assert_eq!(repeated.row_number, row.row_number);
        assert_eq!(repeated.payload, row.payload);
    }
    let team_facts_before: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.teams WHERE id=$1")
            .bind(team_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(team_facts_before, 0, "球队重复预检仅暂存行，不写事实");
    let failing_team_row = preview
        .rows
        .iter()
        .find(|row| row.entity_type == SpreadsheetEntityType::TeamAbilityObservation)
        .unwrap();
    let mut invalid_team_payload = failing_team_row.payload.clone();
    invalid_team_payload["attack_rating"] = json!("invalid-rating");
    sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
        .bind(failing_team_row.id)
        .bind(&invalid_team_payload)
        .execute(&database.pool)
        .await
        .unwrap();
    assert!(
        database
            .store
            .commit_team_monthly_import(preview.batch_id)
            .await
            .is_err(),
        "最后业务行失败必须整条球队链回滚"
    );
    let team_facts_after_failure: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.teams WHERE id=$1")
            .bind(team_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(team_facts_after_failure, 0);
    let team_batch_after_failure: (String,i64,i64,bool) = sqlx::query_as("SELECT status,inserted_count,ended_previous_count,finished_at IS NULL FROM catalog.import_batches WHERE id=$1").bind(preview.batch_id).fetch_one(&database.pool).await.unwrap();
    assert_eq!(team_batch_after_failure, ("pending".into(), 0, 0, true));
    let imported_after_failure: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND status='imported'",
    )
    .bind(preview.batch_id)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(imported_after_failure, 0);
    let team_audits_after_failure: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM audit.events WHERE event_type='team_monthly_workbook_imported' AND entity_id=$1").bind(preview.batch_id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(team_audits_after_failure, 0);
    sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
        .bind(failing_team_row.id)
        .bind(&failing_team_row.payload)
        .execute(&database.pool)
        .await
        .unwrap();
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

    let team_audits: Vec<serde_json::Value> = sqlx::query_scalar("SELECT payload FROM audit.events WHERE event_type='team_monthly_workbook_imported' AND entity_id=$1").bind(preview.batch_id.to_string()).fetch_all(&database.pool).await.unwrap();
    assert_eq!(team_audits.len(), 1, "成功重复提交不得重复审计");
    assert_eq!(
        team_audits[0],
        json!({"inserted":committed.inserted_count,"updated":committed.updated_count,"ended_previous":committed.ended_previous_count,"skipped":committed.skipped_count})
    );
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

    // 沿用本夹具验证人工裁决计数；无新测试目标或数据库入口。
    let localized_name = format!("月度球队-{token}");
    let localized_workbook = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.into(),
        source_file_name: format!("team-localized-{token}.xlsx"),
        source_sha256: format!("team-localized-{token}"),
        rows: vec![SpreadsheetRawRow {
            sheet_name: "球队名称".into(),
            row_number: 4,
            entity_type: SpreadsheetEntityType::TeamName,
            action: SpreadsheetAction::Add,
            values: json!({"team_id":team_id.to_string(),"name_value":localized_name,"language_code":"zh-CN","is_primary":"true"}),
        }],
    };
    let localized_preview = database
        .store
        .preview_team_monthly_import(&localized_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    database
        .store
        .commit_team_monthly_import(localized_preview.batch_id)
        .await
        .unwrap();
    let display_name: String =
        sqlx::query_scalar("SELECT canonical_name FROM football.teams WHERE id=$1")
            .bind(team_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(display_name, localized_name);
    for alias in [team_name.as_str(), localized_name.as_str(), "MTH"] {
        let aliases: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM football.team_names WHERE team_id=$1 AND name=$2",
        )
        .bind(team_id)
        .bind(alias)
        .fetch_one(&database.pool)
        .await
        .unwrap();
        assert_eq!(aliases, 1, "原文名、中文主名和简称必须共存且无重复");
    }
    let conflict_workbook = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.into(),
        source_file_name: format!("team-conflict-{token}.xlsx"),
        source_sha256: format!("team-conflict-{token}"),
        rows: (4..6)
            .map(|row_number| SpreadsheetRawRow {
                sheet_name: "球队总览".into(),
                row_number,
                entity_type: SpreadsheetEntityType::Team,
                action: SpreadsheetAction::Update,
                values: json!({"team_id":team_id.to_string(),"city":"After Conflict"}),
            })
            .collect(),
    };
    let conflict_preview = database
        .store
        .preview_team_monthly_import(&conflict_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    assert_eq!(conflict_preview.counts.ready_update, 2);
    let candidate = json!([{"entity_id":team_id,"display_name":localized_name,"detail":null}]);
    sqlx::query(
        "UPDATE catalog.import_rows SET status='conflict',conflict_candidates=$2 WHERE batch_id=$1",
    )
    .bind(conflict_preview.batch_id)
    .bind(&candidate)
    .execute(&database.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE catalog.import_batches SET error_count=2 WHERE id=$1")
        .bind(conflict_preview.batch_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let skipped_row = conflict_preview.rows[0].id;
    let selected_row = conflict_preview.rows[1].id;
    let invalid_selection = football_domain::SpreadsheetImportResolution {
        row_id: selected_row,
        selected_entity_id: Some(Uuid::new_v4()),
        skip: false,
    };
    assert!(database
        .store
        .resolve_team_monthly_import_conflict(conflict_preview.batch_id, invalid_selection)
        .await
        .is_err());
    let unchanged_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(conflict_preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(unchanged_counts, (0, 2), "非法候选不得改变行或计数");
    let skipped_preview = database
        .store
        .resolve_team_monthly_import_conflict(
            conflict_preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: skipped_row,
                selected_entity_id: None,
                skip: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (
            skipped_preview.counts.skipped,
            skipped_preview.counts.conflict
        ),
        (1, 1)
    );
    let skipped_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(conflict_preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(skipped_counts, (1, 1), "跳过与 pending 计数在共同事务提交");
    let selected_preview = database
        .store
        .resolve_team_monthly_import_conflict(
            conflict_preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: selected_row,
                selected_entity_id: Some(team_id),
                skip: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        (
            selected_preview.counts.skipped,
            selected_preview.counts.conflict,
            selected_preview.counts.ready_update
        ),
        (1, 0, 1)
    );
    let selected_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(conflict_preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(selected_counts, (1, 0), "选择候选同步清除批次阻断计数");
    let conflict_commit = database
        .store
        .commit_team_monthly_import(conflict_preview.batch_id)
        .await
        .unwrap();
    assert_eq!(
        (conflict_commit.updated_count, conflict_commit.skipped_count),
        (1, 1)
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
    // R7-11：重复只读预览保留原行 UUID/工作表/物理行/载荷，不生成业务事实。
    let initial_player_rows = serde_json::to_value(&player_preview.rows).unwrap();
    for _ in 0..2 {
        let readback = database
            .store
            .read_spreadsheet_import_preview(player_preview.batch_id)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&readback.rows).unwrap(),
            initial_player_rows,
            "球员预览读回保持行 UUID、工作表、物理行与载荷"
        );
        for raw in &player_workbook.rows {
            assert!(readback
                .rows
                .iter()
                .any(|row| row.sheet_name == raw.sheet_name
                    && row.row_number == raw.row_number
                    && row.entity_type == raw.entity_type));
        }
    }
    let preview_business_count:i64 = sqlx::query_scalar("SELECT (SELECT count(*) FROM football.players WHERE id=$1) + (SELECT count(*) FROM football.player_team_periods WHERE player_id=$1) + (SELECT count(*) FROM football.teams WHERE canonical_name=$2)")
        .bind(player_id).bind(format!("placeholder-team-{token}")).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        preview_business_count, 0,
        "球员预览及重复读回不得写入球员、效力期或自动创建球队"
    );
    let preview_success_audit_count:i64 = sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='spreadsheet_import_committed' AND entity_id=$1")
        .bind(player_preview.batch_id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        preview_success_audit_count, 0,
        "球员只读预览不得产生提交审计"
    );
    // R7-10：复用球员月度批次验证 pending 门禁、冲突阻断及暂存计数。
    for status in ["running", "failed", "cancelled"] {
        sqlx::query("UPDATE catalog.import_batches SET status=$2 WHERE id=$1")
            .bind(player_preview.batch_id)
            .bind(status)
            .execute(&database.pool)
            .await
            .unwrap();
        database
            .store
            .commit_spreadsheet_import(player_preview.batch_id)
            .await
            .expect_err("非 pending 球员批次不得提交");
        let unchanged: (String, i64, i64) = sqlx::query_as(
            "SELECT status,inserted_count,updated_count FROM catalog.import_batches WHERE id=$1",
        )
        .bind(player_preview.batch_id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
        assert_eq!(
            unchanged,
            (status.into(), 0, 0),
            "非法状态不能改写账本或业务计数"
        );
    }
    sqlx::query("UPDATE catalog.import_batches SET status='pending',error_count=1 WHERE id=$1")
        .bind(player_preview.batch_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let player_conflict_row_id = player_preview.rows[0].id;
    sqlx::query("UPDATE catalog.import_rows SET status='conflict',conflict_candidates='[]'::jsonb WHERE id=$1").bind(player_conflict_row_id).execute(&database.pool).await.unwrap();
    database
        .store
        .commit_spreadsheet_import(player_preview.batch_id)
        .await
        .expect_err("未解决球员冲突不能启动业务写入");
    let skipped_preview = database
        .store
        .resolve_spreadsheet_import_conflict(
            player_preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: player_conflict_row_id,
                selected_entity_id: None,
                skip: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(skipped_preview.counts.skipped, 1);
    assert_eq!(skipped_preview.counts.conflict, 0);
    let pending_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(player_preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(pending_counts, (1, 0), "球员冲突跳过与批次暂存计数共同提交");
    database
        .store
        .resolve_spreadsheet_import_conflict(
            player_preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: player_conflict_row_id,
                selected_entity_id: None,
                skip: true,
            },
        )
        .await
        .expect_err("已解决行不能重复处理冲突");
    let superseded_preview_id = player_preview.batch_id;
    let player_preview = database
        .store
        .preview_spreadsheet_import(&player_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    assert_ne!(
        player_preview.batch_id, superseded_preview_id,
        "pending 同源重新预检创建替代批次"
    );
    let previous_status: String =
        sqlx::query_scalar("SELECT status FROM catalog.import_batches WHERE id=$1")
            .bind(superseded_preview_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(previous_status, "cancelled");
    assert_eq!(
        player_preview.counts.error
            + player_preview.counts.conflict
            + player_preview.counts.skipped,
        0
    );
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
    assert_eq!(
        repeated_player_commit.updated_count,
        player_commit.updated_count
    );
    assert_eq!(
        repeated_player_commit.skipped_count,
        player_commit.skipped_count
    );
    assert_eq!(
        repeated_player_commit.ended_previous_count,
        player_commit.ended_previous_count
    );
    let player_ledger: (String,i64,i64,i64,i64,i64) = sqlx::query_as("SELECT status,inserted_count,updated_count,ended_previous_count,skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
        .bind(player_preview.batch_id).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        player_ledger,
        ("succeeded".into(), 2, 0, 0, 0, 0),
        "球员成功返回与账本全部计数一致"
    );
    let player_audits: Vec<serde_json::Value> = sqlx::query_scalar("SELECT payload FROM audit.events WHERE event_type='spreadsheet_import_committed' AND entity_id=$1")
        .bind(player_preview.batch_id.to_string()).fetch_all(&database.pool).await.unwrap();
    assert_eq!(player_audits.len(), 1, "球员成功批次重试不得重复审计");
    assert_eq!(
        player_audits[0],
        json!({"inserted":player_commit.inserted_count,"updated":player_commit.updated_count,"skipped":player_commit.skipped_count})
    );
    assert!(
        database
            .store
            .read_match_lineup_import_preview(player_preview.batch_id)
            .await
            .is_err(),
        "匹配预览不得跨导入类型读取球员批次"
    );

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

    // R7-13：沿用同一月度夹具验证真实聚合、缺口、边界与只读性。
    let monthly_snapshot_sql = r#"SELECT jsonb_build_object(
        'team',(SELECT to_jsonb(t) FROM football.teams t WHERE id=$1),
        'profile',(SELECT to_jsonb(p) FROM football.team_profiles p WHERE team_id=$1),
        'names',(SELECT jsonb_agg(to_jsonb(n) ORDER BY n.id) FROM football.team_names n WHERE team_id=$1),
        'coaches',(SELECT jsonb_agg(to_jsonb(p) ORDER BY p.id) FROM football.team_coach_periods p WHERE team_id=$1),
        'player',(SELECT to_jsonb(p) FROM football.players p WHERE id=$2),
        'periods',(SELECT jsonb_agg(to_jsonb(p) ORDER BY p.id) FROM football.player_team_periods p WHERE player_id=$2),
        'audits',(SELECT count(*) FROM audit.events),
        'batches',(SELECT count(*) FROM catalog.import_batches),
        'rows',(SELECT count(*) FROM catalog.import_rows))"#;
    let monthly_snapshot_before: serde_json::Value = sqlx::query_scalar(monthly_snapshot_sql)
        .bind(team_id)
        .bind(player_id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    let monthly_team_data = database.store.team_monthly_workbook_data().await.unwrap();
    let exported_team = monthly_team_data
        .teams
        .iter()
        .find(|row| row.team_id == team_id)
        .unwrap();
    assert_eq!(exported_team.official_name, localized_name);
    assert_eq!(exported_team.short_name.as_deref(), Some("MTH"));
    assert_eq!(exported_team.city.as_deref(), Some("After Conflict"));
    assert_eq!(exported_team.stadium, None, "显式 clear 在导出中保持空值");
    assert_eq!(exported_team.team_type, "club");
    assert!((exported_team.data_confidence - 0.88).abs() < 1e-9);
    assert!(monthly_team_data
        .names
        .iter()
        .any(|row| row.team_id == team_id && row.name_value == team_name));
    let exported_coaches: Vec<_> = monthly_team_data
        .coach_periods
        .iter()
        .filter(|row| row.team_id == team_id)
        .collect();
    assert_eq!(exported_coaches.len(), 2, "历史与当前教练任期均保留");
    assert_eq!(exported_coaches[0].coach_id, new_coach_id);
    assert_eq!(exported_coaches[1].coach_id, old_coach_id);
    assert_eq!(
        monthly_team_data
            .formation_usage
            .iter()
            .filter(|row| row.team_id == Some(team_id))
            .count(),
        2
    );
    let tactical = monthly_team_data
        .tactical_observations
        .iter()
        .find(|row| row.team_id == team_id)
        .unwrap();
    assert_eq!(tactical.coach_id, Some(new_coach_id));
    assert_eq!(
        tactical.metadata["source_urls"],
        json!(["https://example.test/tactics"])
    );
    let ability = monthly_team_data
        .ability_observations
        .iter()
        .find(|row| row.team_id == team_id)
        .unwrap();
    assert_eq!(ability.attack_rating, Some(72.5));
    assert_eq!(ability.defence_rating, Some(69.0));
    assert_eq!(
        ability.metadata["source_urls"],
        json!(["https://example.test/ability"])
    );
    let monthly_player_data = database.store.spreadsheet_export_data().await.unwrap();
    let exported_player = monthly_player_data
        .players
        .iter()
        .find(|row| row.player_id == player_id)
        .unwrap();
    assert_eq!(exported_player.nationality_code, None);
    assert_eq!(exported_player.height_cm, None);
    let placeholder = monthly_player_data
        .team_periods
        .iter()
        .find(|row| row.player_id == player_id)
        .unwrap();
    let placeholder_id = placeholder.team_id;
    let empty_profile = monthly_team_data
        .teams
        .iter()
        .find(|row| row.team_id == placeholder_id)
        .unwrap();
    assert_eq!(empty_profile.team_type, "club");
    assert_eq!(empty_profile.data_confidence, 0.5);
    assert_eq!(empty_profile.profile_observed_at, None);
    let empty_profile_gaps: Vec<_> = monthly_team_data
        .data_gaps
        .iter()
        .filter(|row| row.entity_id == placeholder_id)
        .collect();
    for field in [
        "profile",
        "country_code",
        "current_coach",
        "formation_usage",
    ] {
        let gap = empty_profile_gaps
            .iter()
            .find(|row| row.missing_field == field)
            .unwrap();
        assert_eq!(gap.entity_type, "team");
        assert_eq!(gap.last_observed_at, None);
        assert_eq!(gap.stale_days, None, "空资料时间不能伪造为零天");
    }
    assert_eq!(empty_profile_gaps[0].priority, "high");
    let monthly_player_gaps = database.store.player_monthly_data_gaps().await.unwrap();
    assert!(monthly_player_gaps
        .iter()
        .any(|row| row.entity_id == player_id
            && row.missing_field == "position"
            && row.priority == "high"));
    assert!(monthly_player_gaps
        .iter()
        .any(|row| row.entity_id == player_id
            && row.missing_field == "nationality_code"
            && row.priority == "medium"));
    assert!(!monthly_player_gaps
        .iter()
        .any(|row| row.entity_id == player_id && row.missing_field == "birth_date"));
    database.store.team_monthly_workbook_data().await.unwrap();
    database.store.spreadsheet_export_data().await.unwrap();
    let monthly_snapshot_after: serde_json::Value = sqlx::query_scalar(monthly_snapshot_sql)
        .bind(team_id)
        .bind(player_id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    assert_eq!(
        monthly_snapshot_after, monthly_snapshot_before,
        "月度读取/缺口及重复导出不得改变事实、账本或审计"
    );

    sqlx::query("UPDATE football.team_coach_periods SET valid_from=current_date,valid_to=current_date WHERE team_id=$1 AND coach_id=$2").bind(team_id).bind(new_coach_id).execute(&database.pool).await.unwrap();
    sqlx::query("UPDATE feature.formation_usage_observations SET observed_at=now()-interval '89 days' WHERE team_id=$1").bind(team_id).execute(&database.pool).await.unwrap();
    let current_team_gaps = database.store.team_monthly_data_gaps().await.unwrap();
    assert!(
        !current_team_gaps.iter().any(|row| row.entity_id == team_id
            && matches!(
                row.missing_field.as_str(),
                "current_coach" | "formation_usage"
            )),
        "当前任期起止日均包含，近期观察不报缺口"
    );
    sqlx::query("UPDATE football.team_coach_periods SET valid_from=current_date+1,valid_to=NULL WHERE team_id=$1 AND coach_id=$2").bind(team_id).bind(new_coach_id).execute(&database.pool).await.unwrap();
    sqlx::query("UPDATE feature.formation_usage_observations SET observed_at=now()-interval '91 days' WHERE team_id=$1").bind(team_id).execute(&database.pool).await.unwrap();
    let expired_team_gaps = database.store.team_monthly_data_gaps().await.unwrap();
    for field in ["current_coach", "formation_usage"] {
        assert!(
            expired_team_gaps
                .iter()
                .any(|row| row.entity_id == team_id && row.missing_field == field),
            "未来任期与超期观察不能消除当前缺口"
        );
    }
    sqlx::query(
        "UPDATE football.team_profiles SET updated_at=now()+interval '2 days' WHERE team_id=$1",
    )
    .bind(team_id)
    .execute(&database.pool)
    .await
    .unwrap();
    let future_profile_gaps = database.store.team_monthly_data_gaps().await.unwrap();
    assert!(
        future_profile_gaps
            .iter()
            .filter(|row| row.entity_id == team_id)
            .all(|row| row.stale_days == Some(0)),
        "未来资料时间的 stale_days 保持非负"
    );

    // 从数据库日期取月界，避免客户端时区决定缺口；原导出保留历史与未来记录。
    sqlx::query("UPDATE football.player_team_periods SET valid_from=(date_trunc('month',current_date)-interval '1 month')::date,valid_to=date_trunc('month',current_date)::date-1 WHERE player_id=$1").bind(player_id).execute(&database.pool).await.unwrap();
    let future_period_id = Uuid::new_v4();
    sqlx::query("INSERT INTO football.player_team_periods(id,player_id,team_id,valid_from,registration_status) VALUES($1,$2,$3,(date_trunc('month',current_date)+interval '1 month')::date,'registered')").bind(future_period_id).bind(player_id).bind(team_id).execute(&database.pool).await.unwrap();
    let month_boundary_gaps = database.store.player_monthly_data_gaps().await.unwrap();
    assert!(
        month_boundary_gaps
            .iter()
            .any(|row| row.entity_id == player_id && row.missing_field == "team_period"),
        "上月已结束与下月待开始履历不充当当前关系"
    );
    let boundary_period_id = Uuid::new_v4();
    sqlx::query("INSERT INTO football.player_team_periods(id,player_id,team_id,valid_from,valid_to,registration_status) VALUES($1,$2,$3,current_date,current_date,'registered')").bind(boundary_period_id).bind(player_id).bind(team_id).execute(&database.pool).await.unwrap();
    let inclusive_period_gaps = database.store.player_monthly_data_gaps().await.unwrap();
    assert!(
        !inclusive_period_gaps
            .iter()
            .any(|row| row.entity_id == player_id && row.missing_field == "team_period"),
        "球员效力期起止日均包含当前日"
    );
    let history_export = database.store.spreadsheet_export_data().await.unwrap();
    let exported_periods: Vec<_> = history_export
        .team_periods
        .iter()
        .filter(|row| row.player_id == player_id)
        .collect();
    assert_eq!(
        exported_periods.len(),
        3,
        "同一球员多球队及历史/当前/未来履历不得过滤或合并"
    );
    assert_eq!(exported_periods[0].team_id, team_id);
    assert_eq!(exported_periods[2].team_id, placeholder_id);
    assert!(exported_periods
        .windows(2)
        .all(|pair| pair[0].valid_from >= pair[1].valid_from));
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
    assert_eq!(
        (first.id, first.created_at, &first.content_sha256),
        (retry.id, retry.created_at, &retry.content_sha256)
    );
    let mut metadata_retry = draft.clone();
    metadata_retry.metadata = json!({"changed": true});
    assert_eq!(
        database
            .store
            .register_source_policy_version(&metadata_retry)
            .await
            .unwrap()
            .id,
        first.id
    );
    let stored_metadata: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM research.source_policy_versions WHERE id=$1")
            .bind(first.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(stored_metadata, draft.metadata);
    let audits: i64 = sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='source_policy_registered' AND entity_id=$1")
        .bind(first.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(audits, 1);
    let mut invalid = draft.clone();
    invalid.policy_key = format!("invalid-{token}");
    invalid.definition.default_tier = "absent".into();
    assert!(matches!(
        database
            .store
            .register_source_policy_version(&invalid)
            .await,
        Err(PersistenceError::InvalidState(_))
    ));
    let residue: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM research.source_policy_versions WHERE policy_key=$1",
    )
    .bind(&invalid.policy_key)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(residue, 0);

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
    assert_eq!(
        artifact_count, 1,
        "外部模型提供器制品必须幂等登记且仅保留一条"
    );

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

    let data_cutoff_at = (kickoff - Duration::hours(24))
        .with_nanosecond(123_456_789)
        .unwrap();
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
    assert_eq!(
        research.data_cutoff_at,
        data_cutoff_at - Duration::nanoseconds(789),
        "研究记录必须反映实际 SQLx 微秒落库精度"
    );
    let mut changed_research = research_draft.clone();
    changed_research.request_payload = json!({"missing_fields": ["lineup", "injury"]});
    assert!(matches!(
        database.store.create_research_run(&changed_research).await,
        Err(PersistenceError::InvalidState(_))
    ));

    // R8-09: reuse the Stage C database and run for the Fact Pipeline owners.
    let pipeline_context = database
        .store
        .fact_pipeline_context(research.id)
        .await
        .unwrap();
    assert_eq!(pipeline_context.research_run_id, research.id);
    assert_eq!(pipeline_context.match_id, target.id);
    assert_eq!(pipeline_context.home_team_id, Some(home.id));
    assert_eq!(pipeline_context.away_team_id, Some(away.id));
    assert_eq!(pipeline_context.schema_version_id, schema.id);
    assert_eq!(pipeline_context.trace_id, trace_id);
    assert_eq!(pipeline_context.data_cutoff_at, research.data_cutoff_at);
    let normalized_home = home.canonical_name.to_lowercase();
    let compact_home: String = normalized_home
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    let home_candidates = database
        .store
        .find_entity_candidates(
            &pipeline_context,
            "team",
            &normalized_home,
            &compact_home,
            None,
        )
        .await
        .unwrap();
    assert!(home_candidates
        .iter()
        .any(|c| c.entity_id == home.id && c.relation.as_deref() == Some("home")));
    assert!(home_candidates
        .windows(2)
        .all(|pair| pair[0].score >= pair[1].score));
    let resolution_draft = EntityResolutionDraft {
        research_run_id: research.id,
        match_id: target.id,
        trace_id,
        fact_key: "fixture.home-team".into(),
        entity_type: "team".into(),
        raw_name: home.canonical_name.clone(),
        normalized_name: normalized_home,
        external_id: None,
        status: EntityResolutionStatus::Resolved,
        resolved_entity_id: Some(home.id),
        resolved_name: Some(home.canonical_name.clone()),
        strategy: "fixture".into(),
        confidence_score: 100,
        candidates: home_candidates,
        reason: "R8-09 fixture".into(),
        idempotency_key: format!("entity:{token}:home"),
    };
    let resolution = database
        .store
        .append_entity_resolution(&resolution_draft)
        .await
        .unwrap();
    let retry = database
        .store
        .append_entity_resolution(&resolution_draft)
        .await
        .unwrap();
    assert_eq!(
        (
            resolution.id,
            resolution.created_at,
            &resolution.resolution_fingerprint
        ),
        (retry.id, retry.created_at, &retry.resolution_fingerprint)
    );
    let mut changed = resolution_draft.clone();
    changed.reason = "changed".into();
    assert!(matches!(
        database.store.append_entity_resolution(&changed).await,
        Err(PersistenceError::InvalidState(_))
    ));
    let time_draft = TimeAuditDraft {
        research_run_id: research.id,
        match_id: target.id,
        trace_id,
        fact_key: resolution_draft.fact_key.clone(),
        field_key: "home_lineup_status".into(),
        data_cutoff_at,
        published_at: Some(data_cutoff_at - Duration::hours(1)),
        observed_at: None,
        effective_at: None,
        retrieved_at: data_cutoff_at,
        timezone: Some("UTC".into()),
        status: TimeAuditStatus::Accepted,
        reason: "R8-09 fixture".into(),
        idempotency_key: format!("time:{token}:home"),
    };
    let time = database.store.append_time_audit(&time_draft).await.unwrap();
    let retry = database.store.append_time_audit(&time_draft).await.unwrap();
    assert_eq!(
        (time.id, time.created_at, &time.time_fingerprint),
        (retry.id, retry.created_at, &retry.time_fingerprint)
    );
    let mut changed = time_draft.clone();
    changed.reason = "changed".into();
    assert!(matches!(
        database.store.append_time_audit(&changed).await,
        Err(PersistenceError::InvalidState(_))
    ));
    let stored_cutoff: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT data_cutoff_at FROM research.time_audits WHERE id=$1")
            .bind(time.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(stored_cutoff, research.data_cutoff_at);

    let claim_time = data_cutoff_at;
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

    let evaluation_draft = ConflictEvaluationDraft {
        conflict_id: conflict.id,
        research_run_id: research.id,
        match_id: target.id,
        trace_id,
        source_policy_key: "fixture".into(),
        source_policy_version: "1.0.0".into(),
        status: ConflictEvaluationStatus::AutoResolved,
        winning_evidence_ids: vec![claim_a.id],
        winning_value: claim_a_draft.value.clone(),
        ranking: json!([{ "rank": 500 }]),
        reason: "R8-09 fixture".into(),
        idempotency_key: format!("evaluation:{token}"),
    };
    let evaluation = database
        .store
        .append_conflict_evaluation(&evaluation_draft)
        .await
        .unwrap();
    let retry = database
        .store
        .append_conflict_evaluation(&evaluation_draft)
        .await
        .unwrap();
    assert_eq!(
        (
            evaluation.id,
            evaluation.created_at,
            &evaluation.evaluation_fingerprint
        ),
        (retry.id, retry.created_at, &retry.evaluation_fingerprint)
    );
    let mut changed = evaluation_draft.clone();
    changed.winning_value = json!({"changed": true});
    assert!(matches!(
        database.store.append_conflict_evaluation(&changed).await,
        Err(PersistenceError::InvalidState(_))
    ));
    let event_key = format!("resolution:{token}");
    let event_payload = json!({"winning_evidence_ids": [claim_a.id]});
    for _ in 0..2 {
        database
            .store
            .append_conflict_event(
                conflict.id,
                "resolved",
                "fixture",
                &event_payload,
                &event_key,
            )
            .await
            .unwrap();
    }
    assert!(matches!(
        database
            .store
            .append_conflict_event(conflict.id, "resolved", "fixture", &json!({}), &event_key)
            .await,
        Err(PersistenceError::InvalidState(_))
    ));
    let event_count: i64 = sqlx::query_scalar("SELECT count(*) FROM research.evidence_conflict_events WHERE conflict_id=$1 AND idempotency_key=$2")
        .bind(conflict.id).bind(&event_key).fetch_one(&database.pool).await.unwrap();
    assert_eq!(event_count, 1);
    let route_draft = EvidenceRouteDraft {
        research_run_id: research.id,
        match_id: target.id,
        trace_id,
        route_key: format!("fixture:home:{token}"),
        field_key: "home_lineup_status".into(),
        target_module: "fixture".into(),
        target_slot: "home".into(),
        route_registry_version: football_domain::P4_EVIDENCE_ROUTE_VERSION.into(),
        entity_type: Some("team".into()),
        entity_id: Some(home.id),
        status: EvidenceRouteStatus::Routed,
        verification_state: "CONFIRMED".into(),
        selected_evidence_ids: vec![claim_b.id, claim_a.id, claim_a.id],
        selected_value: claim_a_draft.value.clone(),
        reason: "R8-09 fixture".into(),
        idempotency_key: format!("route:{token}"),
    };
    let route = database
        .store
        .append_evidence_route(&route_draft)
        .await
        .unwrap();
    let mut reordered_route = route_draft.clone();
    reordered_route.selected_evidence_ids = vec![claim_a.id, claim_b.id];
    let retry = database
        .store
        .append_evidence_route(&reordered_route)
        .await
        .unwrap();
    assert_eq!(
        (route.id, route.created_at, &route.route_fingerprint),
        (retry.id, retry.created_at, &retry.route_fingerprint)
    );
    let mut changed = route_draft.clone();
    changed.reason = "changed".into();
    assert!(matches!(
        database.store.append_evidence_route(&changed).await,
        Err(PersistenceError::InvalidState(_))
    ));
    let stored_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT selected_evidence_ids FROM research.evidence_routes WHERE id=$1",
    )
    .bind(route.id)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    let mut expected_ids = vec![claim_a.id, claim_b.id];
    expected_ids.sort_unstable();
    assert_eq!(stored_ids, expected_ids);
    for (table, id) in [
        ("entity_resolutions", resolution.id),
        ("time_audits", time.id),
        ("conflict_evaluations", evaluation.id),
        ("evidence_routes", route.id),
    ] {
        let mut tx = database.pool.begin().await.unwrap();
        let mutation = sqlx::query(&format!(
            "UPDATE research.{table} SET reason='tampered' WHERE id=$1"
        ))
        .bind(id)
        .execute(&mut *tx)
        .await;
        assert!(mutation.is_err(), "{table} must remain immutable");
        tx.rollback().await.unwrap();
    }

    // R8-08: use the existing Stage C fixture for the evidence ledger contract.
    let stored_claim: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(c) FROM research.evidence_claims c WHERE id=$1")
            .bind(claim_a.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    let draft_json = serde_json::to_value(&claim_a_draft).unwrap();
    for field in [
        "match_id",
        "entity_type",
        "entity_id",
        "field_key",
        "value",
        "verification_state",
        "source_tier",
        "source_document_id",
        "source_url",
        "source_title",
        "source_domain",
        "timezone",
        "independent_source_count",
        "conflict_group_id",
        "research_run_id",
        "prompt_version_id",
        "prompt_version",
        "schema_version_id",
        "schema_version",
        "idempotency_key",
        "metadata",
    ] {
        assert_eq!(
            stored_claim[field], draft_json[field],
            "claim field {field} must survive unchanged"
        );
    }
    assert_eq!(
        stored_claim["content_sha256"],
        json!(claim_a.content_sha256)
    );
    assert_eq!(
        stored_claim["claim_fingerprint"],
        json!(claim_a.claim_fingerprint)
    );
    for (field, expected) in [
        ("published_at", claim_a_draft.published_at.unwrap()),
        ("observed_at", claim_a_draft.observed_at),
        ("effective_at", claim_a_draft.effective_at.unwrap()),
        ("retrieved_at", claim_a_draft.retrieved_at),
    ] {
        let stored =
            chrono::DateTime::parse_from_rfc3339(stored_claim[field].as_str().unwrap()).unwrap();
        assert_eq!(
            stored,
            expected - Duration::nanoseconds(789),
            "database retains its original microsecond precision"
        );
    }
    for change_metadata in [true, false] {
        let mut changed = claim_a_draft.clone();
        if change_metadata {
            changed.metadata = json!({"changed":true});
        } else {
            changed.observed_at += Duration::nanoseconds(1);
        }
        assert!(
            matches!(
                database.store.append_evidence_claim(&changed).await,
                Err(PersistenceError::InvalidState(_))
            ),
            "original claim fingerprint must reject metadata/raw nanosecond drift"
        );
    }
    let mut reordered = conflict_draft.clone();
    reordered.evidence_ids = vec![claim_b.id, claim_a.id, claim_b.id];
    reordered.metadata = json!({"retry_metadata":"ignored by existing fingerprint"});
    assert_eq!(
        database
            .store
            .create_evidence_conflict(&reordered)
            .await
            .unwrap()
            .id,
        conflict.id
    );
    reordered.trace_id = Uuid::new_v4();
    assert!(matches!(
        database.store.create_evidence_conflict(&reordered).await,
        Err(PersistenceError::InvalidState(_))
    ));

    let audit_before_bad_claims: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit.events WHERE event_type='evidence_claim_appended' AND payload->>'research_run_id'=$1",
    ).bind(research.id.to_string()).fetch_one(&database.pool).await.unwrap();
    for bad_kind in [
        "run",
        "match",
        "schema_version",
        "prompt_pair",
        "prompt_id",
        "conflict_missing",
        "conflict_identity",
        "source_fk",
    ] {
        let mut bad = claim_a_draft.clone();
        bad.idempotency_key = format!("evidence:{token}:bad-{bad_kind}");
        match bad_kind {
            "run" => bad.research_run_id = Uuid::new_v4(),
            "match" => bad.match_id = Uuid::new_v4(),
            "schema_version" => bad.schema_version = "wrong-version".into(),
            "prompt_pair" => bad.prompt_version = Some("unpaired-version".into()),
            "prompt_id" => {
                bad.prompt_version_id = Some(Uuid::new_v4());
                bad.prompt_version = Some("missing-version".into());
            }
            "conflict_missing" => bad.conflict_group_id = Some(Uuid::new_v4()),
            "conflict_identity" => {
                bad.conflict_group_id = Some(conflict.id);
                bad.field_key = "another-field".into();
            }
            "source_fk" => bad.source_document_id = Some(Uuid::new_v4()),
            _ => unreachable!(),
        }
        assert!(
            database.store.append_evidence_claim(&bad).await.is_err(),
            "{bad_kind}"
        );
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM research.evidence_claims WHERE idempotency_key=$1",
        )
        .bind(&bad.idempotency_key)
        .fetch_one(&database.pool)
        .await
        .unwrap();
        assert_eq!(count, 0, "rejected reference/SQL write must leave no claim");
    }
    let audit_after_bad_claims: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM audit.events WHERE event_type='evidence_claim_appended' AND payload->>'research_run_id'=$1",
    ).bind(research.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        audit_after_bad_claims, audit_before_bad_claims,
        "rejected reference/SQL write must leave no audit"
    );
    let mut concurrent_claim = claim_a_draft.clone();
    concurrent_claim.idempotency_key = format!("evidence:{token}:concurrent");
    let (left, right) = tokio::join!(
        database.store.append_evidence_claim(&concurrent_claim),
        database.store.append_evidence_claim(&concurrent_claim)
    );
    let concurrent = left.unwrap();
    assert_eq!(
        concurrent.id,
        right.unwrap().id,
        "same-key first writers share the original transaction lock"
    );
    for id in [claim_a.id, claim_b.id, concurrent.id] {
        let count:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='evidence_claim_appended' AND entity_id=$1")
            .bind(id.to_string()).fetch_one(&database.pool).await.unwrap();
        assert_eq!(
            count, 1,
            "retry/rejection/concurrent claim must not duplicate successful audit"
        );
    }
    let mut other_field = claim_a_draft.clone();
    other_field.idempotency_key = format!("evidence:{token}:other-field");
    other_field.field_key = "injury".into();
    let other_claim = database
        .store
        .append_evidence_claim(&other_field)
        .await
        .unwrap();
    for bad_kind in ["missing", "identity", "duplicates"] {
        let mut bad = conflict_draft.clone();
        bad.conflict_key = format!("conflict:{token}:bad-{bad_kind}");
        bad.evidence_ids = match bad_kind {
            "missing" => vec![claim_a.id, Uuid::new_v4()],
            "identity" => vec![claim_a.id, other_claim.id],
            "duplicates" => vec![claim_a.id, claim_a.id],
            _ => unreachable!(),
        };
        assert!(database.store.create_evidence_conflict(&bad).await.is_err());
        let counts:(i64,i64,i64,i64)=sqlx::query_as(r#"
            SELECT (SELECT count(*) FROM research.evidence_conflicts WHERE conflict_key=$1),
                   (SELECT count(*) FROM research.evidence_conflict_members m JOIN research.evidence_conflicts c ON c.id=m.conflict_id WHERE c.conflict_key=$1),
                   (SELECT count(*) FROM research.evidence_conflict_events e JOIN research.evidence_conflicts c ON c.id=e.conflict_id WHERE c.conflict_key=$1),
                   (SELECT count(*) FROM audit.events WHERE event_type='evidence_conflict_opened' AND payload->>'conflict_key'=$1)
        "#).bind(&bad.conflict_key).fetch_one(&database.pool).await.unwrap();
        assert_eq!(
            counts,
            (0, 0, 0, 0),
            "failed conflict leaves no header/member/event/audit"
        );
    }
    let mut concurrent_conflict = conflict_draft.clone();
    concurrent_conflict.conflict_key = format!("conflict:{token}:concurrent");
    let (left, right) = tokio::join!(
        database
            .store
            .create_evidence_conflict(&concurrent_conflict),
        database
            .store
            .create_evidence_conflict(&concurrent_conflict)
    );
    let concurrent = left.unwrap();
    assert_eq!(concurrent.id, right.unwrap().id);
    for id in [conflict.id, concurrent.id] {
        let counts:(i64,i64,i64)=sqlx::query_as(r#"
            SELECT (SELECT count(*) FROM research.evidence_conflict_members WHERE conflict_id=$1),
                   (SELECT count(*) FROM research.evidence_conflict_events WHERE conflict_id=$1 AND event_type='opened'),
                   (SELECT count(*) FROM audit.events WHERE event_type='evidence_conflict_opened' AND entity_id=$2)
        "#).bind(id).bind(id.to_string()).fetch_one(&database.pool).await.unwrap();
        assert_eq!(
            counts,
            (2, 1, 1),
            "successful/retried/concurrent conflict has one atomic opened event and audit"
        );
    }
    for (sql, id) in [
        (
            "UPDATE research.evidence_claims SET value='{}'::jsonb WHERE id=$1",
            claim_a.id,
        ),
        (
            "DELETE FROM research.evidence_conflicts WHERE id=$1",
            conflict.id,
        ),
        (
            "DELETE FROM research.evidence_conflict_members WHERE conflict_id=$1",
            conflict.id,
        ),
        (
            "UPDATE research.evidence_conflict_events SET payload='{}'::jsonb WHERE conflict_id=$1",
            conflict.id,
        ),
    ] {
        assert!(
            sqlx::query(sql)
                .bind(id)
                .execute(&database.pool)
                .await
                .is_err(),
            "evidence ledger must remain append-only"
        );
    }

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

    // R8-10: reuse this real database/fixture to exercise task identity and the event/audit transaction.
    let planning_context = database
        .store
        .p4_planning_match_context(target.id)
        .await
        .unwrap();
    assert_eq!(
        (
            planning_context.match_id,
            planning_context.match_key.as_str(),
            planning_context.competition_kind
        ),
        (
            target.id,
            target.external_key.as_str(),
            CompetitionKind::League
        )
    );
    assert_eq!(
        database
            .store
            .read_schema_version_by_key(&schema_draft.schema_key, &schema_draft.version)
            .await
            .unwrap()
            .id,
        schema.id
    );
    assert_eq!(
        database
            .store
            .read_research_run(research.id)
            .await
            .unwrap()
            .id,
        research.id
    );
    let horizon_draft = P4FreezeTaskDraft {
        match_id: target.id,
        match_key: target.external_key.clone(),
        horizon: P4Horizon::T24h,
        kickoff_at: planning_context.kickoff_at,
        data_cutoff_at,
        research_due_at: data_cutoff_at - Duration::minutes(15),
        freeze_deadline_at: data_cutoff_at + Duration::minutes(15),
        rule_package_id: registered.id,
        model_version_id,
        parameter_set_id,
        competition_profile_id,
        research_schema_version_id: schema.id,
        snapshot_schema_version_id: schema.id,
        requested_fact_keys: vec!["injury".into(), "lineup".into(), "lineup".into()],
        trace_id: Uuid::new_v4(),
        state: P4FreezeTaskState::Planned,
        idempotency_key: format!("horizon:{token}:t24h"),
        metadata: json!({"integration_test":true}),
    };
    let horizon_task = database
        .store
        .create_p4_freeze_task(&horizon_draft)
        .await
        .unwrap();
    let mut reordered = horizon_draft.clone();
    reordered.requested_fact_keys = vec!["lineup".into(), "injury".into()];
    let retry = database
        .store
        .create_p4_freeze_task(&reordered)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&horizon_task).unwrap(),
        serde_json::to_value(&retry).unwrap()
    );
    assert_eq!(horizon_task.requested_fact_keys, ["injury", "lineup"]);
    assert_eq!(
        horizon_task.data_cutoff_at,
        data_cutoff_at - Duration::nanoseconds(789)
    );
    assert_eq!(
        database
            .store
            .find_p4_freeze_task_by_idempotency(&horizon_draft.idempotency_key)
            .await
            .unwrap()
            .unwrap()
            .id,
        horizon_task.id
    );
    assert_eq!(
        database
            .store
            .read_p4_freeze_task(horizon_task.id)
            .await
            .unwrap()
            .id,
        horizon_task.id
    );
    for change in ["metadata", "trace", "nanosecond"] {
        let mut changed = horizon_draft.clone();
        match change {
            "metadata" => changed.metadata = json!({"changed":true}),
            "trace" => changed.trace_id = Uuid::new_v4(),
            _ => changed.data_cutoff_at += Duration::nanoseconds(1),
        }
        assert!(matches!(
            database.store.create_p4_freeze_task(&changed).await,
            Err(PersistenceError::InvalidState(_))
        ));
    }
    for horizon in [P4Horizon::T90m, P4Horizon::LegacyTN] {
        let mut bad = horizon_draft.clone();
        bad.horizon = horizon;
        bad.idempotency_key = format!("horizon:{token}:{}", horizon.as_str());
        bad.trace_id = Uuid::new_v4();
        assert!(database.store.create_p4_freeze_task(&bad).await.is_err());
        assert!(database
            .store
            .find_p4_freeze_task_by_idempotency(&bad.idempotency_key)
            .await
            .unwrap()
            .is_none());
    }
    let mut concurrent_draft = horizon_draft.clone();
    concurrent_draft.horizon = P4Horizon::T6h;
    concurrent_draft.data_cutoff_at = planning_context.kickoff_at - Duration::hours(6);
    concurrent_draft.research_due_at = concurrent_draft.data_cutoff_at - Duration::minutes(15);
    concurrent_draft.freeze_deadline_at = concurrent_draft.data_cutoff_at + Duration::minutes(15);
    concurrent_draft.idempotency_key = format!("horizon:{token}:concurrent");
    concurrent_draft.trace_id = Uuid::new_v4();
    let (left, right) = tokio::join!(
        database.store.create_p4_freeze_task(&concurrent_draft),
        database.store.create_p4_freeze_task(&concurrent_draft)
    );
    let concurrent_task = left.unwrap();
    assert_eq!(concurrent_task.id, right.unwrap().id);
    let mut rollback = concurrent_draft.clone();
    rollback.horizon = P4Horizon::T1h;
    rollback.data_cutoff_at = planning_context.kickoff_at - Duration::hours(1);
    rollback.research_due_at = rollback.data_cutoff_at - Duration::minutes(15);
    rollback.freeze_deadline_at = rollback.data_cutoff_at + Duration::minutes(15);
    rollback.idempotency_key = format!("horizon:{token}:rollback");
    rollback.trace_id = Uuid::new_v4();
    let valid_rule = rollback.rule_package_id;
    rollback.rule_package_id = Uuid::new_v4();
    assert!(database
        .store
        .create_p4_freeze_task(&rollback)
        .await
        .is_err());
    assert!(database
        .store
        .find_p4_freeze_task_by_idempotency(&rollback.idempotency_key)
        .await
        .unwrap()
        .is_none());
    let audit_before:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='p4_freeze_task_created' AND payload->>'match_id'=$1")
        .bind(target.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(audit_before, 2);
    rollback.rule_package_id = valid_rule;
    let restored = database
        .store
        .create_p4_freeze_task(&rollback)
        .await
        .unwrap();
    let queue_job = database
        .store
        .enqueue_job(&EnqueueJobDraft {
            job_type: "p4_horizon_research".into(),
            payload: json!({"task_id":horizon_task.id}),
            idempotency_key: Some(format!("p4-research-job:{}", horizon_task.id)),
            available_at: Some(horizon_task.research_due_at),
            priority: 10,
            max_attempts: 3,
        })
        .await
        .unwrap();
    let transition = P4FreezeTaskTransition {
        task_id: horizon_task.id,
        expected_state: P4FreezeTaskState::Planned,
        next_state: P4FreezeTaskState::ResearchQueued,
        reason: "integration queued".into(),
        blockers: json!(null),
        payload: json!({"job_id":queue_job.id}),
        research_run_id: Some(research.id),
        research_job_id: Some(queue_job.id),
        freeze_job_id: None,
        snapshot_id: None,
    };
    let queued = database
        .store
        .transition_p4_freeze_task(&transition)
        .await
        .unwrap();
    assert_eq!(
        (queued.research_job_id, queued.research_run_id),
        (Some(queue_job.id), Some(research.id))
    );
    let events = database
        .store
        .list_p4_freeze_task_events(horizon_task.id)
        .await
        .unwrap();
    assert_eq!(events.len(), 2);
    let same = database
        .store
        .transition_p4_freeze_task(&transition)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&queued).unwrap(),
        serde_json::to_value(&same).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&events).unwrap(),
        serde_json::to_value(
            database
                .store
                .list_p4_freeze_task_events(horizon_task.id)
                .await
                .unwrap()
        )
        .unwrap()
    );
    for invalid in ["expected", "illegal", "foreign_key"] {
        let mut rejected = transition.clone();
        rejected.next_state = P4FreezeTaskState::ResearchRunning;
        match invalid {
            "expected" => {}
            "illegal" => {
                rejected.expected_state = P4FreezeTaskState::ResearchQueued;
                rejected.next_state = P4FreezeTaskState::Frozen;
            }
            _ => {
                rejected.expected_state = P4FreezeTaskState::ResearchQueued;
                rejected.research_run_id = Some(Uuid::new_v4());
            }
        }
        assert!(database
            .store
            .transition_p4_freeze_task(&rejected)
            .await
            .is_err());
        assert_eq!(
            serde_json::to_value(&queued).unwrap(),
            serde_json::to_value(
                database
                    .store
                    .read_p4_freeze_task(horizon_task.id)
                    .await
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            database
                .store
                .list_p4_freeze_task_events(horizon_task.id)
                .await
                .unwrap()
                .len(),
            2
        );
    }
    assert_eq!(events[0].from_state, None);
    assert_eq!(events[0].to_state, P4FreezeTaskState::Planned);
    assert_eq!(events[1].from_state, Some(P4FreezeTaskState::Planned));
    assert_eq!(events[1].to_state, P4FreezeTaskState::ResearchQueued);
    let ordered = database
        .store
        .list_p4_freeze_tasks(Some(target.id), 500)
        .await
        .unwrap();
    assert_eq!(
        ordered.iter().map(|task| task.horizon).collect::<Vec<_>>(),
        P4Horizon::CANONICAL
    );
    for task in [&horizon_task, &concurrent_task, &restored] {
        let count:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='p4_freeze_task_created' AND entity_id=$1")
            .bind(task.id.to_string()).fetch_one(&database.pool).await.unwrap();
        assert_eq!(count, 1);
    }
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='p4_freeze_task_transitioned' AND entity_id=$1")
        .bind(horizon_task.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(count, 1);
    assert!(sqlx::query(
        "UPDATE platform.p4_freeze_task_events SET reason='changed' WHERE task_id=$1"
    )
    .bind(horizon_task.id)
    .execute(&database.pool)
    .await
    .is_err());
    assert!(
        sqlx::query("UPDATE platform.p4_freeze_tasks SET trace_id=$2 WHERE id=$1")
            .bind(horizon_task.id)
            .bind(Uuid::new_v4())
            .execute(&database.pool)
            .await
            .is_err()
    );

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
    let probabilities = ["primary", "secondary", "conservative", "full"]
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
    let mut mismatched_cutoff = snapshot_draft.clone();
    mismatched_cutoff.data_cutoff_at += Duration::microseconds(1);
    assert!(
        matches!(
            database
                .store
                .freeze_prematch_snapshot(&mismatched_cutoff)
                .await,
            Err(PersistenceError::InvalidState(_))
        ),
        "真实 1 微秒差异不得被容差放过"
    );
    for field in ["published_at", "effective_at"] {
        let mut future_claim_draft = claim_a_draft.clone();
        future_claim_draft.idempotency_key = format!("evidence:{token}:future-{field}");
        if field == "published_at" {
            future_claim_draft.published_at = Some(data_cutoff_at + Duration::microseconds(1));
        } else {
            future_claim_draft.effective_at = Some(data_cutoff_at + Duration::microseconds(1));
        }
        let future_claim = database
            .store
            .append_evidence_claim(&future_claim_draft)
            .await
            .expect("保留未来证据以验证冻结时的截止门禁");
        let mut future_snapshot = snapshot_draft.clone();
        future_snapshot.idempotency_key = format!("snapshot:{token}:future-{field}");
        future_snapshot.features[0].evidence_ids = vec![future_claim.id];
        assert!(
            matches!(
                database
                    .store
                    .freeze_prematch_snapshot(&future_snapshot)
                    .await,
                Err(PersistenceError::InvalidState(_))
            ),
            "published_at/effective_at 晚于截止 1 微秒不得入冻"
        );
    }
    let count_before: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM feature.snapshots WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(count_before, 0, "拒绝的截止时间不留下快照");
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
    assert_eq!(snapshot.data_cutoff_at, research.data_cutoff_at);
    assert_eq!(snapshot.data_cutoff_at, snapshot_retry.data_cutoff_at);
    assert_eq!(snapshot.frozen_at, snapshot_retry.frozen_at);
    let mut changed_snapshot = snapshot_draft.clone();
    changed_snapshot.input_payload = json!({"tampered": true});
    assert!(
        matches!(
            database
                .store
                .freeze_prematch_snapshot(&changed_snapshot)
                .await,
            Err(PersistenceError::InvalidState(_))
        ),
        "幂等键不能绑定不同冻结载荷"
    );
    let bundle = database
        .store
        .read_prematch_snapshot(snapshot.id)
        .await
        .expect("读取完整不可变快照");
    assert_eq!(bundle.snapshot.data_cutoff_at, snapshot.data_cutoff_at);
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

    // Both recorded-time guards are inclusive; one database microsecond later is excluded.
    let cutoff = target_kickoff - Duration::hours(1);
    sqlx::query(
        "UPDATE football.match_results SET finalized_at = $2, created_at = $2 WHERE match_id = $1",
    )
    .bind(valid_history.id)
    .bind(cutoff)
    .execute(&database.pool)
    .await
    .expect("设置截止相等赛果");
    let at_cutoff = database
        .store
        .prepare_match_prediction_input(target.id, "T-1h", "p4")
        .await
        .expect("截止相等历史输入");
    assert_eq!(
        at_cutoff.data_quality["home"]["team_features"]["history_match_count"],
        json!(1)
    );
    assert_eq!(
        at_cutoff.data_quality["home"]["team_features"]["baseline_match_count"],
        json!(1)
    );
    for column in ["finalized_at", "created_at"] {
        let update = format!("UPDATE football.match_results SET {column} = $2 WHERE match_id = $1");
        sqlx::query(&update)
            .bind(valid_history.id)
            .bind(cutoff + Duration::microseconds(1))
            .execute(&database.pool)
            .await
            .expect("设置截止后一微秒赛果");
        let after_cutoff = database
            .store
            .prepare_match_prediction_input(target.id, "T-1h", "p4")
            .await
            .expect("拒绝截止后一微秒历史");
        assert_eq!(
            after_cutoff.data_quality["home"]["team_features"]["history_match_count"],
            json!(0),
            "{column} 晚一微秒不能进入球队历史"
        );
        assert_eq!(
            after_cutoff.data_quality["home"]["team_features"]["neutral_team_ratings"],
            json!(true)
        );
        sqlx::query(&update)
            .bind(valid_history.id)
            .bind(cutoff)
            .execute(&database.pool)
            .await
            .expect("恢复截止相等赛果");
    }
    // Keep one visible sample so the baseline query also exercises each future-time exclusion.
    for finalized_is_future in [true, false] {
        let (finalized, recorded) = if finalized_is_future {
            (cutoff + Duration::microseconds(1), cutoff)
        } else {
            (cutoff, cutoff + Duration::microseconds(1))
        };
        sqlx::query("UPDATE football.match_results SET finalized_at = $2, created_at = $3 WHERE match_id = $1")
            .bind(late_history.id).bind(finalized).bind(recorded).execute(&database.pool).await.expect("设置单独未来赛果时间");
        let baseline_guard = database
            .store
            .prepare_match_prediction_input(target.id, "T-1h", "p4")
            .await
            .expect("进球基准未来时间隔离");
        assert_eq!(
            baseline_guard.data_quality["home"]["team_features"]["history_match_count"],
            json!(1)
        );
        assert_eq!(
            baseline_guard.data_quality["home"]["team_features"]["baseline_match_count"],
            json!(1),
            "晚一微秒的 finalized/created 均不能进入进球基准"
        );
    }
    // A visible historical baseline is queried only when the selected team has evidence.
    sqlx::query("UPDATE football.match_results SET created_at = $2 WHERE match_id = $1")
        .bind(valid_history.id)
        .bind(cutoff - Duration::microseconds(1))
        .execute(&database.pool)
        .await
        .expect("设置截止前一微秒赛果");
    let before_cutoff = database
        .store
        .prepare_match_prediction_input(target.id, "T-1h", "p4")
        .await
        .expect("截止前一微秒历史输入");
    assert_eq!(
        before_cutoff.data_quality["home"]["team_features"]["history_match_count"],
        json!(1)
    );
    assert_eq!(
        before_cutoff.data_quality["home"]["team_features"]["baseline_match_count"],
        json!(1)
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
    let kickoff = Utc::now() + Duration::hours(2);
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
    let home_players =
        create_lineup_player_drafts(&database, home.id, kickoff, &format!("pair-home-{token}"))
            .await;
    let mut away_players =
        create_lineup_player_drafts(&database, away.id, kickoff, &format!("pair-away-{token}"))
            .await;
    // R7-09：沿用双方阵容夹具，预设预览不得产生正式阵容或审计写入。
    let preset_draft = football_domain::TeamLineupPresetDraft {
        id: None,
        team_id: home.id,
        name: format!("预设-{token}"),
        formation_id: Some(formation.id),
        coach_id: None,
        usage_context: "general".into(),
        usage_probability: Some(0.6),
        is_default: true,
        source_lineup_id: None,
        notes: Some("只读预检".into()),
        members: home_players
            .iter()
            .map(|player| football_domain::TeamLineupPresetMemberDraft {
                player_id: player.player_id,
                position_code: player.position_code.clone(),
                role_code: player.role_code.clone(),
                is_starter: player.is_starter,
                shirt_number: player.shirt_number,
                expected_minutes: player.expected_minutes,
                sequence_no: player.sequence_no,
                bench_order: player.bench_order,
                is_captain: false,
                metadata: player.metadata.clone(),
            })
            .collect(),
    };
    let saved_preset = database
        .store
        .save_team_lineup_preset(&preset_draft)
        .await
        .unwrap();
    let counts_before: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM football.lineups), (SELECT count(*) FROM football.lineup_players), (SELECT count(*) FROM audit.events)",
    ).fetch_one(&database.pool).await.unwrap();
    for _ in 0..2 {
        let preview = database
            .store
            .preview_team_lineup_preset_application(saved_preset.id)
            .await
            .unwrap();
        assert!(
            preview.can_apply && preview.blockers.is_empty(),
            "合法预设可以预览"
        );
        assert_eq!(
            serde_json::to_value(preview.preset).unwrap(),
            serde_json::to_value(&saved_preset).unwrap(),
            "重复预览保留原预设及角色来源"
        );
    }
    let counts_after: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM football.lineups), (SELECT count(*) FROM football.lineup_players), (SELECT count(*) FROM audit.events)",
    ).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        counts_before, counts_after,
        "预览不得写正式阵容、球员或审计"
    );
    assert_eq!(
        serde_json::to_value(
            database
                .store
                .read_team_lineup_preset(saved_preset.id)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&saved_preset).unwrap(),
        "预览不得修改预设版本或时间"
    );

    let mut invalid_preset = preset_draft.clone();
    invalid_preset.id = Some(saved_preset.id);
    invalid_preset.members[0].player_id = away_players[0].player_id;
    database
        .store
        .save_team_lineup_preset(&invalid_preset)
        .await
        .expect_err("客队成员不得保存到主队预设");
    assert_eq!(
        serde_json::to_value(
            database
                .store
                .read_team_lineup_preset(saved_preset.id)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&saved_preset).unwrap(),
        "成员失败不得替换预设或改变默认标志"
    );
    sqlx::query("UPDATE football.player_team_periods SET valid_to=current_date-1 WHERE player_id=$1 AND team_id=$2")
        .bind(home_players[0].player_id).bind(home.id).execute(&database.pool).await.unwrap();
    let departed = database
        .store
        .preview_team_lineup_preset_application(saved_preset.id)
        .await
        .unwrap();
    assert!(
        !departed.can_apply
            && departed
                .blockers
                .iter()
                .any(|message| message.contains("当前不再属于")),
        "过期成员阻止套用"
    );
    sqlx::query(
        "UPDATE football.player_team_periods SET valid_to=NULL WHERE player_id=$1 AND team_id=$2",
    )
    .bind(home_players[0].player_id)
    .bind(home.id)
    .execute(&database.pool)
    .await
    .unwrap();
    database
        .store
        .archive_team_lineup_preset(saved_preset.id)
        .await
        .unwrap();
    assert!(
        !database
            .store
            .preview_team_lineup_preset_application(saved_preset.id)
            .await
            .unwrap()
            .can_apply,
        "归档预设阻止套用"
    );
    assert!(database
        .store
        .list_team_lineup_presets(home.id, false)
        .await
        .unwrap()
        .iter()
        .all(|preset| preset.id != saved_preset.id));
    let copy = database
        .store
        .duplicate_team_lineup_preset(saved_preset.id, &format!("预设副本-{token}"))
        .await
        .unwrap();
    assert_ne!(copy.id, saved_preset.id);
    assert!(!copy.is_default && copy.status == "active" && copy.version == 1);
    assert_eq!(copy.members.len(), saved_preset.members.len());
    database
        .store
        .delete_team_lineup_preset(copy.id)
        .await
        .unwrap();
    database
        .store
        .delete_team_lineup_preset(saved_preset.id)
        .await
        .unwrap();
    let remaining_members: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM football.team_lineup_preset_members WHERE preset_id=ANY($1::uuid[])",
    )
    .bind(vec![copy.id, saved_preset.id])
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(remaining_members, 0, "删除活动与归档预设级联清理成员");
    assert!(database
        .store
        .read_team_lineup_preset(saved_preset.id)
        .await
        .is_err());

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
    let count_after_failure: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .expect("统计失败后的阵容数量");
    assert_eq!(count_after_failure, 0, "任一侧失败后不得保留另一侧阵容");
    let failed_audits: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE (event_type='lineup_created' AND payload->>'match_id'=$1) OR (event_type='lineup_pair_created' AND entity_id=$1)",
    ).bind(target.id.to_string()).fetch_one(&database.pool).await.expect("读取失败双方提交的审计残留");
    assert_eq!(failed_audits, 0, "客队失败时主队创建审计也必须回滚");

    let mut valid_pair = pair;
    valid_pair.away.players[0].player_id = valid_away_first;
    for case in 0..6 {
        let mut invalid_pair = valid_pair.clone();
        match case {
            0 => invalid_pair.away.match_id = Uuid::new_v4(),
            1 => invalid_pair.away.team_id = invalid_pair.home.team_id,
            2 => invalid_pair.away.snapshot_type = "T-1h".into(),
            3 => invalid_pair.away.lineup_type = LineupType::Actual,
            4 => {
                invalid_pair.home.team_id = away.id;
                invalid_pair.away.team_id = home.id;
            }
            _ => {
                let missing_match_id = Uuid::new_v4();
                invalid_pair.home.match_id = missing_match_id;
                invalid_pair.away.match_id = missing_match_id;
            }
        }
        assert!(
            matches!(
                database.store.create_lineup_pair(&invalid_pair).await,
                Err(PersistenceError::InvalidState(_))
            ),
            "错误双方身份/窗口/类型必须拒绝"
        );
    }
    let created = database
        .store
        .create_lineup_pair(&valid_pair)
        .await
        .expect("双方阵容应在一个事务中提交");
    assert_eq!(created.home.team_id, home.id);
    assert_eq!(created.away.team_id, away.id);
    assert!(created.home.model_eligible && created.away.model_eligible);
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

    let archived_record = database.store.read_lineup(first_home_id).await.unwrap();
    assert_eq!(archived_record.id, first_home_id, "隐藏历史仍可按 ID 追溯");
    assert_eq!(archived_record.players.len(), 11);
    assert!(matches!(
        database.store.remove_lineup_history(first_home_id, None).await,
        Err(PersistenceError::InvalidState(message)) if message.contains("已经从历史列表隐藏")
    ));
    assert!(
        database.store.read_lineup(replacement.id).await.is_err(),
        "物理删除版本不能读回"
    );

    // R7-08：删除/恢复与创建必须按父比赛 -> 阵容的同一顺序串行。
    let mut history_blocker = database.pool.begin().await.unwrap();
    let history_blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *history_blocker)
        .await
        .unwrap();
    sqlx::query("SELECT id FROM football.matches WHERE id=$1 FOR UPDATE")
        .bind(target.id)
        .fetch_one(&mut *history_blocker)
        .await
        .unwrap();
    let removal_store = database.store.clone();
    let latest_id = latest.id;
    let mut removal_task = tokio::spawn(async move {
        removal_store
            .remove_lineup_history(latest_id, Some("父锁并发删除"))
            .await
    });
    replacement_home.captured_at = captured_at + Duration::minutes(25);
    let creation_store = database.store.clone();
    let mut creation_task =
        tokio::spawn(async move { creation_store.create_lineup(&replacement_home).await });
    let history_waiting = wait_for_lineup_lock(
        &database.pool,
        history_blocker_pid,
        "SELECT home_team_id, away_team_id FROM football.matches WHERE id=$1 FOR UPDATE",
        2,
    )
    .await;
    if !history_waiting {
        removal_task.abort();
        creation_task.abort();
    }
    history_blocker.rollback().await.unwrap();
    assert!(
        history_waiting,
        "历史删除与创建都必须在版本写入前等待同一父锁"
    );
    let history_results = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        tokio::join!(&mut removal_task, &mut creation_task)
    })
    .await;
    if history_results.is_err() {
        removal_task.abort();
        creation_task.abort();
    }
    let (removed, replacement_result) = history_results.expect("历史删除与创建不能锁顺序死锁");
    let removed = removed.unwrap().unwrap();
    let after_history_race = replacement_result.unwrap().unwrap();
    assert_eq!(removed.lineup_id, latest.id);
    assert_eq!(removed.restored_lineup_id, None, "隐藏前驱不得恢复");
    assert_eq!(after_history_race.status, "active");
    let active_after_history_race: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND snapshot_type='T-6h' AND lineup_type='expected' AND status='active'",
    ).bind(target.id).bind(home.id).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        active_after_history_race, 1,
        "删除/恢复与创建交错后只有一个活动版本"
    );
    let removal_audits: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='lineup_history_removed' AND entity_id=$1",
    ).bind(latest.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(removal_audits, 1, "历史变更只提交一次成功审计");

    let latest = after_history_race;

    // 复用真实预检/提交入口，覆盖 ended_previous 的业务、账本与审计事务。
    let lineup_key = format!("ledger-{token}");
    let mut rows = vec![SpreadsheetRawRow {
        sheet_name: "阵容".into(),
        row_number: 2,
        entity_type: SpreadsheetEntityType::Lineup,
        action: SpreadsheetAction::Add,
        values: json!({
            "lineup_key": lineup_key, "match_key": target.external_key,
            "match_id": target.id.to_string(), "team_id": home.id.to_string(), "team_side": "home",
            "lineup_type": "expected", "snapshot_type": "T-6h",
            "formation": formation.code, "formation_id": formation.id.to_string(),
            "captured_at": (captured_at + Duration::minutes(30)).to_rfc3339(),
            "quality_score": "0.9"
        }),
    }];
    for (index, player) in valid_pair.home.players.iter().enumerate() {
        rows.push(SpreadsheetRawRow {
            sheet_name: "阵容球员".into(),
            row_number: index as u32 + 2,
            entity_type: SpreadsheetEntityType::LineupPlayer,
            action: SpreadsheetAction::Add,
            values: json!({
                "lineup_key": lineup_key, "match_key": target.external_key,
                "match_id": target.id.to_string(), "team_id": home.id.to_string(), "team_side": "home",
                "player_id": player.player_id.to_string(),
                "position_code": player.position_code,
                "is_starter": "true", "sequence_no": (index + 1).to_string(),
                "expected_minutes": "90"
            }),
        });
    }
    let workbook = SpreadsheetParsedWorkbook {
        format_version: "football.match-lineup.v2".into(),
        source_file_name: format!("ledger-{token}.xlsx"),
        source_sha256: format!("ledger-{token}"),
        rows,
    };
    // R7-14：沿用同一双方/工作簿夹具验证读取与预检的事实边界。
    let workbook_read_snapshot_sql = r#"
        SELECT jsonb_build_object(
            'match', (SELECT to_jsonb(fixture) FROM football.matches fixture WHERE id=$1),
            'lineups', (SELECT COALESCE(jsonb_agg(to_jsonb(lineup) ORDER BY lineup.id),'[]'::jsonb)
                        FROM football.lineups lineup WHERE match_id=$1),
            'players', (SELECT COALESCE(jsonb_agg(to_jsonb(member) ORDER BY member.lineup_id,member.player_id),'[]'::jsonb)
                        FROM football.lineup_players member JOIN football.lineups lineup ON lineup.id=member.lineup_id WHERE lineup.match_id=$1),
            'tags', (SELECT count(*) FROM feature.player_dynamic_tags),
            'batches', (SELECT count(*) FROM catalog.import_batches),
            'rows', (SELECT count(*) FROM catalog.import_rows),
            'audits', (SELECT count(*) FROM audit.events)
        )
    "#;
    let workbook_read_before: serde_json::Value = sqlx::query_scalar(workbook_read_snapshot_sql)
        .bind(target.id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    let export = database
        .store
        .match_lineup_export_data(Some(target.id))
        .await
        .unwrap();
    let exported_again = database
        .store
        .match_lineup_export_data(Some(target.id))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&export).unwrap(),
        serde_json::to_value(&exported_again).unwrap(),
        "比赛工作簿重复导出保持引用、活动版本和字段投影"
    );
    assert_eq!(export.selected_match.as_ref().unwrap().id, target.id);
    assert!(export
        .lineups
        .iter()
        .all(|lineup| lineup.match_id == target.id && lineup.status == "active"));
    assert!(export
        .lineups
        .iter()
        .any(|lineup| lineup.team_id == home.id));
    assert!(export
        .lineups
        .iter()
        .any(|lineup| lineup.team_id == away.id));
    assert!(export
        .players
        .iter()
        .any(|player| player.player_id == valid_pair.home.players[0].player_id));
    let context = database
        .store
        .ai_match_package_context(target.id)
        .await
        .unwrap();
    assert_eq!(context.match_record.id, target.id);
    assert_eq!(
        context
            .lineups
            .iter()
            .map(|lineup| lineup.id)
            .collect::<Vec<_>>(),
        export
            .lineups
            .iter()
            .map(|lineup| lineup.id)
            .collect::<Vec<_>>()
    );
    let unique_context_players: std::collections::HashSet<_> = context
        .players
        .iter()
        .map(|player| player.player.id)
        .collect();
    assert_eq!(
        unique_context_players.len(),
        context.players.len(),
        "AI 比赛包跨活动版本只保留一份球员上下文"
    );
    for player in &context.players {
        assert!(matches!(player.lineup_status.as_str(), "starter" | "bench"));
        assert_eq!(
            player.lineup_role, player.tactical_role_code,
            "AI 旧角色字段保留纠正后的战术角色语义"
        );
    }
    let empty_export = database.store.match_lineup_export_data(None).await.unwrap();
    assert!(
        empty_export.selected_match.is_none()
            && empty_export.lineups.is_empty()
            && empty_export.players.is_empty()
            && empty_export.dynamic_tags.is_empty()
    );
    assert!(!empty_export.teams.is_empty() && !empty_export.formations.is_empty());
    database
        .store
        .match_lineup_export_data(Some(Uuid::new_v4()))
        .await
        .expect_err("未知比赛导出保留 Match Catalog 错误");
    let workbook_read_after: serde_json::Value = sqlx::query_scalar(workbook_read_snapshot_sql)
        .bind(target.id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    assert_eq!(
        workbook_read_before, workbook_read_after,
        "比赛导出和 AI 上下文不得修改事实、账本或审计"
    );
    let preview = database
        .store
        .preview_match_lineup_import(&workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检合法双方事务夹具中的主队替代阵容");
    assert_eq!(preview.counts.error, 0);
    assert_eq!(preview.counts.conflict, 0);
    assert_eq!(preview.rows.len(), 12);
    let preview_read = database
        .store
        .read_match_lineup_import_preview(preview.batch_id)
        .await
        .unwrap();
    let preview_read_again = database
        .store
        .read_match_lineup_import_preview(preview.batch_id)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&preview_read).unwrap(),
        serde_json::to_value(&preview_read_again).unwrap(),
        "比赛预检读回保持行身份、载荷和计数"
    );
    for original_row in &preview.rows {
        let stored_row = preview_read
            .rows
            .iter()
            .find(|row| row.id == original_row.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(original_row).unwrap(),
            serde_json::to_value(stored_row).unwrap()
        );
    }
    let preview_snapshot: serde_json::Value = sqlx::query_scalar(workbook_read_snapshot_sql)
        .bind(target.id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    for field in ["match", "lineups", "players", "tags", "audits"] {
        assert_eq!(
            workbook_read_before[field], preview_snapshot[field],
            "比赛预检只暂存批次和行，不写事实或成功审计：{field}"
        );
    }
    assert_eq!(
        preview_snapshot["batches"].as_i64().unwrap(),
        workbook_read_before["batches"].as_i64().unwrap() + 1
    );
    assert_eq!(
        preview_snapshot["rows"].as_i64().unwrap(),
        workbook_read_before["rows"].as_i64().unwrap() + 12
    );
    let mut wrong_side_workbook = workbook.clone();
    wrong_side_workbook.source_sha256 = format!("wrong-side-{token}");
    wrong_side_workbook.rows[0].values["team_side"] = json!("away");
    let wrong_side_preview = database
        .store
        .preview_match_lineup_import(&wrong_side_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    assert_eq!(
        wrong_side_preview.counts.error, 1,
        "工作簿主客身份不符必须预检阻断"
    );
    database
        .store
        .commit_match_lineup_import(wrong_side_preview.batch_id)
        .await
        .expect_err("非法单侧工作簿不得启动事实写入");
    let rejected_snapshot: serde_json::Value = sqlx::query_scalar(workbook_read_snapshot_sql)
        .bind(target.id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
    for field in ["match", "lineups", "players", "tags", "audits"] {
        assert_eq!(
            preview_snapshot[field], rejected_snapshot[field],
            "工作簿阻断后事实与审计不变：{field}"
        );
    }
    let last_player_row = preview
        .rows
        .iter()
        .find(|row| row.entity_type == SpreadsheetEntityType::LineupPlayer && row.row_number == 12)
        .expect("定位最后一个球员预检行");
    // R7-10：沿用已有比赛导入行，检查批次状态与两阶段冲突处理计数。
    assert!(
        database
            .store
            .read_spreadsheet_import_preview(preview.batch_id)
            .await
            .is_err(),
        "球员预览不得跨导入类型读取比赛批次"
    );
    for status in ["running", "failed", "cancelled"] {
        sqlx::query("UPDATE catalog.import_batches SET status=$2 WHERE id=$1")
            .bind(preview.batch_id)
            .bind(status)
            .execute(&database.pool)
            .await
            .unwrap();
        database
            .store
            .commit_match_lineup_import(preview.batch_id)
            .await
            .expect_err("非 pending 比赛批次不得提交");
    }
    sqlx::query("UPDATE catalog.import_batches SET status='pending',error_count=1 WHERE id=$1")
        .bind(preview.batch_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let selected_player = valid_pair.home.players[10].player_id;
    let conflict_candidates =
        json!([{"entity_id":selected_player,"display_name":"原主队球员","detail":null}]);
    sqlx::query(
        "UPDATE catalog.import_rows SET status='conflict',conflict_candidates=$2 WHERE id=$1",
    )
    .bind(last_player_row.id)
    .bind(&conflict_candidates)
    .execute(&database.pool)
    .await
    .unwrap();
    database
        .store
        .commit_match_lineup_import(preview.batch_id)
        .await
        .expect_err("未解决比赛冲突不能启动业务写入");
    database
        .store
        .resolve_match_lineup_import_conflict(
            preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: last_player_row.id,
                selected_entity_id: Some(Uuid::new_v4()),
                skip: false,
            },
        )
        .await
        .expect_err("候选范围之外不能修改冲突或计数");
    let rejected_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(rejected_counts, (0, 1), "候选拒绝不改变暂存计数");

    let skipped = database
        .store
        .resolve_match_lineup_import_conflict(
            preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: last_player_row.id,
                selected_entity_id: None,
                skip: true,
            },
        )
        .await
        .unwrap();
    assert_eq!(skipped.counts.skipped, 1);
    let skip_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(skip_counts, (1, 0), "比赛冲突跳过与批次暂存计数共同提交");
    sqlx::query("UPDATE catalog.import_rows SET status='conflict',payload=$2,conflict_candidates=$3 WHERE id=$1")
        .bind(last_player_row.id).bind(&last_player_row.payload).bind(&conflict_candidates).execute(&database.pool).await.unwrap();
    sqlx::query("UPDATE catalog.import_batches SET skipped_count=0,error_count=1 WHERE id=$1")
        .bind(preview.batch_id)
        .execute(&database.pool)
        .await
        .unwrap();
    let resolved = database
        .store
        .resolve_match_lineup_import_conflict(
            preview.batch_id,
            football_domain::SpreadsheetImportResolution {
                row_id: last_player_row.id,
                selected_entity_id: Some(selected_player),
                skip: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        resolved.counts.error + resolved.counts.conflict + resolved.counts.skipped,
        0,
        "合法候选解决后预览计数清零"
    );
    let resolved_counts: (i64, i64) =
        sqlx::query_as("SELECT skipped_count,error_count FROM catalog.import_batches WHERE id=$1")
            .bind(preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(resolved_counts, (0, 0), "候选解决后的行状态与批次计数一致");
    // 恢复原预检行，继续既有末行失败/整批回滚/重试/审计测试。
    sqlx::query("UPDATE catalog.import_rows SET status=$2,payload=$3,matched_entity_id=$4,conflict_candidates='[]'::jsonb WHERE id=$1")
        .bind(last_player_row.id).bind(last_player_row.status.as_str()).bind(&last_player_row.payload).bind(last_player_row.matched_entity_id).execute(&database.pool).await.unwrap();
    let mut invalid_payload = last_player_row.payload.clone();
    invalid_payload["_resolved_player_id"] = json!(Uuid::new_v4());
    sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
        .bind(last_player_row.id)
        .bind(invalid_payload)
        .execute(&database.pool)
        .await
        .expect("模拟预检后末行球员外键失效");
    let lineup_count_before: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取导入前历史数量");
    database
        .store
        .commit_match_lineup_import(preview.batch_id)
        .await
        .expect_err("末行失败必须回滚替代阵容、前十个球员、账本与审计");
    let rolled_back: (String, i64, i64, i64, i64, i64, bool) = sqlx::query_as(
        "SELECT status,inserted_count,updated_count,ended_previous_count,skipped_count,error_count,finished_at IS NULL FROM catalog.import_batches WHERE id=$1",
    ).bind(preview.batch_id).fetch_one(&database.pool).await.expect("读取回滚后的批次账本");
    assert_eq!(rolled_back, ("pending".into(), 0, 0, 0, 0, 0, true));
    let imported_rows: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND (status='imported' OR imported_at IS NOT NULL)",
    ).bind(preview.batch_id).fetch_one(&database.pool).await.expect("读取回滚后的行状态");
    assert_eq!(imported_rows, 0);
    assert_eq!(
        database
            .store
            .read_lineup(latest.id)
            .await
            .expect("回滚恢复原阵容")
            .status,
        "active"
    );
    let count_after_rollback: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取回滚后的历史数量");
    assert_eq!(count_after_rollback, lineup_count_before);
    let audit_count_before: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='match_lineup_import_committed' AND entity_id=$1",
    ).bind(preview.batch_id.to_string()).fetch_one(&database.pool).await.expect("读取回滚后的审计数量");
    assert_eq!(audit_count_before, 0);

    sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
        .bind(last_player_row.id)
        .bind(&last_player_row.payload)
        .execute(&database.pool)
        .await
        .expect("恢复合法末行后复用原批次重试");
    let committed = database
        .store
        .commit_match_lineup_import(preview.batch_id)
        .await
        .expect("提交替代阵容并写入一致账本");
    assert_eq!(committed.inserted_count, 12);
    assert_eq!(committed.updated_count, 0);
    assert_eq!(committed.ended_previous_count, 1);
    assert_eq!(committed.skipped_count, 0);
    assert_eq!(committed.error_count, 0);
    let ledger: (String, i64, i64, i64, i64, i64, bool) = sqlx::query_as(
        "SELECT status,inserted_count,updated_count,ended_previous_count,skipped_count,error_count,finished_at IS NOT NULL FROM catalog.import_batches WHERE id=$1",
    ).bind(preview.batch_id).fetch_one(&database.pool).await.expect("读取成功批次账本");
    assert_eq!(ledger, ("succeeded".into(), 12, 0, 1, 0, 0, true));
    assert_eq!(
        database
            .store
            .read_lineup(latest.id)
            .await
            .expect("读取被替代版本")
            .status,
        "superseded"
    );
    let active_home: (Uuid, bool) = sqlx::query_as(
        "SELECT id,model_eligible FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND snapshot_type='T-6h' AND lineup_type='expected' AND status='active'",
    ).bind(target.id).bind(home.id).fetch_one(&database.pool).await.expect("读取新活动阵容");
    assert_ne!(active_home.0, latest.id);
    assert!(active_home.1);
    assert_eq!(
        database
            .store
            .read_lineup(active_home.0)
            .await
            .expect("读取导入后的完整阵容")
            .starter_count,
        11
    );
    let audit_payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM audit.events WHERE event_type='match_lineup_import_committed' AND entity_id=$1",
    ).bind(preview.batch_id.to_string()).fetch_one(&database.pool).await.expect("读取提交审计");
    assert_eq!(audit_payload["inserted"], json!(committed.inserted_count));
    assert_eq!(audit_payload["updated"], json!(committed.updated_count));
    assert_eq!(
        audit_payload["ended_previous"],
        json!(committed.ended_previous_count)
    );
    assert_eq!(audit_payload["skipped"], json!(committed.skipped_count));
    assert!(matches!(
        database
            .store
            .commit_match_lineup_import(preview.batch_id)
            .await,
        Err(PersistenceError::InvalidState(_))
    ));
    let audit_count_after: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='match_lineup_import_committed' AND entity_id=$1",
    ).bind(preview.batch_id.to_string()).fetch_one(&database.pool).await.expect("读取重复提交后的审计数量");
    assert_eq!(audit_count_after, 1);
    let lineup_count_after: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取重复提交后的历史数量");
    assert_eq!(lineup_count_after, lineup_count_before + 1);
    let ended_count_after: i64 =
        sqlx::query_scalar("SELECT ended_previous_count FROM catalog.import_batches WHERE id=$1")
            .bind(preview.batch_id)
            .fetch_one(&database.pool)
            .await
            .expect("读取重复提交后的计数");
    assert_eq!(ended_count_after, 1);

    // 在客队球员外键处实际阻塞：主队及其审计已经写入，但整个 pair 尚未提交。
    let pair_audits_before: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='lineup_pair_created' AND entity_id=$1",
    ).bind(target.id.to_string()).fetch_one(&database.pool).await.expect("记录取消前双方审计数量");
    let created_audits_before_cancel: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='lineup_created' AND payload->>'match_id'=$1",
    ).bind(target.id.to_string()).fetch_one(&database.pool).await.unwrap();
    let mut blocker = database
        .pool
        .begin()
        .await
        .expect("锁定客队球员以观察提交前取消");
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("SELECT id FROM football.players WHERE id=$1 FOR UPDATE")
        .bind(valid_away_first)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let mut cancelled_pair = valid_pair.clone();
    cancelled_pair.home.captured_at += Duration::minutes(40);
    cancelled_pair.away.captured_at += Duration::minutes(40);
    let cancel_store = database.store.clone();
    let cancelled =
        tokio::spawn(async move { cancel_store.create_lineup_pair(&cancelled_pair).await });
    let observed = wait_for_lineup_lock(
        &database.pool,
        blocker_pid,
        "%INSERT INTO football.lineup_players%",
        1,
    )
    .await;
    cancelled.abort();
    let cancelled_result = cancelled.await;
    blocker.rollback().await.expect("释放外键阻塞");
    assert!(observed, "取消必须发生在主队写入后、客队明细外键等待期间");
    assert!(cancelled_result
        .expect_err("任务应在提交前取消")
        .is_cancelled());
    // 重新取得父锁，确认被取消的事务已经释放锁，不能靠不可见未提交行冒充回滚。
    let mut released = database.pool.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout='5s'")
        .execute(&mut *released)
        .await
        .unwrap();
    sqlx::query("SELECT id FROM football.matches WHERE id=$1 FOR UPDATE")
        .bind(target.id)
        .fetch_one(&mut *released)
        .await
        .expect("取消后父锁必须释放");
    released.rollback().await.unwrap();
    let after_cancel: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(
        after_cancel, lineup_count_after,
        "取消不得留下半条阵容或结束原活动版本"
    );
    let active_after_cancel: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1 AND status='active'",
    )
    .bind(target.id)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(active_after_cancel, 2);
    let created_audits_after_cancel: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='lineup_created' AND payload->>'match_id'=$1",
    ).bind(target.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        created_audits_after_cancel, created_audits_before_cancel,
        "取消不得遗留主队创建审计"
    );

    let mut concurrent_workbook = workbook.clone();
    concurrent_workbook.source_sha256 = format!("ledger-concurrent-{token}");
    let concurrent_preview = database
        .store
        .preview_match_lineup_import(&concurrent_workbook, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("预检参与并发的合法工作簿");
    assert_eq!(concurrent_preview.counts.error, 0);
    assert_eq!(concurrent_preview.counts.conflict, 0);
    // 两次 pair、一次 single 和一个工作簿在共同父锁前等待。

    let mut blocker = database.pool.begin().await.unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    sqlx::query("SELECT id FROM football.matches WHERE id=$1 FOR UPDATE")
        .bind(target.id)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let mut first_pair = valid_pair.clone();
    first_pair.home.captured_at += Duration::minutes(40);
    first_pair.away.captured_at += Duration::minutes(40);
    let mut second_pair = first_pair.clone();
    second_pair.home.captured_at += Duration::minutes(10);
    second_pair.away.captured_at += Duration::minutes(10);
    let mut single_draft = second_pair.home.clone();
    single_draft.captured_at += Duration::minutes(10);
    let first_store = database.store.clone();
    let second_store = database.store.clone();
    let single_store = database.store.clone();
    let import_store = database.store.clone();
    let mut first = tokio::spawn(async move { first_store.create_lineup_pair(&first_pair).await });
    let mut second =
        tokio::spawn(async move { second_store.create_lineup_pair(&second_pair).await });
    let mut single = tokio::spawn(async move { single_store.create_lineup(&single_draft).await });
    let batch_id = concurrent_preview.batch_id;
    let mut importing =
        tokio::spawn(async move { import_store.commit_match_lineup_import(batch_id).await });
    let observed = wait_for_lineup_lock(
        &database.pool,
        blocker_pid,
        "SELECT home_team_id, away_team_id FROM football.matches WHERE id=$1 FOR UPDATE%",
        4,
    )
    .await;
    if !observed {
        first.abort();
        second.abort();
        single.abort();
        importing.abort();
    }
    blocker.rollback().await.unwrap();
    assert!(
        observed,
        "pair、single 与 workbook 必须在写入前等待同一比赛锁"
    );
    let outcomes = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        tokio::join!(&mut first, &mut second, &mut single, &mut importing)
    })
    .await;
    if outcomes.is_err() {
        first.abort();
        second.abort();
        single.abort();
        importing.abort();
    }
    let (first, second, single, importing) = outcomes.expect("串行放行后并发阵容写入不能挂起");
    let first = first.expect("首个并发任务完成").expect("首个双方提交成功");
    let second = second.expect("第二并发任务完成").expect("第二双方提交成功");
    let single = single.expect("单侧并发任务完成").expect("单侧提交成功");
    let imported = importing
        .expect("工作簿并发任务完成")
        .expect("工作簿并发提交成功");
    assert_eq!(imported.inserted_count, 12);
    assert_eq!(imported.ended_previous_count, 1);
    let import_ledger: (i64, i64) = sqlx::query_as(
        "SELECT inserted_count,ended_previous_count FROM catalog.import_batches WHERE id=$1",
    )
    .bind(batch_id)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(import_ledger, (12, 1));
    let imported_lineup_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND metadata->>'source'='match_lineup_spreadsheet' AND id<>$3",
    ).bind(target.id).bind(home.id).bind(active_home.0).fetch_one(&database.pool).await.unwrap();
    assert_ne!(first.home.id, second.home.id);
    assert_ne!(first.away.id, second.away.id);
    assert_ne!(single.id, first.home.id);
    assert_ne!(single.id, second.home.id);
    for (team_id, new_ids) in [
        (
            home.id,
            vec![first.home.id, second.home.id, single.id, imported_lineup_id],
        ),
        (away.id, vec![first.away.id, second.away.id]),
    ] {
        let active_count: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND snapshot_type='T-6h' AND lineup_type='expected' AND status='active'",
        ).bind(target.id).bind(team_id).fetch_one(&database.pool).await.unwrap();
        assert_eq!(active_count, 1, "并发后每侧必须只有一条活动阵容");
        let incomplete: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM football.lineups lineup WHERE lineup.id=ANY($1) AND (NOT lineup.model_eligible OR lineup.supersedes_lineup_id IS NULL OR (SELECT count(*) FROM football.lineup_players player WHERE player.lineup_id=lineup.id)<>11)",
        ).bind(&new_ids).fetch_one(&database.pool).await.unwrap();
        assert_eq!(incomplete, 0, "并发版本必须保留完整明细、模型门禁与前驱链");
    }
    let after_concurrent: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(after_concurrent, lineup_count_after + 6);
    let pair_audits_after: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='lineup_pair_created' AND entity_id=$1",
    ).bind(target.id.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        pair_audits_after,
        pair_audits_before + 2,
        "只记录实际成功的双方提交"
    );
    for pair in [&first, &second] {
        let payload: serde_json::Value = sqlx::query_scalar(
            "SELECT payload FROM audit.events WHERE event_type='lineup_pair_created' AND entity_id=$1 AND payload->>'home_lineup_id'=$2",
        ).bind(target.id.to_string()).bind(pair.home.id.to_string()).fetch_one(&database.pool).await.unwrap();
        assert_eq!(payload["away_lineup_id"], json!(pair.away.id));
    }

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

    let before_confirmed = database
        .store
        .read_match_lineup_chain_at(
            target.id,
            "T-6h",
            confirmed.0.captured_at - Duration::microseconds(1),
        )
        .await
        .unwrap();
    assert!(before_confirmed.home.selected_lineup_id.is_none());
    assert!(before_confirmed.away.selected_lineup_id.is_none());
    let at_confirmed = database
        .store
        .read_match_lineup_chain_at(target.id, "T-6h", confirmed.0.captured_at)
        .await
        .unwrap();
    assert_eq!(
        at_confirmed.home.selected_lineup_id,
        Some(confirmed.0.id),
        "截止时点包含等时记录"
    );
    assert_eq!(at_confirmed.away.selected_lineup_id, Some(confirmed.1.id));

    // 同时点以 confirmed 优先；仅差一微秒则先按记录时间排序。
    let tied_expected = seed_valid_match_lineups(
        &database,
        MatchLineupSeed {
            match_id: target.id,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff,
            snapshot_type: "T-6h",
            lineup_type: LineupType::Expected,
        },
    )
    .await;
    let tied_chain = database
        .store
        .read_match_lineup_chain_at(target.id, "T-6h", confirmed.0.captured_at)
        .await
        .unwrap();
    assert_eq!(
        tied_chain.home.selected_lineup_id,
        Some(confirmed.0.id),
        "同时间 confirmed 优先于 expected"
    );
    assert_eq!(tied_chain.away.selected_lineup_id, Some(confirmed.1.id));
    let tied_list = database
        .store
        .list_lineups(Some(target.id), 200)
        .await
        .unwrap();
    let mut tied_ids = vec![
        confirmed.0.id,
        confirmed.1.id,
        tied_expected.0.id,
        tied_expected.1.id,
    ];
    tied_ids.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(
        tied_list
            .iter()
            .filter(|item| item.captured_at == confirmed.0.captured_at)
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        tied_ids,
        "等时间历史以 UUID 降序稳定排列"
    );
    for id in [tied_expected.0.id, tied_expected.1.id] {
        sqlx::query("UPDATE football.lineups SET captured_at=$2 WHERE id=$1")
            .bind(id)
            .bind(confirmed.0.captured_at + Duration::microseconds(1))
            .execute(&database.pool)
            .await
            .unwrap();
    }
    let newer_chain = database
        .store
        .read_match_lineup_chain_at(
            target.id,
            "T-6h",
            confirmed.0.captured_at + Duration::microseconds(1),
        )
        .await
        .unwrap();
    assert_eq!(
        newer_chain.home.selected_lineup_id,
        Some(tied_expected.0.id),
        "时间较新的 expected 优先于较旧 confirmed"
    );
    assert_eq!(
        newer_chain.away.selected_lineup_id,
        Some(tied_expected.1.id)
    );

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

    let players =
        create_lineup_player_drafts(&database, home.id, kickoff, &format!("invalid-{token}")).await;
    let mut invalid_draft = LineupDraft {
        match_id: target.id,
        team_id: home.id,
        lineup_type: LineupType::Confirmed,
        snapshot_type: "T-1h".to_string(),
        formation: None,
        formation_id: None,
        coach_id: None,
        captured_at: kickoff - Duration::minutes(50),
        source_document_id: None,
        source_urls: vec![],
        quality_score: Some(0.9),
        metadata: json!({"integration_test": true}),
        players,
    };
    invalid_draft.players[10].is_starter = false;
    invalid_draft.players[10].bench_order = Some(1);
    let count_before: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    let rejected = database
        .store
        .create_lineup(&invalid_draft)
        .await
        .expect_err("10 名首发必须在写入前拒绝");
    assert!(
        matches!(rejected, PersistenceError::InvalidState(message) if message.contains("11 名首发"))
    );
    let count_after: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.lineups WHERE match_id=$1")
            .bind(target.id)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(count_before, count_after, "非法首发不能留下阵容行");
    invalid_draft.players[10].is_starter = true;
    invalid_draft.players[10].bench_order = None;
    let invalid_home = database
        .store
        .create_lineup(&invalid_draft)
        .await
        .expect("合法 11 人可保存，缺失阵型应阻断模型冻结");
    assert_eq!(invalid_home.starter_count, 11);
    assert!(!invalid_home.model_eligible);
    assert_eq!(invalid_home.model_validation_status, "invalid");
    assert!(invalid_home
        .validation_errors
        .iter()
        .any(|item| item.contains("阵型")));
    let actual = seed_team_lineup(
        &database,
        TeamLineupSeed {
            match_id: target.id,
            team_id: away.id,
            kickoff,
            snapshot_type: "T-1h",
            lineup_type: LineupType::Actual,
            starter_count: 11,
            label: "actual",
        },
    )
    .await;
    assert!(!actual.model_eligible, "合法实际阵容仍只用于赛后");
    let reference = kickoff - Duration::minutes(40);
    let blocked_chain = database
        .store
        .read_match_lineup_chain_at(target.id, "T-1h", reference)
        .await
        .unwrap();
    assert!(!blocked_chain.ready_for_model);
    assert!(blocked_chain.home.selected_lineup_id.is_none());
    assert!(blocked_chain.away.selected_lineup_id.is_none());
    assert!(
        matches!(
            database
                .store
                .prepare_match_prediction_input_at(target.id, "T-1h", "p4", reference)
                .await,
            Err(PersistenceError::InvalidState(_))
        ),
        "无有效首发或只有 actual 时必须阻断输入冻结"
    );

    let earliest_home = seed_team_lineup(
        &database,
        TeamLineupSeed {
            match_id: target.id,
            team_id: home.id,
            kickoff,
            snapshot_type: "T-1h",
            lineup_type: LineupType::Expected,
            starter_count: 11,
            label: "window-start",
        },
    )
    .await;
    let window_start = kickoff - Duration::hours(1);
    sqlx::query("UPDATE football.lineups SET captured_at=$2 WHERE id=$1")
        .bind(earliest_home.id)
        .bind(window_start)
        .execute(&database.pool)
        .await
        .unwrap();
    let at_start = database
        .store
        .read_match_lineup_chain_at(target.id, "T-1h", window_start)
        .await
        .unwrap();
    assert_eq!(
        at_start.home.selected_lineup_id,
        Some(earliest_home.id),
        "窗口起点包含等时记录"
    );
    sqlx::query("UPDATE football.lineups SET captured_at=$2 WHERE id=$1")
        .bind(earliest_home.id)
        .bind(window_start - Duration::microseconds(1))
        .execute(&database.pool)
        .await
        .unwrap();
    let before_start = database
        .store
        .read_match_lineup_chain_at(target.id, "T-1h", window_start)
        .await
        .unwrap();
    assert!(
        before_start.home.selected_lineup_id.is_none(),
        "窗口前一微秒不能选择"
    );
    sqlx::query("UPDATE football.lineups SET captured_at=$2, history_hidden_at=now() WHERE id=$1")
        .bind(earliest_home.id)
        .bind(window_start)
        .execute(&database.pool)
        .await
        .unwrap();
    let hidden_chain = database
        .store
        .read_match_lineup_chain_at(target.id, "T-1h", window_start)
        .await
        .unwrap();
    assert!(hidden_chain.home.selected_lineup_id.is_none());
    assert!(
        hidden_chain
            .home
            .versions
            .iter()
            .all(|item| item.id != earliest_home.id),
        "隐藏历史不出现在链列表"
    );
    assert_eq!(
        database
            .store
            .read_lineup(earliest_home.id)
            .await
            .unwrap()
            .players
            .len(),
        11,
        "隐藏历史按 ID 保留明细"
    );
    assert!(database
        .store
        .list_team_match_lineups(home.id, 200)
        .await
        .unwrap()
        .iter()
        .all(|item| item.lineup.id != earliest_home.id));

    // 原 API 的 1..=200 clamp 与等时间 UUID 排序保持，不把内部 500 请求误写为 500 返回。
    let limit_ids = (0..201).map(|_| Uuid::new_v4()).collect::<Vec<_>>();
    sqlx::query(
        "INSERT INTO football.lineups (id,match_id,team_id,lineup_type,snapshot_type,captured_at,status) SELECT id,$2,$3,'expected','T-N',$4 + ordinal * interval '1 microsecond','superseded' FROM unnest($1::uuid[]) WITH ORDINALITY AS item(id,ordinal)",
    ).bind(&limit_ids).bind(target.id).bind(home.id).bind(kickoff - Duration::minutes(10))
        .execute(&database.pool).await.unwrap();
    let mut sorted_ids = limit_ids.clone();
    sorted_ids.reverse();
    let list_zero = database
        .store
        .list_lineups(Some(target.id), 0)
        .await
        .unwrap();
    let list_max = database
        .store
        .list_lineups(Some(target.id), u32::MAX)
        .await
        .unwrap();
    assert_eq!(list_zero.len(), 1);
    assert_eq!(list_zero[0].id, sorted_ids[0]);
    assert_eq!(list_max.len(), 200);
    assert_eq!(
        list_max.iter().map(|item| item.id).collect::<Vec<_>>(),
        sorted_ids[..200]
    );
    assert!(
        list_max.iter().all(|item| item.players.is_empty()),
        "列表保持摘要不载明细"
    );
    let team_zero = database
        .store
        .list_team_match_lineups(home.id, 0)
        .await
        .unwrap();
    let team_max = database
        .store
        .list_team_match_lineups(home.id, u32::MAX)
        .await
        .unwrap();
    assert_eq!(team_zero.len(), 1);
    assert_eq!(team_zero[0].lineup.id, sorted_ids[0]);
    assert_eq!(team_max.len(), 200);
    assert_eq!(
        team_max
            .iter()
            .map(|item| item.lineup.id)
            .collect::<Vec<_>>(),
        sorted_ids[..200]
    );
    let capped_chain = database
        .store
        .read_match_lineup_chain(target.id, "T-N")
        .await
        .unwrap();
    assert_eq!(
        capped_chain.home.versions.len() + capped_chain.away.versions.len(),
        200
    );

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
    let mut players =
        create_lineup_player_drafts(database, seed.team_id, seed.kickoff, seed.label).await;
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
    assert_eq!(
        confirmed.status,
        MatchReviewPackageWorkflowStatus::Confirmed
    );
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
    assert_eq!(
        settled_again.status,
        MatchReviewPackageWorkflowStatus::Settled
    );

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

async fn wait_for_lineup_lock(
    pool: &PgPool,
    blocker_pid: i32,
    query_pattern: &str,
    expected: i64,
) -> bool {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let waiting: i64 = sqlx::query_scalar(
                "SELECT count(*)::bigint FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid)) AND wait_event_type='Lock' AND ltrim(query) LIKE $2",
            ).bind(blocker_pid).bind(query_pattern).fetch_one(pool).await.expect("观察真实阵容锁等待");
            if waiting >= expected { return; }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    }).await.is_ok()
}
