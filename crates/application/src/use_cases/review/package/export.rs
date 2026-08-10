use super::{shared::snapshot_from_lineups, workbook};
use crate::{
    ports::review::{MatchReviewPackageSourcePort, MatchReviewPackageStatePort},
    ApplicationResult,
};
use chrono::Utc;
use football_domain::MatchReviewPackageSummary;
use std::collections::HashSet;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    output_path: String,
    match_id: Uuid,
) -> ApplicationResult<MatchReviewPackageSummary>
where
    P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
{
    let path = workbook::validate_path(&output_path, true)?;
    let package_id = Uuid::new_v4();
    let data = port
        .build_export_data(match_id, package_id, Utc::now())
        .await?;
    let lineup_count = data.pre_match_lineups.len() as u64;
    let player_count = data
        .home_team
        .squad
        .iter()
        .map(|item| item.player_id)
        .chain(data.away_team.squad.iter().map(|item| item.player_id))
        .collect::<HashSet<_>>()
        .len() as u64;
    let pre_match_snapshot = snapshot_from_lineups(
        &data.pre_match_lineups,
        data.match_record.home_team_id,
        data.match_record.away_team_id,
        false,
        None,
    );
    let export_database_snapshot = snapshot_from_lineups(
        &data.pre_match_lineups,
        data.match_record.home_team_id,
        data.match_record.away_team_id,
        true,
        data.existing_result.as_ref(),
    );
    let match_key = data.match_record.external_key.clone();
    workbook::write_package(path.clone(), data).await?;
    let summary = MatchReviewPackageSummary {
        output_path: path.to_string_lossy().to_string(),
        package_id,
        match_id,
        match_key,
        lineup_count,
        player_count,
        content_sha256: workbook::sha256_file(&path)?,
        pre_match_snapshot,
        export_database_snapshot,
    };
    port.register_export(&summary).await?;
    Ok(summary)
}
