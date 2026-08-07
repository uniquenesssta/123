-- P4 接入C：证据、版本与不可变赛前快照持久化。
-- 复用现有比赛、来源、模型、赛果与审计结构；新增缺失的版本账本和证据责任实体。
-- CONTRACT_SHA256 = 13274c8467b68277b904c7abf512d49d853aeabcfaa7e5cd641802a3a11659d8

CREATE SCHEMA IF NOT EXISTS research;

CREATE OR REPLACE FUNCTION platform.reject_immutable_record_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    RAISE EXCEPTION '%.% records are append-only; publish a new version or event instead',
        TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$function$;

CREATE TABLE research.schema_versions (
    id uuid PRIMARY KEY,
    schema_key text NOT NULL,
    version text NOT NULL,
    schema_kind text NOT NULL,
    schema_body jsonb NOT NULL,
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    description text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (schema_key, version)
);

CREATE TRIGGER schema_versions_immutable
BEFORE UPDATE OR DELETE ON research.schema_versions
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.prompt_versions (
    id uuid PRIMARY KEY,
    prompt_key text NOT NULL,
    version text NOT NULL,
    prompt_role text NOT NULL,
    content text NOT NULL,
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (prompt_key, version)
);

CREATE TRIGGER prompt_versions_immutable
BEFORE UPDATE OR DELETE ON research.prompt_versions
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE model.competition_profiles (
    id uuid PRIMARY KEY,
    profile_key text NOT NULL,
    version text NOT NULL,
    name text NOT NULL,
    competition_kind text NOT NULL CHECK (competition_kind IN (
        'league', 'group_stage', 'knockout_single_leg',
        'knockout_two_leg', 'friendly', 'custom'
    )),
    definition jsonb NOT NULL,
    definition_sha256 text NOT NULL CHECK (definition_sha256 ~ '^[0-9a-f]{64}$'),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (profile_key, version)
);

CREATE TRIGGER competition_profiles_immutable
BEFORE UPDATE OR DELETE ON model.competition_profiles
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

ALTER TABLE model.rule_packages
    ADD COLUMN IF NOT EXISTS competition_profile_id uuid
    REFERENCES model.competition_profiles(id);
CREATE INDEX IF NOT EXISTS rule_packages_profile_idx
    ON model.rule_packages (competition_profile_id);

CREATE TABLE research.runs (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    horizon text NOT NULL CHECK (horizon IN ('T-24h', 'T-6h', 'T-90m', 'T-1h', 'T-N')),
    data_cutoff_at timestamptz NOT NULL,
    trace_id uuid NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    request_fingerprint text NOT NULL CHECK (request_fingerprint ~ '^[0-9a-f]{64}$'),
    planner_version text,
    prompt_version_id uuid REFERENCES research.prompt_versions(id),
    schema_version_id uuid NOT NULL REFERENCES research.schema_versions(id),
    status text NOT NULL DEFAULT 'planned' CHECK (status IN (
        'planned', 'running', 'succeeded', 'partial', 'failed', 'cancelled'
    )),
    request_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    response_id text,
    model_id text,
    token_usage jsonb NOT NULL DEFAULT '{}'::jsonb,
    error_category text,
    error_message text,
    attempt_count integer NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (finished_at IS NULL OR finished_at >= created_at)
);
CREATE INDEX research_runs_match_idx
    ON research.runs (match_id, horizon, data_cutoff_at DESC);
CREATE INDEX research_runs_status_idx
    ON research.runs (status, updated_at DESC);
CREATE INDEX research_runs_trace_idx
    ON research.runs (trace_id);

CREATE TABLE research.run_events (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    status text NOT NULL CHECK (status IN (
        'planned', 'running', 'succeeded', 'partial', 'failed', 'cancelled'
    )),
    response_id text,
    model_id text,
    token_usage jsonb NOT NULL DEFAULT '{}'::jsonb,
    error_category text,
    error_message text,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    idempotency_key text NOT NULL,
    event_fingerprint text NOT NULL CHECK (event_fingerprint ~ '^[0-9a-f]{64}$'),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (research_run_id, idempotency_key)
);
CREATE INDEX research_run_events_time_idx
    ON research.run_events (research_run_id, occurred_at, id);

CREATE TRIGGER research_run_events_immutable
BEFORE UPDATE OR DELETE ON research.run_events
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.evidence_conflicts (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    entity_type text NOT NULL,
    entity_id uuid,
    field_key text NOT NULL,
    conflict_key text NOT NULL UNIQUE,
    conflict_fingerprint text NOT NULL CHECK (conflict_fingerprint ~ '^[0-9a-f]{64}$'),
    trace_id uuid NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX evidence_conflicts_match_idx
    ON research.evidence_conflicts (match_id, field_key, created_at DESC);

CREATE TRIGGER evidence_conflicts_immutable
BEFORE UPDATE OR DELETE ON research.evidence_conflicts
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.evidence_claims (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id),
    entity_type text NOT NULL,
    entity_id uuid,
    field_key text NOT NULL,
    value jsonb NOT NULL,
    verification_state text NOT NULL CHECK (verification_state IN (
        'CONFIRMED', 'PROBABLE', 'CONFLICT', 'NOT_FOUND', 'STALE', 'NOT_APPLICABLE'
    )),
    source_tier text NOT NULL,
    source_document_id uuid REFERENCES catalog.source_documents(id),
    source_url text,
    source_title text,
    source_domain text,
    published_at timestamptz,
    observed_at timestamptz NOT NULL,
    effective_at timestamptz,
    retrieved_at timestamptz NOT NULL,
    timezone text NOT NULL,
    independent_source_count integer NOT NULL DEFAULT 0 CHECK (independent_source_count >= 0),
    conflict_group_id uuid REFERENCES research.evidence_conflicts(id),
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    claim_fingerprint text NOT NULL CHECK (claim_fingerprint ~ '^[0-9a-f]{64}$'),
    research_run_id uuid NOT NULL REFERENCES research.runs(id),
    prompt_version_id uuid REFERENCES research.prompt_versions(id),
    prompt_version text,
    schema_version_id uuid NOT NULL REFERENCES research.schema_versions(id),
    schema_version text NOT NULL,
    idempotency_key text NOT NULL UNIQUE,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (
        verification_state IN ('NOT_FOUND', 'NOT_APPLICABLE')
        OR (source_url IS NOT NULL AND source_title IS NOT NULL AND source_domain IS NOT NULL)
    )
);
CREATE INDEX evidence_claims_match_field_idx
    ON research.evidence_claims (match_id, field_key, created_at DESC);
CREATE INDEX evidence_claims_entity_idx
    ON research.evidence_claims (entity_type, entity_id, field_key, created_at DESC);
CREATE INDEX evidence_claims_research_run_idx
    ON research.evidence_claims (research_run_id, created_at);
CREATE INDEX evidence_claims_conflict_idx
    ON research.evidence_claims (conflict_group_id)
    WHERE conflict_group_id IS NOT NULL;

CREATE TRIGGER evidence_claims_immutable
BEFORE UPDATE OR DELETE ON research.evidence_claims
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.evidence_conflict_members (
    conflict_id uuid NOT NULL REFERENCES research.evidence_conflicts(id),
    evidence_id uuid NOT NULL REFERENCES research.evidence_claims(id),
    added_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (conflict_id, evidence_id),
    UNIQUE (evidence_id)
);

CREATE TRIGGER evidence_conflict_members_immutable
BEFORE UPDATE OR DELETE ON research.evidence_conflict_members
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.evidence_conflict_events (
    id uuid PRIMARY KEY,
    conflict_id uuid NOT NULL REFERENCES research.evidence_conflicts(id),
    event_type text NOT NULL CHECK (event_type IN (
        'opened', 'resolved', 'reopened', 'dismissed', 'accepted_unknown'
    )),
    actor text NOT NULL DEFAULT 'desktop-client',
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    idempotency_key text NOT NULL,
    event_fingerprint text NOT NULL CHECK (event_fingerprint ~ '^[0-9a-f]{64}$'),
    occurred_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (conflict_id, idempotency_key)
);
CREATE INDEX evidence_conflict_events_time_idx
    ON research.evidence_conflict_events (conflict_id, occurred_at, id);

CREATE TRIGGER evidence_conflict_events_immutable
BEFORE UPDATE OR DELETE ON research.evidence_conflict_events
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

ALTER TABLE feature.snapshots
    ADD COLUMN IF NOT EXISTS model_version_id uuid REFERENCES model.versions(id),
    ADD COLUMN IF NOT EXISTS parameter_set_id uuid REFERENCES model.parameter_sets(id),
    ADD COLUMN IF NOT EXISTS competition_profile_id uuid REFERENCES model.competition_profiles(id),
    ADD COLUMN IF NOT EXISTS research_run_id uuid REFERENCES research.runs(id),
    ADD COLUMN IF NOT EXISTS schema_version_id uuid REFERENCES research.schema_versions(id),
    ADD COLUMN IF NOT EXISTS trace_id uuid,
    ADD COLUMN IF NOT EXISTS idempotency_key text,
    ADD COLUMN IF NOT EXISTS snapshot_fingerprint text,
    ADD COLUMN IF NOT EXISTS payload_sha256 text,
    ADD COLUMN IF NOT EXISTS feature_set_sha256 text,
    ADD COLUMN IF NOT EXISTS evidence_set_sha256 text,
    ADD COLUMN IF NOT EXISTS probability_set_sha256 text,
    ADD COLUMN IF NOT EXISTS source_kind text NOT NULL DEFAULT 'legacy',
    ADD COLUMN IF NOT EXISTS evidence_scope text NOT NULL DEFAULT 'none',
    ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();

UPDATE feature.snapshots
SET payload_sha256 = COALESCE(payload_sha256, input_sha256),
    snapshot_fingerprint = COALESCE(snapshot_fingerprint, input_sha256),
    created_at = COALESCE(created_at, frozen_at)
WHERE payload_sha256 IS NULL
   OR snapshot_fingerprint IS NULL;

ALTER TABLE feature.snapshots
    DROP CONSTRAINT IF EXISTS feature_snapshots_source_kind_check,
    ADD CONSTRAINT feature_snapshots_source_kind_check CHECK (source_kind IN (
        'legacy', 'runtime', 'real', 'manual', 'synthetic_fixture'
    )),
    DROP CONSTRAINT IF EXISTS feature_snapshots_evidence_scope_check,
    ADD CONSTRAINT feature_snapshots_evidence_scope_check CHECK (evidence_scope IN (
        'none', 'real', 'synthetic'
    )),
    DROP CONSTRAINT IF EXISTS feature_snapshots_p4_horizon_check,
    ADD CONSTRAINT feature_snapshots_p4_horizon_check CHECK (
        source_kind IN ('legacy', 'runtime')
        OR snapshot_type IN ('T-24h', 'T-6h', 'T-90m', 'T-1h')
    ),
    DROP CONSTRAINT IF EXISTS feature_snapshots_source_scope_check,
    ADD CONSTRAINT feature_snapshots_source_scope_check CHECK (
        (source_kind IN ('legacy', 'runtime') AND evidence_scope IN ('none', 'real'))
        OR (source_kind IN ('real', 'manual') AND evidence_scope = 'real')
        OR (source_kind = 'synthetic_fixture' AND evidence_scope = 'synthetic')
    ),
    DROP CONSTRAINT IF EXISTS feature_snapshots_hash_shape_check,
    ADD CONSTRAINT feature_snapshots_hash_shape_check CHECK (
        (snapshot_fingerprint IS NULL OR snapshot_fingerprint ~ '^[0-9a-f]{64}$')
        AND (payload_sha256 IS NULL OR payload_sha256 ~ '^[0-9a-f]{64}$')
        AND (feature_set_sha256 IS NULL OR feature_set_sha256 ~ '^[0-9a-f]{64}$')
        AND (evidence_set_sha256 IS NULL OR evidence_set_sha256 ~ '^[0-9a-f]{64}$')
        AND (probability_set_sha256 IS NULL OR probability_set_sha256 ~ '^[0-9a-f]{64}$')
    );

ALTER TABLE feature.snapshots
    DROP CONSTRAINT IF EXISTS snapshots_match_key_snapshot_type_input_sha256_key;
CREATE UNIQUE INDEX IF NOT EXISTS snapshots_legacy_input_uidx
    ON feature.snapshots (match_key, snapshot_type, input_sha256)
    WHERE source_kind IN ('legacy', 'runtime');

CREATE UNIQUE INDEX IF NOT EXISTS snapshots_idempotency_uidx
    ON feature.snapshots (idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS snapshots_formal_queue_uidx
    ON feature.snapshots (
        match_id, model_version_id, parameter_set_id,
        competition_profile_id, snapshot_type, data_cutoff_time
    )
    WHERE source_kind IN ('real', 'manual')
      AND match_id IS NOT NULL
      AND model_version_id IS NOT NULL
      AND parameter_set_id IS NOT NULL
      AND competition_profile_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS snapshots_trace_idx
    ON feature.snapshots (trace_id)
    WHERE trace_id IS NOT NULL;

CREATE OR REPLACE FUNCTION feature.reject_frozen_snapshot_payload_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    IF OLD.frozen_at IS NOT NULL AND (
        NEW.match_key IS DISTINCT FROM OLD.match_key
        OR NEW.snapshot_type IS DISTINCT FROM OLD.snapshot_type
        OR NEW.data_cutoff_time IS DISTINCT FROM OLD.data_cutoff_time
        OR NEW.frozen_at IS DISTINCT FROM OLD.frozen_at
        OR NEW.schema_version IS DISTINCT FROM OLD.schema_version
        OR NEW.quality_score IS DISTINCT FROM OLD.quality_score
        OR NEW.input_payload IS DISTINCT FROM OLD.input_payload
        OR NEW.input_sha256 IS DISTINCT FROM OLD.input_sha256
        OR NEW.model_version_id IS DISTINCT FROM OLD.model_version_id
        OR NEW.parameter_set_id IS DISTINCT FROM OLD.parameter_set_id
        OR NEW.competition_profile_id IS DISTINCT FROM OLD.competition_profile_id
        OR NEW.research_run_id IS DISTINCT FROM OLD.research_run_id
        OR NEW.schema_version_id IS DISTINCT FROM OLD.schema_version_id
        OR NEW.trace_id IS DISTINCT FROM OLD.trace_id
        OR NEW.idempotency_key IS DISTINCT FROM OLD.idempotency_key
        OR NEW.snapshot_fingerprint IS DISTINCT FROM OLD.snapshot_fingerprint
        OR NEW.payload_sha256 IS DISTINCT FROM OLD.payload_sha256
        OR NEW.feature_set_sha256 IS DISTINCT FROM OLD.feature_set_sha256
        OR NEW.evidence_set_sha256 IS DISTINCT FROM OLD.evidence_set_sha256
        OR NEW.probability_set_sha256 IS DISTINCT FROM OLD.probability_set_sha256
        OR NEW.source_kind IS DISTINCT FROM OLD.source_kind
        OR NEW.evidence_scope IS DISTINCT FROM OLD.evidence_scope
        OR NEW.metadata IS DISTINCT FROM OLD.metadata
    ) THEN
        RAISE EXCEPTION 'frozen prematch snapshot payload is immutable; create a new versioned snapshot';
    END IF;
    RETURN NEW;
END;
$function$;

CREATE OR REPLACE FUNCTION feature.reject_frozen_snapshot_delete()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    IF OLD.frozen_at IS NOT NULL THEN
        RAISE EXCEPTION 'frozen prematch snapshots cannot be deleted';
    END IF;
    RETURN OLD;
END;
$function$;

DROP TRIGGER IF EXISTS snapshots_frozen_payload_immutable ON feature.snapshots;
CREATE TRIGGER snapshots_frozen_payload_immutable
BEFORE UPDATE ON feature.snapshots
FOR EACH ROW EXECUTE FUNCTION feature.reject_frozen_snapshot_payload_mutation();

DROP TRIGGER IF EXISTS snapshots_frozen_delete_rejected ON feature.snapshots;
CREATE TRIGGER snapshots_frozen_delete_rejected
BEFORE DELETE ON feature.snapshots
FOR EACH ROW EXECUTE FUNCTION feature.reject_frozen_snapshot_delete();

CREATE TABLE feature.snapshot_features (
    snapshot_id uuid NOT NULL REFERENCES feature.snapshots(id),
    field_order smallint NOT NULL CHECK (field_order BETWEEN 1 AND 31),
    field_key text NOT NULL,
    value jsonb NOT NULL,
    verification_state text NOT NULL CHECK (verification_state IN (
        'CONFIRMED', 'PROBABLE', 'CONFLICT', 'NOT_FOUND', 'STALE', 'NOT_APPLICABLE'
    )),
    evidence_ids uuid[] NOT NULL DEFAULT '{}',
    value_sha256 text NOT NULL CHECK (value_sha256 ~ '^[0-9a-f]{64}$'),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id, field_order),
    UNIQUE (snapshot_id, field_key)
);

CREATE TRIGGER snapshot_features_immutable
BEFORE UPDATE OR DELETE ON feature.snapshot_features
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE feature.snapshot_evidence (
    snapshot_id uuid NOT NULL REFERENCES feature.snapshots(id),
    field_key text NOT NULL,
    evidence_id uuid NOT NULL REFERENCES research.evidence_claims(id),
    linked_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id, field_key, evidence_id),
    FOREIGN KEY (snapshot_id, field_key)
        REFERENCES feature.snapshot_features(snapshot_id, field_key)
);
CREATE INDEX snapshot_evidence_claim_idx
    ON feature.snapshot_evidence (evidence_id, snapshot_id);

CREATE TRIGGER snapshot_evidence_immutable
BEFORE UPDATE OR DELETE ON feature.snapshot_evidence
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE model.snapshot_probabilities (
    snapshot_id uuid NOT NULL REFERENCES feature.snapshots(id),
    model_run_id uuid REFERENCES model.runs(id),
    chain_key text NOT NULL CHECK (chain_key IN (
        'independent', 'core', 'full', 'shadow_mixture'
    )),
    home_win double precision NOT NULL CHECK (home_win BETWEEN 0 AND 1),
    draw double precision NOT NULL CHECK (draw BETWEEN 0 AND 1),
    away_win double precision NOT NULL CHECK (away_win BETWEEN 0 AND 1),
    btts double precision CHECK (btts IS NULL OR btts BETWEEN 0 AND 1),
    over_2_5 double precision CHECK (over_2_5 IS NULL OR over_2_5 BETWEEN 0 AND 1),
    clean_sheet_home double precision CHECK (
        clean_sheet_home IS NULL OR clean_sheet_home BETWEEN 0 AND 1
    ),
    clean_sheet_away double precision CHECK (
        clean_sheet_away IS NULL OR clean_sheet_away BETWEEN 0 AND 1
    ),
    matrix_sha256 text NOT NULL CHECK (matrix_sha256 ~ '^[0-9a-f]{64}$'),
    matrix_cell_count integer NOT NULL CHECK (matrix_cell_count = 169),
    is_formal boolean NOT NULL,
    shadow_status text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id, chain_key),
    CHECK (abs((home_win + draw + away_win) - 1.0) <= 0.000000001),
    CHECK ((chain_key = 'full') = is_formal),
    CHECK (
        (chain_key = 'shadow_mixture' AND shadow_status = 'SHADOW_ONLY')
        OR (chain_key <> 'shadow_mixture' AND shadow_status IS NULL)
    )
);
CREATE INDEX snapshot_probabilities_run_idx
    ON model.snapshot_probabilities (model_run_id)
    WHERE model_run_id IS NOT NULL;

CREATE TRIGGER snapshot_probabilities_immutable
BEFORE UPDATE OR DELETE ON model.snapshot_probabilities
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE OR REPLACE VIEW research.evidence_claim_history AS
SELECT
    claim.id AS evidence_id,
    claim.match_id,
    claim.entity_type,
    claim.entity_id,
    claim.field_key,
    claim.value,
    claim.verification_state,
    claim.source_tier,
    claim.source_document_id,
    claim.source_url,
    claim.source_title,
    claim.source_domain,
    claim.published_at,
    claim.observed_at,
    claim.effective_at,
    claim.retrieved_at,
    claim.timezone,
    claim.independent_source_count,
    COALESCE(claim.conflict_group_id, member.conflict_id) AS conflict_group_id,
    claim.content_sha256 AS content_hash,
    claim.research_run_id,
    claim.prompt_version_id,
    claim.prompt_version,
    claim.schema_version_id,
    claim.schema_version,
    claim.idempotency_key,
    claim.metadata,
    claim.created_at
FROM research.evidence_claims claim
LEFT JOIN research.evidence_conflict_members member ON member.evidence_id = claim.id;

CREATE OR REPLACE VIEW research.evidence_conflict_state AS
SELECT
    conflict.id,
    conflict.match_id,
    conflict.entity_type,
    conflict.entity_id,
    conflict.field_key,
    conflict.conflict_key,
    conflict.trace_id,
    conflict.metadata,
    conflict.created_at,
    latest.event_type AS current_state,
    latest.payload AS current_state_payload,
    latest.occurred_at AS state_changed_at
FROM research.evidence_conflicts conflict
LEFT JOIN LATERAL (
    SELECT event.event_type, event.payload, event.occurred_at
    FROM research.evidence_conflict_events event
    WHERE event.conflict_id = conflict.id
    ORDER BY event.occurred_at DESC, event.id DESC
    LIMIT 1
) latest ON true;

CREATE OR REPLACE VIEW feature.prematch_snapshot_history AS
SELECT
    snapshot.id,
    snapshot.match_id,
    snapshot.match_key,
    snapshot.snapshot_type AS horizon,
    snapshot.data_cutoff_time,
    snapshot.frozen_at,
    snapshot.model_version_id,
    version.version AS model_version,
    snapshot.parameter_set_id,
    parameters.parameter_version,
    snapshot.competition_profile_id,
    profile.profile_key AS competition_profile,
    profile.version AS competition_profile_version,
    snapshot.schema_version,
    snapshot.quality_score,
    snapshot.snapshot_fingerprint,
    snapshot.idempotency_key,
    snapshot.source_kind,
    snapshot.evidence_scope,
    snapshot.trace_id,
    snapshot.created_at
FROM feature.snapshots snapshot
LEFT JOIN model.versions version ON version.id = snapshot.model_version_id
LEFT JOIN model.parameter_sets parameters ON parameters.id = snapshot.parameter_set_id
LEFT JOIN model.competition_profiles profile ON profile.id = snapshot.competition_profile_id;


DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-persistence-ledger'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version, release_version,
            schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-persistence-ledger', '1.0.0', '0.8.0', '0.9.0',
            'football.p4-persistence-contract.v1',
            '13274c8467b68277b904c7abf512d49d853aeabcfaa7e5cd641802a3a11659d8',
            'C',
            jsonb_build_object(
                'contract_path', 'contracts/p4-persistence-contract.json',
                'feature_field_count', 31,
                'probability_chains', jsonb_build_array(
                    'independent', 'core', 'full', 'shadow_mixture'
                ),
                'p4_4_state', 'SHADOW_ONLY'
            )
        );
    ELSIF existing_hash <> '13274c8467b68277b904c7abf512d49d853aeabcfaa7e5cd641802a3a11659d8' THEN
        RAISE EXCEPTION 'P4 persistence contract hash conflict: existing %, expected %',
            existing_hash,
            '13274c8467b68277b904c7abf512d49d853aeabcfaa7e5cd641802a3a11659d8';
    END IF;
END;
$migration$;
