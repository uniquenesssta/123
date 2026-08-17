use chrono::{Duration, NaiveDate, Utc};
use football_domain::{
    AvailabilityStatus, PlayerAvailabilityDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
            panic!(
                "运行 R6-05 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-05 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-05 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn player_team_periods_and_availability_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let team = database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-05 Team {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r6-05"}),
        })
        .await
        .expect("创建 R6-05 contract team");
    let player = database
        .store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-05 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".to_string()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({"contract": "r6-05"}),
        })
        .await
        .expect("创建 R6-05 contract player");

    let valid_from = NaiveDate::from_ymd_opt(2026, 8, 1).expect("固定有效日期");
    let period = database
        .store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(17),
            valid_from,
            valid_to: Some(valid_from + Duration::days(30)),
            registration_status: " registered ".to_string(),
            source_document_id: None,
        })
        .await
        .expect("写入球员球队效力期");
    assert_eq!(period.player_id, player.id);
    assert_eq!(period.team_id, team.id);
    assert_eq!(period.team_name, team.canonical_name);
    assert_eq!(period.squad_number, Some(17));
    assert_eq!(period.registration_status, "registered");

    let reversed_period = database
        .store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(18),
            valid_from: valid_from + Duration::days(2),
            valid_to: Some(valid_from + Duration::days(1)),
            registration_status: "registered".to_string(),
            source_document_id: None,
        })
        .await
        .expect_err("倒置效力期必须失败");
    assert!(matches!(
        reversed_period,
        PersistenceError::InvalidState(message) if message == "球队效力结束日期不能早于开始日期"
    ));

    let invalid_squad_number = database
        .store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(100),
            valid_from,
            valid_to: None,
            registration_status: "registered".to_string(),
            source_document_id: None,
        })
        .await
        .expect_err("越界球衣号码必须失败");
    assert!(matches!(
        invalid_squad_number,
        PersistenceError::InvalidState(message) if message == "球衣号码必须位于 0–99"
    ));

    let availability_from = Utc::now();
    let availability = database
        .store
        .add_player_availability(&PlayerAvailabilityDraft {
            player_id: player.id,
            team_id: Some(team.id),
            competition_id: None,
            status: AvailabilityStatus::Injured,
            reason: Some("  hamstring  ".to_string()),
            confidence: 0.85,
            valid_from: availability_from,
            valid_to: Some(availability_from + Duration::days(5)),
            source_document_id: None,
            metadata: json!({"contract": "r6-05"}),
        })
        .await
        .expect("写入球员可用性");
    assert_eq!(availability.player_id, player.id);
    assert_eq!(availability.team_id, Some(team.id));
    assert_eq!(
        availability.team_name.as_deref(),
        Some(team.canonical_name.as_str())
    );
    assert_eq!(availability.status, AvailabilityStatus::Injured);
    assert_eq!(availability.reason.as_deref(), Some("hamstring"));
    assert_eq!(availability.confidence, 0.85);

    let invalid_confidence = database
        .store
        .add_player_availability(&PlayerAvailabilityDraft {
            player_id: player.id,
            team_id: Some(team.id),
            competition_id: None,
            status: AvailabilityStatus::Available,
            reason: None,
            confidence: 1.01,
            valid_from: availability_from,
            valid_to: None,
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("越界可用性可信度必须失败");
    assert!(matches!(
        invalid_confidence,
        PersistenceError::InvalidState(message) if message == "可用性可信度必须位于 0–1"
    ));

    let reversed_availability = database
        .store
        .add_player_availability(&PlayerAvailabilityDraft {
            player_id: player.id,
            team_id: Some(team.id),
            competition_id: None,
            status: AvailabilityStatus::Returning,
            reason: Some("return".to_string()),
            confidence: 0.5,
            valid_from: availability_from + Duration::days(2),
            valid_to: Some(availability_from + Duration::days(1)),
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("倒置可用性时间必须失败");
    assert!(matches!(
        reversed_availability,
        PersistenceError::InvalidState(message) if message == "可用性结束时间不能早于开始时间"
    ));

    let detail = database
        .store
        .read_player(player.id)
        .await
        .expect("读取 Player Detail");
    assert!(detail.team_periods.iter().any(|item| item.id == period.id));
    assert!(detail
        .availability
        .iter()
        .any(|item| item.id == availability.id));

    let stored_reason: Option<String> =
        sqlx::query_scalar("SELECT reason FROM football.player_availability WHERE id = $1")
            .bind(availability.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取数据库可用性原因");
    assert_eq!(stored_reason.as_deref(), Some("hamstring"));

    database.close().await;
}
