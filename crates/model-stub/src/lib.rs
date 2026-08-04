use football_domain::{CompetitionKind, MatchContext};
use football_model_api::{
    ModelDescriptor, ModelError, ModelOutput, ModelRequest, ModelResult, PredictionModel,
};
use serde_json::Value;

pub const P4_MODEL_ID: &str = "p4";
pub const P4_LEAGUE_MODEL_ID: &str = "p4_league";
pub const P4_GROUP_STAGE_MODEL_ID: &str = "p4_group_stage";
pub const P4_KNOCKOUT_SINGLE_MODEL_ID: &str = "p4_knockout_90";
pub const P4_KNOCKOUT_TWO_LEG_MODEL_ID: &str = "p4_knockout_two_leg_90";
pub const P4_FRIENDLY_MODEL_ID: &str = "p4_friendly";

pub const P7_MODEL_ID: &str = "p7";
pub const P7_LEAGUE_MODEL_ID: &str = "p7_league";
pub const P7_GROUP_STAGE_MODEL_ID: &str = "p7_group_stage";
pub const P7_KNOCKOUT_SINGLE_MODEL_ID: &str = "p7_knockout_90";
pub const P7_KNOCKOUT_TWO_LEG_MODEL_ID: &str = "p7_knockout_two_leg_90";
pub const P7_FRIENDLY_MODEL_ID: &str = "p7_friendly";

const PUBLIC_ENGINE_VERSION: &str = "external-provider";
const PUBLIC_INPUT_SCHEMA_VERSION: &str = "football.external-model-request.v1";
const PUBLIC_OUTPUT_SCHEMA_VERSION: &str = "football.external-model-response.v1";

#[derive(Debug, Clone)]
pub struct PublicModelStub {
    model_id: &'static str,
    display_name: &'static str,
    supported_competitions: Vec<CompetitionKind>,
}

impl PublicModelStub {
    pub fn new(
        model_id: &'static str,
        display_name: &'static str,
        supported_competitions: Vec<CompetitionKind>,
    ) -> Self {
        Self {
            model_id,
            display_name,
            supported_competitions,
        }
    }

    pub fn generic_p4() -> Self {
        Self::new(
            P4_MODEL_ID,
            "P4 外部模型入口",
            CompetitionKind::ALL.to_vec(),
        )
    }

    pub fn built_in_models() -> Vec<Self> {
        vec![
            Self::generic_p4(),
            Self::new(
                P4_LEAGUE_MODEL_ID,
                "P4 联赛外部模型入口",
                vec![CompetitionKind::League],
            ),
            Self::new(
                P4_GROUP_STAGE_MODEL_ID,
                "P4 小组赛外部模型入口",
                vec![CompetitionKind::GroupStage],
            ),
            Self::new(
                P4_KNOCKOUT_SINGLE_MODEL_ID,
                "P4 单回合淘汰赛外部模型入口",
                vec![CompetitionKind::KnockoutSingleLeg],
            ),
            Self::new(
                P4_KNOCKOUT_TWO_LEG_MODEL_ID,
                "P4 两回合淘汰赛外部模型入口",
                vec![CompetitionKind::KnockoutTwoLeg],
            ),
            Self::new(
                P4_FRIENDLY_MODEL_ID,
                "P4 友谊赛外部模型入口",
                vec![CompetitionKind::Friendly],
            ),
            Self::new(
                P7_MODEL_ID,
                "P7 外部模型入口",
                CompetitionKind::ALL.to_vec(),
            ),
            Self::new(
                P7_LEAGUE_MODEL_ID,
                "P7 联赛外部模型入口",
                vec![CompetitionKind::League],
            ),
            Self::new(
                P7_GROUP_STAGE_MODEL_ID,
                "P7 小组赛外部模型入口",
                vec![CompetitionKind::GroupStage],
            ),
            Self::new(
                P7_KNOCKOUT_SINGLE_MODEL_ID,
                "P7 单回合淘汰赛外部模型入口",
                vec![CompetitionKind::KnockoutSingleLeg],
            ),
            Self::new(
                P7_KNOCKOUT_TWO_LEG_MODEL_ID,
                "P7 两回合淘汰赛外部模型入口",
                vec![CompetitionKind::KnockoutTwoLeg],
            ),
            Self::new(
                P7_FRIENDLY_MODEL_ID,
                "P7 友谊赛外部模型入口",
                vec![CompetitionKind::Friendly],
            ),
        ]
    }

    fn unavailable_message(&self) -> String {
        format!(
            "公开源码仅保留模型入口；模型 {} 的运行时未随仓库分发，请接入私有或外部 ModelProvider",
            self.model_id
        )
    }
}

impl PredictionModel for PublicModelStub {
    fn descriptor(&self) -> ModelDescriptor {
        ModelDescriptor {
            model_id: self.model_id.to_string(),
            display_name: self.display_name.to_string(),
            engine_version: PUBLIC_ENGINE_VERSION.to_string(),
            supported_competitions: self.supported_competitions.clone(),
            input_schema_version: PUBLIC_INPUT_SCHEMA_VERSION.to_string(),
            output_schema_version: PUBLIC_OUTPUT_SCHEMA_VERSION.to_string(),
        }
    }

    fn supports(&self, context: &MatchContext) -> bool {
        self.supported_competitions
            .contains(&context.competition_kind)
    }

    fn validate_input(&self, input: &Value) -> ModelResult<()> {
        if input.is_object() {
            Ok(())
        } else {
            Err(ModelError::InvalidInput(
                "公开模型入口要求输入为 JSON 对象".to_string(),
            ))
        }
    }

    fn validate_parameters(&self, parameters: &Value) -> ModelResult<()> {
        if parameters.is_object() {
            Ok(())
        } else {
            Err(ModelError::InvalidParameters(
                "公开模型入口要求参数为 JSON 对象".to_string(),
            ))
        }
    }

    fn predict(&self, _request: &ModelRequest) -> ModelResult<ModelOutput> {
        Err(ModelError::Unavailable(self.unavailable_message()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use football_domain::{MatchContext, ModelIdentity};
    use serde_json::json;

    #[test]
    fn public_stub_exposes_all_expected_entry_ids() {
        let models = PublicModelStub::built_in_models();
        assert_eq!(models.len(), 12);
        assert!(models
            .iter()
            .all(|model| model.descriptor().engine_version == PUBLIC_ENGINE_VERSION));
    }

    #[test]
    fn public_stub_never_executes_a_prediction() {
        let model = PublicModelStub::generic_p4();
        let request = ModelRequest {
            context: MatchContext {
                match_key: "PUBLIC-SHELL".to_string(),
                kickoff_time: Utc
                    .with_ymd_and_hms(2030, 1, 1, 12, 0, 0)
                    .single()
                    .expect("固定时间必须有效"),
                competition_id: None,
                season_id: None,
                stage_id: None,
                competition_kind: CompetitionKind::Custom,
                home_team_name: "Home".to_string(),
                away_team_name: "Away".to_string(),
                metadata: Value::Null,
            },
            identity: ModelIdentity {
                model_id: P4_MODEL_ID.to_string(),
                model_version: PUBLIC_ENGINE_VERSION.to_string(),
                parameter_version: "external-provider".to_string(),
                rule_package_version: None,
            },
            snapshot_type: "T-N".to_string(),
            input: json!({}),
            parameters: json!({}),
        };

        assert!(matches!(
            model.predict(&request),
            Err(ModelError::Unavailable(_))
        ));
    }
}
