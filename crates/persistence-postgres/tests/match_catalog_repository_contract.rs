use chrono::{Duration, Utc};
use football_domain::{MatchDraft, MatchStatus, TeamDraft};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_catalog_contract_is_preserved() {
    let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
        panic!("运行 R7-01 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库")
    });
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("连接 R7-01 PostgreSQL 测试数据库");
    store.migrate().await.expect("执行数据库迁移");

    let token = Uuid::new_v4().simple().to_string();
    let home = store
        .create_team(&TeamDraft {
            canonical_name: format!("R7 Home {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r7-01"}),
        })
        .await
        .expect("创建主队");
    let away = store
        .create_team(&TeamDraft {
            canonical_name: format!("R7 Away {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r7-01"}),
        })
        .await
        .expect("创建客队");

    let kickoff = Utc::now() + Duration::days(2);
    let created = store
        .create_match(&MatchDraft {
            external_key: String::new(),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff_time: kickoff,
            status: MatchStatus::Scheduled,
            venue: Some("  R7 Test Ground  ".to_string()),
            metadata: json!({"contract": "r7-01", "phase": "create"}),
        })
        .await
        .expect("创建比赛");
    assert!(created.external_key.starts_with("MATCH-"));
    assert_eq!(created.home_team_id, home.id);
    assert_eq!(created.away_team_id, away.id);
    assert_eq!(created.venue.as_deref(), Some("R7 Test Ground"));
    assert!(matches!(created.status, MatchStatus::Scheduled));

    let read = store.read_match(created.id).await.expect("读取单场比赛");
    assert_eq!(read.id, created.id);
    assert_eq!(read.external_key, created.external_key);

    let upcoming = store
        .list_upcoming_matches(250)
        .await
        .expect("读取 upcoming matches");
    assert!(upcoming.iter().any(|item| item.id == created.id));
    let managed = store
        .list_managed_matches(500)
        .await
        .expect("读取 managed matches");
    assert!(managed.iter().any(|item| item.id == created.id));

    let updated = store
        .create_match(&MatchDraft {
            external_key: created.external_key.clone(),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff_time: kickoff + Duration::hours(1),
            status: MatchStatus::Postponed,
            venue: Some(" Updated Ground ".to_string()),
            metadata: json!({"contract": "r7-01", "phase": "upsert"}),
        })
        .await
        .expect("按 external_key 更新比赛");
    assert_eq!(updated.id, created.id);
    assert!(matches!(updated.status, MatchStatus::Postponed));
    assert_eq!(updated.venue.as_deref(), Some("Updated Ground"));

    let bad_sides = store
        .create_match(&MatchDraft {
            external_key: format!("R7-BAD-{token}"),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: home.id,
            kickoff_time: kickoff,
            status: MatchStatus::Scheduled,
            venue: None,
            metadata: json!({}),
        })
        .await;
    assert!(bad_sides.is_err(), "同队作为主客队必须被拒绝");

    store
        .delete_match(created.id)
        .await
        .expect("删除未受保护比赛");
    assert!(store.read_match(created.id).await.is_err());
    store.close().await;
}
