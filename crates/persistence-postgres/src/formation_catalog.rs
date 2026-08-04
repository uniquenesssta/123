use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageEntryRecord, FormationUsageListQuery,
    ResolvedFormationDistribution,
};
use serde_json::json;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

const UNKNOWN_FORMATION_ID: Uuid = Uuid::from_u128(0x076720d204f05b3bad4787f0bfe290bd);

struct FormationUsageGroupKey<'a> {
    scope_type: &'a str,
    team_id: Option<Uuid>,
    coach_id: Option<Uuid>,
    competition_id: Option<Uuid>,
    window_start: NaiveDate,
    window_end: NaiveDate,
    observed_at: DateTime<Utc>,
}

impl PostgresStore {
    pub async fn list_formations(
        &self,
        active_only: bool,
    ) -> PersistenceResult<Vec<FormationRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, name, line_structure, slot_definition,
                   is_builtin, is_active, sort_order, metadata
            FROM football.formations
            WHERE NOT $1 OR is_active
            ORDER BY sort_order, code, id
            "#,
        )
        .bind(active_only)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(formation_from_row).collect()
    }

    pub async fn save_formation_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PersistenceResult<FormationUsageDistributionRecord> {
        validate_distribution_draft(draft)?;
        let (window_start, window_end) = self.resolve_formation_window(draft).await?;

        let mut counts: HashMap<Uuid, i32> = HashMap::new();
        for entry in &draft.entries {
            if entry.usage_count < 0 || entry.usage_count > draft.observed_matches {
                return Err(PersistenceError::InvalidState(
                    "阵型使用次数必须位于 0 到观察场数之间".to_string(),
                ));
            }
            if counts
                .insert(entry.formation_id, entry.usage_count)
                .is_some()
            {
                return Err(PersistenceError::InvalidState(
                    "同一阵型在一个观察窗口中不能重复".to_string(),
                ));
            }
        }

        let formation_ids: Vec<Uuid> = counts.keys().copied().collect();
        if !formation_ids.is_empty() {
            let valid_count: i64 = sqlx::query_scalar(
                "SELECT count(*)::bigint FROM football.formations WHERE id = ANY($1) AND is_active",
            )
            .bind(&formation_ids)
            .fetch_one(&self.pool)
            .await?;
            if valid_count != formation_ids.len() as i64 {
                return Err(PersistenceError::InvalidState(
                    "阵型目录中存在无效或已停用的阵型".to_string(),
                ));
            }
        }

        let total: i32 = counts.values().sum();
        if total > draft.observed_matches {
            return Err(PersistenceError::InvalidState(
                "阵型使用次数合计不能超过观察场数".to_string(),
            ));
        }
        let missing = draft.observed_matches - total;
        if draft.observed_matches == 0 {
            counts.clear();
            counts.insert(UNKNOWN_FORMATION_ID, 0);
        } else if missing > 0 {
            *counts.entry(UNKNOWN_FORMATION_ID).or_insert(0) += missing;
        }
        counts.retain(|formation_id, count| *count > 0 || *formation_id == UNKNOWN_FORMATION_ID);
        let probabilities = calculate_probabilities(&counts, draft.observed_matches, draft.alpha)?;
        let observed_at = Utc::now();
        let mut tx = self.pool.begin().await?;

        for (formation_id, usage_count) in counts {
            let (raw_probability, smoothed_probability) =
                probabilities.get(&formation_id).copied().ok_or_else(|| {
                    PersistenceError::InvalidState("阵型概率计算结果缺失".to_string())
                })?;
            sqlx::query(
                r#"
                INSERT INTO feature.formation_usage_observations (
                    id, scope_type, team_id, coach_id, competition_id, formation_id,
                    window_preset, window_start, window_end, observed_matches,
                    usage_count, raw_probability, smoothed_probability, confidence,
                    smoothing_alpha, source_document_id, observed_at, metadata
                ) VALUES (
                    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(draft.scope_type.trim())
            .bind(draft.team_id)
            .bind(draft.coach_id)
            .bind(draft.competition_id)
            .bind(formation_id)
            .bind(draft.window_preset.trim())
            .bind(window_start)
            .bind(window_end)
            .bind(draft.observed_matches)
            .bind(usage_count)
            .bind(raw_probability)
            .bind(smoothed_probability)
            .bind(draft.confidence)
            .bind(draft.alpha)
            .bind(draft.source_document_id)
            .bind(observed_at)
            .bind(&draft.metadata)
            .execute(&mut *tx)
            .await?;
        }
        write_audit_event(
            &mut tx,
            "formation_usage_saved",
            "formation_usage",
            draft
                .team_id
                .or(draft.coach_id)
                .or(draft.competition_id)
                .map(|id| id.to_string()),
            json!({
                "scope_type": draft.scope_type,
                "team_id": draft.team_id,
                "coach_id": draft.coach_id,
                "competition_id": draft.competition_id,
                "window_start": window_start,
                "window_end": window_end,
                "observed_matches": draft.observed_matches,
                "alpha": draft.alpha,
            }),
        )
        .await?;
        tx.commit().await?;

        self.read_exact_distribution(&FormationUsageGroupKey {
            scope_type: draft.scope_type.trim(),
            team_id: draft.team_id,
            coach_id: draft.coach_id,
            competition_id: draft.competition_id,
            window_start,
            window_end,
            observed_at,
        })
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵型概率保存后无法读取".to_string()))
    }

    pub async fn list_formation_usage_distributions(
        &self,
        query: &FormationUsageListQuery,
    ) -> PersistenceResult<Vec<FormationUsageDistributionRecord>> {
        let rows = sqlx::query(
            r#"
            WITH selected_groups AS (
                SELECT DISTINCT
                    observation.scope_type, observation.team_id, observation.coach_id,
                    observation.competition_id, observation.window_start,
                    observation.window_end, observation.observed_at
                FROM feature.formation_usage_observations observation
                WHERE ($1::uuid IS NULL OR observation.team_id = $1)
                  AND ($2::uuid IS NULL OR observation.coach_id = $2)
                  AND ($3::uuid IS NULL OR observation.competition_id = $3)
            ), limited_groups AS (
                SELECT *
                FROM selected_groups
                ORDER BY observed_at DESC, window_end DESC, scope_type
                LIMIT $4
            )
            SELECT observation.id, observation.scope_type,
                   observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.competition_id, competition.name AS competition_name,
                   observation.window_preset, observation.window_start, observation.window_end,
                   observation.observed_matches, observation.confidence,
                   observation.smoothing_alpha, observation.observed_at,
                   observation.formation_id, formation.code AS formation_code,
                   formation.name AS formation_name, observation.usage_count,
                   observation.raw_probability, observation.smoothed_probability
            FROM limited_groups selected
            JOIN feature.formation_usage_observations observation
              ON observation.scope_type = selected.scope_type
             AND observation.team_id IS NOT DISTINCT FROM selected.team_id
             AND observation.coach_id IS NOT DISTINCT FROM selected.coach_id
             AND observation.competition_id IS NOT DISTINCT FROM selected.competition_id
             AND observation.window_start = selected.window_start
             AND observation.window_end = selected.window_end
             AND observation.observed_at = selected.observed_at
            JOIN football.formations formation ON formation.id = observation.formation_id
            LEFT JOIN football.teams team ON team.id = observation.team_id
            LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
            LEFT JOIN football.competitions competition ON competition.id = observation.competition_id
            ORDER BY observation.observed_at DESC, observation.window_end DESC,
                     observation.scope_type, observation.smoothed_probability DESC,
                     formation.sort_order, formation.code
            "#,
        )
        .bind(query.team_id)
        .bind(query.coach_id)
        .bind(query.competition_id)
        .bind(i64::from(query.limit.clamp(1, 1000)))
        .fetch_all(&self.pool)
        .await?;
        group_distribution_rows(&rows)
    }

    pub async fn resolve_formation_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PersistenceResult<ResolvedFormationDistribution> {
        let as_of = query.as_of.unwrap_or_else(Utc::now);
        let as_of_date = as_of.date_naive();
        let mut competition_id = query.competition_id;

        if let Some(match_id) = query.match_id {
            let lineup_row = sqlx::query(
                r#"
                SELECT lineup.formation_id, formation.code, formation.name,
                       lineup.quality_score, lineup.lineup_type,
                       fixture.competition_id
                FROM football.lineups lineup
                JOIN football.matches fixture ON fixture.id = lineup.match_id
                JOIN football.formations formation ON formation.id = lineup.formation_id
                WHERE lineup.match_id = $1
                  AND lineup.team_id = $2
                  AND lineup.status = 'active'
                  AND lineup.lineup_type IN ('actual','confirmed')
                  AND lineup.captured_at <= $3
                ORDER BY CASE lineup.lineup_type WHEN 'actual' THEN 0 ELSE 1 END,
                         lineup.captured_at DESC, lineup.id DESC
                LIMIT 1
                "#,
            )
            .bind(match_id)
            .bind(query.team_id)
            .bind(as_of)
            .fetch_optional(&self.pool)
            .await?;
            if let Some(row) = lineup_row {
                let formation_id: Uuid = row.try_get("formation_id")?;
                let lineup_type: String = row.try_get("lineup_type")?;
                let (source_level, source_label) = if lineup_type == "actual" {
                    ("actual_lineup", "当前比赛实际阵型")
                } else {
                    ("confirmed_lineup", "当前比赛确认阵型")
                };
                competition_id = competition_id.or(row.try_get("competition_id")?);
                return Ok(ResolvedFormationDistribution {
                    source_level: source_level.to_string(),
                    source_label: source_label.to_string(),
                    team_id: query.team_id,
                    coach_id: query.coach_id,
                    competition_id,
                    window_start: None,
                    window_end: None,
                    observed_matches: 1,
                    confidence: row
                        .try_get::<Option<f64>, _>("quality_score")?
                        .unwrap_or(1.0),
                    entries: vec![FormationUsageEntryRecord {
                        id: Uuid::nil(),
                        formation_id,
                        formation_code: row.try_get("code")?,
                        formation_name: row.try_get("name")?,
                        usage_count: 1,
                        raw_probability: 1.0,
                        smoothed_probability: 1.0,
                    }],
                });
            }
            if competition_id.is_none() {
                competition_id =
                    sqlx::query_scalar("SELECT competition_id FROM football.matches WHERE id=$1")
                        .bind(match_id)
                        .fetch_optional(&self.pool)
                        .await?
                        .flatten();
            }
        }

        let coach_id = if query.coach_id.is_some() {
            query.coach_id
        } else {
            sqlx::query_scalar(
                r#"
                SELECT coach_id
                FROM football.team_coach_periods
                WHERE team_id=$1
                  AND role IN ('head_coach','interim_head_coach','caretaker')
                  AND valid_from <= $2
                  AND (valid_to IS NULL OR valid_to >= $2)
                ORDER BY CASE role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                         valid_from DESC, id DESC
                LIMIT 1
                "#,
            )
            .bind(query.team_id)
            .bind(as_of_date)
            .fetch_optional(&self.pool)
            .await?
        };

        let candidates = [
            (
                "team_coach",
                Some(query.team_id),
                coach_id,
                None,
                "球队 + 教练",
            ),
            ("team", Some(query.team_id), None, None, "球队"),
            ("coach", None, coach_id, None, "教练"),
            (
                "competition_default",
                None,
                None,
                competition_id,
                "赛事默认",
            ),
            ("system_default", None, None, None, "系统默认"),
        ];
        for (scope, team, coach, competition, label) in candidates {
            if (scope == "team_coach" || scope == "coach") && coach.is_none() {
                continue;
            }
            if scope == "competition_default" && competition.is_none() {
                continue;
            }
            if let Some(distribution) = self
                .read_latest_distribution(scope, team, coach, competition, as_of)
                .await?
            {
                return Ok(ResolvedFormationDistribution {
                    source_level: scope.to_string(),
                    source_label: label.to_string(),
                    team_id: query.team_id,
                    coach_id,
                    competition_id,
                    window_start: Some(distribution.window_start),
                    window_end: Some(distribution.window_end),
                    observed_matches: distribution.observed_matches,
                    confidence: distribution.confidence,
                    entries: distribution.entries,
                });
            }
        }

        let unknown = sqlx::query("SELECT id, code, name FROM football.formations WHERE id=$1")
            .bind(UNKNOWN_FORMATION_ID)
            .fetch_one(&self.pool)
            .await?;
        Ok(ResolvedFormationDistribution {
            source_level: "unknown".to_string(),
            source_label: "无可用观察，回退未知".to_string(),
            team_id: query.team_id,
            coach_id,
            competition_id,
            window_start: None,
            window_end: None,
            observed_matches: 0,
            confidence: 0.0,
            entries: vec![FormationUsageEntryRecord {
                id: Uuid::nil(),
                formation_id: unknown.try_get("id")?,
                formation_code: unknown.try_get("code")?,
                formation_name: unknown.try_get("name")?,
                usage_count: 0,
                raw_probability: 1.0,
                smoothed_probability: 1.0,
            }],
        })
    }

    async fn resolve_formation_window(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PersistenceResult<(NaiveDate, NaiveDate)> {
        let today = Utc::now().date_naive();
        match draft.window_preset.trim() {
            "custom" => {
                let start = draft.window_start.ok_or_else(|| {
                    PersistenceError::InvalidState("自定义窗口必须填写开始日期".to_string())
                })?;
                let end = draft.window_end.ok_or_else(|| {
                    PersistenceError::InvalidState("自定义窗口必须填写结束日期".to_string())
                })?;
                if end < start {
                    return Err(PersistenceError::InvalidState(
                        "观察窗口结束日期不能早于开始日期".to_string(),
                    ));
                }
                Ok((start, end))
            }
            "last_5" | "last_10" | "last_20" => {
                let team_id = draft.team_id.ok_or_else(|| {
                    PersistenceError::InvalidState("最近场次窗口必须选择球队".to_string())
                })?;
                let limit = match draft.window_preset.as_str() {
                    "last_5" => 5_i64,
                    "last_10" => 10_i64,
                    _ => 20_i64,
                };
                let dates = sqlx::query_scalar::<_, NaiveDate>(
                    r#"
                    SELECT fixture.kickoff_time::date
                    FROM football.matches fixture
                    WHERE (fixture.home_team_id=$1 OR fixture.away_team_id=$1)
                      AND fixture.status='finished'
                      AND fixture.kickoff_time::date <= $2
                    ORDER BY fixture.kickoff_time DESC, fixture.id DESC
                    LIMIT $3
                    "#,
                )
                .bind(team_id)
                .bind(today)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
                let end = dates.first().copied().ok_or_else(|| {
                    PersistenceError::InvalidState(
                        "数据库中没有可用于最近场次窗口的已结束比赛".to_string(),
                    )
                })?;
                let start = dates.last().copied().unwrap_or(end);
                Ok((start, end))
            }
            "current_coach_term" => {
                let team_id = draft.team_id.ok_or_else(|| {
                    PersistenceError::InvalidState("当前教练任期必须选择球队".to_string())
                })?;
                let coach_id = draft.coach_id.ok_or_else(|| {
                    PersistenceError::InvalidState("当前教练任期必须选择教练".to_string())
                })?;
                sqlx::query_as::<_, (NaiveDate, Option<NaiveDate>)>(
                    r#"
                    SELECT valid_from, valid_to
                    FROM football.team_coach_periods
                    WHERE team_id=$1 AND coach_id=$2
                      AND valid_from <= $3
                    ORDER BY valid_from DESC, id DESC
                    LIMIT 1
                    "#,
                )
                .bind(team_id)
                .bind(coach_id)
                .bind(today)
                .fetch_optional(&self.pool)
                .await?
                .map(|(start, end)| (start, end.unwrap_or(today).min(today)))
                .ok_or_else(|| PersistenceError::InvalidState("没有找到对应的教练任期".to_string()))
            }
            "current_season" => {
                let range = if let Some(competition_id) = draft.competition_id {
                    sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
                        r#"
                        SELECT min(kickoff_time::date), max(kickoff_time::date)
                        FROM football.matches
                        WHERE competition_id=$1
                          AND extract(year from kickoff_time)=$2
                          AND kickoff_time::date <= $3
                        "#,
                    )
                    .bind(competition_id)
                    .bind(today.year())
                    .bind(today)
                    .fetch_one(&self.pool)
                    .await?
                } else if let Some(team_id) = draft.team_id {
                    sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
                        r#"
                        SELECT min(kickoff_time::date), max(kickoff_time::date)
                        FROM football.matches
                        WHERE (home_team_id=$1 OR away_team_id=$1)
                          AND extract(year from kickoff_time)=$2
                          AND kickoff_time::date <= $3
                        "#,
                    )
                    .bind(team_id)
                    .bind(today.year())
                    .bind(today)
                    .fetch_one(&self.pool)
                    .await?
                } else {
                    (None, None)
                };
                if let (Some(start), Some(end)) = range {
                    return Ok((start, end));
                }
                let start = NaiveDate::from_ymd_opt(today.year(), 1, 1).ok_or_else(|| {
                    PersistenceError::InvalidState("无法生成当前赛季日期".to_string())
                })?;
                Ok((start, today))
            }
            other => Err(PersistenceError::InvalidState(format!(
                "未知阵型观察窗口：{other}"
            ))),
        }
    }

    async fn read_latest_distribution(
        &self,
        scope_type: &str,
        team_id: Option<Uuid>,
        coach_id: Option<Uuid>,
        competition_id: Option<Uuid>,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Option<FormationUsageDistributionRecord>> {
        let group = sqlx::query(
            r#"
            SELECT window_start, window_end, observed_at
            FROM feature.formation_usage_observations
            WHERE scope_type=$1
              AND team_id IS NOT DISTINCT FROM $2
              AND coach_id IS NOT DISTINCT FROM $3
              AND competition_id IS NOT DISTINCT FROM $4
              AND window_start <= $5
              AND window_end <= $5
              AND observed_at <= $6
            ORDER BY window_end DESC, observed_at DESC
            LIMIT 1
            "#,
        )
        .bind(scope_type)
        .bind(team_id)
        .bind(coach_id)
        .bind(competition_id)
        .bind(as_of.date_naive())
        .bind(as_of)
        .fetch_optional(&self.pool)
        .await?;
        let Some(group) = group else {
            return Ok(None);
        };
        self.read_exact_distribution(&FormationUsageGroupKey {
            scope_type,
            team_id,
            coach_id,
            competition_id,
            window_start: group.try_get("window_start")?,
            window_end: group.try_get("window_end")?,
            observed_at: group.try_get("observed_at")?,
        })
        .await
    }

    async fn read_exact_distribution(
        &self,
        key: &FormationUsageGroupKey<'_>,
    ) -> PersistenceResult<Option<FormationUsageDistributionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT observation.id, observation.scope_type,
                   observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.competition_id, competition.name AS competition_name,
                   observation.window_preset, observation.window_start, observation.window_end,
                   observation.observed_matches, observation.confidence,
                   observation.smoothing_alpha, observation.observed_at,
                   observation.formation_id, formation.code AS formation_code,
                   formation.name AS formation_name, observation.usage_count,
                   observation.raw_probability, observation.smoothed_probability
            FROM feature.formation_usage_observations observation
            JOIN football.formations formation ON formation.id=observation.formation_id
            LEFT JOIN football.teams team ON team.id=observation.team_id
            LEFT JOIN football.coaches coach ON coach.id=observation.coach_id
            LEFT JOIN football.competitions competition ON competition.id=observation.competition_id
            WHERE observation.scope_type=$1
              AND observation.team_id IS NOT DISTINCT FROM $2
              AND observation.coach_id IS NOT DISTINCT FROM $3
              AND observation.competition_id IS NOT DISTINCT FROM $4
              AND observation.window_start=$5
              AND observation.window_end=$6
              AND observation.observed_at=$7
            ORDER BY observation.smoothed_probability DESC, formation.sort_order, formation.code
            "#,
        )
        .bind(key.scope_type)
        .bind(key.team_id)
        .bind(key.coach_id)
        .bind(key.competition_id)
        .bind(key.window_start)
        .bind(key.window_end)
        .bind(key.observed_at)
        .fetch_all(&self.pool)
        .await?;
        Ok(group_distribution_rows(&rows)?.into_iter().next())
    }
}

fn calculate_probabilities(
    counts: &HashMap<Uuid, i32>,
    observed_matches: i32,
    alpha: f64,
) -> PersistenceResult<HashMap<Uuid, (f64, f64)>> {
    if counts.is_empty() {
        return Err(PersistenceError::InvalidState(
            "阵型概率计算至少需要一个阵型".to_string(),
        ));
    }
    if observed_matches == 0 {
        return Ok(counts
            .keys()
            .copied()
            .map(|formation_id| (formation_id, (1.0, 1.0)))
            .collect());
    }
    let formation_count = counts.len() as f64;
    let denominator = observed_matches as f64 + alpha;
    let prior = 1.0 / formation_count;
    Ok(counts
        .iter()
        .map(|(formation_id, usage_count)| {
            let raw = *usage_count as f64 / observed_matches as f64;
            let smoothed = (*usage_count as f64 + alpha * prior) / denominator;
            (*formation_id, (raw, smoothed))
        })
        .collect())
}

fn validate_distribution_draft(draft: &FormationUsageDistributionDraft) -> PersistenceResult<()> {
    let valid_shape = match draft.scope_type.trim() {
        "team" => {
            draft.team_id.is_some() && draft.coach_id.is_none() && draft.competition_id.is_none()
        }
        "coach" => {
            draft.team_id.is_none() && draft.coach_id.is_some() && draft.competition_id.is_none()
        }
        "team_coach" => {
            draft.team_id.is_some() && draft.coach_id.is_some() && draft.competition_id.is_none()
        }
        "competition_default" => {
            draft.team_id.is_none() && draft.coach_id.is_none() && draft.competition_id.is_some()
        }
        "system_default" => {
            draft.team_id.is_none() && draft.coach_id.is_none() && draft.competition_id.is_none()
        }
        _ => false,
    };
    if !valid_shape {
        return Err(PersistenceError::InvalidState(
            "阵型概率作用域与球队、教练、赛事字段不匹配".to_string(),
        ));
    }
    if draft.observed_matches < 0 {
        return Err(PersistenceError::InvalidState(
            "观察场数不能为负数".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "阵型观察可信度必须位于 0–1".to_string(),
        ));
    }
    if !draft.alpha.is_finite() || draft.alpha <= 0.0 || draft.alpha > 100.0 {
        return Err(PersistenceError::InvalidState(
            "阵型平滑参数 alpha 必须位于 0–100".to_string(),
        ));
    }
    let mut ids = HashSet::new();
    if draft
        .entries
        .iter()
        .any(|entry| !ids.insert(entry.formation_id))
    {
        return Err(PersistenceError::InvalidState(
            "阵型概率条目存在重复阵型".to_string(),
        ));
    }
    Ok(())
}

fn group_distribution_rows(
    rows: &[sqlx::postgres::PgRow],
) -> PersistenceResult<Vec<FormationUsageDistributionRecord>> {
    let mut result: Vec<FormationUsageDistributionRecord> = Vec::new();
    for row in rows {
        let scope_type: String = row.try_get("scope_type")?;
        let team_id: Option<Uuid> = row.try_get("team_id")?;
        let coach_id: Option<Uuid> = row.try_get("coach_id")?;
        let competition_id: Option<Uuid> = row.try_get("competition_id")?;
        let window_start: NaiveDate = row.try_get("window_start")?;
        let window_end: NaiveDate = row.try_get("window_end")?;
        let observed_at: DateTime<Utc> = row.try_get("observed_at")?;
        let index = result.iter().position(|item| {
            item.scope_type == scope_type
                && item.team_id == team_id
                && item.coach_id == coach_id
                && item.competition_id == competition_id
                && item.window_start == window_start
                && item.window_end == window_end
                && item.observed_at == observed_at
        });
        let entry = FormationUsageEntryRecord {
            id: row.try_get("id")?,
            formation_id: row.try_get("formation_id")?,
            formation_code: row.try_get("formation_code")?,
            formation_name: row.try_get("formation_name")?,
            usage_count: row.try_get("usage_count")?,
            raw_probability: row.try_get("raw_probability")?,
            smoothed_probability: row.try_get("smoothed_probability")?,
        };
        if let Some(index) = index {
            result[index].entries.push(entry);
        } else {
            result.push(FormationUsageDistributionRecord {
                scope_type,
                team_id,
                team_name: row.try_get("team_name")?,
                coach_id,
                coach_name: row.try_get("coach_name")?,
                competition_id,
                competition_name: row.try_get("competition_name")?,
                window_preset: row.try_get("window_preset")?,
                window_start,
                window_end,
                observed_matches: row.try_get("observed_matches")?,
                confidence: row.try_get("confidence")?,
                alpha: row.try_get("smoothing_alpha")?,
                observed_at,
                entries: vec![entry],
            });
        }
    }
    Ok(result)
}

fn formation_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<FormationRecord> {
    Ok(FormationRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        line_structure: row.try_get("line_structure")?,
        slot_definition: row.try_get("slot_definition")?,
        is_builtin: row.try_get("is_builtin")?,
        is_active: row.try_get("is_active")?,
        sort_order: row.try_get("sort_order")?,
        metadata: row.try_get("metadata")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::FormationUsageEntryDraft;

    fn draft(scope_type: &str) -> FormationUsageDistributionDraft {
        FormationUsageDistributionDraft {
            scope_type: scope_type.to_string(),
            team_id: Some(Uuid::new_v4()),
            coach_id: None,
            competition_id: None,
            window_preset: "custom".to_string(),
            window_start: NaiveDate::from_ymd_opt(2026, 1, 1),
            window_end: NaiveDate::from_ymd_opt(2026, 1, 31),
            observed_matches: 10,
            confidence: 0.8,
            alpha: 3.0,
            source_document_id: None,
            metadata: json!({}),
            entries: vec![FormationUsageEntryDraft {
                formation_id: Uuid::new_v4(),
                usage_count: 7,
            }],
        }
    }

    #[test]
    fn scope_shape_is_strict() {
        assert!(validate_distribution_draft(&draft("team")).is_ok());
        let mut invalid = draft("coach");
        invalid.coach_id = None;
        assert!(validate_distribution_draft(&invalid).is_err());
    }

    #[test]
    fn confidence_and_alpha_are_bounded() {
        let mut invalid = draft("team");
        invalid.confidence = 1.1;
        assert!(validate_distribution_draft(&invalid).is_err());
        invalid.confidence = 0.8;
        invalid.alpha = 0.0;
        assert!(validate_distribution_draft(&invalid).is_err());
    }

    #[test]
    fn smoothed_probabilities_are_normalized() {
        let counts = HashMap::from([
            (Uuid::new_v4(), 6),
            (Uuid::new_v4(), 3),
            (UNKNOWN_FORMATION_ID, 1),
        ]);
        let probabilities = calculate_probabilities(&counts, 10, 3.0).expect("probabilities");
        let raw_sum: f64 = probabilities.values().map(|value| value.0).sum();
        let smoothed_sum: f64 = probabilities.values().map(|value| value.1).sum();
        assert!((raw_sum - 1.0).abs() < 1e-9);
        assert!((smoothed_sum - 1.0).abs() < 1e-9);
    }
}
