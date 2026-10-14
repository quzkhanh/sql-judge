
WITH ranked AS (
  SELECT s.*,
         ROW_NUMBER() OVER (
           PARTITION BY import_id, external_key
           ORDER BY source_row_number, id
         ) AS rn
  FROM import_staging AS s
), quality AS (
  SELECT i.id AS import_id,
         COUNT(r.id) AS total_rows,
         COUNT(r.id) FILTER (
           WHERE r.external_key IS NOT NULL AND r.rn > 1
         ) AS duplicate_rows,
         COUNT(r.id) FILTER (
           WHERE r.external_key IS NULL OR r.user_id IS NULL
              OR r.payload_status IS NULL
              OR r.payload_status NOT IN ('AVAILABLE','IN_PROGRESS','DONE')
         ) AS invalid_rows
  FROM imports AS i LEFT JOIN ranked AS r ON r.import_id = i.id
  GROUP BY i.id
)
SELECT import_id, total_rows, duplicate_rows, invalid_rows FROM quality;
