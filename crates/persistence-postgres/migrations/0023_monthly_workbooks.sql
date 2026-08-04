-- H 前置阶段 4：球队、球员月度 Excel 工作包与可审计增量导入。

ALTER TABLE catalog.import_batches
    ADD COLUMN IF NOT EXISTS workbook_kind text,
    ADD COLUMN IF NOT EXISTS format_version text,
    ADD COLUMN IF NOT EXISTS ended_previous_count bigint NOT NULL DEFAULT 0;

ALTER TABLE catalog.import_batches
    DROP CONSTRAINT IF EXISTS import_batches_workbook_kind_check;
ALTER TABLE catalog.import_batches
    ADD CONSTRAINT import_batches_workbook_kind_check
    CHECK (workbook_kind IS NULL OR workbook_kind IN ('legacy_player', 'player_monthly', 'team_monthly', 'match_lineup', 'ai_match'));

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_entity_type_check;
ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_entity_type_check CHECK (entity_type IN (
        'team', 'team_name', 'coach', 'coach_name', 'team_coach_period',
        'formation_usage', 'team_tactical_observation', 'team_ability_observation',
        'player', 'player_name', 'player_position', 'player_team_period',
        'player_ability', 'player_availability', 'player_dynamic_tag',
        'external_entity_id', 'match', 'lineup', 'lineup_player'
    ));

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_requested_action_check;
ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_requested_action_check
    CHECK (requested_action IN ('add', 'update', 'clear', 'skip'));

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_status_check;
ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_status_check CHECK (status IN (
        'ready_add', 'ready_update', 'ready_end_previous',
        'conflict', 'error', 'skip', 'imported'
    ));

ALTER TABLE football.team_names ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE football.player_names ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE football.player_positions ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE football.player_team_periods ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE football.coach_names ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE TABLE IF NOT EXISTS feature.team_tactical_observations (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    coach_id uuid REFERENCES football.coaches(id),
    window_start date NOT NULL,
    window_end date NOT NULL,
    build_up_style text,
    progression_style text,
    attacking_width text,
    pressing_intensity text,
    defensive_block text,
    transition_speed text,
    set_piece_tendency text,
    tactical_summary text,
    confidence double precision NOT NULL DEFAULT 0.5 CHECK (confidence BETWEEN 0 AND 1),
    source_urls text[] NOT NULL DEFAULT '{}',
    verified_at timestamptz,
    observed_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (window_end >= window_start),
    UNIQUE NULLS NOT DISTINCT (team_id, coach_id, window_start, window_end, observed_at)
);
CREATE INDEX IF NOT EXISTS team_tactical_observations_lookup_idx
    ON feature.team_tactical_observations (team_id, observed_at DESC, coach_id);

CREATE TABLE IF NOT EXISTS feature.team_ability_observations (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    observed_at timestamptz NOT NULL,
    window_start date NOT NULL,
    window_end date NOT NULL,
    attack_rating double precision CHECK (attack_rating IS NULL OR attack_rating BETWEEN 0 AND 100),
    midfield_rating double precision CHECK (midfield_rating IS NULL OR midfield_rating BETWEEN 0 AND 100),
    defence_rating double precision CHECK (defence_rating IS NULL OR defence_rating BETWEEN 0 AND 100),
    goalkeeper_rating double precision CHECK (goalkeeper_rating IS NULL OR goalkeeper_rating BETWEEN 0 AND 100),
    squad_depth_rating double precision CHECK (squad_depth_rating IS NULL OR squad_depth_rating BETWEEN 0 AND 100),
    stability_rating double precision CHECK (stability_rating IS NULL OR stability_rating BETWEEN 0 AND 100),
    sample_size integer NOT NULL DEFAULT 0 CHECK (sample_size >= 0),
    methodology text,
    confidence double precision NOT NULL DEFAULT 0.5 CHECK (confidence BETWEEN 0 AND 1),
    source_urls text[] NOT NULL DEFAULT '{}',
    verified_at timestamptz,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (window_end >= window_start),
    UNIQUE (team_id, observed_at, window_start, window_end)
);
CREATE INDEX IF NOT EXISTS team_ability_observations_lookup_idx
    ON feature.team_ability_observations (team_id, observed_at DESC);

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'monthly-workbooks'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'monthly-workbooks', '1.0.0', '0.16.0', '0.17.0',
            'football.monthly-workbooks-contract.v1', '412cc3dc63e715d05b4a5b58c5c79768e4d86ac85ac1857e781e8a750a177cf8', 'G',
            jsonb_build_object(
                'delivery_phase', 'H_PRE_STAGE_4',
                'contract_path', 'contracts/monthly-workbooks-contract.json',
                'team_workbook', 'football.team-monthly.v1',
                'player_workbook', 'football.player-monthly.v2',
                'preview_before_commit', true,
                'blank_means_no_change', true,
                'explicit_clear_supported', true,
                'integration_point_h_started', false
            )
        );
    ELSIF existing_hash <> '412cc3dc63e715d05b4a5b58c5c79768e4d86ac85ac1857e781e8a750a177cf8' THEN
        RAISE EXCEPTION 'monthly workbook contract hash conflict: existing %, expected %',
            existing_hash, '412cc3dc63e715d05b4a5b58c5c79768e4d86ac85ac1857e781e8a750a177cf8';
    END IF;
END;
$migration$;
