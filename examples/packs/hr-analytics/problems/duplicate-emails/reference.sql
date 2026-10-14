SELECT LOWER(e.email) AS email, COUNT(*) AS employee_count
FROM employees AS e
GROUP BY LOWER(e.email)
HAVING COUNT(*) > 1;
