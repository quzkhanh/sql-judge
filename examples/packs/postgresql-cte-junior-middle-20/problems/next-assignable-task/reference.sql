
WITH active_counts AS (
  SELECT task_id, COUNT(*) AS active_count
  FROM assignments WHERE status = 'ACTIVE' GROUP BY task_id
), available AS (
  SELECT t.project_id, t.id AS task_id, t.status, t.priority, t.created_at,
         t.capacity::bigint - COALESCE(ac.active_count,0::bigint) AS available_slots
  FROM tasks AS t
  JOIN projects AS p ON p.id = t.project_id AND p.status = 'ACTIVE'
  LEFT JOIN active_counts AS ac ON ac.task_id = t.id
  WHERE t.status IN ('IN_PROGRESS','AVAILABLE')
    AND t.capacity::bigint > COALESCE(ac.active_count,0::bigint)
), ranked AS (
  SELECT project_id, task_id, available_slots,
         ROW_NUMBER() OVER (
           PARTITION BY project_id
           ORDER BY CASE status WHEN 'IN_PROGRESS' THEN 0 ELSE 1 END,
                    priority DESC, created_at ASC, task_id ASC
         ) AS rn
  FROM available
)
SELECT project_id, task_id, available_slots FROM ranked WHERE rn = 1;
