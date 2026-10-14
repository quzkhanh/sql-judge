SELECT ranked.department_name, ranked.employee_name, ranked.salary
FROM (
  SELECT d.name AS department_name,
    e.name AS employee_name,
    e.salary,
    RANK() OVER (PARTITION BY e.department_id ORDER BY e.salary DESC) AS salary_rank
  FROM employees AS e
  JOIN departments AS d ON d.id = e.department_id
  WHERE e.salary IS NOT NULL
) AS ranked
WHERE ranked.salary_rank = 1;
