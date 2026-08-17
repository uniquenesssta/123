use chrono::{Duration, Utc};
use football_domain::{
    PlayerAbilityObservationDraft, PlayerDraft, PlayerDynamicTagDraft,
    PlayerMatchContributionRequest, PlayerStatus, PreferredFoot,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
            panic!("运行 R6-06 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库")
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-06 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-06 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn player_abilities_and_dynamic_tags_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let player = database
        .store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-06 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".to_string()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("创建 R6-06 contract player");

    let dimensions = database
        .store
        .list_ability_dimensions()
        .await
        .expect("读取能力维度");
    assert!(!dimensions.is_empty(), "迁移基线必须提供能力维度");
    let dimension = &dimensions[0];
    let ability_value = (dimension.minimum_value + dimension.maximum_value) / 2.0;
    let observed_at = Utc::now();
    let observation = database
        .store
        .add_player_ability_observation(&PlayerAbilityObservationDraft {
            player_id: player.id,
            dimension_code: format!(" {} ", dimension.code),
            context_type: " overall ".to_string(),
            context_id: None,
            value: ability_value,
            confidence: 0.8,
            sample_size: 5,
            observed_at,
            effective_from: observed_at,
            effective_to: Some(observed_at + Duration::days(1)),
            calculation_version: " r6-06-contract ".to_string(),
            source_document_id: None,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("写入能力观察");
    assert_eq!(observation.player_id, player.id);
    assert_eq!(observation.dimension_code, dimension.code);
    assert_eq!(observation.context_type, "overall");
    assert_eq!(observation.calculation_version, "r6-06-contract");

    let invalid_observation = database
        .store
        .add_player_ability_observation(&PlayerAbilityObservationDraft {
            player_id: player.id,
            dimension_code: dimension.code.clone(),
            context_type: "overall".to_string(),
            context_id: None,
            value: ability_value,
            confidence: 1.1,
            sample_size: 5,
            observed_at,
            effective_from: observed_at,
            effective_to: None,
            calculation_version: "r6-06-contract".to_string(),
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("非法 ability confidence 必须失败");
    assert!(matches!(
        invalid_observation,
        PersistenceError::InvalidState(message) if message == "能力观察可信度或样本量无效"
    ));

    let definitions = database
        .store
        .list_dynamic_tag_definitions()
        .await
        .expect("读取动态标签定义");
    assert!(!definitions.is_empty(), "迁移基线必须提供动态标签定义");
    let definition = &definitions[0];
    let now = Utc::now();
    let tag = database
        .store
        .add_player_dynamic_tag(&PlayerDynamicTagDraft {
            player_id: player.id,
            tag_code: format!(" {} ", definition.code),
            value: definition.default_value,
            label: Some("contract".to_string()),
            confidence: 0.9,
            observed_at: now,
            valid_from: now - Duration::hours(1),
            valid_to: now + Duration::hours(2),
            competition_id: None,
            position_code: None,
            opponent_team_id: None,
            sample_size: 3,
            source_type: " manual ".to_string(),
            calculation_version: " r6-06-contract ".to_string(),
            source_document_id: None,
            metadata: json!({"contract": "r6-06"}),
        })
        .await
        .expect("写入动态标签");
    assert_eq!(tag.player_id, player.id);
    assert_eq!(tag.tag_code, definition.code);
    assert_eq!(tag.source_type, "manual");
    assert_eq!(tag.calculation_version, "r6-06-contract");

    let listed = database
        .store
        .list_player_dynamic_tags(player.id, now)
        .await
        .expect("读取动态标签");
    assert!(listed.iter().any(|item| item.id == tag.id));
    let reread = database
        .store
        .read_player_dynamic_tag(tag.id)
        .await
        .expect("按 ID 读取动态标签");
    assert_eq!(reread.id, tag.id);

    let invalid_tag = database
        .store
        .add_player_dynamic_tag(&PlayerDynamicTagDraft {
            player_id: player.id,
            tag_code: definition.code.clone(),
            value: definition.default_value,
            label: None,
            confidence: 1.1,
            observed_at: now,
            valid_from: now,
            valid_to: now + Duration::hours(1),
            competition_id: None,
            position_code: None,
            opponent_team_id: None,
            sample_size: 1,
            source_type: "manual".to_string(),
            calculation_version: "r6-06-contract".to_string(),
            source_document_id: None,
            metadata: json!({}),
        })
        .await
        .expect_err("非法 dynamic tag confidence 必须失败");
    assert!(matches!(
        invalid_tag,
        PersistenceError::InvalidState(message) if message == "动态标签 confidence 必须在 0–1 之间"
    ));

    let contribution = database
        .store
        .calculate_player_match_contribution(&PlayerMatchContributionRequest {
            player_id: player.id,
            match_id: None,
            competition_id: None,
            position_code: None,
            role_code: None,
            role_origin: None,
            role_source_position_code: None,
            opponent_team_id: None,
            as_of: now,
            data_cutoff_time: None,
            expected_minutes: Some(90),
        })
        .await
        .expect("计算球员比赛贡献");
    assert_eq!(contribution.player_id, player.id);
    assert_eq!(
        contribution.calculation_version,
        "match-contribution-v2-role-context"
    );
    assert!(contribution.effective_contribution >= 0.0);

    let row = sqlx::query(
        "SELECT count(*)::bigint AS count FROM feature.player_ability_observations WHERE player_id=$1",
    )
    .bind(player.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取能力观察落库数量");
    let observation_count: i64 = row.try_get("count").expect("读取 count");
    assert!(observation_count >= 1);

    database.close().await;
}
