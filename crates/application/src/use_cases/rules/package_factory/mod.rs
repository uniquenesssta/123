use crate::model_shell::{
    P4_FRIENDLY_MODEL_ID, P4_GROUP_STAGE_MODEL_ID, P4_KNOCKOUT_SINGLE_MODEL_ID,
    P4_KNOCKOUT_TWO_LEG_MODEL_ID, P4_LEAGUE_MODEL_ID, P4_MODEL_ID, P7_FRIENDLY_MODEL_ID,
    P7_GROUP_STAGE_MODEL_ID, P7_KNOCKOUT_SINGLE_MODEL_ID, P7_KNOCKOUT_TWO_LEG_MODEL_ID,
    P7_LEAGUE_MODEL_ID, P7_MODEL_ID,
};
use football_domain::{CompetitionKind, CompetitionProfile, RulePackageDraft, RuleRouting};
use serde_json::json;

const BUILT_IN_RULE_PACKAGE_REVISION: &str = "public.1";

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

pub(crate) fn built_in_rule_packages() -> Vec<RulePackageDraft> {
    vec![
        make_built_in_rule_package(CompetitionKind::League, P4_LEAGUE_MODEL_ID, "p4", "league", "P4 联赛外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::GroupStage, P4_GROUP_STAGE_MODEL_ID, "p4", "group-stage", "P4 小组赛外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::KnockoutSingleLeg, P4_KNOCKOUT_SINGLE_MODEL_ID, "p4", "knockout-single", "P4 单回合淘汰赛外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::KnockoutTwoLeg, P4_KNOCKOUT_TWO_LEG_MODEL_ID, "p4", "knockout-two-leg", "P4 两回合淘汰赛外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::Friendly, P4_FRIENDLY_MODEL_ID, "p4", "friendly", "P4 友谊赛外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::Custom, P4_MODEL_ID, "p4", "custom", "P4 自定义赛事外部模型入口", 100, true),
        make_built_in_rule_package(CompetitionKind::League, P7_LEAGUE_MODEL_ID, "p7", "league", "P7 联赛外部模型入口", 90, true),
        make_built_in_rule_package(CompetitionKind::GroupStage, P7_GROUP_STAGE_MODEL_ID, "p7", "group-stage", "P7 小组赛外部模型入口", 90, true),
        make_built_in_rule_package(CompetitionKind::KnockoutSingleLeg, P7_KNOCKOUT_SINGLE_MODEL_ID, "p7", "knockout-single", "P7 单回合淘汰赛外部模型入口", 90, true),
        make_built_in_rule_package(CompetitionKind::KnockoutTwoLeg, P7_KNOCKOUT_TWO_LEG_MODEL_ID, "p7", "knockout-two-leg", "P7 两回合淘汰赛外部模型入口", 90, true),
        make_built_in_rule_package(CompetitionKind::Friendly, P7_FRIENDLY_MODEL_ID, "p7", "friendly", "P7 友谊赛外部模型入口", 90, true),
        make_built_in_rule_package(CompetitionKind::Custom, P7_MODEL_ID, "p7", "custom", "P7 自定义赛事外部模型入口", 90, true),
    ]
}

fn make_built_in_rule_package(
    kind: CompetitionKind,
    model_id: &str,
    family: &str,
    suffix: &str,
    display_name: &str,
    priority: i32,
    activate_as_type_default: bool,
) -> RulePackageDraft {
    let mut draft = make_public_rule_package(
        kind,
        model_id,
        family,
        suffix,
        display_name,
        priority,
        activate_as_type_default,
    );
    draft.version = built_in_rule_package_version();
    draft
}

fn built_in_rule_package_version() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        BUILT_IN_RULE_PACKAGE_REVISION
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        use_cases::rules::package_validation::{
            validate_parameter_identity, validate_rule_package_shape,
        },
        ApplicationService,
    };
    use std::collections::HashSet;

    #[test]
    fn built_in_rule_packages_are_unique_and_valid() {
        let service = ApplicationService::new();
        let mut package_keys = HashSet::new();
        let mut model_ids = HashSet::new();
        let expected_version = format!("{}+public.1", env!("CARGO_PKG_VERSION"));

        for draft in built_in_rule_packages() {
            assert!(package_keys.insert(draft.package_key.clone()));
            model_ids.insert(draft.routing.model_id.clone());
            assert_eq!(draft.version, expected_version);
            validate_rule_package_shape(&draft).expect("内置规则包结构无效");
            validate_parameter_identity(&draft).expect("内置规则包版本无效");
            let model = service
                .registry
                .get(&draft.routing.model_id)
                .expect("内置规则包引用了未注册模型");
            assert!(model
                .descriptor()
                .supported_competitions
                .contains(&draft.competition_profile.competition_kind));
            model
                .validate_parameters(&draft.parameters)
                .expect("内置规则包参数无效");
        }

        assert_eq!(package_keys.len(), CompetitionKind::ALL.len() * 2);
        assert_eq!(model_ids.len(), CompetitionKind::ALL.len() * 2);
    }

    #[test]
    fn user_rule_package_template_is_self_consistent() {
        let draft = default_rule_package_template();
        validate_rule_package_shape(&draft).expect("用户规则包模板结构无效");
        validate_parameter_identity(&draft).expect("用户规则包版本无效");
        assert_eq!(draft.format_version, "football.rule-package.v1");
        assert_eq!(draft.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            draft.competition_profile.competition_kind,
            CompetitionKind::Custom
        );
    }
}
