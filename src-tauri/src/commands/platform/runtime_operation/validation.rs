pub(super) const MAX_OPERATION_NAME_CHARS: usize = 160;
pub(super) const MAX_TRACE_ID_CHARS: usize = 160;

pub(super) fn normalize_phase(value: &str) -> Result<&'static str, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "started" => Ok("started"),
        "completed" => Ok("completed"),
        "failed" => Ok("failed"),
        "ui_action" => Ok("ui_action"),
        "navigation" => Ok("navigation"),
        "diagnostic" => Ok("diagnostic"),
        other => Err(format!("不支持的运行日志阶段：{other}")),
    }
}

pub(super) fn validate_text(label: &str, value: &str, max_chars: usize) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    let char_count = trimmed.chars().count();
    if char_count > max_chars {
        return Err(format!("{label}过长：{char_count} > {max_chars}"));
    }
    Ok(trimmed.to_string())
}
