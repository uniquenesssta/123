use super::{ApplicationError, ApplicationResult, ApplicationService};
use chrono::Utc;
use football_domain::{
    ReleaseAcceptanceCategorySummary, ReleaseAcceptanceCheck, ReleaseAcceptanceCostSummary,
    ReleaseAcceptancePerformanceSummary, ReleaseAcceptanceRequest, ReleaseAcceptanceRun,
    ReleaseAcceptanceRunSummary, ReleaseAcceptanceRuntimeFacts, ReleaseAcceptanceStatus,
    RELEASE_ACCEPTANCE_CONTRACT_VERSION, RELEASE_ACCEPTANCE_FIXTURE_VERSION,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

impl ApplicationService {
    pub async fn run_release_acceptance(
        &self,
        mut request: ReleaseAcceptanceRequest,
    ) -> ApplicationResult<ReleaseAcceptanceRun> {
        request.performance_window_days = request.performance_window_days.clamp(1, 365);
        request.cost_window_days = request.cost_window_days.clamp(1, 365);
        validate_budget(request.daily_cost_budget_usd, "单日成本预算")?;
        validate_budget(request.monthly_cost_budget_usd, "周期成本预算")?;

        let started_at = Utc::now();
        let run_id = Uuid::new_v4();
        let store = self.active_store().await?;
        let runtime = store
            .release_acceptance_runtime_facts(
                request.performance_window_days,
                request.cost_window_days,
            )
            .await?;
        let mut checks = Vec::new();
        checks.extend(runtime_chain_checks(run_id, &runtime));
        checks.extend(fixed_fixture_checks(run_id)?);
        checks.extend(performance_checks(run_id, &runtime));
        checks.extend(security_checks(run_id, &runtime));
        checks.extend(cost_checks(run_id, &runtime, &request));
        checks.extend(release_checks(run_id, &runtime));
        for (index, check) in checks.iter_mut().enumerate() {
            check.sequence_no = i32::try_from(index + 1).unwrap_or(i32::MAX);
        }

        let passed_count = u32::try_from(
            checks
                .iter()
                .filter(|check| check.status == ReleaseAcceptanceStatus::Pass)
                .count(),
        )
        .unwrap_or(u32::MAX);
        let warning_count = u32::try_from(
            checks
                .iter()
                .filter(|check| check.status == ReleaseAcceptanceStatus::Warning)
                .count(),
        )
        .unwrap_or(u32::MAX);
        let blocked_count = u32::try_from(
            checks
                .iter()
                .filter(|check| check.status == ReleaseAcceptanceStatus::Blocked)
                .count(),
        )
        .unwrap_or(u32::MAX);
        let overall_status = if blocked_count > 0 {
            ReleaseAcceptanceStatus::Blocked
        } else if warning_count > 0 {
            ReleaseAcceptanceStatus::Warning
        } else {
            ReleaseAcceptanceStatus::Pass
        };
        let category_summaries = summarize_categories(&checks);
        let performance = ReleaseAcceptancePerformanceSummary {
            database_latency_ms: u64::try_from(runtime.database_latency_ms).unwrap_or(u64::MAX),
            recent_model_run_count: u64::try_from(runtime.recent_model_run_count)
                .unwrap_or_default(),
            recent_model_run_p95_ms: runtime.recent_model_run_p95_ms,
            recent_model_failure_count: u64::try_from(runtime.recent_model_failure_count)
                .unwrap_or_default(),
            query_warning_count: u64::try_from(runtime.query_warning_count).unwrap_or_default(),
        };
        let cost = ReleaseAcceptanceCostSummary {
            window_days: request.cost_window_days,
            completed_requests: u64::try_from(runtime.completed_requests).unwrap_or_default(),
            failed_requests: u64::try_from(runtime.failed_requests).unwrap_or_default(),
            search_calls: u64::try_from(runtime.search_calls).unwrap_or_default(),
            estimated_cost_usd: runtime.estimated_cost_usd,
            latest_day_cost_usd: runtime.latest_day_cost_usd,
            daily_budget_usd: request.daily_cost_budget_usd,
            monthly_budget_usd: request.monthly_cost_budget_usd,
        };
        let completed_at = Utc::now();
        let report_sha256 = report_hash(&json!({
            "run_id": run_id,
            "app_version": env!("CARGO_PKG_VERSION"),
            "contract_version": RELEASE_ACCEPTANCE_CONTRACT_VERSION,
            "fixture_version": RELEASE_ACCEPTANCE_FIXTURE_VERSION,
            "overall_status": overall_status.as_str(),
            "started_at": started_at,
            "completed_at": completed_at,
            "requested_by": request.requested_by.as_deref(),
            "passed_count": passed_count,
            "warning_count": warning_count,
            "blocked_count": blocked_count,
            "category_summaries": &category_summaries,
            "performance": &performance,
            "cost": &cost,
            "checks": &checks,
        }))?;
        let run = ReleaseAcceptanceRun {
            id: run_id,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            contract_version: RELEASE_ACCEPTANCE_CONTRACT_VERSION.to_string(),
            fixture_version: RELEASE_ACCEPTANCE_FIXTURE_VERSION.to_string(),
            overall_status,
            started_at,
            completed_at,
            requested_by: request.requested_by,
            report_sha256,
            passed_count,
            warning_count,
            blocked_count,
            category_summaries,
            performance,
            cost,
            checks,
        };
        store.persist_release_acceptance_run(&run).await?;
        Ok(run)
    }

    pub async fn list_release_acceptance_runs(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ReleaseAcceptanceRunSummary>> {
        Ok(self
            .active_store()
            .await?
            .list_release_acceptance_runs(limit)
            .await?)
    }

    pub async fn read_release_acceptance_run(
        &self,
        run_id: Uuid,
    ) -> ApplicationResult<ReleaseAcceptanceRun> {
        Ok(self
            .active_store()
            .await?
            .read_release_acceptance_run(run_id)
            .await?)
    }
}

fn validate_budget(value: Option<f64>, label: &str) -> ApplicationResult<()> {
    if value.is_some_and(|number| !number.is_finite() || number < 0.0) {
        return Err(ApplicationError::Validation(format!(
            "{label}必须是大于或等于 0 的有限数值"
        )));
    }
    Ok(())
}

fn check(
    run_id: Uuid,
    category: &str,
    code: &str,
    title: &str,
    status: ReleaseAcceptanceStatus,
    summary: impl Into<String>,
    remediation: Option<&str>,
    evidence: Value,
    duration_ms: i64,
) -> ReleaseAcceptanceCheck {
    ReleaseAcceptanceCheck {
        id: Uuid::new_v4(),
        run_id,
        sequence_no: 0,
        category: category.to_string(),
        check_code: code.to_string(),
        title: title.to_string(),
        status,
        summary: summary.into(),
        remediation: remediation.map(str::to_string),
        evidence,
        duration_ms: duration_ms.max(0),
    }
}

fn runtime_chain_checks(
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
            "chain",
            "integration_contracts_a_to_i",
            "A–I 接入契约",
            ReleaseAcceptanceStatus::Pass,
            "数据库已锁定 A–I 全部接入契约。",
            None,
            json!({"present_stages": facts.integration_stages, "required_stages": required}),
            0,
        )
    } else {
        check(
            run_id,
            "chain",
            "integration_contracts_a_to_i",
            "A–I 接入契约",
            ReleaseAcceptanceStatus::Blocked,
            format!("缺少接入契约：{}。", missing.join("、")),
            Some("重新执行连续数据库迁移，禁止手工跳过历史迁移。"),
            json!({"present_stages": facts.integration_stages, "missing_stages": missing}),
            0,
        )
    };
    let provider_boundary = check(
        run_id,
        "chain",
        "external_model_provider_boundary",
        "外部模型边界",
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
            "chain",
            "database_migrations",
            "数据库迁移连续性",
            ReleaseAcceptanceStatus::Pass,
            format!("已成功应用 {} 条迁移。", facts.migration_count),
            None,
            json!({"migration_count": facts.migration_count, "required": 27}),
            0,
        )
    } else {
        check(
            run_id,
            "chain",
            "database_migrations",
            "数据库迁移连续性",
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
            "chain",
            "runtime_lifecycle_evidence",
            "真实闭环样本可见性",
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

fn fixed_fixture_checks(run_id: Uuid) -> ApplicationResult<Vec<ReleaseAcceptanceCheck>> {
    Ok(vec![
        check(
            run_id,
            "chain",
            "public_model_boundary",
            "公开模型边界",
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
            "chain",
            "external_model_runtime",
            "外部模型运行时",
            ReleaseAcceptanceStatus::Warning,
            "公开仓库保留模型调用入口，但没有可执行的预测引擎。",
            Some("在私有部署中实现并接入 ModelProvider 后，再执行真实模型验收。"),
            json!({
                "provider_required": true,
                "runtime_status": "not_bundled"
            }),
            0,
        ),
    ])
}

fn performance_checks(
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
            "performance",
            "database_latency",
            "数据库往返延迟",
            db_status,
            format!("当前数据库健康检查耗时 {} ms。", facts.database_latency_ms),
            (db_status != ReleaseAcceptanceStatus::Pass)
                .then_some("检查数据库磁盘、网络、连接池和长事务后重新验收。"),
            json!({"latency_ms": facts.database_latency_ms, "warning_threshold_ms": 500, "blocked_threshold_ms": 1500}),
            0,
        ),
        check(
            run_id,
            "performance",
            "model_run_performance",
            "近期推演性能",
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
            "performance",
            "database_query_health",
            "查询健康快照",
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

fn security_checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
) -> Vec<ReleaseAcceptanceCheck> {
    let immutable_status = if facts.immutable_trigger_count >= 8 {
        ReleaseAcceptanceStatus::Pass
    } else {
        ReleaseAcceptanceStatus::Blocked
    };
    vec![
        check(run_id, "security", "immutable_ledgers", "不可变账本触发器", immutable_status,
            format!("关键 schema 中发现 {} 个不可变保护触发器。", facts.immutable_trigger_count),
            (immutable_status == ReleaseAcceptanceStatus::Blocked).then_some("重新执行连续迁移并确认 integration、snapshot、H、I、J 账本触发器存在。"),
            json!({"immutable_trigger_count": facts.immutable_trigger_count, "minimum": 8}), 0),
        check(run_id, "security", "credential_boundary", "API 密钥边界", ReleaseAcceptanceStatus::Pass,
            "发布契约继续要求 API Key 仅由 Rust/Windows 凭据管理器读取，工作区状态白名单拒绝 password、secret 和 credential 字段。", None,
            json!({"frontend_key_storage": false, "workspace_sensitive_field_filter": true}), 0),
    ]
}

fn cost_checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
    request: &ReleaseAcceptanceRequest,
) -> Vec<ReleaseAcceptanceCheck> {
    let daily_exceeded = request
        .daily_cost_budget_usd
        .is_some_and(|budget| facts.latest_day_cost_usd > budget);
    let period_exceeded = request
        .monthly_cost_budget_usd
        .is_some_and(|budget| facts.estimated_cost_usd > budget);
    let status = if daily_exceeded || period_exceeded {
        ReleaseAcceptanceStatus::Blocked
    } else if request.daily_cost_budget_usd.is_none() || request.monthly_cost_budget_usd.is_none() {
        ReleaseAcceptanceStatus::Warning
    } else {
        ReleaseAcceptanceStatus::Pass
    };
    vec![check(
        run_id,
        "cost",
        "openai_cost_observability",
        "OpenAI 成本与预算",
        status,
        if daily_exceeded || period_exceeded {
            "显式成本预算已经超限，发布验收被阻断。".to_string()
        } else if status == ReleaseAcceptanceStatus::Warning {
            format!(
                "{} 日窗口估算成本 ${:.4}，最新有用量日期成本 ${:.4}；至少一个预算未设置。",
                request.cost_window_days, facts.estimated_cost_usd, facts.latest_day_cost_usd
            )
        } else {
            format!(
                "成本在显式预算内：周期 ${:.4}，最新日 ${:.4}。",
                facts.estimated_cost_usd, facts.latest_day_cost_usd
            )
        },
        match status {
            ReleaseAcceptanceStatus::Blocked => Some("降低调用量或调整经批准的预算后重新验收。"),
            ReleaseAcceptanceStatus::Warning => {
                Some("在本页设置单日与周期预算，避免只监控不设闸门。")
            }
            ReleaseAcceptanceStatus::Pass => None,
        },
        json!({
            "window_days": request.cost_window_days,
            "completed_requests": facts.completed_requests,
            "failed_requests": facts.failed_requests,
            "search_calls": facts.search_calls,
            "estimated_cost_usd": facts.estimated_cost_usd,
            "latest_day_cost_usd": facts.latest_day_cost_usd,
            "daily_budget_usd": request.daily_cost_budget_usd,
            "period_budget_usd": request.monthly_cost_budget_usd
        }),
        0,
    )]
}

fn release_checks(
    run_id: Uuid,
    facts: &ReleaseAcceptanceRuntimeFacts,
) -> Vec<ReleaseAcceptanceCheck> {
    let version_ok = env!("CARGO_PKG_VERSION") == "0.23.0";
    let stage_j = facts.integration_stages.iter().any(|stage| stage == "J");
    vec![check(
        run_id,
        "release",
        "release_artifact_contract",
        "发布版本与 J 契约",
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

fn summarize_categories(
    checks: &[ReleaseAcceptanceCheck],
) -> Vec<ReleaseAcceptanceCategorySummary> {
    let mut summaries: BTreeMap<String, ReleaseAcceptanceCategorySummary> = BTreeMap::new();
    for check in checks {
        let summary = summaries.entry(check.category.clone()).or_insert_with(|| {
            ReleaseAcceptanceCategorySummary {
                category: check.category.clone(),
                ..ReleaseAcceptanceCategorySummary::default()
            }
        });
        match check.status {
            ReleaseAcceptanceStatus::Pass => summary.passed += 1,
            ReleaseAcceptanceStatus::Warning => summary.warnings += 1,
            ReleaseAcceptanceStatus::Blocked => summary.blocked += 1,
        }
    }
    summaries.into_values().collect()
}

fn report_hash(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_summary_counts_each_status() {
        let run_id = Uuid::new_v4();
        let checks = vec![
            check(
                run_id,
                "chain",
                "a",
                "a",
                ReleaseAcceptanceStatus::Pass,
                "",
                None,
                Value::Null,
                0,
            ),
            check(
                run_id,
                "chain",
                "b",
                "b",
                ReleaseAcceptanceStatus::Warning,
                "",
                None,
                Value::Null,
                0,
            ),
            check(
                run_id,
                "chain",
                "c",
                "c",
                ReleaseAcceptanceStatus::Blocked,
                "",
                None,
                Value::Null,
                0,
            ),
        ];
        let summary = summarize_categories(&checks);
        assert_eq!(summary[0].passed, 1);
        assert_eq!(summary[0].warnings, 1);
        assert_eq!(summary[0].blocked, 1);
    }
}
