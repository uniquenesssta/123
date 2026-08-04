-- 阶段 3.5：Excel 导入预检、冲突处理与事务提交。

CREATE TABLE catalog.import_rows (
    id uuid PRIMARY KEY,
    batch_id uuid NOT NULL REFERENCES catalog.import_batches(id) ON DELETE CASCADE,
    sheet_name text NOT NULL,
    row_number integer NOT NULL CHECK (row_number >= 2),
    entity_type text NOT NULL CHECK (entity_type IN (
        'team', 'player', 'player_name', 'player_position',
        'player_team_period', 'player_ability', 'player_availability',
        'external_entity_id'
    )),
    requested_action text NOT NULL CHECK (requested_action IN ('add', 'update', 'skip')),
    status text NOT NULL CHECK (status IN (
        'ready_add', 'ready_update', 'conflict', 'error', 'skip', 'imported'
    )),
    message text,
    payload jsonb NOT NULL,
    matched_entity_id uuid,
    conflict_candidates jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    imported_at timestamptz,
    UNIQUE (batch_id, sheet_name, row_number)
);

CREATE INDEX import_rows_batch_status_idx
    ON catalog.import_rows (batch_id, status, row_number);
CREATE INDEX import_rows_entity_match_idx
    ON catalog.import_rows (entity_type, matched_entity_id)
    WHERE matched_entity_id IS NOT NULL;

ALTER TABLE catalog.import_batches
    ADD COLUMN source_file_name text,
    ADD COLUMN source_sha256 text,
    ADD COLUMN import_mode text CHECK (import_mode IN ('add_only', 'add_and_update'));

CREATE UNIQUE INDEX import_batches_source_hash_active_idx
    ON catalog.import_batches (source_sha256, import_type)
    WHERE source_sha256 IS NOT NULL AND status IN ('pending', 'running', 'succeeded');
