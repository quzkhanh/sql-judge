WITH RECURSIVE chain AS (
  SELECT e.manager_id, e.id AS employee_id, 1 AS depth
  FROM employees AS e
  WHERE e.manager_id IS NOT NULL
  UNION ALL
  SELECT boss.manager_id, chain.employee_id, chain.depth + 1
  FROM chain
  JOIN employees AS boss ON boss.id = chain.manager_id
  WHERE boss.manager_id IS NOT NULL
)
SELECT manager_id, employee_id, depth
FROM chain;
