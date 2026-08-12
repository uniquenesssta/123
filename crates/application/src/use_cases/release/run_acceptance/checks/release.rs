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
    let version_ok = env!("CARGO_PKG_VERSION") == "0.23.0";
    let stage_j = facts.integration_stages.iter().any(|stage| stage == "J");
    vec![check(
        run_id,
        ("release", "release_artifact_contract", "发布版本与 J 契约"),
        if version_ok && stage_j {
            ReleaseAcceptanceStatus::Pass
        } else {
            ReleaseAcceptanceStatus::Blocked
        },
        if version_ok && stage_j {
            "应用版本、J 数据库契约和发布验收 schema 已对齐至 0.23.0。"
        } else {
            "应用版本或 J 数据库契约未对齐。"
        },
        (!(version_ok && stage_j))
            .then_some("停止打包；同步 package、Cargo、Tauri、迁移和 J 契约后重新构建。"),
        json!({"app_version": env!("CARGO_PKG_VERSION"), "stage_j_present": stage_j, "migration_count": facts.migration_count}),
        0,
    )]
}
