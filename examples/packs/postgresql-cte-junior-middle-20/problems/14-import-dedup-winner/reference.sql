
WITH ranked AS (
  SELECT import_id, external_key, id AS staging_id,
         ROW_NUMBER() OVER (
           PARTITION BY import_id, external_key
           ORDER BY source_row_number ASC, id ASC
         ) AS rn
  FROM import_staging WHERE external_key IS NOT NULL
)
SELECT import_id, external_key, staging_id FROM ranked WHERE rn = 1;
