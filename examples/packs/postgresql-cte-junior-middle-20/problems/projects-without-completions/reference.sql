
WITH completed_projects AS (
  SELECT project_id FROM tasks WHERE status = 'DONE' GROUP BY project_id
)
SELECT p.id AS project_id, p.name AS project_name
FROM projects AS p
LEFT JOIN completed_projects AS cp ON cp.project_id = p.id
WHERE cp.project_id IS NULL;
