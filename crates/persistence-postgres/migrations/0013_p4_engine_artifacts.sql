-- P4 接入B：确定性引擎、配置与 Golden Master 不可变制品账本。
-- 运行时不读取工作簿；这里只登记可验证的机器制品指纹。

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
    golden_master_sha256 TEXT NOT NULL CHECK (golden_master_sha256 ~ '^[0-9a-f]{64}$'),
    engine_source_sha256 TEXT NOT NULL CHECK (engine_source_sha256 ~ '^[0-9a-f]{64}$'),
    formal_matrix_key TEXT NOT NULL CHECK (formal_matrix_key = 'full'),
    shadow_status TEXT NOT NULL CHECK (shadow_status = 'SHADOW_ONLY'),
    matrix_cell_count INTEGER NOT NULL CHECK (matrix_cell_count = 169),
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    locked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (engine_key, artifact_version)
);

CREATE OR REPLACE FUNCTION model.reject_engine_artifact_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'P4 engine artifact records are immutable; publish a new artifact_version instead';
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
     WHERE engine_key = 'p4-deterministic-engine'
       AND artifact_version = '1.0.0';

    IF existing_contract_sha256 IS NULL THEN
        INSERT INTO model.engine_artifacts (
            engine_key,
            artifact_version,
            release_version,
            contract_schema_version,
            contract_sha256,
            config_sha256,
            profile_sha256,
            input_schema_sha256,
            output_schema_sha256,
            golden_master_sha256,
            engine_source_sha256,
            formal_matrix_key,
            shadow_status,
            matrix_cell_count,
            metadata
        ) VALUES (
            'p4-deterministic-engine',
            '1.0.0',
            '0.8.0',
            'football.p4-engine-contract.v1',
            'd3fc2bf6244d32401ed7fdb171d78d87b3ea67b29ffa3667681641b5defdd8d3',
            '08d0386199bd6082f8e0032c6dd58a7f1e97a6abb41a6ee3fb54fcc7490badf8',
            '577ac989a5c293ab342ee8f76d55553937694818d32edb2b8753970d5229ca2a',
            '5895f669e0bed21814888c7fbb072dbe06e1dc66cf77752b57e240de34b0689a',
            'c45206b90199d12ded1d3f53bdb8e4b13b0a709bb36e6a0c1b3fc7b9ca626572',
            'b457811435d44754f444ca4ac47a76bceec264e462a52d04d86e5cbc95423a91',
            '73114e79c2dad1b636f12bac15c6daefe15e4857e64b66a44f4bf20d7e42d747',
            'full',
            'SHADOW_ONLY',
            169,
            jsonb_build_object(
                'stage', 'B',
                'runtime_workbook_dependency', false,
                'matrices', jsonb_build_array('independent', 'core', 'full', 'shadow_mixture'),
                'canonical_horizons', jsonb_build_array('T-24h', 'T-6h', 'T-90m', 'T-1h')
            )
        );
    ELSIF existing_contract_sha256 <> 'd3fc2bf6244d32401ed7fdb171d78d87b3ea67b29ffa3667681641b5defdd8d3' THEN
        RAISE EXCEPTION 'P4 engine artifact 1.0.0 exists with a different contract hash';
    END IF;
END;
$$;
