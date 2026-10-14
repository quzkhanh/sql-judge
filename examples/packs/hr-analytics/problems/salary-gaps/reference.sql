SELECT e.id AS employee_id,
  e.name AS employee_name,
  e.salary - LAG(e.salary) OVER (
    PARTITION BY e.department_id
    ORDER BY e.hired_on, e.id
  ) AS salary_gap
FROM employees AS e
WHERE e.salary IS NOT NULL;
