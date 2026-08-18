use chrono::{Duration, Utc};
use football_domain::{
    CoachDraft, CoachListQuery, CoachNameDraft, FormationDistributionQuery,
    FormationUsageDistributionDraft, FormationUsageEntryDraft, FormationUsageListQuery,
    TeamCoachPeriodDraft, TeamDraft,
};
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn coach_and_formation_usage_contract_is_preserved() {
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
    let coach = store
        .create_coach(&CoachDraft {
            canonical_name: format!("  R6-07   Coach {token}  "),
            nationality_code: Some(" ZZ ".into()),
            status: " active ".into(),
            metadata: json!({"contract":"r6-07"}),
        })
        .await
        .expect("create coach");
    assert_eq!(coach.canonical_name, format!("R6-07   Coach {token}"));
    assert_eq!(coach.normalized_name, format!("r6-07 coach {token}"));
    assert_eq!(coach.nationality_code.as_deref(), Some("ZZ"));
    let err = store
        .create_coach(&CoachDraft {
            canonical_name: "   ".into(),
            nationality_code: None,
            status: "active".into(),
            metadata: json!({}),
        })
        .await
        .expect_err("empty coach must fail");
    assert!(matches!(err, PersistenceError::InvalidState(m) if m == "教练姓名不能为空"));
    let alias = store
        .add_coach_name(&CoachNameDraft {
            coach_id: coach.id,
            name: format!("  Alias   {token}  "),
            language_code: Some(" zh-CN ".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("alias");
    assert_eq!(alias.name, format!("Alias   {token}"));
    assert_eq!(alias.normalized_name, format!("alias {token}"));
    assert_eq!(alias.language_code.as_deref(), Some("zh-CN"));
    let listed = store
        .list_coaches(&CoachListQuery {
            search: Some(format!("Alias {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("coach search");
    assert!(listed.iter().any(|item| item.id == coach.id));
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-07 Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("create team");
    let today = Utc::now().date_naive();
    let period = store
        .add_team_coach_period(&TeamCoachPeriodDraft {
            team_id: team.id,
            coach_id: coach.id,
            role: " head_coach ".into(),
            valid_from: today - Duration::days(30),
            valid_to: None,
            is_interim: false,
            confidence: 0.9,
            source_document_id: None,
            end_previous: true,
            metadata: json!({}),
        })
        .await
        .expect("period");
    assert_eq!(period.role, "head_coach");
    let bad_period = store
        .add_team_coach_period(&TeamCoachPeriodDraft {
            team_id: team.id,
            coach_id: coach.id,
            role: "head_coach".into(),
            valid_from: today,
            valid_to: Some(today - Duration::days(1)),
            is_interim: false,
            confidence: 0.9,
            source_document_id: None,
            end_previous: false,
            metadata: json!({}),
        })
        .await
        .expect_err("bad dates");
    assert!(
        matches!(bad_period, PersistenceError::InvalidState(m) if m == "教练任期结束日期早于开始日期")
    );
    let detail = store.read_coach(coach.id).await.expect("coach detail");
    assert!(detail.names.iter().any(|item| item.id == alias.id));
    assert!(detail.team_periods.iter().any(|item| item.id == period.id));

    let formations = store.list_formations(true).await.expect("formations");
    let formation = formations
        .iter()
        .find(|item| item.code != "UNKNOWN")
        .expect("active formation");
    let saved = store
        .save_formation_usage_distribution(&FormationUsageDistributionDraft {
            scope_type: "team_coach".into(),
            team_id: Some(team.id),
            coach_id: Some(coach.id),
            competition_id: None,
            window_preset: "custom".into(),
            window_start: Some(today - Duration::days(14)),
            window_end: Some(today),
            observed_matches: 3,
            confidence: 0.8,
            alpha: 3.0,
            source_document_id: None,
            metadata: json!({"contract":"r6-07"}),
            entries: vec![FormationUsageEntryDraft {
                formation_id: formation.id,
                usage_count: 2,
            }],
        })
        .await
        .expect("save formation usage");
    assert_eq!(saved.entries.iter().map(|e| e.usage_count).sum::<i32>(), 3);
    let smooth_sum: f64 = saved.entries.iter().map(|e| e.smoothed_probability).sum();
    assert!((smooth_sum - 1.0).abs() < 1e-9);
    let history = store
        .list_formation_usage_distributions(&FormationUsageListQuery {
            team_id: Some(team.id),
            coach_id: Some(coach.id),
            competition_id: None,
            limit: 20,
        })
        .await
        .expect("formation history");
    assert!(history
        .iter()
        .any(|item| item.observed_at == saved.observed_at));
    let resolved = store
        .resolve_formation_distribution(&FormationDistributionQuery {
            match_id: None,
            team_id: team.id,
            coach_id: None,
            competition_id: None,
            as_of: Some(Utc::now()),
        })
        .await
        .expect("resolve");
    assert_eq!(resolved.source_level, "team_coach");
    assert_eq!(resolved.coach_id, Some(coach.id));
    let bad_scope = store
        .save_formation_usage_distribution(&FormationUsageDistributionDraft {
            scope_type: "team".into(),
            team_id: Some(team.id),
            coach_id: Some(coach.id),
            competition_id: None,
            window_preset: "custom".into(),
            window_start: Some(today),
            window_end: Some(today),
            observed_matches: 1,
            confidence: 0.8,
            alpha: 3.0,
            source_document_id: None,
            metadata: json!({}),
            entries: vec![FormationUsageEntryDraft {
                formation_id: formation.id,
                usage_count: 1,
            }],
        })
        .await
        .expect_err("bad scope");
    assert!(
        matches!(bad_scope, PersistenceError::InvalidState(m) if m == "阵型概率作用域与球队、教练、赛事字段不匹配")
    );
    store.close().await;
}
