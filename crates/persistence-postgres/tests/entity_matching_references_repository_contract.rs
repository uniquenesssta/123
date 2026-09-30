use chrono::NaiveDate;
use football_domain::{
    CoachDraft, CoachNameDraft, DataProviderDraft, EntityMatchRequest, EntityReferenceQuery,
    ExternalEntityIdDraft, PlayerDraft, PlayerNameDraft, PlayerStatus, PreferredFoot,
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetParsedWorkbook,
    SpreadsheetRawRow, SpreadsheetRowStatus, TeamDraft, TeamNameDraft, PLAYER_IMPORT_FORMAT,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{str::FromStr, sync::Arc};
use tokio::sync::Barrier;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn entity_matching_and_references_contract_is_preserved() {
    let (store, pool) = connect_test_database().await;

    let token = Uuid::new_v4().simple().to_string();
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-08 Team {token}"),
            country_code: Some("PT".into()),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create team");
    let team_alias = format!("Atlético Ref {token}");
    store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: team_alias.clone(),
            language_code: Some("es".into()),
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("team alias");

    let dob = NaiveDate::from_ymd_opt(1995, 2, 3).unwrap();
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-08 Player {token}"),
            date_of_birth: Some(dob),
            nationality_code: Some("BR".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create player");
    let player_alias = format!("João Ref {token}");
    store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: player_alias.clone(),
            language_code: Some("pt".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("player alias");

    let coach = store
        .create_coach(&CoachDraft {
            canonical_name: format!("R6-08 Coach {token}"),
            nationality_code: Some("PT".into()),
            status: "active".into(),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("create coach");
    let coach_alias = format!("José Ref {token}");
    store
        .add_coach_name(&CoachNameDraft {
            coach_id: coach.id,
            name: coach_alias.clone(),
            language_code: Some("pt".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("coach alias");

    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("  R6_08_{token}  "),
            name: format!("  R6-08 Provider {token}  "),
            provider_type: "  official  ".into(),
            base_url: Some("  https://example.test/r6-08  ".into()),
            metadata: json!({"contract":"r6-08"}),
        })
        .await
        .expect("provider");
    assert_eq!(provider.code, format!("r6_08_{token}"));
    assert_eq!(provider.name, format!("R6-08 Provider {token}"));
    assert_eq!(provider.provider_type, "official");
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://example.test/r6-08")
    );
    assert!(store
        .list_data_providers()
        .await
        .expect("providers")
        .iter()
        .any(|item| item.id == provider.id));

    let external_value = format!("EXT-{token}");
    let external = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: team.id,
            external_id: format!("  {external_value}  "),
            metadata: json!({"source":"contract"}),
        })
        .await
        .expect("external id");
    assert_eq!(external.entity_id, team.id);
    assert_eq!(external.external_id, external_value);
    assert_eq!(external.provider_name, provider.name);

    let team_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "team".into(),
            search: Some(format!("atletico ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("team reference search");
    let team_ref = team_refs
        .iter()
        .find(|item| item.id == team.id)
        .expect("team ref");
    assert!(team_ref.aliases.iter().any(|value| value == &team_alias));
    assert!(team_ref
        .external_ids
        .iter()
        .any(|value| value == &external_value));

    let player_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "player".into(),
            search: Some(format!("joao ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("player reference search");
    assert!(player_refs.iter().any(|item| item.id == player.id));

    let coach_refs = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "coach".into(),
            search: Some(format!("jose ref {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("coach reference search");
    assert!(coach_refs.iter().any(|item| item.id == coach.id));

    let stable = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: Some(team.id),
            provider_id: None,
            external_id: None,
            canonical_name: None,
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("stable id");
    assert_eq!(stable.status, "exact");
    assert_eq!(stable.matched_id, Some(team.id));
    assert_eq!(stable.candidates[0].reason, "稳定实体 ID 精确匹配");

    let external_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: Some(provider.id),
            external_id: Some(external_value.clone()),
            canonical_name: Some("wrong fallback name".into()),
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("external id match");
    assert_eq!(external_match.matched_id, Some(team.id));
    assert_eq!(
        external_match.candidates[0].reason,
        "受信数据源外部 ID 精确匹配"
    );

    let team_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(team_alias),
            country_code: Some("pt".into()),
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("team name match");
    assert_eq!(team_name_match.matched_id, Some(team.id));
    assert_eq!(team_name_match.candidates[0].reason, "球队别名");
    assert!((team_name_match.candidates[0].score - 0.95).abs() < f64::EPSILON);

    let player_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "player".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(player_alias),
            country_code: None,
            nationality_code: None,
            date_of_birth: Some(dob),
        })
        .await
        .expect("player name match");
    assert_eq!(player_name_match.matched_id, Some(player.id));
    assert_eq!(player_name_match.candidates[0].reason, "球员别名与出生日期");
    assert!((player_name_match.candidates[0].score - 1.0).abs() < f64::EPSILON);

    let coach_name_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "coach".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(coach_alias),
            country_code: None,
            nationality_code: Some("pt".into()),
            date_of_birth: None,
        })
        .await
        .expect("coach name match");
    assert_eq!(coach_name_match.matched_id, Some(coach.id));
    assert_eq!(coach_name_match.candidates[0].reason, "教练别名与国籍");

    let no_match = store
        .resolve_entity_reference(&EntityMatchRequest {
            entity_type: "team".into(),
            entity_id: None,
            provider_id: None,
            external_id: None,
            canonical_name: Some(format!("missing {token}")),
            country_code: None,
            nationality_code: None,
            date_of_birth: None,
        })
        .await
        .expect("no match");
    assert_eq!(no_match.status, "no_match");
    assert!(no_match.matched_id.is_none());
    assert!(no_match.candidates.is_empty());

    let bad_type = store
        .list_entity_references(&EntityReferenceQuery {
            entity_type: "match".into(),
            search: None,
            active_only: true,
            limit: 20,
        })
        .await
        .expect_err("unsupported reference type");
    assert!(
        matches!(bad_type, PersistenceError::InvalidState(m) if m == "不支持的实体类型：match")
    );

    let bad_provider = store
        .create_data_provider(&DataProviderDraft {
            code: "  ".into(),
            name: "name".into(),
            provider_type: "official".into(),
            base_url: None,
            metadata: json!({}),
        })
        .await
        .expect_err("blank provider code");
    assert!(
        matches!(bad_provider, PersistenceError::InvalidState(m) if m == "数据源代码、名称和类型不能为空")
    );

    let bad_external_type = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "formation".into(),
            entity_id: team.id,
            external_id: "x".into(),
            metadata: json!({}),
        })
        .await
        .expect_err("invalid external id entity type");
    assert!(
        matches!(bad_external_type, PersistenceError::InvalidState(m) if m == "外部 ID 实体类型无效")
    );

    let blank_external = store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: team.id,
            external_id: "   ".into(),
            metadata: json!({}),
        })
        .await
        .expect_err("blank external id");
    assert!(matches!(blank_external, PersistenceError::InvalidState(m) if m == "外部 ID 不能为空"));

    store.close().await;
    pool.close().await;
}

const ID_CONFLICT: &str = "该外部 ID 已绑定到另一条数据库记录，禁止自动改绑";

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
fn external_id_contract_rejects_non_test_database_before_connecting() {
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

async fn connect_test_database() -> (PostgresStore, PgPool) {
    let url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
    let options = test_database_options(&url).expect("外部 ID 测试库前检失败");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .expect("connect test pool");
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("actual database");
    assert!(name.to_lowercase().contains("test"));
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("connect");
    store.migrate().await.expect("migrate");
    (store, pool)
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn external_id_identity_and_import_atomicity_are_preserved() {
    let (store, pool) = connect_test_database().await;
    let token = Uuid::new_v4().simple().to_string();
    let test_store = store.clone();
    let test_pool = pool.clone();
    let test_token = token.clone();
    let result = tokio::spawn(async move {
        verify_external_id_integrity(&test_store, &test_pool, &test_token).await;
    })
    .await;
    // Clean only this invocation's fixtures, including when a behavior assertion panics.
    let cleanup = cleanup_external_id_fixture(&pool, &token).await;
    store.close().await;
    pool.close().await;
    if let Err(error) = result {
        if error.is_panic() {
            std::panic::resume_unwind(error.into_panic());
        }
        panic!("external id test task cancelled: {error}");
    }
    cleanup.expect("cleanup external id fixture");
}

async fn cleanup_external_id_fixture(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM catalog.import_batches WHERE source_file_name LIKE $1")
        .bind(format!("R7-02-{token}-%"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM football.external_entity_ids WHERE provider_id IN (SELECT id FROM catalog.data_providers WHERE metadata->>'fixture' = $1)")
        .bind(token).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM football.players WHERE canonical_name=$1")
        .bind(format!("R7-02 Rolled Back {token}"))
        .execute(&mut *tx)
        .await?;
    for table in [
        "football.players",
        "football.teams",
        "catalog.data_providers",
    ] {
        sqlx::query(&format!(
            "DELETE FROM {table} WHERE metadata->>'fixture' = $1"
        ))
        .bind(token)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

fn assert_id_conflict<T: std::fmt::Debug>(result: Result<T, PersistenceError>) {
    assert!(
        matches!(result, Err(PersistenceError::InvalidState(ref message)) if message == ID_CONFLICT),
        "unexpected result: {result:?}"
    );
}

fn id_draft(
    provider_id: Uuid,
    entity_id: Uuid,
    external_id: &str,
    metadata: Value,
) -> ExternalEntityIdDraft {
    ExternalEntityIdDraft {
        provider_id,
        entity_type: "player".into(),
        entity_id,
        external_id: external_id.into(),
        metadata,
    }
}

async fn binding(pool: &PgPool, provider_id: Uuid, external_id: &str) -> (Uuid, Value) {
    sqlx::query_as("SELECT entity_id, metadata FROM football.external_entity_ids WHERE provider_id=$1 AND entity_type='player' AND external_id=$2")
        .bind(provider_id).bind(external_id).fetch_one(pool).await.expect("read binding")
}

fn workbook(token: &str, name: &str, rows: Vec<SpreadsheetRawRow>) -> SpreadsheetParsedWorkbook {
    SpreadsheetParsedWorkbook {
        format_version: PLAYER_IMPORT_FORMAT.into(),
        source_file_name: format!("R7-02-{token}-{name}.xlsx"),
        source_sha256: format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple()),
        rows,
    }
}

fn external_row(provider_code: &str, external_id: &str, player_id: Uuid) -> SpreadsheetRawRow {
    SpreadsheetRawRow {
        sheet_name: "ExternalIDs".into(),
        row_number: 2,
        entity_type: SpreadsheetEntityType::ExternalEntityId,
        action: SpreadsheetAction::Upsert,
        values: json!({"provider_code":provider_code,"entity_type":"player","external_id":external_id,"player_id":player_id,"notes":"import metadata"}),
    }
}

async fn verify_external_id_integrity(store: &PostgresStore, pool: &PgPool, token: &str) {
    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r7_02_{token}"),
            name: format!("R7-02 Provider {token}"),
            provider_type: "official".into(),
            base_url: None,
            metadata: json!({"fixture":token}),
        })
        .await
        .expect("provider");
    let mut player_ids = Vec::new();
    for index in 0..2 {
        let player = store
            .create_player(&PlayerDraft {
                canonical_name: format!("R7-02 Player {token} {index}"),
                date_of_birth: None,
                nationality_code: None,
                preferred_foot: PreferredFoot::Unknown,
                height_cm: None,
                status: PlayerStatus::Active,
                metadata: json!({"fixture":token}),
            })
            .await
            .expect("player");
        player_ids.push(player.id);
    }
    let [first_id, second_id] = [player_ids[0], player_ids[1]];
    let original = store
        .add_external_entity_id(&id_draft(
            provider.id,
            first_id,
            "  stable  ",
            json!({"original":true,"replace":1}),
        ))
        .await
        .expect("first binding");
    assert_eq!(original.external_id, "stable");
    let retry = store
        .add_external_entity_id(&id_draft(
            provider.id,
            first_id,
            "stable",
            json!({"retry":true,"replace":2}),
        ))
        .await
        .expect("same entity retry");
    assert_eq!(retry.id, original.id);
    assert_eq!(
        retry.metadata,
        json!({"original":true,"retry":true,"replace":2})
    );
    assert_id_conflict(
        store
            .add_external_entity_id(&id_draft(
                provider.id,
                second_id,
                "stable",
                json!({"replace":99,"intruder":true}),
            ))
            .await,
    );
    assert_eq!(
        binding(pool, provider.id, "stable").await,
        (first_id, retry.metadata)
    );

    // Concurrent first claims: exactly one target and its metadata survive.
    let barrier = Arc::new(Barrier::new(2));
    let claim = |entity_id, owner: &'static str| {
        let barrier = Arc::clone(&barrier);
        async move {
            barrier.wait().await;
            store
                .add_external_entity_id(&id_draft(
                    provider.id,
                    entity_id,
                    "race",
                    json!({"owner":owner}),
                ))
                .await
        }
    };
    let (left, right) = tokio::join!(claim(first_id, "left"), claim(second_id, "right"));
    let winner = match (left, right) {
        (Ok(winner), loser) | (loser, Ok(winner)) => {
            assert_id_conflict(loser);
            winner
        }
        (left, right) => panic!("expected one successful claim: {left:?}, {right:?}"),
    };
    assert_eq!(
        binding(pool, provider.id, "race").await,
        (winner.entity_id, winner.metadata)
    );

    // Same target concurrent metadata merges must not lose either patch.
    let left_patch = id_draft(provider.id, first_id, "merge", json!({"left":true}));
    let right_patch = id_draft(provider.id, first_id, "merge", json!({"right":true}));
    let (left, right) = tokio::join!(
        store.add_external_entity_id(&left_patch),
        store.add_external_entity_id(&right_patch),
    );
    assert_eq!(left.expect("left merge").id, right.expect("right merge").id);
    assert_eq!(
        binding(pool, provider.id, "merge").await,
        (first_id, json!({"left":true,"right":true}))
    );

    // The provider/type namespaces remain independent.
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R7-02 Team {token}"),
            country_code: None,
            metadata: json!({"fixture":token}),
        })
        .await
        .expect("team");
    let mut team_draft = id_draft(provider.id, team.id, "stable", json!({"team":true}));
    team_draft.entity_type = "team".into();
    assert_eq!(
        store
            .add_external_entity_id(&team_draft)
            .await
            .expect("type namespace")
            .entity_id,
        team.id
    );
    let other_provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r7_02_other_{token}"),
            name: format!("R7-02 Other {token}"),
            provider_type: "official".into(),
            base_url: None,
            metadata: json!({"fixture":token}),
        })
        .await
        .expect("other provider");
    assert_eq!(
        store
            .add_external_entity_id(&id_draft(other_provider.id, second_id, "stable", json!({})))
            .await
            .expect("provider namespace")
            .entity_id,
        second_id
    );

    // Preview succeeds, then the direct entry claims the same target before commit.
    let same = workbook(
        token,
        "same-target",
        vec![external_row(&provider.code, "same-import", first_id)],
    );
    let preview = store
        .preview_spreadsheet_import(&same, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("same target preview");
    assert_eq!(preview.counts.ready_add, 1);
    let direct = store
        .add_external_entity_id(&id_draft(
            provider.id,
            first_id,
            "same-import",
            json!({"keep":true}),
        ))
        .await
        .expect("direct claim");
    let committed = store
        .commit_spreadsheet_import(preview.batch_id)
        .await
        .expect("same target import");
    assert_eq!(committed.inserted_count, 1);
    let (target, metadata) = binding(pool, provider.id, "same-import").await;
    assert_eq!(target, first_id);
    assert_eq!(metadata["keep"], true);
    assert_eq!(metadata["notes"], "import metadata");
    assert_eq!(
        store
            .add_external_entity_id(&id_draft(provider.id, first_id, "same-import", json!({})))
            .await
            .expect("read retry")
            .id,
        direct.id
    );
    let repeated = store
        .commit_spreadsheet_import(preview.batch_id)
        .await
        .expect("batch retry");
    assert_eq!(repeated.inserted_count, committed.inserted_count);
    let conflict_preview = store
        .preview_spreadsheet_import(
            &workbook(
                token,
                "conflicting-target",
                vec![external_row(&provider.code, "same-import", second_id)],
            ),
            SpreadsheetImportMode::AddAndUpdate,
        )
        .await
        .expect("conflict preview");
    assert_eq!(conflict_preview.counts.error, 1);
    assert!(store
        .commit_spreadsheet_import(conflict_preview.batch_id)
        .await
        .is_err());

    // Preview a new player and two ID rows. A competing direct bind after preview
    // must roll back the new player, the earlier ID, row states, batch counts and audit.
    let new_name = format!("R7-02 Rolled Back {token}");
    let new_player = SpreadsheetRawRow {
        sheet_name: "Players".into(),
        row_number: 2,
        entity_type: SpreadsheetEntityType::Player,
        action: SpreadsheetAction::Add,
        values: json!({"player_key":"new-player","official_name":new_name}),
    };
    let first_row = external_row(&provider.code, "earlier-import", first_id);
    let mut late_row = external_row(&provider.code, "late-import", first_id);
    late_row.row_number = 3;
    late_row.values.as_object_mut().unwrap().remove("player_id");
    late_row.values["player_key"] = json!("new-player");
    let late = workbook(
        token,
        "late-conflict",
        vec![new_player, first_row, late_row],
    );
    let preview = store
        .preview_spreadsheet_import(&late, SpreadsheetImportMode::AddAndUpdate)
        .await
        .expect("late conflict preview");
    assert_eq!(preview.counts.ready_add, 3);
    let claimed = store
        .add_external_entity_id(&id_draft(
            provider.id,
            second_id,
            "late-import",
            json!({"keep":"original"}),
        ))
        .await
        .expect("competing claim");
    assert_id_conflict(store.commit_spreadsheet_import(preview.batch_id).await);
    assert_eq!(
        binding(pool, provider.id, "late-import").await,
        (second_id, claimed.metadata)
    );
    let remaining = store
        .read_spreadsheet_import_preview(preview.batch_id)
        .await
        .expect("rolled back preview");
    assert!(remaining
        .rows
        .iter()
        .all(|row| row.status == SpreadsheetRowStatus::ReadyAdd));
    let state: (String, i64, i64, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as("SELECT status,inserted_count,updated_count,finished_at FROM catalog.import_batches WHERE id=$1").bind(preview.batch_id).fetch_one(pool).await.expect("batch state");
    assert_eq!(state, ("pending".into(), 0, 0, None));
    let new_players: i64 =
        sqlx::query_scalar("SELECT count(*) FROM football.players WHERE canonical_name=$1")
            .bind(&new_name)
            .fetch_one(pool)
            .await
            .expect("rolled back player count");
    assert_eq!(new_players, 0);
    let earlier_ids: i64 = sqlx::query_scalar("SELECT count(*) FROM football.external_entity_ids WHERE provider_id=$1 AND external_id='earlier-import'").bind(provider.id).fetch_one(pool).await.expect("rolled back id count");
    assert_eq!(earlier_ids, 0);
    let success_audits: i64 = sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='spreadsheet_import_committed' AND entity_id=$1").bind(preview.batch_id.to_string()).fetch_one(pool).await.expect("success audit count");
    assert_eq!(success_audits, 0);
}
