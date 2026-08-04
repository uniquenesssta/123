-- 受控强制清除：只允许显式事务在本地开启 football.force_purge 后删除不可变账本记录。
-- 正常更新和删除仍保持原有不可变约束；该开关不会跨事务泄漏。

CREATE OR REPLACE FUNCTION platform.force_purge_enabled()
RETURNS boolean
LANGUAGE sql
STABLE
AS $function$
    SELECT COALESCE(current_setting('football.force_purge', true), '') = 'on';
$function$;

CREATE OR REPLACE FUNCTION platform.reject_immutable_record_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    IF TG_OP = 'DELETE' AND platform.force_purge_enabled() THEN
        RETURN OLD;
    END IF;
    RAISE EXCEPTION '%.% records are append-only; publish a new version or event instead',
        TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$function$;

CREATE OR REPLACE FUNCTION feature.reject_frozen_snapshot_delete()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    IF platform.force_purge_enabled() THEN
        RETURN OLD;
    END IF;
    IF OLD.frozen_at IS NOT NULL THEN
        RAISE EXCEPTION 'frozen prematch snapshots cannot be deleted';
    END IF;
    RETURN OLD;
END;
$function$;

CREATE OR REPLACE FUNCTION analytics.guard_postmatch_evaluation_sample()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
DECLARE
    linked_review_id uuid;
    linked_run_id uuid;
    linked_model_version_id uuid;
    linked_parameter_set_id uuid;
    linked_competition_id uuid;
    linked_profile_id uuid;
    linked_horizon text;
    linked_kickoff_time timestamptz;
    linked_actual_outcome text;
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF platform.force_purge_enabled() THEN
            RETURN OLD;
        END IF;
        IF OLD.settlement_id IS NOT NULL THEN
            RAISE EXCEPTION 'formal postmatch evaluation sample is immutable';
        END IF;
        RETURN OLD;
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF OLD.settlement_id IS NOT NULL OR NEW.settlement_id IS NOT NULL THEN
            RAISE EXCEPTION 'formal postmatch evaluation sample is immutable';
        END IF;
        RETURN NEW;
    END IF;
    IF NEW.settlement_id IS NULL THEN
        RETURN NEW;
    END IF;

    SELECT settlement.match_review_id, settlement.model_run_id,
           settlement.model_version_id, settlement.parameter_set_id,
           settlement.competition_id, settlement.competition_profile_id,
           settlement.horizon, fixture.kickoff_time,
           CASE
               WHEN settlement.home_goals_90 > settlement.away_goals_90 THEN 'home_win'
               WHEN settlement.home_goals_90 < settlement.away_goals_90 THEN 'away_win'
               ELSE 'draw'
           END
    INTO linked_review_id, linked_run_id, linked_model_version_id,
         linked_parameter_set_id, linked_competition_id, linked_profile_id,
         linked_horizon, linked_kickoff_time, linked_actual_outcome
    FROM review.postmatch_settlements settlement
    JOIN football.matches fixture ON fixture.id = settlement.match_id
    WHERE settlement.id = NEW.settlement_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'formal postmatch evaluation sample requires a settlement';
    END IF;
    IF NEW.review_id <> linked_review_id
       OR NEW.run_id <> linked_run_id
       OR NEW.model_version_id <> linked_model_version_id
       OR NEW.parameter_set_id <> linked_parameter_set_id
       OR NEW.competition_id IS DISTINCT FROM linked_competition_id
       OR NEW.competition_profile_id IS DISTINCT FROM linked_profile_id
       OR NEW.snapshot_type <> linked_horizon
       OR NEW.kickoff_time <> linked_kickoff_time
       OR NEW.actual_outcome <> linked_actual_outcome
       OR NEW.calculation_version <> 'postmatch-monitoring-v1' THEN
        RAISE EXCEPTION 'formal postmatch evaluation sample identity mismatch';
    END IF;
    RETURN NEW;
END;
$function$;

CREATE OR REPLACE FUNCTION review.reject_postmatch_record_mutation()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
BEGIN
    IF TG_OP = 'DELETE' AND platform.force_purge_enabled() THEN
        RETURN OLD;
    END IF;
    RAISE EXCEPTION 'postmatch ledger %.% is immutable', TG_TABLE_SCHEMA, TG_TABLE_NAME;
END;
$function$;
