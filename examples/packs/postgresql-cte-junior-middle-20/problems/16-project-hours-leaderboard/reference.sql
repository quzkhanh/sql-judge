
WITH effort AS (
  SELECT t.project_id, w.user_id, SUM(COALESCE(w.hours,0::numeric)) AS logged_hours
  FROM work_logs AS w JOIN tasks AS t ON t.id = w.task_id
  GROUP BY t.project_id, w.user_id
), ranked AS (
  SELECT project_id, user_id, logged_hours,
         DENSE_RANK() OVER (
           PARTITION BY project_id ORDER BY logged_hours DESC
         ) AS activity_rank
  FROM effort
)
SELECT project_id, user_id, logged_hours, activity_rank FROM ranked;
