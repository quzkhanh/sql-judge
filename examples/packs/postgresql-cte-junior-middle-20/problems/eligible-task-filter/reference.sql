
WITH active_projects AS (
  SELECT id FROM projects WHERE status = 'ACTIVE'
)
SELECT t.id AS task_id, t.project_id, t.priority
FROM tasks AS t
JOIN active_projects AS p ON p.id = t.project_id
WHERE t.status = 'AVAILABLE'
  AND t.created_at >= DATE '2026-01-01';
