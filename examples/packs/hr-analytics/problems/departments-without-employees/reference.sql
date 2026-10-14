SELECT d.id AS department_id, d.name AS department_name
FROM departments AS d
LEFT JOIN employees AS e ON e.department_id = d.id
WHERE e.id IS NULL;
