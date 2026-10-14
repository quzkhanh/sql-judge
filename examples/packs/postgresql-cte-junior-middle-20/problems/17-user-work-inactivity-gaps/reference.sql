
WITH activity_days AS (
  SELECT user_id, logged_on FROM work_logs GROUP BY user_id, logged_on
), sequenced AS (
  SELECT user_id, logged_on,
         LAG(logged_on) OVER (
           PARTITION BY user_id ORDER BY logged_on
         ) AS previous_logged_on
  FROM activity_days
)
SELECT user_id, logged_on, previous_logged_on,
       (logged_on - previous_logged_on) AS gap_days
FROM sequenced
WHERE (logged_on - previous_logged_on) > 7;
