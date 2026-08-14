use chrono::{TimeZone, Utc};
use football_domain::{
    CompetitionKind, CompetitionProfile, RulePackageDraft, RuleRouting, RuleSourceReference,
};
use football_model_api::ModelDescriptor;
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

struct TestDatabase {
    store: PostgresStore,
    pool: PgPool,
}

impl TestDatabase {
    async fn connect() -> Self {
        let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
            panic!(
                "运行 R5-03 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-03 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-03 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn descriptor(token: &str) -> ModelDescriptor {
    ModelDescriptor {
        model_id: format!("r503_model_{token}"),
        display_name: format!("R5-03 Model {token}"),
        engine_version: "r5-03-engine-1".to_string(),
        supported_competitions: vec![CompetitionKind::League],
        input_schema_version: "r5-03-input-1".to_string(),
        output_schema_version: "r5-03-output-1".to_string(),
    }
}

fn source_hash(token: &str) -> String {
    format!("{token}{token}")
}

fn draft(token: &str, model_id: &str, version: &str, source_uri: &str) -> RulePackageDraft {
    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: format!("r503-package-{token}"),
        version: version.to_string(),
        display_name: format!("R5-03 Package {version}"),
        competition_profile: CompetitionProfile {
            profile_id: format!("r503-profile-{token}-{version}"),
            name: format!("R5-03 League Profile {version}"),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({"contract": "r5-03", "version": version}),
        },
        routing: RuleRouting {
            model_id: model_id.to_string(),
            model_version: "r5-03-model-version-1".to_string(),
            parameter_version: "r5-03-parameters-1".to_string(),
            priority: 37,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-24h".to_string(), "T-1h".to_string()],
        },
        parameters: json!({"alpha": 0.37, "contract": "r5-03"}),
        feature_requirements: json!({"required": ["lineup", "availability"]}),
        output_contract: json!({"schema": "r5-03-output-contract"}),
        source_document: Some(RuleSourceReference {
            title: Some(format!("R5-03 Rule Source {version}")),
            source_uri: Some(source_uri.to_string()),
            content_sha256: Some(source_hash(token)),
            notes: Some(format!("contract source {version}")),
        }),
        metadata: json!({"contract": "r5-03", "version": version}),
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn rule_package_repository_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let descriptor = descriptor(&token);
    let first_uri = format!("https://example.invalid/r5-03/{token}/first");
    let second_uri = format!("https://example.invalid/r5-03/{token}/second");

    let first_draft = draft(&token, &descriptor.model_id, "1.0.0", &first_uri);
    let first = database
        .store
        .register_rule_package(&descriptor, &first_draft)
        .await
        .expect("注册第一版规则包");

    assert_eq!(first.format_version, "football.rule-package.v1");
    assert_eq!(first.package_key, first_draft.package_key);
    assert_eq!(first.version, "1.0.0");
    assert_eq!(first.display_name, "R5-03 Package 1.0.0");
    assert_eq!(first.competition_kind, CompetitionKind::League);
    assert_eq!(first.model_id, descriptor.model_id);
    assert_eq!(first.model_version, "r5-03-model-version-1");
    assert_eq!(first.parameter_version, "r5-03-parameters-1");
    assert_eq!(first.priority, 37);
    assert_eq!(first.status, "active");
    assert!(!first.content_sha256.is_empty());

    let repeated = database
        .store
        .register_rule_package(&descriptor, &first_draft)
        .await
        .expect("重复注册同内容规则包应保持幂等");
    assert_eq!(repeated.id, first.id);
    assert_eq!(repeated.content_sha256, first.content_sha256);

    let mut conflicting = first_draft.clone();
    conflicting.display_name = "R5-03 conflicting content".to_string();
    let conflict = database
        .store
        .register_rule_package(&descriptor, &conflicting)
        .await
        .expect_err("同 package_key/version 不同内容必须拒绝");
    match conflict {
        PersistenceError::InvalidState(message) => {
            assert!(
                message.contains("已存在但内容不同"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let second_draft = draft(&token, &descriptor.model_id, "2.0.0", &second_uri);
    let second = database
        .store
        .register_rule_package(&descriptor, &second_draft)
        .await
        .expect("注册第二版规则包");
    assert_ne!(second.id, first.id);

    let first_created_at = Utc
        .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
        .single()
        .expect("构造第一条排序时间");
    let second_created_at = Utc
        .with_ymd_and_hms(2026, 2, 1, 0, 0, 0)
        .single()
        .expect("构造第二条排序时间");
    sqlx::query("UPDATE model.rule_packages SET created_at = $2 WHERE id = $1")
        .bind(first.id)
        .bind(first_created_at)
        .execute(&database.pool)
        .await
        .expect("固定第一条排序时间");
    sqlx::query("UPDATE model.rule_packages SET created_at = $2 WHERE id = $1")
        .bind(second.id)
        .bind(second_created_at)
        .execute(&database.pool)
        .await
        .expect("固定第二条排序时间");

    let packages = database
        .store
        .list_rule_packages()
        .await
        .expect("读取规则包列表");
    let package_versions: Vec<_> = packages
        .iter()
        .filter(|record| record.package_key == first_draft.package_key)
        .collect();
    assert_eq!(package_versions.len(), 2);
    assert_eq!(package_versions[0].id, second.id);
    assert_eq!(package_versions[0].version, "2.0.0");
    assert_eq!(package_versions[0].created_at, second_created_at);
    assert_eq!(package_versions[1].id, first.id);
    assert_eq!(package_versions[1].version, "1.0.0");
    assert_eq!(package_versions[1].created_at, first_created_at);

    let raw = sqlx::query(
        r#"
        SELECT manifest, profile, routing, feature_requirements, output_contract,
               source_document_id, competition_profile_id, status, format_version
        FROM model.rule_packages
        WHERE id = $1
        "#,
    )
    .bind(first.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取规则包原始持久化字段");
    let manifest: Value = raw.try_get("manifest").expect("解析 manifest");
    let profile: Value = raw.try_get("profile").expect("解析 profile");
    let routing: Value = raw.try_get("routing").expect("解析 routing");
    let feature_requirements: Value = raw
        .try_get("feature_requirements")
        .expect("解析 feature_requirements");
    let output_contract: Value = raw
        .try_get("output_contract")
        .expect("解析 output_contract");
    let source_document_id: Option<Uuid> = raw
        .try_get("source_document_id")
        .expect("解析 source_document_id");
    let competition_profile_id: Option<Uuid> = raw
        .try_get("competition_profile_id")
        .expect("解析 competition_profile_id");
    assert_eq!(manifest["metadata"]["contract"], "r5-03");
    assert_eq!(profile["metadata"]["contract"], "r5-03");
    assert_eq!(routing["priority"], 37);
    assert_eq!(feature_requirements["required"][0], "lineup");
    assert_eq!(output_contract["schema"], "r5-03-output-contract");
    assert!(source_document_id.is_some());
    assert!(competition_profile_id.is_some());
    assert_eq!(
        raw.try_get::<String, _>("status").expect("解析 status"),
        "active"
    );
    assert_eq!(
        raw.try_get::<String, _>("format_version")
            .expect("解析 format_version"),
        "football.rule-package.v1"
    );

    let source_row = sqlx::query(
        r#"
        SELECT id, source_type, source_uri, content_sha256, metadata
        FROM catalog.source_documents
        WHERE content_sha256 = $1
        "#,
    )
    .bind(source_hash(&token))
    .fetch_one(&database.pool)
    .await
    .expect("读取去重后的 source document");
    assert_eq!(
        source_row
            .try_get::<String, _>("source_type")
            .expect("解析 source_type"),
        "competition_rule_standard"
    );
    assert_eq!(
        source_row
            .try_get::<Option<String>, _>("source_uri")
            .expect("解析 source_uri")
            .as_deref(),
        Some(first_uri.as_str())
    );
    assert_eq!(
        source_row
            .try_get::<String, _>("content_sha256")
            .expect("解析 content_sha256"),
        source_hash(&token)
    );
    let source_metadata: Value = source_row
        .try_get("metadata")
        .expect("解析 source metadata");
    assert_eq!(source_metadata["package_key"], first_draft.package_key);
    assert_eq!(source_metadata["package_version"], "2.0.0");
    assert_eq!(source_metadata["title"], "R5-03 Rule Source 2.0.0");

    let source_id: Uuid = source_row.try_get("id").expect("解析 source id");
    assert_eq!(source_document_id, Some(source_id));

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit.events WHERE event_type = 'rule_package_registered' AND entity_id = $1",
    )
    .bind(first.id.to_string())
    .fetch_one(&database.pool)
    .await
    .expect("统计第一版规则包审计事件");
    assert_eq!(audit_count, 1, "重复同内容注册不得重复写入注册审计事件");

    database.close().await;
}
