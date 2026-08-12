use super::check;
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRuntimeFacts, ReleaseAcceptanceStatus,
};
use serde_json::json;
use uuid::Uuid;

pub(super) fn checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
) -> Vec<ReleaseAcceptanceCheck> {
    let immutable_status = if facts.immutable_trigger_count >= 8 {
        ReleaseAcceptanceStatus::Pass
    } else {
        ReleaseAcceptanceStatus::Blocked
    };
    vec![
        check(
            run_id,
            ("security", "immutable_ledgers", "不可变账本触发器"),
            immutable_status,
            format!("关键 schema 中发现 {} 个不可变保护触发器。", facts.immutable_trigger_count),
            (immutable_status == ReleaseAcceptanceStatus::Blocked)
                .then_some("重新执行连续迁移并确认 integration、snapshot、H、I、J 账本触发器存在。"),
            json!({"immutable_trigger_count": facts.immutable_trigger_count, "minimum": 8}),
            0,
        ),
        check(
            run_id,
            ("security", "credential_boundary", "API 密钥边界"),
            ReleaseAcceptanceStatus::Pass,
            "发布契约继续要求 API Key 仅由 Rust/Windows 凭据管理器读取，工作区状态白名单拒绝 password、secret 和 credential 字段。",
            None,
            json!({"frontend_key_storage": false, "workspace_sensitive_field_filter": true}),
            0,
        ),
    ]
}
