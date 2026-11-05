use super::{sanitize_sensitive, IssueLogEntry};
use crate::file_store::write_atomic;
use chrono::Utc;
use std::{fs, path::Path};
pub(super) fn export_text(output_path: &Path, entries: &[IssueLogEntry]) -> Result<(), String> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建日志导出目录 {}：{error}", parent.display()))?;
    }
    let total_occurrences = entries
        .iter()
        .fold(0u64, |sum, item| sum.saturating_add(item.occurrence_count));
    let repeated = total_occurrences.saturating_sub(entries.len() as u64);
    let mut report = String::new();
    report.push_str("足球赛事模型平台 · 问题日志报告\n");
    report.push_str(&format!("导出时间：{}\n", Utc::now().to_rfc3339()));
    report.push_str(&format!("客户端版本：{}\n", env!("CARGO_PKG_VERSION")));
    report.push_str(&format!(
        "独立问题：{} · 总发生次数：{} · 已聚合重复：{}\n\n",
        entries.len(),
        total_occurrences,
        repeated
    ));

    for (index, item) in entries.iter().enumerate() {
        report.push_str(&format!(
            "{}. [{}] {}\n",
            index + 1,
            item.severity,
            item.user_message
        ));
        report.push_str(&format!("   问题编号：{}\n", item.id));
        report.push_str(&format!("   来源：{}\n", item.source));
        report.push_str(&format!("   发生次数：{}\n", item.occurrence_count));
        report.push_str(&format!("   首次：{}\n", item.first_seen_at.to_rfc3339()));
        report.push_str(&format!("   最近：{}\n", item.last_seen_at.to_rfc3339()));
        report.push_str(&format!("   涉及操作：{}\n", item.operations.join("、")));
        report.push_str(&format!("   技术详情：{}\n\n", item.technical_message));
    }

    write_atomic(output_path, sanitize_sensitive(&report).as_bytes(), false)
        .map_err(|error| format!("无法导出问题日志 {}：{error}", output_path.display()))
}
