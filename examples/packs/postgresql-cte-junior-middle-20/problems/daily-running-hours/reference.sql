
WITH daily AS (
  SELECT logged_on, SUM(COALESCE(hours, 0::numeric)) AS daily_hours
  FROM work_logs GROUP BY logged_on
), running AS (
  SELECT logged_on, daily_hours,
         SUM(daily_hours) OVER (
           ORDER BY logged_on ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
         ) AS cumulative_hours
  FROM daily
)
SELECT logged_on, daily_hours, cumulative_hours FROM running;
