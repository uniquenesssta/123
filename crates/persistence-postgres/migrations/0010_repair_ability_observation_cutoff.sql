-- 兼容修复：确保旧数据库具备能力观察的真实写入时点。
-- 0009 已负责标准升级；本迁移用于修复曾因 Cargo 迁移编译缓存而未嵌入 0009 的客户端构建。

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'feature'
          AND table_name = 'player_ability_observations'
          AND column_name = 'created_at'
          AND is_nullable = 'NO'
          AND column_default IS NOT NULL
    ) THEN
        ALTER TABLE feature.player_ability_observations
            ADD COLUMN IF NOT EXISTS created_at timestamptz;

        UPDATE feature.player_ability_observations
        SET created_at = observed_at
        WHERE created_at IS NULL;

        ALTER TABLE feature.player_ability_observations
            ALTER COLUMN created_at SET DEFAULT now(),
            ALTER COLUMN created_at SET NOT NULL;
    END IF;
END;
$$;

CREATE INDEX IF NOT EXISTS ability_observations_player_cutoff_idx
    ON feature.player_ability_observations
       (player_id, created_at DESC, dimension_code, effective_from DESC);
