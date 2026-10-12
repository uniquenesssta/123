use super::*;
use crate::ports::{
    ai_workspace::ApiWorkspaceSessionPort, research::ResearchGatewayAuditPort, PortError,
    PortErrorKind, PortResult,
};
use crate::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use football_domain::{
    ApiWorkspaceAttachment, ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft,
    ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    OpenAiAttemptDraft, OpenAiAttemptRecord, OpenAiUsageTotals, WebCitationDraft, WebSourceDraft,
};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use uuid::Uuid;

#[derive(Clone, Default)]
struct SessionFixture {
    calls: Arc<Mutex<Vec<Value>>>,
}
impl SessionFixture {
    fn fail<T>(&self, method: &str, value: Value) -> PortResult<T> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"method":method,"value":value}));
        Err(PortError::new(
            PortErrorKind::Infrastructure,
            "fixture persistence failure",
        ))
    }
}
#[async_trait]
impl ApiWorkspaceSessionPort for SessionFixture {
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals> {
        self.calls.lock().unwrap().push(json!("workspace_usage"));
        Ok(OpenAiUsageTotals {
            today_cost_usd: 2.0,
            month_cost_usd: 3.0,
            today_request_count: 2,
            month_request_count: 8,
        })
    }
    async fn create_session(
        &self,
        draft: &ApiWorkspaceSessionDraft,
    ) -> PortResult<ApiWorkspaceSessionRecord> {
        self.fail("create", json!(draft))
    }
    async fn list_sessions(&self, limit: u32) -> PortResult<Vec<ApiWorkspaceSessionRecord>> {
        self.fail("list", json!(limit))
    }
    async fn read_session(&self, id: Uuid) -> PortResult<ApiWorkspaceSessionDetail> {
        self.fail("read", json!(id))
    }
    async fn archive_session(&self, id: Uuid) -> PortResult<()> {
        self.fail("archive", json!(id))
    }
    async fn append_message(
        &self,
        draft: &ApiWorkspaceMessageDraft,
    ) -> PortResult<ApiWorkspaceMessageRecord> {
        self.fail("message", json!(draft))
    }
    async fn append_assistant_bundle(
        &self,
        message: &ApiWorkspaceMessageDraft,
        operations: &[ApiWorkspaceOperationDraft],
        files: &[ApiWorkspaceGeneratedFileDraft],
    ) -> PortResult<ApiWorkspaceSessionDetail> {
        self.fail(
            "bundle",
            json!({"message":message,"operations":operations,"files":files}),
        )
    }
    async fn read_generated_file(&self, id: Uuid) -> PortResult<ApiWorkspaceGeneratedFileContent> {
        self.fail("file", json!(id))
    }
}
#[async_trait]
impl ResearchGatewayAuditPort for SessionFixture {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord> {
        self.fail("unexpected_attempt", json!(draft))
    }
    async fn attempt_number_offset(&self, id: Uuid) -> PortResult<u32> {
        self.fail("unexpected_offset", json!(id))
    }
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals> {
        self.calls.lock().unwrap().push(json!("formal_usage"));
        Ok(OpenAiUsageTotals {
            today_cost_usd: 1.0,
            month_cost_usd: 2.0,
            today_request_count: u64::MAX,
            month_request_count: 7,
        })
    }
    async fn append_web_references(
        &self,
        id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()> {
        self.fail(
            "unexpected_web",
            json!({"id":id,"sources":sources,"citations":citations}),
        )
    }
}
fn draft(preset: &str) -> ApiWorkspaceSessionDraft {
    ApiWorkspaceSessionDraft {
        profile_id: " profile ".into(),
        preset_key: preset.into(),
        title: " title ".into(),
        match_id: None,
        metadata: json!({"保留":true}),
    }
}
fn assert_port_failure<T: std::fmt::Debug>(result: ApplicationResult<T>) {
    let error = result.expect_err("fixture failure is propagated");
    assert!(
        matches!(error, ApplicationError::Port(error) if error.kind==PortErrorKind::Infrastructure && error.message=="fixture persistence failure")
    );
}

#[tokio::test]
async fn session_validation_precedes_database_factory_polling() {
    let polls = Arc::new(AtomicUsize::new(0));
    for preset in ["match_research", "unknown"] {
        let count = polls.clone();
        let factory = async move {
            count.fetch_add(1, Ordering::SeqCst);
            Ok::<_, ApplicationError>(SessionFixture::default())
        };
        let error = create_session(factory, draft(preset))
            .await
            .expect_err("invalid preset/match");
        assert!(matches!(error, ApplicationError::Validation(_)));
    }
    let count = polls.clone();
    let factory = async move {
        count.fetch_add(1, Ordering::SeqCst);
        Ok::<_, ApplicationError>(SessionFixture::default())
    };
    let error = append_user_message(factory, Uuid::nil(), " \n ".into(), &[])
        .await
        .expect_err("empty user message");
    assert!(
        matches!(error,ApplicationError::Validation(message) if message=="请输入问题或选择附件")
    );
    assert_eq!(polls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn user_message_keeps_trimmed_text_metadata_and_persistence_error() {
    let fixture = SessionFixture::default();
    let attachment = ApiWorkspaceAttachment {
        name: "资料.txt".into(),
        media_type: "text/plain".into(),
        content: "正文不进入元数据".into(),
        content_sha256: "hash".into(),
        original_size_bytes: 9,
        truncated: true,
    };
    for text in [" 足球⚽\n ", " \n "] {
        assert_port_failure(
            append_user_message(
                async { Ok(fixture.clone()) },
                Uuid::nil(),
                text.into(),
                std::slice::from_ref(&attachment),
            )
            .await,
        );
    }
    let calls = fixture.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0]["value"]["content"], "足球⚽");
    assert_eq!(calls[1]["value"]["content"], "");
    assert_eq!(calls[0]["value"]["role"], "user");
    assert_eq!(calls[0]["value"]["session_id"], json!(Uuid::nil()));
    assert_eq!(
        calls[0]["value"]["attachments"],
        json!([{"name":"资料.txt","media_type":"text/plain","content_sha256":"hash","original_size_bytes":9,"truncated":true}])
    );
    assert_eq!(calls[0]["value"]["citations"], json!([]));
    assert_eq!(calls[0]["value"]["structured_payload"], json!({}));
    assert_eq!(calls[0]["value"]["token_usage"], json!({}));
    assert!(calls[0]["value"]["provider_response_id"].is_null());
}

#[tokio::test]
async fn session_and_bundle_ports_keep_identity_payload_order_and_errors() {
    let fixture = SessionFixture::default();
    let id = Uuid::from_u128(9);
    let create = draft("plain_chat");
    assert_port_failure(create_session(async { Ok(fixture.clone()) }, create.clone()).await);
    assert_port_failure(list_sessions(async { Ok(fixture.clone()) }, 0).await);
    assert_port_failure(read_session(async { Ok(fixture.clone()) }, id).await);
    assert_port_failure(archive_session(async { Ok(fixture.clone()) }, id).await);
    assert_port_failure(read_generated_file(async { Ok(fixture.clone()) }, id).await);
    let message = ApiWorkspaceMessageDraft {
        session_id: id,
        role: "assistant".into(),
        content: " 原样回答 ".into(),
        structured_payload: json!({"answer":"保留"}),
        citations: json!([{"url":"https://example.com/a#b"}]),
        attachments: json!([]),
        provider_response_id: Some("response".into()),
        model_id: Some("model".into()),
        token_usage: json!({"estimated_cost_usd":0.25}),
    };
    let operations = vec![ApiWorkspaceOperationDraft {
        session_id: id,
        message_id: Uuid::nil(),
        proposal_key: " key ".into(),
        operation_type: "add_team_name".into(),
        payload: json!({"team":"保留"}),
        rationale: " 原样 ".into(),
        confidence: 0.5,
        idempotency_key: " idem ".into(),
    }];
    let files = vec![ApiWorkspaceGeneratedFileDraft {
        session_id: id,
        message_id: Uuid::nil(),
        filename: "output.txt".into(),
        media_type: "text/plain".into(),
        content: "文件原文".into(),
        content_sha256: "provided-hash".into(),
    }];
    assert_port_failure(
        append_assistant_bundle(
            async { Ok(fixture.clone()) },
            message.clone(),
            operations.clone(),
            files.clone(),
        )
        .await,
    );
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        vec![
            json!({"method":"create","value":create}),
            json!({"method":"list","value":0}),
            json!({"method":"read","value":id}),
            json!({"method":"archive","value":id}),
            json!({"method":"file","value":id}),
            json!({"method":"bundle","value":{"message":message,"operations":operations,"files":files}})
        ]
    );
}

#[tokio::test]
async fn usage_totals_keep_formal_then_workspace_order_and_saturating_counts() {
    let fixture = SessionFixture::default();
    let total = usage_totals(async { Ok(fixture.clone()) }).await.unwrap();
    assert_eq!(total.today_cost_usd, 3.0);
    assert_eq!(total.month_cost_usd, 5.0);
    assert_eq!(total.today_request_count, u64::MAX);
    assert_eq!(total.month_request_count, 15);
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        vec![json!("formal_usage"), json!("workspace_usage")]
    );
}
