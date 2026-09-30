use chrono::Utc;
use football_domain::{
    DataProviderDraft, ExternalEntityIdDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{str::FromStr, time::Duration};
use tokio::{
    task::{JoinError, JoinHandle},
    time::{sleep, timeout},
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn safe_permanent_delete_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let test_database = database.clone();
    let result = tokio::spawn(async move {
        verify_original_contract(test_database).await;
    })
    .await;
    database.finish(result).await;
}

async fn verify_original_contract(database: TestDatabase) {
    let store = database.store.clone();
    let pool = database.pool.clone();
    let token = database.token.clone();
    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r6_09_delete_{token}"),
            name: format!("R6-09 Delete Provider {token}"),
            provider_type: "official".into(),
            base_url: Some("https://example.test/r6-09-delete".into()),
            metadata: json!({"contract":"r6-09-at2","fixture":token}),
        })
        .await
        .expect("provider");

    let free_team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Delete Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({"contract":"r6-09-at2","fixture":token}),
        })
        .await
        .expect("free team");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: free_team.id,
            external_id: format!("TEAM-{token}"),
            metadata: json!({"fixture":token}),
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
            metadata: json!({"contract":"r6-09-at2","fixture":token}),
        })
        .await
        .expect("free player");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "player".into(),
            entity_id: free_player.id,
            external_id: format!("PLAYER-{token}"),
            metadata: json!({"fixture":token}),
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
            metadata: json!({"fixture":token}),
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
            metadata: json!({"fixture":token}),
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
}

fn test_database_options(url: &str) -> Result<PgConnectOptions, &'static str> {
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return Err("测试库必须使用 PostgreSQL URL");
    }
    let options = PgConnectOptions::from_str(url).map_err(|_| "测试库 URL 无效")?;
    if !options
        .get_database()
        .is_some_and(|name| name.to_lowercase().contains("test"))
    {
        return Err("已拒绝连接：专用测试数据库名称必须包含 test");
    }
    Ok(options)
}

#[test]
fn safe_delete_rejects_non_test_database_before_connecting() {
    for url in [
        "postgres://localhost/football",
        "postgres://localhost/",
        "sqlite://football_test",
        "not-a-url",
    ] {
        assert!(test_database_options(url).is_err());
    }
    assert!(test_database_options("postgresql://localhost/FOOTBALL_TEST").is_ok());
}

#[derive(Clone)]
struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
    token: String,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url =
            std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
        let options = test_database_options(&connection_url).expect("删除契约测试库前检失败");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .expect("test pool");
        let name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .expect("actual database");
        assert!(name.to_lowercase().contains("test"));
        let store = PostgresStore::connect(&DatabaseOptions {
            connection_url,
            max_connections: 4,
            connect_timeout_seconds: 10,
        })
        .await
        .expect("store");
        store.migrate().await.expect("migrate");
        Self {
            store,
            pool,
            token: Uuid::new_v4().simple().to_string(),
        }
    }

    async fn cleanup(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for query in [
            "DELETE FROM feature.player_dynamic_tags WHERE player_id IN (SELECT id FROM football.players WHERE metadata->>'fixture'=$1)",
            "DELETE FROM football.player_team_periods WHERE player_id IN (SELECT id FROM football.players WHERE metadata->>'fixture'=$1) OR team_id IN (SELECT id FROM football.teams WHERE metadata->>'fixture'=$1)",
            "DELETE FROM football.team_lineup_presets WHERE team_id IN (SELECT id FROM football.teams WHERE metadata->>'fixture'=$1)",
            "DELETE FROM football.external_entity_ids WHERE provider_id IN (SELECT id FROM catalog.data_providers WHERE metadata->>'fixture'=$1)",
            "DELETE FROM football.players WHERE metadata->>'fixture'=$1",
            "DELETE FROM football.teams WHERE metadata->>'fixture'=$1",
            "DELETE FROM catalog.data_providers WHERE metadata->>'fixture'=$1",
        ] { sqlx::query(query).bind(&self.token).execute(&mut *tx).await?; }
        tx.commit().await
    }

    async fn finish(self, result: Result<(), JoinError>) {
        // Clean only this invocation's fixtures even after a failed assertion; retain audit.
        let cleanup = self.cleanup().await;
        self.store.close().await;
        self.pool.close().await;
        if let Err(error) = result {
            if error.is_panic() {
                std::panic::resume_unwind(error.into_panic());
            }
            panic!("safe delete test task cancelled: {error}");
        }
        cleanup.expect("cleanup safe delete fixture");
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn safe_delete_rechecks_concurrent_history_after_parent_lock() {
    let database = TestDatabase::connect().await;
    let test_database = database.clone();
    let result = tokio::spawn(async move {
        for entity_type in ["player", "team"] {
            for commit_reference in [true, false] {
                verify_reference_interleaving(&test_database, entity_type, commit_reference).await;
            }
        }
    })
    .await;
    database.finish(result).await;
}

fn spawn_delete(
    store: PostgresStore,
    entity_type: &'static str,
    id: Uuid,
) -> JoinHandle<Result<(), String>> {
    tokio::spawn(async move {
        if entity_type == "player" {
            store
                .delete_player(id)
                .await
                .map_err(|error| error.to_string())
        } else {
            let result = store
                .bulk_delete_teams(&[id])
                .await
                .map_err(|error| error.to_string())?;
            assert_eq!(result.requested_count, 1);
            if result.blocked.is_empty() {
                assert_eq!(result.deleted_ids, vec![id]);
                Ok(())
            } else {
                assert!(result.deleted_ids.is_empty());
                assert_eq!(result.blocked.len(), 1);
                assert_eq!(result.blocked[0].id, id);
                Err(result.blocked[0].reason.clone())
            }
        }
    })
}

async fn wait_for_parent_lock(pool: &PgPool, writer_pid: i32) -> bool {
    timeout(Duration::from_secs(10), async {
        loop {
            let waiting: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE $1 = ANY(pg_blocking_pids(pid)) AND wait_event_type='Lock' AND query LIKE 'SELECT canonical_name FROM football.%FOR UPDATE')"
            ).bind(writer_pid).fetch_one(pool).await.expect("observe parent lock wait");
            if waiting { return; }
            // Poll an observed lock condition, rather than guessing transaction order with sleep.
            sleep(Duration::from_millis(20)).await;
        }
    }).await.is_ok()
}

async fn audit_count(pool: &PgPool, entity_type: &str, id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type=$1 AND entity_id=$2",
    )
    .bind(format!("{entity_type}_deleted"))
    .bind(id.to_string())
    .fetch_one(pool)
    .await
    .expect("delete audit count")
}

async fn verify_reference_interleaving(
    database: &TestDatabase,
    entity_type: &'static str,
    commit_reference: bool,
) {
    let store = &database.store;
    let pool = &database.pool;
    let token = &database.token;
    let name = format!("R7-03 {entity_type} {commit_reference} {token}");
    let id = if entity_type == "player" {
        store
            .create_player(&PlayerDraft {
                canonical_name: name,
                date_of_birth: None,
                nationality_code: None,
                preferred_foot: PreferredFoot::Unknown,
                height_cm: None,
                status: PlayerStatus::Active,
                metadata: json!({"fixture":token}),
            })
            .await
            .expect("race player")
            .id
    } else {
        store
            .create_team(&TeamDraft {
                canonical_name: name,
                country_code: None,
                metadata: json!({"fixture":token}),
            })
            .await
            .expect("race team")
            .id
    };
    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r7_03_{}", Uuid::new_v4().simple()),
            name: format!("R7-03 Provider {id}"),
            provider_type: "official".into(),
            base_url: None,
            metadata: json!({"fixture":token}),
        })
        .await
        .expect("race provider");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: entity_type.into(),
            entity_id: id,
            external_id: id.to_string(),
            metadata: json!({"keep":"original"}),
        })
        .await
        .expect("external binding");
    let preflight = store
        .check_entity_deletion(entity_type, id)
        .await
        .expect("initial preflight");
    assert!(preflight.can_permanently_delete);

    let mut writer = pool.begin().await.expect("history writer");
    let writer_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *writer)
        .await
        .expect("writer pid");
    let history_id = Uuid::new_v4();
    if entity_type == "player" {
        sqlx::query("INSERT INTO feature.player_dynamic_tags (id,player_id,tag_code,value,confidence,observed_at,valid_from,valid_to,calculation_version,metadata) VALUES ($1,$2,'match_readiness',1,1,now(),now(),now()+interval '1 day','r7-03-contract',$3)")
            .bind(history_id).bind(id).bind(json!({"fixture":token})).execute(&mut *writer).await.expect("uncommitted dynamic tag");
    } else {
        sqlx::query("INSERT INTO football.team_lineup_presets (id,team_id,name) VALUES ($1,$2,$3)")
            .bind(history_id)
            .bind(id)
            .bind(format!("R7-03 Preset {token}"))
            .execute(&mut *writer)
            .await
            .expect("uncommitted preset");
    }
    assert!(
        store
            .check_entity_deletion(entity_type, id)
            .await
            .expect("uncommitted history invisible to preflight")
            .can_permanently_delete
    );
    let mut deleting = spawn_delete(store.clone(), entity_type, id);
    if !wait_for_parent_lock(pool, writer_pid).await {
        deleting.abort();
        writer
            .rollback()
            .await
            .expect("rollback failed test writer");
        let _ = deleting.await;
        panic!("delete did not wait for the parent FOR UPDATE lock");
    }
    if commit_reference {
        writer.commit().await.expect("commit concurrent history");
    } else {
        writer
            .rollback()
            .await
            .expect("rollback concurrent history");
    }
    let outcome = match timeout(Duration::from_secs(10), &mut deleting).await {
        Ok(result) => result.expect("delete task"),
        Err(_) => {
            deleting.abort();
            let _ = deleting.await;
            panic!("delete did not resume after writer");
        }
    };
    if commit_reference {
        assert!(outcome
            .expect_err("committed history must block deletion")
            .contains("只允许归档"));
    } else {
        outcome.expect("rolled back history allows deletion");
    }
    let table = if entity_type == "player" {
        "football.players"
    } else {
        "football.teams"
    };
    let history_table = if entity_type == "player" {
        "feature.player_dynamic_tags"
    } else {
        "football.team_lineup_presets"
    };
    let remains: bool =
        sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=$1)"))
            .bind(id)
            .fetch_one(pool)
            .await
            .expect("entity presence");
    let history_remains: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS(SELECT 1 FROM {history_table} WHERE id=$1)"
    ))
    .bind(history_id)
    .fetch_one(pool)
    .await
    .expect("history presence");
    assert_eq!(remains, commit_reference);
    assert_eq!(history_remains, commit_reference);
    let external: Option<serde_json::Value> = sqlx::query_scalar("SELECT metadata FROM football.external_entity_ids WHERE provider_id=$1 AND entity_type=$2 AND entity_id=$3")
        .bind(provider.id).bind(entity_type).bind(id).fetch_optional(pool).await.expect("external binding retained");
    assert_eq!(
        external,
        if commit_reference {
            Some(json!({"keep":"original"}))
        } else {
            None
        }
    );
    assert_eq!(
        audit_count(pool, entity_type, id).await,
        if commit_reference { 0 } else { 1 }
    );
    let repeated = spawn_delete(store.clone(), entity_type, id)
        .await
        .expect("repeat delete task");
    assert!(repeated.is_err());
    assert_eq!(
        audit_count(pool, entity_type, id).await,
        if commit_reference { 0 } else { 1 }
    );
    if commit_reference {
        let check = store
            .check_entity_deletion(entity_type, id)
            .await
            .expect("protected preflight");
        assert!(check.must_archive);
        let relation = if entity_type == "player" {
            "dynamic_tags"
        } else {
            "team_lineup_presets"
        };
        assert!(check
            .references
            .iter()
            .any(|item| item.relation == relation && item.count == 1));
    }
}
