
WITH project_stats AS (
  SELECT p.id AS project_id,
         COUNT(t.id) AS total_tasks,
         COUNT(t.id) FILTER (WHERE t.status = 'AVAILABLE') AS available_tasks,
         COUNT(t.id) FILTER (WHERE t.status = 'DONE') AS completed_tasks
  FROM projects AS p
  LEFT JOIN tasks AS t ON t.project_id = p.id
  GROUP BY p.id
)
SELECT project_id, total_tasks, available_tasks, completed_tasks
FROM project_stats;
