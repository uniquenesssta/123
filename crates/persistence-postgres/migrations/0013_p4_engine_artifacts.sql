-- Public release boundary: immutable external model-provider artifact ledger.
-- The public repository records interface fingerprints only. It does not bundle engine source,
-- parameters, profiles, fixed fixtures, or provider-owned output topology.

CREATE TABLE IF NOT EXISTS model.engine_artifacts (
    engine_key TEXT NOT NULL,
    artifact_version TEXT NOT NULL,
    release_version TEXT NOT NULL,
    contract_schema_version TEXT NOT NULL,
    contract_sha256 TEXT NOT NULL CHECK (contract_sha256 ~ '^[0-9a-f]{64}$'),
    config_sha256 TEXT NOT NULL CHECK (config_sha256 ~ '^[0-9a-f]{64}$'),
    profile_sha256 TEXT NOT NULL CHECK (profile_sha256 ~ '^[0-9a-f]{64}$'),
    input_schema_sha256 TEXT NOT NULL CHECK (input_schema_sha256 ~ '^[0-9a-f]{64}$'),
    output_schema_sha256 TEXT NOT NULL CHECK (output_schema_sha256 ~ '^[0-9a-f]{64}$'),
    provider_fixture_sha256 TEXT NOT NULL CHECK (provider_fixture_sha256 ~ '^[0-9a-f]{64}$'),
    engine_source_sha256 TEXT NOT NULL CHECK (engine_source_sha256 ~ '^[0-9a-f]{64}$'),
    formal_matrix_key TEXT NOT NULL,
    shadow_status TEXT NOT NULL,
    matrix_cell_count INTEGER NOT NULL CHECK (matrix_cell_count >= 0),
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    locked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (engine_key, artifact_version)
);

CREATE OR REPLACE FUNCTION model.reject_engine_artifact_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'model provider artifact records are immutable; publish a new artifact_version instead';
END;
$$;

DROP TRIGGER IF EXISTS engine_artifacts_immutable ON model.engine_artifacts;
CREATE TRIGGER engine_artifacts_immutable
BEFORE UPDATE OR DELETE ON model.engine_artifacts
FOR EACH ROW
EXECUTE FUNCTION model.reject_engine_artifact_mutation();

DO $$
DECLARE
    existing_contract_sha256 TEXT;
BEGIN
    SELECT contract_sha256
      INTO existing_contract_sha256
      FROM model.engine_artifacts
     WHERE engine_key = 'external-model-provider'
       AND artifact_version = '1.0.0';

    IF existing_contract_sha256 IS NULL THEN
        INSERT INTO model.engine_artifacts (
            engine_key, artifact_version, release_version, contract_schema_version,
            contract_sha256, config_sha256, profile_sha256, input_schema_sha256,
            output_schema_sha256, provider_fixture_sha256, engine_source_sha256,
            formal_matrix_key, shadow_status, matrix_cell_count, metadata
        ) VALUES (
            'external-model-provider',
            '1.0.0',
            '0.23.0',
            'football.model-provider-boundary.v1',
            'c7451f6d7a27ed8946319a346a4c7bceb13d44d51a12f26556f9f4cfca6efdb3',
            '0e7edcb4ccbd2c913afda20d2cca0dcad48866ef27d9c28c17e1cf707ebc1dee',
            '93e294670a9adb025821c0a3719582af12c00949915441b0df60446724bbf2c5',
            '8ec84be05561587b02375342f5cd430b043b7444bae90723953ee3d57b9241a5',
            'a27062d6a4b0e17e3d2257f4144f2cf96bbbb4673e43654d15d08d0b02951ef0',
            'd87c1fc695fadc1eec3b64a68c3bed5d521c2e2f6e7b709b496c6dec25db6c3f',
            'd451088946037da3188b4ea672ddbf05ddbd1c66b16e4889e8c9896dc52e0df7',
            'provider-defined',
            'NOT_BUNDLED',
            0,
            jsonb_build_object(
                'stage', 'B',
                'provider_kind', 'external',
                'bundled_runtime', false,
                'bundled_parameters', false,
                'bundled_fixtures', false,
                'output_topology', 'provider-defined'
            )
        );
    ELSIF existing_contract_sha256 <> 'c7451f6d7a27ed8946319a346a4c7bceb13d44d51a12f26556f9f4cfca6efdb3' THEN
        RAISE EXCEPTION 'external model provider artifact exists with a different contract hash';
    END IF;
END;
$$;
