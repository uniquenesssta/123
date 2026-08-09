use super::*;

pub(super) fn audit_fact_time(
    fact: &ResearchFact,
    cutoff: DateTime<Utc>,
    retrieved_at: DateTime<Utc>,
) -> (TimeAuditStatus, String) {
    if retrieved_at > cutoff {
        return (
            TimeAuditStatus::RejectedRetrievedAfterCutoff,
            "API结果在data_cutoff_at之后才取回，已阻止进入对应赛前模型入口".to_string(),
        );
    }
    if matches!(
        fact.verification_state.as_str(),
        "NOT_FOUND" | "NOT_APPLICABLE"
    ) {
        return (
            TimeAuditStatus::AcceptedNonFact,
            "非事实结论不要求来源时间".to_string(),
        );
    }
    if fact
        .timezone
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return (
            TimeAuditStatus::RejectedMissingTimezone,
            "有事实结论但缺少来源时区".to_string(),
        );
    }
    let timestamps = [fact.published_at, fact.observed_at, fact.effective_at];
    if timestamps.iter().all(Option::is_none) {
        return (
            TimeAuditStatus::RejectedMissingEvidenceTime,
            "有事实结论但published_at、observed_at和effective_at均为空".to_string(),
        );
    }
    if timestamps.iter().flatten().any(|value| *value > cutoff) {
        return (
            TimeAuditStatus::RejectedFuture,
            "事实时间晚于data_cutoff_at，已阻止进入赛前模型入口".to_string(),
        );
    }
    if fact
        .published_at
        .into_iter()
        .chain(fact.observed_at)
        .any(|value| value > retrieved_at)
    {
        return (
            TimeAuditStatus::RejectedInvalidOrder,
            "发布时间或观察时间晚于实际抓取时间".to_string(),
        );
    }
    (
        TimeAuditStatus::Accepted,
        "事实时间不晚于data_cutoff_at且具备明确时区".to_string(),
    )
}
