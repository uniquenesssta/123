use football_persistence_postgres::{DatabaseOptions, ModelRegistration, PostgresStore};
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
    let definition_id = Uuid::new_v4();
    let model_version_id = Uuid::new_v4();
    let parameter_set_id = Uuid::new_v4();
    let profile_id = Uuid::new_v4();
    let rule_package_id = Uuid::new_v4();
    let binding_id = Uuid::new_v4();
    let model_key = format!("r506_model_{token}");
    let package_key = format!("r506-package-{token}");

    sqlx::query("INSERT INTO model.definitions (id, model_key, display_name) VALUES ($1, $2, $3)")
        .bind(definition_id)
        .bind(&model_key)
        .bind("R5-06 Model")
        .execute(&database.pool)
        .await
        .expect("创建 R5-06 model definition");
    sqlx::query(
        r#"
        INSERT INTO model.versions (
            id, model_id, version, engine_version,
            input_schema_version, output_schema_version, status
        ) VALUES ($1, $2, 'r5-06-model-version-1', 'r5-06-engine-1',
                  'r5-06-input-1', 'r5-06-output-1', 'active')
        "#,
    )
    .bind(model_version_id)
    .bind(definition_id)
    .execute(&database.pool)
    .await
    .expect("创建 R5-06 model version");
    sqlx::query(
        r#"
        INSERT INTO model.parameter_sets (
            id, model_version_id, parameter_version, name,
            definition, definition_sha256, status
        ) VALUES ($1, $2, 'r5-06-parameters-1', 'R5-06 parameters',
                  '{"alpha":0.51}'::jsonb, $3, 'active')
        "#,
    )
    .bind(parameter_set_id)
    .bind(model_version_id)
    .bind("c".repeat(64))
    .execute(&database.pool)
    .await
    .expect("创建 R5-06 parameter set");

    let registration = ModelRegistration {
        model_version_id,
        parameter_set_id,
    };
    assert_eq!(registration.model_version_id, model_version_id);
    assert_eq!(registration.parameter_set_id, parameter_set_id);

    sqlx::query(
        r#"
        INSERT INTO model.competition_profiles (
            id, profile_key, version, name, competition_kind,
            definition, definition_sha256, metadata
        ) VALUES ($1, $2, '1.0.0', 'R5-06 League Profile', 'league',
                  '{"contract":"r5-06"}'::jsonb, $3, '{"contract":"r5-06"}'::jsonb)
        "#,
    )
    .bind(profile_id)
    .bind(format!("r506-profile-{token}"))
    .bind("d".repeat(64))
    .execute(&database.pool)
    .await
    .expect("创建 R5-06 competition profile");
    sqlx::query(
        r#"
        INSERT INTO model.rule_packages (
            id, package_key, version, display_name, competition_kind,
            content_sha256, manifest, profile, routing,
            feature_requirements, output_contract,
            model_version_id, parameter_set_id, priority, format_version,
            competition_profile_id, status
        ) VALUES (
            $1, $2, '1.0.0', 'R5-06 Identity Package', 'league',
            $3, '{"contract":"r5-06"}'::jsonb, '{"contract":"r5-06"}'::jsonb,
            '{"contract":"r5-06"}'::jsonb, '{}'::jsonb, '{}'::jsonb,
            $4, $5, 51, 'football.rule-package.v1', $6, 'active'
        )
        "#,
    )
    .bind(rule_package_id)
    .bind(&package_key)
    .bind("e".repeat(64))
    .bind(model_version_id)
    .bind(parameter_set_id)
    .bind(profile_id)
    .execute(&database.pool)
    .await
    .expect("创建 R5-06 rule package");
    sqlx::query(
        r#"
        INSERT INTO model.competition_bindings (
            id, competition_kind, model_version_id, parameter_set_id,
            rule_package_id, binding_name, priority, is_active
        ) VALUES ($1, 'league', $2, $3, $4, $5, 51, true)
        "#,
    )
    .bind(binding_id)
    .bind(model_version_id)
    .bind(parameter_set_id)
    .bind(rule_package_id)
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
        Some(rule_package_id),
        Some(binding_id),
    )
    .await;
    let full = database
        .store
        .read_run(full_run_id)
        .await
        .expect("读取完整 model run identity");
    assert_eq!(full["id"], json!(full_run_id));
    assert_eq!(full["model_key"], model_key);
    assert_eq!(full["model_version"], "r5-06-model-version-1");
    assert_eq!(full["parameter_version"], "r5-06-parameters-1");
    assert_eq!(full["rule_package_id"], json!(rule_package_id));
    assert_eq!(full["rule_package_key"], package_key);
    assert_eq!(full["rule_package_version"], "1.0.0");
    assert_eq!(full["rule_package_name"], "R5-06 Identity Package");
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
