use crate::{PersistenceResult, PostgresStore};
use football_domain::{EntityCandidate, FactPipelineContext};
use sqlx::Row;
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PersistenceResult<Vec<EntityCandidate>> {
        if entity_type == "match" {
            return Ok(vec![EntityCandidate {
                entity_id: context.match_id,
                canonical_name: context.match_key.clone(),
                matched_name: context.match_key.clone(),
                strategy: "research_run_match_scope".to_string(),
                score: 100,
                relation: Some("current_match".to_string()),
            }]);
        }
        if entity_type == "competition" {
            let Some(competition_id) = context.competition_id else {
                return Ok(Vec::new());
            };
            let name = context.competition_name.clone().unwrap_or_default();
            let code = context.competition_code.clone().unwrap_or_default();
            let matches = [name.as_str(), code.as_str()]
                .iter()
                .any(|value| normalize_for_lookup(value) == normalized_name);
            return Ok(if matches {
                vec![EntityCandidate {
                    entity_id: competition_id,
                    canonical_name: name.clone(),
                    matched_name: if normalize_for_lookup(&name) == normalized_name {
                        name
                    } else {
                        code
                    },
                    strategy: "match_competition_exact".to_string(),
                    score: 98,
                    relation: Some("competition".to_string()),
                }]
            } else {
                Vec::new()
            });
        }
        if !matches!(entity_type, "team" | "player") {
            return Ok(Vec::new());
        }

        let mut by_id: BTreeMap<Uuid, EntityCandidate> = BTreeMap::new();
        if let Some(external_id) = external_id.map(str::trim).filter(|value| !value.is_empty()) {
            let rows = if entity_type == "team" {
                sqlx::query(
                    r#"
                    SELECT external.entity_id, team.canonical_name,
                           CASE
                               WHEN external.entity_id = $2 THEN 'home'
                               WHEN external.entity_id = $3 THEN 'away'
                           END AS relation
                    FROM football.external_entity_ids external
                    JOIN football.teams team ON team.id = external.entity_id
                    WHERE external.entity_type = 'team'
                      AND external.external_id = $1
                      AND external.entity_id IN ($2, $3)
                    "#,
                )
                .bind(external_id)
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query(
                    r#"
                    WITH scoped AS (
                        SELECT period.player_id,
                               CASE
                                   WHEN period.team_id = $2 THEN 'home'
                                   WHEN period.team_id = $3 THEN 'away'
                               END AS relation
                        FROM football.player_team_periods period
                        WHERE period.team_id IN ($2, $3)
                          AND period.valid_from <= $4::date
                          AND (period.valid_to IS NULL OR period.valid_to >= $4::date)
                          AND period.registration_status IN ('registered', 'loan', 'trial')
                        UNION
                        SELECT lineup_player.player_id,
                               CASE
                                   WHEN lineup.team_id = $2 THEN 'home'
                                   WHEN lineup.team_id = $3 THEN 'away'
                               END AS relation
                        FROM football.lineups lineup
                        JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = lineup.id
                        WHERE lineup.match_id = $5
                    )
                    SELECT DISTINCT external.entity_id, player.canonical_name, scoped.relation
                    FROM football.external_entity_ids external
                    JOIN football.players player ON player.id = external.entity_id
                    JOIN scoped ON scoped.player_id = external.entity_id
                    WHERE external.entity_type = 'player'
                      AND external.external_id = $1
                    "#,
                )
                .bind(external_id)
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(context.match_id)
                .fetch_all(&self.pool)
                .await?
            };
            for row in rows {
                let entity_id: Uuid = row.try_get("entity_id")?;
                let canonical_name: String = row.try_get("canonical_name")?;
                let relation: Option<String> = row.try_get("relation")?;
                keep_best_candidate(
                    &mut by_id,
                    EntityCandidate {
                        entity_id,
                        canonical_name,
                        matched_name: external_id.to_string(),
                        strategy: "external_id_match_scoped".to_string(),
                        score: 100,
                        relation,
                    },
                );
            }
        }

        match entity_type {
            "team" => {
                let rows = sqlx::query(
                    r#"
                    WITH scoped(team_id, relation) AS (
                        VALUES ($1::uuid, 'home'::text), ($2::uuid, 'away'::text)
                    ), names AS (
                        SELECT team.id AS entity_id, team.canonical_name,
                               team.canonical_name AS matched_name,
                               team.normalized_name,
                               relation, 'canonical'::text AS kind
                        FROM scoped
                        JOIN football.teams team ON team.id = scoped.team_id
                        WHERE scoped.team_id IS NOT NULL
                        UNION ALL
                        SELECT team.id, team.canonical_name, alias.name,
                               alias.normalized_name, relation, 'alias'::text
                        FROM scoped
                        JOIN football.teams team ON team.id = scoped.team_id
                        JOIN football.team_names alias ON alias.team_id = team.id
                        WHERE scoped.team_id IS NOT NULL
                          AND (alias.valid_from IS NULL OR alias.valid_from <= $3::date)
                          AND (alias.valid_to IS NULL OR alias.valid_to >= $3::date)
                    )
                    SELECT entity_id, canonical_name, matched_name, relation, kind
                    FROM names
                    WHERE normalized_name = $4
                       OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $5
                    "#,
                )
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(normalized_name)
                .bind(compact_name)
                .fetch_all(&self.pool)
                .await?;
                for row in rows {
                    let entity_id: Uuid = row.try_get("entity_id")?;
                    let kind: String = row.try_get("kind")?;
                    let candidate = EntityCandidate {
                        entity_id,
                        canonical_name: row.try_get("canonical_name")?,
                        matched_name: row.try_get("matched_name")?,
                        strategy: format!("match_team_{kind}_exact"),
                        score: if kind == "canonical" { 98 } else { 95 },
                        relation: row.try_get("relation")?,
                    };
                    keep_best_candidate(&mut by_id, candidate);
                }
            }
            "player" => {
                let rows = sqlx::query(
                    r#"
                    WITH scoped AS (
                        SELECT period.player_id,
                               CASE
                                   WHEN period.team_id = $1 THEN 'home'
                                   WHEN period.team_id = $2 THEN 'away'
                                   ELSE 'other'
                               END AS relation
                        FROM football.player_team_periods period
                        WHERE period.team_id IN ($1, $2)
                          AND period.valid_from <= $3::date
                          AND (period.valid_to IS NULL OR period.valid_to >= $3::date)
                          AND period.registration_status IN ('registered', 'loan', 'trial')
                        UNION
                        SELECT lineup_player.player_id,
                               CASE
                                   WHEN lineup.team_id = $1 THEN 'home'
                                   WHEN lineup.team_id = $2 THEN 'away'
                                   ELSE 'other'
                               END AS relation
                        FROM football.lineups lineup
                        JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = lineup.id
                        WHERE lineup.match_id = $4
                    ), names AS (
                        SELECT player.id AS entity_id, player.canonical_name,
                               player.canonical_name AS matched_name,
                               player.normalized_name,
                               scoped.relation, 'canonical'::text AS kind
                        FROM scoped
                        JOIN football.players player ON player.id = scoped.player_id
                        UNION ALL
                        SELECT player.id, player.canonical_name, alias.name,
                               alias.normalized_name, scoped.relation, 'alias'::text
                        FROM scoped
                        JOIN football.players player ON player.id = scoped.player_id
                        JOIN football.player_names alias ON alias.player_id = player.id
                        WHERE (alias.valid_from IS NULL OR alias.valid_from <= $3::date)
                          AND (alias.valid_to IS NULL OR alias.valid_to >= $3::date)
                    )
                    SELECT entity_id, canonical_name, matched_name, relation, kind
                    FROM names
                    WHERE normalized_name = $5
                       OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $6
                    "#,
                )
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(context.match_id)
                .bind(normalized_name)
                .bind(compact_name)
                .fetch_all(&self.pool)
                .await?;
                for row in rows {
                    let entity_id: Uuid = row.try_get("entity_id")?;
                    let kind: String = row.try_get("kind")?;
                    let candidate = EntityCandidate {
                        entity_id,
                        canonical_name: row.try_get("canonical_name")?,
                        matched_name: row.try_get("matched_name")?,
                        strategy: format!("match_player_{kind}_exact"),
                        score: if kind == "canonical" { 98 } else { 95 },
                        relation: row.try_get("relation")?,
                    };
                    keep_best_candidate(&mut by_id, candidate);
                }

                if by_id.is_empty() {
                    let rows = sqlx::query(
                        r#"
                        WITH names AS (
                            SELECT player.id AS entity_id, player.canonical_name,
                                   player.canonical_name AS matched_name,
                                   player.normalized_name, 'canonical'::text AS kind
                            FROM football.players player
                            WHERE player.status <> 'retired'
                            UNION ALL
                            SELECT player.id, player.canonical_name, alias.name,
                                   alias.normalized_name, 'alias'::text
                            FROM football.players player
                            JOIN football.player_names alias ON alias.player_id = player.id
                            WHERE player.status <> 'retired'
                              AND (alias.valid_from IS NULL OR alias.valid_from <= $1::date)
                              AND (alias.valid_to IS NULL OR alias.valid_to >= $1::date)
                        )
                        SELECT entity_id, canonical_name, matched_name, kind
                        FROM names
                        WHERE normalized_name = $2
                           OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $3
                        LIMIT 20
                        "#,
                    )
                    .bind(context.data_cutoff_at.date_naive())
                    .bind(normalized_name)
                    .bind(compact_name)
                    .fetch_all(&self.pool)
                    .await?;
                    for row in rows {
                        let entity_id: Uuid = row.try_get("entity_id")?;
                        let kind: String = row.try_get("kind")?;
                        let candidate = EntityCandidate {
                            entity_id,
                            canonical_name: row.try_get("canonical_name")?,
                            matched_name: row.try_get("matched_name")?,
                            strategy: format!("global_player_{kind}_exact"),
                            score: if kind == "canonical" { 78 } else { 72 },
                            relation: None,
                        };
                        keep_best_candidate(&mut by_id, candidate);
                    }
                }
            }
            _ => unreachable!(),
        }
        let mut candidates: Vec<_> = by_id.into_values().collect();
        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.canonical_name.cmp(&right.canonical_name))
                .then_with(|| left.entity_id.cmp(&right.entity_id))
        });
        Ok(candidates)
    }
}

fn keep_best_candidate(map: &mut BTreeMap<Uuid, EntityCandidate>, candidate: EntityCandidate) {
    match map.get(&candidate.entity_id) {
        Some(existing) if existing.score >= candidate.score => {}
        _ => {
            map.insert(candidate.entity_id, candidate);
        }
    }
}

fn normalize_for_lookup(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{keep_best_candidate, normalize_for_lookup};
    use football_domain::EntityCandidate;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[test]
    fn best_candidate_keeps_highest_score_per_entity() {
        let id = Uuid::new_v4();
        let mut map = BTreeMap::new();
        keep_best_candidate(
            &mut map,
            EntityCandidate {
                entity_id: id,
                canonical_name: "A".to_string(),
                matched_name: "A".to_string(),
                strategy: "global".to_string(),
                score: 70,
                relation: None,
            },
        );
        keep_best_candidate(
            &mut map,
            EntityCandidate {
                entity_id: id,
                canonical_name: "A".to_string(),
                matched_name: "Alias".to_string(),
                strategy: "scoped".to_string(),
                score: 95,
                relation: Some("home".to_string()),
            },
        );
        assert_eq!(map.get(&id).expect("candidate").score, 95);
    }
    #[test]
    fn equal_score_keeps_first_candidate_and_distinct_ids_survive() {
        let mut map = BTreeMap::new();
        let first = EntityCandidate {
            entity_id: Uuid::new_v4(),
            canonical_name: "A".into(),
            matched_name: " A ".into(),
            strategy: "canonical".into(),
            score: 95,
            relation: Some("home".into()),
        };
        keep_best_candidate(&mut map, first.clone());
        let mut equal = first.clone();
        equal.strategy = "alias".into();
        equal.relation = Some("away".into());
        keep_best_candidate(&mut map, equal.clone());
        assert_eq!(map.get(&first.entity_id), Some(&first));
        equal.entity_id = Uuid::new_v4();
        keep_best_candidate(&mut map, equal);
        assert_eq!(map.len(), 2);
        assert_eq!(normalize_for_lookup("  Player   A  "), "player a");
    }
}
