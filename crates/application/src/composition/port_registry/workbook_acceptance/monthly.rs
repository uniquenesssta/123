//! Exercise real monthly exports and imports through the existing application facade.
use super::{fixture::enabled_copy, require_clean_preview, PersistenceStore};
use crate::ApplicationService;
use football_domain::{SpreadsheetEntityType, SpreadsheetImportMode};
use std::path::Path;
use uuid::Uuid;

pub(super) struct Receipt {
    pub team_batch: Uuid,
    pub player_batch: Uuid,
    team_id: Uuid,
    short_name: String,
    player_id: Uuid,
    height: i16,
}

pub(super) async fn round_trip(application: &ApplicationService, output: &Path) -> Receipt {
    std::fs::create_dir_all(output).unwrap();
    let teams = output
        .join("team-monthly.xlsx")
        .to_string_lossy()
        .into_owned();
    let players = output
        .join("player-monthly.xlsx")
        .to_string_lossy()
        .into_owned();
    let exported_teams = application
        .export_team_monthly_data(teams.clone())
        .await
        .unwrap();
    let exported_players = application
        .export_player_catalog_data(players.clone())
        .await
        .unwrap();
    assert_eq!(exported_teams.team_count, 4);
    assert_eq!(exported_players.player_count, 72);
    let team_preview = application
        .preview_team_monthly_import(teams.clone(), SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    let player_preview = application
        .preview_player_catalog_import(players.clone(), SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    require_clean_preview(&team_preview);
    require_clean_preview(&player_preview);
    let team_result = application
        .commit_team_monthly_import(team_preview.batch_id)
        .await
        .unwrap();
    let player_result = application
        .commit_player_catalog_import(player_preview.batch_id)
        .await
        .unwrap();
    assert_eq!(team_result.error_count, 0);
    assert_eq!(player_result.error_count, 0);
    assert_eq!(
        team_result.inserted_count, 0,
        "an exported existing team must not become another team"
    );
    assert_eq!(
        player_result.inserted_count, 0,
        "an exported existing player or child row must not duplicate"
    );
    assert_eq!(team_result.updated_count, 0, "exports default to skip");
    assert_eq!(player_result.updated_count, 0, "exports default to skip");
    let team = team_preview
        .rows
        .iter()
        .find(|row| row.entity_type == SpreadsheetEntityType::Team)
        .unwrap();
    let player = player_preview
        .rows
        .iter()
        .find(|row| row.entity_type == SpreadsheetEntityType::Player)
        .unwrap();
    let team_id: Uuid = team.payload["team_id"].as_str().unwrap().parse().unwrap();
    let player_id: Uuid = player.payload["player_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let short_name = format!("QA-{}", &team_preview.batch_id.to_string()[..8]);
    let old_height = player.payload["height_cm"]
        .as_i64()
        .or_else(|| {
            player.payload["height_cm"]
                .as_str()
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(180);
    let height = if old_height >= 230 {
        229
    } else {
        old_height as i16 + 1
    };
    let team_updates = enabled_copy(
        &teams,
        &team_preview,
        SpreadsheetEntityType::Team,
        "short_name",
        &short_name,
    );
    let player_updates = enabled_copy(
        &players,
        &player_preview,
        SpreadsheetEntityType::Player,
        "height_cm",
        &height.to_string(),
    );
    let team_preview = application
        .preview_team_monthly_import(team_updates, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    let player_preview = application
        .preview_player_catalog_import(player_updates, SpreadsheetImportMode::AddAndUpdate)
        .await
        .unwrap();
    require_clean_preview(&team_preview);
    require_clean_preview(&player_preview);
    let team_result = application
        .commit_team_monthly_import(team_preview.batch_id)
        .await
        .unwrap();
    let player_result = application
        .commit_player_catalog_import(player_preview.batch_id)
        .await
        .unwrap();
    assert_eq!(team_result.inserted_count, 0);
    assert_eq!(player_result.inserted_count, 0);
    assert_eq!(team_result.error_count, 0);
    assert_eq!(player_result.error_count, 0);
    assert_eq!(team_result.updated_count, team_preview.counts.ready_update);
    assert!(team_result.updated_count > 0);
    assert_eq!(
        player_result.updated_count,
        player_preview.counts.ready_update
    );
    assert!(player_result.updated_count > 0);
    let team_readback = application
        .read_team_monthly_import_preview(team_preview.batch_id)
        .await
        .unwrap();
    let player_readback = application
        .read_player_catalog_import_preview(player_preview.batch_id)
        .await
        .unwrap();
    assert_eq!(team_readback.source_sha256, team_preview.source_sha256);
    assert_eq!(player_readback.source_sha256, player_preview.source_sha256);
    assert_eq!(team_readback.rows.len(), team_preview.rows.len());
    assert_eq!(player_readback.rows.len(), player_preview.rows.len());
    println!(
        "R17-06 real monthly XLSX: {}",
        serde_json::json!({
            "team_batch":team_preview.batch_id,"player_batch":player_preview.batch_id,
            "team_rows_updated":team_result.updated_count,"player_rows_updated":player_result.updated_count,
            "team_export_count":exported_teams.team_count,"player_export_count":exported_players.player_count,
            "inserted_count":team_result.inserted_count + player_result.inserted_count,
            "export_directory":output
        })
    );
    Receipt {
        team_batch: team_preview.batch_id,
        player_batch: player_preview.batch_id,
        team_id,
        short_name,
        player_id,
        height,
    }
}

pub(super) async fn verify_changes(store: &PersistenceStore, receipt: &Receipt) {
    let teams = store.team_monthly_workbook_data().await.unwrap();
    assert_eq!(
        teams
            .teams
            .iter()
            .find(|team| team.team_id == receipt.team_id)
            .unwrap()
            .short_name
            .as_deref(),
        Some(receipt.short_name.as_str())
    );
    let players = store.spreadsheet_export_data().await.unwrap();
    assert_eq!(
        players
            .players
            .iter()
            .find(|player| player.player_id == receipt.player_id)
            .unwrap()
            .height_cm,
        Some(receipt.height)
    );
    println!("R17-06 monthly changed fields survived reconnection: team {} short_name={}, player {} height_cm={}", receipt.team_id, receipt.short_name, receipt.player_id, receipt.height);
}
