
WITH planned AS (
  SELECT project_id, SUM(COALESCE(estimated_hours,0::numeric)) AS planned_hours
  FROM tasks GROUP BY project_id
), actual AS (
  SELECT t.project_id, SUM(COALESCE(w.hours,0::numeric)) AS actual_hours
  FROM work_logs AS w JOIN tasks AS t ON t.id = w.task_id
  GROUP BY t.project_id
), totals AS (
  SELECT p.id AS project_id,
         COALESCE(pl.planned_hours,0::numeric) AS planned_hours,
         COALESCE(ac.actual_hours,0::numeric) AS actual_hours
  FROM projects AS p
  LEFT JOIN planned AS pl ON pl.project_id = p.id
  LEFT JOIN actual AS ac ON ac.project_id = p.id
)
SELECT project_id, planned_hours, actual_hours,
       actual_hours - planned_hours AS excess_hours
FROM totals WHERE actual_hours > planned_hours;
