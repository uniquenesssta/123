-- H 前置阶段 5：比赛、球队、球员、阵容与模型输入闭环。
-- 阵容版本按四时点保存；历史版本保留，只有通过完整性校验的赛前阵容可进入正式模型输入。

ALTER TABLE football.lineups
    ADD COLUMN IF NOT EXISTS snapshot_type text,
    ADD COLUMN IF NOT EXISTS coach_id uuid REFERENCES football.coaches(id),
    ADD COLUMN IF NOT EXISTS source_urls text[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS supersedes_lineup_id uuid REFERENCES football.lineups(id),
    ADD COLUMN IF NOT EXISTS model_validation_status text NOT NULL DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS model_eligible boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS validation_errors jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS validation_warnings jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

UPDATE football.lineups
SET snapshot_type = CASE
    WHEN metadata->>'snapshot_type' IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')
        THEN metadata->>'snapshot_type'
    ELSE 'T-N'
END
WHERE snapshot_type IS NULL OR btrim(snapshot_type) = '';

ALTER TABLE football.lineups
    ALTER COLUMN snapshot_type SET DEFAULT 'T-N',
    ALTER COLUMN snapshot_type SET NOT NULL;

DO $constraints$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineups_snapshot_type_check'
          AND conrelid = 'football.lineups'::regclass
    ) THEN
        ALTER TABLE football.lineups
            ADD CONSTRAINT lineups_snapshot_type_check
            CHECK (snapshot_type IN ('T-N', 'T-24h', 'T-6h', 'T-90m', 'T-1h'));
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineups_model_validation_status_check'
          AND conrelid = 'football.lineups'::regclass
    ) THEN
        ALTER TABLE football.lineups
            ADD CONSTRAINT lineups_model_validation_status_check
            CHECK (model_validation_status IN ('pending', 'valid', 'invalid'));
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineups_validation_errors_array_check'
          AND conrelid = 'football.lineups'::regclass
    ) THEN
        ALTER TABLE football.lineups
            ADD CONSTRAINT lineups_validation_errors_array_check
            CHECK (jsonb_typeof(validation_errors) = 'array');
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineups_validation_warnings_array_check'
          AND conrelid = 'football.lineups'::regclass
    ) THEN
        ALTER TABLE football.lineups
            ADD CONSTRAINT lineups_validation_warnings_array_check
            CHECK (jsonb_typeof(validation_warnings) = 'array');
    END IF;
END;
$constraints$;

ALTER TABLE football.lineup_players
    ADD COLUMN IF NOT EXISTS starting_probability double precision,
    ADD COLUMN IF NOT EXISTS bench_order smallint,
    ADD COLUMN IF NOT EXISTS membership_override boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS source_urls text[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS validation_warning text;

DO $constraints$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineup_players_starting_probability_check'
          AND conrelid = 'football.lineup_players'::regclass
    ) THEN
        ALTER TABLE football.lineup_players
            ADD CONSTRAINT lineup_players_starting_probability_check
            CHECK (starting_probability IS NULL OR starting_probability BETWEEN 0 AND 1);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'lineup_players_bench_order_check'
          AND conrelid = 'football.lineup_players'::regclass
    ) THEN
        ALTER TABLE football.lineup_players
            ADD CONSTRAINT lineup_players_bench_order_check
            CHECK (bench_order IS NULL OR bench_order BETWEEN 1 AND 99);
    END IF;
END;
$constraints$;

WITH ranked AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY match_id, team_id, snapshot_type, lineup_type
               ORDER BY captured_at DESC, created_at DESC, id DESC
           ) AS rank_no
    FROM football.lineups
    WHERE status = 'active'
)
UPDATE football.lineups lineup
SET status = 'superseded', updated_at = now()
FROM ranked
WHERE lineup.id = ranked.id AND ranked.rank_no > 1;

DROP INDEX IF EXISTS football.lineups_one_active_revision_idx;

CREATE UNIQUE INDEX IF NOT EXISTS lineups_active_horizon_version_uq
    ON football.lineups (match_id, team_id, snapshot_type, lineup_type)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS lineups_model_selection_idx
    ON football.lineups (
        match_id, team_id, snapshot_type, lineup_type,
        model_eligible, captured_at DESC, created_at DESC
    )
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS lineups_team_history_idx
    ON football.lineups (team_id, captured_at DESC, match_id, snapshot_type);

CREATE INDEX IF NOT EXISTS lineup_players_model_input_idx
    ON football.lineup_players (lineup_id, is_starter DESC, sequence_no, bench_order);

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'match-lineup-chain'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'match-lineup-chain', '1.0.0', '0.17.0', '0.18.0',
            'football.match-lineup-chain-contract.v1',
            'd77d272df0fb05f99cd5e74131b76e8de53341f7de2246af8df43dfb0a63a079', 'G',
            jsonb_build_object(
                'delivery_phase', 'H_PRE_STAGE_5',
                'contract_path', 'contracts/match-lineup-chain-contract.json',
                'workbook_format', 'football.match-lineup.v2',
                'formal_horizons', jsonb_build_array('T-24h', 'T-6h', 'T-90m', 'T-1h'),
                'versioned_lineups', true,
                'model_eligibility_gate', true,
                'integration_point_h_started', false
            )
        );
    ELSIF existing_hash <> 'd77d272df0fb05f99cd5e74131b76e8de53341f7de2246af8df43dfb0a63a079' THEN
        RAISE EXCEPTION 'match lineup chain contract hash conflict: existing %, expected %',
            existing_hash, 'd77d272df0fb05f99cd5e74131b76e8de53341f7de2246af8df43dfb0a63a079';
    END IF;
END;
$migration$;
