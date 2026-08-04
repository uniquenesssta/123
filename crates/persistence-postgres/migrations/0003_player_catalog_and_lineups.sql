-- 第三阶段：完整球员目录、可用性、阵容与快速能力读取层。
-- 保留观察历史，使用 current/profile 投影加速超大球员列表。

CREATE TABLE football.team_season_memberships (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    season_id uuid NOT NULL REFERENCES football.seasons(id),
    registration_status text NOT NULL DEFAULT 'registered'
        CHECK (registration_status IN ('registered', 'withdrawn', 'suspended', 'guest')),
    valid_from date,
    valid_to date,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from),
    UNIQUE (team_id, season_id)
);
CREATE INDEX team_season_memberships_season_idx
    ON football.team_season_memberships (season_id, registration_status, team_id);

CREATE TABLE football.player_availability (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id),
    team_id uuid REFERENCES football.teams(id),
    competition_id uuid REFERENCES football.competitions(id),
    status text NOT NULL
        CHECK (status IN ('available', 'doubtful', 'injured', 'suspended', 'rested', 'returning', 'unknown')),
    reason text,
    confidence double precision NOT NULL DEFAULT 1
        CHECK (confidence >= 0 AND confidence <= 1),
    valid_from timestamptz NOT NULL,
    valid_to timestamptz,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE INDEX player_availability_current_idx
    ON football.player_availability (player_id, valid_from DESC, valid_to);
CREATE INDEX player_availability_team_idx
    ON football.player_availability (team_id, status, valid_from DESC);

ALTER TABLE football.lineups
    ADD COLUMN status text NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'superseded', 'withdrawn')),
    ADD COLUMN quality_score double precision
        CHECK (quality_score IS NULL OR (quality_score >= 0 AND quality_score <= 1)),
    ADD COLUMN metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE football.lineup_players
    ADD COLUMN availability_status text
        CHECK (availability_status IS NULL OR availability_status IN ('available', 'doubtful', 'injured', 'suspended', 'rested', 'returning', 'unknown')),
    ADD COLUMN metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD CONSTRAINT lineup_players_shirt_number_check
        CHECK (shirt_number IS NULL OR shirt_number BETWEEN 0 AND 99),
    ADD CONSTRAINT lineup_players_expected_minutes_check
        CHECK (expected_minutes IS NULL OR expected_minutes BETWEEN 0 AND 150),
    ADD CONSTRAINT lineup_players_actual_minutes_check
        CHECK (actual_minutes IS NULL OR actual_minutes BETWEEN 0 AND 150),
    ADD CONSTRAINT lineup_players_sequence_check CHECK (sequence_no >= 0);

CREATE INDEX lineup_players_player_idx
    ON football.lineup_players (player_id, lineup_id);
CREATE INDEX lineups_match_team_time_idx
    ON football.lineups (match_id, team_id, lineup_type, captured_at DESC)
    WHERE status = 'active';

CREATE TABLE football.substitutions (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    player_out_id uuid REFERENCES football.players(id),
    player_in_id uuid REFERENCES football.players(id),
    minute smallint NOT NULL CHECK (minute BETWEEN 0 AND 150),
    period text NOT NULL DEFAULT 'normal_time'
        CHECK (period IN ('first_half', 'second_half', 'extra_time_first', 'extra_time_second', 'normal_time')),
    reason text,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (player_out_id IS NULL OR player_in_id IS NULL OR player_out_id <> player_in_id)
);
CREATE INDEX substitutions_match_idx ON football.substitutions (match_id, team_id, minute);
CREATE INDEX substitutions_player_in_idx ON football.substitutions (player_in_id, match_id);
CREATE INDEX substitutions_player_out_idx ON football.substitutions (player_out_id, match_id);

CREATE TABLE feature.player_current_abilities (
    player_id uuid NOT NULL REFERENCES football.players(id) ON DELETE CASCADE,
    dimension_code text NOT NULL REFERENCES feature.player_ability_dimensions(code),
    observation_id uuid NOT NULL REFERENCES feature.player_ability_observations(id) ON DELETE CASCADE,
    value double precision NOT NULL,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    sample_size integer NOT NULL DEFAULT 1 CHECK (sample_size >= 0),
    observed_at timestamptz NOT NULL,
    effective_from timestamptz NOT NULL,
    effective_to timestamptz,
    calculation_version text NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (player_id, dimension_code)
);
CREATE INDEX player_current_abilities_dimension_idx
    ON feature.player_current_abilities (dimension_code, value DESC, player_id);

CREATE TABLE feature.player_ability_profiles (
    player_id uuid PRIMARY KEY REFERENCES football.players(id) ON DELETE CASCADE,
    abilities jsonb NOT NULL DEFAULT '{}'::jsonb,
    average_value double precision,
    average_confidence double precision,
    dimension_count integer NOT NULL DEFAULT 0,
    latest_observed_at timestamptz,
    next_expiry_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX player_ability_profiles_average_idx
    ON feature.player_ability_profiles (average_value DESC NULLS LAST, player_id);

CREATE OR REPLACE FUNCTION feature.rebuild_player_ability_profile(target_player_id uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO feature.player_ability_profiles (
        player_id, abilities, average_value, average_confidence,
        dimension_count, latest_observed_at, next_expiry_at, updated_at
    )
    SELECT
        target_player_id,
        COALESCE(
            jsonb_object_agg(
                ability_current.dimension_code,
                jsonb_build_object(
                    'value', ability_current.value,
                    'confidence', ability_current.confidence,
                    'sample_size', ability_current.sample_size,
                    'observed_at', ability_current.observed_at,
                    'calculation_version', ability_current.calculation_version
                ) ORDER BY ability_current.dimension_code
            ),
            '{}'::jsonb
        ),
        avg(ability_current.value),
        avg(ability_current.confidence),
        count(ability_current.dimension_code)::integer,
        max(ability_current.observed_at),
        min(ability_current.effective_to) FILTER (WHERE ability_current.effective_to IS NOT NULL),
        now()
    FROM feature.player_current_abilities ability_current
    WHERE ability_current.player_id = target_player_id
      AND (ability_current.effective_to IS NULL OR ability_current.effective_to >= now())
    ON CONFLICT (player_id) DO UPDATE SET
        abilities = EXCLUDED.abilities,
        average_value = EXCLUDED.average_value,
        average_confidence = EXCLUDED.average_confidence,
        dimension_count = EXCLUDED.dimension_count,
        latest_observed_at = EXCLUDED.latest_observed_at,
        next_expiry_at = EXCLUDED.next_expiry_at,
        updated_at = now();
END;
$$;

CREATE OR REPLACE FUNCTION feature.project_player_ability_observation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.effective_from <= now()
       AND (NEW.effective_to IS NULL OR NEW.effective_to >= now()) THEN
        INSERT INTO feature.player_current_abilities (
            player_id, dimension_code, observation_id, value, confidence,
            sample_size, observed_at, effective_from, effective_to,
            calculation_version, updated_at
        ) VALUES (
            NEW.player_id, NEW.dimension_code, NEW.id, NEW.value, NEW.confidence,
            NEW.sample_size, NEW.observed_at, NEW.effective_from, NEW.effective_to,
            NEW.calculation_version, now()
        )
        ON CONFLICT (player_id, dimension_code) DO UPDATE SET
            observation_id = EXCLUDED.observation_id,
            value = EXCLUDED.value,
            confidence = EXCLUDED.confidence,
            sample_size = EXCLUDED.sample_size,
            observed_at = EXCLUDED.observed_at,
            effective_from = EXCLUDED.effective_from,
            effective_to = EXCLUDED.effective_to,
            calculation_version = EXCLUDED.calculation_version,
            updated_at = now()
        WHERE (EXCLUDED.effective_from, EXCLUDED.observed_at, EXCLUDED.observation_id)
            >= (feature.player_current_abilities.effective_from,
                feature.player_current_abilities.observed_at,
                feature.player_current_abilities.observation_id);

        PERFORM feature.rebuild_player_ability_profile(NEW.player_id);
    ELSE
        DELETE FROM feature.player_current_abilities
        WHERE player_id = NEW.player_id
          AND dimension_code = NEW.dimension_code
          AND observation_id = NEW.id;

        INSERT INTO feature.player_current_abilities (
            player_id, dimension_code, observation_id, value, confidence,
            sample_size, observed_at, effective_from, effective_to,
            calculation_version, updated_at
        )
        SELECT
            observation.player_id,
            observation.dimension_code,
            observation.id,
            observation.value,
            observation.confidence,
            observation.sample_size,
            observation.observed_at,
            observation.effective_from,
            observation.effective_to,
            observation.calculation_version,
            now()
        FROM feature.player_ability_observations observation
        WHERE observation.player_id = NEW.player_id
          AND observation.dimension_code = NEW.dimension_code
          AND observation.effective_from <= now()
          AND (observation.effective_to IS NULL OR observation.effective_to >= now())
        ORDER BY
            observation.effective_from DESC,
            observation.observed_at DESC,
            observation.id DESC
        LIMIT 1
        ON CONFLICT (player_id, dimension_code) DO UPDATE SET
            observation_id = EXCLUDED.observation_id,
            value = EXCLUDED.value,
            confidence = EXCLUDED.confidence,
            sample_size = EXCLUDED.sample_size,
            observed_at = EXCLUDED.observed_at,
            effective_from = EXCLUDED.effective_from,
            effective_to = EXCLUDED.effective_to,
            calculation_version = EXCLUDED.calculation_version,
            updated_at = now();

        PERFORM feature.rebuild_player_ability_profile(NEW.player_id);
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER player_ability_observation_projection
AFTER INSERT OR UPDATE OF value, confidence, sample_size, observed_at, effective_from, effective_to
ON feature.player_ability_observations
FOR EACH ROW EXECUTE FUNCTION feature.project_player_ability_observation();

-- 回填第二阶段数据库中可能已经存在的观察记录。
INSERT INTO feature.player_current_abilities (
    player_id, dimension_code, observation_id, value, confidence,
    sample_size, observed_at, effective_from, effective_to,
    calculation_version, updated_at
)
SELECT DISTINCT ON (observation.player_id, observation.dimension_code)
    observation.player_id,
    observation.dimension_code,
    observation.id,
    observation.value,
    observation.confidence,
    observation.sample_size,
    observation.observed_at,
    observation.effective_from,
    observation.effective_to,
    observation.calculation_version,
    now()
FROM feature.player_ability_observations observation
WHERE observation.effective_from <= now()
  AND (observation.effective_to IS NULL OR observation.effective_to >= now())
ORDER BY
    observation.player_id,
    observation.dimension_code,
    observation.effective_from DESC,
    observation.observed_at DESC,
    observation.id DESC
ON CONFLICT (player_id, dimension_code) DO UPDATE SET
    observation_id = EXCLUDED.observation_id,
    value = EXCLUDED.value,
    confidence = EXCLUDED.confidence,
    sample_size = EXCLUDED.sample_size,
    observed_at = EXCLUDED.observed_at,
    effective_from = EXCLUDED.effective_from,
    effective_to = EXCLUDED.effective_to,
    calculation_version = EXCLUDED.calculation_version,
    updated_at = now();

INSERT INTO feature.player_ability_profiles (
    player_id, abilities, average_value, average_confidence,
    dimension_count, latest_observed_at, next_expiry_at, updated_at
)
SELECT
    ability_current.player_id,
    jsonb_object_agg(
        ability_current.dimension_code,
        jsonb_build_object(
            'value', ability_current.value,
            'confidence', ability_current.confidence,
            'sample_size', ability_current.sample_size,
            'observed_at', ability_current.observed_at,
            'calculation_version', ability_current.calculation_version
        ) ORDER BY ability_current.dimension_code
    ),
    avg(ability_current.value),
    avg(ability_current.confidence),
    count(*)::integer,
    max(ability_current.observed_at),
    min(ability_current.effective_to) FILTER (WHERE ability_current.effective_to IS NOT NULL),
    now()
FROM feature.player_current_abilities ability_current
WHERE ability_current.effective_to IS NULL OR ability_current.effective_to >= now()
GROUP BY ability_current.player_id
ON CONFLICT (player_id) DO UPDATE SET
    abilities = EXCLUDED.abilities,
    average_value = EXCLUDED.average_value,
    average_confidence = EXCLUDED.average_confidence,
    dimension_count = EXCLUDED.dimension_count,
    latest_observed_at = EXCLUDED.latest_observed_at,
    next_expiry_at = EXCLUDED.next_expiry_at,
    updated_at = now();

CREATE OR REPLACE FUNCTION feature.refresh_player_ability_projections()
RETURNS bigint
LANGUAGE plpgsql
AS $$
DECLARE
    target_player_id uuid;
    refreshed_count bigint := 0;
BEGIN
    FOR target_player_id IN
        WITH latest_active AS (
            SELECT DISTINCT ON (observation.player_id, observation.dimension_code)
                observation.player_id,
                observation.dimension_code,
                observation.id
            FROM feature.player_ability_observations observation
            WHERE observation.effective_from <= now()
              AND (observation.effective_to IS NULL OR observation.effective_to >= now())
            ORDER BY
                observation.player_id,
                observation.dimension_code,
                observation.effective_from DESC,
                observation.observed_at DESC,
                observation.id DESC
        ), affected AS (
            SELECT ability_current.player_id
            FROM feature.player_current_abilities ability_current
            WHERE ability_current.effective_to IS NOT NULL
              AND ability_current.effective_to < now()
            UNION
            SELECT latest_active.player_id
            FROM latest_active
            LEFT JOIN feature.player_current_abilities ability_current
              ON ability_current.player_id = latest_active.player_id
             AND ability_current.dimension_code = latest_active.dimension_code
            WHERE ability_current.observation_id IS DISTINCT FROM latest_active.id
        )
        SELECT DISTINCT affected.player_id FROM affected
    LOOP
        DELETE FROM feature.player_current_abilities
        WHERE player_id = target_player_id;

        INSERT INTO feature.player_current_abilities (
            player_id, dimension_code, observation_id, value, confidence,
            sample_size, observed_at, effective_from, effective_to,
            calculation_version, updated_at
        )
        SELECT DISTINCT ON (observation.player_id, observation.dimension_code)
            observation.player_id,
            observation.dimension_code,
            observation.id,
            observation.value,
            observation.confidence,
            observation.sample_size,
            observation.observed_at,
            observation.effective_from,
            observation.effective_to,
            observation.calculation_version,
            now()
        FROM feature.player_ability_observations observation
        WHERE observation.player_id = target_player_id
          AND observation.effective_from <= now()
          AND (observation.effective_to IS NULL OR observation.effective_to >= now())
        ORDER BY
            observation.player_id,
            observation.dimension_code,
            observation.effective_from DESC,
            observation.observed_at DESC,
            observation.id DESC;

        PERFORM feature.rebuild_player_ability_profile(target_player_id);
        refreshed_count := refreshed_count + 1;
    END LOOP;
    RETURN refreshed_count;
END;
$$;

-- 大规模目录检索与游标分页索引。
CREATE INDEX players_directory_cursor_idx
    ON football.players (normalized_name text_pattern_ops, id)
    WHERE status <> 'retired';
CREATE INDEX teams_directory_cursor_idx
    ON football.teams (normalized_name text_pattern_ops, id)
    WHERE is_active;
CREATE INDEX player_names_prefix_idx
    ON football.player_names (normalized_name text_pattern_ops, player_id);
CREATE INDEX team_names_prefix_idx
    ON football.team_names (normalized_name text_pattern_ops, team_id);
CREATE INDEX player_team_periods_current_team_idx
    ON football.player_team_periods (team_id, player_id)
    WHERE valid_to IS NULL;
CREATE INDEX player_positions_filter_idx
    ON football.player_positions (position_code, player_id, valid_from, valid_to);
CREATE INDEX player_availability_status_filter_idx
    ON football.player_availability (status, player_id, valid_from DESC, valid_to);
-- 升级已有第二阶段数据库时先收敛可能存在的重复当前记录。
WITH ranked_primary_names AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY player_id
               ORDER BY valid_from DESC NULLS LAST, id DESC
           ) AS row_number
    FROM football.player_names
    WHERE is_primary
)
UPDATE football.player_names name
SET is_primary = false
FROM ranked_primary_names ranked
WHERE name.id = ranked.id
  AND ranked.row_number > 1;

WITH ranked_active_lineups AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY match_id, team_id, lineup_type
               ORDER BY captured_at DESC, id DESC
           ) AS row_number
    FROM football.lineups
    WHERE status = 'active'
)
UPDATE football.lineups lineup
SET status = 'superseded'
FROM ranked_active_lineups ranked
WHERE lineup.id = ranked.id
  AND ranked.row_number > 1;

CREATE UNIQUE INDEX player_names_one_primary_idx
    ON football.player_names (player_id)
    WHERE is_primary;
CREATE UNIQUE INDEX lineups_one_active_revision_idx
    ON football.lineups (match_id, team_id, lineup_type)
    WHERE status = 'active';
