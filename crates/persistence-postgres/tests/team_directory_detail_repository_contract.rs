use football_domain::{TeamDraft, TeamListQuery, TeamNameDraft, TeamProfileDraft};
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
                "运行 R6-01 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-01 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-01 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn profile_draft(short_name: &str) -> TeamProfileDraft {
    TeamProfileDraft {
        short_name: Some(short_name.to_string()),
        team_type: "club".to_string(),
        founded_year: Some(1999),
        city: Some("Test City".to_string()),
        stadium: Some("Test Stadium".to_string()),
        head_coach: None,
        default_formation: Some("4-3-3".to_string()),
        tactical_style: "balanced".to_string(),
        attack_rating: Some(71.0),
        midfield_rating: Some(72.0),
        defence_rating: Some(73.0),
        goalkeeper_rating: Some(74.0),
        reputation: Some(75.0),
        data_confidence: 0.88,
        notes: Some("r6-01 contract".to_string()),
        metadata: json!({"contract": "r6-01"}),
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn team_directory_and_detail_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let alpha_name = format!("R6 Alpha {token}");
    let beta_name = format!("R6 Beta {token}");
    let alias = format!("银河俱乐部 {token}");

    let alpha = database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("  {alpha_name}  "),
            country_code: Some(" zz ".to_string()),
            metadata: json!({"source": "r6-01-contract", "created": true}),
        })
        .await
        .expect("创建 R6-01 alpha 球队");
    let beta = database
        .store
        .create_team(&TeamDraft {
            canonical_name: beta_name.clone(),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"source": "r6-01-contract"}),
        })
        .await
        .expect("创建 R6-01 beta 球队");

    assert_eq!(alpha.canonical_name, alpha_name);
    assert_eq!(alpha.normalized_name, alpha_name.to_lowercase());
    assert_eq!(alpha.country_code.as_deref(), Some("zz"));
    assert!(alpha.is_active);

    database
        .store
        .add_team_name(&TeamNameDraft {
            team_id: alpha.id,
            name: format!("  {alias}  "),
            language_code: Some("zh-CN".to_string()),
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("添加 R6-01 球队中文别名");
    database
        .store
        .upsert_team_profile(alpha.id, &profile_draft("Alpha"))
        .await
        .expect("写入 R6-01 alpha profile");
    database
        .store
        .upsert_team_profile(beta.id, &profile_draft("Beta"))
        .await
        .expect("写入 R6-01 beta profile");

    let options = database
        .store
        .list_team_options(Some(&alias), 20)
        .await
        .expect("按中文别名搜索球队 option");
    let option = options
        .iter()
        .find(|item| item.id == alpha.id)
        .expect("中文别名应命中 alpha");
    assert_eq!(option.team_type, "club");

    let first_page = database
        .store
        .list_teams(&TeamListQuery {
            search: Some(token.clone()),
            country_code: Some("zz".to_string()),
            team_type: Some("club".to_string()),
            active_only: true,
            limit: 1,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("读取 R6-01 第一页球队目录");
    assert_eq!(first_page.items.len(), 1);
    assert!(first_page.has_more);
    assert!(first_page.next_cursor_name.is_some());
    assert!(first_page.next_cursor_id.is_some());

    let second_page = database
        .store
        .list_teams(&TeamListQuery {
            search: Some(token.clone()),
            country_code: Some("ZZ".to_string()),
            team_type: Some("CLUB".to_string()),
            active_only: true,
            limit: 1,
            cursor_name: first_page.next_cursor_name.clone(),
            cursor_id: first_page.next_cursor_id,
        })
        .await
        .expect("读取 R6-01 第二页球队目录");
    assert_eq!(second_page.items.len(), 1);
    assert!(!second_page.has_more);
    assert_ne!(first_page.items[0].id, second_page.items[0].id);

    let invalid_cursor = database
        .store
        .list_teams(&TeamListQuery {
            search: None,
            country_code: None,
            team_type: None,
            active_only: true,
            limit: 20,
            cursor_name: Some("only-name".to_string()),
            cursor_id: None,
        })
        .await;
    assert!(matches!(
        invalid_cursor,
        Err(PersistenceError::InvalidState(message))
            if message == "球队分页游标必须同时包含名称和 ID"
    ));

    let player_id = Uuid::new_v4();
    let localized_player_name = format!("测试球员 {token}");
    let position_code: String = sqlx::query_scalar(
        "SELECT code FROM football.positions ORDER BY sort_order, code LIMIT 1",
    )
    .fetch_one(&database.pool)
    .await
    .expect("读取内置位置");
    sqlx::query(
        r#"
        INSERT INTO football.players (id, canonical_name, normalized_name, status)
        VALUES ($1,$2,$3,'active')
        "#,
    )
    .bind(player_id)
    .bind(format!("Player {token}"))
    .bind(format!("player {token}"))
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 squad player");
    sqlx::query(
        r#"
        INSERT INTO football.player_names (
            id, player_id, name, normalized_name, language_code, is_primary
        ) VALUES ($1,$2,$3,$4,'zh-CN',false)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(&localized_player_name)
    .bind(&localized_player_name)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 localized player name");
    sqlx::query(
        r#"
        INSERT INTO football.player_team_periods (
            id, player_id, team_id, squad_number, valid_from, registration_status
        ) VALUES ($1,$2,$3,9,current_date - 30,'registered')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(alpha.id)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 player team period");
    sqlx::query(
        r#"
        INSERT INTO football.player_positions (
            id, player_id, position_code, proficiency, is_primary, valid_from
        ) VALUES ($1,$2,$3,0.9,true,current_date - 30)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(&position_code)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 player position");
    sqlx::query(
        r#"
        INSERT INTO football.player_availability (
            id, player_id, team_id, status, reason, confidence, valid_from
        ) VALUES ($1,$2,$3,'injured','contract fixture',0.9,now() - interval '1 hour')
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(player_id)
    .bind(alpha.id)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 player availability");

    let match_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO football.matches (
            id, external_key, home_team_id, away_team_id, kickoff_time, status
        ) VALUES ($1,$2,$3,$4,now() - interval '2 days','finished')
        "#,
    )
    .bind(match_id)
    .bind(format!("R6-01-{token}"))
    .bind(alpha.id)
    .bind(beta.id)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 recent match");
    sqlx::query(
        r#"
        INSERT INTO football.match_results (
            match_id, home_goals_90, away_goals_90, finalized_at
        ) VALUES ($1,2,1,now())
        "#,
    )
    .bind(match_id)
    .execute(&database.pool)
    .await
    .expect("创建 R6-01 recent result");

    let filtered = database
        .store
        .list_teams(&TeamListQuery {
            search: Some(alias.clone()),
            country_code: Some("ZZ".to_string()),
            team_type: Some("club".to_string()),
            active_only: true,
            limit: 20,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("读取带 squad 聚合的球队目录");
    let alpha_list = filtered
        .items
        .iter()
        .find(|item| item.id == alpha.id)
        .expect("目录应包含 alpha");
    assert_eq!(alpha_list.current_player_count, 1);
    assert_eq!(alpha_list.unavailable_player_count, 1);
    assert_eq!(alpha_list.profile_confidence, Some(0.88));

    let detail = database
        .store
        .read_team(alpha.id)
        .await
        .expect("读取 R6-01 球队详情");
    assert_eq!(detail.team.id, alpha.id);
    assert!(detail.names.iter().any(|item| item.name == alias));
    assert_eq!(
        detail.profile.as_ref().map(|item| item.team_type.as_str()),
        Some("club")
    );
    assert_eq!(detail.squad.len(), 1);
    assert_eq!(detail.squad[0].player_id, player_id);
    assert_eq!(
        detail.squad[0].localized_name.as_deref(),
        Some(localized_player_name.as_str())
    );
    assert_eq!(
        detail.squad[0]
            .availability_status
            .map(|status| status.as_str()),
        Some("injured")
    );
    assert_eq!(detail.player_periods.len(), 1);
    assert_eq!(detail.recent_matches.len(), 1);
    assert_eq!(detail.recent_matches[0].match_id, match_id);
    assert_eq!(detail.recent_matches[0].goals_for, Some(2));
    assert_eq!(detail.recent_matches[0].goals_against, Some(1));

    let updated_name = format!("R6 Alpha Updated {token}");
    let updated = database
        .store
        .update_team(
            alpha.id,
            &TeamDraft {
                canonical_name: format!("  {updated_name}  "),
                country_code: Some("YY".to_string()),
                metadata: json!({"updated_by": "r6-01-contract"}),
            },
        )
        .await
        .expect("更新 R6-01 球队");
    assert_eq!(updated.canonical_name, updated_name);
    assert_eq!(updated.normalized_name, updated_name.to_lowercase());
    assert_eq!(updated.country_code.as_deref(), Some("YY"));

    let read_after_update = database
        .store
        .read_team(alpha.id)
        .await
        .expect("读取更新后的 R6-01 球队");
    assert_eq!(read_after_update.team.canonical_name, updated_name);

    assert!(database.store.read_team(Uuid::new_v4()).await.is_err());
    assert!(database
        .store
        .update_team(
            Uuid::new_v4(),
            &TeamDraft {
                canonical_name: "missing".to_string(),
                country_code: None,
                metadata: json!({}),
            },
        )
        .await
        .is_err());

    database.close().await;
}
