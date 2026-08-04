-- 完整资料包的一条 Excel 业务行会拆分为多个不同实体的预检记录。
-- 导入行身份必须包含 entity_type，避免球员主体、关系、位置、能力、可用性和动态标签互相冲突。

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_batch_id_sheet_name_row_number_key;

ALTER TABLE catalog.import_rows
    DROP CONSTRAINT IF EXISTS import_rows_batch_sheet_row_entity_key;

ALTER TABLE catalog.import_rows
    ADD CONSTRAINT import_rows_batch_sheet_row_entity_key
    UNIQUE (batch_id, sheet_name, row_number, entity_type);
