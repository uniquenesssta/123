-- 修复球队完整资料包中同一球员物理行同时生成国家队与俱乐部两条效力履历时，
-- player_team_period 子记录被错误视为重复的问题。

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_batch_sheet_row_entity_subrecord_key;

ALTER TABLE catalog.import_rows
    DROP COLUMN IF EXISTS subrecord_key;

ALTER TABLE catalog.import_rows
    ADD COLUMN subrecord_key text
    GENERATED ALWAYS AS (
        CASE entity_type
            WHEN 'player_ability' THEN
                COALESCE(NULLIF(BTRIM(payload ->> 'dimension_code'), ''), '__missing_dimension__')
            WHEN 'player_dynamic_tag' THEN
                COALESCE(NULLIF(BTRIM(payload ->> 'tag_code'), ''), '__missing_tag__')
            WHEN 'player_team_period' THEN
                COALESCE(
                    NULLIF(BTRIM(payload ->> 'team_id'), ''),
                    NULLIF(BTRIM(payload ->> 'team_key'), ''),
                    NULLIF(LOWER(BTRIM(payload ->> 'team_name')), ''),
                    '__missing_team__'
                )
            ELSE ''
        END
    ) STORED;

ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_batch_sheet_row_entity_subrecord_key
    UNIQUE (batch_id, sheet_name, row_number, entity_type, subrecord_key);
