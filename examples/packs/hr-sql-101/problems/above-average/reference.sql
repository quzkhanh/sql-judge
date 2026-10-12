SELECT e.id AS employee_id, e.name AS employee_name
FROM employees AS e
WHERE e.salary > (
  SELECT AVG(peer.salary)
  FROM employees AS peer
  WHERE peer.department_id = e.department_id
);
