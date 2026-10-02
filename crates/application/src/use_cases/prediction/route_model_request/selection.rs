use crate::{model_registry::ModelRegistry, ApplicationError, ApplicationResult};

#[derive(Debug, Clone)]
pub(crate) struct ModelSelection {
    pub(crate) family: &'static str,
    pub(crate) exact_model_id: Option<String>,
}

pub(crate) fn normalize_model_selection(value: &str) -> ApplicationResult<ModelSelection> {
    let normalized = value.trim().to_ascii_lowercase();
    let normalized = if normalized.is_empty() {
        "p4".to_string()
    } else {
        normalized
    };
    let family = if normalized == "p4" || normalized.starts_with("p4_") {
        "p4"
    } else if normalized == "p7" || normalized.starts_with("p7_") {
        "p7"
    } else {
        return Err(ApplicationError::Validation(format!(
            "不支持的模型：{normalized}；请选择已注册的 P4 或 P7 模型"
        )));
    };
    let exact_model_id = if normalized == family {
        None
    } else {
        Some(normalized)
    };
    Ok(ModelSelection {
        family,
        exact_model_id,
    })
}

pub(crate) fn ensure_model_selection_registered(
    registry: &ModelRegistry,
    selection: &ModelSelection,
) -> ApplicationResult<()> {
    if let Some(model_id) = selection.exact_model_id.as_deref() {
        if registry.get(model_id).is_none() {
            return Err(ApplicationError::ModelNotFound(model_id.to_string()));
        }
    }
    Ok(())
}
