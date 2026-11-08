use chrono::Utc;
use football_domain::MatchStatus;
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use std::error::Error;
use std::str::FromStr;
use uuid::Uuid;

async fn capture() -> Result<serde_json::Value, Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 || args[0] != "--match" || args[2] != "--horizon" {
        return Err("usage: --match UUID --horizon T-N|T-24h|T-6h|T-1h".into());
    }
    let match_id = Uuid::parse_str(&args[1])?;
    let horizon = &args[3];
    if !matches!(horizon.as_str(), "T-N" | "T-24h" | "T-6h" | "T-1h") {
        return Err("unsupported calibration horizon".into());
    }
    let url = std::env::var("FOOTBALL_CALIBRATION_DATABASE_URL")?;
    let parsed = sqlx::postgres::PgConnectOptions::from_str(&url)?;
    let database_is_test = parsed.get_database().is_some_and(|name| {
        ["test", "acceptance", "qa"]
            .iter()
            .any(|s| name.to_ascii_lowercase().contains(s))
    });
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url,
        max_connections: 2,
        connect_timeout_seconds: 10,
    })
    .await?;
    let result = async {
        let started = Utc::now();
        let record = store.read_match(match_id).await?;
        if record.kickoff_time <= started || !matches!(record.status, MatchStatus::Scheduled) {
            return Err("calibration capture requires a scheduled future match".into());
        }
        let prepared = store
            .prepare_match_prediction_input_at(match_id, horizon, "p4", started)
            .await?;
        let completed = Utc::now();
        if completed >= prepared.match_record.kickoff_time {
            return Err("kickoff passed while collecting features; no snapshot emitted".into());
        }
        let producer = prepared
            .match_input
            .get("preparation_version")
            .cloned()
            .ok_or("input producer version is missing")?;
        Ok(json!({
            "schema": "football.initial-calibration.producer.v1",
            "producer": producer,
            "captured_at": started.to_rfc3339(),
            "completed_at": completed.to_rfc3339(),
            "database_is_test": database_is_test,
            "prepared": prepared,
        }))
    }
    .await;
    store.close().await;
    result
}

#[tokio::main]
async fn main() {
    match capture().await {
        Ok(value) => println!("{value}"),
        Err(error) => {
            let message = error.to_string();
            let safe = std::env::var("FOOTBALL_CALIBRATION_DATABASE_URL")
                .map_or(message.clone(), |url| message.replace(&url, "[database]"));
            eprintln!("{safe}");
            std::process::exit(1);
        }
    }
}
