-- 完整资料包的一条球员物理行会为每个能力维度和动态标签生成独立审计子记录。
-- 0029 只区分到 entity_type，仍会让多个 player_ability / player_dynamic_tag 互相冲突。

ALTER TABLE catalog.import_rows
    ADD COLUMN IF NOT EXISTS subrecord_key text
    GENERATED ALWAYS AS (
        CASE entity_type
            WHEN 'player_ability' THEN
                COALESCE(NULLIF(BTRIM(payload ->> 'dimension_code'), ''), '__missing_dimension__')
            WHEN 'player_dynamic_tag' THEN
                COALESCE(NULLIF(BTRIM(payload ->> 'tag_code'), ''), '__missing_tag__')
            ELSE ''
        END
    ) STORED;

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_batch_sheet_row_entity_key;

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_batch_sheet_row_entity_subrecord_key;

ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_batch_sheet_row_entity_subrecord_key
    UNIQUE (batch_id, sheet_name, row_number, entity_type, subrecord_key);
