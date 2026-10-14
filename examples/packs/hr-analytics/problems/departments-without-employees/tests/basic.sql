INSERT INTO departments (id, name) VALUES
  (1, 'Engineering'),
  (2, 'Finance'),
  (3, 'Design'),
  (4, 'Legal');
INSERT INTO employees (id, name, email, department_id, manager_id, salary, hired_on) VALUES
  (1, 'Linh', 'linh@example.com', 1, NULL, 135000, '2021-03-01'),
  (2, 'Minh', 'minh@example.com', 1, NULL, 120000, '2021-03-01'),
  (3, 'Quan', 'quan@example.com', 3, NULL, 98000, '2021-03-01');
