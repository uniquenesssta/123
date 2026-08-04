-- 记录阵容与正式赛果实际进入平台的时间。赛前快照同时检查业务时间和入库时间，
-- 防止赛后补录的预计/确认阵容或历史赛果被错误用于更早的历史赛前推演。
-- 旧记录无法还原真实入库时点，统一以升级时刻回填；历史回放会保守地排除这些歧义数据。
ALTER TABLE football.lineups
    ADD COLUMN IF NOT EXISTS created_at timestamptz;

UPDATE football.lineups
SET created_at = now()
WHERE created_at IS NULL;

ALTER TABLE football.lineups
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN created_at SET NOT NULL;

CREATE INDEX IF NOT EXISTS lineups_pre_match_cutoff_idx
    ON football.lineups (match_id, team_id, captured_at DESC, created_at DESC)
    WHERE status = 'active' AND lineup_type IN ('expected', 'confirmed');

ALTER TABLE football.match_results
    ADD COLUMN IF NOT EXISTS created_at timestamptz;

UPDATE football.match_results
SET created_at = now()
WHERE created_at IS NULL;

ALTER TABLE football.match_results
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN created_at SET NOT NULL;

CREATE INDEX IF NOT EXISTS match_results_pre_match_cutoff_idx
    ON football.match_results (finalized_at DESC, created_at DESC);
