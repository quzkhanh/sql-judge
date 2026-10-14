
WITH normalized AS (
  SELECT project_id, to_char(paid_at, 'YYYY-MM') AS month,
         CASE WHEN status = 'PAID' THEN COALESCE(amount,0::numeric)
              WHEN status = 'REFUNDED' THEN -COALESCE(amount,0::numeric)
              ELSE 0::numeric END AS signed_amount
  FROM payments
), monthly AS (
  SELECT project_id, month, SUM(signed_amount) AS net_amount
  FROM normalized GROUP BY project_id, month
)
SELECT project_id, month, net_amount FROM monthly;
