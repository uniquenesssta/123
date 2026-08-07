-- 接入F：四时点任务编排、READY_TO_FREEZE状态机和自动冻结。
-- CONTRACT_SHA256 = b8577c3e80c1d4391839be4f82021018aff79f1421392f7758b8877bf1ba7d3f

ALTER TABLE platform.jobs
    ADD COLUMN available_at timestamptz NOT NULL DEFAULT now();

DROP INDEX IF EXISTS platform.platform_jobs_queue_idx;
DROP INDEX IF EXISTS platform_jobs_queue_idx;
CREATE INDEX platform_jobs_queue_idx
    ON platform.jobs (available_at, priority DESC, created_at, id)
    WHERE status = 'queued' AND cancellation_requested = false;

CREATE TABLE platform.p4_freeze_tasks (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    match_key text NOT NULL,
    horizon text NOT NULL CHECK (horizon IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')),
    kickoff_at timestamptz NOT NULL,
    data_cutoff_at timestamptz NOT NULL,
    research_due_at timestamptz NOT NULL,
    freeze_deadline_at timestamptz NOT NULL,
    rule_package_id uuid NOT NULL REFERENCES model.rule_packages(id),
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    competition_profile_id uuid NOT NULL REFERENCES model.competition_profiles(id),
    research_schema_version_id uuid NOT NULL REFERENCES research.schema_versions(id),
    snapshot_schema_version_id uuid NOT NULL REFERENCES research.schema_versions(id),
    requested_fact_keys text[] NOT NULL CHECK (cardinality(requested_fact_keys) > 0),
    trace_id uuid NOT NULL UNIQUE,
    state text NOT NULL CHECK (state IN (
        'PLANNED', 'RESEARCH_QUEUED', 'RESEARCH_RUNNING', 'RESEARCH_SUCCEEDED',
        'RESEARCH_PARTIAL', 'READY_TO_FREEZE', 'FREEZING', 'FROZEN',
        'BLOCKED', 'MISSED', 'FAILED', 'CANCELLED'
    )),
    research_run_id uuid REFERENCES research.runs(id),
    research_job_id uuid REFERENCES platform.jobs(id),
    freeze_job_id uuid REFERENCES platform.jobs(id),
    snapshot_id uuid REFERENCES feature.snapshots(id),
    blockers jsonb NOT NULL DEFAULT '[]'::jsonb,
    task_fingerprint text NOT NULL CHECK (task_fingerprint ~ '^[0-9a-f]{64}$'),
    idempotency_key text NOT NULL UNIQUE,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (research_due_at <= data_cutoff_at),
    CHECK (data_cutoff_at < kickoff_at),
    CHECK (freeze_deadline_at >= data_cutoff_at),
    UNIQUE (
        match_id, model_version_id, parameter_set_id,
        competition_profile_id, horizon, data_cutoff_at
    )
);

CREATE INDEX p4_freeze_tasks_match_horizon_idx
    ON platform.p4_freeze_tasks (match_id, kickoff_at, horizon);
CREATE INDEX p4_freeze_tasks_state_due_idx
    ON platform.p4_freeze_tasks (state, research_due_at, data_cutoff_at);
CREATE INDEX p4_freeze_tasks_research_run_idx
    ON platform.p4_freeze_tasks (research_run_id)
    WHERE research_run_id IS NOT NULL;

CREATE OR REPLACE FUNCTION platform.guard_p4_freeze_task_identity()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF ROW(
        NEW.match_id, NEW.match_key, NEW.horizon, NEW.kickoff_at,
        NEW.data_cutoff_at, NEW.research_due_at, NEW.freeze_deadline_at,
        NEW.rule_package_id, NEW.model_version_id, NEW.parameter_set_id,
        NEW.competition_profile_id, NEW.research_schema_version_id,
        NEW.snapshot_schema_version_id, NEW.requested_fact_keys,
        NEW.trace_id, NEW.task_fingerprint, NEW.idempotency_key
    ) IS DISTINCT FROM ROW(
        OLD.match_id, OLD.match_key, OLD.horizon, OLD.kickoff_at,
        OLD.data_cutoff_at, OLD.research_due_at, OLD.freeze_deadline_at,
        OLD.rule_package_id, OLD.model_version_id, OLD.parameter_set_id,
        OLD.competition_profile_id, OLD.research_schema_version_id,
        OLD.snapshot_schema_version_id, OLD.requested_fact_keys,
        OLD.trace_id, OLD.task_fingerprint, OLD.idempotency_key
    ) THEN
        RAISE EXCEPTION 'P4 freeze task pinned identity is immutable';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER p4_freeze_tasks_identity_guard
BEFORE UPDATE ON platform.p4_freeze_tasks
FOR EACH ROW EXECUTE FUNCTION platform.guard_p4_freeze_task_identity();

CREATE TABLE platform.p4_freeze_task_events (
    id uuid PRIMARY KEY,
    task_id uuid NOT NULL REFERENCES platform.p4_freeze_tasks(id) ON DELETE CASCADE,
    from_state text CHECK (from_state IS NULL OR from_state IN (
        'PLANNED', 'RESEARCH_QUEUED', 'RESEARCH_RUNNING', 'RESEARCH_SUCCEEDED',
        'RESEARCH_PARTIAL', 'READY_TO_FREEZE', 'FREEZING', 'FROZEN',
        'BLOCKED', 'MISSED', 'FAILED', 'CANCELLED'
    )),
    to_state text NOT NULL CHECK (to_state IN (
        'PLANNED', 'RESEARCH_QUEUED', 'RESEARCH_RUNNING', 'RESEARCH_SUCCEEDED',
        'RESEARCH_PARTIAL', 'READY_TO_FREEZE', 'FREEZING', 'FROZEN',
        'BLOCKED', 'MISSED', 'FAILED', 'CANCELLED'
    )),
    reason text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    idempotency_key text NOT NULL,
    event_fingerprint text NOT NULL CHECK (event_fingerprint ~ '^[0-9a-f]{64}$'),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (task_id, idempotency_key)
);

CREATE INDEX p4_freeze_task_events_task_time_idx
    ON platform.p4_freeze_task_events (task_id, occurred_at, id);

CREATE TRIGGER p4_freeze_task_events_immutable
BEFORE UPDATE OR DELETE ON platform.p4_freeze_task_events
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-four-horizon-orchestration'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-four-horizon-orchestration',
            '1.0.0',
            '0.11.0',
            '0.12.0',
            'football.p4-orchestration-contract.v1',
            'b8577c3e80c1d4391839be4f82021018aff79f1421392f7758b8877bf1ba7d3f',
            'F',
            jsonb_build_object(
                'contract_path', 'contracts/model-orchestration-contract.json',
                'canonical_horizons', jsonb_build_array('T-24h', 'T-6h', 'T-90m', 'T-1h'),
                'ready_state', 'READY_TO_FREEZE',
                'formal_terminal_state', 'FROZEN',
                'legacy_horizon', 'T-N'
            )
        );
    ELSIF existing_hash <> 'b8577c3e80c1d4391839be4f82021018aff79f1421392f7758b8877bf1ba7d3f' THEN
        RAISE EXCEPTION 'P4 orchestration contract hash conflict: existing %, expected %',
            existing_hash, 'b8577c3e80c1d4391839be4f82021018aff79f1421392f7758b8877bf1ba7d3f';
    END IF;
END;
$migration$;
