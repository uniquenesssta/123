-- 接入点 J：全链路验收、性能、安全、成本与发布报告。
-- 验收运行和检查项均为不可变发布证据；历史 A-I 契约保持原样。

CREATE TABLE IF NOT EXISTS platform.release_acceptance_runs (
    id uuid PRIMARY KEY,
    app_version text NOT NULL,
    contract_version text NOT NULL,
    fixture_version text NOT NULL,
    overall_status text NOT NULL CHECK (overall_status IN ('pass', 'warning', 'blocked')),
    started_at timestamptz NOT NULL,
    completed_at timestamptz NOT NULL,
    requested_by text,
    report_sha256 text NOT NULL CHECK (report_sha256 ~ '^[0-9a-f]{64}$'),
    passed_count integer NOT NULL CHECK (passed_count >= 0),
    warning_count integer NOT NULL CHECK (warning_count >= 0),
    blocked_count integer NOT NULL CHECK (blocked_count >= 0),
    category_summaries jsonb NOT NULL DEFAULT '[]'::jsonb,
    performance_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
    cost_summary jsonb NOT NULL DEFAULT '{}'::jsonb,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (completed_at >= started_at)
);
CREATE INDEX IF NOT EXISTS release_acceptance_runs_completed_idx
    ON platform.release_acceptance_runs (completed_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS platform.release_acceptance_checks (
    id uuid PRIMARY KEY,
    run_id uuid NOT NULL REFERENCES platform.release_acceptance_runs(id),
    sequence_no integer NOT NULL CHECK (sequence_no > 0),
    category text NOT NULL CHECK (category IN ('chain', 'performance', 'security', 'cost', 'release')),
    check_code text NOT NULL,
    title text NOT NULL,
    status text NOT NULL CHECK (status IN ('pass', 'warning', 'blocked')),
    summary text NOT NULL,
    remediation text,
    evidence jsonb NOT NULL DEFAULT '{}'::jsonb,
    duration_ms bigint NOT NULL DEFAULT 0 CHECK (duration_ms >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (run_id, sequence_no),
    UNIQUE (run_id, check_code)
);
CREATE INDEX IF NOT EXISTS release_acceptance_checks_status_idx
    ON platform.release_acceptance_checks (run_id, status, category, sequence_no);

DROP TRIGGER IF EXISTS release_acceptance_runs_immutable ON platform.release_acceptance_runs;
CREATE TRIGGER release_acceptance_runs_immutable
BEFORE UPDATE OR DELETE ON platform.release_acceptance_runs
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

DROP TRIGGER IF EXISTS release_acceptance_checks_immutable ON platform.release_acceptance_checks;
CREATE TRIGGER release_acceptance_checks_immutable
BEFORE UPDATE OR DELETE ON platform.release_acceptance_checks
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-release-acceptance'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-release-acceptance', '1.0.0', '0.22.0', '0.23.0',
            'football.release-acceptance.v1',
            '5ca32f5a6ef1e66de55fa121372da7c2d931906005aa97085d08e35330b49938', 'J',
            jsonb_build_object(
                'contract_path', 'contracts/release-acceptance-contract.json',
                'acceptance_mode', 'fixed_fixture_and_runtime',
                'categories', jsonb_build_array('chain','performance','security','cost','release'),
                'immutable_reports', true,
                'ui_reference', 'match_center',
                'ui_aligned_pages', jsonb_build_array('teams','players','prediction'),
                'automatic_parameter_promotion', false,
                'p4_4_state', 'SHADOW_ONLY'
            )
        );
    ELSIF existing_hash <> '5ca32f5a6ef1e66de55fa121372da7c2d931906005aa97085d08e35330b49938' THEN
        RAISE EXCEPTION 'release acceptance contract hash conflict: existing %, expected %',
            existing_hash, '5ca32f5a6ef1e66de55fa121372da7c2d931906005aa97085d08e35330b49938';
    END IF;
END;
$migration$;
