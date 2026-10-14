SELECT d.name AS department_name,
  (
    SELECT DISTINCT e.salary
    FROM employees AS e
    WHERE e.department_id = d.id
      AND e.salary IS NOT NULL
    ORDER BY e.salary DESC
    OFFSET 1
    LIMIT 1
  ) AS second_salary
FROM departments AS d;
