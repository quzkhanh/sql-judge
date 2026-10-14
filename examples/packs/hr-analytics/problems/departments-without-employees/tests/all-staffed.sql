INSERT INTO departments (id, name) VALUES
  (10, 'Research'),
  (20, 'Sales');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (101, 'An', 'an@example.com', 10, NULL, NULL, '2021-03-01'),
  (201, 'Binh', 'binh@example.com', 20, NULL, 3000, '2021-03-01'),
  (202, 'Chi', 'chi@example.com', 20, NULL, 3000, '2021-03-01');
