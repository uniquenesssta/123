-- 接入A：锁定0.6.6第五阶段基线、P4语义映射和后续接入边界。
-- CONTRACT_SHA256 = b7ff6e3cc13afc8c9d6d8cac1b6b4f566fc7b7fd9f171be305fb57725e3a8371

CREATE TABLE IF NOT EXISTS platform.integration_contracts (
    contract_key text NOT NULL,
    contract_version text NOT NULL,
    baseline_source_version text NOT NULL,
    release_version text NOT NULL,
    schema_version text NOT NULL,
    content_sha256 text NOT NULL CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    stage text NOT NULL CHECK (stage ~ '^[A-Z]$'),
    status text NOT NULL DEFAULT 'locked' CHECK (status = 'locked'),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    locked_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (contract_key, contract_version)
);

CREATE OR REPLACE FUNCTION platform.reject_integration_contract_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    RAISE EXCEPTION 'integration contract %/% is immutable', OLD.contract_key, OLD.contract_version;
END;
$function$;

DROP TRIGGER IF EXISTS integration_contracts_immutable
    ON platform.integration_contracts;
CREATE TRIGGER integration_contracts_immutable
BEFORE UPDATE OR DELETE ON platform.integration_contracts
FOR EACH ROW
EXECUTE FUNCTION platform.reject_integration_contract_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-software-integration'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key,
            contract_version,
            baseline_source_version,
            release_version,
            schema_version,
            content_sha256,
            stage,
            metadata
        ) VALUES (
            'p4-software-integration',
            '1.0.0',
            '0.6.6',
            '0.7.0',
            'football.p4-integration-baseline.v1',
            'b7ff6e3cc13afc8c9d6d8cac1b6b4f566fc7b7fd9f171be305fb57725e3a8371',
            'A',
            jsonb_build_object(
                'contract_path', 'contracts/p4-integration-baseline.json',
                'p4_4_state', 'SHADOW_ONLY',
                'canonical_horizons', jsonb_build_array('T-24h', 'T-6h', 'T-90m', 'T-1h'),
                'legacy_horizons', jsonb_build_array('T-N')
            )
        );
    ELSIF existing_hash <> 'b7ff6e3cc13afc8c9d6d8cac1b6b4f566fc7b7fd9f171be305fb57725e3a8371' THEN
        RAISE EXCEPTION 'P4 integration contract hash conflict: existing %, expected %',
            existing_hash,
            'b7ff6e3cc13afc8c9d6d8cac1b6b4f566fc7b7fd9f171be305fb57725e3a8371';
    END IF;
END;
$migration$;
