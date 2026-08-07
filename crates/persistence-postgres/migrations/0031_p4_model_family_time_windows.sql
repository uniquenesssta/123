-- Public model-provider entries and pre-match time-window semantics.
-- T-90m historical data remains readable; new formal flows use T-N / T-24h / T-6h / T-1h.

ALTER TABLE review.postmatch_settlements
    DROP CONSTRAINT IF EXISTS postmatch_settlements_horizon_check,
    ADD CONSTRAINT postmatch_settlements_horizon_check
        CHECK (horizon IN ('T-N', 'T-24h', 'T-6h', 'T-90m', 'T-1h'));

ALTER TABLE analytics.provider_score_snapshots
    DROP CONSTRAINT IF EXISTS provider_score_snapshots_horizon_check,
    ADD CONSTRAINT provider_score_snapshots_horizon_check
        CHECK (horizon IN ('T-N', 'T-24h', 'T-6h', 'T-90m', 'T-1h'));

ALTER TABLE analytics.postmatch_drift_runs
    DROP CONSTRAINT IF EXISTS postmatch_drift_runs_horizon_check,
    ADD CONSTRAINT postmatch_drift_runs_horizon_check
        CHECK (horizon IN ('T-N', 'T-24h', 'T-6h', 'T-90m', 'T-1h'));

ALTER TABLE feature.snapshots
    DROP CONSTRAINT IF EXISTS feature_snapshots_p4_horizon_check,
    ADD CONSTRAINT feature_snapshots_p4_horizon_check CHECK (
        source_kind IN ('legacy', 'runtime')
        OR snapshot_type IN ('T-N', 'T-24h', 'T-6h', 'T-90m', 'T-1h')
    );

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'model-provider-boundary'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'model-provider-boundary', '1.0.0', '0.23.0', '0.23.0',
            'football.model-provider-boundary.v1',
            'ce4e4c13fb76d09888221181e0d8e5006de03c8445da828985949306adc0bd9e', 'J',
            jsonb_build_object(
                'contract_path', 'contracts/model-provider-boundary-contract.json',
                'provider_kind', 'external',
                'bundled_runtime', false,
                'bundled_parameters', false,
                'bundled_fixtures', false,
                'selectable_model_entries', jsonb_build_array('p4', 'p7'),
                'active_time_windows', jsonb_build_array('T-N', 'T-24h', 'T-6h', 'T-1h'),
                'legacy_read_only_windows', jsonb_build_array('T-90m'),
                'failure_mode', 'explicit_unavailable_error'
            )
        );
    ELSIF existing_hash <> 'ce4e4c13fb76d09888221181e0d8e5006de03c8445da828985949306adc0bd9e' THEN
        RAISE EXCEPTION 'Model provider boundary contract hash conflict: existing %, expected %',
            existing_hash,
            'ce4e4c13fb76d09888221181e0d8e5006de03c8445da828985949306adc0bd9e';
    END IF;
END;
$migration$;
