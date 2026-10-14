
WITH counts AS (
  SELECT p.id AS project_id, COUNT(t.id) AS task_count
  FROM projects AS p LEFT JOIN tasks AS t ON t.project_id = p.id
  GROUP BY p.id
), baseline AS (
  SELECT AVG(task_count) AS avg_count FROM counts
)
SELECT c.project_id, c.task_count
FROM counts AS c CROSS JOIN baseline AS b
WHERE c.task_count > b.avg_count;
