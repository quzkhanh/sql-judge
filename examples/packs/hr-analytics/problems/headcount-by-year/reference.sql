SELECT d.name AS department_name,
  EXTRACT(YEAR FROM e.hired_on)::integer AS hire_year,
  COUNT(*) AS hires
FROM employees AS e
JOIN departments AS d ON d.id = e.department_id
GROUP BY d.id, d.name, EXTRACT(YEAR FROM e.hired_on)::integer;
