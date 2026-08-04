use super::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    MatchReviewPackagePreview, MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord,
    MatchReviewPackageWorkflowStatus,
};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn register_match_review_package_export(
        &self,
        summary: &MatchReviewPackageSummary,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            UPDATE review.match_review_package_workflows
            SET status = 'superseded', updated_at = now()
            WHERE match_id = $1
              AND status NOT IN ('settled', 'superseded')
            "#,
        )
        .bind(summary.match_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO review.match_review_package_workflows (
                package_id, match_id, match_key, status,
                export_path, export_sha256,
                pre_match_snapshot, export_database_snapshot,
                preview_ready,
                exported_at, updated_at
            ) VALUES ($1,$2,$3,'exported',$4,$5,$6,$7,false,now(),now())
            "#,
        )
        .bind(summary.package_id)
        .bind(summary.match_id)
        .bind(&summary.match_key)
        .bind(&summary.output_path)
        .bind(&summary.content_sha256)
        .bind(serde_json::to_value(&summary.pre_match_snapshot)?)
        .bind(serde_json::to_value(&summary.export_database_snapshot)?)
        .execute(&mut *tx)
        .await?;

        write_audit_event(
            &mut tx,
            "match_review_package_exported",
            "match_review_package",
            Some(summary.package_id.to_string()),
            json!({
                "match_id": summary.match_id,
                "match_key": summary.match_key,
                "export_path": summary.output_path,
                "export_sha256": summary.content_sha256,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_match_review_package_workflow(summary.package_id)
            .await
    }

    pub async fn read_active_match_review_package_workflow(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Option<MatchReviewPackageWorkflowRecord>> {
        let row = sqlx::query(&format!(
            "{} WHERE workflow.match_id=$1 AND workflow.status <> 'superseded' ORDER BY workflow.updated_at DESC LIMIT 1",
            workflow_select_sql()
        ))
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(workflow_from_row).transpose()
    }

    pub async fn read_match_review_package_workflow(
        &self,
        package_id: Uuid,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let row = sqlx::query(&format!(
            "{} WHERE workflow.package_id=$1",
            workflow_select_sql()
        ))
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("赛后复盘资料包工作流不存在".to_string()))?;
        workflow_from_row(&row)
    }

    pub async fn read_match_review_package_workflow_by_review(
        &self,
        review_id: Uuid,
    ) -> PersistenceResult<Option<MatchReviewPackageWorkflowRecord>> {
        let row = sqlx::query(&format!(
            "{} WHERE workflow.review_id=$1 ORDER BY workflow.updated_at DESC LIMIT 1",
            workflow_select_sql()
        ))
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(workflow_from_row).transpose()
    }

    pub async fn record_match_review_package_preview(
        &self,
        package_id: Uuid,
        preview: &MatchReviewPackagePreview,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let status = if preview.ready && preview.errors.is_empty() {
            "preview_valid"
        } else {
            "preview_blocked"
        };
        let payload = serde_json::to_value(preview)?;
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE review.match_review_package_workflows
            SET status=$2,
                import_path=$3,
                import_sha256=$4,
                preview_ready=$5,
                preview_payload=$6,
                previewed_at=now(),
                updated_at=now()
            WHERE package_id=$1
              AND status IN ('exported','preview_blocked','preview_valid')
            RETURNING package_id
            "#,
        )
        .bind(package_id)
        .bind(status)
        .bind(&preview.source_path)
        .bind(&preview.source_sha256)
        .bind(preview.ready && preview.errors.is_empty())
        .bind(payload)
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "当前资料包已进入确认或后续阶段，不能覆盖预检结果".to_string(),
            ));
        }
        self.read_match_review_package_workflow(package_id).await
    }

    pub async fn confirm_match_review_package_workflow(
        &self,
        package_id: Uuid,
        confirmed_by: Option<&str>,
        confirmation_note: Option<&str>,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE review.match_review_package_workflows
            SET status='confirmed',
                confirmed_by=$2,
                confirmation_note=$3,
                confirmed_at=now(),
                updated_at=now()
            WHERE package_id=$1
              AND status='preview_valid'
              AND preview_ready
              AND preview_payload IS NOT NULL
            RETURNING package_id
            "#,
        )
        .bind(package_id)
        .bind(
            confirmed_by
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(
            confirmation_note
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "资料包尚未通过本轮预检，不能人工确认".to_string(),
            ));
        }
        self.read_match_review_package_workflow(package_id).await
    }

    pub async fn mark_match_review_package_facts_committed(
        &self,
        package_id: Uuid,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE review.match_review_package_workflows
            SET status='facts_committed', facts_committed_at=now(), updated_at=now()
            WHERE package_id=$1 AND status='confirmed'
            RETURNING package_id
            "#,
        )
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "资料包尚未人工确认，不能写入赛后事实".to_string(),
            ));
        }
        self.read_match_review_package_workflow(package_id).await
    }

    pub async fn mark_match_review_package_review_created(
        &self,
        package_id: Uuid,
        review_id: Uuid,
    ) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE review.match_review_package_workflows
            SET status='review_created', review_id=$2,
                review_created_at=now(), updated_at=now()
            WHERE package_id=$1 AND status='facts_committed'
            RETURNING package_id
            "#,
        )
        .bind(package_id)
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "赛后事实尚未写入，不能生成正式复盘".to_string(),
            ));
        }
        self.read_match_review_package_workflow(package_id).await
    }

    pub async fn mark_match_review_package_settled(
        &self,
        review_id: Uuid,
    ) -> PersistenceResult<Option<MatchReviewPackageWorkflowRecord>> {
        let package_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE review.match_review_package_workflows
            SET status='settled', settled_at=COALESCE(settled_at, now()), updated_at=now()
            WHERE review_id=$1 AND status IN ('review_created','settled')
            RETURNING package_id
            "#,
        )
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await?;
        match package_id {
            Some(value) => Ok(Some(self.read_match_review_package_workflow(value).await?)),
            None => Ok(None),
        }
    }

    pub async fn read_match_review_package_preview(
        &self,
        package_id: Uuid,
    ) -> PersistenceResult<MatchReviewPackagePreview> {
        let payload: serde_json::Value = sqlx::query_scalar(
            r#"
            SELECT preview_payload
            FROM review.match_review_package_workflows
            WHERE package_id=$1 AND preview_payload IS NOT NULL
            "#,
        )
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("资料包尚无可用预检快照".to_string()))?;
        Ok(serde_json::from_value(payload)?)
    }
}

fn workflow_select_sql() -> &'static str {
    r#"
    SELECT workflow.package_id, workflow.match_id, workflow.match_key,
           workflow.status, workflow.export_path, workflow.export_sha256,
           workflow.pre_match_snapshot, workflow.export_database_snapshot,
           workflow.import_path, workflow.import_sha256, workflow.preview_ready,
           workflow.preview_payload,
           workflow.confirmed_by, workflow.confirmation_note, workflow.review_id,
           workflow.exported_at, workflow.previewed_at, workflow.confirmed_at,
           workflow.facts_committed_at, workflow.review_created_at,
           workflow.settled_at, workflow.updated_at
    FROM review.match_review_package_workflows workflow
    "#
}

fn workflow_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchReviewPackageWorkflowRecord> {
    let status_value: String = row.try_get("status")?;
    let status = MatchReviewPackageWorkflowStatus::parse(&status_value)
        .map_err(PersistenceError::InvalidState)?;
    Ok(MatchReviewPackageWorkflowRecord {
        package_id: row.try_get("package_id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        status,
        completed_steps: Vec::new(),
        allowed_actions: Vec::new(),
        blocking_reasons: Vec::new(),
        next_action: None,
        export_path: row.try_get("export_path")?,
        export_sha256: row.try_get("export_sha256")?,
        pre_match_snapshot: serde_json::from_value(row.try_get("pre_match_snapshot")?)?,
        export_database_snapshot: serde_json::from_value(row.try_get("export_database_snapshot")?)?,
        import_path: row.try_get("import_path")?,
        import_sha256: row.try_get("import_sha256")?,
        preview_ready: row.try_get("preview_ready")?,
        preview: row
            .try_get::<Option<serde_json::Value>, _>("preview_payload")?
            .map(serde_json::from_value)
            .transpose()?,
        confirmed_by: row.try_get("confirmed_by")?,
        confirmation_note: row.try_get("confirmation_note")?,
        review_id: row.try_get("review_id")?,
        exported_at: row.try_get("exported_at")?,
        previewed_at: row.try_get("previewed_at")?,
        confirmed_at: row.try_get("confirmed_at")?,
        facts_committed_at: row.try_get("facts_committed_at")?,
        review_created_at: row.try_get("review_created_at")?,
        settled_at: row.try_get("settled_at")?,
        updated_at: row.try_get("updated_at")?,
    }
    .with_capabilities())
}
