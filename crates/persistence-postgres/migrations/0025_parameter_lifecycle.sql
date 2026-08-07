-- 接入点 I：候选参数校准、影子验证、人工晋升与绑定级回滚。
-- 接入点 H 的赛果结算、证据队列与漂移契约是运行态硬前置；缺失时只允许查看和生成草案，不允许影子通过或晋升。

ALTER TABLE analytics.parameter_tuning_candidates
    DROP CONSTRAINT IF EXISTS parameter_tuning_candidates_status_check;

ALTER TABLE analytics.parameter_tuning_candidates
    ADD COLUMN IF NOT EXISTS competition_profile_id uuid REFERENCES model.competition_profiles(id),
    ADD COLUMN IF NOT EXISTS partition_key text,
    ADD COLUMN IF NOT EXISTS baseline_model_version_id uuid REFERENCES model.versions(id),
    ADD COLUMN IF NOT EXISTS baseline_parameter_set_id uuid REFERENCES model.parameter_sets(id),
    ADD COLUMN IF NOT EXISTS candidate_model_version_id uuid REFERENCES model.versions(id),
    ADD COLUMN IF NOT EXISTS candidate_parameter_set_id uuid REFERENCES model.parameter_sets(id),
    ADD COLUMN IF NOT EXISTS candidate_model_version text,
    ADD COLUMN IF NOT EXISTS candidate_parameter_version text,
    ADD COLUMN IF NOT EXISTS candidate_definition_sha256 text,
    ADD COLUMN IF NOT EXISTS training_window jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS validation_window jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS holdout_window jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE analytics.parameter_tuning_candidates
    ADD CONSTRAINT parameter_tuning_candidates_status_check CHECK (status IN (
        'pending', 'accepted_for_backtest', 'rejected', 'shadow_running',
        'shadow_passed', 'shadow_failed', 'promoted', 'rolled_back',
        'blocked_by_h', 'superseded'
    ));

DO $constraints$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'parameter_tuning_candidate_hash_check'
          AND conrelid = 'analytics.parameter_tuning_candidates'::regclass
    ) THEN
        ALTER TABLE analytics.parameter_tuning_candidates
            ADD CONSTRAINT parameter_tuning_candidate_hash_check
            CHECK (candidate_definition_sha256 IS NULL OR candidate_definition_sha256 ~ '^[0-9a-f]{64}$');
    END IF;
END;
$constraints$;

CREATE INDEX IF NOT EXISTS parameter_tuning_candidates_partition_idx
    ON analytics.parameter_tuning_candidates (partition_key, status, created_at DESC);

CREATE TABLE IF NOT EXISTS analytics.parameter_shadow_validations (
    id uuid PRIMARY KEY,
    candidate_id uuid NOT NULL REFERENCES analytics.parameter_tuning_candidates(id),
    validation_key text NOT NULL,
    partition_key text NOT NULL,
    sample_count bigint NOT NULL CHECK (sample_count >= 0),
    baseline_metrics jsonb NOT NULL,
    candidate_metrics jsonb NOT NULL,
    metric_deltas jsonb NOT NULL,
    gate_results jsonb NOT NULL,
    status text NOT NULL CHECK (status IN ('passed', 'failed', 'blocked')),
    generated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (candidate_id, validation_key)
);

CREATE INDEX IF NOT EXISTS parameter_shadow_validations_candidate_idx
    ON analytics.parameter_shadow_validations (candidate_id, generated_at DESC);

CREATE TABLE IF NOT EXISTS analytics.parameter_promotion_decisions (
    id uuid PRIMARY KEY,
    candidate_id uuid NOT NULL REFERENCES analytics.parameter_tuning_candidates(id),
    decision text NOT NULL CHECK (decision IN ('promote', 'rollback')),
    previous_binding_state jsonb NOT NULL,
    new_binding_state jsonb NOT NULL,
    decided_by text,
    decision_note text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS parameter_promotion_decisions_candidate_idx
    ON analytics.parameter_promotion_decisions (candidate_id, created_at DESC);

CREATE TABLE IF NOT EXISTS analytics.parameter_binding_changes (
    decision_id uuid NOT NULL REFERENCES analytics.parameter_promotion_decisions(id) ON DELETE CASCADE,
    binding_id uuid NOT NULL REFERENCES model.competition_bindings(id),
    previous_model_version_id uuid NOT NULL REFERENCES model.versions(id),
    previous_parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    new_model_version_id uuid NOT NULL REFERENCES model.versions(id),
    new_parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    PRIMARY KEY (decision_id, binding_id)
);

CREATE OR REPLACE FUNCTION analytics.reject_parameter_lifecycle_record_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    RAISE EXCEPTION 'parameter lifecycle ledger %.% is immutable', TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$function$;

DROP TRIGGER IF EXISTS parameter_shadow_validations_immutable
    ON analytics.parameter_shadow_validations;
CREATE TRIGGER parameter_shadow_validations_immutable
BEFORE UPDATE OR DELETE ON analytics.parameter_shadow_validations
FOR EACH ROW EXECUTE FUNCTION analytics.reject_parameter_lifecycle_record_mutation();

DROP TRIGGER IF EXISTS parameter_promotion_decisions_immutable
    ON analytics.parameter_promotion_decisions;
CREATE TRIGGER parameter_promotion_decisions_immutable
BEFORE UPDATE OR DELETE ON analytics.parameter_promotion_decisions
FOR EACH ROW EXECUTE FUNCTION analytics.reject_parameter_lifecycle_record_mutation();

DROP TRIGGER IF EXISTS parameter_binding_changes_immutable
    ON analytics.parameter_binding_changes;
CREATE TRIGGER parameter_binding_changes_immutable
BEFORE UPDATE OR DELETE ON analytics.parameter_binding_changes
FOR EACH ROW EXECUTE FUNCTION analytics.reject_parameter_lifecycle_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-parameter-lifecycle'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-parameter-lifecycle', '1.0.0', '0.20.0', '0.21.0',
            'football.model-parameter-lifecycle.v1',
            '439c22ace0712e035721a1b754cac616faf77a31df7575720bc09fe2e1867e1b', 'I',
            jsonb_build_object(
                'contract_path', 'contracts/parameter-lifecycle-contract.json',
                'required_h_contract_key', 'p4-postmatch-settlement',
                'formal_partition', 'model_version x competition_profile x horizon',
                'automatic_promotion', false,
                'immutable_candidate_versions', true,
                'binding_level_rollback', true,
                'provider_state', 'NOT_BUNDLED'
            )
        );
    ELSIF existing_hash <> '439c22ace0712e035721a1b754cac616faf77a31df7575720bc09fe2e1867e1b' THEN
        RAISE EXCEPTION 'parameter lifecycle contract hash conflict: existing %, expected %',
            existing_hash, '439c22ace0712e035721a1b754cac616faf77a31df7575720bc09fe2e1867e1b';
    END IF;
END;
$migration$;
