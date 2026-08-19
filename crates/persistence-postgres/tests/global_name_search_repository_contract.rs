use football_domain::{
    CoachDraft, CoachListQuery, CoachNameDraft, EntityReferenceQuery, PlayerDraft, PlayerListQuery,
    PlayerNameDraft, PlayerStatus, PreferredFoot, TeamDraft, TeamListQuery, TeamNameDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn global_name_search_contract_is_preserved() {
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
            canonical_name: format!("São-Paulo Search {token}"),
            country_code: Some("BR".into()),
            metadata: json!({"contract":"r6-10"}),
        })
        .await
        .expect("create primary team");
    store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: format!("圣保罗·验收 {token}"),
            language_code: Some("zh-CN".into()),
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("add team alias");
    let team_two = store
        .create_team(&TeamDraft {
            canonical_name: format!("Sao Paulo Reserve Search {token}"),
            country_code: Some("BR".into()),
            metadata: json!({"contract":"r6-10"}),
        })
        .await
        .expect("create pagination team");

    let options = store
        .list_team_options(Some(&format!("sao paulo {token}")), 20)
        .await
        .expect("team selector search");
    assert!(options.iter().any(|item| item.id == team.id));
    let chinese = store
        .list_teams(&TeamListQuery {
            search: Some(format!("圣保罗验收 {token}")),
            country_code: None,
            team_type: None,
            active_only: true,
            limit: 20,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("team compact alias search");
    assert!(chinese.items.iter().any(|item| item.id == team.id));
    let mixed = store
        .list_teams(&TeamListQuery {
            search: Some(format!("sao 验收 {token}")),
            country_code: None,
            team_type: None,
            active_only: true,
            limit: 20,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("team mixed search");
    assert!(mixed.items.iter().any(|item| item.id == team.id));

    let first = store
        .list_teams(&TeamListQuery {
            search: Some(token.clone()),
            country_code: None,
            team_type: None,
            active_only: true,
            limit: 1,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("first page");
    assert_eq!(first.items.len(), 1);
    assert!(first.has_more);
    let second = store
        .list_teams(&TeamListQuery {
            search: Some(token.clone()),
            country_code: None,
            team_type: None,
            active_only: true,
            limit: 1,
            cursor_name: first.next_cursor_name.clone(),
            cursor_id: first.next_cursor_id,
        })
        .await
        .expect("second page");
    assert_eq!(second.items.len(), 1);
    let ids = first
        .items
        .iter()
        .chain(second.items.iter())
        .map(|item| item.id)
        .collect::<HashSet<_>>();
    assert_eq!(ids, HashSet::from([team.id, team_two.id]));

    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("Marlón Sousa {token}"),
            date_of_birth: None,
            nationality_code: Some("BR".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-10"}),
        })
        .await
        .expect("create player");
    store
        .add_player_name(&PlayerNameDraft {
            player_id: player.id,
            name: format!("马龙·索萨 {token}"),
            language_code: Some("zh-CN".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("player alias");
    let players = store
        .list_players(&PlayerListQuery {
            search: Some(format!("marlon 索萨 {token}")),
            team_id: None,
            position_code: None,
            availability_status: None,
            player_status: Some(PlayerStatus::Active),
            limit: 20,
            cursor_name: None,
            cursor_id: None,
        })
        .await
        .expect("player mixed search");
    assert!(players.items.iter().any(|item| item.id == player.id));

    let coach = store
        .create_coach(&CoachDraft {
            canonical_name: format!("José Search {token}"),
            nationality_code: Some("PT".into()),
            status: "active".into(),
            metadata: json!({"contract":"r6-10"}),
        })
        .await
        .expect("create coach");
    store
        .add_coach_name(&CoachNameDraft {
            coach_id: coach.id,
            name: format!("何塞·教练 {token}"),
            language_code: Some("zh-CN".into()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("coach alias");
    let coaches = store
        .list_coaches(&CoachListQuery {
            search: Some(format!("jose 教练 {token}")),
            active_only: true,
            limit: 20,
        })
        .await
        .expect("coach mixed search");
    assert!(coaches.iter().any(|item| item.id == coach.id));

    for (entity_type, search, id) in [
        ("team", format!("sao 验收 {token}"), team.id),
        ("player", format!("marlon 索萨 {token}"), player.id),
        ("coach", format!("jose 教练 {token}"), coach.id),
    ] {
        let records = store
            .list_entity_references(&EntityReferenceQuery {
                entity_type: entity_type.into(),
                search: Some(search),
                active_only: true,
                limit: 20,
            })
            .await
            .expect("entity reference search");
        assert!(records.iter().any(|item| item.id == id));
    }
    store.close().await;
}
