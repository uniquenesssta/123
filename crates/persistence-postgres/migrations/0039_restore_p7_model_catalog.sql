-- Restore P7 as an explicit selectable model family while preserving P4 as the automatic default.
-- Existing historical P7 runs remain unchanged. Only the newest built-in P7 package per key is reactivated.
-- contract_sha256: 0786dc064a0af4597c6878a81077835f3c3ef4f0d64105eb270eedaad63a5f9c

WITH ranked AS (
    SELECT
        id,
        row_number() OVER (
            PARTITION BY package_key
            ORDER BY created_at DESC, version DESC, id DESC
        ) AS version_rank
    FROM model.rule_packages
    WHERE package_key IN (
        'builtin.p7.league',
        'builtin.p7.group-stage',
        'builtin.p7.knockout-single',
        'builtin.p7.knockout-two-leg',
        'builtin.p7.friendly',
        'builtin.p7.custom'
    )
)
UPDATE model.rule_packages AS package
SET status = CASE WHEN ranked.version_rank = 1 THEN 'active' ELSE 'deprecated' END
FROM ranked
WHERE package.id = ranked.id;

UPDATE model.competition_bindings AS binding
SET is_active = true
FROM model.rule_packages AS package
WHERE binding.rule_package_id = package.id
  AND package.package_key LIKE 'builtin.p7.%'
  AND package.status = 'active';

UPDATE model.competition_bindings AS binding
SET is_active = false
FROM model.rule_packages AS package
WHERE binding.rule_package_id = package.id
  AND package.package_key LIKE 'builtin.p7.%'
  AND package.status <> 'active';
