-- E2：球队常用阵容预设。预设属于球队，应用到比赛时必须复制为独立比赛阵容，禁止反向污染预设。

CREATE TABLE IF NOT EXISTS football.team_lineup_presets (
    id uuid PRIMARY KEY,
    team_id uuid NOT NULL REFERENCES football.teams(id) ON DELETE CASCADE,
    name text NOT NULL,
    formation_id uuid REFERENCES football.formations(id),
    coach_id uuid REFERENCES football.coaches(id),
    usage_context text NOT NULL DEFAULT 'general',
    usage_probability double precision,
    is_default boolean NOT NULL DEFAULT false,
    status text NOT NULL DEFAULT 'active',
    version integer NOT NULL DEFAULT 1,
    source_lineup_id uuid REFERENCES football.lineups(id) ON DELETE SET NULL,
    notes text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT team_lineup_presets_name_check CHECK (btrim(name) <> ''),
    CONSTRAINT team_lineup_presets_probability_check CHECK (
        usage_probability IS NULL OR usage_probability BETWEEN 0 AND 1
    ),
    CONSTRAINT team_lineup_presets_status_check CHECK (status IN ('active', 'archived')),
    CONSTRAINT team_lineup_presets_version_check CHECK (version >= 1)
);

CREATE UNIQUE INDEX IF NOT EXISTS team_lineup_presets_active_name_uq
    ON football.team_lineup_presets (team_id, lower(btrim(name)))
    WHERE status = 'active';

CREATE UNIQUE INDEX IF NOT EXISTS team_lineup_presets_one_default_uq
    ON football.team_lineup_presets (team_id)
    WHERE status = 'active' AND is_default;

CREATE INDEX IF NOT EXISTS team_lineup_presets_team_idx
    ON football.team_lineup_presets (team_id, status, is_default DESC, updated_at DESC);

CREATE TABLE IF NOT EXISTS football.team_lineup_preset_members (
    preset_id uuid NOT NULL REFERENCES football.team_lineup_presets(id) ON DELETE CASCADE,
    player_id uuid NOT NULL REFERENCES football.players(id),
    position_code text REFERENCES football.positions(code),
    role_code text,
    is_starter boolean NOT NULL,
    shirt_number smallint,
    expected_minutes smallint,
    sequence_no smallint NOT NULL,
    bench_order smallint,
    is_captain boolean NOT NULL DEFAULT false,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (preset_id, player_id),
    CONSTRAINT team_lineup_preset_members_sequence_check CHECK (sequence_no >= 0),
    CONSTRAINT team_lineup_preset_members_shirt_check CHECK (
        shirt_number IS NULL OR shirt_number BETWEEN 1 AND 99
    ),
    CONSTRAINT team_lineup_preset_members_minutes_check CHECK (
        expected_minutes IS NULL OR expected_minutes BETWEEN 0 AND 130
    ),
    CONSTRAINT team_lineup_preset_members_bench_check CHECK (
        bench_order IS NULL OR bench_order BETWEEN 1 AND 99
    ),
    CONSTRAINT team_lineup_preset_members_metadata_check CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX IF NOT EXISTS team_lineup_preset_members_order_idx
    ON football.team_lineup_preset_members (
        preset_id, is_starter DESC, sequence_no, bench_order, player_id
    );

CREATE UNIQUE INDEX IF NOT EXISTS team_lineup_preset_one_captain_uq
    ON football.team_lineup_preset_members (preset_id)
    WHERE is_captain;
