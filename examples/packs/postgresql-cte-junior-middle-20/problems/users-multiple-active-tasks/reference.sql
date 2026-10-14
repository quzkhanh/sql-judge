
WITH active_load AS (
  SELECT user_id, COUNT(DISTINCT task_id) AS active_task_count
  FROM assignments WHERE status = 'ACTIVE'
  GROUP BY user_id
)
SELECT user_id, active_task_count
FROM active_load WHERE active_task_count >= 2;
