ALTER TABLE model.runs
    ADD COLUMN IF NOT EXISTS input_audit_version text,
    ADD COLUMN IF NOT EXISTS input_readiness_level text,
    ADD COLUMN IF NOT EXISTS input_readiness_score smallint,
    ADD COLUMN IF NOT EXISTS input_manifest jsonb,
    ADD COLUMN IF NOT EXISTS input_manifest_sha256 text;

UPDATE model.runs
SET input_audit_version = COALESCE(input_audit_version, 'legacy-run-v0'),
    input_readiness_level = COALESCE(input_readiness_level, 'legacy_unknown'),
    input_manifest = COALESCE(input_manifest, input_payload),
    input_manifest_sha256 = COALESCE(input_manifest_sha256, input_sha256)
WHERE input_audit_version IS NULL
   OR input_readiness_level IS NULL
   OR input_manifest IS NULL
   OR input_manifest_sha256 IS NULL;

ALTER TABLE model.runs
    ALTER COLUMN input_audit_version SET NOT NULL,
    ALTER COLUMN input_readiness_level SET NOT NULL,
    ALTER COLUMN input_manifest SET NOT NULL,
    ALTER COLUMN input_manifest_sha256 SET NOT NULL,
    DROP CONSTRAINT IF EXISTS model_runs_input_readiness_level_check,
    ADD CONSTRAINT model_runs_input_readiness_level_check CHECK (
        input_readiness_level IN (
            'formal_ready', 'ready_with_warnings', 'shadow_only',
            'blocked', 'not_assessed', 'legacy_unknown'
        )
    ),
    DROP CONSTRAINT IF EXISTS model_runs_input_readiness_score_check,
    ADD CONSTRAINT model_runs_input_readiness_score_check CHECK (
        input_readiness_score IS NULL OR input_readiness_score BETWEEN 0 AND 100
    ),
    DROP CONSTRAINT IF EXISTS model_runs_input_manifest_sha256_check,
    ADD CONSTRAINT model_runs_input_manifest_sha256_check CHECK (
        input_manifest_sha256 ~ '^[0-9a-f]{64}$'
    );

CREATE INDEX IF NOT EXISTS model_runs_input_readiness_idx
    ON model.runs (input_readiness_level, created_at DESC);

CREATE OR REPLACE FUNCTION model.reject_model_run_input_identity_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.match_id IS DISTINCT FROM OLD.match_id
       OR NEW.match_key IS DISTINCT FROM OLD.match_key
       OR NEW.feature_snapshot_id IS DISTINCT FROM OLD.feature_snapshot_id
       OR NEW.model_version_id IS DISTINCT FROM OLD.model_version_id
       OR NEW.parameter_set_id IS DISTINCT FROM OLD.parameter_set_id
       OR NEW.rule_package_id IS DISTINCT FROM OLD.rule_package_id
       OR NEW.route_binding_id IS DISTINCT FROM OLD.route_binding_id
       OR NEW.snapshot_type IS DISTINCT FROM OLD.snapshot_type
       OR NEW.route_reason IS DISTINCT FROM OLD.route_reason
       OR NEW.input_payload IS DISTINCT FROM OLD.input_payload
       OR NEW.input_sha256 IS DISTINCT FROM OLD.input_sha256
       OR NEW.input_audit_version IS DISTINCT FROM OLD.input_audit_version
       OR NEW.input_readiness_level IS DISTINCT FROM OLD.input_readiness_level
       OR NEW.input_readiness_score IS DISTINCT FROM OLD.input_readiness_score
       OR NEW.input_manifest IS DISTINCT FROM OLD.input_manifest
       OR NEW.input_manifest_sha256 IS DISTINCT FROM OLD.input_manifest_sha256
       OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'model run input identity is immutable';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS model_runs_input_identity_immutable ON model.runs;
CREATE TRIGGER model_runs_input_identity_immutable
BEFORE UPDATE ON model.runs
FOR EACH ROW EXECUTE FUNCTION model.reject_model_run_input_identity_mutation();
