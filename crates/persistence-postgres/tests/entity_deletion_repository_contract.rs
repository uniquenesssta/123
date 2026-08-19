use chrono::{Duration, Utc};
use football_domain::{PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft, PreferredFoot, TeamDraft};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn deletion_preflight_and_archive_contract_is_preserved() {
    let url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url.clone(),
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("connect");
    store.migrate().await.expect("migrate");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("pool");
    let token = Uuid::new_v4().simple().to_string();

    let free = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Free {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("free team");
    let free_check = store
        .check_entity_deletion("team", free.id)
        .await
        .expect("free check");
    assert!(free_check.exists && free_check.can_permanently_delete && !free_check.must_archive);
    assert_eq!(free_check.reason, "没有历史引用，可以永久删除");

    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Archive {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("team");
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({}),
        })
        .await
        .expect("player");
    let from = Utc::now().date_naive();
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(9),
            valid_from: from,
            valid_to: Some(from + Duration::days(30)),
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("period");

    let check = store
        .check_entity_deletion("team", team.id)
        .await
        .expect("protected check");
    assert!(
        !check.can_permanently_delete && check.must_archive && check.reason.contains("只允许归档")
    );
    assert!(check
        .references
        .iter()
        .any(|item| item.relation == "player_team_periods"));

    let result = store
        .bulk_archive_entities("team", &[team.id, team.id])
        .await
        .expect("archive");
    assert_eq!(result.requested_count, 1);
    assert_eq!(result.archived_ids, vec![team.id]);

    let active: bool = sqlx::query_scalar("SELECT is_active FROM football.teams WHERE id=$1")
        .bind(team.id)
        .fetch_one(&pool)
        .await
        .expect("active");
    let periods: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1",
    )
    .bind(team.id)
    .fetch_one(&pool)
    .await
    .expect("periods");
    assert!(!active);
    assert_eq!(periods, 1);

    pool.close().await;
    store.close().await;
}
