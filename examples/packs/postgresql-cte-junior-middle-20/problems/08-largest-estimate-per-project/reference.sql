
WITH ranked AS (
  SELECT project_id, id AS task_id, estimated_hours,
         ROW_NUMBER() OVER (
           PARTITION BY project_id
           ORDER BY estimated_hours DESC, id ASC
         ) AS rn
  FROM tasks WHERE estimated_hours IS NOT NULL
)
SELECT project_id, task_id, estimated_hours FROM ranked WHERE rn = 1;
