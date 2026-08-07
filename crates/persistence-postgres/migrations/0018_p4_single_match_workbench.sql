-- 接入G：单场研究、来源、冲突处理与历史记录工作台。
-- CONTRACT_SHA256 = 8ffcc0634d126bcf1ad7dc21a72778c2950b4cc130b338b41dcf871f01feb337

CREATE TABLE research.manual_route_overrides (
    id uuid PRIMARY KEY,
    task_id uuid NOT NULL REFERENCES platform.p4_freeze_tasks(id),
    research_run_id uuid NOT NULL REFERENCES research.runs(id),
    conflict_id uuid NOT NULL REFERENCES research.evidence_conflicts(id),
    original_route_id uuid NOT NULL REFERENCES research.evidence_routes(id),
    route_key text NOT NULL,
    field_key text NOT NULL,
    target_module text NOT NULL,
    target_slot text NOT NULL,
    entity_type text,
    entity_id uuid,
    decision_kind text NOT NULL CHECK (decision_kind IN ('select_evidence', 'accept_unknown')),
    selected_evidence_ids uuid[] NOT NULL DEFAULT '{}'::uuid[],
    selected_value jsonb NOT NULL DEFAULT 'null'::jsonb,
    route_status text NOT NULL CHECK (route_status IN ('routed', 'missing')),
    verification_state text NOT NULL CHECK (verification_state IN ('PROBABLE', 'NOT_FOUND')),
    reason text NOT NULL,
    actor text NOT NULL,
    note text,
    idempotency_key text NOT NULL UNIQUE,
    override_fingerprint text NOT NULL CHECK (override_fingerprint ~ '^[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (task_id, route_key),
    UNIQUE (task_id, conflict_id),
    CHECK (
        (decision_kind = 'select_evidence'
            AND cardinality(selected_evidence_ids) > 0
            AND selected_value <> 'null'::jsonb
            AND route_status = 'routed'
            AND verification_state = 'PROBABLE')
        OR
        (decision_kind = 'accept_unknown'
            AND cardinality(selected_evidence_ids) = 0
            AND selected_value = 'null'::jsonb
            AND route_status = 'missing'
            AND verification_state = 'NOT_FOUND')
    )
);

CREATE FUNCTION research.validate_manual_route_override_insert()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    task_state text;
    task_cutoff timestamptz;
    task_research_run_id uuid;
    task_match_id uuid;
    task_trace_id uuid;
    route_research_run_id uuid;
    route_match_id uuid;
    route_trace_id uuid;
    route_key_value text;
    route_field_key text;
    route_target_module text;
    route_target_slot text;
    route_entity_type text;
    route_entity_id uuid;
    route_status_value text;
    route_selected_evidence_ids uuid[];
    conflict_match_id uuid;
    conflict_field_key text;
    conflict_entity_type text;
    conflict_entity_id uuid;
    conflict_trace_id uuid;
    latest_evaluation_status text;
    valid_selected_count integer;
BEGIN
    SELECT state, data_cutoff_at, research_run_id, match_id, trace_id
    INTO task_state, task_cutoff, task_research_run_id, task_match_id, task_trace_id
    FROM platform.p4_freeze_tasks
    WHERE id = NEW.task_id
    FOR SHARE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'P4 freeze task does not exist: %', NEW.task_id;
    END IF;
    IF task_state NOT IN ('RESEARCH_PARTIAL', 'BLOCKED') THEN
        RAISE EXCEPTION 'manual route override is not allowed in task state %', task_state;
    END IF;
    IF task_research_run_id IS DISTINCT FROM NEW.research_run_id THEN
        RAISE EXCEPTION 'manual route override research run does not match task';
    END IF;
    IF clock_timestamp() >= task_cutoff THEN
        RAISE EXCEPTION 'manual route override is later than data cutoff %', task_cutoff;
    END IF;

    SELECT research_run_id, match_id, trace_id, route_key, field_key, target_module,
           target_slot, entity_type, entity_id, route_status, selected_evidence_ids
    INTO route_research_run_id, route_match_id, route_trace_id, route_key_value,
         route_field_key, route_target_module, route_target_slot, route_entity_type,
         route_entity_id, route_status_value, route_selected_evidence_ids
    FROM research.evidence_routes
    WHERE id = NEW.original_route_id
    FOR SHARE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'original evidence route does not exist: %', NEW.original_route_id;
    END IF;
    IF route_research_run_id IS DISTINCT FROM NEW.research_run_id
       OR route_match_id IS DISTINCT FROM task_match_id
       OR route_trace_id IS DISTINCT FROM task_trace_id
       OR route_key_value IS DISTINCT FROM NEW.route_key
       OR route_field_key IS DISTINCT FROM NEW.field_key
       OR route_target_module IS DISTINCT FROM NEW.target_module
       OR route_target_slot IS DISTINCT FROM NEW.target_slot
       OR route_entity_type IS DISTINCT FROM NEW.entity_type
       OR route_entity_id IS DISTINCT FROM NEW.entity_id
       OR route_status_value <> 'blocked_conflict' THEN
        RAISE EXCEPTION 'manual route override does not match the original blocked route';
    END IF;

    SELECT match_id, field_key, entity_type, entity_id, trace_id
    INTO conflict_match_id, conflict_field_key, conflict_entity_type,
         conflict_entity_id, conflict_trace_id
    FROM research.evidence_conflicts
    WHERE id = NEW.conflict_id
    FOR SHARE;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'evidence conflict does not exist: %', NEW.conflict_id;
    END IF;
    IF conflict_match_id IS DISTINCT FROM task_match_id
       OR conflict_trace_id IS DISTINCT FROM task_trace_id
       OR conflict_field_key IS DISTINCT FROM NEW.field_key
       OR conflict_entity_type IS DISTINCT FROM NEW.entity_type
       OR conflict_entity_id IS DISTINCT FROM NEW.entity_id THEN
        RAISE EXCEPTION 'manual route override conflict does not belong to the task route';
    END IF;
    IF NOT EXISTS (
        SELECT 1
        FROM research.evidence_conflict_members member
        WHERE member.conflict_id = NEW.conflict_id
          AND member.evidence_id = ANY(route_selected_evidence_ids)
    ) THEN
        RAISE EXCEPTION 'original blocked route does not reference the selected conflict';
    END IF;

    SELECT evaluation_status
    INTO latest_evaluation_status
    FROM research.conflict_evaluations
    WHERE conflict_id = NEW.conflict_id
      AND research_run_id = NEW.research_run_id
    ORDER BY created_at DESC, id DESC
    LIMIT 1;

    IF latest_evaluation_status IS DISTINCT FROM 'manual_required' THEN
        RAISE EXCEPTION 'evidence conflict is not awaiting manual resolution';
    END IF;

    IF NEW.decision_kind = 'select_evidence' THEN
        SELECT count(*)::integer
        INTO valid_selected_count
        FROM research.evidence_conflict_members member
        JOIN research.evidence_claims claim ON claim.id = member.evidence_id
        WHERE member.conflict_id = NEW.conflict_id
          AND claim.id = ANY(NEW.selected_evidence_ids)
          AND claim.research_run_id = NEW.research_run_id
          AND claim.match_id = task_match_id
          AND claim.field_key = NEW.field_key
          AND claim.entity_type IS NOT DISTINCT FROM NEW.entity_type
          AND claim.entity_id IS NOT DISTINCT FROM NEW.entity_id
          AND claim.value = NEW.selected_value;

        IF valid_selected_count <> cardinality(NEW.selected_evidence_ids) THEN
            RAISE EXCEPTION 'selected evidence does not belong to the conflict or has a different fact value';
        END IF;
    END IF;
    RETURN NEW;
END;
$function$;

CREATE TRIGGER manual_route_overrides_validate_insert
BEFORE INSERT ON research.manual_route_overrides
FOR EACH ROW EXECUTE FUNCTION research.validate_manual_route_override_insert();

CREATE INDEX manual_route_overrides_task_route_idx
    ON research.manual_route_overrides (task_id, route_key, created_at DESC, id DESC);
CREATE INDEX manual_route_overrides_conflict_idx
    ON research.manual_route_overrides (conflict_id, created_at DESC, id DESC);

CREATE TRIGGER manual_route_overrides_immutable
BEFORE UPDATE OR DELETE ON research.manual_route_overrides
FOR EACH ROW EXECUTE FUNCTION platform.reject_immutable_record_mutation();

DO $migration$
DECLARE
    existing_hash text;
BEGIN
    SELECT content_sha256
    INTO existing_hash
    FROM platform.integration_contracts
    WHERE contract_key = 'p4-single-match-workbench'
      AND contract_version = '1.0.0';

    IF existing_hash IS NULL THEN
        INSERT INTO platform.integration_contracts (
            contract_key, contract_version, baseline_source_version,
            release_version, schema_version, content_sha256, stage, metadata
        ) VALUES (
            'p4-single-match-workbench',
            '1.0.0',
            '0.12.0',
            '0.13.0',
            'football.p4-workbench-contract.v1',
            '8ffcc0634d126bcf1ad7dc21a72778c2950b4cc130b338b41dcf871f01feb337',
            'G',
            jsonb_build_object(
                'contract_path', 'contracts/model-workbench-contract.json',
                'surface', 'single_match_workspace',
                'manual_conflict_policy', 'append_only_before_cutoff',
                'formal_snapshot_mutable', false
            )
        );
    ELSIF existing_hash <> '8ffcc0634d126bcf1ad7dc21a72778c2950b4cc130b338b41dcf871f01feb337' THEN
        RAISE EXCEPTION 'P4 workbench contract hash conflict: existing %, expected %',
            existing_hash, '8ffcc0634d126bcf1ad7dc21a72778c2950b4cc130b338b41dcf871f01feb337';
    END IF;
END;
$migration$;
