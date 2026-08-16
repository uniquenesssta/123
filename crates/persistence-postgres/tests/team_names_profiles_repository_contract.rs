use chrono::NaiveDate;
use football_domain::{TeamDraft, TeamNameDraft, TeamProfileDraft};
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
                "运行 R6-02 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R6-02 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R6-02 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn profile_draft() -> TeamProfileDraft {
    TeamProfileDraft {
        short_name: Some("  Alpha  ".to_string()),
        team_type: " club ".to_string(),
        founded_year: Some(1999),
        city: Some("  Test City  ".to_string()),
        stadium: Some("  Test Stadium  ".to_string()),
        head_coach: Some("Ignored Draft Coach".to_string()),
        default_formation: Some("  4-3-3  ".to_string()),
        tactical_style: " balanced ".to_string(),
        attack_rating: Some(71.0),
        midfield_rating: Some(72.0),
        defence_rating: Some(73.0),
        goalkeeper_rating: Some(74.0),
        reputation: Some(75.0),
        data_confidence: 0.88,
        notes: Some("  first note  ".to_string()),
        metadata: json!({"first": true, "shared": "old", "source": "r6-02-contract"}),
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn team_names_and_profiles_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let team = database
        .store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-02 Team {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r6-02"}),
        })
        .await
        .expect("创建 R6-02 contract team");

    let valid_from = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let valid_to = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
    let alias = database
        .store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: "  Galaxy   Club  ".to_string(),
            language_code: Some("  zh-CN  ".to_string()),
            valid_from: Some(valid_from),
            valid_to: Some(valid_to),
        })
        .await
        .expect("写入球队历史名称");
    assert_eq!(alias.team_id, team.id);
    assert_eq!(alias.name, "Galaxy   Club");
    assert_eq!(alias.normalized_name, "galaxy club");
    assert_eq!(alias.language_code.as_deref(), Some("zh-CN"));
    assert_eq!(alias.valid_from, Some(valid_from));
    assert_eq!(alias.valid_to, Some(valid_to));

    let no_language = database
        .store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: format!("Alias {token}"),
            language_code: Some("   ".to_string()),
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect("空白 language code 应按既有语义存为 null");
    assert_eq!(no_language.language_code, None);

    let blank_error = database
        .store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: "   ".to_string(),
            language_code: None,
            valid_from: None,
            valid_to: None,
        })
        .await
        .expect_err("空白球队别名必须失败");
    assert!(matches!(
        blank_error,
        PersistenceError::InvalidState(message) if message == "球队别名不能为空"
    ));

    let reversed_error = database
        .store
        .add_team_name(&TeamNameDraft {
            team_id: team.id,
            name: "Historic Alias".to_string(),
            language_code: None,
            valid_from: Some(valid_to),
            valid_to: Some(valid_from),
        })
        .await
        .expect_err("倒置历史名称有效期必须失败");
    assert!(matches!(
        reversed_error,
        PersistenceError::InvalidState(message) if message == "球队别名结束日期早于开始日期"
    ));

    let first_profile = database
        .store
        .upsert_team_profile(team.id, &profile_draft())
        .await
        .expect("首次写入球队 profile");
    assert_eq!(first_profile.short_name.as_deref(), Some("Alpha"));
    assert_eq!(first_profile.team_type, "club");
    assert_eq!(first_profile.city.as_deref(), Some("Test City"));
    assert_eq!(first_profile.stadium.as_deref(), Some("Test Stadium"));
    assert_eq!(first_profile.default_formation.as_deref(), Some("4-3-3"));
    assert_eq!(first_profile.tactical_style, "balanced");
    assert_eq!(first_profile.notes.as_deref(), Some("first note"));
    assert_eq!(first_profile.head_coach, None);
    assert_eq!(first_profile.metadata["first"], true);

    sqlx::query(
        r#"
        UPDATE football.team_profiles
        SET head_coach = 'Persisted Coach', metadata = metadata || '{"db_only":true}'::jsonb
        WHERE team_id = $1
        "#,
    )
    .bind(team.id)
    .execute(&database.pool)
    .await
    .expect("准备 profile 合并与 coach 保留事实");

    let mut second_draft = profile_draft();
    second_draft.short_name = Some("  Beta  ".to_string());
    second_draft.city = Some("   ".to_string());
    second_draft.notes = None;
    second_draft.metadata = json!({"second": true, "shared": "new"});
    let second_profile = database
        .store
        .upsert_team_profile(team.id, &second_draft)
        .await
        .expect("更新球队 profile");
    assert_eq!(second_profile.short_name.as_deref(), Some("Beta"));
    assert_eq!(second_profile.city, None);
    assert_eq!(
        second_profile.head_coach.as_deref(),
        Some("Persisted Coach")
    );
    assert_eq!(second_profile.metadata["first"], true);
    assert_eq!(second_profile.metadata["db_only"], true);
    assert_eq!(second_profile.metadata["second"], true);
    assert_eq!(second_profile.metadata["shared"], "new");

    let mut invalid_type = profile_draft();
    invalid_type.team_type = "invalid".to_string();
    let error = database
        .store
        .upsert_team_profile(team.id, &invalid_type)
        .await
        .expect_err("非法球队类型必须失败");
    assert!(matches!(
        error,
        PersistenceError::InvalidState(message) if message == "球队类型无效"
    ));

    let mut invalid_rating = profile_draft();
    invalid_rating.attack_rating = Some(101.0);
    let error = database
        .store
        .upsert_team_profile(team.id, &invalid_rating)
        .await
        .expect_err("越界评分必须失败");
    assert!(matches!(
        error,
        PersistenceError::InvalidState(message) if message == "进攻评分必须在0到100之间"
    ));

    let mut invalid_confidence = profile_draft();
    invalid_confidence.data_confidence = -0.01;
    let error = database
        .store
        .upsert_team_profile(team.id, &invalid_confidence)
        .await
        .expect_err("越界可信度必须失败");
    assert!(matches!(
        error,
        PersistenceError::InvalidState(message) if message == "球队资料可信度必须在0到1之间"
    ));

    database.close().await;
}
