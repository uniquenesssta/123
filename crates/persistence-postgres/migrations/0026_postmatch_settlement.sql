-- 接入点 H：赛果结算、证据评分队列、供应商评分和正式分区漂移监控。
-- 所有正式统计继续严格隔离 model_version × competition_profile × parameter_version × horizon；P4.4 保持 SHADOW_ONLY。

CREATE TABLE IF NOT EXISTS review.postmatch_settlements (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    match_review_id uuid NOT NULL REFERENCES review.match_reviews(id),
    model_run_id uuid NOT NULL REFERENCES model.runs(id),
    feature_snapshot_id uuid NOT NULL REFERENCES feature.snapshots(id),
    competition_id uuid NOT NULL REFERENCES football.competitions(id),
    competition_profile_id uuid NOT NULL REFERENCES model.competition_profiles(id),
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    rule_package_id uuid NOT NULL REFERENCES model.rule_packages(id),
    horizon text NOT NULL CHECK (horizon IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')),
    home_goals_90 smallint NOT NULL CHECK (home_goals_90 >= 0),
    away_goals_90 smallint NOT NULL CHECK (away_goals_90 >= 0),
    result_finalized_at timestamptz NOT NULL,
    result_fingerprint text NOT NULL CHECK (result_fingerprint ~ '^[0-9a-f]{64}$'),
    settlement_key text NOT NULL UNIQUE,
    settlement_version text NOT NULL,
    status text NOT NULL DEFAULT 'settled' CHECK (status = 'settled'),
    settled_by text,
    settlement_note text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    settled_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (match_review_id, model_run_id)
);
CREATE INDEX IF NOT EXISTS postmatch_settlements_partition_idx
    ON review.postmatch_settlements (
        competition_id, competition_profile_id, model_version_id,
        parameter_set_id, horizon, settled_at DESC
    );
CREATE INDEX IF NOT EXISTS postmatch_settlements_match_idx
    ON review.postmatch_settlements (match_id, settled_at DESC);

CREATE TABLE IF NOT EXISTS review.evidence_scoring_items (
    id uuid PRIMARY KEY,
    settlement_id uuid NOT NULL REFERENCES review.postmatch_settlements(id),
    evidence_id uuid NOT NULL REFERENCES research.evidence_claims(id),
    provider_id uuid REFERENCES catalog.data_providers(id),
    source_document_id uuid REFERENCES catalog.source_documents(id),
    field_key text NOT NULL,
    verification_state text NOT NULL CHECK (verification_state IN (
        'CONFIRMED', 'PROBABLE', 'CONFLICT', 'NOT_FOUND', 'STALE', 'NOT_APPLICABLE'
    )),
    source_tier text NOT NULL,
    published_at timestamptz,
    retrieved_at timestamptz NOT NULL,
    data_cutoff_at timestamptz NOT NULL,
    timeliness_score double precision NOT NULL CHECK (timeliness_score BETWEEN 0 AND 1),
    item_fingerprint text NOT NULL CHECK (item_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (settlement_id, evidence_id, field_key)
);
CREATE INDEX IF NOT EXISTS evidence_scoring_items_settlement_idx
    ON review.evidence_scoring_items (settlement_id, created_at);
CREATE INDEX IF NOT EXISTS evidence_scoring_items_provider_idx
    ON review.evidence_scoring_items (provider_id, created_at DESC)
    WHERE provider_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS review.evidence_scoring_decisions (
    id uuid PRIMARY KEY,
    item_id uuid NOT NULL UNIQUE REFERENCES review.evidence_scoring_items(id),
    verdict text NOT NULL CHECK (verdict IN ('correct', 'partial', 'incorrect', 'not_verifiable')),
    accuracy_score double precision CHECK (accuracy_score IS NULL OR accuracy_score BETWEEN 0 AND 1),
    reliability_score double precision CHECK (reliability_score IS NULL OR reliability_score BETWEEN 0 AND 1),
    decided_by text,
    decision_note text NOT NULL,
    decision_fingerprint text NOT NULL CHECK (decision_fingerprint ~ '^[0-9a-f]{64}$'),
    decided_at timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (verdict = 'not_verifiable' AND accuracy_score IS NULL AND reliability_score IS NULL)
        OR (verdict <> 'not_verifiable' AND accuracy_score IS NOT NULL AND reliability_score IS NOT NULL)
    )
);
CREATE INDEX IF NOT EXISTS evidence_scoring_decisions_time_idx
    ON review.evidence_scoring_decisions (decided_at DESC);

CREATE TABLE IF NOT EXISTS analytics.provider_score_snapshots (
    id uuid PRIMARY KEY,
    provider_id uuid NOT NULL REFERENCES catalog.data_providers(id),
    scope_key text NOT NULL,
    competition_id uuid NOT NULL REFERENCES football.competitions(id),
    competition_profile_id uuid NOT NULL REFERENCES model.competition_profiles(id),
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    horizon text NOT NULL CHECK (horizon IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')),
    sample_size bigint NOT NULL CHECK (sample_size >= 0),
    correct_count bigint NOT NULL CHECK (correct_count >= 0),
    partial_count bigint NOT NULL CHECK (partial_count >= 0),
    incorrect_count bigint NOT NULL CHECK (incorrect_count >= 0),
    not_verifiable_count bigint NOT NULL CHECK (not_verifiable_count >= 0),
    accuracy_mean double precision NOT NULL CHECK (accuracy_mean BETWEEN 0 AND 1),
    timeliness_mean double precision NOT NULL CHECK (timeliness_mean BETWEEN 0 AND 1),
    reliability_mean double precision NOT NULL CHECK (reliability_mean BETWEEN 0 AND 1),
    weighted_score double precision NOT NULL CHECK (weighted_score BETWEEN 0 AND 1),
    decision_set_sha256 text NOT NULL CHECK (decision_set_sha256 ~ '^[0-9a-f]{64}$'),
    snapshot_key text NOT NULL UNIQUE,
    calculation_version text NOT NULL,
    generated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS provider_score_snapshots_scope_idx
    ON analytics.provider_score_snapshots (
        competition_id, competition_profile_id, model_version_id,
        parameter_set_id, horizon, generated_at DESC
    );
CREATE INDEX IF NOT EXISTS provider_score_snapshots_provider_idx
    ON analytics.provider_score_snapshots (provider_id, generated_at DESC);

CREATE TABLE IF NOT EXISTS analytics.postmatch_drift_runs (
    id uuid PRIMARY KEY,
    competition_id uuid NOT NULL REFERENCES football.competitions(id),
    competition_profile_id uuid NOT NULL REFERENCES model.competition_profiles(id),
    model_version_id uuid NOT NULL REFERENCES model.versions(id),
    parameter_set_id uuid NOT NULL REFERENCES model.parameter_sets(id),
    horizon text NOT NULL CHECK (horizon IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')),
    partition_key text NOT NULL,
    baseline_size bigint NOT NULL CHECK (baseline_size >= 0),
    current_size bigint NOT NULL CHECK (current_size >= 0),
    baseline_window jsonb NOT NULL DEFAULT '{}'::jsonb,
    current_window jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL CHECK (status IN ('insufficient', 'stable', 'warning', 'critical')),
    run_key text NOT NULL UNIQUE,
    calculation_version text NOT NULL,
    generated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS postmatch_drift_runs_partition_idx
    ON analytics.postmatch_drift_runs (partition_key, generated_at DESC);

CREATE TABLE IF NOT EXISTS analytics.postmatch_drift_findings (
    run_id uuid NOT NULL REFERENCES analytics.postmatch_drift_runs(id),
    metric_name text NOT NULL,
    baseline_mean double precision NOT NULL,
    current_mean double precision NOT NULL,
    absolute_delta double precision NOT NULL,
    relative_delta double precision,
    severity text NOT NULL CHECK (severity IN ('stable', 'warning', 'critical')),
    direction text NOT NULL CHECK (direction IN ('up', 'down', 'flat')),
    PRIMARY KEY (run_id, metric_name)
);

ALTER TABLE analytics.evaluation_samples
    ADD COLUMN IF NOT EXISTS settlement_id uuid REFERENCES review.postmatch_settlements(id),
    ADD COLUMN IF NOT EXISTS competition_profile_id uuid REFERENCES model.competition_profiles(id);
CREATE INDEX IF NOT EXISTS evaluation_samples_formal_partition_idx
    ON analytics.evaluation_samples (
        model_version_id, competition_profile_id, parameter_set_id,
        snapshot_type, kickoff_time DESC
    ) WHERE settlement_id IS NOT NULL;

CREATE OR REPLACE FUNCTION analytics.guard_postmatch_evaluation_sample()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    linked_review_id uuid;
    linked_run_id uuid;
    linked_model_version_id uuid;
    linked_parameter_set_id uuid;
    linked_competition_id uuid;
    linked_profile_id uuid;
    linked_horizon text;
    linked_kickoff_time timestamptz;
    linked_actual_outcome text;
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.settlement_id IS NOT NULL THEN
            RAISE EXCEPTION 'formal postmatch evaluation sample is immutable';
        END IF;
        RETURN OLD;
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF OLD.settlement_id IS NOT NULL OR NEW.settlement_id IS NOT NULL THEN
            RAISE EXCEPTION 'formal postmatch evaluation sample is immutable';
        END IF;
        RETURN NEW;
    END IF;
    IF NEW.settlement_id IS NULL THEN
        RETURN NEW;
    END IF;

    SELECT settlement.match_review_id, settlement.model_run_id,
           settlement.model_version_id, settlement.parameter_set_id,
           settlement.competition_id, settlement.competition_profile_id,
           settlement.horizon, fixture.kickoff_time,
           CASE
               WHEN settlement.home_goals_90 > settlement.away_goals_90 THEN 'home_win'
               WHEN settlement.home_goals_90 < settlement.away_goals_90 THEN 'away_win'
               ELSE 'draw'
           END
    INTO linked_review_id, linked_run_id, linked_model_version_id,
         linked_parameter_set_id, linked_competition_id, linked_profile_id,
         linked_horizon, linked_kickoff_time, linked_actual_outcome
    FROM review.postmatch_settlements settlement
    JOIN football.matches fixture ON fixture.id = settlement.match_id
    WHERE settlement.id = NEW.settlement_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'formal postmatch evaluation sample requires a settlement';
    END IF;
    IF NEW.review_id <> linked_review_id
       OR NEW.run_id <> linked_run_id
       OR NEW.model_version_id <> linked_model_version_id
       OR NEW.parameter_set_id <> linked_parameter_set_id
       OR NEW.competition_id IS DISTINCT FROM linked_competition_id
       OR NEW.competition_profile_id IS DISTINCT FROM linked_profile_id
       OR NEW.snapshot_type <> linked_horizon
       OR NEW.kickoff_time <> linked_kickoff_time
       OR NEW.actual_outcome <> linked_actual_outcome
       OR NEW.calculation_version <> 'postmatch-monitoring-v1' THEN
        RAISE EXCEPTION 'formal postmatch evaluation sample identity mismatch';
    END IF;
    RETURN NEW;
END;
$function$;

DROP TRIGGER IF EXISTS evaluation_samples_postmatch_guard ON analytics.evaluation_samples;
CREATE TRIGGER evaluation_samples_postmatch_guard
BEFORE INSERT OR UPDATE OR DELETE ON analytics.evaluation_samples
FOR EACH ROW EXECUTE FUNCTION analytics.guard_postmatch_evaluation_sample();

CREATE OR REPLACE FUNCTION review.validate_postmatch_settlement_insert()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    linked_match_id uuid;
    linked_run_id uuid;
    linked_review_status text;
    linked_run_status text;
    linked_snapshot_id uuid;
    linked_model_version_id uuid;
    linked_parameter_set_id uuid;
    linked_rule_package_id uuid;
    linked_horizon text;
    linked_profile_id uuid;
    linked_snapshot_match_id uuid;
    linked_snapshot_model_version_id uuid;
    linked_snapshot_parameter_set_id uuid;
    linked_snapshot_profile_id uuid;
    linked_snapshot_source_kind text;
    linked_snapshot_evidence_scope text;
BEGIN
    SELECT review.match_id, review.source_run_id, review.status,
           run.status, run.feature_snapshot_id, run.model_version_id,
           run.parameter_set_id, run.rule_package_id, run.snapshot_type,
           package.competition_profile_id, snapshot.match_id,
           snapshot.model_version_id, snapshot.parameter_set_id,
           snapshot.competition_profile_id, snapshot.source_kind,
           snapshot.evidence_scope
    INTO linked_match_id, linked_run_id, linked_review_status,
         linked_run_status, linked_snapshot_id, linked_model_version_id,
         linked_parameter_set_id, linked_rule_package_id, linked_horizon,
         linked_profile_id, linked_snapshot_match_id,
         linked_snapshot_model_version_id, linked_snapshot_parameter_set_id,
         linked_snapshot_profile_id, linked_snapshot_source_kind,
         linked_snapshot_evidence_scope
    FROM review.match_reviews review
    JOIN model.runs run ON run.id = review.source_run_id
    JOIN model.rule_packages package ON package.id = run.rule_package_id
    JOIN feature.snapshots snapshot ON snapshot.id = run.feature_snapshot_id
    WHERE review.id = NEW.match_review_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'postmatch settlement requires a review linked to a run, rule package and frozen snapshot';
    END IF;
    IF linked_review_status <> 'finalized' OR linked_run_status <> 'succeeded' THEN
        RAISE EXCEPTION 'postmatch settlement requires finalized review and succeeded run';
    END IF;
    IF linked_match_id <> NEW.match_id OR linked_run_id <> NEW.model_run_id
       OR linked_snapshot_id <> NEW.feature_snapshot_id
       OR linked_model_version_id <> NEW.model_version_id
       OR linked_parameter_set_id <> NEW.parameter_set_id
       OR linked_rule_package_id <> NEW.rule_package_id
       OR linked_horizon <> NEW.horizon
       OR linked_profile_id <> NEW.competition_profile_id THEN
        RAISE EXCEPTION 'postmatch settlement identity does not match review/run/package';
    END IF;
    IF linked_snapshot_match_id <> NEW.match_id
       OR linked_snapshot_model_version_id <> NEW.model_version_id
       OR linked_snapshot_parameter_set_id <> NEW.parameter_set_id
       OR linked_snapshot_profile_id <> NEW.competition_profile_id THEN
        RAISE EXCEPTION 'postmatch settlement identity does not match frozen snapshot';
    END IF;
    IF linked_snapshot_source_kind NOT IN ('real', 'manual')
       OR linked_snapshot_evidence_scope <> 'real' THEN
        RAISE EXCEPTION 'formal postmatch settlement requires a real/manual snapshot with real evidence scope';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM football.matches fixture
        JOIN football.match_results result ON result.match_id = fixture.id
        WHERE fixture.id = NEW.match_id
          AND fixture.competition_id = NEW.competition_id
          AND result.home_goals_90 = NEW.home_goals_90
          AND result.away_goals_90 = NEW.away_goals_90
          AND result.finalized_at = NEW.result_finalized_at
    ) THEN
        RAISE EXCEPTION 'postmatch settlement requires official result and matching competition';
    END IF;
    RETURN NEW;
END;
$function$;

DROP TRIGGER IF EXISTS postmatch_settlements_validate_insert ON review.postmatch_settlements;
CREATE TRIGGER postmatch_settlements_validate_insert
BEFORE INSERT ON review.postmatch_settlements
FOR EACH ROW EXECUTE FUNCTION review.validate_postmatch_settlement_insert();

CREATE OR REPLACE FUNCTION review.validate_evidence_scoring_item_insert()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    linked_snapshot_id uuid;
    linked_cutoff timestamptz;
    linked_source_document_id uuid;
    linked_provider_id uuid;
    linked_verification_state text;
    linked_source_tier text;
    linked_published_at timestamptz;
    linked_retrieved_at timestamptz;
BEGIN
    SELECT settlement.feature_snapshot_id, snapshot.data_cutoff_time,
           claim.source_document_id, document.provider_id,
           claim.verification_state, claim.source_tier,
           claim.published_at, claim.retrieved_at
    INTO linked_snapshot_id, linked_cutoff, linked_source_document_id,
         linked_provider_id, linked_verification_state, linked_source_tier,
         linked_published_at, linked_retrieved_at
    FROM review.postmatch_settlements settlement
    JOIN feature.snapshots snapshot ON snapshot.id = settlement.feature_snapshot_id
    JOIN research.evidence_claims claim ON claim.id = NEW.evidence_id
    LEFT JOIN catalog.source_documents document ON document.id = claim.source_document_id
    WHERE settlement.id = NEW.settlement_id;

    IF NOT FOUND OR NOT EXISTS (
        SELECT 1 FROM feature.snapshot_evidence link
        WHERE link.snapshot_id = linked_snapshot_id
          AND link.evidence_id = NEW.evidence_id
          AND link.field_key = NEW.field_key
    ) THEN
        RAISE EXCEPTION 'evidence scoring item must come from the exact settlement snapshot';
    END IF;
    IF NEW.retrieved_at > linked_cutoff
       OR (NEW.published_at IS NOT NULL AND NEW.published_at > linked_cutoff) THEN
        RAISE EXCEPTION 'evidence scoring item crosses the frozen snapshot cutoff';
    END IF;
    IF NEW.data_cutoff_at <> linked_cutoff
       OR NEW.source_document_id IS DISTINCT FROM linked_source_document_id
       OR NEW.provider_id IS DISTINCT FROM linked_provider_id
       OR NEW.verification_state <> linked_verification_state
       OR NEW.source_tier <> linked_source_tier
       OR NEW.published_at IS DISTINCT FROM linked_published_at
       OR NEW.retrieved_at <> linked_retrieved_at THEN
        RAISE EXCEPTION 'evidence scoring item does not match immutable evidence claim';
    END IF;
    RETURN NEW;
END;
$function$;

DROP TRIGGER IF EXISTS evidence_scoring_items_validate_insert ON review.evidence_scoring_items;
CREATE TRIGGER evidence_scoring_items_validate_insert
BEFORE INSERT ON review.evidence_scoring_items
FOR EACH ROW EXECUTE FUNCTION review.validate_evidence_scoring_item_insert();

CREATE OR REPLACE FUNCTION review.reject_postmatch_record_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    RAISE EXCEPTION 'postmatch ledger %.% is immutable', TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$function$;

DROP TRIGGER IF EXISTS postmatch_settlements_immutable ON review.postmatch_settlements;
CREATE TRIGGER postmatch_settlements_immutable
BEFORE UPDATE OR DELETE ON review.postmatch_settlements
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DROP TRIGGER IF EXISTS evidence_scoring_items_immutable ON review.evidence_scoring_items;
CREATE TRIGGER evidence_scoring_items_immutable
BEFORE UPDATE OR DELETE ON review.evidence_scoring_items
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DROP TRIGGER IF EXISTS evidence_scoring_decisions_immutable ON review.evidence_scoring_decisions;
CREATE TRIGGER evidence_scoring_decisions_immutable
BEFORE UPDATE OR DELETE ON review.evidence_scoring_decisions
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DROP TRIGGER IF EXISTS provider_score_snapshots_immutable ON analytics.provider_score_snapshots;
CREATE TRIGGER provider_score_snapshots_immutable
BEFORE UPDATE OR DELETE ON analytics.provider_score_snapshots
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DROP TRIGGER IF EXISTS postmatch_drift_runs_immutable ON analytics.postmatch_drift_runs;
CREATE TRIGGER postmatch_drift_runs_immutable
BEFORE UPDATE OR DELETE ON analytics.postmatch_drift_runs
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DROP TRIGGER IF EXISTS postmatch_drift_findings_immutable ON analytics.postmatch_drift_findings;
CREATE TRIGGER postmatch_drift_findings_immutable
BEFORE UPDATE OR DELETE ON analytics.postmatch_drift_findings
FOR EACH ROW EXECUTE FUNCTION review.reject_postmatch_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-postmatch-settlement'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-postmatch-settlement', '1.0.0', '0.21.0', '0.22.0',
            'football.p4-postmatch-settlement.v1',
            '04b791e84d6cbc93aafebe4e0701fe09d435df1aad9e01955efc7567ca58592c', 'H',
            jsonb_build_object(
                'contract_path', 'contracts/postmatch-settlement-contract.json',
                'settlement_ready', true,
                'evidence_queue_ready', true,
                'provider_scoring_ready', true,
                'drift_metrics_ready', true,
                'formal_partition', 'model_version x competition_profile x parameter_version x horizon',
                'manual_evidence_verdicts', true,
                'automatic_parameter_promotion', false,
                'p4_4_state', 'SHADOW_ONLY'
            )
        );
    ELSIF existing_hash <> '04b791e84d6cbc93aafebe4e0701fe09d435df1aad9e01955efc7567ca58592c' THEN
        RAISE EXCEPTION 'postmatch settlement contract hash conflict: existing %, expected %',
            existing_hash, '04b791e84d6cbc93aafebe4e0701fe09d435df1aad9e01955efc7567ca58592c';
    END IF;
END;
$migration$;
