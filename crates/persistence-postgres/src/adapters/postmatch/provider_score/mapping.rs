use crate::{postmatch::non_negative_u64, PersistenceResult};
use football_domain::ProviderScoreSnapshotRecord;
use sqlx::Row;

pub(super) fn provider_score_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ProviderScoreSnapshotRecord> {
    Ok(ProviderScoreSnapshotRecord {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        scope_key: row.try_get("scope_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        horizon: row.try_get("horizon")?,
        sample_size: non_negative_u64(row.try_get("sample_size")?, "供应商样本")?,
        correct_count: non_negative_u64(row.try_get("correct_count")?, "正确证据数量")?,
        partial_count: non_negative_u64(row.try_get("partial_count")?, "部分正确证据数量")?,
        incorrect_count: non_negative_u64(row.try_get("incorrect_count")?, "错误证据数量")?,
        not_verifiable_count: non_negative_u64(
            row.try_get("not_verifiable_count")?,
            "不可验证证据数量",
        )?,
        accuracy_mean: row.try_get("accuracy_mean")?,
        timeliness_mean: row.try_get("timeliness_mean")?,
        reliability_mean: row.try_get("reliability_mean")?,
        weighted_score: row.try_get("weighted_score")?,
        decision_set_sha256: row.try_get("decision_set_sha256")?,
        calculation_version: row.try_get("calculation_version")?,
        generated_at: row.try_get("generated_at")?,
    })
}
