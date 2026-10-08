use chrono::{Duration, Utc};
use football_domain::{
    CompetitionKind, CompetitionProfile, MatchContext, ModelIdentity, PredictionSummary,
    RouteDecision, RouteSource, RuleRouting,
};
use football_model_api::{ModelOutput, ModelRequest};
use football_persistence_postgres::{DatabaseOptions, ModelRegistration, PostgresStore};
use serde_json::json;
use sha2::{Digest, Sha256};
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

    // R8-07: exercise the public save/read/visibility chain in the existing PG target.
    let match_id = Uuid::new_v4();
    let snapshot_id = Uuid::new_v4();
    let match_key = format!("R8-07-RUN-{token}");
    let kickoff = Utc::now() + Duration::hours(2);
    sqlx::query("INSERT INTO football.matches (id, external_key, kickoff_time) VALUES ($1,$2,$3)")
        .bind(match_id)
        .bind(&match_key)
        .bind(kickoff)
        .execute(&database.pool)
        .await
        .expect("创建 R8-07 运行比赛");
    let decision = RouteDecision {
        source: RouteSource::ExplicitRulePackage,
        binding_id: Some(binding_id),
        rule_package_id,
        package_key: package_key.clone(),
        package_version: "1.0.0".into(),
        package_display_name: "R5-06 Identity Package".into(),
        model_id: model_key.clone(),
        model_version_id,
        model_version: "r5-06-model-version-1".into(),
        parameter_set_id,
        parameter_version: "r5-06-parameters-1".into(),
        competition_profile_id: profile_id,
        parameters: json!({"alpha":0.51}),
        routing: RuleRouting {
            model_id: model_key.clone(),
            model_version: "r5-06-model-version-1".into(),
            parameter_version: "r5-06-parameters-1".into(),
            priority: 51,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-1h".into()],
        },
        competition_profile: CompetitionProfile {
            profile_id: format!("r506-profile-{token}"),
            name: "R5-06 League Profile".into(),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({}),
        },
        feature_requirements: json!({}),
        output_contract: json!({}),
        priority: 51,
        reason: json!({"source":"r8-07-contract"}),
    };
    let manifest = json!({"match_key":match_key,"snapshot_type":"T-1h"});
    let manifest_hash = hex::encode(Sha256::digest(serde_json::to_vec(&manifest).unwrap()));
    let request = ModelRequest {
        context: MatchContext {
            match_key: match_key.clone(),
            kickoff_time: kickoff,
            competition_id: None,
            season_id: None,
            stage_id: None,
            competition_kind: CompetitionKind::League,
            home_team_name: "R8-07 Home".into(),
            away_team_name: "R8-07 Away".into(),
            metadata: json!({}),
        },
        identity: ModelIdentity {
            model_id: model_key.clone(),
            model_version: decision.model_version.clone(),
            parameter_version: decision.parameter_version.clone(),
            rule_package_version: Some("1.0.0".into()),
        },
        snapshot_type: "T-1h".into(),
        parameters: decision.parameters.clone(),
        input: json!({"match_id":match_key,"database_match_id":match_id,"feature_snapshot_id":snapshot_id,
            "preparation_version":"r8-07-input", "feature_quality_score":0.9,
            "team_a":{"name":"R8-07 Home"},"team_b":{"name":"R8-07 Away"},
            "snapshot":{"snapshot_id":snapshot_id,"type":"T-1h",
                "data_cutoff_time":kickoff-Duration::hours(1),"frozen_at":kickoff-Duration::minutes(30)},
            "input_audit":{"audit_version":"prematch-input-audit-v1",
                "readiness":{"level":"formal_ready","score":96},
                "manifest":manifest,"manifest_sha256":manifest_hash}}),
    };
    let output = ModelOutput {
        identity: request.identity.clone(),
        summary: PredictionSummary {
            home_win: 0.55,
            draw: 0.25,
            away_win: 0.20,
            btts: None,
            over_2_5: None,
        },
        payload: json!({"modules":{"home_attack":{"raw_score":1.25,"confidence":0.8,
            "effective_score":1.0,"multiplier":1.1,"evidence":"preserved"}},
            "scorelines":[{"goals_a":2,"goals_b":0,"probability":0.55,"rank":1,
                "cumulative_probability":0.55,"route":"home"},
                {"goals_a":0,"goals_b":0,"probability":0.25,"rank":2,"cumulative_probability":0.8}]}),
        explanation: json!({"contract":"r8-07","unaltered":true}),
    };
    for invalid_rank in [true, false] {
        let mut invalid_output = output.clone();
        let field = if invalid_rank { "rank" } else { "probability" };
        invalid_output.payload["scorelines"][1][field] = if invalid_rank {
            json!(32768)
        } else {
            json!(2.0)
        };
        assert!(database
            .store
            .save_successful_run(&decision, &request, &invalid_output, 17)
            .await
            .is_err());
        let counts:(i64,i64,i64,i64,i64)=sqlx::query_as(r#"
            SELECT (SELECT count(*) FROM model.runs WHERE match_key=$1),
                   (SELECT count(*) FROM feature.snapshots WHERE match_key=$1),
                   (SELECT count(*) FROM model.run_modules m JOIN model.runs r ON r.id=m.run_id WHERE r.match_key=$1),
                   (SELECT count(*) FROM model.run_scorelines s JOIN model.runs r ON r.id=s.run_id WHERE r.match_key=$1),
                   (SELECT count(*) FROM audit.events WHERE event_type='model_run_completed' AND payload->>'match_key'=$1)
        "#).bind(&match_key).fetch_one(&database.pool).await.unwrap();
        assert_eq!(
            counts,
            (0, 0, 0, 0, 0),
            "run/details/snapshot/audit must roll back together"
        );
    }
    let first = database
        .store
        .save_successful_run(&decision, &request, &output, 17)
        .await
        .unwrap();
    let second = database
        .store
        .save_successful_run(&decision, &request, &output, 19)
        .await
        .unwrap();
    assert_ne!(
        first, second,
        "successful reruns have distinct run identities"
    );
    let snapshot_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM feature.snapshots WHERE match_key=$1")
            .bind(&match_key)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    assert_eq!(snapshot_count, 1, "identical runtime snapshot is reused");
    let expected_input_hash =
        hex::encode(Sha256::digest(serde_json::to_vec(&request.input).unwrap()));
    for (id, duration) in [(first, 17_i64), (second, 19)] {
        let document = database.store.read_run(id).await.unwrap();
        assert_eq!(document["input"], request.input);
        assert_eq!(document["output"], output.payload);
        assert_eq!(document["explanation"], output.explanation);
        assert_eq!(
            document["summary"],
            serde_json::to_value(&output.summary).unwrap()
        );
        assert_eq!(document["input_sha256"], expected_input_hash);
        assert_eq!(document["input_audit"]["manifest_sha256"], manifest_hash);
        assert_eq!(
            document["input_audit"]["feature_snapshot_id"],
            json!(snapshot_id)
        );
        assert_eq!(document["route_binding_id"], json!(binding_id));
        assert_eq!(document["duration_ms"], json!(duration));
        let module: serde_json::Value = sqlx::query_scalar(
            "SELECT details FROM model.run_modules WHERE run_id=$1 AND module_key='home_attack'",
        )
        .bind(id)
        .fetch_one(&database.pool)
        .await
        .unwrap();
        assert_eq!(module, output.payload["modules"]["home_attack"]);
        let route: String =
            sqlx::query_scalar("SELECT route FROM model.run_scorelines WHERE run_id=$1 AND rank=2")
                .bind(id)
                .fetch_one(&database.pool)
                .await
                .unwrap();
        assert_eq!(route, "未分类");
        let audit_count:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='model_run_completed' AND entity_id=$1")
            .bind(id.to_string()).fetch_one(&database.pool).await.unwrap();
        assert_eq!(audit_count, 1);
    }
    let history = database.store.list_recent_runs(500).await.unwrap();
    let first_item = history.iter().find(|r| r.id == first).unwrap();
    assert_eq!(first_item.top_scoreline.as_deref(), Some("2-0"));
    assert_eq!(first_item.top_scoreline_probability, Some(0.55));
    assert_eq!(first_item.home_team_name.as_deref(), Some("R8-07 Home"));
    assert_eq!(database.store.list_recent_runs(0).await.unwrap().len(), 1);
    database
        .store
        .hide_run_from_history(first, Some("  "))
        .await
        .unwrap();
    let hidden_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT history_hidden_at FROM model.runs WHERE id=$1")
            .bind(first)
            .fetch_one(&database.pool)
            .await
            .unwrap();
    database
        .store
        .hide_run_from_history(first, Some("  R8-07 hidden  "))
        .await
        .unwrap();
    let hidden: (chrono::DateTime<Utc>, String) = sqlx::query_as(
        "SELECT history_hidden_at,history_hidden_reason FROM model.runs WHERE id=$1",
    )
    .bind(first)
    .fetch_one(&database.pool)
    .await
    .unwrap();
    assert_eq!(hidden, (hidden_at, "R8-07 hidden".into()));
    assert!(!database
        .store
        .list_recent_runs(500)
        .await
        .unwrap()
        .iter()
        .any(|r| r.id == first));
    assert_eq!(
        database.store.read_run(first).await.unwrap()["input"],
        request.input
    );
    let hidden_audits:i64=sqlx::query_scalar("SELECT count(*) FROM audit.events WHERE event_type='model_run_history_hidden' AND entity_id=$1")
        .bind(first.to_string()).fetch_one(&database.pool).await.unwrap();
    assert_eq!(
        hidden_audits, 2,
        "each successful visibility request retains its audit"
    );
    assert!(database
        .store
        .hide_run_from_history(Uuid::new_v4(), None)
        .await
        .is_err());
    let mutation = sqlx::query("UPDATE model.runs SET input_payload='{}'::jsonb WHERE id=$1")
        .bind(first)
        .execute(&database.pool)
        .await;
    assert!(
        mutation.is_err(),
        "API-saved input identity remains immutable"
    );

    database.close().await;
}
