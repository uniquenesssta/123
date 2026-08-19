use chrono::Utc;
use football_domain::{
    DataProviderDraft, ExternalEntityIdDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft, TeamForceDeleteRequest,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn team_force_delete_preserves_confirmation_scope_and_cleanup_contract() {
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
            code: format!("r6_09_force_{token}"),
            name: format!("R6-09 Force Provider {token}"),
            provider_type: "official".into(),
            base_url: Some("https://example.test/r6-09-force".into()),
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("provider");
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Force Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("team");
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Force Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("player");
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(9),
            valid_from: Utc::now().date_naive(),
            valid_to: None,
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("team period");
    for (entity_type, entity_id, external_id) in [
        ("team", team.id, format!("FORCE-TEAM-{token}")),
        ("player", player.id, format!("FORCE-PLAYER-{token}")),
    ] {
        store
            .add_external_entity_id(&ExternalEntityIdDraft {
                provider_id: provider.id,
                entity_type: entity_type.into(),
                entity_id,
                external_id,
                metadata: json!({}),
            })
            .await
            .expect("external id");
    }

    let preview = store
        .preview_force_delete_team(team.id)
        .await
        .expect("force-delete preview");
    assert_eq!(preview.team_id, team.id);
    assert_eq!(preview.label, team.canonical_name);
    assert_eq!(preview.confirmation_text, team.canonical_name);
    assert!(preview.total_rows >= 3);
    assert!(preview
        .references
        .iter()
        .any(|item| item.relation == "teams" && item.count == 1));
    assert!(preview
        .references
        .iter()
        .any(|item| item.relation == "players" && item.count >= 1));
    assert!(preview
        .references
        .iter()
        .any(|item| item.relation == "player_team_periods" && item.count >= 1));

    let wrong = store
        .force_delete_team(&TeamForceDeleteRequest {
            team_id: team.id,
            confirmation_text: format!("{}-wrong", team.canonical_name),
        })
        .await
        .expect_err("wrong confirmation must be rejected");
    assert!(wrong.to_string().contains("确认文字不匹配"));
    let still_present: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 AND player_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("period after rejected confirmation");
    assert_eq!(still_present, 1);

    let result = store
        .force_delete_team(&TeamForceDeleteRequest {
            team_id: team.id,
            confirmation_text: team.canonical_name.clone(),
        })
        .await
        .expect("force delete");
    assert_eq!(result.team_id, team.id);
    assert_eq!(result.label, team.canonical_name);
    assert!(result.deleted_player_ids.contains(&player.id));
    assert!(
        result
            .deleted_counts
            .get("players")
            .copied()
            .unwrap_or_default()
            >= 1
    );
    assert!(
        result
            .deleted_counts
            .get("player_team_periods")
            .copied()
            .unwrap_or_default()
            >= 1
    );

    let team_count: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.teams WHERE id=$1")
            .bind(team.id)
            .fetch_one(&pool)
            .await
            .expect("team count");
    let player_count: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM football.players WHERE id=$1")
            .bind(player.id)
            .fetch_one(&pool)
            .await
            .expect("player count");
    let period_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 OR player_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("period count");
    let external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_id=$1 OR entity_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("external id count");
    let tombstone_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='team_force_deleted' AND entity_type='team_purge' AND entity_id=$1",
    )
    .bind(team.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("force-delete tombstone");
    let provider_count: i64 =
        sqlx::query_scalar("SELECT count(*)::bigint FROM catalog.data_providers WHERE id=$1")
            .bind(provider.id)
            .fetch_one(&pool)
            .await
            .expect("provider count");

    assert_eq!(team_count, 0);
    assert_eq!(player_count, 0);
    assert_eq!(period_count, 0);
    assert_eq!(external_count, 0);
    assert_eq!(tombstone_count, 1);
    assert_eq!(provider_count, 1);

    pool.close().await;
    store.close().await;
}
