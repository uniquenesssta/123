-- 接入点H前置能力：API 对话、结构化回答、文件与受控数据库操作工作台。
-- CONTRACT_SHA256 = e97695617c4a4b94ab9be401f9eff814d410166926e10183d2ef28b31cb80d9d

CREATE SCHEMA IF NOT EXISTS ai_workspace;

CREATE TABLE ai_workspace.sessions (
    id uuid PRIMARY KEY,
    profile_id text NOT NULL CHECK (btrim(profile_id) <> ''),
    preset_key text NOT NULL CHECK (btrim(preset_key) <> ''),
    title text NOT NULL CHECK (btrim(title) <> '' AND char_length(title) <= 160),
    match_id uuid REFERENCES football.matches(id),
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX api_workspace_sessions_updated_idx
    ON ai_workspace.sessions (updated_at DESC, id DESC);
CREATE INDEX api_workspace_sessions_match_idx
    ON ai_workspace.sessions (match_id, updated_at DESC)
    WHERE match_id IS NOT NULL;

CREATE TABLE ai_workspace.messages (
    id uuid PRIMARY KEY,
    session_id uuid NOT NULL REFERENCES ai_workspace.sessions(id) ON DELETE CASCADE,
    role text NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content text NOT NULL CHECK (char_length(content) <= 100000),
    structured_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    citations jsonb NOT NULL DEFAULT '[]'::jsonb,
    attachments jsonb NOT NULL DEFAULT '[]'::jsonb,
    provider_response_id text,
    model_id text,
    token_usage jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX api_workspace_messages_session_idx
    ON ai_workspace.messages (session_id, created_at, id);
CREATE UNIQUE INDEX api_workspace_messages_provider_response_idx
    ON ai_workspace.messages (provider_response_id)
    WHERE provider_response_id IS NOT NULL;

CREATE TRIGGER api_workspace_messages_immutable
BEFORE UPDATE OR DELETE ON ai_workspace.messages
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE ai_workspace.operation_proposals (
    id uuid PRIMARY KEY,
    session_id uuid NOT NULL REFERENCES ai_workspace.sessions(id) ON DELETE CASCADE,
    message_id uuid NOT NULL REFERENCES ai_workspace.messages(id) ON DELETE CASCADE,
    proposal_key text NOT NULL CHECK (proposal_key ~ '^[a-zA-Z0-9_-]{1,64}$'),
    operation_type text NOT NULL CHECK (operation_type IN (
        'add_player_name',
        'assign_player_position',
        'add_player_availability',
        'add_player_dynamic_tag',
        'add_player_ability_observation'
    )),
    payload jsonb NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    rationale text NOT NULL CHECK (char_length(rationale) <= 4000),
    confidence double precision NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    status text NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending', 'applying', 'applied', 'failed', 'rejected', 'manual_review'
    )),
    result jsonb NOT NULL DEFAULT '{}'::jsonb,
    error_message text,
    idempotency_key text NOT NULL UNIQUE,
    operation_fingerprint text NOT NULL CHECK (operation_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    decided_at timestamptz,
    UNIQUE (message_id, proposal_key)
);

CREATE INDEX api_workspace_operations_session_idx
    ON ai_workspace.operation_proposals (session_id, status, created_at, id);

CREATE OR REPLACE FUNCTION ai_workspace.guard_operation_proposal_update()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF ROW(
        NEW.session_id, NEW.message_id, NEW.proposal_key, NEW.operation_type,
        NEW.payload, NEW.rationale, NEW.confidence, NEW.idempotency_key,
        NEW.operation_fingerprint, NEW.created_at
    ) IS DISTINCT FROM ROW(
        OLD.session_id, OLD.message_id, OLD.proposal_key, OLD.operation_type,
        OLD.payload, OLD.rationale, OLD.confidence, OLD.idempotency_key,
        OLD.operation_fingerprint, OLD.created_at
    ) THEN
        RAISE EXCEPTION 'API workspace operation proposal identity is immutable';
    END IF;
    IF OLD.status IN ('applied', 'rejected') AND NEW.status IS DISTINCT FROM OLD.status THEN
        RAISE EXCEPTION 'terminal API workspace operation status is immutable';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER api_workspace_operation_identity_guard
BEFORE UPDATE ON ai_workspace.operation_proposals
FOR EACH ROW EXECUTE FUNCTION ai_workspace.guard_operation_proposal_update();

CREATE TABLE ai_workspace.generated_files (
    id uuid PRIMARY KEY,
    session_id uuid NOT NULL REFERENCES ai_workspace.sessions(id) ON DELETE CASCADE,
    message_id uuid NOT NULL REFERENCES ai_workspace.messages(id) ON DELETE CASCADE,
    filename text NOT NULL CHECK (
        btrim(filename) <> ''
        AND char_length(filename) <= 120
        AND filename !~ '[\\/:*?"<>|]'
    ),
    media_type text NOT NULL CHECK (media_type IN (
        'text/plain', 'text/markdown', 'application/json', 'text/csv'
    )),
    content text NOT NULL CHECK (octet_length(content) <= 2097152),
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (message_id, filename)
);

CREATE INDEX api_workspace_generated_files_session_idx
    ON ai_workspace.generated_files (session_id, created_at, id);

CREATE TRIGGER api_workspace_generated_files_immutable
BEFORE UPDATE OR DELETE ON ai_workspace.generated_files
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'api-workspace'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'api-workspace',
            '1.0.0',
            '0.13.0',
            '0.13.1',
            'football.api-workspace-contract.v1',
            'e97695617c4a4b94ab9be401f9eff814d410166926e10183d2ef28b31cb80d9d',
            -- integration_contracts.stage is intentionally a single-letter boundary.
            'G',
            jsonb_build_object(
                'delivery_phase', 'G_PRE_H',
                'contract_path', 'contracts/api-workspace-contract.json',
                'response_schema_path', 'schemas/api-workspace-response.schema.json',
                'arbitrary_sql_allowed', false,
                'automatic_apply_allowed', false,
                'formal_p4_evidence_bypass_allowed', false
            )
        );
    ELSIF existing_hash <> 'e97695617c4a4b94ab9be401f9eff814d410166926e10183d2ef28b31cb80d9d' THEN
        RAISE EXCEPTION 'API workspace contract hash conflict: existing %, expected %',
            existing_hash, 'e97695617c4a4b94ab9be401f9eff814d410166926e10183d2ef28b31cb80d9d';
    END IF;
END;
$migration$;
