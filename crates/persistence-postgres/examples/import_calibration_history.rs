use chrono::{DateTime, Utc};
use football_persistence_postgres::calibration_history::connect_research_database;
use serde_json::{json, Value};
use std::error::Error;
use std::io::Read;
use uuid::Uuid;

async fn run(url: &str, args: &[String]) -> Result<Value, Box<dyn Error>> {
    let store = connect_research_database(url).await?;
    let result = async {
        match args {
            [command] if command == "init" => {
                store.migrate().await?;
                Ok(json!({"status": "RESEARCH_SCHEMA_READY"}))
            }
            [command, flag, path] if command == "import" && flag == "--packet" => {
                let mut bytes = Vec::new();
                std::fs::File::open(path)?
                    .take(4 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)?;
                Ok(store.import_calibration_history(&bytes).await?)
            }
            [command, match_flag, id, cutoff_flag, cutoff]
                if command == "inspect" && match_flag == "--match" && cutoff_flag == "--cutoff" =>
            {
                Ok(store
                    .inspect_calibration_history(
                        Uuid::parse_str(id)?,
                        cutoff.parse::<DateTime<Utc>>()?,
                    )
                    .await?)
            }
            _ => Err(
                "usage: init | import --packet FILE | inspect --match UUID --cutoff RFC3339".into(),
            ),
        }
    }
    .await;
    store.close().await;
    result
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let url = std::env::var("FOOTBALL_CALIBRATION_DATABASE_URL").unwrap_or_default();
    let result = if url.is_empty() {
        Err("缺少 FOOTBALL_CALIBRATION_DATABASE_URL".into())
    } else {
        run(&url, &args).await
    };
    match result {
        Ok(value) => println!("{value}"),
        Err(error) => {
            let message = error.to_string();
            let safe = if url.is_empty() {
                message
            } else {
                message.replace(&url, "[database]")
            };
            eprintln!("{}", json!({"status": "BLOCKED", "reason": safe}));
            std::process::exit(1);
        }
    }
}
