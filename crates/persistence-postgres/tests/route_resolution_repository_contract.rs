use chrono::{Duration, Utc};
use football_domain::{
    CompetitionBindingDraft, CompetitionDraft, CompetitionKind, CompetitionProfile, RouteRequest,
    RouteSource, RulePackageDraft, RuleRouting, SeasonDraft, StageDraft,
};
use football_model_api::ModelDescriptor;
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
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
                "运行 R5-05 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-05 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-05 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn descriptor(token: &str) -> ModelDescriptor {
    ModelDescriptor {
        model_id: format!("p4_r505_{token}"),
        display_name: "R5-05 Route Contract Model".to_string(),
        engine_version: "r5-05-engine-1".to_string(),
        supported_competitions: vec![CompetitionKind::League],
        input_schema_version: "r5-05-input-1".to_string(),
        output_schema_version: "r5-05-output-1".to_string(),
    }
}

fn package_draft(token: &str, descriptor: &ModelDescriptor) -> RulePackageDraft {
    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: format!("r505-package-{token}"),
        version: "1.0.0".to_string(),
        display_name: "R5-05 Route Package".to_string(),
        competition_profile: CompetitionProfile {
            profile_id: format!("r505-profile-{token}"),
            name: "R5-05 League Profile".to_string(),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({"contract": "r5-05", "profile": true}),
        },
        routing: RuleRouting {
            model_id: descriptor.model_id.clone(),
            model_version: "r5-05-model-version-1".to_string(),
            parameter_version: "r5-05-parameters-1".to_string(),
            priority: 41,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-24h".to_string(), "T-1h".to_string()],
        },
        parameters: json!({"contract": "r5-05", "alpha": 0.55}),
        feature_requirements: json!({"required": ["lineup", "availability"]}),
        output_contract: json!({"schema": "r5-05-output"}),
        source_document: None,
        metadata: json!({"contract": "r5-05"}),
    }
}

fn binding_draft(
    package_id: Uuid,
    name: &str,
    competition_id: Option<Uuid>,
    season_id: Option<Uuid>,
    stage_id: Option<Uuid>,
    priority: i32,
) -> CompetitionBindingDraft {
    CompetitionBindingDraft {
        binding_name: Some(name.to_string()),
        competition_id,
        season_id,
        stage_id,
        competition_kind: None,
        rule_package_id: package_id,
        priority,
        valid_from: None,
        valid_to: None,
    }
}

fn automatic_request(
    competition_id: Uuid,
    season_id: Uuid,
    stage_id: Uuid,
    kickoff_time: chrono::DateTime<Utc>,
) -> RouteRequest {
    RouteRequest {
        competition_id: Some(competition_id),
        season_id: Some(season_id),
        stage_id: Some(stage_id),
        competition_kind: CompetitionKind::League,
        kickoff_time,
        preferred_model_family: Some("p4".to_string()),
        preferred_model_id: None,
        explicit_rule_package_id: None,
    }
}

async fn set_binding_active(pool: &PgPool, binding_id: Uuid, active: bool) {
    sqlx::query("UPDATE model.competition_bindings SET is_active = $2 WHERE id = $1")
        .bind(binding_id)
        .bind(active)
        .execute(pool)
        .await
        .expect("切换 R5-05 契约绑定状态");
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn route_resolution_repository_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let now = Utc::now();

    let descriptor = descriptor(&token);
    let package = database
        .store
        .register_rule_package(&descriptor, &package_draft(&token, &descriptor))
        .await
        .expect("注册 R5-05 规则包");

    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("R505-{token}"),
            name: format!("R5-05 Competition {token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"contract": "r5-05"}),
        })
        .await
        .expect("创建 R5-05 赛事");
    let season = database
        .store
        .create_season(&SeasonDraft {
            competition_id: competition.id,
            name: "2026/27".to_string(),
            starts_on: None,
            ends_on: None,
            status: "active".to_string(),
            metadata: json!({"contract": "r5-05"}),
        })
        .await
        .expect("创建 R5-05 赛季");
    let stage = database
        .store
        .create_stage(&StageDraft {
            season_id: season.id,
            code: "LEAGUE".to_string(),
            name: "League Phase".to_string(),
            stage_kind: CompetitionKind::League,
            sequence_no: 1,
            rules: json!({"contract": "r5-05"}),
        })
        .await
        .expect("创建 R5-05 阶段");

    let fallback = database
        .store
        .resolve_competition_context(None, None, None, CompetitionKind::KnockoutTwoLeg)
        .await
        .expect("无层级 ID 时保留 fallback kind");
    assert_eq!(fallback.competition_id, None);
    assert_eq!(fallback.season_id, None);
    assert_eq!(fallback.stage_id, None);
    assert_eq!(fallback.competition_kind, CompetitionKind::KnockoutTwoLeg);

    let competition_context = database
        .store
        .resolve_competition_context(
            Some(competition.id),
            None,
            None,
            CompetitionKind::KnockoutTwoLeg,
        )
        .await
        .expect("赛事 ID 应覆盖 fallback kind");
    assert_eq!(competition_context.competition_id, Some(competition.id));
    assert_eq!(competition_context.season_id, None);
    assert_eq!(competition_context.stage_id, None);
    assert_eq!(competition_context.competition_kind, CompetitionKind::League);

    let season_context = database
        .store
        .resolve_competition_context(
            None,
            Some(season.id),
            None,
            CompetitionKind::KnockoutTwoLeg,
        )
        .await
        .expect("赛季应补全赛事与赛事类型");
    assert_eq!(season_context.competition_id, Some(competition.id));
    assert_eq!(season_context.season_id, Some(season.id));
    assert_eq!(season_context.stage_id, None);
    assert_eq!(season_context.competition_kind, CompetitionKind::League);

    let stage_context = database
        .store
        .resolve_competition_context(
            None,
            None,
            Some(stage.id),
            CompetitionKind::KnockoutTwoLeg,
        )
        .await
        .expect("阶段应补全赛事、赛季与阶段类型");
    assert_eq!(stage_context.competition_id, Some(competition.id));
    assert_eq!(stage_context.season_id, Some(season.id));
    assert_eq!(stage_context.stage_id, Some(stage.id));
    assert_eq!(stage_context.competition_kind, CompetitionKind::League);

    let competition_mismatch = database
        .store
        .resolve_competition_context(
            Some(Uuid::new_v4()),
            Some(season.id),
            None,
            CompetitionKind::League,
        )
        .await
        .expect_err("提交赛事与赛季所属赛事不一致必须拒绝");
    match competition_mismatch {
        PersistenceError::InvalidState(message) => {
            assert!(message.contains("赛事层级不一致"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let season_mismatch = database
        .store
        .resolve_competition_context(
            Some(competition.id),
            Some(Uuid::new_v4()),
            Some(stage.id),
            CompetitionKind::League,
        )
        .await
        .expect_err("提交赛季与阶段所属赛季不一致必须拒绝");
    match season_mismatch {
        PersistenceError::InvalidState(message) => {
            assert!(message.contains("赛季层级不一致"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let default_binding_id = database
        .store
        .ensure_type_default_binding(
            package.id,
            CompetitionKind::League,
            5,
            "R5-05 type default",
        )
        .await
        .expect("创建 R5-05 类型默认绑定");
    let competition_binding = database
        .store
        .create_competition_binding(&binding_draft(
            package.id,
            "R5-05 Competition Binding",
            Some(competition.id),
            None,
            None,
            100,
        ))
        .await
        .expect("创建赛事绑定");
    let season_binding = database
        .store
        .create_competition_binding(&binding_draft(
            package.id,
            "R5-05 Season Binding",
            None,
            Some(season.id),
            None,
            200,
        ))
        .await
        .expect("创建赛季绑定");
    let stage_binding = database
        .store
        .create_competition_binding(&binding_draft(
            package.id,
            "R5-05 Stage Binding",
            None,
            None,
            Some(stage.id),
            1,
        ))
        .await
        .expect("创建阶段绑定");
    let future_stage_binding = database
        .store
        .create_competition_binding(&CompetitionBindingDraft {
            valid_from: Some(now + Duration::days(2)),
            valid_to: Some(now + Duration::days(3)),
            ..binding_draft(
                package.id,
                "R5-05 Future Stage Binding",
                None,
                None,
                Some(stage.id),
                999,
            )
        })
        .await
        .expect("创建未来阶段绑定");

    let request = automatic_request(competition.id, season.id, stage.id, now);
    let stage_route = database
        .store
        .resolve_route(&request)
        .await
        .expect("阶段绑定优先于更高 priority 的较宽范围绑定");
    assert!(matches!(stage_route.source, RouteSource::StageBinding));
    assert_eq!(stage_route.binding_id, Some(stage_binding.id));
    assert_eq!(stage_route.rule_package_id, package.id);
    assert_eq!(stage_route.package_key, package.package_key);
    assert_eq!(stage_route.package_version, package.version);
    assert_eq!(stage_route.package_display_name, package.display_name);
    assert_eq!(stage_route.model_id, descriptor.model_id);
    assert_eq!(stage_route.priority, 1);
    assert_eq!(stage_route.parameters, json!({"contract": "r5-05", "alpha": 0.55}));
    assert_eq!(stage_route.feature_requirements, json!({"required": ["lineup", "availability"]}));
    assert_eq!(stage_route.output_contract, json!({"schema": "r5-05-output"}));
    assert_eq!(stage_route.competition_profile.competition_kind, CompetitionKind::League);
    assert_eq!(stage_route.routing.model_id, descriptor.model_id);
    assert_eq!(stage_route.reason["source"], json!("stage_binding"));
    assert_eq!(stage_route.reason["binding_id"], json!(stage_binding.id));
    assert_eq!(stage_route.reason["rule_package_id"], json!(package.id));
    assert_eq!(stage_route.reason["competition_id"], json!(competition.id));
    assert_eq!(stage_route.reason["season_id"], json!(season.id));
    assert_eq!(stage_route.reason["stage_id"], json!(stage.id));
    assert_eq!(stage_route.reason["competition_kind"], json!(CompetitionKind::League));
    assert_eq!(stage_route.reason["preferred_model_family"], json!("p4"));
    assert_eq!(stage_route.reason["preferred_model_id"], json!(null));
    assert_eq!(stage_route.reason["priority"], json!(1));
    assert_ne!(stage_route.binding_id, Some(future_stage_binding.id));

    set_binding_active(&database.pool, stage_binding.id, false).await;
    let season_route = database
        .store
        .resolve_route(&request)
        .await
        .expect("停用阶段绑定后应回退到赛季绑定");
    assert!(matches!(season_route.source, RouteSource::SeasonBinding));
    assert_eq!(season_route.binding_id, Some(season_binding.id));
    assert_eq!(season_route.priority, 200);

    set_binding_active(&database.pool, season_binding.id, false).await;
    let competition_route = database
        .store
        .resolve_route(&request)
        .await
        .expect("停用赛季绑定后应回退到赛事绑定");
    assert!(matches!(
        competition_route.source,
        RouteSource::CompetitionBinding
    ));
    assert_eq!(competition_route.binding_id, Some(competition_binding.id));
    assert_eq!(competition_route.priority, 100);

    set_binding_active(&database.pool, competition_binding.id, false).await;
    let default_route = database
        .store
        .resolve_route(&request)
        .await
        .expect("停用赛事绑定后应回退到赛事类型默认绑定");
    assert!(matches!(
        default_route.source,
        RouteSource::CompetitionKindDefault
    ));
    assert_eq!(default_route.binding_id, Some(default_binding_id));
    assert_eq!(default_route.priority, 5);

    let exact_request = RouteRequest {
        preferred_model_id: Some(descriptor.model_id.clone()),
        ..request.clone()
    };
    let exact_route = database
        .store
        .resolve_route(&exact_request)
        .await
        .expect("精确模型 ID 过滤应保留匹配路由");
    assert_eq!(exact_route.model_id, descriptor.model_id);

    let missing_model = RouteRequest {
        preferred_model_id: Some(format!("p4_missing_{token}")),
        ..request.clone()
    };
    assert!(matches!(
        database.store.resolve_route(&missing_model).await,
        Err(PersistenceError::RouteNotFound)
    ));

    let wrong_family = RouteRequest {
        preferred_model_family: Some("p7".to_string()),
        ..request.clone()
    };
    assert!(matches!(
        database.store.resolve_route(&wrong_family).await,
        Err(PersistenceError::RouteNotFound)
    ));

    let explicit_request = RouteRequest {
        competition_id: Some(Uuid::new_v4()),
        season_id: Some(Uuid::new_v4()),
        stage_id: Some(Uuid::new_v4()),
        competition_kind: CompetitionKind::KnockoutTwoLeg,
        kickoff_time: now,
        preferred_model_family: Some("p4".to_string()),
        preferred_model_id: Some(descriptor.model_id.clone()),
        explicit_rule_package_id: Some(package.id),
    };
    let explicit_route = database
        .store
        .resolve_route(&explicit_request)
        .await
        .expect("显式规则包必须绕过 Binding scope 并直接解析");
    assert!(matches!(
        explicit_route.source,
        RouteSource::ExplicitRulePackage
    ));
    assert_eq!(explicit_route.binding_id, None);
    assert_eq!(explicit_route.rule_package_id, package.id);
    assert_eq!(explicit_route.priority, 41);
    assert_eq!(explicit_route.reason["source"], json!("explicit_rule_package"));
    assert_eq!(explicit_route.reason["binding_id"], json!(null));
    assert_eq!(explicit_route.reason["competition_id"], json!(explicit_request.competition_id));
    assert_eq!(explicit_route.reason["season_id"], json!(explicit_request.season_id));
    assert_eq!(explicit_route.reason["stage_id"], json!(explicit_request.stage_id));
    assert_eq!(
        explicit_route.reason["competition_kind"],
        json!(CompetitionKind::KnockoutTwoLeg)
    );
    assert_eq!(explicit_route.reason["preferred_model_family"], json!("p4"));
    assert_eq!(
        explicit_route.reason["preferred_model_id"],
        json!(descriptor.model_id)
    );

    let explicit_wrong_family = RouteRequest {
        preferred_model_family: Some("p7".to_string()),
        ..explicit_request.clone()
    };
    assert!(matches!(
        database.store.resolve_route(&explicit_wrong_family).await,
        Err(PersistenceError::RouteNotFound)
    ));

    let missing_explicit = RouteRequest {
        explicit_rule_package_id: Some(Uuid::new_v4()),
        ..explicit_request
    };
    assert!(matches!(
        database.store.resolve_route(&missing_explicit).await,
        Err(PersistenceError::RouteNotFound)
    ));

    sqlx::query("UPDATE football.competitions SET is_active = false WHERE id = $1")
        .bind(competition.id)
        .execute(&database.pool)
        .await
        .expect("停用 R5-05 契约赛事");
    let inactive_stage = database
        .store
        .resolve_competition_context(
            Some(competition.id),
            Some(season.id),
            Some(stage.id),
            CompetitionKind::League,
        )
        .await
        .expect_err("停用赛事后阶段 context 必须拒绝");
    match inactive_stage {
        PersistenceError::InvalidState(message) => {
            assert!(
                message.contains("赛事阶段不存在或所属赛事已停用"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    database.close().await;
}
