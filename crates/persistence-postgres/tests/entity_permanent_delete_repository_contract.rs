use chrono::Utc;
use football_domain::{
    DataProviderDraft, ExternalEntityIdDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn safe_permanent_delete_contract_is_preserved() {
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
    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r6_09_delete_{token}"),
            name: format!("R6-09 Delete Provider {token}"),
            provider_type: "official".into(),
            base_url: Some("https://example.test/r6-09-delete".into()),
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("provider");

    let free_team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Delete Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("free team");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: free_team.id,
            external_id: format!("TEAM-{token}"),
            metadata: json!({}),
        })
        .await
        .expect("team external id");
    let team_result = store
        .bulk_delete_teams(&[free_team.id, free_team.id])
        .await
        .expect("bulk delete free team");
    assert_eq!(team_result.requested_count, 1);
    assert_eq!(team_result.deleted_ids, vec![free_team.id]);
    assert!(team_result.blocked.is_empty());
    let team_count: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.teams WHERE id=$1")
            .bind(free_team.id)
            .fetch_one(&pool)
            .await
            .expect("team count");
    let team_external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_type='team' AND entity_id=$1",
    )
    .bind(free_team.id)
    .fetch_one(&pool)
    .await
    .expect("team external count");
    let team_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='team_deleted' AND entity_type='team' AND entity_id=$1",
    )
    .bind(free_team.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("team audit count");
    assert_eq!(team_count, 0);
    assert_eq!(team_external_count, 0);
    assert_eq!(team_audit_count, 1);

    let free_player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Delete Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("free player");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "player".into(),
            entity_id: free_player.id,
            external_id: format!("PLAYER-{token}"),
            metadata: json!({}),
        })
        .await
        .expect("player external id");
    let player_result = store
        .bulk_delete_players(&[free_player.id, free_player.id])
        .await
        .expect("bulk delete free player");
    assert_eq!(player_result.requested_count, 1);
    assert_eq!(player_result.deleted_ids, vec![free_player.id]);
    assert!(player_result.blocked.is_empty());
    let player_count: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.players WHERE id=$1")
            .bind(free_player.id)
            .fetch_one(&pool)
            .await
            .expect("player count");
    let player_external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_type='player' AND entity_id=$1",
    )
    .bind(free_player.id)
    .fetch_one(&pool)
    .await
    .expect("player external count");
    let player_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='player_deleted' AND entity_type='player' AND entity_id=$1",
    )
    .bind(free_player.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("player audit count");
    assert_eq!(player_count, 0);
    assert_eq!(player_external_count, 0);
    assert_eq!(player_audit_count, 1);

    let protected_team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Protected Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("protected team");
    let protected_player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Protected Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Left,
            height_cm: Some(178),
            status: PlayerStatus::Active,
            metadata: json!({}),
        })
        .await
        .expect("protected player");
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: protected_player.id,
            team_id: protected_team.id,
            season_id: None,
            squad_number: Some(7),
            valid_from: Utc::now().date_naive(),
            valid_to: None,
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("protected period");

    let blocked_team = store
        .bulk_delete_teams(&[protected_team.id])
        .await
        .expect("protected team result");
    assert!(blocked_team.deleted_ids.is_empty());
    assert_eq!(blocked_team.blocked.len(), 1);
    assert!(blocked_team.blocked[0].reason.contains("只允许归档"));
    let blocked_player = store
        .bulk_delete_players(&[protected_player.id])
        .await
        .expect("protected player result");
    assert!(blocked_player.deleted_ids.is_empty());
    assert_eq!(blocked_player.blocked.len(), 1);
    assert!(blocked_player.blocked[0].reason.contains("只允许归档"));
    let period_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 AND player_id=$2",
    )
    .bind(protected_team.id)
    .bind(protected_player.id)
    .fetch_one(&pool)
    .await
    .expect("protected period count");
    assert_eq!(period_count, 1);

    pool.close().await;
    store.close().await;
}
