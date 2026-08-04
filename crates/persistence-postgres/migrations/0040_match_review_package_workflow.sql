-- 赛后复盘资料包固定链路与结构化比赛事件。
-- 赛前阵容和模型快照继续保留在原表；本迁移仅记录独立的赛后事实与工作流状态。

CREATE TABLE IF NOT EXISTS review.match_events (
    id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    event_type text NOT NULL CHECK (
        event_type IN (
            'substitution', 'goal', 'assist', 'yellow_card', 'red_card',
            'injury', 'var', 'formation_change', 'other'
        )
    ),
    team_id uuid REFERENCES football.teams(id),
    player_id uuid REFERENCES football.players(id),
    related_player_id uuid REFERENCES football.players(id),
    minute smallint NOT NULL CHECK (minute BETWEEN 0 AND 150),
    stoppage_minute smallint CHECK (stoppage_minute IS NULL OR stoppage_minute BETWEEN 0 AND 30),
    period text NOT NULL CHECK (
        period IN ('first_half', 'second_half', 'extra_time_first', 'extra_time_second', 'normal_time')
    ),
    description text,
    source_urls text[] NOT NULL DEFAULT ARRAY[]::text[],
    confidence double precision NOT NULL DEFAULT 1 CHECK (confidence BETWEEN 0 AND 1),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    recorded_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS match_events_match_time_idx
    ON review.match_events (match_id, minute, stoppage_minute, id);
CREATE INDEX IF NOT EXISTS match_events_match_type_idx
    ON review.match_events (match_id, event_type, minute);
CREATE INDEX IF NOT EXISTS match_events_player_idx
    ON review.match_events (player_id, match_id)
    WHERE player_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS match_events_related_player_idx
    ON review.match_events (related_player_id, match_id)
    WHERE related_player_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS review.match_review_package_workflows (
    package_id uuid PRIMARY KEY,
    match_id uuid NOT NULL REFERENCES football.matches(id) ON DELETE CASCADE,
    match_key text NOT NULL,
    status text NOT NULL CHECK (
        status IN (
            'exported', 'preview_blocked', 'preview_valid', 'confirmed',
            'facts_committed', 'review_created', 'settled', 'superseded'
        )
    ),
    export_path text NOT NULL,
    export_sha256 text NOT NULL,
    pre_match_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    export_database_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    import_path text,
    import_sha256 text,
    preview_ready boolean NOT NULL DEFAULT false,
    preview_payload jsonb,
    confirmed_by text,
    confirmation_note text,
    review_id uuid REFERENCES review.match_reviews(id) ON DELETE SET NULL,
    exported_at timestamptz NOT NULL DEFAULT now(),
    previewed_at timestamptz,
    confirmed_at timestamptz,
    facts_committed_at timestamptz,
    review_created_at timestamptz,
    settled_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS match_review_package_one_active_per_match_uidx
    ON review.match_review_package_workflows (match_id)
    WHERE status NOT IN ('settled', 'superseded');
CREATE INDEX IF NOT EXISTS match_review_package_match_updated_idx
    ON review.match_review_package_workflows (match_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS match_review_package_review_idx
    ON review.match_review_package_workflows (review_id)
    WHERE review_id IS NOT NULL;
