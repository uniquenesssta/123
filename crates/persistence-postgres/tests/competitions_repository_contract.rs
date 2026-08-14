use football_domain::{CompetitionDraft, CompetitionKind};
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
                "运行 R5-01 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可清空的 PostgreSQL 测试数据库"
            )
        });
        let options = DatabaseOptions {
            connection_url: connection_url.clone(),
            max_connections: 4,
            connect_timeout_seconds: 10,
        };
        let store = PostgresStore::connect(&options)
            .await
            .expect("连接 R5-01 PostgreSQL 测试数据库");
        store.migrate().await.expect("执行数据库迁移");
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&connection_url)
            .await
            .expect("建立 R5-01 校验连接池");
        Self { store, pool }
    }

    async fn close(self) {
        self.pool.close().await;
        self.store.close().await;
    }
}

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn competition_directory_and_detail_contract_is_preserved() {
    let database = TestDatabase::connect().await;
    let token = Uuid::new_v4().simple().to_string();
    let code = format!("R5-{token}");
    let name = format!("R5-01 competition {token}");

    let created = database
        .store
        .create_competition(&CompetitionDraft {
            code: format!("  {code}  "),
            name: format!("  {name}  "),
            country_code: Some("ZZ".to_string()),
            timezone: "  UTC  ".to_string(),
            competition_kind: CompetitionKind::League,
            metadata: json!({"sort_order": 7, "contract": "r5-01"}),
        })
        .await
        .expect("创建赛事");

    assert_eq!(created.code, code);
    assert_eq!(created.name, name);
    assert_eq!(created.timezone, "UTC");
    assert_eq!(created.country_code.as_deref(), Some("ZZ"));
    assert_eq!(created.competition_kind, CompetitionKind::League);
    assert!(created.is_active);
    assert_eq!(created.metadata["contract"], "r5-01");

    let read = database
        .store
        .read_competition(created.id)
        .await
        .expect("读取赛事详情");
    assert_eq!(read.id, created.id);
    assert_eq!(read.code, code);
    assert_eq!(read.name, name);
    assert_eq!(read.competition_kind, CompetitionKind::League);

    let listed = database
        .store
        .list_competitions()
        .await
        .expect("读取赛事目录");
    assert!(listed.iter().any(|record| record.id == created.id));

    database
        .store
        .delete_competition(created.id)
        .await
        .expect("软删除赛事");

    assert!(database.store.read_competition(created.id).await.is_err());
    let listed_after_delete = database
        .store
        .list_competitions()
        .await
        .expect("读取删除后的赛事目录");
    assert!(!listed_after_delete
        .iter()
        .any(|record| record.id == created.id));

    let row = sqlx::query(
        "SELECT code, is_active, metadata FROM football.competitions WHERE id = $1",
    )
    .bind(created.id)
    .fetch_one(&database.pool)
    .await
    .expect("核对软删除后的数据库事实");
    let deleted_code: String = row.try_get("code").expect("读取软删除 code");
    let is_active: bool = row.try_get("is_active").expect("读取 is_active");
    let metadata: serde_json::Value = row.try_get("metadata").expect("读取 metadata");
    assert!(!is_active);
    assert!(deleted_code.starts_with(&format!("{code}-DELETED-")));
    assert_eq!(metadata["original_code"], code);
    assert!(metadata.get("deleted_at").is_some());

    database.close().await;
}
