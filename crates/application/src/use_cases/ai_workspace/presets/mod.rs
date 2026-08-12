use crate::{ApplicationError, ApplicationResult};
use football_domain::ApiWorkspacePreset;

#[derive(Debug, Clone)]
pub struct ApiWorkspacePresetSpec {
    pub preset: ApiWorkspacePreset,
    pub instructions: String,
}

pub fn specs() -> Vec<ApiWorkspacePresetSpec> {
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

pub fn presets() -> Vec<ApiWorkspacePreset> {
    specs().into_iter().map(|spec| spec.preset).collect()
}

pub fn spec(key: &str) -> ApplicationResult<ApiWorkspacePresetSpec> {
    specs()
        .into_iter()
        .find(|spec| spec.preset.key == key)
        .ok_or_else(|| ApplicationError::Validation(format!("未知API协作预设：{key}")))
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
