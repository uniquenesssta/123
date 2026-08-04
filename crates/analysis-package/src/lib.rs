use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageManifest, AiAnalysisPackageSummary,
    AiAnalysisResponseManifest, AiAnalysisResponsePreview, AiAnalysisSuggestionDraft,
    AI_ANALYSIS_PACKAGE_FORMAT, AI_ANALYSIS_RESPONSE_FORMAT,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};
use thiserror::Error;
use uuid::Uuid;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MANIFEST_FILE: &str = "manifest.json";
const OVERVIEW_FILE: &str = "model-performance.json";
const DATABASE_FILE: &str = "database-summary.json";
const CALIBRATION_FILE: &str = "calibration.json";
const DRIFT_FILE: &str = "drift.json";
const QUALITY_FILE: &str = "data-quality.json";
const QUERY_FILE: &str = "query-performance.json";
const PLAYER_FILE: &str = "player-review-summary.json";
const TEAM_FILE: &str = "team-review-summary.json";
const ABILITY_FILE: &str = "ability-update-candidates.json";
const SCHEMA_FILE: &str = "schema-summary.json";
const INSTRUCTIONS_FILE: &str = "README.txt";
const RESPONSE_SUGGESTIONS_FILE: &str = "suggestions.json";
const RESPONSE_SCHEMA_FILE: &str = "suggestions.schema.json";
const RESPONSE_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../../schemas/ai-analysis-response-v1.schema.json");

#[derive(Debug, Error)]
pub enum AnalysisPackageError {
    #[error("文件系统错误：{0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP 文件无效：{0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON 无效：{0}")]
    Json(#[from] serde_json::Error),
    #[error("分析包无效：{0}")]
    Invalid(String),
}

pub type AnalysisPackageResult<T> = Result<T, AnalysisPackageError>;

pub fn write_analysis_package(
    output_path: &Path,
    data: &AiAnalysisPackageData,
) -> AnalysisPackageResult<AiAnalysisPackageSummary> {
    let package_id = Uuid::new_v4();
    let created_at = chrono::Utc::now();
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    insert_json(&mut files, OVERVIEW_FILE, &data.overview)?;
    insert_json(&mut files, DATABASE_FILE, &data.database_summary)?;
    insert_json(&mut files, CALIBRATION_FILE, &data.overview.calibration)?;
    insert_json(&mut files, DRIFT_FILE, &data.overview.drift)?;
    insert_json(&mut files, QUALITY_FILE, &data.overview.data_quality)?;
    insert_json(&mut files, QUERY_FILE, &data.overview.query_performance)?;
    insert_json(&mut files, PLAYER_FILE, &data.player_review_summary)?;
    insert_json(&mut files, TEAM_FILE, &data.team_review_summary)?;
    insert_json(&mut files, ABILITY_FILE, &data.ability_candidates)?;
    insert_json(&mut files, SCHEMA_FILE, &data.schema_summary)?;
    files.insert(
        INSTRUCTIONS_FILE.to_string(),
        instructions().as_bytes().to_vec(),
    );
    let content_sha256 = hash_files(&files);
    let manifest = AiAnalysisPackageManifest {
        format_version: AI_ANALYSIS_PACKAGE_FORMAT.to_string(),
        package_id,
        created_at,
        calculation_version: data.overview.calculation_version.clone(),
        sample_size: data.overview.sample_size,
        content_sha256: content_sha256.clone(),
        files: files.keys().cloned().collect(),
    };

    let file = File::create(output_path)?;
    let mut writer = ZipWriter::new(file);
    write_entry(
        &mut writer,
        MANIFEST_FILE,
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    for (name, bytes) in &files {
        write_entry(&mut writer, name, bytes)?;
    }
    writer.finish()?;

    Ok(AiAnalysisPackageSummary {
        package_id,
        output_path: output_path.to_string_lossy().to_string(),
        content_sha256,
        sample_size: data.overview.sample_size,
        created_at,
    })
}

pub fn read_analysis_response(
    package_path: &Path,
) -> AnalysisPackageResult<AiAnalysisResponsePreview> {
    let file = File::open(package_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut warnings = validate_response_archive(&mut archive)?;
    let manifest: AiAnalysisResponseManifest = read_json_entry(&mut archive, MANIFEST_FILE)?;
    if manifest.format_version != AI_ANALYSIS_RESPONSE_FORMAT {
        return Err(AnalysisPackageError::Invalid(format!(
            "回包版本应为 {AI_ANALYSIS_RESPONSE_FORMAT}，实际为 {}",
            manifest.format_version
        )));
    }
    let suggestion_bytes = read_bytes_entry(&mut archive, RESPONSE_SUGGESTIONS_FILE)?;
    let mut suggestions: Vec<AiAnalysisSuggestionDraft> =
        serde_json::from_slice(&suggestion_bytes)?;
    let actual_hash = hash_bytes(&suggestion_bytes);
    let mut blocking_errors = Vec::new();
    if actual_hash != manifest.content_sha256 {
        blocking_errors.push("suggestions.json 校验值与 manifest 不一致".to_string());
    }
    if suggestions.is_empty() {
        warnings.push("回包没有包含任何建议".to_string());
    }
    for (index, suggestion) in suggestions.iter_mut().enumerate() {
        validate_suggestion(index, suggestion, &mut blocking_errors);
    }
    Ok(AiAnalysisResponsePreview {
        manifest,
        suggestions,
        blocking_errors,
        warnings,
    })
}

pub fn response_template_bytes(source_package_id: Option<Uuid>) -> AnalysisPackageResult<Vec<u8>> {
    let suggestions: Vec<AiAnalysisSuggestionDraft> = Vec::new();
    let suggestion_bytes = serde_json::to_vec_pretty(&suggestions)?;
    let manifest = AiAnalysisResponseManifest {
        format_version: AI_ANALYSIS_RESPONSE_FORMAT.to_string(),
        response_id: Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        source_package_id,
        content_sha256: hash_bytes(&suggestion_bytes),
    };
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    write_entry(
        &mut writer,
        MANIFEST_FILE,
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    write_entry(&mut writer, RESPONSE_SUGGESTIONS_FILE, &suggestion_bytes)?;
    write_entry(&mut writer, RESPONSE_SCHEMA_FILE, RESPONSE_SCHEMA_BYTES)?;
    write_entry(
        &mut writer,
        INSTRUCTIONS_FILE,
        b"Edit suggestions.json according to suggestions.schema.json. Then update manifest.json content_sha256 to the SHA-256 of the exact suggestions.json bytes. Import only after review.",
    )?;
    Ok(writer.finish()?.into_inner())
}

fn validate_response_archive(archive: &mut ZipArchive<File>) -> AnalysisPackageResult<Vec<String>> {
    let allowed = [
        MANIFEST_FILE,
        RESPONSE_SUGGESTIONS_FILE,
        RESPONSE_SCHEMA_FILE,
        INSTRUCTIONS_FILE,
    ];
    let mut counts = BTreeMap::<String, usize>::new();
    let mut warnings = Vec::new();
    let mut total_uncompressed = 0_u64;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name().replace('\\', "/");
        if entry.is_dir() || name.starts_with('/') || name.split('/').any(|part| part == "..") {
            return Err(AnalysisPackageError::Invalid(format!(
                "ZIP 包含不安全路径：{name}"
            )));
        }
        total_uncompressed = total_uncompressed.saturating_add(entry.size());
        if total_uncompressed > 24 * 1024 * 1024 {
            return Err(AnalysisPackageError::Invalid(
                "AI 回包解压后总大小超过 24 MB 限制".to_string(),
            ));
        }
        *counts.entry(name.clone()).or_insert(0) += 1;
        if !allowed.contains(&name.as_str()) {
            warnings.push(format!("回包包含未使用文件：{name}"));
        }
    }
    for required in [MANIFEST_FILE, RESPONSE_SUGGESTIONS_FILE] {
        match counts.get(required).copied().unwrap_or(0) {
            1 => {}
            0 => return Err(AnalysisPackageError::Invalid(format!("缺少 {required}"))),
            _ => {
                return Err(AnalysisPackageError::Invalid(format!(
                    "{required} 在 ZIP 中重复出现"
                )))
            }
        }
    }
    Ok(warnings)
}

fn validate_suggestion(
    index: usize,
    suggestion: &mut AiAnalysisSuggestionDraft,
    blocking_errors: &mut Vec<String>,
) {
    let number = index + 1;
    suggestion.title = suggestion.title.trim().to_string();
    suggestion.summary = suggestion.summary.trim().to_string();
    suggestion.suggestion_type = suggestion.suggestion_type.trim().to_string();
    suggestion.severity = suggestion.severity.trim().to_string();
    if suggestion.severity.is_empty() {
        suggestion.severity = "info".to_string();
    }
    if suggestion.title.is_empty() || suggestion.summary.is_empty() {
        blocking_errors.push(format!("第 {number} 条建议缺少标题或摘要"));
    }
    if suggestion.title.chars().count() > 200 || suggestion.summary.chars().count() > 4_000 {
        blocking_errors.push(format!("第 {number} 条建议的标题或摘要超过长度限制"));
    }
    if !matches!(
        suggestion.suggestion_type.as_str(),
        "ability_update" | "model_parameter" | "data_quality" | "database_index" | "review_finding"
    ) {
        blocking_errors.push(format!(
            "第 {number} 条建议类型无效：{}",
            suggestion.suggestion_type
        ));
    }
    if !matches!(
        suggestion.severity.as_str(),
        "info" | "warning" | "critical"
    ) {
        blocking_errors.push(format!(
            "第 {number} 条建议严重级别无效：{}",
            suggestion.severity
        ));
    }
    if !suggestion.scope.is_object()
        || !suggestion.payload.is_object()
        || !suggestion.evidence.is_object()
    {
        blocking_errors.push(format!(
            "第 {number} 条建议的 scope、payload 和 evidence 必须是 JSON 对象"
        ));
    }
    if suggestion.suggestion_type == "ability_update" {
        validate_ability_payload(number, &suggestion.payload, blocking_errors);
    }
}

fn validate_ability_payload(number: usize, payload: &serde_json::Value, errors: &mut Vec<String>) {
    let player_id = payload.get("player_id").and_then(serde_json::Value::as_str);
    if player_id
        .and_then(|value| Uuid::parse_str(value).ok())
        .is_none()
    {
        errors.push(format!("第 {number} 条能力建议缺少有效 player_id"));
    }
    if payload
        .get("dimension_code")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|value| value.trim().is_empty())
    {
        errors.push(format!("第 {number} 条能力建议缺少 dimension_code"));
    }
    if payload
        .get("proposed_value")
        .and_then(serde_json::Value::as_f64)
        .is_none_or(|value| !value.is_finite() || !(0.0..=100.0).contains(&value))
    {
        errors.push(format!(
            "第 {number} 条能力建议 proposed_value 必须在 0–100"
        ));
    }
    if payload
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
    {
        errors.push(format!("第 {number} 条能力建议 confidence 必须在 0–1"));
    }
    if payload
        .get("sample_size")
        .and_then(serde_json::Value::as_i64)
        .is_some_and(|value| value < 0)
    {
        errors.push(format!("第 {number} 条能力建议 sample_size 不能为负数"));
    }
}

fn insert_json<T: Serialize>(
    files: &mut BTreeMap<String, Vec<u8>>,
    name: &str,
    value: &T,
) -> AnalysisPackageResult<()> {
    files.insert(name.to_string(), serde_json::to_vec_pretty(value)?);
    Ok(())
}

fn read_json_entry<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> AnalysisPackageResult<T> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| AnalysisPackageError::Invalid(format!("缺少 {name}")))?;
    if entry.size() > 2 * 1024 * 1024 {
        return Err(AnalysisPackageError::Invalid(format!(
            "{name} 超过 2 MB 限制"
        )));
    }
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn read_bytes_entry(archive: &mut ZipArchive<File>, name: &str) -> AnalysisPackageResult<Vec<u8>> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| AnalysisPackageError::Invalid(format!("缺少 {name}")))?;
    if entry.is_dir() {
        return Err(AnalysisPackageError::Invalid(format!("{name} 不能是目录")));
    }
    if entry.size() > 20 * 1024 * 1024 {
        return Err(AnalysisPackageError::Invalid(format!(
            "{name} 超过 20 MB 限制"
        )));
    }
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn write_entry<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    name: &str,
    bytes: &[u8],
) -> AnalysisPackageResult<()> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer.start_file(name, options)?;
    writer.write_all(bytes)?;
    Ok(())
}

fn hash_files(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    for (name, bytes) in files {
        hasher.update(name.as_bytes());
        hasher.update([0]);
        hasher.update(bytes);
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn instructions() -> &'static str {
    "本包用于长期模型、数据质量、球员复盘和数据库性能分析。请不要直接修改正式数据库。分析后按 football.ai-analysis-response.v1 生成回包；所有建议导入后仍需在客户端审核。"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_template_can_be_read_back() {
        let source_package_id = Uuid::new_v4();
        let bytes = response_template_bytes(Some(source_package_id)).expect("template");
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("response.zip");
        std::fs::write(&path, bytes).expect("write template");

        let preview = read_analysis_response(&path).expect("read template");
        assert_eq!(preview.manifest.source_package_id, Some(source_package_id));
        assert!(preview.blocking_errors.is_empty());
        assert!(preview.suggestions.is_empty());
    }
}
