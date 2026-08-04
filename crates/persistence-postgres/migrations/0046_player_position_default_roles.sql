-- 球员位置档案补充默认战术角色。比赛阵容可以继承该值，仍允许本场显式覆盖。

ALTER TABLE football.player_positions
    ADD COLUMN IF NOT EXISTS default_role_code text;

ALTER TABLE football.player_positions
    DROP CONSTRAINT IF EXISTS player_positions_default_role_code_check;

ALTER TABLE football.player_positions
    ADD CONSTRAINT player_positions_default_role_code_check
    CHECK (
        default_role_code IS NULL
        OR (btrim(default_role_code) <> '' AND char_length(default_role_code) <= 80)
    );

CREATE INDEX IF NOT EXISTS player_positions_role_lookup_idx
    ON football.player_positions (
        player_id,
        position_code,
        is_primary DESC,
        proficiency DESC
    )
    WHERE default_role_code IS NOT NULL;
