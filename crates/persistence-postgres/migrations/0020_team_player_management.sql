-- H 前置第二轮：球队中心、球队资料、批量管理、球员导入自动关联与 API 球队提案。
-- TEAM_PLAYER_CONTRACT_SHA256 = 6c5f0da24656024aece2da2016c7b4d424d83f85e4cd62fa1c8997ea8a17bfc6
-- API_WORKSPACE_V2_CONTRACT_SHA256 = 5d5b800a4d2c001a821637b99eecda17e017d77a8842149a09a2da3aa6ef1ae2

CREATE TABLE football.team_profiles (
    team_id uuid PRIMARY KEY REFERENCES football.teams(id) ON DELETE CASCADE,
    short_name text,
    team_type text NOT NULL DEFAULT 'club'
        CHECK (team_type IN ('club', 'national', 'reserve', 'youth', 'women', 'other')),
    founded_year smallint CHECK (founded_year IS NULL OR founded_year BETWEEN 1850 AND 2100),
    city text,
    stadium text,
    head_coach text,
    default_formation text,
    tactical_style text NOT NULL DEFAULT 'balanced'
        CHECK (tactical_style IN ('balanced', 'possession', 'direct', 'counter', 'pressing', 'defensive', 'custom')),
    attack_rating double precision CHECK (attack_rating IS NULL OR attack_rating BETWEEN 0 AND 100),
    midfield_rating double precision CHECK (midfield_rating IS NULL OR midfield_rating BETWEEN 0 AND 100),
    defence_rating double precision CHECK (defence_rating IS NULL OR defence_rating BETWEEN 0 AND 100),
    goalkeeper_rating double precision CHECK (goalkeeper_rating IS NULL OR goalkeeper_rating BETWEEN 0 AND 100),
    reputation double precision CHECK (reputation IS NULL OR reputation BETWEEN 0 AND 100),
    data_confidence double precision NOT NULL DEFAULT 0.5
        CHECK (data_confidence BETWEEN 0 AND 1),
    notes text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX team_profiles_style_idx
    ON football.team_profiles (team_type, tactical_style, reputation DESC NULLS LAST);

CREATE INDEX teams_country_active_name_idx
    ON football.teams (country_code, is_active, normalized_name, id);

-- The original operation check was created implicitly by migration 0019.
-- Replace it atomically so existing player proposals remain valid and typed team proposals become available.
ALTER TABLE ai_workspace.operation_proposals
    DROP CONSTRAINT IF EXISTS operation_proposals_operation_type_check;

ALTER TABLE ai_workspace.operation_proposals
    ADD CONSTRAINT operation_proposals_operation_type_check
    CHECK (operation_type IN (
        'add_player_name',
        'assign_player_position',
        'add_player_availability',
        'add_player_dynamic_tag',
        'add_player_ability_observation',
        'add_team_name',
        'update_team_profile'
    ));

-- API 协作产生的动态标签仍属于人工确认写入，单独保留来源类型以便审计。
ALTER TABLE feature.player_dynamic_tags
    DROP CONSTRAINT IF EXISTS player_dynamic_tags_source_type_check;

ALTER TABLE feature.player_dynamic_tags
    ADD CONSTRAINT player_dynamic_tags_source_type_check
    CHECK (source_type IN (
        'manual', 'provider', 'lineup_import', 'ai_analysis',
        'match_review', 'calculation', 'api_workspace'
    ));

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'team-player-management'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'team-player-management',
            '1.0.0',
            '0.13.1',
            '0.13.2',
            'football.team-player-management-contract.v1',
            '6c5f0da24656024aece2da2016c7b4d424d83f85e4cd62fa1c8997ea8a17bfc6',
            'G',
            jsonb_build_object(
                'delivery_phase', 'G_PRE_H_TEAM_PLAYER_MANAGEMENT',
                'contract_path', 'contracts/team-player-management-contract.json',
                'team_center', true,
                'player_team_excel_auto_link', true,
                'bulk_delete_requires_confirmation', true,
                'historical_match_team_delete_blocked', true
            )
        );
    ELSIF existing_hash <> '6c5f0da24656024aece2da2016c7b4d424d83f85e4cd62fa1c8997ea8a17bfc6' THEN
        RAISE EXCEPTION 'team-player-management contract hash conflict: existing %, expected %',
            existing_hash, '6c5f0da24656024aece2da2016c7b4d424d83f85e4cd62fa1c8997ea8a17bfc6';
    END IF;

    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'api-workspace'
      AND contract_version = '2.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'api-workspace',
            '2.0.0',
            '0.13.1',
            '0.13.2',
            'football.api-workspace-contract.v2',
            '5d5b800a4d2c001a821637b99eecda17e017d77a8842149a09a2da3aa6ef1ae2',
            'G',
            jsonb_build_object(
                'delivery_phase', 'G_PRE_H_TEAM_PROFILE',
                'contract_path', 'contracts/api-workspace-contract-v2.json',
                'response_schema_path', 'schemas/api-workspace-response-v2.schema.json',
                'team_operation_types', jsonb_build_array('add_team_name', 'update_team_profile'),
                'arbitrary_sql_allowed', false,
                'automatic_apply_allowed', false,
                'formal_p4_evidence_bypass_allowed', false
            )
        );
    ELSIF existing_hash <> '5d5b800a4d2c001a821637b99eecda17e017d77a8842149a09a2da3aa6ef1ae2' THEN
        RAISE EXCEPTION 'API workspace v2 contract hash conflict: existing %, expected %',
            existing_hash, '5d5b800a4d2c001a821637b99eecda17e017d77a8842149a09a2da3aa6ef1ae2';
    END IF;
END;
$migration$;
