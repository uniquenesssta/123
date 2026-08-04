-- 赛前基础能力：内置赛事目录、动态球员标签、比赛阵容交换与单场贡献快照。

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_entity_type_check;
ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_entity_type_check CHECK (entity_type IN (
        'team', 'player', 'player_name', 'player_position',
        'player_team_period', 'player_ability', 'player_availability',
        'player_dynamic_tag', 'external_entity_id',
        'match', 'lineup', 'lineup_player'
    ));

CREATE TABLE feature.player_dynamic_tag_definitions (
    code text PRIMARY KEY,
    name text NOT NULL,
    category text NOT NULL,
    minimum_value double precision NOT NULL,
    maximum_value double precision NOT NULL,
    default_value double precision NOT NULL,
    default_ttl_hours integer NOT NULL CHECK (default_ttl_hours > 0),
    is_multiplier boolean NOT NULL DEFAULT true,
    description text,
    CHECK (maximum_value >= minimum_value),
    CHECK (default_value BETWEEN minimum_value AND maximum_value)
);

CREATE TABLE feature.player_dynamic_tags (
    id uuid PRIMARY KEY,
    player_id uuid NOT NULL REFERENCES football.players(id) ON DELETE CASCADE,
    tag_code text NOT NULL REFERENCES feature.player_dynamic_tag_definitions(code),
    value double precision NOT NULL,
    label text,
    confidence double precision NOT NULL DEFAULT 1 CHECK (confidence BETWEEN 0 AND 1),
    observed_at timestamptz NOT NULL,
    valid_from timestamptz NOT NULL,
    valid_to timestamptz NOT NULL,
    competition_id uuid REFERENCES football.competitions(id),
    position_code text REFERENCES football.positions(code),
    opponent_team_id uuid REFERENCES football.teams(id),
    sample_size integer NOT NULL DEFAULT 1 CHECK (sample_size >= 0),
    source_type text NOT NULL DEFAULT 'manual' CHECK (source_type IN (
        'manual', 'provider', 'lineup_import', 'ai_analysis', 'match_review', 'calculation'
    )),
    source_document_id uuid REFERENCES catalog.source_documents(id),
    calculation_version text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (valid_to > valid_from)
);
CREATE INDEX player_dynamic_tags_player_time_idx
    ON feature.player_dynamic_tags (player_id, valid_from DESC, valid_to DESC);
CREATE INDEX player_dynamic_tags_scope_idx
    ON feature.player_dynamic_tags (tag_code, competition_id, position_code, opponent_team_id, valid_to DESC);

-- 动态标签以历史表为唯一事实源。按 player_id、有效期和作用域索引查询，
-- 避免未来标签覆盖当前标签，也保证任意比赛时点可以准确复现。

CREATE TABLE feature.match_player_contributions (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES football.players(id) ON DELETE CASCADE,
    lineup_id uuid REFERENCES football.lineups(id) ON DELETE SET NULL,
    as_of timestamptz NOT NULL,
    calculation_version text NOT NULL,
    base_ability double precision NOT NULL,
    effective_contribution double precision NOT NULL,
    confidence double precision NOT NULL CHECK (confidence BETWEEN 0 AND 1),
    components jsonb NOT NULL,
    applied_dynamic_tags jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (match_id, player_id, as_of, calculation_version)
);
CREATE INDEX match_player_contributions_match_idx
    ON feature.match_player_contributions (match_id, effective_contribution DESC);

INSERT INTO feature.player_dynamic_tag_definitions
    (code, name, category, minimum_value, maximum_value, default_value, default_ttl_hours, is_multiplier, description)
VALUES
    ('match_readiness', '比赛准备度', 'availability', 0, 1, 1, 168, true, '结合伤停、恢复训练和临场状态的短期可用程度'),
    ('form_multiplier', '近期状态', 'form', 0.75, 1.25, 1, 336, true, '近期表现相对长期能力的短期修正'),
    ('fatigue_multiplier', '体能负荷', 'physical', 0.50, 1, 1, 168, true, '连续比赛、旅行和恢复不足造成的体能折减'),
    ('position_fit', '位置适配', 'role', 0.50, 1.10, 1, 336, true, '本场预计位置与球员能力结构的适配程度'),
    ('tactical_fit', '战术适配', 'role', 0.50, 1.10, 1, 336, true, '本场战术角色与球员特点的适配程度'),
    ('chemistry_fit', '组合熟悉度', 'team', 0.50, 1.10, 1, 336, true, '与本场阵容搭档和体系的熟悉程度'),
    ('starting_probability', '首发概率', 'participation', 0, 1, 0.5, 72, false, '球员在本场比赛进入首发的概率'),
    ('expected_minutes_share', '预计分钟比例', 'participation', 0, 1, 1, 72, true, '预计出场分钟除以标准九十分钟'),
    ('realization_multiplier', '兑现率修正', 'realization', 0.50, 1.25, 1, 336, true, '近期和情境兑现率相对长期能力的修正'),
    ('volatility', '近期波动', 'risk', 0, 1, 0.5, 336, false, '近期表现波动程度，越高代表不确定性越大')
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    category = EXCLUDED.category,
    minimum_value = EXCLUDED.minimum_value,
    maximum_value = EXCLUDED.maximum_value,
    default_value = EXCLUDED.default_value,
    default_ttl_hours = EXCLUDED.default_ttl_hours,
    is_multiplier = EXCLUDED.is_multiplier,
    description = EXCLUDED.description;

INSERT INTO football.competitions
    (id, code, name, country_code, timezone, competition_kind, metadata)
VALUES
    ('a4759642-f0d4-572c-9dd4-996eee0b6186', 'KR-KLEAGUE1', 'K League 1', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K League 1', 'region', '韩国', 'sort_order', 10, 'source_uri', 'https://www.kleague.com/', 'temporary', false)),
    ('73430038-1cec-543e-99c7-579a6892f3e8', 'KR-KLEAGUE2', 'K League 2', 'KR', 'Asia/Seoul', 'league', jsonb_build_object('built_in', true, 'official_name', 'K League 2', 'region', '韩国', 'sort_order', 20, 'source_uri', 'https://www.kleague.com/', 'temporary', false)),
    ('2c8dcf18-86cb-5a73-89d9-e820a700f258', 'KR-KOREA-CUP', '韩国杯', 'KR', 'Asia/Seoul', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Korea Cup', 'region', '韩国', 'sort_order', 30, 'source_uri', 'https://www.kfa.or.kr/', 'temporary', false)),
    ('307556fd-ebd0-53e3-be0d-9015bf4ac7e9', 'KR-PROMOTION-PLAYOFF', '韩国升降级附加赛', 'KR', 'Asia/Seoul', 'knockout_two_leg', jsonb_build_object('built_in', true, 'official_name', 'K League Promotion/Relegation Playoff', 'region', '韩国', 'sort_order', 40, 'source_uri', 'https://www.kleague.com/', 'temporary', false)),
    ('ead9ded7-b8a6-5f93-8982-f83a670c7a26', 'JP-J1', 'J1联赛', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'J1 League', 'region', '日本', 'sort_order', 110, 'source_uri', 'https://www.jleague.co/', 'temporary', false)),
    ('6a1b4585-b89b-5633-b129-f80a6f371cca', 'JP-J2', 'J2联赛', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'J2 League', 'region', '日本', 'sort_order', 120, 'source_uri', 'https://www.jleague.co/', 'temporary', false)),
    ('d236dc7f-456f-510f-bed7-3fc50863711c', 'JP-J3', 'J3联赛', 'JP', 'Asia/Tokyo', 'league', jsonb_build_object('built_in', true, 'official_name', 'J3 League', 'region', '日本', 'sort_order', 130, 'source_uri', 'https://www.jleague.co/', 'temporary', false)),
    ('2139993a-d8cd-5a10-87ee-dc953d7434fb', 'JP-LEVAIN-CUP', '日本联赛杯', 'JP', 'Asia/Tokyo', 'custom', jsonb_build_object('built_in', true, 'official_name', 'J.League YBC Levain Cup', 'region', '日本', 'sort_order', 140, 'source_uri', 'https://www.jleague.co/', 'temporary', false)),
    ('8c920fc1-cdcb-5302-a532-462a718d3868', 'JP-EMPERORS-CUP', '天皇杯', 'JP', 'Asia/Tokyo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Emperor’s Cup', 'region', '日本', 'sort_order', 150, 'source_uri', 'https://www.jfa.jp/', 'temporary', false)),
    ('2c9603fe-8703-5015-9bf4-706453d94e25', 'JP-J1-100YV-2026', 'J1百年构想联赛', 'JP', 'Asia/Tokyo', 'custom', jsonb_build_object('built_in', true, 'official_name', 'J1 100 Year Vision League', 'region', '日本', 'sort_order', 160, 'source_uri', 'https://www.jleague.co/special/2026specialseason/j1/', 'temporary', true)),
    ('cb5b8b0f-fda0-5b67-a1d1-70fde126049c', 'JP-J2J3-100YV-2026', 'J2/J3百年构想联赛', 'JP', 'Asia/Tokyo', 'custom', jsonb_build_object('built_in', true, 'official_name', 'J2/J3 100 Year Vision League', 'region', '日本', 'sort_order', 170, 'source_uri', 'https://www.jleague.co/', 'temporary', true)),
    ('4b1b28bd-492a-5184-b669-a83df5d9555a', 'FI-VEIKKAUSLIIGA', '芬兰超级联赛', 'FI', 'Europe/Helsinki', 'league', jsonb_build_object('built_in', true, 'official_name', 'Veikkausliiga', 'region', '芬兰', 'sort_order', 210, 'source_uri', 'https://www.veikkausliiga.com/', 'temporary', false)),
    ('9e8c20b4-b08b-5ee3-9841-cc68e2d2b707', 'FI-YKKOSLIIGA', '芬兰甲级联赛', 'FI', 'Europe/Helsinki', 'league', jsonb_build_object('built_in', true, 'official_name', 'Ykkösliiga', 'region', '芬兰', 'sort_order', 220, 'source_uri', 'https://www.palloliitto.fi/', 'temporary', false)),
    ('6d803acf-6c74-51b0-ac98-7bd8d8098b03', 'FI-SUOMEN-CUP', '芬兰杯', 'FI', 'Europe/Helsinki', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Suomen Cup', 'region', '芬兰', 'sort_order', 230, 'source_uri', 'https://www.palloliitto.fi/', 'temporary', false)),
    ('273bead0-d69b-5a3a-aba7-0b6f18da6be7', 'FI-LIIGACUP', '芬兰联赛杯', 'FI', 'Europe/Helsinki', 'custom', jsonb_build_object('built_in', true, 'official_name', 'Liigacup', 'region', '芬兰', 'sort_order', 240, 'source_uri', 'https://www.palloliitto.fi/', 'temporary', false)),
    ('79dc359f-da69-5e5a-b1c9-1c00e1ef00ef', 'CH-SUPER-LEAGUE', '瑞士超级联赛', 'CH', 'Europe/Zurich', 'league', jsonb_build_object('built_in', true, 'official_name', 'Swiss Super League', 'region', '瑞士', 'sort_order', 310, 'source_uri', 'https://www.sfl.ch/', 'temporary', false)),
    ('7d7cc9b9-0348-5bc7-8f74-db2da83b53ed', 'CH-CHALLENGE-LEAGUE', '瑞士挑战联赛', 'CH', 'Europe/Zurich', 'league', jsonb_build_object('built_in', true, 'official_name', 'Swiss Challenge League', 'region', '瑞士', 'sort_order', 320, 'source_uri', 'https://www.sfl.ch/', 'temporary', false)),
    ('9a0c8d4d-41b4-5f23-8311-02bcce25787c', 'CH-SWISS-CUP', '瑞士杯', 'CH', 'Europe/Zurich', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Swiss Cup', 'region', '瑞士', 'sort_order', 330, 'source_uri', 'https://www.football.ch/', 'temporary', false)),
    ('21d36aac-e87a-5324-ab85-4763c7a39258', 'NO-ELITESERIEN', '挪威超级联赛', 'NO', 'Europe/Oslo', 'league', jsonb_build_object('built_in', true, 'official_name', 'Eliteserien', 'region', '挪威', 'sort_order', 410, 'source_uri', 'https://www.fotball.no/turneringer/eliteserien/', 'temporary', false)),
    ('78bcd08d-ef56-561d-a341-3e63c4e59d9b', 'NO-OBOS-LIGAEN', '挪威甲级联赛', 'NO', 'Europe/Oslo', 'league', jsonb_build_object('built_in', true, 'official_name', 'OBOS-ligaen', 'region', '挪威', 'sort_order', 420, 'source_uri', 'https://www.fotball.no/turneringer/obosligaen/', 'temporary', false)),
    ('61bcd452-fb3f-52f6-85ad-7bb464516a51', 'NO-NM-MENN', '挪威杯', 'NO', 'Europe/Oslo', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'NM Menn', 'region', '挪威', 'sort_order', 430, 'source_uri', 'https://www.fotball.no/turneringer/nm-menn/', 'temporary', false)),
    ('7aaf507d-1741-51c0-ae58-3b41bfc5fe7d', 'ENG-PREMIER-LEAGUE', '英格兰超级联赛', 'GB', 'Europe/London', 'league', jsonb_build_object('built_in', true, 'official_name', 'Premier League', 'region', '英格兰', 'sort_order', 510, 'source_uri', 'https://www.premierleague.com/', 'temporary', false)),
    ('96a82ba4-0294-5f20-a0b7-6b8aef632410', 'ENG-CHAMPIONSHIP', '英格兰冠军联赛', 'GB', 'Europe/London', 'league', jsonb_build_object('built_in', true, 'official_name', 'EFL Championship', 'region', '英格兰', 'sort_order', 520, 'source_uri', 'https://www.efl.com/', 'temporary', false)),
    ('c7539fe6-0202-5ab9-8398-cf16969469f5', 'ENG-LEAGUE-ONE', '英格兰甲级联赛', 'GB', 'Europe/London', 'league', jsonb_build_object('built_in', true, 'official_name', 'EFL League One', 'region', '英格兰', 'sort_order', 530, 'source_uri', 'https://www.efl.com/', 'temporary', false)),
    ('8074afed-a6f7-5140-aa0c-9a66d29a1900', 'ENG-LEAGUE-TWO', '英格兰乙级联赛', 'GB', 'Europe/London', 'league', jsonb_build_object('built_in', true, 'official_name', 'EFL League Two', 'region', '英格兰', 'sort_order', 540, 'source_uri', 'https://www.efl.com/', 'temporary', false)),
    ('ba6c4596-1255-5a8a-8987-a3a1c5c6aaed', 'ENG-FA-CUP', '英格兰足总杯', 'GB', 'Europe/London', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'FA Cup', 'region', '英格兰', 'sort_order', 550, 'source_uri', 'https://www.thefa.com/competitions/thefacup', 'temporary', false)),
    ('9b607514-3070-570f-bd63-ee80f7c9729e', 'ENG-EFL-CUP', '英格兰联赛杯', 'GB', 'Europe/London', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'EFL Cup', 'region', '英格兰', 'sort_order', 560, 'source_uri', 'https://www.efl.com/competitions/carabao-cup', 'temporary', false)),
    ('112e66e6-569a-5afe-8427-de01d2ede312', 'ENG-EFL-TROPHY', '英格兰联赛锦标赛', 'GB', 'Europe/London', 'custom', jsonb_build_object('built_in', true, 'official_name', 'EFL Trophy', 'region', '英格兰', 'sort_order', 570, 'source_uri', 'https://www.efl.com/competitions/efl-trophy', 'temporary', false)),
    ('6af0d285-bced-5d67-a37e-b418bd508f90', 'UEFA-UCL', '欧洲冠军联赛', 'UEFA', 'Europe/Zurich', 'custom', jsonb_build_object('built_in', true, 'official_name', 'UEFA Champions League', 'region', '欧洲', 'sort_order', 610, 'source_uri', 'https://www.uefa.com/uefachampionsleague/', 'temporary', false)),
    ('7dfab4aa-791f-5908-9041-0e8f2432876a', 'UEFA-UEL', '欧罗巴联赛', 'UEFA', 'Europe/Zurich', 'custom', jsonb_build_object('built_in', true, 'official_name', 'UEFA Europa League', 'region', '欧洲', 'sort_order', 620, 'source_uri', 'https://www.uefa.com/uefaeuropaleague/', 'temporary', false)),
    ('40d739cd-d241-5401-9424-6d1f8855abe9', 'UEFA-UECL', '欧洲协会联赛', 'UEFA', 'Europe/Zurich', 'custom', jsonb_build_object('built_in', true, 'official_name', 'UEFA Conference League', 'region', '欧洲', 'sort_order', 630, 'source_uri', 'https://www.uefa.com/uefaconferenceleague/', 'temporary', false)),
    ('147b67b7-19e0-5fca-86ca-848649842065', 'UEFA-SUPER-CUP', '欧洲超级杯', 'UEFA', 'Europe/Zurich', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'UEFA Super Cup', 'region', '欧洲', 'sort_order', 640, 'source_uri', 'https://www.uefa.com/uefasupercup/', 'temporary', false)),
    ('2a452afc-b89f-56ed-9b72-9c5109607b94', 'UEFA-NATIONS-LEAGUE', '欧洲国家联赛', 'UEFA', 'Europe/Zurich', 'custom', jsonb_build_object('built_in', true, 'official_name', 'UEFA Nations League', 'region', '欧洲', 'sort_order', 650, 'source_uri', 'https://www.uefa.com/uefanationsleague/', 'temporary', false)),
    ('5de3a51f-9ae7-543f-991a-cb8571346fe4', 'UEFA-EURO', '欧洲足球锦标赛', 'UEFA', 'Europe/Zurich', 'custom', jsonb_build_object('built_in', true, 'official_name', 'UEFA EURO', 'region', '欧洲', 'sort_order', 660, 'source_uri', 'https://www.uefa.com/euro2028/', 'temporary', false)),
    ('1dd9eccb-9c33-58b0-9fb3-230859558f40', 'UEFA-EURO-QUAL', '欧洲杯预选赛', 'UEFA', 'Europe/Zurich', 'group_stage', jsonb_build_object('built_in', true, 'official_name', 'UEFA European Qualifiers', 'region', '欧洲', 'sort_order', 670, 'source_uri', 'https://www.uefa.com/european-qualifiers/', 'temporary', false)),
    ('54adaefd-fc48-5095-a74c-176dac84f330', 'UEFA-WCQ', '世界杯欧洲区预选赛', 'UEFA', 'Europe/Zurich', 'group_stage', jsonb_build_object('built_in', true, 'official_name', 'UEFA World Cup Qualifiers', 'region', '欧洲', 'sort_order', 680, 'source_uri', 'https://www.uefa.com/european-qualifiers/', 'temporary', false)),
    ('1cac7a7a-683e-589d-918c-a910e47e406d', 'GENERIC-LEAGUE', '通用联赛', 'INT', 'UTC', 'league', jsonb_build_object('built_in', true, 'official_name', 'Generic League', 'region', '通用', 'sort_order', 900, 'source_uri', 'internal://generic', 'temporary', false)),
    ('4c95643b-558f-512e-977e-d8fe2d43f614', 'GENERIC-GROUP', '通用小组赛', 'INT', 'UTC', 'group_stage', jsonb_build_object('built_in', true, 'official_name', 'Generic Group Stage', 'region', '通用', 'sort_order', 910, 'source_uri', 'internal://generic', 'temporary', false)),
    ('d22dd72b-df3d-5657-970a-217348804c08', 'GENERIC-KNOCKOUT-1', '通用单回合淘汰赛', 'INT', 'UTC', 'knockout_single_leg', jsonb_build_object('built_in', true, 'official_name', 'Generic Single-leg Knockout', 'region', '通用', 'sort_order', 920, 'source_uri', 'internal://generic', 'temporary', false)),
    ('a9b4c63c-08fa-54ff-a018-ebdcdb813efb', 'GENERIC-KNOCKOUT-2', '通用两回合淘汰赛', 'INT', 'UTC', 'knockout_two_leg', jsonb_build_object('built_in', true, 'official_name', 'Generic Two-leg Knockout', 'region', '通用', 'sort_order', 930, 'source_uri', 'internal://generic', 'temporary', false)),
    ('8b58bcf7-f6c5-5630-8dca-d6c54a4a875e', 'GENERIC-FRIENDLY', '通用友谊赛', 'INT', 'UTC', 'friendly', jsonb_build_object('built_in', true, 'official_name', 'Generic Friendly', 'region', '通用', 'sort_order', 940, 'source_uri', 'internal://generic', 'temporary', false))
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    country_code = EXCLUDED.country_code,
    timezone = EXCLUDED.timezone,
    competition_kind = EXCLUDED.competition_kind,
    metadata = football.competitions.metadata || EXCLUDED.metadata,
    updated_at = now();
