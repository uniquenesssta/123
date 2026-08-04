-- 第五阶段：长期模型分析、数据质量、AI 分析包、持久化后台任务和大规模导入基础。

ALTER TABLE platform.jobs
    ADD COLUMN priority integer NOT NULL DEFAULT 0,
    ADD COLUMN attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    ADD COLUMN max_attempts integer NOT NULL DEFAULT 3 CHECK (max_attempts BETWEEN 1 AND 20),
    ADD COLUMN cancellation_requested boolean NOT NULL DEFAULT false,
    ADD COLUMN heartbeat_at timestamptz,
    ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();

CREATE INDEX platform_jobs_queue_idx
    ON platform.jobs (priority DESC, created_at, id)
    WHERE status = 'queued' AND cancellation_requested = false;
CREATE INDEX platform_jobs_status_updated_idx
    ON platform.jobs (status, updated_at DESC);

CREATE TABLE platform.job_events (
    id uuid PRIMARY KEY,
    job_id uuid NOT NULL REFERENCES platform.jobs(id) ON DELETE CASCADE,
    event_type text NOT NULL,
    progress double precision CHECK (progress IS NULL OR progress BETWEEN 0 AND 100),
    message text,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    occurred_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX job_events_job_time_idx ON platform.job_events (job_id, occurred_at DESC);

CREATE TABLE analytics.evaluation_samples (
    id uuid NOT NULL,
    review_id uuid NOT NULL REFERENCES review.match_reviews(id) ON DELETE CASCADE,
    run_id uuid NOT NULL REFERENCES model.runs(id) ON DELETE CASCADE,
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    competition_id uuid REFERENCES football.competitions(id),
    season_id uuid REFERENCES football.seasons(id),
    stage_id uuid REFERENCES football.competition_stages(id),
    snapshot_type text NOT NULL,
    kickoff_time timestamptz NOT NULL,
    actual_outcome text NOT NULL CHECK (actual_outcome IN ('home_win', 'draw', 'away_win')),
    home_win double precision NOT NULL CHECK (home_win BETWEEN 0 AND 1),
    draw double precision NOT NULL CHECK (draw BETWEEN 0 AND 1),
    away_win double precision NOT NULL CHECK (away_win BETWEEN 0 AND 1),
    log_loss double precision NOT NULL CHECK (log_loss >= 0),
    brier double precision NOT NULL CHECK (brier >= 0),
    scoreline_nll double precision,
    data_coverage double precision NOT NULL CHECK (data_coverage BETWEEN 0 AND 1),
    calculation_version text NOT NULL,
    calculated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id, kickoff_time)
) PARTITION BY RANGE (kickoff_time);

DO $$
DECLARE
    target_year integer;
BEGIN
    FOR target_year IN 2010..2040 LOOP
        EXECUTE format(
            'CREATE TABLE analytics.evaluation_samples_y%s PARTITION OF analytics.evaluation_samples FOR VALUES FROM (%L) TO (%L)',
            target_year,
            make_timestamptz(target_year, 1, 1, 0, 0, 0, 'UTC'),
            make_timestamptz(target_year + 1, 1, 1, 0, 0, 0, 'UTC')
        );
    END LOOP;
END $$;

CREATE TABLE analytics.evaluation_samples_default
    PARTITION OF analytics.evaluation_samples DEFAULT;

CREATE UNIQUE INDEX evaluation_samples_run_version_uidx
    ON analytics.evaluation_samples (run_id, kickoff_time, calculation_version);
CREATE INDEX evaluation_samples_run_idx
    ON analytics.evaluation_samples (run_id, kickoff_time DESC);
CREATE INDEX evaluation_samples_model_window_idx
    ON analytics.evaluation_samples (model_version_id, parameter_set_id, kickoff_time DESC);
CREATE INDEX evaluation_samples_competition_window_idx
    ON analytics.evaluation_samples (competition_id, kickoff_time DESC);
CREATE INDEX evaluation_samples_kickoff_brin_idx
    ON analytics.evaluation_samples USING brin (kickoff_time);

CREATE TABLE analytics.calibration_buckets (
    id uuid PRIMARY KEY,
    snapshot_id uuid NOT NULL,
    model_version_id uuid REFERENCES model.versions(id),
    parameter_set_id uuid REFERENCES model.parameter_sets(id),
    competition_id uuid REFERENCES football.competitions(id),
    outcome text NOT NULL CHECK (outcome IN ('home_win', 'draw', 'away_win')),
    bucket_index smallint NOT NULL,
    lower_bound double precision NOT NULL,
    upper_bound double precision NOT NULL,
    sample_size bigint NOT NULL,
    predicted_mean double precision NOT NULL,
    actual_rate double precision NOT NULL,
    absolute_gap double precision NOT NULL,
    ece_component double precision NOT NULL,
    window_start timestamptz,
    window_end timestamptz,
    calculation_version text NOT NULL,
    calculated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX calibration_snapshot_idx ON analytics.calibration_buckets (snapshot_id, outcome, bucket_index);

CREATE TABLE analytics.drift_snapshots (
    id uuid PRIMARY KEY,
    snapshot_id uuid NOT NULL,
    competition_id uuid REFERENCES football.competitions(id),
    metric_name text NOT NULL,
    baseline_mean double precision NOT NULL,
    current_mean double precision NOT NULL,
    absolute_delta double precision NOT NULL,
    relative_delta double precision,
    baseline_size bigint NOT NULL,
    current_size bigint NOT NULL,
    severity text NOT NULL CHECK (severity IN ('stable', 'warning', 'critical')),
    direction text NOT NULL CHECK (direction IN ('up', 'down', 'flat')),
    details jsonb NOT NULL DEFAULT '{}'::jsonb,
    calculation_version text NOT NULL,
    calculated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX drift_snapshot_idx ON analytics.drift_snapshots (snapshot_id, severity, metric_name);

CREATE TABLE analytics.analysis_snapshots (
    id uuid PRIMARY KEY,
    competition_id uuid REFERENCES football.competitions(id),
    window_start timestamptz,
    window_end timestamptz,
    sample_size bigint NOT NULL,
    summary jsonb NOT NULL,
    calculation_version text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX analysis_snapshots_scope_idx
    ON analytics.analysis_snapshots (competition_id, created_at DESC);

ALTER TABLE analytics.calibration_buckets
    ADD CONSTRAINT calibration_buckets_snapshot_fk
    FOREIGN KEY (snapshot_id) REFERENCES analytics.analysis_snapshots(id) ON DELETE CASCADE;
ALTER TABLE analytics.drift_snapshots
    ADD CONSTRAINT drift_snapshots_snapshot_fk
    FOREIGN KEY (snapshot_id) REFERENCES analytics.analysis_snapshots(id) ON DELETE CASCADE;

CREATE TABLE analytics.data_quality_scans (
    id uuid PRIMARY KEY,
    status text NOT NULL CHECK (status IN ('running', 'succeeded', 'failed')),
    scope jsonb NOT NULL DEFAULT '{}'::jsonb,
    summary jsonb,
    error_message text,
    started_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz
);

CREATE TABLE analytics.data_quality_findings (
    id uuid PRIMARY KEY,
    scan_id uuid NOT NULL REFERENCES analytics.data_quality_scans(id) ON DELETE CASCADE,
    severity text NOT NULL CHECK (severity IN ('critical', 'warning', 'info')),
    category text NOT NULL,
    finding_code text NOT NULL,
    entity_type text NOT NULL,
    entity_id text,
    message text NOT NULL,
    evidence jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'ignored')),
    detected_at timestamptz NOT NULL DEFAULT now(),
    resolved_at timestamptz,
    resolution_note text
);
CREATE INDEX data_quality_open_idx
    ON analytics.data_quality_findings (status, severity, detected_at DESC);
CREATE INDEX data_quality_scan_idx
    ON analytics.data_quality_findings (scan_id, severity, category);

CREATE TABLE analytics.query_performance_snapshots (
    id uuid PRIMARY KEY,
    database_size_bytes bigint NOT NULL,
    tables jsonb NOT NULL,
    recommendations jsonb NOT NULL DEFAULT '[]'::jsonb,
    captured_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE analytics.ai_package_exports (
    id uuid PRIMARY KEY,
    package_id uuid NOT NULL UNIQUE,
    output_path text NOT NULL,
    content_sha256 text NOT NULL,
    sample_size bigint NOT NULL,
    calculation_version text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE analytics.ai_package_imports (
    id uuid PRIMARY KEY,
    response_id uuid NOT NULL UNIQUE,
    source_package_id uuid REFERENCES analytics.ai_package_exports(package_id),
    input_path text NOT NULL,
    content_sha256 text NOT NULL,
    suggestion_count integer NOT NULL,
    status text NOT NULL CHECK (status IN ('previewed', 'imported', 'rejected')),
    warnings jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    imported_at timestamptz
);

CREATE TABLE analytics.ai_suggestions (
    id uuid PRIMARY KEY,
    response_id uuid NOT NULL REFERENCES analytics.ai_package_imports(response_id) ON DELETE CASCADE,
    suggestion_type text NOT NULL,
    title text NOT NULL,
    summary text NOT NULL,
    severity text NOT NULL DEFAULT 'info',
    scope jsonb NOT NULL DEFAULT '{}'::jsonb,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    evidence jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'applied')),
    linked_candidate_id uuid REFERENCES review.ability_update_candidates(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    decided_at timestamptz,
    decided_by text,
    decision_note text
);
CREATE INDEX ai_suggestions_status_idx
    ON analytics.ai_suggestions (status, severity, created_at DESC);
CREATE INDEX ai_suggestions_response_idx
    ON analytics.ai_suggestions (response_id, created_at);

CREATE UNLOGGED TABLE catalog.bulk_import_staging (
    batch_id uuid NOT NULL,
    row_number bigint NOT NULL,
    entity_type text NOT NULL,
    payload jsonb NOT NULL,
    payload_sha256 text,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (batch_id, row_number)
);
CREATE INDEX bulk_import_staging_batch_idx ON catalog.bulk_import_staging (batch_id, entity_type);

CREATE TABLE catalog.bulk_import_runs (
    id uuid PRIMARY KEY,
    batch_id uuid NOT NULL UNIQUE,
    import_type text NOT NULL,
    source_path text,
    row_count bigint NOT NULL DEFAULT 0,
    copy_duration_ms bigint,
    validation_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL CHECK (status IN ('staged', 'validated', 'committed', 'failed', 'cancelled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz
);

CREATE TABLE analytics.partition_registry (
    schema_name text NOT NULL,
    parent_table text NOT NULL,
    partition_name text NOT NULL,
    range_start timestamptz,
    range_end timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'detached', 'archived')),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (schema_name, partition_name)
);

INSERT INTO analytics.partition_registry (
    schema_name, parent_table, partition_name, range_start, range_end, status
)
SELECT
    'analytics',
    'evaluation_samples',
    format('evaluation_samples_y%s', target_year),
    make_timestamptz(target_year, 1, 1, 0, 0, 0, 'UTC'),
    make_timestamptz(target_year + 1, 1, 1, 0, 0, 0, 'UTC'),
    'active'
FROM generate_series(2010, 2040) AS target_year
ON CONFLICT DO NOTHING;

INSERT INTO analytics.partition_registry (
    schema_name, parent_table, partition_name, range_start, range_end, status
) VALUES (
    'analytics', 'evaluation_samples', 'evaluation_samples_default', NULL, NULL, 'active'
) ON CONFLICT DO NOTHING;
