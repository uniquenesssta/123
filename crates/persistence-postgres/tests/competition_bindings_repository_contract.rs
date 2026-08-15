use chrono::{Duration, Utc};
use football_domain::{
    CompetitionBindingDraft, CompetitionDraft, CompetitionKind, CompetitionProfile, RouteRequest,
    RouteSource, RulePackageDraft, RuleRouting, SeasonDraft, StageDraft,
};
use football_model_api::ModelDescriptor;
use football_persistence_postgres::{DatabaseOptions, PersistenceError, PostgresStore};
use serde_json::json;
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
                "运行 R5-04 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-04 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-04 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn descriptor(token: &str, kind: CompetitionKind) -> ModelDescriptor {
    ModelDescriptor {
        model_id: format!("r504_model_{token}_{}", kind.as_str()),
        display_name: format!("R5-04 Model {}", kind.as_str()),
        engine_version: "r5-04-engine-1".to_string(),
        supported_competitions: vec![kind],
        input_schema_version: "r5-04-input-1".to_string(),
        output_schema_version: "r5-04-output-1".to_string(),
    }
}

fn package_draft(
    token: &str,
    descriptor: &ModelDescriptor,
    kind: CompetitionKind,
    version: &str,
) -> RulePackageDraft {
    RulePackageDraft {
        format_version: "football.rule-package.v1".to_string(),
        package_key: format!("r504-package-{token}-{}", kind.as_str()),
        version: version.to_string(),
        display_name: format!("R5-04 Package {} {version}", kind.as_str()),
        competition_profile: CompetitionProfile {
            profile_id: format!("r504-profile-{token}-{}-{version}", kind.as_str()),
            name: format!("R5-04 Profile {} {version}", kind.as_str()),
            competition_kind: kind,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({"contract": "r5-04", "kind": kind.as_str()}),
        },
        routing: RuleRouting {
            model_id: descriptor.model_id.clone(),
            model_version: "r5-04-model-version-1".to_string(),
            parameter_version: "r5-04-parameters-1".to_string(),
            priority: 41,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-24h".to_string(), "T-1h".to_string()],
        },
        parameters: json!({"contract": "r5-04", "alpha": 0.41}),
        feature_requirements: json!({"required": ["lineup"]}),
        output_contract: json!({"schema": "r5-04-output"}),
        source_document: None,
        metadata: json!({"contract": "r5-04"}),
    }
}

fn binding_draft(
    package_id: Uuid,
    name: Option<&str>,
    competition_id: Option<Uuid>,
    season_id: Option<Uuid>,
    stage_id: Option<Uuid>,
    competition_kind: Option<CompetitionKind>,
    priority: i32,
) -> CompetitionBindingDraft {
    CompetitionBindingDraft {
        binding_name: name.map(str::to_string),
        competition_id,
        season_id,
        stage_id,
        competition_kind,
        rule_package_id: package_id,
        priority,
        valid_from: None,
        valid_to: None,
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn competition_binding_repository_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();

    let league_descriptor = descriptor(&token, CompetitionKind::League);
    let league_package = database
        .store
        .register_rule_package(
            &league_descriptor,
            &package_draft(&token, &league_descriptor, CompetitionKind::League, "1.0.0"),
        )
        .await
        .expect("注册 League 规则包");

    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("R504-{token}"),
            name: format!("R5-04 Competition {token}"),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"contract": "r5-04"}),
        })
        .await
        .expect("创建 R5-04 赛事");
    let season = database
        .store
        .create_season(&SeasonDraft {
            competition_id: competition.id,
            name: "2026/27".to_string(),
            starts_on: None,
            ends_on: None,
            status: "active".to_string(),
            metadata: json!({"contract": "r5-04"}),
        })
        .await
        .expect("创建 R5-04 赛季");
    let stage = database
        .store
        .create_stage(&StageDraft {
            season_id: season.id,
            code: "LEAGUE".to_string(),
            name: "League Phase".to_string(),
            stage_kind: CompetitionKind::League,
            sequence_no: 1,
            rules: json!({"contract": "r5-04"}),
        })
        .await
        .expect("创建 R5-04 阶段");

    let empty_scope = database
        .store
        .create_competition_binding(&binding_draft(
            league_package.id,
            Some("empty"),
            None,
            None,
            None,
            None,
            1,
        ))
        .await
        .expect_err("空绑定范围必须拒绝");
    match empty_scope {
        PersistenceError::InvalidState(message) => {
            assert!(message.contains("绑定范围不能为空"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let now = Utc::now();
    let invalid_dates = CompetitionBindingDraft {
        valid_from: Some(now + Duration::hours(2)),
        valid_to: Some(now + Duration::hours(1)),
        ..binding_draft(
            league_package.id,
            Some("invalid-dates"),
            Some(competition.id),
            None,
            None,
            None,
            1,
        )
    };
    let invalid_dates_error = database
        .store
        .create_competition_binding(&invalid_dates)
        .await
        .expect_err("结束时间早于开始时间必须拒绝");
    assert!(matches!(invalid_dates_error, PersistenceError::InvalidState(_)));

    let kind_mismatch = database
        .store
        .create_competition_binding(&binding_draft(
            league_package.id,
            Some("kind-mismatch"),
            None,
            None,
            Some(stage.id),
            Some(CompetitionKind::KnockoutTwoLeg),
            1,
        ))
        .await
        .expect_err("显式赛事类型与阶段解析类型不一致必须拒绝");
    match kind_mismatch {
        PersistenceError::InvalidState(message) => {
            assert!(message.contains("与赛事层级解析结果"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let knockout_descriptor = descriptor(&token, CompetitionKind::KnockoutTwoLeg);
    let knockout_package = database
        .store
        .register_rule_package(
            &knockout_descriptor,
            &package_draft(
                &token,
                &knockout_descriptor,
                CompetitionKind::KnockoutTwoLeg,
                "1.0.0",
            ),
        )
        .await
        .expect("注册 Knockout 规则包");
    let package_mismatch = database
        .store
        .create_competition_binding(&binding_draft(
            knockout_package.id,
            Some("package-mismatch"),
            Some(competition.id),
            None,
            None,
            None,
            1,
        ))
        .await
        .expect_err("不同赛事类型规则包必须拒绝绑定");
    match package_mismatch {
        PersistenceError::InvalidState(message) => {
            assert!(message.contains("不能绑定到"), "unexpected message: {message}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }

    let default_id = database
        .store
        .ensure_type_default_binding(
            league_package.id,
            CompetitionKind::League,
            5,
            "R5-04 type default",
        )
        .await
        .expect("创建类型默认绑定");
    let repeated_default_id = database
        .store
        .ensure_type_default_binding(
            league_package.id,
            CompetitionKind::League,
            99,
            "R5-04 different ignored name",
        )
        .await
        .expect("重复类型默认绑定应幂等");
    assert_eq!(repeated_default_id, default_id);

    let competition_binding = database
        .store
        .create_competition_binding(&binding_draft(
            league_package.id,
            Some("  Competition Binding  "),
            Some(competition.id),
            None,
            None,
            None,
            10,
        ))
        .await
        .expect("创建赛事绑定");
    assert_eq!(competition_binding.binding_name, "  Competition Binding  ");
    assert_eq!(competition_binding.competition_id, Some(competition.id));
    assert_eq!(competition_binding.competition_name.as_deref(), Some(competition.name.as_str()));
    assert_eq!(competition_binding.season_id, None);
    assert_eq!(competition_binding.stage_id, None);
    assert_eq!(competition_binding.competition_kind, Some(CompetitionKind::League));
    assert_eq!(competition_binding.rule_package_id, league_package.id);
    assert_eq!(competition_binding.rule_package_name, league_package.display_name);
    assert_eq!(competition_binding.model_id, league_descriptor.model_id);
    assert_eq!(competition_binding.priority, 10);
    assert!(competition_binding.is_active);

    let season_binding = database
        .store
        .create_competition_binding(&binding_draft(
            league_package.id,
            None,
            None,
            Some(season.id),
            None,
            None,
            20,
        ))
        .await
        .expect("创建赛季绑定");
    assert_eq!(season_binding.competition_id, Some(competition.id));
    assert_eq!(season_binding.season_id, Some(season.id));
    assert_eq!(season_binding.stage_id, None);
    assert_eq!(season_binding.competition_kind, Some(CompetitionKind::League));
    assert!(season_binding.binding_name.starts_with("赛事规则绑定-"));

    let stage_binding = database
        .store
        .create_competition_binding(&binding_draft(
            league_package.id,
            Some("Stage Binding"),
            None,
            None,
            Some(stage.id),
            None,
            30,
        ))
        .await
        .expect("创建阶段绑定");
    assert_eq!(stage_binding.competition_id, Some(competition.id));
    assert_eq!(stage_binding.season_id, Some(season.id));
    assert_eq!(stage_binding.stage_id, Some(stage.id));
    assert_eq!(stage_binding.competition_kind, Some(CompetitionKind::League));

    let future_binding = database
        .store
        .create_competition_binding(&CompetitionBindingDraft {
            valid_from: Some(now + Duration::days(2)),
            valid_to: Some(now + Duration::days(3)),
            ..binding_draft(
                league_package.id,
                Some("Future Binding"),
                Some(competition.id),
                None,
                None,
                None,
                100,
            )
        })
        .await
        .expect("创建未来有效绑定");
    assert_eq!(future_binding.binding_name, "Future Binding");

    let bindings = database
        .store
        .list_competition_bindings()
        .await
        .expect("读取有效绑定列表");
    let relevant: Vec<_> = bindings
        .iter()
        .filter(|record| {
            record.id == default_id
                || record.id == competition_binding.id
                || record.id == season_binding.id
                || record.id == stage_binding.id
                || record.id == future_binding.id
        })
        .collect();
    assert_eq!(relevant.len(), 4, "未来有效绑定不得出现在当前有效列表");
    assert_eq!(relevant[0].id, stage_binding.id);
    assert_eq!(relevant[1].id, season_binding.id);
    assert_eq!(relevant[2].id, competition_binding.id);
    assert_eq!(relevant[3].id, default_id);

    let default_row = sqlx::query(
        r#"
        SELECT binding_name, competition_id, season_id, stage_id, competition_kind,
               rule_package_id, priority, is_active
        FROM model.competition_bindings
        WHERE id = $1
        "#,
    )
    .bind(default_id)
    .fetch_one(&database.pool)
    .await
    .expect("读取类型默认绑定原始字段");
    assert_eq!(default_row.try_get::<String, _>("binding_name").unwrap(), "R5-04 type default");
    assert_eq!(default_row.try_get::<Option<Uuid>, _>("competition_id").unwrap(), None);
    assert_eq!(default_row.try_get::<Option<Uuid>, _>("season_id").unwrap(), None);
    assert_eq!(default_row.try_get::<Option<Uuid>, _>("stage_id").unwrap(), None);
    assert_eq!(default_row.try_get::<Option<String>, _>("competition_kind").unwrap().as_deref(), Some("league"));
    assert_eq!(default_row.try_get::<Uuid, _>("rule_package_id").unwrap(), league_package.id);
    assert_eq!(default_row.try_get::<i32, _>("priority").unwrap(), 5);
    assert!(default_row.try_get::<bool, _>("is_active").unwrap());

    let default_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit.events WHERE event_type = 'type_default_binding_created' AND entity_id = $1",
    )
    .bind(default_id.to_string())
    .fetch_one(&database.pool)
    .await
    .expect("统计类型默认绑定审计事件");
    assert_eq!(default_audit_count, 1, "默认绑定重复调用不得重复审计");

    for id in [competition_binding.id, season_binding.id, stage_binding.id, future_binding.id] {
        let audit_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit.events WHERE event_type = 'competition_binding_created' AND entity_id = $1",
        )
        .bind(id.to_string())
        .fetch_one(&database.pool)
        .await
        .expect("统计赛事绑定审计事件");
        assert_eq!(audit_count, 1);
    }

    let route = database
        .store
        .resolve_route(&RouteRequest {
            competition_id: Some(competition.id),
            season_id: Some(season.id),
            stage_id: Some(stage.id),
            competition_kind: CompetitionKind::League,
            kickoff_time: now,
            preferred_model_family: None,
            preferred_model_id: None,
            explicit_rule_package_id: None,
        })
        .await
        .expect("按既有优先级解析阶段绑定");
    assert!(matches!(route.source, RouteSource::StageBinding));
    assert_eq!(route.binding_id, Some(stage_binding.id));
    assert_eq!(route.rule_package_id, league_package.id);
    assert_eq!(route.priority, 30);

    database.close().await;
}
