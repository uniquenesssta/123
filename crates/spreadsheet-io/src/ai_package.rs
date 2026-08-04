use crate::{SpreadsheetError, SpreadsheetResult};
use football_domain::{
    AiMatchPackageContext, AiMatchPackageManifest, AiMatchPackageSummary, AI_MATCH_PACKAGE_FORMAT,
};
use serde_json::to_vec_pretty;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const WORKBOOK_FILE: &str = "match_and_lineup.xlsx";
const CONTEXT_FILE: &str = "database_context.json";
const MANIFEST_FILE: &str = "manifest.json";
const INSTRUCTIONS_FILE: &str = "README.txt";

pub fn write_ai_match_package(
    output_path: &Path,
    workbook_path: &Path,
    context: &AiMatchPackageContext,
) -> SpreadsheetResult<AiMatchPackageSummary> {
    let workbook = std::fs::read(workbook_path)?;
    let context_bytes = to_vec_pretty(context)
        .map_err(|error| SpreadsheetError::InvalidTemplate(error.to_string()))?;
    let content_sha256 = context_hash(&context_bytes);
    let manifest = AiMatchPackageManifest {
        format_version: AI_MATCH_PACKAGE_FORMAT.to_string(),
        created_at: context.generated_at,
        match_id: context.match_record.id,
        match_key: context.match_record.external_key.clone(),
        workbook_file: WORKBOOK_FILE.to_string(),
        context_file: CONTEXT_FILE.to_string(),
        instructions_file: INSTRUCTIONS_FILE.to_string(),
        content_sha256: content_sha256.clone(),
    };
    let manifest_bytes = to_vec_pretty(&manifest)
        .map_err(|error| SpreadsheetError::InvalidTemplate(error.to_string()))?;
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    write_entry(&mut zip, MANIFEST_FILE, &manifest_bytes, zip_options())?;
    write_entry(&mut zip, WORKBOOK_FILE, &workbook, zip_options())?;
    write_entry(&mut zip, CONTEXT_FILE, &context_bytes, zip_options())?;
    write_entry(
        &mut zip,
        INSTRUCTIONS_FILE,
        "本包由足球赛事模型平台导出。请优先编辑 match_and_lineup.xlsx；database_context.json 仅用于分析，不应修改。软件重新导入时会先执行预检和冲突处理。".as_bytes(),
        zip_options(),
    )?;
    zip.finish().map_err(|error| {
        SpreadsheetError::InvalidTemplate(format!("AI 交换包写入失败：{error}"))
    })?;
    Ok(AiMatchPackageSummary {
        output_path: output_path.to_string_lossy().to_string(),
        match_id: context.match_record.id,
        match_key: context.match_record.external_key.clone(),
        player_count: context.players.len() as u64,
        content_sha256,
    })
}

pub fn extract_ai_match_workbook(
    package_path: &Path,
    output_path: &Path,
) -> SpreadsheetResult<AiMatchPackageManifest> {
    let file = File::open(package_path)?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        SpreadsheetError::InvalidTemplate(format!("AI 交换包无法读取：{error}"))
    })?;
    let manifest: AiMatchPackageManifest = {
        let mut entry = archive.by_name(MANIFEST_FILE).map_err(|_| {
            SpreadsheetError::InvalidTemplate("AI 交换包缺少 manifest.json".to_string())
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        serde_json::from_slice(&bytes).map_err(|error| {
            SpreadsheetError::InvalidTemplate(format!("manifest.json 无效：{error}"))
        })?
    };
    if manifest.format_version != AI_MATCH_PACKAGE_FORMAT {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "AI 交换包版本应为 {AI_MATCH_PACKAGE_FORMAT}，实际为 {}",
            manifest.format_version
        )));
    }
    if manifest.workbook_file != WORKBOOK_FILE
        || manifest.context_file != CONTEXT_FILE
        || manifest.instructions_file != INSTRUCTIONS_FILE
    {
        return Err(SpreadsheetError::InvalidTemplate(
            "AI 交换包文件清单不符合固定格式".to_string(),
        ));
    }
    let workbook = {
        let mut entry = archive.by_name(&manifest.workbook_file).map_err(|_| {
            SpreadsheetError::InvalidTemplate("AI 交换包缺少比赛阵容工作簿".to_string())
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        bytes
    };
    let context = {
        let mut entry = archive.by_name(&manifest.context_file).map_err(|_| {
            SpreadsheetError::InvalidTemplate("AI 交换包缺少数据库上下文".to_string())
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        bytes
    };
    let actual_hash = context_hash(&context);
    if actual_hash != manifest.content_sha256 {
        return Err(SpreadsheetError::InvalidTemplate(
            "AI 交换包内容校验失败，文件可能被异常替换".to_string(),
        ));
    }
    std::fs::write(output_path, workbook)?;
    Ok(manifest)
}

fn zip_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

fn write_entry(
    zip: &mut ZipWriter<File>,
    name: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
) -> SpreadsheetResult<()> {
    zip.start_file(name, options)
        .map_err(|error| SpreadsheetError::InvalidTemplate(error.to_string()))?;
    zip.write_all(bytes)?;
    Ok(())
}

fn context_hash(context: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(context);
    let digest = hasher.finalize();
    format!("{digest:x}")
}
