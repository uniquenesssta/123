-- 阶段 D2：将普通比赛事件升级为可查询、可核验、可修订的正式事实。
-- 保留 0040 的基础表与兼容数据，新增稳定事件身份、比分快照、来源和修订状态。

ALTER TABLE review.match_events
    DROP CONSTRAINT IF EXISTS match_events_event_type_check;

ALTER TABLE review.match_events
    ADD COLUMN event_key text,
    ADD COLUMN sequence_no integer,
    ADD COLUMN home_score smallint,
    ADD COLUMN away_score smallint,
    ADD COLUMN verification_status text NOT NULL DEFAULT 'unverified',
    ADD COLUMN revision_status text NOT NULL DEFAULT 'active',
    ADD COLUMN verified_at timestamptz,
    ADD COLUMN source_document_id uuid REFERENCES catalog.source_documents(id) ON DELETE SET NULL,
    ADD COLUMN source_package_id uuid REFERENCES review.match_review_package_workflows(package_id) ON DELETE SET NULL,
    ADD COLUMN revision_of_event_id uuid REFERENCES review.match_events(id) ON DELETE SET NULL,
    ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();

UPDATE review.match_events
SET event_key = 'legacy:' || id::text,
    verification_status = 'verified',
    verified_at = COALESCE(verified_at, recorded_at)
WHERE event_key IS NULL;

WITH ranked AS (
    SELECT id,
           row_number() OVER (
               PARTITION BY match_id
               ORDER BY minute, stoppage_minute NULLS FIRST, recorded_at, id
           )::integer AS sequence_no
    FROM review.match_events
)
UPDATE review.match_events event
SET sequence_no = ranked.sequence_no
FROM ranked
WHERE event.id = ranked.id
  AND event.sequence_no IS NULL;

ALTER TABLE review.match_events
    ALTER COLUMN event_key SET NOT NULL,
    ALTER COLUMN sequence_no SET NOT NULL;

ALTER TABLE review.match_events
    ADD CONSTRAINT match_events_event_key_not_blank_check
        CHECK (BTRIM(event_key) <> ''),
    ADD CONSTRAINT match_events_sequence_no_check
        CHECK (sequence_no > 0),
    ADD CONSTRAINT match_events_event_type_check
        CHECK (event_type IN (
            'substitution', 'goal', 'own_goal', 'assist',
            'penalty_goal', 'penalty_missed',
            'yellow_card', 'second_yellow_card', 'red_card',
            'injury', 'var', 'formation_change', 'goalkeeper_change', 'other'
        )),
    ADD CONSTRAINT match_events_score_pair_check
        CHECK (
            (home_score IS NULL AND away_score IS NULL)
            OR
            (home_score IS NOT NULL AND away_score IS NOT NULL
             AND home_score >= 0 AND away_score >= 0)
        ),
    ADD CONSTRAINT match_events_verification_status_check
        CHECK (verification_status IN ('unverified', 'verified', 'disputed')),
    ADD CONSTRAINT match_events_verified_at_check
        CHECK (verification_status <> 'verified' OR verified_at IS NOT NULL),
    ADD CONSTRAINT match_events_revision_status_check
        CHECK (revision_status IN ('active', 'corrected', 'cancelled', 'superseded')),
    ADD CONSTRAINT match_events_revision_not_self_check
        CHECK (revision_of_event_id IS NULL OR revision_of_event_id <> id);

CREATE UNIQUE INDEX match_events_match_event_key_uidx
    ON review.match_events (match_id, event_key);
CREATE INDEX match_events_match_sequence_idx
    ON review.match_events (match_id, sequence_no, id);
CREATE INDEX match_events_match_revision_idx
    ON review.match_events (match_id, revision_status, event_type, minute);
CREATE INDEX match_events_team_type_idx
    ON review.match_events (team_id, event_type, match_id)
    WHERE team_id IS NOT NULL;
CREATE INDEX match_events_source_package_idx
    ON review.match_events (source_package_id, match_id)
    WHERE source_package_id IS NOT NULL;
CREATE INDEX match_events_revision_of_idx
    ON review.match_events (revision_of_event_id)
    WHERE revision_of_event_id IS NOT NULL;
