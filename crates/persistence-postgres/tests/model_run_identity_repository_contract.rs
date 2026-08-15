use football_domain::{CompetitionKind, CompetitionProfile, RulePackageDraft, RuleRouting};
use football_model_api::ModelDescriptor;
use football_persistence_postgres::{
    DatabaseOptions, ModelRegistration, PersistenceError, PostgresStore,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
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
                "运行 R5-06 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-06 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-06 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn descriptor(token: &str, engine_version: &str) -> ModelDescriptor {
    ModelDescriptor {
        model_id: format!("r506_model_{token}"),
        display_name: format!("R5-06 Model {token}"),
        engine_version: engine_version.to_string(),
        supported_competitions: vec![CompetitionKind::League],
        input_schema_version: "r5-06-input-1".to_string(),
        output_schema_version: "r5-06-output-1".to_string(),
    }
}

fn rule_package_draft(token: &str, model_id: &str) -> RulePackageDraft {
    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: format!("r506-package-{token}"),
        version: "1.0.0".to_string(),
        display_name: "R5-06 Identity Package".to_string(),
        competition_profile: CompetitionProfile {
            profile_id: format!("r506-profile-{token}"),
            name: "R5-06 League Profile".to_string(),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({"contract": "r5-06"}),
        },
        routing: RuleRouting {
            model_id: model_id.to_string(),
            model_version: "r5-06-model-version-1".to_string(),
            parameter_version: "r5-06-parameters-1".to_string(),
            priority: 51,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-1h".to_string()],
        },
        parameters: json!({"alpha": 0.51, "contract": "r5-06"}),
        feature_requirements: json!({"required": ["lineup"]}),
        output_contract: json!({"schema": "r5-06-output"}),
        source_document: None,
        metadata: json!({"contract": "r5-06"}),
    }
}

async fn insert_run(
    database: &TestDatabase,
    run_id: Uuid,
    match_key: &str,
    registration: &ModelRegistration,
    rule_package_id: Option<Uuid>,
    route_binding_id: Option<Uuid>,
) {
    sqlx::query(
        r#"
        INSERT INTO model.runs (
            id, match_key, model_version_id, parameter_set_id,
            rule_package_id, route_binding_id, snapshot_type, route_reason,
            status, input_payload, output_payload, explanation, summary,
            input_sha256, duration_ms, input_audit_version,
            input_readiness_level, input_readiness_score,
            input_manifest, input_manifest_sha256, completed_at
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, 'T-1h', '{"source":"r5-06-contract"}'::jsonb,
            'succeeded', '{"fixture":"r5-06"}'::jsonb,
            '{"result":"ok"}'::jsonb, '{"explanation":"ok"}'::jsonb,
            '{"home_win":0.51}'::jsonb,
            $7, 17, 'prematch-input-audit-v1',
            'formal_ready', 100, '{"fixture":"r5-06"}'::jsonb, $8, now()
        )
        "#,
    )
    .bind(run_id)
    .bind(match_key)
    .bind(registration.model_version_id)
    .bind(registration.parameter_set_id)
    .bind(rule_package_id)
    .bind(route_binding_id)
    .bind("a".repeat(64))
    .bind("b".repeat(64))
    .execute(&database.pool)
    .await
    .expect("写入 R5-06 model run fixture");
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn model_run_identity_repository_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let descriptor = descriptor(&token, "r5-06-engine-1");
    let parameters = json!({"alpha": 0.51, "contract": "r5-06"});

    let registration: ModelRegistration = database
        .store
        .register_model(
            &descriptor,
            "r5-06-model-version-1",
            "r5-06-parameters-1",
            &parameters,
        )
        .await
        .expect("注册 R5-06 模型 identity");
    let repeated = database
        .store
        .register_model(
            &descriptor,
            "r5-06-model-version-1",
            "r5-06-parameters-1",
            &parameters,
        )
        .await
        .expect("相同模型 identity 重复注册必须保持幂等");
    assert_eq!(repeated.model_version_id, registration.model_version_id);
    assert_eq!(repeated.parameter_set_id, registration.parameter_set_id);

    let incompatible_descriptor = descriptor(&token, "r5-06-engine-2");
    let version_conflict = database
        .store
        .register_model(
            &incompatible_descriptor,
            "r5-06-model-version-1",
            "r5-06-parameters-1",
            &parameters,
        )
        .await
        .expect_err("同模型版本的引擎或 Schema 不一致必须拒绝");
    match version_conflict {
        PersistenceError::InvalidState(message) => assert_eq!(
            message,
            "模型版本 r5-06-model-version-1 已存在但引擎或 Schema 不一致；请创建新模型版本"
        ),
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let parameter_conflict = database
        .store
        .register_model(
            &descriptor,
            "r5-06-model-version-1",
            "r5-06-parameters-1",
            &json!({"alpha": 0.99, "contract": "r5-06"}),
        )
        .await
        .expect_err("同参数版本不同内容必须拒绝");
    match parameter_conflict {
        PersistenceError::InvalidState(message) => assert_eq!(
            message,
            "参数版本 r5-06-parameters-1 已存在但内容不同；请创建新参数版本"
        ),
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let package_draft = rule_package_draft(&token, &descriptor.model_id);
    let package = database
        .store
        .register_rule_package(&descriptor, &package_draft)
        .await
        .expect("规则包事务必须复用 R5-06 model registration owner");

    let binding_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO model.competition_bindings (
            id, competition_kind, model_version_id, parameter_set_id,
            rule_package_id, binding_name, priority, is_active
        ) VALUES ($1, 'league', $2, $3, $4, $5, 51, true)
        "#,
    )
    .bind(binding_id)
    .bind(registration.model_version_id)
    .bind(registration.parameter_set_id)
    .bind(package.id)
    .bind(format!("R5-06 identity binding {token}"))
    .execute(&database.pool)
    .await
    .expect("创建 R5-06 route binding fixture");

    let full_run_id = Uuid::new_v4();
    insert_run(
        &database,
        full_run_id,
        &format!("R5-06-FULL-{token}"),
        &registration,
        Some(package.id),
        Some(binding_id),
    )
    .await;
    let full = database
        .store
        .read_run(full_run_id)
        .await
        .expect("读取完整 model run identity");
    assert_eq!(full["id"], json!(full_run_id));
    assert_eq!(full["model_key"], descriptor.model_id);
    assert_eq!(full["model_version"], "r5-06-model-version-1");
    assert_eq!(full["parameter_version"], "r5-06-parameters-1");
    assert_eq!(full["rule_package_id"], json!(package.id));
    assert_eq!(full["rule_package_key"], package_draft.package_key);
    assert_eq!(full["rule_package_version"], package_draft.version);
    assert_eq!(full["rule_package_name"], package_draft.display_name);
    assert_eq!(full["route_binding_id"], json!(binding_id));

    let nullable_run_id = Uuid::new_v4();
    insert_run(
        &database,
        nullable_run_id,
        &format!("R5-06-NULLABLE-{token}"),
        &registration,
        None,
        None,
    )
    .await;
    let nullable = database
        .store
        .read_run(nullable_run_id)
        .await
        .expect("读取 nullable model run identity");
    assert!(nullable["rule_package_id"].is_null());
    assert!(nullable["rule_package_key"].is_null());
    assert!(nullable["rule_package_version"].is_null());
    assert!(nullable["rule_package_name"].is_null());
    assert!(nullable["route_binding_id"].is_null());

    database.close().await;
}
