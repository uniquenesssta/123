-- 第一阶段：补齐阵容编排使用的细分场上位置。

INSERT INTO football.positions (code, name, position_group, sort_order) VALUES
    ('SW', '清道夫', 'defender', 19),
    ('LCB', '左中后卫', 'defender', 20),
    ('RCB', '右中后卫', 'defender', 21),
    ('LWB', '左翼卫', 'defender', 24),
    ('RWB', '右翼卫', 'defender', 25),
    ('LDM', '左后腰', 'midfielder', 29),
    ('RDM', '右后腰', 'midfielder', 31),
    ('LCM', '左中场', 'midfielder', 32),
    ('RCM', '右中场', 'midfielder', 34),
    ('LAM', '左前腰', 'midfielder', 35),
    ('RAM', '右前腰', 'midfielder', 37),
    ('LM', '左边前卫', 'midfielder', 38),
    ('RM', '右边前卫', 'midfielder', 39),
    ('SS', '影锋', 'forward', 40),
    ('CF', '中锋/伪九号', 'forward', 41),
    ('LST', '左前锋', 'forward', 44),
    ('RST', '右前锋', 'forward', 45)
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    position_group = EXCLUDED.position_group,
    sort_order = EXCLUDED.sort_order;
