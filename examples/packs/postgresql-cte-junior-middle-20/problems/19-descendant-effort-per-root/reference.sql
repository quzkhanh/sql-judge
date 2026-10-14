
WITH RECURSIVE tree AS (
  SELECT id AS task_id, id AS root_task_id, 0::integer AS depth
  FROM tasks WHERE parent_task_id IS NULL
  UNION ALL
  SELECT child.id, tree.root_task_id, tree.depth + 1
  FROM tasks AS child JOIN tree ON child.parent_task_id = tree.task_id
), totals AS (
  SELECT root.id AS root_task_id,
         COUNT(descendants.id) AS descendant_count,
         SUM(COALESCE(descendants.estimated_hours,0::numeric)) AS descendant_estimated_hours
  FROM tasks AS root
  LEFT JOIN tree AS t ON t.root_task_id = root.id AND t.depth > 0
  LEFT JOIN tasks AS descendants ON descendants.id = t.task_id
  WHERE root.parent_task_id IS NULL
  GROUP BY root.id
)
SELECT root_task_id, descendant_count, descendant_estimated_hours FROM totals;
