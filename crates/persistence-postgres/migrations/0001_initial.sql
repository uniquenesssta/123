CREATE SCHEMA IF NOT EXISTS platform;
CREATE SCHEMA IF NOT EXISTS catalog;
CREATE SCHEMA IF NOT EXISTS football;
CREATE SCHEMA IF NOT EXISTS feature;
CREATE SCHEMA IF NOT EXISTS model;
CREATE SCHEMA IF NOT EXISTS review;
CREATE SCHEMA IF NOT EXISTS analytics;
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE platform.settings (
    key text PRIMARY KEY,
    value jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE platform.jobs (
    id uuid PRIMARY KEY,
    job_type text NOT NULL,
    status text NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    progress double precision NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    result jsonb,
    error_message text,
    idempotency_key text UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz
);

CREATE TABLE catalog.data_providers (
    id uuid PRIMARY KEY,
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    provider_type text NOT NULL,
    base_url text,
    is_active boolean NOT NULL DEFAULT true,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE catalog.source_documents (
    id uuid PRIMARY KEY,
    provider_id uuid REFERENCES catalog.data_providers(id),
    source_type text NOT NULL,
    source_uri text,
    content_sha256 text NOT NULL,
    published_at timestamptz,
    accessed_at timestamptz NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (content_sha256)
);

CREATE TABLE catalog.import_batches (
    id uuid PRIMARY KEY,
    provider_id uuid REFERENCES catalog.data_providers(id),
    source_document_id uuid REFERENCES catalog.source_documents(id),
    import_type text NOT NULL,
    status text NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
    inserted_count bigint NOT NULL DEFAULT 0,
    updated_count bigint NOT NULL DEFAULT 0,
    skipped_count bigint NOT NULL DEFAULT 0,
    error_count bigint NOT NULL DEFAULT 0,
    started_at timestamptz,
    finished_at timestamptz,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE football.competitions (
    id uuid PRIMARY KEY,
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    country_code text,
    timezone text NOT NULL DEFAULT 'UTC',
    competition_kind text NOT NULL CHECK (competition_kind IN ('league', 'group_stage', 'knockout_single_leg', 'knockout_two_leg', 'friendly', 'custom')),
    is_active boolean NOT NULL DEFAULT true,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE football.seasons (
    id uuid PRIMARY KEY,
    competition_id uuid NOT NULL REFERENCES football.competitions(id),
    name text NOT NULL,
    starts_on date,
    ends_on date,
    status text NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'active', 'completed', 'archived')),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (competition_id, name)
);

CREATE TABLE football.competition_stages (
    id uuid PRIMARY KEY,
    season_id uuid NOT NULL REFERENCES football.seasons(id),
    code text NOT NULL,
    name text NOT NULL,
    stage_kind text NOT NULL CHECK (stage_kind IN ('league', 'group_stage', 'knockout_single_leg', 'knockout_two_leg', 'custom')),
    sequence_no integer NOT NULL DEFAULT 0,
    rules jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (season_id, code)
);

CREATE TABLE football.rounds (
    id uuid PRIMARY KEY,
    stage_id uuid NOT NULL REFERENCES football.competition_stages(id),
    code text NOT NULL,
    name text NOT NULL,
    sequence_no integer NOT NULL DEFAULT 0,
    starts_at timestamptz,
    ends_at timestamptz,
    UNIQUE (stage_id, code)
);

CREATE TABLE football.teams (
    id uuid PRIMARY KEY,
    canonical_name text NOT NULL,
    normalized_name text NOT NULL,
    country_code text,
    is_active boolean NOT NULL DEFAULT true,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX teams_normalized_name_idx ON football.teams (normalized_name);

CREATE TABLE football.team_names (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id) ON DELETE CASCADE,
    name text NOT NULL,
    normalized_name text NOT NULL,
    language_code text,
    valid_from date,
    valid_to date
);
CREATE INDEX team_names_lookup_idx ON football.team_names (normalized_name);

CREATE TABLE football.players (
    id uuid PRIMARY KEY,
    canonical_name text NOT NULL,
    normalized_name text NOT NULL,
    date_of_birth date,
    nationality_code text,
    preferred_foot text CHECK (preferred_foot IN ('left', 'right', 'both', 'unknown')),
    height_cm smallint CHECK (height_cm IS NULL OR height_cm BETWEEN 120 AND 230),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'retired', 'unknown')),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX players_normalized_name_idx ON football.players (normalized_name);
CREATE INDEX players_birth_name_idx ON football.players (date_of_birth, normalized_name);

CREATE TABLE football.player_names (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id) ON DELETE CASCADE,
    name text NOT NULL,
    normalized_name text NOT NULL,
    language_code text,
    is_primary boolean NOT NULL DEFAULT false,
    valid_from date,
    valid_to date
);
CREATE INDEX player_names_lookup_idx ON football.player_names (normalized_name);

CREATE TABLE football.external_entity_ids (
    id uuid PRIMARY KEY,
    provider_id uuid NOT NULL REFERENCES catalog.data_providers(id),
    entity_type text NOT NULL CHECK (entity_type IN ('competition', 'season', 'team', 'player', 'match')),
    entity_id uuid NOT NULL,
    external_id text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (provider_id, entity_type, external_id)
);
CREATE INDEX external_entity_target_idx ON football.external_entity_ids (entity_type, entity_id);

CREATE TABLE football.positions (
    code text PRIMARY KEY,
    name text NOT NULL,
    position_group text NOT NULL,
    sort_order smallint NOT NULL DEFAULT 0
);

CREATE TABLE football.player_positions (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id) ON DELETE CASCADE,
    position_code text NOT NULL REFERENCES football.positions(code),
    proficiency double precision NOT NULL DEFAULT 0.5 CHECK (proficiency >= 0 AND proficiency <= 1),
    is_primary boolean NOT NULL DEFAULT false,
    valid_from date,
    valid_to date,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from),
    UNIQUE NULLS NOT DISTINCT (player_id, position_code, valid_from)
);
CREATE INDEX player_positions_player_idx ON football.player_positions (player_id, is_primary DESC, proficiency DESC);

CREATE TABLE football.player_team_periods (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id),
    team_id uuid NOT NULL REFERENCES football.teams(id),
    season_id uuid REFERENCES football.seasons(id),
    squad_number smallint,
    valid_from date NOT NULL,
    valid_to date,
    registration_status text NOT NULL DEFAULT 'registered',
    source_document_id uuid REFERENCES catalog.source_documents(id),
    CHECK (valid_to IS NULL OR valid_to >= valid_from)
);
CREATE INDEX player_team_periods_player_idx ON football.player_team_periods (player_id, valid_from DESC);
CREATE INDEX player_team_periods_team_idx ON football.player_team_periods (team_id, valid_from DESC);

CREATE TABLE football.matches (
    id uuid PRIMARY KEY,
    external_key text NOT NULL UNIQUE,
    competition_id uuid REFERENCES football.competitions(id),
    season_id uuid REFERENCES football.seasons(id),
    stage_id uuid REFERENCES football.competition_stages(id),
    round_id uuid REFERENCES football.rounds(id),
    home_team_id uuid REFERENCES football.teams(id),
    away_team_id uuid REFERENCES football.teams(id),
    kickoff_time timestamptz NOT NULL,
    status text NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'live', 'finished', 'postponed', 'cancelled')),
    venue text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (home_team_id IS NULL OR away_team_id IS NULL OR home_team_id <> away_team_id)
);
CREATE INDEX matches_competition_kickoff_idx ON football.matches (competition_id, kickoff_time DESC);
CREATE INDEX matches_season_status_kickoff_idx ON football.matches (season_id, status, kickoff_time DESC);

CREATE TABLE football.match_results (
    match_id uuid PRIMARY KEY REFERENCES football.matches(id) ON DELETE CASCADE,
    home_goals_90 smallint NOT NULL CHECK (home_goals_90 >= 0),
    away_goals_90 smallint NOT NULL CHECK (away_goals_90 >= 0),
    home_goals_extra_time smallint,
    away_goals_extra_time smallint,
    home_penalties smallint,
    away_penalties smallint,
    finalized_at timestamptz NOT NULL,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE football.lineups (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    lineup_type text NOT NULL CHECK (lineup_type IN ('expected', 'confirmed', 'actual')),
    formation text,
    captured_at timestamptz NOT NULL,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    UNIQUE (match_id, team_id, lineup_type, captured_at)
);

CREATE TABLE football.lineup_players (
    lineup_id uuid NOT NULL REFERENCES football.lineups(id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES football.players(id),
    position_code text REFERENCES football.positions(code),
    role_code text,
    is_starter boolean NOT NULL,
    shirt_number smallint,
    expected_minutes smallint,
    actual_minutes smallint,
    sequence_no smallint NOT NULL DEFAULT 0,
    PRIMARY KEY (lineup_id, player_id)
);

CREATE TABLE feature.snapshots (
    id uuid PRIMARY KEY,
    match_id uuid REFERENCES football.matches(id),
    match_key text NOT NULL,
    snapshot_type text NOT NULL,
    data_cutoff_time timestamptz NOT NULL,
    frozen_at timestamptz NOT NULL,
    schema_version text NOT NULL,
    quality_score double precision CHECK (quality_score IS NULL OR (quality_score >= 0 AND quality_score <= 1)),
    input_payload jsonb NOT NULL,
    input_sha256 text NOT NULL,
    UNIQUE (match_key, snapshot_type, input_sha256)
);
CREATE INDEX snapshots_match_time_idx ON feature.snapshots (match_key, frozen_at DESC);

CREATE TABLE feature.player_ability_dimensions (
    code text PRIMARY KEY,
    name text NOT NULL,
    category text NOT NULL,
    minimum_value double precision NOT NULL DEFAULT 0,
    maximum_value double precision NOT NULL DEFAULT 100,
    description text
);

CREATE TABLE feature.player_ability_observations (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id),
    dimension_code text NOT NULL REFERENCES feature.player_ability_dimensions(code),
    context_type text NOT NULL DEFAULT 'general',
    context_id uuid,
    value double precision NOT NULL,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    sample_size integer NOT NULL DEFAULT 1 CHECK (sample_size >= 0),
    observed_at timestamptz NOT NULL,
    effective_from timestamptz NOT NULL,
    effective_to timestamptz,
    calculation_version text NOT NULL,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    CHECK (effective_to IS NULL OR effective_to >= effective_from)
);
CREATE INDEX ability_observations_player_idx ON feature.player_ability_observations (player_id, dimension_code, effective_from DESC);

CREATE TABLE feature.player_ability_snapshots (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id),
    as_of timestamptz NOT NULL,
    calculation_version text NOT NULL,
    abilities jsonb NOT NULL,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    source_observation_count integer NOT NULL DEFAULT 0,
    UNIQUE (player_id, as_of, calculation_version)
);
CREATE INDEX ability_snapshots_player_time_idx ON feature.player_ability_snapshots (player_id, as_of DESC);

CREATE TABLE model.definitions (
    id uuid PRIMARY KEY,
    model_key text NOT NULL UNIQUE,
    display_name text NOT NULL,
    description text,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE model.versions (
    id uuid PRIMARY KEY,
    model_id uuid NOT NULL REFERENCES model.definitions(id),
    version text NOT NULL,
    engine_version text NOT NULL,
    input_schema_version text NOT NULL,
    output_schema_version text NOT NULL,
    source_sha256 text,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'deprecated', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (model_id, version)
);

CREATE TABLE model.parameter_sets (
    id uuid PRIMARY KEY,
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_version text NOT NULL,
    name text NOT NULL,
    definition jsonb NOT NULL,
    definition_sha256 text NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'deprecated', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (model_version_id, parameter_version)
);

CREATE TABLE model.rule_packages (
    id uuid PRIMARY KEY,
    package_key text NOT NULL,
    version text NOT NULL,
    content_sha256 text NOT NULL UNIQUE,
    manifest jsonb NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('draft', 'active', 'deprecated', 'retired')),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (package_key, version)
);

CREATE TABLE model.competition_bindings (
    id uuid PRIMARY KEY,
    competition_id uuid REFERENCES football.competitions(id),
    season_id uuid REFERENCES football.seasons(id),
    stage_id uuid REFERENCES football.competition_stages(id),
    competition_kind text,
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    rule_package_id uuid REFERENCES model.rule_packages(id),
    priority integer NOT NULL DEFAULT 0,
    is_active boolean NOT NULL DEFAULT true,
    valid_from timestamptz,
    valid_to timestamptz,
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
CREATE INDEX competition_bindings_route_idx ON model.competition_bindings (competition_id, season_id, stage_id, priority DESC) WHERE is_active;

CREATE TABLE model.runs (
    id uuid PRIMARY KEY,
    match_id uuid REFERENCES football.matches(id),
    match_key text NOT NULL,
    feature_snapshot_id uuid REFERENCES feature.snapshots(id),
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    rule_package_id uuid REFERENCES model.rule_packages(id),
    snapshot_type text NOT NULL,
    route_reason jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL CHECK (status IN ('running', 'succeeded', 'failed', 'cancelled')),
    input_payload jsonb NOT NULL,
    output_payload jsonb,
    explanation jsonb,
    summary jsonb,
    input_sha256 text NOT NULL,
    duration_ms bigint,
    error_message text,
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz
);
CREATE INDEX model_runs_match_idx ON model.runs (match_key, created_at DESC);
CREATE INDEX model_runs_version_idx ON model.runs (model_version_id, parameter_set_id, created_at DESC);
CREATE INDEX model_runs_created_brin_idx ON model.runs USING brin (created_at);

CREATE TABLE model.run_modules (
    run_id uuid NOT NULL REFERENCES model.runs(id) ON DELETE CASCADE,
    module_key text NOT NULL,
    side text,
    raw_score double precision,
    confidence double precision,
    effective_score double precision,
    multiplier double precision,
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (run_id, module_key)
);

CREATE TABLE model.run_scorelines (
    run_id uuid NOT NULL REFERENCES model.runs(id) ON DELETE CASCADE,
    home_goals smallint NOT NULL,
    away_goals smallint NOT NULL,
    probability double precision NOT NULL CHECK (probability >= 0 AND probability <= 1),
    rank smallint NOT NULL,
    cumulative_probability double precision NOT NULL,
    route text NOT NULL,
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (run_id, home_goals, away_goals)
);
CREATE INDEX run_scorelines_rank_idx ON model.run_scorelines (run_id, rank);

CREATE TABLE review.match_reviews (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    review_version text NOT NULL,
    data_coverage numeric(5,4) NOT NULL CHECK (data_coverage >= 0 AND data_coverage <= 1),
    conclusions jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (match_id, review_version)
);

CREATE TABLE review.player_match_reviews (
    id uuid PRIMARY KEY,
    match_review_id uuid NOT NULL REFERENCES review.match_reviews(id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES football.players(id),
    team_id uuid NOT NULL REFERENCES football.teams(id),
    role_code text,
    started boolean NOT NULL,
    minutes_played smallint,
    expected_performance double precision,
    actual_performance double precision,
    realization_ratio double precision,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (match_review_id, player_id)
);
CREATE INDEX player_match_reviews_player_idx ON review.player_match_reviews (player_id, match_review_id);

CREATE TABLE review.team_match_reviews (
    id uuid PRIMARY KEY,
    match_review_id uuid NOT NULL REFERENCES review.match_reviews(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES football.teams(id),
    chemistry_score double precision,
    bench_strength double precision,
    substitution_impact double precision,
    realization_score double precision,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (match_review_id, team_id)
);

CREATE TABLE review.ability_update_candidates (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id),
    dimension_code text NOT NULL REFERENCES feature.player_ability_dimensions(code),
    current_value double precision,
    proposed_value double precision NOT NULL,
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    sample_size integer NOT NULL DEFAULT 0,
    evidence jsonb NOT NULL,
    calculation_version text NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'superseded')),
    created_at timestamptz NOT NULL DEFAULT now(),
    decided_at timestamptz,
    decision_note text
);
CREATE INDEX ability_candidates_status_idx ON review.ability_update_candidates (status, player_id, created_at DESC);

CREATE TABLE analytics.model_metrics (
    id uuid PRIMARY KEY,
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    competition_id uuid REFERENCES football.competitions(id),
    window_start timestamptz NOT NULL,
    window_end timestamptz NOT NULL,
    sample_size bigint NOT NULL,
    metrics jsonb NOT NULL,
    calculation_version text NOT NULL,
    calculated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE NULLS NOT DISTINCT (model_version_id, parameter_set_id, competition_id, window_start, window_end, calculation_version)
);

CREATE TABLE audit.events (
    id uuid PRIMARY KEY,
    event_type text NOT NULL,
    entity_type text NOT NULL,
    entity_id text,
    actor text NOT NULL DEFAULT 'desktop-client',
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    occurred_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX audit_events_entity_idx ON audit.events (entity_type, entity_id, occurred_at DESC);
CREATE INDEX audit_events_time_brin_idx ON audit.events USING brin (occurred_at);

INSERT INTO football.positions (code, name, position_group, sort_order) VALUES
    ('GK', '门将', 'goalkeeper', 10),
    ('CB', '中后卫', 'defender', 20),
    ('LB', '左后卫', 'defender', 21),
    ('RB', '右后卫', 'defender', 22),
    ('DM', '防守型中场', 'midfielder', 30),
    ('CM', '中场', 'midfielder', 31),
    ('AM', '进攻型中场', 'midfielder', 32),
    ('LW', '左边锋', 'forward', 40),
    ('RW', '右边锋', 'forward', 41),
    ('ST', '中锋', 'forward', 42)
ON CONFLICT (code) DO NOTHING;

INSERT INTO feature.player_ability_dimensions (code, name, category, description) VALUES
    ('attack', '进攻能力', 'core', '综合进攻贡献'),
    ('defence', '防守能力', 'core', '综合防守贡献'),
    ('creation', '创造能力', 'core', '制造机会与关键传球'),
    ('progression', '推进能力', 'core', '带球与传球推进'),
    ('finishing', '终结能力', 'core', '射门转化与终结质量'),
    ('physical', '身体对抗', 'physical', '力量、速度与对抗'),
    ('stamina', '体能', 'physical', '持续输出与恢复'),
    ('stability', '稳定性', 'behavior', '跨比赛表现稳定程度'),
    ('discipline', '纪律性', 'behavior', '犯规、牌与战术纪律'),
    ('tactical_execution', '战术执行', 'behavior', '位置与战术任务执行'),
    ('versatility', '多位置适配', 'role', '多位置与多角色覆盖'),
    ('substitute_impact', '替补影响力', 'role', '替补登场后的比赛影响')
ON CONFLICT (code) DO NOTHING;
