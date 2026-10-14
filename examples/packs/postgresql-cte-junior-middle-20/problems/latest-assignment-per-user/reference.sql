
WITH ranked AS (
  SELECT user_id, task_id, assigned_at,
         ROW_NUMBER() OVER (
           PARTITION BY user_id ORDER BY assigned_at DESC, id DESC
         ) AS rn
  FROM assignments
)
SELECT user_id, task_id, assigned_at FROM ranked WHERE rn = 1;
