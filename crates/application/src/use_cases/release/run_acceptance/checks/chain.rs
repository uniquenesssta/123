use super::check;
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRuntimeFacts, ReleaseAcceptanceStatus,
};
use serde_json::json;
use uuid::Uuid;

pub(super) fn runtime_checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
) -> Vec<ReleaseAcceptanceCheck> {
    let required: Vec<String> = ('A'..='I').map(|value| value.to_string()).collect();
    let missing: Vec<String> = required
        .iter()
        .filter(|stage| !facts.integration_stages.contains(stage))
        .cloned()
        .collect();
    let contracts = if missing.is_empty() {
        check(
            run_id,
            ("chain", "integration_contracts_a_to_i", "A–I 接入契约"),
            ReleaseAcceptanceStatus::Pass,
            "数据库已锁定 A–I 全部接入契约。",
            None,
            json!({"present_stages": facts.integration_stages, "required_stages": required}),
            0,
        )
    } else {
        check(
            run_id,
            ("chain", "integration_contracts_a_to_i", "A–I 接入契约"),
            ReleaseAcceptanceStatus::Blocked,
            format!("缺少接入契约：{}。", missing.join("、")),
            Some("重新执行连续数据库迁移，禁止手工跳过历史迁移。"),
            json!({"present_stages": facts.integration_stages, "missing_stages": missing}),
            0,
        )
    };
    let provider_boundary = check(
        run_id,
        ("chain", "external_model_provider_boundary", "外部模型边界"),
        ReleaseAcceptanceStatus::Pass,
        "公开源码只注册外部模型入口，不包含预测引擎、参数或固定回归资产。",
        None,
        json!({
            "bundled_runtime": false,
            "provider_kind": "external",
            "entry_contract": "football.external-model-response.v1"
        }),
        0,
    );
    let migration = if facts.migration_count >= 27 {
        check(
            run_id,
            ("chain", "database_migrations", "数据库迁移连续性"),
            ReleaseAcceptanceStatus::Pass,
            format!("已成功应用 {} 条迁移。", facts.migration_count),
            None,
            json!({"migration_count": facts.migration_count, "required": 27}),
            0,
        )
    } else {
        check(
            run_id,
            ("chain", "database_migrations", "数据库迁移连续性"),
            ReleaseAcceptanceStatus::Blocked,
            format!(
                "只发现 {} 条成功迁移，最低要求为 27 条。",
                facts.migration_count
            ),
            Some("连接当前客户端并完成连续数据库迁移。"),
            json!({"migration_count": facts.migration_count, "required": 27}),
            0,
        )
    };
    let sample_total = facts.freeze_task_count
        + facts.frozen_snapshot_count
        + facts.settlement_count
        + facts.evidence_decision_count
        + facts.shadow_validation_count
        + facts.promotion_decision_count;
    let lifecycle = if facts.frozen_snapshot_count > 0 && facts.settlement_count > 0 {
        ReleaseAcceptanceStatus::Pass
    } else {
        ReleaseAcceptanceStatus::Warning
    };
    vec![
        contracts,
        provider_boundary,
        migration,
        check(
            run_id,
            ("chain", "runtime_lifecycle_evidence", "真实闭环样本可见性"),
            lifecycle,
            if lifecycle == ReleaseAcceptanceStatus::Pass {
                "数据库中已经存在冻结快照和正式结算，可执行真实闭环复核。".to_string()
            } else {
                "公开外壳可完成结构验收，但真实模型运行样本仍取决于外部 Provider。".to_string()
            },
            (lifecycle == ReleaseAcceptanceStatus::Warning).then_some(
                "接入外部 ModelProvider 后积累真实运行与结算样本；不得使用合成样本替代真实统计。",
            ),
            json!({
                "freeze_tasks": facts.freeze_task_count,
                "frozen_snapshots": facts.frozen_snapshot_count,
                "settlements": facts.settlement_count,
                "evidence_decisions": facts.evidence_decision_count,
                "shadow_validations": facts.shadow_validation_count,
                "promotion_decisions": facts.promotion_decision_count,
                "total_visible_records": sample_total
            }),
            0,
        ),
    ]
}

pub(super) fn fixture_checks(run_id: Uuid) -> Vec<ReleaseAcceptanceCheck> {
    vec![
        check(
            run_id,
            ("chain", "public_model_boundary", "公开模型边界"),
            ReleaseAcceptanceStatus::Pass,
            "模型源码、参数、Profile、固定比赛和回归制品未随公开仓库分发。",
            None,
            json!({
                "bundled_runtime": false,
                "bundled_parameters": false,
                "bundled_fixtures": false
            }),
            0,
        ),
        check(
            run_id,
            ("chain", "external_model_runtime", "外部模型运行时"),
            ReleaseAcceptanceStatus::Warning,
            "公开仓库保留模型调用入口，但没有可执行的预测引擎。",
            Some("在私有部署中实现并接入 ModelProvider 后，再执行真实模型验收。"),
            json!({
                "provider_required": true,
                "runtime_status": "not_bundled"
            }),
            0,
        ),
    ]
}
