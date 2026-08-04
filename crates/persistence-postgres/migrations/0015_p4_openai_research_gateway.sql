-- P4 接入D：OpenAI Responses API研究网关、尝试审计与Web Search引用账本。
-- 不保存API密钥；请求载荷和原始响应均为不含Authorization头的JSON。
-- CONTRACT_SHA256 = e35f951d2e1fa22746b39b6639ca9847b9590b26e0764d8f35405db98bfbfb57

CREATE TABLE research.openai_attempts (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    attempt_number integer NOT NULL CHECK (attempt_number > 0),
    model_id text NOT NULL,
    request_fingerprint text NOT NULL CHECK (request_fingerprint ~ '^[0-9a-f]{64}$'),
    attempt_fingerprint text NOT NULL CHECK (attempt_fingerprint ~ '^[0-9a-f]{64}$'),
    request_payload jsonb NOT NULL,
    response_id text,
    provider_request_id text,
    provider_status integer CHECK (provider_status IS NULL OR provider_status BETWEEN 100 AND 599),
    status text NOT NULL CHECK (status IN (
        'queued', 'in_progress', 'completed', 'failed', 'cancelled', 'incomplete'
    )),
    token_usage jsonb NOT NULL DEFAULT '{}'::jsonb,
    latency_ms bigint NOT NULL CHECK (latency_ms >= 0),
    search_call_count integer NOT NULL DEFAULT 0 CHECK (search_call_count >= 0),
    estimated_cost_usd double precision CHECK (estimated_cost_usd IS NULL OR estimated_cost_usd >= 0),
    raw_response jsonb,
    error_category text,
    error_message text,
    retryable boolean NOT NULL DEFAULT false,
    started_at timestamptz NOT NULL,
    finished_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (research_run_id, attempt_number),
    CHECK (finished_at >= started_at),
    CHECK (request_payload::text !~* '(authorization|bearer[[:space:]]+[a-z0-9._-]+|sk-[a-z0-9_-]{20,})'),
    CHECK (raw_response IS NULL OR raw_response::text !~* '(authorization|bearer[[:space:]]+[a-z0-9._-]+|sk-[a-z0-9_-]{20,})')
);
CREATE INDEX openai_attempts_response_idx
    ON research.openai_attempts (response_id)
    WHERE response_id IS NOT NULL;
CREATE INDEX openai_attempts_time_idx
    ON research.openai_attempts (created_at DESC);
CREATE INDEX openai_attempts_status_idx
    ON research.openai_attempts (status, created_at DESC);

CREATE TRIGGER openai_attempts_immutable
BEFORE UPDATE OR DELETE ON research.openai_attempts
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.web_citations (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    response_id text NOT NULL,
    url text NOT NULL,
    title text NOT NULL,
    domain text NOT NULL,
    output_index integer NOT NULL CHECK (output_index >= 0),
    start_index integer CHECK (start_index IS NULL OR start_index >= 0),
    end_index integer CHECK (end_index IS NULL OR end_index >= 0),
    citation_fingerprint text NOT NULL CHECK (citation_fingerprint ~ '^[0-9a-f]{64}$'),
    retrieved_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (research_run_id, response_id, citation_fingerprint),
    CHECK (url ~ '^https://'),
    CHECK (end_index IS NULL OR start_index IS NULL OR end_index >= start_index)
);
CREATE INDEX web_citations_run_idx
    ON research.web_citations (research_run_id, created_at, output_index);
CREATE INDEX web_citations_domain_idx
    ON research.web_citations (domain, created_at DESC);

CREATE TRIGGER web_citations_immutable
BEFORE UPDATE OR DELETE ON research.web_citations
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE TABLE research.web_sources (
    id uuid PRIMARY KEY,
    research_run_id uuid NOT NULL REFERENCES research.runs(id) ON DELETE CASCADE,
    response_id text NOT NULL,
    url text NOT NULL,
    title text,
    domain text NOT NULL,
    source_fingerprint text NOT NULL CHECK (source_fingerprint ~ '^[0-9a-f]{64}$'),
    retrieved_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (research_run_id, response_id, source_fingerprint),
    CHECK (url ~ '^https://')
);
CREATE INDEX web_sources_run_idx
    ON research.web_sources (research_run_id, created_at);
CREATE INDEX web_sources_domain_idx
    ON research.web_sources (domain, created_at DESC);

CREATE TRIGGER web_sources_immutable
BEFORE UPDATE OR DELETE ON research.web_sources
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

CREATE VIEW research.openai_usage_daily AS
SELECT
    date_trunc('day', created_at AT TIME ZONE 'UTC') AS usage_day_utc,
    model_id,
    count(*) FILTER (WHERE status = 'completed') AS completed_requests,
    count(*) FILTER (WHERE status = 'failed') AS failed_requests,
    sum(search_call_count) AS search_calls,
    sum(COALESCE(estimated_cost_usd, 0)) AS estimated_cost_usd,
    sum(COALESCE((token_usage ->> 'input_tokens')::bigint, 0)) AS input_tokens,
    sum(COALESCE((token_usage ->> 'output_tokens')::bigint, 0)) AS output_tokens
FROM research.openai_attempts
GROUP BY date_trunc('day', created_at AT TIME ZONE 'UTC'), model_id;

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-openai-research-gateway'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version, release_version,
            schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-openai-research-gateway',
            '1.0.0',
            '0.9.0',
            '0.10.0',
            'football.p4-research-gateway-contract.v1',
            'e35f951d2e1fa22746b39b6639ca9847b9590b26e0764d8f35405db98bfbfb57',
            'D',
            jsonb_build_object(
                'contract_path', 'contracts/research-gateway-contract.json',
                'api', 'responses',
                'tool', 'web_search',
                'strict_schema', true,
                'secret_storage', 'windows_credential_manager_or_server_environment',
                'provider_runtime_separate', true,
                'entity_resolution_stage', 'E',
                'scheduler_stage', 'F',
                'ui_stage', 'G'
            )
        );
    ELSIF existing_hash <> 'e35f951d2e1fa22746b39b6639ca9847b9590b26e0764d8f35405db98bfbfb57' THEN
        RAISE EXCEPTION 'P4 research gateway contract hash conflict: existing %, expected %',
            existing_hash,
            'e35f951d2e1fa22746b39b6639ca9847b9590b26e0764d8f35405db98bfbfb57';
    END IF;
END;
$migration$;
