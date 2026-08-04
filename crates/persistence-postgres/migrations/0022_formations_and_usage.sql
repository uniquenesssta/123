-- H 前置阶段 3：内置阵型目录、阵容映射与可审计阵型概率分布。
-- FORMATION_USAGE_CONTRACT_SHA256 = bc6cce2b86fd456f323f8970b496e944a6e95e1fe105ed297db9dda33696184d

CREATE TABLE football.formations (
    id uuid PRIMARY KEY,
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    line_structure text NOT NULL,
    slot_definition jsonb NOT NULL DEFAULT '[]'::jsonb,
    is_builtin boolean NOT NULL DEFAULT false,
    is_active boolean NOT NULL DEFAULT true,
    sort_order smallint NOT NULL DEFAULT 0,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX formations_active_sort_idx ON football.formations (is_active, sort_order, code);

INSERT INTO football.formations (
    id, code, name, line_structure, slot_definition, is_builtin, is_active, sort_order, metadata
) VALUES
('fe92425e-1c5e-51f3-9119-b056841f7343','4-3-3','4-3-3','4-3-3','["GK","LB","LCB","RCB","RB","LCM","CM","RCM","LW","ST","RW"]',true,true,10,'{"category":"back_four"}'),
('4737da75-7c7b-52f5-acf5-ea9bfa809c48','4-2-3-1','4-2-3-1','4-2-3-1','["GK","LB","LCB","RCB","RB","LDM","RDM","LAM","CAM","RAM","ST"]',true,true,20,'{"category":"back_four"}'),
('4cf34b21-deb9-5ca0-8a9e-c972037417fe','4-4-2','4-4-2','4-4-2','["GK","LB","LCB","RCB","RB","LM","LCM","RCM","RM","LST","RST"]',true,true,30,'{"category":"back_four"}'),
('9356249c-887f-5e49-8b3b-0ae3299deab8','4-1-4-1','4-1-4-1','4-1-4-1','["GK","LB","LCB","RCB","RB","DM","LM","LCM","RCM","RM","ST"]',true,true,40,'{"category":"back_four"}'),
('7e3b3f5b-4d8f-5548-b505-9115a652564a','4-4-1-1','4-4-1-1','4-4-1-1','["GK","LB","LCB","RCB","RB","LM","LCM","RCM","RM","SS","ST"]',true,true,50,'{"category":"back_four"}'),
('71267945-f654-53a9-b91b-8ad4f463d09a','4-3-1-2','4-3-1-2','4-3-1-2','["GK","LB","LCB","RCB","RB","LCM","CM","RCM","CAM","LST","RST"]',true,true,60,'{"category":"back_four"}'),
('13aa0037-711a-5317-965e-ed18b881ad66','4-1-2-1-2','4-1-2-1-2','4-1-2-1-2','["GK","LB","LCB","RCB","RB","DM","LCM","RCM","CAM","LST","RST"]',true,true,70,'{"category":"back_four"}'),
('8c2a375f-7cfb-5aa8-a828-902fe12f8df9','4-2-2-2','4-2-2-2','4-2-2-2','["GK","LB","LCB","RCB","RB","LDM","RDM","LAM","RAM","LST","RST"]',true,true,80,'{"category":"back_four"}'),
('108a1a5e-579b-51a1-abf4-4f4a125eb877','3-4-3','3-4-3','3-4-3','["GK","LCB","CB","RCB","LWB","LCM","RCM","RWB","LW","ST","RW"]',true,true,90,'{"category":"back_three"}'),
('81188bfb-5b53-5e9a-8b2b-a8e2e96ca727','3-4-2-1','3-4-2-1','3-4-2-1','["GK","LCB","CB","RCB","LWB","LCM","RCM","RWB","LAM","RAM","ST"]',true,true,100,'{"category":"back_three"}'),
('06dadac6-a37f-53f6-862c-da5a52a99314','3-5-2','3-5-2','3-5-2','["GK","LCB","CB","RCB","LWB","LCM","CM","RCM","RWB","LST","RST"]',true,true,110,'{"category":"back_three"}'),
('ec594b39-6ccb-5c69-a98b-96c01112eeae','3-5-1-1','3-5-1-1','3-5-1-1','["GK","LCB","CB","RCB","LWB","LCM","CM","RCM","RWB","SS","ST"]',true,true,120,'{"category":"back_three"}'),
('fb9cd0a1-e6c3-5c88-87f7-b8ec98fc2afa','5-3-2','5-3-2','5-3-2','["GK","LWB","LCB","CB","RCB","RWB","LCM","CM","RCM","LST","RST"]',true,true,130,'{"category":"back_five"}'),
('f016097d-7f71-5fdc-9651-406b20daacf3','5-4-1','5-4-1','5-4-1','["GK","LWB","LCB","CB","RCB","RWB","LM","LCM","RCM","RM","ST"]',true,true,140,'{"category":"back_five"}'),
('6800a940-cd12-5141-81cc-37799894f8bb','4-5-1','4-5-1','4-5-1','["GK","LB","LCB","RCB","RB","LM","LCM","CM","RCM","RM","ST"]',true,true,150,'{"category":"back_four"}'),
('076720d2-04f0-5b3b-ad47-87f0bfe290bd','UNKNOWN','未知','unknown','[]',true,true,900,'{"fallback":true}'),
('184d30b7-8200-505a-a955-d3f70097e9f7','CUSTOM','自定义','custom','[]',true,true,910,'{"custom":true}')
ON CONFLICT (id) DO UPDATE SET
    code=EXCLUDED.code,
    name=EXCLUDED.name,
    line_structure=EXCLUDED.line_structure,
    slot_definition=EXCLUDED.slot_definition,
    is_builtin=EXCLUDED.is_builtin,
    is_active=EXCLUDED.is_active,
    sort_order=EXCLUDED.sort_order,
    metadata=EXCLUDED.metadata,
    updated_at=now();

ALTER TABLE football.lineups ADD COLUMN formation_id uuid REFERENCES football.formations(id);
CREATE INDEX lineups_formation_idx ON football.lineups (formation_id, captured_at DESC);

UPDATE football.lineups lineup
SET formation_id = formation.id
FROM football.formations formation
WHERE lineup.formation_id IS NULL
  AND lineup.formation IS NOT NULL
  AND regexp_replace(lower(trim(lineup.formation)), '\s+', '', 'g') = lower(formation.code);

CREATE TABLE feature.formation_usage_observations (
    id uuid PRIMARY KEY,
    scope_type text NOT NULL CHECK (scope_type IN ('team','coach','team_coach','competition_default','system_default')),
    team_id uuid REFERENCES football.teams(id),
    coach_id uuid REFERENCES football.coaches(id),
    competition_id uuid REFERENCES football.competitions(id),
    formation_id uuid NOT NULL REFERENCES football.formations(id),
    window_preset text NOT NULL DEFAULT 'custom' CHECK (window_preset IN ('last_5','last_10','last_20','current_season','current_coach_term','custom')),
    window_start date NOT NULL,
    window_end date NOT NULL,
    observed_matches integer NOT NULL CHECK (observed_matches >= 0),
    usage_count integer NOT NULL CHECK (usage_count >= 0 AND usage_count <= observed_matches),
    raw_probability double precision NOT NULL CHECK (raw_probability >= 0 AND raw_probability <= 1),
    smoothed_probability double precision NOT NULL CHECK (smoothed_probability >= 0 AND smoothed_probability <= 1),
    confidence double precision NOT NULL DEFAULT 0.5 CHECK (confidence >= 0 AND confidence <= 1),
    smoothing_alpha double precision NOT NULL DEFAULT 3 CHECK (smoothing_alpha > 0),
    source_document_id uuid REFERENCES catalog.source_documents(id),
    observed_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    CHECK (window_end >= window_start),
    CHECK (
        (scope_type='team' AND team_id IS NOT NULL AND coach_id IS NULL AND competition_id IS NULL) OR
        (scope_type='coach' AND team_id IS NULL AND coach_id IS NOT NULL AND competition_id IS NULL) OR
        (scope_type='team_coach' AND team_id IS NOT NULL AND coach_id IS NOT NULL AND competition_id IS NULL) OR
        (scope_type='competition_default' AND team_id IS NULL AND coach_id IS NULL AND competition_id IS NOT NULL) OR
        (scope_type='system_default' AND team_id IS NULL AND coach_id IS NULL AND competition_id IS NULL)
    ),
    UNIQUE NULLS NOT DISTINCT (
        scope_type, team_id, coach_id, competition_id,
        window_start, window_end, observed_at, formation_id
    )
);
CREATE INDEX formation_usage_scope_window_idx
    ON feature.formation_usage_observations (
        scope_type, team_id, coach_id, competition_id,
        window_end DESC, observed_at DESC
    );
CREATE INDEX formation_usage_formation_idx
    ON feature.formation_usage_observations (formation_id, observed_at DESC);

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256 INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'formation-usage'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'formation-usage', '1.0.0', '0.15.0', '0.16.0',
            'football.formation-usage-contract.v1', 'bc6cce2b86fd456f323f8970b496e944a6e95e1fe105ed297db9dda33696184d', 'G',
            jsonb_build_object(
                'delivery_phase', 'H_PRE_STAGE_3',
                'contract_path', 'contracts/formation-usage-contract.json',
                'builtin_formation_catalog', true,
                'probability_normalization', true,
                'smoothing_alpha_default', 3,
                'integration_point_h_started', false
            )
        );
    ELSIF existing_hash <> 'bc6cce2b86fd456f323f8970b496e944a6e95e1fe105ed297db9dda33696184d' THEN
        RAISE EXCEPTION 'formation usage contract hash conflict: existing %, expected %',
            existing_hash, 'bc6cce2b86fd456f323f8970b496e944a6e95e1fe105ed297db9dda33696184d';
    END IF;
END;
$migration$;
