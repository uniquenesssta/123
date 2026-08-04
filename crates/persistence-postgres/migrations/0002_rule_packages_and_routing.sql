ALTER TABLE model.rule_packages
    ADD COLUMN display_name text,
    ADD COLUMN format_version text NOT NULL DEFAULT 'football.rule-package.v1',
    ADD COLUMN competition_kind text,
    ADD COLUMN profile jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN routing jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN feature_requirements jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN output_contract jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN model_version_id uuid REFERENCES model.versions(id),
    ADD COLUMN parameter_set_id uuid REFERENCES model.parameter_sets(id),
    ADD COLUMN priority integer NOT NULL DEFAULT 0,
    ADD COLUMN source_document_id uuid REFERENCES catalog.source_documents(id);

UPDATE model.rule_packages
SET display_name = package_key
WHERE display_name IS NULL;

ALTER TABLE model.rule_packages
    ALTER COLUMN display_name SET NOT NULL,
    ADD CONSTRAINT rule_packages_competition_kind_check CHECK (
        competition_kind IS NULL OR competition_kind IN (
            'league',
            'group_stage',
            'knockout_single_leg',
            'knockout_two_leg',
            'friendly',
            'custom'
        )
    );

CREATE INDEX rule_packages_kind_priority_idx
    ON model.rule_packages (competition_kind, priority DESC, created_at DESC)
    WHERE status = 'active';

ALTER TABLE model.competition_bindings
    ADD COLUMN binding_name text,
    ADD COLUMN created_at timestamptz NOT NULL DEFAULT now();

UPDATE model.competition_bindings
SET binding_name = '历史绑定-' || left(id::text, 8)
WHERE binding_name IS NULL;

ALTER TABLE model.competition_bindings
    ALTER COLUMN binding_name SET NOT NULL,
    ADD CONSTRAINT competition_bindings_competition_kind_check CHECK (
        competition_kind IS NULL OR competition_kind IN (
            'league',
            'group_stage',
            'knockout_single_leg',
            'knockout_two_leg',
            'friendly',
            'custom'
        )
    );

CREATE INDEX competition_bindings_specificity_idx
    ON model.competition_bindings (
        stage_id,
        season_id,
        competition_id,
        competition_kind,
        priority DESC,
        created_at DESC
    )
    WHERE is_active;

ALTER TABLE model.runs
    ADD COLUMN route_binding_id uuid REFERENCES model.competition_bindings(id);

CREATE INDEX model_runs_rule_package_idx
    ON model.runs (rule_package_id, created_at DESC)
    WHERE rule_package_id IS NOT NULL;

ALTER TABLE model.rule_packages
    ALTER COLUMN competition_kind SET NOT NULL,
    ALTER COLUMN model_version_id SET NOT NULL,
    ALTER COLUMN parameter_set_id SET NOT NULL;

ALTER TABLE model.competition_bindings
    ALTER COLUMN rule_package_id SET NOT NULL,
    ADD CONSTRAINT competition_bindings_scope_hierarchy_check CHECK (
        (stage_id IS NULL OR (season_id IS NOT NULL AND competition_id IS NOT NULL))
        AND (season_id IS NULL OR competition_id IS NOT NULL)
        AND (
            competition_id IS NOT NULL
            OR season_id IS NOT NULL
            OR stage_id IS NOT NULL
            OR competition_kind IS NOT NULL
        )
    );
