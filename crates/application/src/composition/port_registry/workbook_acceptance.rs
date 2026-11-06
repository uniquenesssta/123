//! Opt-in real application/XLSX/database acceptance in the existing application lib target.
//! No model execution, API calls, desktop configuration or user credentials are involved.
use super::{DatabaseOptions, PersistenceStore};
use crate::ApplicationService;
use football_domain::{SpreadsheetImportMode, SpreadsheetImportPreview};
use std::{path::PathBuf, sync::Arc};

fn require_clean_preview(preview: &SpreadsheetImportPreview) {
    let failures: Vec<_> = preview
        .rows
        .iter()
        .filter_map(|row| row.message.as_ref())
        .take(8)
        .collect();
    assert_eq!(
        preview.counts.error, 0,
        "real workbook errors: {failures:?}"
    );
    assert_eq!(
        preview.counts.conflict, 0,
        "real workbook conflicts: {failures:?}"
    );
    assert!(!preview.rows.is_empty());
}

#[tokio::test]
#[ignore = "requires the explicitly provisioned R17 isolated PostgreSQL acceptance instance"]
async fn real_acceptance_workbooks_commit_and_survive_reconnection() {
    let connection_url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("explicit QA database");
    let url = url::Url::parse(&connection_url).expect("PostgreSQL URL");
    assert_eq!(url.host_str(), Some("127.0.0.1"));
    assert_eq!(url.port(), Some(55437));
    assert_eq!(url.path(), "/football_r17_test_acceptance_20261107");
    let options = DatabaseOptions {
        connection_url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    };
    let store = PersistenceStore::connect(&options).await.unwrap();
    store
        .reset_to_pristine()
        .await
        .expect("clean approved QA database");
    store.close().await;

    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/全链路模拟");
    let application = Arc::new(ApplicationService::new());
    application.connect_database(options.clone()).await.unwrap();
    let players = application
        .preview_player_catalog_import(
            examples
                .join("01_球队球员模拟导入.xlsx")
                .to_string_lossy()
                .into_owned(),
            SpreadsheetImportMode::AddAndUpdate,
        )
        .await
        .unwrap();
    require_clean_preview(&players);
    let player_result = application
        .commit_player_catalog_import(players.batch_id)
        .await
        .unwrap();
    assert_eq!(player_result.inserted_count, players.rows.len() as u64);
    assert_eq!(player_result.error_count, 0);

    let matches = application
        .preview_match_lineup_import(
            examples
                .join("02_比赛阵容模拟导入.xlsx")
                .to_string_lossy()
                .into_owned(),
            SpreadsheetImportMode::AddAndUpdate,
        )
        .await
        .unwrap();
    require_clean_preview(&matches);
    let match_result = application
        .commit_match_lineup_import(matches.batch_id)
        .await
        .unwrap();
    assert_eq!(match_result.inserted_count, matches.rows.len() as u64);
    assert_eq!(match_result.error_count, 0);
    application.disconnect_database().await;
    drop(application);

    let reopened = Arc::new(ApplicationService::new());
    reopened.connect_database(options.clone()).await.unwrap();
    let player_batch = reopened
        .read_player_catalog_import_preview(players.batch_id)
        .await
        .unwrap();
    let match_batch = reopened
        .read_match_lineup_import_preview(matches.batch_id)
        .await
        .unwrap();
    assert_eq!(player_batch.source_sha256, players.source_sha256);
    assert_eq!(match_batch.source_sha256, matches.source_sha256);
    assert_eq!(player_batch.rows.len(), players.rows.len());
    assert_eq!(match_batch.rows.len(), matches.rows.len());
    let retry = reopened
        .commit_player_catalog_import(players.batch_id)
        .await
        .unwrap();
    assert_eq!(retry.inserted_count, player_result.inserted_count);
    assert!(reopened
        .commit_match_lineup_import(matches.batch_id)
        .await
        .is_err());
    reopened.disconnect_database().await;

    let readback = PersistenceStore::connect(&options).await.unwrap();
    let catalog = readback.spreadsheet_export_data().await.unwrap();
    assert_eq!(catalog.teams.len(), 4);
    assert_eq!(catalog.players.len(), 72);
    assert_eq!(catalog.abilities.len(), 864);
    let fixtures = readback.list_managed_matches(20).await.unwrap();
    assert_eq!(fixtures.len(), 3);
    let mut lineup_count = 0;
    let mut lineup_player_count = 0;
    for fixture in &fixtures {
        let lineups = readback.list_lineups(Some(fixture.id), 100).await.unwrap();
        lineup_count += lineups.len();
        for lineup in lineups {
            lineup_player_count += readback.read_lineup(lineup.id).await.unwrap().players.len();
        }
    }
    assert_eq!(lineup_count, 12);
    assert_eq!(lineup_player_count, 216);
    println!(
        "R17-06 real XLSX readback: {}",
        serde_json::json!({
            "player_batch": players.batch_id, "match_batch": matches.batch_id,
            "player_rows": player_result.inserted_count, "match_rows": match_result.inserted_count,
            "teams": catalog.teams.len(), "players": catalog.players.len(), "abilities": catalog.abilities.len(),
            "matches": fixtures.len(), "lineups": lineup_count, "lineup_players": lineup_player_count,
            "reconnected": true, "duplicate_player_replayed": false, "duplicate_match_rejected": true
        })
    );
    readback.close().await;
}
