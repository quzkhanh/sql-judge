
WITH ranked AS (
  SELECT project_id, id AS task_id, created_at,
         ROW_NUMBER() OVER (
           PARTITION BY project_id ORDER BY created_at DESC, id DESC
         ) AS rn
  FROM tasks
)
SELECT project_id, task_id, created_at FROM ranked WHERE rn = 2;
