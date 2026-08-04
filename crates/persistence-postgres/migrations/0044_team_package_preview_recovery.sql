-- 支持“已确认不可出场但原因类型尚未细分”的通用状态。
-- 该状态区别于 unknown：unavailable 表示明确不能出场，unknown 表示尚未核实。
ALTER TABLE football.player_availability
    DROP CONSTRAINT IF EXISTS player_availability_status_check;

ALTER TABLE football.player_availability
    ADD CONSTRAINT player_availability_status_check
    CHECK (status IN (
        'available', 'doubtful', 'unavailable', 'injured',
        'suspended', 'rested', 'returning', 'unknown'
    ));

ALTER TABLE football.lineup_players
    DROP CONSTRAINT IF EXISTS lineup_players_availability_status_check;

ALTER TABLE football.lineup_players
    ADD CONSTRAINT lineup_players_availability_status_check
    CHECK (
        availability_status IS NULL OR availability_status IN (
            'available', 'doubtful', 'unavailable', 'injured',
            'suspended', 'rested', 'returning', 'unknown'
        )
    );
