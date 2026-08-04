use crate::{ApplicationError, ApplicationResult, ApplicationService};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::{
    ApiWorkspaceApplyResult, ApiWorkspaceAttachment, ApiWorkspaceGeneratedFileContent,
    ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord,
    ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord, ApiWorkspacePreset,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    AvailabilityStatus, OpenAiUsageTotals, PlayerAbilityObservationDraft, PlayerAvailabilityDraft,
    PlayerDynamicTagDraft, PlayerListQuery, PlayerNameDraft, PlayerPositionDraft, TeamNameDraft,
    TeamProfileDraft,
};
use football_spreadsheet_io::read_workbook_for_api;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MAX_ATTACHMENT_FILES: usize = 5;
const MAX_ATTACHMENT_TOTAL_BYTES: u64 = 5 * 1024 * 1024;
const MAX_TEXT_ATTACHMENT_BYTES: usize = 1_500_000;

#[derive(Debug, Clone)]
pub struct ApiWorkspacePresetSpec {
    pub preset: ApiWorkspacePreset,
    pub instructions: String,
}

pub fn api_workspace_preset_specs() -> Vec<ApiWorkspacePresetSpec> {
    vec![
        plain_preset(
            "plain_chat",
            "通用问答",
            "普通文本提问与回答，不联网、不写库、不生成文件。",
            "通用",
            false,
            &[
                "解释当前页面中的数据和字段含义。",
                "比较两个对象的差异并指出需要人工确认的地方。",
                "根据已附加的只读上下文回答我的问题。",
            ],
            "Answer the user's question directly and clearly.",
        ),
        plain_preset(
            "match_research",
            "比赛问答（历史兼容）",
            "围绕已选比赛和客户端只读上下文进行普通文本问答。",
            "比赛",
            true,
            &[
                "根据当前比赛上下文说明还缺少哪些模型输入。",
                "解释双方阵容和可用性数据之间的差异。",
                "只根据客户端上下文整理需要人工核验的项目。",
            ],
            "Answer questions about the selected match using only the supplied read-only desktop context.",
        ),
        plain_preset(
            "availability_verification",
            "球员可用性问答（历史兼容）",
            "解释已保存的伤停、停赛、轮休和复出信息，不进行联网核验。",
            "比赛",
            true,
            &[
                "解释当前可用性记录对阵容输入的影响。",
                "列出上下文中仍处于未知状态的球员。",
                "指出记录之间的时间冲突。",
            ],
            "Explain player availability records without claiming external verification.",
        ),
        plain_preset(
            "lineup_player_cleanup",
            "阵容与球员问答（历史兼容）",
            "解释阵容、球员、位置和身份匹配问题。",
            "资料",
            false,
            &[
                "解释当前阵容名单中有哪些身份匹配风险。",
                "指出位置或角色信息中的缺口。",
                "说明应如何人工处理同名球员。",
            ],
            "Explain lineup and player identity issues using only the supplied context.",
        ),
        plain_preset(
            "player_profile_completion",
            "球员档案问答",
            "围绕当前球员的只读档案进行普通文本问答。",
            "资料",
            false,
            &[
                "解释这个球员档案目前缺少哪些字段。",
                "说明现有位置、履历和可用性记录之间的关系。",
                "列出需要通过球员月度 Excel 更新的内容。",
            ],
            "Answer questions about the selected player's read-only profile.",
        ),
        plain_preset(
            "team_profile_completion",
            "球队档案问答",
            "围绕当前球队的只读档案进行普通文本问答。",
            "资料",
            false,
            &[
                "解释这个球队档案目前缺少哪些字段。",
                "说明现有阵容、比赛和球队资料之间的关系。",
                "列出需要通过球队月度 Excel 更新的内容。",
            ],
            "Answer questions about the selected team's read-only profile.",
        ),
        plain_preset(
            "file_structuring",
            "文件处理（历史兼容）",
            "旧会话兼容项；新 AI 问答不再接收附件或生成文件。",
            "历史",
            false,
            &["说明为什么应改用 Excel 工作包维护资料。"],
            "Explain legacy file-processing conversations without accepting new attachments.",
        ),
        plain_preset(
            "database_quality_audit",
            "数据质量问答（历史兼容）",
            "根据只读上下文解释缺失、冲突和时效问题。",
            "资料",
            false,
            &[
                "解释当前数据中的缺失、重复和时间边界问题。",
                "按严重程度整理需要人工处理的项目。",
            ],
            "Explain data-quality issues without proposing database operations.",
        ),
        plain_preset(
            "custom_analysis",
            "自定义问答（历史兼容）",
            "普通文本解释、比较和梳理。",
            "通用",
            false,
            &[
                "解释这段资料与当前模型输入之间的关系。",
                "列出还需要我补充的资料和下一步操作。",
            ],
            "Answer the user's request in ordinary text.",
        ),
    ]
}

pub fn api_workspace_presets() -> Vec<ApiWorkspacePreset> {
    api_workspace_preset_specs()
        .into_iter()
        .map(|spec| spec.preset)
        .collect()
}

pub fn api_workspace_preset_spec(key: &str) -> ApplicationResult<ApiWorkspacePresetSpec> {
    api_workspace_preset_specs()
        .into_iter()
        .find(|spec| spec.preset.key == key)
        .ok_or_else(|| ApplicationError::Validation(format!("未知API协作预设：{key}")))
}

impl ApplicationService {
    pub async fn api_workspace_openai_usage_totals(&self) -> ApplicationResult<OpenAiUsageTotals> {
        let store = self.active_store().await?;
        let formal = store.openai_usage_totals().await?;
        let workspace = store.api_workspace_usage_totals().await?;
        Ok(OpenAiUsageTotals {
            today_cost_usd: formal.today_cost_usd + workspace.today_cost_usd,
            month_cost_usd: formal.month_cost_usd + workspace.month_cost_usd,
            today_request_count: formal
                .today_request_count
                .saturating_add(workspace.today_request_count),
            month_request_count: formal
                .month_request_count
                .saturating_add(workspace.month_request_count),
        })
    }

    pub async fn list_api_workspace_sessions(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ApiWorkspaceSessionRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_api_workspace_sessions(limit)
            .await?)
    }

    pub async fn create_api_workspace_session(
        &self,
        draft: ApiWorkspaceSessionDraft,
    ) -> ApplicationResult<ApiWorkspaceSessionRecord> {
        let preset = api_workspace_preset_spec(&draft.preset_key)?;
        if preset.preset.requires_match && draft.match_id.is_none() {
            return Err(ApplicationError::Validation(
                "该API协作预设必须选择一场比赛".to_string(),
            ));
        }
        Ok(self
            .active_store()
            .await?
            .create_api_workspace_session(&draft)
            .await?)
    }

    pub async fn archive_api_workspace_session(&self, session_id: Uuid) -> ApplicationResult<()> {
        Ok(self
            .active_store()
            .await?
            .archive_api_workspace_session(session_id)
            .await?)
    }

    pub async fn read_api_workspace_session(
        &self,
        session_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail> {
        Ok(self
            .active_store()
            .await?
            .read_api_workspace_session(session_id)
            .await?)
    }

    pub async fn append_api_workspace_user_message(
        &self,
        session_id: Uuid,
        content: String,
        attachments: &[ApiWorkspaceAttachment],
    ) -> ApplicationResult<ApiWorkspaceMessageRecord> {
        let trimmed = content.trim();
        if trimmed.is_empty() && attachments.is_empty() {
            return Err(ApplicationError::Validation(
                "请输入问题或选择附件".to_string(),
            ));
        }
        let attachment_metadata = attachments
            .iter()
            .map(|attachment| {
                json!({
                    "name": attachment.name,
                    "media_type": attachment.media_type,
                    "content_sha256": attachment.content_sha256,
                    "original_size_bytes": attachment.original_size_bytes,
                    "truncated": attachment.truncated
                })
            })
            .collect::<Vec<_>>();
        let draft = ApiWorkspaceMessageDraft {
            session_id,
            role: "user".to_string(),
            content: trimmed.to_string(),
            structured_payload: json!({}),
            citations: json!([]),
            attachments: Value::Array(attachment_metadata),
            provider_response_id: None,
            model_id: None,
            token_usage: json!({}),
        };
        Ok(self
            .active_store()
            .await?
            .append_api_workspace_message(&draft)
            .await?)
    }

    pub async fn append_api_workspace_assistant_bundle(
        &self,
        message: ApiWorkspaceMessageDraft,
        operations: Vec<ApiWorkspaceOperationDraft>,
        files: Vec<ApiWorkspaceGeneratedFileDraft>,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail> {
        Ok(self
            .active_store()
            .await?
            .append_api_workspace_assistant_bundle(&message, &operations, &files)
            .await?)
    }

    pub async fn api_workspace_context(
        &self,
        match_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> ApplicationResult<Value> {
        let store = self.active_store().await?;
        let match_context = if let Some(match_id) = match_id {
            Some(serde_json::to_value(
                store.ai_match_package_context(match_id).await?,
            )?)
        } else {
            None
        };

        let normalized_entity_type = entity_type.map(str::trim).filter(|value| !value.is_empty());
        match (normalized_entity_type, entity_id) {
            (Some("team"), Some(team_id)) => {
                let references = store.player_catalog_reference_data().await?;
                let team = self.read_team(team_id).await?;
                return Ok(json!({
                    "selected_match": match_context.clone(),
                    "selected_entity": {"type": "team", "id": team_id},
                    "scope": "complete_team_database_context",
                    "team": team,
                    "positions": references.positions,
                    "ability_dimensions": references.ability_dimensions,
                    "dynamic_tag_definitions": references.dynamic_tag_definitions,
                    "note": "The selected team detail is complete for the desktop team center at request time. Public current facts still require source verification."
                }));
            }
            (Some("player"), Some(player_id)) => {
                let references = store.player_catalog_reference_data().await?;
                let player = self.read_player(player_id).await?;
                return Ok(json!({
                    "selected_match": match_context.clone(),
                    "selected_entity": {"type": "player", "id": player_id},
                    "scope": "complete_player_database_context",
                    "player": player,
                    "positions": references.positions,
                    "ability_dimensions": references.ability_dimensions,
                    "dynamic_tag_definitions": references.dynamic_tag_definitions,
                    "note": "The selected player detail is complete for the desktop player center at request time. Public current facts still require source verification."
                }));
            }
            (None, None) => {
                if let Some(context) = match_context {
                    return Ok(context);
                }
            }
            _ => {
                return Err(ApplicationError::Validation(
                    "API协作实体上下文必须同时提供有效的类型和ID".to_string(),
                ));
            }
        }

        let references = store.player_catalog_reference_data().await?;
        let query = PlayerListQuery {
            limit: 200,
            ..PlayerListQuery::default()
        };
        let players = store.list_players(&query).await?;
        Ok(json!({
            "selected_match": null,
            "selected_entity": null,
            "scope": "bounded_general_database_context",
            "teams": references.teams,
            "positions": references.positions,
            "ability_dimensions": references.ability_dimensions,
            "dynamic_tag_definitions": references.dynamic_tag_definitions,
            "managed_matches": references.managed_matches,
            "players": players.items,
            "players_truncated": players.has_more,
            "note": "General context is bounded to 200 active players. Open API collaboration from a team or player center, bind a match, or attach an exported file for complete target context."
        }))
    }

    pub async fn read_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord> {
        Ok(self
            .active_store()
            .await?
            .read_api_workspace_operation(operation_id)
            .await?)
    }

    pub async fn read_api_workspace_generated_file(
        &self,
        file_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceGeneratedFileContent> {
        Ok(self
            .active_store()
            .await?
            .read_api_workspace_generated_file(file_id)
            .await?)
    }

    pub async fn apply_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceApplyResult> {
        let store = self.active_store().await?;
        let operation = store.claim_api_workspace_operation(operation_id).await?;
        let apply_result = self.execute_api_workspace_operation(&operation).await;
        match apply_result {
            Ok(result) => {
                let record = store
                    .complete_api_workspace_operation(operation_id, "applied", result.clone(), None)
                    .await?;
                Ok(ApiWorkspaceApplyResult {
                    operation_id,
                    operation_type: record.operation_type,
                    status: record.status,
                    result,
                    error_message: None,
                })
            }
            Err(error) => {
                let message = error.to_string();
                let record = store
                    .complete_api_workspace_operation(
                        operation_id,
                        "failed",
                        json!({}),
                        Some(&message),
                    )
                    .await?;
                Ok(ApiWorkspaceApplyResult {
                    operation_id,
                    operation_type: record.operation_type,
                    status: record.status,
                    result: json!({}),
                    error_message: Some(message),
                })
            }
        }
    }

    pub async fn reject_api_workspace_operation(
        &self,
        operation_id: Uuid,
        reason: String,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord> {
        let reason = if reason.trim().is_empty() {
            "用户拒绝该数据库提案"
        } else {
            reason.trim()
        };
        Ok(self
            .active_store()
            .await?
            .reject_api_workspace_operation(operation_id, reason)
            .await?)
    }

    async fn execute_api_workspace_operation(
        &self,
        operation: &ApiWorkspaceOperationRecord,
    ) -> ApplicationResult<Value> {
        match operation.operation_type.as_str() {
            "add_player_name" => {
                let draft = PlayerNameDraft {
                    player_id: required_uuid(&operation.payload, "player_id")?,
                    name: required_text(&operation.payload, "name")?,
                    language_code: optional_text(&operation.payload, "language_code"),
                    is_primary: false,
                    valid_from: optional_date(&operation.payload, "valid_from")?,
                    valid_to: optional_date(&operation.payload, "valid_to")?,
                };
                Ok(serde_json::to_value(self.add_player_name(draft).await?)?)
            }
            "assign_player_position" => {
                let draft = PlayerPositionDraft {
                    player_id: required_uuid(&operation.payload, "player_id")?,
                    position_code: required_text(&operation.payload, "position_code")?,
                    proficiency: required_f64(&operation.payload, "proficiency")?,
                    default_role_code: optional_text(&operation.payload, "default_role_code"),
                    is_primary: false,
                    valid_from: optional_date(&operation.payload, "valid_from")?,
                    valid_to: optional_date(&operation.payload, "valid_to")?,
                    source_document_id: None,
                };
                Ok(serde_json::to_value(
                    self.assign_player_position(draft).await?,
                )?)
            }
            "add_player_availability" => {
                let draft = PlayerAvailabilityDraft {
                    player_id: required_uuid(&operation.payload, "player_id")?,
                    team_id: optional_uuid(&operation.payload, "team_id")?,
                    competition_id: optional_uuid(&operation.payload, "competition_id")?,
                    status: availability_status(required_text(&operation.payload, "status")?)?,
                    reason: optional_text(&operation.payload, "reason"),
                    confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                    valid_from: required_datetime(&operation.payload, "valid_from")?,
                    valid_to: optional_datetime(&operation.payload, "valid_to")?,
                    source_document_id: None,
                    metadata: operation_metadata(operation)?,
                };
                Ok(serde_json::to_value(
                    self.add_player_availability(draft).await?,
                )?)
            }
            "add_player_dynamic_tag" => {
                let draft = PlayerDynamicTagDraft {
                    player_id: required_uuid(&operation.payload, "player_id")?,
                    tag_code: required_text(&operation.payload, "tag_code")?,
                    value: required_f64(&operation.payload, "value")?,
                    label: optional_text(&operation.payload, "label"),
                    confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                    observed_at: required_datetime(&operation.payload, "observed_at")?,
                    valid_from: required_datetime(&operation.payload, "valid_from")?,
                    valid_to: required_datetime(&operation.payload, "valid_to")?,
                    competition_id: optional_uuid(&operation.payload, "competition_id")?,
                    position_code: optional_text(&operation.payload, "position_code"),
                    opponent_team_id: optional_uuid(&operation.payload, "opponent_team_id")?,
                    sample_size: optional_i32(&operation.payload, "sample_size").unwrap_or(1),
                    source_type: "api_workspace".to_string(),
                    calculation_version: optional_text(&operation.payload, "calculation_version")
                        .unwrap_or_else(|| "api-workspace-v2".to_string()),
                    source_document_id: None,
                    metadata: operation_metadata(operation)?,
                };
                Ok(serde_json::to_value(
                    self.add_player_dynamic_tag(draft).await?,
                )?)
            }
            "add_player_ability_observation" => {
                let observed_at = required_datetime(&operation.payload, "observed_at")?;
                let effective_from =
                    optional_datetime(&operation.payload, "effective_from")?.unwrap_or(observed_at);
                let draft = PlayerAbilityObservationDraft {
                    player_id: required_uuid(&operation.payload, "player_id")?,
                    dimension_code: required_text(&operation.payload, "dimension_code")?,
                    context_type: optional_text(&operation.payload, "context_type")
                        .unwrap_or_else(|| "general".to_string()),
                    context_id: optional_uuid(&operation.payload, "context_id")?,
                    value: required_f64(&operation.payload, "value")?,
                    confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                    sample_size: optional_i32(&operation.payload, "sample_size").unwrap_or(1),
                    observed_at,
                    effective_from,
                    effective_to: optional_datetime(&operation.payload, "effective_to")?,
                    calculation_version: optional_text(&operation.payload, "calculation_version")
                        .unwrap_or_else(|| "api-workspace-v2".to_string()),
                    source_document_id: None,
                    metadata: operation_metadata(operation)?,
                };
                Ok(serde_json::to_value(
                    self.add_player_ability_observation(draft).await?,
                )?)
            }
            "add_team_name" => {
                let draft = TeamNameDraft {
                    team_id: required_uuid(&operation.payload, "team_id")?,
                    name: required_text(&operation.payload, "name")?,
                    language_code: optional_text(&operation.payload, "language_code"),
                    valid_from: optional_date(&operation.payload, "valid_from")?,
                    valid_to: optional_date(&operation.payload, "valid_to")?,
                };
                Ok(serde_json::to_value(self.add_team_name(draft).await?)?)
            }
            "update_team_profile" => {
                let team_id = required_uuid(&operation.payload, "team_id")?;
                let current = self.read_team(team_id).await?.profile;
                let draft = TeamProfileDraft {
                    short_name: optional_text(&operation.payload, "short_name").or_else(|| {
                        current
                            .as_ref()
                            .and_then(|profile| profile.short_name.clone())
                    }),
                    team_type: optional_text(&operation.payload, "team_type")
                        .or_else(|| current.as_ref().map(|profile| profile.team_type.clone()))
                        .unwrap_or_else(|| "club".to_string()),
                    founded_year: optional_i16(&operation.payload, "founded_year")
                        .or_else(|| current.as_ref().and_then(|profile| profile.founded_year)),
                    city: optional_text(&operation.payload, "city")
                        .or_else(|| current.as_ref().and_then(|profile| profile.city.clone())),
                    stadium: optional_text(&operation.payload, "stadium")
                        .or_else(|| current.as_ref().and_then(|profile| profile.stadium.clone())),
                    head_coach: optional_text(&operation.payload, "head_coach").or_else(|| {
                        current
                            .as_ref()
                            .and_then(|profile| profile.head_coach.clone())
                    }),
                    default_formation: optional_text(&operation.payload, "default_formation")
                        .or_else(|| {
                            current
                                .as_ref()
                                .and_then(|profile| profile.default_formation.clone())
                        }),
                    tactical_style: optional_text(&operation.payload, "tactical_style")
                        .or_else(|| {
                            current
                                .as_ref()
                                .map(|profile| profile.tactical_style.clone())
                        })
                        .unwrap_or_else(|| "balanced".to_string()),
                    attack_rating: optional_f64(&operation.payload, "attack_rating")
                        .or_else(|| current.as_ref().and_then(|profile| profile.attack_rating)),
                    midfield_rating: optional_f64(&operation.payload, "midfield_rating")
                        .or_else(|| current.as_ref().and_then(|profile| profile.midfield_rating)),
                    defence_rating: optional_f64(&operation.payload, "defence_rating")
                        .or_else(|| current.as_ref().and_then(|profile| profile.defence_rating)),
                    goalkeeper_rating: optional_f64(&operation.payload, "goalkeeper_rating")
                        .or_else(|| {
                            current
                                .as_ref()
                                .and_then(|profile| profile.goalkeeper_rating)
                        }),
                    reputation: optional_f64(&operation.payload, "reputation")
                        .or_else(|| current.as_ref().and_then(|profile| profile.reputation)),
                    data_confidence: optional_f64(&operation.payload, "confidence")
                        .or_else(|| current.as_ref().map(|profile| profile.data_confidence))
                        .unwrap_or(operation.confidence),
                    notes: optional_text(&operation.payload, "notes")
                        .or_else(|| current.as_ref().and_then(|profile| profile.notes.clone())),
                    metadata: operation_metadata(operation)?,
                };
                Ok(serde_json::to_value(
                    self.upsert_team_profile(team_id, draft).await?,
                )?)
            }
            other => Err(ApplicationError::Validation(format!(
                "不允许的API数据库操作：{other}"
            ))),
        }
    }
}

pub async fn read_api_workspace_attachments(
    paths: Vec<String>,
) -> ApplicationResult<Vec<ApiWorkspaceAttachment>> {
    if paths.len() > MAX_ATTACHMENT_FILES {
        return Err(ApplicationError::Validation(format!(
            "一次最多选择{MAX_ATTACHMENT_FILES}个附件"
        )));
    }
    let mut total = 0u64;
    let mut attachments = Vec::with_capacity(paths.len());
    for raw_path in paths {
        let path = PathBuf::from(raw_path.trim());
        if !path.is_file() {
            return Err(ApplicationError::Validation(format!(
                "附件不存在：{}",
                path.display()
            )));
        }
        let metadata = std::fs::metadata(&path)?;
        total = total.saturating_add(metadata.len());
        if total > MAX_ATTACHMENT_TOTAL_BYTES {
            return Err(ApplicationError::Validation(
                "附件总大小不能超过5 MiB".to_string(),
            ));
        }
        attachments.push(read_attachment(&path, metadata.len()).await?);
    }
    Ok(attachments)
}

async fn read_attachment(
    path: &Path,
    original_size: u64,
) -> ApplicationResult<ApiWorkspaceAttachment> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| ApplicationError::Validation("附件缺少扩展名".to_string()))?;
    let (media_type, mut content) = match extension.as_str() {
        "txt" => ("text/plain", read_utf8(path)?),
        "md" => ("text/markdown", read_utf8(path)?),
        "json" => {
            let text = read_utf8(path)?;
            let value: Value = serde_json::from_str(&text)?;
            ("application/json", serde_json::to_string_pretty(&value)?)
        }
        "csv" => ("text/csv", read_utf8(path)?),
        "tsv" => ("text/tab-separated-values", read_utf8(path)?),
        "xlsx" => {
            let path = path.to_path_buf();
            let content = tokio::task::spawn_blocking(move || read_workbook_for_api(&path))
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!("Excel附件读取任务失败：{error}"))
                })??;
            (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                content,
            )
        }
        _ => {
            return Err(ApplicationError::Validation(format!(
                "不支持的附件类型：.{extension}；仅支持 txt、md、json、csv、tsv、xlsx"
            )))
        }
    };
    let truncated = content.len() > MAX_TEXT_ATTACHMENT_BYTES;
    if truncated {
        truncate_utf8_bytes(&mut content, MAX_TEXT_ATTACHMENT_BYTES);
        content.push_str("\n[attachment content truncated by the desktop client]\n");
    }
    let content_sha256 = hex::encode(Sha256::digest(content.as_bytes()));
    Ok(ApiWorkspaceAttachment {
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("attachment")
            .to_string(),
        media_type: media_type.to_string(),
        content,
        content_sha256,
        original_size_bytes: original_size,
        truncated,
    })
}

fn read_utf8(path: &Path) -> ApplicationResult<String> {
    let bytes = std::fs::read(path)?;
    String::from_utf8(bytes)
        .map_err(|_| ApplicationError::Validation(format!("附件不是UTF-8文本：{}", path.display())))
}

fn truncate_utf8_bytes(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
}

fn plain_preset(
    key: &str,
    title: &str,
    description: &str,
    category: &str,
    requires_match: bool,
    suggested_questions: &[&str],
    instructions: &str,
) -> ApiWorkspacePresetSpec {
    ApiWorkspacePresetSpec {
        preset: ApiWorkspacePreset {
            key: key.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            web_search_enabled: false,
            requires_match,
            allowed_operation_types: Vec::new(),
            suggested_questions: suggested_questions
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        },
        instructions: instructions.to_string(),
    }
}

fn required_text(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| ApplicationError::Validation(format!("数据库提案缺少字段：{key}")))
}

fn optional_text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn required_uuid(value: &Value, key: &str) -> ApplicationResult<Uuid> {
    let raw = required_text(value, key)?;
    Uuid::parse_str(&raw)
        .map_err(|error| ApplicationError::Validation(format!("字段{key}不是有效UUID：{error}")))
}

fn optional_uuid(value: &Value, key: &str) -> ApplicationResult<Option<Uuid>> {
    optional_text(value, key)
        .map(|raw| {
            Uuid::parse_str(&raw).map_err(|error| {
                ApplicationError::Validation(format!("字段{key}不是有效UUID：{error}"))
            })
        })
        .transpose()
}

fn required_f64(value: &Value, key: &str) -> ApplicationResult<f64> {
    optional_f64(value, key)
        .ok_or_else(|| ApplicationError::Validation(format!("数据库提案缺少数值字段：{key}")))
}

fn optional_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn optional_i32(value: &Value, key: &str) -> Option<i32> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn optional_i16(value: &Value, key: &str) -> Option<i16> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i16::try_from(value).ok())
}

fn required_datetime(value: &Value, key: &str) -> ApplicationResult<DateTime<Utc>> {
    let raw = required_text(value, key)?;
    DateTime::parse_from_rfc3339(&raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| ApplicationError::Validation(format!("字段{key}时间无效：{error}")))
}

fn optional_datetime(value: &Value, key: &str) -> ApplicationResult<Option<DateTime<Utc>>> {
    optional_text(value, key)
        .map(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    ApplicationError::Validation(format!("字段{key}时间无效：{error}"))
                })
        })
        .transpose()
}

fn optional_date(value: &Value, key: &str) -> ApplicationResult<Option<NaiveDate>> {
    optional_text(value, key)
        .map(|raw| {
            NaiveDate::parse_from_str(&raw, "%Y-%m-%d").map_err(|error| {
                ApplicationError::Validation(format!("字段{key}日期无效：{error}"))
            })
        })
        .transpose()
}

fn availability_status(value: String) -> ApplicationResult<AvailabilityStatus> {
    match value.as_str() {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        _ => Err(ApplicationError::Validation(format!(
            "未知球员可用性状态：{value}"
        ))),
    }
}

fn operation_metadata(operation: &ApiWorkspaceOperationRecord) -> ApplicationResult<Value> {
    let mut metadata = optional_text(&operation.payload, "metadata_json")
        .map(|raw| serde_json::from_str::<Value>(&raw))
        .transpose()?
        .unwrap_or_else(|| json!({}));
    if !metadata.is_object() {
        return Err(ApplicationError::Validation(
            "metadata_json必须是JSON对象".to_string(),
        ));
    }
    let object: &mut Map<String, Value> = metadata.as_object_mut().expect("checked object");
    object.insert(
        "api_workspace_operation_id".to_string(),
        json!(operation.id),
    );
    object.insert(
        "api_workspace_session_id".to_string(),
        json!(operation.session_id),
    );
    object.insert(
        "api_workspace_rationale".to_string(),
        json!(operation.rationale),
    );
    object.insert(
        "source_urls".to_string(),
        operation
            .payload
            .get("source_urls")
            .cloned()
            .unwrap_or_else(|| json!([])),
    );
    Ok(metadata)
}
