use chrono::{DateTime, NaiveDate, Utc};
use football_domain::{
    CompetitionDraft, CompetitionKind, RoundDraft, SeasonDraft, StageDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
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
                "运行 R5-02 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-02 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-02 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("解析测试日期")
}

fn instant(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("解析测试时间")
        .with_timezone(&Utc)
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn season_stage_round_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let competition_name = format!("R5-02 competition {token}");
    let competition = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("R502-{token}"),
            name: competition_name.clone(),
            country_code: Some("ZZ".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"contract": "r5-02"}),
        })
        .await
        .expect("创建契约赛事");

    let first_season = database
        .store
        .create_season(&SeasonDraft {
            competition_id: competition.id,
            name: "  2026/27  ".to_string(),
            starts_on: Some(date("2026-08-01")),
            ends_on: Some(date("2027-05-31")),
            status: "  active  ".to_string(),
            metadata: json!({"seed": 1, "contract": "r5-02"}),
        })
        .await
        .expect("创建第一赛季");
    let second_season = database
        .store
        .create_season(&SeasonDraft {
            competition_id: competition.id,
            name: "  2025/26  ".to_string(),
            starts_on: Some(date("2025-08-01")),
            ends_on: Some(date("2026-05-31")),
            status: "  completed  ".to_string(),
            metadata: json!({"seed": 2}),
        })
        .await
        .expect("创建第二赛季");

    assert_eq!(first_season.competition_id, competition.id);
    assert_eq!(first_season.competition_name, competition_name);
    assert_eq!(first_season.name, "2026/27");
    assert_eq!(first_season.status, "active");
    assert_eq!(first_season.starts_on, Some(date("2026-08-01")));
    assert_eq!(first_season.ends_on, Some(date("2027-05-31")));

    let seasons = database.store.list_seasons().await.expect("读取赛季列表");
    let hierarchy_seasons: Vec<_> = seasons
        .iter()
        .filter(|record| record.competition_id == competition.id)
        .collect();
    assert_eq!(hierarchy_seasons.len(), 2);
    assert_eq!(hierarchy_seasons[0].id, first_season.id);
    assert_eq!(hierarchy_seasons[1].id, second_season.id);

    let season_metadata: serde_json::Value = sqlx::query(
        "SELECT metadata FROM football.seasons WHERE id = $1",
    )
    .bind(first_season.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取赛季 metadata")
    .try_get("metadata")
    .expect("解析赛季 metadata");
    assert_eq!(season_metadata["contract"], "r5-02");

    let first_stage = database
        .store
        .create_stage(&StageDraft {
            season_id: first_season.id,
            code: "  LEAGUE  ".to_string(),
            name: "  League Phase  ".to_string(),
            stage_kind: CompetitionKind::League,
            sequence_no: 1,
            rules: json!({"points_for_win": 3, "contract": "r5-02"}),
        })
        .await
        .expect("创建第一阶段");
    let second_stage = database
        .store
        .create_stage(&StageDraft {
            season_id: first_season.id,
            code: "  PLAYOFF  ".to_string(),
            name: "  Playoff  ".to_string(),
            stage_kind: CompetitionKind::KnockoutTwoLeg,
            sequence_no: 2,
            rules: json!({"legs": 2}),
        })
        .await
        .expect("创建第二阶段");

    assert_eq!(first_stage.season_id, first_season.id);
    assert_eq!(first_stage.season_name, "2026/27");
    assert_eq!(first_stage.competition_id, competition.id);
    assert_eq!(first_stage.competition_name, competition_name);
    assert_eq!(first_stage.code, "LEAGUE");
    assert_eq!(first_stage.name, "League Phase");
    assert_eq!(first_stage.stage_kind, CompetitionKind::League);
    assert_eq!(first_stage.sequence_no, 1);

    let stages = database.store.list_stages().await.expect("读取阶段列表");
    let hierarchy_stages: Vec<_> = stages
        .iter()
        .filter(|record| record.season_id == first_season.id)
        .collect();
    assert_eq!(hierarchy_stages.len(), 2);
    assert_eq!(hierarchy_stages[0].id, first_stage.id);
    assert_eq!(hierarchy_stages[1].id, second_stage.id);

    let stage_rules: serde_json::Value = sqlx::query(
        "SELECT rules FROM football.competition_stages WHERE id = $1",
    )
    .bind(first_stage.id)
    .fetch_one(&database.pool)
    .await
    .expect("读取阶段 rules")
    .try_get("rules")
    .expect("解析阶段 rules");
    assert_eq!(stage_rules["contract"], "r5-02");

    let first_round = database
        .store
        .create_round(&RoundDraft {
            stage_id: first_stage.id,
            code: "  R01  ".to_string(),
            name: "  Round 1  ".to_string(),
            sequence_no: 1,
            starts_at: Some(instant("2026-08-15T12:00:00Z")),
            ends_at: Some(instant("2026-08-17T12:00:00Z")),
        })
        .await
        .expect("创建第一轮次");
    let second_round = database
        .store
        .create_round(&RoundDraft {
            stage_id: first_stage.id,
            code: "  R02  ".to_string(),
            name: "  Round 2  ".to_string(),
            sequence_no: 2,
            starts_at: Some(instant("2026-08-22T12:00:00Z")),
            ends_at: Some(instant("2026-08-24T12:00:00Z")),
        })
        .await
        .expect("创建第二轮次");

    assert_eq!(first_round.stage_id, first_stage.id);
    assert_eq!(first_round.stage_name, "League Phase");
    assert_eq!(first_round.code, "R01");
    assert_eq!(first_round.name, "Round 1");
    assert_eq!(first_round.sequence_no, 1);
    assert_eq!(
        first_round.starts_at,
        Some(instant("2026-08-15T12:00:00Z"))
    );
    assert_eq!(
        first_round.ends_at,
        Some(instant("2026-08-17T12:00:00Z"))
    );

    let rounds = database.store.list_rounds().await.expect("读取轮次列表");
    let hierarchy_rounds: Vec<_> = rounds
        .iter()
        .filter(|record| record.stage_id == first_stage.id)
        .collect();
    assert_eq!(hierarchy_rounds.len(), 2);
    assert_eq!(hierarchy_rounds[0].id, first_round.id);
    assert_eq!(hierarchy_rounds[1].id, second_round.id);

    database.close().await;
}
