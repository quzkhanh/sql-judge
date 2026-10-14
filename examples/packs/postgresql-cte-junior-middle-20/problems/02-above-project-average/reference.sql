
WITH project_avg AS (
  SELECT project_id, AVG(estimated_hours) AS avg_hours
  FROM tasks
  GROUP BY project_id
)
SELECT t.id AS task_id, t.project_id, t.estimated_hours
FROM tasks AS t
JOIN project_avg AS pa ON pa.project_id = t.project_id
WHERE t.estimated_hours > pa.avg_hours;
