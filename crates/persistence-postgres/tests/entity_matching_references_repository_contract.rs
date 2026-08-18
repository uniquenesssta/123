use chrono::NaiveDate;
use football_domain::{
    CoachDraft, CoachNameDraft, DataProviderDraft, EntityMatchRequest, EntityReferenceQuery,
    ExternalEntityIdDraft, PlayerDraft, PlayerNameDraft, PlayerStatus, PreferredFoot, TeamDraft,
    TeamNameDraft,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn entity_matching_and_references_contract_is_preserved() {
    let url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("connect");
    store.migrate().await.expect("migrate");

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
}
