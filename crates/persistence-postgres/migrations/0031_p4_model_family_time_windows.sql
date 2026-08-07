-- P4 生产默认与赛前时间窗口语义。
-- T-90m 历史数据继续保留，但新正式链只使用 T-N / T-24h / T-6h / T-1h。

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
    WHERE contract_key = 'p4-model-family-time-windows'
      AND contract_version = '1.1.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-model-family-time-windows', '1.1.0', '0.23.0', '0.23.0',
            'football.p4-model-family-time-windows-contract.v1',
            'd36e757d7d70406231b9672793221901d3dd5953d085d6bd793ed740d46b65b4', 'J',
            jsonb_build_object(
                'contract_path', 'contracts/p4-model-family-time-windows-contract.json',
                'default_model_family', 'p4',
                'selectable_model_families', jsonb_build_array('p4', 'p7'),
                'active_time_windows', jsonb_build_array('T-N', 'T-24h', 'T-6h', 'T-1h'),
                'legacy_read_only_windows', jsonb_build_array('T-90m'),
                'window_semantics', 'latest_record_within_selected_pre_match_window',
                'p4_calculation_and_convergence_same_lineage', true,
                'p4_base_lambda_method', 'p4_p2_time_forward_baseline_v1',
                'p4_base_lambda_min_prior_matches', 8,
                'p4_base_lambda_shrink_matches', 4,
                'generic_dixon_coles_enabled', false,
                'world_cup_knockout_rho', -0.13
            )
        );
    ELSIF existing_hash <> 'd36e757d7d70406231b9672793221901d3dd5953d085d6bd793ed740d46b65b4' THEN
        RAISE EXCEPTION 'P4 model-family/time-window contract hash conflict: existing %, expected %',
            existing_hash,
            'd36e757d7d70406231b9672793221901d3dd5953d085d6bd793ed740d46b65b4';
    END IF;
END;
$migration$;
