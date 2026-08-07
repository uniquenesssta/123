-- P4 接入E：实体解析、时间审计、冲突评估与唯一证据路由。
-- CONTRACT_SHA256 = 2282e4fcd2176d89e729d4eaf2a0d68c97ce50437ef5e8a442d6a571433a26c2


-- STALE may represent an explicitly missing/expired field with no current source.
DO $constraint$
DECLARE
    constraint_name text;
BEGIN
    SELECT conname
    INTO constraint_name
    FROM pg_constraint
    WHERE conrelid = 'research.evidence_claims'::regclass
      AND contype = 'c'
      AND pg_get_constraintdef(oid) LIKE '%source_url IS NOT NULL%';

    IF constraint_name IS NOT NULL THEN
        EXECUTE format(
            'ALTER TABLE research.evidence_claims DROP CONSTRAINT %I',
            constraint_name
        );
    END IF;

    ALTER TABLE research.evidence_claims
        ADD CONSTRAINT evidence_claims_source_required_check CHECK (
            verification_state IN ('NOT_FOUND', 'NOT_APPLICABLE', 'STALE')
            OR (source_url IS NOT NULL AND source_title IS NOT NULL AND source_domain IS NOT NULL)
        );
END;
$constraint$;

CREATE TABLE research.source_policy_versions (
    id uuid PRIMARY KEY,
    policy_key text NOT NULL,
    version text NOT NULL,
    competition_profile_id uuid REFERENCES model.competition_profiles(id),
    definition jsonb NOT NULL,
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (policy_key, version)
);
CREATE INDEX source_policy_profile_idx
    ON research.source_policy_versions (competition_profile_id, created_at DESC);
CREATE TRIGGER source_policy_versions_immutable
BEFORE UPDATE OR DELETE ON research.source_policy_versions
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.entity_resolutions (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    trace_id uuid NOT NULL,
    fact_key text NOT NULL,
    entity_type text NOT NULL CHECK (entity_type IN (
        'match', 'competition', 'venue', 'team', 'player', 'coach', 'official'
    )),
    raw_name text NOT NULL,
    normalized_name text NOT NULL,
    external_id text,
    resolution_status text NOT NULL CHECK (resolution_status IN (
        'resolved', 'ambiguous', 'unmatched', 'unsupported'
    )),
    resolved_entity_id uuid,
    resolved_name text,
    strategy text NOT NULL,
    confidence_score integer NOT NULL CHECK (confidence_score BETWEEN 0 AND 100),
    candidates jsonb NOT NULL DEFAULT '[]'::jsonb,
    reason text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    resolution_fingerprint text NOT NULL CHECK (resolution_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((resolution_status = 'resolved') = (resolved_entity_id IS NOT NULL))
);
CREATE INDEX entity_resolutions_run_idx
    ON research.entity_resolutions (research_run_id, fact_key, created_at);
CREATE INDEX entity_resolutions_target_idx
    ON research.entity_resolutions (entity_type, resolved_entity_id, created_at DESC)
    WHERE resolved_entity_id IS NOT NULL;
CREATE TRIGGER entity_resolutions_immutable
BEFORE UPDATE OR DELETE ON research.entity_resolutions
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.time_audits (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    trace_id uuid NOT NULL,
    fact_key text NOT NULL,
    field_key text NOT NULL,
    data_cutoff_at timestamptz NOT NULL,
    published_at timestamptz,
    observed_at timestamptz,
    effective_at timestamptz,
    retrieved_at timestamptz NOT NULL,
    timezone text,
    audit_status text NOT NULL CHECK (audit_status IN (
        'accepted', 'accepted_non_fact', 'rejected_future',
        'rejected_retrieved_after_cutoff', 'rejected_missing_evidence_time', 'rejected_missing_timezone',
        'rejected_invalid_order'
    )),
    accepted boolean NOT NULL,
    reason text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    time_fingerprint text NOT NULL CHECK (time_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (accepted = (audit_status IN ('accepted', 'accepted_non_fact')))
);
CREATE INDEX time_audits_run_idx
    ON research.time_audits (research_run_id, field_key, created_at);
CREATE INDEX time_audits_rejected_idx
    ON research.time_audits (audit_status, created_at DESC)
    WHERE accepted = false;
CREATE TRIGGER time_audits_immutable
BEFORE UPDATE OR DELETE ON research.time_audits
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.conflict_evaluations (
    id uuid PRIMARY KEY,
    conflict_id uuid NOT NULL REFERENCES research.evidence_conflicts(id),
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    trace_id uuid NOT NULL,
    source_policy_key text NOT NULL,
    source_policy_version text NOT NULL,
    evaluation_status text NOT NULL CHECK (evaluation_status IN (
        'auto_resolved', 'manual_required', 'accepted_unknown'
    )),
    winning_evidence_ids uuid[] NOT NULL DEFAULT '{}'::uuid[],
    winning_value jsonb NOT NULL DEFAULT 'null'::jsonb,
    ranking jsonb NOT NULL,
    reason text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    evaluation_fingerprint text NOT NULL CHECK (evaluation_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX conflict_evaluations_conflict_idx
    ON research.conflict_evaluations (conflict_id, created_at);
CREATE TRIGGER conflict_evaluations_immutable
BEFORE UPDATE OR DELETE ON research.conflict_evaluations
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.evidence_routes (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    trace_id uuid NOT NULL,
    route_key text NOT NULL,
    field_key text NOT NULL,
    target_module text NOT NULL,
    target_slot text NOT NULL,
    route_registry_version text NOT NULL,
    entity_type text,
    entity_id uuid,
    route_status text NOT NULL CHECK (route_status IN (
        'routed', 'missing', 'blocked_entity', 'blocked_time',
        'blocked_conflict', 'blocked_unregistered_field', 'ignored_non_model_fact'
    )),
    verification_state text NOT NULL CHECK (verification_state IN (
        'CONFIRMED', 'PROBABLE', 'CONFLICT', 'NOT_FOUND', 'STALE', 'NOT_APPLICABLE'
    )),
    selected_evidence_ids uuid[] NOT NULL DEFAULT '{}'::uuid[],
    selected_value jsonb NOT NULL DEFAULT 'null'::jsonb,
    reason text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    route_fingerprint text NOT NULL CHECK (route_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (research_run_id, route_key)
);
CREATE INDEX evidence_routes_run_idx
    ON research.evidence_routes (research_run_id, target_module, target_slot, created_at);
CREATE INDEX evidence_routes_blocked_idx
    ON research.evidence_routes (route_status, created_at DESC)
    WHERE route_status <> 'routed';
CREATE TRIGGER evidence_routes_immutable
BEFORE UPDATE OR DELETE ON research.evidence_routes
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE VIEW research.verifiable_fact_routes AS
SELECT
    route.id,
    route.research_run_id,
    route.match_id,
    route.route_key,
    route.field_key,
    route.target_module,
    route.target_slot,
    route.route_registry_version,
    route.entity_type,
    route.entity_id,
    route.route_status,
    route.verification_state,
    route.selected_evidence_ids,
    route.selected_value,
    route.reason,
    route.created_at
FROM research.evidence_routes route;

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-fact-pipeline'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version, release_version,
            schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-fact-pipeline',
            '1.0.0',
            '0.10.2',
            '0.11.0',
            'football.p4-fact-pipeline-contract.v1',
            '2282e4fcd2176d89e729d4eaf2a0d68c97ce50437ef5e8a442d6a571433a26c2',
            'E',
            jsonb_build_object(
                'contract_path', 'contracts/fact-pipeline-contract.json',
                'research_output_schema', 'football.p4-research-output.v2',
                'entity_resolution', true,
                'time_gate', true,
                'conflict_resolution', true,
                'single_evidence_router', true,
                'scheduler_stage', 'F',
                'ui_stage', 'G'
            )
        );
    ELSIF existing_hash <> '2282e4fcd2176d89e729d4eaf2a0d68c97ce50437ef5e8a442d6a571433a26c2' THEN
        RAISE EXCEPTION 'P4 fact pipeline contract hash conflict: existing %, expected %',
            existing_hash,
            '2282e4fcd2176d89e729d4eaf2a0d68c97ce50437ef5e8a442d6a571433a26c2';
    END IF;
END;
$migration$;
