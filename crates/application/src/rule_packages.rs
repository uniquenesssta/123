use super::{ApplicationError, ApplicationResult, ApplicationService};
use crate::model_shell::{
    P4_FRIENDLY_MODEL_ID, P4_GROUP_STAGE_MODEL_ID, P4_KNOCKOUT_SINGLE_MODEL_ID,
    P4_KNOCKOUT_TWO_LEG_MODEL_ID, P4_LEAGUE_MODEL_ID, P4_MODEL_ID, P7_FRIENDLY_MODEL_ID,
    P7_GROUP_STAGE_MODEL_ID, P7_KNOCKOUT_SINGLE_MODEL_ID, P7_KNOCKOUT_TWO_LEG_MODEL_ID,
    P7_LEAGUE_MODEL_ID, P7_MODEL_ID,
};
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind, CompetitionProfile,
    RulePackageDraft, RulePackageSummary, RuleRouting,
};
use serde_json::{json, Value};

impl ApplicationService {
    pub async fn register_rule_package(
        &self,
        draft: RulePackageDraft,
    ) -> ApplicationResult<RulePackageSummary> {
        validate_rule_package_shape(&draft)?;
        let model = self
            .registry
            .get(&draft.routing.model_id)
            .ok_or_else(|| ApplicationError::ModelNotFound(draft.routing.model_id.clone()))?;
        let descriptor = model.descriptor();
        if !descriptor
            .supported_competitions
            .contains(&draft.competition_profile.competition_kind)
        {
            return Err(ApplicationError::Validation(format!(
                "模型入口 {} 不支持赛事类型 {}",
                draft.routing.model_id,
                draft.competition_profile.competition_kind.as_str()
            )));
        }
        model
            .validate_parameters(&draft.parameters)
            .map_err(|error| ApplicationError::Model(error.to_string()))?;
        validate_parameter_identity(&draft)?;

        let store = self.active_store().await?;
        let summary = store.register_rule_package(&descriptor, &draft).await?;
        if draft.routing.activate_as_type_default {
            store
                .ensure_type_default_binding(
                    summary.id,
                    draft.competition_profile.competition_kind,
                    draft.routing.priority,
                    &format!("{} 类型默认", draft.display_name),
                )
                .await?;
        }
        Ok(summary)
    }

    pub async fn create_competition_binding(
        &self,
        draft: CompetitionBindingDraft,
    ) -> ApplicationResult<CompetitionBindingSummary> {
        let store = self.active_store().await?;
        Ok(store.create_competition_binding(&draft).await?)
    }
}

pub fn default_rule_package_template() -> RulePackageDraft {
    let mut draft = make_public_rule_package(
        CompetitionKind::Custom,
        P4_MODEL_ID,
        "p4",
        "custom",
        "自定义赛事 P4 外部模型入口",
        100,
        false,
    );
    draft.package_key = "user.custom.competition.p4".to_string();
    draft.metadata = json!({
        "built_in": false,
        "template": true,
        "public_shell": true,
        "provider_required": true
    });
    draft
}

pub(super) fn built_in_rule_packages() -> Vec<RulePackageDraft> {
    vec![
        make_public_rule_package(
            CompetitionKind::League,
            P4_LEAGUE_MODEL_ID,
            "p4",
            "league",
            "P4 联赛外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::GroupStage,
            P4_GROUP_STAGE_MODEL_ID,
            "p4",
            "group-stage",
            "P4 小组赛外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::KnockoutSingleLeg,
            P4_KNOCKOUT_SINGLE_MODEL_ID,
            "p4",
            "knockout-single",
            "P4 单回合淘汰赛外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::KnockoutTwoLeg,
            P4_KNOCKOUT_TWO_LEG_MODEL_ID,
            "p4",
            "knockout-two-leg",
            "P4 两回合淘汰赛外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::Friendly,
            P4_FRIENDLY_MODEL_ID,
            "p4",
            "friendly",
            "P4 友谊赛外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::Custom,
            P4_MODEL_ID,
            "p4",
            "custom",
            "P4 自定义赛事外部模型入口",
            100,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::League,
            P7_LEAGUE_MODEL_ID,
            "p7",
            "league",
            "P7 联赛外部模型入口",
            90,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::GroupStage,
            P7_GROUP_STAGE_MODEL_ID,
            "p7",
            "group-stage",
            "P7 小组赛外部模型入口",
            90,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::KnockoutSingleLeg,
            P7_KNOCKOUT_SINGLE_MODEL_ID,
            "p7",
            "knockout-single",
            "P7 单回合淘汰赛外部模型入口",
            90,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::KnockoutTwoLeg,
            P7_KNOCKOUT_TWO_LEG_MODEL_ID,
            "p7",
            "knockout-two-leg",
            "P7 两回合淘汰赛外部模型入口",
            90,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::Friendly,
            P7_FRIENDLY_MODEL_ID,
            "p7",
            "friendly",
            "P7 友谊赛外部模型入口",
            90,
            true,
        ),
        make_public_rule_package(
            CompetitionKind::Custom,
            P7_MODEL_ID,
            "p7",
            "custom",
            "P7 自定义赛事外部模型入口",
            90,
            true,
        ),
    ]
}

fn make_public_rule_package(
    kind: CompetitionKind,
    model_id: &str,
    family: &str,
    suffix: &str,
    display_name: &str,
    priority: i32,
    activate_as_type_default: bool,
) -> RulePackageDraft {
    let version = env!("CARGO_PKG_VERSION");
    let family_token = family.to_ascii_uppercase();
    let suffix_token = suffix.replace('-', "_").to_ascii_uppercase();
    let profile_id = format!("PUBLIC_{family_token}_{suffix_token}_PROFILE");
    let model_version = format!("{family_token}_EXTERNAL_PROVIDER");
    let parameter_version = format!("{family_token}_EXTERNAL_PARAMETERS");
    let profile = CompetitionProfile {
        profile_id: profile_id.clone(),
        name: display_name.to_string(),
        competition_kind: kind,
        normal_time_minutes: 90,
        extra_time_possible: matches!(
            kind,
            CompetitionKind::KnockoutSingleLeg | CompetitionKind::KnockoutTwoLeg
        ),
        penalties_possible: matches!(
            kind,
            CompetitionKind::KnockoutSingleLeg | CompetitionKind::KnockoutTwoLeg
        ),
        two_legged: kind == CompetitionKind::KnockoutTwoLeg,
        neutral_venue: false,
        metadata: json!({
            "public_shell": true,
            "provider_required": true,
            "bundled_runtime": false
        }),
    };
    let parameters = json!({
        "model_version": model_version,
        "parameter_version": parameter_version,
        "provider": {
            "kind": "external",
            "bundled": false
        },
        "profile": {
            "profile_id": profile_id,
            "competition_type": kind.as_str(),
            "runtime": "external"
        }
    });

    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: format!("builtin.{family}.{suffix}"),
        version: version.to_string(),
        display_name: display_name.to_string(),
        competition_profile: profile,
        routing: RuleRouting {
            model_id: model_id.to_string(),
            model_version,
            parameter_version,
            priority,
            activate_as_type_default,
            supported_snapshot_types: vec![
                "T-N".to_string(),
                "T-24h".to_string(),
                "T-6h".to_string(),
                "T-1h".to_string(),
                "custom".to_string(),
            ],
        },
        parameters,
        feature_requirements: json!({
            "required": ["match_id", "kickoff_time", "team_a", "team_b"],
            "provider_boundary": "external"
        }),
        output_contract: json!({
            "provider_required": true,
            "bundled_runtime": false,
            "contract": "football.external-model-response.v1"
        }),
        source_document: None,
        metadata: json!({
            "built_in": true,
            "public_shell": true,
            "model_family": family,
            "provider_required": true,
            "bundled_runtime": false
        }),
    }
}

pub(super) fn validate_rule_package_shape(draft: &RulePackageDraft) -> ApplicationResult<()> {
    for (label, value) in [
        ("format_version", &draft.format_version),
        ("package_key", &draft.package_key),
        ("version", &draft.version),
        ("display_name", &draft.display_name),
        ("profile_id", &draft.competition_profile.profile_id),
        ("model_id", &draft.routing.model_id),
        ("model_version", &draft.routing.model_version),
        ("parameter_version", &draft.routing.parameter_version),
    ] {
        if value.trim().is_empty() {
            return Err(ApplicationError::Validation(format!(
                "规则包字段 {label} 不能为空"
            )));
        }
    }
    if draft.format_version != "football.rule-package.v1" {
        return Err(ApplicationError::Validation(format!(
            "不支持的规则包格式：{}",
            draft.format_version
        )));
    }
    if draft.competition_profile.normal_time_minutes == 0 {
        return Err(ApplicationError::Validation(
            "正常比赛时间必须大于 0".to_string(),
        ));
    }
    if !draft.parameters.is_object() {
        return Err(ApplicationError::Validation(
            "规则包 parameters 必须是 JSON 对象".to_string(),
        ));
    }
    if !draft.feature_requirements.is_object() || !draft.output_contract.is_object() {
        return Err(ApplicationError::Validation(
            "feature_requirements 和 output_contract 必须是 JSON 对象".to_string(),
        ));
    }
    if let Some(source) = &draft.source_document {
        let hash = source
            .content_sha256
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                ApplicationError::Validation("规则包包含来源文档时必须提供文档 SHA-256".to_string())
            })?;
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ApplicationError::Validation(
                "来源文档 SHA-256 必须是 64 位十六进制字符串".to_string(),
            ));
        }
    }

    let parameter_profile = draft
        .parameters
        .get("profile")
        .and_then(Value::as_object)
        .ok_or_else(|| ApplicationError::Validation("parameters.profile 缺失".to_string()))?;
    let parameter_profile_id = parameter_profile
        .get("profile_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if parameter_profile_id != draft.competition_profile.profile_id {
        return Err(ApplicationError::Validation(format!(
            "parameters.profile.profile_id 与 competition_profile.profile_id 不一致：{} != {}",
            parameter_profile_id, draft.competition_profile.profile_id
        )));
    }
    let parameter_kind = parameter_profile
        .get("competition_type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if parameter_kind != draft.competition_profile.competition_kind.as_str() {
        return Err(ApplicationError::Validation(format!(
            "parameters.profile.competition_type 与规则包赛事类型不一致：{} != {}",
            parameter_kind,
            draft.competition_profile.competition_kind.as_str()
        )));
    }
    Ok(())
}

pub(super) fn validate_parameter_identity(draft: &RulePackageDraft) -> ApplicationResult<()> {
    let parameter_model_version = required_string(&draft.parameters, "model_version")?;
    let parameter_version = required_string(&draft.parameters, "parameter_version")?;
    if parameter_model_version != draft.routing.model_version {
        return Err(ApplicationError::Validation(format!(
            "parameters.model_version 与 routing.model_version 不一致：{} != {}",
            parameter_model_version, draft.routing.model_version
        )));
    }
    if parameter_version != draft.routing.parameter_version {
        return Err(ApplicationError::Validation(format!(
            "parameters.parameter_version 与 routing.parameter_version 不一致：{} != {}",
            parameter_version, draft.routing.parameter_version
        )));
    }
    Ok(())
}

fn required_string(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{key}")))
}
