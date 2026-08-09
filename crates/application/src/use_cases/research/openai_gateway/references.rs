use super::*;

pub(super) fn citation_drafts(
    research_run_id: Uuid,
    response_id: &str,
    citations: &[WebCitation],
    retrieved_at: DateTime<Utc>,
) -> ApplicationResult<Vec<WebCitationDraft>> {
    citations
        .iter()
        .map(|citation| {
            Ok(WebCitationDraft {
                research_run_id,
                response_id: response_id.to_string(),
                url: citation.url.clone(),
                title: citation.title.clone(),
                domain: verified_reference_domain(&citation.url)?,
                output_index: u32::try_from(citation.location.output_index).map_err(|_| {
                    ApplicationError::Validation(
                        "OpenAI引用输出位置超出u32范围，已拒绝持久化".to_string(),
                    )
                })?,
                start_index: optional_u32(citation.location.start_index, "引用起始位置")?,
                end_index: optional_u32(citation.location.end_index, "引用结束位置")?,
                retrieved_at,
            })
        })
        .collect()
}

pub(super) fn source_drafts(
    research_run_id: Uuid,
    response_id: &str,
    sources: &[WebSource],
    retrieved_at: DateTime<Utc>,
) -> ApplicationResult<Vec<WebSourceDraft>> {
    sources
        .iter()
        .map(|source| {
            Ok(WebSourceDraft {
                research_run_id,
                response_id: response_id.to_string(),
                url: source.url.clone(),
                title: source.title.clone(),
                domain: verified_reference_domain(&source.url)?,
                retrieved_at,
            })
        })
        .collect()
}

fn verified_reference_domain(value: &str) -> ApplicationResult<String> {
    let url = url::Url::parse(value).map_err(|_| {
        ApplicationError::Validation(format!("OpenAI来源URL无效，无法保存真实域名：{value}"))
    })?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err(ApplicationError::Validation(format!(
            "OpenAI来源URL必须使用HTTPS且不能包含用户名或密码：{value}"
        )));
    }
    url.host_str()
        .map(|host| host.trim_start_matches("www.").to_lowercase())
        .filter(|host| !host.is_empty())
        .ok_or_else(|| ApplicationError::Validation(format!("OpenAI来源URL缺少域名：{value}")))
}

fn optional_u32(value: Option<usize>, label: &str) -> ApplicationResult<Option<u32>> {
    value
        .map(|value| {
            u32::try_from(value).map_err(|_| {
                ApplicationError::Validation(format!("{label}超出u32范围，已拒绝持久化"))
            })
        })
        .transpose()
}
