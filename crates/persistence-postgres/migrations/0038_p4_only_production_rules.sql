-- P4-only production routing cleanup.
-- contract_sha256: b7fdbaa596131ad59721169797c1e64d816011571fbda6f212aeda3f2900b527
-- Historical P7 model runs remain readable; only active production packages and bindings are retired.

UPDATE model.rule_packages AS package
SET status = 'retired'
FROM model.versions AS version
JOIN model.definitions AS definition ON definition.id = version.model_id
WHERE package.model_version_id = version.id
  AND (definition.model_key = 'p7' OR definition.model_key LIKE 'p7\_%')
  AND package.status = 'active';

UPDATE model.competition_bindings AS binding
SET is_active = false
FROM model.rule_packages AS package
WHERE binding.rule_package_id = package.id
  AND package.status = 'retired'
  AND binding.is_active;

-- Only the newest active version of each built-in P4 production package participates in routing.
WITH ranked AS (
    SELECT
        id,
        row_number() OVER (
            PARTITION BY package_key
            ORDER BY created_at DESC, version DESC, id DESC
        ) AS version_rank
    FROM model.rule_packages
    WHERE package_key IN (
        'builtin.p4.league',
        'builtin.p4.group-stage',
        'builtin.p4.knockout-single',
        'builtin.p4.knockout-two-leg',
        'builtin.p4.friendly',
        'builtin.p4.custom',
        'builtin.p4.world-cup-knockout'
    )
      AND status = 'active'
)
UPDATE model.rule_packages AS package
SET status = 'deprecated'
FROM ranked
WHERE package.id = ranked.id
  AND ranked.version_rank > 1;

UPDATE model.competition_bindings AS binding
SET is_active = false
FROM model.rule_packages AS package
WHERE binding.rule_package_id = package.id
  AND package.status IN ('deprecated', 'retired')
  AND binding.is_active;
