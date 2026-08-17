use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::PlayerDynamicTagDraft;
use sqlx::Row;

pub(super) async fn validate_dynamic_tag_draft(
    store: &PostgresStore,
    draft: &PlayerDynamicTagDraft,
) -> PersistenceResult<()> {
    if draft.valid_to <= draft.valid_from {
        return Err(PersistenceError::InvalidState(
            "动态标签失效时间必须晚于生效时间".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "动态标签 confidence 必须在 0–1 之间".to_string(),
        ));
    }
    if draft.sample_size < 0 {
        return Err(PersistenceError::InvalidState(
            "动态标签 sample_size 不能为负数".to_string(),
        ));
    }
    if draft.calculation_version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "动态标签 calculation_version 不能为空".to_string(),
        ));
    }
    let range = sqlx::query(
        r#"
        SELECT minimum_value, maximum_value
        FROM feature.player_dynamic_tag_definitions
        WHERE code = $1
        "#,
    )
    .bind(draft.tag_code.trim())
    .fetch_optional(&store.pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState(format!("未知动态标签：{}", draft.tag_code)))?;
    let minimum: f64 = range.try_get("minimum_value")?;
    let maximum: f64 = range.try_get("maximum_value")?;
    if draft.value < minimum || draft.value > maximum {
        return Err(PersistenceError::InvalidState(format!(
            "动态标签 {} 的值必须在 {}–{} 之间",
            draft.tag_code, minimum, maximum
        )));
    }
    Ok(())
}
