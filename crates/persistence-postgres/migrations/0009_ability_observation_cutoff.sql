-- 为球员能力观察补充真实写入时点，使 T-1h/T-6h/T-24h 推演不会读取截止时点之后导入的数据。
-- 已有记录没有历史写入时间，使用 observed_at 作为最保守且可复现的回填代理；新记录由默认值记录实际入库时间。

ALTER TABLE feature.player_ability_observations
    ADD COLUMN IF NOT EXISTS created_at timestamptz;

UPDATE feature.player_ability_observations
SET created_at = observed_at
WHERE created_at IS NULL;

ALTER TABLE feature.player_ability_observations
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN created_at SET NOT NULL;

CREATE INDEX IF NOT EXISTS ability_observations_player_cutoff_idx
    ON feature.player_ability_observations
       (player_id, created_at DESC, dimension_code, effective_from DESC);
