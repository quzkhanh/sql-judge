
WITH active_counts AS (
  SELECT task_id, COUNT(*) AS active_count
  FROM assignments WHERE status = 'ACTIVE'
  GROUP BY task_id
), candidates AS (
  SELECT t.id AS task_id,
         COALESCE(ac.active_count,0::bigint) AS active_count,
         t.capacity::bigint - COALESCE(ac.active_count,0::bigint) AS remaining_slots
  FROM tasks AS t
  JOIN projects AS p ON p.id = t.project_id AND p.status = 'ACTIVE'
  LEFT JOIN active_counts AS ac ON ac.task_id = t.id
  WHERE t.status IN ('AVAILABLE','IN_PROGRESS')
)
SELECT task_id, active_count, remaining_slots
FROM candidates WHERE remaining_slots > 0;
