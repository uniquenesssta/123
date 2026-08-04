-- 用户工作流优化：补全日韩赛事目录、模型参数收敛候选和可追溯审核。

INSERT INTO football.competitions
    (id, code, name, country_code, timezone, competition_kind, metadata)
VALUES
    ('74df118b-21ff-5267-9886-6326fc254ecd', 'KR-K3', '韩国 K3 联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K3 League', 'region', '韩国', 'gender', 'men', 'tier', 3, 'sort_order', 50, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('b9d2c355-e054-50a1-afe0-9419f55bc9bd', 'KR-K4', '韩国 K4 联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K4 League', 'region', '韩国', 'gender', 'men', 'tier', 4, 'sort_order', 60, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('cf199ad6-ebff-5826-92c4-85524c74cebb', 'KR-WK-LEAGUE', '韩国 WK 女足联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'WK League', 'region', '韩国', 'gender', 'women', 'tier', 1, 'sort_order', 70, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('d68fea30-a7e4-5e4c-a128-74b8af801809', 'KR-W-KOREA-CUP', '韩国女子 Korea Cup', 'KR', 'Asia/Seoul', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'W Korea Cup', 'region', '韩国', 'gender', 'women', 'sort_order', 80, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('10afe852-d795-5973-87b6-2df83cca8aee', 'KR-KLEAGUE2-PLAYOFF', 'K League 2 升级附加赛', 'KR', 'Asia/Seoul', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'K League 2 Playoff', 'region', '韩国', 'gender', 'men', 'sort_order', 45, 'source_uri', 'https://www.kleague.com/', 'temporary', false)),
    ('8821d50e-5bf0-5b09-9329-fd486fbec1ed', 'KR-K5', '韩国 K5 联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K5 League', 'region', '韩国', 'gender', 'men', 'tier', 5, 'sort_order', 90, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('d0831810-76bb-5020-b3e9-b518456f80bd', 'KR-K6', '韩国 K6 联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K6 League', 'region', '韩国', 'gender', 'men', 'tier', 6, 'sort_order', 100, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('19f90915-d72c-5236-8707-793ea274b00e', 'KR-K7', '韩国 K7 联赛', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K7 League', 'region', '韩国', 'gender', 'men', 'tier', 7, 'sort_order', 110, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('d87a5bba-1fcc-53aa-bf2e-53279835050c', 'JP-JFL', '日本足球联赛 JFL', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'Japan Football League', 'region', '日本', 'gender', 'men', 'tier', 4, 'sort_order', 180, 'source_uri', 'https://www.jfl.or.jp/', 'temporary', false)),
    ('3dffc194-78e5-5634-97c9-07adc1f43db6', 'JP-JFL-CUP', 'JFL 杯', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'JFL Cup', 'region', '日本', 'gender', 'men', 'sort_order', 190, 'source_uri', 'https://www.jfl.or.jp/', 'temporary', false)),
    ('9858d057-1ff8-5568-8d9b-16b3c52abfad', 'JP-WE-LEAGUE', '日本 WE 女足联赛', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'WE League', 'region', '日本', 'gender', 'women', 'tier', 1, 'sort_order', 200, 'source_uri', 'https://weleague.jp/', 'temporary', false)),
    ('e09e8621-8516-52d8-a364-4ab885c38468', 'JP-NADESHIKO1', '日本抚子联赛 1 部', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'Nadeshiko League Division 1', 'region', '日本', 'gender', 'women', 'tier', 2, 'sort_order', 210, 'source_uri', 'https://www.nadeshikoleague.jp/', 'temporary', false)),
    ('6598899a-4b13-5fa3-95c3-9dd8680961df', 'JP-NADESHIKO2', '日本抚子联赛 2 部', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'Nadeshiko League Division 2', 'region', '日本', 'gender', 'women', 'tier', 3, 'sort_order', 220, 'source_uri', 'https://www.nadeshikoleague.jp/', 'temporary', false)),
    ('5697b0cf-e9e2-593b-8e02-c6d776e12551', 'JP-EMPRESS-CUP', '日本皇后杯', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Empress''s Cup JFA Japan Women''s Football Championship', 'region', '日本', 'gender', 'women', 'sort_order', 230, 'source_uri', 'https://www.jfa.jp/eng/match/empressscup/', 'temporary', false)),
    ('0560f4a7-7723-5ad7-8fae-b4295a82e97e', 'JP-SUPER-CUP', '日本超级杯', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Japanese Super Cup', 'region', '日本', 'gender', 'men', 'sort_order', 155, 'source_uri', 'https://www.jleague.co/', 'temporary', false)),
    ('331a6e70-5c59-543b-aff3-0ba2b077f0e3', 'JP-REGIONAL-CHAMPIONS', '日本地区足球冠军联赛', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Japan Regional Football Champions League', 'region', '日本', 'gender', 'men', 'sort_order', 240, 'source_uri', 'https://www.jfa.jp/eng/match/', 'temporary', false)),
    ('7fe2b04e-9a11-5d19-a71c-888d0e8870cb', 'JP-ALL-JAPAN-ADULTS', '全日本成人足球锦标赛', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'All Japan Adults Football Tournament', 'region', '日本', 'gender', 'men', 'sort_order', 250, 'source_uri', 'https://www.jfa.jp/eng/match/', 'temporary', false)),
    ('1e452fd2-34f0-503f-ac2b-8f57e62ee931', 'JP-ALL-JAPAN-CLUB-TEAMS', '全日本俱乐部球队足球锦标赛', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'All Japan Club Teams Football Tournament', 'region', '日本', 'gender', 'men', 'sort_order', 260, 'source_uri', 'https://www.jfa.jp/eng/match/', 'temporary', false)),
    ('234c6f8f-644e-5ca1-889a-03545bb6d4d4', 'AFC-ACLE', '亚足联冠军精英联赛', 'AFC', 'Asia/Kuala_Lumpur', 'custom', jsonb_build_object('built_in', true, 'official_name', 'AFC Champions League Elite', 'region', '亚洲', 'gender', 'men', 'sort_order', 700, 'source_uri', 'https://www.the-afc.com/en/club/afc_champions_league_elite/home.html', 'temporary', false)),
    ('d8018ebd-636b-5fb0-832a-7b2f80aa8939', 'AFC-ACL2', '亚足联冠军联赛二级联赛', 'AFC', 'Asia/Kuala_Lumpur', 'custom', jsonb_build_object('built_in', true, 'official_name', 'AFC Champions League Two', 'region', '亚洲', 'gender', 'men', 'sort_order', 710, 'source_uri', 'https://www.the-afc.com/en/club/afc_champions_league_two/home.html', 'temporary', false)),
    ('61fc6551-f000-56df-a74d-3bf290880ae1', 'AFC-AWCL', '亚足联女子冠军联赛', 'AFC', 'Asia/Kuala_Lumpur', 'custom', jsonb_build_object('built_in', true, 'official_name', 'AFC Women''s Champions League', 'region', '亚洲', 'gender', 'women', 'sort_order', 720, 'source_uri', 'https://www.the-afc.com/en/club.html', 'temporary', false))
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    country_code = EXCLUDED.country_code,
    timezone = EXCLUDED.timezone,
    competition_kind = EXCLUDED.competition_kind,
    metadata = football.competitions.metadata || EXCLUDED.metadata,
    updated_at = now();

CREATE TABLE analytics.parameter_tuning_candidates (
    id uuid PRIMARY KEY,
    competition_id uuid REFERENCES football.competitions(id) ON DELETE SET NULL,
    model_key text NOT NULL,
    model_version text NOT NULL,
    parameter_version text NOT NULL,
    snapshot_type text NOT NULL,
    target_module text NOT NULL CHECK (target_module IN (
        'lineup_realization', 'history', 'state', 'venue', 'draw_correction', 'synergy'
    )),
    sample_size bigint NOT NULL CHECK (sample_size >= 0),
    baseline_metrics jsonb NOT NULL,
    calibration_bias jsonb NOT NULL,
    proposed_adjustments jsonb NOT NULL,
    constraints jsonb NOT NULL,
    rationale text NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending', 'accepted_for_backtest', 'rejected', 'superseded'
    )),
    created_at timestamptz NOT NULL DEFAULT now(),
    decided_at timestamptz,
    decision_note text
);
CREATE INDEX parameter_tuning_candidates_status_idx
    ON analytics.parameter_tuning_candidates (status, created_at DESC);
CREATE INDEX parameter_tuning_candidates_scope_idx
    ON analytics.parameter_tuning_candidates (competition_id, model_key, target_module, created_at DESC);
