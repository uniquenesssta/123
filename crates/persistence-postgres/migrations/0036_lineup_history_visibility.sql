-- 第一阶段：阵容历史支持删除未引用版本、归档已引用版本，并保持模型血缘。

ALTER TABLE football.lineups
    ADD COLUMN IF NOT EXISTS history_hidden_at timestamptz,
    ADD COLUMN IF NOT EXISTS history_hidden_reason text;

CREATE INDEX IF NOT EXISTS lineups_visible_history_idx
    ON football.lineups (match_id, captured_at DESC, id DESC)
    WHERE history_hidden_at IS NULL;
