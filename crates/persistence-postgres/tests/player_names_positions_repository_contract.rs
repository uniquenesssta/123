use chrono::NaiveDate;
use football_domain::{
    PlayerDraft, PlayerNameDraft, PlayerPositionDraft, PlayerStatus, PreferredFoot,
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
      "运行 R6-04 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
  )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-04 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-04 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn player_names_and_positions_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let player = database
        .store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-04 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".to_string()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({"contract": "r6-04"}),
        })
        .await
        .expect("创建 R6-04 contract player");

    let alias = database
        .store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: format!("  测试   球员 {token}  "),
            language_code: Some(" zh-CN ".to_string()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("添加非主名称");
    assert_eq!(alias.name, format!("测试   球员 {token}"));
    assert_eq!(alias.normalized_name, format!("测试 球员 {token}"));
    assert_eq!(alias.language_code.as_deref(), Some("zh-CN"));
    assert!(!alias.is_primary);

    let primary = database
        .store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: format!("  Primary   Name {token}  "),
            language_code: Some(" en ".to_string()),
            is_primary: true,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("切换主名称");
    assert_eq!(primary.name, format!("Primary   Name {token}"));
    assert_eq!(primary.normalized_name, format!("primary name {token}"));
    assert!(primary.is_primary);
    let player_identity: (String, String) = sqlx::query_as(
        "SELECT canonical_name, normalized_name FROM football.players WHERE id = $1",
    )
    .bind(player.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取主名称投影");
    assert_eq!(player_identity.0, primary.name);
    assert_eq!(player_identity.1, primary.normalized_name);
    let primary_name_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_names WHERE player_id = $1 AND is_primary",
    )
    .bind(player.id)
    .fetch_one(&database.pool)
    .await
    .expect("统计主名称");
    assert_eq!(primary_name_count, 1);

    let blank_name = database
        .store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: "   ".to_string(),
            language_code: None,
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect_err("空白名称必须失败");
    assert!(matches!(
        blank_name,
        PersistenceError::InvalidState(message) if message == "球员名称不能为空"
    ));
    let reversed_name_dates = database
        .store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: "date-check".to_string(),
            language_code: None,
            is_primary: false,
            valid_from: NaiveDate::from_ymd_opt(2026, 2, 2),
            valid_to: NaiveDate::from_ymd_opt(2026, 2, 1),
        })
        .await
        .expect_err("倒置名称日期必须失败");
    assert!(matches!(
        reversed_name_dates,
        PersistenceError::InvalidState(message) if message == "球员名称结束日期不能早于开始日期"
    ));

    let positions = database
        .store
        .list_positions()
        .await
        .expect("读取内置位置目录");
    assert!(positions.len() >= 2, "R6-04 contract 至少需要两个内置位置");
    let first = database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: player.id,
            position_code: format!(" {} ", positions[0].code.to_lowercase()),
            proficiency: 0.91,
            default_role_code: Some(" advanced-forward ".to_string()),
            is_primary: true,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        })
        .await
        .expect("写入第一主位置");
    assert_eq!(first.position_code, positions[0].code);
    assert_eq!(first.default_role_code.as_deref(), Some("advanced-forward"));
    assert!(first.is_primary);

    let second = database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: player.id,
            position_code: positions[1].code.clone(),
            proficiency: 0.88,
            default_role_code: None,
            is_primary: true,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        })
        .await
        .expect("切换主位置");
    assert!(second.is_primary);
    let first_is_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM football.player_positions WHERE id = $1")
            .bind(first.id)
            .fetch_one(&database.pool)
            .await
            .expect("读取第一位置 primary 状态");
    assert!(!first_is_primary);

    let invalid_proficiency = database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: player.id,
            position_code: positions[0].code.clone(),
            proficiency: 1.1,
            default_role_code: None,
            is_primary: false,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        })
        .await
        .expect_err("越界熟练度必须失败");
    assert!(matches!(
        invalid_proficiency,
        PersistenceError::InvalidState(message) if message == "位置熟练度必须位于 0–1"
    ));

    let reversed_position_dates = database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: player.id,
            position_code: positions[0].code.clone(),
            proficiency: 0.5,
            default_role_code: None,
            is_primary: false,
            valid_from: NaiveDate::from_ymd_opt(2026, 2, 2),
            valid_to: NaiveDate::from_ymd_opt(2026, 2, 1),
            source_document_id: None,
        })
        .await
        .expect_err("倒置位置日期必须失败");
    assert!(matches!(
        reversed_position_dates,
        PersistenceError::InvalidState(message) if message == "球员位置结束日期不能早于开始日期"
    ));

    let oversized_role = database
        .store
        .assign_player_position(&PlayerPositionDraft {
            player_id: player.id,
            position_code: positions[0].code.clone(),
            proficiency: 0.5,
            default_role_code: Some("x".repeat(81)),
            is_primary: false,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        })
        .await
        .expect_err("超长默认战术角色必须失败");
    assert!(matches!(
        oversized_role,
        PersistenceError::InvalidState(message) if message == "默认战术角色不能超过 80 个字符"
    ));

    database.close().await;
}
