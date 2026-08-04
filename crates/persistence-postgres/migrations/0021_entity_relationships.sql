-- H 前置阶段 2：教练实体、球队教练任期、统一实体引用与安全归档。
-- ENTITY_RELATIONSHIP_CONTRACT_SHA256 = 16517c61311f5bc58513b6e18684bb3d186abb724f89c1df1fd26f9fd1bb6c76

CREATE TABLE football.coaches (
    id uuid PRIMARY KEY,
    canonical_name text NOT NULL,
    normalized_name text NOT NULL,
    nationality_code text,
    status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'retired', 'unknown')),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX coaches_directory_idx
    ON football.coaches (normalized_name text_pattern_ops, id);
CREATE INDEX coaches_active_idx
    ON football.coaches (status, normalized_name, id);

CREATE TABLE football.coach_names (
    id uuid PRIMARY KEY,
    coach_id uuid NOT NULL REFERENCES football.coaches(id) ON DELETE CASCADE,
    name text NOT NULL,
    normalized_name text NOT NULL,
    language_code text,
    is_primary boolean NOT NULL DEFAULT false,
    valid_from date,
    valid_to date,
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
CREATE INDEX coach_names_lookup_idx
    ON football.coach_names (normalized_name text_pattern_ops, coach_id);
CREATE UNIQUE INDEX coach_names_one_primary_idx
    ON football.coach_names (coach_id)
    WHERE is_primary;

CREATE TABLE football.team_coach_periods (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    coach_id uuid NOT NULL REFERENCES football.coaches(id),
    role text NOT NULL DEFAULT 'head_coach'
        CHECK (role IN ('head_coach', 'assistant_coach', 'interim_head_coach', 'caretaker', 'other')),
    valid_from date NOT NULL,
    valid_to date,
    is_interim boolean NOT NULL DEFAULT false,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    confidence double precision NOT NULL DEFAULT 1
        CHECK (confidence >= 0 AND confidence <= 1),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from),
    UNIQUE NULLS NOT DISTINCT (team_id, coach_id, role, valid_from)
);
CREATE INDEX team_coach_periods_team_idx
    ON football.team_coach_periods (team_id, role, valid_from DESC, id DESC);
CREATE INDEX team_coach_periods_coach_idx
    ON football.team_coach_periods (coach_id, valid_from DESC, id DESC);
CREATE INDEX team_coach_periods_current_idx
    ON football.team_coach_periods (team_id, role, coach_id)
    WHERE valid_to IS NULL;

ALTER TABLE football.external_entity_ids
    DROP CONSTRAINT IF EXISTS external_entity_ids_entity_type_check;
ALTER TABLE football.external_entity_ids
    ADD CONSTRAINT external_entity_ids_entity_type_check
    CHECK (entity_type IN ('competition', 'season', 'team', 'player', 'coach', 'match'));

CREATE OR REPLACE FUNCTION football.refresh_team_head_coach_projection(target_team_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $projection$
DECLARE
    projected_name text;
BEGIN
    SELECT coach.canonical_name
    INTO projected_name
    FROM football.team_coach_periods period
    JOIN football.coaches coach ON coach.id = period.coach_id
    WHERE period.team_id = target_team_id
      AND period.role IN ('head_coach', 'interim_head_coach', 'caretaker')
      AND period.valid_from <= current_date
      AND (period.valid_to IS NULL OR period.valid_to >= current_date)
    ORDER BY
        CASE period.role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
        period.valid_from DESC,
        period.id DESC
    LIMIT 1;

    INSERT INTO football.team_profiles (team_id, head_coach)
    VALUES (target_team_id, projected_name)
    ON CONFLICT (team_id) DO UPDATE
    SET head_coach = EXCLUDED.head_coach,
        updated_at = now();
END;
$projection$;

CREATE OR REPLACE FUNCTION football.sync_team_head_coach_projection()
RETURNS trigger
LANGUAGE plpgsql
AS $trigger$
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM football.refresh_team_head_coach_projection(OLD.team_id);
        RETURN OLD;
    END IF;
    PERFORM football.refresh_team_head_coach_projection(NEW.team_id);
    IF TG_OP = 'UPDATE' AND OLD.team_id <> NEW.team_id THEN
        PERFORM football.refresh_team_head_coach_projection(OLD.team_id);
    END IF;
    RETURN NEW;
END;
$trigger$;

CREATE TRIGGER team_coach_periods_projection_trigger
AFTER INSERT OR UPDATE OR DELETE ON football.team_coach_periods
FOR EACH ROW EXECUTE FUNCTION football.sync_team_head_coach_projection();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'entity-relationships'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'entity-relationships', '1.0.0', '0.14.0', '0.15.0',
            'football.entity-relationship-contract.v1', '16517c61311f5bc58513b6e18684bb3d186abb724f89c1df1fd26f9fd1bb6c76', 'G',
            jsonb_build_object(
                'delivery_phase', 'H_PRE_STAGE_2',
                'contract_path', 'contracts/entity-relationship-contract.json',
                'coach_history', true,
                'safe_archive', true,
                'unified_entity_matching', true,
                'integration_point_h_started', false
            )
        );
    ELSIF existing_hash <> '16517c61311f5bc58513b6e18684bb3d186abb724f89c1df1fd26f9fd1bb6c76' THEN
        RAISE EXCEPTION 'entity relationship contract hash conflict: existing %, expected %',
            existing_hash, '16517c61311f5bc58513b6e18684bb3d186abb724f89c1df1fd26f9fd1bb6c76';
    END IF;
END;
$migration$;
