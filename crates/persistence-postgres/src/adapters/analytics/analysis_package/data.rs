use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::AiAnalysisPackageData;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn build_ai_analysis_data(&self) -> PersistenceResult<AiAnalysisPackageData> {
        let overview = self.analytics_overview().await?;
        let database_summary = self.database_summary_json().await?;
        let player_review_summary = self.player_review_summary_json().await?;
        let team_review_summary = self.team_review_summary_json().await?;
        let ability_candidates = self.ability_candidates_json().await?;
        let schema_summary = self.schema_summary_json().await?;
        Ok(AiAnalysisPackageData {
            overview,
            database_summary,
            player_review_summary,
            team_review_summary,
            ability_candidates,
            schema_summary,
        })
    }

    async fn database_summary_json(&self) -> PersistenceResult<Value> {
        Ok(sqlx::query_scalar(
            r#"SELECT jsonb_build_object(
                'generated_at',now(),'database_size_bytes',pg_database_size(current_database()),
                'counts',jsonb_build_object(
                    'competitions',(SELECT count(*) FROM football.competitions),
                    'teams',(SELECT count(*) FROM football.teams),
                    'players',(SELECT count(*) FROM football.players),
                    'matches',(SELECT count(*) FROM football.matches),
                    'model_runs',(SELECT count(*) FROM model.runs),
                    'match_reviews',(SELECT count(*) FROM review.match_reviews),
                    'ability_observations',(SELECT count(*) FROM feature.player_ability_observations),
                    'pending_ability_candidates',(SELECT count(*) FROM review.ability_update_candidates WHERE status='pending')
                )
            )"#,
        ).fetch_one(&self.pool).await?)
    }

    async fn player_review_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query(
            r#"SELECT player.id,player.canonical_name,count(*)::bigint sample_size,
                      avg(review.actual_performance)::double precision average_performance,
                      avg(review.realization_ratio)::double precision average_realization,
                      avg(review.confidence)::double precision average_confidence
               FROM review.player_match_reviews review JOIN football.players player ON player.id=review.player_id
               WHERE review.minutes_played >= 15 GROUP BY player.id,player.canonical_name
               ORDER BY avg(review.realization_ratio) DESC NULLS LAST LIMIT 200"#,
        ).fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "player_id": row.try_get::<Uuid,_>("id").ok(),
            "player_name": row.try_get::<String,_>("canonical_name").ok(),
            "sample_size": row.try_get::<i64,_>("sample_size").ok(),
            "average_performance": row.try_get::<Option<f64>,_>("average_performance").ok().flatten(),
            "average_realization": row.try_get::<Option<f64>,_>("average_realization").ok().flatten(),
            "average_confidence": row.try_get::<Option<f64>,_>("average_confidence").ok().flatten(),
        })).collect()))
    }

    async fn team_review_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query(
            r#"SELECT team.id,team.canonical_name,count(*)::bigint sample_size,
                      avg(review.chemistry_score)::double precision chemistry,
                      avg(review.bench_strength)::double precision bench_strength,
                      avg(review.substitution_impact)::double precision substitution_impact,
                      avg(review.realization_score)::double precision realization
               FROM review.team_match_reviews review JOIN football.teams team ON team.id=review.team_id
               GROUP BY team.id,team.canonical_name ORDER BY count(*) DESC LIMIT 200"#,
        ).fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "team_id": row.try_get::<Uuid,_>("id").ok(),
            "team_name": row.try_get::<String,_>("canonical_name").ok(),
            "sample_size": row.try_get::<i64,_>("sample_size").ok(),
            "chemistry": row.try_get::<Option<f64>,_>("chemistry").ok().flatten(),
            "bench_strength": row.try_get::<Option<f64>,_>("bench_strength").ok().flatten(),
            "substitution_impact": row.try_get::<Option<f64>,_>("substitution_impact").ok().flatten(),
            "realization": row.try_get::<Option<f64>,_>("realization").ok().flatten(),
        })).collect()))
    }

    async fn ability_candidates_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query("SELECT candidate.id,candidate.player_id,player.canonical_name,candidate.dimension_code,candidate.current_value,candidate.proposed_value,candidate.confidence,candidate.sample_size,candidate.status,candidate.evidence,candidate.created_at FROM review.ability_update_candidates candidate JOIN football.players player ON player.id=candidate.player_id ORDER BY candidate.created_at DESC LIMIT 1000")
            .fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({
            "id": row.try_get::<Uuid,_>("id").ok(),"player_id": row.try_get::<Uuid,_>("player_id").ok(),
            "player_name": row.try_get::<String,_>("canonical_name").ok(),"dimension_code": row.try_get::<String,_>("dimension_code").ok(),
            "current_value": row.try_get::<Option<f64>,_>("current_value").ok().flatten(),"proposed_value": row.try_get::<f64,_>("proposed_value").ok(),
            "confidence": row.try_get::<f64,_>("confidence").ok(),"sample_size": row.try_get::<i32,_>("sample_size").ok(),
            "status": row.try_get::<String,_>("status").ok(),"evidence": row.try_get::<Value,_>("evidence").ok(),"created_at": row.try_get::<DateTime<Utc>,_>("created_at").ok()
        })).collect()))
    }

    async fn schema_summary_json(&self) -> PersistenceResult<Value> {
        let rows = sqlx::query("SELECT table_schema,table_name,count(*)::bigint column_count FROM information_schema.columns WHERE table_schema IN ('platform','catalog','football','feature','model','review','analytics','audit') GROUP BY table_schema,table_name ORDER BY table_schema,table_name")
            .fetch_all(&self.pool).await?;
        Ok(Value::Array(rows.iter().map(|row| json!({"schema":row.try_get::<String,_>("table_schema").ok(),"table":row.try_get::<String,_>("table_name").ok(),"columns":row.try_get::<i64,_>("column_count").ok()})).collect()))
    }
}
