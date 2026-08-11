use crate::{ApplicationError, ApplicationResult};
use football_domain::SpreadsheetImportPreview;

pub(super) fn ensure_preview_committable(
    preview: &SpreadsheetImportPreview,
    label: &str,
) -> ApplicationResult<()> {
    let blocking = preview.counts.conflict + preview.counts.error;
    let ready =
        preview.counts.ready_add + preview.counts.ready_update + preview.counts.ready_end_previous;
    if blocking > 0 {
        return Err(ApplicationError::Validation(format!(
            "{label}预检仍有 {blocking} 条冲突或错误，不能提交"
        )));
    }
    if ready == 0 && preview.counts.imported == 0 {
        return Err(ApplicationError::Validation(format!(
            "{label}预检没有可写入或已完成记录"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use football_domain::{SpreadsheetImportCounts, SpreadsheetImportMode};
    use uuid::Uuid;

    fn preview_with_counts(counts: SpreadsheetImportCounts) -> SpreadsheetImportPreview {
        SpreadsheetImportPreview {
            batch_id: Uuid::nil(),
            source_file_name: "fixture.xlsx".to_string(),
            source_sha256: "fixture".to_string(),
            import_mode: SpreadsheetImportMode::AddAndUpdate,
            counts,
            rows: Vec::new(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn already_imported_team_chain_remains_retryable() {
        let preview = preview_with_counts(SpreadsheetImportCounts {
            total: 8,
            imported: 8,
            ..Default::default()
        });
        ensure_preview_committable(&preview, "球队链")
            .expect("已提交球队链应允许继续重试完整资料包球员链");
    }

    #[test]
    fn truly_empty_preview_is_rejected() {
        let preview = preview_with_counts(SpreadsheetImportCounts::default());
        let error = ensure_preview_committable(&preview, "球队链")
            .expect_err("没有待写入或已导入记录时必须拒绝提交");
        assert!(error.to_string().contains("没有可写入或已完成记录"));
    }
}
