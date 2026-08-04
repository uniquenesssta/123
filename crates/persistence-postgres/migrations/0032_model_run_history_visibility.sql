-- 推演历史只隐藏列表展示，不破坏模型运行、快照、复盘和审计血缘。
ALTER TABLE model.runs
    ADD COLUMN IF NOT EXISTS history_hidden_at timestamptz,
    ADD COLUMN IF NOT EXISTS history_hidden_reason text;

CREATE INDEX IF NOT EXISTS model_runs_visible_history_idx
    ON model.runs (created_at DESC, id DESC)
    WHERE status = 'succeeded' AND history_hidden_at IS NULL;
