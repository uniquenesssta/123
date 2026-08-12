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
    let db_status = if facts.database_latency_ms > 1_500 {
        ReleaseAcceptanceStatus::Blocked
    } else if facts.database_latency_ms > 500 {
        ReleaseAcceptanceStatus::Warning
    } else {
        ReleaseAcceptanceStatus::Pass
    };
    let run_status = match facts.recent_model_run_p95_ms {
        None => ReleaseAcceptanceStatus::Warning,
        Some(value) if value > 5_000.0 => ReleaseAcceptanceStatus::Blocked,
        Some(value) if value > 2_000.0 => ReleaseAcceptanceStatus::Warning,
        Some(_) => ReleaseAcceptanceStatus::Pass,
    };
    let query_status = if facts.query_warning_count > 0 {
        ReleaseAcceptanceStatus::Warning
    } else {
        ReleaseAcceptanceStatus::Pass
    };
    vec![
        check(
            run_id,
            ("performance", "database_latency", "数据库往返延迟"),
            db_status,
            format!("当前数据库健康检查耗时 {} ms。", facts.database_latency_ms),
            (db_status != ReleaseAcceptanceStatus::Pass)
                .then_some("检查数据库磁盘、网络、连接池和长事务后重新验收。"),
            json!({"latency_ms": facts.database_latency_ms, "warning_threshold_ms": 500, "blocked_threshold_ms": 1500}),
            0,
        ),
        check(
            run_id,
            ("performance", "model_run_performance", "近期推演性能"),
            run_status,
            match facts.recent_model_run_p95_ms {
                Some(value) => format!(
                    "近期 {} 次推演的成功运行 P95 为 {:.1} ms，失败 {} 次。",
                    facts.recent_model_run_count, value, facts.recent_model_failure_count
                ),
                None => "当前窗口没有足够的成功推演耗时样本。".to_string(),
            },
            (run_status != ReleaseAcceptanceStatus::Pass)
                .then_some("在目标 Windows + PostgreSQL 环境完成多场推演后重新运行 J 验收。"),
            json!({"run_count": facts.recent_model_run_count, "p95_ms": facts.recent_model_run_p95_ms, "failed": facts.recent_model_failure_count}),
            0,
        ),
        check(
            run_id,
            ("performance", "database_query_health", "查询健康快照"),
            query_status,
            if facts.query_warning_count == 0 {
                "最近查询性能快照没有 warning/critical 项。".to_string()
            } else {
                format!(
                    "最近查询性能快照仍有 {} 项警告。",
                    facts.query_warning_count
                )
            },
            (query_status == ReleaseAcceptanceStatus::Warning)
                .then_some("在分析与历史中查看表级建议，完成索引或 VACUUM/ANALYZE 后重新捕获。"),
            json!({"warning_count": facts.query_warning_count}),
            0,
        ),
    ]
}
