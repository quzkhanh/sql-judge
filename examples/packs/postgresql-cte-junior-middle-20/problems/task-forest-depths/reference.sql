
WITH RECURSIVE tree AS (
  SELECT id AS task_id, id AS root_task_id, 0::integer AS depth
  FROM tasks WHERE parent_task_id IS NULL
  UNION ALL
  SELECT child.id, tree.root_task_id, tree.depth + 1
  FROM tasks AS child JOIN tree ON child.parent_task_id = tree.task_id
)
SELECT task_id, root_task_id, depth FROM tree;
