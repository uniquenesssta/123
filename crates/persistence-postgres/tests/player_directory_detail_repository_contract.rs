use chrono::{Duration, Utc};
use football_domain::{
    AvailabilityStatus, PlayerAvailabilityDraft, PlayerDraft, PlayerListQuery, PlayerNameDraft,
    PlayerPositionDraft, PlayerStatus, PlayerTeamPeriodDraft, PreferredFoot, TeamDraft,
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
                "运行 R6-03 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-03 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-03 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn player_draft(name: String) -> PlayerDraft {
    PlayerDraft {
        canonical_name: name,
        date_of_birth: None,
        nationality_code: Some(" ZZ ".to_string()),
        preferred_foot: PreferredFoot::Right,
        height_cm: Some(181),
        status: PlayerStatus::Active,
        metadata: json!({"first": true, "shared": "old", "contract": "r6-03"}),
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn player_directory_and_detail_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();

    let created = database
        .store
        .create_player(&player_draft(format!("  R6-03 Alpha   Player {token}  ")))
        .await
        .expect("创建 R6-03 contract player");
    assert_eq!(
        created.canonical_name,
        format!("R6-03 Alpha   Player {token}")
    );
    assert_eq!(
        created.normalized_name,
        format!("r6-03 alpha player {token}")
    );
    assert_eq!(created.nationality_code.as_deref(), Some("ZZ"));
    assert_eq!(created.preferred_foot, PreferredFoot::Right);
    assert_eq!(created.height_cm, Some(181));
    assert_eq!(created.status, PlayerStatus::Active);

    let primary_name: (String, String, bool) = sqlx::query_as(
        r#"
        SELECT name, normalized_name, is_primary
        FROM football.player_names
        WHERE player_id = $1
        ORDER BY created_at, id
        LIMIT 1
        "#,
    )
    .bind(created.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取 create_player 自动创建的 primary name");
    assert_eq!(primary_name.0, created.canonical_name);
    assert_eq!(primary_name.1, created.normalized_name);
    assert!(primary_name.2);

    let blank_error = database
        .store
        .create_player(&player_draft("   ".to_string()))
        .await
        .expect_err("空白球员姓名必须失败");
    assert!(matches!(
        blank_error,
        PersistenceError::InvalidState(message) if message == "球员姓名不能为空"
    ));

    let mut invalid_height = player_draft(format!("Invalid Height {token}"));
    invalid_height.height_cm = Some(231);
    let height_error = database
        .store
        .create_player(&invalid_height)
        .await
        .expect_err("越界身高必须失败");
    assert!(matches!(
        height_error,
        PersistenceError::InvalidState(message) if message == "球员身高必须位于 120–230 cm"
    ));

    let alias = database
        .store
        .add_player_name(&PlayerNameDraft {
            player_id: created.id,
            name: format!("测试 球员 {token}"),
            language_code: Some("zh-CN".to_string()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("写入中文别名");
    assert_eq!(alias.player_id, created.id);

    let team = database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-03 Team {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r6-03"}),
        })
        .await
        .expect("创建 Player Directory contract team");

    let reference_data = database
        .store
        .player_catalog_reference_data()
        .await
        .expect("读取球员目录 reference data");
    let position_code = reference_data
        .positions
        .first()
        .expect("baseline migration 必须至少提供一个 position")
        .code
        .clone();

    database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: created.id,
            position_code: position_code.clone(),
            proficiency: 0.91,
            default_role_code: None,
            is_primary: true,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        })
        .await
        .expect("写入 primary position");

    database
        .store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: created.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(17),
            valid_from: Utc::now().date_naive() - Duration::days(1),
            valid_to: None,
            registration_status: "registered".to_string(),
            source_document_id: None,
        })
        .await
        .expect("写入当前球队效力期");

    let availability = database
        .store
        .add_player_availability(&PlayerAvailabilityDraft {
            player_id: created.id,
            team_id: Some(team.id),
            competition_id: None,
            status: AvailabilityStatus::Doubtful,
            reason: Some("  minor issue  ".to_string()),
            confidence: 0.73,
            valid_from: Utc::now() - Duration::hours(1),
            valid_to: Some(Utc::now() + Duration::days(1)),
            source_document_id: None,
            metadata: json!({"contract": "r6-03"}),
        })
        .await
        .expect("写入当前可用性");
    assert_eq!(availability.status, AvailabilityStatus::Doubtful);

    let alias_page = database
        .store
        .list_players(&PlayerListQuery {
            search: Some(format!("测试 {token}")),
            team_id: Some(team.id),
            position_code: Some(position_code.clone()),
            availability_status: Some(AvailabilityStatus::Doubtful),
            player_status: Some(PlayerStatus::Active),
            limit: 25,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("按中文别名、球队、位置、可用性联合检索");
    let listed = alias_page
        .items
        .iter()
        .find(|item| item.id == created.id)
        .expect("联合过滤必须返回目标球员");
    assert_eq!(listed.current_team_id, Some(team.id));
    assert_eq!(
        listed.primary_position_code.as_deref(),
        Some(position_code.as_str())
    );
    assert_eq!(
        listed.availability_status,
        Some(AvailabilityStatus::Doubtful)
    );
    assert_eq!(listed.availability_reason.as_deref(), Some("minor issue"));

    let cursor_error = database
        .store
        .list_players(&PlayerListQuery {
            cursor_name: Some(created.normalized_name.clone()),
            cursor_id: None,
            ..PlayerListQuery::default()
        })
        .await
        .expect_err("分页游标只给名称必须失败");
    assert!(matches!(
        cursor_error,
        PersistenceError::InvalidState(message) if message == "球员分页游标必须同时包含名称和 ID"
    ));

    let mut updated_draft = player_draft(format!(" R6-03 Beta Player {token} "));
    updated_draft.metadata = json!({"second": true, "shared": "new"});
    let updated = database
        .store
        .update_player(created.id, &updated_draft)
        .await
        .expect("更新球员");
    assert_eq!(updated.canonical_name, format!("R6-03 Beta Player {token}"));
    assert_eq!(
        updated.normalized_name,
        format!("r6-03 beta player {token}")
    );

    let primary_names: Vec<(String, bool)> = sqlx::query_as(
        r#"
        SELECT name, is_primary
        FROM football.player_names
        WHERE player_id = $1
        ORDER BY created_at, id
        "#,
    )
    .bind(created.id)
    .fetch_all(&database.pool)
    .await
    .expect("读取 update_player 后名称历史");
    assert_eq!(
        primary_names.iter().filter(|(_, primary)| *primary).count(),
        1
    );
    assert!(primary_names
        .iter()
        .any(|(name, primary)| name == &updated.canonical_name && *primary));

    let metadata: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM football.players WHERE id = $1")
            .bind(created.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取 update_player metadata merge");
    assert_eq!(metadata["first"], true);
    assert_eq!(metadata["second"], true);
    assert_eq!(metadata["shared"], "new");

    let detail = database
        .store
        .read_player(created.id)
        .await
        .expect("读取 PlayerDetail");
    assert_eq!(detail.player.id, created.id);
    assert!(detail.names.iter().any(|name| name.id == alias.id));
    assert!(detail
        .positions
        .iter()
        .any(|position| position.position_code == position_code));
    assert!(detail
        .team_periods
        .iter()
        .any(|period| period.team_id == team.id));
    assert!(detail
        .availability
        .iter()
        .any(|item| item.id == availability.id));

    let missing = database
        .store
        .read_player(Uuid::new_v4())
        .await
        .expect_err("读取不存在球员必须失败");
    assert!(matches!(
        missing,
        PersistenceError::Sqlx(sqlx::Error::RowNotFound)
    ));

    database.close().await;
}
