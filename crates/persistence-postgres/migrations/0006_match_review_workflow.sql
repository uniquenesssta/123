-- 第四阶段：正式赛果、球员与球队复盘、替补影响、预测误差和受控能力回写。

ALTER TABLE review.match_reviews
    ADD COLUMN source_run_id uuid REFERENCES model.runs(id) ON DELETE SET NULL,
    ADD COLUMN status text NOT NULL DEFAULT 'finalized'
        CHECK (status IN ('draft', 'finalized', 'superseded')),
    ADD COLUMN calculation_version text NOT NULL DEFAULT 'review-v1',
    ADD COLUMN result_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN substitutions_snapshot jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN prediction_evaluation jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN finalized_at timestamptz;

CREATE INDEX match_reviews_match_created_idx
    ON review.match_reviews (match_id, created_at DESC);
CREATE INDEX match_reviews_source_run_idx
    ON review.match_reviews (source_run_id)
    WHERE source_run_id IS NOT NULL;

CREATE TABLE review.player_match_observations (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES football.players(id),
    team_id uuid NOT NULL REFERENCES football.teams(id),
    position_code text REFERENCES football.positions(code),
    role_code text,
    started boolean NOT NULL,
    minutes_played smallint NOT NULL CHECK (minutes_played BETWEEN 0 AND 150),
    performance_score double precision CHECK (performance_score IS NULL OR performance_score BETWEEN 0 AND 100),
    input_confidence double precision NOT NULL DEFAULT 1 CHECK (input_confidence BETWEEN 0 AND 1),
    metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    recorded_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (match_id, player_id)
);
CREATE INDEX player_match_observations_match_team_idx
    ON review.player_match_observations (match_id, team_id, started DESC, minutes_played DESC);
CREATE INDEX player_match_observations_player_idx
    ON review.player_match_observations (player_id, match_id);

ALTER TABLE review.player_match_reviews
    ADD COLUMN observation_id uuid REFERENCES review.player_match_observations(id) ON DELETE SET NULL,
    ADD COLUMN entry_type text NOT NULL DEFAULT 'starter'
        CHECK (entry_type IN ('starter', 'substitute', 'unused_substitute')),
    ADD COLUMN contribution_weight double precision NOT NULL DEFAULT 0
        CHECK (contribution_weight BETWEEN 0 AND 1),
    ADD COLUMN ability_candidate_count integer NOT NULL DEFAULT 0 CHECK (ability_candidate_count >= 0);

ALTER TABLE review.team_match_reviews
    ADD COLUMN lineup_continuity double precision CHECK (lineup_continuity IS NULL OR lineup_continuity BETWEEN 0 AND 1),
    ADD COLUMN performance_cohesion double precision CHECK (performance_cohesion IS NULL OR performance_cohesion BETWEEN 0 AND 1),
    ADD COLUMN bench_dropoff double precision,
    ADD COLUMN substitute_count integer NOT NULL DEFAULT 0 CHECK (substitute_count >= 0);

ALTER TABLE review.ability_update_candidates
    ADD COLUMN match_review_id uuid REFERENCES review.match_reviews(id) ON DELETE SET NULL,
    ADD COLUMN player_match_review_id uuid REFERENCES review.player_match_reviews(id) ON DELETE SET NULL,
    ADD COLUMN accepted_observation_id uuid REFERENCES feature.player_ability_observations(id) ON DELETE SET NULL,
    ADD COLUMN decided_by text;

CREATE UNIQUE INDEX ability_candidates_review_player_dimension_uidx
    ON review.ability_update_candidates (match_review_id, player_id, dimension_code)
    WHERE match_review_id IS NOT NULL;
CREATE INDEX ability_candidates_review_idx
    ON review.ability_update_candidates (match_review_id, status, player_id);

CREATE TABLE review.ability_update_decisions (
    id uuid PRIMARY KEY,
    candidate_id uuid NOT NULL REFERENCES review.ability_update_candidates(id) ON DELETE CASCADE,
    decision text NOT NULL CHECK (decision IN ('accepted', 'rejected', 'superseded')),
    previous_value double precision,
    proposed_value double precision NOT NULL,
    applied_observation_id uuid REFERENCES feature.player_ability_observations(id) ON DELETE SET NULL,
    decided_by text NOT NULL,
    decision_note text,
    decided_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ability_update_decisions_candidate_idx
    ON review.ability_update_decisions (candidate_id, decided_at DESC);

CREATE UNIQUE INDEX substitutions_identity_uidx
    ON football.substitutions (match_id, team_id, minute, player_out_id, player_in_id) NULLS NOT DISTINCT;

CREATE INDEX match_results_finalized_idx
    ON football.match_results (finalized_at DESC);

CREATE INDEX matches_review_kickoff_idx
    ON football.matches (kickoff_time DESC, id DESC);

CREATE INDEX model_runs_match_succeeded_idx
    ON model.runs (match_id, completed_at DESC, created_at DESC)
    WHERE status = 'succeeded';
